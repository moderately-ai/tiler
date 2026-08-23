use super::support::{
    alternative, plan_formation, plan_portfolio, semantic, semantic_case, tensor_add_chain,
    test_root,
};
use super::*;

#[test]
fn portfolio_selection_and_evidence_are_recomputed_from_exact_contents() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let target = &product.targets[0];
    let alternatives = &target.portfolio.alternatives;
    let selected = target.portfolio.selection.selected_alternative_id.clone();
    let portfolio = plan_portfolio(&semantic, &request);

    assert!(
        verify_portfolio(
            &semantic,
            &request,
            &plan_formation(&semantic, &request),
            &portfolio,
            alternatives,
            &selected,
            None
        )
        .is_ok()
    );
    assert!(
        verify_portfolio(
            &semantic,
            &request,
            &plan_formation(&semantic, &request),
            &portfolio,
            &[],
            &selected,
            None
        )
        .is_err()
    );
    let selection = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        alternatives,
        "stale-selection",
        None,
    )
    .unwrap_err();
    assert_eq!(selection.context.stage, ExplainStage::Selection);
    assert_eq!(
        selection.context.reason.as_str(),
        "structure-portfolio-selection"
    );

    let mut forged = alternatives.clone();
    forged[0].stable_id = "forged-plan".to_owned();
    let identity = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        &forged,
        &selected,
        None,
    )
    .unwrap_err();
    assert_eq!(identity.context.stage, ExplainStage::Costing);

    let mut forged_artifact = alternatives.clone();
    forged_artifact[0].artifact_plan = forged_artifact[1].artifact_plan.clone();
    let artifact = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        &forged_artifact,
        &selected,
        None,
    )
    .unwrap_err();
    assert_eq!(artifact.context.stage, ExplainStage::ArtifactPlanning);

    let mut forged_numerics = alternatives.clone();
    forged_numerics[0].equivalence = forged_numerics[1].equivalence.clone();
    let numerical = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        &forged_numerics,
        &selected,
        None,
    )
    .unwrap_err();
    assert_eq!(numerical.context.stage, ExplainStage::NumericalLegality);
    assert_eq!(
        numerical.context.reason.as_str(),
        "structure-portfolio-equivalence"
    );
}

/// A retained opaque plan reaches the lowering boundary and is refused there,
/// rather than having its absent schedule filtered out.
#[test]
fn lowering_refuses_an_opaque_plan_before_program_assembly() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let formation = plan_formation(&semantic, &request);
    let mut explain = ExplainWriter::new(&request).unwrap();
    let root = test_root(&mut explain);
    let complete = enumerate_complete_plans(
        &semantic,
        &request,
        &formation,
        &PhysicalAuthorities::governed(),
        &mut explain,
        root,
        None,
    )
    .expect("the governed compile enumerates its support evidence");
    let opaque = crate::selection::opaque_fused_portfolio_fixture(&semantic);
    let plan = opaque
        .plans()
        .iter()
        .find(|plan| {
            crate::program::CoverAssembly::from_plan(
                &semantic,
                plan,
                &crate::lowering::ResolvedLowering::unresolved_for_test(),
            )
            .is_err()
        })
        .expect("one opaque plan");

    let error = build_alternative(
        &semantic,
        &request,
        plan,
        ProgramAlternativeKind::Fused,
        &complete,
        None,
    )
    .unwrap_err();
    assert_eq!(error.context.stage, ExplainStage::ProgramVerification);
    assert_eq!(
        error.context.reason.as_str(),
        "structure-unlowerable-opaque-body"
    );
}

/// Verification independently re-derives the schedule binding and refuses a
/// receipt whose selected plan contains an opaque body.
#[test]
fn verification_refuses_an_alternative_with_an_opaque_plan() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let formation = plan_formation(&semantic, &request);
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let mut forged = alternative(&compiled, ProgramAlternativeKind::Fused).clone();
    let opaque = crate::selection::opaque_fused_portfolio_fixture(&semantic);
    let plan = opaque
        .plans()
        .iter()
        .find(|plan| {
            crate::program::CoverAssembly::from_plan(
                &semantic,
                plan,
                &crate::lowering::ResolvedLowering::unresolved_for_test(),
            )
            .is_err()
        })
        .expect("one opaque plan")
        .clone();
    forged.structural_cost = plan.cost();
    forged.plan = plan;
    forged.identity = ProgramAlternativeIdentity::new(
        SemanticAlternativeOrigin::Baseline,
        &semantic,
        &request,
        &forged.plan,
    );
    forged.stable_id = forged.identity.label();

    let lowering = resolve_lowering(&semantic, &request).unwrap();
    let error = super::verify::verify_alternative(
        &semantic, &request, &formation, &forged, &lowering, None,
    )
    .unwrap_err();
    assert_eq!(error.context.stage, ExplainStage::ProgramVerification);
    assert_eq!(
        error.context.reason.as_str(),
        "structure-portfolio-schedule-binding"
    );
}

