//! The work-span selector a target profile's measured cost row activates.
//!
//! This is the consuming half of the profile's saturated-parallel-fold-steps
//! row. A profile that declares that row states one machine quantity — the fold
//! steps the device retires at once when it is saturated — and this module turns
//! it into a per-plan ordering. A profile that declares nothing here produces
//! nothing here, and selection falls back to the structural Pareto view it has
//! always used.
//!
//! # What this is not
//!
//! **Not a [`crate::selection::PlanStructuralCost`] dimension, and not a second
//! cost-model key entering dominance.** [`crate::component_cost`]'s header
//! records why a second key cannot join the first: `PlanStructuralCost::dominates`
//! returns `false` across differing keys, so plans carrying different keys never
//! dominate each other, the non-dominated set silently becomes the whole set, and
//! Pareto pruning goes dark with nothing reporting it. Nothing here is a
//! `PhysicalCostEstimate`, nothing here carries a model key into `aggregate_cost`,
//! and nothing here has a `dominates`. The structural relation keeps its exact
//! four dimensions, its single key, and its full pruning power; it is still what
//! `SelectedPortfolio::non_dominated` computes, and explain still reports it for
//! every alternative.
//!
//! **Not a latency estimate.** The retained measurement is explicit that the
//! fitted model "is a selector, not a latency estimate" and that its magnitude
//! accuracy is much weaker than its decision accuracy. This module therefore
//! never reports seconds; it reports fold steps, and they are only ever compared.
//!
//! # What it changes about selection, stated rather than absorbed
//!
//! Selection today is a Pareto relation over exact structural counts with a
//! canonical-identity tie break. **A measured term that can prefer a
//! structurally dominated plan is a change to what selection is**, and the change
//! is deliberate rather than incidental, so it is written down:
//!
//! - Structural dominance proves one plan uses no more of four exact resources
//!   and strictly less of one. It was never a proof that the plan is faster, and
//!   [`crate::selection`]'s header says so — "proved structural dominance, not an
//!   uncalibrated latency total order".
//! - **The retained measurement refutes fewer-resources-is-faster on a named
//!   contour.** The serial fold issues no more dispatches, launches strictly
//!   fewer threads, and allocates no more temporary storage than either parallel
//!   strategy, so it structurally dominates both — `pipeline::tests`'
//!   `the_frontier_retains_the_split_beside_the_serial_reduction` already pins
//!   that in so many words, and named this activation as the owner of the
//!   preference. The 2026-08-07 dispatch sweep measured the same fold costing up
//!   to **50.7x** the best parallel plan.
//! - So a term that only broke ties *inside* the non-dominated set could not
//!   express the measured result at all: on this program family that set is a
//!   singleton, which `pipeline::tests`'
//!   `the_parallel_reduction_plans_are_structurally_dominated` asserts rather
//!   than assumes.
//!
//! The measured term therefore ranges over the **retained valid plans**, which is
//! every plan hard feasibility and boundary composition admitted. It can prefer a
//! structurally dominated plan; it can never prefer an infeasible one, because no
//! infeasible plan is in the set. Feasibility was decided by the frontier and is
//! not consulted, weakened, or re-run here.
//!
//! # The model
//!
//! **Measurement, 2026-08-07** —
//! `spikes/program-planning/reduction-dispatch-crossover`, retained at
//! `results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`. The fitted cost is
//!
//! ```text
//! cost = sum over stages of ( encoder + max(work / P, depth) * step )
//! ```
//!
//! and the sweep's own mutation table reports that **only `P` moves a decision**:
//! scaling `encoder` by twenty or `step` by a tenth leaves every predicted winner
//! unchanged, while scaling `P` by a quarter drops held-out agreement from 24 of
//! 26 separated cells to 20 and the worst penalty from 1.81x to 3.04x.
//!
//! This module activates that model **less its two decision-inert parameters**,
//! and the omission is derived rather than convenient:
//!
//! - `step` is one positive factor over the whole sum, so scaling it cannot
//!   reorder two candidates. Dropping it is exactly order-preserving, not
//!   approximately.
//! - `encoder` is a per-stage constant, so it prices *dispatch count* — and
//!   dispatch count is already one of the four exact dimensions
//!   [`crate::selection::PlanStructuralCost`] carries and prunes on. Pricing it a
//!   second time here would put one quantity under two authorities. The sweep
//!   separately measures it inert in the decision.
//!
//! What remains is one term, one declared row, and no free parameters:
//!
//! ```text
//! fold_steps = sum over stages of max( work, depth * P )
//! ```
//!
//! which is `P` times the surviving model and therefore orders candidates
//! identically to it. **The scaling is what keeps this integer**: comparing
//! `max(work / P, depth)` needs a division, and comparing `max(work, depth * P)`
//! does not, so the selector is exact rather than floating-point and two plans
//! that compare equal do so because their step counts are equal.
//!
//! # Boundary
//!
//! `P` is a quantity of one host row. The retained sweep covers one profile
//! (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract, one program family,
//! `f32` only, and it determines `P` only to about a factor of four. A second
//! target profile declares its own row or declares none.

