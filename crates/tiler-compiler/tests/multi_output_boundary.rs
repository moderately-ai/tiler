//! Ordered multi-output programs compile, and what still refuses is the sharing.
//!
//! This file used to record where the multi-output *wall* was, and it moved
//! three times before it fell: not `tiler-ir`, then the planner, then the
//! request-boundary recognition. The guards it described —
//! `select_supported_strategy`'s `output_count() != 1` under `output-arity` and
//! `verify_artifact_refinements`'s arity check under `semantic-output-coverage`
//! — are both gone, relaxed together, and a program declaring several ordered
//! named outputs is now recognized, covered, planned, and assembled like any
//! other. `pipeline::conformance`'s
//! `ordered_multi_output_programs_compile_through_the_ordinary_path` is the
//! end-to-end evidence; this file holds the boundary the caller sees.
//!
//! # What each layer contributed, so the removal is attributable
//!
//! `tiler-ir` was never the wall, and its own tests prove that rather than this
//! file asserting it: `KernelProgramBuilder::push_output` is general and bounded
//! by `MAX_PROGRAM_OUTPUTS` (4096) rather than by one,
//! `program::tests::storage_reuse_is_admitted_only_with_an_explicit_handoff`
//! builds and *verifies* a two-output program, `a_missing_named_output_is_rejected`
//! already refuses a plan naming fewer outputs than the program declares, and
//! `KernelProgramBuildError::DuplicateOutput` refuses two publications of one
//! key. `TensorRole::Output` carrying no ordinal was never the obstruction
//! either: a region writes one owning tensor, several regions write several, and
//! the program layer binds each stage's buffers to values positionally, which
//! `tiler_ir::program::ValueRole::fills` states outright and this file pins.
//!
//! Four widenings then landed under the compiler. `implement-general-dag-partitioning`
//! made covers carry the ordered named outputs with `verify_cover` checking each
//! is produced by exactly one region; `assemble-a-kernel-program-from-an-arbitrary-cover`
//! replaced three fixed single-output plan shapes with a derivation over a cover
//! of any region count; `carry-artifact-program-output-order-into-kernel-program-identity`
//! put output order into kernel-program identity; and
//! `recognize-several-ordered-named-outputs-at-the-compiler-request-boundary`
//! replaced the single walk from `outputs().next()` with one walk per declared
//! output, moving the whole-program obligation onto `check_output_cover`.
//!
//! **The last piece was not a guard but a derivation.** With the guards relaxed
//! and nothing else changed, an independent two-output program reached
//! `phase: "program-assembly", rule: "cover-named-output-attribution"`: the cover
//! stated which regions publish *an* output but not which named result each
//! retained, so `CoverAssembly::from_plan` paired the declared outputs with the
//! publishing regions by execution order — a guess that is invisible with one
//! output and wrong whenever the caller's declaration order disagrees with the
//! cover's canonical region order. `CoverRegion::named_results` supplies the
//! missing fact and the pairing is now by value.
//!
//! # What still refuses, and it is about sharing rather than arity
//!
//! Two declared outputs whose recognition walks share an occurrence refuse at
//! the request boundary under `output-partition-overlap`. That is the branch
//! where one region's owning write would have to serve both a materialization
//! edge and a publication: two output keys naming one value, and a published
//! intermediate that is also consumed. `ValueRole` is exclusive and a region
//! writes one owning tensor, so both are refused a layer down —
//! [`a_published_output_value_cannot_fill_an_intermediate_buffer`] pins the
//! mechanism — and `admit-elementwise-epilogues-over-a-materialized-intermediate`
//! owns the copy stage that would lift the second. That copy stage was itself
//! blocked in `tiler-ir` rather than here until
//! `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
//! landed; `materialized_intermediate_epilogue_wall.rs` now measures a pointwise
//! region reading `TensorRole::Intermediate` being *admitted* by the intrinsic
//! schedule verifier, so what is left is building the stage from a recognized
//! program.
//!
//! One further limit is *not* about outputs at all and is recorded here because
//! multi-output is what makes it reachable: an elementwise walk must read every
//! declared input (`elementwise-reads`), so two outputs each reading a different
//! subset of the program's inputs refuse even though neither shares an
//! occurrence with the other. `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`
//! owns it, and [`disjoint_input_two_output_program`] is the fixture.
//!
//! # Output order is identity at both layers, and this file pins the semantic half
//!
//! Output *order* is identity at the semantic layer and this file proves it:
//! `tiler-ir`'s graph encoding writes the output list in declaration order and
//! seeds its canonical value numbering from it, so two programs differing only in
//! the order of two `output()` calls have distinct graph identities.
//!
//! The artifact layer no longer discards it.
//! `carry-artifact-program-output-order-into-kernel-program-identity` closed that
//! gap: `verify_outputs` pins the published records to the semantic subject's
//! ordered interface — keys in the subject's order, each key's component records
//! contiguous and in the encoded contract's declared component order, anything
//! else refused as `misordered-named-output` — and `encode_identity` folds the
//! list in that order rather than sorting the records by content. So the ordered
//! interface a consumer reads from `VerifiedKernelProgram::outputs` is a fact
//! rather than something it re-derives by key, and a permuted publication is not
//! a second program to distinguish but a program that does not verify.

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
/// deliberately the easiest multi-output program that exists, and both walks
/// read both declared inputs — so it differs from [`one_output_control`] by
/// exactly the second declared output and the occurrence producing it. It is
/// the program the arity guard refused and the program that compiles now.
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
/// It travels with every assertion below so that an outcome — compiling or
/// refusing — is evidence about the program's *outputs* rather than about the
/// target profile, the session boundary, or the shared `a * b` body.
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
/// value rather than naming two, which is the smallest program where one owning
/// write would have to publish twice. It is here because the admission is about
/// what the outputs share, not how many there are: this one still refuses while
/// its two-output neighbour compiles.
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

