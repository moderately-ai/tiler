use super::support::{
    region_subject_key, selected_kind, semantic, semantic_case, semantic_case_with_axis,
};
use super::*;

/// A search bound costs alternatives and leaves the compilation a plan.
///
/// The bounded profile implements no singleton region for this program, so the
/// whole-program region is the only implementable cover it has. A zero per-seed
/// growth budget therefore used to refuse the program outright, which is the
/// defect `region-expansion-exhaustion-loses-the-only-feasible-plan` reports: a
/// bound documented to cost an alternative cost the only plan. Region formation
/// retains both coverage extremes before growth starts, so the fused plan
/// survives the stop while the stop stays on the trace naming what it did cost.
#[test]
fn a_search_budget_costs_alternatives_and_never_the_only_plan() {
    let semantic = semantic(false);
    let mut bounded = CompilationRequest::governed(&semantic);
    bounded.budgets.region_candidates_per_seed = 0;
    let product = compile(bounded).expect("the fused plan survives an exhausted search bound");
    assert!(product.targets[0].failure().is_none());
    assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
    assert_eq!(selected_kind(&product), ProgramAlternativeKind::Fused);
    let explain = &product.targets[0].explain;
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "region.formation.v1"
            && record.event().disposition() == ExplainDisposition::BudgetStopped
    }));
    // The five singletons and the whole-program region: coverage, and nothing
    // the exhausted per-seed bound would have discovered between them.
    assert_eq!(
        explain
            .records()
            .iter()
            .filter(|record| record.rule().key().as_str() == "region.candidate.v1")
            .count(),
        6
    );
}

/// A bound on one region's *shape* can refuse the program, and says which.
///
/// `region_members` bounds no search: it declares the largest region this
/// profile admits, and a program whose only implementable cover needs a larger
/// one has no plan under it. That is an exhausted deterministic budget and never
/// a target's verdict, so it carries `BudgetExhausted` naming the bound to widen
/// rather than `NoFeasiblePlan`. That class retains hard target refusals and
/// conservative mixed or structural empty portfolios, while neither class can
/// turn a budget-truncated search into a verdict about the program.
///
/// **The budget is stated rather than governed, and since
/// `derive-the-region-shape-budgets-from-the-declaration` that is the only way
/// this path is reachable at all.** `region_members` is now the same formula
/// `semantic_operations` is, so a program large enough for the governed bound
/// to truncate its analysis is refused for its *size* by
/// `check_program_budgets` first. This test is therefore what keeps the empty
/// portfolio's `BudgetExhausted` measured; `crate::session`'s reachability
/// inventory cites it for exactly that.
#[test]
fn a_region_shape_budget_below_the_only_implementable_cover_reports_the_budget() {
    let semantic = semantic(false);
    let mut bounded = CompilationRequest::governed(&semantic);
    bounded.budgets.region_members = 1;
    let product = compile(bounded).expect("a target-local refusal is an ordered outcome");
    let CompileError::Explained { source, explain } = product.targets[0]
        .failure()
        .expect("the bounded target has no complete plan")
    else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(
        matches!(
            source.as_ref(),
            CompileError::BudgetExhausted(RequestError::BudgetExceeded {
                resource: BudgetResource::RegionMembers,
                limit: 1,
                ..
            })
        ),
        "the refusal names the bound whose widening would change the answer: {source:?}"
    );
    let failure = explain
        .records()
        .iter()
        .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
        .expect("a terminal failure record");
    assert!(matches!(
        failure.event(),
        ExplainEvent::CompilerFailure {
            stage: ExplainStage::Selection,
            reason,
        } if reason.as_str() == "portfolio-empty-after-budget-stop"
    ));
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "region.formation.v1"
            && record.event().disposition() == ExplainDisposition::BudgetStopped
    }));
}

/// A cover budget never loses the two covers the enumerator retains
/// unconditionally — the all-singleton and the whole-program cover — and any
/// discovered partition it does lose is reported as a typed budget stop.
///
/// The bounded profile implements no singleton region, so the all-singleton
/// cover yields no plan. Losing the discovered two-region partition therefore
/// costs the materialized alternative, which is exactly what the typed stop
/// makes visible instead of silently narrowing the portfolio.
#[test]
fn cover_budget_stops_are_reported_without_losing_either_extreme() {
    let semantic = semantic(false);
    let mut bounded = CompilationRequest::governed(&semantic);
    bounded.budgets.region_covers = 1;
    let product = compile(bounded).unwrap();
    assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
    assert_eq!(selected_kind(&product), ProgramAlternativeKind::Fused);
    assert!(product.targets[0].explain.records().iter().any(|record| {
        record.rule().key().as_str() == "cover.enumeration.v1"
            && record.event().disposition() == ExplainDisposition::BudgetStopped
    }));
}

