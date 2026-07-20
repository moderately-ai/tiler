use std::any::{TypeId, type_name};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

use super::types::{
    IdentityComponent, MAX_IDENTITY_COMPONENT_BYTES, MAX_RESOLVED_TYPE_BYTES, ResolvedValueType,
    ResolvedValueTypeArgument, ResolvedValueTypeField, TypeIdentityError, TypeKey,
};

/// Maximum UTF-8 byte length of one normative-definition reference.
pub const MAX_DEFINITION_REFERENCE_BYTES: usize = 4_096;

/// An intentionally empty local Rust type marker.
///
/// Implementing this trait grants no semantic identity or authority. Only an
/// explicit binding in a [`FrozenSemanticRegistry`] makes a marker usable.
pub trait ValueTypeMarker: 'static {}

/// The standard marker for Tiler's `tiler::f32@1` nominal type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum F32 {}

impl F32 {
    /// Returns the canonical built-in resolved value type.
    ///
    /// # Panics
    ///
    /// Panics only if Tiler's compile-time governed key violates its own key
    /// grammar, which is an internal library defect.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        ResolvedValueType::nominal(
            TypeKey::new("tiler", "f32", 1).expect("the governed f32 key is valid"),
        )
    }
}

impl ValueTypeMarker for F32 {}

/// Stable identity and revision of a semantic-definition provider.
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
    /// Returns [`RegistryError`] for invalid components or revision zero.
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        revision: u32,
    ) -> Result<Self, RegistryError> {
        let namespace = namespace.into();
        let name = name.into();
        validate_provider_component(IdentityComponent::Namespace, &namespace)?;
        validate_provider_component(IdentityComponent::Name, &name)?;
        if revision == 0 {
            return Err(RegistryError::ZeroProviderRevision);
        }
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

    /// Returns the provider name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the output-affecting provider revision.
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

/// Portable semantic definition of one complete resolved value type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueTypeDefinition {
    resolved_type: ResolvedValueType,
    normative_definition: String,
    canonical_facts: ResolvedValueTypeArgument,
}

impl ValueTypeDefinition {
    /// Creates a validated host-canonical semantic definition.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when the normative reference is empty or too
    /// large. The resolved identity and facts are already bounded by their
    /// constructors.
    pub fn new(
        resolved_type: ResolvedValueType,
        normative_definition: impl Into<String>,
        canonical_facts: ResolvedValueTypeArgument,
    ) -> Result<Self, RegistryError> {
        let normative_definition = normative_definition.into();
        if normative_definition.is_empty() {
            return Err(RegistryError::EmptyNormativeDefinition);
        }
        if normative_definition.len() > MAX_DEFINITION_REFERENCE_BYTES {
            return Err(RegistryError::NormativeDefinitionTooLong {
                bytes: normative_definition.len(),
            });
        }
        Ok(Self {
            resolved_type,
            normative_definition,
            canonical_facts,
        })
    }

    /// Returns the complete resolved value type being defined.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns the exact normative-definition reference.
    #[must_use]
    pub fn normative_definition(&self) -> &str {
        &self.normative_definition
    }

