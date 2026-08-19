//! Plan-determinism claims (ADR 0013), and every way the join refuses.

use super::super::{ArtifactBuildError, VerifiedArtifactProgram};
use super::super::{ArtifactLimitKind, PayloadPlanDeterminismVerifier, PlanDeterminismScope};
use super::support::claims::{
    TrustingVerifier, claim_descriptor, claim_receipt, two_entry_claim, validated, with_claim_draft,
};
use super::{
    CLAIM_OBJECT, OTHER_SCALE_BITS, claim_declaration, claim_declaration_of, claim_payload_content,
    claimed_artifact, fused_program, semantic_program,
};
use tiler_ir::kernel::verify_plan_determinism;

/// The same fixture with the claim never published.
fn unclaimed_twin() -> VerifiedArtifactProgram {
    with_claim_draft(Some(claim_declaration()), true, |_, _, _| {})
}

/// A published claim flips exactly the claimed cell, and moves identity.
///
/// The identity assertion is the load-bearing one: a `Plan` claim widens what
/// the artifact promises, so a claimed artifact and its unclaimed twin must
/// never share a canonical identity — otherwise a cache could serve the
/// unclaimed build for the claimed subject.
#[test]
fn a_published_plan_claim_marks_its_cell_and_moves_artifact_identity() {
    let claimed = claimed_artifact();
    let unclaimed = unclaimed_twin();
    let cell = claimed.variants().next().unwrap();
    assert_eq!(
        cell.plan_determinism_scope(0),
        Some(PlanDeterminismScope::Plan)
    );
    let twin_cell = unclaimed.variants().next().unwrap();
    assert_eq!(
        twin_cell.plan_determinism_scope(0),
        Some(PlanDeterminismScope::Unclaimed)
    );
    assert_ne!(
        claimed.canonical_identity().as_bytes(),
        unclaimed.canonical_identity().as_bytes(),
        "a claim that does not move identity is invisible to every cache and pin"
    );
}

/// A claim at a delivery position the variant does not carry is refused.
#[test]
fn a_plan_claim_beyond_the_delivery_positions_is_refused() {
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            let witness = verify_plan_determinism(program).unwrap();
            let receipt = claim_receipt(witness);
            assert_eq!(
                draft.publish_plan(variant, 1, &witness, &[receipt]),
                Err(ArtifactBuildError::StructuralLimit {
                    resource: ArtifactLimitKind::DeliveryPositions,
                    actual: 2,
                    limit: 1,
                })
            );
        },
    );
}

/// A witness over some other program cannot claim this variant.
#[test]
fn a_witness_for_another_program_is_refused_as_missing() {
    let semantic = semantic_program();
    let other = fused_program(&semantic, OTHER_SCALE_BITS);
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            assert_ne!(
                program.canonical_identity().as_bytes(),
                other.canonical_identity().as_bytes(),
                "the negative control requires two distinct programs"
            );
            let witness = verify_plan_determinism(&other).unwrap();
            let receipt = claim_receipt(witness);
            assert_eq!(
                draft.publish_plan(variant, 0, &witness, &[receipt]),
                Err(ArtifactBuildError::MissingPlanDeterminismWitness { variant: 0 })
            );
        },
    );
}

/// A payload that declares no environment cannot be claimed.
#[test]
fn a_claim_without_a_target_environment_declaration_is_refused() {
    with_claim_draft(None, true, |draft, variant, program| {
        let witness = verify_plan_determinism(program).unwrap();
        let receipt = claim_receipt(witness);
        assert_eq!(
            draft.publish_plan(variant, 0, &witness, &[receipt]),
            Err(ArtifactBuildError::MissingTargetEnvironmentDeclaration {
                variant: 0,
                delivery: 0,
                entry: 0,
            })
        );
    });
}

/// A claim with no receipt naming the payload's compilation subject is refused.
#[test]
fn a_claim_without_a_payload_receipt_is_refused() {
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            let witness = verify_plan_determinism(program).unwrap();
            assert_eq!(
                draft.publish_plan(variant, 0, &witness, &[]),
                Err(ArtifactBuildError::MissingPayloadPlanDeterminismReceipt {
                    variant: 0,
                    delivery: 0,
                    entry: 0,
                })
            );
        },
    );
}

