//! Physical implementation frontier: a checked local authority for one legal
//! region on one target profile.
//!
//! Region formation proposes candidates and fusion legality proves one fuses
//! without changing the numerical contract; this module answers a different,
//! strictly *local* question. For one legal region and one target profile it
//! enumerates the physical *implementations* proposed by one or more physical
//! providers and returns the bounded, non-dominated set that is both intrinsically
//! valid and hard-feasible (the [`ImplementationFrontier`], per
//! `docs/compiler/fusion-and-scheduling.md` and the `Implementation frontier`
//! glossary entry).
//!
//! The design keeps four concerns the correctness contract insists on separating:
//!
//! - **Additive multi-provider alternatives, not a singular-capability ambiguity.**
//!   Several providers may each contribute an implementation of the *same* region.
//!   Unlike the lowering-capability registry — where two providers claiming one
//!   occurrence is a resolution ambiguity — here they are legitimate additive
//!   alternatives retained side by side. The provider dimension is a list; the
//!   proposal *body* is an additive sum type ([`ProposalBody`]).
//! - **Every proposal re-enters ordinary checked verification.** A provider is
//!   trusted but never believed: each [`ProposalBody::ScheduledKernel`] is
//!   resubmitted through [`crate::physical::verify_schedule_with_feasibility`],
//!   which runs whole-region intrinsic verification (ADR 0007/0071), the
//!   request-subject binding, and the single hard-feasibility decision (ADR 0043)
//!   before the proposal is admitted. A provider cannot smuggle unverified IR.
//! - **Hard infeasibility is not cost.** A proposal whose exact resource
//!   requirements a target cannot satisfy is a typed [`FrontierRejection::Infeasible`]
//!   naming the disproved capability predicate — never an expensive plan. Cost is
//!   an estimate retained for later selection and used only to prune dominated
//!   feasible proposals; it can neither prove nor disprove feasibility.
//! - **Malformed compiler output is not a valid no-plan result.** A provider that
//!   emits structurally invalid IR is a compiler fault and fails the whole
//!   enumeration closed with [`FrontierError`]. An enumeration that simply finds
//!   no feasible implementation returns `Ok` with an empty admitted set — a
//!   legitimate local result, distinct from an error and distinct from a claim of
//!   global coverage.
//!
//! A frontier is a *local* authority. Its enumeration does not depend on a
//! complete cover and does not prove global coverage; joining independent legal
//! covers with compatible per-region frontiers is the later complete
//! physical-plan-selection authority.
//!
//! The bounded profile admits checked [`ProposalBody::ScheduledKernel`] and
//! [`ProposalBody::KernelSubprogram`] proposals and explicitly rejects the
//! reserved [`ProposalBody::View`] variant while keeping the additive
//! sum-type/provider seam. [`ProposalBody::OpaqueCall`] is admitted too, by a
//! different route: a call is declared and registered ahead of enumeration
//! through [`crate::call_declaration`] and [`crate::call_registry`], and a
//! proposal names an already-checked registration rather than carrying a body
//! this module verifies. Both authorities are crate-private, so opaque
//! admission is not an out-of-crate seam.
//!
//! A subprogram is what makes one semantic occurrence realizable by *several*
//! dispatches: a scheduled kernel is one region and therefore one dispatch, so a
//! split reduction has no spelling in it. Its stages re-enter the same checked
//! verification each scheduled kernel does, and the chain they form is derived
//! and checked rather than declared.
//!
//! A provider also reports the strategies it *considered and withheld*
//! ([`DeclinedStrategy`]). Without that channel an enumeration cannot distinguish
//! "this provider does not implement splitting" from "this request's extents
//! admit no split", and the split's absence is unexplainable — which is the one
//! thing an explainable frontier may not be.
//!
//! Every item here is a reviewed *draft* boundary, not a stable compiler API,
//! until Tom accepts the exact interface.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::schedule::{AccessMode, ResourceRequirements, ScheduledRegion, TensorRole};
use tiler_ir::semantic::ProviderIdentity;

use crate::boundary::{
    AdmittedMemoryDomains, AvailabilityGuarantee, AvailabilityRequirement, ByteAlignment,
    ExecutionAffinity, GuaranteedProperties, GuaranteedProperty, LayoutGuarantee,
    LayoutRequirement, MaterializationForm, MemoryDomainClass, RequiredProperties,
    RequiredProperty, StorageEncoding, VisibilityGuarantee, VisibilityRequirement,
};
use crate::call_declaration::{GuaranteeError, OpaqueCallDeclaration, WorkScaling};
use crate::call_registry::{OpaqueCallProposal, OpaqueCallRegistry, RegisteredCall};
use crate::physical::{
    AdmissionEvidence, PhysicalError, ResourceVerdict, VerifiedScheduledRegion,
    verify_schedule_with_feasibility,
};
use crate::region::SemanticMemberId;
use crate::request::{TargetProfile, TargetProfileKey, VerifiedTargetRequest};
use crate::target::feasibility::{FeasibilityError, RejectionCause, ResolvedPredicate};
use crate::target::honourability::UnhonouredDimension;

/// The single structural cost model the bounded P0 frontier attributes estimates
/// to. It matches the pipeline's structural cost model so a later selector can
/// compare frontier estimates without a model reconciliation.
const COST_MODEL_KEY: &str = "tiler.cost.structural.v1";
/// Canonical domain-separation tag for a physical implementation proposal.
const PROPOSAL_IDENTITY_TAG: &[u8] = b"tiler.compiler.physical-implementation-proposal.v2\0";

/// Which additive proposal-body variant a physical provider offered.
///
/// The declaration order and the encoded tag agree, so the derived total order
/// used for deterministic identity and reporting matches the serialized tag; a
/// reordered variant cannot silently keep its encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PhysicalProposalKind {
    /// A checked scheduled kernel over one bounded index region. Admitted.
    ScheduledKernel,
    /// An ordered chain of dispatches realizing one region subject. Admitted;
    /// every stage re-enters the same checked verification a single kernel does.
    KernelSubprogram,
    /// An opaque physical call. Admitted only by naming a registration the
    /// crate-private call registry already checked, never by carrying a body.
    OpaqueCall,
    /// A metadata-only view. Reserved; the one body variant still rejected.
    View,
}

impl PhysicalProposalKind {
    /// Returns the stable discriminant shared by ordering and encoding.
    const fn tag(self) -> u8 {
        match self {
            Self::ScheduledKernel => 1,
            Self::KernelSubprogram => 2,
            Self::OpaqueCall => 3,
            Self::View => 4,
        }
    }

    /// Returns the stable presentation name of the proposal kind.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::ScheduledKernel => "scheduled-kernel",
            Self::KernelSubprogram => "kernel-subprogram",
            Self::OpaqueCall => "opaque-call",
            Self::View => "view",
        }
    }
}

impl fmt::Display for PhysicalProposalKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// One stage of a proposed kernel subprogram.
///
/// The provider states the region *and* the semantic occurrences that stage
/// claims, because a subprogram's stages do not claim the subject uniformly: a
/// split reduction's partial pass claims the reduction occurrence and its final
/// pass claims none, since that occurrence is already covered and claiming it
/// twice would double-cover the graph.
///
/// Neither field is believed. Each region is resubmitted through
/// [`verify_schedule_with_feasibility`] with the members declared here, and that
/// path's request-subject binding is what decides whether this exact region may
/// claim exactly these occurrences — so a provider that mislabels a pass is
/// rejected by the same authority that checks a single-kernel proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubprogramStage {
    region: ScheduledRegion,
    semantic_members: Vec<SemanticMemberId>,
}

impl SubprogramStage {
    /// Builds one proposed stage from its region and the members it claims.
    pub(crate) const fn new(
        region: ScheduledRegion,
        semantic_members: Vec<SemanticMemberId>,
    ) -> Self {
        Self {
            region,
            semantic_members,
        }
    }
}

/// A proposed multi-dispatch implementation of one region subject.
///
/// This is what makes "one semantic occurrence realized by two dispatches"
/// expressible at all: a [`ProposalBody::ScheduledKernel`] is one region and
/// therefore one dispatch, so a split reduction has no spelling in it. The
/// stages are an *ordered chain* — each stage's owning write is the next
/// stage's input, and only the last stage's write leaves the subprogram — which
/// is the structure [`derive_subprogram_boundary_contract`] checks rather than
/// assumes.
///
/// A subprogram is not a nested region cover: it covers exactly the region
/// subject the frontier is enumerating for, and the union of its stages'
/// claimed members must be that subject exactly. Two dispatches that between
/// them covered a different occurrence set would be a different region, not an
/// implementation of this one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelSubprogram {
    stages: Vec<SubprogramStage>,
}

impl KernelSubprogram {
    /// Builds a subprogram from its ordered stages.
    pub(crate) const fn new(stages: Vec<SubprogramStage>) -> Self {
        Self { stages }
    }
}

/// A minimal typed placeholder for a reserved proposal body.
///
/// It preserves the additive seam without asserting any of the contract the
/// reserved variant will eventually carry: the descriptor is echoed in the
/// rejection diagnostic but is otherwise uninterpreted. [`ProposalBody::View`]
/// is the one body still carrying it; the opaque-call payload it once stood in
/// for is now the typed [`OpaqueCallProposal`], and a subprogram carries its
/// verified stages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReservedProposalSeam {
    descriptor: &'static str,
}

#[allow(
    dead_code,
    reason = "reserved additive seam preserved so an unsupported body rejects explicitly instead of being silently approximated"
)]
impl ReservedProposalSeam {
    /// Wraps an uninterpreted descriptor for a reserved proposal body.
    pub(crate) const fn new(descriptor: &'static str) -> Self {
        Self { descriptor }
    }

    /// Returns the uninterpreted reserved-body descriptor.
    pub(crate) const fn descriptor(&self) -> &'static str {
        self.descriptor
    }
}

/// The additive body of one proposed physical implementation.
///
/// This is the sum-typed seam the mature model grows: the bounded profile
/// implements [`Self::ScheduledKernel`], [`Self::KernelSubprogram`], and
/// [`Self::OpaqueCall`], and reserves [`Self::View`] so an unsupported body
/// rejects explicitly instead of being silently approximated.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "reserved additive seam preserved so an unsupported body rejects explicitly instead of being silently approximated"
)]
pub(crate) enum ProposalBody {
    /// A checked scheduled region carrying a minimal serial schedule. The
    /// frontier resubmits it through ordinary intrinsic + feasibility verification.
    ///
    /// The region is boxed so the scheduled-kernel payload does not inflate every
    /// reserved seam variant to its size.
    ScheduledKernel(Box<ScheduledRegion>),
    /// An ordered chain of dispatches realizing one region subject.
    ///
    /// Boxed for the same reason as [`Self::ScheduledKernel`]: the payload is
    /// several regions and must not inflate every other variant.
    KernelSubprogram(Box<KernelSubprogram>),
    /// An opaque physical call, named by its registered identity.
    ///
    /// The provider proposes an *identity* rather than the call itself, so
    /// registration is the authority on which calls exist: a provider cannot
    /// propose one it never registered. A registered, well-bound call whose
    /// contract derives, whose work resolves, and whose resources the target
    /// proves is **admitted**; each failure on that path is a distinct typed
    /// rejection naming what was wrong.
    OpaqueCall(Box<OpaqueCallProposal>),
    /// A metadata-only view. Reserved; the bounded frontier rejects it.
    View(ReservedProposalSeam),
}

impl ProposalBody {
    /// Returns the proposal kind of this body.
    pub(crate) const fn kind(&self) -> PhysicalProposalKind {
        match self {
            Self::ScheduledKernel(_) => PhysicalProposalKind::ScheduledKernel,
            Self::KernelSubprogram(_) => PhysicalProposalKind::KernelSubprogram,
            Self::OpaqueCall(_) => PhysicalProposalKind::OpaqueCall,
            Self::View(_) => PhysicalProposalKind::View,
        }
    }
}

/// The typed predicate stating which target profiles a proposal applies to.
///
/// Applicability is a coarse pre-feasibility gate: it says a proposal even claims
/// to target a profile. It is distinct from feasibility, which decides whether an
/// applicable proposal's resource requirements are satisfiable. A proposal that
/// does not apply to the assessed target is never enumerated for it and is never
/// an error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetApplicability {
    /// Governed target-profile keys, canonical ascending, unique.
    target_profile_keys: Vec<TargetProfileKey>,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl TargetApplicability {
    /// Builds an applicability predicate over a set of governed target keys.
    ///
    /// The keys are normalized to a canonical, deduplicated ascending order so
    /// two predicates over the same key set share one identity encoding.
    pub(crate) fn for_targets(keys: impl IntoIterator<Item = TargetProfileKey>) -> Self {
        let mut target_profile_keys: Vec<TargetProfileKey> = keys.into_iter().collect();
        target_profile_keys.sort_unstable();
        target_profile_keys.dedup();
        Self {
            target_profile_keys,
        }
    }

    /// Returns whether the proposal applies to `target_profile_key`.
    fn applies_to(&self, target_profile_key: &TargetProfileKey) -> bool {
        self.target_profile_keys.contains(target_profile_key)
    }

    /// Returns the governed target-profile keys in canonical order.
    pub(crate) fn target_profile_keys(&self) -> &[TargetProfileKey] {
        &self.target_profile_keys
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.target_profile_keys.len());
        for key in &self.target_profile_keys {
            push_slice(output, key.as_str().as_bytes());
        }
    }
}

/// A structural cost *estimate* for one proposed implementation.
///
/// A cost estimate is never a feasibility input: it can neither prove nor
/// disprove that a proposal fits a target. It carries an explicit model key so a
/// later selector knows exactly which model produced it, and it is used only to
/// prune strictly dominated feasible proposals from the local frontier. The
/// bounded profile attributes every estimate to [`COST_MODEL_KEY`]; a proposal
/// attributing its estimate to any other model is malformed compiler output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PhysicalCostEstimate {
    model_key: &'static str,
    dispatch_count: u32,
    launched_threads: u64,
    temporary_bytes: u64,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl PhysicalCostEstimate {
    /// Builds a cost estimate attributed to an explicit cost-model key.
    pub(crate) const fn new(
        model_key: &'static str,
        dispatch_count: u32,
        launched_threads: u64,
        temporary_bytes: u64,
    ) -> Self {
        Self {
            model_key,
            dispatch_count,
            launched_threads,
            temporary_bytes,
        }
    }

    /// Builds a cost estimate under the governed structural cost model.
    pub(crate) const fn structural(
        dispatch_count: u32,
        launched_threads: u64,
        temporary_bytes: u64,
    ) -> Self {
        Self::new(
            COST_MODEL_KEY,
            dispatch_count,
            launched_threads,
            temporary_bytes,
        )
    }

    /// Returns the cost-model key this estimate is attributed to.
    pub(crate) const fn model_key(&self) -> &'static str {
        self.model_key
    }

    /// Returns the estimated dispatch count.
    pub(crate) const fn dispatch_count(&self) -> u32 {
        self.dispatch_count
    }

    /// Returns the estimated launched-thread count.
    pub(crate) const fn launched_threads(&self) -> u64 {
        self.launched_threads
    }

    /// Returns the estimated temporary-allocation bytes.
    pub(crate) const fn temporary_bytes(&self) -> u64 {
        self.temporary_bytes
    }

    /// Returns whether this estimate strictly dominates `other`.
    ///
    /// Domination is the standard Pareto relation over the structural dimensions:
    /// no dimension is worse and at least one is strictly better. Estimates from
    /// different cost models are incomparable, so neither dominates the other.
    fn dominates(&self, other: &Self) -> bool {
        if self.model_key != other.model_key {
            return false;
        }
        let no_worse = self.dispatch_count <= other.dispatch_count
            && self.launched_threads <= other.launched_threads
            && self.temporary_bytes <= other.temporary_bytes;
        let strictly_better = self.dispatch_count < other.dispatch_count
            || self.launched_threads < other.launched_threads
            || self.temporary_bytes < other.temporary_bytes;
        no_worse && strictly_better
    }
}

/// ADR 0047's "ownership": how a producer owns the writes to a boundary value.
///
/// This is a guarantee-side qualifier and not a property dimension, because it
/// has no requirement counterpart: a consumer states an
/// [`AccessMode`], not an ownership. ADR 0047 lists the two on opposite sides
/// for the same reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryOwnership {
    /// The implementation writes every owned output position exactly once, so the
    /// tensor is produced totally and race-free (backed by the region's ownership
    /// proof).
    TotalRaceFreeWrite,
}

impl BoundaryOwnership {
    const fn tag(self) -> u8 {
        match self {
            Self::TotalRaceFreeWrite => 1,
        }
    }
}

/// One boundary tensor an implementation requires, with the typed properties it
/// requires of it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryRequirement {
    tensor: TensorRole,
    access: AccessMode,
    properties: RequiredProperties,
}

impl BoundaryRequirement {
    /// Returns the boundary tensor role the requirement is over.
    pub(crate) const fn tensor(&self) -> TensorRole {
        self.tensor
    }

    /// Returns ADR 0047's requirement-side access mode.
    pub(crate) const fn access(&self) -> AccessMode {
        self.access
    }

    /// Returns the typed properties the incoming value must have.
    pub(crate) const fn properties(&self) -> &RequiredProperties {
        &self.properties
    }
}

/// One boundary tensor an implementation guarantees, with the typed properties
/// it guarantees of it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryGuarantee {
    tensor: TensorRole,
    ownership: BoundaryOwnership,
    properties: GuaranteedProperties,
}

impl BoundaryGuarantee {
    /// Returns the boundary tensor role the guarantee is over.
    pub(crate) const fn tensor(&self) -> TensorRole {
        self.tensor
    }

    /// Returns ADR 0047's guarantee-side ownership.
    pub(crate) const fn ownership(&self) -> BoundaryOwnership {
        self.ownership
    }

    /// Returns the typed properties the outgoing value has.
    pub(crate) const fn properties(&self) -> &GuaranteedProperties {
        &self.properties
    }
}

/// The typed boundary contract of one admitted implementation.
///
/// The requirements are the boundary tensors the implementation consumes; the
/// guarantees are the boundary tensors it produces. Both are *derived* from the
/// verified region — never taken from the provider — so a later cover selector
/// can compose regions by matching a producer's guarantee to a consumer's
/// requirement on trustworthy structural facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryContract {
    requirements: Vec<BoundaryRequirement>,
    guarantees: Vec<BoundaryGuarantee>,
}

impl BoundaryContract {
    /// Returns the boundary tensors the implementation requires.
    pub(crate) fn requirements(&self) -> &[BoundaryRequirement] {
        &self.requirements
    }

    /// Returns the boundary tensors the implementation guarantees.
    pub(crate) fn guarantees(&self) -> &[BoundaryGuarantee] {
        &self.guarantees
    }

    /// Whether this contract's requirements are no stronger than `other`'s and
    /// its guarantees are at least as strong, at every boundary tensor.
    ///
    /// This is the boundary half of the accepted dominance relation: "its
    /// applicability covers the other's, its target and boundary requirements are
    /// no stronger, its guarantees are at least as strong".
    ///
    /// The two contracts are paired *positionally*, and the pairing is rejected
    /// unless the two sequences of side qualifiers agree entry for entry. Both
    /// are derived in the verified region's access order, which is deterministic
    /// and already part of `CanonicalScheduledRegionIdentity`, so two
    /// implementations of one region always pair exactly. Matching by tensor role
    /// instead would be wrong rather than merely loose: a region reading two
    /// `Input` boundaries has two entries under one role, and a search for the
    /// role would compare one of them against both — a mis-pairing that can
    /// report dominance where none holds. A contract whose sequence differs
    /// describes a different boundary, and neither side dominates.
    fn subsumes(&self, other: &Self) -> bool {
        let requirements_no_stronger = self.requirements.len() == other.requirements.len()
            && self
                .requirements
                .iter()
                .zip(&other.requirements)
                .all(|(mine, theirs)| {
                    mine.tensor == theirs.tensor
                        && mine.access == theirs.access
                        && mine.properties.is_no_stronger_than(&theirs.properties)
                });
        let guarantees_at_least_as_strong = self.guarantees.len() == other.guarantees.len()
            && self
                .guarantees
                .iter()
                .zip(&other.guarantees)
                .all(|(mine, theirs)| {
                    mine.tensor == theirs.tensor
                        && mine.ownership == theirs.ownership
                        && mine.properties.is_at_least_as_strong_as(&theirs.properties)
                });
        requirements_no_stronger && guarantees_at_least_as_strong
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.requirements.len());
        for requirement in &self.requirements {
            push_tensor_role(output, requirement.tensor);
            output.push(access_mode_tag(requirement.access));
            requirement.properties.encode(output);
        }
        push_len(output, self.guarantees.len());
        for guarantee in &self.guarantees {
            push_tensor_role(output, guarantee.tensor);
            output.push(guarantee.ownership.tag());
            guarantee.properties.encode(output);
        }
    }
}

/// The bounded profile's single symbolic execution affinity.
///
/// ADR 0047's initial execution profile is one symbolic affinity, one live
/// device, and one ordered command stream, with every stage, temporary, and
/// output using that affinity. A second affinity is what a target profile would
/// declare, and it is what makes transfer enforcers reachable.
const BOUNDED_AFFINITY: ExecutionAffinity = ExecutionAffinity::PRIMARY;

