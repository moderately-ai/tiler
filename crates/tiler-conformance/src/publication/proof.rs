//! The proof-case sidecar published beside each envelope.
//!
//! # Why the publishing half owns the operands and the expected bits
//!
//! The routed half could derive both for itself — it holds the semantic program
//! and the same reference oracle — and deriving them there would make the
//! comparison circular in the one way that matters: the bits a route is checked
//! against would be produced by the same reasoning that decided what to publish,
//! at the moment of checking, rather than recorded when the artifact was named.
//! Publishing them here makes the record a *statement about this artifact*, taken
//! before any device ran, that the route then reads back off disk through the
//! artifact layer's own association check.
//!
//! It is also what keeps the sidecar's arity, key, and length obligations live:
//! [`crate::envelope::case_operands`] refuses a case whose payload count, key
//! order, or payload length disagrees with the artifact's declared interface, and
//! those refusals can only be exercised against a record something actually
//! wrote.
//!
//! # One operand payload per declared input
//!
//! [`ProofCaseSpec`] takes one payload per artifact-declared input and the
//! builder places them into the artifact's own interface order, refusing a key
//! the artifact does not declare and a declared key left unsupplied. The
//! one-input reduction and the two-input contraction both go through it
//! unchanged; nothing here treats the first input specially.
//!
//! # Provenance
//!
//! The case tables, the probe stream, and the four pinned stream values below
//! came from `prototypes/serial-sum-compile`, which published this crate's
//! envelopes through `TILER_CONFORMANCE_ARTIFACT_BASE` until the routed half
//! moved into the gate. That prototype still publishes for
//! `prototypes/serial-sum-run` and is no longer this crate's input, so the two
//! copies are now independent rather than a pinned pair — which is why the
//! pinned-pair language that used to accompany them is gone and the checks that
//! remain are the ones that hold against a *measurement*: the probe stream
//! against the probe's own implementation, and the published cell against the
//! extents its retained digest was taken at.
//!
//! # The oracle is told the contract the packaged plan declares
//!
//! Every expected payload below is computed under
//! [`conformance_of`], which reads the two subnormal dimensions off the
//! *packaged kernels'* own [`NumericalRealization`] and the subject off the
//! plan's delivered-realization evidence. Until 2026-08-07 it went through
//! `ReferenceEvaluator::standard()`, which is
//! `under(registry, ReferenceNumericalConformance::strict())` — subnormals
//! preserved, and [`tiler_reference::ConformanceSubject::Unstated`], which
//! reaches every capability with nothing to check. The artifacts those bytes
//! travel with are compiled under
//! [`tiler_compiler::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`],
//! and [`NUMERICAL_IDENTITY`] has always *said* so, so the record claimed a
//! contract its own oracle was not evaluating.
//!
//! **It moved no published byte, and that is a property of the operands rather
//! than of the contract.** `tests::stating_the_packaged_contract_moves_no_published_expectation`
//! names and counts the corpus positions that hold a subnormal operand at all
//! and shows the two readings agreeing at every one of them; the L3 cells hold
//! none, because the probe's `m * 2^-24` stream makes every product and every
//! exact partial sum an integer multiple of `2^-72`, which is normal. A corpus
//! that grew a case those arguments do not cover would have compared a flushing
//! device against a preserving oracle, and that is the window this closes.

use tiler_artifact::program::VerifiedArtifactProgram;
use tiler_artifact::proof::{
    ProofBuildError, ProofCaseKey, ProofCaseSpec, ProofCodecError, ProofNumericalIdentity,
    ProofProvenance, ProofReferenceIdentity, ProofSidecarBuilder,
};
use tiler_compiler::session::PlanAlternative;
use tiler_ir::schedule::{ArithmeticType, NumericalRealization};
use tiler_ir::semantic::{F32, InputKey, OutputKey, SemanticProgram};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, InputBinding, ReferenceElement, ReferenceEvaluator,
    ReferenceNumericalConformance, Tensor, TensorPayloadView, UnsupportedReferenceContract,
};

use crate::envelope::{
    CONTRACTION_ACTIVATIONS_KEY, CONTRACTION_OUTPUT_KEY, CONTRACTION_WEIGHTS_KEY,
};
use crate::serial_sum::{INPUT_KEY, OUTPUT_KEY};

/// Governed key of the numerical contract the expected bytes are normative
/// under.
///
/// It names the flush-to-zero contract because that is the one this crate
/// publishes under; the strict contract is unhonourable on every governed Apple
/// family.
///
/// **It is now also the contract the bytes were computed under**, which it was
/// not while [`reference_bits`] evaluated the strict reading: this key said
/// flush-to-zero and the oracle preserved. [`conformance_of`] closes that by
/// deriving the oracle's contract from the packaged plan, and
/// `tests::the_published_oracle_carries_the_packaged_plans_own_contract` holds
/// the derived contract's two subnormal dimensions against [`super::CONTRACT`]'s.
///
/// **It is not the realization's own key and cannot be derived from one.** This
/// is a governed name in the sidecar's identity domain; a
/// [`NumericalRealization::profile_key`] is the compiler's structural contract
/// key, which encodes all eleven dimensions and the canonical NaN payload
/// (`tiler.contract.f32.v2.037fc0000001…` for this contract). The two name one
/// contract in two domains, nothing converts between them, and the test asserts
/// the packaged key against [`super::CONTRACT`]'s rather than against this one.
const NUMERICAL_IDENTITY: &[u8] = b"tiler.numerical.flush-subnormals-to-zero-f32";
/// Governed key of the implementation that produced the expected bytes.
const REFERENCE_IDENTITY: &[u8] = b"tiler.reference.standard-evaluator.v1";

/// The operand cases published for the serial sum, as `(key, one row pattern)`.
///
/// Each names the numerical class it exists to exercise, rather than a number
/// that happens to be interesting. A contract either holds at these values or is
/// decorative: a reduction that agrees on 1.0, 2.0, 3.0 and disagrees on a
/// non-canonical NaN payload has not been shown to agree.
///
/// The row is cycled to fill whatever shape the artifact declares, and the
/// interesting operand leads each row so a narrower reduction keeps it. That is
/// what makes the same case table meaningful at extent 0, 1, and 3.
const OPERAND_CASES: [(&str, [u32; 3]); 5] = [
    // The ordinary case, and the only one where a reordering defect is
    // invisible: these three sum to the same value in any order.
    ("ordinary", [0x3f80_0000, 0x4000_0000, 0x4040_0000]),
    // Signed zero beside the least positive subnormal. -0.0 + 0.0 is +0.0, so
    // the sign of the result is a statement about the reduction's identity
    // element rather than about the operands.
    (
        "signed-zero-and-subnormal",
        [0x8000_0000, 0x0000_0001, 0x3f80_0000],
    ),
    // A NaN with a non-canonical payload. Whether the payload survives is the
    // difference between propagating a NaN and minting a fresh one.
    ("non-canonical-nan", [0x7fc0_1234, 0x3f80_0000, 0x4000_0000]),
    // Infinity against a finite pair, so the result is infinite rather than a
    // large finite number, and +inf + -1.0 stays +inf.
    ("infinity", [0x7f80_0000, 0x3f80_0000, 0xbf80_0000]),
    // Contraction-sensitive: a large value, its negation, and a small one. A
    // fused multiply-add or a reassociated sum returns a different answer here
    // than a strictly serial one, which is the whole reason the contract is
    // stated. Serially this is (2^24 + -2^24) + 1.0 = 1.0; reassociated as
    // 2^24 + (-2^24 + 1.0) it is 0.0.
    (
        "contraction-sensitive",
        [0x4b80_0000, 0xcb80_0000, 0x3f80_0000],
    ),
];

