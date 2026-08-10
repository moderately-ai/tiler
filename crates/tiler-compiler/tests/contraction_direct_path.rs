//! Where the two-input contraction boundary was, and what moved it.
//!
//! A binary tensor contraction could not reach the compiler at all: both
//! installed recognizers demanded exactly one input over a fixed producer chain,
//! so `request.rs`'s `input_count() != 1` refused the pinned workload's
//! projection at the request boundary before any explain trace existed. What
//! landed is the `direct` realization of the L3 elimination — one invocation per
//! output element, folding its own contracted sequence in ascending order from
//! the first product — carried through recognition, an eighth governed
//! index-access capability, a scheduled region, structured-kernel verification,
//! and single-region program assembly.
//!
//! # What this file is evidence for, and what it is not
//!
//! It is evidence that a contraction **compiles through the ordinary entry
//! point** and reaches a complete verified plan, that an unrecognized shape
//! still refuses with a typed reason, and that the realization has exactly one
//! precondition. It is not evidence about the *numbers*: bit-level agreement
//! with the reference evaluator and with the L3 probe's retained device
//! measurements is `governed::contraction_conformance`, which needs the
//! crate-private lowering registry and therefore lives in-crate.
//!
//! # The one precondition, and the checks that are deliberately absent
//!
//! `direct`'s precondition is `K >= 1` and nothing else. The L3 record's
//! realization table states it as "none beyond `K ≥ 1`", and that is a
//! deliverable rather than an omission: the `tiled` realization refuses `K` not
//! a multiple of sixteen and the split realizations refuse `K` not a multiple of
//! their split width, but `direct` has no tile and no split, so a K-multiple
//! refusal on this path would be a check that can never fire — shipped as if it
//! could. [`no_k_multiple_refusal_exists_on_the_direct_path`] is the assertion
//! that it does not exist, run over every contracted extent a multiple check
//! would reject.
//!
//! The empty contracted domain is not a *second* precondition. The registered
//! family declares `refused-an-unseeded-fold-has-no-empty-result`, and the
//! semantic inferencer refuses a zero contracted extent at construction, so a
//! program with one cannot be built — which is why the case below asserts the
//! refusal at `build()` rather than at `compile()`.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32Constant, F32Multiply,
    F32TensorContraction, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
    StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled. A contraction consumes no numerical
/// permission — it is the declared contributor sequence itself — so the outcome
/// must be identical under all five, and a contract that behaved differently
/// would mean the realization is consuming something it does not declare.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// The profile's index structure, `td,od->to`, spelled with arbitrary frontend
/// labels so the renaming-invariant canonicalization is exercised rather than
/// assumed.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap()
}

/// `activations[m, k] x weights[n, k] -> projected[m, n]`.
///
/// The extents stay inside the governed baseline profile's four-thread launch
/// bound, which is the same bound every fixture in
/// `multi_input_elementwise_boundary.rs` respects. The workload's own extents
/// are refused by that bound rather than by anything this file is about, and
/// that refusal is asserted separately, in `pipeline::tests`.
fn projection(m: u64, n: u64, k: u64) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([m, k]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([n, k]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    builder.build().unwrap()
}

/// The control: the recognized one-input pointwise shape, `(a * 2.0) * 3.0`.
///
/// Without a program that compiles under the identical request, an assertion
/// about the contraction would be consistent with a broken target profile
/// rather than with anything about input cardinality or the contraction family.
/// It is the same fixture `multi_input_elementwise_boundary.rs` uses for the
/// same job, and it compiles under all five contracts because its two
/// operations share a family.
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

/// A contraction compiles end to end under every stated contract.
///
/// `compile_under` returns only after the per-target outcome resolves, so each
/// pass is a complete verified plan and not merely successful strategy
/// selection.
#[test]
fn a_contraction_compiles_through_the_ordinary_entry_point() {
    let program = projection(2, 2, 3);
    assert_eq!(program.input_count(), 2);
    assert_eq!(program.operation_count(), 1);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&one_input_control(), contract),
            Ok(()),
            "{contract:?} refused the one-input control, so nothing this file \
             asserts about the contraction would be evidence about it",
        );
        assert_eq!(
            compile_under(&program, contract),
            Ok(()),
            "{contract:?} refused a contraction of the profile's own structure",
        );
    }
}

/// `direct`'s precondition is `K >= 1` and nothing else.
///
/// Every contracted extent below is one a tile or split width would reject —
/// three is not a multiple of sixteen, of eight, or of two; one is not a
/// multiple of anything but itself; and five and seven are prime. Each compiles.
///
/// This is the deliverable's "assert that no K-multiple refusal exists on this
/// path": the L3 record states `direct`'s preconditions as "none beyond `K ≥
/// 1`", so shipping a K refusal here would be a check that could never fire,
/// reported green because it was never reached.
#[test]
fn no_k_multiple_refusal_exists_on_the_direct_path() {
    for k in [1_u64, 2, 3, 5, 7] {
        assert_eq!(
            compile_under(&projection(2, 2, k), NumericalContract::STRICT_F32),
            Ok(()),
            "a contracted extent of {k} was refused, so a width precondition \
             this realization does not have has been introduced",
        );
    }
    // And the odd free extents too, so the absence above is about the contracted
    // axis rather than about every extent happening to be even.
    assert_eq!(
        compile_under(&projection(1, 3, 3), NumericalContract::STRICT_F32),
        Ok(())
    );
}