use tiler_ir::schedule::{
    Access, KernelSchedule, LogicalAccess, ReductionPass, ReductionTopology, ScheduledRegion,
    element_count,
};

use crate::physical::VerifiedScheduledRegion;

/// The governed key naming this selector.
///
/// Distinct from `tiler.cost.structural.v1` and from
/// `tiler.cost.analytical.v1` by construction: nothing attributed to this key
/// may enter a structural dominance comparison, and this module exposes no
/// `dominates` for one to be written with.
pub(crate) const MEASURED_FOLD_STEP_MODEL_KEY: &str = "tiler.cost.measured-fold-steps.v1";

/// One stage's work and span, in fold steps.
///
/// The two quantities the work-span bound ranges over, named exactly as the
/// retained spike names them so the two derivations can be compared directly.
/// `work` is **not** `threads * depth`: in a cooperative tile the committing
/// participant folds the staged slots while every other participant idles, so
/// charging every invocation for the whole path would overstate the tree by the
/// participant count and hide the crossover it exists to locate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StageWorkSpan {
    /// Fold steps this stage performs, summed over every invocation.
    pub(crate) work: u64,
    /// Fold steps on the longest sequential path through one invocation.
    pub(crate) depth: u64,
}

/// One plan's measured fold-step assessment, with both sides of the `max`.
///
/// Both sides are retained rather than only their maximum, because which side
/// dominated is the whole content of the model: when the launch already saturates
/// the device the work side decides and the cheapest plan is whichever does least
/// total work, and when it does not the span side decides and a shallower
/// critical path wins. A report carrying only the total would say which plan won
/// without saying why.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "every field is a count of fold steps and the unit is the point: the suffix is what stops `work` and `span` being read as seconds, and dropping it would leave three quantities in one struct whose unit a reader has to infer"
)]
pub(crate) struct FoldStepAssessment {
    /// The declared row: fold steps the device retires at once when saturated.
    pub(crate) saturated_parallel_fold_steps: u64,
    /// The work side of the `max`, summed over stages: total fold steps.
    pub(crate) work_steps: u64,
    /// The span side of the `max`, summed over stages: `depth * P`, in the same
    /// scaled units as [`Self::work_steps`] so the two are directly comparable.
    pub(crate) span_steps: u64,
    /// The compared total: `sum over stages of max(work, depth * P)`.
    ///
    /// Deliberately not `max(work_steps, span_steps)`. The `max` is taken per
    /// stage and then summed, because a plan whose first stage is work-bound and
    /// whose second is span-bound pays both; one maximum over the two sums would
    /// charge it for only the larger.
    pub(crate) fold_steps: u64,
}

/// Derives one region's work and span from its own declared schedule.
///
/// Returns `None` for a reduction topology this derivation does not cover, which
/// is the fail-safe direction: a declined stage declines the whole plan, a
/// declined plan declines the whole comparison, and selection falls back to the
/// structural view rather than comparing plans on a total that silently omitted a
/// stage.
pub(crate) fn stage_work_span(region: &ScheduledRegion) -> Option<StageWorkSpan> {
    work_span(&region.schedule, &region.index.accesses)
}

