//! Where the ordered multi-output boundary is, and which layer actually holds it.
//!
//! `select_supported_strategy` refuses every program declaring more than one
//! output under `output-arity` (`crates/tiler-compiler/src/request.rs:2491`), and
//! `verify_artifact_refinements` carries the same condition on the assembly path
//! (`crates/tiler-compiler/src/program.rs:1254`). This file exists because the
//! obvious reading of those two guards — that a schedule or artifact vocabulary
//! below `tiler-compiler` cannot say "a second program output", the way it once
//! could not say "a second input tensor" — is **wrong**, and acting on it would
//! send the widening at the wrong crate.
//!
//! # What the layer below can already do
//!
//! `tiler-ir`'s artifact program vocabulary expresses ordered multi-output today,
//! and its own tests prove it rather than this file asserting it:
//!
//! - `KernelProgramBuilder::push_output` is general, bounded by
//!   `tiler_ir::program::MAX_PROGRAM_OUTPUTS` (4096), not by one.
//! - `program::tests::storage_reuse_is_admitted_only_with_an_explicit_handoff`
//!   builds and *verifies* a two-output program — `sum_a` and `sum_b` over four
//!   stages and five allocations — and asserts `outputs().len() == 2`.
//! - `program::tests::a_missing_named_output_is_rejected` already discharges the
//!   rule that a plan naming fewer outputs than the program declares is refused,
//!   as `KernelProgramDiagnostic::MissingNamedOutput`.
//! - `KernelProgramBuildError::DuplicateOutput` already refuses two publications
//!   of one output key.
//!
//! So the multi-output wall is **not** the shape the multi-input one was. There
//! is no missing `tiler-ir` noun here to add before a guard may move, and the
//! sibling `TensorRole::Output` carrying no ordinal is not one either: a region
//! writes one owning tensor, several regions write several, and the program layer
//! binds each stage's buffers to values positionally — which
//! `tiler_ir::program::ValueRole::fills` states outright, and this file pins.
//!
//! # Where the wall actually is
//!
//! In `tiler-compiler`'s own planner. `verify_artifact_refinements` matches the
//! scheduled regions against exactly three fixed strategy shapes — `[single]`,
//! `[_, _]`, and `[_, _, _]` — which are the fused, materialized two-stage, and
//! split-reduction forms of a **single-output** pipeline, and it then reads the
//! program's one output with `semantic.outputs().next()`. Nothing upstream of it
//! produces a cover that assigns regions to several ordered outputs.
//!
//! That widening is already owned, and not by this file's ticket:
//! `implement-general-dag-partitioning` closing condition 2 is "named and
//! multi-result outputs are planned as ordered graph outputs, not reduced to a
//! single root, and a plan naming fewer outputs than the program declares is
//! rejected rather than accepted as a subset" — this obligation exactly. Until a
//! cover can name two outputs, relaxing `output-arity` could only admit a program
//! the planner cannot cover, failing mid-pipeline instead of refusing at the
//! boundary, which is strictly worse than refusing.
//!
//! # The one ordering fact that is already true, and the one that is not
//!
//! Output *order* is identity at the semantic layer and this file proves it:
//! `tiler-ir`'s graph encoding writes the output list in declaration order and
//! seeds its canonical value numbering from it, so two programs differing only in
//! the order of two `output()` calls have distinct graph identities.
//!
//! It is **not** identity at the artifact layer. `encode_identity` sorts the
//! encoded output records by content before folding them
//! (`crates/tiler-ir/src/program/model.rs:1788`), so two `KernelProgram`s
//! differing only in `push_output` order — same keys, same values, hence the same
//! sorted list — carry the same canonical identity while `outputs()` still yields
//! them in different orders. `verify_outputs` checks output coverage as a *set*
//! and never pins the published order to the semantic interface order, so nothing
//! else recovers it. That is latent rather than live only because
//! `program.core.outputs().len() != 1` currently refuses every program that could
//! exhibit it, and it is filed as
//! `carry-artifact-program-output-order-into-kernel-program-identity`.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::program::ValueRole;
use tiler_ir::schedule::{InputOrdinal, TensorRole};
use tiler_ir::semantic::{
    F32, F32Add, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled for the reason the sibling
/// multi-input file states it: the outcome here is structural, so a contract that
/// behaved differently would mean the boundary moved for a reason this file does
/// not model.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// Two ordered outputs over two inputs: `product = a * b`, `sum = a + b`.
///
/// The two outputs are *independent* — neither reads the other — which is
/// deliberately the easiest multi-output program that exists. A wall that stops
/// this one is not a wall about sharing, materialization, or lifetime; it is a
/// wall about output cardinality alone.
fn two_output_region() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, a, b).unwrap();
    let sum = F32Add::apply(&mut builder, a, b).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// The control: `out = a * b`, the same two inputs and one of the same two roots.
///
/// Without a program that compiles under the identical request, "refuses" above
/// would be consistent with a broken target profile or an unusable session
/// boundary, and this file would prove nothing about output cardinality.
fn one_output_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, a, b).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder.build().unwrap()
}