/// Rule code for a region whose resources deny the address space its own
/// boundary tensors are bound in.
const NO_BOUNDARY_DOMAIN_RULE: &str = "boundary-domain-undetermined";

/// Derives the boundary contract of a verified scheduled region.
///
/// Each read access contributes a requirement on its boundary tensor; the single
/// owning write contributes a guarantee on its boundary tensor. The intrinsic
/// verifier already proved the write is a total, race-free ownership, so the
/// ownership qualifier is sound.
///
/// The typed properties are derived and never declared, and each has a stated
/// source:
///
/// - **layout** is dense row-major on both sides. `tiler_ir::kernel` linearizes a
///   contributor address as "the row-major linearization" of the logical
///   coordinates and the reference evaluator holds elements "in logical row-major
///   order", so both `LogicalAccess::LinearIdentity` and
///   `LogicalAccess::ReductionContributor` already address a dense row-major
///   value. A schedule whose accesses were not dense would need a layout the
///   scheduled-region IR cannot express today;
/// - **encoding** is unpacked. The bounded profile is strict `f32` throughout, so
///   no boundary value is sub-byte packed;
/// - **alignment** is the natural `f32` alignment, for the same reason and with
///   the same bound: `ScheduledRegion` carries no resolved element type, so a
///   widened dtype vocabulary must derive this from the value rather than the
///   profile;
/// - **materialization** is a materialized buffer. Each access is a distinct
///   buffer binding — `derive_requirements` counts `accesses.len()` bindings —
///   and the bounded frontier admits no view or opaque body;
/// - **affinity** is the bounded profile's single affinity, per ADR 0047;
/// - **memory domain** is read from `ResourceRequirements::requires_device_memory`
///   rather than assumed, so a profile that stops requiring an explicitly
///   addressable device space changes the derivation instead of silently keeping
///   a stale one;
/// - **availability** is the producing dispatch on both sides. One scheduled
///   region describes one kernel (ADR 0007), so its guarantee is its own dispatch
///   and its requirement is whichever dispatch produced the value it reads;
/// - **visibility** is coherent on the producing affinity. In the single-affinity
///   profile a producer and a consumer share it, so no coherence action is owed;
///   a second affinity is what makes
///   [`VisibilityGuarantee::RequiresExplicitCoherenceAction`] reachable.
///
/// # Errors
///
/// Returns the rule code of a malformed derivation: a region that binds boundary
/// tensors while its resources deny needing an explicitly addressable device
/// address space names no domain its boundary values could live in, and that is
/// incoherent compiler output rather than an unsatisfiable plan.
fn derive_boundary_contract(
    verified: &VerifiedScheduledRegion,
) -> Result<BoundaryContract, &'static str> {
    let region = verified.region();
    let resources = verified.requirements();
    if !region.index.accesses.is_empty() && !resources.requires_device_memory {
        return Err(NO_BOUNDARY_DOMAIN_RULE);
    }
    let mut requirements = Vec::new();
    let mut guarantees = Vec::new();
    for access in &region.index.accesses {
        if access.ownership.is_some() {
            guarantees.push(BoundaryGuarantee {
                tensor: access.tensor,
                ownership: BoundaryOwnership::TotalRaceFreeWrite,
                properties: bounded_guarantees(),
            });
        } else {
            requirements.push(BoundaryRequirement {
                tensor: access.tensor,
                access: access.mode,
                properties: bounded_requirements(),
            });
        }
    }
    Ok(BoundaryContract {
        requirements,
        guarantees,
    })
}

/// Rule code for a subprogram whose stages do not form one dispatch chain.
const SUBPROGRAM_NOT_CHAINED_RULE: &str = "subprogram-stages-not-chained";

/// Derives the external boundary contract of a verified subprogram chain.
///
/// A subprogram's boundary is what crosses *its* edge, not the union of its
/// stages' edges: the values it stages internally are produced and consumed
/// entirely within it and are invisible to any cover that places it. Deriving
/// the union instead would report a split reduction as guaranteeing two values
/// and requiring two — which `selection::reconcile_boundaries` correctly refuses
/// as an ambiguous boundary, and which would be the wrong answer rather than a
/// conservative one.
///
/// The chain is *checked*, not assumed. Every non-final stage's owning write
/// must be an [`TensorRole::Intermediate`], and the next stage must read exactly
/// that write's iteration domain; both sides of that match are then internal.
/// Every other access stays external, and the final stage's owning write is the
/// subprogram's single guarantee by construction.
///
/// # Errors
///
/// Returns [`SUBPROGRAM_NOT_CHAINED_RULE`] when a non-final stage publishes
/// something it cannot hand on, or hands on a value the next stage never reads.
/// Both are incoherent compiler output rather than unsatisfiable plans — the
/// frontier already verified each stage individually, so a failure here is a
/// claim about how they compose.
fn derive_subprogram_boundary_contract(
    stages: &[VerifiedScheduledRegion],
) -> Result<BoundaryContract, &'static str> {
    let mut requirements = Vec::new();
    let mut guarantees = Vec::new();
    // The value the previous stage handed on. `None` at the head of the chain,
    // and never carried past the stage that consumes it.
    let mut handoff: Option<&tiler_ir::shape::Shape> = None;
    for (position, stage) in stages.iter().enumerate() {
        let region = stage.region();
        if !region.index.accesses.is_empty() && !stage.requirements().requires_device_memory {
            return Err(NO_BOUNDARY_DOMAIN_RULE);
        }
        let last = position + 1 == stages.len();
        let mut owed = handoff;
        for access in &region.index.accesses {
            if access.ownership.is_some() {
                // Only the last stage's write leaves the subprogram; every
                // earlier one is the handoff the next stage must consume. A
                // non-final stage that publishes something other than an
                // intermediate has nothing to hand on, and a chain claiming
                // otherwise is not a chain.
                if last {
                    guarantees.push(BoundaryGuarantee {
                        tensor: access.tensor,
                        ownership: BoundaryOwnership::TotalRaceFreeWrite,
                        properties: bounded_guarantees(),
                    });
                } else if access.tensor == TensorRole::Intermediate {
                    handoff = Some(&region.index.iteration_shape);
                } else {
                    return Err(SUBPROGRAM_NOT_CHAINED_RULE);
                }
                continue;
            }
            if owed.is_some_and(|shape| {
                access.tensor == TensorRole::Intermediate
                    && access_domain_shape(region, access) == Some(shape)
            }) {
                owed = None;
                continue;
            }
            requirements.push(BoundaryRequirement {
                tensor: access.tensor,
                access: access.mode,
                properties: bounded_requirements(),
            });
        }
        // A stage handed a value it never reads leaves that value staged with
        // no consumer, which is a leak the cover cannot see and the program
        // assembler would have to invent an owner for.
        if owed.is_some() {
            return Err(SUBPROGRAM_NOT_CHAINED_RULE);
        }
    }
    Ok(BoundaryContract {
        requirements,
        guarantees,
    })
}

/// Returns the logical domain shape one access addresses, when it names one.
///
/// A linear access addresses the region's own iteration domain; a reduction
/// contributor access addresses its declared input shape. The remaining maps
/// carry no shape a chain could be matched on, so they answer `None` and a
/// subprogram built over them fails the chain check rather than matching by
/// accident.
///
/// The wildcard is the fail-closed direction: `LogicalAccess` is
/// `#[non_exhaustive]`, and a map added upstream reaches it and declines to
/// name a domain, which refuses the chain. Naming a domain by guess is what
/// would be unsafe here — it would splice two stages that address different
/// values.
fn access_domain_shape<'a>(
    region: &'a ScheduledRegion,
    access: &'a tiler_ir::schedule::Access,
) -> Option<&'a tiler_ir::shape::Shape> {
    match &access.map {
        tiler_ir::schedule::LogicalAccess::LinearIdentity => Some(&region.index.iteration_shape),
        tiler_ir::schedule::LogicalAccess::ReductionContributor { input_shape, .. } => {
            Some(input_shape)
        }
        tiler_ir::schedule::LogicalAccess::ScalarBroadcast
        | tiler_ir::schedule::LogicalAccess::PackedU4LsbZeroTail { .. }
        | _ => None,
    }
}

/// Returns the peak resource requirement across a subprogram's stages.
///
/// The **peak**, not the sum. Every axis here is checked against a per-dispatch
/// device bound — grid threads, workgroup threads, buffer bindings, threadgroup
/// bytes — and a subprogram's stages are dispatched in sequence, so summing them
/// would report a requirement no point in its execution ever has. The numerical
/// dimensions are taken from the first stage because every stage of an admitted
/// subprogram implements the same request contract, which
/// [`verify_schedule_with_feasibility`] proved for each of them separately.
fn subprogram_resources(stages: &[VerifiedScheduledRegion]) -> Option<ResourceRequirements> {
    let mut peak = stages.first()?.requirements();
    for stage in &stages[1..] {
        let stage = stage.requirements();
        peak.buffer_bindings = peak.buffer_bindings.max(stage.buffer_bindings);
        peak.threads_per_workgroup = peak.threads_per_workgroup.max(stage.threads_per_workgroup);
        peak.local_memory_bytes = peak.local_memory_bytes.max(stage.local_memory_bytes);
        peak.requires_device_memory |= stage.requires_device_memory;
    }
    Some(peak)
}

/// The typed properties the bounded profile's regions require of an input.
///
/// # Panics
///
/// Panics only if these compile-time constants violate the property model's own
/// well-formedness rules, which no reachable input can cause.
fn bounded_requirements() -> RequiredProperties {
    RequiredProperties::new([
        RequiredProperty::StorageLayout(LayoutRequirement::DenseRowMajor),
        RequiredProperty::StorageEncoding(StorageEncoding::Unpacked),
        RequiredProperty::Alignment(ByteAlignment::F32_NATURAL),
        RequiredProperty::Materialization(MaterializationForm::MaterializedBuffer),
        RequiredProperty::ExecutionAffinity(BOUNDED_AFFINITY),
        RequiredProperty::MemoryDomain(
            AdmittedMemoryDomains::new([MemoryDomainClass::Device])
                .expect("a one-class admitted set is non-empty"),
        ),
        RequiredProperty::Availability(AvailabilityRequirement::AfterProducingDispatch),
        RequiredProperty::Visibility(VisibilityRequirement::ReadableOnRequiringAffinity),
    ])
    .expect("the bounded profile's requirement set is well formed")
}

/// The typed properties the bounded profile's regions guarantee of an output.
///
/// # Panics
///
/// Panics under the same unreachable condition as [`bounded_requirements`].
fn bounded_guarantees() -> GuaranteedProperties {
    GuaranteedProperties::new([
        GuaranteedProperty::StorageLayout(LayoutGuarantee::DenseRowMajor),
        GuaranteedProperty::StorageEncoding(StorageEncoding::Unpacked),
        GuaranteedProperty::Alignment(ByteAlignment::F32_NATURAL),
        GuaranteedProperty::Materialization(MaterializationForm::MaterializedBuffer),
        GuaranteedProperty::ExecutionAffinity(BOUNDED_AFFINITY),
        GuaranteedProperty::MemoryDomain(MemoryDomainClass::Device),
        GuaranteedProperty::Availability(AvailabilityGuarantee::AfterOwnDispatch),
        GuaranteedProperty::Visibility(VisibilityGuarantee::CoherentOnProducingAffinity),
    ])
    .expect("the bounded profile's guarantee set is well formed")
}

/// Maximum bytes in a physical provider's exact explain subject.
const MAX_PHYSICAL_PROVIDER_EXPLAIN_SUBJECT_BYTES: usize = 255;

/// The provenance of one physical implementation provider.
///
/// It reuses the governed [`ProviderIdentity`] (namespace, name, output-affecting
/// revision) so provider provenance is separated from semantic meaning (ADR 0072)
/// and carries a versioned identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PhysicalProviderProvenance {
    provider: ProviderIdentity,
    explain_subject: String,
}

/// Why a physical provider's complete provenance could not be retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PhysicalProviderProvenanceError {
    provider: ProviderIdentity,
    actual: usize,
    maximum: usize,
}

impl PhysicalProviderProvenance {
    /// Records that proposals were produced by `provider`, refusing an identity
    /// whose exact explain subject cannot be retained.
    pub(crate) fn new(provider: ProviderIdentity) -> Result<Self, PhysicalProviderProvenanceError> {
        let explain_subject = provider.to_string();
        let actual = explain_subject.len();
        let maximum = MAX_PHYSICAL_PROVIDER_EXPLAIN_SUBJECT_BYTES;
        if actual > maximum {
            return Err(PhysicalProviderProvenanceError {
                provider,
                actual,
                maximum,
            });
        }
        Ok(Self {
            provider,
            explain_subject,
        })
    }

    /// Returns the provider identity.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the complete exact provider identity in explain-subject form.
    pub(crate) fn explain_subject(&self) -> &str {
        &self.explain_subject
    }
}

impl fmt::Display for PhysicalProviderProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "physical-provider.provenance-too-long: provider {} needs {} bytes, exceeding {}",
            self.provider, self.actual, self.maximum
        )
    }
}

impl Error for PhysicalProviderProvenanceError {}

/// The complete provenance of one admitted implementation: provider and kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImplementationProvenance {
    provider: PhysicalProviderProvenance,
    kind: PhysicalProposalKind,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl ImplementationProvenance {
    /// Returns the provider that produced the implementation.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        self.provider.provider()
    }

    /// Returns the provider's exact bounded explain subject.
    pub(crate) fn provider_explain_subject(&self) -> &str {
        self.provider.explain_subject()
    }

    /// Returns the additive proposal kind of the implementation.
    pub(crate) const fn kind(&self) -> PhysicalProposalKind {
        self.kind
    }
}

/// One physical implementation a provider proposes for a region.
///
/// The provider declares the body, the applicability predicate, and a cost
/// estimate; it does not declare provider identity (the frontier stamps that from
/// the calling provider so a proposal cannot forge another provider), resource
/// requirements (the frontier derives the exact requirements from the verified
/// region), or the boundary contract (also derived).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImplementationProposal {
    body: ProposalBody,
    applicability: TargetApplicability,
    declared_cost: PhysicalCostEstimate,
}

impl ImplementationProposal {
    /// Builds a proposal from a body, an applicability predicate, and a declared
    /// cost estimate.
    pub(crate) const fn new(
        body: ProposalBody,
        applicability: TargetApplicability,
        declared_cost: PhysicalCostEstimate,
    ) -> Self {
        Self {
            body,
            applicability,
            declared_cost,
        }
    }
}

/// The read-only context a provider receives to propose implementations.
///
/// It exposes the verified target request the region belongs to and the region
/// subject the frontier is a local authority for. A provider builds its schedule
/// from this context; it never mutates it and never gains the raw builders or a
/// way to finalize a region — the host resubmits and verifies every body.
pub(crate) struct ImplementationContext<'a> {
    request: &'a VerifiedTargetRequest,
    subject: &'a FrontierRegionSubject,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl ImplementationContext<'_> {
    /// Returns the verified target request.
    pub(crate) const fn request(&self) -> &VerifiedTargetRequest {
        self.request
    }

    /// Returns the region subject the frontier is being enumerated for.
    pub(crate) const fn subject(&self) -> &FrontierRegionSubject {
        self.subject
    }

    /// Returns the key of the target profile this frontier assesses.
    pub(crate) fn target_profile_key(&self) -> &str {
        self.request.target_profile().profile_key().as_str()
    }
}

/// Why a provider considered a strategy for this subject and did not offer it.
///
/// Each cause is a fact about the *request*, decided before any region is
/// constructed. They are distinct from every [`FrontierRejection`] a proposal
/// earns, because nothing was proposed: the enumeration is complete only if it
/// can also say what it deliberately withheld.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StrategyDeclineCause {
    /// The request's resolved numerical contract forbids a freedom the strategy
    /// consumes.
    ///
    /// Never a cost and never a capability: the caller ruled the transform out,
    /// so no target and no re-planning makes the strategy legal.
    NumericalPermissionRefused {
        /// The canonical key of the refused dimension.
        dimension: &'static str,
    },
    /// The strategy has no admissible shape for this request's extents.
    NoAdmissibleShape {
        /// Stable code naming which shape obligation could not be met.
        rule: &'static str,
        /// The extent that admitted none.
        extent: u64,
    },
    /// The strategy's derived extents or shapes are not representable.
    Unrepresentable {
        /// Stable code naming the unrepresentable quantity.
        rule: &'static str,
    },
}

impl StrategyDeclineCause {
    /// Returns the stable reason code of the decline.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::NumericalPermissionRefused { .. } => "numerical-permission-refused",
            Self::NoAdmissibleShape { rule, .. } | Self::Unrepresentable { rule } => rule,
        }
    }

    fn encode(self, output: &mut Vec<u8>) {
        match self {
            Self::NumericalPermissionRefused { dimension } => {
                output.push(0x01);
                push_slice(output, dimension.as_bytes());
            }
            Self::NoAdmissibleShape { rule, extent } => {
                output.push(0x02);
                push_slice(output, rule.as_bytes());
                output.extend_from_slice(&extent.to_be_bytes());
            }
            Self::Unrepresentable { rule } => {
                output.push(0x03);
                push_slice(output, rule.as_bytes());
            }
        }
    }
}

/// One strategy a provider considered for a subject and withheld, with its
/// reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeclinedStrategy {
    strategy: &'static str,
    cause: StrategyDeclineCause,
}

impl DeclinedStrategy {
    /// Records that `strategy` was considered for this subject and withheld.
    pub(crate) const fn new(strategy: &'static str, cause: StrategyDeclineCause) -> Self {
        Self { strategy, cause }
    }
}

/// Everything one provider has to say about one region subject.
///
/// Proposals and declines are returned together rather than through two calls,
/// because they are two halves of one answer: a provider that offered a serial
/// reduction and withheld a split considered both in the same derivation, and
/// splitting the call would let the two disagree about which request they were
/// answering.
#[derive(Debug, Default)]
pub(crate) struct ProviderOffer {
    proposals: Vec<ImplementationProposal>,
    declined: Vec<DeclinedStrategy>,
}

impl ProviderOffer {
    /// An offer of proposals with nothing withheld.
    pub(crate) const fn proposing(proposals: Vec<ImplementationProposal>) -> Self {
        Self {
            proposals,
            declined: Vec::new(),
        }
    }

    /// Records that a strategy was considered for this subject and withheld.
    pub(crate) fn decline(mut self, declined: DeclinedStrategy) -> Self {
        self.declined.push(declined);
        self
    }
}

/// A statically linked provider that proposes physical implementations of a
/// region on a target profile.
///
/// The provider is trusted, deterministic, and side-effect-free: it depends only
/// on its explicit context and returns zero or more proposals, together with the
/// strategies it considered and withheld. Trust does not mean belief — the host
/// resubmits every scheduled-kernel and subprogram body through the ordinary
/// checked verification path before admitting it.
pub(crate) trait PhysicalImplementationProvider {
    /// Returns this provider's provenance.
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError>;

    /// Proposes physical implementations for the region in `context`.
    ///
    /// An empty offer is legitimate: it means the provider recognizes nothing
    /// about this region and target, which is neither an error nor a
    /// global-coverage claim. An offer that proposes nothing but declines a
    /// named strategy says something stronger — the strategy applied and this
    /// request did not admit it — and that difference is what the frontier
    /// records.
    fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer;
}

/// The region one frontier is a local authority for.
///
/// A region is identified by the exact recognized semantic occurrences it covers
/// (its members) and a stable presentation role. The members are supplied to the
/// checked verification of each proposal, so a provider whose schedule binds to a
/// different region than this subject fails the request-subject binding rather
/// than silently implementing the wrong occurrences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrontierRegionSubject {
    role: &'static str,
    semantic_members: Vec<SemanticMemberId>,
}

impl FrontierRegionSubject {
    /// Builds a region subject from a presentation role and its exact members.
    pub(crate) fn new(role: &'static str, semantic_members: Vec<SemanticMemberId>) -> Self {
        Self {
            role,
            semantic_members,
        }
    }

    /// Returns the stable presentation role of the region.
    pub(crate) const fn role(&self) -> &'static str {
        self.role
    }

    /// Returns the exact recognized semantic occurrences the region covers.
    pub(crate) fn semantic_members(&self) -> &[SemanticMemberId] {
        &self.semantic_members
    }
}

/// Collision-free canonical identity of one admitted implementation.
///
/// It folds the shared-IR region identity, the provider provenance, the proposal
/// kind, the applicability predicate, and the derived boundary contract. It
/// deliberately excludes enumeration order and the cost estimate, so two runs
/// with providers supplied in different orders yield the same identities, while
/// two providers proposing the same region stay distinct through their provenance.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ImplementationProposalIdentity(Vec<u8>);

