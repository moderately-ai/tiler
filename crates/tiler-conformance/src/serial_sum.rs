//! The `f32` serial-sum vertical, carried from a semantic program to dispatched
//! device results and compared bit for bit against the independent oracle.
//!
//! # What one run crosses
//!
//! `sum((x * 1.0) + 0.0)` over the inner axis is built as a `tiler_ir`
//! **semantic** program, compiled through `tiler_compiler` against the
//! authoritative macOS Apple9 declaration, emitted as MSL, linked to a
//! `metallib` by the real Apple offline toolchain, and dispatched on this host's
//! GPU. The oracle side evaluates the *same semantic program* through
//! `tiler-reference`, which shares no code with the compiler's lowering, the
//! emitter, or the kernel. An agreement is therefore two independent
//! implementations of one declared contract arriving at the same bits, not one
//! implementation checked against itself.
//!
//! Unlike `crate::bf16_vertical`, the compiler **is** in the path here, so the
//! portfolio, the plan alternatives, and their ABI are all crossed.
//!
//! **Corrected 2026-08-07 as to why — the conclusion above is unchanged.** This
//! read "the recognizer's `dtype-f32` rule admits this program", and that rule
//! is retired: `widen-the-strategy-recognizer-past-the-f32-wall` replaced it
//! with a derivation of the program's own arithmetic, which would admit
//! `crate::bf16_vertical`'s program too, so the recognizer is no longer what
//! separates the two verticals. The **target profile** is. `FIRST_MACOS_APPLE9`
//! declares complete `f32` numerical rows and only the two measured BF16
//! subnormal ones, so this program's `f32` contract resolves against declared
//! rows where the BF16 one meets `Unknown` on the first undeclared consumable
//! dimension and is refused before any plan exists. `crate::bf16_vertical`'s
//! module header states that boundary and names the test that observes it.
//!
//! # The comparison is on exact bit patterns rather than an epsilon
//!
//! The program declares a numerical contract; a result that is close but not
//! equal has violated it, and reporting that as success would make the contract
//! decorative. Where a contract *permits* reassociation the oracle is still the
//! reference's — evaluated against the grouping the physical plan declared — and
//! never a tolerance. [`partitioned_reference`] is that oracle, and
//! [`declared_partition`] is what reads the grouping off the plan rather than
//! assuming it.
//!
//! # Four claims, and why none of them subsumes another
//!
//! **The direct path** dispatches the selected alternative for the
//! [`ROWS`]-by-[`COLUMNS`] shape under
//! `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`, over operands that include
//! a negative zero, the least positive subnormal, a non-canonical NaN payload,
//! and an infinity. It is evidence about the compiler and the emitter on the
//! values where a numerical contract either holds or is decorative.
//!
//! **The parallel strategies** dispatch every alternative a
//! `FLUSH_AND_REASSOCIATE_F32` contract retains — the serial fold, the
//! multi-pass split, and the single-workgroup tree — over
//! [`PARALLEL_OPERANDS`], every grouping of which is exact. That is the claim a
//! *compiling* cooperative golden cannot make: compilation success is not a
//! capability fact, and it says nothing about whether the barrier synchronizes,
//! whether the threadgroup allocation is reachable, or whether the tree computes
//! the declared sum.
//!
//! **The grouping-sensitive case** dispatches the same three alternatives over
//! [`GROUPING_SENSITIVE_OPERANDS`], where the declared regroupings genuinely
//! disagree, and holds each to the grouping *it* published. It is the one the
//! parallel case deliberately cannot make: on exact operands the refusal
//! population among legal groupings is empty, so nothing a reassociating
//! contract permits would have failed it. What it cannot do is separate the two
//! *parallel* strategies, because at [`PARALLEL_COLUMNS`] contributors they
//! declare the same partition.
//!
//! **The separating case** dispatches the same three alternatives at
//! [`SEPARATING_COLUMNS`] contributors, which is the smallest count at which the
//! tree's and the split's rules choose differently — the tree six partitions of
//! two, the split four of three. Over [`SEPARATING_OPERANDS`] those two
//! groupings return *different* `f32` values, both legal under the contract, so
//! holding the tree to the split's declared partition refuses a value the
//! contract permits. That is the wrong-but-in-range refusal, and nothing at four
//! contributors can state it.
//!
//! # Why the pair of operand sets, stated as counts
//!
//! Each half is weak exactly where the other is strong, and the pair runs at
//! both shapes for the same reason.
//! `tests::the_operand_pair_covers_what_each_half_alone_cannot` pins the
//! four-contributor numbers: of the sixteen single-contributor corruptions of
//! the declared grouping — each slot dropped, and each slot taking another
//! slot's value — [`PARALLEL_OPERANDS`] leaves none undetected and
//! [`GROUPING_SENSITIVE_OPERANDS`] leaves one; of the five order-preserving
//! groupings over four contributors, the first set produces one value and the
//! second produces two.
//!
//! `tests::the_separating_operand_pair_covers_what_each_half_alone_cannot` pins
//! the twelve-contributor ones, and the gap there is far wider because
//! [`SEPARATING_OPERANDS`] pads with eight `+0.0`: of the 144 corruptions it
//! leaves 81 undetected under the tree's grouping and 98 under the split's,
//! where [`SEPARATING_EXACT_OPERANDS`] leaves none under either. **A padded set
//! is not a contributor-set claim**, which is exactly why the separating shape
//! carries a genuine twelve-wide exact set beside it rather than the padded one
//! alone. Neither half replaces the other at either shape, and a later edit
//! dropping one would have to change those numbers to do it.

use tiler_build::{BoundMetalCompileDeclaration, BoundMetalDeclarationError};
use tiler_compiler::session::{
    Compilation, CompileFailure, CompileRequest as CompilerRequest, NumericalContract,
    PlanAlternative, compile,
};
use tiler_compiler::target::{TargetRequest, TargetRequestError};
use tiler_ir::program::abi::{AbiRoot, ExprNode};
use tiler_ir::schedule::ContributorPartition;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::applicability::MetalHostApplicabilityRefusal;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
    strict_partitioned_sum,
};

use crate::measurement::Measured;

/// Byte width of one `f32`.
pub(crate) const F32_BYTES: u64 = 4;
/// Interface key of the serial sum's one input.
pub(crate) const INPUT_KEY: &str = "input";
/// Interface key of the serial sum's one output.
pub(crate) const OUTPUT_KEY: &str = "result";

/// Rows of the direct path's input; each row reduces to one output element.
pub(crate) const ROWS: u64 = 4;
/// Columns of the direct path's input; the reduced axis.
///
/// Three contributors per row is what makes a serial reduction's ordering
/// observable while keeping every operand class in [`ROW_PATTERNS`] reachable.
pub(crate) const COLUMNS: u64 = 3;

