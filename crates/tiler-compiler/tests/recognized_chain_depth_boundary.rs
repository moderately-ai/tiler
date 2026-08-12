//! How deep a recognized chain may be, and the measured reason the ceiling is
//! where it is.
//!
//! A *chain* here is a run of regions separated by materialization edges: a
//! folding family writes a value, and the region across the edge reads it. The
//! recognizer admits chains **one** edge deep and refuses the rest by name.
//! `crate::request`'s `StagedOperandAdmission` is the single statement of the
//! rule; this file is the end-to-end measurement of where it leaves a caller,
//! and the trigger that says when the reason for it has expired.
//!
//! # What the rule is, and the two refusals it is not
//!
//! The rule has one guard — `recognize_staged_family`'s `staged-operand-depth`,
//! reached only through `recognize_epilogue_producer`, which is the one function
//! recognition enters across an edge. Two neighbouring refusals also fire on a
//! folded value and say something else, and conflating them is how a widener
//! deletes the wrong guard:
//!
//! - `sum(a, 1) * sum(b, 1)` refuses because one region would read **two** edges,
//!   and `TensorRole::Intermediate` carries no ordinal to attribute them by. That
//!   is chain *width*; the walk is still one boundary deep.
//!   [`admit-a-scheduled-region-that-reads-two-materialization-edges`] owns it.
//! - `sum(sum(x) * 2.0)` refuses because `NormalizedSerialSum` carries no
//!   producer field for a fold's prologue to hang a boundary on, so the discovery
//!   is discarded before any admission runs. It reports
//!   `reduction-contributor-materialization`: that one *is* about depth, but its
//!   wall is structural rather than the guard's.
//!   [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`]
//!   owns its rule name.
//!
//! # The measurement, taken 2026-08-08
//!
//! Handing `recognize_epilogue_producer`'s call site
//! `StagedOperandAdmission::OneEdge` instead of `NoEdge` was run and observed:
//!
//! - `rms_norm(matmul(a, b), w) * w` **is** recognized, into a well-formed
//!   `Epilogue { producer: Staged { producer: Some(Contraction) } }`. The nesting
//!   the widening would need already exists — `NormalizedEpilogue::producer`,
//!   `NormalizedStaged::producer`, every accessor over them, and both subject
//!   arms recurse — so nothing had to be built for the shape to be expressible.
//! - Exactly one test moved:
//!   `a_staged_operand_still_refuses_a_second_edge_and_a_deeper_chain`, the
//!   assertion of the refusal itself. The crate then held 784 tests — this file's
//!   two did not yet exist — and no cover, cost, identity, or subject assertion
//!   among the other 783 noticed.
//! - At that measurement's exact base, the program still did not compile. It
//!   refused `NoFeasiblePlan` instead of naming `staged-operand-depth`.
//!
//! **So the widening buys no program and costs a rule name.** Every program the
//! guard refuses contains a staged occurrence whose operand is an edge, and
//! `physical::staged_plan` has no region for one: its only law arm destructures
//! two `BoundaryRead::Input` operands, so such an occurrence is
//! `RegionVocabularyWall::StagedFamilyUnspellable` however deep the chain around
//! it is. Admitting the chain would move a recognition-time statement about the
//! caller's program into a target rejection that names neither the operand nor
//! the depth. It would also unbound the recognizer's producer recursion, whose
//! depth is the caller's chain and whose bound today is this guard.
//!
//! # The trigger
//!
//! The reason above expires the moment a scheduled region can read two
//! materialization edges. `staged_family_over_a_materialized_intermediate.rs`'s
//! `a_staged_family_over_an_edge_is_recognized_and_stops_at_the_region_vocabulary`
//! is what says so: it asserts that the *one*-boundary chain
//! `rms_norm(matmul(a, b), w)` remains uncompiled, with a class determined by
//! the complete causes under each contract. Strict and flush-only isolate the
//! region-vocabulary wall as `UnsupportedCapability`; reassociation-permitting
//! contracts add fusion-legality `Unknown` and remain `NoFeasiblePlan`. It is
//! that file's assertion rather than this file's so one measurement keeps one
//! owner.
//! When [`admit-a-scheduled-region-that-reads-two-materialization-edges`] lands,
//! that test fails, and
//! [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`]
//! should be reopened rather than the assertion relaxed.
//!
//! [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`]: ../../../tickets/admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md
//! [`admit-a-scheduled-region-that-reads-two-materialization-edges`]: ../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md
//! [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`]: ../../../tickets/name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32Multiply, F32RmsNorm,
    F32TensorContraction, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Shape};

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled, for the reason
/// `staged_family_over_a_materialized_intermediate.rs` states it: the schedule
/// wall and uncompiled outcome are structural, while the public class depends
/// on whether a contract adds fusion-legality `Unknown` to the cause census.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// `ab,bc->ac` over two `[2, 2]` contraction operands.
fn product(
    builder: &mut SemanticProgramBuilder,
    left: tiler_ir::semantic::Value<F32>,
    right: tiler_ir::semantic::Value<F32>,
) -> tiler_ir::semantic::Value<F32> {
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    F32TensorContraction::apply(builder, &structure, left, right).expect("the product")
}

