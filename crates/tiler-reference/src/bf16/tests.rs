//! Two populations, deliberately separate, plus the refusals that bound them.
//!
//! The **exhaustive** population enumerates all 65,536 BF16 encodings and is
//! `exhaustive-finite` evidence about this oracle's own decode and rounding: a
//! finite encoding must survive decode-then-round unchanged, and a NaN must
//! canonicalize. It is available only because the format is sixteen bits;
//! `docs/dtype-support.md`'s dtype-addition recipe records that F64 and F128 are
//! not exhaustively enumerable and need a stated bounded profile instead, so
//! nothing here should be read as the general method.
//!
//! The **witness** population is a named list of operation-specific cases across
//! six categories, and it is evidence about the *stated semantics*. Every
//! expected value below is derived by hand from BF16's parameters — sign 1,
//! exponent 8 with bias 127, trailing significand 7, so precision 8, `emin` -126,
//! `emax` 127 — and from the round-to-nearest-ties-to-even rule. None was
//! obtained by running the implementation and recording what it said; a corpus
//! produced that way agrees with its subject for reasons that say nothing about
//! either being right.
//!
//! Neither population is evidence about a device. Nothing here executes anything
//! on a GPU.

use tiler_ir::semantic::accuracy::{ExactRational, UlpFormatError};
use tiler_ir::semantic::{
    AttributeFieldId, Bf16, Bf16Add, Bf16Constant, Bf16Multiply, CanonicalField, CanonicalValue,
    CanonicalValueView, F32, F32Add, F32Constant, F32Multiply, InputKey, OperationAttributes,
    OutputKey, SCALAR_TYPE_FACT_CLASS, SCALAR_TYPE_FACT_EXPONENT_BIAS,
    SCALAR_TYPE_FACT_HAS_SUBNORMALS, SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
    SCALAR_TYPE_FACT_WIDTH_BITS, SemanticProgramBuilder, TypeKey, arithmetic_bf16_facts,
    builtin_scalar_value_type_facts, canonical_bf16_bits,
};
use tiler_ir::shape::Shape;

use super::{
    BF16_CONSTANT_BITS_ATTRIBUTE, BF16_FACT_CANONICAL_NAN_BITS, Bf16Arithmetic,
    Bf16BinaryReference, Bf16Format, Bf16Value, Bf16ValueValidator, ConstantBf16Reference,
    sign_mask,
};
use crate::evaluate::EvaluationRetention;
use crate::registry::{ReferenceEvaluationRequest, ReferenceOutputs};
use crate::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, ReferenceOperation,
    ReferenceOperationError, ReferenceValueError, ReferenceValueValidator, Tensor,
    TensorPayloadView, UnsupportedBf16Declaration,
};

/// Named BF16 encodings every witness below is written in terms of.
///
/// Each is derived from the format's fields rather than quoted from a table:
/// `bits = sign << 15 | biased_exponent << 7 | trailing_significand`, with the
/// unbiased exponent `biased - 127` and an implicit leading significand bit on
/// every encoding whose exponent field is nonzero.
mod bits {
    /// Positive zero.
    pub(super) const POS_ZERO: u16 = 0x0000;
    /// Negative zero.
    pub(super) const NEG_ZERO: u16 = 0x8000;
    /// The least positive subnormal, `1 * 2^-133`.
    pub(super) const MIN_SUBNORMAL: u16 = 0x0001;
    /// The least negative subnormal.
    pub(super) const NEG_MIN_SUBNORMAL: u16 = 0x8001;
    /// Two least-subnormal quanta, `2 * 2^-133`.
    pub(super) const TWO_SUBNORMAL_QUANTA: u16 = 0x0002;
    /// Three least-subnormal quanta, `3 * 2^-133`.
    pub(super) const THREE_SUBNORMAL_QUANTA: u16 = 0x0003;
    /// Half the least normal, `2^-127`, which is `64 * 2^-133`.
    pub(super) const HALF_MIN_NORMAL: u16 = 0x0040;
    /// The greatest subnormal, `127 * 2^-133`.
    pub(super) const MAX_SUBNORMAL: u16 = 0x007f;
    /// The least positive normal, `2^-126`, which is `128 * 2^-133`.
    pub(super) const MIN_NORMAL: u16 = 0x0080;
    /// The greatest finite value, `255 * 2^120`.
    pub(super) const MAX_FINITE: u16 = 0x7f7f;
    /// The greatest finite negative value.
    pub(super) const NEG_MAX_FINITE: u16 = 0xff7f;
    /// Positive infinity.
    pub(super) const POS_INFINITY: u16 = 0x7f80;
    /// Negative infinity.
    pub(super) const NEG_INFINITY: u16 = 0xff80;
    /// The canonical quiet NaN this family's arithmetic installs.
    pub(super) const CANONICAL_NAN: u16 = 0x7fc0;
    /// A non-canonical NaN payload, used to check canonicalization.
    pub(super) const NONCANONICAL_NAN: u16 = 0x7fc1;
    /// One, `128 * 2^-7`.
    pub(super) const ONE: u16 = 0x3f80;
    /// Negative one.
    pub(super) const NEG_ONE: u16 = 0xbf80;
    /// The value `1 + 2^-7`, one quantum above one.
    pub(super) const ONE_PLUS_ULP: u16 = 0x3f81;
    /// The value `1 + 2^-6`, two quanta above one.
    pub(super) const ONE_PLUS_TWO_ULP: u16 = 0x3f82;
    /// The value `255 * 2^-8`, one quantum below one.
    pub(super) const ONE_MINUS_ULP: u16 = 0x3f7f;
    /// One half.
    pub(super) const HALF: u16 = 0x3f00;
    /// Two.
    pub(super) const TWO: u16 = 0x4000;
    /// Three, `192 * 2^-6`.
    pub(super) const THREE: u16 = 0x4040;
    /// The value `2^-8`, exactly half a quantum in the binade `[1, 2)`.
    ///
    /// Adding it to a value there constructs a tie no BF16 encoding can hold,
    /// which is how the corpus reaches ties-to-even by arithmetic rather than by
    /// writing an unrepresentable literal.
    pub(super) const HALF_ULP_AT_ONE: u16 = 0x3b80;
    /// The negation of [`HALF_ULP_AT_ONE`].
    pub(super) const NEG_HALF_ULP_AT_ONE: u16 = 0xbb80;
    /// `2^118`, one quarter of the top binade's quantum.
    pub(super) const QUARTER_TOP_QUANTUM: u16 = 0x7a80;
    /// `2^119`, exactly half the top binade's quantum.
    pub(super) const HALF_TOP_QUANTUM: u16 = 0x7b00;
    /// The nearest BF16 below one third, `171 * 2^-9`.
    pub(super) const NEAREST_THIRD: u16 = 0x3eab;
    /// One quantum below [`NEAREST_THIRD`], `170 * 2^-9`.
    pub(super) const BELOW_NEAREST_THIRD: u16 = 0x3eaa;
}

