//! The ADR 0013 plan-determinism witness
//!
//! The witness covers the whole canonical program and projects its identities.

use super::support::{
    CANONICAL_NAN, SCALE_BITS, TwoStageShape, canonical_program, complete_two_stage,
    pointwise_region, reduction_kernel, serial_sum_program, wire_two_stage,
};
use crate::kernel::{VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, NumericalPermission, NumericalRealization,
    RegionProgram, ScheduledRegionBuilder, SubnormalMode,
};

/// The witness covers the whole canonical program and projects its identities.
///
/// The positive control every refusal case below leans on, with its population
/// counted from the program: two stages, two scheduled-region identities, one
/// kernel-program identity — each read back through the witness's own
/// accessors and compared against the program's, so the witness cannot claim a
/// program other than the one it borrowed.
#[test]
fn the_plan_determinism_witness_covers_the_canonical_program() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    let witness = crate::kernel::verify_plan_determinism(&program)
        .expect("the strict canonical program is plan deterministic");
    assert_eq!(
        witness.kernel_program_identity().as_bytes(),
        program.canonical_identity().as_bytes(),
        "the witness projects exactly the program it proves",
    );
    let regions: Vec<_> = witness
        .scheduled_region_identities()
        .map(|identity| identity.as_bytes().to_vec())
        .collect();
    assert_eq!(
        regions.len(),
        program.stages().len(),
        "one scheduled-region identity per stage",
    );
    assert_eq!(regions.len(), 2, "the canonical program has two stages");
    for (stage, identity) in program.stages().zip(&regions) {
        assert_eq!(
            stage.kernel().scheduled_region_identity().as_bytes(),
            identity.as_slice(),
            "the topology binding is each stage's own scheduled-region identity",
        );
    }
}

/// Builds the pointwise kernel under a permutation-permitted realization.
///
/// One numerical field moves — the contributor-permutation permission — and
/// nothing else: the region's accesses, proofs, expression, and schedule are
/// the canonical pointwise fixture's own bytes.
fn permutation_permitted_pointwise_kernel(region: u32) -> VerifiedKernel {
    let mut raw = pointwise_region(region, SCALE_BITS).region().clone();
    match &mut raw.index.program {
        RegionProgram::Numerical { numerical, .. } => {
            *numerical = NumericalRealization::new(
                "tiler.test.permutation-permitted-f32",
                CANONICAL_NAN,
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Permitted,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            );
        }
        RegionProgram::PartitionedCopy { .. } => {
            panic!("the pointwise fixture declares a numerical program")
        }
    }
    let region = ScheduledRegionBuilder::from_region(raw)
        .build()
        .expect("the permutation-permitted region verifies");
    lower_scheduled_region(&region).expect("the permissive kernel lowers")
}

/// Granting permutation must not yield a plan-deterministic witness.
///
/// The accepted arrival perturbation, on its reachable spelling: the current
/// builders refuse `NondeterministicArrival`, `AtomicAccumulation`, and
/// `SynchronizationKind::Atomic` by name before a verified schedule exists, so
/// the freedom those spellings consume — the contributor-permutation
/// permission — is the arrival subject a verified program can still carry.
/// The witness refuses it by name, at the exact stage, because nothing in the
/// program proves the granted freedom went unused; accepting it would let a
/// later admitted unfixed-arrival construct arrive already holding a witness.
#[test]
fn a_permutation_permitted_stage_is_refused_as_unfixed_arrival_by_name() {
    let semantic = serial_sum_program(SCALE_BITS);
    // The same program shape as the positive control above; only the pointwise
    // stage's permutation permission moves.
    let program = complete_two_stage(wire_two_stage(
        &semantic,
        &permutation_permitted_pointwise_kernel(0),
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("the permissive program verifies");
    let refusal = crate::kernel::verify_plan_determinism(&program)
        .expect_err("a granted arrival freedom must not inherit plan determinism");
    assert_eq!(
        refusal,
        crate::kernel::PlanDeterminismRefusal::UnfixedContributorArrival { stage: 0 },
        "the refusal names the exact stage and class",
    );
    assert_eq!(
        refusal.to_string(),
        "plan-determinism.unfixed-contributor-arrival: stage 0's declared realization permits \
         contributor permutation, so its arrival order is not fixed by canonical program bytes",
    );
}