    /// Returns the bounded canonical semantic facts.
    #[must_use]
    pub const fn canonical_facts(&self) -> &ResolvedValueTypeArgument {
        &self.canonical_facts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegisteredValueType {
    definition: ValueTypeDefinition,
    provider: ProviderIdentity,
}

/// A statically linked source of semantic type definitions and marker bindings.
///
/// Providers run only while a registry is built. They are not retained by the
/// frozen registry or by semantic programs.
pub trait SemanticRegistryProvider {
    /// Returns stable provider identity and output-affecting revision.
    fn identity(&self) -> ProviderIdentity;

    /// Registers definitions through the host-owned transactional registrar.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for invalid definitions or collisions.
    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError>;
}

/// Mutable, single-use constructor for a frozen semantic registry.
///
/// Freezing consumes the builder, making post-freeze mutation impossible:
///
/// ```compile_fail
/// use tiler_ir::semantic::SemanticRegistryBuilder;
///
/// let builder = SemanticRegistryBuilder::new();
/// let _registry = builder.freeze().unwrap();
/// let _second = builder.freeze();
/// ```
#[derive(Clone, Debug, Default)]
pub struct SemanticRegistryBuilder {
    definitions: BTreeMap<ResolvedValueType, RegisteredValueType>,
    marker_bindings: HashMap<TypeId, MarkerBinding>,
}

#[derive(Clone, Debug)]
struct MarkerBinding {
    marker_name: &'static str,
    resolved_type: ResolvedValueType,
}

impl SemanticRegistryBuilder {
    /// Creates an empty registry builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the mutable governed standard profile.
    ///
    /// External providers may be registered on this builder before its single
    /// freeze transition.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] if a governed built-in violates the same
    /// registration contract applied to extensions.
    pub fn standard() -> Result<Self, RegistryError> {
        let mut builder = Self::new();
        builder.register_provider(&StandardValueTypes)?;
        Ok(builder)
    }

    /// Applies one provider transactionally.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] without changing this builder if the provider
    /// fails, registers nothing, or collides with existing authority.
    pub fn register_provider(
        &mut self,
        provider: &impl SemanticRegistryProvider,
    ) -> Result<(), RegistryError> {
        let mut candidate = self.clone();
        let before = candidate.definitions.len();
        let identity = provider.identity();
        let mut registrar = SemanticRegistryRegistrar {
            builder: &mut candidate,
            provider: identity.clone(),
        };
        provider.register(&mut registrar)?;
        if candidate.definitions.len() == before {
            return Err(RegistryError::ProviderRegisteredNothing { provider: identity });
        }
        *self = candidate;
        Ok(())
    }

    /// Freezes this registry into canonical immutable shared state.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::EmptyRegistry`] when no semantic authority was
    /// registered.
    pub fn freeze(self) -> Result<FrozenSemanticRegistry, RegistryError> {
        if self.definitions.is_empty() {
            return Err(RegistryError::EmptyRegistry);
        }
        for (owner, registered) in &self.definitions {
            let mut missing = None;
            owner.visit_referenced_types(&mut |component| {
                if missing.is_none() && !self.definitions.contains_key(component) {
                    missing = Some(component.clone());
                }
            });
            registered
                .definition
                .canonical_facts
                .visit_referenced_types(&mut |component| {
                    if missing.is_none() && !self.definitions.contains_key(component) {
                        missing = Some(component.clone());
                    }
                });
            if let Some(component) = missing {
                return Err(RegistryError::UnregisteredComponentType {
                    owner: Arc::new(owner.clone()),
                    component: Arc::new(component),
                });
            }
        }
        let identity = compute_identity(&self.definitions);
        Ok(FrozenSemanticRegistry(Arc::new(FrozenRegistryData {
            definitions: self.definitions,
            marker_bindings: self.marker_bindings,
            identity,
        })))
    }
}

/// Host-owned registration surface supplied to one semantic provider.
pub struct SemanticRegistryRegistrar<'a> {
    builder: &'a mut SemanticRegistryBuilder,
    provider: ProviderIdentity,
}

impl SemanticRegistryRegistrar<'_> {
    /// Registers one complete resolved identity and its canonical Rust marker.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] for duplicate marker or semantic authority.
    pub fn register_value_type<T: ValueTypeMarker>(
        &mut self,
        definition: ValueTypeDefinition,
    ) -> Result<(), RegistryError> {
        let marker = TypeId::of::<T>();
        if let Some(existing) = self.builder.marker_bindings.get(&marker) {
            return Err(RegistryError::DuplicateMarker {
                marker_name: existing.marker_name,
            });
        }
        if self
            .builder
            .definitions
            .contains_key(definition.resolved_type())
        {
            return Err(RegistryError::DuplicateResolvedType {
                resolved_type: Arc::new(definition.resolved_type().clone()),
            });
        }
        let resolved_type = definition.resolved_type().clone();
        self.builder.definitions.insert(
            resolved_type.clone(),
            RegisteredValueType {
                definition,
                provider: self.provider.clone(),
            },
        );
        self.builder.marker_bindings.insert(
            marker,
            MarkerBinding {
                marker_name: type_name::<T>(),
                resolved_type,
            },
        );
        Ok(())
    }
}

/// Immutable, cheap-clone semantic authority used by builders and programs.
#[derive(Clone, Debug)]
pub struct FrozenSemanticRegistry(Arc<FrozenRegistryData>);

#[derive(Debug)]
struct FrozenRegistryData {
    definitions: BTreeMap<ResolvedValueType, RegisteredValueType>,
    marker_bindings: HashMap<TypeId, MarkerBinding>,
    identity: CanonicalSemanticRegistryIdentity,
}

