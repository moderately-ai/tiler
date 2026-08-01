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

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    CanonicalScalarDefinitionProjection, CanonicalScalarRegistrySnapshotIdentity, DimensionId,
    DomainRole, FrozenScalarRegistry, IndexBuildError, IndexExprId, IndexInteger,
    IndexRegionBuilder, ScalarAttributes, ScalarOpKey, ScalarReducerBodyBuilder,
    ScalarRegistryError, ScalarResults, ScalarValueId, SourcedExtent, SymbolicExtentError,
    TensorAccessId, TensorId, TensorRole,
};
use tiler_ir::semantic::{
    FrozenSemanticRegistry, OpKey, OperationAttributes, ProviderIdentity, RegistryError,
    ResolvedValueType, SemanticCapabilityAuthority, SemanticRegistrySnapshotIdentity,
};
use tiler_ir::shape::{Extent, Shape};

/// Canonical identity domain-separation tag for a frozen registry snapshot.
const REGISTRY_IDENTITY_TAG: &[u8] = b"tiler.compiler.lowering-capability-registry.v2\0";
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
    /// Returns the governed token naming this family inside a capability key.
    ///
    /// Distinct from [`fmt::Display`], which renders prose for diagnostics. A
    /// capability key is durable identity, so its spelling is written by an
    /// exhaustive match here (ADR 0074 convention 3) and a new family is a
    /// build error rather than a silently unnamed one.
    #[must_use]
    pub const fn key_token(self) -> &'static str {
        match self {
            Self::IndexAccess => "index-access",
            Self::ScalarLowering => "scalar",
        }
    }

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
        push_len(output, self.operands.len());
        for value_type in &self.operands {
            push_slice(output, value_type.canonical_encoding().as_bytes());
        }
        push_len(output, self.results.len());
        for value_type in &self.results {
            push_slice(output, value_type.canonical_encoding().as_bytes());
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
    emitted_scalar_operations: Vec<ScalarOpKey>,
    emitted_scalar_definitions: CanonicalScalarDefinitionProjection,
}

impl LoweringCapabilityAuthority {
    /// Returns the semantic authority of the lowered operation occurrence.
    #[must_use]
    pub const fn operation_authority(&self) -> &SemanticCapabilityAuthority {
        &self.operation_authority
    }

    /// Returns the scalar operations the capability declared it may emit.
    ///
    /// The declaration is a *permission*, not an obligation: one capability
    /// lowers every occurrence of its family and signature, and which of the
    /// declared operations a particular occurrence needs depends on that
    /// occurrence's shapes and attributes. A refinement authority therefore
    /// checks that a region reached nothing beyond this set, never that it
    /// reached all of it.
    #[must_use]
    pub fn emitted_scalar_operations(&self) -> &[ScalarOpKey] {
        &self.emitted_scalar_operations
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
    /// The canonical builder refused a sourced extent.
    ///
    /// Separate from [`Self::Build`] because the two name different
    /// authorities: a structural limit says the emitted region grew too large,
    /// while a source refusal says this region's shape environment does not
    /// declare, supply in time, or prove what the extent needs. Folding the
    /// second into the first would report the environment's answer under the
    /// index layer's name.
    Extent(SymbolicExtentError),
    /// The provider refused the occurrence facts it was handed.
    ///
    /// A provider raises this when the occurrence is outside the exact form it
    /// implements — an unsupported broadcast, a missing attribute, an arity it
    /// does not lower — so an unsupported case rejects explicitly instead of
    /// being approximated by the closest region the provider can emit.
    Occurrence {
        /// Stable rule identifier of the refused fact.
        rule: &'static str,
    },
}

impl fmt::Display for LoweringEmitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(source) => {
                write!(formatter, "canonical builder rejected emission: {source}")
            }
            Self::Extent(source) => {
                write!(formatter, "canonical builder refused an extent: {source}")
            }
            Self::Occurrence { rule } => {
                write!(formatter, "provider refused occurrence fact {rule}")
            }
        }
    }
}

impl Error for LoweringEmitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build(source) => Some(source),
            Self::Extent(source) => Some(source),
            Self::Occurrence { .. } => None,
        }
    }
}

