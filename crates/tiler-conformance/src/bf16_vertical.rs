//! The pure-BF16 vertical, carried from a semantic program to a dispatched
//! device result and compared against the exact-rational oracle.
//!
//! # What one run crosses, and where it stops
//!
//! The oracle side builds `(x * 1.5) + 0.0` as a `tiler_ir` **semantic**
//! program over `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and
//! `tiler::add-bf16@1`, and evaluates it through `tiler-reference`'s registered
//! capabilities under a stated numerical conformance. The device side assembles
//! the same computation as a verified **scheduled region**, lowers it to a
//! structured kernel, emits `bfloat` MSL against the authoritative macOS Apple9
//! declaration, compiles that through the real Apple offline toolchain, and
//! dispatches the linked `metallib` on this host's GPU. The two sides share no
//! host expression: the oracle never sees the kernel and the kernel never sees
//! the semantic graph.
//!
//! ## The compiler is not in the path, and that is a measurement boundary
//!
//! **Fact, at this commit.** `tiler_compiler`'s `select_supported_strategy`
//! refuses every program carrying a non-`f32` value under the rule `dtype-f32`
//! before a subject is normalized, so `compile()` cannot produce a plan for a
//! BF16 semantic program at all —
//! `crates/tiler-compiler/tests/bf16_numerical_contract.rs`'s
//! `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall` asserts
//! exactly that, and
//! `crates/tiler-compiler/src/pipeline/tests.rs`'s BF16 vertical records the
//! same boundary in the same words. The consequence for this run is precise and
//! is not a shortcut taken here: **the optimizer, the artifact envelope, and the
//! runtime routing commit are not crossed**, because the only thing that could
//! produce the `PlanAlternative` all three consume is the call that refuses.
//! What is crossed is semantic construction, the oracle, the schedule and
//! kernel vocabularies, `bfloat` emission, offline compilation, and device
//! execution.
//!
//! Assembling the region through `tiler_ir`'s public builders is therefore the
//! only route to a BF16 kernel that exists, and it is the same route the
//! compiler's own BF16 vertical takes for the same stated reason. Nothing here
//! may be read as evidence that a caller can *ask* for this program.
//!
//! # Why the constants are `1.5` and `+0.0`
//!
//! Both are chosen so the composition is observable rather than convenient.
//!
//! **`1.5` rounds.** Multiplying by `1.5` is exact on an even significand, a
//! tie on an odd one, and an ordinary inexact rounding when the product crosses
//! a binade — so one multiplier reaches the tie-to-even rule, an ordinary
//! rounding, and the overflow boundary from the same corpus. A scale of `1.0`
//! or `2.0` would be exact everywhere and the rounding rule would never be
//! exercised; it would also be an identity the Metal compiler may delete.
//!
//! **`+0.0` cannot be deleted, and `-0.0` can.** `fadd y, -0.0` is the IEEE
//! identity on every operand and folds away without any fast-math licence,
//! which would leave the `add` leg of this vertical vacuous. `fadd y, +0.0` is
//! *not* the identity — `-0 + +0` is `+0` — so removing it needs `nsz`, which
//! the `safe` math mode this profile compiles under does not grant. Finding 27
//! of the [Apple numerical behaviour
//! record](../../../docs/research/apple-targets/numerical-behaviour.md)
//! measures exactly this shape at `bfloat`: `scale_one_bias_zero_bf16` retains
//! one `fadd` under `safe` at `-O2` and none under `relaxed` or `fast`, and
//! under `safe` it returns `0000` for the operand `8000`. That returned `0000`
//! is this run's **execution witness for the add**, on the non-subnormal
//! operand negative zero.
//!
//! **What the `+0.0` bias costs, stated rather than discovered.** It maps both
//! zeros to `+0` at the output, so this run cannot separate a sign-preserving
//! flush from an always-positive one: every flushed subnormal becomes a zero
//! whose sign the trailing add erases. That dimension is evidenced where it can
//! be — finding 24's measured `8040 -> 8000` row, and
//! `tiler-reference`'s own `the_flushed_zero_sign_is_read_on_both_dimensions`
//! — and is **not** claimed here.
//!
//! # The flush is applied to the oracle, and a bit-equal subnormal is a defect
//!
//! Finding 24 measures BF16 arithmetic on the macOS row flushing subnormal
//! operands and results to the sign-preserving zero. The device therefore
//! *cannot* agree with a subnormal-preserving oracle on a subnormal operand,
//! and a run that observed bit equality there would be observing something
//! other than the arithmetic that was measured. So the comparison is performed
//! against the reference evaluated under
//! [`declared_conformance`] — the flushing reading — and
//! [`flush_moved_indices`] names the elements the two readings disagree on, so
//! a corpus that silently stopped containing a subnormal would fail rather than
//! pass.
//!
//! **The conformance is stated at this call site rather than derived from a
//! route.** `ReferenceNumericalConformance::from_realization` discards the
//! format its realization was stated about and has no caller, so no capability
//! yet checks that the conformance it was handed speaks about its own format;
//! that window is owned by
//! `give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`.
//! Both the conformance and the region's realization are read off the same
//! [`NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16`] accessors, so there is
//! one source rather than two transcriptions, and
//! `the_oracle_and_the_region_are_told_one_contract` holds them to it.