/// Rows of the parallel strategies' input.
///
/// **One, because the grouping-sensitive case enumerates one row's orderings.**
/// That case walks every order-preserving grouping of a single contributor
/// sequence, and the `const` assertion standing beside it stops the build if this
/// value moves without the enumeration being made per row. Both operand sets are
/// one row of [`PARALLEL_COLUMNS`] values for the same reason: a second row would
/// need a second set and would add no grouping to observe.
pub(crate) const PARALLEL_ROWS: u64 = 1;

/// Contributors reduced per output on the parallel strategies' input.
///
/// **Four, because below that nothing splits.** Both partition rules require at
/// least two partitions of at least two contributors each — `governed_partition`
/// for the split, `capped_tree_partition` for the tree — so four is the smallest
/// extent at which a split or a workgroup tree exists to be retained at all. The
/// two admit exactly the same extents and differ only in the width they pick, so
/// this floor is one number rather than a coincidence. It is also the smallest
/// extent *above* the sub-four reduction that
/// `correct-the-declined-strategy-record-for-an-unsplittable-reduction` records
/// failing with `InvalidCompilerOutput` under a reassociation-permitting
/// contract: this shape is sized above that defect rather than around it, so a
/// regression there fails here rather than hiding.
pub(crate) const PARALLEL_COLUMNS: u64 = 4;

/// Rows of the separating shape's input.
///
/// One, for the reason [`PARALLEL_ROWS`] is one: the separating case enumerates
/// a single contributor sequence's orderings.
pub(crate) const SEPARATING_ROWS: u64 = 1;

/// Contributors reduced per output at the count where the two parallel rules
/// choose differently.
///
/// **Twelve, and it is the smallest such count.** `capped_tree_partition` walks
/// *down* from `ceiling = min(256, contributors / 2) = 6` for the largest
/// divisor at or below it and takes six partitions of two; `governed_partition`
/// walks down from `isqrt(12) = 3` and takes four of three. The two rules read
/// opposite ends of the divisor lattice, which is why they diverge at all.
/// Counts four through eleven agree — 4, 6, 8, 9 and 10 give both rules the same
/// answer, and 5, 7 and 11 are prime, which both decline — so twelve is the
/// minimum rather than merely a count that works.
///
/// The cap moves the *choice* and never the domain: over `0..200_000` the two
/// rules admit and decline exactly the same counts, so this shape retains the
/// same three alternatives [`PARALLEL_COLUMNS`] does.
/// `tiler_compiler::pipeline::tests::the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs`
/// owns that population claim; what this constant selects is one count at which
/// a *device* can be asked the question.
pub(crate) const SEPARATING_COLUMNS: u64 = 12;

/// Both shapes enumerate one row's contributor sequence, so this holds while
/// each is one row and stops the build otherwise.
const _: () = assert!(
    PARALLEL_ROWS == 1 && SEPARATING_ROWS == 1,
    "the grouping-sensitive and separating cases enumerate one row's orderings; a wider shape \
     needs the enumeration to run per row before these constants move",
);

/// The **contributor-set** half of the parallel operand pair.
///
/// **Every grouping is exact, which is what makes one serial-fold oracle valid
/// for all three strategies.** The contract these run under *permits* ordered
/// regrouping, so a split and a tree may legitimately sum in an order the
/// reference's declared left fold does not. Distinct small powers of two are
/// exactly representable and their partial sums are too, so every partition and
/// every tree depth produces the identical `f32`.
///
/// **Every subset has a distinct sum, so a dropped or double-counted contributor
/// cannot cancel.** These are the failure modes a parallel reduction actually
/// has — a partition boundary off by one, a participant whose partial is never
/// combined, an unsynchronized read of a partial written by another invocation —
/// and with powers of two each of them changes the result to a value no correct
/// grouping produces. [`ROW_PATTERNS`] would not do this job: its rows repeat
/// `1.0`, so dropping one contributor and double-counting another agree.
///
/// **What it cannot say, stated exactly.** Because every grouping is exact,
/// every order-preserving regrouping of these four operands produces
/// `0x41700000` and *no other value* — so a comparison against the serial fold
/// has an empty refusal population among legal groupings and cannot observe
/// rounding at all. That is why it is one half of a pair rather than the whole
/// claim.
pub(crate) const PARALLEL_OPERANDS: [u32; 4] = [
    0x3f80_0000, // 1.0
    0x4000_0000, // 2.0
    0x4080_0000, // 4.0
    0x4100_0000, // 8.0
];

/// The **rounding** half of the parallel operand pair, chosen so the declared
/// regroupings disagree by exactly one rounding step.
///
/// Written as bit patterns rather than decimal literals because the whole point
/// is which representable value each operand is: `4.4703484e-8` names a printed
/// approximation, and `0x3340_0000` names the operand.
///
/// | bits | value | in units of `ulp(1.0)` = `2^-23` |
/// | --- | --- | --- |
/// | `0x3f40_0000` | `0.75` | — |
/// | `0x3e80_0000` | `0.25` | — |
/// | `0x3340_0000` | `3 * 2^-26` | `0.375` |
/// | `0x3300_0000` | `2^-25` | `0.25` |
///
/// **The derivation, so the two answers are attributable rather than merely
/// different.** At four contributors both rules return two partitions of two, so
/// both parallel strategies fold `(a0 + a1) + (a2 + a3)` while the serial fold
/// folds `((a0 + a1) + a2) + a3`; both share the prefix `0.75 + 0.25 = 1.0`,
/// exact. The serial fold then adds `0.375 ulp` and `0.25 ulp` in turn, and each
/// lands below the half-ulp boundary on its own, so each rounds back to `1.0`.
/// The declared regrouping adds them to each other first — `0.625 ulp`, exact,
/// because both are dyadic — and one add of `1.0 + 0.625 ulp` rounds *up*. So the
/// parallel answer is `0x3f800001` and the serial answer is `0x3f800000`: one ULP
/// apart, and the difference is one named rounding step rather than a tolerance.
///
/// **No step is a tie**, deliberately: `0.375`, `0.25`, and `0.625` are each
/// strictly off the half-ulp boundary, so nothing here depends on
/// round-half-to-even and a host resolving ties differently would still produce
/// these bits. Every operand is normal — the smallest is `2^-25`, a hundred
/// binades above the subnormal boundary — so the flush-to-zero half of the
/// contract changes none of them, and `x * 1.0 + 0.0` is bit-identity on each,
/// which is what lets the reduction oracle be applied to these operands rather
/// than to the prologue's output.
///
/// **What it cannot say, stated exactly.** Its subset sums are *not* distinct: of
/// the sixteen single-contributor corruptions of the declared grouping, fifteen
/// change the answer and one does not (slot 3 taking slot 2's value also yields
/// `0x3f800001`). [`PARALLEL_OPERANDS`] leaves none of the sixteen undetected,
/// which is why both sets run rather than one replacing the other.
pub(crate) const GROUPING_SENSITIVE_OPERANDS: [u32; 4] = [
    0x3f40_0000, // 0.75
    0x3e80_0000, // 0.25
    0x3340_0000, // 3 * 2^-26, which is 0.375 ulp(1.0)
    0x3300_0000, // 2^-25,     which is 0.25  ulp(1.0)
];

