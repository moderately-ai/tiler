//! Draft index-region refinement authority (compiler-owned legality evidence).
//!
//! This module implements the checked authority that `docs/ir.md` Layer 2 calls
//! *compiler-owned legality evidence* and that `docs/compiler/optimizer.md` names
//! the `LowerIndexRegions` boundary: it binds a lowered
//! [`VerifiedIndexRegion`] to the exact
//! semantic occurrence it realizes and records the reached scalar-definition and
//! lowering-provider provenance that compilation and artifact identity require.
//!
//! It composes two sibling authorities without collapsing either:
//!
//! - the [`crate::capability`] registry resolves *which* provider lowers an
//!   occurrence and drives it through the canonical `tiler-ir` builders; and
//! - the crate-internal `region` formation stage separates reusable region
//!   *content* identity from a graph *occurrence* identity.
//!
//! Refinement mirrors that same discipline. It keeps reusable
//! [`RefinementContent`] — the structural region identity, the ordered value
//! interface, the numerical/effect evidence, and the provider-independent reached
//! definitions — distinct from the occurrence binding that pins the exact
//! semantic source, the selected-provider provenance, and the ordered
//! value/access bindings.
//!
//! The load-bearing invariant is that *registration or a successful builder
//! construction alone is not refinement evidence*. The structural index verifier
//! proves a region is internally well formed; it does not establish that the
//! region implements any semantic operation. Refinement independently proves the
//! emitted region *realizes the occurrence*: the ordered operand and result
//! interface (type, shape, arity, aliasing) agrees, the reached scalar authority
//! stays inside the authority the capability was admitted to emit, the semantic
//! type authorities of the capability and the region agree, and every ordinary
//! write carries complete unique-ownership evidence. Matching shapes, dtypes, or
//! operation names never *substitute* for that binding; they are checked as part
//! of it.
//!
//! Refinement does not re-derive per-point arithmetic. Its structural and
//! authority binding is exactly what makes the region *checkable* against the
//! independent `tiler-reference` index-region oracle: the oracle can execute the
//! refined region on concrete inputs bound through [`IndexRefinement::operand_bindings`]
//! and its outputs are the occurrence's ordered results.
//!
//! Scope boundary: this authority proves refinement of one occurrence to one
//! index region. It selects no cover, chooses no physical implementation,
//! schedules nothing, and costs nothing. It refines only the
//! [`LoweringFamily::IndexAccess`] family, because only that family emits a
//! standalone region; a scalar-lowering capability is rejected explicitly.
//!
//! Every public item here is a reviewed *draft* boundary. It is not a stable
//! compiler API and must not be treated as one until Tom accepts the exact
//! interface.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    CanonicalIndexRegionIdentity, FrozenScalarRegistry, IndexRegionBuildError,
    IndexRegionDiagnostic, ScalarAuthorityEvidence, ScalarRegistryError, TensorRole,
    UnknownIndexDomainPredicate, VerifiedIndexHandleError, VerifiedIndexRegion,
    VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
};
use tiler_ir::semantic::{
    OpKey, OperationAttributes, OperationEffect, ProviderIdentity, ResolvedValueType,
};
use tiler_ir::shape::Shape;

use crate::capability::{
    IndexAccessLoweringContext, LoweredOccurrence, LoweringCapabilityAuthority,
    LoweringCapabilityRevision, LoweringEmitError, LoweringFamily, LoweringRegistryError,
    OccurrenceBoundary, ResolvedLoweringCapability,
};
use crate::index_discharge::AuthorizedIndexDomainProof;

/// Canonical domain-separation tag for reusable refinement content.
const CONTENT_IDENTITY_TAG: &[u8] = b"tiler.compiler.index-refinement-content.v2\0";
/// Canonical domain-separation tag for one refinement occurrence binding.
const OCCURRENCE_IDENTITY_TAG: &[u8] = b"tiler.compiler.index-refinement-occurrence.v1\0";

/// Occurrence-local identity of one semantic value the occurrence references.
///
/// Two operands that carry the same value are aliases of one semantic value and
/// therefore lower to one region input boundary. The concrete integer is only an
/// occurrence-local name; reusable content canonicalizes it to a first-occurrence
/// position so aliasing structure is content while the naming is not.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrenceValueId(pub u32);

/// One ordered operand of a semantic occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceOperand {
    value: OccurrenceValueId,
    value_type: ResolvedValueType,
    shape: Shape,
}

impl OccurrenceOperand {
    /// Binds one ordered operand value, its element type, and its boundary shape.
    #[must_use]
    pub const fn new(
        value: OccurrenceValueId,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Self {
        Self {
            value,
            value_type,
            shape,
        }
    }

    /// Returns the semantic value this operand references.
    #[must_use]
    pub const fn value(&self) -> OccurrenceValueId {
        self.value
    }

    /// Returns the operand element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the operand boundary shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// One ordered result of a semantic occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceResult {
    value_type: ResolvedValueType,
    shape: Shape,
}

impl OccurrenceResult {
    /// Binds one ordered result element type and boundary shape.
    #[must_use]
    pub const fn new(value_type: ResolvedValueType, shape: Shape) -> Self {
        Self { value_type, shape }
    }

    /// Returns the result element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the result boundary shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Opaque collision-free identity of the exact semantic source being lowered.
///
/// The caller supplies the semantic occurrence identity produced by region
/// formation (or any collision-free semantic-source identity). Refinement treats
/// it as opaque bytes: it is the *selected semantic source* the region is bound
/// to, never re-derived here.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticOccurrenceIdentity(Vec<u8>);

impl SemanticOccurrenceIdentity {
    /// Wraps opaque collision-free semantic-source identity bytes.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the opaque semantic-source identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Canonical identity of the numerical contract an occurrence is lowered under.
///
/// Refinement binds the contract as evidence; it does not itself re-check
/// numerical policy. The exact per-scalar numerical behaviour is pinned by the
/// bound [`ScalarAuthorityEvidence`], which names the exact scalar definitions
/// and their admission providers.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NumericalContractIdentity(Vec<u8>);

impl NumericalContractIdentity {
    /// Identifies the numerical contract by its canonical key.
    #[must_use]
    pub fn from_key(key: &str) -> Self {
        Self(key.as_bytes().to_vec())
    }

    /// Returns the canonical numerical-contract identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// The exact semantic occurrence one index region is refined against.
///
/// It describes the occurrence independently of the region: the operation, the
/// ordered operand values with element type and shape (operands may alias), the
/// ordered result element types and shapes, the host-canonical attributes, the
/// observable effect, the numerical contract, and the opaque semantic-source
/// identity. Refinement then proves an emitted region realizes it rather than
/// trusting that a provider ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticOccurrence {
    operation: OpKey,
    operands: Vec<OccurrenceOperand>,
    results: Vec<OccurrenceResult>,
    attributes: OperationAttributes,
    effect: OperationEffect,
    numerical_contract: NumericalContractIdentity,
    identity: SemanticOccurrenceIdentity,
}

impl SemanticOccurrence {
    /// Describes one semantic occurrence to refine an index region against.
    #[must_use]
    pub fn new(
        operation: OpKey,
        operands: Vec<OccurrenceOperand>,
        results: Vec<OccurrenceResult>,
        attributes: OperationAttributes,
        effect: OperationEffect,
        numerical_contract: NumericalContractIdentity,
        identity: SemanticOccurrenceIdentity,
    ) -> Self {
        Self {
            operation,
            operands,
            results,
            attributes,
            effect,
            numerical_contract,
            identity,
        }
    }

    /// Returns the lowered semantic operation family key.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the occurrence's host-canonical attributes.
    #[must_use]
    pub const fn attributes(&self) -> &OperationAttributes {
        &self.attributes
    }

    /// Returns the ordered operands, including aliased repetitions.
    #[must_use]
    pub fn operands(&self) -> &[OccurrenceOperand] {
        &self.operands
    }

    /// Returns the ordered results.
    #[must_use]
    pub fn results(&self) -> &[OccurrenceResult] {
        &self.results
    }

    /// Returns the observable effect class.
    #[must_use]
    pub const fn effect(&self) -> OperationEffect {
        self.effect
    }

    /// Returns the bound numerical-contract identity.
    #[must_use]
    pub const fn numerical_contract(&self) -> &NumericalContractIdentity {
        &self.numerical_contract
    }

    /// Returns the opaque semantic-source identity.
    #[must_use]
    pub const fn identity(&self) -> &SemanticOccurrenceIdentity {
        &self.identity
    }

    /// Returns the ordered operand element types.
    fn operand_types(&self) -> Vec<ResolvedValueType> {
        self.operands
            .iter()
            .map(|operand| operand.value_type.clone())
            .collect()
    }