/// The derivation itself, over the two parts of a region it actually reads.
///
/// Separated from the region so the arithmetic can be driven directly, for the
/// reason `component_cost::repeated_work` is separated from its match arm: two of
/// these arms are unreachable through any plan this build assembles, and an arm
/// whose outcome is unreachable through its caller is an arm nothing has shown
/// can produce one.
///
/// [`ReductionTopology`] and [`ReductionPass`] are `#[non_exhaustive]` outside
/// `tiler-ir`, so the wildcard arms are required rather than chosen — deleting
/// either fails the build with E0004. They decline, which is why a topology
/// arriving from that crate cannot quietly be costed as though it folded
/// nothing.
///
/// Declining is still silent, though, and silence is what
/// `every_reduction_topology_states_a_verdict` and
/// `every_reduction_pass_states_a_verdict` convert into a build error: both size
/// their census from the enum, so a widened vocabulary stops compiling here
/// until its new member's verdict is stated.
fn work_span(schedule: &KernelSchedule, accesses: &[Access]) -> Option<StageWorkSpan> {
    let work_items = schedule.work_items;
    match &schedule.reduction {
        // No fold: one step per launched iteration point, no sequential run.
        ReductionTopology::None => Some(StageWorkSpan {
            work: work_items,
            depth: 1,
        }),
        // One invocation per output position folds the whole contributor run, so
        // the run *is* the critical path and the total work is the reduced input.
        // Both are read from the region's own contributor access rather than from
        // the launch, because the launch states how many folds run and the access
        // states how long each one is.
        ReductionTopology::Serial { .. } => {
            let (input, output) = serial_contributor_extents(accesses)?;
            Some(StageWorkSpan {
                work: input,
                depth: exact_ratio(input, output)?,
            })
        }
        // The contracted index space is the fold, and the region states its shape
        // directly rather than through an access relation: a contraction's two
        // operands each name a different subset of the free indices, so neither
        // stands in the reduction's input-to-output relation.
        ReductionTopology::Contraction {
            contracted_shape, ..
        } => {
            let contributors = element_count(contracted_shape).ok()?;
            Some(StageWorkSpan {
                work: work_items.checked_mul(contributors)?,
                depth: contributors,
            })
        }
        // Each pass folds a run of its own length once per launched invocation,
        // and that run is both the pass's critical path and its per-invocation
        // work. The partial pass launches one invocation per partition and folds
        // `contributors_per_partition`; the final pass launches one per output
        // position and folds the `partitions` staged partials.
        ReductionTopology::MultiPass { pass, coverage, .. } => {
            let partition = coverage.partition();
            let depth = match pass {
                ReductionPass::Partial => partition.contributors_per_partition,
                ReductionPass::Final => partition.partitions,
                _ => return None,
            };
            Some(StageWorkSpan {
                work: work_items.checked_mul(depth)?,
                depth,
            })
        }
        // One workgroup per output position. Per round every participant folds its
        // own `contributors_per_partition` run — `work_items *
        // contributors_per_partition` steps over the whole launch — and then the
        // committing participant of each output position folds the `partitions`
        // staged slots, which is one further step per launched participant, hence
        // the `+ 1`.
        //
        // **The span is the sum of the two phases rather than their maximum**: the
        // staged fold happens after the barrier, not beside it.
        //
        // `rounds` multiplies both, because the tile's own contract states the
        // phases are a round body — each round rewrites the slots the previous one
        // read. **Every plan this build assembles carries `rounds == 1`**, which
        // `workgroup_tree_tile` states, so the multiplier is correct by derivation
        // and unexercised by the compile path; the unit test below drives it for
        // that reason.
        ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } => {
            let partition = coverage.partition();
            let per_round_work =
                work_items.checked_mul(partition.contributors_per_partition.checked_add(1)?)?;
            let per_round_depth = partition
                .contributors_per_partition
                .checked_add(partition.partitions)?;
            Some(StageWorkSpan {
                work: per_round_work.checked_mul(tile.rounds)?,
                depth: per_round_depth.checked_mul(tile.rounds)?,
            })
        }
        // One invocation per output position, and the tile stages *operands*
        // rather than partials. That is the whole difference from the arm above:
        // no participant ever folds another's staged slots, so there is no
        // `+ 1`, and the span is one invocation's own contracted run rather than
        // a two-phase sum across a barrier. The topology's own contract states
        // the relation — `commit` names every participant, the iteration domain
        // *is* the output, and `owned_output_positions` equals the work-item
        // count.
        //
        // Derived from the topology's own tile and rounds, which are the
        // quantities the schedule states about *this* realization: one round
        // folds `contracted_tile`'s points per invocation, and the round loop
        // repeats that `tile.rounds` times. The admission functions require
        // every tile extent to divide its contracted extent with the quotient
        // product equal to `rounds`, so the tiled derivation and the contracted
        // space must agree. Disagreeing means one of the three facts is
        // malformed, and declining is what keeps a malformed triple from
        // producing a plausible number — the reason `exact_ratio` refuses to
        // truncate.
        //
        // **`work_items` is the logical output population, not the launch.** A
        // predicated tail's grid is a strict superset stated separately as
        // `launch.grid_threads`, and its masked invocations retire no fold step;
        // `admit_predicated_cooperative_contraction` returns the two as
        // separate fields for exactly that reason.
        ReductionTopology::CooperativeContraction {
            tile,
            contracted_shape,
            contracted_tile,
            ..
        } => {
            let per_round = element_count(contracted_tile).ok()?;
            let depth = per_round.checked_mul(tile.rounds)?;
            if depth != element_count(contracted_shape).ok()? {
                return None;
            }
            Some(StageWorkSpan {
                work: work_items.checked_mul(depth)?,
                depth,
            })
        }
        // Required by `rustc`, not chosen: `ReductionTopology` is
        // `#[non_exhaustive]` outside `tiler-ir`, so deleting this arm fails
        // with E0004. `every_reduction_topology_states_a_verdict` is what stops
        // it absorbing a widened vocabulary silently.
        //
        // **What a decline costs, which is more than this plan's own score.**
        // `assess_fold_steps` propagates it with `?`, so one declined stage
        // declines that plan; `pipeline::planning::measured_scores` then
        // collects `Option<Vec<_>>` over *every* retained alternative, so one
        // declining alternative collapses the measured comparison for all of
        // them and `select_non_dominated` falls back to the structural Pareto
        // view for the whole target. The declining plan is neither withheld nor
        // outranked — it stays offered and selectable — but the term that exists
        // to correct structural dominance stops applying to its neighbours too.
        // The retained sweep measured the structurally dominant serial fold up
        // to 50.7x slower than the best parallel plan, which is the regime that
        // fallback silently re-enters.
        _ => None,
    }
}