/// One named conformance witness: an arithmetic applied to exact operands.
struct Witness {
    /// What this case is evidence about.
    name: &'static str,
    /// The arithmetic applied.
    arithmetic: Bf16Arithmetic,
    /// Left operand encoding.
    left: u16,
    /// Right operand encoding.
    right: u16,
    /// The exact result encoding the stated semantics require.
    expected: u16,
}

fn format() -> Bf16Format {
    Bf16Format::governed().expect("the governed bf16 declarations parameterize this reference")
}

fn evaluate(format: &Bf16Format, witness: &Witness) -> u16 {
    format.round(
        &witness
            .arithmetic
            .apply(&format.decode(witness.left), &format.decode(witness.right)),
    )
}

/// The witnesses, in the six categories the stated semantics have rules for.
///
/// Separate functions rather than one table so the census below can report what
/// each exceptional class contributes: a corpus whose subnormal group silently
/// emptied would otherwise still look like a passing corpus.
fn witness_categories() -> Vec<(&'static str, Vec<Witness>)> {
    vec![
        ("zeros and signs", zero_witnesses()),
        ("subnormals and underflow", subnormal_witnesses()),
        ("ties", tie_witnesses()),
        ("ordinary rounding", rounding_witnesses()),
        ("overflow", overflow_witnesses()),
        ("infinities and NaN", exceptional_witnesses()),
    ]
}

fn witnesses() -> Vec<Witness> {
    witness_categories()
        .into_iter()
        .flat_map(|(_, cases)| cases)
        .collect()
}

/// Zeros, their signs, and the cancellation that produces one.
fn zero_witnesses() -> Vec<Witness> {
    use Bf16Arithmetic::{Add, Multiply};
    use bits::{NEG_ONE, NEG_ZERO, ONE, POS_ZERO};
    vec![
        Witness {
            name: "positive zero plus positive zero is positive zero",
            arithmetic: Add,
            left: POS_ZERO,
            right: POS_ZERO,
            expected: POS_ZERO,
        },
        Witness {
            name: "negative zero plus negative zero is the only signed-zero sum that is negative",
            arithmetic: Add,
            left: NEG_ZERO,
            right: NEG_ZERO,
            expected: NEG_ZERO,
        },
        Witness {
            name: "opposite-signed zeros sum to positive zero under round-to-nearest",
            arithmetic: Add,
            left: POS_ZERO,
            right: NEG_ZERO,
            expected: POS_ZERO,
        },
        Witness {
            name: "negative zero times positive zero is negative zero: the signs exclusive-or",
            arithmetic: Multiply,
            left: NEG_ZERO,
            right: POS_ZERO,
            expected: NEG_ZERO,
        },
        Witness {
            name: "negative one times positive zero is negative zero",
            arithmetic: Multiply,
            left: NEG_ONE,
            right: POS_ZERO,
            expected: NEG_ZERO,
        },
        Witness {
            name: "a value plus its negation is positive zero, not the zero of either operand",
            arithmetic: Add,
            left: ONE,
            right: NEG_ONE,
            expected: POS_ZERO,
        },
    ]
}

/// Subnormals preserved exactly, and gradual underflow into and through them.
fn subnormal_witnesses() -> Vec<Witness> {
    use Bf16Arithmetic::{Add, Multiply};
    use bits::{
        HALF, HALF_MIN_NORMAL, MAX_SUBNORMAL, MIN_NORMAL, MIN_SUBNORMAL, NEG_MIN_SUBNORMAL,
        NEG_ONE, NEG_ZERO, POS_ZERO, TWO, TWO_SUBNORMAL_QUANTA,
    };
    vec![
        Witness {
            // 2^-133 * 2 = 2^-132, which is two subnormal quanta.
            name: "the least positive subnormal doubles to the next subnormal rather than flushing",
            arithmetic: Multiply,
            left: MIN_SUBNORMAL,
            right: TWO,
            expected: TWO_SUBNORMAL_QUANTA,
        },
        Witness {
            // 2^-134 is exactly half a subnormal quantum; the tie goes to the even
            // count, which is zero, and round-to-nearest keeps the sign.
            name: "half the least positive subnormal underflows to positive zero at the tie",
            arithmetic: Multiply,
            left: MIN_SUBNORMAL,
            right: HALF,
            expected: POS_ZERO,
        },
        Witness {
            // The same underflow from the other side, which is where the sign rule
            // becomes observable.
            name: "half the least negative subnormal underflows to negative zero",
            arithmetic: Multiply,
            left: NEG_MIN_SUBNORMAL,
            right: HALF,
            expected: NEG_ZERO,
        },
        Witness {
            // 127 * 2^-133 + 1 * 2^-133 = 128 * 2^-133 = 2^-126.
            name: "the greatest subnormal plus the least subnormal is exactly the least normal",
            arithmetic: Add,
            left: MAX_SUBNORMAL,
            right: MIN_SUBNORMAL,
            expected: MIN_NORMAL,
        },
        Witness {
            // 2^-126 / 2 = 2^-127 = 64 * 2^-133, a subnormal reached with no
            // rounding at all.
            name: "the least normal halves into the subnormal range exactly",
            arithmetic: Multiply,
            left: MIN_NORMAL,
            right: HALF,
            expected: HALF_MIN_NORMAL,
        },
        Witness {
            name: "a subnormal keeps its magnitude and takes a sign through a negation",
            arithmetic: Multiply,
            left: MIN_SUBNORMAL,
            right: NEG_ONE,
            expected: NEG_MIN_SUBNORMAL,
        },
    ]
}