use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::NumericalContract;
use tiler_ir::kernel::{VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::StorageScalar;
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExecutionBinding,
    InputOrdinal, KernelSchedule, LaunchPlan, LogicalAccess, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseBf16ExpressionBuilder, ReductionTopology,
    RegionId, ScalarProgram, ScheduledRegionBuilder, TailPolicy, TensorRole,
    VerifiedScheduledRegion,
};
use tiler_ir::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, CANONICAL_BF16_ARITHMETIC_NAN_BITS, InputKey,
    OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::record::MetalTranslationUnit;
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, InputBinding, ReferenceElement, ReferenceEvaluator,
    ReferenceNumericalConformance, Tensor, TensorPayloadView,
};

/// The scale constant, `1.5` in BF16.
pub(crate) const SCALE_BITS: u16 = 0x3fc0;
/// The bias constant, `+0.0` in BF16.
pub(crate) const BIAS_BITS: u16 = 0x0000;
/// Interface key of the vertical's one input.
const INPUT_KEY: &str = "operand";
/// Interface key of the vertical's one output.
const OUTPUT_KEY: &str = "result";

/// Named BF16 encodings the corpus is written in terms of.
///
/// Each is derived from the format's own fields rather than quoted: sign 1,
/// exponent 8 with bias 127, trailing significand 7, so precision 8, `emin`
/// -126, `emax` 127, and `bits = sign << 15 | biased_exponent << 7 | trailing`.
/// One subnormal quantum is `2^-133`, written `q` below.
mod bits {
    /// Positive zero.
    pub(super) const POS_ZERO: u16 = 0x0000;
    /// Negative zero.
    pub(super) const NEG_ZERO: u16 = 0x8000;
    /// The least positive subnormal, `1q`.
    pub(super) const MIN_SUBNORMAL: u16 = 0x0001;
    /// The least negative subnormal, `-1q`.
    pub(super) const NEG_MIN_SUBNORMAL: u16 = 0x8001;
    /// Half the least normal, `64q` — finding 24's measured input-flush operand.
    pub(super) const HALF_MIN_NORMAL: u16 = 0x0040;
    /// The negation of [`HALF_MIN_NORMAL`] — finding 24's measured sign row.
    pub(super) const NEG_HALF_MIN_NORMAL: u16 = 0x8040;
    /// The greatest subnormal, `127q`.
    pub(super) const MAX_SUBNORMAL: u16 = 0x007f;
    /// The least positive normal, `2^-126`, which is `128q`.
    pub(super) const MIN_NORMAL: u16 = 0x0080;
    /// `1.0`.
    pub(super) const ONE: u16 = 0x3f80;
    /// `1.5`, which is also the scale, so `1.0` maps onto it exactly.
    pub(super) const ONE_AND_A_HALF: u16 = 0x3fc0;
    /// `1 + 2^-7`, one quantum above one.
    pub(super) const ONE_PLUS_ULP: u16 = 0x3f81;
    /// `255 * 2^-7`, the greatest value below two.
    pub(super) const BELOW_TWO: u16 = 0x3fff;
    /// The greatest finite value, `255 * 2^120`.
    pub(super) const MAX_FINITE: u16 = 0x7f7f;
    /// Positive infinity.
    pub(super) const POS_INFINITY: u16 = 0x7f80;
    /// Negative infinity.
    pub(super) const NEG_INFINITY: u16 = 0xff80;
    /// A quiet NaN whose payload is not the family's canonical one.
    pub(super) const NONCANONICAL_NAN: u16 = 0x7fc1;
    /// The canonical quiet NaN this family's arithmetic installs.
    pub(super) const CANONICAL_NAN: u16 = 0x7fc0;
}

