//! Where the multi-input elementwise boundary was, and what moved it.
//!
//! The approved `tensor!` region `sym n; in a, b, c; out (a * b) + c` now
//! compiles. It did not at `b623670`, at `e6a47d9`, or when this file first
//! recorded the refusal, and the reason was never the recognizer that observed
//! it: the scheduled-region vocabulary below `tiler-compiler` had no way to say
//! "a second input tensor", so `TensorRole::Input` named a class of boundary
//! tensor without saying which, and `ScalarProgram::PointwiseF32` carried a
//! single nullary `PointwiseF32Node::Input` leaf. A recognizer widened on its
//! own could only have produced a program the physical layer cannot express —
//! admitted at the boundary and failing mid-pipeline, which is strictly worse
//! than refusing.
//!
//! Both halves moved together, and this file is what makes that transition
//! demonstrated rather than asserted. The first test was the refusal and is now
//! the compilation, reaching a complete verified plan rather than merely
//! passing strategy selection. The second was the obstruction — a second
//! `input()` returning `DuplicateInput` — and is now the indexed leaf that
//! replaced it.
//!
//! The one-input control stays, and its job is unchanged: without a program
//! that compiles under the identical request, an assertion about the
//! three-input region would be consistent with a broken target profile rather
//! than with anything about input cardinality.
//!
//! # The one contract that still refuses, and why it is not this boundary
//!
//! `RelaxedF32` is the contract that *permits* arithmetic contraction, and
//! under it the approved region has no feasible plan. That refusal is neither a
//! defect nor a residue of the widening: `fusion_legality`'s
//! `ArithmeticContraction` obligation returns `unrealized-contraction` for any
//! region holding a multiply adjacent to an add when the contract permits
//! contraction, because fusing them exposes an FMA the materialized form cannot
//! perform. `a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`
//! records that this was *eliminated rather than deferred*, with a measurement:
//! a permitting realization carries no `NoFloatingPointContraction` obligation
//! into the artifact, and the measured Apple row fuses a written multiply/add
//! pair under `-ffp-contract=fast`.
//!
//! The pinned pair below is the evidence that it is orthogonal to input
//! cardinality. A one-input `(a * 2.0) + 3.0` refuses under exactly the same
//! contract, and the same-family one-input control compiles under all four — so
//! what `RelaxedF32` declines is the mixed multiply/add body, at any input
//! count. Admitting it needs either a physical form that declares its
//! contraction or an implementation for the materialized cover's single-operation
//! regions; both are separate work, and neither is a vocabulary question.
//!
//! What still refuses for *recognition* is recorded here too. Widening the
//! vocabulary is not licence to accept an unrecognized program, so a body
//! outside the recognized two-operation shape must still refuse with a typed
//! reason naming what was not recognized.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::schedule::{
    InputOrdinal, PointwiseF32ExpressionBuilder, PointwiseF32ExpressionDiagnostic, PointwiseF32Node,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled: the outcome is structural, so a
/// contract that behaved differently would mean the boundary moved for a reason
/// this file does not model, and sampling one preset would hide it.
const CONTRACTS: [NumericalContract; 4] = [
    NumericalContract::StrictF32,
    NumericalContract::FlushSubnormalsToZeroF32,
    NumericalContract::RelaxedF32,
    NumericalContract::ReassociateF32,
];

/// The one contract that permits arithmetic contraction.
///
/// Named rather than matched inline so the reason a mixed multiply/add body
/// behaves differently under it is the *contraction permission* and not the
/// preset's name.
const CONTRACTION_PERMITTED: NumericalContract = NumericalContract::RelaxedF32;

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

/// `(a * 2.0f32) + 3.0f32`: one input, two constants, mixed families.
///
/// Differs from [`one_input_control`] in exactly one place — the root operation
/// — which is what makes the pair evidence about the multiply/add adjacency
/// rather than about anything else the two programs share.
fn one_input_mixed_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let three = F32Constant::apply(&mut builder, 3.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, a, two).unwrap();
    let root = F32Add::apply(&mut builder, scaled, three).unwrap();
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
            outcome
                .outcome()
                .map(|_| ())
                .map_err(TargetCompileFailure::class)
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