/// Exact halfway cases, resolved to the even significand.
fn tie_witnesses() -> Vec<Witness> {
    use Bf16Arithmetic::{Add, Multiply};
    use bits::{
        HALF, HALF_ULP_AT_ONE, NEG_HALF_ULP_AT_ONE, NEG_ONE, ONE, ONE_PLUS_TWO_ULP, ONE_PLUS_ULP,
        THREE_SUBNORMAL_QUANTA, TWO_SUBNORMAL_QUANTA,
    };
    vec![
        Witness {
            // 1 + 2^-8 is 128.5 quanta of 2^-7; the floor 128 is even, so the tie
            // rounds down and the result is one.
            name: "a tie half a quantum above one rounds down to the even significand",
            arithmetic: Add,
            left: ONE,
            right: HALF_ULP_AT_ONE,
            expected: ONE,
        },
        Witness {
            // 1 + 2^-7 + 2^-8 is 129.5 quanta; the floor 129 is odd, so the same
            // rule rounds up. The pair is what shows the rule is ties-to-even
            // rather than ties-downward.
            name: "a tie half a quantum above one-plus-one-quantum rounds up to the even significand",
            arithmetic: Add,
            left: ONE_PLUS_ULP,
            right: HALF_ULP_AT_ONE,
            expected: ONE_PLUS_TWO_ULP,
        },
        Witness {
            // The tie is decided on the magnitude, so the negative side resolves
            // toward zero here exactly as the positive side rounds down.
            name: "a negative tie half a quantum below negative one rounds to the even significand",
            arithmetic: Add,
            left: NEG_ONE,
            right: NEG_HALF_ULP_AT_ONE,
            expected: NEG_ONE,
        },
        Witness {
            // 1.5 subnormal quanta; the floor 1 is odd, so the tie rounds up to
            // two. Ties-to-even applies on the subnormal spacing, which is uniform
            // rather than binade-relative.
            name: "a tie between two subnormals rounds up to the even quantum count",
            arithmetic: Multiply,
            left: THREE_SUBNORMAL_QUANTA,
            right: HALF,
            expected: TWO_SUBNORMAL_QUANTA,
        },
    ]
}

/// Rounding that is not a tie, paired with an exactly representable neighbour.
fn rounding_witnesses() -> Vec<Witness> {
    use Bf16Arithmetic::Multiply;
    use bits::{BELOW_NEAREST_THIRD, NEAREST_THIRD, ONE, ONE_MINUS_ULP, THREE};
    vec![
        Witness {
            // 3 * 171 * 2^-9 = 513 * 2^-9 = 513/512, which is 128.25 quanta of
            // 2^-7 in the binade [1, 2). The fraction is a quarter, so this is a
            // genuine round-to-nearest and not a tie.
            name: "three times the nearest BF16 below one third rounds down to exactly one",
            arithmetic: Multiply,
            left: THREE,
            right: NEAREST_THIRD,
            expected: ONE,
        },
        Witness {
            // The neighbour one quantum below: 3 * 170 * 2^-9 = 255/256, which is
            // 255 quanta of 2^-8 in the binade [1/2, 1) and needs no rounding at
            // all. The pair separates "rounded to one" from "landed on one".
            name: "three times one quantum below that is exactly representable and unrounded",
            arithmetic: Multiply,
            left: THREE,
            right: BELOW_NEAREST_THIRD,
            expected: ONE_MINUS_ULP,
        },
    ]
}

/// Overflow to infinity, and the boundary reached by arithmetic on both sides.
fn overflow_witnesses() -> Vec<Witness> {
    use Bf16Arithmetic::{Add, Multiply};
    use bits::{
        HALF_TOP_QUANTUM, MAX_FINITE, NEG_INFINITY, NEG_MAX_FINITE, POS_INFINITY,
        QUARTER_TOP_QUANTUM, TWO,
    };
    vec![
        Witness {
            name: "the greatest finite value doubled overflows to infinity",
            arithmetic: Multiply,
            left: MAX_FINITE,
            right: TWO,
            expected: POS_INFINITY,
        },
        Witness {
            name: "the greatest finite value plus itself overflows to infinity",
            arithmetic: Add,
            left: MAX_FINITE,
            right: MAX_FINITE,
            expected: POS_INFINITY,
        },
        Witness {
            name: "the greatest negative finite value doubled overflows to negative infinity",
            arithmetic: Multiply,
            left: NEG_MAX_FINITE,
            right: TWO,
            expected: NEG_INFINITY,
        },
        Witness {
            // 255 * 2^120 + 2^118 is 255.25 quanta of the top binade: above the
            // largest finite value, below the midpoint, so it rounds back down and
            // stays finite.
            name: "a quarter of the top quantum above the greatest finite value stays finite",
            arithmetic: Add,
            left: MAX_FINITE,
            right: QUARTER_TOP_QUANTUM,
            expected: MAX_FINITE,
        },
        Witness {
            // 255 * 2^120 + 2^119 is exactly 255.5 quanta: the midpoint itself,
            // which overflows rather than rounding back. With the case above, this
            // is the boundary decided from both sides by arithmetic.
            name: "exactly half the top quantum above the greatest finite value overflows",
            arithmetic: Add,
            left: MAX_FINITE,
            right: HALF_TOP_QUANTUM,
            expected: POS_INFINITY,
        },
    ]
}