/// The **rounding** half of the separating shape's operand pair: the four
/// operands above, padded to [`SEPARATING_COLUMNS`] with eight `+0.0`.
///
/// **The padding is what makes the two parallel groupings disagree at twelve.**
/// The tree's six partitions of two put `0.75 + 0.25` in the first and
/// `0.375 ulp + 0.25 ulp` in the second, so the exact `0.625 ulp` reaches the
/// combining add and `1.0 + 0.625 ulp` rounds *up* — the same derivation
/// [`GROUPING_SENSITIVE_OPERANDS`] states at four, reached at twelve. The split's
/// four partitions of three put `0.75 + 0.25 + 0.375 ulp` in the first, which
/// rounds back to `1.0`, and `0.25 ulp` alone in the second, which then rounds
/// back to `1.0` again. So the tree returns `0x3f800001` and the split
/// `0x3f800000`, and the eight `+0.0` contribute nothing but the two partition
/// boundaries.
///
/// Both values are order-preserving blocked regroupings of the declared
/// sequence, so both are legal under `FLUSH_AND_REASSOCIATE_F32`. That is the
/// whole point: holding the tree to the split's declared partition refuses a
/// value the contract *permits*, which no tolerance and no permitted-set
/// membership test could refuse.
///
/// The split's answer here coincides with the serial fold's, and the coincidence
/// is stated rather than hidden — what this shape separates is the tree from the
/// split, which is the claim four contributors cannot make. Separating the split
/// from the fold is [`GROUPING_SENSITIVE_OPERANDS`]'s claim and stays there.
///
/// **What it cannot say, stated exactly.** Eight of its twelve slots are the
/// reduction's own identity, so dropping one changes nothing and one zero slot
/// taking another zero's value changes nothing either: of the 144
/// single-contributor corruptions it leaves 81 undetected under the tree's
/// grouping and 98 under the split's. A padded set is not a contributor-set
/// claim, and [`SEPARATING_EXACT_OPERANDS`] is why this one does not have to be.
pub(crate) const SEPARATING_OPERANDS: [u32; 12] = [
    0x3f40_0000, // 0.75
    0x3e80_0000, // 0.25
    0x3340_0000, // 3 * 2^-26, which is 0.375 ulp(1.0)
    0x3300_0000, // 2^-25,     which is 0.25  ulp(1.0)
    0x0000_0000, // +0.0, and eight of them: the padding to twelve contributors
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
];

/// The **contributor-set** half of the separating shape's operand pair: twelve
/// distinct powers of two.
///
/// **A genuine twelve-wide set rather than a padded four-wide one**, which is
/// the whole reason it exists. [`SEPARATING_OPERANDS`] separates the tree's
/// grouping from the split's and detects almost no dropped or double-counted
/// contributor; this set inverts both properties, exactly as
/// [`PARALLEL_OPERANDS`] does at four.
///
/// `2^0` through `2^11` sum to `4095`, and every partial sum of any subset is an
/// integer below `2^24`, so every one of the 58,786 order-preserving groupings
/// produces `0x457ff000` and no other value. Every subset has a distinct sum —
/// that is what a binary representation is — so a partition boundary off by one,
/// a participant whose partial is never combined, or an unsynchronized read of
/// another invocation's slot each moves the answer to a value no correct
/// grouping produces. All 144 single-contributor corruptions are detected under
/// both declared groupings.
pub(crate) const SEPARATING_EXACT_OPERANDS: [u32; 12] = [
    0x3f80_0000, // 1.0
    0x4000_0000, // 2.0
    0x4080_0000, // 4.0
    0x4100_0000, // 8.0
    0x4180_0000, // 16.0
    0x4200_0000, // 32.0
    0x4280_0000, // 64.0
    0x4300_0000, // 128.0
    0x4380_0000, // 256.0
    0x4400_0000, // 512.0
    0x4480_0000, // 1024.0
    0x4500_0000, // 2048.0
];

/// The operand pattern each row of the direct path's input is filled from.
///
/// Chosen to exercise the contract rather than to be arithmetically convenient:
/// a negative zero, the least positive subnormal, a non-canonical NaN payload,
/// and an infinity all appear, because those are the values where a numerical
/// contract either holds or is decorative. The interesting operand leads each
/// row, so a narrower reduction keeps every one of them.
pub(crate) const ROW_PATTERNS: [[u32; 3]; 4] = [
    [0x3f80_0000, 0x4000_0000, 0x4040_0000], // 1.0, 2.0, 3.0
    [0x8000_0000, 0x0000_0001, 0x3f80_0000], // -0.0, least subnormal, 1.0
    [0x7fc0_1234, 0x3f80_0000, 0x4000_0000], // non-canonical NaN, 1.0, 2.0
    [0x7f80_0000, 0x3f80_0000, 0xbf80_0000], // +inf, 1.0, -1.0
];

/// Fills one `rows` by `columns` input from [`ROW_PATTERNS`].
///
/// Cycling rather than indexing, so the pattern defines an input for any shape.
/// At the direct path's own four-by-three shape it reproduces exactly twelve
/// operands.
pub(crate) fn input_bits(rows: u64, columns: u64) -> Vec<u32> {
    let mut bits = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let pattern = ROW_PATTERNS[usize::try_from(row % 4).expect("a bounded row index")];
            bits.push(pattern[usize::try_from(column % 3).expect("a bounded column index")]);
        }
    }
    bits
}

/// The authoritative macOS Metal declaration this vertical compiles and emits
/// under.
///
/// Stated by `tiler-build` rather than here, so nothing in this crate holds a
/// second hand-written copy of a target whose rows have named authorities.
///
/// # Errors
///
/// Returns the declaration's own refusal when it does not assemble.
pub(crate) fn declaration() -> Result<BoundMetalCompileDeclaration, BoundMetalDeclarationError> {
    BoundMetalCompileDeclaration::first_macos_apple9()
}

/// Why this vertical could not reach a compiled portfolio.
#[derive(Debug)]
pub(crate) enum CompileRefusal {
    /// The declared profile is not a valid singleton target request.
    TargetRequest(TargetRequestError),
    /// The program did not compile.
    Compile(CompileFailure),
    /// The portfolio retained no target result at all.
    NoTarget,
    /// The target cannot honour the kernels' declared numerical contract.
    UnrealizableNumerics,
}

impl std::fmt::Display for CompileRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetRequest(cause) => write!(
                formatter,
                "the declared profile is not a valid target request: {cause}",
            ),
            Self::Compile(failure) => {
                write!(formatter, "the program did not compile: {failure:?}")
            }
            Self::NoTarget => formatter.write_str("the compile produced no target result"),
            Self::UnrealizableNumerics => formatter
                .write_str("the target cannot honour the kernels' declared numerical contract"),
        }
    }
}

impl std::error::Error for CompileRefusal {}