    /// Returns the ordered result element types.
    fn result_types(&self) -> Vec<ResolvedValueType> {
        self.results
            .iter()
            .map(|result| result.value_type.clone())
            .collect()
    }

    /// Returns the distinct operand values in first-occurrence order.
    ///
    /// # Errors
    ///
    /// Returns [`RefinementError::AliasedOperandInconsistent`] when two operands
    /// share a value identity but disagree on element type or shape, which is an
    /// internally inconsistent occurrence.
    fn distinct_operands(&self) -> Result<Vec<&OccurrenceOperand>, RefinementError> {
        let mut distinct: Vec<&OccurrenceOperand> = Vec::new();
        for (position, operand) in self.operands.iter().enumerate() {
            if let Some(seen) = distinct
                .iter()
                .find(|candidate| candidate.value == operand.value)
            {
                if seen.value_type != operand.value_type || seen.shape != operand.shape {
                    return Err(RefinementError::AliasedOperandInconsistent { operand: position });
                }
            } else {
                distinct.push(operand);
            }
        }
        Ok(distinct)
    }
}

/// One ordered operand value bound to the region input boundary that carries it.
///
/// Aliased operands share one `input_tensor`. The binding is a runtime handle
/// for the host to feed the reference oracle; it is deliberately not part of any
/// canonical identity, because verified handles are transient lookup
/// capabilities rather than stable identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperandBinding {
    operand: usize,
    value: OccurrenceValueId,
    input_tensor: VerifiedTensorId,
}

impl OperandBinding {
    /// Returns the ordered operand position.
    #[must_use]
    pub const fn operand(&self) -> usize {
        self.operand
    }

    /// Returns the semantic value this operand references.
    #[must_use]
    pub const fn value(&self) -> OccurrenceValueId {
        self.value
    }

    /// Returns the region input boundary that realizes this operand.
    #[must_use]
    pub const fn input_tensor(&self) -> VerifiedTensorId {
        self.input_tensor
    }
}

/// One ordered result value bound to the region output root that produces it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResultBinding {
    result: usize,
    output_tensor: VerifiedTensorId,
    write_access: VerifiedTensorAccessId,
    written_value: VerifiedScalarValueId,
}

impl ResultBinding {
    /// Returns the ordered result position.
    #[must_use]
    pub const fn result(&self) -> usize {
        self.result
    }

    /// Returns the region output boundary that realizes this result.
    #[must_use]
    pub const fn output_tensor(&self) -> VerifiedTensorId {
        self.output_tensor
    }

    /// Returns the complete unique write that initializes the result.
    #[must_use]
    pub const fn write_access(&self) -> VerifiedTensorAccessId {
        self.write_access
    }

    /// Returns the scalar value written to the result boundary.
    #[must_use]
    pub const fn written_value(&self) -> VerifiedScalarValueId {
        self.written_value
    }
}

/// Collision-free identity of reusable refinement content.
///
/// Content is site- and provider-independent: two occurrences of the same
/// operation and interface lowered to the same region under the same authority
/// share these bytes. The graph site, selected provider, and admission
/// provenance are deliberately absent; they belong to [`IndexRefinementIdentity`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RefinementContentIdentity(Vec<u8>);

impl RefinementContentIdentity {
    /// Returns the canonical content bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Collision-free identity of one refinement occurrence binding.
///
/// This is reusable content plus the exact semantic source, the selected
/// lowering provider, the capability revision, and provider-attributed admission
/// provenance. It pins *this* realization at *this* site.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementIdentity(Vec<u8>);

impl IndexRefinementIdentity {
    /// Returns the canonical occurrence-binding bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Reusable, site-independent refinement content.
///
/// Equal content proves the same reusable fact: this canonical index region
/// realizes an operation with this ordered value interface, under this numerical
/// contract and effect, reaching exactly these provider-independent scalar and
/// semantic definitions. It carries no graph site and no provider selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefinementContent {
    region_identity: CanonicalIndexRegionIdentity,
    operation: OpKey,
    operand_interface: Vec<(u32, ResolvedValueType, Shape)>,
    result_interface: Vec<(ResolvedValueType, Shape)>,
    attributes: OperationAttributes,
    effect: OperationEffect,
    numerical_contract: NumericalContractIdentity,
    scalar_authority: ScalarAuthorityEvidence,
    index_domain_proofs: Vec<AuthorizedIndexDomainProof>,
    identity: RefinementContentIdentity,
}

impl RefinementContent {
    /// Returns the structural identity of the realizing index region.
    #[must_use]
    pub const fn region_identity(&self) -> &CanonicalIndexRegionIdentity {
        &self.region_identity
    }

    /// Returns the realized semantic operation family key.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the host-canonical attributes the region realizes.
    #[must_use]
    pub const fn attributes(&self) -> &OperationAttributes {
        &self.attributes
    }

    /// Returns the observable effect the region realizes.
    #[must_use]
    pub const fn effect(&self) -> OperationEffect {
        self.effect
    }

    /// Returns the bound numerical-contract identity.
    #[must_use]
    pub const fn numerical_contract(&self) -> &NumericalContractIdentity {
        &self.numerical_contract
    }

    /// Returns the checked scalar authority evidence bound to this region.
    ///
    /// The receipt is bound to the exact structural region identity and keeps its
    /// provider-independent reached definitions separate from provider-attributed
    /// admission provenance.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }

    /// Returns the number of residual predicates discharged after IR verification.
    #[must_use]
    pub fn index_domain_discharge_count(&self) -> usize {
        self.index_domain_proofs.len()
    }

    /// Returns the compiler-sealed proofs over residual index-domain predicates.
    pub(crate) fn index_domain_proofs(&self) -> &[AuthorizedIndexDomainProof] {
        &self.index_domain_proofs
    }

    /// Returns the reusable content identity.
    #[must_use]
    pub const fn identity(&self) -> &RefinementContentIdentity {
        &self.identity
    }
}

/// A proved refinement of one semantic occurrence to one canonical index region.
///
/// It binds the reusable [`RefinementContent`] to the exact semantic source, the
/// selected lowering provider, and the ordered value/access bindings. Holding an
/// `IndexRefinement` is evidence that the emitted region realizes the occurrence,
/// not merely that a provider produced a well-formed region.
#[derive(Clone, Debug)]
pub struct IndexRefinement {
    content: RefinementContent,
    occurrence: SemanticOccurrenceIdentity,
    provider: ProviderIdentity,
    revision: LoweringCapabilityRevision,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    region: VerifiedIndexRegion,
    identity: IndexRefinementIdentity,
}

impl IndexRefinement {
    /// Returns the reusable, site-independent content.
    #[must_use]
    pub const fn content(&self) -> &RefinementContent {
        &self.content
    }

    /// Returns the occurrence-binding identity that pins this realization.
    #[must_use]
    pub const fn identity(&self) -> &IndexRefinementIdentity {
        &self.identity
    }

    /// Returns the opaque identity of the realized semantic source.
    #[must_use]
    pub const fn occurrence(&self) -> &SemanticOccurrenceIdentity {
        &self.occurrence
    }

    /// Returns the selected lowering provider.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the selected capability's output-affecting revision.
    #[must_use]
    pub const fn revision(&self) -> LoweringCapabilityRevision {
        self.revision
    }

    /// Returns the ordered operand-to-input bindings, including aliases.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }

    /// Returns the ordered result-to-output bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }

    /// Returns the checked scalar authority evidence.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        self.content.scalar_authority()
    }

    /// Returns the realizing verified index region.
    ///
    /// The region can be evaluated directly by the independent
    /// `tiler-reference` oracle; feed each input boundary the operand tensor
    /// named by [`Self::operand_bindings`].
    #[must_use]
    pub const fn region(&self) -> &VerifiedIndexRegion {
        &self.region
    }
}

/// A structurally and interface-checked realization awaiting semantic discharge.
///
/// This state owns the exact verified region whose region-local residual
/// predicates it exposes. It also retains the occurrence, scalar-authority
/// receipt, and ordered bindings already checked before the residual was found,
/// so a discharge stage never needs to drive the provider a second time.
///
/// Holding this value is not execution permission and is not an
/// [`IndexRefinement`]. Every value returned by [`Self::obligations`] must first
/// receive named semantic evidence.
#[derive(Clone, Debug)]
pub struct PendingIndexRefinement {
    occurrence: SemanticOccurrence,
    provider: ProviderIdentity,
    revision: LoweringCapabilityRevision,
    capability_authority: LoweringCapabilityAuthority,
    scalars: FrozenScalarRegistry,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    scalar_authority: ScalarAuthorityEvidence,
    region: VerifiedIndexRegion,
}

impl PendingIndexRefinement {
    /// Returns the exact semantic occurrence this realization is checked against.
    #[must_use]
    pub const fn occurrence(&self) -> &SemanticOccurrence {
        &self.occurrence
    }