/// Infinity arithmetic, the invalid operations, and NaN canonicalization.
fn exceptional_witnesses() -> Vec<Witness> {
    use Bf16Arithmetic::{Add, Multiply};
    use bits::{
        CANONICAL_NAN, NEG_INFINITY, NEG_ONE, NONCANONICAL_NAN, ONE, POS_INFINITY, POS_ZERO,
    };
    vec![
        Witness {
            name: "infinity times zero is invalid and gives the canonical NaN",
            arithmetic: Multiply,
            left: POS_INFINITY,
            right: POS_ZERO,
            expected: CANONICAL_NAN,
        },
        Witness {
            name: "infinity minus infinity is invalid and gives the canonical NaN",
            arithmetic: Add,
            left: POS_INFINITY,
            right: NEG_INFINITY,
            expected: CANONICAL_NAN,
        },
        Witness {
            name: "infinity plus a finite value is infinity",
            arithmetic: Add,
            left: POS_INFINITY,
            right: ONE,
            expected: POS_INFINITY,
        },
        Witness {
            name: "infinity times a negative finite value is negative infinity",
            arithmetic: Multiply,
            left: POS_INFINITY,
            right: NEG_ONE,
            expected: NEG_INFINITY,
        },
        Witness {
            name: "negative infinity squared is positive infinity",
            arithmetic: Multiply,
            left: NEG_INFINITY,
            right: NEG_INFINITY,
            expected: POS_INFINITY,
        },
        Witness {
            name: "a non-canonical NaN operand canonicalizes through multiplication",
            arithmetic: Multiply,
            left: NONCANONICAL_NAN,
            right: ONE,
            expected: CANONICAL_NAN,
        },
        Witness {
            name: "a non-canonical NaN operand canonicalizes through addition",
            arithmetic: Add,
            left: NONCANONICAL_NAN,
            right: ONE,
            expected: CANONICAL_NAN,
        },
    ]
}

/// Every one of the 65,536 encodings survives decode-then-round, NaNs excepted.
///
/// **Exhaustive-finite.** The population is named and counted by class before the
/// verdict, because a check reporting a uniform answer over a set without knowing
/// how many members each class has is indistinguishable from one that did not
/// run. The class counts follow from the format's fields: two zeros,
/// `2 * (2^7 - 1)` subnormals, `2 * (2^8 - 2) * 2^7` normals, two infinities, and
/// `2 * (2^7 - 1)` NaNs.
///
/// NaN is deliberately not an identity here: it canonicalizes, which is the one
/// place decode-then-round is required *not* to reproduce its input.
#[test]
fn every_encoding_round_trips_except_the_nans_that_canonicalize() {
    let format = format();
    let least_normal = ExactRational::power_of_two(format.ulp.min_exponent());
    let (mut zeros, mut subnormals, mut normals, mut infinities, mut nans) = (0, 0, 0, 0, 0);
    let mut failures = Vec::new();
    for encoding in 0..=u16::MAX {
        let value = format.decode(encoding);
        let expected = match &value {
            Bf16Value::Nan => {
                nans += 1;
                format.canonical_nan_bits
            }
            Bf16Value::Infinite { .. } => {
                infinities += 1;
                encoding
            }
            Bf16Value::Finite { value, .. } => {
                if value.is_zero() {
                    zeros += 1;
                } else if value.abs() < least_normal {
                    subnormals += 1;
                } else {
                    normals += 1;
                }
                encoding
            }
        };
        let actual = format.round(&value);
        if actual != expected {
            failures.push(format!(
                "{encoding:#06x} round-tripped to {actual:#06x}, expected {expected:#06x}"
            ));
        }
    }
    assert_eq!(
        (zeros, subnormals, normals, infinities, nans),
        (2_usize, 254, 65_024, 2, 254),
        "the enumerated population is the whole format"
    );
    assert_eq!(zeros + subnormals + normals + infinities + nans, 65_536);
    assert!(failures.is_empty(), "{failures:?}");
}

/// Every hand-derived witness agrees, and every category contributes.
#[test]
fn the_hand_derived_witness_corpus_agrees_in_every_category() {
    let format = format();
    let mut failures = Vec::new();
    let mut total = 0_usize;
    for (category, cases) in witness_categories() {
        assert!(!cases.is_empty(), "the {category} category is empty");
        total += cases.len();
        for witness in cases {
            let actual = evaluate(&format, &witness);
            if actual != witness.expected {
                failures.push(format!(
                    "[{category}] {}: expected {:#06x}, produced {actual:#06x}",
                    witness.name, witness.expected
                ));
            }
        }
    }
    assert_eq!(
        total, 30,
        "the witness population is the one that was derived"
    );
    assert!(failures.is_empty(), "{failures:?}");
}

