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
//! Nothing here is a compiler API: `frontier` is a private module carrying no
//! `pub` item and no re-export, so this is a crate-internal draft vocabulary.
//! The draft discipline still holds — every shape below is provisional and
//! carries no compatibility story — and the acceptance a public boundary owes
//! Tom is owed at the point any of it is first exported, not here.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::schedule::{
    AccessMode, ResourceRequirements, ScalarProgram, ScheduledRegion, TensorRole,
};
use tiler_ir::semantic::ProviderIdentity;

use crate::boundary::{
    AdmittedMemoryDomains, AvailabilityGuarantee, AvailabilityRequirement, ByteAlignment,
    ExecutionAffinity, GuaranteedProperties, GuaranteedProperty, LayoutGuarantee,
    LayoutRequirement, MaterializationForm, MemoryDomainClass, RequiredProperties,
    RequiredProperty, StorageEncoding, StorageScalar, VisibilityGuarantee, VisibilityRequirement,
};
use crate::call_declaration::{GuaranteeError, OpaqueCallDeclaration, WorkScaling};
use crate::call_registry::{OpaqueCallProposal, OpaqueCallRegistry, RegisteredCall};
use crate::physical::{
    AdmissionEvidence, PhysicalError, ResourceVerdict, VerifiedScheduledRegion,
    verify_schedule_with_feasibility,
};
use crate::region::SemanticStage;
use crate::request::{TargetProfile, TargetProfileKey, VerifiedTargetRequest};
use crate::target::feasibility::{
    FeasibilityError, RejectionCause, ResolvedPredicate, UnrealizableSynchronization,
};
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
/// The provider states the region *and* the attribution atoms that stage claims,
/// because a subprogram's stages do not claim the subject uniformly: a split
/// reduction's partial pass claims the reduction occurrence's first stage and
/// its final pass claims the stage after it, so each names the part of the
/// occurrence it computes instead of one of them naming nothing.
///
/// Neither field is believed. Each region is resubmitted through
/// [`verify_schedule_with_feasibility`] with the members declared here, and that
/// path's request-subject binding is what decides whether this exact region may
/// claim exactly these occurrences — so a provider that mislabels a pass is
/// rejected by the same authority that checks a single-kernel proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubprogramStage {
    region: ScheduledRegion,
    semantic_members: Vec<SemanticStage>,
}