/// One corpus element: an operand and the two readings of `(x * 1.5) + 0.0`.
///
/// **Both expected encodings are derived by hand** from the format parameters
/// and the round-to-nearest-ties-to-even rule, and are stated here rather than
/// read back from any run — of the oracle, of the interpreter, or of the
/// device. A corpus obtained by recording what an implementation said agrees
/// with that implementation for reasons that say nothing about either being
/// right.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Case {
    /// What this element is evidence about.
    pub(crate) name: &'static str,
    /// The operand encoding placed in the input buffer.
    pub(crate) operand: u16,
    /// The result under a subnormal-*preserving* reading of both dimensions.
    pub(crate) preserved: u16,
    /// The result under the declared reading: both dimensions flushing to the
    /// sign-preserving zero, which is what the macOS Apple9 row delivers.
    pub(crate) declared: u16,
}

/// The corpus, covering every value class this vertical's arithmetic can reach.
///
/// Fifteen elements. Every derivation below is written out so a reader can check
/// it without running anything:
///
/// - `1.5` in units of the operand's own binade quantum multiplies the
///   significand by three and halves the exponent step, so an even significand
///   is exact, an odd one lands exactly halfway and meets the tie rule, and a
///   product crossing into the next binade meets a grid twice as coarse.
/// - The overflow boundary is the midpoint above the largest finite value,
///   `255.5 * 2^120`; `1.5 * 255 * 2^120` is `382.5 * 2^120` and is above it.
/// - The trailing `+ 0.0` is exact on every finite operand and changes only the
///   two zeros, where `-0 + +0` is `+0`.
pub(crate) const fn corpus() -> [Case; 15] {
    [
        Case {
            // +0 * 1.5 is +0, and +0 + +0 is +0.
            name: "positive zero",
            operand: bits::POS_ZERO,
            preserved: bits::POS_ZERO,
            declared: bits::POS_ZERO,
        },
        Case {
            // -0 * 1.5 is -0 by the sign exclusive-or, and -0 + +0 is +0 under
            // round-to-nearest. **The add's execution witness**: a compilation
            // that deleted the addition would return 0x8000 here, and a zero is
            // not subnormal, so nothing about the flush can explain the change.
            name: "negative zero, the add's execution witness",
            operand: bits::NEG_ZERO,
            preserved: bits::POS_ZERO,
            declared: bits::POS_ZERO,
        },
        Case {
            // 1q * 1.5 is 1.5q, a tie between 1q and 2q resolved to the even
            // significand 2. Flushed, the operand is +0 before the multiply.
            name: "least positive subnormal",
            operand: bits::MIN_SUBNORMAL,
            preserved: 0x0002,
            declared: bits::POS_ZERO,
        },
        Case {
            // The same from the other sign: -1.5q rounds to -2q.
            name: "least negative subnormal",
            operand: bits::NEG_MIN_SUBNORMAL,
            preserved: 0x8002,
            declared: bits::POS_ZERO,
        },
        Case {
            // 64q * 1.5 is 96q exactly, still subnormal. Finding 24's measured
            // `0040 -> 0000` input-flush row supplies the flushed reading.
            name: "half the least normal, finding 24's input-flush operand",
            operand: bits::HALF_MIN_NORMAL,
            preserved: 0x0060,
            declared: bits::POS_ZERO,
        },
        Case {
            // Finding 24's measured `8040 -> 8000` sign row. The flushed zero is
            // negative, and the trailing `+ 0.0` then makes it positive — which
            // is why this run does not claim the flushed sign.
            name: "the negation of the same, finding 24's sign row",
            operand: bits::NEG_HALF_MIN_NORMAL,
            preserved: 0x8060,
            declared: bits::POS_ZERO,
        },
        Case {
            // 127q * 1.5 is 190.5q, which is *normal* (>= 128q) and a tie on the
            // grid of that binade: 190 has the even trailing significand 62.
            name: "greatest subnormal, whose product is a normal tie",
            operand: bits::MAX_SUBNORMAL,
            preserved: 0x00be,
            declared: bits::POS_ZERO,
        },
        Case {
            // 128q * 1.5 is 192q, exact and normal. Neither dimension touches
            // it, which is what makes it the neighbour the moved elements are
            // separated from.
            name: "least normal, which the flush does not move",
            operand: bits::MIN_NORMAL,
            preserved: 0x00c0,
            declared: 0x00c0,
        },
        Case {
            // **The multiply's execution witness.** 1.0 * 1.5 is 1.5, exact; a
            // compilation that deleted the multiplication would return 0x3f80.
            name: "one, the multiply's execution witness",
            operand: bits::ONE,
            preserved: bits::ONE_AND_A_HALF,
            declared: bits::ONE_AND_A_HALF,
        },
        Case {
            // 129 * 1.5 is 193.5 quanta of the binade [1, 2); the tie resolves
            // to the even trailing significand 66, which is 194 quanta.
            name: "a tie resolved to the even significand",
            operand: bits::ONE_PLUS_ULP,
            preserved: 0x3fc2,
            declared: 0x3fc2,
        },
        Case {
            // 255 * 1.5 is 382.5 quanta of [1, 2), which is 191.25 quanta of
            // [2, 4) — a quarter of a quantum above 191, so nearness decides it
            // and the tie rule is not consulted.
            name: "an ordinary rounding, decided by nearness rather than the tie rule",
            operand: bits::BELOW_TWO,
            preserved: 0x403f,
            declared: 0x403f,
        },
        Case {
            // 382.5 * 2^120 is above the 255.5 * 2^120 overflow midpoint.
            name: "the greatest finite value, which overflows",
            operand: bits::MAX_FINITE,
            preserved: bits::POS_INFINITY,
            declared: bits::POS_INFINITY,
        },
        Case {
            name: "positive infinity",
            operand: bits::POS_INFINITY,
            preserved: bits::POS_INFINITY,
            declared: bits::POS_INFINITY,
        },
        Case {
            name: "negative infinity",
            operand: bits::NEG_INFINITY,
            preserved: bits::NEG_INFINITY,
            declared: bits::NEG_INFINITY,
        },
        Case {
            // An arithmetic NaN result carries the declared payload and never
            // the operand's, so a non-canonical payload does not survive.
            name: "a non-canonical NaN that canonicalizes",
            operand: bits::NONCANONICAL_NAN,
            preserved: bits::CANONICAL_NAN,
            declared: bits::CANONICAL_NAN,
        },
    ]
}