/// The overflow boundary is decided at the midpoint, on both sides of it.
///
/// A rounding that simply saturated at the largest finite value would pass every
/// witness above and still be wrong here, and one that overflowed a quantum early
/// would lose a representable value. Both sides are checked against the threshold
/// the format's own parameters fix rather than against an encoding.
#[test]
fn the_overflow_boundary_is_decided_at_the_midpoint() {
    let format = format();
    let threshold = format.overflow_threshold.clone();
    let largest = format.ulp.largest_finite();
    assert!(
        threshold > largest,
        "the overflow threshold lies above the largest finite value"
    );

    let finite = |value: ExactRational| {
        format.round(&Bf16Value::Finite {
            value,
            zero_is_negative: false,
        })
    };
    // The midpoint between the largest finite value and the threshold is strictly
    // below the threshold, so it rounds back down.
    assert_eq!(
        finite(largest.add(&threshold).scale_by_power_of_two(-1)),
        bits::MAX_FINITE
    );
    // The threshold itself overflows, and so does its negation.
    assert_eq!(finite(threshold.clone()), bits::POS_INFINITY);
    assert_eq!(finite(threshold.negate()), bits::NEG_INFINITY);
    // The largest finite value is still itself, in both signs.
    assert_eq!(finite(largest.clone()), bits::MAX_FINITE);
    assert_eq!(finite(largest.negate()), bits::NEG_MAX_FINITE);
}

/// Rounds one exact value with ties resolved away from zero instead of to even.
///
/// The perturbation the corpus is measured against. Everything else is this
/// module's own — the decode, the arithmetic, the overflow threshold, the binade
/// selection, and the encode — so a disagreement is evidence about the tie rule
/// rather than about a second rounding implementation.
fn round_ties_away(format: &Bf16Format, value: &Bf16Value) -> u16 {
    let Bf16Value::Finite {
        value: magnitude, ..
    } = value
    else {
        return format.round(value);
    };
    if magnitude.is_zero() {
        return format.round(value);
    }
    let negative = magnitude.is_negative();
    let magnitude = magnitude.abs();
    if magnitude >= format.overflow_threshold {
        return format.infinity(negative);
    }
    let ulp = &format.ulp;
    let precision = i32::try_from(ulp.precision()).expect("a bounded precision fits i32");
    let binade = magnitude
        .floor_log2_abs()
        .expect("a nonzero magnitude has a binade")
        .max(ulp.min_exponent());
    let quantum = ExactRational::power_of_two(binade - precision + 1);
    let quotient = magnitude.divide(&quantum).expect("a quantum is nonzero");
    let floor = quotient.floor_to_binary_grid(0);
    let quanta = if quotient.subtract(&floor) < ExactRational::power_of_two(-1) {
        floor
    } else {
        floor.add(&ExactRational::one())
    };
    let sign = if negative { sign_mask() } else { 0 };
    format
        .encode_magnitude(&quanta.multiply(&quantum))
        .map_or_else(|| format.infinity(negative), |encoded| sign | encoded)
}

/// The tie rule is load-bearing: changing only it breaks named witnesses.
///
/// Without this the corpus could pass while measuring nothing. The normative rule
/// must disagree with the corpus nowhere, and the perturbation must disagree in
/// exactly the four places a halfway case is resolved toward the even neighbour
/// that ties-away-from-zero does not choose — including the two where that
/// neighbour is a zero.
#[test]
fn changing_only_the_tie_rule_breaks_the_corpus() {
    let format = format();
    let mut normative = Vec::new();
    let mut perturbed = Vec::new();
    for witness in witnesses() {
        if evaluate(&format, &witness) != witness.expected {
            normative.push(witness.name);
        }
        let exact = witness
            .arithmetic
            .apply(&format.decode(witness.left), &format.decode(witness.right));
        if round_ties_away(&format, &exact) != witness.expected {
            perturbed.push(witness.name);
        }
    }
    assert!(
        normative.is_empty(),
        "round-to-nearest-ties-to-even disagrees nowhere: {normative:?}"
    );
    assert_eq!(
        perturbed,
        vec![
            "half the least positive subnormal underflows to positive zero at the tie",
            "half the least negative subnormal underflows to negative zero",
            "a tie half a quantum above one rounds down to the even significand",
            "a negative tie half a quantum below negative one rounds to the even significand",
        ],
        "ties-away-from-zero disagrees exactly where the halfway case is decided"
    );
}