/// The serial-sum operand-case population the published matrix consumes.
pub(crate) const fn serial_sum_case_count() -> usize {
    OPERAND_CASES.len()
}

/// The exact extents [`CONTRACTION_CASES`] is written for.
///
/// Stated as constants and checked rather than assumed, because the table is
/// literal `[[u32; K]; M]` and `[[u32; K]; N]` rows: moving the published
/// contraction to another shape while leaving this table alone would publish
/// operands for a program that was not compiled, and the sidecar builder would
/// only catch it if the element count happened to disagree.
const CONTRACTION_M: u64 = 2;
/// See [`CONTRACTION_M`].
const CONTRACTION_N: u64 = 2;
/// See [`CONTRACTION_M`].
const CONTRACTION_K: u64 = 3;

/// One contraction operand case: a stable key, the activations rows, and the
/// weights rows, each as `[M or N][K]` big-endian `f32` bit patterns.
///
/// Named rather than written inline because the tuple is the shape the case
/// table repeats five times, and a reader repairing one row needs to see which
/// literal is which operand.
///
/// The extents are literals rather than `CONTRACTION_M as usize` and friends,
/// because an array length is a `usize` and the constants are `u64` extents; the
/// cast is what the lint objects to and what [`cases_for`] makes unnecessary — it
/// refuses any shape but the one above, so the literals here and the constants
/// above are held together by a check rather than by a conversion.
type ContractionCase = (&'static str, [[u32; 3]; 2], [[u32; 3]; 2]);

/// The operand cases published for the contraction, as
/// `(key, activations rows, weights rows)`.
///
/// **Each case is the same numerical class the serial-sum table names, restated
/// at the two-operand site**, because that is where the contraction's own
/// obligations live: the contributor sequence is over *products* of two operands
/// rather than over one operand's elements, so a case that exercises a reduction
/// of stored values does not by itself exercise a reduction of computed ones.
///
/// `td,od->to`: `projected[t, o]` is the fold over `d` of
/// `activations[t, d] * weights[o, d]`, seeded from the first product and never
/// from `+0.0`.
const CONTRACTION_CASES: [ContractionCase; 5] = [
    // The ordinary case. Every product and every partial sum is exactly
    // representable, so this is the row where a disagreement means a wiring
    // defect rather than a rounding one:
    // [1,2,3]·[1,1,1] = 6, [1,2,3]·[2,2,2] = 12,
    // [4,5,6]·[1,1,1] = 15, [4,5,6]·[2,2,2] = 30.
    (
        "ordinary",
        [
            [0x3f80_0000, 0x4000_0000, 0x4040_0000], // 1.0, 2.0, 3.0
            [0x4080_0000, 0x40a0_0000, 0x40c0_0000], // 4.0, 5.0, 6.0
        ],
        [
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000], // 1.0, 1.0, 1.0
            [0x4000_0000, 0x4000_0000, 0x4000_0000], // 2.0, 2.0, 2.0
        ],
    ),
    // **The unseeded fold, made observable.** Every product of the first
    // activation row is `-0.0`, so the fold's result is `-0.0` if and only if it
    // is seeded from the first product. A kernel that seeds at `+0.0` returns
    // `0x0000_0000`, which is the exact counterexample the L3 record measured and
    // the reason the profile declares no seed.
    (
        "negative-zero-fold",
        [
            [0x8000_0000, 0x8000_0000, 0x8000_0000], // -0.0, -0.0, -0.0
            [0x3f80_0000, 0x0000_0001, 0xbf80_0000], // 1.0, least subnormal, -1.0
        ],
        [
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000], // 1.0, 1.0, 1.0
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000], // 1.0, 1.0, 1.0
        ],
    ),
    // A NaN with a non-canonical payload, entering through the activations
    // operand. Whether the payload survives the multiply and the fold is the
    // difference between propagating a NaN and minting a canonical one.
    (
        "non-canonical-nan",
        [
            [0x7fc0_1234, 0x3f80_0000, 0x4000_0000], // non-canonical NaN, 1.0, 2.0
            [0x3f80_0000, 0x4000_0000, 0x4040_0000], // 1.0, 2.0, 3.0
        ],
        [
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
        ],
    ),
    // Infinity in one operand against a finite other, so the product is infinite
    // and the fold stays infinite rather than becoming a large finite number.
    (
        "infinity",
        [
            [0x7f80_0000, 0x3f80_0000, 0xbf80_0000], // +inf, 1.0, -1.0
            [0x3f80_0000, 0x4000_0000, 0x4040_0000],
        ],
        [
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
        ],
    ),
    // **Contraction-sensitive, and the case the whole numerical contract is
    // for.** Serially the first output is `(2^24 + -2^24) + 1.0 = 1.0`;
    // reassociated as `2^24 + (-2^24 + 1.0)` it is `0.0`, and a fused
    // multiply-add reaches a third value. The L3 record attributes the `direct`
    // realization uniquely to the strict fold on exactly this distinction.
    (
        "contraction-sensitive",
        [
            [0x4b80_0000, 0xcb80_0000, 0x3f80_0000], // 2^24, -2^24, 1.0
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
        ],
        [
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
            [0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
        ],
    ),
];

/// The probe's workload seed, `contraction_probe.py`'s `WORKLOAD_SEED`.
const WORKLOAD_SEED: u64 = 0x5445_524D;
/// The probe's right-operand seed derivation, `host.m`'s `fill_prng` call.
const RIGHT_SEED_MASK: u64 = 0xA5A5_A5A5_A5A5_A5A5;
/// The stable case key of the one operand set the probe measured, per cell.
///
/// One case rather than five: the retained digest is a measurement of that exact
/// workload, and an adversarial operand class published beside it would carry no
/// retained value to be compared against. The five adversarial classes stay on
/// the `2x2x3` member, which is what that member is for.
const L3_CELL_CASE_KEY: &str = "probe-workload";

/// Which program family a published member carries.
///
/// The sidecar's operand and expectation shapes follow from the family, so it is
/// stated once — on the member being published — rather than rebuilt at each
/// call site.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProofFamily {
    /// `sum((input * 1.0) + 0.0)` over the reduced axis of a `[rows, columns]`
    /// input.
    SerialSum {
        /// Rows of the declared input; each reduces to one output element.
        rows: u64,
        /// Columns of the declared input; the reduced axis.
        columns: u64,
    },
    /// The L3 profile's index structure `td,od->to` over `[m, k]` activations and
    /// `[n, k]` weights, publishing `[m, n]`.
    Contraction {
        /// Rows of the activations operand and of the result.
        m: u64,
        /// Rows of the weights operand and columns of the result.
        n: u64,
        /// The contracted extent, shared by both operands.
        k: u64,
    },
    /// The same index structure over the L3 realization probe's **own** workload
    /// operands, at a cell for which a `result_sha256` was retained.
    ///
    /// A separate variant rather than a shape of [`ProofFamily::Contraction`],
    /// because the operands come from somewhere else and that is the whole point
    /// of the member: [`ProofFamily::Contraction`]'s five cases are adversarial
    /// numerical classes chosen here, and this one's single case is a
    /// pseudorandom stream that must be reproduced byte for byte or the retained
    /// digest is a comparison against unrelated bits.
    L3CorrectnessCell {
        /// Rows of the activations operand and of the result.
        m: u64,
        /// Rows of the weights operand and columns of the result.
        n: u64,
        /// The contracted extent, shared by both operands.
        k: u64,
    },
}