/// The operand encodings, in corpus order.
pub(crate) fn operands() -> Vec<u16> {
    corpus().iter().map(|case| case.operand).collect()
}

/// The expected encodings under the declared flushing realization.
pub(crate) fn declared_expectations() -> Vec<u16> {
    corpus().iter().map(|case| case.declared).collect()
}

/// The expected encodings under a subnormal-preserving reading.
pub(crate) fn preserved_expectations() -> Vec<u16> {
    corpus().iter().map(|case| case.preserved).collect()
}

/// The corpus positions the declared flush moves away from the preserved answer.
///
/// Named rather than counted, so a corpus whose subnormal group silently
/// emptied fails instead of passing: a run in which this is empty is a run whose
/// comparison could not have observed the flush at all.
pub(crate) fn flush_moved_indices() -> Vec<usize> {
    corpus()
        .iter()
        .enumerate()
        .filter(|(_, case)| case.preserved != case.declared)
        .map(|(index, _)| index)
        .collect()
}

/// The contract this vertical is compiled, scheduled, and compared under.
///
/// One value read by both the region's realization and the oracle's
/// conformance, so neither can drift from the other.
pub(crate) const fn declared_contract() -> NumericalContract {
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16
}

/// The realization the scheduled region declares.
///
/// Every dimension is read off [`declared_contract`] rather than written out,
/// so a contract dimension that moved would move this region with it instead of
/// leaving a stale copy that still verifies.
pub(crate) fn declared_realization() -> NumericalRealization {
    realization_of(declared_contract())
}