/// The three keys evaluate through the standard evaluator to exact bits.
///
/// The end-to-end path: a verified semantic program, the governed reference
/// registry, and the exact encodings the witnesses above derive. This is what
/// `MissingCapability` used to answer for.
#[test]
fn a_pure_bf16_program_evaluates_to_exact_bits() {
    for (arithmetic, left, right, expected) in [
        // 3 * (171 * 2^-9) = 513/512, which rounds down to one.
        (
            Bf16Arithmetic::Multiply,
            bits::THREE,
            bits::NEAREST_THIRD,
            bits::ONE,
        ),
        // The greatest finite value doubled leaves the finite range.
        (
            Bf16Arithmetic::Multiply,
            bits::MAX_FINITE,
            bits::TWO,
            bits::POS_INFINITY,
        ),
        // 1 + 2^-8 is a tie that resolves to the even significand, which is one.
        (
            Bf16Arithmetic::Add,
            bits::ONE,
            bits::HALF_ULP_AT_ONE,
            bits::ONE,
        ),
        // A non-canonical NaN operand does not survive an arithmetic result.
        (
            Bf16Arithmetic::Add,
            bits::NONCANONICAL_NAN,
            bits::ONE,
            bits::CANONICAL_NAN,
        ),
    ] {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let left_value = Bf16Constant::apply(&mut graph, left).unwrap();
        let right_value = Bf16Constant::apply(&mut graph, right).unwrap();
        let result = match arithmetic {
            Bf16Arithmetic::Multiply => Bf16Multiply::apply(&mut graph, left_value, right_value),
            Bf16Arithmetic::Add => Bf16Add::apply(&mut graph, left_value, right_value),
        }
        .unwrap();
        graph
            .output(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = graph.build().unwrap();
        let outputs = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(&program, &[])
            .unwrap();
        assert_eq!(
            bf16_bits(&outputs[0]),
            vec![expected],
            "{arithmetic:?} of {left:#06x} and {right:#06x}"
        );
    }
}

/// The constant preserves its payload; only an arithmetic result canonicalizes.
///
/// `BF16_FACT_NAN_BEHAVIOUR` says a constant is
/// `preserved-exactly-the-declared-payload-is-not-canonicalized` while the
/// arithmetic canonicalizes every NaN result. The two are checked against each
/// other here, because an implementation that canonicalized at the constant would
/// satisfy every arithmetic witness above and still be wrong.
#[test]
fn a_constant_preserves_a_non_canonical_nan_payload_that_arithmetic_removes() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let payload = Bf16Constant::apply(&mut graph, bits::NONCANONICAL_NAN).unwrap();
    let one = Bf16Constant::apply(&mut graph, bits::ONE).unwrap();
    let product = Bf16Multiply::apply(&mut graph, payload, one).unwrap();
    graph
        .output(OutputKey::new("constant").unwrap(), payload)
        .unwrap();
    graph
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    let program = graph.build().unwrap();
    let outputs = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&program, &[])
        .unwrap();
    assert_eq!(bf16_bits(&outputs[0]), vec![bits::NONCANONICAL_NAN]);
    assert_eq!(bf16_bits(&outputs[1]), vec![bits::CANONICAL_NAN]);
}

/// A BF16 tensor is evaluated elementwise with the scalar broadcast the operation
/// admits, and a mismatched operand pair is refused.
#[test]
fn a_shaped_bf16_program_broadcasts_a_scalar_and_refuses_a_mismatch() {
    let key = InputKey::new("x").unwrap();
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let input = graph
        .input::<Bf16>(key.clone(), Shape::from_dims([3]))
        .unwrap();
    let scale = Bf16Constant::apply(&mut graph, bits::TWO).unwrap();
    let scaled = Bf16Multiply::apply(&mut graph, input, scale).unwrap();
    graph
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let program = graph.build().unwrap();
    let tensor = bf16_tensor(
        Shape::from_dims([3]),
        &[bits::ONE, bits::MIN_SUBNORMAL, bits::MAX_FINITE],
    );
    let outputs = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&program, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(
        bf16_bits(&outputs[0]),
        // 1 * 2 = 2; 2^-133 * 2 is the next subnormal; the greatest finite value
        // doubled overflows. One tensor, three value classes, one rounding each.
        vec![bits::TWO, bits::TWO_SUBNORMAL_QUANTA, bits::POS_INFINITY]
    );

    // Two operands of different nonzero ranks are not broadcast: the graph admits
    // no implicit broadcasting, and neither does the reference.
    let reference = Bf16BinaryReference::new(format(), Bf16Arithmetic::Add);
    let other = bf16_tensor(Shape::from_dims([2]), &[bits::ONE, bits::ONE]);
    assert_eq!(
        reference.combine(&tensor, &other),
        Err(ReferenceOperationError::InvalidApplication)
    );
}

/// An operand of another dtype is refused rather than reinterpreted.
///
/// The registry's signature dispatch already prevents this on the program path,
/// so the check exists for the direct one: BF16 and binary32 elements are both
/// opaque byte runs, and a reference that read four bytes as two would answer for
/// a dtype it was never asked about.
#[test]
fn a_non_bf16_operand_is_refused_by_every_bf16_capability() {
    let reference = Bf16BinaryReference::new(format(), Bf16Arithmetic::Multiply);
    let bf16 = bf16_tensor(Shape::new([]), &[bits::ONE]);
    let f32_scalar = Tensor::scalar(
        F32::resolved_type(),
        ReferenceElement::from_float_bits(
            1.0_f32.to_bits().to_be_bytes(),
            FloatBitOrder::MostSignificantByteFirst,
        )
        .unwrap(),
    )
    .unwrap();
    for (left, right) in [
        (&bf16, &f32_scalar),
        (&f32_scalar, &bf16),
        (&f32_scalar, &f32_scalar),
    ] {
        assert_eq!(
            reference.combine(left, right),
            Err(ReferenceOperationError::InvalidApplication)
        );
    }
    // The same pair in bf16 is admitted, so the refusals are about the operand
    // type rather than about the fixture.
    assert!(reference.combine(&bf16, &bf16).is_ok());

    let validator = Bf16ValueValidator {
        payload_bytes: format().payload_bytes(),
    };
    assert_eq!(
        validator.validate(&f32_scalar),
        Err(ReferenceValueError::InvalidRepresentation)
    );
    assert_eq!(validator.validate(&bf16), Ok(()));
}

/// The validator admits exactly the element width the descriptor declares.
///
/// The width is read from the descriptor rather than written down here, so this
/// pins the read: a two-byte element is admitted and every other width refused,
/// including the four-byte one a binary32 payload would carry.
#[test]
fn the_value_validator_admits_only_the_declared_element_width() {
    let payload_bytes = format().payload_bytes();
    assert_eq!(
        payload_bytes, 2,
        "the bf16 descriptor states a 16-bit width"
    );
    let validator = Bf16ValueValidator { payload_bytes };
    for width in [0_usize, 1, 3, 4, 8] {
        let tensor = Tensor::scalar(
            Bf16::resolved_type(),
            ReferenceElement::new(vec![0_u8; width]).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validator.validate(&tensor),
            Err(ReferenceValueError::InvalidRepresentation),
            "a {width}-byte element is not a bf16 element"
        );
    }
    assert_eq!(
        validator.validate(&bf16_tensor(Shape::new([]), &[bits::ONE])),
        Ok(())
    );
}

