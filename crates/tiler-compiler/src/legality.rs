//! Compiler integration for IR-owned index-region refinement evidence.
//!
//! [`tiler_ir::index::FrozenIndexRealizationLawRegistry`] resolves the semantic
//! provider's candidate-blind law, and [`tiler_ir::index::ResolvedIndexRealization`] owns
//! the dependency-neutral semantic-subject/verified-region check and mints its
//! opaque receipt. This
//! module owns the compiler envelope around that receipt: provider provenance,
//! reusable content identity, occurrence identity, and integration with the
//! compiler's domain-discharge authority.
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
    CanonicalIndexRegionSequenceIdentity, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry,
    IndexRefinementDomainProof, IndexRefinementReceipt, IndexRefinementSubject,
    IndexRefinementVerificationError, IndexRefinementVerificationOutcome, IndexRegionDiagnostic,
    IndexRegionSequenceError, NumericalContractIdentity, OperandBinding, ResultBinding,
    ScalarAuthorityEvidence, ScalarRegistryError, UnknownIndexDomainPredicate,
    VerifiedIndexHandleError, VerifiedIndexRegion, VerifiedIndexRegionSequence,
};
use tiler_ir::semantic::{
    OpKey, OperationAttributes, OperationEffect, ProviderIdentity, ResolvedValueType,
};
use tiler_ir::shape::Shape;

use crate::capability::{
    IndexAccessSequenceContext, IndexAccessStageFailure, LoweringCapabilityAuthority,
    LoweringCapabilityRevision, LoweringEmitError, LoweringFamily, ResolvedLoweringCapability,
};

/// Canonical domain-separation tag for reusable single-region refinement content.
const CONTENT_IDENTITY_TAG: &[u8] = b"tiler.compiler.index-refinement-content.v2\0";
/// Canonical domain-separation tag for reusable staged refinement content.
///
/// A one-stage realization keeps [`CONTENT_IDENTITY_TAG`] and encodes exactly
/// the bytes it always has, because a one-stage sequence identity *is* its
/// region's identity and a one-stage realization retains no leading stage. Only
/// a chain is written under this tag, and neither tag is a prefix of the other,
/// so the two preimages are disjoint.
const STAGED_CONTENT_IDENTITY_TAG: &[u8] = b"tiler.compiler.index-refinement-content.staged.v1\0";
/// Canonical domain-separation tag for one single-region refinement occurrence.
const OCCURRENCE_IDENTITY_TAG: &[u8] = b"tiler.compiler.index-refinement-occurrence.v2\0";
/// Canonical domain-separation tag for one staged refinement occurrence binding.
///
/// Domain-separated for the reason [`STAGED_CONTENT_IDENTITY_TAG`] is: an
/// occurrence binding over a chain carries every stage's provider-attributed
/// admission provenance, and a one-stage binding carries exactly the one it
/// always has.
const STAGED_OCCURRENCE_IDENTITY_TAG: &[u8] =
    b"tiler.compiler.index-refinement-occurrence.staged.v1\0";

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
    realization_identity: CanonicalIndexRegionSequenceIdentity,
    stage_count: usize,
    operation: OpKey,
    operand_interface: Vec<(u32, ResolvedValueType, Shape)>,
    result_interface: Vec<(ResolvedValueType, Shape)>,
    attributes: OperationAttributes,
    effect: OperationEffect,
    numerical_contract: NumericalContractIdentity,
    /// Every stage's evidence except the final one's, in stage order.
    ///
    /// Split the way [`VerifiedIndexRegionSequence`] splits its stages: a
    /// realization always has a final stage, and the reached scalar authority
    /// genuinely differs between a fold and the pass consuming it, so a chain is
    /// not the same reusable fact as either stage alone.
    leading_scalar_authorities: Vec<ScalarAuthorityEvidence>,
    scalar_authority: ScalarAuthorityEvidence,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
    identity: RefinementContentIdentity,
}

impl RefinementContent {
    /// Returns the canonical identity of the whole ordered realization.
    ///
    /// For a one-stage realization these are the realizing region's own
    /// canonical bytes; for a chain they are the sequence's, under its own
    /// domain tag.
    #[must_use]
    pub const fn realization_identity(&self) -> &CanonicalIndexRegionSequenceIdentity {
        &self.realization_identity
    }

