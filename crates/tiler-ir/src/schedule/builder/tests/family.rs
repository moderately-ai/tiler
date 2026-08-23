use super::super::{
    AccessOrdinal, FamilyTopology, ParallelFamily, ReductionTopology, STRICT_AFFINE_CODES_ROLE,
    STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, ScalarProgram, ScheduledRegionBuilder,
    TensorRole, split_family,
};
use super::support::{
    SPLIT, bare_sum, cooperative_builder, cooperative_tile_fixture, maximum_scalar,
    partial_pass_builder, read_from, scale_bias_expression, scale_epilogue,
    serial_reduction_builder, set_numerical, set_scalar, squared_sum_with_epilogue,
    strict_numerical,
};
use crate::schedule::PointwiseBf16ExpressionBuilder;
use crate::schedule::model::ContributorOrder;
use crate::shape::{Axis, Shape};
use std::mem::variant_count;

/// One inhabitant of every [`ScalarProgram`] variant and its expected
/// reduction-family classification.
///
/// The array is sized from the type rather than from a hand-written count:
/// widening the scalar vocabulary without classifying its new inhabitant is
/// therefore a compile error here instead of a smaller census that stays
/// green.
struct ScalarProgramFamilyCase {
    name: &'static str,
    program: ScalarProgram,
    parallel: Option<ParallelFamily>,
}

fn scalar_program_family_population() -> [ScalarProgramFamilyCase; variant_count::<ScalarProgram>()]
{
    let mut bf16 = PointwiseBf16ExpressionBuilder::new();
    let bf16_input = bf16.input(AccessOrdinal::FIRST).unwrap();
    let bf16 = bf16.build(bf16_input).unwrap();
    [
        ScalarProgramFamilyCase {
            name: "pointwise f32",
            program: ScalarProgram::PointwiseF32(scale_bias_expression(
                1.0_f32.to_bits(),
                0.0_f32.to_bits(),
            )),
            parallel: None,
        },
        ScalarProgramFamilyCase {
            name: "pointwise bf16",
            program: ScalarProgram::PointwiseBf16(bf16),
            parallel: None,
        },
        ScalarProgramFamilyCase {
            name: "strict affine u4 decode",
            program: ScalarProgram::StrictAffineU4Dequantize {
                codes_role: STRICT_AFFINE_CODES_ROLE,
                scale_role: STRICT_AFFINE_SCALE_ROLE,
                zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            },
            parallel: None,
        },
        ScalarProgramFamilyCase {
            name: "strict serial sum",
            program: bare_sum(vec![Axis::new(1)]),
            parallel: Some(ParallelFamily::Split { final_pass: true }),
        },
        ScalarProgramFamilyCase {
            name: "scale-bias prologue",
            program: ScalarProgram::FusedMultiplyAddSerialSum {
                scale_bits: 1.0_f32.to_bits(),
                bias_bits: 0.0_f32.to_bits(),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
                contraction: false,
            },
            parallel: Some(ParallelFamily::Split { final_pass: false }),
        },
        ScalarProgramFamilyCase {
            name: "squaring prologue",
            program: ScalarProgram::SquaredSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            parallel: Some(ParallelFamily::Split { final_pass: false }),
        },
        ScalarProgramFamilyCase {
            name: "squaring prologue with epilogue",
            program: squared_sum_with_epilogue(scale_epilogue()),
            parallel: Some(ParallelFamily::SerialOnly),
        },
        ScalarProgramFamilyCase {
            name: "strict tensor contraction",
            program: ScalarProgram::StrictTensorContraction {
                contracted_shape: Shape::from_dims([6]),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            },
            parallel: None,
        },
        ScalarProgramFamilyCase {
            name: "extrema fold",
            program: maximum_scalar(),
            parallel: Some(ParallelFamily::Split { final_pass: true }),
        },
    ]
}

/// Makes a parallel fixture's numerical declaration agree with one family.
///
/// Every sum family consumes reassociation; maximum is order-insensitive and
/// consumes none. The edit moves the topology declaration and realization
/// together so the contributor-tensor comparison below is the only varying
/// admission fact.
fn declare_family_reassociation(builder: &mut ScheduledRegionBuilder, program: &ScalarProgram) {
    if !matches!(program, ScalarProgram::StrictSerialMaximum { .. }) {
        return;
    }
    set_numerical(builder, strict_numerical());
    match &mut builder
        .schedule
        .as_mut()
        .expect("the fixture has a schedule")
        .reduction
    {
        ReductionTopology::MultiPass {
            permits_reassociation,
            ..
        }
        | ReductionTopology::CooperativeWorkgroup {
            permits_reassociation,
            ..
        } => *permits_reassociation = false,
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::CooperativeContraction { .. }
        | ReductionTopology::LiveContraction { .. } => {
            panic!("the fixture has a parallel reduction")
        }
    }
}