/// Compiles one program against the authoritative declaration's profile under a
/// stated contract.
///
/// The contract is an argument rather than a default because it is the whole
/// difference between the direct path and the parallel strategies: a
/// flush-only contract grants no regrouping and retains no split, and a
/// composed one is the only contract under which a split or a tree is a legal
/// implementation of this program on this hardware.
///
/// # Errors
///
/// Returns the refusal of whichever stage declined.
pub(crate) fn compile_under(
    declaration: &BoundMetalCompileDeclaration,
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<Compilation, CompileRefusal> {
    let targets = TargetRequest::new([declaration.profile().clone()])
        .map_err(CompileRefusal::TargetRequest)?;
    compile(CompilerRequest::new(program, contract, targets))
        .map_err(CompileRefusal::Compile)?
        .into_targets()
        .pop()
        .ok_or(CompileRefusal::NoTarget)?
        .into_parts()
        .1
        .map_err(|_| CompileRefusal::UnrealizableNumerics)
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis of a given shape.
pub(crate) fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new(INPUT_KEY).expect("the input key is valid"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(
            OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Builds the reference tensor one operand set states.
///
/// Bit patterns throughout, most-significant byte first, which is Tiler's
/// canonical float payload order and a different question from the byte order a
/// device buffer uses; the two are separated at [`pack_f32`]. A signed zero, a
/// subnormal, and a non-canonical NaN must reach the oracle unchanged, which they
/// would not if these were parsed as numbers.
pub(crate) fn operand_tensor(bits: &[u32], rows: u64, columns: u64) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([rows, columns]),
        bits.iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed")
}

/// Reads a dense reference tensor back as `f32` bit patterns.
///
/// # Panics
///
/// Panics when the tensor is not dense, which no `f32` reference output is.
pub(crate) fn dense_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a dense f32 reference output was expected");
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// Evaluates the same semantic program through the independent oracle.
pub(crate) fn reference_bits(
    program: &SemanticProgram,
    bits: &[u32],
    rows: u64,
    columns: u64,
) -> Vec<u32> {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = operand_tensor(bits, rows, columns);
    let outputs = ReferenceEvaluator::standard()
        .expect("the governed reference profile composes")
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    dense_bits(&outputs[0])
}

/// Packs `f32` bit patterns into a dense device payload at one stated stride.
///
/// Little-endian, because the device reads the shared buffer in its own byte
/// order and this run is bounded to an Apple silicon row where that order is the
/// host's. **The width is here, in safe code, rather than inside an unsafe
/// site**: `crate::device_buffer` exposes a byte interface precisely so a
/// mis-derived element width can be perturbed without perturbing a raw-pointer
/// copy with it.
pub(crate) fn pack_f32(bits: &[u32], stride: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; bits.len() * stride];
    for (index, value) in bits.iter().enumerate() {
        let start = index * stride;
        bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Reads `count` `f32` bit patterns out of a dense device payload at one stride.
///
/// # Panics
///
/// Panics when `bytes` is shorter than `count` elements at `stride`, which is a
/// caller that mis-derived one of the two.
pub(crate) fn unpack_f32(bytes: &[u8], stride: usize, count: usize) -> Vec<u32> {
    (0..count)
        .map(|index| {
            let start = index * stride;
            u32::from_le_bytes(
                <[u8; 4]>::try_from(&bytes[start..start + 4])
                    .expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// Which parallel reduction strategy one retained alternative realizes.
///
/// **Recognized by an observable each strategy alone has, not by a name.** The
/// compiler publishes a plan alternative's kernels and its ABI, never its
/// reduction topology, so asking "is this the tree?" has to be answered from what
/// the alternative *declares*. The multi-pass split is the only alternative with
/// three stages — pointwise, partial, and final. The single-workgroup tree is the
/// only one declaring an entry wider than one thread per workgroup: it launches
/// one invocation per participant inside one workgroup, where every
/// independent-invocation region declares a width of one. The serial fold
/// declares neither.
///
/// This mirrors `tiler-build`'s own
/// `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio`, which
/// recognizes the same two strategies through the same two observables. It is
/// deliberately the same rule rather than a second one: that fixture proves the
/// portfolio *retains* them on this profile, and this vertical proves they *run*,
/// so a divergence in what "the tree" means would make the two claims about
/// different things.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParallelStrategy {
    /// Three stages: map, reduce each partition, combine the partials.
    MultiPassSplit,
    /// One workgroup whose participants reduce cooperatively through a tree.
    SingleWorkgroupTree,
}

impl ParallelStrategy {
    /// A stable lowercase identifier for this strategy.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::MultiPassSplit => "multi-pass-split",
            Self::SingleWorkgroupTree => "single-workgroup-tree",
        }
    }
}

impl std::fmt::Display for ParallelStrategy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Names one classified alternative for a message.
pub(crate) fn strategy_label(strategy: Option<ParallelStrategy>) -> &'static str {
    match strategy {
        Some(strategy) => strategy.as_str(),
        None => "serial-fold",
    }
}

/// Why a run could not read the grouping an alternative declared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GroupingRefusal {
    /// An ABI launch quantity is not the declared literal this reader requires.
    ///
    /// **A position is not a width, and reading it as one is silent.** Every
    /// launch quantity on the alternatives this profile retains is a declared
    /// literal, so a node that is not one means the derivation moved and this
    /// reader stopped measuring what it names. That is a refusal rather than a
    /// skip: a skipped entry would leave a strategy unrecognized and the run
    /// would report proving one strategy while believing it had proved two.
    NonLiteralLaunch {
        /// The arena position that was read.
        position: u32,
        /// What was found there instead.
        node: String,
    },
    /// An alternative's published launch geometry names no covering partition.
    ///
    /// A refusal rather than a fallback to some default grouping: both parallel
    /// strategies decline an inexact split rather than padding one, so a
    /// partition that does not cover the contributor sequence exactly once each
    /// means this reader stopped measuring what it names — and an oracle asked
    /// about the wrong order would report the device as wrong.
    UndeclaredGrouping {
        /// The strategy whose geometry was read.
        strategy: String,
        /// What made the partition unreadable.
        detail: String,
    },
}

impl std::fmt::Display for GroupingRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonLiteralLaunch { position, node } => write!(
                formatter,
                "ABI arena position {position} is not a declared unsigned literal: {node}",
            ),
            Self::UndeclaredGrouping { strategy, detail } => write!(
                formatter,
                "the {strategy} publishes no contributor partition this oracle can be asked \
                 about: {detail}",
            ),
        }
    }
}

impl std::error::Error for GroupingRefusal {}

/// Resolves one ABI arena position to the unsigned literal it must be.
///
/// # Errors
///
/// Returns [`GroupingRefusal::NonLiteralLaunch`] for anything else.
pub(crate) fn literal_extent(
    expressions: &[ExprNode],
    position: u32,
) -> Result<u64, GroupingRefusal> {
    let index = usize::try_from(position).expect("an arena position fits a usize");
    match expressions.get(index) {
        Some(ExprNode::Root(AbiRoot::UnsignedLiteral(value))) => Ok(*value),
        other => Err(GroupingRefusal::NonLiteralLaunch {
            position,
            node: format!("{other:?}"),
        }),
    }
}

