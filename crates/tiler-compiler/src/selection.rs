//! Complete physical-plan selection: the first authority allowed to join a
//! legal complete cover with compatible per-region physical implementations.
//!
//! Complete-cover enumeration ([`crate::cover`]) is a strictly *global* legality
//! authority: it answers which bounded partitionings legally cover the whole
//! semantic graph, choosing no implementation. The per-region implementation
//! frontier ([`crate::frontier`]) is a strictly *local* authority: for one region
//! and one target it enumerates the feasible, verified implementations, proving
//! no global coverage. Neither depends on the other. This module is the first
//! authority allowed to *join* them: it takes one independently verified legal
//! cover plus one already-enumerated implementation frontier per region and
//! selects the complete physical plans whose per-region implementations compose,
//! per `docs/compiler/fusion-and-scheduling.md`.
//!
//! The join keeps the concerns the correctness contract insists on separating:
//!
//! - **Complete occurrence/output coverage, re-verified.** The cover is
//!   re-verified from the program by [`crate::cover::verify_cover`], so a stale or
//!   forged cover cannot enter a plan. Every cover region is bound to exactly one
//!   selected implementation whose verified region covers exactly that region's
//!   members; a plan that leaves a region unimplemented or double-selects a region
//!   is rejected, never silently repaired.
//! - **Boundary agreement across each materialization edge.** For each
//!   [`crate::cover::MaterializationEdge`] the producer region's selected
//!   implementation must guarantee the materialized cross-region tensor and every
//!   consumer region's selected implementation must require it, with every typed
//!   property the consumer requires discharged by the producer's guarantee under
//!   [`crate::boundary::unsatisfied_properties`]. A property the producer is
//!   silent on does not pass. The reconciliation is derived from the verified
//!   regions' boundary contracts, never from a provider claim, and fails closed
//!   on any dangling read, leaked intermediate, undischarged handoff, or
//!   ambiguous boundary.
//! - **Hard feasibility stays distinct from cost.** Feasibility was already
//!   decided by the frontier; a selected implementation is feasible by
//!   construction. Cost never gates validity: the portfolio retains every valid
//!   complete plan, and structural dominance ([`SelectedPortfolio::non_dominated`])
//!   is a pure *view* that only prunes a plan another plan beats on the exact
//!   structural dimensions. This is proved structural dominance, not an
//!   uncalibrated latency total order: the dimensions are exact structural counts
//!   and are compared only by the Pareto relation, never collapsed into a scalar.
//! - **A non-forgeable, deterministic receipt.** A [`SelectedPlan`] is produced
//!   only by the checked constructor [`select_physical_plans`]; its
//!   [`SelectedPlanIdentity`] folds the cover identity, the per-region
//!   implementation identities, the satisfied handoffs, the aggregate guards, and
//!   the aggregate structural cost in a canonical length-prefixed byte encoding
//!   over content-derived coordinates, excluding transient ordinals and any
//!   `HashMap` order. [`verify_selected_plan`] re-derives the whole plan and must
//!   reproduce it exactly, so a tampered field or a foreign implementation fails
//!   closed.
//!
//! This receipt is deliberately *not* the final executable-program authority. It
//! is a selection-level receipt over verified scheduled regions, distinct from
//! structured KIR and from [`tiler_ir::kernel::VerifiedKernel`] and a
//! `KernelProgram`. Post-KIR `KernelProgram` assembly — buffers, initialization,
//! lifetimes, aliasing, storage handoffs, ABI/launch references, executable stage
//! coverage, and routing — is a later authority and is neither performed nor
//! pre-empted here.
//!
//! Every item here is a reviewed *draft* boundary, not a stable compiler API,
//! until Tom accepts the exact interface.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::schedule::{AccessMode, TensorRole};
use tiler_ir::semantic::SemanticProgram;

use crate::boundary::{
    GuaranteedProperties, RequiredProperties, UnsatisfiedProperty, unsatisfied_properties,
};
use crate::cover::{CoverError, MaterializationEdge, RegionCover, verify_cover};
use crate::feasibility::ResolvedPredicate;
use crate::frontier::{
    AdmittedImplementation, BoundaryOwnership, FrontierRegionSubject, ImplementationFrontier,
};
use crate::honourability::HonouredDimension;
use crate::region::{RegionFormationOutcome, RegionOccurrenceIdentity};
use crate::request::DeterministicBudgets;

/// Canonical domain-separation tag for one selected-plan identity.
const SELECTED_PLAN_IDENTITY_TAG: &[u8] = b"tiler.compiler.selected-physical-plan.v1\0";
/// Canonical domain-separation tag for one selected-portfolio identity.
const SELECTED_PORTFOLIO_IDENTITY_TAG: &[u8] = b"tiler.compiler.selected-physical-portfolio.v1\0";
/// The structural-Pareto selection policy this authority applies. It matches the
/// pipeline's policy key so a later selector compares plans under one named policy.
const SELECTION_POLICY_KEY: &str = "tiler.selection.structural-pareto.v1";

/// A deterministic budget that bounds complete-plan enumeration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum PlanBudgetResource {
    /// Complete-plan combinations admitted for one cover source.
    Combinations,
}

impl PlanBudgetResource {
    /// Returns the stable resource key.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Combinations => "physical-plan-combinations",
        }
    }
}

/// One declared plan budget and the demand it refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanBudgetStop {
    /// The budget that fired.
    pub(crate) resource: PlanBudgetResource,
    /// The declared limit.
    pub(crate) limit: u64,
    /// The refused demand observed at the stop point.
    ///
    /// This is a lower bound on the unexplored combinations rather than their
    /// exact count: enumeration stops at the first combination the limit refuses.
    pub(crate) actual: u64,
    /// The cover identity whose enumeration the budget stopped.
    pub(crate) cover: Vec<u8>,
}

/// The exact aggregate structural cost of one complete physical plan.
///
/// It is the exact structural sum of the per-region implementation estimates plus
/// the number of deliberate cross-region materializations the cover realizes.
/// Every dimension is an exact structural count, and dominance is only ever the
/// Pareto relation over them — never a scalar latency. A cost is never a
/// feasibility input and never gates validity; it only prunes a dominated plan
/// from the [`SelectedPortfolio::non_dominated`] view. Estimates from different
/// cost models are incomparable, so neither dominates the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlanStructuralCost {
    model_key: &'static str,
    dispatch_count: u64,
    launched_threads: u64,
    temporary_bytes: u64,
    materialization_count: u64,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl PlanStructuralCost {
    /// Returns the cost-model key every aggregated estimate was attributed to.
    pub(crate) const fn model_key(&self) -> &'static str {
        self.model_key
    }

    /// Returns the aggregate dispatch count across the plan's regions.
    pub(crate) const fn dispatch_count(&self) -> u64 {
        self.dispatch_count
    }

    /// Returns the aggregate launched-thread count across the plan's regions.
    pub(crate) const fn launched_threads(&self) -> u64 {
        self.launched_threads
    }

    /// Returns the aggregate temporary-allocation bytes across the plan's regions.
    pub(crate) const fn temporary_bytes(&self) -> u64 {
        self.temporary_bytes
    }

    /// Returns the number of deliberate cross-region materializations.
    pub(crate) const fn materialization_count(&self) -> u64 {
        self.materialization_count
    }

    /// Returns whether this cost strictly dominates `other`.
    ///
    /// Domination is the standard Pareto relation over the structural dimensions:
    /// no dimension is worse and at least one is strictly better. Costs from
    /// different cost models are incomparable.
    pub(crate) fn dominates(&self, other: &Self) -> bool {
        if self.model_key != other.model_key {
            return false;
        }
        let no_worse = self.dispatch_count <= other.dispatch_count
            && self.launched_threads <= other.launched_threads
            && self.temporary_bytes <= other.temporary_bytes
            && self.materialization_count <= other.materialization_count;
        let strictly_better = self.dispatch_count < other.dispatch_count
            || self.launched_threads < other.launched_threads
            || self.temporary_bytes < other.temporary_bytes
            || self.materialization_count < other.materialization_count;
        no_worse && strictly_better
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_slice(output, self.model_key.as_bytes());
        output.extend_from_slice(&self.dispatch_count.to_be_bytes());
        output.extend_from_slice(&self.launched_threads.to_be_bytes());
        output.extend_from_slice(&self.temporary_bytes.to_be_bytes());
        output.extend_from_slice(&self.materialization_count.to_be_bytes());
    }
}

/// One cross-region materialization handoff whose boundary contracts agree.
///
/// It records the materialized value's content-derived coordinates (the producing
/// member's canonical position and its result position), the producing region's
/// occurrence identity, the consuming regions' occurrence identities in canonical
/// order, and the tensor role plus the two side qualifiers — the producer's
/// ownership and the consumer's access mode — that the discharged edge carried.
/// Holding one is evidence that the producer's [`BoundaryGuarantee`] discharged
/// every typed property every consumer's [`BoundaryRequirement`] named for the
/// edge.
///
/// [`BoundaryGuarantee`]: crate::frontier::BoundaryGuarantee
/// [`BoundaryRequirement`]: crate::frontier::BoundaryRequirement
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SatisfiedHandoff {
    producer_position: u32,
    result_position: u32,
    producer: RegionOccurrenceIdentity,
    consumers: Vec<RegionOccurrenceIdentity>,
    role: TensorRole,
    ownership: BoundaryOwnership,
    access: AccessMode,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl SatisfiedHandoff {
    /// Returns the canonical position of the producing member.
    pub(crate) const fn producer_position(&self) -> u32 {
        self.producer_position
    }

    /// Returns the result position of the materialized value on its producer.
    pub(crate) const fn result_position(&self) -> u32 {
        self.result_position
    }

    /// Returns the occurrence identity of the producing region.
    pub(crate) const fn producer(&self) -> &RegionOccurrenceIdentity {
        &self.producer
    }

    /// Returns the occurrence identities of the consuming regions.
    pub(crate) fn consumers(&self) -> &[RegionOccurrenceIdentity] {
        &self.consumers
    }

    /// Returns the materialized tensor role of the handoff.
    pub(crate) const fn role(&self) -> TensorRole {
        self.role
    }

    /// Returns the ownership with which the producer wrote the handed-off value.
    pub(crate) const fn ownership(&self) -> BoundaryOwnership {
        self.ownership
    }

    /// Encodes the handoff's content-derived coordinates and its two side
    /// qualifiers.
    ///
    /// The discharged property sets are deliberately absent: each region's
    /// contract is already folded into its own `ImplementationProposalIdentity`,
    /// which this plan's identity folds in turn, so encoding the properties again
    /// here would restate the same facts under a second authority.
    fn encode(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.producer_position.to_be_bytes());
        output.extend_from_slice(&self.result_position.to_be_bytes());
        push_slice(output, self.producer.as_bytes());
        push_len(output, self.consumers.len());
        for consumer in &self.consumers {
            push_slice(output, consumer.as_bytes());
        }
        output.push(tensor_role_tag(self.role));
        output.push(ownership_tag(self.ownership));
        output.push(access_mode_tag(self.access));
    }

    fn sort_key(&self) -> Vec<u8> {
        let mut key = Vec::new();
        self.encode(&mut key);
        key
    }
}

/// One region occurrence's selected implementation within a plan.
///
/// It binds a cover region's occurrence identity to the exact
/// [`AdmittedImplementation`] chosen for it. The implementation is an unforgeable
/// checked token: holding one is evidence it passed whole-region intrinsic
/// verification, the request-subject binding, and hard feasibility on the frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionSelection {
    occurrence: RegionOccurrenceIdentity,
    implementation: AdmittedImplementation,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl RegionSelection {
    /// Returns the cover region occurrence this selection implements.
    pub(crate) const fn occurrence(&self) -> &RegionOccurrenceIdentity {
        &self.occurrence
    }

    /// Returns the selected admitted implementation.
    pub(crate) const fn implementation(&self) -> &AdmittedImplementation {
        &self.implementation
    }
}

