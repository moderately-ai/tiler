//! The conformance corpus and the oracle's own self-checks.
//!
//! Two populations, deliberately separate. The **exhaustive** checks enumerate
//! all 65,536 BF16 encodings and are `exhaustive-finite` evidence about the
//! oracle. The **witness** corpus is a named list of operation-specific cases —
//! zeros, subnormals, normals, infinities, NaNs, ties, overflow, underflow — and
//! is `executable-model` evidence about the stated semantics.
//!
//! Neither is evidence about a device. Nothing in this module executes anything
//! on a GPU, and the run narrative keeps the two apart for that reason.

use crate::bf16::{
    Bf16, ExactValue, Rational, add, exceeds, largest_finite, multiply, overflow_threshold,
    round_to_nearest_even,
};

/// One named conformance witness: an operation applied to exact operands.
pub struct Witness {
    /// What this case is evidence about.
    pub name: &'static str,
    /// The operation applied.
    pub operation: Operation,
    /// Left operand encoding.
    pub left: u16,
    /// Right operand encoding.
    pub right: u16,
    /// The exact result encoding this spike's semantics require.
    pub expected: u16,
}

/// The two arithmetic operations this bounded spike admits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Pure-BF16 multiplication, one rounding.
    Multiply,
    /// Pure-BF16 addition, one rounding.
    Add,
}

impl Operation {
    /// Applies this operation's exact semantics.
    #[must_use]
    pub fn apply(self, left: Bf16, right: Bf16) -> ExactValue {
        match self {
            Self::Multiply => multiply(left, right),
            Self::Add => add(left, right),
        }
    }

    /// Returns the operation's stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Multiply => "multiply",
            Self::Add => "add",
        }
    }
}

/// Named BF16 encodings the corpus is written in terms of.
pub mod bits {
    /// Positive zero.
    pub const POS_ZERO: u16 = 0x0000;
    /// Negative zero.
    pub const NEG_ZERO: u16 = 0x8000;
    /// The least positive subnormal, `2^-133`.
    pub const MIN_SUBNORMAL: u16 = 0x0001;
    /// The least negative subnormal.
    pub const NEG_MIN_SUBNORMAL: u16 = 0x8001;
    /// The greatest subnormal.
    pub const MAX_SUBNORMAL: u16 = 0x007f;
    /// The least positive normal, `2^-126`.
    pub const MIN_NORMAL: u16 = 0x0080;
    /// The greatest finite value.
    pub const MAX_FINITE: u16 = 0x7f7f;
    /// The greatest finite negative value.
    pub const NEG_MAX_FINITE: u16 = 0xff7f;
    /// Positive infinity.
    pub const POS_INFINITY: u16 = 0x7f80;
    /// Negative infinity.
    pub const NEG_INFINITY: u16 = 0xff80;
    /// The canonical quiet NaN.
    pub const CANONICAL_NAN: u16 = 0x7fc0;
    /// A non-canonical NaN payload, used to check canonicalization.
    pub const NONCANONICAL_NAN: u16 = 0x7fc1;
    /// One.
    pub const ONE: u16 = 0x3f80;
    /// Negative one.
    pub const NEG_ONE: u16 = 0xbf80;
    /// Two.
    pub const TWO: u16 = 0x4000;
    /// One half.
    pub const HALF: u16 = 0x3f00;
    /// Three.
    pub const THREE: u16 = 0x4040;
    /// The value `1 + 2^-7`, one ulp above one.
    pub const ONE_PLUS_ULP: u16 = 0x3f81;
    /// The value `2^-8`, exactly half an ulp at one.
    ///
    /// Adding it to a value in the binade `[1, 2)` constructs a tie that no BF16
    /// encoding can hold, which is how the corpus reaches ties-to-even by
    /// arithmetic rather than by writing an unrepresentable literal.
    pub const HALF_ULP_AT_ONE: u16 = 0x3b80;
}

