//! Draft compiler lowering-capability registry.
//!
//! This module owns the compiler-side capability registration and resolution
//! that ADR 0044 and the operation-extension contract defer past the semantic
//! authority. It composes the frozen semantic and scalar authorities from
//! `tiler-ir` and binds one or more *lowering* providers to each semantic
//! operation occurrence for two capability families:
//!
//! - [`LoweringFamily::IndexAccess`] providers emit the iteration domain,
//!   tensor accesses, and output roots of one occurrence; and
//! - [`LoweringFamily::ScalarLowering`] providers emit the per-point scalar
//!   computation of one occurrence.
//!
//! A provider only ever receives a narrow checked context
//! ([`IndexAccessLoweringContext`] or [`ScalarLoweringContext`]) that delegates
//! to the canonical `tiler-ir` builders. It cannot construct provider-owned IR,
//! carry an opaque payload, downcast the host context, or finalize the region;
//! the host owns verification. This mirrors the reference-capability registry
//! merged in `tiler-reference` rather than the semantic registry's provider
//! transaction surface.
//!
//! Scope boundary: this registry *resolves available lowering knowledge* with
//! deterministic collision, ambiguity, and missing diagnostics plus canonical
//! provenance. It does not prove that a resolved provider's emitted index work
//! actually refines the semantic occurrence — that separate checked authority
//! belongs to `prototype-semantic-index-refinement`. Registration therefore
//! never asserts numerical, value, or access correctness.
//!
//! Every public item here is a reviewed *draft* boundary. It is not a stable
//! compiler-session API and must not be treated as one until Tom accepts the
//! exact interface.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::index::{
    CanonicalScalarDefinitionProjection, DimensionId, DomainRole, FrozenScalarRegistry,
    IndexBuildError, IndexExprId, IndexInteger, IndexRegionBuilder, ScalarAttributes, ScalarOpKey,
    ScalarReducerBodyBuilder, ScalarRegistryError, ScalarResults, ScalarValueId, TensorAccessId,
    TensorId, TensorRole,
};
use tiler_ir::semantic::{
    FrozenSemanticRegistry, OpKey, ProviderIdentity, RegistryError, ResolvedValueType,
    SemanticCapabilityAuthority,
};
use tiler_ir::shape::{Extent, Shape};

/// Canonical identity domain-separation tag for a frozen registry snapshot.
const REGISTRY_IDENTITY_TAG: &[u8] = b"tiler.compiler.lowering-capability-registry.v1\0";
/// Maximum capabilities admitted by one frozen registry.
const MAX_LOWERING_CAPABILITIES: usize = 65_536;
/// Maximum operand or result types admitted by one signature.
const MAX_SIGNATURE_TYPES: usize = 4_096;
/// Maximum distinct scalar operations one capability may declare it emits.
const MAX_EMITTED_SCALAR_OPERATIONS: usize = 4_096;
/// Maximum canonical identity bytes retained by one frozen registry.
const MAX_LOWERING_REGISTRY_IDENTITY_BYTES: usize = 32 * 1024 * 1024;

/// Which lowering capability family a provider belongs to.
///
/// The declaration order and the encoded discriminant agree, so the derived
/// total order used for durable identity iteration matches the serialized
/// discriminant; a reordered family cannot silently keep its tag.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LoweringFamily {
    /// Emits the iteration domain, tensor accesses, and output roots.
    IndexAccess,
    /// Emits the per-point scalar computation.
    ScalarLowering,
}

impl LoweringFamily {
    /// Returns the stable discriminant shared by ordering and encoding.
    const fn tag(self) -> u8 {
        match self {
            Self::IndexAccess => 1,
            Self::ScalarLowering => 2,
        }
    }
}

impl fmt::Display for LoweringFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IndexAccess => "index-access lowering",
            Self::ScalarLowering => "scalar lowering",
        })
    }
}

/// A nonzero output-affecting revision of one registered lowering capability.
///
/// This is distinct from the admitting [`ProviderIdentity`] revision: a provider
/// may own several capabilities, and each capability declares its own revision
/// covering the exact lowering it emits.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LoweringCapabilityRevision(u32);

impl LoweringCapabilityRevision {
    /// Creates a nonzero capability revision.
    ///
    /// Returns [`None`] for revision zero, which is reserved for "unset".
    #[must_use]
    pub const fn new(revision: u32) -> Option<Self> {
        match revision {
            0 => None,
            revision => Some(Self(revision)),
        }
    }

    /// Returns the nonzero revision value.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The exact resolved operand/result signature one capability lowers.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LoweringSignature {
    operands: Vec<ResolvedValueType>,
    results: Vec<ResolvedValueType>,
}

impl LoweringSignature {
    /// Creates a bounded signature from ordered operand and result types.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringRegistryError::SignatureTooLarge`] when either list
    /// exceeds the governed structural bound.
    pub fn new(
        operands: impl IntoIterator<Item = ResolvedValueType>,
        results: impl IntoIterator<Item = ResolvedValueType>,
    ) -> Result<Self, LoweringRegistryError> {
        let operands = collect_bounded(operands, MAX_SIGNATURE_TYPES)?;
        let results = collect_bounded(results, MAX_SIGNATURE_TYPES)?;
        Ok(Self { operands, results })
    }

    /// Returns the ordered operand types.
    #[must_use]
    pub fn operands(&self) -> &[ResolvedValueType] {
        &self.operands
    }

    /// Returns the ordered result types.
    #[must_use]
    pub fn results(&self) -> &[ResolvedValueType] {
        &self.results
    }

    fn encode(&self, output: &mut Vec<u8>) {
        encode_len(output, self.operands.len());
        for value_type in &self.operands {
            encode_bytes(output, value_type.canonical_encoding().as_bytes());
        }
        encode_len(output, self.results.len());
        for value_type in &self.results {
            encode_bytes(output, value_type.canonical_encoding().as_bytes());
        }
    }
}

fn collect_bounded(
    values: impl IntoIterator<Item = ResolvedValueType>,
    limit: usize,
) -> Result<Vec<ResolvedValueType>, LoweringRegistryError> {
    let mut collected = Vec::new();
    for value in values {
        if collected.len() == limit {
            return Err(LoweringRegistryError::SignatureTooLarge {
                actual: limit.saturating_add(1),
            });
        }
        collected.push(value);
    }
    Ok(collected)
}

