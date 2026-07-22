use std::any::{TypeId, type_name};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

use crate::shape::Axis;

use super::operation::{
    CanonicalValueKind, F32_CONSTANT_BITS_ATTRIBUTE, OpKey, OperationArity,
    OperationAttributeSchema, OperationAttributes, OperationConformance, OperationDefinition,
    OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferencer,
    OperationSchema, REDUCTION_AXES_ATTRIBUTE, ValueFact, add_f32_op, constant_f32_op,
    multiply_f32_op, strict_serial_sum_f32_op,
};
use super::types::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView, QuantSchemeKey,
    ResolvedValueType, TypeIdentityError, TypeKey,
};

const MAX_DEFINITION_REFERENCE_BYTES: usize = 4 * 1024;
/// Maximum aggregate subjects in one frozen or reached semantic-authority closure.
pub const MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS: usize = 4_096;
/// Maximum roots consumed from caller-owned semantic-authority iterators.
pub const MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS: usize = 1_000_000;

/// Bounded resource counted while closing semantic authority transitively.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SemanticAuthorityResource {
    /// Aggregate roots yielded by caller-owned iterators.
    RootItems,
    /// Aggregate unique type definitions, concrete instances, and operations.
    ClosureItems,
}

impl fmt::Display for SemanticAuthorityResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootItems => formatter.write_str("semantic authority root items"),
            Self::ClosureItems => formatter.write_str("semantic authority closure items"),
        }
    }
}

/// Open local marker implemented by Rust types used for exact typed handles.
///
/// Implementing this trait grants no semantic authority. A frozen registry
/// must separately bind the marker to an admitted complete resolved type.
pub trait ValueTypeMarker: 'static {}

/// Governed Rust marker for IEEE binary32 values.
pub enum F32 {}

impl ValueTypeMarker for F32 {}

impl F32 {
    /// Returns the governed complete F32 semantic identity.
    ///
    /// # Panics
    ///
    /// Panics only if Tiler's compile-time governed key violates its own
    /// canonical identity grammar.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        ResolvedValueType::nominal(
            TypeKey::new("tiler", "f32", 1).expect("the governed F32 key is valid"),
        )
    }
}

/// Stable identity and output-affecting revision of one semantic provider.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderIdentity {
    namespace: String,
    name: String,
    revision: u32,
}

impl ProviderIdentity {
    /// Creates a validated provider identity.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when a component is invalid or revision zero
    /// is supplied.
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        revision: u32,
    ) -> Result<Self, RegistryError> {
        let namespace = namespace.into();
        let name = name.into();
        TypeKey::new(namespace.clone(), name.clone(), revision)
            .map_err(RegistryError::InvalidProviderIdentity)?;
        Ok(Self {
            namespace,
            name,
            revision,
        })
    }

    /// Returns the provider namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the name within the provider namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the nonzero output-affecting provider revision.
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    fn encode(&self, output: &mut Vec<u8>) {
        encode_bytes(output, self.namespace.as_bytes());
        encode_bytes(output, self.name.as_bytes());
        output.extend_from_slice(&self.revision.to_be_bytes());
    }
}

impl fmt::Display for ProviderIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}::{}@{}",
            self.namespace, self.name, self.revision
        )
    }
}

/// Validated identity-bearing reference to a normative semantic definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NormativeDefinitionRef(String);

impl NormativeDefinitionRef {
    /// Creates a nonempty bounded normative reference.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when the reference is empty or over the
    /// canonical byte bound.
    pub fn new(value: impl Into<String>) -> Result<Self, RegistryError> {
        let value = value.into();
        if value.is_empty() {
            return Err(RegistryError::EmptyNormativeDefinition);
        }
        if value.len() > MAX_DEFINITION_REFERENCE_BYTES {
            return Err(RegistryError::NormativeDefinitionTooLong { bytes: value.len() });
        }
        Ok(Self(value))
    }

    /// Returns the exact reference text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Bounded canonical descriptive facts owned by one type definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeDefinitionFacts(CanonicalValue);

impl TypeDefinitionFacts {
    /// Wraps an already bounded canonical value in its definition-fact role.
    #[must_use]
    pub const fn new(value: CanonicalValue) -> Self {
        Self(value)
    }

    /// Returns the canonical value.
    #[must_use]
    pub const fn value(&self) -> &CanonicalValue {
        &self.0
    }
}

/// Which family of resolved types one semantic definition governs.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueTypeDefinitionKey {
    /// One exact nominal type.
    Nominal(TypeKey),
    /// Every admitted instance of one parameterized constructor.
    Parameterized(TypeKey),
    /// Every admitted static contract for one encoded-numeric scheme.
    EncodedNumeric(QuantSchemeKey),
}

impl ValueTypeDefinitionKey {
    fn for_value(value: &ResolvedValueType) -> Self {
        if let Some(key) = value.nominal_key() {
            return Self::Nominal(key.clone());
        }
        if let Some((constructor, _)) = value.parameterized_parts() {
            return Self::Parameterized(constructor.clone());
        }
        let (scheme, _) = value
            .encoded_numeric_parts()
            .expect("resolved value type has one governed variant");
        Self::EncodedNumeric(scheme.clone())
    }

    fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::Nominal(key) => {
                output.push(1);
                encode_type_key(output, key);
            }
            Self::Parameterized(key) => {
                output.push(2);
                encode_type_key(output, key);
            }
            Self::EncodedNumeric(key) => {
                output.push(3);
                encode_bytes(output, key.namespace().as_bytes());
                encode_bytes(output, key.name().as_bytes());
                output.extend_from_slice(&key.semantic_version().to_be_bytes());
            }
        }
    }
}

/// Typed rejection produced by a semantic type-family validator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeInstanceError {
    code: String,
    message: String,
}

impl TypeInstanceError {
    /// Creates a provider-attributed, stable-code instance rejection.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns provider diagnostic detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TypeInstanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for TypeInstanceError {}

/// Additional immutable validation for instances of one registered type family.
///
/// Structural bounds, family-key matching, and referenced-type admission are
/// always checked by the host before this validator runs.
pub trait ValueTypeInstanceValidator: Send + Sync + 'static {
    /// Validates additional semantic predicates for one bounded instance.
    ///
    /// # Errors
    ///
    /// Returns a stable provider diagnostic when the instance is not admitted.
    fn validate(&self, value: &ResolvedValueType) -> Result<(), TypeInstanceError>;
}

#[derive(Debug)]
struct AcceptStructurallyValid;

impl ValueTypeInstanceValidator for AcceptStructurallyValid {
    fn validate(&self, _: &ResolvedValueType) -> Result<(), TypeInstanceError> {
        Ok(())
    }
}

/// Portable semantic definition of one nominal type, constructor, or scheme.
#[derive(Clone)]
pub struct ValueTypeDefinition {
    key: ValueTypeDefinitionKey,
    normative_definition: NormativeDefinitionRef,
    canonical_facts: TypeDefinitionFacts,
    validator: Arc<dyn ValueTypeInstanceValidator>,
}

impl fmt::Debug for ValueTypeDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValueTypeDefinition")
            .field("key", &self.key)
            .field("normative_definition", &self.normative_definition)
            .field("canonical_facts", &self.canonical_facts)
            .field("validator", &"ValueTypeInstanceValidator(..)")
            .finish()
    }
}

impl ValueTypeDefinition {
    /// Creates a definition with additional immutable instance validation.
    #[must_use]
    pub fn new(
        key: ValueTypeDefinitionKey,
        normative_definition: NormativeDefinitionRef,
        canonical_facts: TypeDefinitionFacts,
        validator: Arc<dyn ValueTypeInstanceValidator>,
    ) -> Self {
        Self {
            key,
            normative_definition,
            canonical_facts,
            validator,
        }
    }

    /// Creates a definition whose family-key, structure, and references are
    /// sufficient for admission.
    #[must_use]
    pub fn structurally_valid(
        key: ValueTypeDefinitionKey,
        normative_definition: NormativeDefinitionRef,
        canonical_facts: TypeDefinitionFacts,
    ) -> Self {
        Self::new(
            key,
            normative_definition,
            canonical_facts,
            Arc::new(AcceptStructurallyValid),
        )
    }

    /// Returns the governed family key.
    #[must_use]
    pub const fn key(&self) -> &ValueTypeDefinitionKey {
        &self.key
    }

    /// Returns the normative definition reference.
    #[must_use]
    pub const fn normative_definition(&self) -> &NormativeDefinitionRef {
        &self.normative_definition
    }