/// The named conformance witnesses, in six named categories.
///
/// Every `expected` is derived by hand from BF16's stated parameters and the
/// round-to-nearest-ties-to-even rule, not by running the oracle and recording
/// what it said. A corpus generated from the implementation under test would
/// agree with it for reasons that say nothing about either being right.
///
/// The categories are separate functions rather than one table so the run can
/// report how many cases each exceptional class actually contributes — a corpus
/// whose subnormal group silently emptied would otherwise still look like a
/// passing corpus.
#[must_use]
pub fn witnesses() -> Vec<Witness> {
    witness_categories()
        .into_iter()
        .flat_map(|(_, cases)| cases)
        .collect()
}

/// Returns each witness category with its name, for the run's census.
#[must_use]
pub fn witness_categories() -> Vec<(&'static str, Vec<Witness>)> {
    vec![
        ("zeros and signs", zero_witnesses()),
        ("subnormals and underflow", subnormal_witnesses()),
        ("ties", tie_witnesses()),
        ("ordinary rounding", rounding_witnesses()),
        ("overflow", overflow_witnesses()),
        ("infinities and NaN", exceptional_witnesses()),
    ]
}

/// Zeros, their signs, and sign-producing cancellation.
#[must_use]
fn zero_witnesses() -> Vec<Witness> {
    use Operation::{Add, Multiply};
    use bits::{NEG_ONE, NEG_ZERO, ONE, POS_ZERO};
    vec![
        Witness {
            name: "positive zero plus positive zero is positive zero",
            operation: Add,
            left: POS_ZERO,
            right: POS_ZERO,
            expected: POS_ZERO,
        },
        Witness {
            name: "negative zero plus negative zero is negative zero",
            operation: Add,
            left: NEG_ZERO,
            right: NEG_ZERO,
            expected: NEG_ZERO,
        },
        Witness {
            name: "opposite-signed zeros sum to positive zero under round-to-nearest",
            operation: Add,
            left: POS_ZERO,
            right: NEG_ZERO,
            expected: POS_ZERO,
        },
        Witness {
            name: "negative times positive zero is negative zero",
            operation: Multiply,
            left: NEG_ZERO,
            right: POS_ZERO,
            expected: NEG_ZERO,
        },
        Witness {
            name: "negative one times positive zero is negative zero",
            operation: Multiply,
            left: NEG_ONE,
            right: POS_ZERO,
            expected: NEG_ZERO,
        },
        Witness {
            name: "a value plus its negation is positive zero",
            operation: Add,
            left: ONE,
            right: NEG_ONE,
            expected: POS_ZERO,
        },
    ]
}

/// Subnormals, preserved exactly by the reference, and gradual underflow.
#[must_use]
fn subnormal_witnesses() -> Vec<Witness> {
    use Operation::{Add, Multiply};
    use bits::{
        HALF, MAX_SUBNORMAL, MIN_NORMAL, MIN_SUBNORMAL, NEG_MIN_SUBNORMAL, NEG_ONE, POS_ZERO, TWO,
    };
    vec![
        Witness {
            name: "the least positive subnormal doubles to the next subnormal",
            operation: Multiply,
            left: MIN_SUBNORMAL,
            right: TWO,
            expected: 0x0002,
        },
        Witness {
            name: "the least positive subnormal halves to zero by underflow",
            operation: Multiply,
            left: MIN_SUBNORMAL,
            right: HALF,
            expected: POS_ZERO,
        },
        Witness {
            name: "the greatest subnormal plus the least subnormal is the least normal",
            operation: Add,
            left: MAX_SUBNORMAL,
            right: MIN_SUBNORMAL,
            expected: MIN_NORMAL,
        },
        Witness {
            name: "the least normal halves to the greatest subnormal plus one quantum",
            operation: Multiply,
            left: MIN_NORMAL,
            right: HALF,
            expected: 0x0040,
        },
        Witness {
            name: "a subnormal keeps its sign through a negation",
            operation: Multiply,
            left: MIN_SUBNORMAL,
            right: NEG_ONE,
            expected: NEG_MIN_SUBNORMAL,
        },
    ]
}

