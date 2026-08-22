//! One negative case per insertion-time builder rule.

use super::super::{
    AbiBinaryOp, AbiRoot, AbiType, AbiUnaryOp, ArtifactBuildError, ArtifactProgramBuilder,
    CompilationEnvironment, DeferredPredicateSpec, PayloadId, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef,
};
use super::{
    Formulas, OTHER_SCALE_BITS, SCALE_BITS, declare_realization, entry, formulas, fused_program,
    lowering_provider, payload, prepared_requirement, profile, selection, semantic_program,
    spare_provider, variant,
};
use tiler_ir::program::VerifiedKernelProgram;
use tiler_ir::program::abi::{ExprNode, TargetPropertyRequirementRelation};

// -------------------------------------------------------------------------
// Negative tests, one per insertion-time rule
// -------------------------------------------------------------------------

/// The membership half of the rule: a provider this environment never offered
/// **at all** cannot be attributed work.
///
/// The refused identity differs from the offered one only in its revision, so a
/// membership predicate that compared anything less than the whole identity
/// would admit it.
#[test]
fn rejects_a_lowering_provider_the_environment_never_offered() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    assert_eq!(
        draft.select_lowering_provider(selection(lowering_provider(9))),
        Err(ArtifactBuildError::LoweringProviderNotOffered {
            provider: Box::new(lowering_provider(9)),
        }),
    );
}

/// The **role** half of the same rule, and the reason the two offered sets are
/// never unioned: `spare_provider(7)` *is* offered here, as a physical
/// implementer, and is still refused lowering authority it was never granted.
///
/// Distinct from the membership case above on purpose. Its identity shares
/// nothing but a namespace with the offered lowering provider, so this stays
/// green under a weakened membership predicate and reddens only when the
/// physical set starts answering a lowering question.
#[test]
fn rejects_a_provider_offered_only_in_the_physical_role() {
    let semantic = semantic_program();
    let environment =
        CompilationEnvironment::new([lowering_provider(1)], [spare_provider(7)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    assert_eq!(
        draft.select_lowering_provider(selection(spare_provider(7))),
        Err(ArtifactBuildError::LoweringProviderNotOffered {
            provider: Box::new(spare_provider(7)),
        }),
    );
}

/// The positive control the two refusals above need: offering one identity in
/// both roles is two grants, and the lowering one is honoured.
///
/// Without this, a `select_lowering_provider` that refused *everything* would
/// satisfy both negative cases.
#[test]
fn a_provider_offered_in_both_roles_keeps_its_lowering_authority() {
    let semantic = semantic_program();
    let environment =
        CompilationEnvironment::new([lowering_provider(1)], [lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    assert_eq!(
        draft.select_lowering_provider(selection(lowering_provider(1))),
        Ok(()),
    );
}

#[test]
fn rejects_a_deferred_requirement_naming_no_entry() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            requirement: prepared_requirement(
                1,
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            ),
            entry: 1,
        }];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::DeferredQueryEntryOutOfRange {
            entry: 1,
            entries: 1,
        }),
    );
}

#[test]
fn a_target_query_provider_is_distinct_from_a_selected_lowering_provider() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            requirement: prepared_requirement(
                1,
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            ),
            entry: 0,
        }];
        draft.push_variant(program, spec)
    });
    assert!(outcome.is_ok());
}

#[test]
fn accepts_a_complete_prepared_entry_requirement() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft
        .select_lowering_provider(selection(provider.clone()))
        .unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.deferred_predicates = vec![DeferredPredicateSpec {
        requirement: prepared_requirement(
            1,
            TargetPropertyRequirementRelation::ObservedAtLeastRequired,
        ),
        entry: 0,
    }];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();
    let deferred = artifact
        .variants()
        .next()
        .expect("one variant")
        .deferred_predicates()
        .next()
        .expect("one deferred predicate");
    assert_eq!(deferred.entry().backend_entry_key().as_bytes(), b"fused");
    assert_eq!(deferred.requirement().required(), 1);
    assert_eq!(
        deferred.requirement().relation(),
        TargetPropertyRequirementRelation::ObservedAtLeastRequired,
    );
    assert_eq!(
        deferred.requirement().query().provider().name(),
        "prepared-entry-properties",
    );
}

#[test]
fn a_reversed_directional_predicate_does_not_match_its_requirement() {
    let requirement = prepared_requirement(
        8,
        TargetPropertyRequirementRelation::ObservedAtLeastRequired,
    );
    let roots = [
        ExprNode::Root(AbiRoot::TargetProperty {
            key: requirement.query().key().clone(),
            phase: requirement.query().available_at(),
        }),
        ExprNode::Root(AbiRoot::UnsignedLiteral(requirement.required())),
    ];
    let correct = [
        roots[0].clone(),
        roots[1].clone(),
        ExprNode::Binary {
            op: AbiBinaryOp::LessOrEqual,
            left: 1,
            right: 0,
        },
    ];
    assert!(
        super::super::model::deferred_predicate_matches_requirement(&correct, 2, &requirement),
        "required <= observed is the admitted direction",
    );

    let reversed = [
        roots[0].clone(),
        roots[1].clone(),
        ExprNode::Binary {
            op: AbiBinaryOp::LessOrEqual,
            left: 0,
            right: 1,
        },
    ];
    assert!(
        !super::super::model::deferred_predicate_matches_requirement(&reversed, 2, &requirement),
        "observed <= required must not masquerade as an at-least requirement",
    );
}