    /// Returns how many ordered stages the realization retains, never zero.
    #[must_use]
    pub const fn stage_count(&self) -> usize {
        self.stage_count
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

    /// Returns the checked scalar authority evidence bound to the final stage.
    ///
    /// The receipt is bound to the exact structural region identity and keeps its
    /// provider-independent reached definitions separate from provider-attributed
    /// admission provenance. For a one-stage realization the final stage is the
    /// only stage; [`Self::scalar_authorities`] answers the whole chain.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }

    /// Returns every stage's checked scalar authority evidence, in stage order.
    #[must_use]
    pub fn scalar_authorities(&self) -> Vec<ScalarAuthorityEvidence> {
        let mut authorities = self.leading_scalar_authorities.clone();
        authorities.push(self.scalar_authority.clone());
        authorities
    }

    /// Returns the number of residual predicates discharged after IR verification.
    #[must_use]
    pub fn index_domain_discharge_count(&self) -> usize {
        self.index_domain_proofs.len()
    }

    /// Returns the IR-sealed proofs over residual index-domain predicates.
    pub(crate) fn index_domain_proofs(&self) -> &[IndexRefinementDomainProof] {
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
    receipt: IndexRefinementReceipt,
    provider: ProviderIdentity,
    revision: LoweringCapabilityRevision,
    realization: VerifiedIndexRegionSequence,
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
    pub const fn receipt(&self) -> &IndexRefinementReceipt {
        &self.receipt
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

    /// Returns ordered operand-to-input bindings, including aliases and one
    /// binding per encoded component in semantic contract order.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        self.receipt.operand_bindings()
    }

    /// Returns the ordered result-to-output bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        self.receipt.result_bindings()
    }

    /// Returns the checked scalar authority evidence of the final stage.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        self.content.scalar_authority()
    }

    /// Returns the exact ordered realization the provider emitted.
    ///
    /// Its stages, their wiring, and every handed intermediate's shape,
    /// ownership, and lifetime are reachable from here.
    #[must_use]
    pub const fn realization(&self) -> &VerifiedIndexRegionSequence {
        &self.realization
    }

    /// Returns the one realizing region, when the realization has exactly one.
    ///
    /// Such a region can be evaluated directly by the independent
    /// `tiler-reference` oracle; feed each input boundary the operand tensor
    /// named by [`Self::operand_bindings`].
    ///
    /// **The answer is `None` for a chain rather than the final stage**, because
    /// a chain's final stage reads a value no occurrence operand carries, so the
    /// binding above does not compose for it. A consumer that can only evaluate
    /// one region therefore refuses a staged realization explicitly instead of
    /// evaluating a third of it and reporting the result as the occurrence's.
    #[must_use]
    pub fn single_region(&self) -> Option<&VerifiedIndexRegion> {
        self.realization
            .is_single_stage()
            .then(|| self.realization.final_stage())
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
    receipt: Box<tiler_ir::index::PendingIndexRefinementReceipt>,
    provider: ProviderIdentity,
    revision: LoweringCapabilityRevision,
    capability_authority: LoweringCapabilityAuthority,
}

impl PendingIndexRefinement {
    pub(crate) const fn ir_receipt(&self) -> &tiler_ir::index::PendingIndexRefinementReceipt {
        &self.receipt
    }

    /// Returns the exact semantic occurrence this realization is checked against.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        self.receipt.subject()
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

    /// Returns the already-checked ordered operand bindings, with encoded
    /// components expanded in semantic contract order.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        self.receipt.operand_bindings()
    }

    /// Returns the already-checked ordered result bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        self.receipt.result_bindings()
    }

    /// Returns the scalar-authority receipt bound to the final retained stage.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        self.receipt.scalar_authority()
    }

    /// Returns every stage's scalar-authority receipt, in stage order.
    #[must_use]
    pub fn scalar_authorities(&self) -> Vec<ScalarAuthorityEvidence> {
        self.receipt.scalar_authorities()
    }

    /// Returns the exact structurally verified realization awaiting discharge.
    #[must_use]
    pub const fn realization(&self) -> &VerifiedIndexRegionSequence {
        self.receipt.realization()
    }

    /// Returns the one retained region, when the realization has exactly one.
    ///
    /// `None` for a chain, for the reason [`IndexRefinement::single_region`]
    /// gives.
    #[must_use]
    pub fn single_region(&self) -> Option<&VerifiedIndexRegion> {
        let realization = self.receipt.realization();
        realization
            .is_single_stage()
            .then(|| realization.final_stage())
    }

    /// Returns every exact residual predicate in canonical region order.
    #[must_use]
    pub fn obligations(&self) -> impl ExactSizeIterator<Item = UnknownIndexDomainPredicate> + '_ {
        self.receipt.obligations()
    }
}

