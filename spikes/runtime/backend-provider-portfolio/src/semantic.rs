//! The one semantic program every family in this portfolio packages.
//!
//! The compiler's pointwise normalization admits exactly two program shapes.
//! The smaller-looking `(input * 2.0) + 1.0` is refused as
//! `pointwise-association`. `(input * 2.0) * 1.0` is the minimum admitted
//! pointwise shape, which is why this spike uses it rather than a more
//! convenient arithmetic.

use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// Rows of the workload.
pub const ROWS: u64 = 4;
/// Columns of the workload.
pub const COLUMNS: u64 = 3;
/// Interface key of the program's one input.
pub const INPUT_KEY: &str = "input";
/// Interface key of the program's one output.
pub const OUTPUT_KEY: &str = "result";

/// The operand pattern the workload is filled from.
///
/// The Apple Metal declaration this portfolio shares assesses
/// `FLUSH_SUBNORMALS_TO_ZERO_F32`, so this vector omits subnormals: a
/// subnormal operand would make a host-preserving CPU interpreter disagree
/// with Metal and with a strict `tiler-reference` evaluation of the same
/// graph. Signed zero, a non-canonical NaN, both infinities, and ordinary
/// finites remain.
pub const OPERANDS: [u32; 12] = [
    0x3f80_0000, // 1.0
    0x8000_0000, // -0.0
    0x3f00_0000, // 0.5
    0x7fc0_1234, // a non-canonical NaN payload
    0x7f80_0000, // +inf
    0xff80_0000, // -inf
    0xbf80_0000, // -1.0
    0x4040_0000, // 3.0
    0x4000_0000, // 2.0
    0x0080_0000, // least positive normal
    0x477f_e000, // 65504.0
    0xc0c0_0000, // -6.0
];

/// Builds the smallest pointwise program this compiler's normalization admits.
#[must_use]
pub fn program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new(INPUT_KEY).expect("the input key is valid"),
            Shape::from_dims([ROWS, COLUMNS]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).expect("the scale applies");
    let unit = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the unit applies");
    let scaled = F32Multiply::apply(&mut builder, input, scale).expect("the scaling applies");
    let mapped = F32Multiply::apply(&mut builder, scaled, unit).expect("the unit multiply applies");
    builder
        .output(
            OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
            mapped,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Evaluates the same semantic program through the independent oracle.
///
/// Returns the output bits **and** the reference registry's own canonical
/// identity. The two identities are deliberately reported side by side and
/// never conflated.
#[must_use]
pub fn reference_bits(program: &SemanticProgram) -> (Vec<u32>, Vec<u8>) {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([ROWS, COLUMNS]),
        OPERANDS
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed");
    let evaluator =
        ReferenceEvaluator::standard().expect("the governed reference profile composes");
    let identity = evaluator
        .registry()
        .canonical_identity()
        .as_bytes()
        .to_vec();
    let outputs = evaluator
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("this program declares one dense f32 output");
    };
    let bits = elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect();
    (bits, identity)
}