impl ImplementationProposalIdentity {
    /// Returns the canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One admitted physical implementation on the frontier.
///
/// Holding one is evidence that the provider's proposed body re-entered and
/// passed whole-region intrinsic verification, the request-subject binding, and
/// the hard-feasibility decision for this exact region and target. It carries the
/// verified region, the exact feasibility resources, the resolved feasibility
/// predicates, the derived boundary contract, the retained cost estimate, and the
/// provider provenance.
/// What an admitted implementation actually is.
///
/// `AdmittedImplementation` currently holds a `VerifiedScheduledRegion`
/// directly, which is why `ProposalBody::OpaqueCall` is rejected: an opaque
/// call has no schedule, no index region, and no iteration domain, so there is
/// nothing to put in that field.
///
/// # Why a sum rather than a trait
///
/// A trait would let both bodies answer one interface, and the interface would
/// have to be the *intersection* of what they can say — which is small, and
/// which hides that the difference matters. Lowering a scheduled region and
/// invoking an opaque call are not two implementations of one operation; the
/// second is a call into code this compiler did not produce. A sum makes every
/// consumer state which it handles, and `AGENTS.md`'s requirement that
/// unsupported cases reject explicitly rather than silently approximating is
/// exactly what a trait's shared default would erode.
///
/// # What both can answer, and what only one can
///
/// Both carry semantic members and a target profile key — those identify *what*
/// was implemented and *for where*, which an opaque call has as much as a
/// scheduled region does. Neither an iteration domain nor an access list has
/// any meaning for a call whose body is not modelled, so a consumer that needs
/// one must handle its absence rather than receive a substitute.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the sum AdmittedImplementation's body will become; landed with its tests ahead of the field change and the nine call sites that follow it"
)]
pub(crate) enum ImplementationBody {
    /// A region this compiler scheduled and will lower itself.
    Scheduled(Box<VerifiedScheduledRegion>),
    /// An ordered chain of regions this compiler scheduled and will lower as
    /// several dispatches of one region subject.
    ///
    /// Distinct from [`Self::Scheduled`] rather than a one-or-many collapse of
    /// it, because "how many dispatches realize this subject" is the fact a
    /// consumer must handle: a program assembler binds a different number of
    /// stages, and a cost model counts a different number of launches. Nothing
    /// that needs exactly one region should silently receive the first of
    /// several.
    Subprogram(Vec<VerifiedScheduledRegion>),
    /// A call into code this compiler did not produce.
    Opaque(Box<RegisteredCall>),
}

#[allow(
    dead_code,
    reason = "see the type's own allow: accessors land with the sum, ahead of the consumers that will match on it"
)]
impl ImplementationBody {
    /// The scheduled region, when this is exactly one.
    ///
    /// `Option` rather than a panicking accessor: a consumer that needs a
    /// schedule and receives an opaque call has to say what it does about that,
    /// and the type is where it is made to. A subprogram answers `None` here
    /// deliberately — it *has* regions, but not one, and returning its first
    /// would hand a single-dispatch consumer a plan whose remaining dispatches
    /// it would never emit.
    pub(crate) fn scheduled(&self) -> Option<&VerifiedScheduledRegion> {
        match self {
            Self::Scheduled(region) => Some(region),
            Self::Subprogram(_) | Self::Opaque(_) => None,
        }
    }

    /// Every region this body dispatches, in execution order.
    ///
    /// The accessor a consumer that can handle any dispatch count uses: cost
    /// components fold over it and program assembly binds one stage per entry.
    /// An opaque call still answers `None`, because it has no scheduled region
    /// at all rather than several.
    pub(crate) fn scheduled_stages(&self) -> Option<&[VerifiedScheduledRegion]> {
        match self {
            Self::Scheduled(region) => Some(std::slice::from_ref(region)),
            Self::Subprogram(stages) => Some(stages),
            Self::Opaque(_) => None,
        }
    }

    /// The registered call, when this is one.
    pub(crate) fn opaque(&self) -> Option<&RegisteredCall> {
        match self {
            Self::Opaque(call) => Some(call),
            Self::Scheduled(_) | Self::Subprogram(_) => None,
        }
    }

    /// The stable code naming which kind this is, for typed rejections.
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::Scheduled(_) => "scheduled-region",
            Self::Subprogram(_) => "kernel-subprogram",
            Self::Opaque(_) => "opaque-call",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedImplementation {
    provenance: ImplementationProvenance,
    /// The semantic members this admission implements.
    ///
    /// Held here rather than read through the body because it is a property of
    /// the **admission** — *what* was implemented — and not of how. An opaque
    /// call has members as much as a scheduled region does, while
    /// `RegisteredCall` cannot hold them: a call is registered once and admitted
    /// per region and per target, so one registration would need different
    /// members per admission.
    semantic_members: Vec<SemanticMemberId>,
    /// The target profile this admission is for.
    ///
    /// Here for the same reason as the members: *for where*, which both bodies
    /// have and neither owns.
    target_profile: TargetProfile,
    body: ImplementationBody,
    admission: AdmissionEvidence,
    boundary: BoundaryContract,
    cost: PhysicalCostEstimate,
    identity: ImplementationProposalIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl AdmittedImplementation {
    /// Returns the provider and kind that produced this implementation.
    pub(crate) const fn provenance(&self) -> &ImplementationProvenance {
        &self.provenance
    }

    /// Returns the verified scheduled region backing this implementation.
    /// The semantic members this admission implements.
    pub(crate) fn semantic_members(&self) -> &[SemanticMemberId] {
        &self.semantic_members
    }

    /// The target profile this admission is for.
    pub(crate) fn target_profile_key(&self) -> &str {
        self.target_profile.profile_key().as_str()
    }

    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    /// The scheduled region this admission lowers, when it is one.
    ///
    /// `Option` because an admission may be an opaque call, which has no
    /// schedule. A consumer that needs one must say what it does about the
    /// absence rather than receive a substitute.
    pub(crate) fn scheduled(&self) -> Option<&VerifiedScheduledRegion> {
        self.body.scheduled()
    }

    /// Every region this admission dispatches, in execution order.
    pub(crate) fn scheduled_stages(&self) -> Option<&[VerifiedScheduledRegion]> {
        self.body.scheduled_stages()
    }

    /// What this admission is.
    pub(crate) const fn body(&self) -> &ImplementationBody {
        &self.body
    }

    /// Returns the exact resource requirements used for the feasibility decision.
    pub(crate) fn resources(&self) -> ResourceRequirements {
        // Every body answers, from different authorities: a scheduled region
        // derives its requirements, a subprogram takes the peak across its
        // stages, and an opaque call declares them as proven — which is why the
        // declaration carries `ResourceRequirements` and not the uncertain
        // estimate class. None is defaulted; feasibility must never be told a
        // call needs nothing because nobody said.
        match &self.body {
            ImplementationBody::Scheduled(region) => region.requirements(),
            ImplementationBody::Subprogram(stages) => subprogram_resources(stages)
                .expect("an admitted subprogram has at least one verified stage"),
            ImplementationBody::Opaque(call) => *call.declaration().resources(),
        }
    }

    /// Returns the complete admission evidence, including deferred obligations.
    pub(crate) const fn admission(&self) -> &AdmissionEvidence {
        &self.admission
    }

    /// Returns the derived typed boundary contract.
    pub(crate) const fn boundary(&self) -> &BoundaryContract {
        &self.boundary
    }

    /// Returns the retained cost estimate (never a feasibility input).
    pub(crate) const fn cost(&self) -> PhysicalCostEstimate {
        self.cost
    }

    /// Returns the canonical, order-independent proposal identity.
    pub(crate) const fn identity(&self) -> &ImplementationProposalIdentity {
        &self.identity
    }

    /// Whether this implementation dominates `other` under the accepted relation.
    ///
    /// Boundary subsumption is checked *first* and cost last, so a cheaper
    /// candidate never prunes one that asks less of its producers or offers more
    /// to its consumers. Both are feasible by construction — holding an
    /// [`AdmittedImplementation`] is that evidence — so this relation ranks
    /// retained alternatives and can neither establish nor refute feasibility.
    fn dominates(&self, other: &Self) -> bool {
        self.boundary.subsumes(&other.boundary) && self.cost.dominates(&other.cost)
    }
}

/// A proposal that did not enter the frontier, with a typed reason.
///
/// These are legitimate local dispositions, not compiler faults: an applicable
/// proposal whose resources a target cannot satisfy is [`Self::Infeasible`]; a
/// reserved body variant is [`Self::UnsupportedVariant`]; a proposal that does
/// not target this profile is [`Self::NotApplicable`]. None of them fails the
/// enumeration; malformed compiler output does, through [`FrontierError`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FrontierRejection {
    /// The proposal is applicable and valid but hard-infeasible on this target.
    /// The disproved capability predicate is reported exactly; this is never a
    /// cost.
    Infeasible {
        /// The provider whose proposal was rejected.
        provider: PhysicalProviderProvenance,
        /// The canonical key of the disproved capability axis.
        axis: &'static str,
        /// The amount the proposal required on that axis.
        required: u64,
        /// The amount the target profile made available on that axis.
        available: u64,
    },
    /// The proposal is applicable and valid, but the target declares it cannot
    /// honour a dimension of the request's numerical contract.
    ///
    /// Distinct from [`Self::Infeasible`] because the two say different things
    /// to a caller: a capability rejection means this plan does not fit and
    /// another plan might, while an unhonourable dimension means the target
    /// cannot compute what the caller asked for at all. Neither is ever a cost.
    Unhonourable {
        /// The provider whose proposal was rejected.
        provider: PhysicalProviderProvenance,
        /// The dimension, required behaviour, declared means, honoured
        /// alternative, and declaring profile.
        cause: UnhonouredDimension,
    },
    /// An opaque call proposal was refused, retaining its complete identity,
    /// ordered bindings, and typed cause.
    OpaqueCall {
        /// The physical provider that emitted the proposal.
        provider: PhysicalProviderProvenance,
        /// The exact proposal, including ordered parameter-to-tensor bindings.
        proposal: OpaqueCallProposal,
        /// The typed stage-local refusal.
        cause: OpaqueCallRejectionCause,
    },
    /// A provider considered a named strategy for this subject and withheld it.
    ///
    /// Distinct from every variant above because no proposal was made: those
    /// answer "why was this candidate not admitted", and this answers "why was
    /// there no candidate". Without it, a request whose extents admit no
    /// balanced split and one whose provider does not implement splitting at all
    /// produce byte-identical enumerations, and the split's absence is
    /// unexplainable.
    StrategyDeclined {
        /// The provider that withheld the strategy.
        provider: PhysicalProviderProvenance,
        /// The stable strategy name.
        strategy: &'static str,
        /// The typed reason it was withheld.
        cause: StrategyDeclineCause,
    },
    /// The proposal body is a reserved variant the P0 frontier does not implement.
    UnsupportedVariant {
        /// The provider whose proposal was rejected.
        provider: PhysicalProviderProvenance,
        /// The reserved proposal kind.
        kind: PhysicalProposalKind,
    },
    /// The proposal's applicability predicate excludes this target profile.
    NotApplicable {
        /// The provider whose proposal did not apply.
        provider: PhysicalProviderProvenance,
        /// The proposal kind that did not apply.
        kind: PhysicalProposalKind,
        /// The assessed target profile key the proposal did not target.
        target_profile_key: TargetProfileKey,
    },
}

/// Why one exact opaque call proposal did not enter the frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OpaqueCallRejectionCause {
    /// The proposal's applicability excludes the assessed target.
    NotApplicable {
        /// The target profile assessed.
        target_profile_key: TargetProfileKey,
    },
    /// No registered call owns the proposed identity.
    Unregistered,
    /// The ordered parameter bindings do not satisfy the call ABI.
    MalformedBinding(crate::call_abi::BindingError),
    /// The declaration cannot derive a complete boundary contract.
    ContractUnderivable(GuaranteeError),
    /// The declaration's numerical requirements differ from the request.
    NumericalContractMismatch,
    /// The work-scaling declaration cannot be resolved from this proposal.
    WorkUnresolvable(WorkResolutionError),
    /// A target capability bound rejects the call.
    TargetInfeasible(ResolvedPredicate),
    /// The target cannot honour one required numerical dimension.
    TargetUnhonourable(UnhonouredDimension),
}

impl FrontierRejection {
    fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::Infeasible {
                provider,
                axis,
                required,
                available,
            } => {
                output.push(1);
                encode_provider(output, provider.provider());
                push_slice(output, axis.as_bytes());
                output.extend_from_slice(&required.to_be_bytes());
                output.extend_from_slice(&available.to_be_bytes());
            }
            Self::Unhonourable { provider, cause } => {
                output.push(4);
                encode_provider(output, provider.provider());
                cause.encode(output);
            }
            Self::OpaqueCall {
                provider,
                proposal,
                cause,
            } => {
                output.push(5);
                encode_provider(output, provider.provider());
                encode_opaque_call_proposal(output, proposal);
                encode_opaque_call_cause(output, cause);
            }
            Self::StrategyDeclined {
                provider,
                strategy,
                cause,
            } => {
                output.push(6);
                encode_provider(output, provider.provider());
                push_slice(output, strategy.as_bytes());
                cause.encode(output);
            }
            Self::UnsupportedVariant { provider, kind } => {
                output.push(2);
                encode_provider(output, provider.provider());
                output.push(kind.tag());
            }
            Self::NotApplicable {
                provider,
                kind,
                target_profile_key,
            } => {
                output.push(3);
                encode_provider(output, provider.provider());
                output.push(kind.tag());
                push_slice(output, target_profile_key.as_str().as_bytes());
            }
        }
    }
}

/// The bounded local implementation frontier for one region and target profile.
///
/// The admitted implementations are the feasible, verified proposals in canonical
/// identity order. An empty admitted set is a valid, legitimate local result — no
/// provider offered a feasible implementation — not an error and not a claim
/// about global coverage. The rejections retain every non-admitted proposal with
/// its typed reason for a complete explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImplementationFrontier {
    target_profile: TargetProfile,
    region_role: &'static str,
    admitted: Vec<AdmittedImplementation>,
    rejections: Vec<FrontierRejection>,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl ImplementationFrontier {
    /// Returns the assessed target profile key.
    pub(crate) fn target_profile_key(&self) -> &str {
        self.target_profile.profile_key().as_str()
    }

    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    /// Returns the region presentation role this frontier is an authority for.
    pub(crate) const fn region_role(&self) -> &'static str {
        self.region_role
    }

    /// Returns the admitted implementations in canonical identity order.
    pub(crate) fn admitted(&self) -> &[AdmittedImplementation] {
        &self.admitted
    }

    /// Returns the typed rejections in canonical order.
    pub(crate) fn rejections(&self) -> &[FrontierRejection] {
        &self.rejections
    }

    /// Returns whether no implementation was admitted.
    ///
    /// An empty frontier is a valid local no-plan result, distinct from a
    /// malformed-output [`FrontierError`].
    pub(crate) fn is_empty(&self) -> bool {
        self.admitted.is_empty()
    }

    /// Returns the non-dominated admitted implementations, in canonical order.
    ///
    /// Domination is the accepted relation, not cost alone: an implementation is
    /// removed only when another admitted implementation's *boundary
    /// requirements are no stronger*, its *guarantees are at least as strong*,
    /// and its cost estimate strictly dominates. Cost alone is not sufficient,
    /// because "interesting boundary properties such as useful unit-stride axes
    /// are retained on a bounded Pareto frontier even when they are not locally
    /// cheapest": a cheaper implementation that demands more of its inputs or
    /// delivers less to its consumers is not a replacement for one that does not.
    ///
    /// Domination still runs strictly *after* feasibility admission and never
    /// establishes or refutes feasibility.
    pub(crate) fn non_dominated(&self) -> Vec<&AdmittedImplementation> {
        self.admitted
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                !self
                    .admitted
                    .iter()
                    .enumerate()
                    .any(|(other_index, other)| *index != other_index && other.dominates(candidate))
            })
            .map(|(_, candidate)| candidate)
            .collect()
    }
}

/// A malformed-compiler-output fault during frontier enumeration.
///
/// This is invalid compiler output — a provider that emitted structurally invalid
/// IR or attributed its cost estimate to an ungoverned model — and it fails the
/// whole enumeration closed. It is deliberately distinct from every valid local
/// disposition, including an empty [`ImplementationFrontier`]: a frontier with no
/// admitted implementation is a legitimate no-plan result, whereas a malformed
/// proposal is a bug that must not be silently dropped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FrontierError {
    /// A provider identity cannot be retained exactly in the bounded explain
    /// vocabulary.
    UnrepresentableProviderProvenance {
        /// The typed provenance-construction fault.
        source: PhysicalProviderProvenanceError,
    },
    /// A provider emitted a scheduled-kernel body that failed intrinsic
    /// verification, the request-subject binding, or shape validity — a
    /// non-feasibility [`PhysicalError`].
    MalformedProposal {
        /// The provider whose proposal was invalid.
        provider: ProviderIdentity,
        /// The re-entered checked-verification fault.
        source: PhysicalError,
    },
    /// A provider attributed its cost estimate to an ungoverned cost model.
    MalformedCostProvenance {
        /// The provider whose proposal was invalid.
        provider: ProviderIdentity,
        /// The ungoverned cost-model key the provider declared.
        declared_model_key: &'static str,
    },
    /// A verified region's own facts do not determine a boundary property its
    /// contract must state.
    ///
    /// Distinct from [`Self::MalformedProposal`] because the region passed
    /// intrinsic verification: the inconsistency is between the region's accesses
    /// and its derived resources, which no intrinsic invariant relates.
    UndeterminedBoundaryProperty {
        /// The provider whose proposal could not be described at its boundary.
        provider: ProviderIdentity,
        /// A stable rule code naming the undetermined property.
        rule: &'static str,
    },
    /// The shared feasibility authority rejected an opaque call's profile or
    /// proposal as malformed.
    MalformedOpaqueCallAssessment {
        /// The provider that emitted the call proposal.
        provider: ProviderIdentity,
        /// The exact proposal being assessed.
        proposal: Box<OpaqueCallProposal>,
        /// The shared authority's intrinsic fault.
        source: FeasibilityError,
    },
    /// The shared feasibility authority could neither prove nor reject an
    /// opaque call at the compile-profile phase.
    UnresolvedOpaqueCallAssessment {
        /// The provider that emitted the call proposal.
        provider: ProviderIdentity,
        /// The exact proposal whose proof path was incomplete.
        proposal: Box<OpaqueCallProposal>,
    },
}

impl FrontierError {
    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::UnrepresentableProviderProvenance { .. } => "unrepresentable-provider-provenance",
            Self::MalformedProposal { .. } => "malformed-proposal",
            Self::MalformedCostProvenance { .. } => "malformed-cost-provenance",
            Self::UndeterminedBoundaryProperty { .. } => "undetermined-boundary-property",
            Self::MalformedOpaqueCallAssessment { .. } => "malformed-opaque-call-assessment",
            Self::UnresolvedOpaqueCallAssessment { .. } => "unresolved-opaque-call-assessment",
        }
    }
}

impl fmt::Display for FrontierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrepresentableProviderProvenance { source } => write!(
                formatter,
                "frontier.unrepresentable-provider-provenance: {source}"
            ),
            Self::MalformedProposal { provider, source } => write!(
                formatter,
                "frontier.malformed-proposal: provider {provider} emitted invalid IR: {source}"
            ),
            Self::MalformedCostProvenance {
                provider,
                declared_model_key,
            } => write!(
                formatter,
                "frontier.malformed-cost-provenance: provider {provider} declared ungoverned cost model {declared_model_key}"
            ),
            Self::UndeterminedBoundaryProperty { provider, rule } => write!(
                formatter,
                "frontier.undetermined-boundary-property: provider {provider} emitted a region whose boundary property {rule} is undetermined"
            ),
            Self::MalformedOpaqueCallAssessment {
                provider, source, ..
            } => write!(
                formatter,
                "frontier.malformed-opaque-call-assessment: provider {provider} produced an invalid opaque-call feasibility assessment: {source:?}"
            ),
            Self::UnresolvedOpaqueCallAssessment { provider, .. } => write!(
                formatter,
                "frontier.unresolved-opaque-call-assessment: provider {provider} did not resolve at compile-profile feasibility"
            ),
        }
    }
}

impl Error for FrontierError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnrepresentableProviderProvenance { source } => Some(source),
            Self::MalformedProposal { source, .. } => Some(source),
            Self::MalformedCostProvenance { .. }
            | Self::UndeterminedBoundaryProperty { .. }
            | Self::MalformedOpaqueCallAssessment { .. }
            | Self::UnresolvedOpaqueCallAssessment { .. } => None,
        }
    }
}