/// The constant refuses a payload declaring another float format or another width.
#[test]
fn the_constant_refuses_a_payload_of_another_format_or_width() {
    let reference = ConstantBf16Reference { format: format() };
    let bf16_key = Bf16::resolved_type().nominal_key().unwrap().clone();
    for payload in [
        CanonicalValue::float_bits(
            TypeKey::new("tiler", "f32", 1).unwrap(),
            1.0_f32.to_bits().to_be_bytes(),
        )
        .unwrap(),
        CanonicalValue::float_bits(bf16_key, [0x3f, 0x80, 0x00, 0x00]).unwrap(),
    ] {
        assert_eq!(
            apply_constant(&reference, &constant_attributes(payload)),
            Err(ReferenceOperationError::InvalidApplication)
        );
    }
    // The governed payload is admitted, so the refusals are about the declaration
    // rather than about the harness.
    assert_eq!(
        apply_constant(
            &reference,
            &constant_attributes(canonical_bf16_bits(bits::ONE))
        ),
        Ok(vec![bits::ONE])
    );
}

/// The canonical NaN payload is read from the declaration, not restated here.
///
/// `tiler-ir` builds `BF16_FACT_CANONICAL_NAN_BITS` from its own governed
/// constant and this reference reads that field, so the check binds the two: an
/// arithmetic NaN result carries exactly the payload the operation declares, and
/// the crate's canonicalization rule cannot drift into a second answer for BF16.
#[test]
fn the_arithmetic_nan_payload_is_the_one_the_operation_declares() {
    let facts = arithmetic_bf16_facts();
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("the governed bf16 arithmetic facts are a record");
    };
    let declared = fields
        .iter()
        .find(|field| field.id() == BF16_FACT_CANONICAL_NAN_BITS)
        .map(CanonicalField::value)
        .expect("the arithmetic declares a canonical NaN payload");
    let CanonicalValueView::FloatBits(payload) = declared.view() else {
        panic!("the declared payload is exact float bits");
    };
    let declared = u16::from_be_bytes(<[u8; 2]>::try_from(payload.bits()).unwrap());
    let format = format();
    assert_eq!(format.canonical_nan_bits, declared);
    assert_eq!(
        format.round(&Bf16Arithmetic::Multiply.apply(
            &format.decode(bits::POS_INFINITY),
            &format.decode(bits::POS_ZERO)
        )),
        declared
    );
}

/// The declared parameters are the ones the catalog row states.
///
/// Nothing in the module writes these numbers down, so this is where a catalog
/// edit becomes visible: a descriptor that stopped describing BF16 would move
/// them, and every value the corpus derives with them.
#[test]
fn the_governed_declarations_fix_the_bf16_parameters() {
    let format = format();
    let ulp = &format.ulp;
    assert_eq!(ulp.class(), "bfloat");
    assert_eq!(ulp.precision(), 8);
    assert_eq!(ulp.min_exponent(), -126);
    assert_eq!(ulp.max_exponent(), 127);
    assert!(ulp.has_subnormals());
    assert_eq!(format.payload_bytes, 2);
    assert_eq!(format.canonical_nan_bits, bits::CANONICAL_NAN);
    // The largest finite value and the overflow threshold, written from the
    // parameters: `(2^8 - 1) * 2^120` and half a quantum above it.
    assert_eq!(
        ulp.largest_finite(),
        ExactRational::from_integer(255).scale_by_power_of_two(120)
    );
    assert_eq!(
        format.overflow_threshold,
        ExactRational::from_integer(511).scale_by_power_of_two(119)
    );
}