impl From<IndexBuildError> for LoweringEmitError {
    fn from(source: IndexBuildError) -> Self {
        Self::Build(source)
    }
}

impl From<SymbolicExtentError> for LoweringEmitError {
    fn from(source: SymbolicExtentError) -> Self {
        Self::Extent(source)
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

/// One ordered value boundary of the occurrence being lowered.
///
/// The boundary is the host's checked description of a semantic value: its
/// resolved element type and its exact static shape. A rank-zero shape is a
/// genuine scalar boundary, not an absent one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceBoundary {
    value_type: ResolvedValueType,
    shape: Shape,
}

impl OccurrenceBoundary {
    /// Binds one boundary element type and static shape.
    #[must_use]
    pub const fn new(value_type: ResolvedValueType, shape: Shape) -> Self {
        Self { value_type, shape }
    }

    /// Returns the boundary element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the boundary static shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// The checked facts about the exact occurrence a provider is lowering.
///
/// An index/access-lowering provider emits a whole region, so unlike a
/// scalar-lowering provider it cannot be handed already-checked operand values.
/// It is handed this instead: the distinct input boundaries it must declare in
/// order, which ordered operand reads each of them, the ordered result
/// boundaries it must write, and the occurrence's host-canonical attributes.
/// Without them a provider could only emit one fixed region, so a governed
/// family would need one registered provider per shape and per attribute value —
/// and two occurrences of one family that differ only in an attribute would be
/// an unresolvable registry ambiguity rather than two lowerings.
///
/// The facts are read-only and provider-independent. They confer no ability to
/// construct provider-owned IR, and the host still proves independently that the
/// emitted region realizes the occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweredOccurrence {
    inputs: Vec<OccurrenceBoundary>,
    operands: Vec<usize>,
    results: Vec<OccurrenceBoundary>,
    attributes: OperationAttributes,
}

impl LoweredOccurrence {
    /// Binds the occurrence facts one index/access lowering may read.
    ///
    /// `inputs` are the distinct operand values in first-occurrence order, which
    /// is the order the emitted region must declare its input boundaries in.
    /// `operands` maps each ordered semantic operand to its position in
    /// `inputs`, so aliased operands repeat one position.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringRegistryError::UnboundOperand`] when an ordered operand
    /// names a position no input boundary occupies.
    pub fn new(
        inputs: Vec<OccurrenceBoundary>,
        operands: Vec<usize>,
        results: Vec<OccurrenceBoundary>,
        attributes: OperationAttributes,
    ) -> Result<Self, LoweringRegistryError> {
        if let Some(operand) = operands.iter().position(|input| *input >= inputs.len()) {
            return Err(LoweringRegistryError::UnboundOperand { operand });
        }
        Ok(Self {
            inputs,
            operands,
            results,
            attributes,
        })
    }

    /// Returns the distinct input boundaries in region-declaration order.
    #[must_use]
    pub fn inputs(&self) -> &[OccurrenceBoundary] {
        &self.inputs
    }

    /// Returns, per ordered semantic operand, its input-boundary position.
    #[must_use]
    pub fn operands(&self) -> &[usize] {
        &self.operands
    }

    /// Returns the ordered result boundaries.
    #[must_use]
    pub fn results(&self) -> &[OccurrenceBoundary] {
        &self.results
    }

    /// Returns the occurrence's host-canonical attributes.
    #[must_use]
    pub const fn attributes(&self) -> &OperationAttributes {
        &self.attributes
    }
}

/// A narrow checked context for one index/access-lowering provider.
///
/// The context delegates to the canonical [`IndexRegionBuilder`] and exposes its
/// constructive surface — dimensions, tensor boundaries, index expressions,
/// accesses, scalar applications, reductions, and output roots — plus the
/// checked [`LoweredOccurrence`] facts, but never the raw builder or region
/// finalization. The host verifies the region afterwards.
pub struct IndexAccessLoweringContext<'a> {
    builder: &'a mut IndexRegionBuilder,
    occurrence: &'a LoweredOccurrence,
}

impl<'a> IndexAccessLoweringContext<'a> {
    /// Binds a host-owned index/access-lowering context over a canonical builder.
    #[must_use]
    pub fn new(builder: &'a mut IndexRegionBuilder, occurrence: &'a LoweredOccurrence) -> Self {
        Self {
            builder,
            occurrence,
        }
    }

