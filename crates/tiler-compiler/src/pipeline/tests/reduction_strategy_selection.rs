use super::support::{semantic_case_with_axis, test_root};
use super::*;

/// The fitted saturated-fold-step row of the retained 2026-08-07 sweep.
///
/// `spikes/program-planning/reduction-dispatch-crossover` fits
/// `parallel_threads = 1.056e3` on the qualified Apple9 macOS host, and
/// `BoundMetalCompileDeclaration::first_macos_apple9` declares that value.
/// Restated here rather than imported because `tiler-compiler` may not depend on
/// `tiler-build`; that crate's `the_declared_profile_states_the_measured_cost_row`
/// is what keeps the two from drifting.
const MEASURED_SATURATED_FOLD_STEPS: u64 = 1_280;

/// The reduction family the retained sweep measured: an affine prologue feeding
/// a sum over the trailing axis.
fn reduction_family(rows: u64, contributors: u64) -> SemanticProgram {
    semantic_case_with_axis(
        Shape::from_dims([rows, contributors]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    )
}

/// A profile wide enough for all three strategies at every shape below.
///
/// The threadgroup row is the authoritative Apple9 declaration's 32,768 bytes,
/// which is 8,192 participants' worth and far above the capped tree's widest
/// staging here. The grid axis is `2^24`, which is the element cap the retained
/// sweep's own matrix stops at, so no shape below is bounded by the profile
/// rather than by the measurement.
fn three_strategy_target(saturated_parallel_fold_steps: Option<u64>) -> TargetProfile {
    TargetProfile::workgroup_tree_target_with_cost_row_for_test(
        32_768,
        1 << 24,
        Some(crate::target::SynchronizationSupport::Realized),
        saturated_parallel_fold_steps,
    )
}

/// Which reduction strategy a retained alternative realizes.
///
/// Recognized by an observable each strategy alone has, never by a name, which is
/// deliberately the same rule
/// `spikes/program-planning/reduction-dispatch-crossover`,
/// `tiler_build::metal_plan`'s parallel-portfolio fixture, and
/// `prototypes/serial-sum-run` all use — a divergence in what "the tree" means
/// cannot then make two of those claims about different things.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ReductionStrategy {
    /// Two dispatches, one thread per workgroup: the prologue and a serial fold.
    SerialFold,
    /// A cooperative fold whose declared workgroup exceeds one thread.
    SingleWorkgroupTree,
    /// Three dispatches: the multi-pass split.
    MultiPassSplit,
}

impl ReductionStrategy {
    /// Whether this strategy parallelizes the fold, which the retained sweep
    /// found to be the consequential decision on this program family: "a model
    /// that picks the wrong parallel strategy costs a few percent; one that
    /// parallelizes on the wrong side of the contour costs a factor."
    const fn parallelizes(self) -> bool {
        match self {
            Self::SerialFold => false,
            Self::SingleWorkgroupTree | Self::MultiPassSplit => true,
        }
    }
}

fn classify(alternative: &ProgramAlternative) -> ReductionStrategy {
    let widest = alternative
        .scheduled_regions
        .iter()
        .map(|region| region.region().schedule.threads_per_workgroup)
        .max()
        .unwrap_or(1);
    if alternative.scheduled_regions.len() >= 3 {
        ReductionStrategy::MultiPassSplit
    } else if widest > 1 {
        ReductionStrategy::SingleWorkgroupTree
    } else {
        ReductionStrategy::SerialFold
    }
}