impl PartialEq for PendingIndexRefinement {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider
            && self.revision == other.revision
            && self.capability_authority == other.capability_authority
            && self.receipt == other.receipt
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
    /// The occurrence effect cannot be realized as a pure index region.
    EffectNotIndexable {
        /// The rejected effect class.
        effect: OperationEffect,
    },
    /// The provider rejected emission of one stage through the canonical builder.
    Emit {
        /// Ordered realization stage that failed to emit.
        stage: usize,
        /// Typed emission cause.
        source: LoweringEmitError,
    },
    /// One emitted stage failed whole-region structural verification.
    Build {
        /// Ordered realization stage whose region was rejected.
        stage: usize,
        /// Deterministic structural diagnostics.
        diagnostics: Vec<IndexRegionDiagnostic>,
    },
    /// The emitted stages do not compose into a well-formed ordered chain.
    ///
    /// Distinct from every interface refusal below: each stage verified on its
    /// own and the disagreement is in the composition — a value handed on that
    /// nothing reads, a handed boundary the consumer disagrees with, or a stage
    /// population beyond the governed ceiling.
    Realization {
        /// Typed chain refusal.
        source: IndexRegionSequenceError,
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
    /// An encoded semantic input declared no component boundary to bind.
    EmptyEncodedOperandComponents {
        /// Position in the distinct semantic input population.
        input: usize,
    },
    /// The region declares a different number of inputs than the expanded
    /// semantic input boundary requires.
    OperandArity {
        /// Region input boundary count.
        region_inputs: usize,
        /// Expected ordinary inputs plus ordered encoded components, saturated
        /// at `usize::MAX` if count arithmetic overflowed.
        expanded_inputs: usize,
    },
    /// A region input boundary disagrees with its expanded semantic input type
    /// or shape.
    OperandInterface {
        /// Position in the ordered expanded semantic input boundaries.
        position: usize,
    },
    /// Alias and component expansion exceeded the receipt binding population.
    OperandBindingsTooLarge {
        /// Binding count, saturated at `usize::MAX` on arithmetic overflow.
        actual: usize,
        /// Maximum operand bindings retained by one receipt.
        limit: usize,
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
    /// The IR-owned authority refused the semantic subject or emitted region.
    IrVerifier(Arc<IndexRefinementVerificationError>),
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
            Self::EffectNotIndexable { effect } => write!(
                formatter,
                "occurrence effect {effect:?} cannot be realized as a pure index region"
            ),
            Self::Emit { stage, source } => {
                write!(
                    formatter,
                    "provider emission failed at stage {stage}: {source}"
                )
            }
            Self::Build { stage, diagnostics } => write!(
                formatter,
                "emitted stage {stage} failed verification with {} diagnostic(s)",
                diagnostics.len()
            ),
            Self::Realization { source } => {
                write!(formatter, "emitted stages do not chain: {source}")
            }
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
            Self::EmptyEncodedOperandComponents { input } => write!(
                formatter,
                "encoded semantic input {input} declares no component boundaries"
            ),
            Self::OperandArity {
                region_inputs,
                expanded_inputs,
            } => write!(
                formatter,
                "region declares {region_inputs} inputs for {expanded_inputs} expanded semantic input boundaries"
            ),
            Self::OperandInterface { position } => {
                write!(
                    formatter,
                    "region input {position} does not match its expanded semantic input boundary"
                )
            }
            Self::OperandBindingsTooLarge { actual, limit } => write!(
                formatter,
                "expanded operand bindings {actual} exceed receipt limit {limit}"
            ),
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
            Self::IrVerifier(source) => {
                write!(formatter, "IR refinement authority refused: {source}")
            }
        }
    }
}