/// The stored key for one registered capability.
///
/// The admitting provider participates in the key, so two providers may each
/// claim the same occurrence; the resulting contradiction is a deterministic
/// resolution ambiguity rather than a silent last-wins selection.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct LoweringCapabilityKey {
    family: LoweringFamily,
    operation: OpKey,
    signature: LoweringSignature,
    provider: ProviderIdentity,
}

/// The provider-independent selector used to resolve a capability.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct LoweringSelector<'a> {
    family: LoweringFamily,
    operation: &'a OpKey,
    signature: &'a LoweringSignature,
}

/// Complete reached authority one lowering capability was admitted against.
///
/// Both subjects are provider-independent projections over the composed frozen
/// authorities. Neither uses `TypeId`, vtable, function, or allocation addresses.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LoweringCapabilityAuthority {
    operation_authority: SemanticCapabilityAuthority,
    emitted_scalar_definitions: CanonicalScalarDefinitionProjection,
}

impl LoweringCapabilityAuthority {
    /// Returns the semantic authority of the lowered operation occurrence.
    #[must_use]
    pub const fn operation_authority(&self) -> &SemanticCapabilityAuthority {
        &self.operation_authority
    }

    /// Returns the reached provider-independent scalar definitions the
    /// capability declared it emits.
    #[must_use]
    pub const fn emitted_scalar_definitions(&self) -> &CanonicalScalarDefinitionProjection {
        &self.emitted_scalar_definitions
    }
}

/// One family-typed provider implementation.
#[derive(Clone)]
enum LoweringImplementation {
    IndexAccess(Arc<dyn IndexAccessLoweringProvider>),
    ScalarLowering(Arc<dyn ScalarLoweringProvider>),
}

impl LoweringImplementation {
    const fn family(&self) -> LoweringFamily {
        match self {
            Self::IndexAccess(_) => LoweringFamily::IndexAccess,
            Self::ScalarLowering(_) => LoweringFamily::ScalarLowering,
        }
    }
}

#[derive(Clone)]
struct RegisteredLoweringCapability {
    revision: LoweringCapabilityRevision,
    authority: LoweringCapabilityAuthority,
    implementation: LoweringImplementation,
}

/// A statically linked provider that emits the per-point scalar computation of
/// one semantic operation occurrence.
///
/// The provider is trusted, deterministic, and side-effect-free: it may only
/// depend on its explicit context. Its sole output channel is the canonical
/// scalar builder wrapped by [`ScalarLoweringContext`].
pub trait ScalarLoweringProvider: Send + Sync + 'static {
    /// Emits ordered scalar result values through the canonical builder.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects an
    /// emission or the declared result arity is not satisfied.
    fn lower(
        &self,
        context: &mut ScalarLoweringContext<'_>,
    ) -> Result<ScalarLoweringResults, LoweringEmitError>;
}

/// A statically linked provider that emits the iteration domain, tensor
/// accesses, and output roots of one semantic operation occurrence.
///
/// The provider is trusted, deterministic, and side-effect-free. Its sole output
/// channel is the canonical region builder wrapped by
/// [`IndexAccessLoweringContext`]; the host verifies the region afterwards.
pub trait IndexAccessLoweringProvider: Send + Sync + 'static {
    /// Emits the region structure through the canonical builder.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects an
    /// emission.
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError>;
}

/// Ordered scalar result values one scalar-lowering provider produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarLoweringResults(Vec<ScalarValueId>);

impl ScalarLoweringResults {
    /// Wraps ordered emitted scalar result values.
    #[must_use]
    pub fn new(values: Vec<ScalarValueId>) -> Self {
        Self(values)
    }

    /// Returns the ordered scalar result values.
    #[must_use]
    pub fn values(&self) -> &[ScalarValueId] {
        &self.0
    }

    /// Returns the number of emitted results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether no result was emitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A typed emission failure surfaced to a lowering provider.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoweringEmitError {
    /// The canonical builder rejected an emission.
    Build(IndexBuildError),
}

impl fmt::Display for LoweringEmitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(source) => {
                write!(formatter, "canonical builder rejected emission: {source}")
            }
        }
    }
}

impl Error for LoweringEmitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build(source) => Some(source),
        }
    }
}

impl From<IndexBuildError> for LoweringEmitError {
    fn from(source: IndexBuildError) -> Self {
        Self::Build(source)
    }
}

/// A narrow checked context for one scalar-lowering provider.
///
/// The context delegates to the canonical [`IndexRegionBuilder`] and exposes
/// only pointwise scalar emission over the occurrence's checked operand values.
/// It never exposes the raw builder, region finalization, or a way to construct
/// provider-owned IR.
pub struct ScalarLoweringContext<'a> {
    builder: &'a mut IndexRegionBuilder,
    operands: &'a [ScalarValueId],
    attributes: &'a ScalarAttributes,
    signature: &'a LoweringSignature,
}

impl<'a> ScalarLoweringContext<'a> {
    /// Binds a host-owned scalar-lowering context over a canonical builder.
    ///
    /// The `operands` are the checked scalar values that lower the occurrence's
    /// semantic operands; `attributes` are its host-canonical attributes.
    #[must_use]
    pub fn new(
        builder: &'a mut IndexRegionBuilder,
        operands: &'a [ScalarValueId],
        attributes: &'a ScalarAttributes,
        signature: &'a LoweringSignature,
    ) -> Self {
        Self {
            builder,
            operands,
            attributes,
            signature,
        }
    }

    /// Returns the checked operand scalar values.
    #[must_use]
    pub fn operands(&self) -> &[ScalarValueId] {
        self.operands
    }

    /// Returns the occurrence's host-canonical attributes.
    #[must_use]
    pub fn attributes(&self) -> &ScalarAttributes {
        self.attributes
    }

    /// Returns the occurrence's exact resolved signature.
    #[must_use]
    pub fn signature(&self) -> &LoweringSignature {
        self.signature
    }