/// Collision-free, order-independent identity of one complete physical plan.
///
/// It folds the cover identity, the per-region selected implementation identities,
/// the satisfied handoffs, the aggregate guards, and the aggregate structural cost
/// over content-derived canonical coordinates. Transient ordinals and enumeration
/// order are deliberately absent.
#[derive(Clone, Debug)]
pub(crate) struct SelectedPlanIdentity {
    /// The canonical bytes, which are the identity.
    bytes: Vec<u8>,
    /// `digest(bytes)`, folded once at construction.
    ///
    /// The label and the label check both need it, and a profile charged 4.3%
    /// of a compile to computing it twice per alternative — once to build a
    /// plan's `stable_id` and once to verify that id against the same bytes.
    ///
    /// **Caching does not weaken that verification.** Its power is that
    /// `stable_id` is a `String` a caller could have tampered with, compared
    /// against a value derived from `bytes`; a digest folded at construction is
    /// still derived from `bytes`, so a forged id is still refused. It is not
    /// `verify_portfolio`'s situation, which re-derives a plan and must never be
    /// handed the plan it is checking — this re-hashes bytes it already holds.
    digest: u64,
}

/// Every comparison is over the bytes alone, never the cached digest.
///
/// The digest is a function of the bytes, so including it could only ever
/// agree — and excluding it makes that impossible to get wrong. This type is a
/// map key and a sort key, so two identities with equal bytes comparing unequal
/// would be an identity defect, which is why these are written out rather than
/// derived over a struct that now has a second field.
impl PartialEq for SelectedPlanIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for SelectedPlanIdentity {}

impl PartialOrd for SelectedPlanIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SelectedPlanIdentity {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl SelectedPlanIdentity {
    /// Returns the canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns a bounded explain label for this plan.
    ///
    /// The label is a digest of the canonical bytes and is presentation only.
    /// Equality decisions always use [`Self::as_bytes`].
    pub(crate) fn label(&self) -> String {
        format!("selected-plan:{:016x}", self.digest)
    }

    /// Whether `label` is exactly what [`Self::label`] would produce, without
    /// building that `String` to find out.
    ///
    /// The verification pass compares a stored `stable_id` against a freshly
    /// formatted label once per alternative, and a sampling profile charged
    /// 2.14% of the compile's active self time to `label`. Only the comparison
    /// is avoidable — the `stable_id` a plan carries still has to be built. The
    /// digest itself is folded once at construction; see [`Self::digest`].
    ///
    /// **Exactly equivalent to `label == self.label()`, not a looser test.**
    /// `{:016x}` over a `u64` emits exactly sixteen lowercase hex digits, so
    /// requiring that prefix, that length, that alphabet, and that value admits
    /// precisely the one string `label` would have returned. The alphabet check
    /// is the load-bearing one: `u64::from_str_radix` also accepts uppercase,
    /// and without it a tampered `stable_id` spelled `…:ABC…` would compare
    /// equal here while the `String` comparison it replaces rejected it, which
    /// would weaken a tamper check in exchange for the allocation.
    pub(crate) fn is_labelled(&self, label: &str) -> bool {
        let Some(hex) = label.strip_prefix("selected-plan:") else {
            return false;
        };
        hex.len() == 16
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            && u64::from_str_radix(hex, 16).is_ok_and(|value| value == self.digest)
    }
}

/// One complete, checked physical plan: a legal cover joined with one compatible
/// implementation per region.
///
/// Every cover region is implemented exactly once, every materialization edge's
/// boundary contracts agree, the guards are the aggregate feasibility evidence the
/// plan rests on, and the cost is the exact aggregate structural cost. This is a
/// selection-level receipt over verified scheduled regions; it is not a
/// `KernelProgram`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectedPlan {
    cover: RegionCover,
    selections: Vec<RegionSelection>,
    handoffs: Vec<SatisfiedHandoff>,
    guards: Vec<ResolvedPredicate>,
    /// The honoured numerical dimensions the plan rests on, deduplicated and in
    /// canonical order.
    ///
    /// Retained beside the capability guards rather than folded into them
    /// because the *means* is part of the guard: two plans that honour one
    /// dimension by different means emit different operations, so collapsing
    /// them would give two different programs one plan identity.
    honoured: Vec<HonouredDimension>,
    cost: PlanStructuralCost,
    identity: SelectedPlanIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl SelectedPlan {
    /// Returns the legal complete cover this plan implements.
    pub(crate) const fn cover(&self) -> &RegionCover {
        &self.cover
    }

    /// Returns the per-region selections in canonical occurrence order.
    pub(crate) fn selections(&self) -> &[RegionSelection] {
        &self.selections
    }

    /// Returns the satisfied cross-region handoffs in canonical order.
    pub(crate) fn handoffs(&self) -> &[SatisfiedHandoff] {
        &self.handoffs
    }

    /// Returns the aggregate feasibility guards the plan rests on, canonical.
    pub(crate) fn guards(&self) -> &[ResolvedPredicate] {
        &self.guards
    }

    /// Returns the aggregate honoured numerical dimensions, canonical order.
    pub(crate) fn honoured(&self) -> &[HonouredDimension] {
        &self.honoured
    }

    /// Returns the exact aggregate structural cost of the plan.
    pub(crate) const fn cost(&self) -> PlanStructuralCost {
        self.cost
    }

    /// Returns the canonical, order-independent plan identity.
    pub(crate) const fn identity(&self) -> &SelectedPlanIdentity {
        &self.identity
    }
}

/// Collision-free, order-independent identity of one selected portfolio.
///
/// It folds the retained complete plans' identities in canonical order. Rejections
/// and budget stops are diagnostics, not part of the re-derivable core, so they are
/// deliberately absent — mirroring how cover and frontier identities exclude budget
/// stops and rejections.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SelectedPortfolioIdentity(Vec<u8>);

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl SelectedPortfolioIdentity {
    /// Returns the canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns a bounded explain label for this portfolio.
    ///
    /// The label is a digest of the canonical bytes and is presentation only.
    /// Equality decisions always use [`Self::as_bytes`].
    pub(crate) fn label(&self) -> String {
        format!("selected-portfolio:{:016x}", digest(&self.0))
    }
}

/// The deterministic result of selecting complete physical plans once.
///
/// The retained plans are every valid complete plan, in canonical identity order —
/// validity, never cost, decides retention. [`Self::non_dominated`] is a pure
/// structural view over them. An empty plan set with non-empty rejections is a
/// legitimate no-plan result (no cover joined with a compatible implementation set),
/// distinct from a [`SelectionError`] compiler fault.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectedPortfolio {
    policy_key: &'static str,
    plans: Vec<SelectedPlan>,
    rejections: Vec<PlanRejection>,
    budget_stops: Vec<PlanBudgetStop>,
    identity: SelectedPortfolioIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl SelectedPortfolio {
    /// Returns the structural-Pareto selection policy the portfolio was built under.
    pub(crate) const fn policy_key(&self) -> &'static str {
        self.policy_key
    }

    /// Returns every retained valid complete plan in canonical identity order.
    pub(crate) fn plans(&self) -> &[SelectedPlan] {
        &self.plans
    }

    /// Returns every recorded plan rejection in canonical order.
    pub(crate) fn rejections(&self) -> &[PlanRejection] {
        &self.rejections
    }

    /// Returns every budget that stopped a plan-enumeration path.
    pub(crate) fn budget_stops(&self) -> &[PlanBudgetStop] {
        &self.budget_stops
    }

    /// Returns whether no complete plan was retained.
    ///
    /// An empty portfolio is a valid no-plan result, distinct from a malformed
    /// [`SelectionError`] fault.
    pub(crate) fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }

    /// Returns the canonical, order-independent portfolio identity.
    pub(crate) const fn identity(&self) -> &SelectedPortfolioIdentity {
        &self.identity
    }

    /// Returns the structurally non-dominated plans, in canonical order.
    ///
    /// A plan is retained unless another retained plan strictly dominates its
    /// structural cost. Domination runs strictly *after* validity retention and
    /// only ever removes a plan another plan beats on the exact structural
    /// dimensions; it never establishes or refutes validity or feasibility.
    pub(crate) fn non_dominated(&self) -> Vec<&SelectedPlan> {
        self.plans
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                !self.plans.iter().enumerate().any(|(other_index, other)| {
                    *index != other_index && other.cost.dominates(&candidate.cost)
                })
            })
            .map(|(_, candidate)| candidate)
            .collect()
    }
}

/// Why a candidate complete plan was not retained as valid.
///
/// These are legitimate dispositions, not compiler faults: a cover region with no
/// admitted implementation is [`Self::RegionUnimplemented`]; a combination whose
/// boundary contracts do not compose is [`Self::BoundaryDisagreement`]. Neither
/// fails the enumeration; malformed compiler output does, through [`SelectionError`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlanRejection {
    /// A cover region has no feasible implementation on this target, so no complete
    /// plan can cover it. This is the legitimate reason a cover contributes no plan.
    RegionUnimplemented {
        /// The region presentation role that had no implementation.
        role: &'static str,
        /// The cover identity whose region was unimplemented.
        cover: Vec<u8>,
    },
    /// A candidate combination's per-region boundary contracts do not compose.
    BoundaryDisagreement {
        /// The typed boundary disagreement.
        disagreement: BoundaryDisagreement,
        /// The cover identity the combination was drawn from.
        cover: Vec<u8>,
    },
}

impl PlanRejection {
    fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::RegionUnimplemented { role, cover } => {
                output.push(1);
                push_slice(output, role.as_bytes());
                push_slice(output, cover);
            }
            Self::BoundaryDisagreement {
                disagreement,
                cover,
            } => {
                output.push(2);
                disagreement.encode(output);
                push_slice(output, cover);
            }
        }
    }

    fn sort_key(&self) -> Vec<u8> {
        let mut key = Vec::new();
        self.encode(&mut key);
        key
    }
}

/// Why a candidate plan's per-region boundary contracts do not compose.
///
/// Each variant names the offending region occurrence in its canonical bytes, so
/// the disagreement is explainable and deterministic. A producer that does not
/// materialize the cross-region tensor the cover requires, a consumer that does not
/// require it, an undischarged handoff, a leaked intermediate, a dangling read, or
/// an ambiguous boundary each fails closed rather than being silently repaired.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryDisagreement {
    /// An edge's producer region does not guarantee the materialized tensor.
    ProducerGuaranteeMissing {
        /// The producing region occurrence in canonical bytes.
        region: Vec<u8>,
    },
    /// An edge's consumer region does not require the materialized tensor.
    ConsumerRequirementMissing {
        /// The consuming region occurrence in canonical bytes.
        region: Vec<u8>,
    },
    /// A producer guarantee does not discharge a consumer requirement.
    ///
    /// The typed [`UnsatisfiedProperty`] names the exact dimension, the value
    /// required, the value guaranteed if there was one, and whether the producer
    /// offered the wrong value or none at all. That distinction is what an
    /// enforcer needs: a wrong value may be converted, and silence cannot be.
    UndischargedHandoff {
        /// The producing region occurrence in canonical bytes.
        producer: Vec<u8>,
        /// The consuming region occurrence in canonical bytes.
        consumer: Vec<u8>,
        /// The first property, in canonical dimension order, that was not
        /// discharged.
        unsatisfied: UnsatisfiedProperty,
    },
    /// A region guarantees a cross-region tensor no materialization edge consumes.
    UnconsumedGuarantee {
        /// The producing region occurrence in canonical bytes.
        region: Vec<u8>,
    },
    /// A region requires a cross-region tensor no materialization edge produces.
    UnsatisfiedRequirement {
        /// The consuming region occurrence in canonical bytes.
        region: Vec<u8>,
    },
    /// A region carries more than one cross-region guarantee or requirement, so the
    /// coarse tensor-role handoff cannot be bound to an edge unambiguously.
    AmbiguousBoundary {
        /// The offending region occurrence in canonical bytes.
        region: Vec<u8>,
    },
}