/// Enumerates the bounded implementation frontier for one region and target.
///
/// Each provider is asked for proposals over the region subject; every proposal
/// is processed in this fixed order:
///
/// 1. provenance — the provider's complete governed identity must fit the exact
///    explain subject retained on every outcome;
/// 2. applicability — a proposal not targeting this profile is recorded as
///    [`FrontierRejection::NotApplicable`] and skipped;
/// 3. cost provenance — a proposal attributing its cost estimate to an ungoverned
///    model fails closed as [`FrontierError::MalformedCostProvenance`];
/// 4. body variant — a reserved (non-scheduled-kernel) body is recorded as
///    [`FrontierRejection::UnsupportedVariant`] and skipped, preserving the seam;
/// 5. checked verification — a scheduled-kernel body is resubmitted through
///    [`verify_schedule_with_feasibility`]. A [`FeasibilityOutcome::Proven`] verdict
///    admits it with derived resources, boundary contract, and feasibility
///    evidence; a [`PhysicalError::Target`] records [`FrontierRejection::Infeasible`];
///    any other [`PhysicalError`] fails closed as [`FrontierError::MalformedProposal`].
///
/// The admitted implementations and rejections are returned in canonical,
/// provider-order-independent order. An `Ok` with an empty admitted set is a valid
/// local no-plan result.
///
/// [`FeasibilityOutcome::Proven`]: crate::target::feasibility::FeasibilityOutcome::Proven
///
/// # Errors
///
/// Returns [`FrontierError`] when a provider emits malformed compiler output: a
/// structurally invalid scheduled-kernel body or a cost estimate attributed to an
/// ungoverned cost model.
pub(crate) fn enumerate_frontier(
    request: &VerifiedTargetRequest,
    subject: &FrontierRegionSubject,
    providers: &[&dyn PhysicalImplementationProvider],
    calls: &OpaqueCallRegistry,
) -> Result<ImplementationFrontier, FrontierError> {
    #[cfg(test)]
    crate::workcount::FRONTIER_ENUMERATIONS.record();
    let target_profile = request.target_profile().clone();
    let target_profile_key = target_profile.profile_key().clone();
    let applicable_key = target_profile_key.clone();
    let mut admitted = Vec::new();
    let mut rejections = Vec::new();
    for provider in providers {
        let provenance = provider
            .provenance()
            .map_err(|source| FrontierError::UnrepresentableProviderProvenance { source })?;
        let context = ImplementationContext { request, subject };
        let offer = provider.propose(&context);
        // A withheld strategy is recorded before the offered ones are assessed,
        // so a reader sees what the provider ruled out for this request beside
        // what it proposed for it rather than only in the proposals' absence.
        for declined in offer.declined {
            rejections.push(FrontierRejection::StrategyDeclined {
                provider: provenance.clone(),
                strategy: declined.strategy,
                cause: declined.cause,
            });
        }
        for proposal in offer.proposals {
            let kind = proposal.body.kind();
            if !proposal.applicability.applies_to(&applicable_key) {
                match &proposal.body {
                    ProposalBody::OpaqueCall(proposed) => {
                        rejections.push(FrontierRejection::OpaqueCall {
                            provider: provenance.clone(),
                            proposal: (**proposed).clone(),
                            cause: OpaqueCallRejectionCause::NotApplicable {
                                target_profile_key: target_profile_key.clone(),
                            },
                        });
                    }
                    ProposalBody::ScheduledKernel(_)
                    | ProposalBody::KernelSubprogram(_)
                    | ProposalBody::View(_) => {
                        rejections.push(FrontierRejection::NotApplicable {
                            provider: provenance.clone(),
                            kind,
                            target_profile_key: target_profile_key.clone(),
                        });
                    }
                }
                continue;
            }
            if proposal.declared_cost.model_key != COST_MODEL_KEY {
                return Err(FrontierError::MalformedCostProvenance {
                    provider: provenance.provider().clone(),
                    declared_model_key: proposal.declared_cost.model_key,
                });
            }
            let region = match proposal.body {
                ProposalBody::ScheduledKernel(region) => *region,
                ProposalBody::OpaqueCall(ref proposed) => {
                    let Some(registered) = calls.get(proposed.call()) else {
                        rejections.push(FrontierRejection::OpaqueCall {
                            provider: provenance.clone(),
                            proposal: (**proposed).clone(),
                            cause: OpaqueCallRejectionCause::Unregistered,
                        });
                        continue;
                    };
                    // The provider's binding claim, checked against the call's
                    // own ABI before anything downstream trusts it.
                    if let Err(fault) = crate::call_abi::check_bindings(
                        registered.declaration().abi(),
                        proposed.bindings(),
                    ) {
                        rejections.push(FrontierRejection::OpaqueCall {
                            provider: provenance.clone(),
                            proposal: (**proposed).clone(),
                            cause: OpaqueCallRejectionCause::MalformedBinding(fault),
                        });
                        continue;
                    }
                    let boundary = match derive_call_boundary_contract(
                        registered.declaration(),
                        proposed.bindings(),
                    ) {
                        Ok(boundary) => boundary,
                        Err(fault) => {
                            rejections.push(FrontierRejection::OpaqueCall {
                                provider: provenance.clone(),
                                proposal: (**proposed).clone(),
                                cause: OpaqueCallRejectionCause::ContractUnderivable(fault),
                            });
                            continue;
                        }
                    };
                    // The call's declared numerics must match the request's
                    // resolved contract, not merely be feasible on the target.
                    // `assess_resources` below checks the eight dimensions
                    // against the *target profile*, which is a different
                    // question: a call permitting contraction can be feasible on
                    // a device that offers it while still violating a program
                    // whose contract forbids it. Nothing else compares these, so
                    // omitting it would admit a call that computes something the
                    // caller ruled out.
                    let declared = registered.declaration().resources();
                    let contract = request.numerical_contract().realization();
                    if declared.input_subnormals != contract.input_subnormals
                        || declared.result_subnormals != contract.result_subnormals
                        || declared.contraction != contract.contraction
                        || declared.reassociation != contract.reassociation
                        || declared.permutation != contract.permutation
                        || declared.signed_zero != contract.signed_zero
                        || declared.nan_assumptions != contract.nan_assumptions
                        || declared.infinity_assumptions != contract.infinity_assumptions
                    {
                        rejections.push(FrontierRejection::OpaqueCall {
                            provider: provenance.clone(),
                            proposal: (**proposed).clone(),
                            cause: OpaqueCallRejectionCause::NumericalContractMismatch,
                        });
                        continue;
                    }
                    let work_items = match resolve_work_items(
                        registered.declaration().work(),
                        proposed.bindings(),
                        request,
                    ) {
                        Ok(work_items) => work_items,
                        Err(fault) => {
                            rejections.push(FrontierRejection::OpaqueCall {
                                provider: provenance.clone(),
                                proposal: (**proposed).clone(),
                                cause: OpaqueCallRejectionCause::WorkUnresolvable(fault),
                            });
                            continue;
                        }
                    };
                    // The same feasibility verdict a scheduled region gets,
                    // attributed to this call rather than to a region it does
                    // not have.
                    let feasibility = match crate::physical::assess_resources(
                        *registered.declaration().resources(),
                        // The admission has already required the call's declared
                        // numerics to match the request's resolved contract, so
                        // the contract's arithmetic type is the call's — the
                        // same derivation the scheduled path uses one layer up.
                        request.numerical_contract().arithmetic,
                        work_items,
                        request.target_profile(),
                    ) {
                        Ok(feasibility) => feasibility,
                        Err(verdict) => {
                            rejections.push(classify_opaque_resource_verdict(
                                &provenance,
                                proposed,
                                verdict,
                            )?);
                            continue;
                        }
                    };
                    let identity = encode_proposal_identity(
                        &encode_call_subject(proposed),
                        provenance.provider(),
                        kind,
                        &proposal.applicability,
                        &boundary,
                        &feasibility,
                    );
                    admitted.push(AdmittedImplementation {
                        provenance: ImplementationProvenance {
                            provider: provenance.clone(),
                            kind,
                        },
                        semantic_members: subject.semantic_members.clone(),
                        target_profile: target_profile.clone(),
                        body: ImplementationBody::Opaque(Box::new(registered.clone())),
                        admission: feasibility,
                        boundary,
                        cost: proposal.declared_cost,
                        identity,
                    });
                    continue;
                }
                ProposalBody::KernelSubprogram(subprogram) => {
                    match admit_subprogram(
                        *subprogram,
                        subject,
                        request,
                        &provenance,
                        &proposal.applicability,
                        proposal.declared_cost,
                    )? {
                        Ok(admission) => admitted.push(admission),
                        Err(rejection) => rejections.push(rejection),
                    }
                    continue;
                }
                ProposalBody::View(_) => {
                    rejections.push(FrontierRejection::UnsupportedVariant {
                        provider: provenance.clone(),
                        kind,
                    });
                    continue;
                }
            };
            match verify_schedule_with_feasibility(
                region,
                subject.semantic_members.clone(),
                request,
            ) {
                Ok(verified) => {
                    let admission = verified.admission().clone();
                    admitted.push(admit_verified(
                        verified,
                        admission,
                        &provenance,
                        kind,
                        &proposal.applicability,
                        proposal.declared_cost,
                    )?);
                }
                Err(PhysicalError::Target {
                    rule,
                    required,
                    available,
                    ..
                }) => {
                    rejections.push(FrontierRejection::Infeasible {
                        provider: provenance.clone(),
                        axis: rule,
                        required,
                        available,
                    });
                }
                Err(PhysicalError::Numerical { cause, .. }) => {
                    rejections.push(FrontierRejection::Unhonourable {
                        provider: provenance.clone(),
                        cause,
                    });
                }
                Err(
                    source @ (PhysicalError::Intrinsic { .. }
                    | PhysicalError::Refinement { .. }
                    | PhysicalError::ShapeProductOverflow { .. }),
                ) => {
                    return Err(FrontierError::MalformedProposal {
                        provider: provenance.provider().clone(),
                        source,
                    });
                }
            }
        }
    }
    admitted.sort_by(|left, right| left.identity.as_bytes().cmp(right.identity.as_bytes()));
    rejections.sort_by_key(encode_rejection);
    Ok(ImplementationFrontier {
        target_profile,
        region_role: subject.role,
        admitted,
        rejections,
    })
}

fn classify_opaque_resource_verdict(
    provider: &PhysicalProviderProvenance,
    proposal: &OpaqueCallProposal,
    verdict: ResourceVerdict,
) -> Result<FrontierRejection, FrontierError> {
    let cause = match verdict {
        ResourceVerdict::Rejected(RejectionCause::Capability(predicate)) => {
            OpaqueCallRejectionCause::TargetInfeasible(predicate)
        }
        ResourceVerdict::Rejected(RejectionCause::Numerical(cause)) => {
            OpaqueCallRejectionCause::TargetUnhonourable(cause)
        }
        ResourceVerdict::Intrinsic(source) => {
            return Err(FrontierError::MalformedOpaqueCallAssessment {
                provider: provider.provider().clone(),
                proposal: Box::new(proposal.clone()),
                source,
            });
        }
        ResourceVerdict::Unknown => {
            return Err(FrontierError::UnresolvedOpaqueCallAssessment {
                provider: provider.provider().clone(),
                proposal: Box::new(proposal.clone()),
            });
        }
    };
    Ok(FrontierRejection::OpaqueCall {
        provider: provider.clone(),
        proposal: proposal.clone(),
        cause,
    })
}

/// Assembles an opaque call's boundary contract from its declaration and the
/// provider's parameter bindings.
///
/// The opaque twin of [`derive_boundary_contract`]: the same operation over
/// different evidence, which is why it lives here rather than beside the two
/// halves it calls. `BoundaryRequirement` and `BoundaryGuarantee` are built with
/// struct literals private to this module, and giving them constructors to move
/// this out would widen a type's surface to serve one caller.
///
/// # Why picking "any" bound parameter per role is well defined
///
/// A contract states one answer per tensor role, and several parameters may bind
/// one role. `call_abi::check_bindings` already refuses a binding whose
/// same-role parameters disagree about layout, encoding, or alignment, so by the
/// time this runs they provably agree and the first is as good as any. Without
/// that rule this would be picking arbitrarily and calling it a derivation.
///
/// # Errors
///
/// Propagates [`GuaranteeError`] when a written role cannot be given a
/// guarantee — an ambiguous write domain, in practice.
fn derive_call_boundary_contract(
    declaration: &OpaqueCallDeclaration,
    bindings: &[(&'static str, TensorRole)],
) -> Result<BoundaryContract, GuaranteeError> {
    let mut requirements = Vec::new();
    let mut guarantees = Vec::new();
    let mut seen: Vec<TensorRole> = Vec::new();

    for (_, role) in bindings {
        if seen.contains(role) {
            continue;
        }
        seen.push(*role);

        // Selected by what the parameter *does*, not by the negation of the
        // other direction. An `InOut` both reads and writes, so selecting the
        // read side with `!writes()` silently dropped its read requirement —
        // the contract then guaranteed a role the call also reads, with no
        // requirement at all, and a producer of that tensor was never asked to
        // satisfy anything.
        let bound = |selects: fn(crate::call_abi::ParameterRole) -> bool| {
            bindings
                .iter()
                .filter(|(_, bound_role)| bound_role == role)
                .find_map(|(name, _)| {
                    let parameter = declaration.abi().parameter(name)?;
                    selects(parameter.role()).then_some(parameter)
                })
        };

        if let Some(parameter) = bound(crate::call_abi::ParameterRole::reads)
            && let Some(properties) =
                crate::call_declaration::required_properties_for(parameter, declaration.placement())
        {
            requirements.push(BoundaryRequirement {
                tensor: *role,
                access: AccessMode::Read,
                properties,
            });
        }
        if let Some(parameter) = bound(crate::call_abi::ParameterRole::writes) {
            let properties = crate::call_declaration::guaranteed_properties_for(
                parameter,
                declaration.effects(),
                declaration.placement(),
            )?;
            guarantees.push(BoundaryGuarantee {
                tensor: *role,
                // A call that writes a tensor owns that write completely; a
                // partial or racing write is not something this vocabulary can
                // express, so admitting one would be claiming more than the
                // declaration says.
                ownership: BoundaryOwnership::TotalRaceFreeWrite,
                properties,
            });
        }
    }

    Ok(BoundaryContract {
        requirements,
        guarantees,
    })
}

/// The canonical bytes an opaque call proposal is identified over.
///
/// The analogue of a scheduled region's `CanonicalScheduledRegionIdentity`, and
/// it must include the **bindings**, not only the call. The same registered call
/// bound to different tensor roles is a different implementation — it computes a
/// different thing — so two such proposals must not share an identity. Omitting
/// the bindings would make them collide, and the collision would surface as one
/// silently shadowing the other in the admitted set.
///
/// Bindings are encoded in their given order, which the frontier does not sort:
/// a provider that emits them in a varying order gets varying identities, which
/// is its own defect to fix and not something a canonical form can paper over
/// without deciding that binding order carries no meaning.
fn encode_call_subject(proposed: &OpaqueCallProposal) -> Vec<u8> {
    let mut bytes = Vec::new();
    let call = proposed.call();
    push_slice(&mut bytes, call.provider().as_bytes());
    push_slice(&mut bytes, call.call().as_bytes());
    bytes.extend_from_slice(&call.revision().to_be_bytes());
    for (name, role) in proposed.bindings() {
        push_slice(&mut bytes, name.as_bytes());
        push_tensor_role(&mut bytes, *role);
    }
    bytes
}

fn encode_opaque_call_proposal(output: &mut Vec<u8>, proposal: &OpaqueCallProposal) {
    let call = proposal.call();
    push_slice(output, call.provider().as_bytes());
    push_slice(output, call.call().as_bytes());
    output.extend_from_slice(&call.revision().to_be_bytes());
    push_len(output, proposal.bindings().len());
    for (name, role) in proposal.bindings() {
        push_slice(output, name.as_bytes());
        push_tensor_role(output, *role);
    }
}

fn encode_opaque_call_cause(output: &mut Vec<u8>, cause: &OpaqueCallRejectionCause) {
    match cause {
        OpaqueCallRejectionCause::NotApplicable { target_profile_key } => {
            output.push(0x01);
            push_slice(output, target_profile_key.as_str().as_bytes());
        }
        OpaqueCallRejectionCause::Unregistered => output.push(0x02),
        OpaqueCallRejectionCause::MalformedBinding(fault) => {
            output.push(0x03);
            encode_binding_error(output, *fault);
        }
        OpaqueCallRejectionCause::ContractUnderivable(fault) => {
            output.push(0x04);
            output.push(match fault {
                GuaranteeError::NotAWrite => 0x01,
                GuaranteeError::AmbiguousWriteDomain => 0x02,
            });
        }
        OpaqueCallRejectionCause::NumericalContractMismatch => output.push(0x05),
        OpaqueCallRejectionCause::WorkUnresolvable(fault) => {
            output.push(0x06);
            match fault {
                WorkResolutionError::UnknownParameter(parameter) => {
                    output.push(0x01);
                    push_slice(output, parameter.as_bytes());
                }
                WorkResolutionError::IntermediateShapeUnavailable { parameter } => {
                    output.push(0x02);
                    push_slice(output, parameter.as_bytes());
                }
            }
        }
        OpaqueCallRejectionCause::TargetInfeasible(predicate) => {
            output.push(0x07);
            push_slice(output, predicate.axis().key().as_bytes());
            output.extend_from_slice(&predicate.required().value().to_be_bytes());
            output.extend_from_slice(&predicate.available().value().to_be_bytes());
        }
        OpaqueCallRejectionCause::TargetUnhonourable(cause) => {
            output.push(0x08);
            cause.encode(output);
        }
    }
}

fn encode_binding_error(output: &mut Vec<u8>, fault: crate::call_abi::BindingError) {
    match fault {
        crate::call_abi::BindingError::UnboundParameter(parameter) => {
            output.push(0x01);
            push_slice(output, parameter.as_bytes());
        }
        crate::call_abi::BindingError::UnknownParameter(parameter) => {
            output.push(0x02);
            push_slice(output, parameter.as_bytes());
        }
        crate::call_abi::BindingError::ParameterBoundTwice(parameter) => {
            output.push(0x03);
            push_slice(output, parameter.as_bytes());
        }
        crate::call_abi::BindingError::RoleStorageDisagreement { first, second } => {
            output.push(0x04);
            push_slice(output, first.as_bytes());
            push_slice(output, second.as_bytes());
        }
    }
}

/// Why a call's declared work scaling could not be resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkResolutionError {
    /// The scaling names a parameter absent from this proposal's bindings.
    UnknownParameter(&'static str),
    /// The parameter is bound to an intermediate whose cover-specific shape is
    /// unavailable during local frontier enumeration.
    IntermediateShapeUnavailable {
        /// The parameter whose tensor shape is unavailable.
        parameter: &'static str,
    },
}

/// Evaluates an opaque call's declared work scaling against the request.
///
/// `assess_region` needs a work-item count, and a scheduled region reads one
/// from its schedule. An opaque call declares how its work scales
/// ([`WorkScaling`]) and this resolves that against the bound tensors.
///
/// `Fixed` resolves directly. `PerElementOf` resolves through the tensor role
/// the parameter is bound to: the bounded profile's normalized request states
/// `input_elements` and `output_elements`, which is exactly the count a call
/// over that tensor performs work proportional to.
///
/// # Why `Intermediate` declines
///
/// An intermediate is a cover-level artefact — it exists because a cover chose
/// to materialize between two regions — and its element count is a property of
/// that cover, which the frontier does not hold when enumerating for a subject.
///
/// A previous revision resolved it to `input_elements` on the claim that "the
/// bounded profile has exactly one materialization: the pointwise result".
/// That claim was false: `enumerate_covers` retains the all-singleton cover
/// unconditionally, and that cover materializes **every** internal value —
/// including rank-0 scalar constants, whose element count is 1, not the
/// input's. Substituting `input_elements` there is exactly the
/// confidently-wrong feasibility verdict `WorkScaling` exists to prevent, so a
/// shape-dependent call bound to an intermediate is refused rather than
/// mis-sized. Resolving it correctly needs the cover edge's actual value
/// shape, which arrives with the cover, not the subject.
fn resolve_work_items(
    work: WorkScaling,
    bindings: &[(&'static str, TensorRole)],
    request: &VerifiedTargetRequest,
) -> Result<u64, WorkResolutionError> {
    match work {
        WorkScaling::Fixed(count) => Ok(count),
        WorkScaling::PerElementOf(name) => {
            let (_, role) = bindings
                .iter()
                .find(|(bound, _)| *bound == name)
                .ok_or(WorkResolutionError::UnknownParameter(name))?;
            match role {
                // Every input of a recognized request shares one shape — the
                // pointwise recognizer refuses a program whose inputs disagree,
                // and the serial-sum one admits a single input — so the ordinal
                // does not change the answer here. A strategy admitting inputs
                // of different extents must resolve this per ordinal rather
                // than inherit this arm.
                TensorRole::Input { .. } => Ok(request.normalized().input_elements()),
                TensorRole::Output => Ok(request.normalized().output_elements()),
                TensorRole::Intermediate => {
                    Err(WorkResolutionError::IntermediateShapeUnavailable { parameter: name })
                }
            }
        }
    }
}

/// Turns one region that passed checked verification into an admitted
/// implementation, deriving its boundary contract and its canonical identity.
///
/// The provider supplies only the applicability predicate and the cost estimate;
/// the contract and the identity are derived here from the verified region, so a
/// provider can neither declare a boundary it does not honour nor forge an
/// identity.
///
/// # Errors
///
/// Returns [`FrontierError::UndeterminedBoundaryProperty`] when the verified
/// region's own facts do not determine a property its contract must state.
fn admit_verified(
    verified: VerifiedScheduledRegion,
    feasibility: AdmissionEvidence,
    provider: &PhysicalProviderProvenance,
    kind: PhysicalProposalKind,
    applicability: &TargetApplicability,
    cost: PhysicalCostEstimate,
) -> Result<AdmittedImplementation, FrontierError> {
    let boundary = derive_boundary_contract(&verified).map_err(|rule| {
        FrontierError::UndeterminedBoundaryProperty {
            provider: provider.provider().clone(),
            rule,
        }
    })?;
    let identity = encode_proposal_identity(
        verified.canonical_identity().as_bytes(),
        provider.provider(),
        kind,
        applicability,
        &boundary,
        &feasibility,
    );
    Ok(AdmittedImplementation {
        provenance: ImplementationProvenance {
            provider: provider.clone(),
            kind,
        },
        semantic_members: verified.semantic_members().to_vec(),
        target_profile: verified.target_profile().clone(),
        body: ImplementationBody::Scheduled(Box::new(verified)),
        admission: feasibility,
        boundary,
        cost,
        identity,
    })
}

/// Verifies one proposed subprogram and turns it into an admission or a typed
/// rejection.
///
/// Every stage re-enters [`verify_schedule_with_feasibility`] with the members
/// that stage claims, so a provider can neither smuggle an unverified region nor
/// let one pass claim occurrences the request-subject binding does not grant it.
/// On top of that per-stage check the subprogram carries one obligation no stage
/// can see: the occurrences its stages claim between them must be exactly the
/// subject's, each once. A chain covering less would silently drop work the
/// cover assigned to this region, and a chain covering more would compute an
/// occurrence another region also computes.
///
/// The feasibility verdict is the **subprogram's**, taken once over the peak
/// requirement across its stages, rather than a merge of per-stage evidences: a
/// merge would be a second derivation of one decision, and ADR 0043 keeps the
/// decision single.
///
/// # Errors
///
/// Returns [`FrontierError`] for malformed compiler output — a stage that fails
/// intrinsic verification or its subject binding, a chain that does not compose,
/// or a feasibility assessment the shared authority cannot resolve. A legitimate
/// target refusal is the `Err` arm of the returned `Ok`, not an error.
fn admit_subprogram(
    subprogram: KernelSubprogram,
    subject: &FrontierRegionSubject,
    request: &VerifiedTargetRequest,
    provider: &PhysicalProviderProvenance,
    applicability: &TargetApplicability,
    cost: PhysicalCostEstimate,
) -> Result<Result<AdmittedImplementation, FrontierRejection>, FrontierError> {
    let malformed = |rule: &'static str| FrontierError::UndeterminedBoundaryProperty {
        provider: provider.provider().clone(),
        rule,
    };
    if subprogram.stages.len() < 2 {
        return Err(malformed(SUBPROGRAM_NOT_CHAINED_RULE));
    }
    let mut claimed: Vec<SemanticMemberId> = Vec::new();
    for stage in &subprogram.stages {
        claimed.extend_from_slice(&stage.semantic_members);
    }
    claimed.sort_unstable();
    if claimed != subject.semantic_members {
        return Err(malformed("subprogram-coverage"));
    }
    let mut verified = Vec::with_capacity(subprogram.stages.len());
    for stage in subprogram.stages {
        match verify_schedule_with_feasibility(stage.region, stage.semantic_members, request) {
            Ok(region) => verified.push(region),
            Err(PhysicalError::Target {
                rule,
                required,
                available,
                ..
            }) => {
                return Ok(Err(FrontierRejection::Infeasible {
                    provider: provider.clone(),
                    axis: rule,
                    required,
                    available,
                }));
            }
            Err(PhysicalError::Numerical { cause, .. }) => {
                return Ok(Err(FrontierRejection::Unhonourable {
                    provider: provider.clone(),
                    cause,
                }));
            }
            Err(
                source @ (PhysicalError::Intrinsic { .. }
                | PhysicalError::Refinement { .. }
                | PhysicalError::ShapeProductOverflow { .. }),
            ) => {
                return Err(FrontierError::MalformedProposal {
                    provider: provider.provider().clone(),
                    source,
                });
            }
        }
    }
    let boundary = derive_subprogram_boundary_contract(&verified).map_err(malformed)?;
    let resources = subprogram_resources(&verified).ok_or_else(|| malformed("subprogram-empty"))?;
    let work_items = verified
        .iter()
        .map(|stage| stage.region().schedule.work_items)
        .max()
        .ok_or_else(|| malformed("subprogram-empty"))?;
    let feasibility = match crate::physical::assess_resources(
        resources,
        request.numerical_contract().arithmetic,
        work_items,
        request.target_profile(),
    ) {
        Ok(feasibility) => feasibility,
        Err(ResourceVerdict::Rejected(RejectionCause::Capability(predicate))) => {
            return Ok(Err(FrontierRejection::Infeasible {
                provider: provider.clone(),
                axis: predicate.axis().key(),
                required: predicate.required().value(),
                available: predicate.available().value(),
            }));
        }
        Err(ResourceVerdict::Rejected(RejectionCause::Numerical(cause))) => {
            return Ok(Err(FrontierRejection::Unhonourable {
                provider: provider.clone(),
                cause,
            }));
        }
        Err(ResourceVerdict::Intrinsic(_) | ResourceVerdict::Unknown) => {
            return Err(malformed("subprogram-assessment-unresolved"));
        }
    };
    let identity = encode_proposal_identity(
        &encode_subprogram_subject(&verified),
        provider.provider(),
        PhysicalProposalKind::KernelSubprogram,
        applicability,
        &boundary,
        &feasibility,
    );
    Ok(Ok(AdmittedImplementation {
        provenance: ImplementationProvenance {
            provider: provider.clone(),
            kind: PhysicalProposalKind::KernelSubprogram,
        },
        semantic_members: subject.semantic_members.clone(),
        target_profile: request.target_profile().clone(),
        body: ImplementationBody::Subprogram(verified),
        admission: feasibility,
        boundary,
        cost,
        identity,
    }))
}

/// The canonical bytes a subprogram admission is identified over.
///
/// The **ordered** chain, length-framed: two subprograms over the same regions
/// in different orders compute different things, and a set-like encoding would
/// give them one identity. The stage count is framed too, so a chain is never a
/// prefix of a longer one.
fn encode_subprogram_subject(stages: &[VerifiedScheduledRegion]) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_len(&mut bytes, stages.len());
    for stage in stages {
        push_slice(&mut bytes, stage.canonical_identity().as_bytes());
    }
    bytes
}

/// The physical implementation authorities one compilation enumerates against.
///
/// The provider list and the opaque-call registry are composed together because
/// they are two halves of one answer to *what physical implementations exist for
/// this compilation*: a provider proposes a call and only the registry says what
/// that call is, so a provider installed without the declaration it names
/// proposes something that cannot be admitted, and a declaration registered
/// without a provider is admitted by nothing. Composing them apart is what left
/// the sole production enumeration constructing an empty registry inline while
/// the admission path was fully implemented.
///
/// Crate-private and passed down the compile path rather than carried on the
/// request. Two reasons, and the second is the load-bearing one. A provider
/// holds no ownership the request model could express — it is a borrowed
/// statically linked implementation, while `VerifiedCompilationRequest` is an
/// owned, cloned, comparable value. And the request's canonical identity binds
/// what the caller *asked for*; which implementations this build offers is
/// bound instead where it is used, in each admitted implementation's provenance
/// and so in the plan identity. A compilation offering no opaque call therefore
/// has exactly the request subject it had before this type existed.
pub(crate) struct PhysicalAuthorities<'providers> {
    providers: Vec<&'providers dyn PhysicalImplementationProvider>,
    calls: OpaqueCallRegistry,
}

impl<'providers> PhysicalAuthorities<'providers> {
    /// The authorities this build ships: the governed provider, and no call.
    ///
    /// An empty registry is not a degenerate case here but the ordinary one —
    /// Tiler declares no opaque call of its own — and a program referencing none
    /// needs none.
    pub(crate) fn governed() -> Self {
        Self {
            providers: vec![&GovernedPhysicalProvider],
            calls: OpaqueCallRegistry::new(),
        }
    }

    /// Composes a stated provider list with the calls those providers may name.
    ///
    /// Taken together rather than installed one at a time, so the pair a
    /// compilation plans against is stated in one place: a caller cannot leave
    /// half of it behind.
    #[allow(
        dead_code,
        reason = "the composition seam for a non-governed authority; its first caller is the compile-path test proving a registered call reaches admission, and a governed opaque declaration would be its first production one"
    )]
    pub(crate) fn composed(
        providers: Vec<&'providers dyn PhysicalImplementationProvider>,
        calls: OpaqueCallRegistry,
    ) -> Self {
        Self { providers, calls }
    }

    /// The providers proposing implementations for each region subject.
    pub(crate) fn providers(&self) -> &[&'providers dyn PhysicalImplementationProvider] {
        &self.providers
    }

    /// The opaque calls a proposal from those providers may name.
    pub(crate) const fn calls(&self) -> &OpaqueCallRegistry {
        &self.calls
    }
}

