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
//! The bounded P0 profile admits only checked [`ProposalBody::ScheduledKernel`]
//! proposals and explicitly rejects the reserved [`ProposalBody::KernelSubprogram`],
//! [`ProposalBody::OpaqueCall`], and [`ProposalBody::View`] variants while keeping
//! the additive sum-type/provider seam. Opaque physical-call contracts are owned
//! by the reviewed `implement-opaque-physical-call-providers` ticket.
//!
//! Every item here is a reviewed *draft* boundary, not a stable compiler API,
//! until Tom accepts the exact interface.

#![allow(
    dead_code,
    reason = "reviewed draft authority; the bounded frontier is exercised by its own tests and is not yet wired into the private compile() facade, which the complete physical-plan-selection slice will do"
)]

use std::error::Error;
use std::fmt;

use tiler_ir::schedule::{
    CanonicalScheduledRegionIdentity, ResourceRequirements, ScheduledRegion, TensorRole,
};
use tiler_ir::semantic::ProviderIdentity;

use crate::feasibility::ResolvedPredicate;
use crate::physical::{PhysicalError, VerifiedScheduledRegion, verify_schedule_with_feasibility};
use crate::region::SemanticMemberId;
use crate::request::VerifiedTargetRequest;

/// The single structural cost model the bounded P0 frontier attributes estimates
/// to. It matches the pipeline's structural cost model so a later selector can
/// compare frontier estimates without a model reconciliation.
const COST_MODEL_KEY: &str = "tiler.cost.structural.v1";
/// Canonical domain-separation tag for a physical implementation proposal.
const PROPOSAL_IDENTITY_TAG: &[u8] = b"tiler.compiler.physical-implementation-proposal.v1\0";

/// Which additive proposal-body variant a physical provider offered.
///
/// The declaration order and the encoded tag agree, so the derived total order
/// used for deterministic identity and reporting matches the serialized tag; a
/// reordered variant cannot silently keep its encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum PhysicalProposalKind {
    /// A checked scheduled kernel over one bounded index region. P0-admitted.
    ScheduledKernel,
    /// A nested kernel subprogram. Reserved; rejected in P0.
    KernelSubprogram,
    /// An opaque physical call. Reserved; owned by the opaque-call ticket.
    OpaqueCall,
    /// A metadata-only view. Reserved; rejected in P0.
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

/// A minimal typed placeholder for a reserved (non-P0) proposal body.
///
/// It preserves the additive seam without asserting any of the contract the
/// reserved variant will eventually carry: the descriptor is echoed in the
/// rejection diagnostic but is otherwise uninterpreted. The
/// `implement-opaque-physical-call-providers` ticket replaces the
/// [`ProposalBody::OpaqueCall`] payload with its typed ABI, effect, aliasing,
/// placement, and evidence contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReservedProposalSeam {
    descriptor: &'static str,
}

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
/// implements only [`Self::ScheduledKernel`] and reserves the rest so an
/// unsupported body rejects explicitly instead of being silently approximated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProposalBody {
    /// A checked scheduled region carrying a minimal serial schedule. The
    /// frontier resubmits it through ordinary intrinsic + feasibility verification.
    ///
    /// The region is boxed so the scheduled-kernel payload does not inflate every
    /// reserved seam variant to its size.
    ScheduledKernel(Box<ScheduledRegion>),
    /// A nested kernel subprogram. Reserved; the P0 frontier rejects it.
    KernelSubprogram(ReservedProposalSeam),
    /// An opaque physical call. Reserved; the P0 frontier rejects it.
    OpaqueCall(ReservedProposalSeam),
    /// A metadata-only view. Reserved; the P0 frontier rejects it.
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
    target_profile_keys: Vec<&'static str>,
}

impl TargetApplicability {
    /// Builds an applicability predicate over a set of governed target keys.
    ///
    /// The keys are normalized to a canonical, deduplicated ascending order so
    /// two predicates over the same key set share one identity encoding.
    pub(crate) fn for_targets(keys: impl IntoIterator<Item = &'static str>) -> Self {
        let mut target_profile_keys: Vec<&'static str> = keys.into_iter().collect();
        target_profile_keys.sort_unstable();
        target_profile_keys.dedup();
        Self {
            target_profile_keys,
        }
    }

    /// Returns whether the proposal applies to `target_profile_key`.
    fn applies_to(&self, target_profile_key: &'static str) -> bool {
        self.target_profile_keys.contains(&target_profile_key)
    }

