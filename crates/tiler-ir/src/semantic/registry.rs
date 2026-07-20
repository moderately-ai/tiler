use std::any::{TypeId, type_name};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

use super::types::{
    CanonicalField, CanonicalValue, QuantSchemeKey, ResolvedValueType, TypeIdentityError, TypeKey,
};

const MAX_DEFINITION_REFERENCE_BYTES: usize = 4 * 1024;

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
    marker_bindings: HashMap<TypeId, MarkerBinding>,
}

impl fmt::Debug for SemanticRegistryBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SemanticRegistryBuilder")
            .field("definition_count", &self.definitions.len())
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
        builder.register_provider(&StandardValueTypes)?;
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
        if batch.definitions.is_empty() && batch.marker_bindings.is_empty() {
            return Err(RegistryError::ProviderRegisteredNothing { provider: identity });
        }
        for key in batch.definitions.keys() {
            if self.definitions.contains_key(key) {
                return Err(RegistryError::DuplicateTypeAuthority {
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
        self.marker_bindings.extend(batch.marker_bindings);
        Ok(())
    }

    /// Freezes this registry into canonical immutable shared state.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when empty or when a marker target is not
    /// admitted by the completed semantic authority set.
    pub fn freeze(self) -> Result<FrozenSemanticRegistry, RegistryError> {
        if self.definitions.is_empty() {
            return Err(RegistryError::EmptyRegistry);
        }
        let registry = FrozenSemanticRegistry(Arc::new(FrozenRegistryData {
            identity: OnceLock::new(),
            definitions: self.definitions,
            marker_bindings: self.marker_bindings,
        }));
        for binding in registry.0.marker_bindings.values() {
            registry.validate_type(&binding.resolved_type)?;
        }
        let _ = registry.canonical_identity();
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
        let key = definition.key.clone();
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
            .field("marker_count", &self.0.marker_bindings.len())
            .finish()
    }
}

struct FrozenRegistryData {
    definitions: BTreeMap<ValueTypeDefinitionKey, RegisteredValueType>,
    marker_bindings: HashMap<TypeId, MarkerBinding>,
    identity: OnceLock<CanonicalSemanticRegistryIdentity>,
}

impl FrozenSemanticRegistry {
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
        let key = ValueTypeDefinitionKey::for_value(value);
        let registered = self.0.definitions.get(&key).ok_or_else(|| {
            RegistryError::UnregisteredTypeAuthority {
                key: Arc::new(key.clone()),
            }
        })?;
        let mut failure = None;
        value.visit_referenced_types(&mut |component| {
            if failure.is_none()
                && let Err(error) = self.validate_type(component)
            {
                failure = Some(error);
            }
        });
        registered
            .definition
            .canonical_facts
            .0
            .visit_referenced_types(&mut |component| {
                if failure.is_none()
                    && let Err(error) = self.validate_type(component)
                {
                    failure = Some(error);
                }
            });
        if let Some(error) = failure {
            return Err(error);
        }
        registered
            .definition
            .validator
            .validate(value)
            .map_err(|source| {
                RegistryError::RejectedTypeInstance(Arc::new(TypeInstanceRejection {
                    key,
                    provider: registered.provider.clone(),
                    source,
                }))
            })
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

    /// Returns complete frozen semantic-registry provenance.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalSemanticRegistryIdentity {
        self.0
            .identity
            .get_or_init(|| compute_identity(&self.0.definitions))
    }
}

/// Collision-free canonical provenance for a frozen semantic registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalSemanticRegistryIdentity(Vec<u8>);

impl CanonicalSemanticRegistryIdentity {
    /// Returns the canonical provenance bytes.
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

struct StandardValueTypes;

impl SemanticRegistryProvider for StandardValueTypes {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler", "standard-value-types", 2)
            .expect("the governed standard provider identity is valid")
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let facts = TypeDefinitionFacts::new(
            CanonicalValue::record([
                CanonicalField::new(
                    1,
                    CanonicalValue::utf8("ieee-binary").expect("the governed F32 class is bounded"),
                ),
                CanonicalField::new(2, CanonicalValue::unsigned(32)),
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
        )
    }
}

fn compute_identity(
    definitions: &BTreeMap<ValueTypeDefinitionKey, RegisteredValueType>,
) -> CanonicalSemanticRegistryIdentity {
    let mut bytes = b"tiler.semantic-registry.v2\0".to_vec();
    encode_len(&mut bytes, definitions.len());
    for (key, registered) in definitions {
        key.encode(&mut bytes);
        registered.provider.encode(&mut bytes);
        encode_bytes(
            &mut bytes,
            registered
                .definition
                .normative_definition
                .as_str()
                .as_bytes(),
        );
        registered.definition.canonical_facts.0.encode(&mut bytes);
    }
    CanonicalSemanticRegistryIdentity(bytes)
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
                TypeDefinitionFacts::new(CanonicalValue::unsigned(8)),
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
                1,
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
    fn registry_identity_ignores_provider_registration_order() {
        let mut first = SemanticRegistryBuilder::new();
        first.register_provider(&StandardValueTypes).unwrap();
        first.register_provider(&ExternalProvider).unwrap();

        let mut second = SemanticRegistryBuilder::new();
        second.register_provider(&ExternalProvider).unwrap();
        second.register_provider(&StandardValueTypes).unwrap();

        assert_eq!(
            first.freeze().unwrap().canonical_identity(),
            second.freeze().unwrap().canonical_identity()
        );
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
}