impl FrozenSemanticRegistry {
    /// Builds the standard registry profile containing `tiler::f32@1`.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] if the governed built-in definition violates
    /// the same public registration contract used by extensions.
    pub fn standard() -> Result<Self, RegistryError> {
        static STANDARD: OnceLock<Result<FrozenSemanticRegistry, RegistryError>> = OnceLock::new();
        STANDARD
            .get_or_init(|| SemanticRegistryBuilder::standard()?.freeze())
            .clone()
    }

    /// Resolves one local Rust marker through this exact frozen snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryLookupError::UnregisteredMarker`] when implementing
    /// [`ValueTypeMarker`] was not accompanied by explicit registration.
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

    /// Returns whether this snapshot contains semantic authority for a complete
    /// resolved value type.
    #[must_use]
    pub fn contains(&self, resolved_type: &ResolvedValueType) -> bool {
        self.0.definitions.contains_key(resolved_type)
    }

    /// Returns the canonical provider-independent definition when registered.
    #[must_use]
    pub fn definition(&self, resolved_type: &ResolvedValueType) -> Option<&ValueTypeDefinition> {
        self.0
            .definitions
            .get(resolved_type)
            .map(|registered| &registered.definition)
    }

    /// Returns the provider identity governing one registered definition.
    #[must_use]
    pub fn provider(&self, resolved_type: &ResolvedValueType) -> Option<&ProviderIdentity> {
        self.0
            .definitions
            .get(resolved_type)
            .map(|registered| &registered.provider)
    }

    /// Returns complete frozen semantic-registry provenance.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalSemanticRegistryIdentity {
        &self.0.identity
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

/// Failure to build or freeze semantic registry authority.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistryError {
    /// A provider identity component was invalid.
    InvalidProviderIdentity(TypeIdentityError),
    /// Provider revision zero is reserved.
    ZeroProviderRevision,
    /// A normative-definition reference was empty.
    EmptyNormativeDefinition,
    /// A normative-definition reference exceeded its byte bound.
    NormativeDefinitionTooLong {
        /// Actual UTF-8 byte length.
        bytes: usize,
    },
    /// A provider transaction registered no semantic definition.
    ProviderRegisteredNothing {
        /// Provider that supplied no definition.
        provider: ProviderIdentity,
    },
    /// A Rust marker already had a binding.
    DuplicateMarker {
        /// Diagnostic-only local Rust type name.
        marker_name: &'static str,
    },
    /// A complete resolved identity already had semantic authority.
    DuplicateResolvedType {
        /// Colliding complete identity.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// A definition referenced a complete type absent from the same snapshot.
    UnregisteredComponentType {
        /// Definition containing the reference.
        owner: Arc<ResolvedValueType>,
        /// Missing referenced type.
        component: Arc<ResolvedValueType>,
    },
    /// A frozen registry cannot be empty.
    EmptyRegistry,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProviderIdentity(error) => error.fmt(formatter),
            Self::ZeroProviderRevision => formatter.write_str("provider revision zero is reserved"),
            Self::EmptyNormativeDefinition => {
                formatter.write_str("normative-definition reference is empty")
            }
            Self::NormativeDefinitionTooLong { bytes } => write!(
                formatter,
                "normative-definition reference has {bytes} bytes, exceeding {MAX_DEFINITION_REFERENCE_BYTES}"
            ),
            Self::ProviderRegisteredNothing { provider } => {
                write!(
                    formatter,
                    "provider {provider} registered no semantic definitions"
                )
            }
            Self::DuplicateMarker { marker_name } => {
                write!(formatter, "Rust marker {marker_name} is already registered")
            }
            Self::DuplicateResolvedType { resolved_type } => write!(
                formatter,
                "resolved value type {:?} already has semantic authority",
                resolved_type.canonical_encoding().as_bytes()
            ),
            Self::UnregisteredComponentType { owner, component } => write!(
                formatter,
                "resolved value type {:?} references unregistered component {:?}",
                owner.canonical_encoding().as_bytes(),
                component.canonical_encoding().as_bytes()
            ),
            Self::EmptyRegistry => formatter.write_str("semantic registry is empty"),
        }
    }
}

impl Error for RegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidProviderIdentity(error) => Some(error),
            _ => None,
        }
    }
}

