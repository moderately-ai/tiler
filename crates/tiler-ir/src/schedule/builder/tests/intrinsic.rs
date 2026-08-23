use super::super::{
    BlockedWorkgroupRule, ExecutionBinding, OwnershipProofKind, RegionId, ScalarProgram,
    ScheduledRegionDiagnostic, TailPolicy, VectorLaneRule,
};
use super::support::{
    contraction_builder, cooperative_rejection, float_rows, into_fixed_vector_map,
    pointwise_builder, serial_reduction_builder,
};
use super::support_contraction::{
    admitted_operand_tile, operand_contraction_builder, operand_tile_fixture,
};
use crate::schedule::model::ContributorOrder;
use crate::schedule::numerics::NumericalPermission;
use crate::shape::{Axis, Shape};

#[test]
fn launch_that_undercounts_the_domain_is_rejected() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    builder
        .schedule
        .as_mut()
        .expect("schedule was set")
        .work_items = 5;
    let error = builder.build().unwrap_err();
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::LaunchCoverage]
    );
}

/// The accepted exact fixed-vector map admits pointwise work under a fully
/// strict contract: `work_items = N`, `grid_threads = N / W`, and no
/// numerical permission is consumed or required by grouping independent
/// outputs into packets.
#[test]
fn the_fixed_vector_map_admits_exact_pointwise_work_under_a_strict_contract() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    into_fixed_vector_map(&mut builder, 2, 3);
    let verified = builder.build().unwrap();
    assert_eq!(verified.region().schedule.work_items, 6);
    assert_eq!(verified.region().schedule.launch.grid_threads, 3);
    // The ownership population stays the scalar-output population: packet
    // `p`, lane `l` is the one owning invocation of output `2p + l`.
    assert_eq!(
        verified.region().index.ownership_proof.kind,
        OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 }
    );
    // The strict contract passes through untouched — the admission read
    // none of it, so nothing was consumed, relaxed, or newly required.
    let requirements = verified.requirements();
    assert_eq!(
        float_rows(&requirements).contraction,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).reassociation,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).permutation,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).signed_zero,
        NumericalPermission::Forbidden
    );
}

/// The strict serial fold across independent outputs is the second and
/// last admitted pairing: the fold inside each output stays serial and
/// order-preserving, and the lanes group only the independent outputs.
#[test]
fn the_fixed_vector_map_admits_the_strict_serial_fold_across_independent_outputs() {
    let mut builder = serial_reduction_builder(ScalarProgram::StrictSerialSum {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0,
    });
    into_fixed_vector_map(&mut builder, 2, 1);
    let verified = builder.build().unwrap();
    assert_eq!(verified.region().schedule.work_items, 2);
    assert_eq!(verified.region().schedule.launch.grid_threads, 1);
    assert_eq!(
        float_rows(&verified.requirements()).reassociation,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&verified.requirements()).permutation,
        NumericalPermission::Forbidden
    );
}

/// Lane counts zero and one are refused at construction, each under its
/// own name: invalidity and the duplicate scalar spelling are different
/// refusals a producer must be able to tell apart.
#[test]
fn lane_counts_zero_and_one_are_refused_at_construction_by_name() {
    use crate::schedule::error::VectorLaneCountError;

    let zero = super::super::super::model::VectorLaneCount::new(0).unwrap_err();
    assert_eq!(zero, VectorLaneCountError::Zero);
    assert_eq!(zero.rule(), "vector-lane-count-zero");
    let one = super::super::super::model::VectorLaneCount::new(1).unwrap_err();
    assert_eq!(one, VectorLaneCountError::ScalarSpelling);
    assert_eq!(one.rule(), "vector-lane-count-scalar-spelling");
    assert_eq!(
        super::super::super::model::VectorLaneCount::new(2)
            .unwrap()
            .get(),
        2
    );
}

/// `N mod W != 0` is refused by its own rule: the verifier never rounds
/// the iteration count, masks implicitly, or peels a scalar tail.
#[test]
fn a_nondivisible_fixed_vector_domain_is_refused_by_name() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    into_fixed_vector_map(&mut builder, 4, 1);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::VectorLaneBinding {
            rule: VectorLaneRule::NondivisibleCoverage
        }]
    );
}