    /// Applies one registered scalar operation through the canonical builder.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects the
    /// application.
    pub fn apply(
        &mut self,
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<ScalarResults, LoweringEmitError> {
        Ok(self.builder.apply(key, attributes, operands)?)
    }

    /// Applies one scalar operation in an additional evaluation scope.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects the
    /// application.
    pub fn apply_in(
        &mut self,
        dimensions: &[DimensionId],
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<ScalarResults, LoweringEmitError> {
        Ok(self
            .builder
            .apply_in(dimensions, key, attributes, operands)?)
    }
}

/// A narrow checked context for one index/access-lowering provider.
///
/// The context delegates to the canonical [`IndexRegionBuilder`] and exposes its
/// constructive surface — dimensions, tensor boundaries, index expressions,
/// accesses, scalar applications, reductions, and output roots — but never the
/// raw builder or region finalization. The host verifies the region afterwards.
pub struct IndexAccessLoweringContext<'a> {
    builder: &'a mut IndexRegionBuilder,
}

impl<'a> IndexAccessLoweringContext<'a> {
    /// Binds a host-owned index/access-lowering context over a canonical builder.
    #[must_use]
    pub fn new(builder: &'a mut IndexRegionBuilder) -> Self {
        Self { builder }
    }

    /// Adds one static half-open iteration dimension.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn dimension(
        &mut self,
        role: DomainRole,
        extent: Extent,
    ) -> Result<DimensionId, LoweringEmitError> {
        Ok(self.builder.dimension(role, extent)?)
    }

    /// Declares one input tensor boundary.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn input_tensor(
        &mut self,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Result<TensorId, LoweringEmitError> {
        Ok(self.builder.tensor(TensorRole::Input, value_type, shape)?)
    }

    /// Declares one output tensor boundary.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn output_tensor(
        &mut self,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Result<TensorId, LoweringEmitError> {
        Ok(self.builder.tensor(TensorRole::Output, value_type, shape)?)
    }

    /// Creates or reuses an exact constant index expression.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn constant(&mut self, value: IndexInteger) -> Result<IndexExprId, LoweringEmitError> {
        Ok(self.builder.constant(value)?)
    }

    /// Creates or reuses a dimension index expression.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn dimension_expr(
        &mut self,
        dimension: DimensionId,
    ) -> Result<IndexExprId, LoweringEmitError> {
        Ok(self.builder.dimension_expr(dimension)?)
    }

    /// Creates a normalized affine linear combination.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn linear_combination(
        &mut self,
        constant: IndexInteger,
        terms: &[(IndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, LoweringEmitError> {
        Ok(self.builder.linear_combination(constant, terms)?)
    }

    /// Creates Euclidean floor division by a positive constant.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn floor_div(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
    ) -> Result<IndexExprId, LoweringEmitError> {
        Ok(self.builder.floor_div(dividend, divisor)?)
    }

    /// Creates Euclidean modulo by a positive constant.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn modulo(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
    ) -> Result<IndexExprId, LoweringEmitError> {
        Ok(self.builder.modulo(dividend, divisor)?)
    }

    /// Creates or reuses a read access and its scalar value.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn read(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, LoweringEmitError> {
        Ok(self.builder.read(tensor, domain, coordinates)?)
    }

    /// Creates or reuses a write access.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn write(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<TensorAccessId, LoweringEmitError> {
        Ok(self.builder.write(tensor, domain, coordinates)?)
    }

    /// Applies one registered scalar operation.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn apply(
        &mut self,
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<ScalarResults, LoweringEmitError> {
        Ok(self.builder.apply(key, attributes, operands)?)
    }

    /// Applies one scalar operation in an additional evaluation scope.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn apply_in(
        &mut self,
        dimensions: &[DimensionId],
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<ScalarResults, LoweringEmitError> {
        Ok(self
            .builder
            .apply_in(dimensions, key, attributes, operands)?)
    }

    /// Builds an exact lexicographic left-fold reduction.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects the
    /// reduction or its nested body.
    pub fn reduce<F>(
        &mut self,
        dimensions: &[DimensionId],
        init: &[ScalarValueId],
        contributors: &[ScalarValueId],
        build: F,
    ) -> Result<ScalarResults, LoweringEmitError>
    where
        F: FnOnce(&mut ScalarReducerBodyBuilder<'_>) -> Result<(), IndexBuildError>,
    {
        Ok(self.builder.reduce(dimensions, init, contributors, build)?)
    }

    /// Adds one ordered output root.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn output(
        &mut self,
        access: TensorAccessId,
        value: ScalarValueId,
    ) -> Result<(), LoweringEmitError> {
        Ok(self.builder.output(access, value)?)
    }
}

/// A mutable, single-use constructor for a frozen lowering-capability registry.
///
/// Registration is transactional per call: every declared authority is validated
/// before the capability is retained, so a rejected registration leaves the
/// builder unchanged.
pub struct LoweringCapabilityRegistryBuilder {
    semantic: FrozenSemanticRegistry,
    scalar: FrozenScalarRegistry,
    capabilities: BTreeMap<LoweringCapabilityKey, RegisteredLoweringCapability>,
    canonical_bytes: usize,
}

impl fmt::Debug for LoweringCapabilityRegistryBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoweringCapabilityRegistryBuilder")
            .field("capability_count", &self.capabilities.len())
            .finish_non_exhaustive()
    }
}

impl LoweringCapabilityRegistryBuilder {
    /// Creates an empty builder over exact frozen semantic and scalar authorities.
    #[must_use]
    pub fn new(semantic: FrozenSemanticRegistry, scalar: FrozenScalarRegistry) -> Self {
        let canonical_bytes = REGISTRY_IDENTITY_TAG
            .len()
            .saturating_add(encoded_bytes_len(
                semantic.snapshot_identity().as_bytes().len(),
            ))
            .saturating_add(encoded_bytes_len(
                scalar.snapshot_identity().as_bytes().len(),
            ))
            .saturating_add(size_of::<u64>());
        Self {
            semantic,
            scalar,
            capabilities: BTreeMap::new(),
            canonical_bytes,
        }
    }