/// Restates one caller contract as a region's own declared realization.
///
/// Parameterized because the *strict* BF16 contract is the perturbation this
/// vertical watches emission refuse: the measured macOS row flushes, so a
/// region declaring preservation must be told no, and asking that requires
/// building the same region under the other contract rather than describing it.
pub(crate) fn realization_of(contract: NumericalContract) -> NumericalRealization {
    NumericalRealization::new(
        contract.key(),
        u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
        contract.input_subnormals(),
        contract.result_subnormals(),
        contract.contraction(),
        contract.reassociation(),
        contract.permutation(),
        contract.signed_zero(),
        contract.nan_assumptions(),
        contract.infinity_assumptions(),
    )
}

/// The numerical conformance the oracle is evaluated under.
///
/// Stated here, from the same contract the region carries. See the module
/// header for why it is not derived through
/// `ReferenceNumericalConformance::from_realization`.
pub(crate) fn declared_conformance() -> ReferenceNumericalConformance {
    let contract = declared_contract();
    ReferenceNumericalConformance::new(contract.input_subnormals(), contract.result_subnormals())
}

/// Builds the semantic `(x * 1.5) + 0.0` program the oracle evaluates.
pub(crate) fn semantic_program(key: &InputKey, elements: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed semantic profile composes");
    let input = builder
        .input::<Bf16>(key.clone(), Shape::from_dims([elements]))
        .expect("the bf16 input binds");
    let scale = Bf16Constant::apply(&mut builder, SCALE_BITS).expect("the scale applies");
    let product = Bf16Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let bias = Bf16Constant::apply(&mut builder, BIAS_BITS).expect("the bias applies");
    let root = Bf16Add::apply(&mut builder, product, bias).expect("the bias applies");
    builder
        .output(
            OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
            root,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Assembles the same computation as a verified BF16 scheduled region.
///
/// Through `tiler_ir`'s public builders, because the compiler's recognizer
/// refuses every non-`f32` program before a subject is normalized. The module
/// header carries that boundary; it is restated here because this function is
/// the place a reader would otherwise expect a `compile()` call.
pub(crate) fn scheduled_region(elements: u64) -> VerifiedScheduledRegion {
    region_under(elements, declared_realization())
}

/// Assembles the region under one stated realization.
pub(crate) fn region_under(
    elements: u64,
    realization: NumericalRealization,
) -> VerifiedScheduledRegion {
    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("the region reads its first input");
    let scale = expression
        .constant(SCALE_BITS)
        .expect("a bf16 constant is a bounded node");
    let product = expression
        .multiply(input, scale)
        .expect("the multiplication applies");
    let bias = expression
        .constant(BIAS_BITS)
        .expect("a bf16 constant is a bounded node");
    let root = expression.add(product, bias).expect("the addition applies");
    let expression = expression.build(root).expect("the expression verifies");

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder
        .iteration_shape(Shape::from_dims([elements]))
        .expect("a one-dimensional iteration shape is admitted");
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("the read access is admitted");
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("the write access is admitted");
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Output),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .expect("a linear bounds proof is admitted");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .expect("one invocation per output is admitted");
    builder
        .scalar_program(ScalarProgram::PointwiseBf16(expression))
        .expect("a pointwise bf16 program is admitted");
    builder
        .numerical(realization)
        .expect("the stated realization is admitted");
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: elements,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: elements,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("the schedule is admitted");
    builder.build().expect("the region verifies")
}

/// Evaluates the semantic program through the oracle under one stated contract.
pub(crate) fn reference_bits(conformance: ReferenceNumericalConformance) -> Vec<u16> {
    let encodings = operands();
    let elements = u64::try_from(encodings.len()).expect("the corpus length fits a u64");
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let program = semantic_program(&key, elements);
    let tensor = reference_tensor(&encodings);
    let outputs = ReferenceEvaluator::under(
        FrozenReferenceRegistry::standard().expect("the governed reference profile composes"),
        conformance,
    )
    .evaluate(&program, &[InputBinding::new(&key, &tensor)])
    .expect("a pure-bf16 program evaluates");
    reference_encodings(&outputs[0])
}

/// Builds the oracle's operand tensor from BF16 encodings.
///
/// Most-significant byte first, which is Tiler's canonical float payload order
/// and is a different question from the byte order a device buffer uses; the
/// two are separated at [`pack`].
fn reference_tensor(encodings: &[u16]) -> Tensor {
    Tensor::dense(
        Bf16::resolved_type(),
        Shape::from_dims([u64::try_from(encodings.len()).expect("a corpus length fits a u64")]),
        encodings
            .iter()
            .map(|encoding| {
                ReferenceElement::from_float_bits(
                    encoding.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("a two-byte payload is a bounded element")
            })
            .collect(),
    )
    .expect("a bounded bf16 tensor")
}

/// Reads a dense BF16 reference tensor back as encodings.
fn reference_encodings(tensor: &Tensor) -> Vec<u16> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a bf16 reference result is dense");
    };
    elements
        .iter()
        .map(|element| {
            u16::from_be_bytes(
                <[u8; 2]>::try_from(element.as_bytes()).expect("a bf16 element is two bytes"),
            )
        })
        .collect()
}