    /// Returns the selected lowering provider.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the selected capability's output-affecting revision.
    #[must_use]
    pub const fn revision(&self) -> LoweringCapabilityRevision {
        self.revision
    }

    /// Returns the authority the resolved capability was admitted against.
    #[must_use]
    pub const fn capability_authority(&self) -> &LoweringCapabilityAuthority {
        &self.capability_authority
    }

    /// Returns the frozen scalar authority required to revalidate a discharged region.
    #[must_use]
    pub const fn scalar_registry(&self) -> &FrozenScalarRegistry {
        &self.scalars
    }

    /// Returns the already-checked ordered operand bindings.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }

    /// Returns the already-checked ordered result bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }

    /// Returns the scalar-authority receipt bound to the exact retained region.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }

    /// Returns the exact structurally verified region awaiting discharge.
    #[must_use]
    pub const fn region(&self) -> &VerifiedIndexRegion {
        &self.region
    }

    /// Returns every exact residual predicate in canonical region order.
    #[must_use]
    pub fn obligations(&self) -> impl ExactSizeIterator<Item = UnknownIndexDomainPredicate> + '_ {
        self.region.unknown_index_domain_predicates()
    }
}

impl PartialEq for PendingIndexRefinement {
    fn eq(&self, other: &Self) -> bool {
        self.occurrence == other.occurrence
            && self.provider == other.provider
            && self.revision == other.revision
            && self.capability_authority == other.capability_authority
            && self.scalars.snapshot_identity() == other.scalars.snapshot_identity()
            && self.operand_bindings == other.operand_bindings
            && self.result_bindings == other.result_bindings
            && self.scalar_authority == other.scalar_authority
            && self.region.canonical_identity() == other.region.canonical_identity()
    }
}

impl Eq for PendingIndexRefinement {}

/// Result of checking one provider emission against one semantic occurrence.
///
/// A pending result is valid checked analysis state, not a refinement failure
/// and not execution evidence. Callers must route it through the named semantic
/// discharge stage before any program work.
#[derive(Clone, Debug)]
#[must_use]
pub enum IndexRefinementOutcome {
    /// Every index-domain predicate was discharged and refinement is complete.
    Refined(Box<IndexRefinement>),
    /// Structural, authority, and interface checks passed, but exact residual
    /// predicates still require semantic discharge.
    Pending(Box<PendingIndexRefinement>),
}

impl IndexRefinementOutcome {
    /// Returns completed refinement evidence, when every predicate was discharged.
    #[must_use]
    pub const fn refined(&self) -> Option<&IndexRefinement> {
        match self {
            Self::Refined(refinement) => Some(refinement),
            Self::Pending(_) => None,
        }
    }

    /// Returns exact pending state, when semantic discharge is still required.
    #[must_use]
    pub const fn pending(&self) -> Option<&PendingIndexRefinement> {
        match self {
            Self::Refined(_) => None,
            Self::Pending(pending) => Some(pending),
        }
    }

    /// Consumes this outcome and returns completed refinement evidence.
    #[must_use]
    pub fn into_refined(self) -> Option<IndexRefinement> {
        match self {
            Self::Refined(refinement) => Some(*refinement),
            Self::Pending(_) => None,
        }
    }
}

/// A failure to refine a resolved lowering capability against an occurrence.
///
/// Every variant is a refusal to certify a realization. A build failure is the
/// provider emitting an invalid region; the remaining variants are well-formed
/// regions that do not realize the occurrence or that lack the required binding
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RefinementError {
    /// The resolved capability is not an index/access-lowering capability.
    WrongFamily {
        /// The family that was resolved instead.
        actual: LoweringFamily,
    },
    /// The resolved index/access provider handle was absent.
    MissingIndexProvider,
    /// The capability lowers a different operation than the occurrence.
    OperationMismatch {
        /// Operation the capability lowers.
        capability: Box<OpKey>,
        /// Operation the occurrence names.
        occurrence: Box<OpKey>,
    },
    /// The capability's admitted signature does not match the occurrence types.
    CapabilitySignatureMismatch,
    /// Two operands alias one value yet disagree on element type or shape.
    AliasedOperandInconsistent {
        /// Position of the inconsistent operand.
        operand: usize,
    },
    /// The occurrence could not be projected into provider-visible facts.
    OccurrenceFacts(LoweringRegistryError),
    /// The occurrence effect cannot be realized as a pure index region.
    EffectNotIndexable {
        /// The rejected effect class.
        effect: OperationEffect,
    },
    /// The provider rejected emission through the canonical builder.
    Emit(LoweringEmitError),
    /// The emitted region failed whole-region structural verification.
    Build {
        /// Deterministic structural diagnostics.
        diagnostics: Vec<IndexRegionDiagnostic>,
    },
    /// The region's scalar authority rejected revalidation.
    ScalarAuthority(Arc<ScalarRegistryError>),
    /// The capability and region disagree on the semantic type authority.
    SemanticAuthorityMismatch,
    /// The region reached a scalar authority the capability may not emit.
    ScalarAuthorityConformance,
    /// A verified region handle failed to resolve, so the region is malformed.
    Handle(VerifiedIndexHandleError),
    /// A boundary tensor exposed no static shape in this bounded profile.
    SymbolicBoundary,
    /// The region declares a different number of inputs than distinct operands.
    OperandArity {
        /// Region input boundary count.
        region_inputs: usize,
        /// Distinct occurrence operand count.
        distinct_operands: usize,
    },
    /// A region input boundary disagrees with its operand type or shape.
    OperandInterface {
        /// Ordered distinct-operand position.
        position: usize,
    },
    /// The region produces a different number of outputs than results.
    ResultArity {
        /// Region output-root count.
        region_outputs: usize,
        /// Occurrence result count.
        results: usize,
    },
    /// A region output boundary disagrees with its result type or shape.
    ResultInterface {
        /// Ordered result position.
        position: usize,
    },
    /// A region output writes a value of the wrong result type.
    ResultValueType {
        /// Ordered result position.
        position: usize,
    },
    /// A region output is not backed by a complete unique write.
    IncompleteWrite {
        /// Ordered result position.
        position: usize,
    },
}

impl fmt::Display for RefinementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongFamily { actual } => {
                write!(
                    formatter,
                    "resolved a {actual} capability, not index access"
                )
            }
            Self::MissingIndexProvider => {
                formatter.write_str("resolved index/access capability exposed no provider")
            }
            Self::OperationMismatch {
                capability,
                occurrence,
            } => write!(
                formatter,
                "capability lowers {capability} but the occurrence is {occurrence}"
            ),
            Self::CapabilitySignatureMismatch => formatter
                .write_str("capability signature does not match the occurrence value types"),
            Self::AliasedOperandInconsistent { operand } => write!(
                formatter,
                "operand {operand} aliases another value but disagrees on type or shape"
            ),
            Self::OccurrenceFacts(source) => {
                write!(formatter, "occurrence facts are inconsistent: {source}")
            }
            Self::EffectNotIndexable { effect } => write!(
                formatter,
                "occurrence effect {effect:?} cannot be realized as a pure index region"
            ),
            Self::Emit(source) => write!(formatter, "provider emission failed: {source}"),
            Self::Build { diagnostics } => write!(
                formatter,
                "emitted region failed verification with {} diagnostic(s)",
                diagnostics.len()
            ),
            Self::ScalarAuthority(source) => {
                write!(formatter, "region scalar authority failed: {source}")
            }
            Self::SemanticAuthorityMismatch => {
                formatter.write_str("capability and region disagree on the semantic type authority")
            }
            Self::ScalarAuthorityConformance => formatter.write_str(
                "region reached a scalar authority the capability was not admitted to emit",
            ),
            Self::Handle(source) => write!(formatter, "verified region handle failed: {source}"),
            Self::SymbolicBoundary => {
                formatter.write_str("a boundary tensor exposed no static shape")
            }
            Self::OperandArity {
                region_inputs,
                distinct_operands,
            } => write!(
                formatter,
                "region declares {region_inputs} inputs for {distinct_operands} distinct operands"
            ),
            Self::OperandInterface { position } => {
                write!(
                    formatter,
                    "region input {position} does not match its operand"
                )
            }
            Self::ResultArity {
                region_outputs,
                results,
            } => write!(
                formatter,
                "region produces {region_outputs} outputs for {results} results"
            ),
            Self::ResultInterface { position } => {
                write!(
                    formatter,
                    "region output {position} does not match its result"
                )
            }
            Self::ResultValueType { position } => {
                write!(
                    formatter,
                    "region output {position} writes the wrong result type"
                )
            }
            Self::IncompleteWrite { position } => write!(
                formatter,
                "region output {position} lacks complete unique-write evidence"
            ),
        }
    }
}