#[test]
fn global_semantic_selection_rejects_a_forged_winner() {
    let semantic = semantic(false);
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let mut portfolio = compiled.targets[0].portfolio.clone();
    let forged = portfolio
        .alternatives
        .iter()
        .find(|alternative| alternative.stable_id != portfolio.selection.selected_alternative_id)
        .expect("the fixture retains a non-selected physical alternative")
        .stable_id
        .clone();
    portfolio.selection.selected_alternative_id = forged;

    let error = verify_global_selection(&portfolio, &TargetProfile::governed()).unwrap_err();
    assert!(matches!(
        error,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
            ProgramError::Structure {
                rule: "semantic-portfolio-selection"
            }
        ))
    ));
}

#[test]
fn final_portfolio_verifier_rejects_deletion_owner_and_origin_misbinding() {
    let semantic = semantic(false);
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let portfolio = compiled.targets[0].portfolio.clone();
    let expected_identities = portfolio
        .alternatives
        .iter()
        .map(|alternative| alternative.identity.clone())
        .collect();
    let expected = [ExpectedCandidateOwner {
        key: "semantic:baseline".to_owned(),
        origin: SemanticAlternativeOrigin::Baseline,
        semantic: &semantic,
        request: request.clone(),
        alternatives: expected_identities,
    }];
    assert!(verify_global_portfolio(&portfolio, &expected, &TargetProfile::governed()).is_ok());

    let mut deleted = portfolio.clone();
    deleted.alternatives.pop();
    assert!(matches!(
        verify_global_portfolio(&deleted, &expected, &TargetProfile::governed()),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "semantic-portfolio-owner-set"
            })
        ))
    ));

    let mut misowned = portfolio.clone();
    misowned.alternatives[0].owner_key = "semantic:wrong-owner".to_owned();
    assert!(matches!(
        verify_global_portfolio(&misowned, &expected, &TargetProfile::governed()),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "semantic-portfolio-owner-binding"
            })
        ))
    ));

    let wrong_origin = RewriteRuleIdentity::new("test", "wrong-origin", 1).unwrap();
    let wrong_expected = [ExpectedCandidateOwner {
        key: "semantic:baseline".to_owned(),
        origin: SemanticAlternativeOrigin::Rewrite(wrong_origin),
        semantic: &semantic,
        request,
        alternatives: portfolio
            .alternatives
            .iter()
            .map(|alternative| alternative.identity.clone())
            .collect(),
    }];
    assert!(matches!(
        verify_global_portfolio(&portfolio, &wrong_expected, &TargetProfile::governed()),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "semantic-portfolio-owner-binding"
            })
        ))
    ));
}

fn owner_binding_text(error: &CompileError) -> String {
    error.to_string()
}

/// A same-shape serial-sum program whose constant payloads differ from
/// [`semantic`], so the complete semantic identity moves.
fn distinct_semantic() -> SemanticProgram {
    semantic_case(
        Shape::from_dims([2, 2]),
        3.0_f32.to_bits(),
        4.0_f32.to_bits(),
        false,
    )
}

fn baseline_owner<'a>(
    semantic: &'a SemanticProgram,
    request: crate::request::VerifiedTargetRequest,
    portfolio: &ProgramPortfolio,
) -> ExpectedCandidateOwner<'a> {
    ExpectedCandidateOwner {
        key: "semantic:baseline".to_owned(),
        origin: SemanticAlternativeOrigin::Baseline,
        semantic,
        request,
        alternatives: portfolio
            .alternatives
            .iter()
            .map(|alternative| alternative.identity.clone())
            .collect(),
    }
}