/// Namespace of Tiler's own governed physical implementation provider.
const GOVERNED_PHYSICAL_NAMESPACE: &str = "tiler";
/// Name of Tiler's own governed physical implementation provider.
const GOVERNED_PHYSICAL_NAME: &str = "prototype-serial-sum-physical";
/// Output-affecting revision of the governed physical provider.
const GOVERNED_PHYSICAL_REVISION: u32 = 1;

/// Tiler's own governed physical implementation provider for the bounded profile.
///
/// It offers one checked scheduled-kernel body per *recognized* region subject —
/// the materialized pointwise prologue, the materialized reduction, and the fused
/// whole-program region — and nothing at all for any other member set. Offering
/// nothing is a legitimate local result, so a cover this profile cannot implement
/// is reported by complete-plan selection as an unimplemented region rather than
/// being silently approximated.
///
/// The provider declares only a body, an applicability predicate, and a cost
/// estimate. It cannot stamp its own provenance, derive its resources, or bypass
/// verification: the frontier resubmits every body through the ordinary checked
/// path in [`crate::physical::verify_schedule_with_feasibility`].
pub(crate) struct GovernedPhysicalProvider;

impl GovernedPhysicalProvider {
    /// Returns the governed physical provider identity.
    ///
    /// # Panics
    ///
    /// Panics only if Tiler's compile-time governed provider components violate
    /// the canonical provider-identity grammar.
    pub(crate) fn identity() -> ProviderIdentity {
        ProviderIdentity::new(
            GOVERNED_PHYSICAL_NAMESPACE,
            GOVERNED_PHYSICAL_NAME,
            GOVERNED_PHYSICAL_REVISION,
        )
        .expect("the governed physical provider identity is valid")
    }
}

impl PhysicalImplementationProvider for GovernedPhysicalProvider {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(Self::identity())
    }

    fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
        let request = context.request();
        let members = context.subject().semantic_members();
        let input_elements = request.normalized().input_elements();
        let output_elements = request.normalized().output_elements();
        // A materialized f32 intermediate costs four bytes per element. The
        // estimate is structural and is never a feasibility input.
        let intermediate_bytes = input_elements.saturating_mul(4);
        let applicability =
            TargetApplicability::for_targets([request.target_profile().profile_key().clone()]);
        let mut split = None;
        let (region, cost) = if let Some(pointwise) = request.pointwise() {
            if members != pointwise.members {
                return ProviderOffer::default();
            }
            (
                crate::physical::pointwise_region(request).0,
                PhysicalCostEstimate::structural(1, output_elements, 0),
            )
        } else if members == request.serial_sum().members.pointwise() {
            (
                crate::physical::pointwise_region(request).0,
                PhysicalCostEstimate::structural(1, input_elements, intermediate_bytes),
            )
        } else if members == request.serial_sum().members.reduction() {
            // The reduction subject is the one place a split is even a
            // candidate, so it is the one place the strategy is considered and
            // — when this request does not admit it — the one place the decline
            // is stated. The serial alternative is offered either way; a split
            // is additive and never replaces it.
            split = Some(propose_split(request, &applicability));
            (
                crate::physical::reduction_region(request).0,
                PhysicalCostEstimate::structural(1, output_elements, 0),
            )
        } else if members == request.serial_sum().members.all() {
            // Whether the whole-program region may be *fused* belongs to the
            // numerical-legality authority and whether it *fits* belongs to this
            // target; neither is a capability question. Every occurrence the
            // region covers already resolved its lowering capability before any
            // cover reached this proposer, so no capability gap is left to defer.
            (
                crate::physical::fused_region(request).0,
                PhysicalCostEstimate::structural(1, output_elements, 0),
            )
        } else {
            return ProviderOffer::default();
        };
        let serial = ImplementationProposal::new(
            ProposalBody::ScheduledKernel(Box::new(region)),
            applicability,
            cost,
        );
        match split {
            None => ProviderOffer::proposing(vec![serial]),
            Some(Ok(split)) => ProviderOffer::proposing(vec![serial, split]),
            Some(Err(declined)) => ProviderOffer::proposing(vec![serial]).decline(declined),
        }
    }
}

/// Offers the multi-pass split of one request's reduction, or states why not.
///
/// The cost is structural and never a feasibility input: two dispatches, the
/// partial pass's launched threads plus the final pass's, and the four bytes per
/// partial value the split stages. It is deliberately worse than the serial
/// alternative's on every dimension under this model — a split trades those for
/// parallelism the structural model does not measure, which is exactly why
/// `calibrate-and-activate-parallel-reduction-selection` owns preference and
/// this slice only enumerates.
fn propose_split(
    request: &VerifiedTargetRequest,
    applicability: &TargetApplicability,
) -> Result<ImplementationProposal, DeclinedStrategy> {
    let split = crate::physical::split_reduction_regions(request).map_err(|unavailable| {
        DeclinedStrategy::new(
            crate::physical::MULTI_PASS_SPLIT_STRATEGY,
            match unavailable {
                crate::physical::SplitUnavailable::ReassociationForbidden => {
                    StrategyDeclineCause::NumericalPermissionRefused {
                        dimension: crate::target::honourability::NumericalDimension::Reassociation
                            .key(),
                    }
                }
                crate::physical::SplitUnavailable::NoAdmissiblePartition { contributors } => {
                    StrategyDeclineCause::NoAdmissibleShape {
                        rule: unavailable.reason(),
                        extent: contributors,
                    }
                }
                crate::physical::SplitUnavailable::Unrepresentable => {
                    StrategyDeclineCause::Unrepresentable {
                        rule: unavailable.reason(),
                    }
                }
            },
        )
    })?;
    let output_elements = request.normalized().output_elements();
    let partial_elements = output_elements.saturating_mul(split.partition.partitions);
    let stages = split
        .stages
        .into_iter()
        .map(|(region, members)| SubprogramStage::new(region, members))
        .collect();
    Ok(ImplementationProposal::new(
        ProposalBody::KernelSubprogram(Box::new(KernelSubprogram::new(stages))),
        applicability.clone(),
        PhysicalCostEstimate::structural(
            2,
            partial_elements.saturating_add(output_elements),
            partial_elements.saturating_mul(4),
        ),
    ))
}

fn encode_proposal_identity(
    subject_bytes: &[u8],
    provider: &ProviderIdentity,
    kind: PhysicalProposalKind,
    applicability: &TargetApplicability,
    boundary: &BoundaryContract,
    feasibility: &AdmissionEvidence,
) -> ImplementationProposalIdentity {
    let mut bytes = PROPOSAL_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, subject_bytes);
    encode_provider(&mut bytes, provider);
    bytes.push(kind.tag());
    applicability.encode(&mut bytes);
    boundary.encode(&mut bytes);
    match feasibility.deferred() {
        None => bytes.push(0),
        Some(deferred) => {
            bytes.push(1);
            push_len(&mut bytes, deferred.predicates().len());
            for predicate in deferred.predicates() {
                push_slice(&mut bytes, predicate.axis().key().as_bytes());
                bytes.extend_from_slice(&predicate.required().value().to_be_bytes());
                push_slice(&mut bytes, &predicate.requirement().canonical_bytes());
            }
        }
    }
    ImplementationProposalIdentity(bytes)
}

fn encode_rejection(rejection: &FrontierRejection) -> Vec<u8> {
    let mut bytes = Vec::new();
    rejection.encode(&mut bytes);
    bytes
}

/// Appends one boundary tensor role to a canonical encoding.
///
/// An input writes its ordinal after its tag. Two boundary facets over two
/// different input tensors are different facets, and a one-byte role would give
/// them one encoding — so a plan reading `a * b` and one reading `a * a` would
/// share a receipt identity.
///
/// Written as an exhaustive match rather than read from the discriminant, so
/// adding or reordering a role is a build error here instead of a silent change
/// to every identity ever encoded (ADR 0074 convention 5b).
fn push_tensor_role(output: &mut Vec<u8>, role: TensorRole) {
    match role {
        TensorRole::Input { ordinal } => {
            output.push(1);
            output.extend_from_slice(&ordinal.get().to_be_bytes());
        }
        TensorRole::Intermediate => output.push(2),
        TensorRole::Output => output.push(3),
    }
}