    /// Returns the governed target-profile keys in canonical order.
    pub(crate) fn target_profile_keys(&self) -> &[&'static str] {
        &self.target_profile_keys
    }

    fn encode(&self, output: &mut Vec<u8>) {
        encode_len(output, self.target_profile_keys.len());
        for key in &self.target_profile_keys {
            encode_bytes(output, key.as_bytes());
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

/// How a boundary tensor an implementation reads must be available beforehand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryAvailability {
    /// The tensor must be materialized in the device address space before the
    /// implementation runs.
    MaterializedInDeviceMemory,
}

impl BoundaryAvailability {
    const fn tag(self) -> u8 {
        match self {
            Self::MaterializedInDeviceMemory => 1,
        }
    }
}

/// How a boundary tensor an implementation writes is produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryProduction {
    /// The implementation writes every owned output position exactly once, so the
    /// tensor is produced totally and race-free (backed by the region's ownership
    /// proof).
    TotalRaceFreeWrite,
}

impl BoundaryProduction {
    const fn tag(self) -> u8 {
        match self {
            Self::TotalRaceFreeWrite => 1,
        }
    }
}

/// One boundary tensor an implementation requires to be available before it runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryRequirement {
    tensor: TensorRole,
    availability: BoundaryAvailability,
}

impl BoundaryRequirement {
    /// Returns the boundary tensor role the requirement is over.
    pub(crate) const fn tensor(self) -> TensorRole {
        self.tensor
    }

    /// Returns how the required tensor must be available.
    pub(crate) const fn availability(self) -> BoundaryAvailability {
        self.availability
    }
}

/// One boundary tensor an implementation guarantees to produce.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryGuarantee {
    tensor: TensorRole,
    production: BoundaryProduction,
}

impl BoundaryGuarantee {
    /// Returns the boundary tensor role the guarantee is over.
    pub(crate) const fn tensor(self) -> TensorRole {
        self.tensor
    }

    /// Returns how the guaranteed tensor is produced.
    pub(crate) const fn production(self) -> BoundaryProduction {
        self.production
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

    fn encode(&self, output: &mut Vec<u8>) {
        encode_len(output, self.requirements.len());
        for requirement in &self.requirements {
            output.push(tensor_role_tag(requirement.tensor));
            output.push(requirement.availability.tag());
        }
        encode_len(output, self.guarantees.len());
        for guarantee in &self.guarantees {
            output.push(tensor_role_tag(guarantee.tensor));
            output.push(guarantee.production.tag());
        }
    }
}

/// Derives the boundary contract of a verified scheduled region.
///
/// Each read access contributes a requirement on its boundary tensor; the single
/// owning write contributes a guarantee on its boundary tensor. The intrinsic
/// verifier already proved the write is a total, race-free ownership, so the
/// guarantee is sound.
fn derive_boundary_contract(region: &ScheduledRegion) -> BoundaryContract {
    let mut requirements = Vec::new();
    let mut guarantees = Vec::new();
    for access in &region.index.accesses {
        if access.ownership.is_some() {
            guarantees.push(BoundaryGuarantee {
                tensor: access.tensor,
                production: BoundaryProduction::TotalRaceFreeWrite,
            });
        } else {
            requirements.push(BoundaryRequirement {
                tensor: access.tensor,
                availability: BoundaryAvailability::MaterializedInDeviceMemory,
            });
        }
    }
    BoundaryContract {
        requirements,
        guarantees,
    }
}

/// The provenance of one physical implementation provider.
///
/// It reuses the governed [`ProviderIdentity`] (namespace, name, output-affecting
/// revision) so provider provenance is separated from semantic meaning (ADR 0072)
/// and carries a versioned identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PhysicalProviderProvenance {
    provider: ProviderIdentity,
}

impl PhysicalProviderProvenance {
    /// Records that proposals were produced by `provider`.
    pub(crate) const fn new(provider: ProviderIdentity) -> Self {
        Self { provider }
    }

    /// Returns the provider identity.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }
}

/// The complete provenance of one admitted implementation: provider and kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImplementationProvenance {
    provider: ProviderIdentity,
    kind: PhysicalProposalKind,
}

impl ImplementationProvenance {
    /// Returns the provider that produced the implementation.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
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
    pub(crate) fn target_profile_key(&self) -> &'static str {
        self.request.target_profile().key
    }
}