#[test]
fn infeasible_baseline_does_not_suppress_a_feasible_fused_plan() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([70_000, 2]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(0),
    );

    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let target = &product.targets[0];
    assert_eq!(target.portfolio.alternatives.len(), 1);
    assert_eq!(
        target.portfolio.alternatives[0].kind,
        ProgramAlternativeKind::Fused
    );
    let pointwise = region_subject_key(&target.explain, "pointwise")
        .expect("the pointwise region subject reached the frontier");
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "target.grid-axis"
            && record.subjects()[0].key().as_str() == pointwise
            && record.event().disposition() == ExplainDisposition::RejectedTarget
            && matches!(
                record.event(),
                ExplainEvent::Feasibility {
                    required: Quantity::Threads(140_000),
                    available: Quantity::Threads(4),
                    ..
                }
            )
    }));
    // The cover whose pointwise region the target refused is retained in the
    // terminal ledger as an infeasible alternative rather than disappearing.
    assert!(target.explain.records().iter().any(|record| {
        matches!(
            record.event(),
            ExplainEvent::Selection {
                outcome: SelectionOutcome::Infeasible,
                ..
            }
        )
    }));
}

#[test]
fn the_governed_grid_authority_admits_four_and_refuses_five() {
    let bounded = semantic_case(
        Shape::from_dims([4, 1]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
    );
    let accepted = compile(CompilationRequest::governed(&bounded))
        .expect("the governed four-thread serial sum compiles");
    assert!(accepted.targets[0].compiled().is_some());

    let oversized = semantic_case(
        Shape::from_dims([5, 1]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
    );
    let refused = compile(CompilationRequest::governed(&oversized))
        .expect("a target-local refusal remains an ordered compilation outcome");
    let CompileError::Explained { source, explain } = refused.targets[0]
        .failure()
        .expect("the five-thread target is refused")
    else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        source.as_ref(),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target { .. }))
    ));
    assert!(explain.records().iter().any(|record| {
        matches!(
            record.event(),
            ExplainEvent::Feasibility {
                predicate,
                required: Quantity::Threads(5),
                available: Quantity::Threads(4),
                ..
            } if predicate.as_str() == "grid-axis"
        )
    }));
}

#[test]
fn no_feasible_plan_retains_a_typed_terminal_failure_trace() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([70_000, 70_000]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let product = compile(CompilationRequest::governed(&semantic))
        .expect("a target-local refusal is an ordered outcome");
    let CompileError::Explained { source, explain } = product.targets[0]
        .failure()
        .expect("the target has no feasible plan")
    else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        source.as_ref(),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target { .. }))
    ));
    assert_eq!(
        explain
            .records()
            .iter()
            .filter(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .count(),
        1
    );
    let failure = explain
        .records()
        .iter()
        .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
        .unwrap();
    assert!(matches!(
        failure.event(),
        ExplainEvent::CompilerFailure {
            stage: ExplainStage::TargetFeasibility,
            reason,
        } if reason.as_str() == "target-grid-axis"
    ));
    let causal_rejections = failure
        .causes()
        .iter()
        .map(|cause| {
            explain
                .records()
                .iter()
                .find(|record| record.id() == *cause)
                .expect("every failure cause is a retained exact target rejection")
        })
        .collect::<Vec<_>>();
    assert!(!causal_rejections.is_empty());
    assert!(
        causal_rejections
            .iter()
            .all(|record| { record.event().disposition() == ExplainDisposition::RejectedTarget })
    );
    // Every recognized region the target refused is named exactly once, by the
    // region's own explain subject rather than by its role: the roles below
    // resolve to three distinct occurrence labels, and a rejection keyed by role
    // could not have told them apart from the eleven other subjects this
    // program covers.
    let mut subjects = causal_rejections
        .iter()
        .map(|record| record.subjects()[0].key().as_str().to_owned())
        .collect::<Vec<_>>();
    subjects.sort();
    let mut expected = ["pointwise", "reduction", "whole-program"]
        .into_iter()
        .map(|role| {
            region_subject_key(explain, role)
                .unwrap_or_else(|| panic!("the {role} region subject reached the frontier"))
        })
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(subjects, expected);
}