/// Perturbing only the retained program, leaving owner key and identity, must
/// fail closed. The existing owner-key and origin checks cannot see this.
#[test]
fn final_portfolio_verifier_rejects_an_independently_perturbed_retained_candidate() {
    let program = semantic(false);
    let other = distinct_semantic();
    assert_ne!(
        program.semantic_identity(),
        other.semantic_identity(),
        "the perturbation must change the complete semantic identity"
    );
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let compiled = compile(CompilationRequest::governed(&program)).unwrap();
    let portfolio = compiled.targets[0].portfolio.clone();
    let expected = [baseline_owner(&program, request, &portfolio)];
    assert!(verify_global_portfolio(&portfolio, &expected, &TargetProfile::governed()).is_ok());

    let mut perturbed = portfolio;
    let original_key = perturbed.alternatives[0].owner_key.clone();
    let original_identity = perturbed.alternatives[0].identity.clone();
    perturbed.alternatives[0].semantic = other;
    assert_eq!(perturbed.alternatives[0].owner_key, original_key);
    assert_eq!(perturbed.alternatives[0].identity, original_identity);

    let error = verify_global_portfolio(&perturbed, &expected, &TargetProfile::governed())
        .expect_err("an independently swapped candidate must fail owner-binding");
    assert!(
        matches!(
            error,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "semantic-portfolio-owner-binding"
                }
            ))
        ),
        "{error:?}"
    );
    assert_eq!(
        owner_binding_text(&error),
        "program.structure.semantic-portfolio-owner-binding: rejected"
    );
}

/// Two compiler-minted candidates with distinct semantic identities cannot be
/// swapped onto each other's retained alternatives.
#[test]
fn final_portfolio_verifier_rejects_swapped_retained_candidates() {
    let semantic = tensor_add_chain();
    let rewritten = explore_algebraic_alternatives_owned(
        semantic.clone(),
        crate::request::DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed_relaxed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap()
    .alternatives()
    .iter()
    .find(|alternative| alternative.rule() == crate::rewrite::ORDERED_REASSOCIATE_ADD_RULE.unwrap())
    .expect("the relaxed contract admits the add reassociation")
    .candidate()
    .clone();
    assert_ne!(
        semantic.semantic_identity(),
        rewritten.semantic_identity(),
        "the swapped pair must differ in complete semantic identity"
    );
    let verified = verify_planned_request(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let compiled = compile(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let mut portfolio = compiled.targets[0].portfolio.clone();
    let baseline = portfolio
        .alternatives
        .iter()
        .position(|alternative| alternative.owner_key == "semantic:baseline")
        .expect("the baseline owner retains a physical alternative");
    let rewrite = portfolio
        .alternatives
        .iter()
        .position(|alternative| {
            alternative
                .owner_key
                .contains("ordered-reassociate-add-f32.v1")
        })
        .expect("the rewrite owner retains a physical alternative");
    let rewrite_key = portfolio.alternatives[rewrite].owner_key.clone();
    let expected = [
        ExpectedCandidateOwner {
            key: "semantic:baseline".to_owned(),
            origin: SemanticAlternativeOrigin::Baseline,
            semantic: &semantic,
            request: request.clone(),
            alternatives: portfolio
                .alternatives
                .iter()
                .filter(|alternative| alternative.owner_key == "semantic:baseline")
                .map(|alternative| alternative.identity.clone())
                .collect(),
        },
        ExpectedCandidateOwner {
            key: rewrite_key,
            origin: SemanticAlternativeOrigin::Rewrite(
                crate::rewrite::ORDERED_REASSOCIATE_ADD_RULE.unwrap(),
            ),
            semantic: &rewritten,
            request,
            alternatives: portfolio
                .alternatives
                .iter()
                .filter(|alternative| {
                    alternative
                        .owner_key
                        .contains("ordered-reassociate-add-f32.v1")
                })
                .map(|alternative| alternative.identity.clone())
                .collect(),
        },
    ];
    assert!(verify_global_portfolio(&portfolio, &expected, &TargetProfile::governed()).is_ok());

    let left = portfolio.alternatives[baseline].semantic.clone();
    let right = portfolio.alternatives[rewrite].semantic.clone();
    portfolio.alternatives[baseline].semantic = right;
    portfolio.alternatives[rewrite].semantic = left;

    let error = verify_global_portfolio(&portfolio, &expected, &TargetProfile::governed())
        .expect_err("swapping two retained candidates must fail owner-binding");
    assert!(
        matches!(
            error,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "semantic-portfolio-owner-binding"
                }
            ))
        ),
        "{error:?}"
    );
    assert_eq!(
        owner_binding_text(&error),
        "program.structure.semantic-portfolio-owner-binding: rejected"
    );
}

/// Construction-time verification refuses an alternative whose retained
/// candidate was dropped and replaced after `build_alternative` minted it.
#[test]
fn construction_path_refuses_a_dropped_retained_candidate() {
    let program = semantic(false);
    let other = distinct_semantic();
    assert_ne!(program.semantic_identity(), other.semantic_identity());
    let request = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let compiled = compile(CompilationRequest::governed(&program)).unwrap();
    let mut dropped = alternative(&compiled, ProgramAlternativeKind::Fused).clone();
    dropped.semantic = other;
    let formation = plan_formation(&program, &request);
    let lowering = resolve_lowering(&program, &request).unwrap();

    let error = super::verify::verify_alternative(
        &program, &request, &formation, &dropped, &lowering, None,
    )
    .expect_err("dropping the retained candidate on a construction receipt must fail");
    assert_eq!(error.context.stage, ExplainStage::ProgramVerification);
    assert_eq!(
        error.context.reason.as_str(),
        "structure-portfolio-retained-semantic-binding"
    );
    assert_eq!(
        error.source.to_string(),
        "program.structure.portfolio-retained-semantic-binding: rejected"
    );
}

