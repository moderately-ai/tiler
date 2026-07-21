#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]
//! Runtime and identity contract for checked shape evidence.

use std::mem::{size_of, size_of_val};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
    ShapeRefineError, ShapeWitnessError, ShapedValue, StrictSerialF32Sum, Value,
};
use tiler_ir::shape::{Axis, Rank, Shape, ShapeExpectation, StaticShape};

type Matrix = StaticShape<2, { [2, 3] }>;
type Transposed = StaticShape<2, { [3, 2] }>;

fn input(
    builder: &mut SemanticProgramBuilder,
    name: &str,
    dims: impl IntoIterator<Item = u64>,
) -> Value<F32> {
    builder
        .input(InputKey::new(name).unwrap(), Shape::from_dims(dims))
        .unwrap()
}

#[test]
fn refinement_is_checked_graph_owned_and_non_mutating() {
    let mut first = SemanticProgramBuilder::try_standard().unwrap();
    let matrix = input(&mut first, "matrix", [2, 3]);
    let shaped = first.refine::<_, Matrix>(matrix).unwrap();
    assert_eq!(shaped.weaken(), matrix);
    assert_eq!(size_of_val(&shaped), size_of::<Value<F32>>());

    assert_eq!(
        first.refine::<_, Transposed>(matrix),
        Err(ShapeRefineError::EvidenceMismatch {
            expected: ShapeExpectation::Exact(Shape::from_dims([3, 2])),
            actual: Shape::from_dims([2, 3]),
        })
    );
    assert_eq!(
        first
            .refine::<_, Transposed>(matrix)
            .unwrap_err()
            .to_string(),
        "requested exact shape [3, 2] does not match authoritative shape [2, 3]"
    );

    let mut second = SemanticProgramBuilder::try_standard().unwrap();
    let foreign = input(&mut second, "foreign", [2, 3]);
    assert!(matches!(
        first.refine::<_, Matrix>(foreign),
        Err(ShapeRefineError::Handle(_))
    ));
}

#[test]
fn shaped_and_plain_facades_produce_identical_semantics() {
    fn plain() -> tiler_ir::semantic::SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let left = input(&mut builder, "left", [2, 3]);
        let right = input(&mut builder, "right", [2, 3]);
        let sum = F32Add::apply(&mut builder, left, right).unwrap();
        builder.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        builder.build().unwrap()
    }

    fn shaped() -> tiler_ir::semantic::SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let left = input(&mut builder, "left", [2, 3]);
        let right = input(&mut builder, "right", [2, 3]);
        let left = builder.refine::<_, Matrix>(left).unwrap();
        let right = builder.refine::<_, Matrix>(right).unwrap();
        let sum = F32Add::apply_shaped(&mut builder, left, right).unwrap();
        builder
            .output(OutputKey::new("sum").unwrap(), sum.weaken())
            .unwrap();
        builder.build().unwrap()
    }

    assert_eq!(
        plain().semantic_graph_identity(),
        shaped().semantic_graph_identity()
    );
}

#[test]
fn evidence_is_preserved_only_for_revalidated_exact_relationships() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let scalar = F32Constant::apply_shaped(&mut builder, 1.0_f32.to_bits()).unwrap();
    let matrix = input(&mut builder, "matrix", [2, 3]);
    let matrix = builder.refine::<_, Matrix>(matrix).unwrap();
    let sum = F32Add::apply_shaped(&mut builder, matrix, matrix).unwrap();
    let sum = F32Add::apply_scalar_right(&mut builder, sum, scalar).unwrap();
    let sum = F32Multiply::apply_scalar_left(&mut builder, scalar, sum).unwrap();
    let ranked: ShapedValue<F32, Rank<2>> = sum.forget_extents();
    let reduced: Value<F32> =
        StrictSerialF32Sum::apply(&mut builder, ranked.weaken(), [Axis::new(1)]).unwrap();

    let _: Value<F32> = scalar.weaken();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder.build().unwrap();
}

#[test]
fn same_shape_witnesses_bind_graph_and_ordered_subjects() {
    let mut first = SemanticProgramBuilder::try_standard().unwrap();
    let left = input(&mut first, "left", [2, 3]);
    let right = input(&mut first, "right", [2, 3]);
    let unequal = input(&mut first, "unequal", [3, 2]);
    let witness = first.prove_same_shape(left, right).unwrap();
    first
        .validate_same_shape_witness(&witness, left, right)
        .unwrap();
    assert_eq!(
        first.validate_same_shape_witness(&witness, right, left),
        Err(ShapeWitnessError::SubjectMismatch)
    );
    assert!(matches!(
        first.prove_same_shape(left, unequal),
        Err(ShapeWitnessError::NotSameShape { .. })
    ));

    let mut second = SemanticProgramBuilder::try_standard().unwrap();
    let second_left = input(&mut second, "left", [2, 3]);
    let second_right = input(&mut second, "right", [2, 3]);
    assert_eq!(
        second.validate_same_shape_witness(&witness, second_left, second_right),
        Err(ShapeWitnessError::ForeignWitness)
    );
}

#[test]
fn completed_program_rechecks_evidence_and_does_not_accept_draft_witnesses() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let left = input(&mut builder, "left", [2, 3]);
    let right = input(&mut builder, "right", [2, 3]);
    let draft_witness = builder.prove_same_shape(left, right).unwrap();
    let left_output = builder
        .output(OutputKey::new("left").unwrap(), left)
        .unwrap();
    let right_output = builder
        .output(OutputKey::new("right").unwrap(), right)
        .unwrap();
    let program = builder.build().unwrap();
    let left = program.resolve_typed_output(&left_output).unwrap().value();
    let right = program.resolve_typed_output(&right_output).unwrap().value();

    let refined = program.refine::<_, Matrix>(left).unwrap();
    assert_eq!(refined.weaken(), left);
    assert_eq!(
        program.validate_same_shape_witness(&draft_witness, left, right),
        Err(ShapeWitnessError::ForeignWitness)
    );
    let witness = program.prove_same_shape(left, right).unwrap();
    program
        .validate_same_shape_witness(&witness, left, right)
        .unwrap();
}