/// The governed tag naming an access mode in a canonical encoding.
///
/// An out-of-crate total map onto an identity tag, so `AccessMode` is an ADR 0074
/// convention 5b vocabulary and must not become `#[non_exhaustive]`: a wildcard
/// here would have to invent a tag, and two distinct access modes sharing one
/// would give two distinct boundary contracts one identity.
const fn access_mode_tag(mode: AccessMode) -> u8 {
    match mode {
        AccessMode::Read => 1,
        AccessMode::Write => 2,
    }
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::{
        AdmittedImplementation, BoundaryOwnership, FrontierError, FrontierRegionSubject,
        FrontierRejection, GovernedPhysicalProvider, ImplementationBody, ImplementationContext,
        ImplementationFrontier, ImplementationProposal, KernelSubprogram, OpaqueCallRejectionCause,
        PhysicalCostEstimate, PhysicalImplementationProvider, PhysicalProposalKind,
        PhysicalProviderProvenance, PhysicalProviderProvenanceError, ProposalBody, ProviderOffer,
        ReservedProposalSeam, SubprogramStage, TargetApplicability, bounded_guarantees,
        bounded_requirements, enumerate_frontier,
    };
    use crate::boundary::{
        BoundaryProperty, GuaranteedProperty, LayoutRequirement, MaterializationForm,
        MemoryDomainClass, RequiredProperties, RequiredProperty,
    };
    use crate::call_registry::{OpaqueCallIdentity, OpaqueCallProposal, OpaqueCallRegistry};
    use crate::physical::{build_fused_scheduled_region, pointwise_region};
    use crate::request::{
        CompilationRequest, TargetProfileKey, VerifiedTargetRequest, verify_request,
    };
    use tiler_ir::schedule::{
        AccessMode, ExceptionalValueAssumption, InputOrdinal, NumericalPermission, ScheduledRegion,
        SubnormalMode, TensorRole,
    };
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ProviderIdentity,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    const GOVERNED_TARGET_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";

    fn request(shape: Shape, axes: impl IntoIterator<Item = Axis>) -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, axes).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_request(CompilationRequest::governed(&program)).unwrap();
        request.for_target(0).unwrap()
    }

    fn provider_identity(name: &str, revision: u32) -> ProviderIdentity {
        ProviderIdentity::new("tiler.test.physical", name, revision).unwrap()
    }

    fn fused_subject(request: &VerifiedTargetRequest) -> FrontierRegionSubject {
        FrontierRegionSubject::new("fused", request.serial_sum().members.all())
    }

    fn pointwise_subject(request: &VerifiedTargetRequest) -> FrontierRegionSubject {
        FrontierRegionSubject::new(
            "pointwise",
            request.serial_sum().members.pointwise().to_vec(),
        )
    }

    fn fused_region(request: &VerifiedTargetRequest) -> ScheduledRegion {
        build_fused_scheduled_region(request)
            .unwrap()
            .region()
            .clone()
    }

    fn governed_applicability() -> TargetApplicability {
        TargetApplicability::for_targets([TargetProfileKey::governed(GOVERNED_TARGET_KEY)])
    }

    /// A provider that proposes one checked scheduled-kernel body for the fused
    /// region with a caller-chosen provider identity and cost estimate.
    struct FusedScheduledKernelProvider {
        provider: ProviderIdentity,
        cost: PhysicalCostEstimate,
    }

    impl PhysicalImplementationProvider for FusedScheduledKernelProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(self.provider.clone())
        }

        fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
            ProviderOffer::proposing(vec![ImplementationProposal::new(
                ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                governed_applicability(),
                self.cost,
            )])
        }
    }

    #[test]
    fn oversized_provider_provenance_fails_before_proposal_enumeration() {
        struct OversizedProvider {
            identity: ProviderIdentity,
        }

        impl PhysicalImplementationProvider for OversizedProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(self.identity.clone())
            }

            fn propose(&self, _: &ImplementationContext<'_>) -> ProviderOffer {
                panic!("unrepresentable provenance must fail before proposals are requested")
            }
        }

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let identity = ProviderIdentity::new("n".repeat(128), "p".repeat(128), 1)
            .expect("each provider component is individually governed");
        let provider = OversizedProvider {
            identity: identity.clone(),
        };
        let error = enumerate_frontier(
            &request,
            &fused_subject(&request),
            &[&provider],
            &OpaqueCallRegistry::new(),
        )
        .expect_err("the complete provider subject exceeds explain's bound");
        assert!(matches!(
            error,
            FrontierError::UnrepresentableProviderProvenance {
                source: PhysicalProviderProvenanceError { provider, .. }
            } if provider == identity
        ));
    }

    #[test]
    fn additive_providers_both_admit_the_same_region() {
        // Two independent providers each contribute a checked implementation of
        // the same fused region. Unlike a singular-capability registry, this is
        // additive: both are admitted rather than colliding into an ambiguity.
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let first = FusedScheduledKernelProvider {
            provider: provider_identity("alpha", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let second = FusedScheduledKernelProvider {
            provider: provider_identity("beta", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let providers: [&dyn PhysicalImplementationProvider; 2] = [&first, &second];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();

        assert_eq!(frontier.admitted().len(), 2);
        assert!(frontier.rejections().is_empty());
        let providers: Vec<&ProviderIdentity> = frontier
            .admitted()
            .iter()
            .map(|admitted| admitted.provenance().provider())
            .collect();
        assert!(providers.contains(&&provider_identity("alpha", 1)));
        assert!(providers.contains(&&provider_identity("beta", 1)));
        // Distinct providers of the same region are distinct proposals.
        assert_ne!(
            frontier.admitted()[0].identity(),
            frontier.admitted()[1].identity()
        );
    }

    #[test]
    fn every_admitted_proposal_carries_the_derived_boundary_contract_and_resources() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = FusedScheduledKernelProvider {
            provider: provider_identity("alpha", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();

        let admitted = &frontier.admitted()[0];
        assert_eq!(
            admitted.provenance().kind(),
            PhysicalProposalKind::ScheduledKernel
        );
        // Exact feasibility resources are derived from the verified region.
        assert_eq!(admitted.resources().buffer_bindings, 2);
        assert_eq!(
            admitted.resources().input_subnormals,
            SubnormalMode::Preserve
        );
        assert_eq!(
            admitted.resources().contraction,
            NumericalPermission::Forbidden
        );
        // The feasibility admission carries resolved predicates as evidence.
        assert!(!admitted.admission().proven().is_empty());
        // The fused region reads an Input boundary and produces the Output boundary.
        let requirements = admitted.boundary().requirements();
        assert_eq!(requirements.len(), 1);
        assert_eq!(
            requirements[0].tensor(),
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST
            }
        );
        assert_eq!(requirements[0].access(), AccessMode::Read);
        let guarantees = admitted.boundary().guarantees();
        assert_eq!(guarantees.len(), 1);
        assert_eq!(guarantees[0].tensor(), TensorRole::Output);
        assert_eq!(
            guarantees[0].ownership(),
            BoundaryOwnership::TotalRaceFreeWrite
        );

        // Every governed dimension is stated on both sides. A derivation that
        // left one out would compose only by accident, because a requirement no
        // guarantee speaks to fails closed rather than passing.
        let needed = requirements[0].properties();
        let offered = guarantees[0].properties();
        assert_eq!(
            needed.properties().len(),
            crate::boundary::CANONICAL_PROPERTIES.len()
        );
        assert_eq!(
            offered.properties().len(),
            crate::boundary::CANONICAL_PROPERTIES.len()
        );

        // The derived values are the bounded profile's, and each is read from the
        // region rather than declared by the provider.
        assert_eq!(
            needed.get(BoundaryProperty::Materialization),
            Some(&RequiredProperty::Materialization(
                MaterializationForm::MaterializedBuffer
            ))
        );
        assert_eq!(
            offered.get(BoundaryProperty::MemoryDomain),
            Some(&GuaranteedProperty::MemoryDomain(MemoryDomainClass::Device)),
            "the domain is read from the region's own resource requirements"
        );
    }

    /// The bounded profile's two property sets discharge each other on every
    /// governed dimension, so no boundary in it is ever undischarged.
    ///
    /// This is a *trigger*, not a guarantee anyone wants to keep. Both sets are
    /// compile-time constants with no per-region variation, so
    /// [`crate::boundary::unsatisfied_properties`] cannot return a non-empty
    /// result anywhere on the production path and
    /// `BoundaryDisagreement::UndischargedHandoff` is unreachable. That is what
    /// makes `implement-boundary-property-enforcers` unstartable: an enforcer
    /// reconciles a mismatch, and the bounded profile admits none, so the six
    /// enforcer kinds the ticket names would all be exercised only by synthetic
    /// property sets a test wrote for them.
    ///
    /// **When this test fails, that ticket becomes startable**, and the mismatch
    /// that failed it is the enforcer's first real case. Do not repair the test
    /// by widening the sets back into agreement.
    #[test]
    fn the_bounded_profile_admits_no_undischarged_boundary() {
        let needed = bounded_requirements();
        let offered = bounded_guarantees();

        // Every dimension is spoken to on both sides. A dimension missing from
        // either set would make the check below vacuous on it rather than false,
        // which is the failure mode this pair of assertions exists to catch.
        for property in crate::boundary::CANONICAL_PROPERTIES {
            assert!(
                needed.get(property).is_some(),
                "{property} is not required, so satisfaction says nothing about it"
            );
            assert!(
                offered.get(property).is_some(),
                "{property} is not guaranteed, so satisfaction says nothing about it"
            );
        }

        let unsatisfied = crate::boundary::unsatisfied_properties(&needed, &offered);
        assert!(
            unsatisfied.is_empty(),
            "the bounded profile now admits an undischarged boundary, which makes \
             implement-boundary-property-enforcers startable on: {unsatisfied:?}"
        );
    }

    /// The check above can say no.
    ///
    /// A test that only ever asserts an empty list would pass just as happily if
    /// `unsatisfied_properties` returned nothing at all, so this drives the same
    /// relation with a requirement the bounded guarantee genuinely fails and
    /// confirms it reports the failure. `UnitStrideOnAxis` on a non-last axis is
    /// the one well-formed mismatch the current vocabulary admits against
    /// `DenseRowMajor`, which is why it is the case chosen here.
    #[test]
    fn an_unsatisfiable_requirement_is_reported_rather_than_passed() {
        let offered = bounded_guarantees();
        let needed = RequiredProperties::new([RequiredProperty::StorageLayout(
            LayoutRequirement::UnitStrideOnAxis {
                axis: Axis::new(0),
                rank: 2,
            },
        )])
        .expect("unit stride on axis 0 of a rank-2 value is well formed");

        let unsatisfied = crate::boundary::unsatisfied_properties(&needed, &offered);
        assert_eq!(
            unsatisfied.len(),
            1,
            "a dense row-major guarantee does not give unit stride on axis 0 of a \
             rank-2 value, so this must be reported"
        );
        assert_eq!(
            unsatisfied[0].property(),
            BoundaryProperty::StorageLayout,
            "the reported dimension must be the one that failed"
        );
    }

    #[test]
    fn identity_and_ordering_are_independent_of_provider_order() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let alpha = FusedScheduledKernelProvider {
            provider: provider_identity("alpha", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let beta = FusedScheduledKernelProvider {
            provider: provider_identity("beta", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };

        let forward: [&dyn PhysicalImplementationProvider; 2] = [&alpha, &beta];
        let reverse: [&dyn PhysicalImplementationProvider; 2] = [&beta, &alpha];
        let first =
            enumerate_frontier(&request, &subject, &forward, &OpaqueCallRegistry::new()).unwrap();
        let second =
            enumerate_frontier(&request, &subject, &reverse, &OpaqueCallRegistry::new()).unwrap();

        let identities = |frontier: &super::ImplementationFrontier| -> Vec<Vec<u8>> {
            frontier
                .admitted()
                .iter()
                .map(|admitted| admitted.identity().as_bytes().to_vec())
                .collect()
        };
        assert_eq!(identities(&first), identities(&second));
    }

    #[test]
    fn a_reserved_opaque_body_is_rejected_but_keeps_the_additive_seam() {
        // A checked scheduled kernel and a reserved opaque call are proposed for
        // the same region. The scheduled kernel is admitted; the opaque call is
        // explicitly rejected without failing the enumeration, preserving the
        // additive sum-type seam the opaque-call ticket will implement.
        struct OpaqueProvider;
        impl PhysicalImplementationProvider for OpaqueProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(provider_identity("opaque", 1))
            }
            fn propose(&self, _: &ImplementationContext<'_>) -> ProviderOffer {
                ProviderOffer::proposing(vec![
                    ImplementationProposal::new(
                        ProposalBody::OpaqueCall(Box::new(
                            OpaqueCallProposal::new(
                                OpaqueCallIdentity::new("test", "mystery", 1).expect("named"),
                                Vec::new(),
                            )
                            .expect("fixture proposal is exactly reportable"),
                        )),
                        governed_applicability(),
                        PhysicalCostEstimate::structural(1, 2, 0),
                    ),
                    ImplementationProposal::new(
                        ProposalBody::View(ReservedProposalSeam::new("view")),
                        governed_applicability(),
                        PhysicalCostEstimate::structural(1, 2, 0),
                    ),
                ])
            }
        }

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let scheduled = FusedScheduledKernelProvider {
            provider: provider_identity("alpha", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let opaque = OpaqueProvider;
        let providers: [&dyn PhysicalImplementationProvider; 2] = [&scheduled, &opaque];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();

        assert_eq!(frontier.admitted().len(), 1);
        assert_eq!(
            frontier.admitted()[0].provenance().kind(),
            PhysicalProposalKind::ScheduledKernel
        );
        let rejected_kinds: Vec<PhysicalProposalKind> = frontier
            .rejections()
            .iter()
            .filter_map(|rejection| match rejection {
                FrontierRejection::UnsupportedVariant { kind, .. } => Some(*kind),
                _ => None,
            })
            .collect();
        assert!(rejected_kinds.contains(&PhysicalProposalKind::View));
        // `KernelSubprogram` left this list when the frontier implemented it.
        // The seam it demonstrated is the same one `View` now demonstrates: an
        // unimplemented body rejects explicitly and the enumeration survives.
        assert!(!rejected_kinds.contains(&PhysicalProposalKind::KernelSubprogram));

        // The opaque proposal names an identity no entry claims, so it is
        // rejected *earlier* and differently: an opaque `Unregistered`, not
        // `UnsupportedVariant`. The two say different things — one is the
        // provider naming something that does not exist, the other this
        // compiler's limitation — and reporting the second would tell a caller
        // to wait for a feature when the fix is to register the call.
        assert!(
            !rejected_kinds.contains(&PhysicalProposalKind::OpaqueCall),
            "an unregistered call was reported as an unsupported variant"
        );
        assert!(
            frontier.rejections().iter().any(|rejection| matches!(
                rejection,
                FrontierRejection::OpaqueCall {
                    proposal,
                    cause: OpaqueCallRejectionCause::Unregistered,
                    ..
                } if proposal.call().call() == "mystery"
            )),
            "the unregistered opaque call was not reported by name"
        );
    }

    #[test]
    fn a_cheap_infeasible_proposal_is_rejected_while_an_expensive_feasible_one_is_admitted() {
        // Infeasibility is a disproved capability predicate, never a cost: a
        // proposal with a tiny cost estimate whose grid exceeds the profile is
        // rejected, while a proposal with a large cost estimate that fits is
        // admitted. Cost never gates feasibility in either direction.
        struct InfeasibleProvider;
        impl PhysicalImplementationProvider for InfeasibleProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(provider_identity("infeasible", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
                let (region, _) = pointwise_region(context.request());
                ProviderOffer::proposing(vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(region)),
                    governed_applicability(),
                    // A deliberately cheap estimate cannot rescue an infeasible plan.
                    PhysicalCostEstimate::structural(1, 1, 0),
                )])
            }
        }

        let large = request(Shape::from_dims([70_000, 1]), [Axis::new(1)]);
        let infeasible_subject = pointwise_subject(&large);
        let infeasible = InfeasibleProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&infeasible];
        let frontier = enumerate_frontier(
            &large,
            &infeasible_subject,
            &providers,
            &OpaqueCallRegistry::new(),
        )
        .unwrap();
        assert!(
            frontier.is_empty(),
            "an infeasible frontier is a valid empty result"
        );
        assert_eq!(frontier.rejections().len(), 1);
        let FrontierRejection::Infeasible {
            axis,
            required,
            available,
            ..
        } = &frontier.rejections()[0]
        else {
            panic!("expected a hard-infeasibility rejection, not a cost");
        };
        assert_eq!(*axis, "grid-axis");
        assert_eq!(*required, 70_000);
        assert_eq!(*available, 4);

        // A feasible proposal with an expensive estimate is still admitted.
        let small = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let feasible_subject = fused_subject(&small);
        let expensive = FusedScheduledKernelProvider {
            provider: provider_identity("expensive", 1),
            cost: PhysicalCostEstimate::structural(u32::MAX, u64::MAX, u64::MAX),
        };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&expensive];
        let frontier = enumerate_frontier(
            &small,
            &feasible_subject,
            &providers,
            &OpaqueCallRegistry::new(),
        )
        .unwrap();
        assert_eq!(
            frontier.admitted().len(),
            1,
            "cost never rejects a feasible plan"
        );
    }

    #[test]
    fn a_malformed_scheduled_kernel_fails_the_enumeration_closed() {
        // A provider that corrupts the numerical realization of its scheduled
        // region emits invalid IR. Re-entering checked verification rejects it,
        // and the frontier fails closed with a malformed-proposal error, distinct
        // from a valid empty no-plan result.
        struct MalformedProvider;
        impl PhysicalImplementationProvider for MalformedProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(provider_identity("malformed", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
                let mut region = fused_region(context.request());
                region.index.numerical.canonical_arithmetic_nan_bits ^= 1;
                ProviderOffer::proposing(vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(region)),
                    governed_applicability(),
                    PhysicalCostEstimate::structural(1, 2, 0),
                )])
            }
        }

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let malformed = MalformedProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&malformed];
        let error = enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new())
            .unwrap_err();
        assert!(matches!(error, FrontierError::MalformedProposal { .. }));
    }

    #[test]
    fn an_ungoverned_cost_model_is_malformed_output() {
        struct WrongCostModelProvider;
        impl PhysicalImplementationProvider for WrongCostModelProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(provider_identity("wrong-cost", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
                ProviderOffer::proposing(vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                    governed_applicability(),
                    PhysicalCostEstimate::new("tiler.cost.ungoverned.v9", 1, 2, 0),
                )])
            }
        }

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = WrongCostModelProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let error = enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new())
            .unwrap_err();
        assert!(matches!(
            error,
            FrontierError::MalformedCostProvenance {
                declared_model_key: "tiler.cost.ungoverned.v9",
                ..
            }
        ));
    }

    /// An analytical component cost cannot be admitted as a structural estimate.
    ///
    /// `crate::component_cost` reports costs under its own governed key and must
    /// never reach dominance: plans carrying different model keys do not dominate
    /// each other, so admitting a second key here would make the non-dominated
    /// set the whole set and turn Pareto pruning off with nothing reporting it.
    ///
    /// The type system already separates them — a `ComponentCost` is not a
    /// `PhysicalCostEstimate` and has no conversion — so this test guards the
    /// remaining route, which is someone constructing an estimate that *claims*
    /// the analytical key. The frontier must refuse it by name.
    #[test]
    fn an_analytical_cost_key_is_refused_by_the_frontier() {
        struct AnalyticalCostProvider;
        impl PhysicalImplementationProvider for AnalyticalCostProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(provider_identity("analytical", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
                ProviderOffer::proposing(vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                    governed_applicability(),
                    PhysicalCostEstimate::new(crate::component_cost::ANALYTICAL_MODEL_KEY, 1, 2, 0),
                )])
            }
        }

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = AnalyticalCostProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let error = enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new())
            .unwrap_err();
        assert!(
            matches!(
                error,
                FrontierError::MalformedCostProvenance {
                    declared_model_key: "tiler.cost.analytical.v1",
                    ..
                }
            ),
            "an analytical key reached the structural frontier: {error:?}"
        );
    }

    /// Each body answers for itself and declines the other's question.
    ///
    /// The `Option` accessors are the point: a consumer needing a schedule and
    /// holding an opaque call must handle the absence rather than receive a
    /// substitute. Both directions are asserted, so an accessor returning
    /// `Some` unconditionally fails on one of them.
    #[test]
    fn an_implementation_body_answers_only_for_its_own_kind() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let scheduled = ImplementationBody::Scheduled(Box::new(
            crate::physical::build_fused_scheduled_region(&request)
                .expect("the fused region builds"),
        ));

        assert!(scheduled.scheduled().is_some());
        assert!(
            scheduled.opaque().is_none(),
            "a scheduled region answered as an opaque call"
        );
        assert_eq!(scheduled.kind(), "scheduled-region");
    }

    /// A read binding yields a requirement and a write binding a guarantee, each
    /// keyed by the tensor role the provider bound it to.
    ///
    /// The roles are what a contract is keyed by, so this also confirms the
    /// derivation follows the *binding* rather than the parameter's position or
    /// its own role name.
    #[test]
    fn an_opaque_contract_follows_the_provider_bindings() {
        use super::derive_call_boundary_contract;
        use crate::boundary::{AdmittedMemoryDomains, ExecutionAffinity, MemoryDomainClass};
        use crate::call_abi::{CallAbi, ParameterLayout, ParameterRole, ParameterSpec};
        use crate::call_declaration::{OpaqueCallDeclaration, WorkScaling};
        use crate::call_placement::CallPlacement;
        use crate::effects::{Aliasing, CallEffects, Elimination, Motion};
        use tiler_ir::schedule::{NumericalPermission, ResourceRequirements, SubnormalMode};

        let spec = |name, role| ParameterSpec {
            name,
            role,
            layout: match role {
                ParameterRole::In => {
                    ParameterLayout::Required(crate::boundary::LayoutRequirement::DenseRowMajor)
                }
                _ => ParameterLayout::Guaranteed(crate::boundary::LayoutGuarantee::DenseRowMajor),
            },
            encoding: crate::boundary::StorageEncoding::Unpacked,
            alignment: crate::boundary::ByteAlignment::F32_NATURAL,
        };
        let declaration = OpaqueCallDeclaration::check(
            CallAbi::declare([spec("x", ParameterRole::In), spec("y", ParameterRole::Out)])
                .expect("well formed"),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
                &[MemoryDomainClass::Device],
            )
            .expect("supported"),
            ResourceRequirements {
                buffer_bindings: 4,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                requires_device_memory: true,
                input_subnormals: SubnormalMode::Preserve,
                result_subnormals: SubnormalMode::Preserve,
                contraction: NumericalPermission::Forbidden,
                reassociation: NumericalPermission::Forbidden,
                permutation: NumericalPermission::Forbidden,
                signed_zero: NumericalPermission::Forbidden,
                nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
                infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            },
            WorkScaling::Fixed(1),
        )
        .expect("coherent");

        let contract = derive_call_boundary_contract(
            &declaration,
            &[
                (
                    "x",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                ),
                ("y", TensorRole::Output),
            ],
        )
        .expect("a single admitted domain gives a guarantee");

        assert_eq!(contract.requirements.len(), 1);
        assert_eq!(
            contract.requirements[0].tensor(),
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST
            }
        );
        assert_eq!(contract.guarantees.len(), 1);
        assert_eq!(contract.guarantees[0].tensor(), TensorRole::Output);

        // Binding the *same* parameters to swapped roles moves the contract with
        // them: the derivation reads the binding, not the parameter order.
        let swapped = derive_call_boundary_contract(
            &declaration,
            &[
                ("x", TensorRole::Output),
                (
                    "y",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                ),
            ],
        )
        .expect("still one domain");
        assert_eq!(swapped.requirements[0].tensor(), TensorRole::Output);
        assert_eq!(
            swapped.guarantees[0].tensor(),
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST
            }
        );
    }

    /// A declared scaling resolves through the role its parameter is bound to.
    ///
    /// The two roles must give *different* counts, or the test would pass
    /// against a resolution that ignored the binding entirely — the shapes are
    /// chosen so `input_elements` and `output_elements` differ.
    #[test]
    fn work_scaling_resolves_through_the_bound_role() {
        use super::{WorkScaling, resolve_work_items};

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let normalized = request.serial_sum();
        assert_ne!(
            normalized.input_elements, normalized.output_elements,
            "the fixture cannot distinguish the two roles"
        );

        let bindings = [
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            ("y", TensorRole::Output),
        ];
        assert_eq!(
            resolve_work_items(WorkScaling::PerElementOf("x"), &bindings, &request),
            Ok(normalized.input_elements)
        );
        assert_eq!(
            resolve_work_items(WorkScaling::PerElementOf("y"), &bindings, &request),
            Ok(normalized.output_elements)
        );
        assert_eq!(
            resolve_work_items(WorkScaling::Fixed(7), &bindings, &request),
            Ok(7),
            "a fixed scaling was not taken directly"
        );
    }

    /// An unbound name and an intermediate binding both decline.
    ///
    /// Declining is the point: a work count nothing supports would produce a
    /// feasibility verdict that is confidently wrong in either direction.
    #[test]
    fn an_unresolvable_scaling_declines_rather_than_guessing() {
        use super::{WorkResolutionError, WorkScaling, resolve_work_items};

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let bindings = [
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            ("z", TensorRole::Intermediate),
        ];

        assert_eq!(
            resolve_work_items(WorkScaling::PerElementOf("absent"), &bindings, &request),
            Err(WorkResolutionError::UnknownParameter("absent")),
            "a scaling naming an unbound parameter produced a count"
        );
        // An intermediate declines. A previous revision resolved it to the
        // input's count on a falsified premise — the all-singleton cover
        // materializes every internal value, including rank-0 constants — so a
        // count here would be confidently wrong for exactly the covers that
        // exist. The decline is the honest answer until the cover edge's own
        // shape is in hand.
        assert_eq!(
            resolve_work_items(WorkScaling::PerElementOf("z"), &bindings, &request),
            Err(WorkResolutionError::IntermediateShapeUnavailable { parameter: "z" }),
            "an intermediate binding produced a count the subject cannot support"
        );
    }

    fn strict_call_resources() -> tiler_ir::schedule::ResourceRequirements {
        let contract = crate::request::StrictF32NumericalContract::governed().realization();
        tiler_ir::schedule::ResourceRequirements {
            buffer_bindings: 2,
            threads_per_workgroup: 1,
            local_memory_bytes: 0,
            requires_device_memory: true,
            input_subnormals: contract.input_subnormals,
            result_subnormals: contract.result_subnormals,
            contraction: contract.contraction,
            reassociation: contract.reassociation,
            permutation: contract.permutation,
            signed_zero: contract.signed_zero,
            nan_assumptions: contract.nan_assumptions,
            infinity_assumptions: contract.infinity_assumptions,
        }
    }

    fn call_declaration(
        resources: tiler_ir::schedule::ResourceRequirements,
    ) -> crate::call_declaration::OpaqueCallDeclaration {
        use crate::boundary::{
            AdmittedMemoryDomains, ByteAlignment, ExecutionAffinity, LayoutGuarantee,
            LayoutRequirement, MemoryDomainClass, StorageEncoding,
        };
        use crate::call_abi::{CallAbi, ParameterLayout, ParameterRole, ParameterSpec};
        use crate::call_declaration::{OpaqueCallDeclaration, WorkScaling};
        use crate::call_placement::CallPlacement;
        use crate::effects::{Aliasing, CallEffects, Elimination, Motion};

        let spec = |name, role| ParameterSpec {
            name,
            role,
            layout: match role {
                ParameterRole::In => ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
                _ => ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
            },
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::F32_NATURAL,
        };
        OpaqueCallDeclaration::check(
            CallAbi::declare([spec("x", ParameterRole::In), spec("y", ParameterRole::Out)])
                .expect("well formed"),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
                &[MemoryDomainClass::Device],
            )
            .expect("supported"),
            resources,
            WorkScaling::PerElementOf("x"),
        )
        .expect("coherent")
    }

    struct CallProvider(
        crate::call_registry::OpaqueCallIdentity,
        Vec<(&'static str, TensorRole)>,
    );

    impl PhysicalImplementationProvider for CallProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(provider_identity("opaque", 1))
        }
        fn propose(&self, _context: &ImplementationContext<'_>) -> ProviderOffer {
            ProviderOffer::proposing(vec![ImplementationProposal::new(
                ProposalBody::OpaqueCall(Box::new(
                    OpaqueCallProposal::new(self.0, self.1.clone())
                        .expect("fixture proposal is exactly reportable"),
                )),
                governed_applicability(),
                PhysicalCostEstimate::structural(1, 2, 0),
            )])
        }
    }

    /// **The ticket's core claim:** a scheduled kernel and an opaque call are
    /// alternatives for one region, and the frontier admits both.
    ///
    /// Neither is preferred by construction — both enter the admitted set and
    /// the choice between them is left to cost, which is what "additive
    /// coexistence" has to mean. A frontier that admitted only one, or that
    /// ordered them by kind, would pass every other test in this file.
    #[test]
    fn a_scheduled_kernel_and_an_opaque_call_coexist_as_alternatives() {
        use crate::call_registry::OpaqueCallIdentity;

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let identity = OpaqueCallIdentity::new("test", "both", 1).expect("named");
        let bindings = vec![
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            ("y", TensorRole::Output),
        ];

        let mut registry = OpaqueCallRegistry::new();
        registry
            .register(identity, call_declaration(strict_call_resources()))
            .expect("one call");

        let scheduled = GovernedPhysicalProvider;
        let opaque = CallProvider(identity, bindings);
        let providers: [&dyn PhysicalImplementationProvider; 2] = [&scheduled, &opaque];
        let frontier = enumerate_frontier(&request, &subject, &providers, &registry).unwrap();

        assert_eq!(
            frontier.rejections().len(),
            0,
            "one of the two was rejected: {:?}",
            frontier.rejections()
        );
        assert_eq!(frontier.admitted().len(), 2, "both were not admitted");

        let kinds: Vec<PhysicalProposalKind> = frontier
            .admitted()
            .iter()
            .map(|admitted| admitted.provenance().kind())
            .collect();
        assert!(kinds.contains(&PhysicalProposalKind::ScheduledKernel));
        assert!(kinds.contains(&PhysicalProposalKind::OpaqueCall));

        // Exactly one carries a schedule, which is what makes them genuinely
        // different implementations rather than two spellings of one.
        assert_eq!(
            frontier
                .admitted()
                .iter()
                .filter(|admitted| admitted.scheduled().is_some())
                .count(),
            1
        );
    }

    /// An `InOut` parameter's role carries both a requirement and a guarantee.
    ///
    /// The regression this pins: selecting the read parameter by `!writes()`
    /// silently excluded `InOut` (which writes), so its read requirement was
    /// dropped and a producer of that tensor was never asked to satisfy
    /// anything. Both halves are asserted, and the declaration uses
    /// `MayAliasInputs` because an in-place parameter beside `Distinct` is
    /// refused by the coherence check.
    #[test]
    fn an_in_out_binding_yields_both_a_requirement_and_a_guarantee() {
        use crate::boundary::{
            AdmittedMemoryDomains, ByteAlignment, ExecutionAffinity, LayoutGuarantee,
            LayoutRequirement, MemoryDomainClass, StorageEncoding,
        };
        use crate::call_abi::{CallAbi, ParameterLayout, ParameterRole, ParameterSpec};
        use crate::call_declaration::{OpaqueCallDeclaration, WorkScaling};
        use crate::call_placement::CallPlacement;
        use crate::effects::{Aliasing, CallEffects, Elimination, Motion};
        use tiler_ir::schedule::{NumericalPermission, SubnormalMode};

        let declaration = OpaqueCallDeclaration::check(
            CallAbi::declare([ParameterSpec {
                name: "buffer",
                role: ParameterRole::InOut,
                layout: ParameterLayout::Both {
                    requires: LayoutRequirement::DenseRowMajor,
                    guarantees: LayoutGuarantee::DenseRowMajor,
                },
                encoding: StorageEncoding::Unpacked,
                alignment: ByteAlignment::F32_NATURAL,
            }])
            .expect("well formed"),
            CallEffects::declared(
                Elimination::Required,
                Motion::Ordered,
                Aliasing::MayAliasInputs,
            ),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
                &[MemoryDomainClass::Device],
            )
            .expect("supported"),
            tiler_ir::schedule::ResourceRequirements {
                buffer_bindings: 1,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                requires_device_memory: true,
                input_subnormals: SubnormalMode::Preserve,
                result_subnormals: SubnormalMode::Preserve,
                contraction: NumericalPermission::Forbidden,
                reassociation: NumericalPermission::Forbidden,
                permutation: NumericalPermission::Forbidden,
                signed_zero: NumericalPermission::Forbidden,
                nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
                infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            },
            WorkScaling::Fixed(1),
        )
        .expect("coherent");

        let contract = super::derive_call_boundary_contract(
            &declaration,
            &[(
                "buffer",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            )],
        )
        .expect("one admitted domain");

        assert_eq!(
            contract.requirements.len(),
            1,
            "the in-out parameter's read requirement was dropped"
        );
        assert_eq!(
            contract.requirements[0].tensor(),
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST
            }
        );
        assert_eq!(
            contract.guarantees.len(),
            1,
            "the in-out parameter's write guarantee was dropped"
        );
        assert_eq!(
            contract.guarantees[0].tensor(),
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST
            }
        );
    }

    /// The opaque derivation can produce a guarantee the bounded profile's own
    /// requirements refuse — so the enforcers deferral trigger is no longer
    /// sufficient on its own.
    ///
    /// `the_bounded_profile_admits_no_undischarged_boundary` compares the two
    /// compile-time constant property sets, and boundary contracts are now
    /// *also* built from provider declarations (`derive_call_boundary_contract`
    /// via `call_declaration`). This pins that the new path genuinely reaches a
    /// mismatch the constant test structurally cannot see: a call declaring
    /// `MayAliasInputs` guarantees `AliasView`, which no `MaterializedBuffer`
    /// requirement accepts. Green today because no compile-path provider
    /// proposes such a call; the enforcers ticket's startable-condition is
    /// therefore "a compile-path provider proposes one", not "the constant
    /// test fails".
    #[test]
    fn an_opaque_declaration_can_produce_a_guarantee_the_bounded_profile_refuses() {
        use crate::boundary::{
            AdmittedMemoryDomains, BoundaryProperty, ExecutionAffinity, MemoryDomainClass,
            unsatisfied_properties,
        };
        use crate::call_abi::{CallAbi, ParameterLayout, ParameterRole, ParameterSpec};
        use crate::call_declaration::{
            OpaqueCallDeclaration, WorkScaling, guaranteed_properties_for,
        };
        use crate::call_placement::CallPlacement;
        use crate::effects::{Aliasing, CallEffects, Elimination, Motion};
        use tiler_ir::schedule::{NumericalPermission, ResourceRequirements, SubnormalMode};

        let declaration = OpaqueCallDeclaration::check(
            CallAbi::declare([
                ParameterSpec {
                    name: "x",
                    role: ParameterRole::In,
                    layout: ParameterLayout::Required(
                        crate::boundary::LayoutRequirement::DenseRowMajor,
                    ),
                    encoding: crate::boundary::StorageEncoding::Unpacked,
                    alignment: crate::boundary::ByteAlignment::F32_NATURAL,
                },
                ParameterSpec {
                    name: "y",
                    role: ParameterRole::Out,
                    layout: ParameterLayout::Guaranteed(
                        crate::boundary::LayoutGuarantee::DenseRowMajor,
                    ),
                    encoding: crate::boundary::StorageEncoding::Unpacked,
                    alignment: crate::boundary::ByteAlignment::F32_NATURAL,
                },
            ])
            .expect("well formed"),
            CallEffects::declared(
                Elimination::Required,
                Motion::Ordered,
                Aliasing::MayAliasInputs,
            ),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
                &[MemoryDomainClass::Device],
            )
            .expect("supported"),
            ResourceRequirements {
                buffer_bindings: 2,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                requires_device_memory: true,
                input_subnormals: SubnormalMode::Preserve,
                result_subnormals: SubnormalMode::Preserve,
                contraction: NumericalPermission::Forbidden,
                reassociation: NumericalPermission::Forbidden,
                permutation: NumericalPermission::Forbidden,
                signed_zero: NumericalPermission::Forbidden,
                nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
                infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            },
            WorkScaling::Fixed(1),
        )
        .expect("coherent");

        let guaranteed = guaranteed_properties_for(
            declaration.abi().parameter("y").expect("declared"),
            declaration.effects(),
            declaration.placement(),
        )
        .expect("a write parameter guarantees");

        let unsatisfied = unsatisfied_properties(&bounded_requirements(), &guaranteed);
        assert!(
            unsatisfied
                .iter()
                .any(|u| u.property() == BoundaryProperty::Materialization),
            "an AliasView guarantee satisfied a MaterializedBuffer requirement, \
             so the mismatch the enforcers ticket waits for is not producible \
             and its deferral note is wrong the other way"
        );
    }

    /// A call whose declared numerics differ from the request's contract is
    /// refused, even though the target could honour them.
    ///
    /// The two questions are different: `assess_resources` asks whether the
    /// *device* offers the behaviour, and this asks whether the *program* asked
    /// for it. A call permitting contraction is feasible on a device that offers
    /// it and still wrong for a program whose contract forbids it.
    #[test]
    fn a_call_whose_numerics_differ_from_the_contract_is_refused() {
        use crate::call_registry::OpaqueCallIdentity;
        use tiler_ir::schedule::NumericalPermission;

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let identity = OpaqueCallIdentity::new("test", "loose", 1).expect("named");
        let bindings = vec![
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            ("y", TensorRole::Output),
        ];

        // Permitting contraction where the governed contract forbids it.
        let mut resources = strict_call_resources();
        resources.contraction = NumericalPermission::Permitted;
        let declaration = call_declaration(resources);

        let mut registry = OpaqueCallRegistry::new();
        registry.register(identity, declaration).expect("one call");

        let provider = CallProvider(identity, bindings);
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier = enumerate_frontier(&request, &subject, &providers, &registry).unwrap();

        assert!(frontier.admitted().is_empty(), "a loose call was admitted");
        assert!(
            frontier.rejections().iter().any(|rejection| matches!(
                rejection,
                FrontierRejection::OpaqueCall {
                    cause: OpaqueCallRejectionCause::NumericalContractMismatch,
                    ..
                }
            )),
            "the refusal did not name the numerical mismatch: {:?}",
            frontier.rejections()
        );
    }

    /// A registered, well-bound, feasible opaque call is admitted.
    ///
    /// The payoff for the whole seam: an implementation this compiler did not
    /// produce enters the frontier beside a scheduled kernel, carrying a
    /// boundary contract derived from its declaration and feasibility proved by
    /// the same authority a region uses.
    #[test]
    fn a_registered_well_bound_opaque_call_is_admitted() {
        use super::{derive_call_boundary_contract, resolve_work_items};
        use crate::boundary::{
            AdmittedMemoryDomains, ExecutionAffinity, LayoutGuarantee, LayoutRequirement,
            MemoryDomainClass, StorageEncoding,
        };
        use crate::call_abi::{CallAbi, ParameterLayout, ParameterRole, ParameterSpec};
        use crate::call_declaration::{OpaqueCallDeclaration, WorkScaling};
        use crate::call_placement::CallPlacement;
        use crate::effects::{Aliasing, CallEffects, Elimination, Motion};
        use tiler_ir::schedule::{NumericalPermission, ResourceRequirements, SubnormalMode};

        let spec = |name, role| ParameterSpec {
            name,
            role,
            layout: match role {
                ParameterRole::In => ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
                _ => ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
            },
            encoding: StorageEncoding::Unpacked,
            alignment: crate::boundary::ByteAlignment::F32_NATURAL,
        };
        let declaration = OpaqueCallDeclaration::check(
            CallAbi::declare([spec("x", ParameterRole::In), spec("y", ParameterRole::Out)])
                .expect("well formed"),
            CallEffects::declared(Elimination::Required, Motion::Ordered, Aliasing::Distinct),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("non-empty"),
                &[MemoryDomainClass::Device],
            )
            .expect("supported"),
            ResourceRequirements {
                buffer_bindings: 2,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                requires_device_memory: true,
                input_subnormals: SubnormalMode::Preserve,
                result_subnormals: SubnormalMode::Preserve,
                contraction: NumericalPermission::Forbidden,
                reassociation: NumericalPermission::Forbidden,
                permutation: NumericalPermission::Forbidden,
                signed_zero: NumericalPermission::Forbidden,
                nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
                infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            },
            WorkScaling::PerElementOf("x"),
        )
        .expect("coherent");

        let identity = OpaqueCallIdentity::new("test", "sum", 1).expect("named");
        let bindings = vec![
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            ("y", TensorRole::Output),
        ];
        let mut registry = OpaqueCallRegistry::new();
        registry
            .register(identity, declaration.clone())
            .expect("one call");

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = CallProvider(identity, bindings.clone());
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier = enumerate_frontier(&request, &subject, &providers, &registry).unwrap();

        assert_eq!(
            frontier.rejections().len(),
            0,
            "a well-formed opaque call was rejected: {:?}",
            frontier.rejections()
        );
        assert_eq!(frontier.admitted().len(), 1);
        let admitted = &frontier.admitted()[0];
        assert_eq!(
            admitted.provenance().kind(),
            PhysicalProposalKind::OpaqueCall
        );
        assert!(
            admitted.scheduled().is_none(),
            "an opaque admission reported a scheduled region"
        );
        assert_eq!(admitted.body().kind(), "opaque-call");

        // The contract is the one the declaration derives, and the work count is
        // the one the binding resolves — asserted against the same functions the
        // admission used, so a wired-up-but-wrong admission fails here.
        let expected = derive_call_boundary_contract(&declaration, &bindings).expect("derivable");
        assert_eq!(
            admitted.boundary().requirements().len(),
            expected.requirements().len()
        );
        assert_eq!(
            resolve_work_items(WorkScaling::PerElementOf("x"), &bindings, &request),
            Ok(request.serial_sum().input_elements)
        );
    }

    /// Every opaque refusal has a distinct typed canonical spelling, and that
    /// spelling retains the complete call identity and ordered bindings.
    #[test]
    fn opaque_rejection_identity_and_all_causes_are_canonical() {
        use super::{WorkResolutionError, classify_opaque_resource_verdict, encode_rejection};
        use crate::call_abi::BindingError;
        use crate::call_declaration::GuaranteeError;
        use crate::physical::ResourceVerdict;
        use crate::target::feasibility::{RejectionCause, TargetProfileIdentity};
        use crate::target::honourability::{
            DeclaredBehaviour, DimensionBehaviour, HonouringMeans, NumericalDimension,
            UnhonouredDimension, governed_profile_source,
        };
        use tiler_ir::schedule::{ArithmeticType, NumericalPermission};

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let provider = PhysicalProviderProvenance::new(provider_identity("opaque-causes", 7))
            .expect("fixture provider is exactly reportable");
        let call = OpaqueCallIdentity::new("owner", "call", 9).expect("canonical");
        let proposal = OpaqueCallProposal::new(
            call,
            vec![
                (
                    "x",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                ),
                ("y", TensorRole::Output),
            ],
        )
        .expect("fixture proposal is exactly reportable");

        let mut excessive = strict_call_resources();
        excessive.buffer_bindings = u32::MAX;
        let capability = match crate::physical::assess_resources(
            excessive,
            request.numerical_contract().arithmetic,
            1,
            request.target_profile(),
        )
        .expect_err("the buffer requirement exceeds the target")
        {
            ResourceVerdict::Rejected(RejectionCause::Capability(predicate)) => predicate,
            other => panic!("expected a capability rejection, got {other:?}"),
        };
        // Honest checked evidence, not a synthetic summary: a refusal names a
        // fact a profile declared, so the fixture declares one and attributes
        // it, exactly as a target profile does.
        let required = DimensionBehaviour::Transform(NumericalPermission::Permitted);
        let unhonourable = UnhonouredDimension::new(
            DeclaredBehaviour::new(
                NumericalDimension::Contraction,
                ArithmeticType::F32,
                F32::resolved_type(),
                required,
                HonouringMeans::Unsupported,
                governed_profile_source(),
            )
            .attributed_to(TargetProfileIdentity::new("tiler.test.profile.v1")),
            required,
            Some(DimensionBehaviour::Transform(
                NumericalPermission::Forbidden,
            )),
        );

        let causes = [
            OpaqueCallRejectionCause::NotApplicable {
                target_profile_key: TargetProfileKey::governed("tiler.test.other.v1"),
            },
            OpaqueCallRejectionCause::Unregistered,
            OpaqueCallRejectionCause::MalformedBinding(BindingError::UnboundParameter("x")),
            OpaqueCallRejectionCause::ContractUnderivable(GuaranteeError::AmbiguousWriteDomain),
            OpaqueCallRejectionCause::NumericalContractMismatch,
            OpaqueCallRejectionCause::WorkUnresolvable(
                WorkResolutionError::IntermediateShapeUnavailable { parameter: "x" },
            ),
            OpaqueCallRejectionCause::TargetInfeasible(capability),
            OpaqueCallRejectionCause::TargetUnhonourable(unhonourable.clone()),
        ];
        let encodings: Vec<Vec<u8>> = causes
            .into_iter()
            .map(|cause| {
                encode_rejection(&FrontierRejection::OpaqueCall {
                    provider: provider.clone(),
                    proposal: proposal.clone(),
                    cause,
                })
            })
            .collect();
        for (index, encoding) in encodings.iter().enumerate() {
            assert!(
                encodings[..index].iter().all(|seen| seen != encoding),
                "opaque cause {index} collided with an earlier cause"
            );
        }

        let variants = [
            OpaqueCallProposal::new(
                OpaqueCallIdentity::new("other", "call", 9).unwrap(),
                proposal.bindings().to_vec(),
            )
            .expect("fixture proposal is exactly reportable"),
            OpaqueCallProposal::new(
                OpaqueCallIdentity::new("owner", "other", 9).unwrap(),
                proposal.bindings().to_vec(),
            )
            .expect("fixture proposal is exactly reportable"),
            OpaqueCallProposal::new(
                OpaqueCallIdentity::new("owner", "call", 10).unwrap(),
                proposal.bindings().to_vec(),
            )
            .expect("fixture proposal is exactly reportable"),
            OpaqueCallProposal::new(
                call,
                vec![
                    ("y", TensorRole::Output),
                    (
                        "x",
                        TensorRole::Input {
                            ordinal: InputOrdinal::FIRST,
                        },
                    ),
                ],
            )
            .expect("fixture proposal is exactly reportable"),
            OpaqueCallProposal::new(
                call,
                vec![
                    ("x", TensorRole::Output),
                    (
                        "y",
                        TensorRole::Input {
                            ordinal: InputOrdinal::FIRST,
                        },
                    ),
                ],
            )
            .expect("fixture proposal is exactly reportable"),
        ];
        let baseline = encode_rejection(&FrontierRejection::OpaqueCall {
            provider: provider.clone(),
            proposal: proposal.clone(),
            cause: OpaqueCallRejectionCause::Unregistered,
        });
        for variant in variants {
            assert_ne!(
                baseline,
                encode_rejection(&FrontierRejection::OpaqueCall {
                    provider: provider.clone(),
                    proposal: variant,
                    cause: OpaqueCallRejectionCause::Unregistered,
                }),
                "a call identity or ordered-binding distinction was erased"
            );
        }

        let binding_faults = [
            BindingError::UnboundParameter("x"),
            BindingError::UnknownParameter("x"),
            BindingError::ParameterBoundTwice("x"),
            BindingError::RoleStorageDisagreement {
                first: "x",
                second: "y",
            },
        ];
        let fault_encodings: Vec<Vec<u8>> = binding_faults
            .into_iter()
            .map(|fault| {
                encode_rejection(&FrontierRejection::OpaqueCall {
                    provider: provider.clone(),
                    proposal: proposal.clone(),
                    cause: OpaqueCallRejectionCause::MalformedBinding(fault),
                })
            })
            .collect();
        for (index, encoding) in fault_encodings.iter().enumerate() {
            assert!(
                fault_encodings[..index].iter().all(|seen| seen != encoding),
                "binding fault {index} collided with an earlier fault"
            );
        }

        assert!(matches!(
            classify_opaque_resource_verdict(
                &provider,
                &proposal,
                ResourceVerdict::Rejected(RejectionCause::Capability(capability)),
            ),
            Ok(FrontierRejection::OpaqueCall {
                cause: OpaqueCallRejectionCause::TargetInfeasible(_),
                ..
            })
        ));
        assert!(matches!(
            classify_opaque_resource_verdict(
                &provider,
                &proposal,
                ResourceVerdict::Rejected(RejectionCause::Numerical(unhonourable)),
            ),
            Ok(FrontierRejection::OpaqueCall {
                cause: OpaqueCallRejectionCause::TargetUnhonourable(_),
                ..
            })
        ));
        assert!(matches!(
            classify_opaque_resource_verdict(
                &provider,
                &proposal,
                ResourceVerdict::Intrinsic(
                    crate::target::feasibility::FeasibilityError::MalformedProposal {
                        rule: "test"
                    },
                ),
            ),
            Err(FrontierError::MalformedOpaqueCallAssessment { .. })
        ));
        assert!(matches!(
            classify_opaque_resource_verdict(&provider, &proposal, ResourceVerdict::Unknown),
            Err(FrontierError::UnresolvedOpaqueCallAssessment { .. })
        ));
    }

    #[test]
    fn a_proposal_for_another_target_is_not_applicable() {
        struct ForeignTargetProvider;
        impl PhysicalImplementationProvider for ForeignTargetProvider {
            fn provenance(
                &self,
            ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
                PhysicalProviderProvenance::new(provider_identity("foreign", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
                ProviderOffer::proposing(vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                    TargetApplicability::for_targets([TargetProfileKey::governed(
                        "tiler.some-other-target.v1",
                    )]),
                    PhysicalCostEstimate::structural(1, 2, 0),
                )])
            }
        }

        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = ForeignTargetProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();
        assert!(frontier.admitted().is_empty());
        assert_eq!(frontier.rejections().len(), 1);
        assert!(matches!(
            &frontier.rejections()[0],
            FrontierRejection::NotApplicable {
                kind: PhysicalProposalKind::ScheduledKernel,
                target_profile_key,
                ..
            } if target_profile_key.as_str() == GOVERNED_TARGET_KEY
        ));
    }

    /// One refusing fact reaches every rejection surface with its provenance
    /// intact, and provenance alone moves the canonical identity.
    ///
    /// Both halves are the point. The first is that no surface reconstructs the
    /// fact: each rejection cites the very instance the feasibility authority
    /// refused on, checked by pointer, which structural equality could not
    /// distinguish from a plausible rebuild. The second is that the identity
    /// those surfaces are sorted and deduplicated by actually reads the
    /// provenance — before this, two profiles refusing the same behaviour on
    /// different measured builds encoded identically, so one refusal's
    /// explanation could stand in for the other's.
    #[test]
    fn one_refusing_fact_reaches_every_rejection_surface_with_its_provenance() {
        use super::encode_rejection;
        use crate::request::{ContractRejection, StrictF32NumericalContract};
        use crate::target::TargetProfile;
        use crate::target::feasibility::{FeasibilityOutcome, RejectionCause};
        use crate::target::honourability::{
            FactSourceProvenance, UnhonouredDimension, governed_profile_source,
            measured_profile_source,
        };

        fn refusal(key: &str, source: std::sync::Arc<FactSourceProvenance>) -> UnhonouredDimension {
            let profile = TargetProfile::refusing_preserved_subnormals_for_test(key, source);
            let FeasibilityOutcome::Rejected(rejection) =
                crate::physical::assess_contract(&profile, StrictF32NumericalContract::governed())
                    .expect("the refusing test profile is intrinsically valid")
            else {
                panic!("a declared refusal disproves a hard predicate");
            };
            let RejectionCause::Numerical(cause) = rejection.representative() else {
                panic!("a contract-only proposal states no capability requirement");
            };
            cause
        }

        let provider = PhysicalProviderProvenance::new(provider_identity("refusal-carry", 3))
            .expect("fixture provider is exactly reportable");
        let call = OpaqueCallIdentity::new("owner", "call", 9).expect("canonical");
        let proposal = OpaqueCallProposal::new(
            call,
            vec![(
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            )],
        )
        .expect("fixture proposal is exactly reportable");

        let cause = refusal(
            "test.refusal-carry.v1",
            measured_profile_source("test.probe.v1", "1.0", "build-1"),
        );
        let origin = cause.evidence();
        assert_eq!(
            origin.authority(),
            crate::target::feasibility::FactAuthority::MeasuredProfile
        );

        // Contract rejection, frontier rejection, and opaque-call rejection all
        // carry the one fact onward rather than summarizing it.
        let contract = ContractRejection::Unhonourable {
            contract_key: StrictF32NumericalContract::governed().key,
            cause: cause.clone(),
        };
        let ContractRejection::Unhonourable { cause: carried, .. } = &contract else {
            panic!("the contract rejection retains its declared refusal");
        };
        assert!(carried.evidence().cites_same_fact(&origin));

        let frontier = FrontierRejection::Unhonourable {
            provider: provider.clone(),
            cause: cause.clone(),
        };
        let FrontierRejection::Unhonourable { cause: carried, .. } = &frontier else {
            panic!("the frontier rejection retains its declared refusal");
        };
        assert!(carried.evidence().cites_same_fact(&origin));

        let opaque = FrontierRejection::OpaqueCall {
            provider: provider.clone(),
            proposal: proposal.clone(),
            cause: OpaqueCallRejectionCause::TargetUnhonourable(cause.clone()),
        };
        let FrontierRejection::OpaqueCall {
            cause: OpaqueCallRejectionCause::TargetUnhonourable(carried),
            ..
        } = &opaque
        else {
            panic!("the opaque-call rejection retains its declared refusal");
        };
        assert!(carried.evidence().cites_same_fact(&origin));

        // Perturbing one provenance field at a time. Each must move both
        // canonical spellings; none may move what the caller required.
        let baseline_frontier = encode_rejection(&frontier);
        let baseline_opaque = encode_rejection(&opaque);
        for (label, perturbed) in [
            (
                "authority and validity",
                refusal("test.refusal-carry.v1", governed_profile_source()),
            ),
            (
                "authority identity",
                refusal(
                    "test.refusal-carry.v1",
                    measured_profile_source("test.other-probe.v1", "1.0", "build-1"),
                ),
            ),
            (
                "compiler build",
                refusal(
                    "test.refusal-carry.v1",
                    measured_profile_source("test.probe.v1", "2.0", "build-1"),
                ),
            ),
            (
                "execution environment",
                refusal(
                    "test.refusal-carry.v1",
                    measured_profile_source("test.probe.v1", "1.0", "build-2"),
                ),
            ),
        ] {
            assert_eq!(
                perturbed.required(),
                cause.required(),
                "{label} changed what the caller required",
            );
            assert_eq!(
                perturbed.dimension(),
                cause.dimension(),
                "{label} changed the refused dimension",
            );
            assert_ne!(
                baseline_frontier,
                encode_rejection(&FrontierRejection::Unhonourable {
                    provider: provider.clone(),
                    cause: perturbed.clone(),
                }),
                "{label} left the frontier rejection identity unchanged",
            );
            assert_ne!(
                baseline_opaque,
                encode_rejection(&FrontierRejection::OpaqueCall {
                    provider: provider.clone(),
                    proposal: proposal.clone(),
                    cause: OpaqueCallRejectionCause::TargetUnhonourable(perturbed),
                }),
                "{label} left the opaque-call rejection identity unchanged",
            );
        }
    }

    /// Dominance is boundary-aware: two implementations of *different*
    /// boundaries are incomparable however their costs compare.
    ///
    /// The bounded profile derives one contract per region, so two admitted
    /// implementations of one region always share a contract and cost is what
    /// separates them; the boundary half of the relation is therefore exercised
    /// against two regions' real derived contracts rather than end to end. That
    /// is a measurement boundary on this test, not on the relation.
    #[test]
    fn boundary_subsumption_gates_dominance_before_cost_is_consulted() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let fused = enumerate_frontier(
            &request,
            &fused_subject(&request),
            &[&FusedScheduledKernelProvider {
                provider: provider_identity("fused", 1),
                cost: PhysicalCostEstimate::structural(1, 2, 0),
            } as &dyn PhysicalImplementationProvider],
            &OpaqueCallRegistry::new(),
        )
        .unwrap();
        let pointwise = enumerate_frontier(
            &request,
            &pointwise_subject(&request),
            &[&GovernedPhysicalProvider as &dyn PhysicalImplementationProvider],
            &OpaqueCallRegistry::new(),
        )
        .unwrap();

        let fused_contract = fused.admitted()[0].boundary();
        let pointwise_contract = pointwise.admitted()[0].boundary();
        // The fused region produces the program Output; the pointwise prologue
        // produces the cross-region Intermediate. Different boundaries.
        assert_ne!(
            fused_contract.guarantees()[0].tensor(),
            pointwise_contract.guarantees()[0].tensor()
        );
        assert!(!fused_contract.subsumes(pointwise_contract));
        assert!(!pointwise_contract.subsumes(fused_contract));
        // A contract always subsumes itself, so cost remains the separator for
        // two implementations of one region.
        assert!(fused_contract.subsumes(fused_contract));
    }

    #[test]
    fn non_domination_retains_the_pareto_frontier_after_feasibility() {
        // Three feasible proposals of the same region: a dominated one (worse on a
        // dimension, no better on any) is pruned; two incomparable ones are both
        // retained. Pruning runs strictly after feasibility admission.
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let cheap = FusedScheduledKernelProvider {
            provider: provider_identity("cheap", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let dominated = FusedScheduledKernelProvider {
            provider: provider_identity("dominated", 1),
            cost: PhysicalCostEstimate::structural(1, 4, 0),
        };
        let trade_off = FusedScheduledKernelProvider {
            provider: provider_identity("tradeoff", 1),
            cost: PhysicalCostEstimate::structural(2, 1, 0),
        };
        let providers: [&dyn PhysicalImplementationProvider; 3] = [&cheap, &dominated, &trade_off];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();

        assert_eq!(frontier.admitted().len(), 3, "feasibility admits all three");
        let non_dominated: Vec<&ProviderIdentity> = frontier
            .non_dominated()
            .iter()
            .map(|admitted| admitted.provenance().provider())
            .collect();
        assert_eq!(non_dominated.len(), 2);
        assert!(non_dominated.contains(&&provider_identity("cheap", 1)));
        assert!(non_dominated.contains(&&provider_identity("tradeoff", 1)));
        assert!(!non_dominated.contains(&&provider_identity("dominated", 1)));
    }

    #[test]
    fn a_frontier_with_no_providers_is_a_valid_empty_result() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let providers: [&dyn PhysicalImplementationProvider; 0] = [];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();
        assert!(frontier.is_empty());
        assert!(frontier.rejections().is_empty());
        assert_eq!(frontier.region_role(), "fused");
        assert_eq!(frontier.target_profile_key(), GOVERNED_TARGET_KEY);
    }

    #[test]
    fn accessors_expose_seam_metadata() {
        // Exercise the reserved-seam and cost accessors so the draft surface is
        // covered rather than latently dead.
        let seam = ReservedProposalSeam::new("intrinsic.mystery");
        assert_eq!(seam.descriptor(), "intrinsic.mystery");
        let cost = PhysicalCostEstimate::structural(2, 3, 4);
        assert_eq!(cost.model_key(), "tiler.cost.structural.v1");
        assert_eq!(cost.dispatch_count(), 2);
        assert_eq!(cost.launched_threads(), 3);
        assert_eq!(cost.temporary_bytes(), 4);
        let applicability = TargetApplicability::for_targets([
            TargetProfileKey::governed(GOVERNED_TARGET_KEY),
            TargetProfileKey::governed(GOVERNED_TARGET_KEY),
        ]);
        assert_eq!(
            applicability.target_profile_keys(),
            [TargetProfileKey::governed(GOVERNED_TARGET_KEY)],
        );
    }

    /// Keeps the unused-field lint honest for the reserved seam descriptor and the
    /// admitted-implementation verified region accessor.
    #[test]
    fn admitted_exposes_its_verified_region() {
        fn _uses_admitted(admitted: &AdmittedImplementation) {
            let _ = admitted
                .scheduled()
                .expect("a scheduled admission")
                .region();
            let _ = admitted.cost();
        }
    }

    // -----------------------------------------------------------------------
    // The subprogram seam: every new check driven against a case that fails
    // -----------------------------------------------------------------------

    /// A relaxed-contract request whose reduction admits a balanced split.
    fn splittable_request(shape: Shape) -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_request(CompilationRequest::governed_under(
            &program,
            crate::request::StrictF32NumericalContract::governed_relaxed(),
        ))
        .unwrap();
        request.for_target(0).unwrap()
    }

    fn reduction_subject(request: &VerifiedTargetRequest) -> FrontierRegionSubject {
        FrontierRegionSubject::new(
            "reduction",
            request.serial_sum().members.reduction().to_vec(),
        )
    }

    /// A provider that proposes a caller-supplied subprogram verbatim.
    ///
    /// It exists to drive the host's own composition checks: every stage it
    /// hands over is a *verifying* region, so anything the frontier refuses is
    /// refused for how the stages compose rather than for what any one of them
    /// is.
    struct SubprogramProvider {
        stages: Vec<SubprogramStage>,
    }

    impl PhysicalImplementationProvider for SubprogramProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(provider_identity("subprogram", 1))
        }
        fn propose(&self, _: &ImplementationContext<'_>) -> ProviderOffer {
            ProviderOffer::proposing(vec![ImplementationProposal::new(
                ProposalBody::KernelSubprogram(Box::new(KernelSubprogram::new(
                    self.stages.clone(),
                ))),
                governed_applicability(),
                PhysicalCostEstimate::structural(2, 6, 16),
            )])
        }
    }

    /// Returns the governed split's two raw passes for one request.
    fn split_stages(request: &VerifiedTargetRequest) -> Vec<SubprogramStage> {
        crate::physical::split_reduction_regions(request)
            .expect("a four-contributor relaxed request admits the split")
            .stages
            .into_iter()
            .map(|(region, members)| SubprogramStage::new(region, members))
            .collect()
    }

    /// Enumerates one subprogram against a subject and returns the outcome.
    fn enumerate_subprogram(
        request: &VerifiedTargetRequest,
        subject: &FrontierRegionSubject,
        stages: Vec<SubprogramStage>,
    ) -> Result<ImplementationFrontier, FrontierError> {
        let provider = SubprogramProvider { stages };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        enumerate_frontier(request, subject, &providers, &OpaqueCallRegistry::new())
    }

    /// The governed split composes; three perturbations of it do not.
    ///
    /// Each perturbation leaves every stage individually valid — the same
    /// regions, the same claimed members — and changes only how they compose,
    /// which is exactly the class of fault no single region can see. Without
    /// these the chain derivation could return a contract for any sequence of
    /// verified regions and nothing would notice.
    #[test]
    fn a_subprogram_chain_that_does_not_compose_is_malformed_output() {
        let request = splittable_request(Shape::from_dims([1, 4]));
        let subject = reduction_subject(&request);

        // The governed chain composes, and its boundary is the one the serial
        // reduction offers: one intermediate read in, one output write out. The
        // partial tensor never appears, because it never leaves.
        let admitted = enumerate_subprogram(&request, &subject, split_stages(&request)).unwrap();
        assert_eq!(admitted.admitted().len(), 1);
        let boundary = admitted.admitted()[0].boundary();
        assert_eq!(boundary.requirements().len(), 1);
        assert_eq!(
            boundary.requirements()[0].tensor(),
            TensorRole::Intermediate
        );
        assert_eq!(boundary.guarantees().len(), 1);
        assert_eq!(boundary.guarantees()[0].tensor(), TensorRole::Output);

        // Reversed: the first stage now publishes the program output, which is
        // not something a non-final stage can hand on.
        let mut reversed = split_stages(&request);
        reversed.reverse();
        assert!(matches!(
            enumerate_subprogram(&request, &subject, reversed).unwrap_err(),
            FrontierError::UndeterminedBoundaryProperty {
                rule: "subprogram-stages-not-chained",
                ..
            }
        ));

        // The partial pass followed by the prologue: both stages verify, and
        // between them they cover the fused subject exactly, so coverage does
        // not catch it. The prologue reads the program input rather than the
        // staged partials, which leaves those partials with no consumer — the
        // leak the cover cannot see and the assembler would have to invent an
        // owner for.
        let leaking = vec![split_stages(&request)[0].clone(), {
            let (region, members) = pointwise_region(&request);
            SubprogramStage::new(region, members)
        }];
        assert!(matches!(
            enumerate_subprogram(&request, &fused_subject(&request), leaking).unwrap_err(),
            FrontierError::UndeterminedBoundaryProperty {
                rule: "subprogram-stages-not-chained",
                ..
            }
        ));

        // One stage is not a chain: a single dispatch is a scheduled kernel,
        // and admitting it here would give one region two identities.
        let stages = split_stages(&request);
        assert!(matches!(
            enumerate_subprogram(&request, &subject, vec![stages[0].clone()]).unwrap_err(),
            FrontierError::UndeterminedBoundaryProperty {
                rule: "subprogram-stages-not-chained",
                ..
            }
        ));
    }

    /// A subprogram must cover its subject exactly, and neither less nor more.
    ///
    /// The coverage check runs before any stage is verified, because it is a
    /// claim about the *set* of occurrences two dispatches realize between them
    /// — something no per-stage request-subject binding can see. Offered against
    /// the pointwise subject, the governed split covers the reduction instead:
    /// admitting it would compute the reduction twice and the prologue never.
    #[test]
    fn a_subprogram_covering_another_subject_is_refused_before_verification() {
        let request = splittable_request(Shape::from_dims([1, 4]));
        let foreign = pointwise_subject(&request);
        assert!(matches!(
            enumerate_subprogram(&request, &foreign, split_stages(&request)).unwrap_err(),
            FrontierError::UndeterminedBoundaryProperty {
                rule: "subprogram-coverage",
                ..
            }
        ));
    }

    /// A split's identity separates it from the serial reduction and from a
    /// split of a different shape.
    ///
    /// Both directions matter. The first is what keeps the two alternatives
    /// distinct in the admitted set; the second is what keeps two different
    /// splits from colliding, which folding only the stage count would allow.
    #[test]
    fn a_subprogram_identity_folds_its_ordered_chain() {
        let request = splittable_request(Shape::from_dims([1, 4]));
        let subject = reduction_subject(&request);
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();
        let identities: Vec<&[u8]> = frontier
            .admitted()
            .iter()
            .map(|admitted| admitted.identity().as_bytes())
            .collect();
        assert_eq!(identities.len(), 2);
        assert_ne!(identities[0], identities[1]);

        // A different splittable extent is a different chain, so a different
        // identity. Reusing one would let a cached artifact answer for the
        // wrong program.
        let wider = splittable_request(Shape::from_dims([1, 6]));
        let wider_subject = reduction_subject(&wider);
        let other = enumerate_frontier(
            &wider,
            &wider_subject,
            &providers,
            &OpaqueCallRegistry::new(),
        )
        .unwrap();
        for admitted in other.admitted() {
            assert!(
                !identities.contains(&admitted.identity().as_bytes()),
                "two splits over different extents share one identity"
            );
        }
    }
}
