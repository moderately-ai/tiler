//! Where the whole-program recognition boundary was, and what moved it.
//!
//! `select_supported_strategy` was a three-way template match. It tried a
//! scale-bias-then-strict-serial `Sum` over exactly one declared input, a
//! well-formed binary contraction, and a two-operation pointwise expression over
//! three leaves — and when all three refused, the program was refused before any
//! target-qualified explain trace existed. A program *composing* two of those
//! families was not a shape any of them spelled, so it was refused for a
//! property of the templates rather than for anything the compiler could not do.
//!
//! What landed is a recognizer that classifies the occurrence producing the
//! output and then walks outward through the occurrences feeding it. The
//! elementwise dimension is now the general `PointwiseF32Expression` vocabulary:
//! any depth, any number of declared inputs, mixed families, and shared reads.
//! This file is the evidence that a composed program reaches an emitted region
//! through the ordinary `tiler_compiler::session` entry point, and that the
//! programs still outside the boundary refuse under a rule that names what was
//! not recognized.
//!
//! # What is asserted, and what would make each assertion vacuous
//!
//! Every compilation here resolves its per-target outcome, so a pass is a
//! complete verified plan rather than successful strategy selection. Every
//! positive case travels with a one-input control the superseded template *did*
//! spell: without a program that compiles under the identical request, a pass
//! would be consistent with a permissive target rather than with anything about
//! recognition. Every refusal travels with the accepted
//! neighbour it differs from in exactly one occurrence, so the rule that fires is
//! attributable to that occurrence and not to the fixture.
//!
//! # The one contract that declines both mixed bodies
//!
//! `RelaxedF32` permits arithmetic contraction, and a body holding a multiply
//! adjacent to an add has no feasible plan under it — a decline the sibling
//! `multi_input_elementwise_boundary` file owns, pins, and records as eliminated
//! rather than deferred. The composed program and the scale-bias control are
//! *both* such bodies, so both are asserted to refuse under it and for the same
//! class. Asserting them together is the point: the decline reads the adjacency
//! and not the composition.
//!
//! # What is deliberately not asserted here
//!
//! Bit-level agreement with the reference evaluator is the in-crate conformance
//! work's, because it needs the crate-private lowering registry. Which *plan*
//! wins is `selection`'s. What this file does record about plans is a
//! vocabulary fact rather than a preference: a general prologue has no fused
//! single-region alternative at all, because
//! `ScalarProgram::FusedMultiplyAddSerialSum` applies one scale and one bias per
//! contributor and cannot spell `(a * b) + c`.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, F32Silu, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled, for the reason the sibling boundary
/// files state it: the outcome of *recognition* is structural, so a contract
/// that changed it would mean the boundary moved for a reason this file does not
/// model.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// The one contract that permits arithmetic contraction.
///
/// A mixed multiply/add body is declined under it, which the sibling
/// `multi_input_elementwise_boundary` file pins and owns. Named here so the one
/// case below that skips it does so for that stated reason rather than silently.
const CONTRACTION_PERMITTED: NumericalContract = NumericalContract::RELAXED_F32;

/// The contributor domain of every fixture below.
///
/// Four elements in one row, and every extent is load-bearing rather than
/// arbitrary. The governed baseline profile declares a four-thread grid axis, so
/// the prologue's launch — one invocation per contributor — must be at most
/// four; and the fold's four contributors are what admit a balanced exact split,
/// which keeps every parallel strategy *offered* rather than declined. A
/// narrower domain would make each positive case below pass for a reason this
/// file does not model.
fn domain() -> Shape {
    Shape::from_dims([1, 4])
}