impl Error for RefinementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emit { source, .. } => Some(source),
            Self::Realization { source } => Some(source),
            Self::ScalarAuthority(source) => Some(source.as_ref()),
            Self::Handle(source) => Some(source),
            Self::IrVerifier(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<IndexAccessStageFailure> for RefinementError {
    fn from(source: IndexAccessStageFailure) -> Self {
        match source {
            IndexAccessStageFailure::Emit { stage, source } => Self::Emit { stage, source },
            IndexAccessStageFailure::Build { stage, diagnostics } => {
                Self::Build { stage, diagnostics }
            }
            IndexAccessStageFailure::Chain(source) => Self::Realization { source },
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
    subject: &IndexRefinementSubject,
    realizations: &FrozenIndexRealizationLawRegistry,
    scalars: &FrozenScalarRegistry,
) -> Result<IndexRefinementOutcome, RefinementError> {
    if capability.family() != LoweringFamily::IndexAccess {
        return Err(RefinementError::WrongFamily {
            actual: capability.family(),
        });
    }
    if capability.operation() != subject.operation() {
        return Err(RefinementError::OperationMismatch {
            capability: Box::new(capability.operation().clone()),
            occurrence: Box::new(subject.operation().clone()),
        });
    }
    let resolution = realizations
        .resolve(subject)
        .map_err(|source| map_ir_verifier_error(source, capability, subject))?;
    let realization = emit_realization(capability, subject, scalars)?;
    // Registration and a successful build are not refinement evidence. Everything
    // below independently proves the emitted realization realizes the occurrence.
    let verified = resolution
        .verify_sequence(capability.authority().refinement(), &realization)
        .map_err(|source| map_ir_verifier_error(source, capability, subject))?;

    // A residual domain obligation is not permission to skip independent
    // authority and interface checks. Report those harder provider defects
    // first, then retain the otherwise-conforming region as pending checked
    // state rather than misclassifying it as refinement evidence or a provider
    // defect.
    if let IndexRefinementVerificationOutcome::Pending(receipt) = verified {
        return Ok(IndexRefinementOutcome::Pending(Box::new(
            PendingIndexRefinement {
                receipt,
                provider: capability.provider().clone(),
                revision: capability.revision(),
                capability_authority: capability.authority().clone(),
            },
        )));
    }

    let IndexRefinementVerificationOutcome::Verified(receipt) = verified else {
        unreachable!()
    };
    let scalar_authorities = receipt.scalar_authorities();
    let content = assemble_content(subject, &realization, scalar_authorities, Vec::new());
    let identity = encode_occurrence_identity(
        &content,
        capability.provider(),
        capability.revision(),
        capability.authority(),
        subject,
        &receipt,
    );
    Ok(IndexRefinementOutcome::Refined(Box::new(IndexRefinement {
        content,
        receipt: *receipt,
        provider: capability.provider().clone(),
        revision: capability.revision(),
        realization,
        identity,
    })))
}

fn map_ir_verifier_error(
    source: IndexRefinementVerificationError,
    capability: &ResolvedLoweringCapability,
    subject: &IndexRefinementSubject,
) -> RefinementError {
    match source {
        IndexRefinementVerificationError::OperationMismatch => RefinementError::OperationMismatch {
            capability: Box::new(capability.operation().clone()),
            occurrence: Box::new(subject.operation().clone()),
        },
        IndexRefinementVerificationError::CapabilitySignatureMismatch => {
            RefinementError::CapabilitySignatureMismatch
        }
        IndexRefinementVerificationError::EffectNotIndexable { effect } => {
            RefinementError::EffectNotIndexable { effect }
        }
        IndexRefinementVerificationError::ScalarAuthority(source) => {
            RefinementError::ScalarAuthority(source)
        }
        IndexRefinementVerificationError::SemanticAuthorityMismatch => {
            RefinementError::SemanticAuthorityMismatch
        }
        IndexRefinementVerificationError::ScalarAuthorityConformance => {
            RefinementError::ScalarAuthorityConformance
        }
        IndexRefinementVerificationError::Handle(source) => RefinementError::Handle(source),
        IndexRefinementVerificationError::SymbolicBoundary => RefinementError::SymbolicBoundary,
        IndexRefinementVerificationError::EmptyEncodedOperandComponents { input } => {
            RefinementError::EmptyEncodedOperandComponents { input }
        }
        IndexRefinementVerificationError::OperandArity {
            region_inputs,
            expanded_inputs,
        } => RefinementError::OperandArity {
            region_inputs,
            expanded_inputs,
        },
        IndexRefinementVerificationError::OperandInterface { position } => {
            RefinementError::OperandInterface { position }
        }
        IndexRefinementVerificationError::OperandBindingsTooLarge { actual, limit } => {
            RefinementError::OperandBindingsTooLarge { actual, limit }
        }
        IndexRefinementVerificationError::ResultArity {
            region_outputs,
            results,
        } => RefinementError::ResultArity {
            region_outputs,
            results,
        },
        IndexRefinementVerificationError::ResultInterface { position } => {
            RefinementError::ResultInterface { position }
        }
        IndexRefinementVerificationError::ResultValueType { position } => {
            RefinementError::ResultValueType { position }
        }
        IndexRefinementVerificationError::IncompleteWrite { position } => {
            RefinementError::IncompleteWrite { position }
        }
        other => RefinementError::IrVerifier(Arc::new(other)),
    }
}

/// Drives the resolved provider through the canonical builders and verifies it.
///
/// Every stage is built and structurally verified as it is emitted, and the
/// retained stages are then proved to compose. The provider's own result is
/// deliberately not the last word: a stage failure recorded on the context is
/// reported even when the provider discarded it and returned `Ok`, so a
/// swallowed refusal cannot reach the semantic comparison as a shorter chain
/// that happens to be well formed.
fn emit_realization(
    capability: &ResolvedLoweringCapability,
    occurrence: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
) -> Result<VerifiedIndexRegionSequence, RefinementError> {
    let provider = capability
        .index_access_provider()
        .ok_or(RefinementError::MissingIndexProvider)?;
    let mut sequence = IndexAccessSequenceContext::new(scalars, occurrence);
    let emitted = provider.lower_sequence(&mut sequence);
    // Three refusals can be in play at once, and the order below is what keeps
    // each one's diagnosis attributable. A recorded stage failure is the most
    // specific — it names the stage and whether emission or verification
    // refused — and it is also the one a provider can have discarded. A
    // provider raising its own refusal without opening a stage reports it next,
    // attributed to the stage it was about to emit, because reporting an empty
    // chain instead would replace "this provider does not lower this
    // occurrence" with "no stage was emitted". Only then does composition speak.
    let pending_stage = sequence.stage_count();
    if let Some(failure) = sequence.take_failure() {
        return Err(failure.into());
    }
    emitted.map_err(|source| RefinementError::Emit {
        stage: pending_stage,
        source,
    })?;
    sequence.finish().map_err(RefinementError::from)
}

/// Assembles reusable content and its canonical identity.
///
/// `scalar_authorities` is every stage's evidence in stage order, which the IR
/// receipt always answers with at least one entry.
fn assemble_content(
    occurrence: &IndexRefinementSubject,
    realization: &VerifiedIndexRegionSequence,
    mut scalar_authorities: Vec<ScalarAuthorityEvidence>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
) -> RefinementContent {
    let operand_interface = canonical_operand_interface(occurrence);
    let result_interface = occurrence
        .results()
        .iter()
        .map(|result| (result.value_type().clone(), result.shape().clone()))
        .collect();
    let realization_identity = realization.identity().clone();
    let scalar_authority = scalar_authorities
        .pop()
        .expect("a verified realization retains one scalar authority per stage");
    let leading_scalar_authorities = scalar_authorities;
    let identity = encode_content_identity(
        &realization_identity,
        realization.is_single_stage(),
        occurrence,
        &operand_interface,
        &leading_scalar_authorities,
        &scalar_authority,
        &index_domain_proofs,
    );
    RefinementContent {
        realization_identity,
        stage_count: realization.stage_count(),
        operation: occurrence.operation().clone(),
        operand_interface,
        result_interface,
        attributes: occurrence.attributes().clone(),
        effect: occurrence.effect(),
        numerical_contract: occurrence.numerical_contract().clone(),
        leading_scalar_authorities,
        scalar_authority,
        index_domain_proofs,
        identity,
    }
}

/// Completes one pending refinement from sealed proofs over every exact residual.
///
/// IR seals the proof vector and revalidates that the completed receipt belongs
/// to this exact pending occurrence before compiler content is assembled. A
/// crossed receipt is a typed IR-verifier refusal, never a compiler assertion.
pub(crate) fn complete_pending_index_refinement(
    pending: PendingIndexRefinement,
    receipt: tiler_ir::index::IndexRefinementReceipt,
) -> Result<IndexRefinement, RefinementError> {
    pending
        .receipt
        .verify_completion(&receipt)
        .map_err(|source| RefinementError::IrVerifier(Arc::new(source)))?;
    let index_domain_proofs = receipt.index_domain_proofs().to_vec();
    let subject = pending.subject().clone();
    let realization = pending.realization().clone();
    let scalar_authorities = pending.scalar_authorities();
    let PendingIndexRefinement {
        provider,
        revision,
        capability_authority,
        ..
    } = pending;
    let content = assemble_content(
        &subject,
        &realization,
        scalar_authorities,
        index_domain_proofs,
    );
    let identity = encode_occurrence_identity(
        &content,
        &provider,
        revision,
        &capability_authority,
        &subject,
        &receipt,
    );
    Ok(IndexRefinement {
        content,
        receipt,
        provider,
        revision,
        realization,
        identity,
    })
}

/// Canonicalizes operands to first-occurrence local names plus type and shape.
///
/// The aliasing structure is retained as content while the occurrence-local
/// value identifiers are not, mirroring how region content renumbers members to
/// region-local positions.
fn canonical_operand_interface(
    occurrence: &IndexRefinementSubject,
) -> Vec<(u32, ResolvedValueType, Shape)> {
    let mut interface = Vec::with_capacity(occurrence.operands().len());
    for input in occurrence.operands() {
        let boundary = &occurrence.inputs()[*input];
        interface.push((
            u32::try_from(*input).unwrap_or(u32::MAX),
            boundary.value_type().clone(),
            boundary.shape().clone(),
        ));
    }
    interface
}

/// Encodes reusable refinement content.
///
/// **Injectivity across the two domains.** A one-stage realization is written
/// under [`CONTENT_IDENTITY_TAG`] exactly as it always has been: a one-stage
/// sequence identity is its region's identity byte for byte, and a one-stage
/// realization has no leading stage, so no byte of the previous encoding moves.
/// A chain is written under [`STAGED_CONTENT_IDENTITY_TAG`], neither tag being a
/// prefix of the other, and carries every leading stage's reached authority
/// length-framed after the final stage's. That block is load-bearing rather than
/// decorative: a fold and the pass consuming it reach different scalar
/// operations, so two chains agreeing on the final stage alone are different
/// reusable facts.
fn encode_content_identity(
    realization_identity: &CanonicalIndexRegionSequenceIdentity,
    single_stage: bool,
    occurrence: &IndexRefinementSubject,
    operand_interface: &[(u32, ResolvedValueType, Shape)],
    leading_scalar_authorities: &[ScalarAuthorityEvidence],
    scalar_authority: &ScalarAuthorityEvidence,
    index_domain_proofs: &[IndexRefinementDomainProof],
) -> RefinementContentIdentity {
    let mut bytes = if single_stage {
        CONTENT_IDENTITY_TAG.to_vec()
    } else {
        STAGED_CONTENT_IDENTITY_TAG.to_vec()
    };
    push_slice(&mut bytes, realization_identity.as_bytes());
    encode_op_key(&mut bytes, occurrence.operation());
    push_len(&mut bytes, operand_interface.len());
    for (local, value_type, shape) in operand_interface {
        bytes.extend_from_slice(&local.to_be_bytes());
        push_slice(&mut bytes, value_type.canonical_encoding().as_bytes());
        encode_shape(&mut bytes, shape);
    }
    push_len(&mut bytes, occurrence.results().len());
    for result in occurrence.results() {
        push_slice(
            &mut bytes,
            result.value_type().canonical_encoding().as_bytes(),
        );
        encode_shape(&mut bytes, result.shape());
    }
    bytes.push(effect_tag(occurrence.effect()));
    // Attributes are content: two occurrences of one family that differ only in
    // an attribute are different reusable facts even when their emitted regions
    // happen to coincide.
    push_slice(
        &mut bytes,
        occurrence.attributes().canonical_encoding().as_bytes(),
    );
    push_slice(&mut bytes, occurrence.numerical_contract().as_bytes());
    // Provider-independent reached authority is content; provider-attributed
    // admission provenance is deliberately withheld for the occurrence binding.
    push_slice(&mut bytes, scalar_authority.definitions().as_bytes());
    push_slice(&mut bytes, scalar_authority.type_definitions().as_bytes());
    push_slice(&mut bytes, scalar_authority.semantic_snapshot().as_bytes());
    push_slice(&mut bytes, scalar_authority.scalar_snapshot().as_bytes());
    if !single_stage {
        push_len(&mut bytes, leading_scalar_authorities.len());
        for authority in leading_scalar_authorities {
            push_slice(&mut bytes, authority.definitions().as_bytes());
            push_slice(&mut bytes, authority.type_definitions().as_bytes());
            push_slice(&mut bytes, authority.semantic_snapshot().as_bytes());
            push_slice(&mut bytes, authority.scalar_snapshot().as_bytes());
        }
    }
    push_len(&mut bytes, index_domain_proofs.len());
    for proof in index_domain_proofs {
        push_slice(&mut bytes, proof.identity().as_bytes());
    }
    RefinementContentIdentity(bytes)
}

/// Encodes one occurrence binding, domain-separated the way content is.
///
/// The trailing provider-attributed admissions are per stage, because a stage's
/// [`ScalarAuthorityEvidence`] is the authority that stage actually *reached*
/// rather than the capability's whole declared permission. A one-stage binding
/// carries exactly the one admission pair it always has, under the unchanged
/// [`OCCURRENCE_IDENTITY_TAG`].
fn encode_occurrence_identity(
    content: &RefinementContent,
    provider: &ProviderIdentity,
    revision: LoweringCapabilityRevision,
    authority: &LoweringCapabilityAuthority,
    occurrence: &IndexRefinementSubject,
    receipt: &IndexRefinementReceipt,
) -> IndexRefinementIdentity {
    let single_stage = content.stage_count == 1;
    let mut bytes = if single_stage {
        OCCURRENCE_IDENTITY_TAG.to_vec()
    } else {
        STAGED_OCCURRENCE_IDENTITY_TAG.to_vec()
    };
    push_slice(&mut bytes, content.identity.as_bytes());
    push_slice(&mut bytes, occurrence.graph().as_bytes());
    bytes.extend_from_slice(&occurrence.occurrence().get().to_be_bytes());
    push_slice(&mut bytes, receipt.identity().as_bytes());
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
    if !single_stage {
        push_len(&mut bytes, content.leading_scalar_authorities.len());
        for stage in &content.leading_scalar_authorities {
            push_slice(&mut bytes, stage.admission().as_bytes());
            push_slice(&mut bytes, stage.type_admission().as_bytes());
        }
    }
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
    use std::fmt::Write as _;
    use std::sync::Arc;

    use tiler_ir::index::{
        DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry,
        IndexRefinementSubject, IndexRefinementVerificationError, NumericalContractIdentity,
        ScalarArity, ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
        ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract,
        ScalarOperationDefinition, ScalarOperationInferencer, ScalarRegistryBuilder, SourcedExtent,
    };
    use tiler_ir::program::SemanticOccurrence;
    use tiler_ir::semantic::{
        CanonicalValue, F32, FrozenSemanticRegistry, InputKey, NormativeDefinitionRef, OutputKey,
        ProviderDiagnosticCode, ProviderIdentity, ResolvedValueType, SemanticProgram,
        SemanticProgramBuilder, constant_f32_op, multiply_f32_op,
    };
    use tiler_ir::shape::{Extent, Shape};

    use tiler_reference::{
        FloatBitOrder, FrozenReferenceRegistry, IndexRegionAuthority, IndexRegionEvaluator,
        IndexRegionInput, ReferenceCapabilityRevision, ReferenceElement, ReferenceOperationError,
        ReferenceSignature, ScalarReferenceOperation, ScalarReferenceOutputs,
        ScalarReferenceRegistryBuilder, ScalarReferenceRequest, Tensor, TensorPayloadView,
    };

    use super::{
        RefinementError, emit_realization, map_ir_verifier_error,
        refine_index_region as refine_index_region_with_registry,
    };
    use crate::capability::{
        FrozenLoweringCapabilityRegistry, IndexAccessLoweringContext, IndexAccessLoweringProvider,
        LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision, LoweringEmitError,
        LoweringFamily, LoweringSignature, ResolvedLoweringCapability, ScalarLoweringContext,
        ScalarLoweringProvider, ScalarLoweringResults,
    };
    use crate::region::form_region_candidates;
    use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

    const LENGTH: u64 = 4;

    fn refine_index_region(
        capability: &ResolvedLoweringCapability,
        subject: &IndexRefinementSubject,
        scalars: &FrozenScalarRegistry,
    ) -> Result<super::IndexRefinementOutcome, RefinementError> {
        let realizations =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic(), scalars.clone()).unwrap();
        refine_index_region_with_registry(capability, subject, &realizations, scalars)
    }

    fn f32_type() -> ResolvedValueType {
        F32::resolved_type()
    }

    fn scalar_key(name: &str) -> ScalarOpKey {
        match name {
            "multiply" => tiler_ir::index::multiply_f32_scalar_op(),
            "add" => tiler_ir::index::add_f32_scalar_op(),
            _ => ScalarOpKey::new("example", name, 1).unwrap(),
        }
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

    fn scalar_registry_with_extra_definition() -> FrozenScalarRegistry {
        let mut builder = ScalarRegistryBuilder::new(semantic());
        let scalars = provider("f32-scalars");
        for name in ["multiply", "add", "extra"] {
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
                let two = SourcedExtent::Static(Extent::new(2));
                let modulo = context.modulo(read, two.clone())?;
                let quotient = context.floor_div(read, two)?;
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
        let mut builder =
            LoweringCapabilityRegistryBuilder::new(semantic(), scalar_registry()).unwrap();
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
        NumericalContractIdentity::try_from_key(
            crate::request::StrictF32NumericalContract::governed().key,
        )
        .unwrap()
    }

    fn square_occurrence(site: &[u8]) -> IndexRefinementSubject {
        square_occurrence_with_length(site, LENGTH)
    }

    fn square_occurrence_with_length(site: &[u8], length: u64) -> IndexRefinementSubject {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let mut suffix = String::with_capacity(site.len() * 2);
        for byte in site {
            write!(&mut suffix, "{byte:02x}").expect("writing to a String cannot fail");
        }
        let input = builder
            .input::<F32>(
                InputKey::new(format!("input-{suffix}")).unwrap(),
                Shape::from_dims([length]),
            )
            .unwrap();
        let result = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = builder.build().unwrap();
        IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap()
    }

    fn constant_subject(bits: u32, contract_key: &str) -> IndexRefinementSubject {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let result = tiler_ir::semantic::F32Constant::apply(&mut builder, bits).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = builder.build().unwrap();
        IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            NumericalContractIdentity::try_from_key(contract_key).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn a_resolution_for_other_attributes_mints_no_receipt() {
        let scalars = crate::governed::governed_scalars().unwrap();
        let registry = crate::governed::governed_lowering_capabilities(&scalars).unwrap();
        let signature = LoweringSignature::new([], [f32_type()]).unwrap();
        let resolved = registry
            .resolve_index_access(&constant_f32_op(), &signature)
            .unwrap();
        let numerical_contract = contract();
        let admitted = constant_subject(1.0_f32.to_bits(), numerical_contract.as_str());
        let changed = constant_subject(2.0_f32.to_bits(), numerical_contract.as_str());
        let realizations = crate::governed::governed_realization_laws(&scalars);
        let resolution = realizations.resolve(&admitted).unwrap();
        let realization = emit_realization(&resolved, &changed, &scalars).unwrap();

        let error = resolution
            .verify(resolved.authority().refinement(), realization.final_stage())
            .unwrap_err();
        assert!(matches!(
            error,
            IndexRefinementVerificationError::SemanticRealizationMismatch { .. }
        ));
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
            refinement
                .single_region()
                .expect("the square fixture realizes its occurrence in one region")
                .canonical_identity()
        );
    }

    #[test]
    fn reusable_content_is_separate_from_occurrence_identity() {
        let scalars = scalar_registry();
        let frozen = square_registry();
        let resolved = frozen
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(
                InputKey::new("shared-input").unwrap(),
                Shape::from_dims([LENGTH]),
            )
            .unwrap();
        let first_result =
            tiler_ir::semantic::F32Multiply::apply(&mut builder, input, input).unwrap();
        let second_result =
            tiler_ir::semantic::F32Multiply::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("first").unwrap(), first_result)
            .unwrap();
        builder
            .output(OutputKey::new("second").unwrap(), second_result)
            .unwrap();
        let program = builder.build().unwrap();
        let first_subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        let second_subject = IndexRefinementSubject::derive(
            &program,
            program.operations().nth(1).unwrap().id(),
            contract(),
        )
        .unwrap();

        let first = refine_index_region(&resolved, &first_subject, &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");
        let second = refine_index_region(&resolved, &second_subject, &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");

        // Same operation, interface, and region: identical reusable content.
        assert_eq!(first.content().identity(), second.content().identity());
        assert_eq!(
            first.content().realization_identity(),
            second.content().realization_identity()
        );
        // Different semantic source: distinct occurrence bindings.
        assert_ne!(first.identity(), second.identity());
        assert_eq!(first.receipt().graph(), second.receipt().graph());
        assert_ne!(first.receipt().occurrence(), second.receipt().occurrence());
        assert_ne!(first.receipt().identity(), second.receipt().identity());
    }

    #[test]
    fn a_verifier_from_another_scalar_snapshot_mints_no_receipt() {
        let lowering_scalars = scalar_registry();
        let verifier_scalars = scalar_registry_with_extra_definition();
        assert_ne!(
            lowering_scalars.snapshot_identity(),
            verifier_scalars.snapshot_identity()
        );
        let resolved = square_registry()
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let subject = square_occurrence(b"scalar-snapshot-mismatch");
        let realization = emit_realization(&resolved, &subject, &lowering_scalars).unwrap();
        let realizations =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic(), verifier_scalars).unwrap();
        let resolution = realizations.resolve(&subject).unwrap();

        assert_eq!(
            resolution
                .verify(resolved.authority().refinement(), realization.final_stage())
                .unwrap_err(),
            IndexRefinementVerificationError::ScalarSnapshotMismatch
        );
    }

    #[test]
    fn an_add_region_cannot_mint_a_multiply_receipt() {
        let scalars = scalar_registry();
        let emitted = [scalar_key("multiply"), scalar_key("add")];
        let square = index_registry(Arc::new(PointwiseSquare { length: LENGTH }), &emitted)
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let add = index_registry(Arc::new(PointwiseAdd { length: LENGTH }), &emitted)
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let occurrence = square_occurrence(b"same-occurrence");

        let square = refine_index_region(&square, &occurrence, &scalars)
            .unwrap()
            .into_refined()
            .expect("the square fixture discharges every predicate");
        let error = refine_index_region(&add, &occurrence, &scalars).unwrap_err();

        assert!(matches!(error, RefinementError::IrVerifier(_)));
        assert_eq!(square.receipt().graph(), occurrence.graph());
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
                refinement
                    .single_region()
                    .expect("the square fixture realizes its occurrence in one region"),
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
        let _candidate = outcome
            .whole_program_candidate()
            .expect("the single multiply is its own whole-program region");
        let scalars = scalar_registry();
        let resolved = square_registry()
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let occurrence = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        let refinement = refine_index_region(&resolved, &occurrence, &scalars)
            .unwrap()
            .into_refined()
            .expect("the fixture discharges every index-domain predicate");
        assert_eq!(
            refinement.receipt().graph(),
            program.semantic_identity().graph()
        );
        assert_eq!(
            refinement.receipt().occurrence(),
            SemanticOccurrence::new(0)
        );
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
        let mut builder =
            LoweringCapabilityRegistryBuilder::new(semantic(), scalar_registry()).unwrap();
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
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([LENGTH]))
            .unwrap();
        let result = tiler_ir::semantic::F32Add::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = builder.build().unwrap();
        let occurrence = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
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

    #[test]
    fn refinement_operand_errors_preserve_expanded_boundary_semantics() {
        assert_eq!(
            RefinementError::OperandArity {
                region_inputs: 1,
                expanded_inputs: 3,
            }
            .to_string(),
            "region declares 1 inputs for 3 expanded semantic input boundaries"
        );
        assert_eq!(
            RefinementError::OperandInterface { position: 2 }.to_string(),
            "region input 2 does not match its expanded semantic input boundary"
        );
        assert_eq!(
            RefinementError::EmptyEncodedOperandComponents { input: 2 }.to_string(),
            "encoded semantic input 2 declares no component boundaries"
        );
        assert_eq!(
            RefinementError::OperandBindingsTooLarge {
                actual: 17_408,
                limit: tiler_ir::index::MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
            }
            .to_string(),
            "expanded operand bindings 17408 exceed receipt limit 16384"
        );

        let registry = square_registry();
        let capability = registry
            .resolve_index_access(&multiply_f32_op(), &binary_signature())
            .unwrap();
        let subject = square_occurrence(b"empty-encoded-components");
        assert_eq!(
            map_ir_verifier_error(
                IndexRefinementVerificationError::EmptyEncodedOperandComponents { input: 0 },
                &capability,
                &subject,
            ),
            RefinementError::EmptyEncodedOperandComponents { input: 0 }
        );
        assert_eq!(
            map_ir_verifier_error(
                IndexRefinementVerificationError::OperandBindingsTooLarge {
                    actual: 17_408,
                    limit: tiler_ir::index::MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
                },
                &capability,
                &subject,
            ),
            RefinementError::OperandBindingsTooLarge {
                actual: 17_408,
                limit: tiler_ir::index::MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
            }
        );
    }
}