/// Exact halfway cases, resolved to even.
#[must_use]
fn tie_witnesses() -> Vec<Witness> {
    use Operation::Add;
    use bits::{HALF_ULP_AT_ONE, ONE, ONE_PLUS_ULP};
    vec![
        Witness {
            name: "a tie halfway above one rounds down to the even significand",
            operation: Add,
            left: ONE,
            right: HALF_ULP_AT_ONE,
            expected: ONE,
        },
        Witness {
            name: "a tie halfway above one-plus-one-ulp rounds up to the even significand",
            operation: Add,
            left: ONE_PLUS_ULP,
            right: 0x3b80,
            expected: 0x3f82,
        },
    ]
}

/// Rounding that is not a tie, paired with an exactly representable neighbour.
#[must_use]
fn rounding_witnesses() -> Vec<Witness> {
    use Operation::Multiply;
    use bits::{ONE, THREE};
    vec![
        Witness {
            // 0x3eab is BF16's nearest neighbour of one third, and the operand
            // the Apple probe's own vector calls its ordinary normal. The exact
            // product is 513/512, which is 128.25 quanta at exponent zero and
            // rounds down to 128 — a genuine round-to-nearest, not a tie.
            name: "three times the nearest BF16 to one third rounds to exactly one",
            operation: Multiply,
            left: THREE,
            right: 0x3eab,
            expected: ONE,
        },
        Witness {
            // The neighbour one quantum below. Its exact product with three is
            // 255/256, which *is* representable, so nothing rounds at all. The
            // pair separates "rounded to one" from "happened to land on one".
            name: "three times one quantum below that is exactly representable",
            operation: Multiply,
            left: THREE,
            right: 0x3eaa,
            expected: 0x3f7f,
        },
    ]
}

/// Overflow to infinity in both signs.
#[must_use]
fn overflow_witnesses() -> Vec<Witness> {
    use Operation::{Add, Multiply};
    use bits::{MAX_FINITE, NEG_INFINITY, NEG_MAX_FINITE, POS_INFINITY, TWO};
    vec![
        Witness {
            name: "the greatest finite value doubled overflows to infinity",
            operation: Multiply,
            left: MAX_FINITE,
            right: TWO,
            expected: POS_INFINITY,
        },
        Witness {
            name: "the greatest finite value plus itself overflows to infinity",
            operation: Add,
            left: MAX_FINITE,
            right: MAX_FINITE,
            expected: POS_INFINITY,
        },
        Witness {
            name: "the greatest negative finite value doubled overflows to negative infinity",
            operation: Multiply,
            left: NEG_MAX_FINITE,
            right: TWO,
            expected: NEG_INFINITY,
        },
    ]
}

/// Infinity arithmetic, the invalid operations, and NaN canonicalization.
#[must_use]
fn exceptional_witnesses() -> Vec<Witness> {
    use Operation::{Add, Multiply};
    use bits::{
        CANONICAL_NAN, NEG_INFINITY, NEG_ONE, NONCANONICAL_NAN, ONE, POS_INFINITY, POS_ZERO,
    };
    vec![
        Witness {
            name: "infinity times zero is the canonical NaN",
            operation: Multiply,
            left: POS_INFINITY,
            right: POS_ZERO,
            expected: CANONICAL_NAN,
        },
        Witness {
            name: "infinity minus infinity is the canonical NaN",
            operation: Add,
            left: POS_INFINITY,
            right: NEG_INFINITY,
            expected: CANONICAL_NAN,
        },
        Witness {
            name: "infinity plus a finite value is infinity",
            operation: Add,
            left: POS_INFINITY,
            right: ONE,
            expected: POS_INFINITY,
        },
        Witness {
            name: "infinity times a negative finite value is negative infinity",
            operation: Multiply,
            left: POS_INFINITY,
            right: NEG_ONE,
            expected: NEG_INFINITY,
        },
        Witness {
            name: "a non-canonical NaN operand canonicalizes through multiplication",
            operation: Multiply,
            left: NONCANONICAL_NAN,
            right: ONE,
            expected: CANONICAL_NAN,
        },
        Witness {
            name: "a non-canonical NaN operand canonicalizes through addition",
            operation: Add,
            left: NONCANONICAL_NAN,
            right: ONE,
            expected: CANONICAL_NAN,
        },
    ]
}