/// Classifies one retained alternative by the two observables above.
///
/// Returns `None` for the serial fold, which declares neither three stages nor a
/// workgroup wider than one thread.
///
/// # Errors
///
/// Returns [`GroupingRefusal::NonLiteralLaunch`] when a launch quantity is not a
/// declared literal, because a skipped entry would leave a strategy
/// unrecognized.
pub(crate) fn classify_strategy(
    alternative: PlanAlternative<'_>,
) -> Result<Option<ParallelStrategy>, GroupingRefusal> {
    let abi = alternative.abi();
    let expressions = abi.expressions();
    let mut widest = 0_u64;
    for entry in abi.entries() {
        widest = widest.max(literal_extent(expressions, entry.threads_per_workgroup())?);
    }
    if widest > 1 {
        return Ok(Some(ParallelStrategy::SingleWorkgroupTree));
    }
    if alternative.kernels().len() == 3 {
        return Ok(Some(ParallelStrategy::MultiPassSplit));
    }
    Ok(None)
}

/// The blocked contributor partition one alternative declares.
///
/// **Read from the plan's own published launch geometry, never assumed.** Each
/// strategy publishes it in a different observable, which is why this reads a
/// different quantity per strategy rather than one field:
///
/// - The **tree** runs one participant per partition inside one workgroup, so its
///   declared `threads_per_workgroup` is the partition count. That is the same
///   observable [`classify_strategy`] recognizes it by, read here from the kernel
///   program's own stages rather than from the artifact-facing ABI view, because
///   these are the literals the dispatch actually encodes.
/// - The **split** stages the partials in a tensor whose partition axis is
///   innermost, so its partial pass launches `output_elements * partitions`
///   threads where its final pass launches `output_elements`. The ratio is the
///   partition count and needs no row count from this build.
/// - The **serial fold** declares no split at all, and the degenerate partition of
///   one contributor each is exactly its left fold. Nothing is read for it, and
///   that is stated rather than hidden: what makes it non-circular is that the
///   grouping-sensitive run cross-checks this partition's oracle against
///   `tiler-reference`'s evaluation of the whole semantic program, which is the
///   independent statement of the declared order.
///
/// **This vertical runs both sides of that distinction.** The split reads
/// `governed_partition` and the tree reads `capped_tree_partition` since the tree
/// took its measured participant cap, and the two agree at [`PARALLEL_COLUMNS`]
/// while diverging from [`SEPARATING_COLUMNS`] upward — six partitions of two
/// against four of three. So at one shape this function returns one partition for
/// both strategies and at the other it returns two different ones, from the same
/// three observables. Reading each partition from its own published geometry is
/// what makes that a measured difference rather than a restatement of whichever
/// rule the reader had in mind.
///
/// # Errors
///
/// Returns [`GroupingRefusal`] when the geometry is unreadable or the partition
/// does not cover the contributor sequence exactly once each. Both strategies
/// decline an inexact split rather than padding one, so a ragged partition here
/// means this reader stopped measuring what it names.
pub(crate) fn declared_partition(
    alternative: PlanAlternative<'_>,
    strategy: Option<ParallelStrategy>,
    contributors: u64,
) -> Result<ContributorPartition, GroupingRefusal> {
    let abi = alternative.abi();
    let program = abi.kernel_program();
    let expressions = program.abi_expressions();
    let mut stages = Vec::new();
    for stage in program.execution_order() {
        let launch = stage.launch();
        stages.push((
            literal_extent(expressions, launch.grid_threads)?,
            literal_extent(expressions, launch.threads_per_workgroup)?,
        ));
    }
    let partitions = match strategy {
        Some(ParallelStrategy::SingleWorkgroupTree) => stages
            .iter()
            .map(|(_, workgroup)| *workgroup)
            .max()
            .unwrap_or(1),
        Some(ParallelStrategy::MultiPassSplit) => {
            let [_pointwise, partial, combine] = stages.as_slice() else {
                return Err(GroupingRefusal::UndeclaredGrouping {
                    strategy: strategy_label(strategy).to_owned(),
                    detail: format!(
                        "a split declares three stages and this one declares {}",
                        stages.len(),
                    ),
                });
            };
            if combine.0 == 0 {
                return Err(GroupingRefusal::UndeclaredGrouping {
                    strategy: strategy_label(strategy).to_owned(),
                    detail:
                        "the combining stage launches no thread, so it names no partition count"
                            .to_owned(),
                });
            }
            partial.0 / combine.0
        }
        None => contributors,
    };
    if partitions == 0 || !contributors.is_multiple_of(partitions) {
        return Err(GroupingRefusal::UndeclaredGrouping {
            strategy: strategy_label(strategy).to_owned(),
            detail: format!(
                "{partitions} partition(s) do not cover {contributors} contributor(s) exactly once \
                 each",
            ),
        });
    }
    let partition = ContributorPartition {
        partitions,
        contributors_per_partition: contributors / partitions,
    };
    if !partition.covers(contributors) {
        return Err(GroupingRefusal::UndeclaredGrouping {
            strategy: strategy_label(strategy).to_owned(),
            detail: format!("{partition:?} does not cover {contributors} contributor(s)"),
        });
    }
    Ok(partition)
}

/// Evaluates the reduction one declared grouping computes, through the
/// independent oracle.
///
/// `tiler_reference::strict_partitioned_sum` is the second exact oracle the
/// reference crate already owns for exactly this question, and its own
/// documentation states why it has to exist: "a contract that permits
/// reassociation admits a set of results, so no oracle can answer *the* value for
/// it; what a plan can be checked against is the one order it selected". This
/// vertical reaches that oracle from a device rather than restating it.
///
/// It is applied to the *operands* rather than to the pointwise prologue's
/// output, and that is sound only while `x * 1.0 + 0.0` is bit-identity on this
/// operand set — which the grouping-sensitive run's calibration step checks by
/// requiring the degenerate partition's answer to equal the reference evaluator's
/// answer for the whole program, prologue included.
///
/// # Errors
///
/// Returns [`GroupingRefusal::UndeclaredGrouping`] when the split is not
/// evaluable.
pub(crate) fn partitioned_reference(
    bits: &[u32],
    rows: u64,
    columns: u64,
    partition: ContributorPartition,
) -> Result<Vec<u32>, GroupingRefusal> {
    let tensor = operand_tensor(bits, rows, columns);
    let reduced = strict_partitioned_sum(
        &tensor,
        &[Axis::new(1)],
        partition.partitions,
        partition.contributors_per_partition,
    )
    .map_err(|cause| GroupingRefusal::UndeclaredGrouping {
        strategy: "reference".to_owned(),
        detail: format!("{partition:?} is not an evaluable split: {cause}"),
    })?;
    Ok(dense_bits(&reduced))
}