/// A region the widened vocabulary still cannot express: `(a * b) + (c * c)`.
///
/// Three inputs, so it stays inside every governed budget, but three operations
/// where the recognized body has two — the root's second operand is produced
/// rather than being a leaf. This is the program that keeps the widening honest:
/// what landed is "N inputs across the recognized body", not "anything with
/// several inputs".
fn unrecognized_three_input_region() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let mut inputs = Vec::new();
    for key in ["a", "b", "c"] {
        inputs.push(
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([4]))
                .unwrap(),
        );
    }
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let square = F32Multiply::apply(&mut builder, inputs[2], inputs[2]).unwrap();
    let root = F32Add::apply(&mut builder, product, square).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// The approved three-input region compiles under every contract that admits a
/// mixed multiply/add body.
///
/// `compile_under` returns only after the per-target outcome resolves, so each
/// pass is a complete verified plan and not merely successful strategy
/// selection.
#[test]
fn the_three_input_region_compiles_wherever_a_mixed_body_is_admitted() {
    let region = three_input_region();
    assert_eq!(region.input_count(), 3);
    assert_eq!(region.operation_count(), 2);
    let control = one_input_control();
    assert_eq!(control.input_count(), 1);
    assert_eq!(control.operation_count(), 4);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the recognized one-input control, so nothing \
             this file asserts about the three-input region would be evidence \
             about input cardinality",
        );
        if contract == CONTRACTION_PERMITTED {
            continue;
        }
        assert_eq!(
            compile_under(&region, contract),
            Ok(()),
            "{contract:?} refused the approved three-input region",
        );
    }
}

/// The contraction-permitting contract declines a mixed body at any input count.
///
/// Both halves are asserted together because the pair is the claim: the
/// three-input region and a one-input `(a * 2.0) + 3.0` refuse identically, so
/// the refusal reads the multiply/add adjacency and not the number of tensors.
/// Without the one-input half this would be indistinguishable from a
/// multi-input defect, which is exactly the misreading the ticket this file
/// belongs to spent three measurements correcting.
#[test]
fn the_contraction_permitting_contract_declines_a_mixed_body_at_any_input_count() {
    for program in [three_input_region(), one_input_mixed_control()] {
        assert_eq!(
            compile_under(&program, CONTRACTION_PERMITTED),
            Err(CompileFailureClass::NoFeasiblePlan),
        );
    }
    // The same one input and the same two constants, multiplied twice instead
    // of multiplied and added, compiles — so what is declined is the adjacency.
    assert_eq!(
        compile_under(&one_input_control(), CONTRACTION_PERMITTED),
        Ok(()),
    );
}

/// A program outside the recognized body still refuses, with a named rule.
///
/// The refusal names the operation family the leaf walk expected and did not
/// find, rather than the combined `signature` this file used to assert: each
/// gate now names its own refusal, so a reader learns which of input arity,
/// output arity, the operation set, or the dtype was not recognized instead of
/// having to re-derive it.
#[test]
fn an_unrecognized_multi_input_region_still_refuses_with_a_typed_reason() {
    let region = unrecognized_three_input_region();
    assert_eq!(region.input_count(), 3);
    assert_eq!(region.operation_count(), 3);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "operation-family"
            }),
            "{contract:?} admitted a body the recognizer does not cover",
        );
    }
}

/// The physical `f32` expression names which input tensor each leaf reads.
///
/// This is the obstruction that used to sit below `tiler-compiler`: the builder
/// refused a second input outright, so no recognizer this crate could write was
/// able to route around it. It now takes an ordinal, and two ordinals are two
/// distinct leaves.
#[test]
fn the_physical_pointwise_expression_names_each_input_tensor() {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let a = expression
        .input(InputOrdinal::new(0))
        .expect("the first input is admitted");
    let b = expression
        .input(InputOrdinal::new(1))
        .expect("a second input tensor is now nameable");
    let c = expression.input(InputOrdinal::new(2)).expect("and a third");
    let product = expression.multiply(a, b).unwrap();
    let root = expression.add(product, c).unwrap();
    let built = expression
        .build(root)
        .expect("the dense ordinal set builds");
    assert_eq!(built.input_count(), 3);
    assert_eq!(
        built.nodes()[0],
        PointwiseF32Node::Input {
            ordinal: InputOrdinal::new(0)
        },
    );

    // The vocabulary is still bounded: an ordinal set with a gap names a
    // binding no read access would supply, and is refused by build.
    let mut sparse = PointwiseF32ExpressionBuilder::new();
    let first = sparse.input(InputOrdinal::new(0)).unwrap();
    let third = sparse.input(InputOrdinal::new(2)).unwrap();
    let root = sparse.add(first, third).unwrap();
    assert_eq!(
        sparse.build(root).unwrap_err().diagnostic(),
        PointwiseF32ExpressionDiagnostic::SparseInputOrdinals { missing: 1 },
    );
}
