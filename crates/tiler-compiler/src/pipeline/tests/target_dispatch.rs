use super::support::{outcome_for_key, request_with_targets, semantic};
use super::*;

#[test]
fn budget_exhaustion_is_not_reported_as_unsupported() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.budgets.semantic_operations = 4;
    let error = compile(request).unwrap_err();
    assert_eq!(
        error,
        CompileError::BudgetExhausted(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticOperations,
            limit: 4,
            reported: 5,
        })
    );
}

#[test]
fn malformed_request_is_not_reported_as_missing_capability() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.target_profiles.clear();
    assert_eq!(
        compile(request),
        Err(CompileError::InvalidRequest(RequestError::EmptyTargetSet))
    );
}

#[test]
fn target_outcomes_preserve_caller_order_in_both_directions() {
    let semantic = semantic(false);
    let success = TargetProfile::governed_with_key_for_test("test.success.v1");
    let no_contract = TargetProfile::without_numerical_declarations_for_test("test.no-contract.v1");
    for profiles in [
        vec![success.clone(), no_contract.clone()],
        vec![no_contract.clone(), success.clone()],
    ] {
        let expected_keys = profiles
            .iter()
            .map(|profile| profile.profile_key().as_str().to_owned())
            .collect::<Vec<_>>();
        let product = compile(request_with_targets(
            &semantic,
            profiles,
            vec![StrictF32NumericalContract::governed()],
        ))
        .expect("a target-local numerical refusal does not fail the batch");
        assert_eq!(
            product
                .targets
                .iter()
                .map(|outcome| outcome.target_profile().profile_key().as_str())
                .collect::<Vec<_>>(),
            expected_keys.iter().map(String::as_str).collect::<Vec<_>>()
        );
        assert!(
            outcome_for_key(&product, "test.success.v1")
                .compiled()
                .is_some()
        );
        assert!(matches!(
            outcome_for_key(&product, "test.no-contract.v1").failure(),
            Some(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
                RequestError::NoResolvableNumericalContract { .. }
            )))
        ));
    }
}

#[test]
fn target_identity_is_independent_of_batch_order() {
    let semantic = semantic(false);
    let success = TargetProfile::governed_with_key_for_test("test.identity.v1");
    let no_contract = TargetProfile::without_numerical_declarations_for_test("test.companion.v1");
    let compile_order = |profiles| {
        compile(request_with_targets(
            &semantic,
            profiles,
            vec![StrictF32NumericalContract::governed()],
        ))
        .unwrap()
    };
    let forward = compile_order(vec![success.clone(), no_contract.clone()]);
    let reverse = compile_order(vec![no_contract, success]);
    assert_eq!(
        outcome_for_key(&forward, "test.identity.v1").compiled(),
        outcome_for_key(&reverse, "test.identity.v1").compiled()
    );
}

#[test]
fn distinct_resolved_contracts_are_compiled_as_two_groups() {
    let semantic = semantic(false);
    let strict = TargetProfile::governed_with_key_for_test("test.strict.v1");
    let flush = TargetProfile::flush_only_for_test("test.flush.v1");
    let (result, group_count) = observe_contract_group_compilations(|| {
        compile(request_with_targets(
            &semantic,
            vec![strict, flush],
            vec![
                StrictF32NumericalContract::governed(),
                StrictF32NumericalContract::governed_flush_to_zero(),
            ],
        ))
    });
    let product = result.unwrap();
    assert_eq!(group_count, 2);
    assert_eq!(
        outcome_for_key(&product, "test.strict.v1")
            .compiled()
            .unwrap()
            .resolved_contract,
        StrictF32NumericalContract::governed()
    );
    assert_eq!(
        outcome_for_key(&product, "test.flush.v1")
            .compiled()
            .unwrap()
            .resolved_contract,
        StrictF32NumericalContract::governed_flush_to_zero()
    );
}