impl Error for RefinementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emit(source) => Some(source),
            Self::ScalarAuthority(source) => Some(source.as_ref()),
            Self::Handle(source) => Some(source),
            Self::OccurrenceFacts(source) => Some(source),
            _ => None,
        }
    }
}

impl From<VerifiedIndexHandleError> for RefinementError {
    fn from(source: VerifiedIndexHandleError) -> Self {
        Self::Handle(source)
    }
}

/// Refines one resolved index/access lowering capability against an occurrence.
///
/// The capability's provider is driven through the canonical `tiler-ir` builder,
/// the emitted region is structurally verified, and the region is then proved to
/// realize the occurrence. A successful build alone is never accepted: the
/// ordered value interface, reached scalar authority, semantic type authority,
/// and unique-write evidence are all checked before an
/// [`IndexRefinementOutcome`] is returned. Exact residual predicates produce
/// [`IndexRefinementOutcome::Pending`], never false refinement evidence or a
/// provider failure.
///
/// `scalars` is both the authority the region is built under and the authority
/// that revalidates it; it must be the same frozen scalar snapshot the
/// capability was registered against.
///
/// # Errors
///
/// Returns a [`RefinementError`] when the capability is the wrong family or
/// operation, the provider emits or the builder rejects an invalid region, the
/// scalar or semantic authority disagrees, or the emitted region does not
/// realize the occurrence's ordered value interface.
pub fn refine_index_region(
    capability: &ResolvedLoweringCapability,
    occurrence: &SemanticOccurrence,
    scalars: &FrozenScalarRegistry,
) -> Result<IndexRefinementOutcome, RefinementError> {
    bind_capability_to_occurrence(capability, occurrence)?;

    if occurrence.effect != OperationEffect::Pure {
        return Err(RefinementError::EffectNotIndexable {
            effect: occurrence.effect,
        });
    }

    let region = emit_region(capability, occurrence, scalars)?;
    // Registration and a successful build are not refinement evidence. Everything
    // below independently proves the emitted region realizes the occurrence.
    let scalar_authority = scalars
        .revalidate_region(&region)
        .map_err(|source| RefinementError::ScalarAuthority(Arc::new(source)))?;
    check_authority_conformance(capability, &scalar_authority)?;

    let operand_bindings = bind_operands(occurrence, &region)?;
    let result_bindings = bind_results(occurrence, &region)?;

    // A residual domain obligation is not permission to skip independent
    // authority and interface checks. Report those harder provider defects
    // first, then retain the otherwise-conforming region as pending checked
    // state rather than misclassifying it as refinement evidence or a provider
    // defect.
    if region.unknown_index_domain_predicates().len() != 0 {
        return Ok(IndexRefinementOutcome::Pending(Box::new(
            PendingIndexRefinement {
                occurrence: occurrence.clone(),
                provider: capability.provider().clone(),
                revision: capability.revision(),
                capability_authority: capability.authority().clone(),
                scalars: scalars.clone(),
                operand_bindings,
                result_bindings,
                scalar_authority,
                region,
            },
        )));
    }

    let content = assemble_content(occurrence, &region, scalar_authority, Vec::new());
    let identity = encode_occurrence_identity(
        &content,
        capability.provider(),
        capability.revision(),
        capability.authority(),
        occurrence,
    );
    Ok(IndexRefinementOutcome::Refined(Box::new(IndexRefinement {
        content,
        occurrence: occurrence.identity.clone(),
        provider: capability.provider().clone(),
        revision: capability.revision(),
        operand_bindings,
        result_bindings,
        region,
        identity,
    })))
}

/// Proves the resolved capability was resolved for exactly this occurrence.
fn bind_capability_to_occurrence(
    capability: &ResolvedLoweringCapability,
    occurrence: &SemanticOccurrence,
) -> Result<(), RefinementError> {
    if capability.family() != LoweringFamily::IndexAccess {
        return Err(RefinementError::WrongFamily {
            actual: capability.family(),
        });
    }
    if capability.operation() != &occurrence.operation {
        return Err(RefinementError::OperationMismatch {
            capability: Box::new(capability.operation().clone()),
            occurrence: Box::new(occurrence.operation.clone()),
        });
    }
    let signature = capability.signature();
    if signature.operands() != occurrence.operand_types().as_slice()
        || signature.results() != occurrence.result_types().as_slice()
    {
        return Err(RefinementError::CapabilitySignatureMismatch);
    }
    Ok(())
}

/// Drives the resolved provider through the canonical builder and verifies it.
fn emit_region(
    capability: &ResolvedLoweringCapability,
    occurrence: &SemanticOccurrence,
    scalars: &FrozenScalarRegistry,
) -> Result<VerifiedIndexRegion, RefinementError> {
    let provider = capability
        .index_access_provider()
        .ok_or(RefinementError::MissingIndexProvider)?;
    let facts = lowered_occurrence(occurrence)?;
    let mut builder = tiler_ir::index::IndexRegionBuilder::new(scalars.clone())
        .map_err(LoweringEmitError::from)?;
    {
        let mut context = IndexAccessLoweringContext::new(&mut builder, &facts);
        provider.lower(&mut context)?;
    }
    builder
        .build()
        .map_err(|error: IndexRegionBuildError| RefinementError::Build {
            diagnostics: error.diagnostics().to_vec(),
        })
}

/// Projects the occurrence into the read-only facts a provider may see.
///
/// The distinct-operand order is the same one [`bind_operands`] later binds the
/// region's input boundaries in, so a provider that declares its inputs in the
/// order it was handed cannot be rejected for an ordering the host never stated.
fn lowered_occurrence(
    occurrence: &SemanticOccurrence,
) -> Result<LoweredOccurrence, RefinementError> {
    let distinct = occurrence.distinct_operands()?;
    let inputs = distinct
        .iter()
        .map(|operand| OccurrenceBoundary::new(operand.value_type.clone(), operand.shape.clone()))
        .collect();
    let mut operands = Vec::with_capacity(occurrence.operands.len());
    for (position, operand) in occurrence.operands.iter().enumerate() {
        let index = distinct
            .iter()
            .position(|candidate| candidate.value == operand.value)
            .ok_or(RefinementError::OperandInterface { position })?;
        operands.push(index);
    }
    let results = occurrence
        .results
        .iter()
        .map(|result| OccurrenceBoundary::new(result.value_type.clone(), result.shape.clone()))
        .collect();
    LoweredOccurrence::new(inputs, operands, results, occurrence.attributes.clone())
        .map_err(RefinementError::OccurrenceFacts)
}

impl From<LoweringEmitError> for RefinementError {
    fn from(source: LoweringEmitError) -> Self {
        Self::Emit(source)
    }
}

/// Proves the region reached nothing beyond the authority the capability may emit.
///
/// Containment rather than equality is the exact property: one capability lowers
/// every occurrence of its family and signature, and which declared scalar
/// operations a given occurrence needs depends on that occurrence's shapes and
/// attributes. A single-contributor reduction reaches no scalar operation at all
/// and an empty one reaches only the identity constant, yet both are lowered by
/// the same capability. Requiring equality would force a provider to be
/// registered per shape; requiring containment still refuses every region that
/// reached an authority the capability was never admitted to emit.
fn check_authority_conformance(
    capability: &ResolvedLoweringCapability,
    scalar_authority: &ScalarAuthorityEvidence,
) -> Result<(), RefinementError> {
    let authority = capability.authority();
    if scalar_authority.semantic_snapshot() != authority.operation_authority().registry_snapshot() {
        return Err(RefinementError::SemanticAuthorityMismatch);
    }
    let declared = authority.emitted_scalar_operations();
    if scalar_authority
        .reached_operations()
        .iter()
        .any(|reached| !declared.contains(reached))
    {
        return Err(RefinementError::ScalarAuthorityConformance);
    }
    Ok(())
}

/// Binds the occurrence's ordered operands to the region's input boundaries.
fn bind_operands(
    occurrence: &SemanticOccurrence,
    region: &VerifiedIndexRegion,
) -> Result<Vec<OperandBinding>, RefinementError> {
    let inputs: Vec<_> = region
        .tensors()
        .filter(|tensor| tensor.role() == TensorRole::Input)
        .collect();
    let distinct = occurrence.distinct_operands()?;
    if inputs.len() != distinct.len() {
        return Err(RefinementError::OperandArity {
            region_inputs: inputs.len(),
            distinct_operands: distinct.len(),
        });
    }
    for (position, (operand, input)) in distinct.iter().zip(&inputs).enumerate() {
        let shape = input
            .static_shape()
            .ok_or(RefinementError::SymbolicBoundary)?;
        if input.value_type() != &operand.value_type || shape != &operand.shape {
            return Err(RefinementError::OperandInterface { position });
        }
    }
    // The distinct-operand order fixes the input boundary of every value, so each
    // ordered operand (aliases included) resolves to its value's input tensor.
    let mut bindings = Vec::with_capacity(occurrence.operands.len());
    for (position, operand) in occurrence.operands.iter().enumerate() {
        let distinct_index = distinct
            .iter()
            .position(|candidate| candidate.value == operand.value)
            .ok_or(RefinementError::OperandInterface { position })?;
        bindings.push(OperandBinding {
            operand: position,
            value: operand.value,
            input_tensor: inputs[distinct_index].id(),
        });
    }
    Ok(bindings)
}