impl ProofFamily {
    /// The extents a contraction family names, for the members that have them.
    ///
    /// Returned rather than matched at each call site because every published
    /// contraction member carries contraction extents and the routed half asserts
    /// them against the artifact's own declaration; a second match would be a
    /// second place for the two to disagree.
    pub(crate) const fn contraction_extents(self) -> Option<(u64, u64, u64)> {
        match self {
            Self::SerialSum { .. } => None,
            Self::Contraction { m, n, k } | Self::L3CorrectnessCell { m, n, k } => Some((m, n, k)),
        }
    }

    /// The iteration-step allowance the oracle is asked to fold this family
    /// under.
    ///
    /// **A stated number, and the statement is the authorization.** The reference
    /// holds one occurrence to
    /// [`crate::envelope::REFERENCE_DEFAULT_STEP_ALLOWANCE`] by default, and four
    /// of the six L3 correctness cells fold past it — the largest at 402,653,184
    /// multiply-accumulate steps. A caller that has decided to pay for a larger
    /// fold says so in visible code, which is exactly what this is; what it never
    /// does is widen what *one* walk may cost, because the evaluator windows an
    /// occurrence above the bound into folds each passing the test a
    /// single-window fold passes.
    ///
    /// Written as a maximum rather than as the fold itself, so a family under the
    /// default keeps the ordinary evaluator's own number and this returns an
    /// authorization only where one is needed. The two cells the ordinary gate
    /// routes are on that side of the line, which is
    /// `crate::envelope::L3CorrectnessCell::folds_under_the_default_allowance`.
    fn iteration_step_allowance(self) -> usize {
        let default = usize::try_from(crate::envelope::REFERENCE_DEFAULT_STEP_ALLOWANCE)
            .expect("the reference's default allowance fits a usize");
        match self {
            Self::SerialSum { .. } | Self::Contraction { .. } => default,
            Self::L3CorrectnessCell { m, n, k } => m
                .checked_mul(n)
                .and_then(|outputs| outputs.checked_mul(k))
                .and_then(|steps| usize::try_from(steps).ok())
                .map_or(default, |steps| steps.max(default)),
        }
    }
}

/// Why the oracle's numerical contract could not be derived from a packaged
/// plan.
///
/// Its own vocabulary rather than a `String`, because the four cases are four
/// different things to do next: a plan packaging nothing, two packaged kernels
/// declaring different realizations, a delivered realization naming other than
/// one scalar subject, and a realization the reference cannot answer for at all.
/// Only the last is [`tiler_reference`]'s refusal; the first three are this
/// route's own preconditions for having a realization and a subject to hand it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnderivableOracleContract {
    /// The plan packages no kernel, so nothing declares a realization.
    NoPackagedKernel,
    /// Two packaged kernels declare different numerical realizations.
    ///
    /// Refused rather than resolved by taking the first: the sidecar states one
    /// contract for the whole member, and a plan whose stages disagree has no
    /// single contract for it to state. Nothing this build packages disagrees, so
    /// this has not been observed firing; what *is* exercised is the comparison,
    /// on the two-stage serial-sum role — one publication, two kernels — which is
    /// why taking the first kernel and calling it the member's contract would be
    /// an unchecked assumption rather than a shortcut.
    DisagreeingRealizations {
        /// The realization the first packaged kernel declares.
        first: NumericalRealization,
        /// The first realization that disagrees with it.
        other: NumericalRealization,
    },
    /// The delivered realization named other than exactly one scalar subject.
    ///
    /// ADR 0076's evidence carries one subject per selected scalar contract, so
    /// a plan reaching here with another count is one whose contract this route
    /// cannot name a single arithmetic type for — and stating one anyway is
    /// precisely the assertion [`ReferenceNumericalConformance::from_realization`]
    /// exists to refuse.
    AmbiguousSubject {
        /// How many subjects the delivered realization stated.
        stated: usize,
    },
    /// The reference refused the realization or the subject stated for it.
    Unsupported(UnsupportedReferenceContract),
}

impl std::fmt::Display for UnderivableOracleContract {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPackagedKernel => formatter.write_str(
                "the published plan packages no kernel, so no packaged stage declares the \
                 numerical realization the oracle would be evaluated under",
            ),
            Self::DisagreeingRealizations { first, other } => write!(
                formatter,
                "two packaged kernels declare different numerical realizations ({first:?} and \
                 {other:?}), so the sidecar has no one contract to state for the member",
            ),
            Self::AmbiguousSubject { stated } => write!(
                formatter,
                "the delivered realization states {stated} scalar-arithmetic subject(s) and the \
                 oracle's conformance is resolved for exactly one",
            ),
            Self::Unsupported(cause) => write!(
                formatter,
                "the reference refused the packaged plan's declared contract: {cause}",
            ),
        }
    }
}

impl std::error::Error for UnderivableOracleContract {}

/// Why a proof-case sidecar could not be published.
#[derive(Debug)]
pub(crate) enum SidecarFailure {
    /// The draft refused the artifact, the provenance, or the case.
    Build(ProofBuildError),
    /// The verified record did not encode.
    Encode(ProofCodecError),
    /// The oracle's contract could not be derived from the published plan.
    ///
    /// A refusal here stops the publication rather than falling back to the
    /// strict reading: an expected payload computed under a contract nobody
    /// declared is exactly the record this member must not write.
    Oracle(UnderivableOracleContract),
    /// A published contraction shape has no operand table written for it.
    ///
    /// Its own class rather than a panic: [`CONTRACTION_CASES`] is literal rows,
    /// so moving the published contraction to another shape must move the table
    /// with it, and the refusal names both shapes so a reader sees which one to
    /// change.
    UnwrittenContractionShape {
        /// The extents the publication asked for.
        requested: (u64, u64, u64),
        /// The extents the case table is written for.
        written: (u64, u64, u64),
    },
    /// A published L3 cell is not one a retained `result_sha256` describes.
    ///
    /// Its own class, and a stricter one than [`Self::UnwrittenContractionShape`]:
    /// that refusal says an operand table would have to be written, and this one
    /// says a *measurement* would have to be taken. The probe's stream is defined
    /// at every shape, so generating operands for another cell would succeed and
    /// publish a member whose expected bytes no retained digest describes.
    UnretainedProbeCell {
        /// The extents the publication asked for.
        requested: (u64, u64, u64),
        /// The extents a retained digest exists for, in the record's own order.
        retained: Vec<(u64, u64, u64)>,
    },
}