fn partial_family_builder(
    program: ScalarProgram,
    read_tensor: TensorRole,
) -> ScheduledRegionBuilder {
    let mut builder = partial_pass_builder(SPLIT);
    read_from(&mut builder, read_tensor);
    declare_family_reassociation(&mut builder, &program);
    set_scalar(&mut builder, program);
    builder
}

fn cooperative_family_builder(
    program: ScalarProgram,
    read_tensor: TensorRole,
) -> ScheduledRegionBuilder {
    let mut builder = cooperative_builder(cooperative_tile_fixture());
    read_from(&mut builder, read_tensor);
    declare_family_reassociation(&mut builder, &program);
    set_scalar(&mut builder, program);
    builder
}

/// The scalar-program population derives exactly five serial fold families,
/// four of which also state a parallel split.
#[test]
fn the_scalar_program_population_derives_five_serial_and_four_parallel_families() {
    let population = scalar_program_family_population();
    assert_eq!(
        population
            .iter()
            .filter(|case| case.parallel.is_some())
            .count(),
        5,
        "five ScalarProgram variants are serial fold families",
    );
    assert_eq!(
        population
            .iter()
            .filter(|case| matches!(case.parallel, Some(ParallelFamily::Split { .. })))
            .count(),
        4,
        "four serial families also state a parallel split",
    );
    for case in population {
        let derived = split_family(&case.program).map(|family| family.parallel);
        assert_eq!(derived, case.parallel, "{} classification", case.name);
    }
}

/// Every family shared by the three topologies admits the same boundary
/// contributor tensors.
///
/// Three roles cover the complete fieldless predicate vocabulary: an input,
/// the materialized intermediate, and the refused output. The expected answer comes from the family derivation and
/// is checked independently through each production admission, so changing
/// only the serial gate's read predicate makes this test fail even though the
/// family table and both parallel gates still agree.
#[test]
fn shared_families_admit_the_same_contributor_tensors_in_every_topology() {
    let tensors = [
        TensorRole::Input,
        TensorRole::Intermediate,
        TensorRole::Output,
    ];
    for case in scalar_program_family_population()
        .into_iter()
        .filter(|case| matches!(case.parallel, Some(ParallelFamily::Split { .. })))
    {
        let family = split_family(&case.program).expect("the case is a fold family");
        for tensor in tensors {
            let expected = family
                .read_tensor(FamilyTopology::Serial)
                .expect("every fold has a serial contributor tensor")
                .admits(tensor);

            let mut serial = serial_reduction_builder(case.program.clone());
            read_from(&mut serial, tensor);
            let serial_admitted = serial.build().is_ok();
            let partial_admitted = partial_family_builder(case.program.clone(), tensor)
                .build()
                .is_ok();
            let cooperative_admitted = cooperative_family_builder(case.program.clone(), tensor)
                .build()
                .is_ok();

            assert_eq!(
                serial_admitted, expected,
                "serial {} reading {tensor:?}",
                case.name,
            );
            assert_eq!(
                partial_admitted, expected,
                "partial {} reading {tensor:?}",
                case.name,
            );
            assert_eq!(
                cooperative_admitted, expected,
                "cooperative {} reading {tensor:?}",
                case.name,
            );
        }
    }
}

/// A fused family that requests contraction remains outside every fold
/// topology; shared derivation must not erase this per-family residual.
#[test]
fn a_contracted_fused_program_is_not_a_reduction_family() {
    let mut contracted = scalar_program_family_population()
        .into_iter()
        .find(|case| case.name == "scale-bias prologue")
        .expect("the population contains the fused family")
        .program;
    let ScalarProgram::FusedMultiplyAddSerialSum { contraction, .. } = &mut contracted else {
        panic!("the named population member is the fused family")
    };
    *contraction = true;
    assert!(split_family(&contracted).is_none());
    assert!(
        serial_reduction_builder(contracted.clone())
            .build()
            .is_err()
    );
    assert!(
        partial_family_builder(contracted.clone(), TensorRole::Input)
            .build()
            .is_err()
    );
    assert!(
        cooperative_family_builder(contracted, TensorRole::Input)
            .build()
            .is_err()
    );
}
