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
use crate::call_declaration::{GuaranteeError, OpaqueCallDeclaration};
use crate::call_registry::{
    OpaqueCallIdentity, OpaqueCallProposal, OpaqueCallRegistry, RegisteredCall,
};
use crate::feasibility::ProvenEvidence;
use crate::honourability::UnhonouredDimension;
use crate::physical::{PhysicalError, VerifiedScheduledRegion, verify_schedule_with_feasibility};
use crate::region::SemanticMemberId;
use crate::request::{TargetProfileKey, VerifiedTargetRequest};

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
/// implements only [`Self::ScheduledKernel`] and reserves the rest so an
/// unsupported body rejects explicitly instead of being silently approximated.
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
    /// A nested kernel subprogram. Reserved; the P0 frontier rejects it.
    KernelSubprogram(ReservedProposalSeam),
    /// An opaque physical call, named by its registered identity.
    ///
    /// The provider proposes an *identity* rather than the call itself, so
    /// registration is the authority on which calls exist: a provider cannot
    /// propose one it never registered. Still rejected by the P0 frontier —
    /// admitting it needs feasibility, boundary, and cost derived from the
    /// declaration rather than from a scheduled region — but an unregistered
    /// identity is now a distinct, earlier rejection.
    OpaqueCall(Box<OpaqueCallProposal>),
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
            output.push(tensor_role_tag(requirement.tensor));
            output.push(access_mode_tag(requirement.access));
            requirement.properties.encode(output);
        }
        push_len(output, self.guarantees.len());
        for guarantee in &self.guarantees {
            output.push(tensor_role_tag(guarantee.tensor));
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

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
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
    /// A call into code this compiler did not produce.
    Opaque(Box<RegisteredCall>),
}

#[allow(
    dead_code,
    reason = "see the type's own allow: accessors land with the sum, ahead of the consumers that will match on it"
)]
impl ImplementationBody {
    /// The scheduled region, when this is one.
    ///
    /// `Option` rather than a panicking accessor: a consumer that needs a
    /// schedule and receives an opaque call has to say what it does about that,
    /// and the type is where it is made to.
    pub(crate) fn scheduled(&self) -> Option<&VerifiedScheduledRegion> {
        match self {
            Self::Scheduled(region) => Some(region),
            Self::Opaque(_) => None,
        }
    }

    /// The registered call, when this is one.
    pub(crate) fn opaque(&self) -> Option<&RegisteredCall> {
        match self {
            Self::Opaque(call) => Some(call),
            Self::Scheduled(_) => None,
        }
    }