impl std::fmt::Display for SidecarFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Build(cause) => write!(formatter, "the proof sidecar draft refused: {cause:?}"),
            Self::Encode(cause) => write!(formatter, "the proof sidecar did not encode: {cause:?}"),
            Self::Oracle(cause) => write!(
                formatter,
                "the proof sidecar's expected bytes have no stated contract: {cause}",
            ),
            Self::UnwrittenContractionShape {
                requested: (m, n, k),
                written: (wm, wn, wk),
            } => write!(
                formatter,
                "a {m}x{n}x{k} contraction is published and this module's operand table is written \
                 for {wm}x{wn}x{wk}; move the table with the shape",
            ),
            Self::UnretainedProbeCell {
                requested: (m, n, k),
                retained,
            } => write!(
                formatter,
                "a {m}x{n}x{k} L3 cell is published and the retained realization-probe digests it \
                 would be compared against were measured at {}; a cell with no retained \
                 measurement cannot be published through this family",
                retained
                    .iter()
                    .map(|(rm, rn, rk)| format!("{rm}x{rn}x{rk}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}

impl std::error::Error for SidecarFailure {}

/// One named operand a case supplies, with the shape it is read at.
struct Operand {
    /// The interface key the artifact declares this operand under.
    key: InputKey,
    /// The shape it is read at.
    shape: Shape,
    /// Its `f32` bit patterns, in row-major order.
    bits: Vec<u32>,
}

/// Fills one `rows` by `columns` input by cycling one operand row.
///
/// Cycling rather than indexing, so a case's row defines an input for any shape
/// an artifact might declare — including the empty domain, where this correctly
/// produces no elements at all.
fn input_bits(pattern: [u32; 3], rows: u64, columns: u64) -> Vec<u32> {
    let mut bits = Vec::new();
    for _ in 0..rows {
        for column in 0..columns {
            bits.push(pattern[usize::try_from(column % 3).expect("a bounded column index")]);
        }
    }
    bits
}

/// Flattens a literal row table into one dense operand.
fn rows_of(table: [[u32; 3]; 2]) -> Vec<u32> {
    table.into_iter().flatten().collect()
}

/// The probe's `SplitMix64` finalizer, transcribed rather than approximated.
fn splitmix64(x: u64) -> u64 {
    let x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let z = x;
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The probe's operand value at one index: `m * 2^-24` with `m` an integer in
/// `[-2^23, 2^23)`.
///
/// Every such value is exactly representable in binary32, which is the property
/// the probe chose it for: the operands themselves introduce no rounding, so any
/// difference a comparison reports is a difference in how the contraction was
/// evaluated rather than in how its inputs were written down.
fn probe_value(seed: u64, index: u64) -> f32 {
    let bits = splitmix64(seed.wrapping_add(index.wrapping_mul(0x2545_F491_4F6C_DD1D)));
    let field =
        i64::from(u32::try_from((bits >> 40) & 0xFF_FFFF).expect("a 24-bit field fits in u32"));
    #[expect(
        clippy::cast_precision_loss,
        reason = "an integer in [-2^23, 2^23) is exactly representable in binary32"
    )]
    let magnitude = (field - 8_388_608) as f32;
    magnitude * (1.0 / 16_777_216.0)
}

/// Generates one dense operand from the probe's stream, in row-major index order.
///
/// The index is the flat row-major position, which is what makes this the probe's
/// operand rather than a permutation of it: the probe fills a `[rows, columns]`
/// buffer linearly and the device reads it the same way, so an index derived from
/// anything but the flat position would produce the same multiset of values and a
/// different tensor.
fn probe_bits(elements: u64, seed: u64) -> Vec<u32> {
    (0..elements)
        .map(|index| probe_value(seed, index).to_bits())
        .collect()
}

/// Builds one reference tensor from big-endian `f32` bit patterns.
fn tensor(shape: &Shape, bits: &[u32]) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        shape.clone(),
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

/// The numerical realization every packaged kernel of one plan declares.
///
/// **Read off the packaged kernels rather than off the contract this module
/// compiles under.** [`VerifiedKernel::numerical`](tiler_ir::kernel::VerifiedKernel::numerical)
/// hands back the realization the scheduled region the kernel refines recorded,
/// so this is the plan's own choice at the sites its contract left free — not a
/// second transcription of `super::CONTRACT` that a change to planning could
/// leave stale.
///
/// Every kernel is examined rather than the first taken, because a member is one
/// sidecar over one program: the two-stage serial-sum role packages two kernels
/// and a member whose stages disagreed would have no single contract to publish.
fn packaged_realization(
    plan: PlanAlternative<'_>,
) -> Result<NumericalRealization, UnderivableOracleContract> {
    let mut packaged = plan.kernels().iter();
    let first = packaged
        .next()
        .ok_or(UnderivableOracleContract::NoPackagedKernel)?
        .numerical();
    for kernel in packaged {
        let other = kernel.numerical();
        if other != first {
            return Err(UnderivableOracleContract::DisagreeingRealizations { first, other });
        }
    }
    Ok(first)
}

/// The arithmetic type one plan's delivered-realization evidence is stated for.
///
/// **This is where the subject comes from on a plan-derived route.** The BF16
/// vertical reads its subject off a [`RealizationWitness`](tiler_ir::schedule::RealizationWitness),
/// which needs a `VerifiedScheduledRegion`; a publication is handed a plan and
/// holds no region, so that route is unavailable here. What a plan does hold is
/// ADR 0076's delivered-realization evidence, whose subject table is materialized
/// from the *selected contract's* own arithmetic type — one row per selected
/// scalar contract — and [`ScalarArithmeticSubject::arithmetic`](tiler_ir::numerics::ScalarArithmeticSubject::arithmetic)
/// is that type. So the subject is compiler-minted evidence about the plan
/// rather than a constant this file writes down beside it.
///
/// The count is checked rather than assumed: the surface is an
/// `ExactSizeIterator`, and a plan naming two subjects is one this route cannot
/// resolve a single conformance for.
fn packaged_subject(
    plan: PlanAlternative<'_>,
) -> Result<ArithmeticType, UnderivableOracleContract> {
    let mut subjects = plan.delivered_realization().scalar_arithmetic();
    let stated = subjects.len();
    if stated != 1 {
        return Err(UnderivableOracleContract::AmbiguousSubject { stated });
    }
    let subject = subjects
        .next()
        .ok_or(UnderivableOracleContract::AmbiguousSubject { stated })?;
    Ok(subject.subject().arithmetic())
}

/// Derives the oracle's numerical contract from the plan a member packages.
///
/// The checked bridge this route was missing. Both arguments
/// [`ReferenceNumericalConformance::from_realization`] takes come from the same
/// plan — the realization from its packaged kernels and the subject from its
/// delivered-realization evidence — and the bridge cross-checks them against each
/// other through the realization's own declared canonical arithmetic NaN
/// payload, so a subject drawn from somewhere other than the plan that declared
/// the realization is refused rather than carried.
///
/// # Errors
///
/// Returns [`UnderivableOracleContract`] naming which of the four boundaries
/// refused.
fn conformance_of(
    plan: PlanAlternative<'_>,
) -> Result<ReferenceNumericalConformance, UnderivableOracleContract> {
    conformance_stated_for(plan, packaged_subject(plan)?)
}

/// [`conformance_of`] with the subject supplied rather than derived.
///
/// Split out so the refusal can be watched *firing on this route*: a test states
/// a subject the packaged realization contradicts and reads back the bridge's own
/// answer, which is a stronger claim than calling
/// [`ReferenceNumericalConformance::from_realization`] beside the route and
/// checking that it refuses. The publishing path never calls this directly —
/// [`conformance_of`] is its one entry point, and the subject it passes is the
/// plan's.
///
/// # Errors
///
/// As [`conformance_of`], less the subject-count boundary.
fn conformance_stated_for(
    plan: PlanAlternative<'_>,
    arithmetic: ArithmeticType,
) -> Result<ReferenceNumericalConformance, UnderivableOracleContract> {
    let realization = packaged_realization(plan)?;
    ReferenceNumericalConformance::from_realization(&realization, arithmetic)
        .map_err(UnderivableOracleContract::Unsupported)
}

/// Evaluates the program under the governed reference to get expected outputs.
///
/// **Every declared operand is bound, not the first one.** The evaluator takes
/// the whole binding set, so a family with two inputs supplies two bindings and
/// the oracle evaluates the program the artifact actually declares. A version of
/// this that bound only the leading operand would have evaluated a different
/// program and reported its bits as normative.
///
/// **The allowance is the caller's and is stated rather than defaulted.** Four of
/// the six L3 correctness cells fold past the reference's own per-occurrence
/// bound, and the number that authorizes them comes from
/// [`ProofFamily::iteration_step_allowance`] — visible caller code — rather than
/// from a constant nobody re-derives. A family under the bound is handed the
/// evaluator's own default, so publishing it authorizes nothing.
///
/// **The contract is the caller's too, and is derived rather than defaulted.**
/// `ReferenceEvaluator::standard()` is `under(registry, strict())`, which
/// preserves subnormals and states no subject; the registry here is that same
/// governed snapshot with the packaged plan's own contract in place of the strict
/// one, so the oracle answers the question the device half was compiled to
/// answer. [`conformance_of`] is where the contract comes from.
fn reference_bits(
    program: &SemanticProgram,
    operands: &[Operand],
    iteration_step_allowance: usize,
    conformance: ReferenceNumericalConformance,
) -> Vec<u32> {
    let tensors: Vec<Tensor> = operands
        .iter()
        .map(|operand| tensor(&operand.shape, &operand.bits))
        .collect();
    let bindings: Vec<InputBinding<'_>> = operands
        .iter()
        .zip(&tensors)
        .map(|(operand, tensor)| InputBinding::new(&operand.key, tensor))
        .collect();
    let outputs = ReferenceEvaluator::under(
        FrozenReferenceRegistry::standard().expect("the governed reference profile composes"),
        conformance,
    )
    .with_iteration_step_allowance(iteration_step_allowance)
    .evaluate(program, &bindings)
    .expect("the reference evaluates the program");
    match outputs[0].payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| {
                u32::from_be_bytes(
                    <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
                )
            })
            .collect(),
        _ => panic!("expected a dense f32 reference output"),
    }
}

