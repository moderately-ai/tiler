//! Where `tiler::softmax-f32@1`'s ceiling actually is, demonstrated.
//!
//! The softmax family is admitted at R5: registered, reference-evaluated, given a
//! fusion role and a capability row, and carrying a structured-kernel construct
//! and a Metal emission. It nevertheless compiles no whole program, and the
//! reason is *not* anything about the family — it is that
//! `select_supported_strategy` has no shape for it.
//!
//! **The ceiling is no longer shared with `tiler::rms-norm-f32@1`, and the
//! difference is exactly one registry row.** The recognizer's staged arm is
//! law-derived: an occurrence whose registered `IndexRealizationLaw` realizes a
//! region *sequence* is recognized as a program stage, with no operation key
//! named. The normalization carries `StagedRootMeanSquareScaleF32` and is
//! therefore recognized, reaches its own lowering, has both of its realization
//! stages spelled by scheduled regions, and now has no ceiling above it at all:
//! it compiles end to end and its dispatched kernels agree with
//! `tiler-reference` bit for bit
//! (`pipeline::tests::a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`).
//! The softmax carries no law at all, so the same arm answers `false` for it and
//! the refusal below is the recognizer's — which is why the assertions here are
//! unchanged and are now about *this* family rather than about a class of them.
//!
//! **This file exists so that the claim is checked rather than asserted in a
//! roadmap cell.** A ceiling stated only in prose drifts silently in both
//! directions: it stays written after a recognizer widens, and it gets copied
//! forward onto a family whose ceiling is somewhere else. The control below is
//! what keeps the refusal from being consistent with a broken session boundary.
//!
//! **What the softmax still lacks, in order.** A softmax occurrence realizes as
//! *three* regions — a maximum fold, an exponential-and-sum pass, and a
//! normalizing pass. Emitting a chain is no longer the obstacle:
//! `IndexAccessLoweringProvider::lower_sequence` emits an ordered chain and
//! `GovernedRootMeanSquareScaleF32` is a shipped provider that does. What is
//! missing is the law, which needs a governed **maximum** scalar key that
//! [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`] owns
//! *and* a handed value with more than one reader, which
//! [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`]
//! owns. Once the law is registered this file's refusal moves to the same
//! vocabulary wall the normalization sits at.
//!
//! [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`]: ../../../tickets/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md
//! [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`]: ../../../tickets/admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md

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