    /// The stable code naming which kind this is, for typed rejections.
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::Scheduled(_) => "scheduled-region",
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
    target_profile_key: &'static str,
    body: ImplementationBody,
    feasibility: ProvenEvidence,
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
    pub(crate) const fn target_profile_key(&self) -> &'static str {
        self.target_profile_key
    }

    /// The scheduled region this admission lowers, when it is one.
    ///
    /// `Option` because an admission may be an opaque call, which has no
    /// schedule. A consumer that needs one must say what it does about the
    /// absence rather than receive a substitute.
    pub(crate) fn scheduled(&self) -> Option<&VerifiedScheduledRegion> {
        self.body.scheduled()
    }

    /// What this admission is.
    pub(crate) const fn body(&self) -> &ImplementationBody {
        &self.body
    }

    /// Returns the exact resource requirements used for the feasibility decision.
    pub(crate) fn resources(&self) -> ResourceRequirements {
        // Both bodies answer, from different authorities: a scheduled region
        // derives its requirements, and an opaque call declares them as proven
        // — which is why the declaration carries `ResourceRequirements` and not
        // the uncertain estimate class. Neither is defaulted; feasibility must
        // never be told a call needs nothing because nobody said.
        match &self.body {
            ImplementationBody::Scheduled(region) => region.requirements(),
            ImplementationBody::Opaque(call) => *call.declaration().resources(),
        }
    }

    /// Returns the feasibility evidence admitting this implementation: the
    /// resolved capability predicates and the honoured numerical dimensions.
    pub(crate) const fn feasibility(&self) -> &ProvenEvidence {
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
        provider: ProviderIdentity,
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
        provider: ProviderIdentity,
        /// The dimension, required behaviour, declared means, honoured
        /// alternative, and declaring profile.
        cause: UnhonouredDimension,
    },
    /// The proposal's parameter bindings do not match the call's own ABI.
    ///
    /// Distinct from an unregistered call: the call exists, and the provider
    /// described how to bind it wrongly. Carries the ABI's own typed fault so
    /// the rejection says which parameter and how.
    MalformedBinding {
        /// The provider whose proposal was rejected.
        provider: ProviderIdentity,
        /// The call whose bindings did not match.
        call: OpaqueCallIdentity,
        /// What the ABI said was wrong.
        fault: crate::call_abi::BindingError,
    },
    /// The proposal names an opaque call no registry entry claims.
    ///
    /// Distinct from [`Self::UnsupportedVariant`] because it says something
    /// different and is actionable in a different way: an unsupported variant is
    /// this compiler's limitation, while an unregistered identity is the
    /// provider naming something that does not exist. Reporting the second as
    /// the first would tell a caller to wait for a feature when the fix is to
    /// register the call.
    UnregisteredCall {
        /// The provider whose proposal named it.
        provider: ProviderIdentity,
        /// The identity no entry claims.
        call: OpaqueCallIdentity,
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
                push_slice(output, axis.as_bytes());
                output.extend_from_slice(&required.to_be_bytes());
                output.extend_from_slice(&available.to_be_bytes());
            }
            Self::Unhonourable { provider, cause } => {
                output.push(4);
                encode_provider(output, provider);
                output.push(cause.dimension().tag());
                output.extend_from_slice(&cause.required().tag());
                push_slice(output, cause.means().key().as_bytes());
                match cause.honoured() {
                    Some(honoured) => {
                        output.push(1);
                        output.extend_from_slice(&honoured.tag());
                    }
                    None => output.push(0),
                }
                push_slice(output, cause.profile().key().as_bytes());
            }
            Self::MalformedBinding {
                provider,
                call,
                fault,
            } => {
                output.push(6);
                encode_provider(output, provider);
                push_slice(output, call.call().as_bytes());
                push_slice(output, format!("{fault}").as_bytes());
            }
            Self::UnregisteredCall { provider, call } => {
                output.push(5);
                encode_provider(output, provider);
                push_slice(output, call.provider().as_bytes());
                push_slice(output, call.call().as_bytes());
                output.extend_from_slice(&call.revision().to_be_bytes());
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
                push_slice(output, target_profile_key.as_bytes());
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

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
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
}

impl FrontierError {
    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::MalformedProposal { .. } => "malformed-proposal",
            Self::MalformedCostProvenance { .. } => "malformed-cost-provenance",
            Self::UndeterminedBoundaryProperty { .. } => "undetermined-boundary-property",
        }
    }
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
            Self::UndeterminedBoundaryProperty { provider, rule } => write!(
                formatter,
                "frontier.undetermined-boundary-property: provider {provider} emitted a region whose boundary property {rule} is undetermined"
            ),
        }
    }
}