impl BoundaryDisagreement {
    /// Returns the stable reason code of the disagreement.
    ///
    /// [`Self::UndischargedHandoff`] delegates to the property model's own reason
    /// code, so an explanation names *which* property was not supplied rather
    /// than only that the handoff failed.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::ProducerGuaranteeMissing { .. } => "producer-guarantee-missing",
            Self::ConsumerRequirementMissing { .. } => "consumer-requirement-missing",
            Self::UndischargedHandoff { unsatisfied, .. } => unsatisfied.reason().code(),
            Self::UnconsumedGuarantee { .. } => "unconsumed-guarantee",
            Self::UnsatisfiedRequirement { .. } => "unsatisfied-requirement",
            Self::AmbiguousBoundary { .. } => "ambiguous-boundary",
        }
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_slice(output, self.reason().as_bytes());
        match self {
            Self::ProducerGuaranteeMissing { region }
            | Self::ConsumerRequirementMissing { region }
            | Self::UnconsumedGuarantee { region }
            | Self::UnsatisfiedRequirement { region }
            | Self::AmbiguousBoundary { region } => push_slice(output, region),
            Self::UndischargedHandoff {
                producer,
                consumer,
                unsatisfied,
            } => {
                push_slice(output, producer);
                push_slice(output, consumer);
                unsatisfied.encode(output);
            }
        }
    }
}

impl fmt::Display for BoundaryDisagreement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndischargedHandoff { unsatisfied, .. } => {
                write!(formatter, "selection.boundary.{unsatisfied}")
            }
            Self::ProducerGuaranteeMissing { .. }
            | Self::ConsumerRequirementMissing { .. }
            | Self::UnconsumedGuarantee { .. }
            | Self::UnsatisfiedRequirement { .. }
            | Self::AmbiguousBoundary { .. } => {
                write!(formatter, "selection.boundary.{}", self.reason())
            }
        }
    }
}

/// A malformed-compiler-output fault during plan selection or verification.
///
/// This is invalid compiler output that fails closed — a cover that no longer
/// re-derives from the program, a supplied frontier that does not correspond to a
/// cover region, or a structural inconsistency in an assembled plan. It is
/// deliberately distinct from every valid disposition, including an empty
/// [`SelectedPortfolio`] and a recorded [`PlanRejection`]: a valid no-plan result
/// is a legitimate outcome, whereas a malformed input or plan is a bug that must
/// not be silently accepted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SelectionError {
    /// A supplied or embedded cover failed re-verification against the program.
    MalformedCover(CoverError),
    /// The one-to-one binding between cover regions and supplied per-region
    /// frontiers (or selections) is violated.
    FrontierBinding {
        /// A stable rule code.
        rule: &'static str,
    },
    /// A claimed-valid plan's boundary contracts do not actually compose.
    InvalidComposition(BoundaryDisagreement),
    /// An assembled or verified plan or portfolio carried structurally invalid state.
    Structure {
        /// A stable rule code.
        rule: &'static str,
    },
}

impl SelectionError {
    /// Returns the coarse class of the fault: `cover`, `binding`, `composition`,
    /// or `structure`.
    pub(crate) const fn class(&self) -> &'static str {
        match self {
            Self::MalformedCover(_) => "cover",
            Self::FrontierBinding { .. } => "binding",
            Self::InvalidComposition(_) => "composition",
            Self::Structure { .. } => "structure",
        }
    }

    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::MalformedCover(error) => error.reason(),
            Self::FrontierBinding { rule } | Self::Structure { rule } => rule,
            Self::InvalidComposition(disagreement) => disagreement.reason(),
        }
    }
}

impl fmt::Display for SelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedCover(error) => write!(formatter, "selection.cover: {error}"),
            Self::FrontierBinding { rule } => write!(formatter, "selection.binding.{rule}"),
            Self::InvalidComposition(disagreement) => {
                write!(formatter, "selection.composition: {disagreement}")
            }
            Self::Structure { rule } => write!(formatter, "selection.structure.{rule}"),
        }
    }
}

impl Error for SelectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MalformedCover(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoverError> for SelectionError {
    fn from(value: CoverError) -> Self {
        Self::MalformedCover(value)
    }
}

/// One cover region's already-enumerated implementation frontier.
///
/// The subject carries the region's presentation role and its exact semantic
/// members, so the join can bind an empty frontier to its cover region and can
/// cross-check that every admitted implementation covers exactly that region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionFrontier {
    subject: FrontierRegionSubject,
    frontier: ImplementationFrontier,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl RegionFrontier {
    /// Binds a subject to the frontier that was enumerated for it.
    pub(crate) const fn new(
        subject: FrontierRegionSubject,
        frontier: ImplementationFrontier,
    ) -> Self {
        Self { subject, frontier }
    }

    /// Returns the region subject the frontier is an authority for.
    pub(crate) const fn subject(&self) -> &FrontierRegionSubject {
        &self.subject
    }

    /// Returns the enumerated implementation frontier.
    pub(crate) const fn frontier(&self) -> &ImplementationFrontier {
        &self.frontier
    }
}

/// One legal complete cover paired with a per-region implementation frontier.
///
/// The regions must be a one-to-one correspondence with the cover's regions,
/// matched by semantic members; the join fails closed if the correspondence is
/// violated.
///
/// The cover is borrowed from the enumeration that produced it rather than
/// owned. Selection only ever reads it, and the enumeration outlives every
/// source built from it, so owning one meant deep-copying each cover's regions
/// and materialization edges for the duration of one call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverFrontiers<'cover> {
    cover: &'cover RegionCover,
    regions: Vec<RegionFrontier>,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl<'cover> CoverFrontiers<'cover> {
    /// Pairs a cover with one implementation frontier per region.
    pub(crate) const fn new(cover: &'cover RegionCover, regions: Vec<RegionFrontier>) -> Self {
        Self { cover, regions }
    }

    /// Returns the legal complete cover.
    pub(crate) const fn cover(&self) -> &RegionCover {
        self.cover
    }

    /// Returns the per-region frontiers.
    pub(crate) fn regions(&self) -> &[RegionFrontier] {
        &self.regions
    }
}

/// Selects the complete physical plans that join legal covers with compatible
/// per-region implementations.
///
/// Each source's cover is re-verified from the program, its regions are bound
/// one-to-one to the supplied frontiers by semantic members, and every complete
/// combination of one admitted implementation per region whose boundary contracts
/// compose becomes a retained [`SelectedPlan`]. Retention is by validity only; the
/// returned portfolio exposes a structural-Pareto view but never prunes a valid
/// plan from `plans()`. An `Ok` with an empty plan set is a valid no-plan result;
/// a cover region with no implementation is recorded as a
/// [`PlanRejection::RegionUnimplemented`], and a non-composing combination as a
/// [`PlanRejection::BoundaryDisagreement`].
///
/// # Errors
///
/// Returns a [`SelectionError`] on malformed compiler output: a cover that fails
/// re-verification, a frontier that does not correspond one-to-one to the cover's
/// regions, admitted implementations that bind to the wrong region or a
/// disagreeing target profile, or a structural inconsistency (a cost-model
/// disagreement or an aggregate overflow) while assembling a plan.
pub(crate) fn select_physical_plans(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    formation: &RegionFormationOutcome,
    sources: &[CoverFrontiers<'_>],
) -> Result<SelectedPortfolio, SelectionError> {
    let target_profile_key = coherent_target_profile(sources)?;
    let mut retained: BTreeMap<SelectedPlanIdentity, SelectedPlan> = BTreeMap::new();
    let mut rejections: BTreeMap<Vec<u8>, PlanRejection> = BTreeMap::new();
    let mut budget_stops: BTreeMap<Vec<u8>, PlanBudgetStop> = BTreeMap::new();

    for source in sources {
        verify_cover(program, formation, source.cover)?;
        let cover_identity = source.cover.identity().as_bytes();
        let region_impls =
            bind_region_frontiers(source.cover, &source.regions, target_profile_key)?;

        // A cover region with no admitted implementation cannot be completed.
        //
        // The rejection is keyed by role and cover, so several unimplemented
        // regions sharing a role within one cover describe one rejection and
        // `or_insert` kept the first. Recognising the repeat here drops it
        // before it costs a copy of the cover identity and an encoded sort key:
        // the governed program records 38 of these per compile, every one of
        // them the role `unrecognized`.
        let mut unimplemented = false;
        let mut rejected_roles: Vec<&'static str> = Vec::new();
        for entry in &region_impls {
            if entry.admitted.is_empty() {
                unimplemented = true;
                if !rejected_roles.contains(&entry.role) {
                    rejected_roles.push(entry.role);
                    let rejection = PlanRejection::RegionUnimplemented {
                        role: entry.role,
                        cover: cover_identity.to_vec(),
                    };
                    rejections.entry(rejection.sort_key()).or_insert(rejection);
                }
            }
        }
        if unimplemented {
            continue;
        }

        enumerate_cover_plans(
            source.cover,
            &region_impls,
            cover_identity,
            budgets.physical_plan_combinations,
            &mut retained,
            &mut rejections,
            &mut budget_stops,
        )?;
    }

    let mut plans: Vec<SelectedPlan> = retained.into_values().collect();
    plans.sort_by(|left, right| left.identity.as_bytes().cmp(right.identity.as_bytes()));
    let identity = encode_portfolio_identity(&plans);
    Ok(SelectedPortfolio {
        policy_key: SELECTION_POLICY_KEY,
        plans,
        rejections: rejections.into_values().collect(),
        budget_stops: budget_stops.into_values().collect(),
        identity,
    })
}

/// Re-derives and validates one selected complete physical plan.
///
/// The plan's cover is re-verified from the program, the plan is re-assembled from
/// its cover and selections, and the re-assembled plan must reproduce the receipt
/// exactly. Any deviation — a foreign cover, an implementation bound to the wrong
/// region, boundary contracts that do not actually compose, or a tampered
/// identity, guard set, or cost — fails closed with a typed [`SelectionError`].
///
/// # Errors
///
/// Returns a [`SelectionError`] whose [`SelectionError::class`] is `cover` for a
/// cover that no longer re-derives, `binding` for a mis-bound implementation,
/// `composition` for boundary contracts that do not compose, and `structure` for a
/// receipt whose re-derivation does not reproduce it.
pub(crate) fn verify_selected_plan(
    program: &SemanticProgram,
    formation: &RegionFormationOutcome,
    plan: &SelectedPlan,
) -> Result<(), SelectionError> {
    verify_cover(program, formation, &plan.cover)?;
    let recomputed =
        assemble_plan(&plan.cover, plan.selections.clone()).map_err(PlanFault::into_error)?;
    if recomputed != *plan {
        return Err(SelectionError::Structure {
            rule: "plan-mismatch",
        });
    }
    Ok(())
}

/// Re-derives and validates one selected portfolio.
///
/// Every plan is verified, the plans must be in canonical identity order with
/// distinct identities, and the portfolio identity must recompute exactly.
///
/// # Errors
///
/// Returns a [`SelectionError`] when any plan fails re-derivation, the plans are
/// not in canonical order or are not distinct, or the portfolio identity does not
/// reproduce.
pub(crate) fn verify_selected_portfolio(
    program: &SemanticProgram,
    formation: &RegionFormationOutcome,
    portfolio: &SelectedPortfolio,
) -> Result<(), SelectionError> {
    if portfolio.policy_key != SELECTION_POLICY_KEY {
        return Err(SelectionError::Structure {
            rule: "portfolio-policy",
        });
    }
    for plan in &portfolio.plans {
        verify_selected_plan(program, formation, plan)?;
    }
    for pair in portfolio.plans.windows(2) {
        if pair[0].identity.as_bytes() >= pair[1].identity.as_bytes() {
            return Err(SelectionError::Structure {
                rule: "portfolio-order",
            });
        }
    }
    let identity = encode_portfolio_identity(&portfolio.plans);
    if identity != portfolio.identity {
        return Err(SelectionError::Structure {
            rule: "portfolio-identity-mismatch",
        });
    }
    Ok(())
}

/// The admitted implementations of one cover region, with its subject role.
struct RegionFrontierBinding<'a> {
    role: &'static str,
    admitted: &'a [AdmittedImplementation],
}

/// Requires every supplied frontier to assess one target profile and returns it.
///
/// A portfolio is a per-target artifact, so covers whose frontiers disagree on the
/// assessed target profile cannot be joined into one portfolio.
fn coherent_target_profile(
    sources: &[CoverFrontiers<'_>],
) -> Result<Option<&'static str>, SelectionError> {
    let mut target: Option<&'static str> = None;
    for source in sources {
        for region in &source.regions {
            let key = region.frontier.target_profile_key();
            match target {
                None => target = Some(key),
                Some(existing) if existing == key => {}
                Some(_) => {
                    return Err(SelectionError::Structure {
                        rule: "target-profile",
                    });
                }
            }
        }
    }
    Ok(target)
}

/// Binds each cover region to its supplied frontier by semantic members.
fn bind_region_frontiers<'a>(
    cover: &RegionCover,
    regions: &'a [RegionFrontier],
    target_profile_key: Option<&'static str>,
) -> Result<Vec<RegionFrontierBinding<'a>>, SelectionError> {
    if regions.len() != cover.regions().len() {
        return Err(SelectionError::FrontierBinding {
            rule: "region-frontier-count",
        });
    }
    let mut by_members: BTreeMap<Vec<u32>, &RegionFrontier> = BTreeMap::new();
    for region in regions {
        let key = member_key(region.subject.semantic_members());
        if by_members.insert(key, region).is_some() {
            return Err(SelectionError::FrontierBinding {
                rule: "duplicate-region-frontier",
            });
        }
    }
    let mut bound = Vec::with_capacity(cover.regions().len());
    for region in cover.regions() {
        let entry = by_members.get(&member_key(region.members())).ok_or(
            SelectionError::FrontierBinding {
                rule: "region-frontier-missing",
            },
        )?;
        // Every admitted implementation must cover exactly this region and target.
        for admitted in entry.frontier.admitted() {
            if member_key(admitted.semantic_members()) != member_key(region.members()) {
                return Err(SelectionError::FrontierBinding {
                    rule: "frontier-region-members",
                });
            }
            if target_profile_key.is_some_and(|key| key != admitted.target_profile_key()) {
                return Err(SelectionError::FrontierBinding {
                    rule: "frontier-region-target",
                });
            }
        }
        bound.push(RegionFrontierBinding {
            role: entry.subject.role(),
            admitted: entry.frontier.admitted(),
        });
    }
    Ok(bound)
}

