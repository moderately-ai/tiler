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
//! # What the rule is, and the refusal it is not
//!
//! The rule has two guards, and they are one rule about *sides* written for the
//! two recognized shapes that can place an edge: `recognize_staged_family`'s
//! `staged-operand-depth` for a staged occurrence's operand, and
//! `ReductionContributorAdmission`'s `reduction-contributor-depth` for a fold's
//! contributor. Both are reached only through `recognize_epilogue_producer`,
//! which is the one function recognition enters across an edge and the only site
//! that hands either `NoEdge`.
//!
//! One neighbouring refusal also fires on a folded value and says something
//! else, and conflating them is how a widener deletes the wrong guard:
//! `sum(a, 1) * sum(b, 1)` refuses because one region would read **two** edges,
//! and `TensorRole::Intermediate` carries no ordinal to attribute them by. That
//! is chain *width*; the walk is still one boundary deep.
//! [`admit-a-scheduled-region-that-reads-two-materialization-edges`] owns it.
//!
//! **`sum(sum(x) * 2.0)` was a third wall here and is not one now.** It refused
//! because `NormalizedSerialSum` carried no producer field for the discovered
//! boundary to hang on, so the finding was discarded before any admission ran,
//! and it reported `reduction-contributor-materialization`.
//! `replace-the-serial-sum-contributor-fields-with-the-exhaustive-source` gave
//! the recognized fold a contributor source whose materialized arm retains the
//! producing shape and the elementwise continuation between it and the fold, so
//! that program compiles and the retired rule is unreachable. What refuses in
//! its place is one edge further out — `sum(sum(sum(x) * 2.0) * 2.0)` — under
//! the reduction guard above, which is the depth rule proper rather than a
//! structural wall.
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
//! `rms_norm(matmul(a, b), w)` remains uncompiled, with the complete causes
//! determined at each of the fixture's five named F32 preset points. Strict and
//! flush-only isolate the region-vocabulary wall as `UnsupportedCapability`;
//! reassociation-permitting presets add fusion-legality `Unknown` and remain
//! `NoFeasiblePlan`. It is
//! that file's assertion rather than this file's so one measurement keeps one
//! owner.
//! When [`admit-a-scheduled-region-that-reads-two-materialization-edges`] lands,
//! that test fails, and
//! [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`]
//! should be reopened rather than the assertion relaxed.
//!
//! [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`]: ../../../tickets/admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md
//! [`admit-a-scheduled-region-that-reads-two-materialization-edges`]: ../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md
//! [`replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`]: ../../../tickets/replace-the-serial-sum-contributor-fields-with-the-exhaustive-source.md

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32Multiply, F32RmsNorm,
    F32TensorContraction, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Shape};

#[path = "support/staged_rms_profile.rs"]
mod staged_rms_profile;
use staged_rms_profile::{RmsRealizationFixture, staged_rms_profile};

/// The five named F32 contract points this boundary suite exercises.
///
/// Named together rather than sampled at one preset, for the reason
/// `staged_family_over_a_materialized_intermediate.rs` states: the schedule wall
/// and uncompiled outcome are structural, while the public class depends on
/// whether a preset adds fusion-legality `Unknown` to the cause census. This is
/// not the complete population of caller-composable numerical contracts.
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
/// This builder is the program-structure neighbour that makes the refusal above
/// attributable. Relative to [`two_boundary_chain`], it keeps the declared
/// inputs, contraction, and trailing multiply against the same independent
/// weight and removes the normalization between them, taking the second
/// materialization boundary with it. Target-profile authority is deliberately
/// outside this builder comparison: the test documentation below records why
/// the RMS subject uses the synthetic profile while this RMS-free control uses
/// the governed profile.
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

/// Compiles one program under one contract against one exact profile.
fn compile_under(
    program: &SemanticProgram,
    contract: NumericalContract,
    profile: &TargetProfile,
) -> Result<(), CompileFailureClass> {
    let targets = TargetRequest::new([profile.clone()]).unwrap();
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
    let profile = staged_rms_profile(RmsRealizationFixture::Discharging);
    let mut refused = 0;
    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&program, contract, &profile),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "staged-operand-depth",
            }),
            "{contract:?} did not reach the recognized chain-depth rule",
        );
        refused += 1;
    }
    assert_eq!(refused, CONTRACTS.len());
}

/// The RMS-free chain one boundary shallower compiles under the governed profile.
///
/// Without it the assertion above is consistent with a broken session boundary
/// or a fixture that never reaches the recognizer, and this file would be
/// evidence for nothing. It deliberately does not borrow the subject's
/// synthetic RMS authority: removing RMS also makes the governed profile a
/// sufficient independent control. The subject and neighbour share their
/// declared inputs, contraction, trailing multiply, and five named F32 preset
/// points, but not their target profile. The count is asserted rather than
/// described so a population that stopped compiling cannot look like a
/// population that never ran.
///
/// Watched failing under a deliberate perturbation of the subject, and of this
/// property alone: replacing `physical::spell_output`'s epilogue arm with
/// `Err(RegionVocabularyWall::PartialCoverage)` reports *all five named contract
/// presets must compile the one-boundary chain, or the refusal above is evidence
/// about the session boundary rather than about chain depth*, while the depth
/// assertion above stays green. The two perturbations are separate because
/// either alone leaves the other's claim standing, which is what says each
/// assertion is load-bearing on its own.
#[test]
fn the_governed_rms_free_one_boundary_neighbour_compiles_at_all_five_preset_points() {
    let control = one_boundary_chain();
    let profile = TargetProfile::governed();
    let compiled = CONTRACTS
        .into_iter()
        .filter(|contract| compile_under(&control, *contract, &profile).is_ok())
        .count();
    assert_eq!(
        compiled,
        CONTRACTS.len(),
        "all five named contract presets must compile the one-boundary chain, or the refusal \
         above is evidence about the session boundary rather than about chain depth",
    );
}