/// Declares the three `[2, 2]` inputs both fixtures share.
fn inputs(
    builder: &mut SemanticProgramBuilder,
) -> (
    tiler_ir::semantic::Value<F32>,
    tiler_ir::semantic::Value<F32>,
    tiler_ir::semantic::Value<F32>,
) {
    let shape = Shape::from_dims([2, 2]);
    let left = builder
        .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("b").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), shape)
        .unwrap();
    (left, right, weight)
}

/// `rms_norm(matmul(a, b), w) * w`: a chain two materialization boundaries deep.
///
/// The contraction writes an edge the normalization's producing stage reads, and
/// the normalization writes an edge the trailing multiply reads. Each region
/// reads *one* intermediate, which is what makes this a depth question rather
/// than the width question `sum(a, 1) * sum(b, 1)` asks.
fn two_boundary_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let (left, right, weight) = inputs(&mut builder);
    let matmul = product(&mut builder, left, right);
    let normalized = F32RmsNorm::apply(
        &mut builder,
        matmul,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .expect("the normalization");
    let root = F32Multiply::apply(&mut builder, normalized, weight).expect("the trailing pass");
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// `matmul(a, b) * w`: the same chain one boundary shallower.
///
/// The neighbour that makes the refusal above attributable, and it is written to
/// differ by *one* thing. Same declared inputs, same contraction, same trailing
/// multiply against the same independent weight input; the only edit is that the
/// normalization between them is gone, taking the second materialization
/// boundary with it. So a caller can see the depth being refused rather than the
/// epilogue, the contraction, the operand, or the request.
fn one_boundary_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let (left, right, weight) = inputs(&mut builder);
    let matmul = product(&mut builder, left, right);
    let root = F32Multiply::apply(&mut builder, matmul, weight).expect("the trailing pass");
    builder
        .output(OutputKey::new("result").unwrap(), root)
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

/// A chain two materialization boundaries deep is refused by name, at
/// recognition.
///
/// **The rule string is what carries the claim, and a bare `is_err` would not.**
/// The neighbouring shapes refuse under `staged-operand` (an operand no region
/// materializes), `staged-operand-conflict` (two edges into one occurrence), and
/// `operation-set` (two edges into one walk) — all of them
/// `UnsupportedCapability` from the same phase — so only the name separates the
/// property that was actually declined. `NoFeasiblePlan` here would mean the
/// refusal had slipped past recognition into the region vocabulary, which is
/// precisely the regression the measurement in this file's header argues against
/// accepting.
///
/// Watched failing under a deliberate perturbation of the subject, and of this
/// property alone: handing `recognize_epilogue_producer`'s call site
/// `StagedOperandAdmission::OneEdge` reports *left: `Err(NoFeasiblePlan)`,
/// right: `Err(UnsupportedCapability { rule: "staged-operand-depth" })`*, while
/// the control below stays green.
#[test]
fn a_chain_two_materialization_boundaries_deep_refuses_at_recognition_by_name() {
    let program = two_boundary_chain();
    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&program, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "staged-operand-depth",
            }),
            "the chain is two materialization boundaries deep and the depth rule names it, \
             and {contract:?} is not what would change that",
        );
    }
}

/// The same chain one boundary shallower compiles under the same request.
///
/// Without it the assertion above is consistent with a broken session boundary
/// or a fixture that never reaches the recognizer, and this file would be
/// evidence for nothing. The count is asserted rather than described so a
/// population that stopped compiling cannot look like a population that never
/// ran.
///
/// Watched failing under a deliberate perturbation of the subject, and of this
/// property alone: replacing `physical::spell_output`'s epilogue arm with
/// `Err(RegionVocabularyWall::PartialCoverage)` reports *at least one contract
/// must compile the one-boundary chain, or the refusal above is evidence about
/// the session boundary rather than about chain depth*, while the depth
/// assertion above stays green. The two perturbations are separate because
/// either alone leaves the other's claim standing, which is what says each
/// assertion is load-bearing on its own.
#[test]
fn a_chain_one_materialization_boundary_deep_compiles_under_the_same_request() {
    let control = one_boundary_chain();
    let compiled = CONTRACTS
        .into_iter()
        .filter(|contract| compile_under(&control, *contract).is_ok())
        .count();
    assert!(
        compiled > 0,
        "at least one contract must compile the one-boundary chain, or the refusal above is \
         evidence about the session boundary rather than about chain depth",
    );
}