/// A statically linked provider that proposes physical implementations of a
/// region on a target profile.
///
/// The provider is trusted, deterministic, and side-effect-free: it depends only
/// on its explicit context and returns zero or more proposals. Trust does not
/// mean belief — the host resubmits every scheduled-kernel body through the
/// ordinary checked verification path before admitting it.
pub(crate) trait PhysicalImplementationProvider {
    /// Returns this provider's provenance.
    fn provenance(&self) -> PhysicalProviderProvenance;

    /// Proposes physical implementations for the region in `context`.
    ///
    /// Returning an empty vector is legitimate: it means the provider offers no
    /// implementation for this region and target, which is neither an error nor a
    /// global-coverage claim.
    fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal>;
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedImplementation {
    provenance: ImplementationProvenance,
    verified: VerifiedScheduledRegion,
    feasibility: Vec<ResolvedPredicate>,
    boundary: BoundaryContract,
    cost: PhysicalCostEstimate,
    identity: ImplementationProposalIdentity,
}

impl AdmittedImplementation {
    /// Returns the provider and kind that produced this implementation.
    pub(crate) const fn provenance(&self) -> &ImplementationProvenance {
        &self.provenance
    }

    /// Returns the verified scheduled region backing this implementation.
    pub(crate) const fn verified(&self) -> &VerifiedScheduledRegion {
        &self.verified
    }

    /// Returns the exact resource requirements used for the feasibility decision.
    pub(crate) fn resources(&self) -> ResourceRequirements {
        self.verified.requirements()
    }

    /// Returns the resolved feasibility predicates admitting this implementation.
    pub(crate) fn feasibility(&self) -> &[ResolvedPredicate] {
        &self.feasibility
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
        provider: ProviderIdentity,
        /// The canonical key of the disproved capability axis.
        axis: &'static str,
        /// The amount the proposal required on that axis.
        required: u64,
        /// The amount the target profile made available on that axis.
        available: u64,
    },
    /// The proposal body is a reserved variant the P0 frontier does not implement.
    UnsupportedVariant {
        /// The provider whose proposal was rejected.
        provider: ProviderIdentity,
        /// The reserved proposal kind.
        kind: PhysicalProposalKind,
    },
    /// The proposal's applicability predicate excludes this target profile.
    NotApplicable {
        /// The provider whose proposal did not apply.
        provider: ProviderIdentity,
        /// The proposal kind that did not apply.
        kind: PhysicalProposalKind,
        /// The assessed target profile key the proposal did not target.
        target_profile_key: &'static str,
    },
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
                encode_provider(output, provider);
                encode_bytes(output, axis.as_bytes());
                output.extend_from_slice(&required.to_be_bytes());
                output.extend_from_slice(&available.to_be_bytes());
            }
            Self::UnsupportedVariant { provider, kind } => {
                output.push(2);
                encode_provider(output, provider);
                output.push(kind.tag());
            }
            Self::NotApplicable {
                provider,
                kind,
                target_profile_key,
            } => {
                output.push(3);
                encode_provider(output, provider);
                output.push(kind.tag());
                encode_bytes(output, target_profile_key.as_bytes());
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
    target_profile_key: &'static str,
    region_role: &'static str,
    admitted: Vec<AdmittedImplementation>,
    rejections: Vec<FrontierRejection>,
}

impl ImplementationFrontier {
    /// Returns the assessed target profile key.
    pub(crate) const fn target_profile_key(&self) -> &'static str {
        self.target_profile_key
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
    /// An implementation is retained unless another admitted implementation
    /// strictly dominates its cost estimate. Domination runs strictly *after*
    /// feasibility admission and only ever removes a proposal another proposal
    /// beats on cost; it never establishes or refutes feasibility.
    pub(crate) fn non_dominated(&self) -> Vec<&AdmittedImplementation> {
        self.admitted
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                !self
                    .admitted
                    .iter()
                    .enumerate()
                    .any(|(other_index, other)| {
                        *index != other_index && other.cost.dominates(&candidate.cost)
                    })
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
}

impl fmt::Display for FrontierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
        }
    }
}

impl Error for FrontierError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MalformedProposal { source, .. } => Some(source),
            Self::MalformedCostProvenance { .. } => None,
        }
    }
}

