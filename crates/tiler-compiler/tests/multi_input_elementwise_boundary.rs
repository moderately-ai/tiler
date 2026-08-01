//! Where the multi-input elementwise boundary is, and what actually holds it.
//!
//! The approved `tensor!` region `sym n; in a, b, c; out (a * b) + c` does not
//! compile. That fact has now been derived by hand three times — at `b623670`,
//! at `e6a47d9`, and again here — because it lived only in ticket prose, so
//! every reader who needed it paid to rediscover it. This file is that
//! measurement made executable, and it exists to answer two different
//! questions that a bare "it refuses" answer conflates.
//!
//! The first is *what the public boundary does today*: refuse, under every
//! stated numerical contract, before any target-qualified trace opens — and
//! refuse this program specifically rather than refusing everything, which the
//! compiling one-input control proves.
//!
//! The second is *what would have to change to stop refusing*, and this is the
//! part prose kept getting wrong by naming the recognizer. Widening
//! `normalize_pointwise` in `tiler-compiler` cannot admit the region, because
//! the layer below has no way to say "a second input tensor". A region's
//! `ScalarProgram::PointwiseF32` carries a single `PointwiseF32Node::Input`
//! leaf; `PointwiseF32ExpressionBuilder::build` is its only constructor and it
//! refuses a second input outright. So a widened recognizer could only produce
//! a program the physical layer cannot express — admitted at the boundary and
//! failing mid-pipeline, which is strictly worse than the refusal.
//!
//! The obstruction test below is therefore not incidental colour: it is the
//! evidence that this ticket's work belongs in `tiler-ir`, not here. When that
//! widening lands, both tests must change together — the refusal becomes a
//! compilation, and the `DuplicateInput` expectation becomes an indexed input
//! leaf. Their edit is what makes the transition demonstrated rather than
//! asserted.

use tiler_compiler::session::{CompileFailureClass, CompileRequest, NumericalContract, compile};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::schedule::{PointwiseF32ExpressionAdmissionError, PointwiseF32ExpressionBuilder};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled: the refusal is structural, so a
/// contract that admitted the program would mean the boundary moved for a
/// reason this file does not model, and sampling one preset would hide it.
const CONTRACTS: [NumericalContract; 4] = [
    NumericalContract::StrictF32,
    NumericalContract::FlushSubnormalsToZeroF32,
    NumericalContract::RelaxedF32,
    NumericalContract::ReassociateF32,
];

/// The approved inline region: `sym n; in a, b, c; out (a * b) + c`.
///
/// Three tensor inputs and no constant, which is every region the approved
/// grammar can express: its body has exactly the operand and binary-`*`/`+`
/// productions, so a region always has N tensor inputs and zero constants.
fn three_input_region() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let c = builder
        .input::<F32>(InputKey::new("c").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, a, b).unwrap();
    let sum = F32Add::apply(&mut builder, product, c).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// The control: `(a * 2.0f32) * 3.0f32`, one input and two constants.
///
/// This is the recognized standalone pointwise shape — four operations, one
/// tensor input — and it is here to keep the refusal above honest. Without a
/// program that compiles under the identical request, "refuses" would be
/// consistent with a broken target profile or an unusable session boundary,
/// and the file would prove nothing about input cardinality.
fn one_input_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let three = F32Constant::apply(&mut builder, 3.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, a, two).unwrap();
    let root = F32Multiply::apply(&mut builder, scaled, three).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// Compiles one program under one contract against the governed profile.
fn compile_under(
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<(), CompileFailureClass> {
    let targets = TargetRequest::new([TargetProfile::governed()]).unwrap();
    match compile(CompileRequest::new(program, contract, targets)) {
        Ok(batch) => {
            let outcome = batch.targets().next().expect("one requested profile");
            outcome.outcome().map(|_| ()).map_err(|failure| {
                panic!("the governed profile refused per-target: {failure:?}");
            })
        }
        Err(failure) => {
            assert!(
                failure.explain().is_none(),
                "a strategy-admission refusal precedes any target-qualified trace",
            );
            Err(failure.class())
        }
    }
}

/// The approved three-input region refuses, and a one-input program does not.
#[test]
fn the_three_input_region_refuses_under_every_contract() {
    let region = three_input_region();
    assert_eq!(region.input_count(), 3);
    assert_eq!(region.operation_count(), 2);
    let control = one_input_control();
    assert_eq!(control.input_count(), 1);
    assert_eq!(control.operation_count(), 4);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability { rule: "signature" }),
            "{contract:?} admitted the three-input region",
        );
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the recognized one-input control, so the \
             refusal above is not specific to input cardinality",
        );
    }
}

/// The physical `f32` expression cannot name a second input tensor.
///
/// This is the obstruction, and it sits below `tiler-compiler` entirely. The
/// expression's fields are private and `build` is its only constructor, so no
/// recognizer this crate could write is able to route around the refusal —
/// which is why admitting the region is `tiler-ir` work.
#[test]
fn the_physical_pointwise_expression_admits_exactly_one_input() {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input().expect("the first input is admitted");
    assert_eq!(
        expression.input().unwrap_err(),
        PointwiseF32ExpressionAdmissionError::DuplicateInput,
        "a second input tensor must be refused by the physical vocabulary",
    );
    // A constant is still admitted after the refusal, so the rejection is of
    // the second *input* and not of any further node.
    let constant = expression
        .constant(2.0_f32.to_bits())
        .expect("constants remain admissible");
    let root = expression.multiply(input, constant).unwrap();
    assert!(expression.build(root).is_ok());
}