/// Two output keys publishing one semantic value: `product` and `alias`.
///
/// Distinct from [`two_output_region`] in that the two outputs *collide* on one
/// value rather than naming two. It is here because a widening that counted
/// distinct produced values rather than declared outputs would admit this one
/// while still refusing the other, and the interface it must produce has two
/// ordered entries either way.
fn colliding_output_region() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, a, b).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("alias").unwrap(), product)
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

/// An ordered two-output program refuses under `output-arity`, at every contract.
///
/// The one-output control travels with it and must compile under the identical
/// request, which is what makes the refusal evidence about output cardinality
/// rather than about the profile, the session boundary, or the shared `a * b`
/// body the two programs have in common.
#[test]
fn an_ordered_two_output_program_refuses_under_output_arity() {
    let region = two_output_region();
    assert_eq!(region.input_count(), 2);
    assert_eq!(region.output_count(), 2);
    let control = one_output_control();
    assert_eq!(control.output_count(), 1);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the one-output control, so nothing this test \
             asserts about the two-output region would be evidence about output \
             cardinality",
        );
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "output-arity"
            }),
            "{contract:?} admitted a program the planner cannot cover",
        );
    }
}

/// Two output keys colliding on one value refuse under the same rule.
///
/// The refusal reads the *declared output count*, not the number of distinct
/// produced values — so a widening cannot discharge this case by observing that
/// one region already computes everything the program publishes.
#[test]
fn two_output_keys_publishing_one_value_refuse_under_output_arity() {
    let region = colliding_output_region();
    assert_eq!(region.output_count(), 2);
    // One produced value, published twice: the operation count is the control's.
    assert_eq!(
        region.operation_count(),
        one_output_control().operation_count()
    );

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "output-arity"
            }),
        );
    }
}

/// Output order is identity at the semantic layer.
///
/// Two programs holding the same inputs, the same operations, and the same two
/// output keys bound to the same two values — differing *only* in which
/// `output()` call came first — have distinct graph identities. `tiler-ir`'s
/// graph encoding writes the output list in declaration order and seeds its
/// canonical value numbering by visiting outputs in that order, so the ordering
/// reaches identity twice over.
///
/// This is the half of the ticket's ordering obligation that is already
/// discharged, and pinning it is what makes the *other* half — the artifact
/// layer's sorted output encoding — a located gap rather than a suspicion.
#[test]
fn two_programs_differing_only_in_output_order_have_distinct_identities() {
    fn ordered(product_first: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let b = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, a, b).unwrap();
        let sum = F32Add::apply(&mut builder, a, b).unwrap();
        let product_key = OutputKey::new("product").unwrap();
        let sum_key = OutputKey::new("sum").unwrap();
        if product_first {
            builder.output(product_key, product).unwrap();
            builder.output(sum_key, sum).unwrap();
        } else {
            builder.output(sum_key, sum).unwrap();
            builder.output(product_key, product).unwrap();
        }
        builder.build().unwrap()
    }

    let product_first = ordered(true);
    let sum_first = ordered(false);
    // Same interface content, in the two possible orders.
    assert_eq!(product_first.output_count(), sum_first.output_count());
    assert_ne!(
        product_first.semantic_identity().graph(),
        sum_first.semantic_identity().graph(),
        "output order must be identity, not presentation",
    );
    // The check can say no: re-declaring the same order reproduces the identity,
    // so the inequality above is about the order and not about rebuilding.
    assert_eq!(
        product_first.semantic_identity().graph(),
        ordered(true).semantic_identity().graph(),
    );
}

/// A value published as a program output cannot also feed a later stage.
///
/// `ValueRole` is exclusive — a materialized value is `Temporary` *or* `Output` —
/// and `fills` refuses an `Output` value for any buffer that is not the region's
/// own `TensorRole::Output`. `KernelProgramBuilder`'s stage-access check is where
/// that bites.
///
/// The consequence is a real cost the vocabulary imposes rather than a wall it
/// raises: a program publishing an intermediate *and* consuming it needs a copy
/// stage reading `TensorRole::Intermediate` and writing `TensorRole::Output`.
/// That is the shape `pipeline::conformance`'s multi-output fixture has — it
/// publishes `scaled` and reduces it into `reduced` — and it is also the shape
/// `admit-elementwise-epilogues-over-a-materialized-intermediate` owns, because
/// no elementwise region this profile builds reads a materialized intermediate.
///
/// Pinned here so that a `ValueRole` widening which made publication and
/// consumption compatible fails this test and reports itself, rather than
/// silently changing what the multi-output work has to plan for.
#[test]
fn a_published_output_value_cannot_fill_an_intermediate_buffer() {
    assert!(ValueRole::Output.fills(TensorRole::Output));
    assert!(!ValueRole::Output.fills(TensorRole::Intermediate));
    assert!(!ValueRole::Output.fills(TensorRole::Input {
        ordinal: InputOrdinal::new(0)
    }));
    // The neighbour that does compose, so the refusals above are about the
    // published role rather than about `fills` declining everything.
    assert!(ValueRole::Temporary.fills(TensorRole::Intermediate));
    assert!(ValueRole::Input.fills(TensorRole::Input {
        ordinal: InputOrdinal::new(0)
    }));
}