/// Every `f32` value an order-preserving regrouping of one contributor sequence
/// can produce.
///
/// This is the *permitted set* — the population a reassociating contract
/// authorizes — and it is deliberately not the acceptance criterion. Requiring
/// membership would accept a strategy that produced some other legal grouping
/// than the one it declared, which is precisely the failure a declared-grouping
/// oracle exists to catch. What it is used for is the refusal population: every
/// member that is not the declared grouping's answer is a wrong-but-in-range
/// answer the oracle must say no to, and a run that cannot name one has a check
/// that cannot fail.
///
/// **Membership is not asserted either, and the absence is deliberate.** A
/// [`ContributorPartition`] expresses only a blocked uniform split, and every
/// blocked split of a contributor sequence is an order-preserving regrouping of
/// it, so an assertion that the declared grouping's value lies in this set could
/// not fail for any partition [`declared_partition`] can return.
///
/// Enumerated by splitting at every position and combining the two sides' values,
/// which is the same construction
/// [Numerical semantics](../../../docs/numerical-semantics.md)'s bounded
/// result-set oracle uses for three through six leaves; for four contributors it
/// yields the five full binary groupings that preserve leaf order.
pub(crate) fn ordered_associations(bits: &[u32]) -> Vec<u32> {
    let [single] = bits else {
        let mut values = Vec::new();
        for split in 1..bits.len() {
            for left in ordered_associations(&bits[..split]) {
                for right in ordered_associations(&bits[split..]) {
                    values.push((f32::from_bits(left) + f32::from_bits(right)).to_bits());
                }
            }
        }
        return values;
    };
    vec![*single]
}

/// The one comparison a declared-grouping oracle makes.
///
/// Named rather than written inline at each site, because "the check was watched
/// failing" is only true if the refusal is produced by the *same* expression that
/// accepted the observed answer. Two spellings of equality would leave the
/// observed comparison unexercised by the refusal.
pub(crate) fn declared_grouping_admits(expected: &[u32], candidate: &[u32]) -> bool {
    expected == candidate
}

/// What one dispatched alternative did, beyond the bits it produced.
///
/// Reported so a run's own output distinguishes the three strategies by evidence
/// rather than by the label this crate assigned them: a "tree" that launched one
/// thread per workgroup and reserved no threadgroup memory would be a
/// misclassification, and carrying both quantities is what makes that visible
/// instead of plausible.
#[derive(Clone, Debug)]
pub(crate) struct AlternativeRun {
    /// The strategy this alternative was classified as, or `None` for the fold.
    pub(crate) strategy: Option<ParallelStrategy>,
    /// The alternative's own stable identifier.
    pub(crate) stable_id: String,
    /// The blocked partition this alternative published.
    pub(crate) partition: ContributorPartition,
    /// The bit patterns the device wrote back.
    pub(crate) bits: Vec<u32>,
    /// Widest workgroup any stage of this alternative declared.
    pub(crate) widest_workgroup: u64,
    /// Most threadgroup memory any of its compiled pipelines statically reserves.
    pub(crate) threadgroup_bytes: u64,
    /// How many command encoders the submission carried, in execution order.
    pub(crate) encoders: usize,
    /// Bytes of `metallib` this alternative linked.
    ///
    /// Reported by the dispatch that linked it and summed into the measurement
    /// boundary, both of which are `apple`'s, so on a host that links nothing
    /// the field is written by nobody and read by nobody. It stays on the
    /// struct so `AlternativeRun` means the same thing on both hosts.
    #[cfg_attr(
        not(target_os = "macos"),
        allow(
            dead_code,
            reason = "the field is written and read only by `apple`, which a non-Apple host does not compile; stated per item because it is the whole of this module's non-Apple dead population, and stated under the negated predicate so a genuinely unread field is still a red build on the host that links"
        )
    )]
    pub(crate) metallib_bytes: usize,
}

impl AlternativeRun {
    /// This alternative's label in a run's own output.
    pub(crate) fn label(&self) -> &'static str {
        strategy_label(self.strategy)
    }
}

#[cfg(target_os = "macos")]
mod apple {
    //! Emitting, linking, and dispatching this vertical on a real device.