/// Failure to resolve process-local evidence in a frozen registry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistryLookupError {
    /// The marker trait was implemented but no provider bound this marker.
    UnregisteredMarker {
        /// Diagnostic-only local Rust type name.
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
        ProviderIdentity::new("tiler", "standard-value-types", 1)
            .expect("the governed standard provider identity is valid")
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let facts = ResolvedValueTypeArgument::record([
            ResolvedValueTypeField::new(
                1,
                ResolvedValueTypeArgument::utf8("ieee-binary")
                    .expect("the governed f32 class is bounded"),
            ),
            ResolvedValueTypeField::new(2, ResolvedValueTypeArgument::unsigned(32)),
        ])
        .expect("the governed f32 facts are canonical");
        registrar.register_value_type::<F32>(ValueTypeDefinition::new(
            F32::resolved_type(),
            "IEEE 754-2019 binary32; tiler::f32@1",
            facts,
        )?)
    }
}

fn compute_identity(
    definitions: &BTreeMap<ResolvedValueType, RegisteredValueType>,
) -> CanonicalSemanticRegistryIdentity {
    let mut bytes = b"tiler.semantic-registry.v1\0".to_vec();
    encode_len(&mut bytes, definitions.len());
    for (resolved_type, registered) in definitions {
        resolved_type.encode(&mut bytes);
        registered.provider.encode(&mut bytes);
        encode_bytes(
            &mut bytes,
            registered.definition.normative_definition.as_bytes(),
        );
        registered.definition.canonical_facts.encode(&mut bytes);
    }
    CanonicalSemanticRegistryIdentity(bytes)
}