/// Enumerates every complete-plan combination for one bound cover.
#[allow(
    clippy::too_many_arguments,
    reason = "the deterministic enumerator threads the cover, its bound frontiers, and every accumulator explicitly rather than hiding them behind shared mutable state"
)]
fn enumerate_cover_plans(
    cover: &RegionCover,
    region_impls: &[RegionFrontierBinding<'_>],
    cover_identity: &[u8],
    max_combinations: u64,
    retained: &mut BTreeMap<SelectedPlanIdentity, SelectedPlan>,
    rejections: &mut BTreeMap<Vec<u8>, PlanRejection>,
    budget_stops: &mut BTreeMap<Vec<u8>, PlanBudgetStop>,
) -> Result<(), SelectionError> {
    let counts: Vec<usize> = region_impls
        .iter()
        .map(|entry| entry.admitted.len())
        .collect();
    if counts.contains(&0) {
        return Ok(());
    }
    let mut indices = vec![0_usize; counts.len()];
    let mut produced = 0_u64;
    loop {
        if produced >= max_combinations {
            let stop = PlanBudgetStop {
                resource: PlanBudgetResource::Combinations,
                limit: max_combinations,
                actual: max_combinations.saturating_add(1),
                cover: cover_identity.to_vec(),
            };
            budget_stops.entry(cover_identity.to_vec()).or_insert(stop);
            break;
        }
        let mut selections = Vec::with_capacity(region_impls.len());
        for (region, (entry, index)) in cover
            .regions()
            .iter()
            .zip(region_impls.iter().zip(&indices))
        {
            selections.push(RegionSelection {
                occurrence: region.occurrence().clone(),
                implementation: entry.admitted[*index].clone(),
            });
        }
        match assemble_plan(cover, selections) {
            Ok(plan) => {
                retained.entry(plan.identity.clone()).or_insert(plan);
            }
            Err(PlanFault::Disagreement(disagreement)) => {
                let rejection = PlanRejection::BoundaryDisagreement {
                    disagreement,
                    cover: cover_identity.to_vec(),
                };
                rejections.entry(rejection.sort_key()).or_insert(rejection);
            }
            Err(fault) => return Err(fault.into_error()),
        }
        produced = produced.saturating_add(1);
        if !advance(&mut indices, &counts) {
            break;
        }
    }
    Ok(())
}

/// Advances the odometer over per-region implementation indices.
///
/// Returns whether a next combination exists.
fn advance(indices: &mut [usize], counts: &[usize]) -> bool {
    for position in (0..indices.len()).rev() {
        indices[position] += 1;
        if indices[position] < counts[position] {
            return true;
        }
        indices[position] = 0;
    }
    false
}

/// A fault while assembling one complete plan from a cover and its selections.
enum PlanFault {
    /// The selections' boundary contracts do not compose — a legitimate rejection.
    Disagreement(BoundaryDisagreement),
    /// The selections do not bind one-to-one to the cover's regions.
    Binding(&'static str),
    /// The assembled plan carried structurally invalid state.
    Structure(&'static str),
}

impl PlanFault {
    /// Maps an assembly fault onto the public selection-error contract.
    ///
    /// A boundary disagreement observed while *verifying* a claimed-valid plan is a
    /// composition fault, not a legitimate rejection, because a retained plan must
    /// compose.
    fn into_error(self) -> SelectionError {
        match self {
            Self::Disagreement(disagreement) => SelectionError::InvalidComposition(disagreement),
            Self::Binding(rule) => SelectionError::FrontierBinding { rule },
            Self::Structure(rule) => SelectionError::Structure { rule },
        }
    }
}

/// Assembles one complete plan from a cover and one selection per region.
///
/// It binds the selections one-to-one to the cover's regions, reconciles the
/// boundary contracts across every materialization edge, aggregates the guards and
/// the structural cost, and folds the canonical plan identity. It is the single
/// derivation both selection and verification use, so a verified plan is exactly a
/// re-selectable plan.
fn assemble_plan(
    cover: &RegionCover,
    selections: Vec<RegionSelection>,
) -> Result<SelectedPlan, PlanFault> {
    if selections.is_empty() || selections.len() != cover.regions().len() {
        return Err(PlanFault::Binding("selection-count"));
    }

    // One-to-one binding: each selection covers exactly one cover region and each
    // region is selected exactly once.
    let mut by_occurrence: BTreeMap<Vec<u8>, &RegionSelection> = BTreeMap::new();
    for selection in &selections {
        if by_occurrence
            .insert(selection.occurrence.as_bytes().to_vec(), selection)
            .is_some()
        {
            return Err(PlanFault::Binding("duplicate-selection"));
        }
    }
    if by_occurrence.len() != cover.regions().len() {
        return Err(PlanFault::Binding("selection-count"));
    }
    let mut boundaries: BTreeMap<Vec<u8>, RegionBoundary> = BTreeMap::new();
    for region in cover.regions() {
        let selection = by_occurrence
            .get(region.occurrence().as_bytes())
            .ok_or(PlanFault::Binding("region-unselected"))?;
        if member_key(selection.implementation.semantic_members()) != member_key(region.members()) {
            return Err(PlanFault::Binding("member-mismatch"));
        }
        boundaries.insert(
            region.occurrence().as_bytes().to_vec(),
            region_boundary(&selection.implementation),
        );
    }

    let handoffs = reconcile_boundaries(cover, &boundaries).map_err(|outcome| match outcome {
        BoundaryOutcome::Disagreement(disagreement) => PlanFault::Disagreement(disagreement),
        BoundaryOutcome::Structure(rule) => PlanFault::Structure(rule),
    })?;

    let cost = aggregate_cost(cover, &selections)?;
    let guards = aggregate_guards(&selections);
    let honoured = aggregate_honoured(&selections);

    let mut ordered = selections;
    ordered.sort_by(|left, right| left.occurrence.as_bytes().cmp(right.occurrence.as_bytes()));
    let identity = encode_plan_identity(cover, &ordered, &handoffs, &guards, &honoured, cost);
    Ok(SelectedPlan {
        cover: cover.clone(),
        selections: ordered,
        handoffs,
        guards,
        honoured,
        cost,
        identity,
    })
}

/// A region's cross-region boundary facet within a candidate plan.
///
/// It is derived from the region's selected implementation boundary contract; the
/// join reconciles these facets against the cover's materialization edges.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RegionBoundary {
    guarantees: Vec<GuaranteeFacet>,
    requirements: Vec<RequirementFacet>,
}

/// One guarantee a region's selected implementation makes at its boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
struct GuaranteeFacet {
    tensor: TensorRole,
    ownership: BoundaryOwnership,
    properties: GuaranteedProperties,
}

/// One requirement a region's selected implementation places at its boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RequirementFacet {
    tensor: TensorRole,
    access: AccessMode,
    properties: RequiredProperties,
}

/// Derives a region's boundary facet from its selected implementation contract.
fn region_boundary(implementation: &AdmittedImplementation) -> RegionBoundary {
    let contract = implementation.boundary();
    RegionBoundary {
        guarantees: contract
            .guarantees()
            .iter()
            .map(|guarantee| GuaranteeFacet {
                tensor: guarantee.tensor(),
                ownership: guarantee.ownership(),
                properties: guarantee.properties().clone(),
            })
            .collect(),
        requirements: contract
            .requirements()
            .iter()
            .map(|requirement| RequirementFacet {
                tensor: requirement.tensor(),
                access: requirement.access(),
                properties: requirement.properties().clone(),
            })
            .collect(),
    }
}

/// Why boundary reconciliation could not produce a satisfied handoff set.
enum BoundaryOutcome {
    /// The boundary contracts do not compose — a legitimate disagreement.
    Disagreement(BoundaryDisagreement),
    /// A structural inconsistency (a cover edge names an unbound region).
    Structure(&'static str),
}

/// Reconciles the cover's materialization edges with the regions' boundary facets.
///
/// A cross-region materialized value is a [`TensorRole::Intermediate`] handoff. The
/// bounded profile schedules at most one intermediate per region boundary, so the
/// reconciliation binds each edge to a region's unique intermediate guarantee and
/// requirement, failing closed on ambiguity (more than one), on a leaked
/// intermediate (a guarantee no edge consumes), on a dangling read (a requirement
/// no edge produces), or on an undischarged handoff.
fn reconcile_boundaries(
    cover: &RegionCover,
    boundaries: &BTreeMap<Vec<u8>, RegionBoundary>,
) -> Result<Vec<SatisfiedHandoff>, BoundaryOutcome> {
    let mut produced: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
    let mut consumed: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
    for edge in cover.materializations() {
        *produced
            .entry(edge.producer().as_bytes().to_vec())
            .or_default() += 1;
        for consumer in edge.consumers() {
            *consumed.entry(consumer.as_bytes().to_vec()).or_default() += 1;
        }
    }

    // Per-region closure: the intermediate guarantees/requirements a region carries
    // must exactly match the edges it produces/consumes, without ambiguity.
    for (occurrence, boundary) in boundaries {
        let guarantees = intermediate_count(boundary.guarantees.iter().map(|facet| facet.tensor));
        let requirements =
            intermediate_count(boundary.requirements.iter().map(|facet| facet.tensor));
        if guarantees > 1 || requirements > 1 {
            return Err(BoundaryOutcome::Disagreement(
                BoundaryDisagreement::AmbiguousBoundary {
                    region: occurrence.clone(),
                },
            ));
        }
        let produced_edges = produced.get(occurrence).copied().unwrap_or(0);
        let consumed_edges = consumed.get(occurrence).copied().unwrap_or(0);
        if guarantees != produced_edges {
            return Err(BoundaryOutcome::Disagreement(
                if guarantees < produced_edges {
                    BoundaryDisagreement::ProducerGuaranteeMissing {
                        region: occurrence.clone(),
                    }
                } else {
                    BoundaryDisagreement::UnconsumedGuarantee {
                        region: occurrence.clone(),
                    }
                },
            ));
        }
        if requirements != consumed_edges {
            return Err(BoundaryOutcome::Disagreement(
                if requirements < consumed_edges {
                    BoundaryDisagreement::ConsumerRequirementMissing {
                        region: occurrence.clone(),
                    }
                } else {
                    BoundaryDisagreement::UnsatisfiedRequirement {
                        region: occurrence.clone(),
                    }
                },
            ));
        }
    }

    let mut handoffs = Vec::with_capacity(cover.materializations().len());
    for edge in cover.materializations() {
        handoffs.push(satisfy_edge(edge, boundaries)?);
    }
    handoffs.sort_by_key(SatisfiedHandoff::sort_key);
    Ok(handoffs)
}

/// Binds one materialization edge to the producer guarantee and consumer
/// requirements that discharge it.
fn satisfy_edge(
    edge: &MaterializationEdge,
    boundaries: &BTreeMap<Vec<u8>, RegionBoundary>,
) -> Result<SatisfiedHandoff, BoundaryOutcome> {
    let producer_bytes = edge.producer().as_bytes().to_vec();
    let producer_boundary = boundaries
        .get(&producer_bytes)
        .ok_or(BoundaryOutcome::Structure("edge-producer-unbound"))?;
    let guarantee = producer_boundary
        .guarantees
        .iter()
        .find(|facet| facet.tensor == TensorRole::Intermediate)
        .ok_or(BoundaryOutcome::Disagreement(
            BoundaryDisagreement::ProducerGuaranteeMissing {
                region: producer_bytes.clone(),
            },
        ))?;

    let mut access = None;
    for consumer in edge.consumers() {
        let consumer_bytes = consumer.as_bytes().to_vec();
        let consumer_boundary = boundaries
            .get(&consumer_bytes)
            .ok_or(BoundaryOutcome::Structure("edge-consumer-unbound"))?;
        let requirement = consumer_boundary
            .requirements
            .iter()
            .find(|facet| facet.tensor == TensorRole::Intermediate)
            .ok_or(BoundaryOutcome::Disagreement(
                BoundaryDisagreement::ConsumerRequirementMissing {
                    region: consumer_bytes.clone(),
                },
            ))?;
        // Every typed property the consumer requires must be discharged by the
        // producer's guarantee. The first unsatisfied one, in canonical dimension
        // order, is what the disagreement names; the rest are recoverable by
        // re-running the relation on the same pair, so a plan rejection carries
        // one actionable reason rather than a list.
        if let Some(unsatisfied) =
            unsatisfied_properties(&requirement.properties, &guarantee.properties)
                .into_iter()
                .next()
        {
            return Err(BoundaryOutcome::Disagreement(
                BoundaryDisagreement::UndischargedHandoff {
                    producer: producer_bytes.clone(),
                    consumer: consumer_bytes,
                    unsatisfied,
                },
            ));
        }
        access = Some(requirement.access);
    }
    let access = access.ok_or(BoundaryOutcome::Structure("edge-without-consumer"))?;

    Ok(SatisfiedHandoff {
        producer_position: edge.producer_position(),
        result_position: edge.result_position(),
        producer: edge.producer().clone(),
        consumers: edge.consumers().to_vec(),
        role: TensorRole::Intermediate,
        ownership: guarantee.ownership,
        access,
    })
}

/// Counts how many facets carry the intermediate cross-region role.
fn intermediate_count(roles: impl Iterator<Item = TensorRole>) -> u64 {
    count(
        roles
            .filter(|role| *role == TensorRole::Intermediate)
            .count(),
    )
}

/// Aggregates the exact structural cost of one complete plan.
fn aggregate_cost(
    cover: &RegionCover,
    selections: &[RegionSelection],
) -> Result<PlanStructuralCost, PlanFault> {
    let mut model_key: Option<&'static str> = None;
    let mut dispatch_count = 0_u64;
    let mut launched_threads = 0_u64;
    let mut temporary_bytes = 0_u64;
    for selection in selections {
        let cost = selection.implementation.cost();
        match model_key {
            None => model_key = Some(cost.model_key()),
            Some(existing) if existing == cost.model_key() => {}
            Some(_) => return Err(PlanFault::Structure("cost-model")),
        }
        dispatch_count = dispatch_count
            .checked_add(u64::from(cost.dispatch_count()))
            .ok_or(PlanFault::Structure("cost-overflow"))?;
        launched_threads = launched_threads
            .checked_add(cost.launched_threads())
            .ok_or(PlanFault::Structure("cost-overflow"))?;
        temporary_bytes = temporary_bytes
            .checked_add(cost.temporary_bytes())
            .ok_or(PlanFault::Structure("cost-overflow"))?;
    }
    let model_key = model_key.ok_or(PlanFault::Structure("empty-plan"))?;
    Ok(PlanStructuralCost {
        model_key,
        dispatch_count,
        launched_threads,
        temporary_bytes,
        materialization_count: count(cover.materializations().len()),
    })
}

/// Aggregates the deduplicated capability guards of one complete plan.
fn aggregate_guards(selections: &[RegionSelection]) -> Vec<ResolvedPredicate> {
    let mut guards: Vec<ResolvedPredicate> = selections
        .iter()
        .flat_map(|selection| {
            selection
                .implementation
                .feasibility()
                .predicates()
                .iter()
                .copied()
        })
        .collect();
    guards.sort_by(|left, right| {
        (
            left.axis(),
            left.required().value(),
            left.available().value(),
        )
            .cmp(&(
                right.axis(),
                right.required().value(),
                right.available().value(),
            ))
    });
    guards.dedup();
    guards
}

/// Aggregates the deduplicated honoured numerical dimensions of one plan.
///
/// Sorted by dimension, then behaviour, then means, then declaring profile —
/// every field, because two entries agreeing on a dimension and disagreeing on
/// how it was honoured are two different claims and must both survive.
fn aggregate_honoured(selections: &[RegionSelection]) -> Vec<HonouredDimension> {
    let mut honoured: Vec<HonouredDimension> = selections
        .iter()
        .flat_map(|selection| {
            selection
                .implementation
                .feasibility()
                .honoured()
                .iter()
                .copied()
        })
        .collect();
    honoured.sort_by_key(|entry| {
        (
            entry.dimension(),
            entry.arithmetic(),
            entry.behaviour().canonical_key(),
            entry.means().key(),
            entry.profile().key(),
        )
    });
    honoured.dedup();
    honoured
}

fn encode_plan_identity(
    cover: &RegionCover,
    selections: &[RegionSelection],
    handoffs: &[SatisfiedHandoff],
    guards: &[ResolvedPredicate],
    honoured: &[HonouredDimension],
    cost: PlanStructuralCost,
) -> SelectedPlanIdentity {
    let mut bytes = SELECTED_PLAN_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, cover.identity().as_bytes());
    push_len(&mut bytes, selections.len());
    for selection in selections {
        push_slice(&mut bytes, selection.occurrence.as_bytes());
        push_slice(&mut bytes, selection.implementation.identity().as_bytes());
    }
    push_len(&mut bytes, handoffs.len());
    for handoff in handoffs {
        handoff.encode(&mut bytes);
    }
    push_len(&mut bytes, guards.len());
    for guard in guards {
        encode_guard(&mut bytes, *guard);
    }
    push_len(&mut bytes, honoured.len());
    for entry in honoured {
        encode_honoured(&mut bytes, *entry);
    }
    cost.encode(&mut bytes);
    SelectedPlanIdentity {
        digest: digest(&bytes),
        bytes,
    }
}

fn encode_portfolio_identity(plans: &[SelectedPlan]) -> SelectedPortfolioIdentity {
    let mut bytes = SELECTED_PORTFOLIO_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, SELECTION_POLICY_KEY.as_bytes());
    push_len(&mut bytes, plans.len());
    for plan in plans {
        push_slice(&mut bytes, plan.identity.as_bytes());
    }
    SelectedPortfolioIdentity(bytes)
}