    /// Returns bounded canonical semantic facts.
    #[must_use]
    pub const fn canonical_facts(&self) -> &TypeDefinitionFacts {
        &self.canonical_facts
    }
}

#[derive(Clone)]
struct RegisteredValueType {
    definition: ValueTypeDefinition,
    provider: ProviderIdentity,
}

#[derive(Clone)]
struct RegisteredOperation {
    definition: OperationDefinition,
    provider: ProviderIdentity,
}

/// A statically linked source of semantic definitions and optional marker bindings.
///
/// The provider callback runs only while a registration batch is staged. It is
/// not retained by the frozen registry.
pub trait SemanticRegistryProvider: Send + Sync + 'static {
    /// Returns stable provider identity and output-affecting revision.
    fn identity(&self) -> ProviderIdentity;

    /// Stages semantic definitions and local marker bindings.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] without mutating the destination builder.
    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError>;
}

/// Mutable, single-use constructor for a frozen semantic registry.
#[derive(Default)]
pub struct SemanticRegistryBuilder {
    definitions: BTreeMap<ValueTypeDefinitionKey, RegisteredValueType>,
    operations: BTreeMap<OpKey, RegisteredOperation>,
    marker_bindings: HashMap<TypeId, MarkerBinding>,
}

impl fmt::Debug for SemanticRegistryBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SemanticRegistryBuilder")
            .field("definition_count", &self.definitions.len())
            .field("operation_count", &self.operations.len())
            .field("marker_count", &self.marker_bindings.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
struct MarkerBinding {
    marker_name: &'static str,
    resolved_type: ResolvedValueType,
}

#[derive(Default)]
struct RegistrationBatch {
    definitions: BTreeMap<ValueTypeDefinitionKey, ValueTypeDefinition>,
    operations: BTreeMap<OpKey, OperationDefinition>,
    marker_bindings: HashMap<TypeId, MarkerBinding>,
}

impl SemanticRegistryBuilder {
    /// Creates an empty registry builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the mutable governed standard profile.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] if a governed built-in violates the public
    /// provider contract.
    pub fn standard() -> Result<Self, RegistryError> {
        let mut builder = Self::new();
        builder.register_provider(&StandardSemantics)?;
        Ok(builder)
    }

    /// Applies one provider transactionally through an isolated staging batch.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] without changing this builder when the
    /// provider fails, registers nothing, or collides with existing authority.
    pub fn register_provider(
        &mut self,
        provider: &(dyn SemanticRegistryProvider + 'static),
    ) -> Result<(), RegistryError> {
        let identity = provider.identity();
        let mut batch = RegistrationBatch::default();
        provider.register(&mut SemanticRegistryRegistrar { batch: &mut batch })?;
        if batch.definitions.is_empty()
            && batch.operations.is_empty()
            && batch.marker_bindings.is_empty()
        {
            return Err(RegistryError::ProviderRegisteredNothing { provider: identity });
        }
        for key in batch.definitions.keys() {
            if self.definitions.contains_key(key) {
                return Err(RegistryError::DuplicateTypeAuthority {
                    key: Arc::new(key.clone()),
                });
            }
        }
        for key in batch.operations.keys() {
            if self.operations.contains_key(key) {
                return Err(RegistryError::DuplicateOperationAuthority {
                    key: Arc::new(key.clone()),
                });
            }
        }
        for (marker, binding) in &batch.marker_bindings {
            if let Some(existing) = self.marker_bindings.get(marker) {
                return Err(RegistryError::DuplicateMarker {
                    marker_name: existing.marker_name,
                });
            }
            if self
                .marker_bindings
                .values()
                .any(|existing| existing.resolved_type == binding.resolved_type)
                || batch.marker_bindings.iter().any(|(other, existing)| {
                    other != marker && existing.resolved_type == binding.resolved_type
                })
            {
                return Err(RegistryError::DuplicateResolvedTypeMarker {
                    resolved_type: Arc::new(binding.resolved_type.clone()),
                });
            }
        }
        self.definitions
            .extend(batch.definitions.into_iter().map(|(key, definition)| {
                (
                    key,
                    RegisteredValueType {
                        definition,
                        provider: identity.clone(),
                    },
                )
            }));
        self.operations
            .extend(batch.operations.into_iter().map(|(key, definition)| {
                (
                    key,
                    RegisteredOperation {
                        definition,
                        provider: identity.clone(),
                    },
                )
            }));
        self.marker_bindings.extend(batch.marker_bindings);
        Ok(())
    }

    /// Freezes this registry into canonical immutable shared state.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when the registry is empty; when any marker,
    /// type definition, operation schema default, operation fact, or
    /// conformance value transitively references missing or rejected type
    /// authority; or when root ingestion or the unique authority closure
    /// exceeds its governed resource bound. Finite definition cycles are
    /// admitted and traversed without recursion.
    pub fn freeze(self) -> Result<FrozenSemanticRegistry, RegistryError> {
        if self.definitions.is_empty() && self.operations.is_empty() {
            return Err(RegistryError::EmptyRegistry);
        }
        let registry = FrozenSemanticRegistry(Arc::new(FrozenRegistryData {
            identity: OnceLock::new(),
            definitions: self.definitions,
            operations: self.operations,
            marker_bindings: self.marker_bindings,
        }));
        registry.close_authority(
            registry
                .0
                .marker_bindings
                .values()
                .map(|binding| &binding.resolved_type),
            registry.0.definitions.keys(),
            registry.0.operations.keys(),
            std::iter::empty(),
        )?;
        let _ = registry.snapshot_identity();
        Ok(registry)
    }
}

/// Host-owned registration surface supplied to one semantic provider.
pub struct SemanticRegistryRegistrar<'a> {
    batch: &'a mut RegistrationBatch,
}

impl SemanticRegistryRegistrar<'_> {
    /// Registers one semantic nominal, constructor, or scheme definition.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for duplicate authority within this provider.
    pub fn register_value_type(
        &mut self,
        definition: ValueTypeDefinition,
    ) -> Result<(), RegistryError> {
        let key = definition.key().clone();
        if self
            .batch
            .definitions
            .insert(key.clone(), definition)
            .is_some()
        {
            return Err(RegistryError::DuplicateTypeAuthority { key: Arc::new(key) });
        }
        Ok(())
    }

    /// Registers one atomic semantic operation family.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for duplicate authority within this provider.
    pub fn register_operation(
        &mut self,
        definition: OperationDefinition,
    ) -> Result<(), RegistryError> {
        let key = definition.key().clone();
        if self
            .batch
            .operations
            .insert(key.clone(), definition)
            .is_some()
        {
            return Err(RegistryError::DuplicateOperationAuthority { key: Arc::new(key) });
        }
        Ok(())
    }

    /// Binds one local Rust marker to a complete resolved semantic type.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for duplicate marker or resolved-type binding
    /// within this provider batch.
    pub fn bind_marker<T: ValueTypeMarker>(
        &mut self,
        resolved_type: ResolvedValueType,
    ) -> Result<(), RegistryError> {
        let marker = TypeId::of::<T>();
        if self.batch.marker_bindings.contains_key(&marker) {
            return Err(RegistryError::DuplicateMarker {
                marker_name: type_name::<T>(),
            });
        }
        if self
            .batch
            .marker_bindings
            .values()
            .any(|binding| binding.resolved_type == resolved_type)
        {
            return Err(RegistryError::DuplicateResolvedTypeMarker {
                resolved_type: Arc::new(resolved_type),
            });
        }
        self.batch.marker_bindings.insert(
            marker,
            MarkerBinding {
                marker_name: type_name::<T>(),
                resolved_type,
            },
        );
        Ok(())
    }

    /// Registers a definition and marker binding through the two independent
    /// primitives as one provider convenience.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for either registration failure.
    pub fn register_marked_value_type<T: ValueTypeMarker>(
        &mut self,
        definition: ValueTypeDefinition,
        resolved_type: ResolvedValueType,
    ) -> Result<(), RegistryError> {
        self.register_value_type(definition)?;
        self.bind_marker::<T>(resolved_type)
    }
}

/// Immutable, cheap-clone semantic authority used by builders and programs.
#[derive(Clone)]
pub struct FrozenSemanticRegistry(Arc<FrozenRegistryData>);

impl fmt::Debug for FrozenSemanticRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenSemanticRegistry")
            .field("definition_count", &self.0.definitions.len())
            .field("operation_count", &self.0.operations.len())
            .field("marker_count", &self.0.marker_bindings.len())
            .finish()
    }
}