    /// Registers one scalar-lowering capability.
    ///
    /// `emitted_scalar_operations` declares the scalar operations the provider
    /// will emit; they become the capability's reached scalar authority and are
    /// validated against the composed scalar registry.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringRegistryError`] for a duplicate capability, an operation
    /// or signature type without semantic authority, an emitted scalar operation
    /// without scalar authority, or an exceeded resource bound.
    pub fn register_scalar_lowering(
        &mut self,
        provider: ProviderIdentity,
        operation: OpKey,
        signature: LoweringSignature,
        emitted_scalar_operations: &[ScalarOpKey],
        revision: LoweringCapabilityRevision,
        implementation: Arc<dyn ScalarLoweringProvider>,
    ) -> Result<(), LoweringRegistryError> {
        self.register(
            provider,
            operation,
            signature,
            emitted_scalar_operations,
            revision,
            LoweringImplementation::ScalarLowering(implementation),
        )
    }

    /// Registers one index/access-lowering capability.
    ///
    /// `emitted_scalar_operations` declares the scalar operations the provider
    /// will emit; they become the capability's reached scalar authority and are
    /// validated against the composed scalar registry.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringRegistryError`] for a duplicate capability, an operation
    /// or signature type without semantic authority, an emitted scalar operation
    /// without scalar authority, or an exceeded resource bound.
    pub fn register_index_access(
        &mut self,
        provider: ProviderIdentity,
        operation: OpKey,
        signature: LoweringSignature,
        emitted_scalar_operations: &[ScalarOpKey],
        revision: LoweringCapabilityRevision,
        implementation: Arc<dyn IndexAccessLoweringProvider>,
    ) -> Result<(), LoweringRegistryError> {
        self.register(
            provider,
            operation,
            signature,
            emitted_scalar_operations,
            revision,
            LoweringImplementation::IndexAccess(implementation),
        )
    }

    fn register(
        &mut self,
        provider: ProviderIdentity,
        operation: OpKey,
        signature: LoweringSignature,
        emitted_scalar_operations: &[ScalarOpKey],
        revision: LoweringCapabilityRevision,
        implementation: LoweringImplementation,
    ) -> Result<(), LoweringRegistryError> {
        let family = implementation.family();
        let key = LoweringCapabilityKey {
            family,
            operation,
            signature,
            provider,
        };
        if self.capabilities.contains_key(&key) {
            return Err(LoweringRegistryError::DuplicateCapability {
                family,
                operation: Box::new(key.operation),
                signature: Box::new(key.signature),
                provider: Box::new(key.provider),
            });
        }
        let count = self.capabilities.len().saturating_add(1);
        if count > MAX_LOWERING_CAPABILITIES {
            return Err(LoweringRegistryError::ResourceExceeded {
                resource: LoweringRegistryResource::Capabilities,
                limit: MAX_LOWERING_CAPABILITIES,
                actual: count,
            });
        }
        if emitted_scalar_operations.len() > MAX_EMITTED_SCALAR_OPERATIONS {
            return Err(LoweringRegistryError::TooManyEmittedOperations {
                actual: emitted_scalar_operations.len(),
            });
        }
        let operation_authority = self
            .semantic
            .project_operation_authority(
                &key.operation,
                key.signature.operands(),
                key.signature.results(),
            )
            .map_err(|source| LoweringRegistryError::OperationAuthority {
                operation: Box::new(key.operation.clone()),
                source: Arc::new(source),
            })?;
        let emitted_scalar_definitions = self
            .scalar
            .project_reached(emitted_scalar_operations.iter())
            .map_err(|source| LoweringRegistryError::ScalarAuthority {
                source: Arc::new(source),
            })?;
        let authority = LoweringCapabilityAuthority {
            operation_authority,
            emitted_scalar_definitions,
        };
        let added = capability_identity_len(&key, &authority, revision);
        let bytes = self.canonical_bytes.saturating_add(added);
        if bytes > MAX_LOWERING_REGISTRY_IDENTITY_BYTES {
            return Err(LoweringRegistryError::ResourceExceeded {
                resource: LoweringRegistryResource::CanonicalIdentityBytes,
                limit: MAX_LOWERING_REGISTRY_IDENTITY_BYTES,
                actual: bytes,
            });
        }
        self.capabilities.insert(
            key,
            RegisteredLoweringCapability {
                revision,
                authority,
                implementation,
            },
        );
        self.canonical_bytes = bytes;
        Ok(())
    }

    /// Freezes an immutable, cheap-clone lowering-capability snapshot.
    #[must_use]
    pub fn freeze(self) -> FrozenLoweringCapabilityRegistry {
        let identity = compute_identity(
            self.semantic.snapshot_identity().as_bytes(),
            self.scalar.snapshot_identity().as_bytes(),
            &self.capabilities,
        );
        debug_assert_eq!(
            identity.0.len(),
            self.canonical_bytes,
            "the running identity byte budget matches the frozen encoding"
        );
        FrozenLoweringCapabilityRegistry(Arc::new(FrozenLoweringCapabilityRegistryData {
            capabilities: self.capabilities,
            identity,
        }))
    }
}

struct FrozenLoweringCapabilityRegistryData {
    capabilities: BTreeMap<LoweringCapabilityKey, RegisteredLoweringCapability>,
    identity: CanonicalLoweringRegistryIdentity,
}

/// An immutable, cheap-clone lowering-capability registry snapshot.
#[derive(Clone)]
pub struct FrozenLoweringCapabilityRegistry(Arc<FrozenLoweringCapabilityRegistryData>);

impl fmt::Debug for FrozenLoweringCapabilityRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenLoweringCapabilityRegistry")
            .field("capability_count", &self.0.capabilities.len())
            .finish()
    }
}