/// Which element width the host derives its buffer layout from.
///
/// **The composition's one width decision, made explicit so it can be
/// perturbed.** Every layer of this vertical is tested against its neighbour
/// with counts that agree on both sides, and that is exactly the arrangement a
/// wrong width survives: a two-byte element counted as four passes every
/// single-layer test. This enum is what lets one side of the composition be
/// given the neighbouring width while the kernel keeps its own, which is the
/// asymmetry a real typo produces and the symmetric version cannot expose —
/// packing *and* unpacking at four bytes round-trips correctly and proves
/// nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperandStride {
    /// The width the physical carrier declares for `tiler::bf16@1`.
    Declared,
    /// The neighbouring `f32` carrier's width, for the perturbation.
    NeighbouringF32,
}

impl OperandStride {
    /// The bytes one element occupies under this reading.
    ///
    /// Read from [`StorageScalar`], the single width authority, rather than
    /// written as a literal — so the declared reading cannot be two while the
    /// carrier says something else.
    pub(crate) fn bytes(self) -> usize {
        let width = match self {
            Self::Declared => StorageScalar::Bf16.byte_width(),
            Self::NeighbouringF32 => StorageScalar::F32.byte_width(),
        };
        usize::try_from(width).expect("a carrier width fits a usize")
    }
}

/// Packs BF16 encodings into a dense device payload at one stated stride.
///
/// Little-endian, because the device reads the shared buffer in its own byte
/// order and this run is bounded to an Apple silicon row where that order is
/// the host's. A wrong choice here is not silent: `1.0` would reach the kernel
/// as `0x803f` and no expected encoding in the corpus would hold.
pub(crate) fn pack(encodings: &[u16], stride: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; encodings.len() * stride];
    for (index, encoding) in encodings.iter().enumerate() {
        let start = index * stride;
        bytes[start..start + 2].copy_from_slice(&encoding.to_le_bytes());
    }
    bytes
}

/// Reads `count` BF16 encodings out of a dense device payload at one stride.
pub(crate) fn unpack(bytes: &[u8], stride: usize, count: usize) -> Vec<u16> {
    (0..count)
        .map(|index| {
            let start = index * stride;
            u16::from_le_bytes(
                <[u8; 2]>::try_from(&bytes[start..start + 2]).expect("a bf16 element is two bytes"),
            )
        })
        .collect()
}

/// The emitted, target-bound half of the vertical, device-free.
///
/// Everything here composes on any host: emission consults the authoritative
/// declaration's recorded facts and binds no device and no toolchain.
pub(crate) struct EmittedVertical {
    /// The authoritative macOS Apple9 declaration this unit was emitted against.
    pub(crate) declaration: BoundMetalCompileDeclaration,
    /// The verified structured kernel the region lowered to.
    pub(crate) kernel: VerifiedKernel,
    /// The emitted Metal translation unit.
    pub(crate) unit: MetalTranslationUnit,
    /// Argument-table index of the read buffer.
    pub(crate) operand_index: u64,
    /// Argument-table index of the write buffer.
    pub(crate) result_index: u64,
    /// Addressable elements the kernel declares on each boundary.
    pub(crate) element_count: u64,
    /// Threads the schedule's launch covers.
    pub(crate) grid_threads: u64,
    /// Threads per workgroup the schedule declares.
    pub(crate) threads_per_workgroup: u32,
}