struct FrozenRegistryData {
    definitions: BTreeMap<ValueTypeDefinitionKey, RegisteredValueType>,
    operations: BTreeMap<OpKey, RegisteredOperation>,
    marker_bindings: HashMap<TypeId, MarkerBinding>,
    identity: OnceLock<SemanticRegistrySnapshotIdentity>,
}

#[derive(Default)]
struct SemanticAuthorityClosure {
    type_keys: BTreeSet<ValueTypeDefinitionKey>,
    type_instances: BTreeSet<ResolvedValueType>,
    operation_keys: BTreeSet<OpKey>,
}

impl FrozenSemanticRegistry {
    fn close_authority<'a>(
        &self,
        concrete_type_roots: impl IntoIterator<Item = &'a ResolvedValueType>,
        definition_roots: impl IntoIterator<Item = &'a ValueTypeDefinitionKey>,
        operation_roots: impl IntoIterator<Item = &'a OpKey>,
        canonical_roots: impl IntoIterator<Item = &'a CanonicalValue>,
    ) -> Result<SemanticAuthorityClosure, RegistryError> {
        let mut closure = SemanticAuthorityClosure::default();
        let mut pending_instances = BTreeSet::new();
        let mut pending_definitions = BTreeSet::new();
        let mut pending_operations = BTreeSet::new();
        let mut observed_roots = 0_usize;
        for value in concrete_type_roots {
            observe_authority_root(&mut observed_roots)?;
            enqueue_type_instance(value.clone(), &mut closure, &mut pending_instances)?;
        }
        for key in definition_roots {
            observe_authority_root(&mut observed_roots)?;
            enqueue_type_definition(key.clone(), &mut closure, &mut pending_definitions)?;
        }
        for key in operation_roots {
            observe_authority_root(&mut observed_roots)?;
            enqueue_operation(key.clone(), &mut closure, &mut pending_operations)?;
        }
        for value in canonical_roots {
            observe_authority_root(&mut observed_roots)?;
            enqueue_canonical_types(value, &mut closure, &mut pending_instances)?;
        }

        while !(pending_instances.is_empty()
            && pending_definitions.is_empty()
            && pending_operations.is_empty())
        {
            if let Some(value) = pending_instances.pop_first() {
                let key = ValueTypeDefinitionKey::for_value(&value);
                let registered = self.0.definitions.get(&key).ok_or_else(|| {
                    RegistryError::UnregisteredTypeAuthority {
                        key: Arc::new(key.clone()),
                    }
                })?;
                enqueue_type_definition(key.clone(), &mut closure, &mut pending_definitions)?;
                enqueue_referenced_types(&value, &mut closure, &mut pending_instances)?;
                registered
                    .definition
                    .validator
                    .validate(&value)
                    .map_err(|source| {
                        RegistryError::RejectedTypeInstance(Arc::new(TypeInstanceRejection {
                            key,
                            provider: registered.provider.clone(),
                            source,
                        }))
                    })?;
                continue;
            }

            if let Some(key) = pending_definitions.pop_first() {
                let registered = self.0.definitions.get(&key).ok_or_else(|| {
                    RegistryError::UnregisteredTypeAuthority {
                        key: Arc::new(key.clone()),
                    }
                })?;
                enqueue_canonical_types(
                    registered.definition.canonical_facts().value(),
                    &mut closure,
                    &mut pending_instances,
                )?;
                continue;
            }

            let key = pending_operations
                .pop_first()
                .expect("nonempty closure worklist has an operation");
            let registered = self.0.operations.get(&key).ok_or_else(|| {
                RegistryError::UnregisteredOperationAuthority {
                    key: Arc::new(key.clone()),
                }
            })?;
            for field in registered.definition.schema().attributes() {
                if let Some(default) = field.default() {
                    enqueue_canonical_types(default, &mut closure, &mut pending_instances)?;
                }
            }
            enqueue_canonical_types(
                registered.definition.canonical_facts().value(),
                &mut closure,
                &mut pending_instances,
            )?;
            enqueue_canonical_types(
                registered.definition.conformance().value(),
                &mut closure,
                &mut pending_instances,
            )?;
        }
        Ok(closure)
    }

    fn validate_canonical_value_types(&self, value: &CanonicalValue) -> Result<(), RegistryError> {
        self.close_authority(
            std::iter::empty(),
            std::iter::empty(),
            std::iter::empty(),
            [value],
        )
        .map(|_| ())
    }

    /// Builds the governed standard registry profile.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] if the governed definition violates the same
    /// public contract used by extensions.
    pub fn standard() -> Result<Self, RegistryError> {
        static STANDARD: OnceLock<Result<FrozenSemanticRegistry, RegistryError>> = OnceLock::new();
        STANDARD
            .get_or_init(|| SemanticRegistryBuilder::standard()?.freeze())
            .clone()
    }

    /// Resolves one local marker through this exact frozen snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryLookupError::UnregisteredMarker`] when the marker was
    /// not explicitly bound.
    pub fn resolve_marker<T: ValueTypeMarker>(
        &self,
    ) -> Result<&ResolvedValueType, RegistryLookupError> {
        self.0
            .marker_bindings
            .get(&TypeId::of::<T>())
            .map(|binding| &binding.resolved_type)
            .ok_or(RegistryLookupError::UnregisteredMarker {
                marker_name: type_name::<T>(),
            })
    }

    /// Validates one complete resolved type against registered family authority.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for missing authority, an unregistered nested
    /// component, or provider semantic rejection.
    pub fn validate_type(&self, value: &ResolvedValueType) -> Result<(), RegistryError> {
        self.close_authority(
            [value],
            std::iter::empty(),
            std::iter::empty(),
            std::iter::empty(),
        )
        .map(|_| ())
    }

    /// Returns whether this snapshot admits a complete resolved value type.
    #[must_use]
    pub fn contains(&self, resolved_type: &ResolvedValueType) -> bool {
        self.validate_type(resolved_type).is_ok()
    }

    /// Returns the governing family definition when registered.
    #[must_use]
    pub fn definition(&self, resolved_type: &ResolvedValueType) -> Option<&ValueTypeDefinition> {
        self.0
            .definitions
            .get(&ValueTypeDefinitionKey::for_value(resolved_type))
            .map(|registered| &registered.definition)
    }

    /// Returns the provider governing one resolved type family.
    #[must_use]
    pub fn provider(&self, resolved_type: &ResolvedValueType) -> Option<&ProviderIdentity> {
        self.0
            .definitions
            .get(&ValueTypeDefinitionKey::for_value(resolved_type))
            .map(|registered| &registered.provider)
    }

    /// Returns the registered semantic definition for one operation family.
    #[must_use]
    pub fn operation_definition(&self, key: &OpKey) -> Option<&OperationDefinition> {
        self.0.operations.get(key).map(|entry| &entry.definition)
    }

    /// Returns the provider governing one operation family.
    #[must_use]
    pub fn operation_provider(&self, key: &OpKey) -> Option<&ProviderIdentity> {
        self.0.operations.get(key).map(|entry| &entry.provider)
    }

    /// Validates one application and derives all ordered result facts.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for missing authority, invalid operand/result
    /// types, or semantic inference rejection.
    pub fn infer_operation(
        &self,
        key: &OpKey,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueFact>, RegistryError> {
        let registered = self.0.operations.get(key).ok_or_else(|| {
            RegistryError::UnregisteredOperationAuthority {
                key: Arc::new(key.clone()),
            }
        })?;
        for operand in operands {
            self.validate_type(operand.resolved_type())?;
        }
        for field in attributes.fields() {
            self.validate_canonical_value_types(field.value())?;
        }
        let results = registered
            .definition
            .infer(operands, attributes)
            .map_err(|source| {
                RegistryError::RejectedOperationApplication(Arc::new(
                    OperationApplicationRejection {
                        key: key.clone(),
                        provider: registered.provider.clone(),
                        source,
                    },
                ))
            })?;
        if results.is_empty() {
            return Err(RegistryError::OperationProducedNoResults {
                key: Arc::new(key.clone()),
            });
        }
        for result in &results {
            self.validate_type(result.resolved_type())?;
        }
        Ok(results)
    }

    /// Validates and canonicalizes one operation attribute record under its registered schema.
    ///
    /// Explicit values equal to schema defaults normalize to omission.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for missing authority or invalid attributes.
    pub fn normalize_operation_attributes(
        &self,
        key: &OpKey,
        attributes: OperationAttributes,
    ) -> Result<OperationAttributes, RegistryError> {
        let registered = self.0.operations.get(key).ok_or_else(|| {
            RegistryError::UnregisteredOperationAuthority {
                key: Arc::new(key.clone()),
            }
        })?;
        let canonical = registered
            .definition
            .schema()
            .normalize_attributes(&attributes)
            .map_err(|source| {
                RegistryError::RejectedOperationApplication(Arc::new(
                    OperationApplicationRejection {
                        key: key.clone(),
                        provider: registered.provider.clone(),
                        source,
                    },
                ))
            })?;
        for field in canonical.fields() {
            self.validate_canonical_value_types(field.value())?;
        }
        drop(attributes);
        Ok(canonical)
    }

    /// Returns complete frozen semantic-registry snapshot provenance.
    #[must_use]
    pub fn snapshot_identity(&self) -> &SemanticRegistrySnapshotIdentity {
        self.0
            .identity
            .get_or_init(|| compute_identity(&self.0.definitions, &self.0.operations))
    }

    #[cfg(test)]
    fn project_caller_rooted_definitions<'a>(
        &self,
        value_types: impl IntoIterator<Item = &'a ResolvedValueType>,
        operations: impl IntoIterator<Item = &'a OpKey>,
    ) -> Result<SemanticDefinitionProjectionIdentity, RegistryError> {
        let closure = self.close_authority(
            value_types,
            std::iter::empty(),
            operations,
            std::iter::empty(),
        )?;
        Ok(self.encode_definition_projection(&closure))
    }

    pub(super) fn project_program_authority<'a>(
        &self,
        value_types: impl IntoIterator<Item = &'a ResolvedValueType>,
        operations: impl IntoIterator<Item = &'a OpKey>,
        occurrence_attributes: impl IntoIterator<Item = &'a CanonicalValue>,
    ) -> Result<
        (
            SemanticDefinitionProjectionIdentity,
            SemanticAdmissionProvenanceIdentity,
        ),
        RegistryError,
    > {
        let closure = self.close_authority(
            value_types,
            std::iter::empty(),
            operations,
            occurrence_attributes,
        )?;
        Ok((
            self.encode_definition_projection(&closure),
            self.encode_admission_provenance(&closure),
        ))
    }

    fn encode_definition_projection(
        &self,
        closure: &SemanticAuthorityClosure,
    ) -> SemanticDefinitionProjectionIdentity {
        let mut bytes = b"tiler.semantic-definition-projection.v3\0".to_vec();
        encode_len(&mut bytes, closure.type_keys.len());
        for key in &closure.type_keys {
            let registered = self
                .0
                .definitions
                .get(key)
                .expect("a frozen authority closure contains only registered types");
            encode_type_definition(&mut bytes, key, &registered.definition);
        }
        encode_len(&mut bytes, closure.operation_keys.len());
        for key in &closure.operation_keys {
            let registered = self
                .0
                .operations
                .get(key)
                .expect("a frozen authority closure contains only registered operations");
            encode_operation_definition(&mut bytes, key, &registered.definition);
        }
        SemanticDefinitionProjectionIdentity(bytes)
    }

    fn encode_admission_provenance(
        &self,
        closure: &SemanticAuthorityClosure,
    ) -> SemanticAdmissionProvenanceIdentity {
        let mut bytes = b"tiler.semantic-admission-provenance.v1\0".to_vec();
        encode_len(&mut bytes, closure.type_keys.len());
        for key in &closure.type_keys {
            let registered = self
                .0
                .definitions
                .get(key)
                .expect("a frozen authority closure contains only registered types");
            key.encode(&mut bytes);
            registered.provider.encode(&mut bytes);
        }
        encode_len(&mut bytes, closure.operation_keys.len());
        for key in &closure.operation_keys {
            let registered = self
                .0
                .operations
                .get(key)
                .expect("a frozen authority closure contains only registered operations");
            key.encode(&mut bytes);
            registered.provider.encode(&mut bytes);
        }
        SemanticAdmissionProvenanceIdentity(bytes)
    }
}