impl FrozenLoweringCapabilityRegistry {
    /// Returns deterministic complete registry provenance.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalLoweringRegistryIdentity {
        &self.0.identity
    }

    /// Returns the number of registered capabilities.
    #[must_use]
    pub fn capability_count(&self) -> usize {
        self.0.capabilities.len()
    }

    /// Resolves the scalar-lowering capability for one exact occurrence.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringResolveError::MissingCapability`] when no capability
    /// applies, or [`LoweringResolveError::AmbiguousCapability`] when more than
    /// one provider claims the occurrence.
    pub fn resolve_scalar_lowering(
        &self,
        operation: &OpKey,
        signature: &LoweringSignature,
    ) -> Result<ResolvedLoweringCapability, LoweringResolveError> {
        self.resolve(LoweringSelector {
            family: LoweringFamily::ScalarLowering,
            operation,
            signature,
        })
    }

    /// Resolves the index/access-lowering capability for one exact occurrence.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringResolveError::MissingCapability`] when no capability
    /// applies, or [`LoweringResolveError::AmbiguousCapability`] when more than
    /// one provider claims the occurrence.
    pub fn resolve_index_access(
        &self,
        operation: &OpKey,
        signature: &LoweringSignature,
    ) -> Result<ResolvedLoweringCapability, LoweringResolveError> {
        self.resolve(LoweringSelector {
            family: LoweringFamily::IndexAccess,
            operation,
            signature,
        })
    }

    fn resolve(
        &self,
        selector: LoweringSelector<'_>,
    ) -> Result<ResolvedLoweringCapability, LoweringResolveError> {
        // The map is ordered by the full key, so matches for one selector are
        // yielded in ascending provider order regardless of registration order.
        let mut matches = self.0.capabilities.iter().filter(|(key, _)| {
            key.family == selector.family
                && &key.operation == selector.operation
                && &key.signature == selector.signature
        });
        let Some((key, capability)) = matches.next() else {
            return Err(LoweringResolveError::MissingCapability {
                family: selector.family,
                operation: Box::new(selector.operation.clone()),
                signature: Box::new(selector.signature.clone()),
            });
        };
        if matches.clone().next().is_some() {
            let mut candidates = vec![key.provider.clone()];
            candidates.extend(matches.map(|(key, _)| key.provider.clone()));
            return Err(LoweringResolveError::AmbiguousCapability {
                family: selector.family,
                operation: Box::new(selector.operation.clone()),
                signature: Box::new(selector.signature.clone()),
                candidates,
            });
        }
        Ok(ResolvedLoweringCapability {
            family: key.family,
            operation: key.operation.clone(),
            signature: key.signature.clone(),
            provider: key.provider.clone(),
            revision: capability.revision,
            authority: capability.authority.clone(),
            implementation: capability.implementation.clone(),
        })
    }
}

/// The complete resolution of one lowering-capability occurrence.
///
/// The family-typed provider handle is the resolved implementation. The
/// registry does not invoke it; a later refinement authority binds an exact
/// occurrence to it and proves the emitted work refines that occurrence.
#[derive(Clone)]
pub struct ResolvedLoweringCapability {
    family: LoweringFamily,
    operation: OpKey,
    signature: LoweringSignature,
    provider: ProviderIdentity,
    revision: LoweringCapabilityRevision,
    authority: LoweringCapabilityAuthority,
    implementation: LoweringImplementation,
}

impl fmt::Debug for ResolvedLoweringCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedLoweringCapability")
            .field("family", &self.family)
            .field("operation", &self.operation)
            .field("signature", &self.signature)
            .field("provider", &self.provider)
            .field("revision", &self.revision)
            .finish_non_exhaustive()
    }
}

impl ResolvedLoweringCapability {
    /// Returns the capability family.
    #[must_use]
    pub const fn family(&self) -> LoweringFamily {
        self.family
    }

    /// Returns the lowered semantic operation family.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the exact resolved signature.
    #[must_use]
    pub const fn signature(&self) -> &LoweringSignature {
        &self.signature
    }

    /// Returns the admitting provider identity.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the output-affecting capability revision.
    #[must_use]
    pub const fn revision(&self) -> LoweringCapabilityRevision {
        self.revision
    }

    /// Returns the reached authority the capability was admitted against.
    #[must_use]
    pub const fn authority(&self) -> &LoweringCapabilityAuthority {
        &self.authority
    }

    /// Returns the scalar-lowering provider, when this is a scalar capability.
    #[must_use]
    pub fn scalar_provider(&self) -> Option<&dyn ScalarLoweringProvider> {
        match &self.implementation {
            LoweringImplementation::ScalarLowering(provider) => Some(provider.as_ref()),
            LoweringImplementation::IndexAccess(_) => None,
        }
    }

    /// Returns the index/access-lowering provider, when this is such a capability.
    #[must_use]
    pub fn index_access_provider(&self) -> Option<&dyn IndexAccessLoweringProvider> {
        match &self.implementation {
            LoweringImplementation::IndexAccess(provider) => Some(provider.as_ref()),
            LoweringImplementation::ScalarLowering(_) => None,
        }
    }
}

/// A governed resource retained by one frozen lowering-capability registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoweringRegistryResource {
    /// Registered capabilities.
    Capabilities,
    /// Canonical registry identity bytes.
    CanonicalIdentityBytes,
}

impl fmt::Display for LoweringRegistryResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Capabilities => "capabilities",
            Self::CanonicalIdentityBytes => "canonical identity bytes",
        })
    }
}

/// A failure to register one lowering capability.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoweringRegistryError {
    /// A signature exceeded the governed operand/result bound.
    SignatureTooLarge {
        /// First rejected type count.
        actual: usize,
    },
    /// A capability declared more emitted scalar operations than admitted.
    TooManyEmittedOperations {
        /// Declared emitted-operation count.
        actual: usize,
    },
    /// The same provider already registered this exact family/operation/signature.
    DuplicateCapability {
        /// Duplicated family.
        family: LoweringFamily,
        /// Duplicated operation.
        operation: Box<OpKey>,
        /// Duplicated signature.
        signature: Box<LoweringSignature>,
        /// Duplicating provider.
        provider: Box<ProviderIdentity>,
    },
    /// The lowered operation or a signature type lacked semantic authority.
    OperationAuthority {
        /// Operation being lowered.
        operation: Box<OpKey>,
        /// Typed semantic-registry cause.
        source: Arc<RegistryError>,
    },
    /// A declared emitted scalar operation lacked scalar authority.
    ScalarAuthority {
        /// Typed scalar-registry cause.
        source: Arc<ScalarRegistryError>,
    },
    /// A registry resource exceeded its governed bound.
    ResourceExceeded {
        /// Bounded resource.
        resource: LoweringRegistryResource,
        /// Active limit.
        limit: usize,
        /// First rejected size.
        actual: usize,
    },
}