/// Binds the occurrence's ordered results to the region's output roots.
fn bind_results(
    occurrence: &SemanticOccurrence,
    region: &VerifiedIndexRegion,
) -> Result<Vec<ResultBinding>, RefinementError> {
    let roots: Vec<_> = region.outputs().collect();
    if roots.len() != occurrence.results.len() {
        return Err(RefinementError::ResultArity {
            region_outputs: roots.len(),
            results: occurrence.results.len(),
        });
    }
    let mut bindings = Vec::with_capacity(roots.len());
    for (position, (root, result)) in roots.iter().zip(&occurrence.results).enumerate() {
        let access = region.access(root.access())?;
        // A refined result must be a complete unique ordinary write. Any retained
        // ownership proof witnesses that; its absence is an incomplete write.
        if access.write_ownership_proof().is_none() {
            return Err(RefinementError::IncompleteWrite { position });
        }
        let output = region.tensor(access.tensor())?;
        let shape = output
            .static_shape()
            .ok_or(RefinementError::SymbolicBoundary)?;
        if output.role() != TensorRole::Output
            || output.value_type() != &result.value_type
            || shape != &result.shape
        {
            return Err(RefinementError::ResultInterface { position });
        }
        let written = region.scalar_value(root.value())?;
        if written.value_type() != &result.value_type {
            return Err(RefinementError::ResultValueType { position });
        }
        bindings.push(ResultBinding {
            result: position,
            output_tensor: output.id(),
            write_access: root.access(),
            written_value: root.value(),
        });
    }
    Ok(bindings)
}

/// Assembles reusable content and its canonical identity.
fn assemble_content(
    occurrence: &SemanticOccurrence,
    region: &VerifiedIndexRegion,
    scalar_authority: ScalarAuthorityEvidence,
    index_domain_proofs: Vec<AuthorizedIndexDomainProof>,
) -> RefinementContent {
    let operand_interface = canonical_operand_interface(occurrence);
    let result_interface = occurrence
        .results
        .iter()
        .map(|result| (result.value_type.clone(), result.shape.clone()))
        .collect();
    let region_identity = region.canonical_identity().clone();
    let identity = encode_content_identity(
        &region_identity,
        occurrence,
        &operand_interface,
        &scalar_authority,
        &index_domain_proofs,
    );
    RefinementContent {
        region_identity,
        operation: occurrence.operation.clone(),
        operand_interface,
        result_interface,
        attributes: occurrence.attributes.clone(),
        effect: occurrence.effect,
        numerical_contract: occurrence.numerical_contract.clone(),
        scalar_authority,
        index_domain_proofs,
        identity,
    }
}

/// Completes one pending refinement from sealed proofs over every exact residual.
///
/// The proof vector is compiler-owned and unforgeable outside this crate. Its
/// canonical order must match the retained region's canonical obligation order;
/// violating that invariant is a compiler defect rather than an input refusal.
pub(crate) fn complete_pending_index_refinement(
    pending: PendingIndexRefinement,
    index_domain_proofs: Vec<AuthorizedIndexDomainProof>,
) -> IndexRefinement {
    assert_eq!(
        pending.obligations().len(),
        index_domain_proofs.len(),
        "semantic discharge must prove every retained index-domain obligation"
    );
    assert!(
        pending
            .obligations()
            .zip(&index_domain_proofs)
            .all(|(obligation, proof)| obligation == proof.obligation()),
        "semantic-discharge proofs must preserve canonical obligation order"
    );
    let PendingIndexRefinement {
        occurrence,
        provider,
        revision,
        capability_authority,
        scalars: _,
        operand_bindings,
        result_bindings,
        scalar_authority,
        region,
    } = pending;
    let content = assemble_content(&occurrence, &region, scalar_authority, index_domain_proofs);
    let identity = encode_occurrence_identity(
        &content,
        &provider,
        revision,
        &capability_authority,
        &occurrence,
    );
    IndexRefinement {
        content,
        occurrence: occurrence.identity,
        provider,
        revision,
        operand_bindings,
        result_bindings,
        region,
        identity,
    }
}

/// Canonicalizes operands to first-occurrence local names plus type and shape.
///
/// The aliasing structure is retained as content while the occurrence-local
/// value identifiers are not, mirroring how region content renumbers members to
/// region-local positions.
fn canonical_operand_interface(
    occurrence: &SemanticOccurrence,
) -> Vec<(u32, ResolvedValueType, Shape)> {
    let mut order: Vec<OccurrenceValueId> = Vec::new();
    let mut interface = Vec::with_capacity(occurrence.operands.len());
    for operand in &occurrence.operands {
        let local = if let Some(index) = order.iter().position(|value| *value == operand.value) {
            u32::try_from(index).unwrap_or(u32::MAX)
        } else {
            let index = u32::try_from(order.len()).unwrap_or(u32::MAX);
            order.push(operand.value);
            index
        };
        interface.push((local, operand.value_type.clone(), operand.shape.clone()));
    }
    interface
}

fn encode_content_identity(
    region_identity: &CanonicalIndexRegionIdentity,
    occurrence: &SemanticOccurrence,
    operand_interface: &[(u32, ResolvedValueType, Shape)],
    scalar_authority: &ScalarAuthorityEvidence,
    index_domain_proofs: &[AuthorizedIndexDomainProof],
) -> RefinementContentIdentity {
    let mut bytes = CONTENT_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region_identity.as_bytes());
    encode_op_key(&mut bytes, &occurrence.operation);
    push_len(&mut bytes, operand_interface.len());
    for (local, value_type, shape) in operand_interface {
        bytes.extend_from_slice(&local.to_be_bytes());
        push_slice(&mut bytes, value_type.canonical_encoding().as_bytes());
        encode_shape(&mut bytes, shape);
    }
    push_len(&mut bytes, occurrence.results.len());
    for result in &occurrence.results {
        push_slice(
            &mut bytes,
            result.value_type.canonical_encoding().as_bytes(),
        );
        encode_shape(&mut bytes, &result.shape);
    }
    bytes.push(effect_tag(occurrence.effect));
    // Attributes are content: two occurrences of one family that differ only in
    // an attribute are different reusable facts even when their emitted regions
    // happen to coincide.
    push_slice(
        &mut bytes,
        occurrence.attributes.canonical_encoding().as_bytes(),
    );
    push_slice(&mut bytes, occurrence.numerical_contract.as_bytes());
    // Provider-independent reached authority is content; provider-attributed
    // admission provenance is deliberately withheld for the occurrence binding.
    push_slice(&mut bytes, scalar_authority.definitions().as_bytes());
    push_slice(&mut bytes, scalar_authority.type_definitions().as_bytes());
    push_slice(&mut bytes, scalar_authority.semantic_snapshot().as_bytes());
    push_slice(&mut bytes, scalar_authority.scalar_snapshot().as_bytes());
    push_len(&mut bytes, index_domain_proofs.len());
    for proof in index_domain_proofs {
        push_slice(&mut bytes, proof.identity());
    }
    RefinementContentIdentity(bytes)
}

fn encode_occurrence_identity(
    content: &RefinementContent,
    provider: &ProviderIdentity,
    revision: LoweringCapabilityRevision,
    authority: &LoweringCapabilityAuthority,
    occurrence: &SemanticOccurrence,
) -> IndexRefinementIdentity {
    let mut bytes = OCCURRENCE_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, content.identity.as_bytes());
    push_slice(&mut bytes, occurrence.identity.as_bytes());
    encode_provider(&mut bytes, provider);
    bytes.extend_from_slice(&revision.get().to_be_bytes());
    push_slice(
        &mut bytes,
        authority
            .operation_authority()
            .admission_provenance()
            .as_bytes(),
    );
    push_slice(&mut bytes, content.scalar_authority.admission().as_bytes());
    push_slice(
        &mut bytes,
        content.scalar_authority.type_admission().as_bytes(),
    );
    IndexRefinementIdentity(bytes)
}