/// Collision-free canonical provenance for a complete frozen registry snapshot.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticRegistrySnapshotIdentity(Vec<u8>);

impl SemanticRegistrySnapshotIdentity {
    /// Returns the canonical provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Provider-independent canonical semantic definitions reached by a program.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticDefinitionProjectionIdentity(Vec<u8>);

impl SemanticDefinitionProjectionIdentity {
    /// Returns the collision-free canonical projection bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Provider-attributed admission provenance reached by one semantic program.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticAdmissionProvenanceIdentity(Vec<u8>);

impl SemanticAdmissionProvenanceIdentity {
    /// Returns the collision-free canonical provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Complete provider-attributed rejection of one concrete type instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeInstanceRejection {
    key: ValueTypeDefinitionKey,
    provider: ProviderIdentity,
    source: TypeInstanceError,
}

/// Complete provider-attributed rejection of one operation application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationApplicationRejection {
    key: OpKey,
    provider: ProviderIdentity,
    source: OperationInferenceError,
}

impl OperationApplicationRejection {
    /// Returns the rejected operation family.
    #[must_use]
    pub const fn key(&self) -> &OpKey {
        &self.key
    }

    /// Returns the governing provider.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the provider-attributed inference error.
    #[must_use]
    pub const fn source_error(&self) -> &OperationInferenceError {
        &self.source
    }
}

impl TypeInstanceRejection {
    /// Returns the governing family key.
    #[must_use]
    pub const fn key(&self) -> &ValueTypeDefinitionKey {
        &self.key
    }

    /// Returns the governing provider.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the provider-attributed instance error.
    #[must_use]
    pub const fn source_error(&self) -> &TypeInstanceError {
        &self.source
    }
}

/// Failure to build, freeze, or validate semantic registry authority.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistryError {
    /// Provider identity did not satisfy canonical identity rules.
    InvalidProviderIdentity(TypeIdentityError),
    /// No semantic definition was registered.
    EmptyRegistry,
    /// A provider transaction contained no semantic contribution.
    ProviderRegisteredNothing {
        /// Provider which registered nothing.
        provider: ProviderIdentity,
    },
    /// Two providers or registrations claimed one semantic family key.
    DuplicateTypeAuthority {
        /// Duplicated family key.
        key: Arc<ValueTypeDefinitionKey>,
    },
    /// Two providers or registrations claimed one operation key.
    DuplicateOperationAuthority {
        /// Duplicated operation key.
        key: Arc<OpKey>,
    },
    /// One Rust marker was bound more than once.
    DuplicateMarker {
        /// Diagnostic-only Rust marker name.
        marker_name: &'static str,
    },
    /// Two markers attempted to represent one complete resolved type.
    DuplicateResolvedTypeMarker {
        /// Duplicated complete type.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// A resolved type had no registered nominal, constructor, or scheme authority.
    UnregisteredTypeAuthority {
        /// Missing family key.
        key: Arc<ValueTypeDefinitionKey>,
    },
    /// A registered family rejected one concrete instance.
    RejectedTypeInstance(Arc<TypeInstanceRejection>),
    /// An operation family had no registered semantic authority.
    UnregisteredOperationAuthority {
        /// Missing operation key.
        key: Arc<OpKey>,
    },
    /// A registered operation rejected one application.
    RejectedOperationApplication(Arc<OperationApplicationRejection>),
    /// An operation authority inferred no results.
    OperationProducedNoResults {
        /// Invalid operation authority.
        key: Arc<OpKey>,
    },
    /// A transitive semantic-authority closure exceeded its governed bound.
    SemanticAuthorityResourceExceeded {
        /// Governed aggregate resource.
        resource: SemanticAuthorityResource,
        /// Maximum admitted unique subjects.
        limit: usize,
        /// First rejected aggregate count.
        actual: usize,
    },
    /// A normative definition reference was empty.
    EmptyNormativeDefinition,
    /// A normative definition reference exceeded its byte bound.
    NormativeDefinitionTooLong {
        /// Actual UTF-8 bytes.
        bytes: usize,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProviderIdentity(source) => {
                write!(formatter, "invalid provider identity: {source}")
            }
            Self::EmptyRegistry => formatter.write_str("semantic registry is empty"),
            Self::ProviderRegisteredNothing { provider } => {
                write!(formatter, "semantic provider {provider} registered nothing")
            }
            Self::DuplicateTypeAuthority { key } => {
                write!(formatter, "duplicate type authority for {key:?}")
            }
            Self::DuplicateOperationAuthority { key } => {
                write!(formatter, "duplicate operation authority for {key}")
            }
            Self::DuplicateMarker { marker_name } => {
                write!(formatter, "Rust marker {marker_name} is already bound")
            }
            Self::DuplicateResolvedTypeMarker { resolved_type } => write!(
                formatter,
                "resolved type {:?} already has a Rust marker",
                resolved_type.canonical_encoding().as_bytes()
            ),
            Self::UnregisteredTypeAuthority { key } => {
                write!(formatter, "no semantic authority for {key:?}")
            }
            Self::RejectedTypeInstance(rejection) => {
                write!(
                    formatter,
                    "provider {} rejected {:?}: {}",
                    rejection.provider, rejection.key, rejection.source
                )
            }
            Self::UnregisteredOperationAuthority { key } => {
                write!(formatter, "no semantic authority for operation {key}")
            }
            Self::RejectedOperationApplication(rejection) => write!(
                formatter,
                "provider {} rejected operation {}: {}",
                rejection.provider, rejection.key, rejection.source
            ),
            Self::OperationProducedNoResults { key } => {
                write!(formatter, "operation authority {key} produced no results")
            }
            Self::SemanticAuthorityResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "{resource} count {actual} exceeds governed limit {limit}"
            ),
            Self::EmptyNormativeDefinition => {
                formatter.write_str("normative definition reference is empty")
            }
            Self::NormativeDefinitionTooLong { bytes } => write!(
                formatter,
                "normative definition reference has {bytes} bytes, exceeding {MAX_DEFINITION_REFERENCE_BYTES}"
            ),
        }
    }
}