/// A receipt bound to another program's identity is refused at the join.
///
/// The publishing witness matches the variant, so the disagreement is between
/// the receipt and the witness — the case where a producer reused a receipt
/// minted for a different compilation.
#[test]
fn a_receipt_for_another_program_is_refused_as_a_program_mismatch() {
    let semantic = semantic_program();
    let other = fused_program(&semantic, OTHER_SCALE_BITS);
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            let stale = verify_plan_determinism(&other).unwrap();
            let receipt = claim_receipt(stale);
            let witness = verify_plan_determinism(program).unwrap();
            assert_eq!(
                draft.publish_plan(variant, 0, &witness, &[receipt]),
                Err(ArtifactBuildError::PlanDeterminismProgramMismatch {
                    variant: 0,
                    delivery: 0,
                    entry: 0,
                })
            );
        },
    );
}

/// An uncarried payload cannot be claimed even with a matching receipt.
///
/// The claim fixes exact executable objects through the envelope digest, so a
/// payload whose object is still pending has nothing the claim could bind.
#[test]
fn a_pending_payload_cannot_be_claimed() {
    with_claim_draft(
        Some(claim_declaration()),
        false,
        |draft, variant, program| {
            let witness = verify_plan_determinism(program).unwrap();
            let receipt = claim_receipt(witness);
            assert_eq!(
                draft.publish_plan(variant, 0, &witness, &[receipt]),
                Err(ArtifactBuildError::PlanDeterminismPayloadMismatch {
                    variant: 0,
                    delivery: 0,
                    entry: 0,
                })
            );
        },
    );
}

/// A receipt over different object bytes than the carried ones is refused.
#[test]
fn a_receipt_over_other_object_bytes_is_refused_as_a_payload_mismatch() {
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            let witness = verify_plan_determinism(program).unwrap();
            let declaration = claim_declaration();
            let content = claim_payload_content(b"fused", CLAIM_OBJECT);
            let descriptor = claim_descriptor(&content, Some(declaration.clone()));
            let receipt = TrustingVerifier
                .verify(
                    &witness,
                    &descriptor,
                    b"a relinked object the receipt never saw",
                    &validated(&declaration),
                )
                .unwrap();
            assert_eq!(
                draft.publish_plan(variant, 0, &witness, &[receipt]),
                Err(ArtifactBuildError::PlanDeterminismPayloadMismatch {
                    variant: 0,
                    delivery: 0,
                    entry: 0,
                })
            );
        },
    );
}

/// A receipt bound to a different environment than the payload declares.
///
/// The self-pair (`first_entry == entry`) names the receipt/declaration
/// disagreement: the verifier attested one environment, the payload row
/// declares another, and neither is allowed to win silently.
#[test]
fn a_receipt_for_another_environment_is_refused_as_a_self_paired_mismatch() {
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            let witness = verify_plan_determinism(program).unwrap();
            let attested = claim_declaration_of(b"process-arithmetic-v2");
            let content = claim_payload_content(b"fused", CLAIM_OBJECT);
            let descriptor = claim_descriptor(&content, Some(attested.clone()));
            let receipt = TrustingVerifier
                .verify(&witness, &descriptor, CLAIM_OBJECT, &validated(&attested))
                .unwrap();
            assert_eq!(
                draft.publish_plan(variant, 0, &witness, &[receipt]),
                Err(ArtifactBuildError::PlanDeterminismEnvironmentMismatch {
                    variant: 0,
                    delivery: 0,
                    first_entry: 0,
                    entry: 0,
                })
            );
        },
    );
}

/// Two entries whose payloads declare different environments cannot share one
/// claimed cell.
///
/// Each entry is individually coherent — its receipt matches its own payload's
/// declaration — so what refuses the claim is exactly the cross-entry
/// agreement obligation, reported as the disagreeing pair.
#[test]
fn a_claim_across_entries_with_disagreeing_environments_is_refused() {
    let (outcome, _) = two_entry_claim([
        claim_declaration(),
        claim_declaration_of(b"process-arithmetic-v2"),
    ]);
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::PlanDeterminismEnvironmentMismatch {
            variant: 0,
            delivery: 0,
            first_entry: 0,
            entry: 1,
        })
    );
}