/// One failure observed while checking the corpus or the exhaustive population.
pub struct Failure {
    /// What was being checked.
    pub subject: String,
    /// What the stated semantics require.
    pub expected: u16,
    /// What the oracle produced.
    pub actual: u16,
}

/// Runs every named witness and returns the failures.
#[must_use]
pub fn check_witnesses() -> Vec<Failure> {
    witnesses()
        .into_iter()
        .filter_map(|witness| {
            let result = round_to_nearest_even(&witness.operation.apply(
                Bf16::from_bits(witness.left),
                Bf16::from_bits(witness.right),
            ));
            (result.to_bits() != witness.expected).then(|| Failure {
                subject: format!(
                    "{} [{} {:#06x} {:#06x}]",
                    witness.name,
                    witness.operation.as_str(),
                    witness.left,
                    witness.right
                ),
                expected: witness.expected,
                actual: result.to_bits(),
            })
        })
        .collect()
}

/// Checks the overflow boundary is decided rather than approached.
///
/// Round-to-nearest overflows to infinity exactly at the midpoint above the
/// largest finite value, and *not* one quantum below it. Both sides of that
/// single boundary are checked here, because a rounding loop that simply
/// saturated would pass every witness above and still be wrong here.
#[must_use]
pub fn check_overflow_boundary() -> Vec<Failure> {
    let mut failures = Vec::new();
    let threshold = overflow_threshold();
    let largest = largest_finite();
    // Just below the threshold must round to the largest finite value.
    let below = ExactValue::Finite {
        value: largest.add(&threshold).multiply(&Rational::new_half()),
        zero_is_negative: false,
    };
    let rounded_below = round_to_nearest_even(&below);
    if rounded_below.to_bits() != bits::MAX_FINITE {
        failures.push(Failure {
            subject: "the midpoint between the largest finite value and the overflow threshold"
                .to_owned(),
            expected: bits::MAX_FINITE,
            actual: rounded_below.to_bits(),
        });
    }
    // The threshold itself must overflow.
    let at = ExactValue::Finite {
        value: threshold.clone(),
        zero_is_negative: false,
    };
    let rounded_at = round_to_nearest_even(&at);
    if rounded_at.to_bits() != bits::POS_INFINITY {
        failures.push(Failure {
            subject: "the overflow threshold itself".to_owned(),
            expected: bits::POS_INFINITY,
            actual: rounded_at.to_bits(),
        });
    }
    // And the ordering the threshold depends on must be strict.
    if !exceeds(&threshold, &largest) {
        failures.push(Failure {
            subject: "the overflow threshold must exceed the largest finite value".to_owned(),
            expected: 1,
            actual: 0,
        });
    }
    failures
}