/// Two independent ordered outputs whose walks read *different* declared inputs.
///
/// `doubled = a + a` and `squared = b * b`: the same shape as
/// [`two_output_region`] and the same independence, differing only in that
/// neither walk reads the input the other does.
fn disjoint_input_two_output_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let doubled = F32Add::apply(&mut builder, a, a).unwrap();
    let squared = F32Multiply::apply(&mut builder, b, b).unwrap();
    builder
        .output(OutputKey::new("doubled").unwrap(), doubled)
        .unwrap();
    builder
        .output(OutputKey::new("squared").unwrap(), squared)
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

/// An ordered two-output program compiles, at every contract.
///
/// **This assertion was the inverse of itself until the arity guards were
/// relaxed**, and it is the same program either way: two independent ordered
/// named outputs over the same two declared inputs. The one-output control
/// travels with it and must compile under the identical request, so a green run
/// is evidence that output cardinality no longer decides admission rather than
/// evidence that the profile happens to accept everything.
///
/// Every contract is stated rather than sampled, for the reason the sibling
/// multi-input file gives: the outcome is structural, so a contract behaving
/// differently would mean the boundary moved for a reason this file does not
/// model.
#[test]
fn an_ordered_two_output_program_compiles() {
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
            Ok(()),
            "{contract:?} refused a program whose only difference from the \
             control is a second independent ordered named output",
        );
    }
}

/// Two output keys colliding on one value refuse, and it is about the sharing.
///
/// Its accepted neighbour is [`two_output_region`], which declares the same
/// number of outputs and compiles. What differs is that both keys here name one
/// produced value, so whichever region owns that write would have to publish it
/// twice — refused at the request boundary under `output-partition-overlap`
/// rather than admitted and dropped when
/// `tiler_ir::program::KernelProgramBuilder` refuses the second publication.
#[test]
fn two_output_keys_publishing_one_value_refuse_for_their_shared_write() {
    let region = colliding_output_region();
    assert_eq!(region.output_count(), 2);
    // One produced value, published twice: the operation count is the control's.
    assert_eq!(
        region.operation_count(),
        one_output_control().operation_count()
    );
    let accepted = two_output_region();
    assert_eq!(accepted.output_count(), region.output_count());

    for contract in CONTRACTS {
        assert_eq!(compile_under(&accepted, contract), Ok(()));
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "output-partition-overlap"
            }),
        );
    }
}

/// Two independent outputs reading *different* declared inputs still refuse.
///
/// Recorded here because multi-output admission is what makes this reachable and
/// because it is the one refusal above that is not about outputs at all. Each
/// elementwise walk must read every declared input (`elementwise-reads`), since
/// the region it builds binds one buffer per declared program input and the
/// expression's dense-ordinal rule refuses a gap. With one declared output the
/// state cannot arise — a frozen program drops an input no output reaches — so
/// nothing observed it until two outputs could split the input set between them.
///
/// Its accepted neighbour is [`two_output_region`]: the same two inputs, the
/// same two families, the same two independent outputs, differing only in that
/// each walk there reads both inputs.
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` owns
/// the widening.
#[test]
fn two_outputs_reading_disjoint_declared_inputs_refuse_for_their_unread_buffers() {
    let region = disjoint_input_two_output_program();
    assert_eq!(region.input_count(), 2);
    assert_eq!(region.output_count(), 2);
    let accepted = two_output_region();

    for contract in CONTRACTS {
        assert_eq!(compile_under(&accepted, contract), Ok(()));
        assert_eq!(
            compile_under(&region, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "elementwise-reads"
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
/// This is the half of the ordering obligation this file pins; the artifact
/// layer's half is discharged in `tiler-ir`, whose
/// `published_output_interface_order_reaches_program_identity` and
/// `publishing_the_outputs_out_of_interface_order_is_rejected` own it.
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
/// no elementwise region this profile *builds* reads a materialized
/// intermediate. That is now a compiler gap and nothing more: the `tiler-ir`
/// access contract that also forbade it was widened by
/// `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`,
/// which `materialized_intermediate_epilogue_wall.rs` measures.
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
