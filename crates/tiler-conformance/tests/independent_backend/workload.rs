//! The program this fixture routes, and the only authority on what it computes.
//!
//! `tiler-reference` is the sole mathematical oracle here. Nothing in this file
//! states an expected value: the expectation is whatever the reference
//! evaluator returns for the same semantic graph the compiler was handed. A
//! fixture that restated the arithmetic would be comparing one implementation
//! against a second copy of itself, which is the shared-implementation failure
//! `docs/correctness-and-testing.md` names and the exact defect the "callbacks
//! that can manufacture success" clause of the conformance trigger is about.

use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// Rows of this fixture's workload.
pub(crate) const ROWS: u64 = 3;

/// Columns of this fixture's workload.
pub(crate) const COLUMNS: u64 = 4;

/// Interface key of the program's one input.
pub(crate) const INPUT_KEY: &str = "operand";

/// Interface key of the program's one output.
pub(crate) const OUTPUT_KEY: &str = "folded";

/// The operand pattern this workload is filled from.
///
/// **Subnormals are present, and that is a consequence of the profile rather
/// than a preference.** This backend declares `SubnormalMode::Preserve` exact
/// and both flushing modes unsupported, and it compiles under `STRICT_F32`, so
/// a preserved subnormal is what both this evaluator and the reference are
/// required to produce. A backend assessed under a flush-to-zero contract could
/// not carry these operands at all, which is why the retained three-family
/// portfolio's Metal-assessed vector omits them; the difference is in the
/// declared numerics, not in the fixture's taste.
pub(crate) const OPERANDS: [u32; 12] = [
    0x3f80_0000, // 1.0
    0x0000_0001, // least positive subnormal
    0x8000_0000, // -0.0
    0x7fc0_4321, // a non-canonical NaN payload
    0xff80_0000, // -inf
    0x7f80_0000, // +inf
    0x4049_0fdb, // 3.14159274
    0xc000_0000, // -2.0
    0x0080_0000, // least positive normal
    0x477f_e000, // 65504.0
    0x3eaa_aaab, // 1/3, rounded
    0xbf00_0000, // -0.5
];

/// The scale the program's first multiply applies.
pub(crate) const SCALE_BITS: u32 = 0x4040_0000;

/// The scale the program's second multiply applies.
pub(crate) const HALVE_BITS: u32 = 0x3f00_0000;

/// Builds the semantic program this fixture packages and routes.
///
/// `(operand * 3.0) * 0.5`. Two multiplies rather than a multiply and an add
/// because the compiler's pointwise normalization refuses the mixed shape, so
/// this is the smallest program that reaches a plan at all — a constraint of
/// the compiler this fixture compiles against, not a shape borrowed from
/// another fixture. The two constants are its own.
pub(crate) fn program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed semantic profile composes");
    let operand = builder
        .input::<F32>(
            InputKey::new(INPUT_KEY).expect("the input key is valid"),
            Shape::from_dims([ROWS, COLUMNS]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, SCALE_BITS).expect("the scale applies");
    let halve = F32Constant::apply(&mut builder, HALVE_BITS).expect("the halving applies");
    let scaled = F32Multiply::apply(&mut builder, operand, scale).expect("the scaling applies");
    let folded = F32Multiply::apply(&mut builder, scaled, halve).expect("the halving applies");
    builder
        .output(
            OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
            folded,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Evaluates the same semantic graph through the independent oracle.
///
/// The returned bits are the only expectation this fixture holds. They are
/// derived, never written down.
pub(crate) fn reference_bits(program: &SemanticProgram) -> Vec<u32> {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([ROWS, COLUMNS]),
        OPERANDS
            .iter()
            .map(|bits| {
                ReferenceElement::from_float_bits(
                    bits.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed");
    let evaluator =
        ReferenceEvaluator::standard().expect("the governed reference profile composes");
    let outputs = evaluator
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("this program declares one dense f32 output")
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}