/// Encodes one observable effect class into refinement content identity.
///
/// The sibling of `fusion_legality::effect_tag`, and exhaustive for the same
/// reason (ADR 0074 convention 3). Refinement rejects every non-pure effect
/// before content is assembled, so no non-pure tag can enter an accepted
/// identity today; that is a property of the caller and not of the encoder, and
/// it is exactly the kind of reasoning a wildcard arm would silently outlive.
const fn effect_tag(effect: OperationEffect) -> u8 {
    match effect {
        OperationEffect::Pure => 1,
    }
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

fn encode_shape(output: &mut Vec<u8>, shape: &Shape) {
    push_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::Arc;

    use tiler_ir::index::{
        DomainRole, FrozenScalarRegistry, IndexDomainSoundProof, ScalarArity,
        ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
        ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract,
        ScalarOperationDefinition, ScalarOperationInferencer, ScalarRegistryBuilder,
        UnknownIndexDomainPredicate, VerifiedIndexRegion,
    };
    use tiler_ir::semantic::{
        CanonicalValue, F32, FrozenSemanticRegistry, InputKey, NormativeDefinitionRef, OutputKey,
        ProviderDiagnosticCode, ProviderIdentity, ResolvedValueType, SemanticProgram,
        SemanticProgramBuilder, multiply_f32_op,
    };
    use tiler_ir::shape::{Extent, Shape};

    use tiler_reference::{
        FloatBitOrder, FrozenReferenceRegistry, IndexRegionAuthority, IndexRegionEvaluator,
        IndexRegionInput, ReferenceCapabilityRevision, ReferenceElement, ReferenceOperationError,
        ReferenceSignature, ScalarReferenceOperation, ScalarReferenceOutputs,
        ScalarReferenceRegistryBuilder, ScalarReferenceRequest, Tensor, TensorPayloadView,
    };

    use super::{
        NumericalContractIdentity, OccurrenceOperand, OccurrenceResult, OccurrenceValueId,
        RefinementError, SemanticOccurrence, SemanticOccurrenceIdentity, refine_index_region,
    };
    use crate::capability::{
        FrozenLoweringCapabilityRegistry, IndexAccessLoweringContext, IndexAccessLoweringProvider,
        LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision, LoweringEmitError,
        LoweringFamily, LoweringSignature, ScalarLoweringContext, ScalarLoweringProvider,
        ScalarLoweringResults,
    };
    use crate::index_discharge::{
        IndexDomainDischargeAuthority, IndexDomainDischargeClaim, IndexDomainDischargeProof,
        IndexDomainDischargeProvider, IndexDomainDischargeRefusalKind, IndexDomainDisproof,
        discharge_with,
    };
    use crate::region::form_region_candidates;
    use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

    const LENGTH: u64 = 4;

    fn f32_type() -> ResolvedValueType {
        F32::resolved_type()
    }

    fn scalar_key(name: &str) -> ScalarOpKey {
        ScalarOpKey::new("example", name, 1).unwrap()
    }

    fn provider(name: &str) -> ProviderIdentity {
        ProviderIdentity::new("example", name, 1).unwrap()
    }

    fn revision() -> LoweringCapabilityRevision {
        LoweringCapabilityRevision::new(1).unwrap()
    }

    fn binary_signature() -> LoweringSignature {
        LoweringSignature::new([f32_type(), f32_type()], [f32_type()]).unwrap()
    }

    fn pending_refinement() -> super::PendingIndexRefinement {
        let scalars = scalar_registry();
        let length = 65_535;
        let resolved = index_registry(
            Arc::new(ConservativeReadSquare { length, rounds: 8 }),
            &[scalar_key("multiply")],
        )
        .resolve_index_access(&multiply_f32_op(), &binary_signature())
        .unwrap();
        let occurrence = square_occurrence_with_length(b"pending-site", length);
        let outcome = refine_index_region(&resolved, &occurrence, &scalars).unwrap();
        outcome
            .pending()
            .expect("the conservative read exceeds the governed proof budget")
            .clone()
    }

    #[derive(Clone, Copy)]
    enum DischargeMode {
        Sound,
        Exhaustive,
        Disproved,
        Unknown,
    }

    struct TestDischarge {
        authority: IndexDomainDischargeAuthority,
        mode: DischargeMode,
        calls: Cell<usize>,
    }

    impl TestDischarge {
        fn new(mode: DischargeMode, revision: u32) -> Self {
            Self {
                authority: IndexDomainDischargeAuthority::builtin(
                    "test.index-domain-discharge",
                    "test-index-domain-rule",
                    revision,
                ),
                mode,
                calls: Cell::new(0),
            }
        }
    }

    impl IndexDomainDischargeProvider for TestDischarge {
        fn authority(&self) -> &IndexDomainDischargeAuthority {
            &self.authority
        }

        fn assess(
            &self,
            _region: &VerifiedIndexRegion,
            obligation: UnknownIndexDomainPredicate,
        ) -> IndexDomainDischargeClaim {
            let ordinal = self.calls.get();
            self.calls.set(ordinal + 1);
            match self.mode {
                DischargeMode::Sound => {
                    IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::Sound {
                        proof: IndexDomainSoundProof::Interval,
                        derivation: b"test-sound-derivation".as_slice().into(),
                    })
                }
                DischargeMode::Exhaustive => {
                    IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::ExhaustiveFinite {
                        points: 65_535,
                        derivation: b"test-exhaustive-derivation".as_slice().into(),
                    })
                }
                DischargeMode::Disproved => IndexDomainDischargeClaim::Disproved(
                    IndexDomainDisproof::new("test-counterexample", b"counterexample".as_slice()),
                ),
                DischargeMode::Unknown => IndexDomainDischargeClaim::Unknown(obligation.reason()),
            }
        }
    }

    fn semantic() -> FrozenSemanticRegistry {
        FrozenSemanticRegistry::standard().unwrap()
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
            outputs.try_push(first.clone())
        }
    }

    fn scalar_definition(name: &str) -> ScalarOperationDefinition {
        ScalarOperationDefinition::new(
            scalar_key(name),
            NormativeDefinitionRef::from_owned(format!("urn:example:{name}:v1")).unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::record([]).unwrap(),
                CanonicalValue::record([]).unwrap(),
            ),
            Arc::new(SameType),
        )
    }

    fn scalar_registry() -> FrozenScalarRegistry {
        // Ad-hoc: pairs the scalars with a lowering provider whose extent is a
        // registration-time constant, so a fixture can register a provider that
        // deliberately disagrees with the occurrence it is resolved for. A governed
        // provider reads its extents from the occurrence facts and cannot disagree.
        let mut builder = ScalarRegistryBuilder::new(semantic());
        let scalars = provider("f32-scalars");
        for name in ["multiply", "add"] {
            builder
                .register(scalars.clone(), scalar_definition(name))
                .unwrap();
        }
        builder.freeze()
    }

    /// Emits `out[i] = mul(in[i], in[i])` over a parallel domain of `length`.
    ///
    /// The length is a registration-time constant so the fixtures can register a
    /// provider that deliberately disagrees with the occurrence it is resolved
    /// for; a governed provider reads its extents from the occurrence facts.
    struct PointwiseSquare {
        length: u64,
    }
    impl IndexAccessLoweringProvider for PointwiseSquare {
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
            let squared = product.get(0).expect("multiply yields one result");
            let write = context.write(output, &[i], &[row])?;
            context.output(write, squared)?;
            Ok(())
        }
    }

    /// Emits a square whose read is equal to `i` but interval-conservative.
    struct ConservativeReadSquare {
        length: u64,
        rounds: usize,
    }
    impl IndexAccessLoweringProvider for ConservativeReadSquare {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let shape = Shape::from_dims([self.length]);
            let i = context.dimension(DomainRole::Parallel, Extent::new(self.length))?;
            let input = context.input_tensor(f32_type(), shape.clone())?;
            let output = context.output_tensor(f32_type(), shape)?;
            let row = context.dimension_expr(i)?;
            let mut read = row;
            for _ in 0..self.rounds {
                let modulo = context.modulo(read, 2)?;
                let quotient = context.floor_div(read, 2)?;
                read = context.linear_combination(
                    0_i128.into(),
                    &[(2_i128.into(), quotient), (1_i128.into(), modulo)],
                )?;
            }
            let value = context.read(input, &[i], &[read])?;
            let product = context.apply(
                scalar_key("multiply"),
                ScalarAttributes::empty(),
                &[value, value],
            )?;
            let squared = product.get(0).expect("multiply yields one result");
            let write = context.write(output, &[i], &[row])?;
            context.output(write, squared)
        }
    }

    /// Emits a well-formed `out[i] = add(in[i], in[i])`, reaching `add` only.
    struct PointwiseAdd {
        length: u64,
    }
    impl IndexAccessLoweringProvider for PointwiseAdd {
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
            let sum = context.apply(
                scalar_key("add"),
                ScalarAttributes::empty(),
                &[value, value],
            )?;
            let total = sum.get(0).expect("add yields one result");
            let write = context.write(output, &[i], &[row])?;
            context.output(write, total)?;
            Ok(())
        }
    }

    /// Emits a well-formed square with two identical output roots.
    struct TwoOutputSquare {
        length: u64,
    }
    impl IndexAccessLoweringProvider for TwoOutputSquare {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let shape = Shape::from_dims([self.length]);
            let i = context.dimension(DomainRole::Parallel, Extent::new(self.length))?;
            let input = context.input_tensor(f32_type(), shape.clone())?;
            let first = context.output_tensor(f32_type(), shape.clone())?;
            let second = context.output_tensor(f32_type(), shape)?;
            let row = context.dimension_expr(i)?;
            let value = context.read(input, &[i], &[row])?;
            let product = context.apply(
                scalar_key("multiply"),
                ScalarAttributes::empty(),
                &[value, value],
            )?;
            let squared = product.get(0).expect("multiply yields one result");
            let first_write = context.write(first, &[i], &[row])?;
            context.output(first_write, squared)?;
            let second_write = context.write(second, &[i], &[row])?;
            context.output(second_write, squared)?;
            Ok(())
        }
    }

    /// A scalar-lowering provider, used to prove refinement rejects that family.
    struct ScalarMultiply;
    impl ScalarLoweringProvider for ScalarMultiply {
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

    fn index_registry(
        index_provider: Arc<dyn IndexAccessLoweringProvider>,
        emitted: &[ScalarOpKey],
    ) -> FrozenLoweringCapabilityRegistry {
        let mut builder = LoweringCapabilityRegistryBuilder::new(semantic(), scalar_registry());
        builder
            .register_index_access(
                provider("index"),
                multiply_f32_op(),
                binary_signature(),
                emitted,
                revision(),
                index_provider,
            )
            .unwrap();
        builder.freeze()
    }

    fn square_registry() -> FrozenLoweringCapabilityRegistry {
        index_registry(
            Arc::new(PointwiseSquare { length: LENGTH }),
            &[scalar_key("multiply")],
        )
    }

    fn contract() -> NumericalContractIdentity {
        NumericalContractIdentity::from_key(
            crate::request::StrictF32NumericalContract::governed().key,
        )
    }

    fn square_occurrence(site: &[u8]) -> SemanticOccurrence {
        square_occurrence_with_length(site, LENGTH)
    }

    fn square_occurrence_with_length(site: &[u8], length: u64) -> SemanticOccurrence {
        let shape = Shape::from_dims([length]);
        let v = OccurrenceValueId(0);
        SemanticOccurrence::new(
            multiply_f32_op(),
            vec![
                OccurrenceOperand::new(v, f32_type(), shape.clone()),
                OccurrenceOperand::new(v, f32_type(), shape.clone()),
            ],
            vec![OccurrenceResult::new(f32_type(), shape)],
            tiler_ir::semantic::OperationAttributes::empty(),
            tiler_ir::semantic::OperationEffect::Pure,
            contract(),
            SemanticOccurrenceIdentity::from_bytes(site.to_vec()),
        )
    }

    #[test]
    fn refines_a_well_formed_square_and_binds_ordered_values() {
        let scalars = scalar_registry();
        let frozen = square_registry();
        let resolved = frozen
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let occurrence = square_occurrence(b"occurrence-a");

        let refinement = refine_index_region(&resolved, &occurrence, &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");

        // Both aliased operands bind to the one input boundary; the single result
        // binds to the one output root with a complete unique write.
        assert_eq!(refinement.operand_bindings().len(), 2);
        let inputs = refinement.operand_bindings();
        assert_eq!(inputs[0].input_tensor(), inputs[1].input_tensor());
        assert_eq!(refinement.result_bindings().len(), 1);
        assert_eq!(refinement.provider(), &provider("index"));
        assert_eq!(refinement.revision(), revision());
        // The scalar authority receipt is bound to the exact structural region.
        assert_eq!(
            refinement.scalar_authority().region(),
            refinement.region().canonical_identity()
        );
    }

    #[test]
    fn reusable_content_is_separate_from_occurrence_identity() {
        let scalars = scalar_registry();
        let frozen = square_registry();
        let resolved = frozen
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();

        let first = refine_index_region(&resolved, &square_occurrence(b"site-1"), &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");
        let second = refine_index_region(&resolved, &square_occurrence(b"site-2"), &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");

        // Same operation, interface, and region: identical reusable content.
        assert_eq!(first.content().identity(), second.content().identity());
        assert_eq!(
            first.content().region_identity(),
            second.content().region_identity()
        );
        // Different semantic source: distinct occurrence bindings.
        assert_ne!(first.identity(), second.identity());
        assert_ne!(first.occurrence(), second.occurrence());
    }

    #[test]
    fn refinement_output_is_checkable_against_the_reference_oracle() {
        let scalars = scalar_registry();
        let frozen = square_registry();
        let resolved = frozen
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let occurrence = square_occurrence(b"occurrence-oracle");
        let refinement = refine_index_region(&resolved, &occurrence, &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");

        // Independently execute the refined region on concrete inputs, feeding the
        // one input boundary that both operands bound to.
        let input_tensor = refinement.operand_bindings()[0].input_tensor();
        let input = f32_tensor([0.0, 1.0, 2.0, 3.0]);
        let evaluator = IndexRegionEvaluator::new(
            FrozenReferenceRegistry::standard().unwrap(),
            multiply_reference(&scalars),
        );
        let evaluation = evaluator
            .evaluate(
                refinement.region(),
                IndexRegionAuthority::new(&scalars),
                &[IndexRegionInput::new(input_tensor, &input)],
            )
            .unwrap();

        // The occurrence is `out[i] = in[i] * in[i]`; the oracle agrees.
        assert_eq!(
            f32_values(&evaluation.outputs()[0]),
            vec![0.0, 1.0, 4.0, 9.0]
        );
    }

    #[test]
    fn refines_an_occurrence_identity_produced_by_region_formation() {
        // Genuine composition with region formation: a real singleton occurrence
        // identity is refined against the emitted region.
        let program = square_program();
        let outcome = form_region_candidates(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap();
        let candidate = outcome
            .whole_program_candidate()
            .expect("the single multiply is its own whole-program region");
        let site = candidate.occurrence().as_bytes().to_vec();

        let scalars = scalar_registry();
        let resolved = square_registry()
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let occurrence = square_occurrence(&site);
        let refinement = refine_index_region(&resolved, &occurrence, &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");
        assert_eq!(refinement.occurrence().as_bytes(), site.as_slice());
    }

    #[test]
    fn a_well_formed_region_with_the_wrong_result_shape_is_rejected() {
        // The provider emits a valid length-8 square; the occurrence is length-4.
        let scalars = scalar_registry();
        let resolved = index_registry(
            Arc::new(PointwiseSquare { length: 8 }),
            &[scalar_key("multiply")],
        )
        .resolve_index_access(&multiply_f32_op(), &binary_signature())
        .unwrap();
        let error =
            refine_index_region(&resolved, &square_occurrence(b"site"), &scalars).unwrap_err();
        assert!(matches!(
            error,
            RefinementError::OperandInterface { position: 0 }
        ));
    }

    #[test]
    fn a_residual_bound_does_not_mask_a_wrong_provider_interface() {
        // The conservative read exceeds the proof-cell budget, but the emitted
        // length-65_535 interface is independently wrong for the length-4
        // occurrence. The harder provider defect must remain the diagnosis.
        let scalars = scalar_registry();
        let resolved = index_registry(
            Arc::new(ConservativeReadSquare {
                length: 65_535,
                rounds: 8,
            }),
            &[scalar_key("multiply")],
        )
        .resolve_index_access(&multiply_f32_op(), &binary_signature())
        .unwrap();
        let error =
            refine_index_region(&resolved, &square_occurrence(b"site"), &scalars).unwrap_err();
        assert!(matches!(
            error,
            RefinementError::OperandInterface { position: 0 }
        ));
    }

    #[test]
    fn a_residual_outcome_retains_the_exact_checked_region_and_bindings() {
        let scalars = scalar_registry();
        let length = 65_535;
        let resolved = index_registry(
            Arc::new(ConservativeReadSquare { length, rounds: 8 }),
            &[scalar_key("multiply")],
        )
        .resolve_index_access(&multiply_f32_op(), &binary_signature())
        .unwrap();
        let occurrence = square_occurrence_with_length(b"pending-site", length);

        let outcome = refine_index_region(&resolved, &occurrence, &scalars).unwrap();
        let pending = outcome
            .pending()
            .expect("the conservative read exceeds the governed proof budget");
        assert_eq!(pending.occurrence(), &occurrence);
        assert_eq!(pending.provider(), resolved.provider());
        assert_eq!(pending.revision(), resolved.revision());
        assert_eq!(pending.capability_authority(), resolved.authority());
        assert_eq!(
            pending.scalar_registry().snapshot_identity(),
            scalars.snapshot_identity()
        );
        assert_eq!(pending.operand_bindings().len(), 2);
        assert_eq!(pending.result_bindings().len(), 1);
        assert_eq!(
            pending.scalar_authority().region(),
            pending.region().canonical_identity()
        );
        let obligations = pending.obligations().collect::<Vec<_>>();
        assert!(!obligations.is_empty());
        for obligation in obligations {
            assert_eq!(
                pending
                    .region()
                    .index_domain_unknown(obligation.subject(), obligation.predicate())
                    .unwrap(),
                Some(obligation)
            );
        }
    }

    #[test]
    fn semantic_discharge_assesses_every_exact_obligation_once() {
        let pending = pending_refinement();
        let obligations = pending.obligations().len();
        assert_ne!(
            obligations, 0,
            "the fixture must retain an exact obligation"
        );
        let provider = TestDischarge::new(DischargeMode::Sound, 1);

        let refinement = discharge_with(&provider, pending).unwrap();

        assert_eq!(provider.calls.get(), obligations);
        assert_eq!(
            refinement.content().index_domain_discharge_count(),
            obligations
        );
    }

    #[test]
    fn semantic_discharge_keeps_disproved_and_unknown_distinct_and_atomic() {
        let disproved = discharge_with(
            &TestDischarge::new(DischargeMode::Disproved, 1),
            pending_refinement(),
        )
        .unwrap_err();
        assert_eq!(disproved.kind(), IndexDomainDischargeRefusalKind::Disproved);

        let unknown = discharge_with(
            &TestDischarge::new(DischargeMode::Unknown, 1),
            pending_refinement(),
        )
        .unwrap_err();
        assert_eq!(unknown.kind(), IndexDomainDischargeRefusalKind::Unknown);

        assert_eq!(
            unknown.pending().obligations().len(),
            unknown.assessments().len(),
            "a refusal retains the complete pending state and every assessment"
        );
    }

    #[test]
    fn semantic_discharge_authority_and_proof_basis_move_content_identity() {
        let sound_v1 = discharge_with(
            &TestDischarge::new(DischargeMode::Sound, 1),
            pending_refinement(),
        )
        .unwrap();
        let sound_v2 = discharge_with(
            &TestDischarge::new(DischargeMode::Sound, 2),
            pending_refinement(),
        )
        .unwrap();
        let exhaustive = discharge_with(
            &TestDischarge::new(DischargeMode::Exhaustive, 1),
            pending_refinement(),
        )
        .unwrap();

        assert_ne!(sound_v1.content().identity(), sound_v2.content().identity());
        assert_ne!(
            sound_v1.content().identity(),
            exhaustive.content().identity()
        );
    }

    #[test]
    fn a_well_formed_region_with_an_extra_output_is_rejected() {
        let scalars = scalar_registry();
        let resolved = index_registry(
            Arc::new(TwoOutputSquare { length: LENGTH }),
            &[scalar_key("multiply")],
        )
        .resolve_index_access(&multiply_f32_op(), &binary_signature())
        .unwrap();
        let error =
            refine_index_region(&resolved, &square_occurrence(b"site"), &scalars).unwrap_err();
        assert!(matches!(
            error,
            RefinementError::ResultArity {
                region_outputs: 2,
                results: 1
            }
        ));
    }

    #[test]
    fn a_region_reaching_an_undeclared_scalar_authority_is_rejected() {
        // Registered emitting `multiply`, the provider instead reaches `add`.
        let scalars = scalar_registry();
        let resolved = index_registry(
            Arc::new(PointwiseAdd { length: LENGTH }),
            &[scalar_key("multiply")],
        )
        .resolve_index_access(&multiply_f32_op(), &binary_signature())
        .unwrap();
        let error =
            refine_index_region(&resolved, &square_occurrence(b"site"), &scalars).unwrap_err();
        assert!(matches!(error, RefinementError::ScalarAuthorityConformance));
    }

    #[test]
    fn a_scalar_lowering_capability_is_not_an_index_refinement() {
        let scalars = scalar_registry();
        let mut builder = LoweringCapabilityRegistryBuilder::new(semantic(), scalar_registry());
        builder
            .register_scalar_lowering(
                provider("scalar"),
                multiply_f32_op(),
                binary_signature(),
                &[scalar_key("multiply")],
                revision(),
                Arc::new(ScalarMultiply),
            )
            .unwrap();
        let resolved = builder
            .freeze()
            .resolve_scalar_lowering(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let error =
            refine_index_region(&resolved, &square_occurrence(b"site"), &scalars).unwrap_err();
        assert!(matches!(
            error,
            RefinementError::WrongFamily {
                actual: LoweringFamily::ScalarLowering
            }
        ));
    }

    #[test]
    fn an_occurrence_naming_another_operation_is_rejected() {
        let scalars = scalar_registry();
        let resolved = square_registry()
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let shape = Shape::from_dims([LENGTH]);
        let v = OccurrenceValueId(0);
        let occurrence = SemanticOccurrence::new(
            tiler_ir::semantic::add_f32_op(),
            vec![
                OccurrenceOperand::new(v, f32_type(), shape.clone()),
                OccurrenceOperand::new(v, f32_type(), shape.clone()),
            ],
            vec![OccurrenceResult::new(f32_type(), shape)],
            tiler_ir::semantic::OperationAttributes::empty(),
            tiler_ir::semantic::OperationEffect::Pure,
            contract(),
            SemanticOccurrenceIdentity::from_bytes(b"site".to_vec()),
        );
        let error = refine_index_region(&resolved, &occurrence, &scalars).unwrap_err();
        assert!(matches!(error, RefinementError::OperationMismatch { .. }));
    }

    // Reference-oracle helpers.

    fn square_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([LENGTH]))
            .unwrap();
        let product = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), product)
            .unwrap();
        builder.build().unwrap()
    }

    struct MultiplyReference;
    impl ScalarReferenceOperation for MultiplyReference {
        fn evaluate(
            &self,
            request: ScalarReferenceRequest<'_>,
            outputs: &mut ScalarReferenceOutputs,
        ) -> Result<(), ReferenceOperationError> {
            let [left, right] = request.operands() else {
                return Err(ReferenceOperationError::InvalidApplication);
            };
            let value = decode(left) * decode(right);
            outputs.push(reference_scalar(value)?)
        }
    }

    fn multiply_reference(
        scalars: &FrozenScalarRegistry,
    ) -> tiler_reference::FrozenScalarReferenceRegistry {
        let mut builder = ScalarReferenceRegistryBuilder::new(scalars.clone());
        builder
            .register(
                ProviderIdentity::new("example", "f32-scalar-reference", 1).unwrap(),
                scalar_key("multiply"),
                ReferenceSignature::new([f32_type(), f32_type()], [f32_type()]).unwrap(),
                ReferenceCapabilityRevision::new(1).unwrap(),
                Arc::new(MultiplyReference),
            )
            .unwrap();
        builder.freeze().unwrap()
    }

    fn element(value: f32) -> ReferenceElement {
        ReferenceElement::from_float_bits(
            value.to_bits().to_be_bytes(),
            FloatBitOrder::MostSignificantByteFirst,
        )
        .unwrap()
    }

    fn reference_scalar(value: f32) -> Result<Tensor, ReferenceOperationError> {
        Tensor::scalar(f32_type(), element(value))
            .map_err(|_| ReferenceOperationError::InvalidApplication)
    }

    fn decode(tensor: &Tensor) -> f32 {
        let TensorPayloadView::Dense([value]) = tensor.payload() else {
            panic!("expected a dense scalar")
        };
        f32::from_bits(u32::from_be_bytes(
            <[u8; 4]>::try_from(value.as_bytes()).unwrap(),
        ))
    }

    fn f32_tensor<const N: usize>(values: [f32; N]) -> Tensor {
        Tensor::dense(
            f32_type(),
            Shape::from_dims([u64::try_from(N).unwrap()]),
            values.into_iter().map(element).collect(),
        )
        .unwrap()
    }

    fn f32_values(tensor: &Tensor) -> Vec<f32> {
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            panic!("expected a dense f32 tensor")
        };
        elements
            .iter()
            .map(|value| {
                f32::from_bits(u32::from_be_bytes(
                    <[u8; 4]>::try_from(value.as_bytes()).unwrap(),
                ))
            })
            .collect()
    }
}
