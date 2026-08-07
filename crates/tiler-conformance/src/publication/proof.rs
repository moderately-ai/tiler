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

use tiler_artifact::program::VerifiedArtifactProgram;
use tiler_artifact::proof::{
    ProofBuildError, ProofCaseKey, ProofCaseSpec, ProofCodecError, ProofNumericalIdentity,
    ProofProvenance, ProofReferenceIdentity, ProofSidecarBuilder,
};
use tiler_ir::semantic::{F32, InputKey, OutputKey, SemanticProgram};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
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

/// The extents of the one L3 correctness cell published here, `w_decode_kv`.
///
/// Stated and checked for the same reason [`CONTRACTION_M`] is, and one reason
/// more: a *retained* `result_sha256` exists for exactly this cell at exactly
/// these extents — `crate::envelope::L3_CELL_RESULT_SHA256` — so operands
/// generated at any other shape would be compared against a digest that never
/// described them.
const L3_CELL_M: u64 = 1;
/// See [`L3_CELL_M`].
const L3_CELL_N: u64 = 1024;
/// See [`L3_CELL_M`].
const L3_CELL_K: u64 = 1024;

/// The probe's workload seed, `contraction_probe.py`'s `WORKLOAD_SEED`.
const WORKLOAD_SEED: u64 = 0x5445_524D;
/// The probe's right-operand seed derivation, `host.m`'s `fill_prng` call.
const RIGHT_SEED_MASK: u64 = 0xA5A5_A5A5_A5A5_A5A5;
/// The stable case key of the one operand set the probe measured.
///
/// One case rather than five: the retained digest is a measurement of this exact
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
    /// Returned rather than matched at each call site because two published
    /// members carry contraction extents and the routed half asserts them against
    /// the artifact's own declaration; a second match would be a second place for
    /// the two to disagree.
    pub(crate) const fn contraction_extents(self) -> Option<(u64, u64, u64)> {
        match self {
            Self::SerialSum { .. } => None,
            Self::Contraction { m, n, k } | Self::L3CorrectnessCell { m, n, k } => Some((m, n, k)),
        }
    }
}

/// Why a proof-case sidecar could not be published.
#[derive(Debug)]
pub(crate) enum SidecarFailure {
    /// The draft refused the artifact, the provenance, or the case.
    Build(ProofBuildError),
    /// The verified record did not encode.
    Encode(ProofCodecError),
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
    /// A published L3 cell is not the one a retained `result_sha256` describes.
    ///
    /// Its own class, and a stricter one than [`Self::UnwrittenContractionShape`]:
    /// that refusal says an operand table would have to be written, and this one
    /// says a *measurement* would have to be taken. The probe's stream is defined
    /// at every shape, so generating operands for another cell would succeed and
    /// publish a member whose expected bytes no retained digest describes.
    UnretainedProbeCell {
        /// The extents the publication asked for.
        requested: (u64, u64, u64),
        /// The extents a retained digest exists for.
        retained: (u64, u64, u64),
    },
}

impl std::fmt::Display for SidecarFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Build(cause) => write!(formatter, "the proof sidecar draft refused: {cause:?}"),
            Self::Encode(cause) => write!(formatter, "the proof sidecar did not encode: {cause:?}"),
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
                retained: (rm, rn, rk),
            } => write!(
                formatter,
                "a {m}x{n}x{k} L3 cell is published and the retained realization-probe digest it \
                 is compared against was measured at {rm}x{rn}x{rk}; a cell with no retained \
                 measurement cannot be published through this family",
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

/// Evaluates the program under the governed reference to get expected outputs.
///
/// **Every declared operand is bound, not the first one.** The evaluator takes
/// the whole binding set, so a family with two inputs supplies two bindings and
/// the oracle evaluates the program the artifact actually declares. A version of
/// this that bound only the leading operand would have evaluated a different
/// program and reported its bits as normative.
fn reference_bits(program: &SemanticProgram, operands: &[Operand]) -> Vec<u32> {
    let tensors: Vec<Tensor> = operands
        .iter()
        .map(|operand| tensor(&operand.shape, &operand.bits))
        .collect();
    let bindings: Vec<InputBinding<'_>> = operands
        .iter()
        .zip(&tensors)
        .map(|(operand, tensor)| InputBinding::new(&operand.key, tensor))
        .collect();
    let outputs = ReferenceEvaluator::standard()
        .expect("the governed reference profile composes")
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
            if (m, n, k) != (L3_CELL_M, L3_CELL_N, L3_CELL_K) {
                return Err(SidecarFailure::UnretainedProbeCell {
                    requested: (m, n, k),
                    retained: (L3_CELL_M, L3_CELL_N, L3_CELL_K),
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
/// # Errors
///
/// Returns [`SidecarFailure`] naming the boundary that refused. Nothing is worked
/// around: a rejection here means the record would not have described the
/// artifact it travels with.
pub(crate) fn encoded(
    artifact: &VerifiedArtifactProgram,
    program: &SemanticProgram,
    family: ProofFamily,
) -> Result<Vec<u8>, SidecarFailure> {
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
    for (key, operands) in cases_for(family)? {
        let expected = reference_bits(program, &operands);
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
    use super::{
        CONTRACTION_K, CONTRACTION_M, CONTRACTION_N, L3_CELL_K, L3_CELL_M, L3_CELL_N,
        RIGHT_SEED_MASK, WORKLOAD_SEED, probe_bits, probe_value,
    };
    use crate::envelope::CONTRACTION_MEMBERS;

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
        let activations = probe_bits(L3_CELL_M * L3_CELL_K, WORKLOAD_SEED);
        let weights = probe_bits(L3_CELL_N * L3_CELL_K, WORKLOAD_SEED ^ RIGHT_SEED_MASK);
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
        let activations = probe_bits(L3_CELL_M * L3_CELL_K, WORKLOAD_SEED);
        let weights = probe_bits(L3_CELL_M * L3_CELL_K, WORKLOAD_SEED ^ RIGHT_SEED_MASK);
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
        assert_eq!(
            declared,
            [
                Some((CONTRACTION_M, CONTRACTION_N, CONTRACTION_K)),
                Some((L3_CELL_M, L3_CELL_N, L3_CELL_K)),
            ],
            "a published contraction member moved away from the operand table or the retained \
             measurement written for it here",
        );
    }
}