impl fmt::Display for LoweringRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignatureTooLarge { actual } => {
                write!(
                    formatter,
                    "lowering signature has {actual} types, exceeding the bound"
                )
            }
            Self::TooManyEmittedOperations { actual } => write!(
                formatter,
                "lowering capability declared {actual} emitted scalar operations, exceeding the bound"
            ),
            Self::DuplicateCapability {
                family, operation, ..
            } => write!(
                formatter,
                "duplicate {family} capability for operation {operation}"
            ),
            Self::OperationAuthority { operation, source } => write!(
                formatter,
                "operation {operation} lacks semantic authority: {source}"
            ),
            Self::ScalarAuthority { source } => {
                write!(
                    formatter,
                    "declared emitted scalar authority failed: {source}"
                )
            }
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "lowering registry {resource} count {actual} exceeds governed limit {limit}"
            ),
        }
    }
}

impl Error for LoweringRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OperationAuthority { source, .. } => Some(source.as_ref()),
            Self::ScalarAuthority { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// A failure to resolve a lowering capability for one occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoweringResolveError {
    /// No registered capability applies to the occurrence.
    MissingCapability {
        /// Requested family.
        family: LoweringFamily,
        /// Requested operation.
        operation: Box<OpKey>,
        /// Requested signature.
        signature: Box<LoweringSignature>,
    },
    /// More than one provider claims the occurrence.
    ///
    /// The candidate providers are listed in canonical ascending order,
    /// independent of registration order.
    AmbiguousCapability {
        /// Requested family.
        family: LoweringFamily,
        /// Requested operation.
        operation: Box<OpKey>,
        /// Requested signature.
        signature: Box<LoweringSignature>,
        /// Contending providers in canonical order.
        candidates: Vec<ProviderIdentity>,
    },
}

impl fmt::Display for LoweringResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCapability {
                family, operation, ..
            } => write!(
                formatter,
                "no {family} capability for operation {operation}"
            ),
            Self::AmbiguousCapability {
                family,
                operation,
                candidates,
                ..
            } => write!(
                formatter,
                "{} providers contend for {family} of operation {operation}",
                candidates.len()
            ),
        }
    }
}

impl Error for LoweringResolveError {}

/// Collision-free canonical provenance for a frozen lowering-capability registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalLoweringRegistryIdentity(Vec<u8>);

impl CanonicalLoweringRegistryIdentity {
    /// Returns the canonical provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

fn compute_identity(
    semantic_snapshot: &[u8],
    scalar_snapshot: &[u8],
    capabilities: &BTreeMap<LoweringCapabilityKey, RegisteredLoweringCapability>,
) -> CanonicalLoweringRegistryIdentity {
    let mut bytes = REGISTRY_IDENTITY_TAG.to_vec();
    encode_bytes(&mut bytes, semantic_snapshot);
    encode_bytes(&mut bytes, scalar_snapshot);
    encode_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        encode_capability(&mut bytes, key, &capability.authority, capability.revision);
    }
    CanonicalLoweringRegistryIdentity(bytes)
}

fn encode_capability(
    output: &mut Vec<u8>,
    key: &LoweringCapabilityKey,
    authority: &LoweringCapabilityAuthority,
    revision: LoweringCapabilityRevision,
) {
    output.push(key.family.tag());
    encode_op_key(output, &key.operation);
    key.signature.encode(output);
    encode_provider(output, &key.provider);
    output.extend_from_slice(&revision.get().to_be_bytes());
    encode_bytes(output, authority.emitted_scalar_definitions.as_bytes());
    encode_bytes(
        output,
        authority
            .operation_authority
            .reached_definitions()
            .as_bytes(),
    );
    encode_bytes(
        output,
        authority
            .operation_authority
            .admission_provenance()
            .as_bytes(),
    );
    encode_bytes(
        output,
        authority.operation_authority.registry_snapshot().as_bytes(),
    );
}