impl Error for RegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidProviderIdentity(source) => Some(source),
            Self::RejectedTypeInstance(rejection) => Some(&rejection.source),
            Self::RejectedOperationApplication(rejection) => Some(&rejection.source),
            _ => None,
        }
    }
}

/// Failure to resolve a local Rust marker.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistryLookupError {
    /// Marker was implemented but never explicitly bound.
    UnregisteredMarker {
        /// Diagnostic-only Rust type name.
        marker_name: &'static str,
    },
}

impl fmt::Display for RegistryLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnregisteredMarker { marker_name } => {
                write!(formatter, "Rust marker {marker_name} is not registered")
            }
        }
    }
}

impl Error for RegistryLookupError {}

struct StandardSemantics;

impl SemanticRegistryProvider for StandardSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler", "standard-semantics", 4)
            .expect("the governed standard provider identity is valid")
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let facts = TypeDefinitionFacts::new(
            CanonicalValue::record([
                CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::utf8("ieee-binary").expect("the governed F32 class is bounded"),
                ),
                CanonicalField::new(AttributeFieldId::new(2), CanonicalValue::unsigned_u32(32)),
            ])
            .expect("the governed F32 facts are canonical"),
        );
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(
                    TypeKey::new("tiler", "f32", 1).expect("the governed F32 key is valid"),
                ),
                NormativeDefinitionRef::new("IEEE 754-2019 binary32; tiler::f32@1")?,
                facts,
            ),
            F32::resolved_type(),
        )?;
        registrar.register_operation(OperationDefinition::new(
            constant_f32_op(),
            exact_schema(
                0,
                1,
                [OperationAttributeSchema::required(
                    F32_CONSTANT_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                )],
            ),
            NormativeDefinitionRef::new("tiler::constant-f32@1; exact IEEE-754 payload")?,
            OperationDefinitionFacts::new(
                CanonicalValue::record([CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::utf8("exact-binary32-bits")
                        .expect("the governed constant fact is bounded"),
                )])
                .expect("the governed constant facts are canonical"),
            ),
            standard_conformance("constant-f32"),
            OperationEffect::Pure,
            Arc::new(ConstantF32),
        ))?;
        registrar.register_operation(OperationDefinition::new(
            multiply_f32_op(),
            exact_schema(2, 1, []),
            NormativeDefinitionRef::new("tiler::multiply-f32@1; separate binary32 multiply")?,
            OperationDefinitionFacts::new(arithmetic_f32_facts()),
            standard_conformance("multiply-f32"),
            OperationEffect::Pure,
            Arc::new(BinaryF32),
        ))?;
        registrar.register_operation(OperationDefinition::new(
            add_f32_op(),
            exact_schema(2, 1, []),
            NormativeDefinitionRef::new("tiler::add-f32@1; separate binary32 addition")?,
            OperationDefinitionFacts::new(arithmetic_f32_facts()),
            standard_conformance("add-f32"),
            OperationEffect::Pure,
            Arc::new(BinaryF32),
        ))?;
        registrar.register_operation(OperationDefinition::new(
            strict_serial_sum_f32_op(),
            exact_schema(
                1,
                1,
                [OperationAttributeSchema::required(
                    REDUCTION_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                )],
            ),
            NormativeDefinitionRef::new(
                "tiler::strict-serial-sum-f32@1; lexicographic serial contributors",
            )?,
            OperationDefinitionFacts::new(
                CanonicalValue::record([
                    CanonicalField::new(
                        AttributeFieldId::new(1),
                        CanonicalValue::utf8("strict-left-fold")
                            .expect("the governed reduction fact is bounded"),
                    ),
                    CanonicalField::new(
                        AttributeFieldId::new(2),
                        CanonicalValue::utf8("binary32-each-step")
                            .expect("the governed accumulation fact is bounded"),
                    ),
                    CanonicalField::new(
                        AttributeFieldId::new(3),
                        canonical_f32_bits(super::operation::CANONICAL_F32_ARITHMETIC_NAN_BITS),
                    ),
                ])
                .expect("the governed reduction facts are canonical"),
            ),
            standard_conformance("strict-serial-sum-f32"),
            OperationEffect::Pure,
            Arc::new(StrictSerialSumF32),
        ))
    }
}

fn exact_schema<const N: usize>(
    operands: u32,
    results: u32,
    attributes: [OperationAttributeSchema; N],
) -> OperationSchema {
    OperationSchema::new(
        OperationArity::exact(operands),
        OperationArity::exact(results),
        attributes,
    )
    .expect("governed operation schema is valid")
}

fn standard_conformance(name: &str) -> OperationConformance {
    OperationConformance::new(
        CanonicalValue::record([
            CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::utf8(format!("tiler.conformance.{name}"))
                    .expect("governed conformance identity is bounded"),
            ),
            CanonicalField::new(AttributeFieldId::new(2), CanonicalValue::unsigned_u32(1)),
        ])
        .expect("governed conformance identity is canonical"),
    )
}

fn arithmetic_f32_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            AttributeFieldId::new(1),
            CanonicalValue::utf8("binary32-round-to-nearest-ties-even")
                .expect("the governed rounding fact is bounded"),
        ),
        CanonicalField::new(
            AttributeFieldId::new(2),
            canonical_f32_bits(super::operation::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
        CanonicalField::new(AttributeFieldId::new(3), CanonicalValue::boolean(false)),
    ])
    .expect("the governed f32 arithmetic facts are canonical")
}

struct ConstantF32;

fn canonical_f32_bits(bits: u32) -> CanonicalValue {
    CanonicalValue::float_bits(
        TypeKey::new("tiler", "f32", 1).expect("the governed F32 key is valid"),
        bits.to_be_bytes(),
    )
    .expect("binary32 has a nonempty bounded payload")
}

impl OperationInferencer for ConstantF32 {
    fn infer(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        if !operands.is_empty() {
            return Err(op_error("constant.arity", "constant requires no operands"));
        }
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "constant.attributes",
                "constant requires exactly the bits attribute",
            ));
        }
        let Some(CanonicalValueView::FloatBits(bits)) = attributes
            .get(F32_CONSTANT_BITS_ATTRIBUTE)
            .map(CanonicalValue::view)
        else {
            return Err(op_error(
                "constant.bits",
                "constant bits must be exact binary32 FloatBits",
            ));
        };
        if bits.format() != &TypeKey::new("tiler", "f32", 1).expect("the governed F32 key is valid")
            || bits.bits().len() != 4
        {
            return Err(op_error(
                "constant.bits",
                "constant bits must use the binary32 format and width",
            ));
        }
        Ok(vec![ValueFact::new(
            F32::resolved_type(),
            crate::shape::Shape::new([]),
        )])
    }
}

struct BinaryF32;