/// Compiles the reduction family at one shape and reports what was selected,
/// beside every strategy the portfolio retained.
///
/// The contract is `governed_reassociating`: every parallel strategy regroups the
/// declared contributor sequence, so a contract forbidding reassociation retains
/// none of them and the comparison would have nothing to decide.
fn selected_reduction_strategy(
    semantic: &SemanticProgram,
    profile: TargetProfile,
) -> (ReductionStrategy, Vec<ReductionStrategy>) {
    let mut request = CompilationRequest::governed_under(
        semantic,
        StrictF32NumericalContract::governed_reassociating(),
    );
    request.target_profiles = vec![profile];
    let product = compile(request).expect("the reduction family compiles");
    let target = product.targets[0]
        .compiled()
        .expect("the widened profile compiles the reduction family");
    let selected = target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        })
        .expect("the selection names a retained alternative");
    let mut retained: Vec<ReductionStrategy> =
        target.portfolio.alternatives.iter().map(classify).collect();
    retained.sort_unstable();
    retained.dedup();
    (classify(selected), retained)
}

/// Asserts that a shape retained all three strategies, so a verdict about it is
/// a preference rather than the only surviving option.
fn assert_all_three_retained(retained: &[ReductionStrategy], shape: &str) {
    assert_eq!(
        retained,
        [
            ReductionStrategy::SerialFold,
            ReductionStrategy::SingleWorkgroupTree,
            ReductionStrategy::MultiPassSplit
        ],
        "{shape} did not retain all three strategies, so preferring one proves nothing"
    );
}

/// **The design premise, asserted rather than assumed: the parallel reduction
/// plans are structurally *dominated*, not merely non-dominated.**
///
/// This is the fact that decides what shape a measured term may take. The
/// activating ticket names two correctness constraints, and the second is that
/// selection is a Pareto relation over exact structural counts with a
/// canonical-identity tie break — so the obvious cheap shape for a measured term
/// is a *better tie break inside the non-dominated set*. That shape cannot
/// express the retained measurement at all, and this is why: on the reduction
/// family the non-dominated view holds exactly one plan, so there is no tie to
/// break.
///
/// The serial fold issues no more dispatches, launches strictly fewer threads,
/// and allocates no more temporary storage than either parallel strategy. The
/// frontier-level statement of the same fact is
/// `the_frontier_retains_the_split_beside_the_serial_reduction`, which already
/// recorded that preference belongs to this activation. The 2026-08-07 dispatch
/// sweep measured that same fold costing up to 50.7x the best parallel plan at
/// four rows of 8,192 contributors, so structural dominance and measured speed
/// disagree by two orders of magnitude — and the measured term must therefore be
/// allowed to range over the retained *valid* plans.
///
/// **Watched failing.** Asserting a non-dominated count of two or more fails with
/// the singleton this reports, which is the whole point; and the retained-set
/// assertion beside it is what stops the singleton being an artefact of a
/// portfolio that held one plan to begin with.
#[test]
fn the_parallel_reduction_plans_are_structurally_dominated() {
    let semantic = reduction_family(1, 4_096);
    let mut request = CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_reassociating(),
    );
    request.target_profiles = vec![three_strategy_target(None)];
    let verified = verify_planned_request(request).expect("the contract is admitted");
    let verified = verified
        .for_target(verified.target_profiles()[0])
        .expect("the widened profile resolves the contract");
    let formation = form_region_candidates_with_realizations(
        &semantic,
        verified.budgets(),
        verified.numerical_contract(),
        verified.realization_laws(),
    )
    .expect("region formation succeeds");
    let physical = PhysicalAuthorities::governed();
    let mut explain = ExplainWriter::new(&verified).expect("an explain writer");
    let root = test_root(&mut explain);
    let plans = enumerate_complete_plans(
        &semantic,
        &verified,
        &formation,
        &physical,
        &mut explain,
        root,
        None,
    )
    .expect("the reduction family enumerates complete plans");

    // The portfolio really does hold all three strategies, so the singleton below
    // is a statement about dominance rather than about an empty search.
    assert_eq!(
        plans.portfolio.plans().len(),
        3,
        "the portfolio is not holding all three strategies"
    );
    assert_eq!(
        plans.portfolio.non_dominated().len(),
        1,
        "the structural Pareto view is not a singleton, so a tie break inside it \
         could have carried the measured preference after all"
    );
    // And the survivor is the cheapest structural plan: two dispatches, the
    // fewest launched threads, no partial tensor. That is exactly the serial fold
    // the measurement says is up to 50.7x slower where the row count cannot
    // saturate the device.
    let survivor = plans.portfolio.non_dominated()[0];
    assert_eq!(survivor.cost().dispatch_count(), 2);
    let others: Vec<u64> = plans
        .portfolio
        .plans()
        .iter()
        .filter(|plan| plan.identity() != survivor.identity())
        .map(|plan| plan.cost().launched_threads())
        .collect();
    assert_eq!(others.len(), 2);
    assert!(
        others
            .iter()
            .all(|threads| *threads > survivor.cost().launched_threads()),
        "a parallel plan did not launch strictly more threads than the fold, so \
         the dominance this rests on is not the one described: {others:?} against {}",
        survivor.cost().launched_threads()
    );
}

