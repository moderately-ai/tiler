use super::support::{semantic, tensor_add_chain, test_root};
use super::*;

fn algebraic_add_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let leaves = [1.0e20_f32, -1.0e20, 1.0]
        .map(|value| F32Constant::apply(&mut builder, value.to_bits()).unwrap());
    let left = F32Add::apply(&mut builder, leaves[0], leaves[1]).unwrap();
    let root = F32Add::apply(&mut builder, left, leaves[2]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn live_semantic_portfolio_explains_every_governed_rule_decline_stably() {
    let semantic = semantic(false);
    let first = compile(CompilationRequest::governed(&semantic)).unwrap();
    let second = compile(CompilationRequest::governed(&semantic)).unwrap();
    let first = first.targets[0].compilation_explain.render();
    let second = second.targets[0].compilation_explain.render();

    assert_eq!(first, second);
    for rule in [
        "ordered-reassociate-add-f32.v1",
        "ordered-reassociate-multiply-f32.v1",
    ] {
        assert!(
            first.contains(rule),
            "the complete rule identity must remain visible when the rule declines"
        );
    }
    assert!(first.contains("disproved:semantic.no-left-associated-chain"));
}

#[test]
fn relaxed_reassociation_reaches_verified_global_physical_selection() {
    let semantic = tensor_add_chain();
    let product = compile(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let target = &product.targets[0];

    assert!(
        target.portfolio.alternatives.iter().any(|alternative| {
            alternative.owner_key == "semantic:baseline" && alternative.program.stage_count() == 1
        }),
        "the unchanged semantic baseline remains physically available",
    );
    let reassociated = target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative
                .owner_key
                .contains("ordered-reassociate-add-f32.v1")
                && alternative.program.stage_count() == 1
        })
        .expect("the accepted reassociation reaches a verified program under its own owner");
    assert_eq!(
        reassociated.scheduled_regions[0].semantic_members(),
        [
            crate::region::SemanticStage::first(crate::region::SemanticMemberId(0)),
            crate::region::SemanticStage::first(crate::region::SemanticMemberId(1)),
            crate::region::SemanticStage::first(crate::region::SemanticMemberId(2)),
            crate::region::SemanticStage::first(crate::region::SemanticMemberId(3)),
        ],
    );
    assert_eq!(reassociated.equivalence.legality().len(), 1);
    let exploration = explore_algebraic_alternatives_owned(
        semantic.clone(),
        crate::request::DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed_relaxed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap();
    let rewritten = exploration
        .alternatives()
        .iter()
        .find(|alternative| {
            alternative.rule() == crate::rewrite::ORDERED_REASSOCIATE_ADD_RULE.unwrap()
        })
        .expect("the relaxed contract admits the add reassociation")
        .candidate();
    let rewritten_request = verify_planned_request(CompilationRequest::governed_under(
        rewritten,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let rewritten_request = rewritten_request
        .for_target(rewritten_request.target_profiles()[0])
        .unwrap();
    let lowering = resolve_lowering(rewritten, &rewritten_request).unwrap();
    assert_eq!(
        lowering
            .occurrences()
            .iter()
            .map(crate::lowering::OccurrenceLowering::member)
            .collect::<Vec<_>>(),
        [
            crate::region::SemanticMemberId(0),
            crate::region::SemanticMemberId(1),
            crate::region::SemanticMemberId(2),
            crate::region::SemanticMemberId(3),
        ],
        "the rewritten program resolves all four semantic occurrences",
    );
    assert!(
        lowering
            .occurrences()
            .iter()
            .all(|occurrence| matches!(occurrence.evidence(), OccurrenceEvidence::Refined(_))),
        "each rewritten occurrence carries checked refinement evidence",
    );
    assert!(
        target.portfolio.alternatives.iter().any(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        }),
        "global selection names one verified flattened alternative",
    );
}

#[test]
fn pointwise_region_roles_require_the_exact_whole_program_subject() {
    let semantic = tensor_add_chain();
    let verified = verify_planned_request(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let members = [
        crate::region::SemanticStage::first(crate::region::SemanticMemberId(0)),
        crate::region::SemanticStage::first(crate::region::SemanticMemberId(1)),
        crate::region::SemanticStage::first(crate::region::SemanticMemberId(2)),
        crate::region::SemanticStage::first(crate::region::SemanticMemberId(3)),
    ];

    assert_eq!(region_role(&request, &members), "whole-program");
    for member in members {
        assert_eq!(region_role(&request, &[member]), "unrecognized");
    }
}

#[test]
fn strict_contract_keeps_the_pointwise_baseline_and_declines_reassociation() {
    let semantic = tensor_add_chain();
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let target = &product.targets[0];

    assert!(
        target
            .portfolio
            .alternatives
            .iter()
            .all(|alternative| alternative.owner_key == "semantic:baseline"),
    );
    assert!(
        target
            .compilation_explain
            .render()
            .contains("numerical.reassociation-forbidden"),
    );
}

#[test]
fn live_semantic_portfolio_renders_per_rule_disablement() {
    let semantic = semantic(false);
    let add = crate::rewrite::ORDERED_REASSOCIATE_ADD_RULE.unwrap();
    let configuration = AlgebraicRuleConfiguration::all().with(add, false);
    let product = compile_configured(
        CompilationRequest::governed(&semantic),
        configuration,
        &PhysicalAuthorities::governed(),
    )
    .unwrap();
    let rendered = product.targets[0].compilation_explain.render();

    assert!(
        rendered.contains("rewrite.configuration-enabled:disproved:configuration.rule-disabled")
    );
    assert!(rendered.contains("rewrite-provider:identity=tiler.algebraic"));
    assert!(rendered.contains("rewrite-rule:identity=ordered-reassociate-add-f32.v1"));
    assert!(rendered.contains("rewrite-revision:count=1"));
    assert!(
        rendered.contains("ordered-reassociate-multiply-f32.v1"),
        "disabling add must not remove multiply's independent assessment"
    );
}

#[test]
fn top_level_emitter_renders_strict_numerical_decline_and_algebraic_budget_stop() {
    let chain = algebraic_add_chain();
    let strict = crate::normalize::explore_algebraic_alternatives_owned(
        chain.clone(),
        crate::request::DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap();
    let AlgebraicExplorationParts { assessments, .. } = strict.into_parts();
    let binding = semantic(false);
    let verified = verify_planned_request(CompilationRequest::governed(&binding)).unwrap();
    let target = verified.for_target(verified.target_profiles()[0]).unwrap();
    let mut writer = ExplainWriter::new(&target).unwrap();
    let root = test_root(&mut writer);
    record_algebraic_exploration(&mut writer, root, &assessments, None, &[]).unwrap();
    let alternative = writer
        .subject(SubjectKind::Alternative, "alternative:test")
        .unwrap();
    writer
        .note_selection(alternative, SelectionOutcome::Selected, None)
        .unwrap();
    let strict = writer
        .finish_success(&["alternative:test"], "alternative:test")
        .unwrap()
        .render();
    assert!(strict.contains("rewrite.semantic-applicable:proven"));
    assert!(
        strict.contains("rewrite.numerically-legal:disproved:numerical.reassociation-forbidden")
    );
    assert!(strict.contains("rewrite-rule:identity=ordered-reassociate-add-f32.v1"));
    assert!(strict.contains("rewrite-revision:count=1"));

    let mut budgets = crate::request::DeterministicBudgets::governed();
    budgets.normalization_rewrites = 0;
    let stopped = crate::normalize::explore_algebraic_alternatives_owned(
        chain,
        budgets,
        StrictF32NumericalContract::governed_relaxed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap();
    let AlgebraicExplorationParts {
        assessments,
        budget_stop,
        ..
    } = stopped.into_parts();
    let mut writer = ExplainWriter::new(&target).unwrap();
    let root = test_root(&mut writer);
    record_algebraic_exploration(&mut writer, root, &assessments, budget_stop, &[]).unwrap();
    let alternative = writer
        .subject(SubjectKind::Alternative, "alternative:test")
        .unwrap();
    writer
        .note_selection(alternative, SelectionOutcome::Selected, None)
        .unwrap();
    let stopped = writer
        .finish_success(&["alternative:test"], "alternative:test")
        .unwrap()
        .render();
    assert!(stopped.contains("budget-stop:normalization-rewrites:0:1"));
}