/// Assesses one plan's stages against a declared saturated-fold-step row.
///
/// Returns `None` when any stage declines, when the arithmetic would overflow, or
/// when the plan dispatches nothing. Each is a refusal to state a total rather
/// than a total that quietly dropped a stage.
pub(crate) fn assess_fold_steps(
    stages: &[VerifiedScheduledRegion],
    saturated_parallel_fold_steps: u64,
) -> Option<FoldStepAssessment> {
    if stages.is_empty() || saturated_parallel_fold_steps == 0 {
        return None;
    }
    let mut work_steps = 0_u64;
    let mut span_steps = 0_u64;
    let mut fold_steps = 0_u64;
    for stage in stages {
        let StageWorkSpan { work, depth } = stage_work_span(stage.region())?;
        let span = depth.checked_mul(saturated_parallel_fold_steps)?;
        work_steps = work_steps.checked_add(work)?;
        span_steps = span_steps.checked_add(span)?;
        fold_steps = fold_steps.checked_add(work.max(span))?;
    }
    Some(FoldStepAssessment {
        saturated_parallel_fold_steps,
        work_steps,
        span_steps,
        fold_steps,
    })
}

/// The reduced input and output element counts of a serial fold's own access.
///
/// Read from the region's [`LogicalAccess::ReductionContributor`] read, which is
/// the one authority stating both shapes. A region declaring a serial topology
/// without one is malformed rather than free, so this declines.
fn serial_contributor_extents(accesses: &[Access]) -> Option<(u64, u64)> {
    accesses.iter().find_map(|access| {
        let LogicalAccess::ReductionContributor {
            input_shape,
            output_shape,
            ..
        } = &access.map
        else {
            return None;
        };
        Some((
            element_count(input_shape).ok()?,
            element_count(output_shape).ok()?,
        ))
    })
}