/// Why the emitted half could not be assembled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EmitFailure {
    /// The authoritative declaration did not assemble.
    Declaration(String),
    /// The region did not lower to a verified kernel.
    Lowering(String),
    /// Emission refused the kernel for this target.
    Emission(String),
    /// The target cannot honour the region's declared numerical realization.
    UnrealizableNumerics(String),
    /// The emitted unit did not declare the boundary shape this run binds.
    UnexpectedSignature(String),
}

impl std::fmt::Display for EmitFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declaration(cause) => write!(formatter, "the declaration refused: {cause}"),
            Self::Lowering(cause) => write!(formatter, "the region did not lower: {cause}"),
            Self::Emission(cause) => write!(formatter, "emission refused: {cause}"),
            Self::UnrealizableNumerics(cause) => write!(
                formatter,
                "the target cannot honour the declared realization: {cause}",
            ),
            Self::UnexpectedSignature(detail) => {
                write!(
                    formatter,
                    "the emitted signature is not the bound one: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for EmitFailure {}

/// Emits the vertical's kernel against the authoritative macOS Apple9 row.
///
/// # Errors
///
/// Returns the named refusal of whichever layer declined: the declaration, the
/// lowering, emission itself, the declared-realization conformance check, or
/// the signature this run binds against.
pub(crate) fn emit_vertical(elements: u64) -> Result<EmittedVertical, EmitFailure> {
    let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
        .map_err(|cause| EmitFailure::Declaration(cause.to_string()))?;
    let region = scheduled_region(elements);
    let kernel = lower_scheduled_region(&region)
        .map_err(|cause| EmitFailure::Lowering(format!("{cause:?}")))?;
    let unit = emit_translation_unit(
        &[&kernel],
        declaration.metal_facts(),
        declaration.emission(),
    )
    .map_err(|cause| EmitFailure::Emission(cause.to_string()))?;
    // Emission succeeds even when the target cannot honour the declared
    // contract, so conformance is asked explicitly rather than inferred. A
    // strict BF16 region reaches `SubnormalFlushInArithmetic` here; the
    // flush-accepting one this vertical declares is honoured.
    unit.require_declared_realization()
        .map_err(|cause| EmitFailure::UnrealizableNumerics(cause.to_string()))?;

    let [entry] = unit.entry_points() else {
        return Err(EmitFailure::UnexpectedSignature(format!(
            "{} entry point(s), expected one",
            unit.entry_points().len()
        )));
    };
    let mut operand_index = None;
    let mut result_index = None;
    let mut element_count = None;
    for binding in entry.buffers() {
        let parameter = binding.parameter();
        // Exhaustive over the boundary vocabulary rather than written with a
        // wildcard: a pointwise region publishing its result carries exactly one
        // read input and one written output, and a materialized intermediate
        // would mean a second dispatch this run does not encode.
        match parameter.tensor {
            TensorRole::Input { .. } => operand_index = Some(u64::from(binding.index())),
            TensorRole::Output => result_index = Some(u64::from(binding.index())),
            TensorRole::Intermediate => {
                return Err(EmitFailure::UnexpectedSignature(
                    "the entry point binds a materialized intermediate".to_owned(),
                ));
            }
        }
        // The two boundaries of a pointwise region address the same element
        // count, and this run binds one buffer length for both. Reading it from
        // the kernel rather than from `elements` is what makes the buffer length
        // the *kernel's* claim; a disagreement between the two boundaries would
        // be a defect this refusal names rather than a length silently taken
        // from the first parameter.
        match element_count {
            None => element_count = Some(parameter.element_count),
            Some(declared) if declared == parameter.element_count => {}
            Some(declared) => {
                return Err(EmitFailure::UnexpectedSignature(format!(
                    "the boundaries declare {declared} and {} elements",
                    parameter.element_count
                )));
            }
        }
    }
    let (Some(operand_index), Some(result_index), Some(element_count)) =
        (operand_index, result_index, element_count)
    else {
        return Err(EmitFailure::UnexpectedSignature(
            "the entry point does not declare one read and one write boundary".to_owned(),
        ));
    };

    let launch = region.region().schedule.launch;
    Ok(EmittedVertical {
        declaration,
        kernel,
        unit,
        operand_index,
        result_index,
        element_count,
        grid_threads: launch.grid_threads,
        threads_per_workgroup: launch.threads_per_workgroup,
    })
}

#[cfg(test)]
mod tests;
