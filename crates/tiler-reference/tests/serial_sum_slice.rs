//! Downstream-style proof of the public semantic and reference-evaluation path.

use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape, StaticShape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

fn f32_tensor(shape: Shape, values: impl IntoIterator<Item = f32>) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        shape,
        values
            .into_iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_bits().to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap()
}

fn f32_values(tensor: &Tensor) -> Vec<f32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("expected dense f32 tensor")
    };
    elements
        .iter()
        .map(|element| {
            let bits = <[u8; 4]>::try_from(element.as_bytes()).unwrap();
            f32::from_bits(u32::from_be_bytes(bits))
        })
        .collect()
}

fn build_program(insert_dead_value_first: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    if insert_dead_value_first {
        F32Constant::apply(&mut builder, f32::NAN.to_bits()).unwrap();
    }
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let input = builder
        .refine::<_, StaticShape<2, { [2, 3] }>>(input)
        .unwrap();
    let scale = F32Constant::apply_shaped(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply_shaped(&mut builder, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply_scalar_right(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply_scalar_right(&mut builder, product, bias).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, mapped.weaken(), [Axis::new(1)]).unwrap();

    builder
        .output(OutputKey::new("mapped").unwrap(), mapped.weaken())
        .unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn public_semantic_program_evaluates_independently_of_construction_history() {
    let first = build_program(false);
    let second = build_program(true);
    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );

    let key = InputKey::new("input").unwrap();
    let input = f32_tensor(Shape::from_dims([2, 3]), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let binding = [InputBinding::new(&key, &input)];
    let evaluator = ReferenceEvaluator::standard().unwrap();
    let first_outputs = evaluator.evaluate(&first, &binding).unwrap();
    let second_outputs = evaluator.evaluate(&second, &binding).unwrap();

    assert_eq!(first_outputs, second_outputs);
    assert_eq!(first_outputs[0].shape(), &Shape::from_dims([2, 3]));
    assert_eq!(
        f32_values(&first_outputs[0]),
        [3.0, 5.0, 7.0, 9.0, 11.0, 13.0]
    );
    assert_eq!(first_outputs[1].shape(), &Shape::from_dims([2]));
    assert_eq!(f32_values(&first_outputs[1]), [15.0, 33.0]);
}