    /// Returns the checked facts about the occurrence being lowered.
    #[must_use]
    pub const fn occurrence(&self) -> &LoweredOccurrence {
        self.occurrence
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

    /// Creates Euclidean floor division by a proven-positive extent.
    ///
    /// The divisor is the canonical index vocabulary rather than a bare integer,
    /// so a provider spells a literal as [`SourcedExtent::Static`]. A symbolic
    /// divisor is refused for as long as this context's region is built without
    /// a shape environment — as a
    /// [`tiler_ir::index::ExtentSourceError::UndeclaredSymbol`], which is
    /// exactly what it is here — rather than being unrepresentable and so
    /// unable to say why.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn floor_div(
        &mut self,
        dividend: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, LoweringEmitError> {
        Ok(self.builder.floor_div(dividend, divisor)?)
    }

    /// Creates Euclidean modulo by a proven-positive extent.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringEmitError`] when the canonical builder rejects it.
    pub fn modulo(
        &mut self,
        dividend: IndexExprId,
        divisor: SourcedExtent,
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
            .saturating_add(size_of::<u64>())
            // The interned pool's count prefix, written once for the registry.
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
        // The governed capability key names family, operation, and operation
        // version; a consumer records the provider beside it. That pair is a
        // complete name only while it determines one signature, so a second
        // signature from one provider for one family and operation is refused
        // here rather than allowed to mint a key already in use. Two *different*
        // providers may still register different signatures: the recorded
        // provider tells those apart, and ADR 0072 requires that contended case
        // to reach a deterministic resolution ambiguity rather than a
        // registration failure.
        //
        // Scanned rather than indexed because the map is keyed by the whole
        // four-tuple with the signature ahead of the provider, and because the
        // registry is bounded by `MAX_LOWERING_CAPABILITIES` and built once.
        if let Some(registered) = self.capabilities.keys().find(|existing| {
            existing.family == key.family
                && existing.operation == key.operation
                && existing.provider == key.provider
                && existing.signature != key.signature
        }) {
            return Err(LoweringRegistryError::ConflatedCapabilityKey {
                family,
                operation: Box::new(key.operation),
                provider: Box::new(key.provider),
                registered: Box::new(registered.signature.clone()),
                rejected: Box::new(key.signature),
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
        let mut declared = emitted_scalar_operations.to_vec();
        declared.sort_unstable();
        declared.dedup();
        let authority = LoweringCapabilityAuthority {
            operation_authority,
            emitted_scalar_operations: declared,
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
        // A bound, not an equality, since the encoding interns the authority
        // identities that capabilities share and the running total counts each
        // capability's in full. The direction is the one that matters: the
        // budget checked on every insert can only be *larger* than what is
        // written, so a registry admitted by the budget always encodes within
        // it. An encoding that exceeded the running total would mean the
        // accounting had stopped covering some part of the identity, which is
        // what this catches.
        debug_assert!(
            identity.0.len() <= self.canonical_bytes,
            "the frozen encoding is {} bytes, above the running budget of {}",
            identity.0.len(),
            self.canonical_bytes,
        );
        FrozenLoweringCapabilityRegistry(Arc::new(FrozenLoweringCapabilityRegistryData {
            capabilities: self.capabilities,
            semantic_snapshot: self.semantic.snapshot_identity().clone(),
            scalar_snapshot: self.scalar.snapshot_identity().clone(),
            identity,
        }))
    }
}

struct FrozenLoweringCapabilityRegistryData {
    capabilities: BTreeMap<LoweringCapabilityKey, RegisteredLoweringCapability>,
    semantic_snapshot: SemanticRegistrySnapshotIdentity,
    scalar_snapshot: CanonicalScalarRegistrySnapshotIdentity,
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

    /// Returns the exact semantic-registry snapshot capabilities were admitted
    /// against.
    #[must_use]
    pub fn semantic_snapshot(&self) -> &SemanticRegistrySnapshotIdentity {
        &self.0.semantic_snapshot
    }

    /// Returns the exact scalar-registry snapshot capabilities were admitted
    /// against.
    ///
    /// A host that drives a resolved provider must build and revalidate the
    /// emitted region under this exact snapshot; pairing a registry with any
    /// other scalar authority is a request error rather than a lowering failure.
    #[must_use]
    pub fn scalar_snapshot(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        &self.0.scalar_snapshot
    }

    /// Returns the distinct admitting providers in canonical ascending order.
    #[must_use]
    pub fn providers(&self) -> Vec<ProviderIdentity> {
        let mut providers: Vec<_> = self
            .0
            .capabilities
            .keys()
            .map(|key| key.provider.clone())
            .collect();
        providers.sort_unstable();
        providers.dedup();
        providers
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
    /// An ordered operand named an input boundary the occurrence does not have.
    UnboundOperand {
        /// Ordered operand position.
        operand: usize,
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
    /// One provider registered a second signature for one family and operation.
    ///
    /// The governed capability key `tiler.capability.<family>.<namespace>.<name>.v<version>`
    /// (`lowering.rs`) deliberately excludes the resolved signature, and every
    /// consumer records the *provider* beside it, so a key and a provider name
    /// one capability exactly as long as that pair determines one signature.
    /// This rejection is what keeps that true: without it a second signature
    /// would mint a key already in use and two capabilities would become
    /// indistinguishable in artifact identity, silently.
    ///
    /// It restricts what a provider may register, and that restriction is the
    /// point. Per-shape or per-attribute signatures for one operation family are
    /// a reasonable thing to want; they are refused here so that admitting them
    /// is a decision someone makes about the key's encoding rather than a
    /// property that quietly stops holding.
    ConflatedCapabilityKey {
        /// Family both registrations share.
        family: LoweringFamily,
        /// Operation both registrations share.
        operation: Box<OpKey>,
        /// Provider registering both.
        provider: Box<ProviderIdentity>,
        /// Signature already registered for that family and operation.
        registered: Box<LoweringSignature>,
        /// Signature this registration attempted to add.
        rejected: Box<LoweringSignature>,
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
            Self::UnboundOperand { operand } => write!(
                formatter,
                "operand {operand} names an input boundary the occurrence does not have"
            ),
            Self::DuplicateCapability {
                family, operation, ..
            } => write!(
                formatter,
                "duplicate {family} capability for operation {operation}"
            ),
            Self::ConflatedCapabilityKey {
                family,
                operation,
                provider,
                ..
            } => write!(
                formatter,
                "provider {provider} already registered a different {family} signature for operation {operation}; the governed capability key cannot distinguish them"
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

/// Encodes the registry's canonical identity with its sub-identities interned.
///
/// **Why a pool and not an inline copy.** Every capability names four authority
/// identities, and capabilities registered against one registry overwhelmingly
/// name the *same* ones — the governed profile's five capabilities restated a
/// single 1,496-byte registry snapshot five times, 7,480 bytes of a 15,583-byte
/// identity, which in turn was 77% of the explain subject that is hashed once
/// per compilation and compared on every evidence binding. Writing each distinct
/// value once and referring to it by position is the same shape
/// `tiler_ir::semantic::identity::compute_graph_identity` already uses, and the
/// same one that took kernel-program identity from 13,309 bytes to 3,118.
///
/// **Injectivity is preserved.** The pool is written in ascending byte order, is
/// count-prefixed, and is complete before any capability refers to it, so a
/// fixed-width position determines its referent exactly as an inline copy did.
/// Two registries differing in any sub-identity differ in the pool; two
/// differing only in which capability names which differ in the positions.
fn compute_identity(
    semantic_snapshot: &[u8],
    scalar_snapshot: &[u8],
    capabilities: &BTreeMap<LoweringCapabilityKey, RegisteredLoweringCapability>,
) -> CanonicalLoweringRegistryIdentity {
    let mut pool: BTreeSet<&[u8]> = BTreeSet::new();
    for capability in capabilities.values() {
        for blob in authority_identities(&capability.authority) {
            pool.insert(blob);
        }
    }
    let pooled: Vec<&[u8]> = pool.into_iter().collect();

    let mut bytes = REGISTRY_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, semantic_snapshot);
    push_slice(&mut bytes, scalar_snapshot);
    push_len(&mut bytes, pooled.len());
    for blob in &pooled {
        push_slice(&mut bytes, blob);
    }
    push_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        encode_capability_key(&mut bytes, key, capability.revision);
        for blob in authority_identities(&capability.authority) {
            let position = pooled
                .binary_search(&blob)
                .expect("every authority identity was pooled above");
            bytes.extend_from_slice(&(position as u64).to_be_bytes());
        }
    }
    CanonicalLoweringRegistryIdentity(bytes)
}

/// The four authority identities one capability names, in their encoding order.
///
/// One function so the pool and the reference list cannot disagree about which
/// values exist or what order they are written in; two parallel lists here would
/// be a defect a later edit could introduce silently.
fn authority_identities(authority: &LoweringCapabilityAuthority) -> [&[u8]; 4] {
    [
        authority.emitted_scalar_definitions.as_bytes(),
        authority
            .operation_authority
            .reached_definitions()
            .as_bytes(),
        authority
            .operation_authority
            .admission_provenance()
            .as_bytes(),
        authority.operation_authority.registry_snapshot().as_bytes(),
    ]
}

/// Appends the part of a capability that is its own, not a shared authority.
fn encode_capability_key(
    output: &mut Vec<u8>,
    key: &LoweringCapabilityKey,
    revision: LoweringCapabilityRevision,
) {
    output.push(key.family.tag());
    encode_op_key(output, &key.operation);
    key.signature.encode(output);
    encode_provider(output, &key.provider);
    output.extend_from_slice(&revision.get().to_be_bytes());
}

/// A conservative upper bound on what one capability adds to the identity.
///
/// It measures the *un-interned* encoding — the four authority identities in
/// full rather than as pooled positions — so it over-counts whenever a
/// capability shares an authority with one already registered, which is the
/// common case. That is deliberate: the bound is checked as capabilities are
/// added, one at a time, and whether a value will end up shared is not known
/// until the registry is closed. Over-counting rejects a registry slightly
/// earlier than the true encoding would require, which fails closed; the
/// alternative of accounting the pooled size would have to revise every
/// previous capability's contribution on each insert.
fn capability_identity_len(
    key: &LoweringCapabilityKey,
    authority: &LoweringCapabilityAuthority,
    revision: LoweringCapabilityRevision,
) -> usize {
    let mut scratch = Vec::new();
    encode_capability_key(&mut scratch, key, revision);
    // The pool contribution, bounded by this capability's own identities written
    // in full: interning can only make the pool smaller, never larger.
    let pooled: usize = authority_identities(authority)
        .iter()
        .map(|blob| encoded_bytes_len(blob.len()))
        .sum();
    // Plus the four fixed-width positions this capability writes, which it pays
    // whether or not its identities turn out to be shared with another's.
    let references = size_of::<u64>() * authority_identities(authority).len();
    scratch.len() + pooled + references
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

const fn encoded_bytes_len(bytes: usize) -> usize {
    size_of::<u64>().saturating_add(bytes)
}

/// Registers the lowering capabilities this build ships onto `builder`, skipping
/// the operation families named in `substituted`.
///
/// The composition an external provider needs: register your own capability for
/// one family and take Tiler's for the rest, in one registry that resolves them
/// all. Pass an empty slice to install all four.
///
/// Without this, an out-of-crate caller substituting one family had to
/// re-implement the other three, because the shipped descriptors were
/// crate-private — which is why the conformance case that exercises exactly this
/// composition could only live inside the compiler.
///
/// # Errors
///
/// Returns [`LoweringRegistryError`] when the builder refuses a registration —
/// in practice, when `builder` was composed over a different scalar authority
/// than the governed capabilities were written against.
pub fn install_governed_index_access(
    builder: &mut LoweringCapabilityRegistryBuilder,
    substituted: &[OpKey],
) -> Result<(), LoweringRegistryError> {
    crate::governed::install_governed_index_access(builder, substituted)
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
        LoweredOccurrence, LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision,
        LoweringEmitError, LoweringFamily, LoweringRegistryError, LoweringResolveError,
        LoweringSignature, OccurrenceBoundary, ScalarLoweringContext, ScalarLoweringProvider,
        ScalarLoweringResults,
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

    /// A second signature over the same authority as [`binary_signature`].
    ///
    /// It reaches exactly the same types and operation, so it passes the same
    /// authority projection; only the operand count differs. That is what makes
    /// it the right probe for a key that excludes the signature.
    fn unary_signature() -> LoweringSignature {
        LoweringSignature::new([f32_type()], [f32_type()]).unwrap()
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
        // Ad-hoc: an `example` scalar namespace on purpose. The subject is capability
        // resolution and provider identity, not the governed vocabulary, and binding
        // these fixtures to `tiler.scalar::*` would make a change to the governed
        // profile's arity or attributes break tests that are not about it.
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

    /// Emits `out[i] = mul(in[i], in[i])` over the occurrence's own extent.
    ///
    /// The provider reads the length from the occurrence facts rather than a
    /// registration-time constant, so one registered capability lowers every
    /// length instead of colliding with a sibling registered for another.
    struct PointwiseSquareLowering;
    impl IndexAccessLoweringProvider for PointwiseSquareLowering {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let shape = context.occurrence().results()[0].shape().clone();
            let length = shape.extents()[0].get();
            let i = context.dimension(DomainRole::Parallel, Extent::new(length))?;
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

    fn square_occurrence(length: u64) -> LoweredOccurrence {
        let shape = Shape::from_dims([length]);
        LoweredOccurrence::new(
            vec![OccurrenceBoundary::new(f32_type(), shape.clone())],
            vec![0, 0],
            vec![OccurrenceBoundary::new(f32_type(), shape)],
            tiler_ir::semantic::OperationAttributes::empty(),
        )
        .unwrap()
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
                Arc::new(PointwiseSquareLowering),
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
                Arc::new(PointwiseSquareLowering),
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
                Arc::new(PointwiseSquareLowering),
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

    /// The registration boundary does not constrain one operation to one
    /// signature, so the governed key's conflation is reachable, not hypothetical.
    ///
    /// `register` validates a signature by projecting the *authority* its types
    /// and operation transitively reach — `project_operation_authority` closes
    /// over the named types and the named operation and fails when one of them
    /// is absent from the semantic registry. It does not compare the signature
    /// against the operation's own arity or type contract, so a unary `f32`
    /// signature for a binary `f32` multiply registers.
    ///
    /// That is the fact `resolve-capability-key-signature-conflation` recorded
    /// as an inference in the opposite direction: the *governed* profile happens
    /// to register one signature per operation, but nothing at this boundary
    /// makes that so, and an externally registered provider reaches the second
    /// signature today. The guard below is what keeps the exclusion of the
    /// signature from the governed key safe.
    #[test]
    fn one_operation_admits_more_than_one_registrable_signature() {
        let mut builder = empty_builder();
        builder
            .register_scalar_lowering(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                unary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .expect("a unary signature for a binary operation registers");
    }

    /// One provider may register one signature per family and operation.
    ///
    /// The governed capability key `tiler.capability.<family>.<ns>.<name>.v<v>`
    /// excludes the signature, and consumers record the provider beside it, so
    /// the pair names one capability only while this holds. Without the guard
    /// the second registration would mint a key already in use and the two
    /// capabilities would be indistinguishable in artifact identity — with no
    /// diagnostic, which is the failure mode the ticket exists to remove.
    ///
    /// The guard is deliberately scoped to one provider. Two providers claiming
    /// one operation with different signatures still register, because the
    /// recorded provider distinguishes them and ADR 0072 requires a contended
    /// claim to reach a deterministic resolution ambiguity rather than a
    /// registration failure.
    #[test]
    fn a_second_signature_for_one_family_and_operation_is_refused() {
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
                unary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .unwrap_err();
        assert_eq!(
            error,
            LoweringRegistryError::ConflatedCapabilityKey {
                family: LoweringFamily::ScalarLowering,
                operation: Box::new(multiply_f32_op()),
                provider: Box::new(provider("scalar-lowering", 1)),
                registered: Box::new(binary_signature()),
                rejected: Box::new(unary_signature()),
            }
        );

        // A different provider registering the second signature is admitted:
        // the recorded provider is what tells the two keys apart.
        builder
            .register_scalar_lowering(
                provider("other-scalar-lowering", 1),
                multiply_f32_op(),
                unary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiplyLowering),
            )
            .expect("the guard is per provider, not per operation");

        // So is a second *family* for one operation and provider: the family is
        // in the key, so those two capabilities are already distinguishable.
        builder
            .register_index_access(
                provider("scalar-lowering", 1),
                multiply_f32_op(),
                unary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(PointwiseSquareLowering),
            )
            .expect("the guard is per family, and the family is in the key");
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

    /// Two revisions of one provider are an ambiguity, not a version choice.
    ///
    /// `LoweringCapabilityKey` carries the whole `ProviderIdentity`, and that
    /// identity is `{namespace, name, revision}` — so two registrations differing
    /// only in revision are two *distinct* keys and both insert.
    /// `DuplicateCapability` fires on an exact key repeat and never here. Both
    /// then match one selector, because `resolve` filters on family, operation,
    /// and signature and not on provider, so the result is an ambiguity listing
    /// both.
    ///
    /// **The registry has no newer-wins rule and no supersession**, and this test
    /// is what says so. Sibling coverage does not, which was measured rather than
    /// assumed: simulating newer-wins inside `resolve` leaves
    /// `contradictory_providers_resolve_to_a_deterministic_ambiguity` and
    /// `duplicate_registration_of_one_provider_is_a_collision` both green and
    /// fails only this test. One registers two provider *names* at a single
    /// revision; the other re-registers an identical key.
    #[test]
    fn two_revisions_of_one_provider_resolve_to_an_ambiguity() {
        let expected = vec![provider("aardvark", 1), provider("aardvark", 2)];
        // Both registration orders, so a newer-wins rule cannot hide behind the
        // order the revisions happened to arrive in.
        for order in [[1, 2], [2, 1]] {
            let mut builder = empty_builder();
            for revision_number in order {
                builder
                    .register_scalar_lowering(
                        provider("aardvark", revision_number),
                        multiply_f32_op(),
                        binary_signature(),
                        &[scalar_key("multiply")],
                        revision(),
                        Arc::new(ScalarMultiplyLowering),
                    )
                    .expect("a second revision of one provider is a distinct key, so it inserts");
            }
            let error = builder
                .freeze()
                .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
                .unwrap_err();
            let LoweringResolveError::AmbiguousCapability { candidates, .. } = error else {
                panic!("expected an ambiguity diagnostic, not a resolution");
            };
            assert_eq!(
                candidates, expected,
                "candidates must be both identities in canonical ascending order"
            );
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

    /// A contraction resolves at its own binary signature and nowhere else.
    ///
    /// ADR 0087 item 4 requires a contraction no installed capability covers to
    /// be a typed resolution refusal rather than a silent approximation. The
    /// governed registry now ships one — `[f32, f32] -> [f32]`, the family's own
    /// arity — so the refusal this test asserts moved from "no contraction at
    /// all" to "not at a signature the family does not have". Both halves are
    /// checked, because the resolution alone would also be satisfied by a
    /// registry that answered for every signature it was asked about.
    #[test]
    fn a_contraction_resolves_only_at_its_registered_binary_signature() {
        let installed = crate::request::CompilerCapabilitySnapshot::governed();
        let contraction = OpKey::new("tiler", "strict-tensor-contraction-f32", 1).unwrap();
        assert!(
            installed
                .lowering()
                .resolve_index_access(&contraction, &binary_signature())
                .is_ok()
        );
        let error = installed
            .lowering()
            .resolve_index_access(&contraction, &unary_signature())
            .unwrap_err();
        assert!(matches!(
            error,
            LoweringResolveError::MissingCapability {
                family: LoweringFamily::IndexAccess,
                ..
            }
        ));
        assert_eq!(
            error.to_string(),
            "no index-access lowering capability for operation tiler::strict-tensor-contraction-f32@1"
        );
        // A family the build registers no capability for at all still refuses,
        // so the resolution above is a decision about what is installed rather
        // than a registry that answers for anything it is handed.
        let unregistered = OpKey::new("tiler", "rms-norm-f32", 1).unwrap();
        assert!(
            installed
                .lowering()
                .resolve_index_access(&unregistered, &unary_signature())
                .is_err()
        );
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
        let occurrence = square_occurrence(4);
        {
            let mut context = IndexAccessLoweringContext::new(&mut builder, &occurrence);
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