/// **The silence rule, proved by an unchanged golden.**
///
/// A profile declaring no cost row selects bit-identically to a build without the
/// family at all. Both halves of this test differ in exactly one thing — whether
/// the profile carries the row — so an equal outcome is a statement about the row
/// and nothing else.
///
/// The canonical descriptor is asserted equal too, and that is the stronger half:
/// the descriptor is folded into every artifact identity and cache subject
/// derived from the profile, so a family written unconditionally would move every
/// existing profile's identity to record that it still has no preference.
/// `complete_descriptor` states the derivation that keeps the section conditional.
///
/// The shape is the one the row *does* move — one row of 4,096 contributors,
/// where `perturbing_the_declared_cost_row_moves_the_selected_reduction` shows
/// the declared row selecting a parallel plan — so silence here is silence at a
/// cell where speaking would have changed the answer.
///
/// **Watched failing.** Making the cost-row section unconditional in
/// `complete_descriptor` fails the descriptor assertion; making `measured_scores`
/// score an undeclared row fails the selection assertion by moving the winner to
/// the split.
#[test]
fn a_profile_declaring_no_cost_row_selects_and_encodes_exactly_as_before() {
    let silent = three_strategy_target(None);
    let baseline = TargetProfile::workgroup_tree_target_for_test(
        32_768,
        1 << 24,
        Some(crate::target::SynchronizationSupport::Realized),
    );
    assert_eq!(
        silent.canonical_descriptor(),
        baseline.canonical_descriptor(),
        "declaring no cost row moved the canonical descriptor"
    );
    assert_eq!(
        silent.saturated_parallel_fold_steps(AvailabilityPhase::CompileProfile),
        crate::target::TargetCostRowResolution::Unknown,
        "silence about a cost row must resolve Unknown, never a zero or a refusal"
    );

    let semantic = reduction_family(1, 4_096);
    let (selected, retained) = selected_reduction_strategy(&semantic, silent);
    assert_all_three_retained(&retained, "one row of 4,096 contributors");
    assert_eq!(
        selected,
        ReductionStrategy::SerialFold,
        "a silent profile did not select the structural winner"
    );
}