fn capability_identity_len(
    key: &LoweringCapabilityKey,
    authority: &LoweringCapabilityAuthority,
    revision: LoweringCapabilityRevision,
) -> usize {
    let mut scratch = Vec::new();
    encode_capability(&mut scratch, key, authority, revision);
    scratch.len()
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    encode_bytes(output, key.namespace().as_bytes());
    encode_bytes(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    encode_bytes(output, provider.namespace().as_bytes());
    encode_bytes(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

const fn encoded_bytes_len(bytes: usize) -> usize {
    size_of::<u64>().saturating_add(bytes)
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u64::try_from(value).unwrap_or(u64::MAX).to_be_bytes());
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tiler_ir::index::{
        DomainRole, IndexRegionBuilder, ScalarArity, ScalarAttributeField, ScalarAttributeSchema,
        ScalarAttributes, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
        ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
        ScalarOperationInferencer, ScalarRegistryBuilder, ScalarResults, ScalarValueId, TensorRole,
        VerifiedIndexRegion,
    };
    use tiler_ir::semantic::{
        AttributeFieldId, CanonicalValue, CanonicalValueKind, F32, FrozenSemanticRegistry,
        NormativeDefinitionRef, OpKey, ProviderDiagnosticCode, ProviderIdentity, ResolvedValueType,
        multiply_f32_op,
    };
    use tiler_ir::shape::{Extent, Shape};

    use super::{
        FrozenLoweringCapabilityRegistry, IndexAccessLoweringContext, IndexAccessLoweringProvider,
        LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision, LoweringEmitError,
        LoweringFamily, LoweringRegistryError, LoweringResolveError, LoweringSignature,
        ScalarLoweringContext, ScalarLoweringProvider, ScalarLoweringResults,
    };

    const CONSTANT_BITS: AttributeFieldId = AttributeFieldId::new(1);

    fn f32_type() -> ResolvedValueType {
        F32::resolved_type()
    }

    fn scalar_key(name: &str) -> ScalarOpKey {
        ScalarOpKey::new("example", name, 1).unwrap()
    }

    fn provider(name: &str, revision: u32) -> ProviderIdentity {
        ProviderIdentity::new("example", name, revision).unwrap()
    }

    fn revision() -> LoweringCapabilityRevision {
        LoweringCapabilityRevision::new(1).unwrap()
    }

    fn binary_signature() -> LoweringSignature {
        LoweringSignature::new([f32_type(), f32_type()], [f32_type()]).unwrap()
    }

    fn empty_record() -> CanonicalValue {
        CanonicalValue::record([]).unwrap()
    }

    struct FixedF32;
    impl ScalarOperationInferencer for FixedF32 {
        fn infer(
            &self,
            _: ScalarInferenceRequest<'_>,
            outputs: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            outputs.try_push(f32_type())
        }
    }

    struct SameType;
    impl ScalarOperationInferencer for SameType {
        fn infer(
            &self,
            request: ScalarInferenceRequest<'_>,
            outputs: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            let Some(first) = request.operands().first() else {
                return Err(ScalarInferenceError::new(
                    ProviderDiagnosticCode::new("example.arity").unwrap(),
                    "at least one operand is required",
                )
                .unwrap());
            };
            if request.operands().iter().any(|operand| operand != first) {
                return Err(ScalarInferenceError::new(
                    ProviderDiagnosticCode::new("example.type").unwrap(),
                    "operand types differ",
                )
                .unwrap());
            }
            outputs.try_push(first.clone())
        }
    }

    fn scalar_definition(
        name: &str,
        operands: usize,
        attributes: ScalarAttributeSchema,
        inferencer: Arc<dyn ScalarOperationInferencer>,
    ) -> ScalarOperationDefinition {
        ScalarOperationDefinition::new(
            scalar_key(name),
            NormativeDefinitionRef::from_owned(format!("urn:example:{name}:v1")).unwrap(),
            ScalarOperationContract::new(
                attributes,
                ScalarArity::exact(operands).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                empty_record(),
                empty_record(),
            ),
            inferencer,
        )
    }

    fn scalar_registry() -> tiler_ir::index::FrozenScalarRegistry {
        let mut builder = ScalarRegistryBuilder::new(FrozenSemanticRegistry::standard().unwrap());
        let scalars = provider("f32-scalars", 1);
        let constant_schema = ScalarAttributeSchema::new([ScalarAttributeField::required(
            CONSTANT_BITS,
            CanonicalValueKind::FloatBits,
        )])
        .unwrap();
        builder
            .register(
                scalars.clone(),
                scalar_definition("constant", 0, constant_schema, Arc::new(FixedF32)),
            )
            .unwrap();
        for name in ["multiply", "add"] {
            builder
                .register(
                    scalars.clone(),
                    scalar_definition(name, 2, ScalarAttributeSchema::empty(), Arc::new(SameType)),
                )
                .unwrap();
        }
        builder.freeze()
    }

    fn semantic() -> FrozenSemanticRegistry {
        FrozenSemanticRegistry::standard().unwrap()
    }

    fn empty_builder() -> LoweringCapabilityRegistryBuilder {
        LoweringCapabilityRegistryBuilder::new(semantic(), scalar_registry())
    }

    /// Emits `mul(a, b)` for the two checked operand values.
    struct ScalarMultiplyLowering;
    impl ScalarLoweringProvider for ScalarMultiplyLowering {
        fn lower(
            &self,
            context: &mut ScalarLoweringContext<'_>,
        ) -> Result<ScalarLoweringResults, LoweringEmitError> {
            let operands = context.operands().to_vec();
            let product =
                context.apply(scalar_key("multiply"), ScalarAttributes::empty(), &operands)?;
            Ok(ScalarLoweringResults::new(product.iter().collect()))
        }
    }

    /// Emits `out[i] = mul(in[i], in[i])` over a parallel domain of `length`.
    struct PointwiseSquareLowering {
        length: u64,
    }
    impl IndexAccessLoweringProvider for PointwiseSquareLowering {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let shape = Shape::from_dims([self.length]);
            let i = context.dimension(DomainRole::Parallel, Extent::new(self.length))?;
            let input = context.input_tensor(f32_type(), shape.clone())?;
            let output = context.output_tensor(f32_type(), shape)?;
            let row = context.dimension_expr(i)?;
            let value = context.read(input, &[i], &[row])?;
            let product = context.apply(
                scalar_key("multiply"),
                ScalarAttributes::empty(),
                &[value, value],
            )?;
            let squared = squared_result(&product);
            let write = context.write(output, &[i], &[row])?;
            context.output(write, squared)?;
            Ok(())
        }
    }

    fn squared_result(results: &ScalarResults) -> ScalarValueId {
        results
            .get(0)
            .expect("the multiply contract produces exactly one result")
    }

    fn register_both(
        builder: &mut LoweringCapabilityRegistryBuilder,
        scalar_provider: &str,
        index_provider: &str,
    ) {
        builder
            .register_scalar_lowering(
                provider(scalar_provider, 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap();
        builder
            .register_index_access(
                provider(index_provider, 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(PointwiseSquareLowering { length: 4 }),
            )
            .unwrap();
    }

    #[test]
    fn registers_two_families_and_resolves_each_to_its_provider() {
        let mut builder = empty_builder();
        register_both(&mut builder, "scalar-lowering", "index-access-lowering");
        let frozen = builder.freeze();
        assert_eq!(frozen.capability_count(), 2);

        let scalar = frozen
            .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
            .unwrap();
        assert_eq!(scalar.family(), LoweringFamily::ScalarLowering);
        assert_eq!(scalar.provider(), &provider("scalar-lowering", 1));
        assert!(scalar.scalar_provider().is_some());
        assert!(scalar.index_access_provider().is_none());

        let index = frozen
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        assert_eq!(index.family(), LoweringFamily::IndexAccess);
        assert_eq!(index.provider(), &provider("index-access-lowering", 1));
        assert!(index.index_access_provider().is_some());
        assert!(index.scalar_provider().is_none());
    }

    #[test]
    fn snapshot_identity_is_independent_of_registration_order() {
        let mut first = empty_builder();
        first
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap();
        first
            .register_index_access(
                provider("index-access-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(PointwiseSquareLowering { length: 4 }),
            )
            .unwrap();

        let mut second = empty_builder();
        // Reverse registration order.
        second
            .register_index_access(
                provider("index-access-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(PointwiseSquareLowering { length: 4 }),
            )
            .unwrap();
        second
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap();

        assert_eq!(
            first.freeze().canonical_identity(),
            second.freeze().canonical_identity()
        );
    }

    #[test]
    fn duplicate_registration_of_one_provider_is_a_collision() {
        let mut builder = empty_builder();
        builder
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap();
        let error = builder
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            LoweringRegistryError::DuplicateCapability { .. }
        ));
    }

    #[test]
    fn contradictory_providers_resolve_to_a_deterministic_ambiguity() {
        let expected = vec![provider("aardvark", 1), provider("zebra", 1)];
        // Register in each order many times; the ambiguity candidates must stay
        // in canonical ascending provider order regardless.
        for _ in 0..32 {
            for order in [["zebra", "aardvark"], ["aardvark", "zebra"]] {
                let mut builder = empty_builder();
                for name in order {
                    builder
                        .register_scalar_lowering(
                            provider(name, 1),
                            multiply_f32_op(),
                            binary_signature(),
                            &[scalar_key("multiply")],
                            revision(),
                            Arc::new(ScalarMultiplyLowering),
                        )
                        .unwrap();
                }
                let error = builder
                    .freeze()
                    .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
                    .unwrap_err();
                let LoweringResolveError::AmbiguousCapability { candidates, .. } = error else {
                    panic!("expected an ambiguity diagnostic");
                };
                assert_eq!(candidates, expected);
            }
        }
    }

    #[test]
    fn a_missing_capability_resolves_to_a_typed_diagnostic() {
        let frozen = empty_builder().freeze();
        let error = frozen
            .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
            .unwrap_err();
        assert!(matches!(
            error,
            LoweringResolveError::MissingCapability {
                family: LoweringFamily::ScalarLowering,
                ..
            }
        ));
    }

    #[test]
    fn registration_rejects_an_operation_without_semantic_authority() {
        let mut builder = empty_builder();
        let error = builder
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                OpKey::new("example", "not-a-semantic-op", 1).unwrap(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            LoweringRegistryError::OperationAuthority { .. }
        ));
    }

    #[test]
    fn registration_is_transactional_and_leaves_no_partial_state() {
        let mut builder = empty_builder();
        // A declared emitted scalar operation without scalar authority fails.
        let error = builder
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("nonexistent")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            LoweringRegistryError::ScalarAuthority { .. }
        ));
        // The rejected registration retained nothing, so the valid one succeeds
        // and is the only capability.
        builder
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap();
        let frozen = builder.freeze();
        assert_eq!(frozen.capability_count(), 1);
        assert!(
            frozen
                .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
                .is_ok()
        );
    }

    #[test]
    fn capability_revision_participates_in_snapshot_identity() {
        let identity = |capability_revision: u32| {
            let mut builder = empty_builder();
            builder
                .register_scalar_lowering(
                    provider("scalar-lowering", 1),
                    multiply_f32_op(),
                    binary_signature(),
                    &[scalar_key("multiply")],
                    LoweringCapabilityRevision::new(capability_revision).unwrap(),
                    Arc::new(ScalarMultiplyLowering),
                )
                .unwrap();
            builder.freeze().canonical_identity().clone()
        };
        assert_ne!(identity(1), identity(2));
    }

    #[test]
    fn a_resolved_scalar_provider_emits_through_the_canonical_builder() {
        let scalars = scalar_registry();
        let frozen =
            LoweringCapabilityRegistryBuilder::new(semantic(), scalars.clone()).apply_multiply();
        let resolved = frozen
            .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
            .unwrap();

        // Host wiring: read one input value, hand the provider its checked
        // operands, then finish and verify the region.
        let mut builder = IndexRegionBuilder::new(scalars).unwrap();
        let i = builder
            .dimension(DomainRole::Parallel, Extent::new(4))
            .unwrap();
        let input = builder
            .tensor(TensorRole::Input, f32_type(), Shape::from_dims([4]))
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, f32_type(), Shape::from_dims([4]))
            .unwrap();
        let row = builder.dimension_expr(i).unwrap();
        let value = builder.read(input, &[i], &[row]).unwrap();
        let operands = [value, value];
        let attributes = ScalarAttributes::empty();
        let signature = binary_signature();

        let results = {
            let mut context =
                ScalarLoweringContext::new(&mut builder, &operands, &attributes, &signature);
            resolved
                .scalar_provider()
                .expect("scalar family")
                .lower(&mut context)
                .unwrap()
        };
        assert_eq!(results.len(), 1);

        let write = builder.write(output, &[i], &[row]).unwrap();
        builder.output(write, results.values()[0]).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(region.outputs().len(), 1);
        assert_eq!(region.scalar_operations().count(), 1);
    }

    #[test]
    fn a_resolved_index_access_provider_emits_a_verified_region() {
        let scalars = scalar_registry();
        let frozen =
            LoweringCapabilityRegistryBuilder::new(semantic(), scalars.clone()).apply_multiply();
        let resolved = frozen
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();

        let mut builder = IndexRegionBuilder::new(scalars).unwrap();
        {
            let mut context = IndexAccessLoweringContext::new(&mut builder);
            resolved
                .index_access_provider()
                .expect("index-access family")
                .lower(&mut context)
                .unwrap();
        }
        let region: VerifiedIndexRegion = builder.build().unwrap();
        assert_eq!(region.tensors().count(), 2);
        assert_eq!(region.outputs().len(), 1);
        assert_eq!(region.dimensions().count(), 1);
    }

    /// Test-only convenience that registers both demonstration providers.
    impl LoweringCapabilityRegistryBuilder {
        fn apply_multiply(mut self) -> FrozenLoweringCapabilityRegistry {
            register_both(&mut self, "scalar-lowering", "index-access-lowering");
            self.freeze()
        }
    }
}