impl OperationInferencer for BinaryF32 {
    fn infer(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        if operands.len() != 2 {
            return Err(op_error(
                "binary.arity",
                "binary operation requires two operands",
            ));
        }
        if !attributes.fields().is_empty() {
            return Err(op_error(
                "binary.attributes",
                "binary operation has no attributes",
            ));
        }
        if operands
            .iter()
            .any(|operand| operand.resolved_type() != &F32::resolved_type())
        {
            return Err(op_error("binary.type", "both operands must be f32"));
        }
        let left = operands[0].shape();
        let right = operands[1].shape();
        let shape = if left.rank() == 0 {
            right.clone()
        } else if right.rank() == 0 || left == right {
            left.clone()
        } else {
            return Err(op_error(
                "binary.shape",
                "operand shapes must match or one operand must be scalar",
            ));
        };
        Ok(vec![ValueFact::new(F32::resolved_type(), shape)])
    }
}

struct StrictSerialSumF32;

impl OperationInferencer for StrictSerialSumF32 {
    fn infer(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        if operands.len() != 1 {
            return Err(op_error("sum.arity", "Sum requires one operand"));
        }
        if operands[0].resolved_type() != &F32::resolved_type() {
            return Err(op_error("sum.type", "strict serial Sum requires f32"));
        }
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "sum.attributes",
                "Sum requires exactly the axes attribute",
            ));
        }
        let Some(CanonicalValueView::Sequence(values)) = attributes
            .get(REDUCTION_AXES_ATTRIBUTE)
            .map(CanonicalValue::view)
        else {
            return Err(op_error("sum.axes", "Sum axes must be a sequence"));
        };
        if values.is_empty() {
            return Err(op_error("sum.axes.empty", "Sum axes cannot be empty"));
        }
        let mut reduced_axes = Vec::with_capacity(values.len());
        for value in values {
            let CanonicalValueView::Unsigned { width, bits } = value.view() else {
                return Err(op_error("sum.axes.type", "Sum axes must be unsigned"));
            };
            if width != super::types::CanonicalIntegerWidth::Bits32 {
                return Err(op_error("sum.axes.width", "Sum axes must use u32"));
            }
            let logical_axis = Axis::new(
                u32::try_from(bits)
                    .map_err(|_| op_error("sum.axes.width", "Sum axis exceeds u32"))?,
            );
            if usize::try_from(logical_axis.get())
                .map_or(true, |position| position >= operands[0].shape().rank())
            {
                return Err(op_error("sum.axes.range", "Sum axis is out of range"));
            }
            if reduced_axes
                .last()
                .is_some_and(|prior: &Axis| prior >= &logical_axis)
            {
                return Err(op_error(
                    "sum.axes.canonical",
                    "Sum axes must be unique and strictly ascending",
                ));
            }
            reduced_axes.push(logical_axis);
        }
        Ok(vec![ValueFact::new(
            F32::resolved_type(),
            operands[0].shape().without_axes(&reduced_axes),
        )])
    }
}

fn op_error(code: &str, message: &str) -> OperationInferenceError {
    OperationInferenceError::new(code, message)
}

fn compute_identity(
    definitions: &BTreeMap<ValueTypeDefinitionKey, RegisteredValueType>,
    operations: &BTreeMap<OpKey, RegisteredOperation>,
) -> SemanticRegistrySnapshotIdentity {
    let mut bytes = b"tiler.semantic-registry.v5\0".to_vec();
    encode_len(&mut bytes, definitions.len());
    for (key, registered) in definitions {
        encode_registered_type(&mut bytes, key, registered);
    }
    encode_len(&mut bytes, operations.len());
    for (key, registered) in operations {
        encode_registered_operation(&mut bytes, key, registered);
    }
    SemanticRegistrySnapshotIdentity(bytes)
}

fn observe_authority_root(observed: &mut usize) -> Result<(), RegistryError> {
    *observed = observed.checked_add(1).unwrap_or(usize::MAX);
    if *observed > MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS {
        return Err(RegistryError::SemanticAuthorityResourceExceeded {
            resource: SemanticAuthorityResource::RootItems,
            limit: MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS,
            actual: *observed,
        });
    }
    Ok(())
}

fn enqueue_type_instance(
    value: ResolvedValueType,
    closure: &mut SemanticAuthorityClosure,
    pending: &mut BTreeSet<ResolvedValueType>,
) -> Result<(), RegistryError> {
    if closure.type_instances.insert(value.clone()) {
        check_closure_budget(closure)?;
        pending.insert(value);
    }
    Ok(())
}

fn enqueue_type_definition(
    key: ValueTypeDefinitionKey,
    closure: &mut SemanticAuthorityClosure,
    pending: &mut BTreeSet<ValueTypeDefinitionKey>,
) -> Result<(), RegistryError> {
    if closure.type_keys.insert(key.clone()) {
        check_closure_budget(closure)?;
        pending.insert(key);
    }
    Ok(())
}

fn enqueue_operation(
    key: OpKey,
    closure: &mut SemanticAuthorityClosure,
    pending: &mut BTreeSet<OpKey>,
) -> Result<(), RegistryError> {
    if closure.operation_keys.insert(key.clone()) {
        check_closure_budget(closure)?;
        pending.insert(key);
    }
    Ok(())
}

fn enqueue_referenced_types(
    value: &ResolvedValueType,
    closure: &mut SemanticAuthorityClosure,
    pending: &mut BTreeSet<ResolvedValueType>,
) -> Result<(), RegistryError> {
    let mut failure = None;
    value.visit_referenced_types(&mut |component| {
        if failure.is_none()
            && let Err(error) = enqueue_type_instance(component.clone(), closure, pending)
        {
            failure = Some(error);
        }
    });
    failure.map_or(Ok(()), Err)
}

fn enqueue_canonical_types(
    value: &CanonicalValue,
    closure: &mut SemanticAuthorityClosure,
    pending: &mut BTreeSet<ResolvedValueType>,
) -> Result<(), RegistryError> {
    let mut failure = None;
    value.visit_referenced_types(&mut |component| {
        if failure.is_none()
            && let Err(error) = enqueue_type_instance(component.clone(), closure, pending)
        {
            failure = Some(error);
        }
    });
    failure.map_or(Ok(()), Err)
}

fn check_closure_budget(closure: &SemanticAuthorityClosure) -> Result<(), RegistryError> {
    let actual = closure
        .type_keys
        .len()
        .checked_add(closure.type_instances.len())
        .and_then(|count| count.checked_add(closure.operation_keys.len()))
        .unwrap_or(usize::MAX);
    if actual > MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS {
        return Err(RegistryError::SemanticAuthorityResourceExceeded {
            resource: SemanticAuthorityResource::ClosureItems,
            limit: MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS,
            actual,
        });
    }
    Ok(())
}

fn encode_registered_type(
    output: &mut Vec<u8>,
    key: &ValueTypeDefinitionKey,
    registered: &RegisteredValueType,
) {
    encode_type_definition(output, key, &registered.definition);
    registered.provider.encode(output);
}

fn encode_type_definition(
    output: &mut Vec<u8>,
    key: &ValueTypeDefinitionKey,
    definition: &ValueTypeDefinition,
) {
    key.encode(output);
    encode_bytes(output, definition.normative_definition.as_str().as_bytes());
    definition.canonical_facts.0.encode(output);
}

fn encode_registered_operation(
    output: &mut Vec<u8>,
    key: &OpKey,
    registered: &RegisteredOperation,
) {
    encode_operation_definition(output, key, &registered.definition);
    registered.provider.encode(output);
}

fn encode_operation_definition(
    output: &mut Vec<u8>,
    key: &OpKey,
    definition: &OperationDefinition,
) {
    key.encode(output);
    encode_bytes(
        output,
        definition.normative_definition().as_str().as_bytes(),
    );
    definition.schema().encode(output);
    definition.canonical_facts().value().encode(output);
    definition.conformance().value().encode(output);
    output.push(match definition.effect() {
        OperationEffect::Pure => 1,
    });
}