/// Every declaration refusal this module can reach is watched refusing.
///
/// One variant is deliberately absent. `MissingDescriptor` is reachable only from
/// a catalog that stopped registering `tiler::bf16@1` at all, which no perturbable
/// input here can construct; every other rule is driven by a record differing from
/// the governed one in exactly one field.
#[test]
fn each_unrealizable_declaration_is_refused_by_name() {
    let key = Bf16::resolved_type().nominal_key().unwrap().clone();
    let descriptor = builtin_scalar_value_type_facts(&Bf16::resolved_type()).unwrap();
    let arithmetic = arithmetic_bf16_facts();
    // The governed pair is accepted, so every refusal below is about its own
    // perturbation rather than about the harness.
    assert!(Bf16Format::from_declarations(key.clone(), &descriptor, &arithmetic).is_ok());

    let refuse = |facts: &CanonicalValue, arithmetic: &CanonicalValue| {
        Bf16Format::from_declarations(key.clone(), facts, arithmetic)
            .expect_err("the perturbed declaration is refused")
    };
    assert_eq!(
        refuse(
            &without_field(&descriptor, SCALAR_TYPE_FACT_WIDTH_BITS),
            &arithmetic
        ),
        UnsupportedBf16Declaration::MalformedFact {
            field: "the encoded width"
        }
    );
    assert_eq!(
        refuse(
            &with_field(
                &descriptor,
                SCALAR_TYPE_FACT_CLASS,
                CanonicalValue::utf8("not-a-registered-class").unwrap()
            ),
            &arithmetic
        ),
        UnsupportedBf16Declaration::IncompatibleFormat(UlpFormatError::UnrecognizedClass {
            class: "not-a-registered-class".to_owned()
        })
    );
    assert_eq!(
        refuse(
            &with_field(
                &descriptor,
                SCALAR_TYPE_FACT_WIDTH_BITS,
                CanonicalValue::unsigned_u32(32)
            ),
            &arithmetic
        ),
        UnsupportedBf16Declaration::UnsupportedWidth { width_bits: 32 }
    );
    assert_eq!(
        refuse(
            &with_field(
                &descriptor,
                SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
                CanonicalValue::unsigned_u32(6)
            ),
            &arithmetic
        ),
        UnsupportedBf16Declaration::InconsistentExponentRange {
            exponent_bits: 9,
            max_exponent: 127
        }
    );
    assert_eq!(
        refuse(
            &with_field(
                &descriptor,
                SCALAR_TYPE_FACT_EXPONENT_BIAS,
                CanonicalValue::signed_i32(15)
            ),
            &arithmetic
        ),
        UnsupportedBf16Declaration::OverriddenExponentBias {
            declared: 15,
            derived: 127
        }
    );
    assert_eq!(
        refuse(
            &with_field(
                &descriptor,
                SCALAR_TYPE_FACT_HAS_SUBNORMALS,
                CanonicalValue::boolean(false)
            ),
            &arithmetic
        ),
        UnsupportedBf16Declaration::SubnormalsAbsent
    );
    assert_eq!(
        refuse(
            &descriptor,
            &without_field(&arithmetic, BF16_FACT_CANONICAL_NAN_BITS)
        ),
        UnsupportedBf16Declaration::MalformedFact {
            field: "the declared canonical arithmetic NaN payload"
        }
    );
    assert_eq!(
        refuse(
            &descriptor,
            &with_field(
                &arithmetic,
                BF16_FACT_CANONICAL_NAN_BITS,
                canonical_bf16_bits(bits::ONE)
            )
        ),
        UnsupportedBf16Declaration::ArithmeticNanPayloadIsNotNan { bits: bits::ONE }
    );
}

/// An F32 program is unchanged by the arrival of the BF16 capabilities.
///
/// The registry these F32 answers come from is the one that now also carries
/// BF16, which is the whole content of the check: the reference registry identity
/// moved when the four capabilities were added, and these bit patterns did not.
#[test]
fn an_f32_program_evaluates_identically_through_the_widened_registry() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let value = F32Constant::apply(&mut graph, 1.5_f32.to_bits()).unwrap();
    let scale = F32Constant::apply(&mut graph, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut graph, (-0.5_f32).to_bits()).unwrap();
    let product = F32Multiply::apply(&mut graph, value, scale).unwrap();
    let mapped = F32Add::apply(&mut graph, product, bias).unwrap();
    graph
        .output(OutputKey::new("mapped").unwrap(), mapped)
        .unwrap();
    let program = graph.build().unwrap();
    let evaluator = ReferenceEvaluator::standard().unwrap();
    let outputs = evaluator.evaluate(&program, &[]).unwrap();
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("an f32 result is dense");
    };
    // 1.5 * 2.0 - 0.5 = 2.5, exactly representable, so these bits are decided by
    // the arithmetic rather than by a rounding.
    assert_eq!(elements[0].as_bytes(), 2.5_f32.to_bits().to_be_bytes());

    // The same evaluator answers for BF16, so the F32 answer above came from the
    // widened oracle rather than from a narrower one built beside it.
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let left = Bf16Constant::apply(&mut graph, bits::ONE).unwrap();
    let right = Bf16Constant::apply(&mut graph, bits::TWO).unwrap();
    let product = Bf16Multiply::apply(&mut graph, left, right).unwrap();
    graph
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    let program = graph.build().unwrap();
    assert_eq!(
        bf16_bits(&evaluator.evaluate(&program, &[]).unwrap()[0]),
        vec![bits::TWO]
    );
}

fn constant_attributes(payload: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(BF16_CONSTANT_BITS_ATTRIBUTE, payload)])
        .expect("the constant attribute record is canonical")
}

fn apply_constant(
    reference: &ConstantBf16Reference,
    attributes: &OperationAttributes,
) -> Result<Vec<u16>, ReferenceOperationError> {
    let mut outputs = ReferenceOutputs::new(1, EvaluationRetention::default());
    let callback = reference.evaluate(
        ReferenceEvaluationRequest {
            operands: &[],
            attributes,
        },
        &mut outputs,
    );
    outputs.finish(callback).map(|values| bf16_bits(&values[0]))
}

fn bf16_tensor(shape: Shape, encodings: &[u16]) -> Tensor {
    Tensor::dense(
        Bf16::resolved_type(),
        shape,
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

fn bf16_bits(tensor: &Tensor) -> Vec<u16> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a bf16 result is dense");
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

/// Returns the record with one field replaced, for a deliberate perturbation.
fn with_field(
    facts: &CanonicalValue,
    id: AttributeFieldId,
    value: CanonicalValue,
) -> CanonicalValue {
    let mut fields = retained_fields(facts, id);
    fields.push(CanonicalField::new(id, value));
    CanonicalValue::record(fields).expect("a perturbed record is canonical")
}

/// Returns the record with one field removed, for a deliberate perturbation.
fn without_field(facts: &CanonicalValue, id: AttributeFieldId) -> CanonicalValue {
    CanonicalValue::record(retained_fields(facts, id)).expect("a perturbed record is canonical")
}

fn retained_fields(facts: &CanonicalValue, id: AttributeFieldId) -> Vec<CanonicalField> {
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("a governed fact record is a record");
    };
    fields
        .iter()
        .filter(|field| field.id() != id)
        .map(|field| CanonicalField::new(field.id(), field.value().clone()))
        .collect()
}