impl SubprogramStage {
    /// Builds one proposed stage from its region and the members it claims.
    pub(crate) const fn new(region: ScheduledRegion, semantic_members: Vec<SemanticStage>) -> Self {
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

/// Rule code for a region whose scalar program fixes no single boundary carrier.
///
/// The derived contract states one storage encoding and one alignment for every
/// value the region binds. A program whose boundary values have different
/// carriers has no such single answer, so it is refused with a reason rather
/// than served the widest of them — over-alignment is not a conservative default
/// here, it is a requirement stated about a value that does not have it.
const UNMODELLED_BOUNDARY_CARRIER_RULE: &str = "boundary-carrier-unmodelled";

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
/// - **alignment** is the natural alignment of the boundary value's own element
///   type, derived rather than stated: [`boundary_carrier`] reads the region's
///   scalar program for the physical carrier its boundary values have, and
///   [`ByteAlignment::natural_for`] takes that carrier's width from
///   `StorageScalar::byte_width`. A region whose program fixes no single carrier
///   is refused with [`UNMODELLED_BOUNDARY_CARRIER_RULE`] rather than given the
///   widest one. In the bounded profile every admitted program is `f32`
///   throughout, so the derived answer is four bytes — but it is now four
///   because the element is four wide, not because the profile said so;
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
    let carrier =
        boundary_carrier(&region.index.scalar_program).ok_or(UNMODELLED_BOUNDARY_CARRIER_RULE)?;
    let mut requirements = Vec::new();
    let mut guarantees = Vec::new();
    for access in &region.index.accesses {
        if access.ownership.is_some() {
            guarantees.push(BoundaryGuarantee {
                tensor: access.tensor,
                ownership: BoundaryOwnership::TotalRaceFreeWrite,
                properties: bounded_guarantees(carrier),
            });
        } else {
            requirements.push(BoundaryRequirement {
                tensor: access.tensor,
                access: access.mode,
                properties: bounded_requirements(carrier),
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
    // Whether this chain's last stage is a publishing copy. It is the one shape
    // in which a *non-final* stage's write also leaves the subprogram: the copy
    // exists because the value is published and consumed, so the value it copies
    // from is the materialization edge some other cover region reads. Declaring
    // only the copy's own write would state a contract that omits a buffer the
    // cover joins on, and the plan would then fail to compose for a reason that
    // is a gap in this derivation rather than a fact about the regions.
    let publishes_a_copy = stages
        .last()
        .is_some_and(|stage| stage.region().index.id == crate::physical::PUBLISHING_COPY_REGION);
    for (position, stage) in stages.iter().enumerate() {
        let region = stage.region();
        if !region.index.accesses.is_empty() && !stage.requirements().requires_device_memory {
            return Err(NO_BOUNDARY_DOMAIN_RULE);
        }
        // Per stage rather than once for the chain: a stage's boundary values
        // are its own program's, and the handoff between two stages is checked
        // by shape below rather than assumed to share a carrier.
        let carrier = boundary_carrier(&region.index.scalar_program)
            .ok_or(UNMODELLED_BOUNDARY_CARRIER_RULE)?;
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
                        properties: bounded_guarantees(carrier),
                    });
                } else if access.tensor == TensorRole::Intermediate {
                    handoff = Some(&region.index.iteration_shape);
                    if publishes_a_copy {
                        guarantees.push(BoundaryGuarantee {
                            tensor: access.tensor,
                            ownership: BoundaryOwnership::TotalRaceFreeWrite,
                            properties: bounded_guarantees(carrier),
                        });
                    }
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
                properties: bounded_requirements(carrier),
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
///
/// **The synchronization requirement is not a peak, and it is not aggregated.**
/// It is one atomic subject, so "the largest of them" is undefined: two stages
/// requiring different realizations require *both*, and carrying either one
/// forward would compose a permission for one stage out of a fact about the
/// other. A subprogram whose stages disagree therefore has no single requirement
/// and is refused here, before any target is asked. Stages that all require
/// nothing carry `None`; stages that all require the same subject carry it once.
fn subprogram_resources(stages: &[VerifiedScheduledRegion]) -> Option<ResourceRequirements> {
    let mut peak = stages.first()?.requirements();
    for stage in &stages[1..] {
        let stage = stage.requirements();
        peak.buffer_bindings = peak.buffer_bindings.max(stage.buffer_bindings);
        peak.threads_per_workgroup = peak.threads_per_workgroup.max(stage.threads_per_workgroup);
        peak.local_memory_bytes = peak.local_memory_bytes.max(stage.local_memory_bytes);
        peak.requires_device_memory |= stage.requires_device_memory;
        peak.synchronization = match (peak.synchronization, stage.synchronization) {
            (None, other) | (other, None) => other,
            (Some(left), Some(right)) if left == right => Some(left),
            (Some(_), Some(_)) => return None,
        };
    }
    Some(peak)
}

/// The physical storage carrier every boundary value of `program` has.
///
/// The region's scalar program is what fixes this, and it can always be asked:
/// `IndexRegion::scalar_program` is a required field, so there is no region
/// whose carrier is unknown because the program is absent. `tiler-ir`'s
/// `verify_signature` derives each buffer's kernel type from the same match, so
/// this states the physical carrier beside a type map that already exists rather
/// than introducing a second authority for what a region's elements are.
///
/// Exhaustive with no wildcard arm, deliberately: [`ScalarProgram`] is not
/// `#[non_exhaustive]` precisely so that a new program is a build error at every
/// site that must classify it, and a carrier guessed for an unrecognized program
/// is the silently-wrong answer this derivation exists to prevent.
///
/// `None` means the program's boundary values do not share one carrier, which is
/// a refusal rather than a defect — see [`UNMODELLED_BOUNDARY_CARRIER_RULE`].
const fn boundary_carrier(program: &ScalarProgram) -> Option<StorageScalar> {
    match program {
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        // The epilogue computes at the region's own arithmetic width and commits
        // its result, so the boundary this region writes carries the same `f32`
        // payload the bare fold's does.
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::StrictTensorContraction { .. }
        | ScalarProgram::StrictSerialMaximum { .. } => Some(StorageScalar::F32),
        // The physical carrier half of BF16's vertical: a two-byte carrier whose
        // natural alignment `ByteAlignment::natural_for` derives from
        // `StorageScalar::byte_width`, so the boundary contract this function
        // feeds states two bytes rather than four without a second width table.
        // `natural_access_type` pairs it with `KernelType::Bf16`, which is the
        // element type the kernel signature independently expects — the two
        // agree by derivation rather than by both being written down.
        ScalarProgram::PointwiseBf16(_) => Some(StorageScalar::Bf16),
        // The one program in the vocabulary whose boundary values disagree:
        // `tiler-ir`'s `verify_signature` fixes its reads at `[U8, F32, U8]`,
        // and its code component is bit-packed rather than unpacked. The
        // contract below states one encoding and one alignment for the whole
        // region, so no single answer here is right — and answering with the
        // widest of the three would over-align the code buffer, which is the
        // "passes for the wrong reason" outcome this derivation exists to stop.
        // `physical::verify_region_subject_binding` already refuses this program
        // upstream, so no region reaching here today takes this arm.
        ScalarProgram::StrictAffineU4Dequantize { .. } => None,
    }
}

/// The typed properties the bounded profile's regions require of an input.
///
/// The alignment is derived from `carrier`, the boundary value's own element
/// type, rather than stated by the profile; every other dimension is still the
/// profile's, and [`derive_boundary_contract`] documents each one's source.
///
/// # Panics
///
/// Panics only if these values violate the property model's own well-formedness
/// rules, which no reachable input can cause.
fn bounded_requirements(carrier: StorageScalar) -> RequiredProperties {
    RequiredProperties::new([
        RequiredProperty::StorageLayout(LayoutRequirement::DenseRowMajor),
        RequiredProperty::StorageEncoding(StorageEncoding::Unpacked),
        RequiredProperty::Alignment(ByteAlignment::natural_for(carrier)),
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
/// The alignment derives from `carrier` for the same reason it does on the
/// requirement side: a guarantee that over-states alignment is as much a claim
/// about the value's element type as an under-stated requirement is.
///
/// # Panics
///
/// Panics under the same unreachable condition as [`bounded_requirements`].
fn bounded_guarantees(carrier: StorageScalar) -> GuaranteedProperties {
    GuaranteedProperties::new([
        GuaranteedProperty::StorageLayout(LayoutGuarantee::DenseRowMajor),
        GuaranteedProperty::StorageEncoding(StorageEncoding::Unpacked),
        GuaranteedProperty::Alignment(ByteAlignment::natural_for(carrier)),
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
    /// The schedule vocabulary has no region spelling the occurrences this
    /// cover region groups.
    ///
    /// The one cause that is a fact about the *region* rather than about the
    /// request. The three above are decided from the request's permissions and
    /// extents before any region exists, and hold for every region of that
    /// request; this one holds for the exact occurrences a cover placed
    /// together, and a different cover of the same program hits a different
    /// answer.
    ///
    /// **Which occurrences those are is the region's canonical occurrence
    /// identity, which the frontier record is keyed by, and it is deliberately
    /// not restated here as a member ordinal.** A [`SemanticStage`]'s member is a
    /// graph-local *authoring* coordinate: the two spellings of the governed
    /// program that `product_is_deterministic_and_preserves_the_materialized_boundary`
    /// compares number the same occurrence `0` and `1`, so a cause carrying one
    /// would put an authoring accident into a canonical encoding and into the
    /// trace. The occurrence count is a property of the region itself and
    /// carries safely.
    UnspellableRegion {
        /// Stable code naming which region-vocabulary wall the region hit.
        rule: &'static str,
        /// How many occurrences the region covers.
        covered: u32,
    },
}

impl StrategyDeclineCause {
    /// Returns the stable reason code of the decline.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::NumericalPermissionRefused { .. } => "numerical-permission-refused",
            Self::NoAdmissibleShape { rule, .. }
            | Self::Unrepresentable { rule }
            | Self::UnspellableRegion { rule, .. } => rule,
        }
    }

    /// Appends this cause's canonical encoding.
    ///
    /// **Appends-only, carried by per-tag injectivity at this site rather than
    /// by a green gate.** Each variant writes a distinct leading tag byte —
    /// `0x01`, `0x02`, `0x03`, `0x04` — and no variant writes another's, so two
    /// causes can share an encoding only if one variant's payload equals its
    /// own for two distinct values. Within `0x04` the rule is length-prefixed
    /// and the count is a fixed four-byte field, so the payload is a bijection
    /// onto `(rule, covered)`. `0x04` was unused before this variant existed,
    /// so every previously encoded cause keeps its exact bytes and no pinned
    /// identity moves.
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
            Self::UnspellableRegion { rule, covered } => {
                output.push(0x04);
                push_slice(output, rule.as_bytes());
                output.extend_from_slice(&covered.to_be_bytes());
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
    semantic_members: Vec<SemanticStage>,
    /// Element counts of the cover-materialized intermediates this region reads,
    /// deduplicated and ascending.
    ///
    /// An intermediate exists because a *cover* chose to materialize between two
    /// regions, so its size is a fact the cover holds and the region subject does
    /// not — which is why it is stated here by the caller that holds the cover
    /// rather than derived from the members. Empty means the subject was stated
    /// without a cover, and a work scaling that needs one then declines rather
    /// than being sized against another tensor.
    intermediate_elements: Vec<u64>,
    /// The tensor this region's owning write targets, as the cover assigned it.
    ///
    /// Stated here for the same reason `intermediate_elements` is: it is a fact
    /// the *cover* holds. The same elementwise occurrences write a declared
    /// program output in one cover and a materialized intermediate in another,
    /// and asking the request instead gives every region of one program the
    /// same answer.
    write: crate::physical::RegionWrite,
}

impl FrontierRegionSubject {
    /// Builds a region subject from a presentation role, its exact members, and
    /// the tensor its owning write targets.
    ///
    /// The subject reads no cover-materialized intermediate. A local enumeration
    /// for a region considered outside any cover is exactly this case, and a
    /// shape-dependent opaque call bound to an intermediate declines for it.
    #[allow(
        dead_code,
        reason = "the coverless constructor states the local-authority case this module documents; the compile path always holds a cover and states its edges instead, so it is exercised by this authority's own tests"
    )]
    pub(crate) fn new(
        role: &'static str,
        semantic_members: Vec<SemanticStage>,
        write: crate::physical::RegionWrite,
    ) -> Self {
        Self {
            role,
            semantic_members,
            intermediate_elements: Vec::new(),
            write,
        }
    }

    /// Builds a region subject that reads cover-materialized intermediates.
    ///
    /// The counts are normalized to a deduplicated ascending order, so two
    /// subjects reading the same set of intermediate sizes are one subject and
    /// share one enumeration.
    pub(crate) fn reading_intermediates(
        role: &'static str,
        semantic_members: Vec<SemanticStage>,
        intermediate_elements: impl IntoIterator<Item = u64>,
        write: crate::physical::RegionWrite,
    ) -> Self {
        let mut intermediate_elements: Vec<u64> = intermediate_elements.into_iter().collect();
        intermediate_elements.sort_unstable();
        intermediate_elements.dedup();
        Self {
            role,
            semantic_members,
            intermediate_elements,
            write,
        }
    }

    /// Returns the stable presentation role of the region.
    pub(crate) const fn role(&self) -> &'static str {
        self.role
    }

    /// Returns the tensor this region's owning write targets.
    pub(crate) const fn write(&self) -> crate::physical::RegionWrite {
        self.write
    }

    /// Returns the exact recognized semantic occurrences the region covers.
    pub(crate) fn semantic_members(&self) -> &[SemanticStage] {
        &self.semantic_members
    }

    /// Returns the one element count every intermediate this region reads has.
    ///
    /// `None` when the subject reads none, or when it reads intermediates of
    /// different sizes — the second is a genuine ambiguity rather than a missing
    /// fact, and both decline for the same reason a work scaling may not be
    /// answered with a number nothing derived.
    fn intermediate_elements(&self) -> Option<u64> {
        match self.intermediate_elements.as_slice() {
            [count] => Some(*count),
            [] | [_, _, ..] => None,
        }
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
    semantic_members: Vec<SemanticStage>,
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
    pub(crate) fn semantic_members(&self) -> &[SemanticStage] {
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
    /// The proposal is applicable and valid, but the target declares it cannot
    /// realize the synchronization the proposal's dataflow requires.
    ///
    /// Distinct from both variants above. A capability rejection says this plan
    /// does not fit; an unhonourable dimension says the target cannot compute
    /// what was asked. This says the target cannot *order* what this strategy's
    /// staged handoff requires — so a different strategy may well fit, but not by
    /// adjusting a bound.
    Unsynchronizable {
        /// The provider whose proposal was rejected.
        provider: PhysicalProviderProvenance,
        /// The complete refused subject and the fact that refused it.
        cause: Box<UnrealizableSynchronization>,
    },
    /// The proposal is applicable and valid, and no available target fact speaks
    /// to the synchronization its dataflow requires.
    ///
    /// Distinct from [`Self::Unsynchronizable`], which carries a fact that
    /// refused. This carries none, because there is none — and that difference
    /// is the whole reason the two are separate. A refusal names an authority a
    /// caller can go and argue with; silence names a question nobody asked, and
    /// reporting one as the other would either invent a refusing profile or hide
    /// that a target was never measured for this realization.
    ///
    /// It is a rejection rather than a [`FrontierError`]: the provider emitted
    /// valid IR, so failing the whole enumeration would attribute the target's
    /// silence to the provider.
    SynchronizationUndeclared {
        /// The provider whose proposal was rejected.
        provider: PhysicalProviderProvenance,
        /// The complete subject the proposal required and nothing declared.
        subject: tiler_ir::schedule::SynchronizationSubject,
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
            // Appended tag `7`: every earlier rejection keeps its tag and its
            // field layout. The whole subject is encoded, because two refusals
            // differing only in fenced domain are different refusals.
            Self::Unsynchronizable { provider, cause } => {
                let subject = cause.subject();
                output.push(7);
                encode_provider(output, provider.provider());
                output.push(subject.kind.tag());
                output.push(subject.execution_scope.tag());
                output.push(subject.visibility_scope.tag());
                output.push(u8::from(subject.fenced_spaces.workgroup));
                output.push(u8::from(subject.fenced_spaces.device));
                output.push(subject.ordering.tag());
                push_slice(output, cause.fact().provenance().profile().key().as_bytes());
            }
            // Appended tag `8`, with no profile key after the subject: there is
            // no declaring profile, and writing an empty slice there would give
            // this rejection the shape of a refusal by an unnamed authority.
            Self::SynchronizationUndeclared { provider, subject } => {
                output.push(8);
                encode_provider(output, provider.provider());
                output.push(subject.kind.tag());
                output.push(subject.execution_scope.tag());
                output.push(subject.visibility_scope.tag());
                output.push(u8::from(subject.fenced_spaces.workgroup));
                output.push(u8::from(subject.fenced_spaces.device));
                output.push(subject.ordering.tag());
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
                        subject,
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
                Err(PhysicalError::Synchronization { cause, .. }) => {
                    rejections.push(FrontierRejection::Unsynchronizable {
                        provider: provenance.clone(),
                        cause,
                    });
                }
                Err(PhysicalError::UnrealizedSynchronization { subject, .. }) => {
                    rejections.push(FrontierRejection::SynchronizationUndeclared {
                        provider: provenance.clone(),
                        subject,
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
        // A registered call declares its own resource requirements, and that
        // declaration surface cannot state a synchronization one — so a call
        // never carries the requirement and the refusal is unreachable today. It
        // shares the unresolved arm rather than being wildcarded: both mean this
        // call's feasibility was not decided, which is the same thing to report,
        // and spelling the pattern keeps a later reachable path visible in this
        // match instead of swallowed by a `_`.
        ResourceVerdict::Rejected(RejectionCause::Synchronization(_))
        | ResourceVerdict::UnrealizedSynchronization(_)
        | ResourceVerdict::Unknown => {
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
    /// The parameter is bound to an intermediate whose cover-specific shape this
    /// subject does not determine — it reads none, or it reads several of
    /// different sizes.
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
/// # How `Intermediate` resolves, and when it still declines
///
/// An intermediate is a cover-level artefact — it exists because a cover chose
/// to materialize between two regions — so its element count is a property of
/// that cover. The subject now *carries* that count when it was stated from a
/// cover: [`crate::cover::MaterializationEdge`] holds the materialized value's
/// element count, and the caller that holds the cover states the counts of the
/// edges this region consumes on [`FrontierRegionSubject::reading_intermediates`].
///
/// It still declines in the two cases where nothing derived an answer: a subject
/// stated without a cover reads no intermediate at all, and a subject reading
/// intermediates of *different* sizes has no single count for a scaling that
/// names the role rather than a particular edge.
///
/// A previous revision resolved it to `input_elements` on the claim that "the
/// bounded profile has exactly one materialization: the pointwise result".
/// That claim was false: `enumerate_covers` retains the all-singleton cover
/// unconditionally, and that cover materializes **every** internal value —
/// including rank-0 scalar constants, whose element count is 1, not the
/// input's. Substituting `input_elements` there is exactly the
/// confidently-wrong feasibility verdict `WorkScaling` exists to prevent, and
/// the reason the count is taken from the cover edge rather than from any
/// tensor that happens to be in scope.
fn resolve_work_items(
    work: WorkScaling,
    bindings: &[(&'static str, TensorRole)],
    subject: &FrontierRegionSubject,
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
                // Resolved per ordinal, because the contraction strategy admits
                // two inputs of different extents: `td,od->to` binds `[M, K]` to
                // ordinal 0 and `[N, K]` to ordinal 1, and answering with either
                // one for both would size a call against the wrong tensor. An
                // ordinal no declared input occupies is a refusal rather than
                // another tensor's size.
                //
                // Both roles are additionally resolved only when every recognized
                // output *agrees* on the count, and the reason is that an opaque
                // call's binding names the tensor role rather than a particular
                // tensor. `TensorRole::Output` carries no ordinal at all, so with
                // several declared outputs nothing on the binding says which
                // published tensor it means; and two outputs may read one
                // declared input at different domains, a reduction at its
                // contributor shape and an elementwise sibling at its own.
                // Answering with one of them would size a call against a tensor
                // the caller did not name, which is the confidently-wrong verdict
                // `WorkScaling` exists to prevent, so a disagreement refuses by
                // the same route an unoccupied ordinal does.
                TensorRole::Input { ordinal } => request
                    .normalized()
                    .agreed_input_elements_at(*ordinal)
                    .ok_or(WorkResolutionError::UnknownParameter(name)),
                TensorRole::Output => request
                    .normalized()
                    .agreed_output_elements()
                    .ok_or(WorkResolutionError::UnknownParameter(name)),
                TensorRole::Intermediate => subject
                    .intermediate_elements()
                    .ok_or(WorkResolutionError::IntermediateShapeUnavailable { parameter: name }),
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
/// can see: the atoms its stages claim between them must realize exactly the
/// subject's occurrences, each once, which
/// [`crate::region::chain_realizes_subject`] decides. A chain covering less
/// would silently drop work the cover assigned to this region, and a chain
/// covering more would compute an occurrence another region also computes.
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
    let mut claimed: Vec<SemanticStage> = Vec::new();
    for stage in &subprogram.stages {
        claimed.extend_from_slice(&stage.semantic_members);
    }
    if !crate::region::chain_realizes_subject(&mut claimed, &subject.semantic_members) {
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
            Err(PhysicalError::Synchronization { cause, .. }) => {
                return Ok(Err(FrontierRejection::Unsynchronizable {
                    provider: provider.clone(),
                    cause,
                }));
            }
            Err(PhysicalError::UnrealizedSynchronization { subject, .. }) => {
                return Ok(Err(FrontierRejection::SynchronizationUndeclared {
                    provider: provider.clone(),
                    subject,
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
        Err(ResourceVerdict::Rejected(RejectionCause::Synchronization(cause))) => {
            return Ok(Err(FrontierRejection::Unsynchronizable {
                provider: provider.clone(),
                cause: Box::new(cause),
            }));
        }
        Err(ResourceVerdict::UnrealizedSynchronization(subject)) => {
            return Ok(Err(FrontierRejection::SynchronizationUndeclared {
                provider: provider.clone(),
                subject,
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

/// Stable rule code naming a strategy with no place for a publishing dispatch.
///
/// A fact about the *region* the cover placed rather than about the request,
/// which is why it is carried as an [`StrategyDeclineCause::UnspellableRegion`]:
/// the same reduction in a cover that does not publish its value keeps both
/// parallel strategies.
const PUBLISHING_COPY_COMPOSITION_RULE: &str = "region-publishes-a-copy";

/// Namespace of Tiler's own governed physical implementation provider.
const GOVERNED_PHYSICAL_NAMESPACE: &str = "tiler";
/// Name of Tiler's own governed physical implementation provider.
const GOVERNED_PHYSICAL_NAME: &str = "prototype-serial-sum-physical";
/// Output-affecting revision of the governed physical provider.
const GOVERNED_PHYSICAL_REVISION: u32 = 1;

/// Tiler's own governed physical implementation provider for the bounded profile.
///
/// It answers for **every** region a cover places: the occurrences the subject
/// names are spelled against the schedule vocabulary by
/// [`crate::physical::spell_region`], and the answer is either one checked
/// scheduled-kernel body — with the split and the workgroup tree additive beside
/// it where the subject admits them — or a [`DeclinedStrategy`] naming the
/// region-vocabulary wall it hit.
///
/// **Silence is not among its answers for a region a cover placed**, and that is
/// the whole of what generalizing it changed. This build installs exactly one
/// provider, so its empty offer was indistinguishable from a coverage gap it
/// should have named: complete-plan selection saw an unimplemented region and a
/// reader of the trace saw an absence. A subject naming *no* occurrence is the
/// one case that still answers with an empty offer, because it is the local
/// enumeration the trait's contract describes rather than a region a cover
/// placed.
///
/// The provider declares only a body, an applicability predicate, and a cost
/// estimate. It cannot stamp its own provenance, derive its resources, or bypass
/// verification: the frontier resubmits every body through the ordinary checked
/// path in [`crate::physical::verify_schedule_with_feasibility`], so the
/// generalization gains it no trust it did not have.
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
        let subject = context.subject();
        let members = subject.semantic_members();
        // Structural cost inputs, taken over every recognized output rather than
        // resolved per region: they bound the widest thing a plan for this
        // request could stage, and a cost is an upper bound rather than a
        // feasibility answer. The region's *own* shapes come from the output the
        // spelling resolves below.
        let input_elements = request.normalized().max_input_elements();
        let output_elements = request.normalized().max_output_elements();
        // A materialized f32 intermediate costs four bytes per element. The
        // estimate is structural and is never a feasibility input.
        let intermediate_bytes = input_elements.saturating_mul(4);
        let applicability =
            TargetApplicability::for_targets([request.target_profile().profile_key().clone()]);
        // A subject naming no occurrence is the coverless local enumeration the
        // trait's contract describes, not a region a cover placed, so the empty
        // offer is the honest answer and a decline would name a wall no cover
        // hit.
        if members.is_empty() {
            return ProviderOffer::default();
        }
        let spelling = match crate::physical::spell_region(request, members, subject.write()) {
            Ok(spelling) => spelling,
            Err(wall) => {
                return ProviderOffer::default().decline(DeclinedStrategy::new(
                    crate::physical::SERIAL_BASELINE_STRATEGY,
                    StrategyDeclineCause::UnspellableRegion {
                        rule: wall.reason(),
                        covered: u32::try_from(members.len()).unwrap_or(u32::MAX),
                    },
                ));
            }
        };
        let mut split = None;
        let mut tree = None;
        // The recognized output whose partition this region belongs to. The
        // spelling resolved it from the cover's own occurrences, so every shape,
        // expression, and member set below is that output's rather than a
        // whole-program value that would answer the same for every region.
        let output = request.output_at(spelling.output());
        // Every region except the epilogue itself is built from the *producer's*
        // recognized shape, which is the output itself for a standalone one and
        // the staged producer for a chain. Asking for it here is what lets each
        // builder below stay written against one recognized family rather than
        // against "the output, unless it is a chain".
        let producer = output.producer_shape();
        let (region, cost) = match spelling.kind() {
            // One elementwise pass, whichever tensor the cover assigned its
            // write. The two write roles cost differently because the cover
            // decided differently: a region whose result another region reads
            // stages that result, and one that writes a declared program output
            // stages nothing.
            crate::physical::RegionSpellingKind::Pointwise(write) => (
                crate::physical::pointwise_region(request, producer, write).0,
                match write {
                    crate::physical::RegionWrite::ProgramOutput => {
                        PhysicalCostEstimate::structural(1, output_elements, 0)
                    }
                    // The staging half of a published-and-consumed region costs
                    // exactly what a materializing one costs; the publishing
                    // dispatch is added below, where the body becomes a
                    // subprogram, so that the cost and the dispatch count move
                    // together instead of at two sites.
                    crate::physical::RegionWrite::Materialized
                    | crate::physical::RegionWrite::MaterializedAndPublished => {
                        PhysicalCostEstimate::structural(1, input_elements, intermediate_bytes)
                    }
                },
            ),
            // The reduction subject is the one place a split is even a
            // candidate, so it is the one place the strategy is considered and
            // — when this request does not admit it — the one place the decline
            // is stated. The serial alternative is offered either way; a split
            // is additive and never replaces it.
            crate::physical::RegionSpellingKind::SerialSum => {
                // Both parallel strategies end in a pass that writes the tensor
                // the cover assigned, and a published-and-consumed region's
                // publication is a *further* dispatch after that pass. Composing
                // the two would be a three-pass shape nothing below here
                // assembles, so the strategies are declined by name rather than
                // offered as plans the assembler would refuse.
                if subject.write().publishes_a_copy() {
                    split = Some(Err(DeclinedStrategy::new(
                        crate::physical::MULTI_PASS_SPLIT_STRATEGY,
                        StrategyDeclineCause::UnspellableRegion {
                            rule: PUBLISHING_COPY_COMPOSITION_RULE,
                            covered: u32::try_from(members.len()).unwrap_or(u32::MAX),
                        },
                    )));
                    tree = Some(Err(DeclinedStrategy::new(
                        crate::physical::SINGLE_WORKGROUP_TREE_STRATEGY,
                        StrategyDeclineCause::UnspellableRegion {
                            rule: PUBLISHING_COPY_COMPOSITION_RULE,
                            covered: u32::try_from(members.len()).unwrap_or(u32::MAX),
                        },
                    )));
                } else {
                    split = Some(propose_split(
                        request,
                        producer,
                        &applicability,
                        subject.write(),
                    ));
                    tree = Some(propose_workgroup_tree(
                        request,
                        producer,
                        &applicability,
                        subject.write(),
                    ));
                }
                (
                    crate::physical::reduction_region(request, producer, subject.write()).0,
                    PhysicalCostEstimate::structural(1, output_elements, 0),
                )
            }
            // No split: a contraction's fold is the declared contributor
            // sequence, and splitting it would consume the reassociation this
            // family declares forbidden.
            crate::physical::RegionSpellingKind::Contraction => (
                crate::physical::contraction_region(request, producer, subject.write()).0,
                PhysicalCostEstimate::structural(1, output_elements, 0),
            ),
            // Whether the whole-program region may be *fused* belongs to the
            // numerical-legality authority and whether it *fits* belongs to this
            // target; neither is a capability question. Every occurrence the
            // region covers already resolved its lowering capability before any
            // cover reached this proposer, so no capability gap is left to
            // defer. A prologue with no fused spelling never reaches here —
            // `spell_region` declines it by name, so the materialized cover's
            // two regions remain the plan and the lost candidate is recorded.
            crate::physical::RegionSpellingKind::FusedSerialSum => (
                crate::physical::fused_region(request, producer, subject.write())
                    .expect("a fused spelling is decided before the region is built")
                    .0,
                PhysicalCostEstimate::structural(1, output_elements, 0),
            ),
            // The producing stage of a staged family. No split and no tile: the
            // epilogue applies to the complete fold, so `tiler-ir`'s own split
            // admissions answer `None` for this program and a proposal carrying
            // either topology would be rejected as invalid compiler output rather
            // than costed against the serial one.
            //
            // Its launched threads are its *own* iteration count rather than the
            // request's widest output, because a staged fold's domain is neither:
            // it is one point per folded row, which is smaller than the published
            // domain whenever the fold removes an axis. The staging bytes are the
            // request-wide bound the other materializing arms use.
            crate::physical::RegionSpellingKind::StagedFold => {
                let region = crate::physical::staged_fold_region(
                    request,
                    producer
                        .staged()
                        .expect("a staged spelling resolves to a staged output"),
                    subject.write(),
                )
                .0;
                let threads =
                    tiler_ir::schedule::element_count(&region.index.iteration_shape).unwrap_or(0);
                (
                    region,
                    PhysicalCostEstimate::structural(1, threads, intermediate_bytes),
                )
            }
            // The consuming stage, costed like any other elementwise pass over
            // the published domain.
            crate::physical::RegionSpellingKind::StagedPass(write) => (
                crate::physical::staged_pass_region(
                    request,
                    producer
                        .staged()
                        .expect("a staged spelling resolves to a staged output"),
                    write,
                )
                .0,
                match write {
                    crate::physical::RegionWrite::ProgramOutput => {
                        PhysicalCostEstimate::structural(1, output_elements, 0)
                    }
                    crate::physical::RegionWrite::Materialized
                    | crate::physical::RegionWrite::MaterializedAndPublished => {
                        PhysicalCostEstimate::structural(1, output_elements, intermediate_bytes)
                    }
                },
            ),
            // The consumer half of a chain, costed like any other elementwise
            // pass: one dispatch over its own domain, staging bytes only when
            // the cover made it a producer in turn.
            crate::physical::RegionSpellingKind::Epilogue(write) => (
                crate::physical::epilogue_region(
                    request,
                    output
                        .epilogue()
                        .expect("an epilogue spelling resolves to an epilogue output"),
                    write,
                )
                .0,
                match write {
                    crate::physical::RegionWrite::ProgramOutput => {
                        PhysicalCostEstimate::structural(1, output_elements, 0)
                    }
                    // The staging half, for the reason the pointwise arm states.
                    crate::physical::RegionWrite::Materialized
                    | crate::physical::RegionWrite::MaterializedAndPublished => {
                        PhysicalCostEstimate::structural(1, output_elements, intermediate_bytes)
                    }
                },
            ),
        };
        // A published-and-consumed region is two dispatches: the one just built,
        // which stages the value its consumer reads across, and a copy that
        // moves those bytes into the buffer the interface publishes. The copy is
        // a *planned* dispatch rather than something the assembler synthesizes,
        // because a full-tensor copy is a real cost the planner must see — a
        // cover that avoids the publication entirely is a legal alternative, and
        // hiding the copy would let the planner prefer this one for free.
        let serial = if subject.write().publishes_a_copy() {
            let domain = region.index.iteration_shape.clone();
            let staged_elements = tiler_ir::schedule::element_count(&domain).unwrap_or(0);
            let (copy, copy_members) =
                crate::physical::publishing_copy_region(request, domain, staged_elements);
            let passes = vec![
                SubprogramStage::new(region, members.to_vec()),
                SubprogramStage::new(copy, copy_members),
            ];
            ImplementationProposal::new(
                ProposalBody::KernelSubprogram(Box::new(KernelSubprogram::new(passes))),
                applicability,
                PhysicalCostEstimate::structural(
                    2,
                    cost.launched_threads().saturating_add(staged_elements),
                    cost.temporary_bytes(),
                ),
            )
        } else {
            ImplementationProposal::new(
                ProposalBody::ScheduledKernel(Box::new(region)),
                applicability,
                cost,
            )
        };
        // The serial alternative is offered unconditionally, and each parallel
        // strategy is additive beside it: a request that admits neither still
        // has a legal plan, and one that admits both retains all three. Whether
        // a parallel one *wins* is not decided here.
        let mut proposals = vec![serial];
        let mut offer_declines = Vec::new();
        for outcome in [split, tree].into_iter().flatten() {
            match outcome {
                Ok(proposal) => proposals.push(proposal),
                Err(declined) => offer_declines.push(declined),
            }
        }
        offer_declines
            .into_iter()
            .fold(ProviderOffer::proposing(proposals), ProviderOffer::decline)
    }
}

/// Offers the multi-pass split of one request's reduction, or states why not.
///
/// The cost is structural and never a feasibility input: two dispatches, the
/// partial pass's launched threads plus the final pass's, and the four bytes per
/// partial value the split stages. It is deliberately worse than the serial
/// alternative's on every dimension under this model — a split trades those for
/// parallelism the structural model does not measure, which is why this slice
/// only enumerates.
///
/// **Measurement, 2026-08-07 — that trade is quantified, and preference is
/// assigned now.** [The retained dispatch sweep] timed all three strategies over
/// 92 shapes on the qualified Apple9 macOS host: where the row count alone cannot
/// saturate the device a parallel plan wins by up to 50.7 times, and where it can
/// the serial fold wins by up to 1.78 times. What the structural model does not
/// measure is therefore worth a factor rather than a rounding, and
/// `activate-measured-reduction-selection-from-a-target-cost-row` landed the
/// machine quantity that contour turns on as a **measured cost row** on the
/// qualified profile. [`crate::measured_cost`] consults it; this proposal is
/// unchanged, which is the point — the strategy still arrives as a generator, a
/// typed decline, and a structural cost, and contributes no comparison.
///
/// [The retained dispatch sweep]:
///     ../../../spikes/program-planning/reduction-dispatch-crossover/README.md
fn propose_split(
    request: &VerifiedTargetRequest,
    output: &crate::request::NormalizedOutput,
    applicability: &TargetApplicability,
    write: crate::physical::RegionWrite,
) -> Result<ImplementationProposal, DeclinedStrategy> {
    let split = crate::physical::split_reduction_regions(request, output, write).map_err(
        |unavailable| {
            DeclinedStrategy::new(
                crate::physical::MULTI_PASS_SPLIT_STRATEGY,
                match unavailable {
                    crate::physical::SplitUnavailable::ReassociationForbidden => {
                        StrategyDeclineCause::NumericalPermissionRefused {
                            dimension:
                                crate::target::honourability::NumericalDimension::Reassociation
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
        },
    )?;
    let output_elements = output.output_elements();
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

/// Offers the single-workgroup tree of one request's reduction, or states why
/// not.
///
/// The cost is structural and never a feasibility input: one dispatch, the
/// launched invocations (`participants` per output position rather than one),
/// and no materialized intermediate — the partials live in workgroup memory,
/// which this model does not count as a materialization because nothing outside
/// the dispatch can observe them.
///
/// Under this model the tree launches strictly more threads than the serial
/// alternative and shares its dispatch count, so it does not win here and is not
/// meant to. What it trades those threads for — a shorter critical path per
/// output — is not something the structural model measures, which is why this
/// slice only enumerates.
///
/// **Measurement, 2026-08-07 — the shorter critical path is what decides the
/// contour, and it is worth up to 50.7 times.** [The retained dispatch sweep]
/// timed all three strategies over 92 shapes on the qualified Apple9 macOS host,
/// and a three-parameter work-span model reproduces the measured winner on 24 of
/// the 26 held-out shapes whose verdict is separated from the noise. Its one
/// decision-bearing parameter is the fold steps the device retires at once: the
/// tree wins exactly where a stage's critical path dominates its total work
/// divided by that number. That number is now *declared by the qualified target
/// profile* as a measured cost row and consulted by [`crate::measured_cost`],
/// under `activate-measured-reduction-selection-from-a-target-cost-row`. This
/// proposal is unchanged all the same: the strategy contributes a generator, a
/// typed decline, and a structural cost, and never a comparison.
///
/// [The retained dispatch sweep]:
///     ../../../spikes/program-planning/reduction-dispatch-crossover/README.md
fn propose_workgroup_tree(
    request: &VerifiedTargetRequest,
    output: &crate::request::NormalizedOutput,
    applicability: &TargetApplicability,
    write: crate::physical::RegionWrite,
) -> Result<ImplementationProposal, DeclinedStrategy> {
    let (region, _) = crate::physical::single_workgroup_tree_region(request, output, write)
        .map_err(|unavailable| {
            DeclinedStrategy::new(
                crate::physical::SINGLE_WORKGROUP_TREE_STRATEGY,
                match unavailable {
                    crate::physical::WorkgroupTreeUnavailable::ReassociationForbidden => {
                        StrategyDeclineCause::NumericalPermissionRefused {
                            dimension:
                                crate::target::honourability::NumericalDimension::Reassociation
                                    .key(),
                        }
                    }
                    crate::physical::WorkgroupTreeUnavailable::NoAdmissibleParticipantCount {
                        contributors,
                    } => StrategyDeclineCause::NoAdmissibleShape {
                        rule: unavailable.reason(),
                        extent: contributors,
                    },
                    crate::physical::WorkgroupTreeUnavailable::Unrepresentable => {
                        StrategyDeclineCause::Unrepresentable {
                            rule: unavailable.reason(),
                        }
                    }
                },
            )
        })?;
    let launched = region.schedule.work_items;
    Ok(ImplementationProposal::new(
        ProposalBody::ScheduledKernel(Box::new(region)),
        applicability.clone(),
        PhysicalCostEstimate::structural(1, launched, 0),
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
        ReservedProposalSeam, SubprogramStage, TargetApplicability, boundary_carrier,
        bounded_guarantees, bounded_requirements, enumerate_frontier,
    };
    use crate::boundary::{
        BoundaryProperty, GuaranteedProperty, LayoutRequirement, MaterializationForm,
        MemoryDomainClass, RequiredProperties, RequiredProperty, StorageScalar,
    };
    use crate::call_registry::{OpaqueCallIdentity, OpaqueCallProposal, OpaqueCallRegistry};
    use crate::physical::{build_fused_scheduled_region, pointwise_region};
    use crate::request::{
        CompilationRequest, TargetProfileKey, VerifiedTargetRequest, verify_planned_request,
    };
    use tiler_ir::schedule::{
        AccessMode, ContributorOrder, ExceptionalValueAssumption, InputOrdinal,
        NumericalPermission, ScalarProgram, ScheduledRegion, SubnormalMode, TensorRole,
    };
    use tiler_ir::semantic::EncodedComponentRole;
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
        let request = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        request.for_target(0).unwrap()
    }

    fn provider_identity(name: &str, revision: u32) -> ProviderIdentity {
        ProviderIdentity::new("tiler.test.physical", name, revision).unwrap()
    }

    fn fused_subject(request: &VerifiedTargetRequest) -> FrontierRegionSubject {
        FrontierRegionSubject::new(
            "fused",
            request.serial_sum().members.all(),
            crate::physical::RegionWrite::ProgramOutput,
        )
    }

    fn pointwise_subject(request: &VerifiedTargetRequest) -> FrontierRegionSubject {
        FrontierRegionSubject::new(
            "pointwise",
            request.serial_sum().members.pointwise().to_vec(),
            crate::physical::RegionWrite::Materialized,
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

        // The alignment on a real derived boundary is the region's own carrier
        // width, taken from the region rather than from a profile constant. It
        // is asserted against `natural_for` of the carrier `boundary_carrier`
        // reports for this exact region, so a derivation that silently reverted
        // to a fixed four would still have to agree with the region's program.
        let scheduled = admitted
            .scheduled()
            .expect("the fused proposal carries a scheduled region");
        let carrier = boundary_carrier(&scheduled.region().index.scalar_program)
            .expect("the fused region's boundary values share one carrier");
        let derived = crate::boundary::ByteAlignment::natural_for(carrier);
        assert_eq!(u64::from(derived.bytes()), carrier.byte_width());
        assert_eq!(
            needed.get(BoundaryProperty::Alignment),
            Some(&RequiredProperty::Alignment(derived)),
            "the required alignment is not the boundary value's own element width"
        );
        assert_eq!(
            offered.get(BoundaryProperty::Alignment),
            Some(&GuaranteedProperty::Alignment(derived)),
            "the guaranteed alignment is not the boundary value's own element width"
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
    /// The boundary carrier is read off the region's scalar program.
    ///
    /// The three programs the bounded profile can build are `f32` on every
    /// boundary value, so the derived alignment is four — but it is four because
    /// the carrier is four bytes wide, not because a profile constant said so,
    /// and this drives each variant rather than assuming they agree.
    ///
    /// The BF16 program is the case that makes the derivation observable rather
    /// than merely stated: it is the only scalar program in the vocabulary whose
    /// carrier is not `F32`, so a site that had reverted to the constant would
    /// answer four bytes for a two-byte value and over-align every BF16 binding.
    #[test]
    fn the_boundary_carrier_is_derived_from_the_scalar_program() {
        for program in [serial_sum_program(), fused_multiply_add_program()] {
            assert_eq!(
                boundary_carrier(&program),
                Some(StorageScalar::F32),
                "{program:?} stopped reporting the f32 carrier its buffers have"
            );
        }

        assert_eq!(
            boundary_carrier(&bf16_pointwise_program()),
            Some(StorageScalar::Bf16),
            "a bf16 region's boundary values are carried in two bytes"
        );
        assert_eq!(
            crate::boundary::ByteAlignment::natural_for(StorageScalar::Bf16).bytes(),
            2,
            "the bf16 carrier's natural alignment is derived from its own width"
        );
        // The requirement the profile states for that carrier is the derived
        // alignment and not the `f32` neighbour's, which is the whole point of
        // deriving it.
        assert_eq!(
            bounded_requirements(StorageScalar::Bf16).get(BoundaryProperty::Alignment),
            Some(&RequiredProperty::Alignment(
                crate::boundary::ByteAlignment::natural_for(StorageScalar::Bf16)
            )),
        );
        assert_ne!(
            crate::boundary::ByteAlignment::natural_for(StorageScalar::Bf16),
            crate::boundary::ByteAlignment::natural_for(StorageScalar::F32),
        );
    }

    /// The `(x * 3.0) + (-0.0)` BF16 pointwise program.
    fn bf16_pointwise_program() -> ScalarProgram {
        use tiler_ir::schedule::{InputOrdinal, PointwiseBf16ExpressionBuilder};
        let mut expression = PointwiseBf16ExpressionBuilder::new();
        let input = expression.input(InputOrdinal::FIRST).unwrap();
        let scale = expression.constant(0x4040).unwrap();
        let product = expression.multiply(input, scale).unwrap();
        let bias = expression.constant(0x8000).unwrap();
        let root = expression.add(product, bias).unwrap();
        ScalarProgram::PointwiseBf16(expression.build(root).unwrap())
    }

    /// The carrier derivation can say no, and does for the one program that
    /// binds boundary values of different carriers.
    ///
    /// A derivation that answered every program would be indistinguishable from
    /// the constant it replaced. `StrictAffineU4Dequantize` reads `[U8, F32, U8]`
    /// — `tiler-ir`'s `verify_signature` fixes that signature — so no single
    /// carrier describes its boundary, and answering `F32` would over-align its
    /// two `U8` buffers: a check that passes for the wrong reason, which is the
    /// defect this ticket exists to remove rather than relocate.
    ///
    /// `physical::verify_region_subject_binding` refuses this program upstream,
    /// so the refusal is unreachable through a compiled region today. That makes
    /// driving the derivation directly the only way to watch it fail, and leaving
    /// it undriven would mean shipping a branch nothing had ever executed.
    #[test]
    fn a_program_whose_boundary_carriers_disagree_is_refused_rather_than_widened() {
        let mixed = ScalarProgram::StrictAffineU4Dequantize {
            codes_role: EncodedComponentRole::new(0),
            scale_role: EncodedComponentRole::new(1),
            zero_point_role: EncodedComponentRole::new(2),
        };
        assert_eq!(
            boundary_carrier(&mixed),
            None,
            "a program binding U8 and F32 boundary values was given one carrier"
        );
    }

    /// The profile's own property builders state the carrier's alignment, not four.
    ///
    /// This is the case that can catch a reverted production site. Every region
    /// the bounded profile compiles is `f32`, so a
    /// [`bounded_requirements`]/[`bounded_guarantees`] that hard-coded four again
    /// would still produce the right *value* everywhere it is actually called,
    /// and no test over compiled regions could tell the difference. Driving the
    /// builders with a one-byte carrier is what makes the two distinguishable:
    /// a constant answers four here and the derivation answers one.
    #[test]
    fn the_property_builders_state_the_carriers_alignment_rather_than_a_constant() {
        let one = crate::boundary::ByteAlignment::natural_for(StorageScalar::U8);
        assert_eq!(one.bytes(), 1, "the one-byte carrier stopped deriving one");

        assert_eq!(
            bounded_requirements(StorageScalar::U8).get(BoundaryProperty::Alignment),
            Some(&RequiredProperty::Alignment(one)),
            "a one-byte carrier was required to meet some other element's alignment"
        );
        assert_eq!(
            bounded_guarantees(StorageScalar::U8).get(BoundaryProperty::Alignment),
            Some(&GuaranteedProperty::Alignment(one)),
            "a one-byte carrier was made to guarantee some other element's alignment"
        );
    }

    /// A `StrictSerialSum` program, minimal but well formed for carrier queries.
    fn serial_sum_program() -> ScalarProgram {
        ScalarProgram::StrictSerialSum {
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7FC0_0000,
            empty_identity_bits: 0,
        }
    }

    /// A `FusedMultiplyAddSerialSum` program, likewise.
    fn fused_multiply_add_program() -> ScalarProgram {
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits: 0x3F80_0000,
            bias_bits: 0,
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7FC0_0000,
            empty_identity_bits: 0,
            contraction: false,
        }
    }

    #[test]
    fn the_bounded_profile_admits_no_undischarged_boundary() {
        let needed = bounded_requirements(StorageScalar::F32);
        let offered = bounded_guarantees(StorageScalar::F32);

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
        let offered = bounded_guarantees(StorageScalar::F32);
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
                let (region, _) = pointwise_region(
                    context.request(),
                    context.request().sole_output(),
                    crate::physical::RegionWrite::Materialized,
                );
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
            alignment: crate::boundary::ByteAlignment::natural_for(StorageScalar::F32),
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
                synchronization: None,
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
            resolve_work_items(
                WorkScaling::PerElementOf("x"),
                &bindings,
                &coverless_subject(),
                &request
            ),
            Ok(normalized.input_elements)
        );
        assert_eq!(
            resolve_work_items(
                WorkScaling::PerElementOf("y"),
                &bindings,
                &coverless_subject(),
                &request
            ),
            Ok(normalized.output_elements)
        );
        assert_eq!(
            resolve_work_items(
                WorkScaling::Fixed(7),
                &bindings,
                &coverless_subject(),
                &request
            ),
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
            resolve_work_items(
                WorkScaling::PerElementOf("absent"),
                &bindings,
                &coverless_subject(),
                &request
            ),
            Err(WorkResolutionError::UnknownParameter("absent")),
            "a scaling naming an unbound parameter produced a count"
        );
        // An intermediate declines when the subject was stated without a cover.
        // A previous revision resolved it to the input's count on a falsified
        // premise — the all-singleton cover materializes every internal value,
        // including rank-0 constants — so a count from any tensor in scope would
        // be confidently wrong for exactly the covers that exist.
        assert_eq!(
            resolve_work_items(
                WorkScaling::PerElementOf("z"),
                &bindings,
                &coverless_subject(),
                &request
            ),
            Err(WorkResolutionError::IntermediateShapeUnavailable { parameter: "z" }),
            "a subject stated without a cover produced an intermediate count"
        );

        // Stated from a cover, the same binding resolves to the edge's own
        // element count — not the input's, and not the output's.
        let normalized = request.serial_sum();
        let edge_elements = normalized.input_elements + normalized.output_elements + 1;
        assert_eq!(
            resolve_work_items(
                WorkScaling::PerElementOf("z"),
                &bindings,
                &FrontierRegionSubject::reading_intermediates(
                    "region",
                    Vec::new(),
                    [edge_elements],
                    crate::physical::RegionWrite::ProgramOutput,
                ),
                &request
            ),
            Ok(edge_elements),
            "a cover-stated intermediate must resolve to the edge's own size"
        );

        // Two intermediates of different sizes are an ambiguity rather than a
        // missing fact, and a scaling that names the role rather than an edge
        // declines for it.
        assert_eq!(
            resolve_work_items(
                WorkScaling::PerElementOf("z"),
                &bindings,
                &FrontierRegionSubject::reading_intermediates(
                    "region",
                    Vec::new(),
                    [1, 2],
                    crate::physical::RegionWrite::ProgramOutput,
                ),
                &request
            ),
            Err(WorkResolutionError::IntermediateShapeUnavailable { parameter: "z" }),
            "two differently sized intermediates must not resolve to one of them"
        );
    }

    /// Two ordered named outputs over disjoint declared inputs of different
    /// extents.
    ///
    /// `doubled = a + a` over `[2, 3]` and `halved = b + b` over `[4]`. Neither
    /// walk reaches the other's declared input, and the two element counts
    /// differ, so a resolution that answered from the wrong output — or from
    /// either output indiscriminately — cannot pass here by coincidence.
    fn disjoint_two_output_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let second = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let doubled = F32Add::apply(&mut builder, first, first).unwrap();
        let halved = F32Add::apply(&mut builder, second, second).unwrap();
        builder
            .output(OutputKey::new("doubled").unwrap(), doubled)
            .unwrap();
        builder
            .output(OutputKey::new("halved").unwrap(), halved)
            .unwrap();
        let program = builder.build().unwrap();
        verify_planned_request(CompilationRequest::governed(&program))
            .unwrap()
            .for_target(0)
            .unwrap()
    }

    /// One output whose epilogue reads a declared input its producer never
    /// folds.
    ///
    /// `scaled = sum(a, axis 1) * b` over `a: [2, 3]` and `b: [2]`. The fold
    /// iterates the contributor domain of six elements and reads only `a`; the
    /// epilogue iterates the published domain of two and reads only `b` beside
    /// the staged value. Ordinal `1` therefore has exactly one reading region,
    /// and it is inside a single recognized output — so the disagreement a
    /// volunteering producer half causes is not one the program-scoped fold
    /// could filter out.
    fn epilogue_reading_an_unfolded_input_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let folded = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let reduced = StrictSerialF32Sum::apply(&mut builder, folded, [Axis::new(1)]).unwrap();
        let scaled = F32Multiply::apply(&mut builder, reduced, scale).unwrap();
        builder
            .output(OutputKey::new("scaled").unwrap(), scaled)
            .unwrap();
        let program = builder.build().unwrap();
        verify_planned_request(CompilationRequest::governed(&program))
            .unwrap()
            .for_target(0)
            .unwrap()
    }

    /// Two ordered named outputs reading one declared input at two domains.
    ///
    /// `doubled = w + w` iterates `[2]` and `scaled = a * broadcast(w)`
    /// iterates `[2, 2]`, so declared input `0` is read by two regions that
    /// iterate different domains while declared input `1` is read only by the
    /// widening one. A binding names the tensor *role*, so nothing on it says
    /// which of the two regions a call over ordinal `0` means.
    ///
    /// The widening read is what makes the two domains differ at all: a dense
    /// read binds its region's domain to the tensor's own shape, so two outputs
    /// reading one input densely always agree.
    ///
    /// **Measurement boundary.** The disagreement this fixture presents is
    /// between two *domains*, `2` and `4`, and declared input `0` holds two
    /// elements in both — the pointwise arm of
    /// `NormalizedOutput::input_elements_at` answers the reading region's
    /// domain rather than the tensor's own count for a widening read. That is a
    /// separate defect, owned by
    /// `answer-input-element-counts-as-the-declared-tensors-own-count`, and
    /// settling it will make these two outputs agree — so that ticket has to
    /// re-found this refusal on a fixture whose disagreement survives, not just
    /// re-run it.
    fn shared_input_two_domain_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let weight = builder
            .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let activations = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let doubled = F32Add::apply(&mut builder, weight, weight).unwrap();
        let mapping = tiler_ir::semantic::BroadcastAxisMapping::new(
            [
                tiler_ir::shape::Extent::new(2),
                tiler_ir::shape::Extent::new(2),
            ],
            [
                tiler_ir::semantic::BroadcastAxisSource::Replicate,
                tiler_ir::semantic::BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .expect("one replicated axis over a rank-one operand is an admitted relation");
        let widened = tiler_ir::semantic::F32Broadcast::apply(&mut builder, &mapping, weight)
            .expect("the standard registry admits the broadcast family");
        let scaled = F32Multiply::apply(&mut builder, activations, widened).unwrap();
        builder
            .output(OutputKey::new("doubled").unwrap(), doubled)
            .unwrap();
        builder
            .output(OutputKey::new("scaled").unwrap(), scaled)
            .unwrap();
        let program = builder.build().unwrap();
        verify_planned_request(CompilationRequest::governed(&program))
            .unwrap()
            .for_target(0)
            .unwrap()
    }

    /// A bound ordinal resolves from the output that reads it, not from a
    /// sibling that never loads it.
    ///
    /// **The false negative this closes.** `NormalizedOutput::input_elements_at`
    /// answered for every ordinal below the *program's* declared arity in its
    /// serial-sum and pointwise arms, because `input_keys` is the whole
    /// program's declaration list. An output iterating one domain therefore
    /// volunteered its own count for an ordinal only its sibling loads, the
    /// agreement fold saw two unequal counts, and a scaling the reading output
    /// sizes exactly was refused as `UnknownParameter`.
    ///
    /// **Three perturbations: two authorities carry the admission and one
    /// carries the refusal.** Watched failing once each before the restoration:
    ///
    /// - Dropping the `reads_declared_input` gate from the serial-sum and
    ///   pointwise arms of `NormalizedOutput::input_elements_at`, back to
    ///   `(ordinal < normalized.input_keys.len()).then_some(…)`, made the
    ///   epilogue fixture's ordinal `1` report `Err(UnknownParameter)` instead
    ///   of `Ok(2)`: the fold half volunteered its six-element contributor
    ///   domain for an input it never reads, and the chain arm read the two
    ///   halves as a disagreement. The disjoint fixture is deliberately *not*
    ///   what observes this, and that is the point of having both — the
    ///   program-scoped filter below already excludes a non-reading output
    ///   there, so nothing asks the arm.
    /// - Dropping the `reads_declared_input` filter from
    ///   `NormalizedProgram::agreed_input_elements_at` made the disjoint
    ///   fixture's two assertions report `Err(UnknownParameter)` instead of
    ///   `Ok(6)` and `Ok(4)`: a silent output's `None` is a value the agreement
    ///   fold compares rather than an abstention.
    /// - Replacing that fold's agreement with the first reading claimant's
    ///   answer made the shared fixture's ordinal `0` report `Ok(2)` instead of
    ///   `Err(UnknownParameter)` — which is the widening overshooting into the
    ///   confidently-wrong verdict, and the reason the refusing neighbour is
    ///   here rather than only the two admissions.
    ///
    /// The last fixture is the neighbour that must keep refusing, and it pairs
    /// the two findings on one program: ordinal `0` is read by two regions at
    /// two domains and has no single count, while ordinal `1` is read by
    /// exactly one of them and resolves. A fix that answered from the first
    /// claimant rather than requiring agreement would size a call against a
    /// domain the other region does not iterate, and this says no to it.
    #[test]
    fn a_bound_ordinal_resolves_from_the_output_that_reads_it() {
        use super::{WorkResolutionError, WorkScaling, resolve_work_items};

        let bindings = [
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            (
                "y",
                TensorRole::Input {
                    ordinal: InputOrdinal::new(1),
                },
            ),
            (
                "z",
                TensorRole::Input {
                    ordinal: InputOrdinal::new(2),
                },
            ),
        ];
        let resolve = |name, request: &VerifiedTargetRequest| {
            resolve_work_items(
                WorkScaling::PerElementOf(name),
                &bindings,
                &coverless_subject(),
                request,
            )
        };

        let disjoint = disjoint_two_output_request();
        assert_eq!(
            disjoint.normalized().outputs().len(),
            2,
            "the disjoint fixture must present two recognized outputs to disagree",
        );
        assert_eq!(
            resolve("x", &disjoint),
            Ok(6),
            "the only output reading ordinal 0 sizes it",
        );
        assert_eq!(
            resolve("y", &disjoint),
            Ok(4),
            "the only output reading ordinal 1 sizes it",
        );
        // An ordinal no declared input occupies still refuses, so the widening
        // is to *read* ordinals rather than to every ordinal.
        assert_eq!(
            resolve("z", &disjoint),
            Err(WorkResolutionError::UnknownParameter("z")),
            "an ordinal no declared input occupies produced a count",
        );

        // One recognized output, two regions, and one ordinal each reads. The
        // producer's own contributor domain is six and the epilogue's published
        // domain is two, so a producer half answering for an ordinal it never
        // folds contradicts the half that does read it.
        let chained = epilogue_reading_an_unfolded_input_request();
        assert_eq!(
            chained.normalized().outputs().len(),
            1,
            "the epilogue fixture must present one recognized output, so nothing is filtered",
        );
        assert_eq!(
            resolve("x", &chained),
            Ok(6),
            "the fold reads ordinal 0 at its contributor domain",
        );
        assert_eq!(
            resolve("y", &chained),
            Ok(2),
            "the epilogue reads ordinal 1 at its published domain",
        );

        let shared = shared_input_two_domain_request();
        assert_eq!(
            shared.normalized().outputs().len(),
            2,
            "the shared fixture must present two recognized outputs to disagree",
        );
        assert_eq!(
            resolve("x", &shared),
            Err(WorkResolutionError::UnknownParameter("x")),
            "one declared input read at two domains has no single count",
        );
        assert_eq!(
            resolve("y", &shared),
            Ok(4),
            "the input only the widening output reads still resolves",
        );
    }

    /// A region subject stated outside any cover, reading no intermediate.
    fn coverless_subject() -> FrontierRegionSubject {
        FrontierRegionSubject::new(
            "region",
            Vec::new(),
            crate::physical::RegionWrite::ProgramOutput,
        )
    }

    fn strict_call_resources() -> tiler_ir::schedule::ResourceRequirements {
        let contract = crate::request::StrictF32NumericalContract::governed().realization();
        tiler_ir::schedule::ResourceRequirements {
            buffer_bindings: 2,
            threads_per_workgroup: 1,
            local_memory_bytes: 0,
            requires_device_memory: true,
            synchronization: None,
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
            alignment: ByteAlignment::natural_for(StorageScalar::F32),
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
                alignment: ByteAlignment::natural_for(StorageScalar::F32),
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
                synchronization: None,
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
                    alignment: crate::boundary::ByteAlignment::natural_for(StorageScalar::F32),
                },
                ParameterSpec {
                    name: "y",
                    role: ParameterRole::Out,
                    layout: ParameterLayout::Guaranteed(
                        crate::boundary::LayoutGuarantee::DenseRowMajor,
                    ),
                    encoding: crate::boundary::StorageEncoding::Unpacked,
                    alignment: crate::boundary::ByteAlignment::natural_for(StorageScalar::F32),
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
                synchronization: None,
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

        let unsatisfied =
            unsatisfied_properties(&bounded_requirements(StorageScalar::F32), &guaranteed);
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
            alignment: crate::boundary::ByteAlignment::natural_for(StorageScalar::F32),
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
                synchronization: None,
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
            resolve_work_items(
                WorkScaling::PerElementOf("x"),
                &bindings,
                &coverless_subject(),
                &request
            ),
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

    /// A whole-program elementwise request, recognized as a pointwise program.
    fn whole_program_pointwise_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let root = F32Add::apply(&mut builder, product, bias).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        request.for_target(0).unwrap()
    }

    /// The `ab,bc->ac` matrix product, recognized as a contraction program.
    fn contraction_request() -> VerifiedTargetRequest {
        use tiler_ir::semantic::{
            ContractionIndex, ContractionIndexStructure, F32TensorContraction,
        };

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let left = builder
            .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let right = builder
            .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let structure = ContractionIndexStructure::new(
            [
                vec![ContractionIndex::new(0), ContractionIndex::new(1)],
                vec![ContractionIndex::new(1), ContractionIndex::new(2)],
            ],
            [ContractionIndex::new(0), ContractionIndex::new(2)],
        )
        .unwrap();
        let product = F32TensorContraction::apply(&mut builder, &structure, left, right).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), product)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        request.for_target(0).unwrap()
    }

    /// Every region subject the governed provider spells, reported as one line
    /// per admitted implementation: role, kind, the three structural cost
    /// dimensions, and the canonical proposal identity in hex.
    fn governed_proposal_report(
        request: &VerifiedTargetRequest,
        subject: &FrontierRegionSubject,
    ) -> Vec<String> {
        use std::fmt::Write as _;

        let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
        let frontier =
            enumerate_frontier(request, subject, &providers, &OpaqueCallRegistry::new()).unwrap();
        assert!(
            !frontier.admitted().is_empty(),
            "no admitted implementation for {}: {:?}",
            subject.role(),
            frontier.rejections()
        );
        frontier
            .admitted()
            .iter()
            .map(|admitted| {
                let cost = admitted.cost();
                let mut line = format!(
                    "{}|{}|{}|{}|{}|",
                    subject.role(),
                    admitted.provenance().kind(),
                    cost.dispatch_count(),
                    cost.launched_threads(),
                    cost.temporary_bytes(),
                );
                for byte in admitted.identity().as_bytes() {
                    write!(line, "{byte:02x}").expect("writing to a String cannot fail");
                }
                line
            })
            .collect()
    }

    /// The recognized region subjects keep their exact proposals, byte for byte.
    ///
    /// **This is the regression check the region-general provider is measured
    /// against, and it is deliberately a golden rather than a property.** A
    /// proposal identity folds the shared-IR region identity, the provider
    /// provenance, the proposal kind, the applicability predicate, and the
    /// derived boundary contract, and it is carried through plan selection into
    /// kernel-program and artifact identity. So a generalization that changed
    /// *anything* about how a recognized region is built — its accesses, its
    /// written tensor role, its ownership proof, its scalar program, or the
    /// structural cost attributed to it — moves bytes that are observable to a
    /// caller and invalidate every cache entry derived from them.
    ///
    /// The five cases are every branch the member-set-matching provider had:
    /// the serial sum's prologue, its reduction, and its fused whole-program
    /// region, plus the two whole-program recognizers. Recorded at `a48b38ea`,
    /// before the provider read the cover region subject.
    ///
    /// Regenerate only when a proposal is *deliberately* changed, and step the
    /// proposal identity tag in the same commit when the encoding moves.
    #[test]
    fn the_recognized_region_subjects_keep_their_exact_proposals() {
        let serial = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let pointwise = whole_program_pointwise_request();
        let contraction = contraction_request();
        let whole_pointwise_subject = FrontierRegionSubject::new(
            "whole-program",
            pointwise.pointwise().unwrap().members.clone(),
            crate::physical::RegionWrite::ProgramOutput,
        );
        let whole_contraction_subject = FrontierRegionSubject::new(
            "whole-program",
            contraction.contraction().unwrap().members.clone(),
            crate::physical::RegionWrite::ProgramOutput,
        );

        let mut report = Vec::new();
        report.extend(governed_proposal_report(
            &serial,
            &pointwise_subject(&serial),
        ));
        report.extend(governed_proposal_report(
            &serial,
            &reduction_subject(&serial),
        ));
        report.extend(governed_proposal_report(&serial, &fused_subject(&serial)));
        report.extend(governed_proposal_report(
            &pointwise,
            &whole_pointwise_subject,
        ));
        report.extend(governed_proposal_report(
            &contraction,
            &whole_contraction_subject,
        ));

        assert_eq!(
            report, GOVERNED_PROPOSALS,
            "a recognized region subject's proposal moved; every plan, kernel-program, and \
             artifact identity derived from it moves with it",
        );
    }

    /// The recorded proposals of [`the_recognized_region_subjects_keep_their_exact_proposals`].
    const GOVERNED_PROPOSALS: [&str; 5] = [
        "pointwise|scheduled-kernel|1|4|16|74696c65722e636f6d70696c65722e706879736963616c2d696d706c656d656e746174696f6e2d70726f706f73616c2e76320000000000000001d774696c65722e7363686564756c652e7635000000000000000002000000000000000200000000000000020000000000000002010000000000010100000000000200020100000001010000000000000000000000020000000001000000000011000000000000000400000001020011000000000000000400000000020000000000000004240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000006274696c65722e636f6e74726163742e6633322e76322e303337666330303030303031303130313032303130313033303230313034303230313035303230313036303230313037303230313038303330313039303430313061303430313062303530317fc0000001010101010101010100000000000000040000000101000000003100000000000000040000000101000000000000000574696c6572000000000000001d70726f746f747970652d73657269616c2d73756d2d706879736963616c00000001010000000000000001000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e763100000000000000010100000000010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790600000000000000010107010801000000000000000102010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790601070108010100000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000000100000000000000ce74696c65722e70726570617265642d656e7472792d7461726765742d726571756972656d656e742e763100000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f7065727469657300000001000000000000000101",
        "reduction|scheduled-kernel|1|2|0|74696c65722e636f6d70696c65722e706879736963616c2d696d706c656d656e746174696f6e2d70726f706f73616c2e763200000000000000019074696c65722e7363686564756c652e76350000000000000000010000000000000002000000000000000202000102000000000000000200000000000000020000000000000002000000000000000100000000000000020000000000000001000000010100000002000300020100000003010000000100000000000000020000000202001200000000000000020000000000000002000000000000000200000000000000010000000000000002000000000000000100000001010000000303001100000000000000020000000103000000000000000222000000000000000100000001017fc0000000000000000000000000006274696c65722e636f6e74726163742e6633322e76322e303337666330303030303031303130313032303130313033303230313034303230313035303230313036303230313037303230313038303330313039303430313061303430313062303530317fc0000001010101010101010100000000000000020000000101000000013200000000000000010000000101000000000000000000020000000101000000000000000574696c6572000000000000001d70726f746f747970652d73657269616c2d73756d2d706879736963616c00000001010000000000000001000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e7631000000000000000102010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790600000000000000010107010801000000000000000103010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790601070108010100000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000000100000000000000ce74696c65722e70726570617265642d656e7472792d7461726765742d726571756972656d656e742e763100000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f7065727469657300000001000000000000000101",
        "fused|scheduled-kernel|1|2|0|74696c65722e636f6d70696c65722e706879736963616c2d696d706c656d656e746174696f6e2d70726f706f73616c2e76320000000000000001a174696c65722e7363686564756c652e763500000000000000000100000000000000020000000000000002010000000000010200000000000000020000000000000002000000000000000200000000000000010000000000000002000000000000000100000001010000000000030002010000000101000000000000000000000002000000000100000000001200000000000000020000000000000002000000000000000200000000000000010000000000000002000000000000000100000001010000000103001100000000000000020000000003000000000000000223400000003f800000000000000000000100000001017fc000000000000000000000000000006274696c65722e636f6e74726163742e6633322e76322e303337666330303030303031303130313032303130313033303230313034303230313035303230313036303230313037303230313038303330313039303430313061303430313062303530317fc0000001010101010101010100000000000000020000000101000000003200000000000000010000000101000000000000000000020000000101000000000000000574696c6572000000000000001d70726f746f747970652d73657269616c2d73756d2d706879736963616c00000001010000000000000001000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e763100000000000000010100000000010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790600000000000000010107010801000000000000000103010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790601070108010100000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000000100000000000000ce74696c65722e70726570617265642d656e7472792d7461726765742d726571756972656d656e742e763100000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f7065727469657300000001000000000000000101",
        "whole-program|scheduled-kernel|1|4|0|74696c65722e636f6d70696c65722e706879736963616c2d696d706c656d656e746174696f6e2d70726f706f73616c2e76320000000000000001d774696c65722e7363686564756c652e7635000000000000000002000000000000000200000000000000020000000000000002010000000000010100000000000300020100000001010000000000000000000000020000000001000000000011000000000000000400000001030011000000000000000400000000030000000000000004240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000006274696c65722e636f6e74726163742e6633322e76322e303337666330303030303031303130313032303130313033303230313034303230313035303230313036303230313037303230313038303330313039303430313061303430313062303530317fc0000001010101010101010100000000000000040000000101000000003100000000000000040000000101000000000000000574696c6572000000000000001d70726f746f747970652d73657269616c2d73756d2d706879736963616c00000001010000000000000001000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e763100000000000000010100000000010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790600000000000000010107010801000000000000000103010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790601070108010100000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000000100000000000000ce74696c65722e70726570617265642d656e7472792d7461726765742d726571756972656d656e742e763100000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f7065727469657300000001000000000000000101",
        "whole-program|scheduled-kernel|1|4|0|74696c65722e636f6d70696c65722e706879736963616c2d696d706c656d656e746174696f6e2d70726f706f73616c2e763200000000000000020874696c65722e7363686564756c652e76350000000000000000020000000000000002000000000000000200000000000000030100000000000105000000000000000200000000000000020000000000000002000000000000000200000000000000020000000000000002000000000000000100000000000000020000000000000002010000000002000000000100000000000100000001000105000000000000000200000000000000020000000000000002000000000000000200000000000000020000000000000002000000000000000100000000000000020000000000000002020000000001000000010100000001000300020100000002010000000000000000000000030000000001000000000011000000000000000400000001010000000100110000000000000004000000020300110000000000000004000000000300000000000000042700000000000000010000000000000002017fc00000000000000000006274696c65722e636f6e74726163742e6633322e76322e303337666330303030303031303130313032303130313033303230313034303230313035303230313036303230313037303230313038303330313039303430313061303430313062303530317fc000000101010101010101010000000000000004000000010100000000340000000000000001000000000000000201000000000000000000040000000101000000000000000574696c6572000000000000001d70726f746f747970652d73657269616c2d73756d2d706879736963616c00000001010000000000000001000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e763100000000000000020100000000010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d61727906000000000000000101070108010100000001010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790600000000000000010107010801000000000000000103010000000000000008010102010300000004040105000000000000001674696c65722e616666696e6974792e7072696d6172790601070108010100000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000000100000000000000ce74696c65722e70726570617265642d656e7472792d7461726765742d726571756972656d656e742e763100000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f7065727469657300000001000000000000000101",
    ];

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
        let request = verify_planned_request(CompilationRequest::governed_under(
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
            crate::physical::RegionWrite::ProgramOutput,
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
        crate::physical::split_reduction_regions(
            request,
            request.sole_output(),
            crate::physical::RegionWrite::ProgramOutput,
        )
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
            let (region, members) = pointwise_region(
                &request,
                request.sole_output(),
                crate::physical::RegionWrite::Materialized,
            );
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
