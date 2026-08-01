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
//! What still refuses for *recognition* is recorded here too. Widening is not
//! licence to accept an unrecognized program, so a program the physical layer
//! cannot realize must still refuse with a typed reason naming what was not
//! recognized.
//!
//! # Where that recognition boundary moved next
//!
//! When this file first landed, the recognized body was exactly two operations
//! over three leaves, and `(a * b) + (c * c)` was the program that kept the
//! widening honest by refusing. It no longer refuses: the request boundary's
//! three whole-program templates were replaced by a general walk over the
//! occurrences an expression contains, so depth, arity, family mixing, and
//! shared reads are now properties of the caller's program rather than of a
//! shape the recognizer was taught. That program is asserted below as a
//! *compilation*, and the refusal it used to provide is now carried by
//! `tiler::silu-f32@1` — a registered semantic family with a registered
//! lowering capability and no `PointwiseF32Node` to spell it, which is a
//! physical-vocabulary wall in exactly the sense the second input tensor once
//! was.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::schedule::{
    InputOrdinal, PointwiseF32ExpressionBuilder, PointwiseF32ExpressionDiagnostic, PointwiseF32Node,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, F32Silu, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled: the outcome is structural, so a
/// contract that behaved differently would mean the boundary moved for a reason
/// this file does not model, and sampling one preset would hide it.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// The one contract that permits arithmetic contraction.
///
/// Named rather than matched inline so the reason a mixed multiply/add body
/// behaves differently under it is the *contraction permission* and not the
/// preset's name.
const CONTRACTION_PERMITTED: NumericalContract = NumericalContract::RELAXED_F32;

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

/// A region the two-operation body could not express: `(a * b) + (c * c)`.
///
/// Three inputs and three operations, where the superseded recognized body had
/// two — the root's second operand is produced rather than being a leaf. It was
/// this file's refusal case and is now its depth case: the general recognizer
/// walks to it, so what the boundary admits is bounded by the expression
/// vocabulary rather than by a leaf count.
fn deeper_three_input_region() -> SemanticProgram {
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

/// The deeper three-input body compiles wherever a mixed body is admitted.
///
/// This is the assertion that changed direction. `(a * b) + (c * c)` refused
/// here until the request boundary generalized, and it refused for its *depth*
/// — the root's second operand is produced rather than being a leaf — which was
/// a property of the template rather than of anything the physical layer could
/// not do. It is now recognized by the same walk that recognizes `(a * b) + c`,
/// and it compiles to a complete verified plan under every contract that admits
/// a mixed multiply/add body.
///
/// The one-input control travels with it for the reason it always has: without
/// a program that compiles under the identical request, a pass here would be
/// consistent with a permissive target rather than with anything about the body.
#[test]
fn the_deeper_three_input_region_compiles_wherever_a_mixed_body_is_admitted() {
    let region = deeper_three_input_region();
    assert_eq!(region.input_count(), 3);
    assert_eq!(region.operation_count(), 3);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&one_input_control(), contract),
            Ok(()),
            "{contract:?} refused the recognized one-input control, so nothing \
             this test asserts about the deeper body would be evidence about \
             expression depth",
        );
        if contract == CONTRACTION_PERMITTED {
            continue;
        }
        assert_eq!(
            compile_under(&region, contract),
            Ok(()),
            "{contract:?} refused the deeper three-input body",
        );
    }
}

/// A family the region vocabulary cannot spell still refuses, with a named rule.
///
/// `tiler::silu-f32@1` is registered semantics *and* a registered index-access
/// lowering capability, and it is still refused at the request boundary — which
/// is the whole point of the pair. Recognition generalized over the expression
/// vocabulary, not over everything the compiler can lower: no
/// `PointwiseF32Node` spells a sigmoid-weighted linear unit, and decomposing one
/// here into multiply, exp, add, and divide nodes would be this boundary
/// re-deriving a provider's lowering rather than resolving it.
///
/// The refusal names `operation-set` — the property that was not recognized —
/// and precedes any target-qualified trace, which `compile_under` asserts.
#[test]
fn a_family_outside_the_expression_vocabulary_refuses_with_a_typed_reason() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let activated = F32Silu::apply(&mut builder, a).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), activated)
        .unwrap();
    let region = builder.build().unwrap();
    assert_eq!(region.input_count(), 1);
    assert_eq!(region.operation_count(), 1);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "operation-set"
            }),
            "{contract:?} admitted a family no scalar program spells",
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