/// `input / output`, only when the division is exact and the divisor nonzero.
///
/// A reduction's output shape is its input shape with the reduced axes removed,
/// so the ratio is the contributor count exactly. Requiring exactness rather than
/// truncating is what keeps a malformed pair from producing a plausible number.
const fn exact_ratio(input: u64, output: u64) -> Option<u64> {
    if output == 0 || !input.is_multiple_of(output) {
        return None;
    }
    Some(input / output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::variant_count;

    use tiler_ir::schedule::{
        AccessMode, AccessOrdinal, ArithmeticType, BoundsWitnessId, ContributorArrival,
        ContributorCoverage, ContributorOrder, ContributorPartition, CooperativeTile,
        ExecutionBinding, LaunchPlan, OwnershipWitnessId, TailPolicy, TensorRole,
        blocked_operand_tile, workgroup_tree_tile,
    };
    use tiler_ir::shape::{Axis, Shape};

    /// A linear schedule carrying one reduction topology and nothing else.
    fn schedule(work_items: u64, reduction: ReductionTopology) -> KernelSchedule {
        KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction,
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        }
    }

    /// The contributor read a serial fold's derivation reads its extents from.
    fn contributor_access(input: [u64; 2], output: [u64; 1]) -> Access {
        Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: Shape::from_dims(input),
                output_shape: Shape::from_dims(output),
                axes: Vec::new(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        }
    }

    fn cooperative(partition: ContributorPartition, tile: CooperativeTile) -> ReductionTopology {
        ReductionTopology::CooperativeWorkgroup {
            coverage: ContributorCoverage::Exact(partition),
            tile,
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
            arrival: ContributorArrival::AscendingParticipant,
        }
    }

    /// The exact-ratio guard can say no in both directions.
    ///
    /// A truncating division would turn a malformed reduction into a plausible
    /// contributor count, which is the failure a reader cannot see.
    #[test]
    fn a_contributor_ratio_is_exact_or_absent() {
        assert_eq!(exact_ratio(8_192, 4), Some(2_048));
        assert_eq!(exact_ratio(9, 4), None, "an inexact ratio must decline");
        assert_eq!(exact_ratio(9, 0), None, "a zero divisor must decline");
    }

    /// The serial arm reproduces the retained sweep's own fold triples.
    ///
    /// The sweep records `1:4:4` for the serial reduction stage at one row of
    /// four contributors and `1:8192:8192` at one row of 8,192 — one invocation,
    /// `elements` fold steps, a depth of the whole contributor run.
    #[test]
    fn the_serial_arm_reproduces_the_retained_fold_triples() {
        let serial = || ReductionTopology::Serial {
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            permits_reassociation: true,
            permits_permutation: false,
        };
        assert_eq!(
            work_span(&schedule(1, serial()), &[contributor_access([1, 4], [1])]),
            Some(StageWorkSpan { work: 4, depth: 4 })
        );
        assert_eq!(
            work_span(
                &schedule(1, serial()),
                &[contributor_access([1, 8_192], [1])]
            ),
            Some(StageWorkSpan {
                work: 8_192,
                depth: 8_192
            })
        );
        // 16,384 rows of four: sixteen thousand independent folds four deep.
        assert_eq!(
            work_span(
                &schedule(16_384, serial()),
                &[contributor_access([16_384, 4], [16_384])]
            ),
            Some(StageWorkSpan {
                work: 65_536,
                depth: 4
            })
        );
    }

    /// A serial fold with no contributor access declines rather than costing
    /// zero.
    ///
    /// The fail-safe direction: a comparison run over a total that dropped a
    /// stage would prefer whichever plan the derivation happened to understand
    /// least, which is the opposite of what the evidence supports.
    #[test]
    fn a_serial_fold_without_its_contributor_access_declines() {
        let reduction = ReductionTopology::Serial {
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            permits_reassociation: true,
            permits_permutation: false,
        };
        assert_eq!(work_span(&schedule(4, reduction), &[]), None);
    }

    /// A live contraction has no specialized contributor count to cost.
    ///
    /// Baking `S` into the span would prefer one neighbour over the other and
    /// put the live value into a selector identity it does not own.
    #[test]
    fn a_live_contraction_work_span_declines_rather_than_baking_s() {
        let live = ReductionTopology::LiveContraction {
            live_access: AccessOrdinal::FIRST,
            live_axis: Axis::new(1),
            order: ContributorOrder::OriginalAxisLexicographic,
            permits_reassociation: false,
            permits_permutation: false,
        };
        assert_eq!(work_span(&schedule(6, live), &[]), None);
    }

    /// The prologue and the two split passes reproduce the retained triples.
    ///
    /// At one row of sixteen contributors the sweep records the split as
    /// `16:16:1|4:16:4|1:4:4` — the prologue, then four invocations folding four
    /// each, then one folding the four partials.
    #[test]
    fn the_prologue_and_split_arms_reproduce_the_retained_triples() {
        assert_eq!(
            work_span(&schedule(16, ReductionTopology::None), &[]),
            Some(StageWorkSpan { work: 16, depth: 1 })
        );
        let partition = ContributorPartition {
            partitions: 4,
            contributors_per_partition: 4,
        };
        let pass = |pass| ReductionTopology::MultiPass {
            pass,
            coverage: ContributorCoverage::Exact(partition),
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
        };
        assert_eq!(
            work_span(&schedule(4, pass(ReductionPass::Partial)), &[]),
            Some(StageWorkSpan { work: 16, depth: 4 })
        );
        assert_eq!(
            work_span(&schedule(1, pass(ReductionPass::Final)), &[]),
            Some(StageWorkSpan { work: 4, depth: 4 })
        );
    }

    /// The cooperative arm reproduces the retained spike's own triples.
    ///
    /// **This is the cross-check that keeps two derivations of one topology from
    /// drifting.** `spikes/program-planning/reduction-dispatch-crossover`'s
    /// `model.rs` records `work = elements + rows * partitions` and
    /// `depth = per_partition + partitions` for the single-workgroup tree, and
    /// the retained TSV carries the resulting triples per cell. The two cells
    /// asserted here are read off that file: at one row of four contributors the
    /// tree stage is `2:6:4`, and at one row of sixteen it is `4:20:8`.
    ///
    /// It also drives the `rounds` multiplier, which no plan this build assembles
    /// can reach — `workgroup_tree_tile` states `rounds: 1` — so leaving it to
    /// the compile path would leave it untested.
    #[test]
    fn the_cooperative_arm_reproduces_the_retained_stage_triples() {
        let span = |work_items: u64, partitions: u64, per_partition: u64, rounds: u64| {
            let mut tile = workgroup_tree_tile(partitions).expect("a representable tile");
            tile.rounds = rounds;
            let partition = ContributorPartition {
                partitions,
                contributors_per_partition: per_partition,
            };
            work_span(&schedule(work_items, cooperative(partition, tile)), &[])
                .expect("the cooperative arm states a work span")
        };

        // One row of four contributors: two participants folding two each. The
        // retained sweep records `2:6:4` — two invocations, six fold steps, a
        // depth of four.
        assert_eq!(
            span(2, 2, 2, 1),
            StageWorkSpan { work: 6, depth: 4 },
            "the tree's work span diverged from the retained sweep at (1, 4)"
        );
        // One row of sixteen: four participants folding four each, `4:20:8`.
        assert_eq!(
            span(4, 4, 4, 1),
            StageWorkSpan { work: 20, depth: 8 },
            "the tree's work span diverged from the retained sweep at (1, 16)"
        );
        // The round multiplier scales both, and is what no compiled plan drives.
        assert_eq!(
            span(4, 4, 4, 2),
            StageWorkSpan {
                work: 40,
                depth: 16
            }
        );
    }

    /// The tiled contraction is scored from its own tile and round count.
    ///
    /// `K = 32` over a 16-wide contracted tile is two rounds, the smallest shape
    /// that separates a per-round quantity from a whole-space one: a derivation
    /// that forgot the round multiplier would report a depth of sixteen here and
    /// the two would be indistinguishable at one round.
    ///
    /// **The tiled fold and the direct fold cost the same, and that is the
    /// finding rather than a defect in the arm.** This model counts fold steps,
    /// the tiling changes which memory a contributor is read from and nothing
    /// about how many combining steps run, and
    /// `ReductionTopology::CooperativeContraction` says so — one invocation owns
    /// one output position and folds that output's contributors in ascending
    /// contracted order across the whole round loop. A selector that preferred
    /// the tiled plan on fold steps alone would be pricing a memory schedule
    /// through a quantity that cannot see it.
    #[test]
    fn the_tiled_contraction_is_scored_from_its_tile_and_rounds() {
        let tiled = |work_items: u64, contracted: u64, tile: u64, rounds: u64| {
            work_span(
                &schedule(
                    work_items,
                    ReductionTopology::CooperativeContraction {
                        tile: blocked_operand_tile(16, rounds).expect("a representable tile"),
                        contracted_shape: Shape::from_dims([contracted]),
                        contracted_tile: Shape::from_dims([tile]),
                        order: ContributorOrder::OriginalAxisLexicographic,
                        accumulation: ArithmeticType::F32,
                        permits_reassociation: false,
                        permits_permutation: false,
                    },
                ),
                &[],
            )
        };

        assert_eq!(
            tiled(6, 32, 16, 2),
            Some(StageWorkSpan {
                work: 192,
                depth: 32
            }),
            "the tiled arm must fold every contracted point once per output position"
        );
        // The same contracted space at one round: the tile is the whole space,
        // and the two derivations must agree because the topology's admission
        // makes them the same quantity.
        assert_eq!(
            tiled(6, 32, 32, 1),
            Some(StageWorkSpan {
                work: 192,
                depth: 32
            })
        );
        // The negative control the arm exists beside: the direct fold over the
        // same contracted space is still scored, and scores identically.
        assert_eq!(
            work_span(
                &schedule(
                    6,
                    ReductionTopology::Contraction {
                        contracted_shape: Shape::from_dims([32]),
                        order: ContributorOrder::OriginalAxisLexicographic,
                        permits_reassociation: false,
                        permits_permutation: false,
                    }
                ),
                &[]
            ),
            Some(StageWorkSpan {
                work: 192,
                depth: 32
            }),
            "the direct fold must still be scored"
        );
    }

    /// A tiled contraction whose tile and rounds miss its contracted space
    /// declines rather than reporting the number its tiling implies.
    ///
    /// The three facts are related by the topology's own admission — every tile
    /// extent divides its contracted extent and the quotient product is the
    /// round count — so a triple that breaks the relation is malformed, and
    /// costing it from two of the three would state a plausible total for a
    /// schedule that covers a different contracted space. The fail-safe
    /// direction is the one `exact_ratio` already takes.
    #[test]
    fn a_tiled_contraction_whose_rounds_miss_its_contracted_space_declines() {
        let mismatched = |contracted: u64, tile: u64, rounds: u64| {
            work_span(
                &schedule(
                    6,
                    ReductionTopology::CooperativeContraction {
                        tile: blocked_operand_tile(16, rounds).expect("a representable tile"),
                        contracted_shape: Shape::from_dims([contracted]),
                        contracted_tile: Shape::from_dims([tile]),
                        order: ContributorOrder::OriginalAxisLexicographic,
                        accumulation: ArithmeticType::F32,
                        permits_reassociation: false,
                        permits_permutation: false,
                    },
                ),
                &[],
            )
        };
        // Two rounds of sixteen cover 32, not 48.
        assert_eq!(
            mismatched(48, 16, 2),
            None,
            "a short round loop must decline"
        );
        // Three rounds of sixteen cover 48, not 32.
        assert_eq!(
            mismatched(32, 16, 3),
            None,
            "a long round loop must decline"
        );
        // A tile that does not divide the contracted extent at all.
        assert_eq!(
            mismatched(30, 16, 2),
            None,
            "an indivisible tile must decline"
        );
    }

    /// Every reduction topology in the vocabulary states a verdict here.
    ///
    /// **The final arm of `work_span` is required by rustc, not chosen.**
    /// [`ReductionTopology`] is non-exhaustive outside `tiler-ir`, so deleting
    /// the arm fails the build with E0004, "non-exhaustive patterns", whose own
    /// note reads that the type is marked as non-exhaustive so a wildcard is
    /// necessary to match exhaustively. Removing it is not available.
    ///
    /// The hazard it carries is real all the same: a topology added to the
    /// vocabulary falls to that arm and scores `None` with nothing reporting
    /// that it did, and a declined stage declines the whole plan. This census is
    /// what converts the silence into a build error. The array is sized by
    /// `variant_count::<ReductionTopology>()`, exactly as
    /// [`crate::request::BudgetResource::ALL`] is sized from its own enum, so a
    /// widened vocabulary stops compiling here until someone adds the topology
    /// and states its verdict. A hand-written length would be satisfied by a
    /// list that had stopped covering its own enum.
    ///
    /// Declining is a verdict, not an omission: `LiveContraction` is listed with
    /// `None` because its contracted extent is a runtime fact, which is what
    /// [`a_live_contraction_work_span_declines_rather_than_baking_s`] proves.
    #[test]
    fn every_reduction_topology_states_a_verdict() {
        struct Verdict {
            topology: &'static str,
            work_items: u64,
            reduction: ReductionTopology,
            accesses: Vec<Access>,
            expected: Option<StageWorkSpan>,
        }

        let partition = ContributorPartition {
            partitions: 2,
            contributors_per_partition: 2,
        };
        let census: [Verdict; variant_count::<ReductionTopology>()] = [
            Verdict {
                topology: "None",
                work_items: 16,
                reduction: ReductionTopology::None,
                accesses: Vec::new(),
                expected: Some(StageWorkSpan { work: 16, depth: 1 }),
            },
            Verdict {
                topology: "Serial",
                work_items: 1,
                reduction: ReductionTopology::Serial {
                    axes: Vec::new(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: true,
                    permits_permutation: false,
                },
                accesses: vec![contributor_access([1, 4], [1])],
                expected: Some(StageWorkSpan { work: 4, depth: 4 }),
            },
            Verdict {
                topology: "MultiPass",
                work_items: 4,
                reduction: ReductionTopology::MultiPass {
                    pass: ReductionPass::Partial,
                    coverage: ContributorCoverage::Exact(ContributorPartition {
                        partitions: 4,
                        contributors_per_partition: 4,
                    }),
                    axes: Vec::new(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation: true,
                    permits_permutation: false,
                },
                accesses: Vec::new(),
                expected: Some(StageWorkSpan { work: 16, depth: 4 }),
            },
            Verdict {
                topology: "Contraction",
                work_items: 6,
                reduction: ReductionTopology::Contraction {
                    contracted_shape: Shape::from_dims([4]),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                accesses: Vec::new(),
                expected: Some(StageWorkSpan { work: 24, depth: 4 }),
            },
            Verdict {
                topology: "LiveContraction",
                work_items: 6,
                reduction: ReductionTopology::LiveContraction {
                    live_access: AccessOrdinal::FIRST,
                    live_axis: Axis::new(1),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                accesses: Vec::new(),
                expected: None,
            },
            Verdict {
                topology: "CooperativeWorkgroup",
                work_items: 2,
                reduction: cooperative(
                    partition,
                    workgroup_tree_tile(partition.partitions).expect("a representable tile"),
                ),
                accesses: Vec::new(),
                expected: Some(StageWorkSpan { work: 6, depth: 4 }),
            },
            Verdict {
                topology: "CooperativeContraction",
                work_items: 6,
                reduction: ReductionTopology::CooperativeContraction {
                    tile: blocked_operand_tile(16, 2).expect("a representable tile"),
                    contracted_shape: Shape::from_dims([32]),
                    contracted_tile: Shape::from_dims([16]),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                accesses: Vec::new(),
                expected: Some(StageWorkSpan {
                    work: 192,
                    depth: 32,
                }),
            },
        ];

        for verdict in census {
            assert_eq!(
                work_span(
                    &schedule(verdict.work_items, verdict.reduction),
                    &verdict.accesses
                ),
                verdict.expected,
                "{} scored differently from the verdict this census states for it",
                verdict.topology,
            );
        }
    }

    /// Every multi-pass role states a verdict, sized from its own enum.
    ///
    /// The second wildcard in `work_span` is over [`ReductionPass`], which is
    /// non-exhaustive for the same reason and therefore required for the same
    /// reason. Censused separately from the topology above because the two
    /// vocabularies widen independently: a run that reddened both could not say
    /// which arm had stopped covering its domain.
    #[test]
    fn every_reduction_pass_states_a_verdict() {
        let partition = ContributorPartition {
            partitions: 4,
            contributors_per_partition: 4,
        };
        let pass = |pass| ReductionTopology::MultiPass {
            pass,
            coverage: ContributorCoverage::Exact(partition),
            axes: Vec::new(),
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
        };
        let census: [(ReductionPass, u64, StageWorkSpan); variant_count::<ReductionPass>()] = [
            (
                ReductionPass::Partial,
                4,
                StageWorkSpan { work: 16, depth: 4 },
            ),
            (ReductionPass::Final, 1, StageWorkSpan { work: 4, depth: 4 }),
        ];
        for (role, work_items, expected) in census {
            assert_eq!(
                work_span(&schedule(work_items, pass(role)), &[]),
                Some(expected),
                "{role:?} scored differently from the verdict this census states for it",
            );
        }
    }

    /// Both sides of the `max` are summed per stage, never once over the sums.
    ///
    /// Driven on the arithmetic directly, because the distinction is invisible on
    /// a plan whose stages are all bound on the same side: it takes one
    /// work-bound stage beside one span-bound stage, which is exactly what a
    /// prologue feeding a deep fold is.
    #[test]
    fn the_maximum_is_taken_per_stage_and_then_summed() {
        let stages = [
            StageWorkSpan {
                work: 1_000,
                depth: 1,
            },
            StageWorkSpan {
                work: 10,
                depth: 100,
            },
        ];
        let parallel = 8_u64;
        let per_stage: u64 = stages
            .iter()
            .map(|stage| stage.work.max(stage.depth * parallel))
            .sum();
        let work_total: u64 = stages.iter().map(|stage| stage.work).sum();
        let span_total: u64 = stages.iter().map(|stage| stage.depth * parallel).sum();
        assert_eq!(per_stage, 1_800);
        assert_eq!(work_total.max(span_total), 1_010);
        assert_ne!(
            per_stage,
            work_total.max(span_total),
            "the two orders of max and sum agree here, so this case proves nothing"
        );
    }
}
