use super::super::{
    BoundsProofKind, OwnershipProof, OwnershipProofKind, RegionId, ScheduledRegionDiagnostic,
    TensorRole,
};
use super::support::{
    cooperative_builder, cooperative_tile_fixture, into_fixed_vector_map, pointwise_builder,
    three_input_builder,
};
use super::support_contraction::{
    admitted_operand_tile, operand_contraction_builder, operand_tile_fixture,
};
use crate::schedule::handles::{BoundsWitnessId, OwnershipWitnessId};
use crate::shape::Shape;

/// A wrong output-owner population keeps its existing independent refusal:
/// the ownership proof must still cover exactly the `N` scalar outputs.
#[test]
fn a_fixed_vector_region_with_a_wrong_owner_population_is_refused() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    into_fixed_vector_map(&mut builder, 2, 3);
    builder
        .ownership_proof
        .as_mut()
        .expect("proof was set")
        .kind = OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 3 };
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ProofReference]
    );
}

#[test]
fn dangling_bounds_witness_is_rejected_by_proof_reference() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    builder.accesses[0].bounds = BoundsWitnessId::new(9);
    let error = builder.build().unwrap_err();
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::ProofReference]
    );
    // The builder is recovered intact for amend-and-retry.
    let (recovered, _) = error.into_parts();
    assert_eq!(recovered.accesses.len(), 2);
}

/// Two read proofs may not claim one witness identity.
///
/// The positional zip above proves each record against the access at its own
/// ordinal, so a duplicated id survives it: both records are well formed where
/// they sit. The defect is that nothing can then *address* the second one —
/// every resolver in the tree takes the first record bearing an id — while it
/// is still folded into canonical scheduled-region identity.
#[test]
fn two_read_proofs_may_not_claim_one_bounds_witness() {
    let mut builder = three_input_builder(4);
    builder.accesses[1].bounds = BoundsWitnessId::new(0);
    builder.bounds_proofs[1].id = BoundsWitnessId::new(0);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ProofReference]
    );
}

/// A read proof and the write proof may not claim one witness identity either.
///
/// The same rule, driven from the other side. Stated as its own subject because
/// this pairing was refused by a narrower clause before the distinctness rule
/// replaced it, and a reader needs to see that the replacement did not trade
/// one half of the invariant for the other.
#[test]
fn a_read_proof_and_the_write_proof_may_not_claim_one_bounds_witness() {
    let mut builder = three_input_builder(4);
    builder.accesses[0].bounds = BoundsWitnessId::new(3);
    builder.bounds_proofs[0].id = BoundsWitnessId::new(3);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ProofReference]
    );
}

/// The ownership proof counts output positions, never invocations.
///
/// Without this the cooperative region would have claimed one owned position
/// per invocation — six for two outputs — and every consumer reading the
/// proof would have sized the output tensor three times too large.
#[test]
fn a_cooperative_region_owns_one_position_per_workgroup() {
    let mut builder = cooperative_builder(cooperative_tile_fixture());
    builder.ownership_proof = Some(OwnershipProof {
        id: OwnershipWitnessId::new(0),
        tensor: TensorRole::Output,
        kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 },
    });
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ProofReference]
    );
}

/// Ownership is not `work_items / participants` merely because a tile is present.
#[test]
fn a_helper_that_infers_reduction_ownership_from_a_tile_is_refused() {
    let mut builder = operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
    // The false helper would report 1024 / 256 = 4 owned positions.
    builder.ownership_proof = Some(OwnershipProof {
        id: OwnershipWitnessId::new(0),
        tensor: TensorRole::Output,
        kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
    });
    builder.bounds_proofs[2].kind = BoundsProofKind::LinearRange { element_count: 4 };
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ProofReference]
    );
}