/// Encodes big-endian `f32` bit patterns as the sidecar's payload bytes.
///
/// Most-significant byte first throughout, matching the order the reference
/// elements were built from, so a payload never depends on host endianness.
fn payload_bytes(bits: &[u32]) -> Vec<u8> {
    bits.iter().flat_map(|value| value.to_be_bytes()).collect()
}

/// Returns this family's named cases, each as its full operand set.
///
/// # Errors
///
/// Returns [`SidecarFailure::UnwrittenContractionShape`] for a contraction shape
/// with no operand table and [`SidecarFailure::UnretainedProbeCell`] for an L3
/// cell with no retained digest.
fn cases_for(family: ProofFamily) -> Result<Vec<(&'static str, Vec<Operand>)>, SidecarFailure> {
    match family {
        ProofFamily::SerialSum { rows, columns } => Ok(OPERAND_CASES
            .into_iter()
            .map(|(key, pattern)| {
                (
                    key,
                    vec![Operand {
                        key: InputKey::new(INPUT_KEY).expect("the input key is valid"),
                        shape: Shape::from_dims([rows, columns]),
                        bits: input_bits(pattern, rows, columns),
                    }],
                )
            })
            .collect()),
        ProofFamily::Contraction { m, n, k } => {
            if (m, n, k) != (CONTRACTION_M, CONTRACTION_N, CONTRACTION_K) {
                return Err(SidecarFailure::UnwrittenContractionShape {
                    requested: (m, n, k),
                    written: (CONTRACTION_M, CONTRACTION_N, CONTRACTION_K),
                });
            }
            Ok(CONTRACTION_CASES
                .into_iter()
                .map(|(key, activations, weights)| {
                    (
                        key,
                        vec![
                            Operand {
                                key: InputKey::new(CONTRACTION_ACTIVATIONS_KEY)
                                    .expect("the activations key is valid"),
                                shape: Shape::from_dims([m, k]),
                                bits: rows_of(activations),
                            },
                            Operand {
                                key: InputKey::new(CONTRACTION_WEIGHTS_KEY)
                                    .expect("the weights key is valid"),
                                shape: Shape::from_dims([n, k]),
                                bits: rows_of(weights),
                            },
                        ],
                    )
                })
                .collect())
        }
        ProofFamily::L3CorrectnessCell { m, n, k } => {
            // Membership in the retained set rather than equality with one cell:
            // the probe's stream is defined at every shape, so the refusal has to
            // come from whether a *measurement* exists rather than from whether
            // operands can be generated.
            if !crate::envelope::L3_CORRECTNESS_CELLS
                .iter()
                .any(|cell| cell.extents() == (m, n, k))
            {
                return Err(SidecarFailure::UnretainedProbeCell {
                    requested: (m, n, k),
                    retained: crate::envelope::L3_CORRECTNESS_CELLS
                        .iter()
                        .map(crate::envelope::L3CorrectnessCell::extents)
                        .collect(),
                });
            }
            // The right operand's seed is the workload seed masked, exactly as
            // the probe's host derives it. Seeding both operands identically
            // would make `activations` a prefix of `weights` and the contraction
            // a self-inner-product, which is a different measurement.
            Ok(vec![(
                L3_CELL_CASE_KEY,
                vec![
                    Operand {
                        key: InputKey::new(CONTRACTION_ACTIVATIONS_KEY)
                            .expect("the activations key is valid"),
                        shape: Shape::from_dims([m, k]),
                        bits: probe_bits(m * k, WORKLOAD_SEED),
                    },
                    Operand {
                        key: InputKey::new(CONTRACTION_WEIGHTS_KEY)
                            .expect("the weights key is valid"),
                        shape: Shape::from_dims([n, k]),
                        bits: probe_bits(n * k, WORKLOAD_SEED ^ RIGHT_SEED_MASK),
                    },
                ],
            )])
        }
    }
}

/// Returns the output key this family publishes under.
fn output_key_for(family: ProofFamily) -> OutputKey {
    let key = match family {
        ProofFamily::SerialSum { .. } => OUTPUT_KEY,
        ProofFamily::Contraction { .. } | ProofFamily::L3CorrectnessCell { .. } => {
            CONTRACTION_OUTPUT_KEY
        }
    };
    OutputKey::new(key).expect("the output key is valid")
}

/// Builds and encodes the proof-case sidecar for one published artifact.
///
/// The plan is taken beside the artifact because the expected payloads are
/// normative under a contract, and the plan is what declares it:
/// [`conformance_of`] reads the realization and the subject off it, so the
/// oracle cannot be told a contract the packaged program does not carry.
///
/// # Errors
///
/// Returns [`SidecarFailure`] naming the boundary that refused. Nothing is worked
/// around: a rejection here means the record would not have described the
/// artifact it travels with.
pub(crate) fn encoded(
    artifact: &VerifiedArtifactProgram,
    program: &SemanticProgram,
    family: ProofFamily,
    plan: PlanAlternative<'_>,
) -> Result<Vec<u8>, SidecarFailure> {
    let conformance = conformance_of(plan).map_err(SidecarFailure::Oracle)?;
    let mut draft = ProofSidecarBuilder::new(
        artifact,
        ProofProvenance {
            semantic_graph: program.semantic_identity().graph().clone(),
            numerical: ProofNumericalIdentity::from_bytes(NUMERICAL_IDENTITY)
                .expect("the governed numerical key is in bounds"),
            reference: ProofReferenceIdentity::from_bytes(REFERENCE_IDENTITY)
                .expect("the governed reference key is in bounds"),
        },
    )
    .map_err(SidecarFailure::Build)?;

    let output_key = output_key_for(family);
    // Every case over the same program, so the routed half compares one artifact
    // against several operand classes rather than needing an artifact each.
    let allowance = family.iteration_step_allowance();
    for (key, operands) in cases_for(family)? {
        let expected = reference_bits(program, &operands, allowance, conformance);
        draft
            .push_case(ProofCaseSpec {
                key: ProofCaseKey::new(key).expect("the case key is valid"),
                inputs: operands
                    .iter()
                    .map(|operand| (operand.key.clone(), payload_bytes(&operand.bits)))
                    .collect(),
                expected: vec![(output_key.clone(), payload_bytes(&expected))],
            })
            .map_err(SidecarFailure::Build)?;
    }

    draft
        .build()
        .map_err(SidecarFailure::Build)?
        .encode()
        .map_err(SidecarFailure::Encode)
}