/// Enumerates the bounded implementation frontier for one region and target.
///
/// Each provider is asked for proposals over the region subject; every proposal
/// is processed in this fixed order:
///
/// 1. applicability — a proposal not targeting this profile is recorded as
///    [`FrontierRejection::NotApplicable`] and skipped;
/// 2. cost provenance — a proposal attributing its cost estimate to an ungoverned
///    model fails closed as [`FrontierError::MalformedCostProvenance`];
/// 3. body variant — a reserved (non-scheduled-kernel) body is recorded as
///    [`FrontierRejection::UnsupportedVariant`] and skipped, preserving the seam;
/// 4. checked verification — a scheduled-kernel body is resubmitted through
///    [`verify_schedule_with_feasibility`]. A [`FeasibilityOutcome::Proven`] verdict
///    admits it with derived resources, boundary contract, and feasibility
///    evidence; a [`PhysicalError::Target`] records [`FrontierRejection::Infeasible`];
///    any other [`PhysicalError`] fails closed as [`FrontierError::MalformedProposal`].
///
/// The admitted implementations and rejections are returned in canonical,
/// provider-order-independent order. An `Ok` with an empty admitted set is a valid
/// local no-plan result.
///
/// [`FeasibilityOutcome::Proven`]: crate::feasibility::FeasibilityOutcome::Proven
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
) -> Result<ImplementationFrontier, FrontierError> {
    let target_profile_key = request.target_profile().key;
    let mut admitted = Vec::new();
    let mut rejections = Vec::new();
    for provider in providers {
        let provenance = provider.provenance();
        let context = ImplementationContext { request, subject };
        for proposal in provider.propose(&context) {
            let kind = proposal.body.kind();
            if !proposal.applicability.applies_to(target_profile_key) {
                rejections.push(FrontierRejection::NotApplicable {
                    provider: provenance.provider().clone(),
                    kind,
                    target_profile_key,
                });
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
                ProposalBody::KernelSubprogram(_)
                | ProposalBody::OpaqueCall(_)
                | ProposalBody::View(_) => {
                    rejections.push(FrontierRejection::UnsupportedVariant {
                        provider: provenance.provider().clone(),
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
                Ok((verified, feasibility)) => {
                    let boundary = derive_boundary_contract(verified.region());
                    let identity = encode_proposal_identity(
                        verified.canonical_identity(),
                        provenance.provider(),
                        kind,
                        &proposal.applicability,
                        &boundary,
                    );
                    admitted.push(AdmittedImplementation {
                        provenance: ImplementationProvenance {
                            provider: provenance.provider().clone(),
                            kind,
                        },
                        verified,
                        feasibility,
                        boundary,
                        cost: proposal.declared_cost,
                        identity,
                    });
                }
                Err(PhysicalError::Target {
                    rule,
                    required,
                    available,
                    ..
                }) => {
                    rejections.push(FrontierRejection::Infeasible {
                        provider: provenance.provider().clone(),
                        axis: rule,
                        required,
                        available,
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
        target_profile_key,
        region_role: subject.role,
        admitted,
        rejections,
    })
}

fn encode_proposal_identity(
    region_identity: &CanonicalScheduledRegionIdentity,
    provider: &ProviderIdentity,
    kind: PhysicalProposalKind,
    applicability: &TargetApplicability,
    boundary: &BoundaryContract,
) -> ImplementationProposalIdentity {
    let mut bytes = PROPOSAL_IDENTITY_TAG.to_vec();
    encode_bytes(&mut bytes, region_identity.as_bytes());
    encode_provider(&mut bytes, provider);
    bytes.push(kind.tag());
    applicability.encode(&mut bytes);
    boundary.encode(&mut bytes);
    ImplementationProposalIdentity(bytes)
}

fn encode_rejection(rejection: &FrontierRejection) -> Vec<u8> {
    let mut bytes = Vec::new();
    rejection.encode(&mut bytes);
    bytes
}

const fn tensor_role_tag(role: TensorRole) -> u8 {
    match role {
        TensorRole::Input => 1,
        TensorRole::Intermediate => 2,
        TensorRole::Output => 3,
    }
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    encode_bytes(output, provider.namespace().as_bytes());
    encode_bytes(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
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
    use super::{
        AdmittedImplementation, BoundaryAvailability, BoundaryProduction, FrontierError,
        FrontierRegionSubject, FrontierRejection, ImplementationContext, ImplementationProposal,
        PhysicalCostEstimate, PhysicalImplementationProvider, PhysicalProposalKind,
        PhysicalProviderProvenance, ProposalBody, ReservedProposalSeam, TargetApplicability,
        enumerate_frontier,
    };
    use crate::physical::{build_fused_scheduled_region, pointwise_region};
    use crate::request::{CompilationRequest, VerifiedTargetRequest, verify_request};
    use tiler_ir::schedule::{ScheduledRegion, TensorRole};
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
        request.for_target(request.target_profiles()[0]).unwrap()
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
        TargetApplicability::for_targets([GOVERNED_TARGET_KEY])
    }

    /// A provider that proposes one checked scheduled-kernel body for the fused
    /// region with a caller-chosen provider identity and cost estimate.
    struct FusedScheduledKernelProvider {
        provider: ProviderIdentity,
        cost: PhysicalCostEstimate,
    }

    impl PhysicalImplementationProvider for FusedScheduledKernelProvider {
        fn provenance(&self) -> PhysicalProviderProvenance {
            PhysicalProviderProvenance::new(self.provider.clone())
        }

        fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
            vec![ImplementationProposal::new(
                ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                governed_applicability(),
                self.cost,
            )]
        }
    }

    #[test]
    fn additive_providers_both_admit_the_same_region() {
        // Two independent providers each contribute a checked implementation of
        // the same fused region. Unlike a singular-capability registry, this is
        // additive: both are admitted rather than colliding into an ambiguity.
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        let frontier = enumerate_frontier(&request, &subject, &providers).unwrap();

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
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = FusedScheduledKernelProvider {
            provider: provider_identity("alpha", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier = enumerate_frontier(&request, &subject, &providers).unwrap();

        let admitted = &frontier.admitted()[0];
        assert_eq!(
            admitted.provenance().kind(),
            PhysicalProposalKind::ScheduledKernel
        );
        // Exact feasibility resources are derived from the verified region.
        assert_eq!(admitted.resources().buffer_bindings, 2);
        assert!(admitted.resources().requires_strict_f32);
        // The feasibility admission carries resolved predicates as evidence.
        assert!(!admitted.feasibility().is_empty());
        // The fused region reads an Input boundary and produces the Output boundary.
        let requirements = admitted.boundary().requirements();
        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].tensor(), TensorRole::Input);
        assert_eq!(
            requirements[0].availability(),
            BoundaryAvailability::MaterializedInDeviceMemory
        );
        let guarantees = admitted.boundary().guarantees();
        assert_eq!(guarantees.len(), 1);
        assert_eq!(guarantees[0].tensor(), TensorRole::Output);
        assert_eq!(
            guarantees[0].production(),
            BoundaryProduction::TotalRaceFreeWrite
        );
    }

    #[test]
    fn identity_and_ordering_are_independent_of_provider_order() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        let first = enumerate_frontier(&request, &subject, &forward).unwrap();
        let second = enumerate_frontier(&request, &subject, &reverse).unwrap();

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
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("opaque", 1))
            }
            fn propose(&self, _: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                vec![
                    ImplementationProposal::new(
                        ProposalBody::OpaqueCall(ReservedProposalSeam::new("intrinsic.mystery")),
                        governed_applicability(),
                        PhysicalCostEstimate::structural(1, 2, 0),
                    ),
                    ImplementationProposal::new(
                        ProposalBody::KernelSubprogram(ReservedProposalSeam::new("subprogram")),
                        governed_applicability(),
                        PhysicalCostEstimate::structural(1, 2, 0),
                    ),
                    ImplementationProposal::new(
                        ProposalBody::View(ReservedProposalSeam::new("view")),
                        governed_applicability(),
                        PhysicalCostEstimate::structural(1, 2, 0),
                    ),
                ]
            }
        }

        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let scheduled = FusedScheduledKernelProvider {
            provider: provider_identity("alpha", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let opaque = OpaqueProvider;
        let providers: [&dyn PhysicalImplementationProvider; 2] = [&scheduled, &opaque];
        let frontier = enumerate_frontier(&request, &subject, &providers).unwrap();

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
        assert!(rejected_kinds.contains(&PhysicalProposalKind::OpaqueCall));
        assert!(rejected_kinds.contains(&PhysicalProposalKind::KernelSubprogram));
        assert!(rejected_kinds.contains(&PhysicalProposalKind::View));
    }

    #[test]
    fn a_cheap_infeasible_proposal_is_rejected_while_an_expensive_feasible_one_is_admitted() {
        // Infeasibility is a disproved capability predicate, never a cost: a
        // proposal with a tiny cost estimate whose grid exceeds the profile is
        // rejected, while a proposal with a large cost estimate that fits is
        // admitted. Cost never gates feasibility in either direction.
        struct InfeasibleProvider;
        impl PhysicalImplementationProvider for InfeasibleProvider {
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("infeasible", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                let (region, _) = pointwise_region(context.request());
                vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(region)),
                    governed_applicability(),
                    // A deliberately cheap estimate cannot rescue an infeasible plan.
                    PhysicalCostEstimate::structural(1, 1, 0),
                )]
            }
        }

        let large = request(Shape::from_dims([70_000, 1]), [Axis::new(1)]);
        let infeasible_subject = pointwise_subject(&large);
        let infeasible = InfeasibleProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&infeasible];
        let frontier = enumerate_frontier(&large, &infeasible_subject, &providers).unwrap();
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
        assert_eq!(*available, 65_535);

        // A feasible proposal with an expensive estimate is still admitted.
        let small = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let feasible_subject = fused_subject(&small);
        let expensive = FusedScheduledKernelProvider {
            provider: provider_identity("expensive", 1),
            cost: PhysicalCostEstimate::structural(u32::MAX, u64::MAX, u64::MAX),
        };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&expensive];
        let frontier = enumerate_frontier(&small, &feasible_subject, &providers).unwrap();
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
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("malformed", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                let mut region = fused_region(context.request());
                region.index.numerical.canonical_arithmetic_nan_bits ^= 1;
                vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(region)),
                    governed_applicability(),
                    PhysicalCostEstimate::structural(1, 2, 0),
                )]
            }
        }

        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let malformed = MalformedProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&malformed];
        let error = enumerate_frontier(&request, &subject, &providers).unwrap_err();
        assert!(matches!(error, FrontierError::MalformedProposal { .. }));
    }

    #[test]
    fn an_ungoverned_cost_model_is_malformed_output() {
        struct WrongCostModelProvider;
        impl PhysicalImplementationProvider for WrongCostModelProvider {
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("wrong-cost", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                    governed_applicability(),
                    PhysicalCostEstimate::new("tiler.cost.ungoverned.v9", 1, 2, 0),
                )]
            }
        }

        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = WrongCostModelProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let error = enumerate_frontier(&request, &subject, &providers).unwrap_err();
        assert!(matches!(
            error,
            FrontierError::MalformedCostProvenance {
                declared_model_key: "tiler.cost.ungoverned.v9",
                ..
            }
        ));
    }

    #[test]
    fn a_proposal_for_another_target_is_not_applicable() {
        struct ForeignTargetProvider;
        impl PhysicalImplementationProvider for ForeignTargetProvider {
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("foreign", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                    TargetApplicability::for_targets(["tiler.some-other-target.v1"]),
                    PhysicalCostEstimate::structural(1, 2, 0),
                )]
            }
        }

        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = ForeignTargetProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier = enumerate_frontier(&request, &subject, &providers).unwrap();
        assert!(frontier.admitted().is_empty());
        assert_eq!(frontier.rejections().len(), 1);
        assert!(matches!(
            frontier.rejections()[0],
            FrontierRejection::NotApplicable {
                kind: PhysicalProposalKind::ScheduledKernel,
                target_profile_key: GOVERNED_TARGET_KEY,
                ..
            }
        ));
    }

    #[test]
    fn non_domination_retains_the_pareto_frontier_after_feasibility() {
        // Three feasible proposals of the same region: a dominated one (worse on a
        // dimension, no better on any) is pruned; two incomparable ones are both
        // retained. Pruning runs strictly after feasibility admission.
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        let frontier = enumerate_frontier(&request, &subject, &providers).unwrap();

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
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let providers: [&dyn PhysicalImplementationProvider; 0] = [];
        let frontier = enumerate_frontier(&request, &subject, &providers).unwrap();
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
        let applicability =
            TargetApplicability::for_targets([GOVERNED_TARGET_KEY, GOVERNED_TARGET_KEY]);
        assert_eq!(applicability.target_profile_keys(), [GOVERNED_TARGET_KEY]);
    }

    /// Keeps the unused-field lint honest for the reserved seam descriptor and the
    /// admitted-implementation verified region accessor.
    #[test]
    fn admitted_exposes_its_verified_region() {
        fn _uses_admitted(admitted: &AdmittedImplementation) {
            let _ = admitted.verified().region();
            let _ = admitted.cost();
        }
    }
}