/// `sum((a * b) + c, axis 1)`: two admitted families in one program.
///
/// **This is the composed program.** Its prologue is the multi-input elementwise
/// body `admit-multi-input-elementwise-programs-at-the-compiler-boundary`
/// landed, and its fold is the strict serial reduction the serial-sum family
/// owns. No superseded normalization matched it: the serial-sum template
/// demanded exactly one declared input and the exact `x * scale + bias`
/// prologue, and the pointwise template refused any program containing a
/// reduction.
fn composed_region() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), domain())
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// `sum(x * 2.0 + 1.0, axis 1)`: the one shape the superseded template spelled.
///
/// It is the exact scale-bias prologue over one declared input that
/// `normalize_serial_sum` matched — the *only* program shape the superseded
/// template spelled — so it is the program every generalization below is
/// measured against.
///
/// It carries a multiply adjacent to an add, which the contraction-permitting
/// contract declines; the sibling `multi_input_elementwise_boundary` file pins
/// that decline for one-input mixed bodies, and [`homogeneous_control`] is the
/// neighbour that travels under that contract instead.
fn scale_bias_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let shifted = F32Add::apply(&mut builder, scaled, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, shifted, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// `sum((x * 2.0) * 3.0, axis 1)`: the neighbour every contract admits.
///
/// One declared input and a *homogeneous* prologue, so it carries no
/// multiply/add adjacency for the contraction-permitting contract to decline —
/// which is what [`scale_bias_control`] does carry. It is the control that
/// travels with every case below, because a neighbour that itself refuses under
/// one contract would leave that contract's assertions standing alone.
fn homogeneous_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let three = F32Constant::apply(&mut builder, 3.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, two).unwrap();
    let root = F32Multiply::apply(&mut builder, scaled, three).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, root, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// `sum(a, axis 1)`: a fold whose contributor tensor is a declared input.
///
/// The simplest reduction there is, and the one program in this file where the
/// wall is genuinely *below* the request boundary rather than in it: `tiler-ir`'s
/// schedule verifier requires a `ScalarProgram::StrictSerialSum` region's
/// contributor access to read `TensorRole::Intermediate`, so no region this
/// profile can build reads the input directly. Synthesizing an identity prologue
/// to satisfy it is not the alternative — that would add a materialization, and
/// its observable rounding boundary, that the caller's program never asked for.
fn prologue_less_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// [`composed_region`] with one occurrence's operation set perturbed.
///
/// The multiply becomes `tiler::silu-f32@1`, and nothing else changes: same
/// declared inputs at the same domain, same add, same fold, same output. The
/// activation is registered semantics *and* a registered index-access lowering
/// capability, so what refuses it is the region vocabulary — no
/// `PointwiseF32Node` spells a sigmoid-weighted linear unit — rather than an
/// unknown operation.
fn composed_region_with_an_unspellable_occurrence() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let c = builder
        .input::<F32>(InputKey::new("c").unwrap(), domain())
        .unwrap();
    let activated = F32Silu::apply(&mut builder, a).unwrap();
    let biased = F32Add::apply(&mut builder, activated, c).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// Compiles one program under one contract against the governed profile.
///
/// Returns only after the per-target outcome resolves, so `Ok` is a complete
/// verified plan. A strategy-admission refusal is additionally required to
/// precede any target-qualified trace, which is what makes "refused at the
/// boundary" distinguishable from "refused after planning".
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

/// The composed program compiles through the ordinary entry point.
///
/// This is the ticket's closing condition: a semantic program no normalization
/// matched — one composing the multi-input elementwise family with the
/// serial-sum family — reaches an emitted region rather than being refused at
/// the boundary.
#[test]
fn a_program_composing_two_admitted_families_compiles_through_the_session() {
    let composed = composed_region();
    assert_eq!(composed.input_count(), 3);
    assert_eq!(composed.operation_count(), 3);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&homogeneous_control(), contract),
            Ok(()),
            "{contract:?} refused the one-input control, so nothing this test \
             asserts about the composed program would be evidence about \
             recognition",
        );
        // Both mixed multiply/add bodies skip the contraction-permitting
        // contract, and they skip it *together*, which is what makes the skip a
        // statement about the adjacency rather than about the composition: the
        // shape the superseded template spelled and the shape it could not are
        // declined identically under it. The sibling
        // `multi_input_elementwise_boundary` file owns and pins that decline.
        if contract == CONTRACTION_PERMITTED {
            assert_eq!(
                compile_under(&scale_bias_control(), contract),
                Err(CompileFailureClass::NoFeasiblePlan),
            );
            assert_eq!(
                compile_under(&composed, contract),
                Err(CompileFailureClass::NoFeasiblePlan),
            );
            continue;
        }
        assert_eq!(
            compile_under(&scale_bias_control(), contract),
            Ok(()),
            "{contract:?} refused the one shape the superseded template spelled",
        );
        assert_eq!(
            compile_under(&composed, contract),
            Ok(()),
            "{contract:?} refused the composed program",
        );
    }
}

/// A fold over a declared input refuses under its own rule.
///
/// This is where the ticket's second boundary sits. Recognition generalized over
/// the expression vocabulary, and it did *not* generalize past what the physical
/// layer can express: a region that folds a program input directly is rejected by
/// `tiler-ir`'s schedule verifier as malformed compiler output, so admitting the
/// program here would produce exactly the mid-pipeline death the precedent
/// declined to ship. `admit-a-reduction-over-a-declared-input-tensor` owns the
/// widening, and depends on this.
///
/// Its accepted neighbour is [`homogeneous_control`]: the same fold over the
/// same declared input with an elementwise prologue between them, so what the
/// rule reads is the missing prologue and not the fold.
#[test]
fn a_reduction_over_a_declared_input_refuses_under_the_prologue_rule() {
    let fold = prologue_less_fold();
    assert_eq!(fold.input_count(), 1);
    assert_eq!(fold.operation_count(), 1);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&homogeneous_control(), contract),
            Ok(()),
            "{contract:?} refused the one-input neighbour, so the refusal below \
             would not be evidence about the missing prologue",
        );
        assert_eq!(
            compile_under(&fold, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "reduction-prologue"
            }),
            "{contract:?} admitted a fold no scheduled region reads",
        );
    }
}

/// Perturbing one occurrence's operation set fires the rule that names it.
///
/// The perturbed program differs from [`composed_region`] in exactly one
/// occurrence, and that occurrence is the only reason it refuses. The rule is
/// `operation-set` — the property that was not recognized — and the refusal
/// precedes any target-qualified trace, which `compile_under` asserts.
///
/// This is the boundary the ticket draws: recognition generalized over the
/// expression vocabulary, and admission did not become silent.
#[test]
fn perturbing_one_occurrence_out_of_the_vocabulary_refuses_by_name() {
    let perturbed = composed_region_with_an_unspellable_occurrence();
    assert_eq!(perturbed.operation_count(), 3);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&perturbed, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "operation-set"
            }),
            "{contract:?} admitted an occurrence no scalar program spells",
        );
    }
}

/// A second named output refuses under its own rule, not the operation set's.
///
/// The accepted neighbour is [`composed_region`] itself: the same three
/// occurrences over the same three inputs, with one further value named as an
/// output. Every region builder writes exactly one owning tensor, so multi-output
/// admission is `admit-ordered-multi-output-programs-at-the-compiler-request-boundary`'s
/// to land, and refusing it here is what keeps a program the physical layer
/// cannot assemble from reaching the pipeline.
#[test]
fn a_second_named_output_refuses_under_the_output_arity_rule() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), domain())
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder
        .output(OutputKey::new("biased").unwrap(), biased)
        .unwrap();
    let two_outputs = builder.build().unwrap();
    assert_eq!(two_outputs.output_count(), 2);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&two_outputs, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "output-arity"
            }),
            "{contract:?} admitted a second named output",
        );
    }
}