impl Error for FrontierError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MalformedProposal { source, .. } => Some(source),
            Self::MalformedCostProvenance { .. } | Self::UndeterminedBoundaryProperty { .. } => {
                None
            }
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
    calls: &OpaqueCallRegistry,
) -> Result<ImplementationFrontier, FrontierError> {
    #[cfg(test)]
    crate::workcount::FRONTIER_ENUMERATIONS.record();
    let target_profile_key = request.target_profile().key;
    // The applicability predicate speaks in `TargetProfileKey`; the rejection
    // diagnostics below still carry the raw key, which stays a `&'static str`
    // until the profile itself becomes caller-declared.
    let applicable_key = TargetProfileKey::governed(target_profile_key);
    let mut admitted = Vec::new();
    let mut rejections = Vec::new();
    for provider in providers {
        let provenance = provider.provenance();
        let context = ImplementationContext { request, subject };
        for proposal in provider.propose(&context) {
            let kind = proposal.body.kind();
            if !proposal.applicability.applies_to(&applicable_key) {
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
                ProposalBody::OpaqueCall(ref proposed) => {
                    let Some(registered) = calls.get(proposed.call()) else {
                        rejections.push(FrontierRejection::UnregisteredCall {
                            provider: provenance.provider().clone(),
                            call: proposed.call(),
                        });
                        continue;
                    };
                    // The provider's binding claim, checked against the call's
                    // own ABI before anything downstream trusts it.
                    if let Err(fault) = crate::call_abi::check_bindings(
                        registered.declaration().abi(),
                        proposed.bindings(),
                    ) {
                        rejections.push(FrontierRejection::MalformedBinding {
                            provider: provenance.provider().clone(),
                            call: proposed.call(),
                            fault,
                        });
                        continue;
                    }
                    // Admitting needs feasibility evidence, and the only
                    // producer is `verify_schedule_with_feasibility`, which
                    // bundles it with verifying a schedule an opaque call does
                    // not have. Until a resource-only feasibility check exists,
                    // a well-bound registered call is still unsupported — and
                    // fabricating `ProvenEvidence` here would tell feasibility a
                    // call was proven when nothing proved it.
                    let _ = derive_call_boundary_contract(
                        registered.declaration(),
                        proposed.bindings(),
                    );
                    rejections.push(FrontierRejection::UnsupportedVariant {
                        provider: provenance.provider().clone(),
                        kind,
                    });
                    continue;
                }
                ProposalBody::KernelSubprogram(_) | ProposalBody::View(_) => {
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
                    admitted.push(admit_verified(
                        verified,
                        feasibility,
                        provenance.provider(),
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
                        provider: provenance.provider().clone(),
                        axis: rule,
                        required,
                        available,
                    });
                }
                Err(PhysicalError::Numerical { cause, .. }) => {
                    rejections.push(FrontierRejection::Unhonourable {
                        provider: provenance.provider().clone(),
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
        target_profile_key,
        region_role: subject.role,
        admitted,
        rejections,
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
#[allow(
    dead_code,
    reason = "the opaque contract assembly; lands with its tests ahead of the admission that calls it"
)]
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

        let bound = |wants_write: bool| {
            bindings
                .iter()
                .filter(|(_, bound_role)| bound_role == role)
                .find_map(|(name, _)| {
                    let parameter = declaration.abi().parameter(name)?;
                    (parameter.role().writes() == wants_write).then_some(parameter)
                })
        };

        if let Some(parameter) = bound(false)
            && let Some(properties) =
                crate::call_declaration::required_properties_for(parameter, declaration.placement())
        {
            requirements.push(BoundaryRequirement {
                tensor: *role,
                access: AccessMode::Read,
                properties,
            });
        }
        if let Some(parameter) = bound(true) {
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
#[allow(
    dead_code,
    reason = "the opaque proposal's canonical bytes; lands with the contract derivation, ahead of the admission that will pair them"
)]
fn encode_call_subject(proposed: &OpaqueCallProposal) -> Vec<u8> {
    let mut bytes = Vec::new();
    let call = proposed.call();
    push_slice(&mut bytes, call.provider().as_bytes());
    push_slice(&mut bytes, call.call().as_bytes());
    bytes.extend_from_slice(&call.revision().to_be_bytes());
    for (name, role) in proposed.bindings() {
        push_slice(&mut bytes, name.as_bytes());
        // Written as an exhaustive match rather than read from the discriminant,
        // so adding or reordering a role is a build error here instead of a
        // silent change to every opaque proposal identity ever encoded.
        bytes.push(match role {
            TensorRole::Input => 0x01,
            TensorRole::Intermediate => 0x02,
            TensorRole::Output => 0x03,
        });
    }
    bytes
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
    feasibility: ProvenEvidence,
    provider: &ProviderIdentity,
    kind: PhysicalProposalKind,
    applicability: &TargetApplicability,
    cost: PhysicalCostEstimate,
) -> Result<AdmittedImplementation, FrontierError> {
    let boundary = derive_boundary_contract(&verified).map_err(|rule| {
        FrontierError::UndeterminedBoundaryProperty {
            provider: provider.clone(),
            rule,
        }
    })?;
    let identity = encode_proposal_identity(
        verified.canonical_identity().as_bytes(),
        provider,
        kind,
        applicability,
        &boundary,
    );
    Ok(AdmittedImplementation {
        provenance: ImplementationProvenance {
            provider: provider.clone(),
            kind,
        },
        semantic_members: verified.semantic_members().to_vec(),
        target_profile_key: verified.target_profile_key(),
        body: ImplementationBody::Scheduled(Box::new(verified)),
        feasibility,
        boundary,
        cost,
        identity,
    })
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
    fn provenance(&self) -> PhysicalProviderProvenance {
        PhysicalProviderProvenance::new(Self::identity())
    }

    fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
        let request = context.request();
        let members = context.subject().semantic_members();
        let recognized = &request.serial_sum().members;
        let input_elements = request.serial_sum().input_elements;
        let output_elements = request.serial_sum().output_elements;
        // A materialized f32 intermediate costs four bytes per element. The
        // estimate is structural and is never a feasibility input.
        let intermediate_bytes = input_elements.saturating_mul(4);
        let applicability = TargetApplicability::for_targets([TargetProfileKey::governed(
            request.target_profile().key,
        )]);
        let (region, cost) = if members == recognized.pointwise() {
            (
                crate::physical::pointwise_region(request).0,
                PhysicalCostEstimate::structural(1, input_elements, intermediate_bytes),
            )
        } else if members == recognized.reduction() {
            (
                crate::physical::reduction_region(request).0,
                PhysicalCostEstimate::structural(1, output_elements, 0),
            )
        } else if members == recognized.all() {
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
            return Vec::new();
        };
        vec![ImplementationProposal::new(
            ProposalBody::ScheduledKernel(Box::new(region)),
            applicability,
            cost,
        )]
    }
}

fn encode_proposal_identity(
    subject_bytes: &[u8],
    provider: &ProviderIdentity,
    kind: PhysicalProposalKind,
    applicability: &TargetApplicability,
    boundary: &BoundaryContract,
) -> ImplementationProposalIdentity {
    let mut bytes = PROPOSAL_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, subject_bytes);
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
        ImplementationProposal, PhysicalCostEstimate, PhysicalImplementationProvider,
        PhysicalProposalKind, PhysicalProviderProvenance, ProposalBody, ReservedProposalSeam,
        TargetApplicability, bounded_guarantees, bounded_requirements, enumerate_frontier,
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
        AccessMode, NumericalPermission, ScheduledRegion, SubnormalMode, TensorRole,
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
        TargetApplicability::for_targets([TargetProfileKey::governed(GOVERNED_TARGET_KEY)])
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
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        assert!(!admitted.feasibility().is_empty());
        // The fused region reads an Input boundary and produces the Output boundary.
        let requirements = admitted.boundary().requirements();
        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].tensor(), TensorRole::Input);
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
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("opaque", 1))
            }
            fn propose(&self, _: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                vec![
                    ImplementationProposal::new(
                        ProposalBody::OpaqueCall(Box::new(OpaqueCallProposal::new(
                            OpaqueCallIdentity::new("test", "mystery", 1).expect("named"),
                            Vec::new(),
                        ))),
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
        assert!(rejected_kinds.contains(&PhysicalProposalKind::KernelSubprogram));
        assert!(rejected_kinds.contains(&PhysicalProposalKind::View));

        // The opaque proposal names an identity no entry claims, so it is
        // rejected *earlier* and differently: `UnregisteredCall`, not
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
                FrontierRejection::UnregisteredCall { call, .. }
                    if call.call() == "mystery"
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
        assert_eq!(*available, 65_535);

        // A feasible proposal with an expensive estimate is still admitted.
        let small = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        let error = enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new())
            .unwrap_err();
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
            fn provenance(&self) -> PhysicalProviderProvenance {
                PhysicalProviderProvenance::new(provider_identity("analytical", 1))
            }
            fn propose(&self, context: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
                vec![ImplementationProposal::new(
                    ProposalBody::ScheduledKernel(Box::new(fused_region(context.request()))),
                    governed_applicability(),
                    PhysicalCostEstimate::new(crate::component_cost::ANALYTICAL_MODEL_KEY, 1, 2, 0),
                )]
            }
        }

        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        use crate::call_declaration::OpaqueCallDeclaration;
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
                barriers: 0,
                requires_device_memory: true,
                input_subnormals: SubnormalMode::Preserve,
                result_subnormals: SubnormalMode::Preserve,
                contraction: NumericalPermission::Forbidden,
                reassociation: NumericalPermission::Forbidden,
            },
        )
        .expect("coherent");

        let contract = derive_call_boundary_contract(
            &declaration,
            &[("x", TensorRole::Input), ("y", TensorRole::Output)],
        )
        .expect("a single admitted domain gives a guarantee");

        assert_eq!(contract.requirements.len(), 1);
        assert_eq!(contract.requirements[0].tensor(), TensorRole::Input);
        assert_eq!(contract.guarantees.len(), 1);
        assert_eq!(contract.guarantees[0].tensor(), TensorRole::Output);

        // Binding the *same* parameters to swapped roles moves the contract with
        // them: the derivation reads the binding, not the parameter order.
        let swapped = derive_call_boundary_contract(
            &declaration,
            &[("x", TensorRole::Output), ("y", TensorRole::Input)],
        )
        .expect("still one domain");
        assert_eq!(swapped.requirements[0].tensor(), TensorRole::Output);
        assert_eq!(swapped.guarantees[0].tensor(), TensorRole::Input);
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
                    TargetApplicability::for_targets([TargetProfileKey::governed(
                        "tiler.some-other-target.v1",
                    )]),
                    PhysicalCostEstimate::structural(1, 2, 0),
                )]
            }
        }

        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let subject = fused_subject(&request);
        let provider = ForeignTargetProvider;
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&provider];
        let frontier =
            enumerate_frontier(&request, &subject, &providers, &OpaqueCallRegistry::new()).unwrap();
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
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
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
}