#[cfg(test)]
mod tests {
    use tiler_build::BoundMetalCompileDeclaration;
    use tiler_compiler::session::Compilation;
    use tiler_ir::schedule::ArithmeticType;
    use tiler_ir::semantic::{
        CANONICAL_BF16_ARITHMETIC_NAN_BITS, CANONICAL_F32_ARITHMETIC_NAN_BITS, InputKey,
        SemanticProgram,
    };
    use tiler_ir::shape::Shape;
    use tiler_reference::{
        ConformanceSubject, ReferenceEvaluator, ReferenceNumericalConformance,
        UnsupportedReferenceContract,
    };

    use super::{
        CONTRACTION_K, CONTRACTION_M, CONTRACTION_N, NUMERICAL_IDENTITY, Operand, ProofFamily,
        RIGHT_SEED_MASK, UnderivableOracleContract, WORKLOAD_SEED, cases_for, conformance_of,
        conformance_stated_for, packaged_realization, probe_bits, probe_value, reference_bits,
    };
    use crate::envelope::{
        CONTRACTION_MEMBERS, L3_CORRECTNESS_CELLS, REDUCTION_CLASSES,
        REFERENCE_DEFAULT_STEP_ALLOWANCE, contraction_program,
    };
    use crate::publication::{CONTRACT, PUBLISHED_ROWS};
    use crate::serial_sum::{INPUT_KEY, compile_under, serial_sum_program};

    /// The extents of the cell whose retained digest this crate first routed.
    ///
    /// Kept as a literal so the two tests below that need *one* cell name it
    /// rather than indexing the table they are checking.
    const DECODE_KV: (u64, u64, u64) = (1, 1024, 1024);

    /// Compiles one published program exactly as [`super::super::publish_member`]
    /// receives it.
    ///
    /// The real `compile()` under the real [`CONTRACT`], against the
    /// authoritative macOS Apple9 declaration — none of which needs a device or
    /// the offline Apple toolchain, which is what lets the three tests below run
    /// on every host. Publishing the result would need both; deriving the
    /// oracle's contract from it needs neither, and that derivation is what these
    /// check.
    ///
    /// The [`Compilation`] is returned rather than a `PlanAlternative`, because
    /// the alternative borrows it.
    fn published_compilation(program: &SemanticProgram) -> Compilation {
        let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
            .expect("the authoritative declaration assembles");
        compile_under(&declaration, program, CONTRACT)
            .expect("a published member's program compiles under the published contract")
    }

    /// Every published member's `(family, program)` pair, less the L3 cells.
    ///
    /// The L3 cells are excluded because their operand streams run to
    /// `3072x1024` and each expected payload costs a reference fold of up to
    /// 1,094,713,344 steps to state, which is the cost the ordinary gate already
    /// declines. What they contribute to the argument below is covered instead by
    /// the probe stream's own property, stated in this module's header: every
    /// operand is `m * 2^-24`, so every product and every exact partial sum is an
    /// integer multiple of `2^-72` and no subnormal can arise for either reading
    /// to disagree about.
    fn adversarial_members() -> Vec<(ProofFamily, SemanticProgram)> {
        let mut members: Vec<(ProofFamily, SemanticProgram)> = REDUCTION_CLASSES
            .into_iter()
            .map(|(_, columns)| {
                (
                    ProofFamily::SerialSum {
                        rows: PUBLISHED_ROWS,
                        columns,
                    },
                    serial_sum_program(PUBLISHED_ROWS, columns),
                )
            })
            .collect();
        members.push((
            ProofFamily::Contraction {
                m: CONTRACTION_M,
                n: CONTRACTION_N,
                k: CONTRACTION_K,
            },
            contraction_program(CONTRACTION_M, CONTRACTION_N, CONTRACTION_K),
        ));
        members
    }

    /// The oracle's contract is the packaged plan's own, carried with its subject.
    ///
    /// **What this route was missing, and the shape of what replaced it.** Every
    /// expected payload used to be computed through `ReferenceEvaluator::standard()`
    /// — `under(registry, strict())` — so the oracle preserved subnormals and
    /// carried [`ConformanceSubject::Unstated`], which reaches every capability
    /// with nothing to check, while the artifact beside it was compiled under
    /// [`CONTRACT`]. This asserts the three things that changed: the subject is
    /// the plan's `f32`, both subnormal dimensions are the contract's flush, and
    /// the realization those came off is the one the packaged kernels declare —
    /// [`CONTRACT`]'s structural key — rather than a value this module wrote down
    /// beside them.
    ///
    /// [`NUMERICAL_IDENTITY`] is deliberately not compared against that key. It
    /// is a governed name in the sidecar's own identity domain and the profile
    /// key is the compiler's structural one; they name one contract in two
    /// domains and nothing converts between them, so what the sidecar's claim can
    /// be held to here is the *dimensions*, which is what the assertions above
    /// do.
    ///
    /// The hand-stated transcription is constructed here so the difference is
    /// exhibited rather than described: equal on both dimensions every applier
    /// reads, and speaking about no value set at all.
    #[test]
    fn the_published_oracle_carries_the_packaged_plans_own_contract() {
        let program = contraction_program(CONTRACTION_M, CONTRACTION_N, CONTRACTION_K);
        let compilation = published_compilation(&program);
        let plan = compilation
            .selected()
            .expect("the portfolio retains a selected plan");

        let conformance = conformance_of(plan).expect("the packaged f32 realization bridges");
        assert_eq!(
            conformance.subject(),
            ConformanceSubject::Arithmetic(ArithmeticType::F32),
            "the oracle was handed a conformance naming no format, or another one",
        );
        assert_eq!(conformance.input_subnormals(), CONTRACT.input_subnormals());
        assert_eq!(
            conformance.result_subnormals(),
            CONTRACT.result_subnormals()
        );

        let transcribed = ReferenceNumericalConformance::new(
            CONTRACT.input_subnormals(),
            CONTRACT.result_subnormals(),
        );
        assert_eq!(transcribed.subject(), ConformanceSubject::Unstated);
        assert_ne!(
            transcribed, conformance,
            "the derived conformance must differ from the hand-stated one, or the subject the \
             capability checks is not being carried",
        );
        assert_ne!(
            ReferenceNumericalConformance::strict(),
            conformance,
            "the oracle is still evaluating the strict reading the artifact does not deliver",
        );

        // The realization those two dimensions were read off is the packaged
        // kernels' own, and it is this module's declared contract rather than a
        // realization assembled beside the plan.
        let realization = packaged_realization(plan).expect("the packaged kernels agree");
        assert_eq!(realization.profile_key, CONTRACT.key());
        assert_ne!(
            realization.profile_key.as_bytes(),
            NUMERICAL_IDENTITY,
            "the sidecar's governed numerical key and the compiler's structural contract key are \
             separate identity domains; a build in which they coincided would make the comment on \
             NUMERICAL_IDENTITY wrong rather than this assertion",
        );
        assert_eq!(
            realization.canonical_arithmetic_nan_bits, CANONICAL_F32_ARITHMETIC_NAN_BITS,
            "the packaged realization declares another width's canonical arithmetic NaN",
        );
    }