fn validate_provider_component(
    component: IdentityComponent,
    value: &str,
) -> Result<(), RegistryError> {
    if value.is_empty() {
        return Err(RegistryError::InvalidProviderIdentity(
            TypeIdentityError::EmptyComponent { component },
        ));
    }
    if value.len() > MAX_IDENTITY_COMPONENT_BYTES {
        return Err(RegistryError::InvalidProviderIdentity(
            TypeIdentityError::ComponentTooLong {
                component,
                bytes: value.len(),
            },
        ));
    }
    for (index, byte) in value.bytes().enumerate() {
        let valid =
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'.' | b'_' | b'-'));
        if !valid {
            return Err(RegistryError::InvalidProviderIdentity(
                TypeIdentityError::InvalidComponentCharacter {
                    component,
                    byte_index: index,
                },
            ));
        }
    }
    Ok(())
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("bounded registry collection length fits u64")
            .to_be_bytes(),
    );
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    debug_assert!(value.len() <= MAX_RESOLVED_TYPE_BYTES);
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    enum ExternalF8 {}
    impl ValueTypeMarker for ExternalF8 {}

    struct ExternalProvider;

    impl SemanticRegistryProvider for ExternalProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("acme", "f8-semantics", 7).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type::<ExternalF8>(ValueTypeDefinition::new(
                ResolvedValueType::nominal(TypeKey::new("acme", "f8-special", 1).unwrap()),
                "https://example.invalid/acme/f8-special/v1",
                ResolvedValueTypeArgument::record([
                    ResolvedValueTypeField::new(1, ResolvedValueTypeArgument::unsigned(8)),
                    ResolvedValueTypeField::new(
                        2,
                        ResolvedValueTypeArgument::utf8("external-test-format").unwrap(),
                    ),
                ])
                .unwrap(),
            )?)
        }
    }

    #[test]
    fn standard_and_external_types_use_the_same_provider_path() {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder.register_provider(&ExternalProvider).unwrap();
        let registry = builder.freeze().unwrap();
        assert_eq!(
            registry.resolve_marker::<F32>().unwrap(),
            &F32::resolved_type()
        );
        let resolved = registry.resolve_marker::<ExternalF8>().unwrap();
        assert_eq!(resolved.nominal_key().unwrap().namespace(), "acme");
        assert_eq!(registry.provider(resolved).unwrap().revision(), 7);
    }

    #[test]
    fn marker_implementation_alone_grants_no_authority() {
        assert!(matches!(
            FrozenSemanticRegistry::standard()
                .unwrap()
                .resolve_marker::<ExternalF8>(),
            Err(RegistryLookupError::UnregisteredMarker { .. })
        ));
    }

    #[test]
    fn provider_registration_is_transactional_on_collision() {
        let mut builder = SemanticRegistryBuilder::new();
        builder.register_provider(&ExternalProvider).unwrap();
        let identity_before = compute_identity(&builder.definitions);

        assert!(matches!(
            builder.register_provider(&ExternalProvider),
            Err(RegistryError::DuplicateMarker { .. })
        ));
        assert_eq!(compute_identity(&builder.definitions), identity_before);
    }

    #[test]
    fn invalid_definition_and_duplicate_identity_are_rejected() {
        enum Alias {}
        impl ValueTypeMarker for Alias {}
        struct AliasProvider;
        impl SemanticRegistryProvider for AliasProvider {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("another", "f8-provider", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), RegistryError> {
                registrar.register_value_type::<Alias>(ValueTypeDefinition::new(
                    ResolvedValueType::nominal(TypeKey::new("acme", "f8-special", 1).unwrap()),
                    "https://example.invalid/another/f8/v1",
                    ResolvedValueTypeArgument::boolean(true),
                )?)
            }
        }

        assert_eq!(
            ValueTypeDefinition::new(
                F32::resolved_type(),
                "",
                ResolvedValueTypeArgument::boolean(true)
            ),
            Err(RegistryError::EmptyNormativeDefinition)
        );

        let mut builder = SemanticRegistryBuilder::new();
        builder.register_provider(&ExternalProvider).unwrap();
        assert!(matches!(
            builder.register_provider(&AliasProvider),
            Err(RegistryError::DuplicateResolvedType { .. })
        ));
    }

    #[test]
    fn registry_identity_ignores_marker_type_ids_and_registration_order() {
        enum Alias {}
        impl ValueTypeMarker for Alias {}

        struct AliasProvider;
        impl SemanticRegistryProvider for AliasProvider {
            fn identity(&self) -> ProviderIdentity {
                ExternalProvider.identity()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), RegistryError> {
                let definition = ValueTypeDefinition::new(
                    ResolvedValueType::nominal(TypeKey::new("acme", "f8-special", 1).unwrap()),
                    "https://example.invalid/acme/f8-special/v1",
                    ResolvedValueTypeArgument::record([
                        ResolvedValueTypeField::new(1, ResolvedValueTypeArgument::unsigned(8)),
                        ResolvedValueTypeField::new(
                            2,
                            ResolvedValueTypeArgument::utf8("external-test-format").unwrap(),
                        ),
                    ])
                    .unwrap(),
                )?;
                registrar.register_value_type::<Alias>(definition)
            }
        }

        let mut first = SemanticRegistryBuilder::new();
        first.register_provider(&ExternalProvider).unwrap();
        let mut second = SemanticRegistryBuilder::new();
        second.register_provider(&AliasProvider).unwrap();
        assert_eq!(
            first.freeze().unwrap().canonical_identity(),
            second.freeze().unwrap().canonical_identity()
        );

        let mut standard_then_external = SemanticRegistryBuilder::new();
        standard_then_external
            .register_provider(&StandardValueTypes)
            .unwrap();
        standard_then_external
            .register_provider(&ExternalProvider)
            .unwrap();
        let mut external_then_standard = SemanticRegistryBuilder::new();
        external_then_standard
            .register_provider(&ExternalProvider)
            .unwrap();
        external_then_standard
            .register_provider(&StandardValueTypes)
            .unwrap();
        assert_eq!(
            standard_then_external
                .freeze()
                .unwrap()
                .canonical_identity(),
            external_then_standard
                .freeze()
                .unwrap()
                .canonical_identity()
        );
    }

    #[test]
    fn freeze_rejects_dangling_parameterized_component_types() {
        enum ComplexF32 {}
        impl ValueTypeMarker for ComplexF32 {}

        struct ComplexProvider;
        impl SemanticRegistryProvider for ComplexProvider {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("acme", "complex", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), RegistryError> {
                let resolved = ResolvedValueType::parameterized(
                    TypeKey::new("tiler", "complex", 1).unwrap(),
                    [ResolvedValueTypeArgument::value_type(F32::resolved_type())],
                )
                .unwrap();
                registrar.register_value_type::<ComplexF32>(ValueTypeDefinition::new(
                    resolved,
                    "https://example.invalid/complex/v1",
                    ResolvedValueTypeArgument::boolean(true),
                )?)
            }
        }

        let mut incomplete = SemanticRegistryBuilder::new();
        incomplete.register_provider(&ComplexProvider).unwrap();
        assert!(matches!(
            incomplete.freeze(),
            Err(RegistryError::UnregisteredComponentType { .. })
        ));

        let mut complete = SemanticRegistryBuilder::standard().unwrap();
        complete.register_provider(&ComplexProvider).unwrap();
        assert!(complete.freeze().is_ok());
    }
}