fn encode_type_key(output: &mut Vec<u8>, key: &TypeKey) {
    encode_bytes(output, key.namespace().as_bytes());
    encode_bytes(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{EncodedNumericContract, TypeArguments};

    struct TestProvider {
        name: &'static str,
        revision: u32,
        definitions: Vec<ValueTypeDefinition>,
        operations: Vec<OperationDefinition>,
    }

    impl SemanticRegistryProvider for TestProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", self.name, self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            for definition in &self.definitions {
                registrar.register_value_type(definition.clone())?;
            }
            for operation in &self.operations {
                registrar.register_operation(operation.clone())?;
            }
            Ok(())
        }
    }

    fn nominal_definition(key: TypeKey, facts: CanonicalValue) -> ValueTypeDefinition {
        ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(key),
            NormativeDefinitionRef::new("test nominal definition").unwrap(),
            TypeDefinitionFacts::new(facts),
        )
    }

    enum ExternalF8 {}
    impl ValueTypeMarker for ExternalF8 {}

    fn external_f8() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("acme", "f8-special", 1).unwrap())
    }

    struct ExternalProvider;
    impl SemanticRegistryProvider for ExternalProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("acme", "value-types", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("acme", "f8-special", 1).unwrap()),
                NormativeDefinitionRef::new("acme f8 special v1")?,
                TypeDefinitionFacts::new(CanonicalValue::unsigned_u32(8)),
            ))?;
            registrar.bind_marker::<ExternalF8>(external_f8())
        }
    }

    struct Families;
    impl SemanticRegistryProvider for Families {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("tiler", "families", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Parameterized(TypeKey::new("tiler", "complex", 1).unwrap()),
                NormativeDefinitionRef::new("tiler complex v1")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))?;
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::EncodedNumeric(
                    QuantSchemeKey::new("tiler", "affine", 1).unwrap(),
                ),
                NormativeDefinitionRef::new("tiler affine v1")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))
        }
    }

    fn external_identity_op() -> OpKey {
        OpKey::new("acme", "identity", 1).unwrap()
    }

    struct IdentityInferencer;
    impl OperationInferencer for IdentityInferencer {
        fn infer(
            &self,
            operands: &[ValueFact],
            attributes: &OperationAttributes,
        ) -> Result<Vec<ValueFact>, OperationInferenceError> {
            if operands.len() != 1 || !attributes.fields().is_empty() {
                return Err(OperationInferenceError::new(
                    "acme.identity.signature",
                    "identity requires one operand and no attributes",
                ));
            }
            Ok(vec![operands[0].clone()])
        }
    }

    struct ExternalOperationProvider;
    impl SemanticRegistryProvider for ExternalOperationProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("acme", "operations", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_operation(OperationDefinition::new(
                external_identity_op(),
                exact_schema(1, 1, []),
                NormativeDefinitionRef::new("acme identity v1")?,
                OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
                OperationConformance::new(CanonicalValue::utf8("acme.identity.tests.v1").unwrap()),
                OperationEffect::Pure,
                Arc::new(IdentityInferencer),
            ))
        }
    }

    #[test]
    fn semantic_definition_does_not_require_marker() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&Families).unwrap();
        let registry = builder.freeze().unwrap();
        let complex = ResolvedValueType::parameterized(
            TypeKey::new("tiler", "complex", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(F32::resolved_type())]).unwrap(),
        )
        .unwrap();
        assert!(registry.contains(&complex));
    }

    #[test]
    fn encoded_instance_validates_referenced_storage() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&ExternalProvider).unwrap();
        builder.register_provider(&Families).unwrap();
        let registry = builder.freeze().unwrap();
        let encoded = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("tiler", "affine", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::value_type(external_f8()),
            )])
            .unwrap(),
        )
        .unwrap();
        assert!(registry.contains(&encoded));
    }

    #[test]
    fn marker_binding_is_optional_and_checked() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&ExternalProvider).unwrap();
        let registry = builder.freeze().unwrap();
        assert_eq!(
            registry.resolve_marker::<ExternalF8>().unwrap(),
            &external_f8()
        );
    }

    #[test]
    fn registry_snapshot_identity_ignores_provider_registration_order() {
        let mut first = SemanticRegistryBuilder::new();
        first.register_provider(&StandardSemantics).unwrap();
        first.register_provider(&ExternalProvider).unwrap();

        let mut second = SemanticRegistryBuilder::new();
        second.register_provider(&ExternalProvider).unwrap();
        second.register_provider(&StandardSemantics).unwrap();

        assert_eq!(
            first.freeze().unwrap().snapshot_identity(),
            second.freeze().unwrap().snapshot_identity()
        );
    }

    #[test]
    fn external_operation_uses_the_same_checked_inference_path() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder
            .register_provider(&ExternalOperationProvider)
            .unwrap();
        let registry = builder.freeze().unwrap();
        let operand = ValueFact::new(F32::resolved_type(), crate::shape::Shape::from_dims([2, 3]));

        let results = registry
            .infer_operation(
                &external_identity_op(),
                std::slice::from_ref(&operand),
                &OperationAttributes::empty(),
            )
            .unwrap();

        assert_eq!(results, vec![operand]);
    }

    #[test]
    fn operation_attributes_require_registered_embedded_type_authority() {
        let registry = SemanticRegistryBuilder::standard()
            .unwrap()
            .freeze()
            .unwrap();
        let attributes = OperationAttributes::new([CanonicalField::new(
            F32_CONSTANT_BITS_ATTRIBUTE,
            CanonicalValue::float_bits(
                TypeKey::new("external", "unregistered-float", 1).unwrap(),
                [0_u8; 4],
            )
            .unwrap(),
        )])
        .unwrap();

        assert!(matches!(
            registry.normalize_operation_attributes(&constant_f32_op(), attributes),
            Err(RegistryError::UnregisteredTypeAuthority { .. })
        ));
    }

    #[test]
    fn reached_projection_excludes_unrelated_registered_operations() {
        let standard = SemanticRegistryBuilder::standard()
            .unwrap()
            .freeze()
            .unwrap();
        let mut extended = SemanticRegistryBuilder::standard().unwrap();
        extended
            .register_provider(&ExternalOperationProvider)
            .unwrap();
        let extended = extended.freeze().unwrap();

        let standard_projection = standard
            .project_caller_rooted_definitions([&F32::resolved_type()], [&multiply_f32_op()])
            .unwrap();
        let extended_projection = extended
            .project_caller_rooted_definitions([&F32::resolved_type()], [&multiply_f32_op()])
            .unwrap();

        assert_eq!(standard_projection, extended_projection);
        assert_ne!(standard.snapshot_identity(), extended.snapshot_identity());
    }

    #[test]
    fn freeze_rejects_marker_without_semantic_authority() {
        struct MarkerOnly;
        impl SemanticRegistryProvider for MarkerOnly {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("acme", "marker-only", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), RegistryError> {
                registrar.bind_marker::<ExternalF8>(external_f8())
            }
        }

        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&MarkerOnly).unwrap();
        assert!(matches!(
            builder.freeze(),
            Err(RegistryError::UnregisteredTypeAuthority { .. })
        ));
    }

    #[test]
    fn duplicate_authority_rejection_preserves_existing_builder() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&ExternalProvider).unwrap();
        assert!(matches!(
            builder.register_provider(&ExternalProvider),
            Err(RegistryError::DuplicateTypeAuthority { .. })
        ));
        assert!(builder.freeze().unwrap().contains(&external_f8()));
    }

    #[test]
    fn family_validator_can_reject_a_structurally_valid_instance() {
        struct Reject;
        impl ValueTypeInstanceValidator for Reject {
            fn validate(&self, _: &ResolvedValueType) -> Result<(), TypeInstanceError> {
                Err(TypeInstanceError::new(
                    "unsupported-component",
                    "the component is not a real scalar",
                ))
            }
        }

        struct RejectingFamily;
        impl SemanticRegistryProvider for RejectingFamily {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("acme", "rejecting-family", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), RegistryError> {
                registrar.register_value_type(ValueTypeDefinition::new(
                    ValueTypeDefinitionKey::Parameterized(
                        TypeKey::new("acme", "rejected", 1).unwrap(),
                    ),
                    NormativeDefinitionRef::new("acme rejected family")?,
                    TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
                    Arc::new(Reject),
                ))
            }
        }

        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&RejectingFamily).unwrap();
        let registry = builder.freeze().unwrap();
        let value = ResolvedValueType::parameterized(
            TypeKey::new("acme", "rejected", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(F32::resolved_type())]).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            registry.validate_type(&value),
            Err(RegistryError::RejectedTypeInstance(_))
        ));
    }

    #[test]
    fn provider_failure_is_transactional_without_cloning_callback_state() {
        struct Failing;
        impl SemanticRegistryProvider for Failing {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("acme", "failing", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), RegistryError> {
                registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                    ValueTypeDefinitionKey::Nominal(TypeKey::new("acme", "temporary", 1).unwrap()),
                    NormativeDefinitionRef::new("temporary")?,
                    TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
                ))?;
                Err(RegistryError::EmptyNormativeDefinition)
            }
        }

        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        assert!(builder.register_provider(&Failing).is_err());
        let registry = builder.freeze().unwrap();
        assert!(!registry.contains(&ResolvedValueType::nominal(
            TypeKey::new("acme", "temporary", 1).unwrap()
        )));
    }

    #[test]
    fn authority_closure_follows_nested_parameterized_and_encoded_components() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&ExternalProvider).unwrap();
        builder.register_provider(&Families).unwrap();
        let registry = builder.freeze().unwrap();
        let encoded = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("tiler", "affine", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::value_type(external_f8()),
            )])
            .unwrap(),
        )
        .unwrap();
        let nested = ResolvedValueType::parameterized(
            TypeKey::new("tiler", "complex", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(encoded)]).unwrap(),
        )
        .unwrap();

        let closure = registry
            .close_authority(
                [&nested],
                std::iter::empty(),
                std::iter::empty(),
                std::iter::empty(),
            )
            .unwrap();

        assert!(
            closure
                .type_keys
                .contains(&ValueTypeDefinitionKey::Parameterized(
                    TypeKey::new("tiler", "complex", 1).unwrap()
                ))
        );
        assert!(
            closure
                .type_keys
                .contains(&ValueTypeDefinitionKey::EncodedNumeric(
                    QuantSchemeKey::new("tiler", "affine", 1).unwrap()
                ))
        );
        assert!(closure.type_keys.contains(&ValueTypeDefinitionKey::Nominal(
            TypeKey::new("acme", "f8-special", 1).unwrap()
        )));
    }

    #[test]
    fn authority_closure_follows_type_and_float_bits_occurrence_attributes() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&ExternalProvider).unwrap();
        let registry = builder.freeze().unwrap();
        let type_only = CanonicalValue::value_type(external_f8());
        let float_only =
            CanonicalValue::float_bits(TypeKey::new("acme", "f8-special", 1).unwrap(), [0_u8])
                .unwrap();

        for attribute in [&type_only, &float_only] {
            let closure = registry
                .close_authority(
                    std::iter::empty(),
                    std::iter::empty(),
                    std::iter::empty(),
                    [attribute],
                )
                .unwrap();
            assert!(closure.type_keys.contains(&ValueTypeDefinitionKey::Nominal(
                TypeKey::new("acme", "f8-special", 1).unwrap()
            )));
        }
    }

    #[test]
    fn operation_authority_closure_follows_defaults_facts_and_conformance() {
        let default_key = TypeKey::new("test", "default-type", 1).unwrap();
        let facts_key = TypeKey::new("test", "facts-type", 1).unwrap();
        let conformance_key = TypeKey::new("test", "conformance-float", 1).unwrap();
        let operation_key = OpKey::new("test", "metadata-closure", 1).unwrap();
        let provider = TestProvider {
            name: "operation-metadata",
            revision: 1,
            definitions: vec![
                nominal_definition(default_key.clone(), CanonicalValue::boolean(true)),
                nominal_definition(facts_key.clone(), CanonicalValue::boolean(true)),
                nominal_definition(conformance_key.clone(), CanonicalValue::boolean(true)),
            ],
            operations: vec![OperationDefinition::new(
                operation_key.clone(),
                exact_schema(
                    1,
                    1,
                    [OperationAttributeSchema::defaulted(
                        AttributeFieldId::new(1),
                        CanonicalValueKind::Type,
                        CanonicalValue::value_type(ResolvedValueType::nominal(default_key.clone())),
                    )
                    .unwrap()],
                ),
                NormativeDefinitionRef::new("test metadata closure").unwrap(),
                OperationDefinitionFacts::new(CanonicalValue::value_type(
                    ResolvedValueType::nominal(facts_key.clone()),
                )),
                OperationConformance::new(
                    CanonicalValue::float_bits(conformance_key.clone(), [0_u8; 4]).unwrap(),
                ),
                OperationEffect::Pure,
                Arc::new(IdentityInferencer),
            )],
        };
        let mut builder = SemanticRegistryBuilder::new();
        builder.register_provider(&provider).unwrap();
        let registry = builder.freeze().unwrap();

        let closure = registry
            .close_authority(
                std::iter::empty(),
                std::iter::empty(),
                [&operation_key],
                std::iter::empty(),
            )
            .unwrap();
        for key in [default_key, facts_key, conformance_key] {
            assert!(
                closure
                    .type_keys
                    .contains(&ValueTypeDefinitionKey::Nominal(key))
            );
        }
    }

    #[test]
    fn freeze_rejects_type_definition_fact_without_authority() {
        let missing = ResolvedValueType::nominal(TypeKey::new("test", "missing", 1).unwrap());
        let provider = TestProvider {
            name: "missing-fact",
            revision: 1,
            definitions: vec![nominal_definition(
                TypeKey::new("test", "root", 1).unwrap(),
                CanonicalValue::value_type(missing),
            )],
            operations: Vec::new(),
        };
        let mut builder = SemanticRegistryBuilder::new();
        builder.register_provider(&provider).unwrap();
        assert!(matches!(
            builder.freeze(),
            Err(RegistryError::UnregisteredTypeAuthority { .. })
        ));
    }

    #[test]
    fn finite_type_definition_cycles_are_deterministic_and_cycle_safe() {
        let left_key = TypeKey::new("test", "cycle-left", 1).unwrap();
        let right_key = TypeKey::new("test", "cycle-right", 1).unwrap();
        let left = nominal_definition(
            left_key.clone(),
            CanonicalValue::value_type(ResolvedValueType::nominal(right_key.clone())),
        );
        let right = nominal_definition(
            right_key.clone(),
            CanonicalValue::value_type(ResolvedValueType::nominal(left_key.clone())),
        );
        let build = |definitions| {
            let provider = TestProvider {
                name: "finite-cycle",
                revision: 1,
                definitions,
                operations: Vec::new(),
            };
            let mut builder = SemanticRegistryBuilder::new();
            builder.register_provider(&provider).unwrap();
            builder.freeze().unwrap()
        };
        let first = build(vec![left.clone(), right.clone()]);
        let second = build(vec![right, left]);
        let root = ResolvedValueType::nominal(left_key.clone());
        let closure = first
            .close_authority(
                [&root],
                std::iter::empty(),
                std::iter::empty(),
                std::iter::empty(),
            )
            .unwrap();

        assert_eq!(
            closure.type_keys,
            BTreeSet::from([
                ValueTypeDefinitionKey::Nominal(left_key),
                ValueTypeDefinitionKey::Nominal(right_key),
            ])
        );

        assert_eq!(
            first
                .project_caller_rooted_definitions([&root], std::iter::empty())
                .unwrap(),
            second
                .project_caller_rooted_definitions([&root], std::iter::empty())
                .unwrap()
        );
    }

    #[test]
    fn aggregate_authority_closure_rejects_the_first_item_over_limit() {
        let definition_count = MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS / 2 + 1;
        let keys: Vec<_> = (0..definition_count)
            .map(|index| TypeKey::new("limit", format!("type-{index:04}"), 1).unwrap())
            .collect();
        let definitions = keys
            .iter()
            .enumerate()
            .map(|(index, key)| {
                let facts = keys.get(index + 1).map_or_else(
                    || CanonicalValue::boolean(true),
                    |next| CanonicalValue::value_type(ResolvedValueType::nominal(next.clone())),
                );
                nominal_definition(key.clone(), facts)
            })
            .collect();
        let provider = TestProvider {
            name: "closure-limit",
            revision: 1,
            definitions,
            operations: Vec::new(),
        };
        let mut builder = SemanticRegistryBuilder::new();
        builder.register_provider(&provider).unwrap();

        assert!(matches!(
            builder.freeze(),
            Err(RegistryError::SemanticAuthorityResourceExceeded {
                resource: SemanticAuthorityResource::ClosureItems,
                limit: MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS,
                actual,
            }) if actual == MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS + 1
        ));
    }

    #[test]
    fn authority_root_ingestion_stops_at_the_first_item_over_limit() {
        let registry = FrozenSemanticRegistry::standard().unwrap();
        let root = F32::resolved_type();
        let polls = std::cell::Cell::new(0_usize);
        let roots = std::iter::repeat_n(&root, MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS + 2)
            .inspect(|_| polls.set(polls.get() + 1));

        assert!(matches!(
            registry.close_authority(
                roots,
                std::iter::empty(),
                std::iter::empty(),
                std::iter::empty(),
            ),
            Err(RegistryError::SemanticAuthorityResourceExceeded {
                resource: SemanticAuthorityResource::RootItems,
                limit: MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS,
                actual,
            }) if actual == MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS + 1
        ));
        assert_eq!(polls.get(), MAX_SEMANTIC_AUTHORITY_ROOT_ITEMS + 1);
    }
}