fn encode_guard(output: &mut Vec<u8>, guard: ResolvedPredicate) {
    push_slice(output, guard.axis().key().as_bytes());
    output.extend_from_slice(&guard.required().value().to_be_bytes());
    output.extend_from_slice(&guard.available().value().to_be_bytes());
}

/// Encodes one honoured numerical dimension into a plan identity.
///
/// The means and the declaring profile are both encoded. Two plans that honour
/// one behaviour natively and by emulation emit different operations, and two
/// that rely on declarations from different profiles rest on different evidence;
/// either omission would give distinguishable plans one identity.
fn encode_honoured(output: &mut Vec<u8>, honoured: HonouredDimension) {
    push_slice(output, honoured.dimension().key().as_bytes());
    output.push(honoured.arithmetic().tag());
    honoured.behaviour().encode(output);
    push_slice(output, honoured.means().key().as_bytes());
    push_slice(output, honoured.profile().key().as_bytes());
}

fn member_key(members: &[crate::region::SemanticMemberId]) -> Vec<u32> {
    let mut ordinals: Vec<u32> = members.iter().map(|member| member.0).collect();
    ordinals.sort_unstable();
    ordinals.dedup();
    ordinals
}

const fn tensor_role_tag(role: TensorRole) -> u8 {
    match role {
        TensorRole::Input => 1,
        TensorRole::Intermediate => 2,
        TensorRole::Output => 3,
    }
}

const fn ownership_tag(ownership: BoundaryOwnership) -> u8 {
    match ownership {
        BoundaryOwnership::TotalRaceFreeWrite => 1,
    }
}