/// **The mutation proof, on the declared term.**
///
/// Perturbing the declared value changes the selected alternative on a named
/// shape, and the perturbation is the retained sweep's own quarter/quadruple
/// pair: **1,248 rows of 64 contributors, at the re-measured fitted 1,280 and
/// at four times it.** That scale is the one that matters — the 2026-08-18
/// record's own perturbation table shows quartering the row degrading the
/// separated held-out agreement while the fitted value holds it — so a
/// mutation test at a smaller factor would prove the term is read without
/// proving the *measured* value is what decides.
///
/// The shape is where the contour runs. The selector's serial-or-parallel
/// crossing is at `rows * contributors ~ contributors * P`, that is at `rows ~ P`,
/// which is the physics the model asserts: the fold wins exactly where the row
/// count alone already saturates the device. 1,248 rows sits just under the
/// re-measured fitted 1,280, so the verdict is genuinely near the boundary
/// rather than deep in one regime.
///
/// **This cell is deliberately not offered as agreement with the measurement.**
/// Its retained medians are 7.24 microseconds for the fold, 7.61 for the split
/// and 7.93 for the tree, against sample standard deviations near 3 — the sweep's
/// separation rule does not resolve it, and `the_selection_agrees_with_the_retained_sweep`
/// is where agreement is checked, on cells that are separated.
///
/// **Watched failing.** Making `select_non_dominated` ignore the declared value
/// leaves the fold selected at every row and fails the quadrupled assertion.
#[test]
fn perturbing_the_declared_cost_row_moves_the_selected_reduction() {
    let near_contour = reduction_family(1_248, 64);
    let at =
        |steps: u64| selected_reduction_strategy(&near_contour, three_strategy_target(Some(steps)));

    let (fitted, retained) = at(MEASURED_SATURATED_FOLD_STEPS);
    assert_all_three_retained(&retained, "1,248 rows of 64 contributors");
    assert_eq!(
        fitted,
        ReductionStrategy::SerialFold,
        "at the fitted row this shape did not prefer the fold"
    );
    // Quartering leaves it where it was; quadrupling moves it across the contour.
    assert_eq!(
        at(MEASURED_SATURATED_FOLD_STEPS / 4).0,
        ReductionStrategy::SerialFold
    );
    assert!(
        at(MEASURED_SATURATED_FOLD_STEPS * 4).0.parallelizes(),
        "quadrupling the declared row did not move the selected alternative, so \
         the term is being read and discarded"
    );

    // The same mutation on a shape deep in the parallel regime moves nothing,
    // which is what makes the flip above a contour rather than a global switch.
    let deep = reduction_family(1, 4_096);
    for steps in [
        MEASURED_SATURATED_FOLD_STEPS / 4,
        MEASURED_SATURATED_FOLD_STEPS,
        MEASURED_SATURATED_FOLD_STEPS * 4,
    ] {
        let (selected, retained) =
            selected_reduction_strategy(&deep, three_strategy_target(Some(steps)));
        assert_all_three_retained(&retained, "one row of 4,096 contributors");
        assert!(
            selected.parallelizes(),
            "the deep shape stopped parallelizing at a declared row of {steps}"
        );
    }
}

/// **The shapes the new selection prefers, checked against the retained TSV.**
///
/// Two cells of
/// `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/sweep.tsv`,
/// both **separated** by that sweep's own rule — the gap exceeds two combined
/// standard errors of the two medians — and on opposite sides of the contour.
/// The medians are quoted from the retained file rather than re-derived:
///
/// | shape | serial fold | tree | split | measured verdict |
/// | --- | --- | --- | --- | --- |
/// | 1,024 x 4,096 | 250.31 | 203.23 | 207.66 | parallelize |
/// | 16,384 x 32 | 27.57 | 50.16 | 31.91 | do not |
///
/// The selector agrees with both at the fitted row. That is the ticket's
/// "measured shapes the new selection prefers are the ones the retained sweep
/// measured faster, checked against the retained TSV rather than re-argued".
///
/// **One measurement boundary, recorded rather than glossed.** The retained sweep
/// dispatched the tree at `governed_partition`'s balanced split, because
/// `MEASURED_TREE_PARTICIPANT_CAP` landed after it. The compiler now emits the
/// capped width, so at 1,024 x 4,096 the tree it would dispatch is 256
/// participants folding 16 rather than the measured 64 folding 64. That moves
/// which *parallel* plan the selector prefers and not whether it parallelizes,
/// which is exactly the distinction the sweep found consequential — the two
/// parallel strategies are inside each other's noise almost everywhere, and only
/// the binary decision costs a factor. Both assertions below are therefore about
/// the binary decision.
#[test]
fn the_selection_agrees_with_the_retained_sweep() {
    let target = || three_strategy_target(Some(MEASURED_SATURATED_FOLD_STEPS));

    // 1,024 x 4,096 — the fold is measured 1.23x the best parallel plan.
    let parallel_cell = reduction_family(1_024, 4_096);
    let (selected, retained) = selected_reduction_strategy(&parallel_cell, target());
    assert_all_three_retained(&retained, "1,024 rows of 4,096 contributors");
    assert!(
        selected.parallelizes(),
        "the selector kept the fold where the sweep measured it 1.23x slower"
    );

    // 16,384 x 32 — the fold is measured 0.86x the best parallel plan, and the
    // tree costs 1.82x it.
    let serial_cell = reduction_family(16_384, 32);
    let (selected, retained) = selected_reduction_strategy(&serial_cell, target());
    assert_all_three_retained(&retained, "16,384 rows of 32 contributors");
    assert_eq!(
        selected,
        ReductionStrategy::SerialFold,
        "the selector parallelized where the sweep measured the fold faster"
    );
}