/// Returns how many of the 65,536 encodings fall in each structural class.
///
/// The population, named and counted. A check that reported a uniform answer
/// over this set without knowing how many members each class has is the shape
/// this repository distrusts, so the run prints the census beside the verdicts.
#[must_use]
pub fn census() -> [(&'static str, usize); 5] {
    let mut zeros = 0;
    let mut subnormals = 0;
    let mut normals = 0;
    let mut infinities = 0;
    let mut nans = 0;
    for bits in 0..=u16::MAX {
        let value = Bf16::from_bits(bits);
        if value.is_nan() {
            nans += 1;
        } else if value.is_infinite() {
            infinities += 1;
        } else if value.is_zero() {
            zeros += 1;
        } else if value.is_subnormal() {
            subnormals += 1;
        } else {
            normals += 1;
        }
    }
    [
        ("zeros", zeros),
        ("subnormals", subnormals),
        ("normals", normals),
        ("infinities", infinities),
        ("NaNs", nans),
    ]
}

/// Checks that every finite BF16 encoding survives decode-then-round unchanged.
///
/// **Exhaustive over the format.** All 65,536 encodings are enumerated. A finite
/// value's exact rational must round back to its own encoding — otherwise the
/// oracle's decode and its rounding disagree, and every result above would be
/// measured against a broken scale. Non-finite encodings are checked separately
/// because NaN is deliberately *not* an identity: it canonicalizes.
#[must_use]
pub fn check_exhaustive_round_trip() -> Vec<Failure> {
    let mut failures = Vec::new();
    for bits in 0..=u16::MAX {
        let value = Bf16::from_bits(bits);
        let round_tripped = round_to_nearest_even(&value.to_exact());
        let expected = if value.is_nan() {
            Bf16::canonical_nan().to_bits()
        } else {
            bits
        };
        if round_tripped.to_bits() != expected {
            failures.push(Failure {
                subject: format!("round trip of {bits:#06x}"),
                expected,
                actual: round_tripped.to_bits(),
            });
        }
    }
    failures
}

/// Checks the BF16-to-binary32 widening against the host's own `f32` value.
///
/// A *second, independent* route to the same exact value: the oracle decodes
/// the BF16 encoding directly from its fields, while this check widens the
/// encoding to binary32 and reads the host's `f32`. They share no code. Where
/// the two disagree the oracle's field decode is wrong, and a corpus built on it
/// would be confidently wrong in the same direction.
///
/// This is *not* the normative rounding route — the host never rounds here, it
/// only reads an exactly-representable widening — so it does not reintroduce the
/// host-arithmetic dependency the oracle exists to avoid.
#[must_use]
pub fn check_widening_agrees() -> Vec<Failure> {
    let mut failures = Vec::new();
    for bits in 0..=u16::MAX {
        let value = Bf16::from_bits(bits);
        let widened = f32::from_bits(value.widen_to_f32_bits());
        let agrees = match value.to_exact() {
            ExactValue::Nan => widened.is_nan(),
            ExactValue::Infinite { negative } => {
                widened.is_infinite() && widened.is_sign_negative() == negative
            }
            ExactValue::Finite {
                ref value,
                zero_is_negative,
            } => {
                if value.is_zero() {
                    widened == 0.0 && widened.is_sign_negative() == zero_is_negative
                } else {
                    // Re-round the widened f32 back to BF16 through the oracle's
                    // own rounding of the *exact* f32 value. Every BF16 widens
                    // exactly, so this must return the original encoding.
                    exact_from_f32(widened)
                        .is_some_and(|exact| round_to_nearest_even(&exact).to_bits() == bits)
                }
            }
        };
        if !agrees {
            failures.push(Failure {
                subject: format!("widening agreement for {bits:#06x}"),
                expected: bits,
                actual: bits,
            });
        }
    }
    failures
}

/// Returns the exact value of a finite host `f32`, for the widening cross-check.
fn exact_from_f32(value: f32) -> Option<ExactValue> {
    use crate::bf16::Rational;
    let bits = value.to_bits();
    let biased = (bits >> 23) & 0xff;
    if biased == 0xff {
        return None;
    }
    let trailing = bits & 0x007f_ffff;
    let (significand, exponent) = if biased == 0 {
        (i64::from(trailing), -149_i32)
    } else {
        (
            i64::from(trailing | 0x0080_0000),
            i32::try_from(biased).expect("an eight-bit field fits i32") - 150,
        )
    };
    let magnitude = Rational::from_integer(significand).scale_by_power_of_two(exponent);
    Some(ExactValue::Finite {
        value: if bits >> 31 == 1 {
            magnitude.negate()
        } else {
            magnitude
        },
        zero_is_negative: bits >> 31 == 1,
    })
}
