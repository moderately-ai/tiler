//! Where `tiler::softmax-f32@1`'s ceiling actually is, demonstrated.
//!
//! The softmax family is admitted at R5: registered, reference-evaluated, given a
//! fusion role and a capability row, and carrying a structured-kernel construct
//! and a Metal emission. It nevertheless compiles no whole program, and the
//! reason is *not* anything about the family — it is that
//! `select_supported_strategy` recognizes three whole-program shapes and none of
//! them contains a softmax. That is the same ceiling holding
//! `tiler::silu-f32@1` and `tiler::rms-norm-f32@1`, and it belongs to
//! [`reach-a-verified-kernel-through-the-structural-families`].
//!
//! **This file exists so that the claim is checked rather than asserted in a
//! roadmap cell.** A ceiling stated only in prose drifts silently in both
//! directions: it stays written after a recognizer widens, and it gets copied
//! forward onto a family whose ceiling is somewhere else. The control below is
//! what keeps the refusal from being consistent with a broken session boundary.
//!
//! **It also records what is deliberately *not* here.** This vertical did not
//! widen the recognizer, and it registered no index-access lowering capability.
//! A softmax occurrence realizes as *three* regions — a maximum fold, an
//! exponential-and-sum pass, and a normalizing pass — and what once blocked that
//! was `GovernedIndexAccess` emitting exactly one region per occurrence. That
//! limit is gone: `IndexAccessLoweringProvider::lower_sequence` emits an ordered
//! chain, and `GovernedRootMeanSquareScaleF32` is a shipped provider that does.
//! What this family still lacks is its own `IndexRealizationLaw` — which needs a
//! governed **maximum** scalar key that
//! [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`] owns —
//! and, above that, the recognizer widening this file's assertions pin.
//!
//! [`reach-a-verified-kernel-through-the-structural-families`]: ../../../tickets/reach-a-verified-kernel-through-the-structural-families.md
//! [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`]: ../../../tickets/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, F32Softmax, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Shape};

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled, because the outcome is structural: a
/// contract under which the softmax program compiled would mean the ceiling is
/// not where this file says it is.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// A one-occurrence softmax program over the C1 row's score shape.
fn softmax_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let scores = builder
        .input::<F32>(
            InputKey::new("s").unwrap(),
            Shape::from_dims([8, 2, 10, 10]),
        )
        .unwrap();
    let weights = F32Softmax::apply(&mut builder, scores, Axis::new(3)).unwrap();
    builder
        .output(OutputKey::new("w").unwrap(), weights)
        .unwrap();
    builder.build().unwrap()
}

/// The control: a recognized standalone pointwise program, `(a * 2.0) * 3.0`.
///
/// Without a program that compiles under the identical request, "the softmax
/// refuses" would be consistent with a broken target profile or an unusable
/// session boundary, and this file would prove nothing about the recognizer.
fn recognized_control() -> SemanticProgram {
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
            outcome
                .outcome()
                .map(|_| ())
                .map_err(TargetCompileFailure::class)
        }
        Err(failure) => Err(failure.class()),
    }
}

/// A softmax program is refused at the recognizer, under every contract.
///
/// The refusal is uniform across all four contracts, which is what shows it is
/// *structural* rather than numerical: no permission a caller can grant admits
/// the shape, because the shape is not one the recognizer knows at all.
#[test]
fn a_softmax_program_is_refused_by_the_whole_program_recognizer() {
    let program = softmax_program();
    for contract in CONTRACTS {
        assert!(
            compile_under(&program, contract).is_err(),
            "the recognizer admits no whole-program shape containing a softmax, \
             and {contract:?} is not what would change that"
        );
    }
}

/// The control compiles under the same request, so the refusal is the recognizer's.
#[test]
fn the_recognized_control_compiles_under_the_same_request() {
    let control = recognized_control();
    let mut compiled = 0_usize;
    for contract in CONTRACTS {
        if compile_under(&control, contract).is_ok() {
            compiled += 1;
        }
    }
    assert!(
        compiled > 0,
        "at least one contract must compile the recognized shape, or the refusal above \
         is evidence about the session boundary rather than about the softmax"
    );
}

/// The program itself is well formed, so the refusal is not a construction error.
///
/// This is the half that makes the ceiling attributable. A softmax occurrence
/// verifies, infers its shape, and reaches a built `SemanticProgram`; what it
/// does not reach is a recognized whole-program strategy. Those are different
/// failures and the roadmap row would be wrong if it named the first.
#[test]
fn the_refused_program_is_itself_a_verified_semantic_program() {
    let program = softmax_program();
    assert_eq!(program.operations().count(), 1);
    let occurrence = program.operations().next().expect("one occurrence");
    assert_eq!(
        occurrence.key(),
        &tiler_ir::semantic::softmax_f32_op(),
        "the program's one operation is the softmax"
    );
}