/// `grid_threads = N` with a reinterpreted builtin is exactly the launch
/// identity the acceptance forbids, and it is refused as a wrong packet
/// population rather than admitted for an emitter to reinterpret.
#[test]
fn a_fixed_vector_launch_keeping_the_scalar_grid_is_refused() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    into_fixed_vector_map(&mut builder, 2, 6);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::VectorLaneBinding {
            rule: VectorLaneRule::PacketPopulation
        }]
    );
}

/// An overflowing packet product is a product that does not exist, named
/// apart from a wrong packet count.
#[test]
fn overflowing_fixed_vector_packet_arithmetic_is_refused_by_name() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    into_fixed_vector_map(&mut builder, 2, u64::MAX);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::VectorLaneBinding {
            rule: VectorLaneRule::PacketArithmeticOverflow
        }]
    );
}

/// Every unadmitted reduction/binding pairing is one refusal, reached
/// independently of coverage and launch arithmetic.
#[test]
fn an_unsupported_fixed_vector_reduction_pairing_is_refused_by_name() {
    let mut builder = contraction_builder();
    into_fixed_vector_map(&mut builder, 2, 2);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::VectorLaneBinding {
            rule: VectorLaneRule::UnsupportedReduction
        }]
    );
}

/// The fixed-vector map admits `Exact` alone; a predicated tail is refused
/// under the binding's own rule rather than as a launch-coverage failure.
#[test]
fn a_non_exact_fixed_vector_tail_is_refused_by_name() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    into_fixed_vector_map(&mut builder, 2, 3);
    builder.schedule.as_mut().expect("schedule was set").tail = TailPolicy::Predicated;
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::VectorLaneBinding {
            rule: VectorLaneRule::ExactTailRequired
        }]
    );
}

/// Perturbing the binding tag and the lane count each separates canonical
/// identity, isolated on a zero domain where every other schedule byte —
/// including the packet population — is identical.
#[test]
fn the_fixed_vector_binding_tag_and_lane_count_separate_identity() {
    let scalar = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0)
        .build()
        .unwrap();
    let mut two = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0);
    into_fixed_vector_map(&mut two, 2, 0);
    let two = two.build().unwrap();
    let mut three = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0);
    into_fixed_vector_map(&mut three, 3, 0);
    let three = three.build().unwrap();

    // Binding tag: the vector region differs from the scalar one although
    // work items, launch, accesses, program, and contract are all equal.
    assert_ne!(
        scalar.canonical_identity().as_bytes(),
        two.canonical_identity().as_bytes()
    );
    // Lane count: two widths are two programs and never share bytes.
    assert_ne!(
        two.canonical_identity().as_bytes(),
        three.canonical_identity().as_bytes()
    );
    // The appended arm is a tag plus a fixed-width count: the vector
    // encoding is exactly eight bytes (the lane count) longer than the
    // scalar one, so the widening moved no earlier field.
    assert_eq!(
        two.canonical_identity().as_bytes().len(),
        scalar.canonical_identity().as_bytes().len() + 8
    );
}

/// Two invocations claiming one output is an overlap, not a gap.
#[test]
fn a_blocked_map_with_an_overlapping_axis_is_refused() {
    let mut builder = operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
    let ExecutionBinding::BlockedWorkgroup { workgroups, .. } =
        &mut builder.schedule.as_mut().unwrap().binding
    else {
        panic!("the fixture carries the blocked binding")
    };
    *workgroups = Shape::from_dims([3, 2]);
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::BlockedWorkgroup {
            rule: BlockedWorkgroupRule::MappingOverlap,
        }
    );
}

/// An output coordinate with no preimage is a gap, not an overlap.
#[test]
fn a_blocked_map_with_a_gapped_axis_is_refused() {
    let mut builder = operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
    let ExecutionBinding::BlockedWorkgroup { workgroups, .. } =
        &mut builder.schedule.as_mut().unwrap().binding
    else {
        panic!("the fixture carries the blocked binding")
    };
    *workgroups = Shape::from_dims([1, 2]);
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::BlockedWorkgroup {
            rule: BlockedWorkgroupRule::MappingGap,
        }
    );
}

/// The blocked binding is required; `GlobalLinearInvocation` is not a default.
#[test]
fn a_cooperative_contraction_without_the_blocked_binding_is_refused() {
    let mut builder = operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
    builder.schedule.as_mut().unwrap().binding = ExecutionBinding::GlobalLinearInvocation;
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::BlockedWorkgroup {
            rule: BlockedWorkgroupRule::BindingRequired,
        }
    );
}