/// **The explain report names the deciding term and both sides of the `max`.**
///
/// Not merely `selected`. The measured cost record carries four terms: the
/// declared row itself, the work side of the `max` summed over stages, the span
/// side already scaled by that row so the two are directly comparable, and the
/// per-stage maximum of the two summed — which is what was compared. A reader can
/// see which side decided, and on this shape it is the span: the fold's critical
/// path is the whole 4,096-contributor run, so its span term is 4,096 x the
/// declared row and dwarfs its work.
///
/// **Watched failing.** Dropping either side of the `max` from the record fails
/// the term loop; recording the total without the row fails the term assertion,
/// which is the difference between explaining a verdict and restating it.
#[test]
fn the_selected_alternative_explains_the_measured_term_and_both_max_sides() {
    let semantic = reduction_family(1, 4_096);
    let mut request = CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_reassociating(),
    );
    request.target_profiles = vec![three_strategy_target(Some(MEASURED_SATURATED_FOLD_STEPS))];
    let product = compile(request).expect("the reduction family compiles");
    let target = product.targets[0]
        .compiled()
        .expect("the widened profile compiles");
    let selected_id = target.portfolio.selection.selected_alternative_id.clone();

    // The rendered trace is the report a reader actually reads, so the assertion
    // is over that rather than over the record's private fields: a term present in
    // the structure but absent from the rendering would explain nothing.
    let rendered = target.explain.render();
    let line = rendered
        .lines()
        .find(|line| {
            line.contains("event=cost:tiler.cost.measured-fold-steps.v1:")
                && line.contains(&format!("subject=alternative:{selected_id}"))
        })
        .unwrap_or_else(|| {
            panic!("the selected alternative carries no measured cost record:\n{rendered}")
        });

    // A fitted quantity reported as a checked invariant would be an evidence
    // escalation, so the basis is part of the claim.
    assert!(
        line.contains("cost:tiler.cost.measured-fold-steps.v1:assumption:retained:"),
        "the measured record's basis or disposition moved: {line}"
    );
    // The deciding term, named and carrying the declared value.
    assert!(
        line.contains(&format!(
            "saturated-parallel-fold-steps:operations={MEASURED_SATURATED_FOLD_STEPS}"
        )),
        "the record does not name the deciding term and its value: {line}"
    );
    // Both sides of the `max`, and the total they produced.
    for metric in ["work-steps", "span-steps", "fold-steps"] {
        assert!(
            line.contains(&format!("{metric}:operations=")),
            "the record does not name `{metric}`, so a reader cannot see which \
             side of the max decided: {line}"
        );
    }

    // Every retained alternative is scored, so the comparison is auditable rather
    // than a bare verdict about the winner; the losers carry `higher-cost`.
    let measured: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains("event=cost:tiler.cost.measured-fold-steps.v1:"))
        .collect();
    assert_eq!(
        measured.len(),
        target.portfolio.alternatives.len(),
        "the measured record set does not cover every retained alternative"
    );
    assert_eq!(
        measured
            .iter()
            .filter(|line| line.contains(":assumption:retained:"))
            .count(),
        1,
        "more than one alternative was recorded as the measured winner"
    );
}