/// The governed tag naming an access mode in a canonical encoding.
///
/// A second out-of-crate total map over `AccessMode`, alongside
/// `crate::frontier`'s; both are ADR 0074 convention 5b sites and neither may
/// carry a wildcard.
const fn access_mode_tag(mode: AccessMode) -> u8 {
    match mode {
        AccessMode::Read => 1,
        AccessMode::Write => 2,
    }
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
/// Builds the mixed scheduled/opaque portfolio shared by downstream tests.
pub(crate) fn opaque_fused_portfolio_fixture(program: &SemanticProgram) -> SelectedPortfolio {
    tests::opaque_fused_portfolio(program)
}

#[cfg(test)]
mod tests {

    /// The formation these cases run under, derived once per call site.
    ///
    /// The selection entry points take it rather than deriving it, so a test
    /// supplies the same value the compile path threads in.
    fn formation_of(program: &SemanticProgram) -> crate::region::RegionFormationOutcome {
        crate::region::form_region_candidates(program, budgets(), contract())
            .expect("the fixture forms regions")
    }
    use super::{
        AccessMode, BoundaryDisagreement, CoverFrontiers, GuaranteeFacet, GuaranteedProperties,
        PlanRejection, RegionBoundary, RegionFrontier, RequiredProperties, RequirementFacet,
        SelectionError, TensorRole, reconcile_boundaries, select_physical_plans,
        verify_selected_plan, verify_selected_portfolio,
    };
    use crate::boundary::{
        AdmittedMemoryDomains, AvailabilityGuarantee, AvailabilityRequirement, BoundaryProperty,
        ByteAlignment, ExecutionAffinity, GuaranteedProperty, LayoutGuarantee, LayoutRequirement,
        MaterializationForm, MemoryDomainClass, RequiredProperty, StorageEncoding,
        UnsatisfiedReason, VisibilityGuarantee, VisibilityRequirement,
    };
    use crate::cover::{RegionCover, enumerate_covers};
    use crate::frontier::{
        BoundaryOwnership, FrontierRegionSubject, ImplementationContext, ImplementationProposal,
        PhysicalCostEstimate, PhysicalImplementationProvider, PhysicalProviderProvenance,
        PhysicalProviderProvenanceError, ProposalBody, TargetApplicability, enumerate_frontier,
    };
    use crate::physical::{ScheduledRegion, build_fused_scheduled_region, build_scheduled_regions};
    use crate::request::{
        CompilationRequest, DeterministicBudgets, StrictF32NumericalContract,
        VerifiedTargetRequest, verify_request,
    };
    use std::collections::BTreeMap;
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ProviderIdentity,
        SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    const GOVERNED_TARGET_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";

    /// The governed serial-sum chain: two pointwise constants, multiply, add, sum.
    fn serial_sum_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn request_for(program: &SemanticProgram) -> VerifiedTargetRequest {
        let request = verify_request(CompilationRequest::governed(program)).unwrap();
        request.for_target(request.target_profiles()[0]).unwrap()
    }

    fn provider_identity(name: &str, revision: u32) -> ProviderIdentity {
        ProviderIdentity::new("tiler.test.selection", name, revision).unwrap()
    }

    fn governed_applicability() -> TargetApplicability {
        TargetApplicability::for_targets([crate::request::TargetProfileKey::governed(
            GOVERNED_TARGET_KEY,
        )])
    }

    /// A provider that proposes one checked scheduled-kernel body built from a
    /// closure, with a caller-chosen provider identity and cost estimate.
    struct FixedRegionProvider {
        provider: ProviderIdentity,
        cost: PhysicalCostEstimate,
        region: ScheduledRegion,
    }

    struct FixedCallProvider {
        identity: crate::call_registry::OpaqueCallIdentity,
        bindings: Vec<(&'static str, TensorRole)>,
        cost: PhysicalCostEstimate,
    }

    impl PhysicalImplementationProvider for FixedCallProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(provider_identity("opaque", 1))
        }

        fn propose(&self, _: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
            vec![ImplementationProposal::new(
                ProposalBody::OpaqueCall(Box::new(
                    crate::call_registry::OpaqueCallProposal::new(
                        self.identity,
                        self.bindings.clone(),
                    )
                    .expect("fixture proposal is exactly reportable"),
                )),
                governed_applicability(),
                self.cost,
            )]
        }
    }

    fn opaque_declaration(
        aliasing: crate::effects::Aliasing,
    ) -> crate::call_declaration::OpaqueCallDeclaration {
        use crate::boundary::{
            AdmittedMemoryDomains, ExecutionAffinity, LayoutGuarantee, LayoutRequirement,
            MemoryDomainClass,
        };
        use crate::call_abi::{CallAbi, ParameterLayout, ParameterRole, ParameterSpec};
        use crate::call_declaration::{OpaqueCallDeclaration, WorkScaling};
        use crate::call_placement::CallPlacement;
        use crate::effects::{CallEffects, Elimination, Motion};

        let spec = |name, role| ParameterSpec {
            name,
            role,
            layout: match role {
                ParameterRole::In => ParameterLayout::Required(LayoutRequirement::DenseRowMajor),
                ParameterRole::Out => ParameterLayout::Guaranteed(LayoutGuarantee::DenseRowMajor),
                ParameterRole::InOut => unreachable!("the fixture has separate input and output"),
            },
            encoding: StorageEncoding::Unpacked,
            alignment: ByteAlignment::F32_NATURAL,
        };
        let realization = StrictF32NumericalContract::governed().realization();
        OpaqueCallDeclaration::check(
            CallAbi::declare([spec("x", ParameterRole::In), spec("y", ParameterRole::Out)])
                .expect("well-formed fixture ABI"),
            CallEffects::declared(Elimination::Required, Motion::Ordered, aliasing),
            CallPlacement::declare(
                ExecutionAffinity::PRIMARY,
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).expect("one domain"),
                &[MemoryDomainClass::Device],
            )
            .expect("coherent placement"),
            tiler_ir::schedule::ResourceRequirements {
                buffer_bindings: 2,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                barriers: 0,
                requires_device_memory: true,
                input_subnormals: realization.input_subnormals,
                result_subnormals: realization.result_subnormals,
                contraction: realization.contraction,
                reassociation: realization.reassociation,
                permutation: realization.permutation,
                signed_zero: realization.signed_zero,
                nan_assumptions: realization.nan_assumptions,
                infinity_assumptions: realization.infinity_assumptions,
            },
            WorkScaling::Fixed(2),
        )
        .expect("coherent opaque-call declaration")
    }

    fn frontier_with_opaque(
        request: &VerifiedTargetRequest,
        subject: FrontierRegionSubject,
        call_name: &'static str,
        output: TensorRole,
        aliasing: crate::effects::Aliasing,
        scheduled: Option<ScheduledRegion>,
    ) -> RegionFrontier {
        let identity =
            crate::call_registry::OpaqueCallIdentity::new("test", call_name, 1).expect("named");
        let mut registry = crate::call_registry::OpaqueCallRegistry::new();
        registry
            .register(identity, opaque_declaration(aliasing))
            .expect("one call");
        let opaque = FixedCallProvider {
            identity,
            bindings: vec![("x", TensorRole::Input), ("y", output)],
            cost: PhysicalCostEstimate::structural(1, 2, 0),
        };
        let scheduled = scheduled.map(|region| FixedRegionProvider {
            provider: provider_identity("scheduled", 1),
            cost: PhysicalCostEstimate::structural(1, 2, 0),
            region,
        });
        let mut providers: Vec<&dyn PhysicalImplementationProvider> = Vec::new();
        if let Some(host) = &scheduled {
            providers.push(host);
        }
        providers.push(&opaque);
        let frontier = enumerate_frontier(request, &subject, &providers, &registry).unwrap();
        RegionFrontier::new(subject, frontier)
    }

    /// Builds the one-region portfolio used to exercise consumers below the
    /// frontier. Its two plans differ only in choosing a scheduled or opaque
    /// implementation for the same cover region.
    pub(super) fn opaque_fused_portfolio(program: &SemanticProgram) -> super::SelectedPortfolio {
        let request = request_for(program);
        let cover = cover_with_partitions(program, &[vec![0, 1, 2, 3, 4]]);
        let subject = FrontierRegionSubject::new("fused", request.serial_sum().members.all());
        let source = CoverFrontiers::new(
            &cover,
            vec![frontier_with_opaque(
                &request,
                subject,
                "fused",
                TensorRole::Output,
                crate::effects::Aliasing::Distinct,
                Some(fused_raw(&request)),
            )],
        );
        select_physical_plans(program, budgets(), &formation_of(program), &[source]).unwrap()
    }

    impl PhysicalImplementationProvider for FixedRegionProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(self.provider.clone())
        }

        fn propose(&self, _: &ImplementationContext<'_>) -> Vec<ImplementationProposal> {
            vec![ImplementationProposal::new(
                ProposalBody::ScheduledKernel(Box::new(self.region.clone())),
                governed_applicability(),
                self.cost,
            )]
        }
    }

    fn pointwise_raw(request: &VerifiedTargetRequest) -> ScheduledRegion {
        build_scheduled_regions(request).unwrap()[0]
            .region()
            .clone()
    }

    fn reduction_raw(request: &VerifiedTargetRequest) -> ScheduledRegion {
        build_scheduled_regions(request).unwrap()[1]
            .region()
            .clone()
    }

    fn fused_raw(request: &VerifiedTargetRequest) -> ScheduledRegion {
        build_fused_scheduled_region(request)
            .unwrap()
            .region()
            .clone()
    }

    fn pointwise_frontier(
        request: &VerifiedTargetRequest,
        provider: &str,
        cost: PhysicalCostEstimate,
    ) -> RegionFrontier {
        let subject = FrontierRegionSubject::new(
            "pointwise",
            request.serial_sum().members.pointwise().to_vec(),
        );
        let host = FixedRegionProvider {
            provider: provider_identity(provider, 1),
            cost,
            region: pointwise_raw(request),
        };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&host];
        let frontier = enumerate_frontier(
            request,
            &subject,
            &providers,
            &crate::call_registry::OpaqueCallRegistry::new(),
        )
        .unwrap();
        RegionFrontier::new(subject, frontier)
    }

    fn reduction_frontier(
        request: &VerifiedTargetRequest,
        provider: &str,
        cost: PhysicalCostEstimate,
    ) -> RegionFrontier {
        let subject = FrontierRegionSubject::new(
            "reduction",
            request.serial_sum().members.reduction().to_vec(),
        );
        let host = FixedRegionProvider {
            provider: provider_identity(provider, 1),
            cost,
            region: reduction_raw(request),
        };
        let providers: [&dyn PhysicalImplementationProvider; 1] = [&host];
        let frontier = enumerate_frontier(
            request,
            &subject,
            &providers,
            &crate::call_registry::OpaqueCallRegistry::new(),
        )
        .unwrap();
        RegionFrontier::new(subject, frontier)
    }

    fn empty_frontier(
        subject: FrontierRegionSubject,
        request: &VerifiedTargetRequest,
    ) -> RegionFrontier {
        let providers: [&dyn PhysicalImplementationProvider; 0] = [];
        let frontier = enumerate_frontier(
            request,
            &subject,
            &providers,
            &crate::call_registry::OpaqueCallRegistry::new(),
        )
        .unwrap();
        RegionFrontier::new(subject, frontier)
    }

    fn fused_frontier(
        request: &VerifiedTargetRequest,
        providers: &[(&str, PhysicalCostEstimate)],
    ) -> RegionFrontier {
        let subject = FrontierRegionSubject::new("fused", request.serial_sum().members.all());
        let hosts: Vec<FixedRegionProvider> = providers
            .iter()
            .map(|(name, cost)| FixedRegionProvider {
                provider: provider_identity(name, 1),
                cost: *cost,
                region: fused_raw(request),
            })
            .collect();
        let refs: Vec<&dyn PhysicalImplementationProvider> = hosts
            .iter()
            .map(|host| host as &dyn PhysicalImplementationProvider)
            .collect();
        let frontier = enumerate_frontier(
            request,
            &subject,
            &refs,
            &crate::call_registry::OpaqueCallRegistry::new(),
        )
        .unwrap();
        RegionFrontier::new(subject, frontier)
    }

    /// Finds the enumerated cover whose region member-sets exactly match `expected`.
    fn cover_with_partitions(program: &SemanticProgram, expected: &[Vec<u32>]) -> RegionCover {
        let enumeration = enumerate_covers(
            program,
            DeterministicBudgets::governed(),
            &formation_of(program),
        )
        .unwrap();
        let want: std::collections::BTreeSet<Vec<u32>> = expected.iter().cloned().collect();
        enumeration
            .covers()
            .iter()
            .find(|cover| {
                let have: std::collections::BTreeSet<Vec<u32>> = cover
                    .regions()
                    .iter()
                    .map(|region| region.members().iter().map(|member| member.0).collect())
                    .collect();
                have == want
            })
            .expect("the requested partition is an enumerated cover")
            .clone()
    }

    fn budgets() -> DeterministicBudgets {
        DeterministicBudgets::governed()
    }

    fn contract() -> StrictF32NumericalContract {
        StrictF32NumericalContract::governed()
    }

    /// The two-region {pointwise, reduction} cover joined with the pointwise and
    /// reduction frontiers is one complete plan with one satisfied handoff.
    #[test]
    fn a_complete_plan_joins_a_two_region_cover_with_a_boundary_handoff() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let source = CoverFrontiers::new(
            &cover,
            vec![
                pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                reduction_frontier(&request, "rd", PhysicalCostEstimate::structural(1, 2, 0)),
            ],
        );
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &[source]).unwrap();

        assert_eq!(portfolio.plans().len(), 1, "exactly one complete plan");
        let plan = &portfolio.plans()[0];
        // The two regions are each implemented once.
        assert_eq!(plan.selections().len(), 2);
        // The single pointwise -> reduction intermediate is a satisfied handoff.
        assert_eq!(plan.handoffs().len(), 1);
        assert_eq!(plan.handoffs()[0].role(), TensorRole::Intermediate);
        assert_eq!(plan.handoffs()[0].consumers().len(), 1);
        // Aggregate structural cost sums the regions and counts one materialization.
        assert_eq!(plan.cost().dispatch_count(), 2);
        assert_eq!(plan.cost().launched_threads(), 8);
        assert_eq!(plan.cost().materialization_count(), 1);
        // Feasibility guards are aggregated from both regions.
        assert!(!plan.guards().is_empty());

        verify_selected_plan(&program, &formation_of(&program), plan).unwrap();
        verify_selected_portfolio(&program, &formation_of(&program), &portfolio).unwrap();
    }

    /// Selection retains a scheduled implementation and an opaque call as two
    /// genuinely distinct complete plans for the same cover.
    #[test]
    fn a_scheduled_and_opaque_admission_are_distinct_plan_alternatives() {
        let program = serial_sum_program();
        let portfolio = opaque_fused_portfolio(&program);

        assert_eq!(portfolio.plans().len(), 2);
        assert_eq!(
            portfolio
                .plans()
                .iter()
                .filter(|plan| plan.selections()[0].implementation().scheduled().is_some())
                .count(),
            1
        );
        assert_eq!(
            portfolio
                .plans()
                .iter()
                .filter(|plan| {
                    plan.selections()[0]
                        .implementation()
                        .body()
                        .opaque()
                        .is_some()
                })
                .count(),
            1
        );
        assert_ne!(
            portfolio.plans()[0].identity().as_bytes(),
            portfolio.plans()[1].identity().as_bytes(),
            "the body choice was absent from plan identity"
        );
    }

    /// Analytical components that require a scheduled region decline an opaque
    /// plan instead of treating the missing schedule as zero work.
    #[test]
    fn opaque_plan_costs_are_unknown_where_a_schedule_is_required() {
        use crate::component_cost::{CostComponent, CostValue, analytical_plan_cost};

        let program = serial_sum_program();
        let portfolio = opaque_fused_portfolio(&program);
        let plan = portfolio
            .plans()
            .iter()
            .find(|plan| {
                plan.selections()[0]
                    .implementation()
                    .body()
                    .opaque()
                    .is_some()
            })
            .expect("one opaque plan");
        let cost = analytical_plan_cost(plan);
        for component in [
            CostComponent::Indexing,
            CostComponent::RedundantWork,
            CostComponent::MemoryTraffic,
        ] {
            assert_eq!(
                cost.get(component).expect("every component").value(),
                CostValue::Unknown,
                "{component} treated an absent schedule as a numeric cost"
            );
        }
    }

    /// A call that may return an alias view cannot feed the bounded scheduled
    /// consumer, which requires a materialized buffer.
    #[test]
    fn an_opaque_alias_view_is_refused_by_a_materialized_consumer() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let pointwise_subject = FrontierRegionSubject::new(
            "pointwise",
            request.serial_sum().members.pointwise().to_vec(),
        );
        let source = CoverFrontiers::new(
            &cover,
            vec![
                frontier_with_opaque(
                    &request,
                    pointwise_subject,
                    "aliasing-producer",
                    TensorRole::Intermediate,
                    crate::effects::Aliasing::MayAliasInputs,
                    None,
                ),
                reduction_frontier(
                    &request,
                    "materialized-consumer",
                    PhysicalCostEstimate::structural(1, 2, 0),
                ),
            ],
        );
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &[source]).unwrap();

        assert!(portfolio.plans().is_empty());
        assert!(portfolio.rejections().iter().any(|rejection| matches!(
            rejection,
            PlanRejection::BoundaryDisagreement {
                disagreement: BoundaryDisagreement::UndischargedHandoff {
                    unsatisfied,
                    ..
                },
                ..
            } if unsatisfied.property() == BoundaryProperty::Materialization
        )));
    }

    /// A cover region with no admitted implementation is a valid no-plan result,
    /// distinct from a malformed fault.
    #[test]
    fn an_unimplemented_region_is_a_valid_no_plan_result() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let reduction_subject = FrontierRegionSubject::new(
            "reduction",
            request.serial_sum().members.reduction().to_vec(),
        );
        let source = CoverFrontiers::new(
            &cover,
            vec![
                pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                // The reduction region is deliberately left unimplemented.
                empty_frontier(reduction_subject, &request),
            ],
        );
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &[source]).unwrap();

        assert!(
            portfolio.is_empty(),
            "no complete plan without every region"
        );
        assert_eq!(portfolio.rejections().len(), 1);
        assert!(matches!(
            portfolio.rejections()[0],
            PlanRejection::RegionUnimplemented {
                role: "reduction",
                ..
            }
        ));
    }

    /// The bounded profile's guarantee set, for facets assembled by hand.
    fn profile_guarantees() -> GuaranteedProperties {
        GuaranteedProperties::new([
            GuaranteedProperty::StorageLayout(LayoutGuarantee::DenseRowMajor),
            GuaranteedProperty::StorageEncoding(StorageEncoding::Unpacked),
            GuaranteedProperty::Alignment(ByteAlignment::F32_NATURAL),
            GuaranteedProperty::Materialization(MaterializationForm::MaterializedBuffer),
            GuaranteedProperty::ExecutionAffinity(ExecutionAffinity::PRIMARY),
            GuaranteedProperty::MemoryDomain(MemoryDomainClass::Device),
            GuaranteedProperty::Availability(AvailabilityGuarantee::AfterOwnDispatch),
            GuaranteedProperty::Visibility(VisibilityGuarantee::CoherentOnProducingAffinity),
        ])
        .unwrap()
    }

    /// The bounded profile's requirement set, for facets assembled by hand.
    fn profile_requirements() -> RequiredProperties {
        RequiredProperties::new([
            RequiredProperty::StorageLayout(LayoutRequirement::DenseRowMajor),
            RequiredProperty::StorageEncoding(StorageEncoding::Unpacked),
            RequiredProperty::Alignment(ByteAlignment::F32_NATURAL),
            RequiredProperty::Materialization(MaterializationForm::MaterializedBuffer),
            RequiredProperty::ExecutionAffinity(ExecutionAffinity::PRIMARY),
            RequiredProperty::MemoryDomain(
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).unwrap(),
            ),
            RequiredProperty::Availability(AvailabilityRequirement::AfterProducingDispatch),
            RequiredProperty::Visibility(VisibilityRequirement::ReadableOnRequiringAffinity),
        ])
        .unwrap()
    }

    fn guarantee_facet(tensor: TensorRole, properties: GuaranteedProperties) -> GuaranteeFacet {
        GuaranteeFacet {
            tensor,
            ownership: BoundaryOwnership::TotalRaceFreeWrite,
            properties,
        }
    }

    fn requirement_facet(tensor: TensorRole, properties: RequiredProperties) -> RequirementFacet {
        RequirementFacet {
            tensor,
            access: AccessMode::Read,
            properties,
        }
    }

    /// Boundary reconciliation rejects a materialization edge whose producer does
    /// not guarantee the cross-region intermediate.
    #[test]
    fn boundary_reconciliation_rejects_a_producer_without_the_guarantee() {
        let program = serial_sum_program();
        // A real two-region cover supplies real occurrences and one real edge.
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let edge = &cover.materializations()[0];
        let producer = edge.producer().as_bytes().to_vec();
        let consumer = edge.consumers()[0].as_bytes().to_vec();

        // Producer materializes an Output (not the Intermediate the cover requires);
        // consumer requires the Intermediate. The counts disagree, so reconciliation
        // fails closed rather than silently repairing the handoff.
        let mut boundaries: BTreeMap<Vec<u8>, RegionBoundary> = BTreeMap::new();
        boundaries.insert(
            producer,
            RegionBoundary {
                guarantees: vec![guarantee_facet(TensorRole::Output, profile_guarantees())],
                requirements: vec![requirement_facet(TensorRole::Input, profile_requirements())],
            },
        );
        boundaries.insert(
            consumer,
            RegionBoundary {
                guarantees: vec![guarantee_facet(TensorRole::Output, profile_guarantees())],
                requirements: vec![requirement_facet(
                    TensorRole::Intermediate,
                    profile_requirements(),
                )],
            },
        );

        let outcome = reconcile_boundaries(&cover, &boundaries);
        let Err(super::BoundaryOutcome::Disagreement(disagreement)) = outcome else {
            panic!("expected a boundary disagreement");
        };
        assert!(matches!(
            disagreement,
            BoundaryDisagreement::ProducerGuaranteeMissing { .. }
        ));
        assert_eq!(disagreement.reason(), "producer-guarantee-missing");
    }

    /// A producer that guarantees the cross-region intermediate but delivers it
    /// in the wrong storage layout is refused, and the refusal names the exact
    /// property rather than only the handoff.
    ///
    /// The bounded profile's own regions all guarantee and require dense
    /// row-major, so this case cannot arise end to end; the relation is exercised
    /// directly against the reconciliation with real cover occurrences. That is a
    /// measurement boundary on this test, not a gap in the guard, and it matches
    /// how the sibling disagreement guards above are exercised.
    #[test]
    fn a_handoff_whose_properties_disagree_names_the_property_that_failed() {
        let program = serial_sum_program();
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let edge = &cover.materializations()[0];
        let producer = edge.producer().as_bytes().to_vec();
        let consumer = edge.consumers()[0].as_bytes().to_vec();

        // The consumer needs unit stride on the leading axis — what a vectorized
        // reduction over that axis would ask for. A dense row-major producer has
        // unit stride on the trailing axis only.
        let leading_axis =
            RequiredProperties::new(profile_requirements().properties().iter().map(|property| {
                match property {
                    RequiredProperty::StorageLayout(_) => {
                        RequiredProperty::StorageLayout(LayoutRequirement::UnitStrideOnAxis {
                            axis: Axis::new(0),
                            rank: 2,
                        })
                    }
                    other => other.clone(),
                }
            }))
            .unwrap();

        let mut boundaries: BTreeMap<Vec<u8>, RegionBoundary> = BTreeMap::new();
        boundaries.insert(
            producer,
            RegionBoundary {
                guarantees: vec![guarantee_facet(
                    TensorRole::Intermediate,
                    profile_guarantees(),
                )],
                requirements: vec![requirement_facet(TensorRole::Input, profile_requirements())],
            },
        );
        boundaries.insert(
            consumer,
            RegionBoundary {
                guarantees: vec![guarantee_facet(TensorRole::Output, profile_guarantees())],
                requirements: vec![requirement_facet(TensorRole::Intermediate, leading_axis)],
            },
        );

        let outcome = reconcile_boundaries(&cover, &boundaries);
        let Err(super::BoundaryOutcome::Disagreement(
            disagreement @ BoundaryDisagreement::UndischargedHandoff { .. },
        )) = outcome
        else {
            panic!("expected an undischarged handoff");
        };
        let BoundaryDisagreement::UndischargedHandoff { unsatisfied, .. } = &disagreement else {
            unreachable!("matched above")
        };
        assert_eq!(unsatisfied.property(), BoundaryProperty::StorageLayout);
        assert_eq!(unsatisfied.reason(), UnsatisfiedReason::NotSatisfied);
        assert!(unsatisfied.guaranteed().is_some());
        assert_eq!(disagreement.reason(), "property-not-satisfied");
    }

    /// A valid two-region plan reconciles into exactly one satisfied handoff, so
    /// the positive composition path is exercised on real boundary contracts.
    #[test]
    fn a_valid_two_region_composition_reconciles_one_handoff() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let source = CoverFrontiers::new(
            &cover,
            vec![
                pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                reduction_frontier(&request, "rd", PhysicalCostEstimate::structural(1, 2, 0)),
            ],
        );
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &[source]).unwrap();
        assert!(portfolio.rejections().is_empty());
        assert_eq!(portfolio.plans()[0].handoffs().len(), 1);
    }

    /// Structural dominance is a pure view: every valid plan is retained, and
    /// `non_dominated` prunes only a plan another plan beats on the structural
    /// dimensions. Feasibility never enters, and the dominated plan stays retained.
    #[test]
    fn structural_dominance_prunes_the_non_dominated_view_only() {
        let program = serial_sum_program();
        let request = request_for(&program);
        // The fused whole-program cover: one region, two feasible implementations.
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3, 4]]);
        let source = CoverFrontiers::new(
            &cover,
            vec![fused_frontier(
                &request,
                &[
                    ("cheap", PhysicalCostEstimate::structural(1, 2, 0)),
                    ("dominated", PhysicalCostEstimate::structural(1, 4, 0)),
                ],
            )],
        );
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &[source]).unwrap();

        // Validity retains both plans; cost never gates retention.
        assert_eq!(portfolio.plans().len(), 2);
        // The structural view prunes the dominated plan.
        let non_dominated = portfolio.non_dominated();
        assert_eq!(non_dominated.len(), 1);
        assert_eq!(non_dominated[0].cost().launched_threads(), 2);
        verify_selected_portfolio(&program, &formation_of(&program), &portfolio).unwrap();
    }

    /// Plan and portfolio identities are deterministic and independent of the order
    /// in which sources and providers are supplied.
    #[test]
    fn identity_is_deterministic_and_order_independent() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let two_region = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let fused = cover_with_partitions(&program, &[vec![0, 1, 2, 3, 4]]);

        let build = |forward: bool| {
            let two = CoverFrontiers::new(
                &two_region,
                vec![
                    pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                    reduction_frontier(&request, "rd", PhysicalCostEstimate::structural(1, 2, 0)),
                ],
            );
            let whole = CoverFrontiers::new(
                &fused,
                vec![fused_frontier(
                    &request,
                    &[("fx", PhysicalCostEstimate::structural(1, 2, 0))],
                )],
            );
            let sources = if forward {
                vec![two, whole]
            } else {
                vec![whole, two]
            };
            select_physical_plans(&program, budgets(), &formation_of(&program), &sources).unwrap()
        };

        let first = build(true);
        let second = build(false);
        assert_eq!(
            first.identity().as_bytes(),
            second.identity().as_bytes(),
            "portfolio identity must not depend on source order"
        );
        let ids = |portfolio: &super::SelectedPortfolio| -> Vec<Vec<u8>> {
            portfolio
                .plans()
                .iter()
                .map(|plan| plan.identity().as_bytes().to_vec())
                .collect()
        };
        assert_eq!(ids(&first), ids(&second));
        assert!(!first.identity().label().is_empty());
    }

    /// Equal bytes are equal identities, and the cached digest cannot change that.
    ///
    /// `SelectedPlanIdentity` is a map key and a sort key, so two identities
    /// over equal bytes comparing unequal would be an identity defect — far
    /// worse than the compile time the cache saves. The comparisons are written
    /// out rather than derived precisely so the second field cannot enter them,
    /// and this is what says so.
    #[test]
    fn the_cached_digest_stays_out_of_identity_comparison() {
        use super::{SelectedPlanIdentity, digest};

        let bytes = b"selected-plan-identity-bytes".to_vec();
        let honest = SelectedPlanIdentity {
            digest: digest(&bytes),
            bytes: bytes.clone(),
        };
        // A deliberately wrong cache. Nothing can construct this outside the
        // module, which is the point: if a comparison ever consulted the digest,
        // this pair would stop being equal and the assertions below would fail.
        let corrupted = SelectedPlanIdentity {
            digest: honest.digest ^ 0xFFFF_FFFF_FFFF_FFFF,
            bytes,
        };

        assert_eq!(honest, corrupted, "equality consulted the cached digest");
        assert_eq!(
            honest.cmp(&corrupted),
            core::cmp::Ordering::Equal,
            "ordering consulted the cached digest"
        );
        assert_eq!(honest.as_bytes(), corrupted.as_bytes());

        // And the label *does* consult it, which is what makes the cache worth
        // having — so the two labels differ, proving the field is live rather
        // than dead weight the compiler folded away.
        assert_ne!(
            honest.label(),
            corrupted.label(),
            "the label must read the cached digest, or caching bought nothing"
        );
    }

    /// `is_labelled` admits exactly the string `label` produces, and rejects the
    /// near-misses that a looser parse would have let through.
    ///
    /// The uppercase case is the one that matters: it is what `from_str_radix`
    /// alone would have accepted, so it is the difference between replacing an
    /// allocation and quietly widening a tamper check.
    #[test]
    fn is_labelled_admits_only_the_label_it_replaces() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let sources = vec![CoverFrontiers::new(
            &cover,
            vec![
                pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                reduction_frontier(&request, "rd", PhysicalCostEstimate::structural(1, 2, 0)),
            ],
        )];
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &sources)
                .expect("portfolio");
        let identity = portfolio.plans().first().expect("one plan").identity();

        let label = identity.label();
        assert!(
            identity.is_labelled(&label),
            "the real label must be admitted"
        );

        let hex = label
            .strip_prefix("selected-plan:")
            .expect("the label carries its prefix");
        for (case, forged) in [
            (
                "uppercase hex",
                format!("selected-plan:{}", hex.to_uppercase()),
            ),
            ("wrong prefix", format!("selected-portfolio:{hex}")),
            ("no prefix", hex.to_owned()),
            ("truncated", format!("selected-plan:{}", &hex[..15])),
            ("lengthened", format!("selected-plan:0{hex}")),
            (
                "flipped digit",
                format!(
                    "selected-plan:{}{}",
                    if hex.starts_with('0') { '1' } else { '0' },
                    &hex[1..]
                ),
            ),
        ] {
            assert!(
                !identity.is_labelled(&forged),
                "{case} must be rejected, but `{forged}` was admitted"
            );
        }

        // The uppercase spelling is only a meaningful case if the label actually
        // contains a letter to change; a digit-only digest would make that row
        // vacuously pass and the guard it covers untested.
        assert!(
            hex.bytes().any(|byte| byte.is_ascii_alphabetic()),
            "this digest has no hex letter, so the uppercase row proved nothing"
        );
    }

    /// A forged receipt fails re-derivation: a foreign program, a tampered cost,
    /// and a swapped implementation each fail closed with a typed error.
    #[test]
    fn a_forged_plan_fails_re_derivation() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        let source = CoverFrontiers::new(
            &cover,
            vec![
                pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                reduction_frontier(&request, "rd", PhysicalCostEstimate::structural(1, 2, 0)),
            ],
        );
        let portfolio =
            select_physical_plans(&program, budgets(), &formation_of(&program), &[source]).unwrap();
        let plan = portfolio.plans()[0].clone();

        // A genuinely different program fails cover occurrence re-derivation.
        let error =
            verify_selected_plan(&diamond_program(), &formation_of(&diamond_program()), &plan)
                .unwrap_err();
        assert_eq!(error.class(), "cover");

        // A tampered aggregate cost no longer matches the re-derived plan.
        let mut tampered = plan.clone();
        tampered.cost.launched_threads += 1;
        let error = verify_selected_plan(&program, &formation_of(&program), &tampered).unwrap_err();
        assert_eq!(error.class(), "structure");
        assert_eq!(error.reason(), "plan-mismatch");

        // Swapping the pointwise region's implementation for a reduction one (whose
        // members are a different set) breaks the region binding.
        let reduction_only =
            reduction_frontier(&request, "rd", PhysicalCostEstimate::structural(1, 2, 0));
        let foreign_impl = reduction_only.frontier().admitted()[0].clone();
        let mut swapped = plan.clone();
        let pointwise_index = swapped
            .selections
            .iter()
            .position(|selection| selection.implementation().semantic_members().len() == 4)
            .expect("the pointwise region covers four members");
        swapped.selections[pointwise_index].implementation = foreign_impl;
        let error = verify_selected_plan(&program, &formation_of(&program), &swapped).unwrap_err();
        assert_eq!(error.class(), "binding");
    }

    /// A cover that no longer re-derives from the program is a malformed fault, not
    /// a valid no-plan result.
    #[test]
    fn a_foreign_cover_fails_the_selection_closed() {
        let program = serial_sum_program();
        let request = request_for(&program);
        // A cover enumerated from a structurally different program.
        let foreign_cover = cover_with_partitions(&diamond_program(), &[vec![0, 1, 2, 3, 4]]);
        let source = CoverFrontiers::new(
            &foreign_cover,
            vec![fused_frontier(
                &request,
                &[("fx", PhysicalCostEstimate::structural(1, 2, 0))],
            )],
        );
        let error = select_physical_plans(&program, budgets(), &formation_of(&program), &[source])
            .unwrap_err();
        assert_eq!(error.class(), "cover");
    }

    /// A supplied frontier that does not correspond to a cover region fails the
    /// one-to-one binding closed.
    #[test]
    fn a_mismatched_region_frontier_fails_binding() {
        let program = serial_sum_program();
        let request = request_for(&program);
        let cover = cover_with_partitions(&program, &[vec![0, 1, 2, 3], vec![4]]);
        // Two frontiers, but both for the pointwise region: the reduction region has
        // no corresponding frontier.
        let source = CoverFrontiers::new(
            &cover,
            vec![
                pointwise_frontier(&request, "pw", PhysicalCostEstimate::structural(1, 6, 0)),
                pointwise_frontier(&request, "pw2", PhysicalCostEstimate::structural(1, 6, 0)),
            ],
        );
        let error = select_physical_plans(&program, budgets(), &formation_of(&program), &[source])
            .unwrap_err();
        assert_eq!(error.class(), "binding");
    }

    /// Error taxonomy: each variant reports its exact class and reason.
    #[test]
    fn errors_report_their_class_and_reason() {
        let binding = SelectionError::FrontierBinding {
            rule: "region-frontier-missing",
        };
        assert_eq!(binding.class(), "binding");
        assert_eq!(binding.reason(), "region-frontier-missing");
        assert_eq!(
            binding.to_string(),
            "selection.binding.region-frontier-missing"
        );

        let structure = SelectionError::Structure {
            rule: "plan-mismatch",
        };
        assert_eq!(structure.class(), "structure");
        assert_eq!(structure.to_string(), "selection.structure.plan-mismatch");

        let unsatisfied = crate::boundary::unsatisfied_properties(
            &RequiredProperties::new([RequiredProperty::Alignment(
                ByteAlignment::new(16).unwrap(),
            )])
            .unwrap(),
            &GuaranteedProperties::new([GuaranteedProperty::Alignment(ByteAlignment::F32_NATURAL)])
                .unwrap(),
        )
        .remove(0);
        let composition =
            SelectionError::InvalidComposition(BoundaryDisagreement::UndischargedHandoff {
                producer: vec![1],
                consumer: vec![2],
                unsatisfied,
            });
        assert_eq!(composition.class(), "composition");
        // The reason is delegated to the property model, so a composition fault
        // names which property was not supplied.
        assert_eq!(composition.reason(), "property-not-satisfied");
    }

    /// A diamond program that is structurally distinct from the serial-sum chain,
    /// used to exercise foreign-cover re-derivation faults.
    fn diamond_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let shared = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let left = F32Multiply::apply(&mut builder, shared, shared).unwrap();
        let right = F32Add::apply(&mut builder, shared, shared).unwrap();
        let joined = F32Add::apply(&mut builder, left, right).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), joined)
            .unwrap();
        builder.build().unwrap()
    }
}