/// Rebuilding only the outer identity after changing the retained program still
/// fails: the owner binding re-derives the candidate's complete semantic
/// identity rather than trusting the restored bytes.
#[test]
fn final_portfolio_verifier_rejects_a_retained_program_change_that_restores_only_outer_identity() {
    let program = semantic(false);
    let other = distinct_semantic();
    assert_ne!(program.semantic_identity(), other.semantic_identity());
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let compiled = compile(CompilationRequest::governed(&program)).unwrap();
    let portfolio = compiled.targets[0].portfolio.clone();
    let expected = [baseline_owner(&program, request.clone(), &portfolio)];

    let mut forged = portfolio;
    let original_identity = forged.alternatives[0].identity.clone();
    let original_stable = forged.alternatives[0].stable_id.clone();
    forged.alternatives[0].semantic = other.clone();
    forged.alternatives[0].identity = ProgramAlternativeIdentity::new(
        SemanticAlternativeOrigin::Baseline,
        &other,
        &request,
        &forged.alternatives[0].plan,
    );
    forged.alternatives[0].stable_id = forged.alternatives[0].identity.label();
    forged.alternatives[0].identity = original_identity;
    forged.alternatives[0].stable_id = original_stable;

    let error = verify_global_portfolio(&forged, &expected, &TargetProfile::governed())
        .expect_err("restoring only the outer identity must not hide a candidate swap");
    assert!(
        matches!(
            error,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "semantic-portfolio-owner-binding"
                }
            ))
        ),
        "{error:?}"
    );
    assert_eq!(
        owner_binding_text(&error),
        "program.structure.semantic-portfolio-owner-binding: rejected"
    );
}

/// Two independently built programs of one meaning compare equal on a retained
/// alternative. Pointer equality on the `Arc` would fail this.
#[test]
fn program_alternative_equality_uses_verified_semantic_identity() {
    let program = semantic(false);
    let compiled = compile(CompilationRequest::governed(&program)).unwrap();
    let mut left = compiled.targets[0].portfolio.alternatives[0].clone();
    let mut right = left.clone();
    left.semantic = semantic(false);
    right.semantic = semantic(false);
    assert_eq!(
        left.semantic.semantic_identity(),
        right.semantic.semantic_identity()
    );
    assert_eq!(left, right);

    right.semantic = distinct_semantic();
    assert_ne!(
        left.semantic.semantic_identity(),
        right.semantic.semantic_identity()
    );
    assert_ne!(left, right);
}

/// Retention is session evidence. The alternative identity domain, selected-plan
/// identity, and packaged program identity stay on the bytes they already minted.
#[test]
fn retaining_the_candidate_does_not_move_canonical_identities() {
    let program = semantic(false);
    let first = compile(CompilationRequest::governed(&program)).unwrap();
    let second = compile(CompilationRequest::governed(&program)).unwrap();
    let first = &first.targets[0].portfolio.alternatives;
    let second = &second.targets[0].portfolio.alternatives;
    assert_eq!(first.len(), second.len());
    for (left, right) in first.iter().zip(second.iter()) {
        assert_eq!(left.identity, right.identity);
        assert_eq!(left.plan.identity(), right.plan.identity());
        assert_eq!(
            left.program.core().canonical_identity().as_bytes(),
            right.program.core().canonical_identity().as_bytes()
        );
        assert_eq!(
            left.artifact_plan
                .verified_program()
                .core()
                .canonical_identity()
                .as_bytes(),
            right
                .artifact_plan
                .verified_program()
                .core()
                .canonical_identity()
                .as_bytes()
        );
        assert_eq!(left.stable_id, right.stable_id);
        assert_eq!(
            left.semantic.semantic_identity(),
            program.semantic_identity()
        );
    }
    let labels: Vec<&str> = first
        .iter()
        .map(|alternative| alternative.stable_id.as_str())
        .collect();
    assert_eq!(
        labels,
        [
            "program-alternative:c527a8ac5399e781",
            "program-alternative:d0e6fb8b6fa9ea68",
        ],
        "successful plan identities must not move when retention is added"
    );
}