    /// A subject the packaged realization contradicts is refused, watched firing
    /// on this route.
    ///
    /// **The perturbation that shows this route is checked rather than merely
    /// called.** [`conformance_of`] is the only way this module obtains an oracle
    /// contract, and the check that makes its derived subject an *agreement*
    /// rather than an assertion lives in the bridge: the realization carries the
    /// canonical arithmetic NaN pattern of the region's own type, so a subject
    /// drawn from anywhere but the plan that declared it is refused. Perturbing
    /// the subject is what exercises that; perturbing an assertion would not.
    ///
    /// Both refusal classes are walked, because they answer different questions.
    /// `Bf16` is a format this reference *does* evaluate, so its rejection is the
    /// declaration cross-check firing; `F16` and `F64` are formats it evaluates
    /// nothing in, so theirs is the evaluability boundary. A bridge that only had
    /// the second would still refuse `Bf16` here and would be checking nothing
    /// about the declaration.
    #[test]
    fn a_subject_the_packaged_realization_contradicts_is_refused_on_this_route() {
        let program = contraction_program(CONTRACTION_M, CONTRACTION_N, CONTRACTION_K);
        let compilation = published_compilation(&program);
        let plan = compilation
            .selected()
            .expect("the portfolio retains a selected plan");

        assert_eq!(
            conformance_stated_for(plan, ArithmeticType::Bf16),
            Err(UnderivableOracleContract::Unsupported(
                UnsupportedReferenceContract::DeclaredNanPayloadMismatch {
                    arithmetic: ArithmeticType::Bf16,
                    declared: CANONICAL_F32_ARITHMETIC_NAN_BITS,
                    expected: u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
                },
            )),
            "an f32 plan accepted a bf16 subject, so the oracle's subject is asserted rather than \
             agreed",
        );

        let mut refused = 0_usize;
        for arithmetic in [ArithmeticType::F16, ArithmeticType::F64] {
            assert_eq!(
                conformance_stated_for(plan, arithmetic),
                Err(UnderivableOracleContract::Unsupported(
                    UnsupportedReferenceContract::ArithmeticNotEvaluable { arithmetic },
                )),
            );
            refused += 1;
        }
        assert_eq!(refused, 2, "both unevaluable formats were exercised");

        // And the plan's own subject bridges on the same call, so the refusals
        // above are decisions about the subject rather than a route that never
        // succeeds.
        assert_eq!(
            conformance_stated_for(plan, ArithmeticType::F32),
            conformance_of(plan),
        );
        assert!(conformance_of(plan).is_ok());
    }

    /// Telling the oracle the packaged contract moves no published expectation,
    /// over a counted subnormal population.
    ///
    /// **The agreement is a property of the operands, and this is what names
    /// them.** Flushing and preserving differ only at a subnormal, so a corpus
    /// holding none would satisfy this test for a reason that says nothing about
    /// the contract. The positions that hold one are therefore counted rather
    /// than assumed: two, the `signed-zero-and-subnormal` serial-sum case at the
    /// `nontrivial` extent and the `negative-zero-fold` contraction case's least
    /// positive subnormal, both `0x00000001`. A corpus that lost them turns this
    /// red instead of leaving a vacuous pass, and a corpus that grew a case the
    /// two readings disagree on turns it red for the opposite reason — which is
    /// the outcome that would mean the published bytes had genuinely moved and
    /// the retained digests they are compared against need re-measuring.
    ///
    /// **The last section is the one that watches the contract reach the
    /// oracle.** Everything above is satisfied by a [`reference_bits`] that
    /// accepted the conformance and dropped it, because both readings would then
    /// be the same reading. So it is closed by an operand the two readings
    /// genuinely disagree on — the least positive subnormal, alone, through the
    /// singleton reduction — where the declared contract must return `0x00000000`
    /// and the strict one `0x00000001`. Reverting the evaluator to
    /// `ReferenceEvaluator::standard()` fails exactly there and nowhere else.
    #[test]
    fn stating_the_packaged_contract_moves_no_published_expectation() {
        let strict = ReferenceNumericalConformance::strict();
        let mut subnormal_operands = 0_usize;
        let mut compared = 0_usize;

        for (family, program) in adversarial_members() {
            let compilation = published_compilation(&program);
            let plan = compilation
                .selected()
                .expect("the portfolio retains a selected plan");
            let declared = conformance_of(plan).expect("the packaged f32 realization bridges");
            let allowance = family.iteration_step_allowance();

            for (key, operands) in cases_for(family).expect("the published family has a case table")
            {
                subnormal_operands += operands
                    .iter()
                    .flat_map(|operand| operand.bits.iter())
                    .filter(|bits| f32::from_bits(**bits).is_subnormal())
                    .count();
                assert_eq!(
                    reference_bits(&program, &operands, allowance, declared),
                    reference_bits(&program, &operands, allowance, strict),
                    "{family:?} case {key}: the declared and the strict readings disagree, so \
                     publishing the declared one moves bytes a retained digest was measured \
                     against",
                );
                compared += 1;
            }
        }

        assert_eq!(
            compared, 20,
            "five operand classes over three serial-sum extents and the adversarial contraction",
        );
        assert_eq!(
            subnormal_operands, 2,
            "the corpus holds no subnormal operand, so the agreement above is about a population \
             that cannot distinguish the two readings",
        );

        // The two readings *are* distinguishable by this oracle, on this route,
        // at a published member's own program: the least positive subnormal alone
        // through the singleton reduction. Without this the agreements above
        // would also be satisfied by an evaluator that ignored the contract it
        // was handed.
        let singleton = serial_sum_program(PUBLISHED_ROWS, 1);
        let compilation = published_compilation(&singleton);
        let declared = conformance_of(
            compilation
                .selected()
                .expect("the portfolio retains a selected plan"),
        )
        .expect("the packaged f32 realization bridges");
        let probe = vec![Operand {
            key: InputKey::new(INPUT_KEY).expect("the input key is valid"),
            shape: Shape::from_dims([PUBLISHED_ROWS, 1]),
            bits: vec![0x0000_0001],
        }];
        let allowance = ProofFamily::SerialSum {
            rows: PUBLISHED_ROWS,
            columns: 1,
        }
        .iteration_step_allowance();
        assert_eq!(
            reference_bits(&singleton, &probe, allowance, strict),
            vec![0x0000_0001],
            "the strict reading must carry the least positive subnormal through unchanged",
        );
        assert_eq!(
            reference_bits(&singleton, &probe, allowance, declared),
            vec![0x0000_0000],
            "the declared contract did not reach the oracle: a flushing conformance returned the \
             preserving answer",
        );
    }