#[test]
fn one_target_failure_does_not_erase_a_companion_in_the_same_group() {
    let semantic = semantic(false);
    let success = TargetProfile::governed_with_key_for_test("test.isolation.success.v1");
    let bounded = TargetProfile::with_grid_axis_limit_for_test("test.isolation.bounded.v1", 1);
    let (result, group_count) = observe_contract_group_compilations(|| {
        compile(request_with_targets(
            &semantic,
            vec![success, bounded],
            vec![StrictF32NumericalContract::governed()],
        ))
    });
    let product = result.expect("target-local feasibility cannot erase its companion");
    assert_eq!(group_count, 1);
    assert!(
        outcome_for_key(&product, "test.isolation.success.v1")
            .compiled()
            .is_some()
    );
    assert!(matches!(
        outcome_for_key(&product, "test.isolation.bounded.v1").failure(),
        Some(CompileError::Explained { source, .. })
            if matches!(
                source.as_ref(),
                CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                    PhysicalError::Target { .. }
                ))
            )
    ));
}

#[test]
fn empty_and_duplicate_target_sets_are_outer_request_failures() {
    let semantic = semantic(false);
    let mut empty = CompilationRequest::governed(&semantic);
    empty.target_profiles.clear();
    assert_eq!(
        compile(empty),
        Err(CompileError::InvalidRequest(RequestError::EmptyTargetSet))
    );

    let duplicate = TargetProfile::governed_with_key_for_test("test.duplicate.v1");
    assert_eq!(
        compile(request_with_targets(
            &semantic,
            vec![duplicate.clone(), duplicate],
            vec![StrictF32NumericalContract::governed()],
        )),
        Err(CompileError::InvalidRequest(
            RequestError::DuplicateTargetProfile
        ))
    );
}

#[test]
fn target_group_cardinality_mismatch_is_an_outer_compiler_invariant() {
    let semantic = semantic(false);
    let verified = verify_planned_request(request_with_targets(
        &semantic,
        vec![
            TargetProfile::governed_with_key_for_test("test.group.first.v1"),
            TargetProfile::governed_with_key_for_test("test.group.second.v1"),
        ],
        vec![StrictF32NumericalContract::governed()],
    ))
    .unwrap();
    let group = resolved_target_groups(&verified).remove(0);
    let candidate = verified
        .readmit_candidate(&semantic, &group.target_indexes[..1])
        .unwrap();
    assert_eq!(
        verify_target_group_coordination(&verified, &group, &candidate),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "target-group-cardinality"
            })
        ))
    );
}

#[test]
fn invalid_compiler_output_from_target_compilation_remains_outer() {
    let target = TargetProfile::governed_with_key_for_test("test.outer-invariant.v1");
    let result = target_compilation_outcome(
        &target,
        Err(TargetCompileFailure::Outer(
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "test-target-compiler-invariant",
                },
            )),
        )),
    );
    assert!(matches!(
        result,
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "test-target-compiler-invariant"
            })
        ))
    ));
}

#[test]
fn a_caller_declared_target_profile_reaches_target_feasibility() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.target_profiles[0] = crate::request::TargetProfile::governed_with_grid_axis_limit(1);
    let product = compile(request).expect("the well-formed caller profile is admitted");
    assert!(matches!(
        product.targets[0].failure(),
        Some(CompileError::Explained { source, .. })
            if matches!(
                source.as_ref(),
                CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                    PhysicalError::Target { .. }
                ))
            )
    ));
}

/// An installed authority that lowers nothing is a deferred capability, and
/// it stops the compilation instead of quietly producing a narrower
/// portfolio: an occurrence nobody can lower has no valid plan at all.
#[test]
fn a_registry_without_capabilities_defers_and_fails_closed() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.capabilities = CompilerCapabilitySnapshot::without_capabilities();
    let error = compile(request).unwrap_err();
    let CompileError::Explained { source, explain } = error else {
        panic!("target compilation failures retain their explain trace");
    };
    assert_eq!(
        *source,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: "missing-capability",
        })
    );
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "capability.index-access-resolution.v1"
            && record.event().disposition() == ExplainDisposition::DeferredUnsupported
    }));
    let failure = explain
        .records()
        .iter()
        .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
        .expect("a terminal failure record");
    assert!(matches!(
        failure.event(),
        ExplainEvent::CompilerFailure {
            stage: ExplainStage::CapabilityResolution,
            reason,
        } if reason.as_str() == "lowering-missing-capability"
    ));
}