#[test]
fn rejects_a_repeated_deferred_predicate() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let predicate = DeferredPredicateSpec {
            requirement: prepared_requirement(
                1,
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            ),
            entry: 0,
        };
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![predicate.clone(), predicate];
        draft.push_variant(program, spec)
    });
    assert_eq!(outcome, Err(ArtifactBuildError::DuplicateDeferredPredicate));
}

#[test]
fn rejects_a_repeated_launch_precondition() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].launch.preconditions = vec![formulas.always, formulas.always];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::DuplicateLaunchPrecondition { entry: 0 }),
    );
}

#[test]
fn rejects_an_entry_count_that_disagrees_with_the_program() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries.push(entry(formulas, descriptor, b"extra"));
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::EntryCardinality {
            expected: 1,
            actual: 2,
        }),
    );
}

#[test]
fn rejects_a_binding_count_that_disagrees_with_the_kernel_signature() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].bindings.pop();
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::BindingCardinality {
            entry: 0,
            expected: 2,
            actual: 1,
        }),
    );
}

#[test]
fn rejects_a_duplicate_plan_variant() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft.push_variant(&program, variant(&formulas, descriptor, b"other")),
        Err(ArtifactBuildError::DuplicateVariant),
    );
}

/// Every variant of one artifact declares one target profile.
///
/// The refusal that makes "share one compiled object across variants declaring
/// different profiles" a shape no artifact can express — the sentence
/// `docs/artifact-abi.md` withdrew for exactly that reason. It went unpinned
/// through the delivery-position step, so nothing would have caught the check
/// weakening and leaving the contract describing a build that no longer
/// refused.
///
/// The accepting half is asserted first and is load-bearing: the two variants
/// differ only in their declared profile between the halves, so a refusal alone
/// could not distinguish this rule from the duplicate-variant and delivery
/// rules beside it. Both fields of the profile are exercised, because a
/// descriptor that moved under an unchanged key is a different target with the
/// same name.
#[test]
fn refuses_a_second_variant_declaring_a_different_target_profile() {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let provider = lowering_provider(1);

    let assemble = |declared: TargetProfileRef| {
        let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft
            .select_lowering_provider(selection(provider.clone()))
            .unwrap();
        let primary = draft.push_payload(payload(0xa1)).unwrap();
        let spare = draft.push_payload(payload(0xb1)).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&first, variant(&formulas, primary, b"fused"))
            .unwrap();
        let mut spec = variant(&formulas, spare, b"alternate");
        spec.target_profile = declared;
        draft.push_variant(&second, spec).map(|_| ())
    };

    assert_eq!(
        assemble(profile()),
        Ok(()),
        "agreeing siblings are accepted"
    );
    assert_eq!(
        assemble(TargetProfileRef {
            key: TargetProfileKey::new("tiler.test.other").unwrap(),
            descriptor: profile().descriptor,
        }),
        Err(ArtifactBuildError::TargetProfileMismatch),
    );
    assert_eq!(
        assemble(TargetProfileRef {
            key: profile().key,
            descriptor: TargetProfileDescriptorDigest::from_bytes([0x09, 0x09]).unwrap(),
        }),
        Err(ArtifactBuildError::TargetProfileMismatch),
        "the descriptor is half the profile, not decoration",
    );
}

#[test]
fn rejects_a_repeated_payload_descriptor() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.push_payload(payload(0xa1)).unwrap();
    assert_eq!(
        draft.push_payload(payload(0xa1)),
        Err(ArtifactBuildError::DuplicatePayload),
    );
}

#[test]
fn rejects_a_mistyped_expression_operand() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let number = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    assert_eq!(
        draft.push_unary(AbiUnaryOp::Not, number),
        Err(ArtifactBuildError::OperandType {
            expected: AbiType::Boolean,
            actual: AbiType::Unsigned,
        }),
    );
    let predicate = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    assert_eq!(
        draft.push_select(predicate, number, predicate),
        Err(ArtifactBuildError::SelectBranchType {
            if_true: AbiType::Unsigned,
            if_false: AbiType::Boolean,
        }),
    );
}

// -------------------------------------------------------------------------
// Test-local helpers
// -------------------------------------------------------------------------

/// Runs one rejection case against the canonical draft state.
fn with_default_draft<T>(
    case: impl FnOnce(
        &mut ArtifactProgramBuilder,
        &Formulas,
        PayloadId,
        &VerifiedKernelProgram,
    ) -> Result<T, ArtifactBuildError>,
) -> Result<T, ArtifactBuildError> {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    case(&mut draft, &formulas, descriptor, &program)
}