/// An empty contracted domain is refused at construction, not at compilation.
///
/// The registered family declares `refused-an-unseeded-fold-has-no-empty-result`
/// — there is no value an unseeded fold could commit for zero contributors — so
/// the refusal belongs to the semantic inferencer and fires before a request
/// exists. Asserting it here keeps "K >= 1" a checked property of the whole path
/// rather than a claim about the recognizer alone.
#[test]
fn an_empty_contracted_domain_is_refused_before_a_request_exists() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([2, 0]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([2, 0]))
        .unwrap();
    let error =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .expect_err("an unseeded fold has no empty result");
    assert!(
        format!("{error}").contains("empty-contracted-domain"),
        "the refusal must name the empty contracted domain: {error}"
    );
}

/// A contraction whose operands are one tensor read twice is not recognized.
///
/// `projection_structure()` is equivalent to `mk,nk->mn`, and this fixture
/// supplies the same declared `ValueId` for both logical operands. The binary
/// family still requires two distinct declared-input reads, so the refusal is
/// the operand-binding rule rather than the width of the declared interface.
#[test]
fn a_contraction_over_one_declared_input_refuses_with_a_typed_reason() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let square = builder
        .input::<F32>(InputKey::new("square").unwrap(), Shape::from_dims([3, 3]))
        .unwrap();
    let contracted =
        F32TensorContraction::apply(&mut builder, &projection_structure(), square, square).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), contracted)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(program.input_count(), 1);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&program, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "contraction-operands"
            }),
            "{contract:?} admitted a contraction the recognizer does not cover",
        );
    }
}

/// A binary contraction beside an independently retained third declaration.
///
/// The second output is load-bearing: it keeps `side` in the semantic program
/// without making it a contraction operand, so this subject isolates the
/// registered schema's `OperationArity::exact(2)` and the compiler's fixed
/// two-read form. ADR 0087 leaves multi-operand semantic contraction reserved;
/// this fixture does not decide it. The contraction maps its two operands to
/// their program declaration ordinals without reading or discarding the
/// independently retained input.
#[test]
fn a_contraction_beside_a_retained_third_input_compiles() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([2, 3]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let side = builder
        .input::<F32>(InputKey::new("side").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .unwrap();
    let retained = F32Multiply::apply(&mut builder, side, side).unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    builder
        .output(OutputKey::new("retained").unwrap(), retained)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(program.input_count(), 3);
    assert_eq!(program.output_count(), 2);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&program, contract),
            Ok(()),
            "{contract:?} must admit the binary contraction and its independent sibling",
        );
    }
}

/// A contraction with an elementwise epilogue compiles as a chain.
///
/// **This assertion measured a refusal until
/// `admit-elementwise-epilogues-over-a-materialized-intermediate` landed**, and
/// the reason it did is what the name now has to stop claiming: the recognized
/// shape *was* exactly one operation, so squaring the result was "an operation
/// outside the recognized set" and was refused under `operation-set` rather than
/// dropped. The recognized shape is now a two-region chain — the contraction
/// stages its result and the epilogue reads it — so the extra occurrence is
/// covered by a region of its own rather than by nothing.
///
/// `projected * projected` is the narrow case worth keeping here rather than a
/// fresh one: both operands name the *same* staged value, so the epilogue mints
/// one read and not two. A second read would be inadmissible —
/// `TensorRole::Intermediate` carries no ordinal to attribute a second
/// materialization edge to — and the program that would need one is the
/// neighbour below.
#[test]
fn a_contraction_with_an_elementwise_epilogue_compiles_as_a_chain() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([2, 3]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .unwrap();
    let squared = F32Multiply::apply(&mut builder, projected, projected).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), squared)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(program.operation_count(), 2);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&program, contract),
            Ok(()),
            "{contract:?} refused a contraction feeding an elementwise epilogue",
        );
    }

    // The neighbour that still refuses, and it refuses for the reason the
    // epilogue admission is bounded by rather than for the reason the name above
    // used to state: a fold whose own contributors are another fold's result
    // needs two staged reads in one region, and nothing attributes the second to
    // a materialization edge.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([2, 3]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, projected, [Axis::new(1)]).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, folded, two).unwrap();
    let refolded = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), refolded)
        .unwrap();
    let nested = builder.build().unwrap();
    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&nested, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "operation-set"
            }),
            "{contract:?} admitted a chain two materialization boundaries deep",
        );
    }
}

/// A general binary structure compiles, not only the profile's matmul spelling.
///
/// `abc,b->ac` binds its contracted index to axis 1 of the first operand and
/// axis 0 of the second, which is the case a realization keyed on "the last axis
/// of both operands" would get wrong while `td,od->to` still passed. It is also
/// the case that decides the schedule's own vocabulary: a reduction topology
/// naming reduced *axes* cannot express it, which is why the contraction carries
/// its contracted iteration shape instead.
#[test]
fn a_structure_whose_contracted_index_sits_at_different_axes_compiles() {
    let structure = ContractionIndexStructure::new(
        [
            vec![
                ContractionIndex::new(0),
                ContractionIndex::new(1),
                ContractionIndex::new(2),
            ],
            vec![ContractionIndex::new(1)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let left = builder
        .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2, 3, 2]))
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([3]))
        .unwrap();
    let contracted = F32TensorContraction::apply(&mut builder, &structure, left, right).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), contracted)
        .unwrap();
    let program = builder.build().unwrap();

    assert_eq!(
        compile_under(&program, NumericalContract::STRICT_F32),
        Ok(()),
        "a general binary structure is refused, so the widening is narrower than \
         the representation admits",
    );
}