    /// The published cell's operand stream is the probe's, pinned against values
    /// the probe's own Python produced.
    ///
    /// **These four literals are the only independent evidence in this file, and
    /// that is why they are literals.** Every other property below — the value
    /// rule, the two distinct streams, the element counts — is satisfied by any
    /// stream of the same *shape*, so a transcription that had drifted in the
    /// mixing constants would pass all of them. The literals were produced by
    /// running `splitmix64`/`prng_value` out of
    /// `spikes/scheduling/metal_contraction_vertical/contraction_probe.py`, which
    /// is the probe's own reference implementation of the stream `host.m` filled
    /// the device buffers from, so they are a second implementation's answer
    /// rather than this one's recorded back.
    ///
    /// To reproduce or refute them, copy `MASK64`, `splitmix64`, and `prng_value`
    /// out of that file — importing it would run a probe — and print
    /// `prng_value(0x5445524D, 0)` and
    /// `prng_value(0x5445524D ^ 0xA5A5A5A5A5A5A5A5, 0)` as `f32` bit patterns.
    #[test]
    fn the_probe_stream_is_pinned_against_the_probes_own_implementation() {
        let right_seed = WORKLOAD_SEED ^ RIGHT_SEED_MASK;
        assert_eq!(right_seed, 0xa5a5_a5a5_f1e0_f7e8);
        assert_eq!(probe_value(WORKLOAD_SEED, 0).to_bits(), 0x3e32_47dc);
        assert_eq!(probe_value(WORKLOAD_SEED, 1023).to_bits(), 0xbd54_d680);
        assert_eq!(probe_value(right_seed, 0).to_bits(), 0x3ea3_db76);
        assert_eq!(probe_value(right_seed, 1_048_575).to_bits(), 0xbeed_2a46);
    }

    /// Every operand of the published cell is exactly representable as
    /// `m * 2^-24`, over a counted population.
    ///
    /// The property the probe chose the stream *for*: the operands introduce no
    /// rounding of their own, so a difference the digest comparison reports is a
    /// difference in how the contraction was evaluated. Counted rather than
    /// sampled, because a check that silently examined nothing would be
    /// indistinguishable from one that examined everything — the exact failure
    /// mode this repository has recorded.
    #[test]
    fn every_published_cell_operand_is_exactly_representable() {
        let (m, n, k) = DECODE_KV;
        let activations = probe_bits(m * k, WORKLOAD_SEED);
        let weights = probe_bits(n * k, WORKLOAD_SEED ^ RIGHT_SEED_MASK);
        assert_eq!(activations.len(), 1024);
        assert_eq!(weights.len(), 1_048_576);

        let mut examined = 0_usize;
        for bits in activations.iter().chain(&weights) {
            let scaled = f32::from_bits(*bits) * 16_777_216.0;
            assert!(
                scaled.fract() == 0.0 && (-8_388_608.0..8_388_608.0).contains(&scaled),
                "{bits:#010x} scales to {scaled}, which is not an integer in [-2^23, 2^23)",
            );
            examined += 1;
        }
        assert_eq!(
            examined,
            1024 + 1_048_576,
            "the loop must have examined every operand of both streams",
        );
    }

    /// The two operands are drawn from different streams.
    ///
    /// A mask applied to the wrong side, or dropped, would make `weights` open
    /// with `activations`' 1,024 values — turning the cell into a partial
    /// self-inner-product that still contracts, still publishes 1,024 elements,
    /// and disagrees with the retained digest for a reason no other check here
    /// names.
    #[test]
    fn the_two_operands_are_not_the_same_stream() {
        let (m, _, k) = DECODE_KV;
        let activations = probe_bits(m * k, WORKLOAD_SEED);
        let weights = probe_bits(m * k, WORKLOAD_SEED ^ RIGHT_SEED_MASK);
        assert_ne!(activations, weights);
    }

    /// Each published contraction member names the extents its own case source is
    /// written for.
    ///
    /// **The guard this file's two refusals exist to make reachable, checked
    /// without a toolchain.** `cases_for` refuses a shape it has no operand table
    /// or no retained measurement for, but that refusal only fires while
    /// something publishes; this compares the extents
    /// `crate::envelope::CONTRACTION_MEMBERS` declares against the extents this
    /// module is written for, so a member moved there fails in the ordinary gate
    /// rather than on the first host that publishes.
    #[test]
    fn the_published_contraction_extents_are_the_ones_this_module_is_written_for() {
        let declared: Vec<Option<(u64, u64, u64)>> = CONTRACTION_MEMBERS
            .iter()
            .map(|member| member.family.contraction_extents())
            .collect();
        let mut expected = vec![Some((CONTRACTION_M, CONTRACTION_N, CONTRACTION_K))];
        expected.extend(
            L3_CORRECTNESS_CELLS
                .iter()
                .filter(|cell| cell.fits_one_proof_payload())
                .map(|cell| Some(cell.extents())),
        );
        assert_eq!(
            declared, expected,
            "a published contraction member moved away from the operand table or the retained \
             measurement written for it here",
        );
        assert_eq!(
            declared.len(),
            6,
            "the adversarial member and the five correctness cells a sidecar can carry are routed",
        );
    }

    /// The restated reference bound is the reference's own, and it decides which
    /// cells the gate routes.
    ///
    /// **Restating a `pub(crate)` constant is only safe while something compares
    /// it**, and this crate can: the evaluator publishes the number it is holding
    /// one occurrence to. Without this the four cells that need a stated allowance
    /// would be selected against a number that had silently stopped being the
    /// reference's, and the gate's split would be about a literal rather than
    /// about a bound.
    #[test]
    fn the_restated_reference_bound_is_the_evaluators_own() {
        let evaluator = ReferenceEvaluator::standard().expect("the governed profile composes");
        assert_eq!(
            u64::try_from(evaluator.iteration_step_allowance())
                .expect("the reference's allowance fits a u64"),
            REFERENCE_DEFAULT_STEP_ALLOWANCE,
            "this crate restates the reference's per-occurrence bound and the reference has moved \
             it; which cells the ordinary gate routes is decided by that number",
        );
    }

    /// Only the cells whose fold exceeds the bound ask for an allowance, and each
    /// asks for exactly its own fold.
    ///
    /// **The negative half is the load-bearing one.** An allowance handed to every
    /// family would authorize a larger fold for the serial sum and the adversarial
    /// contraction — which need none — and the authorization would then be
    /// invisible rather than stated. The positive half pins that a cell asks for
    /// its own step count rather than for some round number that happens to cover
    /// it.
    #[test]
    fn only_the_cells_past_the_bound_state_an_allowance() {
        let default = usize::try_from(REFERENCE_DEFAULT_STEP_ALLOWANCE).expect("it fits a usize");
        assert_eq!(
            ProofFamily::SerialSum {
                rows: 1,
                columns: 3
            }
            .iteration_step_allowance(),
            default,
        );
        assert_eq!(
            ProofFamily::Contraction {
                m: CONTRACTION_M,
                n: CONTRACTION_N,
                k: CONTRACTION_K,
            }
            .iteration_step_allowance(),
            default,
        );

        let mut stated = 0_usize;
        let mut defaulted = 0_usize;
        for cell in &L3_CORRECTNESS_CELLS {
            let (m, n, k) = cell.extents();
            let allowance = ProofFamily::L3CorrectnessCell { m, n, k }.iteration_step_allowance();
            assert_eq!(
                m * n * k,
                cell.fold_steps,
                "{}: the pinned fold must be the product of the cell's own extents",
                cell.id,
            );
            if cell.folds_under_the_default_allowance() {
                assert_eq!(
                    allowance, default,
                    "{}: a cell under the bound must be published by the ordinary evaluator",
                    cell.id,
                );
                defaulted += 1;
            } else {
                assert_eq!(
                    u64::try_from(allowance).expect("an allowance fits a u64"),
                    cell.fold_steps,
                    "{}: a cell past the bound states its own fold and no more",
                    cell.id,
                );
                stated += 1;
            }
        }
        assert_eq!(
            (defaulted, stated),
            (2, 4),
            "two correctness cells fold under the reference's own bound and four state an \
             allowance; a split that moved would change what the ordinary gate authorizes",
        );
    }
}