    use metal::{Buffer, Device, MTLResourceOptions};
    use tiler_build::BoundMetalCompileDeclaration;
    use tiler_compiler::session::{NumericalContract, PlanAlternative};
    use tiler_ir::program::{ValueRole, VerifiedKernelProgram};
    use tiler_metal::applicability::MetalHostApplicabilityRefusal;
    use tiler_metal::emit::emit_translation_unit;
    use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};

    use super::{
        AlternativeRun, COLUMNS, F32_BYTES, ROWS, classify_strategy, compile_under, declaration,
        declared_partition, input_bits, literal_extent, pack_f32, serial_sum_program, unpack_f32,
    };
    use crate::applicability::{describe, refuse_to_offer_the_declared_profile};
    use crate::device_buffer::write_bytes;
    use crate::device_preflight::PreflightRefusal;
    use crate::dispatch::{
        DeviceFacts, PreparedStage, admit_stage, device_facts, observe_metal_host, pipeline_for,
        run_stages,
    };
    use crate::measurement::host::{self, Unresolved};
    use crate::measurement::{Measured, MeasurementBoundary};

    /// The device storage one alternative's program needs, with inputs written.
    ///
    /// **One buffer per *allocation*, never per binding.** Two stages of a split
    /// address one intermediate, and the program states that by placing both
    /// values in one allocation. A host allocating per binding would hand the
    /// consumer a fresh buffer, the producer's partials would never reach it, and
    /// the reduction would read uninitialised device memory — a wrong answer
    /// rather than a refusal. `AllocationRef` compares by identity within its
    /// program, which is what makes the lookup exact rather than a length
    /// coincidence.
    struct AlternativeStorage {
        /// One buffer per program allocation, in the program's own order.
        buffers: Vec<Buffer>,
        /// The buffer the named program output lands in.
        output: Buffer,
        /// How many `f32` elements to read back out of it.
        readback: usize,
    }

    /// Allocates every buffer one alternative's program needs, input written.
    fn allocate_alternative(
        device: &Device,
        program: &VerifiedKernelProgram,
        bits: &[u32],
    ) -> Result<AlternativeStorage, String> {
        let allocations: Vec<_> = program.allocations().collect();
        let mut buffers = Vec::with_capacity(allocations.len());
        for allocation in &allocations {
            // Host-visible only where the host actually reads or writes: a
            // program input it fills and a program output it reads back. A
            // temporary is private, which is what a split's intermediate is.
            let host_visible = allocation
                .values()
                .any(|value| matches!(value.role(), ValueRole::Input | ValueRole::Output));
            let options = if host_visible {
                MTLResourceOptions::StorageModeShared
            } else {
                MTLResourceOptions::StorageModePrivate
            };
            buffers.push(device.new_buffer(allocation.capacity_bytes().max(1), options));
        }

        let index_of = |target: &_| {
            allocations
                .iter()
                .position(|candidate| candidate == target)
                .expect("every value's allocation is one this program declares")
        };

        let mut output = None;
        let mut readback = 0_usize;
        let mut inputs = 0_usize;
        for value in program.values() {
            let slot = index_of(&value.allocation());
            match value.role() {
                ValueRole::Input => {
                    // **One program input, and this path says so rather than
                    // assuming it.** `bits` is a single operand slice, so a
                    // program with two inputs would have the same operands
                    // written into both — a plausible tensor computed from the
                    // wrong bytes, which is the one failure class worse than a
                    // refusal. A multi-operand program routes through
                    // `crate::envelope`, where the artifact's declared interface
                    // supplies the ordinals.
                    inputs += 1;
                    if inputs > 1 {
                        return Err(format!(
                            "this vertical binds one operand slice by local knowledge and the \
                             program declares {inputs} tensor input(s)",
                        ));
                    }
                    write_bytes(
                        &buffers[slot],
                        &pack_f32(
                            bits,
                            usize::try_from(F32_BYTES).expect("a carrier width fits a usize"),
                        ),
                    );
                }
                ValueRole::Output => {
                    readback = usize::try_from(value.required_bytes() / F32_BYTES)
                        .expect("an output element count fits a usize");
                    output = Some(buffers[slot].clone());
                }
                ValueRole::Temporary => {}
            }
        }

        Ok(AlternativeStorage {
            output: output
                .ok_or_else(|| "the alternative's program publishes no named output".to_owned())?,
            buffers,
            readback,
        })
    }

    /// Emits, compiles, and dispatches one retained alternative on this device.
    ///
    /// **Every dispatch parameter is read from the compiler's own record**, and
    /// that is what makes a multi-stage launch here evidence rather than a
    /// hand-written guess: the argument-table index of each buffer comes from the
    /// emitter's binding table, the byte window from the program's own view, and
    /// both launch extents from the ABI arena. Nothing about the topology is
    /// assumed, which is why one function dispatches the fold, the split, and the
    /// tree unchanged.
    fn dispatch_alternative(
        device: &Device,
        facts: &DeviceFacts,
        declaration: &BoundMetalCompileDeclaration,
        alternative: PlanAlternative<'_>,
        bits: &[u32],
        contributors: u64,
    ) -> Result<AlternativeRun, String> {
        let strategy = classify_strategy(alternative).map_err(|cause| cause.to_string())?;
        let partition = declared_partition(alternative, strategy, contributors)
            .map_err(|cause| cause.to_string())?;

        let kernels: Vec<_> = alternative.kernels().iter().collect();
        let unit =
            emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
                .map_err(|cause| format!("emission refused: {cause}"))?;
        // Emission succeeds even when the target cannot honour the declared
        // contract, so conformance is asked explicitly rather than inferred.
        unit.require_declared_realization().map_err(|cause| {
            format!("the target cannot honour the declared realization: {cause}")
        })?;
        let request = CompileRequest::new(
            unit.source(),
            declaration.aot_target(),
            OptimizationLevel::Default,
            declaration.numerical_realization(),
        );
        let compiled = tiler_metal_aot::driver::Toolchain::system()
            .compile(&request)
            .map_err(|cause| format!("the emitted unit did not compile and link: {cause}"))?;

        let abi = alternative.abi();
        let program = abi.kernel_program();
        let expressions = program.abi_expressions();
        let storage = allocate_alternative(device, program, bits)?;

        // Resolved before the submission, so the encode looks nothing up and has
        // no failure of its own to report.
        let mut stages = Vec::new();
        let mut widest_workgroup = 0_u64;
        let mut threadgroup_bytes = 0_u64;
        for stage in program.execution_order() {
            let identity = stage.kernel().canonical_identity();
            let emitted = unit
                .entry_points()
                .iter()
                .find(|entry| entry.kernel_identity() == identity)
                .ok_or_else(|| {
                    "the emitted unit publishes no entry point for a stage's kernel".to_owned()
                })?;
            let pipeline = pipeline_for(device, &compiled.metallib, emitted.symbol())
                .map_err(|cause| cause.to_string())?;

            let launch = stage.launch();
            let grid_threads =
                literal_extent(expressions, launch.grid_threads).map_err(|c| c.to_string())?;
            let threads_per_workgroup = literal_extent(expressions, launch.threads_per_workgroup)
                .map_err(|cause| cause.to_string())?;
            // The declared workgroup against what this pipeline admits, and the
            // reserved threadgroup memory against what this device admits, both
            // before anything is encoded.
            admit_stage(
                stages.len(),
                emitted.symbol(),
                threads_per_workgroup,
                &pipeline,
                facts,
            )
            .map_err(|refusal: PreflightRefusal| refusal.to_string())?;
            widest_workgroup = widest_workgroup.max(threads_per_workgroup);
            threadgroup_bytes = threadgroup_bytes.max(pipeline.static_threadgroup_memory_length());

            // The emitter states which argument-table index each buffer
            // parameter binds at, and a stage binds its buffers to its accesses
            // positionally.
            let buffers = emitted.buffers();
            let mut placements = Vec::new();
            for (position, access) in stage.accesses().enumerate() {
                let binding = buffers.get(position).ok_or_else(|| {
                    format!("the emitted entry declares no buffer for access {position}")
                })?;
                let view = access.view();
                let slot = program
                    .allocations()
                    .position(|candidate| candidate == view.value().allocation())
                    .expect("every accessed value's allocation is one this program declares");
                placements.push((
                    u64::from(binding.index()),
                    storage.buffers[slot].clone(),
                    view.window().offset,
                ));
            }
            stages.push(PreparedStage {
                pipeline,
                placements,
                grid_threads,
                threads_per_workgroup,
                // No shape this vertical runs produces a zero-thread stage; the
                // flag is read from the launch so that stays a fact rather than
                // an assumption.
                skipped: false,
            });
        }

        let encoders = stages.len();
        let bytes = run_stages(device, &stages, &storage.output, storage.readback * 4)
            .map_err(|cause| format!("the dispatch did not complete: {cause}"))?;
        // Both `storage` and `stages` are still live here, which is the retention
        // this function owes: `run_stages` waits for the command buffer's
        // terminal state before returning, so every buffer the encode bound is
        // held across the whole device lifetime of the work that reads it. They
        // hold *clones* of one `MTLBuffer` per allocation — a retain, not a copy
        // — so it is the pair outliving the submission that matters.
        drop(stages);
        let readback = storage.readback;
        drop(storage);
        Ok(AlternativeRun {
            strategy,
            stable_id: alternative.stable_id().to_string(),
            partition,
            bits: unpack_f32(&bytes, 4, readback),
            widest_workgroup,
            threadgroup_bytes,
            encoders,
            metallib_bytes: compiled.metallib.len(),
        })
    }

    /// Asks this host, with its device observed, to earn the declared profile.
    ///
    /// Reported as a measured half rather than as a device-free case because the
    /// two device predicates are what carry the row past the ambient ones: an
    /// unobserved host refuses at `device-name` and says nothing about ADR
    /// 0086, and the deliverable is the *authority* refusal. Nothing is
    /// compiled, so the boundary's linked byte count is zero — the honest
    /// zero-unit case rather than a number this run did not produce.
    pub(super) fn run_offer() -> Measured<(String, MetalHostApplicabilityRefusal)> {
        let apple = match host::resolve() {
            Ok(apple) => apple,
            Err(Unresolved::Absent(reason)) => return Measured::Unavailable(reason),
            Err(Unresolved::Defect(detail)) => return Measured::Failed(detail),
        };
        let declaration = match declaration() {
            Ok(declaration) => declaration,
            Err(cause) => {
                return Measured::Failed(format!(
                    "the authoritative Metal declaration did not assemble: {cause}"
                ));
            }
        };
        let (observation, probed) = observe_metal_host(&apple.device);
        let refusal = refuse_to_offer_the_declared_profile(&observation);
        let boundary: MeasurementBoundary = host::boundary(&apple, &declaration, 0);
        Measured::Ran {
            boundary: Box::new(boundary),
            observed: (describe(&observation, probed), refusal),
        }
    }

    /// Runs the direct path's measured half: the selected alternative alone.
    pub(super) fn run_direct() -> Measured<Vec<u32>> {
        let apple = match host::resolve() {
            Ok(apple) => apple,
            Err(Unresolved::Absent(reason)) => return Measured::Unavailable(reason),
            Err(Unresolved::Defect(detail)) => return Measured::Failed(detail),
        };
        let declaration = match declaration() {
            Ok(declaration) => declaration,
            Err(cause) => {
                return Measured::Failed(format!(
                    "the authoritative Metal declaration did not assemble: {cause}"
                ));
            }
        };
        let program = serial_sum_program(ROWS, COLUMNS);
        let compilation = match compile_under(
            &declaration,
            &program,
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        ) {
            Ok(compilation) => compilation,
            Err(cause) => return Measured::Failed(cause.to_string()),
        };
        let Some(selected) = compilation.selected() else {
            return Measured::Failed("the portfolio retained no selected plan".to_owned());
        };
        let facts = device_facts(&apple.device);
        eprintln!("serial sum direct: device preflight — {facts}");
        let bits = input_bits(ROWS, COLUMNS);
        // The direct path's own shape reduces `COLUMNS` contributors per row, and
        // its selected plan is the serial fold, so the degenerate partition is
        // the grouping to read.
        match dispatch_alternative(
            &apple.device,
            &facts,
            &declaration,
            selected,
            &bits,
            COLUMNS,
        ) {
            Ok(run) => {
                eprintln!(
                    "serial sum direct: {} ({}), {} encoder(s), {:08x?}",
                    run.label(),
                    run.stable_id,
                    run.encoders,
                    run.bits,
                );
                let boundary: MeasurementBoundary =
                    host::boundary(&apple, &declaration, run.metallib_bytes);
                Measured::Ran {
                    boundary: Box::new(boundary),
                    observed: run.bits,
                }
            }
            Err(detail) => Measured::Failed(detail),
        }
    }

    /// Runs every alternative a reassociating contract retains, over one operand
    /// set at one shape.
    ///
    /// **The shape is an argument because the contributor count is what decides
    /// whether the two parallel rules agree.** At [`super::PARALLEL_COLUMNS`]
    /// they declare one partition between them and at
    /// [`super::SEPARATING_COLUMNS`] they declare two, and a run that hard-coded
    /// the first could not ask the second question at all. `columns` is both the
    /// reduced extent of the program built here and the contributor count each
    /// alternative's published geometry is read against, so the two cannot
    /// disagree.
    pub(super) fn run_portfolio(
        bits: &[u32],
        rows: u64,
        columns: u64,
    ) -> Measured<Vec<AlternativeRun>> {
        let apple = match host::resolve() {
            Ok(apple) => apple,
            Err(Unresolved::Absent(reason)) => return Measured::Unavailable(reason),
            Err(Unresolved::Defect(detail)) => return Measured::Failed(detail),
        };
        let declaration = match declaration() {
            Ok(declaration) => declaration,
            Err(cause) => {
                return Measured::Failed(format!(
                    "the authoritative Metal declaration did not assemble: {cause}"
                ));
            }
        };
        let program = serial_sum_program(rows, columns);
        // The composed contract, stated rather than defaulted. Every parallel
        // reduction regroups the declared contributor sequence, and Apple `f32`
        // arithmetic flushes subnormals in every math mode, so this is the one
        // contract under which a split or a tree is a legal implementation of
        // this program on this hardware.
        let compilation = match compile_under(
            &declaration,
            &program,
            NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        ) {
            Ok(compilation) => compilation,
            Err(cause) => return Measured::Failed(cause.to_string()),
        };
        let facts = device_facts(&apple.device);
        eprintln!("serial sum portfolio: device preflight — {facts}");

        let mut runs = Vec::new();
        let mut linked = 0_usize;
        for alternative in compilation.alternatives() {
            match dispatch_alternative(
                &apple.device,
                &facts,
                &declaration,
                alternative,
                bits,
                columns,
            ) {
                Ok(run) => {
                    linked += run.metallib_bytes;
                    runs.push(run);
                }
                Err(detail) => return Measured::Failed(detail),
            }
        }
        let boundary: MeasurementBoundary = host::boundary(&apple, &declaration, linked);
        Measured::Ran {
            boundary: Box::new(boundary),
            observed: runs,
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod apple {
    use tiler_metal::applicability::MetalHostApplicabilityRefusal;

    use super::AlternativeRun;
    use crate::measurement::{Measured, absent_apple_row};

    /// Reports the offer path's measured half as unavailable.
    pub(super) fn run_offer() -> Measured<(String, MetalHostApplicabilityRefusal)> {
        Measured::Unavailable(absent_apple_row())
    }

    /// Reports the direct path's measured half as unavailable.
    pub(super) fn run_direct() -> Measured<Vec<u32>> {
        Measured::Unavailable(absent_apple_row())
    }

    /// Reports the portfolio's measured half as unavailable.
    pub(super) fn run_portfolio(
        _bits: &[u32],
        _rows: u64,
        _columns: u64,
    ) -> Measured<Vec<AlternativeRun>> {
        Measured::Unavailable(absent_apple_row())
    }
}

/// Asks this host to earn the declared profile, or states why it cannot be
/// asked.
pub(crate) fn measured_offer() -> Measured<(String, MetalHostApplicabilityRefusal)> {
    apple::run_offer()
}

/// Runs the direct path's measured half, or states why this host cannot.
pub(crate) fn measured_direct() -> Measured<Vec<u32>> {
    apple::run_direct()
}

/// Runs the retained portfolio's measured half at one shape, or states why this
/// host cannot.
pub(crate) fn measured_portfolio(
    bits: &[u32],
    rows: u64,
    columns: u64,
) -> Measured<Vec<AlternativeRun>> {
    apple::run_portfolio(bits, rows, columns)
}

#[cfg(test)]
mod tests;
