//! The BF16 value set and its deterministic exact-rational reference oracle.
//!
//! # Why the oracle is exact rational rather than host arithmetic
//!
//! Every operation here is evaluated as an exact rational — no precision bound,
//! no intermediate format — and rounded exactly once, at the observable
//! materialization, by round-to-nearest-ties-to-even over BF16's value set. The
//! reason is that the oracle must not depend on an arithmetic argument about
//! intermediate widths, because that argument is *true for the two operations
//! this spike admits and false for the fused one it does not*.
//!
//! **Correction, `design-the-bf16-computation-and-accumulator-contract`.** This
//! comment previously argued that a host-`f32` oracle would be wrong because
//! `f32`'s 24-bit significand "does not exceed twice `bf16`'s 8-bit significand
//! by enough to make the second rounding innocuous". That reads the bound
//! backwards. Figueroa's condition for an innocuous double rounding of one
//! `+ - * /` or square root is `q >= 2p + 2`, which here is `24 >= 18` and
//! **holds** — which is exactly why finding 24 of the retained Apple record says
//! no *single* operation can expose an `f32` intermediate, and why
//! `crates/tiler-metal/src/target.rs` states the same inequality in the same
//! direction. [`crate::promotion`] now checks it directly: over 524,288 cases a
//! promoted route and one exact rounding never disagree for a multiply or an
//! add, and a precision-9 intermediate makes them disagree at once.
//!
//! The conclusion the original comment reached is unchanged and the reason is
//! replaced. An exact-rational oracle is the right choice because it is
//! independent of that bound rather than because the bound fails: the bound
//! covers neither a fused multiply-add — where [`crate::promotion`] exhibits
//! operands on which a promoted route differs by one ulp — nor an accumulation,
//! nor any future operation this spike's children add.
//!
//! # The format
//!
//! `tiler::bf16@1` as the governed catalog registers it: 16 bits, 1 sign bit, 8
//! exponent bits, 7 trailing significand bits, with infinities, NaN, zero,
//! signed zero, and subnormals all present. Its normative definition is the
//! ratified RISC-V BF16 operand format. The parameters are read from the
//! registered descriptor rather than restated, so a catalog change is a failure
//! here rather than a silent divergence.

use std::cmp::Ordering;

use num_bigint::{BigInt, Sign};
use num_traits::{One, Signed, Zero};

/// Sign bit count of the BF16 format.
pub const SIGN_BITS: u32 = 1;
/// Exponent field width of the BF16 format.
pub const EXPONENT_BITS: u32 = 8;
/// Trailing significand width of the BF16 format.
pub const TRAILING_SIGNIFICAND_BITS: u32 = 7;
/// Total encoded width of the BF16 format.
pub const WIDTH_BITS: u32 = 16;

/// Precision in bits: the trailing significand plus the implicit leading bit.
pub const PRECISION: u32 = TRAILING_SIGNIFICAND_BITS + 1;
/// Largest exponent of a finite BF16 value.
pub const MAX_EXPONENT: i32 = (1 << (EXPONENT_BITS - 1)) - 1;
/// Smallest exponent of a normal BF16 value.
pub const MIN_EXPONENT: i32 = 1 - MAX_EXPONENT;
/// Exponent bias of the BF16 encoding.
pub const EXPONENT_BIAS: i32 = MAX_EXPONENT;

/// The trailing significand width as a signed exponent offset.
///
/// Every exponent computation below shifts by `precision - 1`, which is exactly
/// the trailing significand width. Naming it once as a signed constant keeps the
/// arithmetic in one type instead of casting an unsigned width at five sites.
const SIGNIFICAND_OFFSET: i32 = 7;

// The signed offset and the unsigned width are the same number in two types, so
// this pins them together: moving one without the other is a build error rather
// than a silently reparameterized format.
const _: () = assert!(SIGNIFICAND_OFFSET.unsigned_abs() == TRAILING_SIGNIFICAND_BITS);

/// Exponent of the least significant bit of the smallest subnormal.
const MIN_SUBNORMAL_EXPONENT: i32 = MIN_EXPONENT - SIGNIFICAND_OFFSET;

/// One BF16 value, held as its exact 16-bit encoding.
///
/// A bit pattern rather than a host float: a NaN payload and the two zeros are
/// observable in this spike's comparisons, and a host type that canonicalized
/// either would erase the distinction the corpus exists to check.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bf16(u16);

/// An exact rational value, or one of the BF16 value set's non-finite members.
///
/// The reference evaluates in this domain. It is deliberately not "an
/// `ExactRational` plus a flag": an infinity is not a number with an attribute,
/// and every arithmetic rule below has to dispatch on the distinction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExactValue {
    /// A mathematically exact rational, sign-carrying, with an explicit zero sign.
    Finite {
        /// The exact value. Zero carries no sign here; `zero_is_negative` does.
        value: Rational,
        /// Which zero this is, meaningful only when `value` is exactly zero.
        zero_is_negative: bool,
    },
    /// A signed infinity.
    Infinite {
        /// `true` for negative infinity.
        negative: bool,
    },
    /// Not a number. The payload is not modelled; canonicalization is stated.
    Nan,
}

/// A minimal exact rational over arbitrary-precision integers.
///
/// `tiler_ir`'s `ExactRational` is the in-tree peer of this type and is public,
/// but its only float ingress is `from_f32`, so it cannot state a BF16 value
/// without going through a host `f32` first. This spike keeps its own so the
/// oracle's ingress is the BF16 encoding itself; the seam audit records the gap
/// rather than working around it silently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

impl Rational {
    /// Returns exactly one half, the midpoint helper the overflow check needs.
    #[must_use]
    pub fn new_half() -> Self {
        Self::new(BigInt::one(), BigInt::from(2))
    }

    /// Returns exact zero.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            numerator: BigInt::zero(),
            denominator: BigInt::one(),
        }
    }

    /// Creates an exact rational from a signed integer.
    #[must_use]
    pub fn from_integer(value: i64) -> Self {
        Self {
            numerator: BigInt::from(value),
            denominator: BigInt::one(),
        }
    }

    /// Creates `numerator / denominator`, normalized to lowest terms.
    ///
    /// # Panics
    ///
    /// Panics when `denominator` is zero, which no caller here constructs.
    #[must_use]
    pub fn new(numerator: BigInt, denominator: BigInt) -> Self {
        assert!(!denominator.is_zero(), "a rational denominator is nonzero");
        let mut numerator = numerator;
        let mut denominator = denominator;
        if denominator.sign() == Sign::Minus {
            numerator = -numerator;
            denominator = -denominator;
        }
        let divisor = gcd(&numerator, &denominator);
        if !divisor.is_one() {
            numerator /= &divisor;
            denominator /= &divisor;
        }
        Self {
            numerator,
            denominator,
        }
    }

    /// Returns `self * 2^exponent` exactly.
    #[must_use]
    pub fn scale_by_power_of_two(&self, exponent: i32) -> Self {
        if exponent >= 0 {
            let shift = u32::try_from(exponent).expect("a nonnegative exponent fits u32");
            Self::new(self.numerator.clone() << shift, self.denominator.clone())
        } else {
            let shift = u32::try_from(-exponent).expect("a negated negative exponent fits u32");
            Self::new(self.numerator.clone(), self.denominator.clone() << shift)
        }
    }

    /// Returns the exact sum.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::new(
            &self.numerator * &other.denominator + &other.numerator * &self.denominator,
            &self.denominator * &other.denominator,
        )
    }

    /// Returns the exact product.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self::new(
            &self.numerator * &other.numerator,
            &self.denominator * &other.denominator,
        )
    }

    /// Returns the exact negation.
    #[must_use]
    pub fn negate(&self) -> Self {
        Self {
            numerator: -self.numerator.clone(),
            denominator: self.denominator.clone(),
        }
    }

    /// Returns whether this value is exactly zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.numerator.is_zero()
    }

    /// Returns whether this value is strictly negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.numerator.sign() == Sign::Minus
    }

    /// Returns the exact absolute value.
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            numerator: self.numerator.abs(),
            denominator: self.denominator.clone(),
        }
    }

    /// Returns the normalized numerator.
    #[must_use]
    pub fn numerator(&self) -> &BigInt {
        &self.numerator
    }

    /// Returns the normalized, strictly positive denominator.
    #[must_use]
    pub fn denominator(&self) -> &BigInt {
        &self.denominator
    }
}

fn gcd(left: &BigInt, right: &BigInt) -> BigInt {
    let mut a = left.abs();
    let mut b = right.abs();
    while !b.is_zero() {
        let remainder = &a % &b;
        a = b;
        b = remainder;
    }
    a
}

impl Bf16 {
    /// Creates a BF16 from its exact 16-bit encoding.
    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    /// Returns the exact 16-bit encoding.
    #[must_use]
    pub const fn to_bits(self) -> u16 {
        self.0
    }

    /// Returns the canonical quiet NaN this spike's realization produces.
    ///
    /// Stated rather than inherited: an arithmetic NaN result is canonicalized,
    /// so the corpus can assert an exact bit pattern instead of accepting any
    /// NaN. `0x7fc0` is the BF16 pattern whose `f32` widening is `0x7fc00000`,
    /// the canonical quiet NaN `tiler-reference` already names for binary32.
    #[must_use]
    pub const fn canonical_nan() -> Self {
        Self(0x7fc0)
    }

    /// Returns whether this encoding is a NaN.
    #[must_use]
    pub const fn is_nan(self) -> bool {
        self.biased_exponent() == 0xff && self.trailing_significand() != 0
    }

    /// Returns whether this encoding is an infinity.
    #[must_use]
    pub const fn is_infinite(self) -> bool {
        self.biased_exponent() == 0xff && self.trailing_significand() == 0
    }

    /// Returns whether this encoding is a subnormal (including neither zero).
    #[must_use]
    pub const fn is_subnormal(self) -> bool {
        self.biased_exponent() == 0 && self.trailing_significand() != 0
    }

    /// Returns whether this encoding is either zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.biased_exponent() == 0 && self.trailing_significand() == 0
    }

    /// Returns whether the sign bit is set.
    #[must_use]
    pub const fn is_sign_negative(self) -> bool {
        self.0 >> 15 == 1
    }

    const fn biased_exponent(self) -> u16 {
        (self.0 >> 7) & 0xff
    }

    const fn trailing_significand(self) -> u16 {
        self.0 & 0x7f
    }

    /// Widens this value to the exactly equal binary32 encoding.
    ///
    /// **Exact and total.** Every BF16 encoding is the high 16 bits of a binary32
    /// encoding denoting the same value, including every subnormal, both zeros,
    /// both infinities, and every NaN payload. This is a property of BF16's
    /// parameters — same exponent width and bias as binary32 — and not a general
    /// float widening; it is what lets the device kernel and the host agree on a
    /// carrier without a conversion operation this spike has not defined.
    #[must_use]
    pub const fn widen_to_f32_bits(self) -> u32 {
        (self.0 as u32) << 16
    }

    /// Returns this encoding's exact mathematical value.
    #[must_use]
    pub fn to_exact(self) -> ExactValue {
        if self.is_nan() {
            return ExactValue::Nan;
        }
        if self.is_infinite() {
            return ExactValue::Infinite {
                negative: self.is_sign_negative(),
            };
        }
        let negative = self.is_sign_negative();
        if self.is_zero() {
            return ExactValue::Finite {
                value: Rational::zero(),
                zero_is_negative: negative,
            };
        }
        let (significand, exponent) = if self.biased_exponent() == 0 {
            (
                i64::from(self.trailing_significand()),
                MIN_SUBNORMAL_EXPONENT,
            )
        } else {
            (
                i64::from(self.trailing_significand()) | (1 << TRAILING_SIGNIFICAND_BITS),
                i32::from(self.biased_exponent()) - EXPONENT_BIAS - SIGNIFICAND_OFFSET,
            )
        };
        let magnitude = Rational::from_integer(significand).scale_by_power_of_two(exponent);
        ExactValue::Finite {
            value: if negative {
                magnitude.negate()
            } else {
                magnitude
            },
            zero_is_negative: false,
        }
    }
}

/// The largest finite BF16 magnitude, exactly.
#[must_use]
pub fn largest_finite() -> Rational {
    // (2 - 2^-(p-1)) * 2^emax, stated as an exact integer significand scaled.
    let significand = (1_i64 << PRECISION) - 1;
    Rational::from_integer(significand).scale_by_power_of_two(MAX_EXPONENT - SIGNIFICAND_OFFSET)
}

/// The overflow threshold: the midpoint above the largest finite value.
///
/// Round-to-nearest-ties-to-even overflows to infinity exactly when the exact
/// magnitude is at or above this bound — the same rule IEEE 754 states, applied
/// to BF16's parameters. Stating the threshold explicitly is what keeps the
/// boundary case (`value == threshold` overflows; just below it does not) a
/// decided rule rather than an accident of the rounding loop.
#[must_use]
pub fn overflow_threshold() -> Rational {
    let significand = (1_i64 << PRECISION) * 2 - 1;
    Rational::from_integer(significand).scale_by_power_of_two(MAX_EXPONENT - SIGNIFICAND_OFFSET - 1)
}

/// How a rounding rule resolves an exact halfway case.
///
/// The spike's normative rule is [`TieRule::ToEven`]. The other variant exists
/// so a perturbation can change *exactly* the tie decision and nothing else,
/// which is what makes the resulting disagreement evidence about ties-to-even
/// rather than about a second rounding implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TieRule {
    /// IEEE 754's default: a tie goes to the even significand.
    ToEven,
    /// A tie goes to the larger magnitude. Not this spike's semantics.
    AwayFromZero,
}

/// Rounds one exact value into BF16 by round-to-nearest-ties-to-even.
///
/// This is the spike's single observable-materialization rule. It is applied
/// exactly once per materialized result, never per intermediate.
#[must_use]
pub fn round_to_nearest_even(value: &ExactValue) -> Bf16 {
    round_with_tie_rule(value, TieRule::ToEven)
}

/// Rounds one exact value into BF16 under an explicit tie rule.
#[must_use]
pub fn round_with_tie_rule(value: &ExactValue, tie: TieRule) -> Bf16 {
    match value {
        ExactValue::Nan => Bf16::canonical_nan(),
        ExactValue::Infinite { negative } => {
            Bf16::from_bits(if *negative { 0xff80 } else { 0x7f80 })
        }
        ExactValue::Finite {
            value,
            zero_is_negative,
        } => {
            if value.is_zero() {
                return Bf16::from_bits(if *zero_is_negative { 0x8000 } else { 0x0000 });
            }
            let negative = value.is_negative();
            let magnitude = value.abs();
            let sign_bits = u16::from(negative) << 15;
            if !less_than(&magnitude, &overflow_threshold()) {
                return Bf16::from_bits(sign_bits | 0x7f80);
            }
            Bf16::from_bits(sign_bits | round_magnitude(&magnitude, tie))
        }
    }
}

/// Returns whether `left < right` for two nonnegative exact rationals.
fn less_than(left: &Rational, right: &Rational) -> bool {
    left.numerator() * right.denominator() < right.numerator() * left.denominator()
}

/// Returns whether `left > right` for two nonnegative exact rationals.
#[must_use]
pub fn exceeds(left: &Rational, right: &Rational) -> bool {
    less_than(right, left)
}

/// Rounds a strictly positive exact magnitude to BF16's significand and exponent.
///
/// Returns the encoding's low fifteen bits: biased exponent and trailing
/// significand, with no sign.
fn round_magnitude(magnitude: &Rational, tie: TieRule) -> u16 {
    // Find the unique exponent e with 2^e <= magnitude < 2^(e+1), then clamp it
    // to the subnormal floor so the quantum below is the format's, not the
    // value's. A binary search on the exponent avoids assuming the magnitude
    // fits any host type.
    let mut exponent = binade_exponent(magnitude).max(MIN_EXPONENT);
    // The quantum at this exponent, and the integer count of quanta, rounded to
    // nearest with ties to even. Computed as an exact integer division with an
    // explicit remainder comparison rather than a float, so the tie is decided
    // rather than inherited.
    let mut quanta = round_quanta(magnitude, exponent, tie);
    // Rounding up can carry into the next binade (0x7f -> 0x80 of significand),
    // which is a legal single step and needs the exponent incremented once.
    if quanta >= (1_i64 << PRECISION) {
        exponent += 1;
        quanta = round_quanta(magnitude, exponent, tie);
    }
    encode_magnitude(quanta, exponent)
}

/// Returns the exponent `e` with `2^e <= magnitude < 2^(e+1)`.
fn binade_exponent(magnitude: &Rational) -> i32 {
    let mut low = MIN_SUBNORMAL_EXPONENT - 1;
    let mut high = MAX_EXPONENT + 2;
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if less_than(
            magnitude,
            &Rational::from_integer(1).scale_by_power_of_two(middle),
        ) {
            high = middle;
        } else {
            low = middle;
        }
    }
    low
}

/// Returns the count of quanta at `exponent`, rounded to nearest, ties to even.
fn round_quanta(magnitude: &Rational, exponent: i32, tie: TieRule) -> i64 {
    let quantum_exponent = exponent - SIGNIFICAND_OFFSET;
    let scaled = magnitude.scale_by_power_of_two(-quantum_exponent);
    let numerator = scaled.numerator();
    let denominator = scaled.denominator();
    let floor = numerator / denominator;
    let remainder = numerator - &floor * denominator;
    let twice_remainder: BigInt = &remainder * 2;
    let rounded = match twice_remainder.cmp(denominator) {
        Ordering::Greater => &floor + 1,
        Ordering::Less => floor.clone(),
        // The exact halfway case, and the only place the two rules differ:
        // ties-to-even keeps an already-even quantum count, and every other
        // combination steps up.
        Ordering::Equal => match tie {
            TieRule::ToEven if (&floor % 2_i32).is_zero() => floor.clone(),
            TieRule::ToEven | TieRule::AwayFromZero => &floor + 1,
        },
    };
    i64::try_from(rounded).expect("a BF16 quantum count fits i64")
}

/// Encodes a rounded quantum count at an exponent into BF16's low fifteen bits.
fn encode_magnitude(quanta: i64, exponent: i32) -> u16 {
    if exponent <= MIN_EXPONENT && quanta < (1_i64 << (PRECISION - 1)) {
        // Subnormal: biased exponent zero, the quanta are the trailing bits.
        return u16::try_from(quanta).expect("a BF16 subnormal significand fits u16");
    }
    let biased = u16::try_from(exponent + EXPONENT_BIAS).expect("a BF16 biased exponent fits u16");
    let trailing = u16::try_from(quanta).expect("a BF16 significand fits u16")
        & ((1 << TRAILING_SIGNIFICAND_BITS) - 1);
    (biased << TRAILING_SIGNIFICAND_BITS) | trailing
}

/// Multiplies two BF16 values under this spike's exact semantics.
///
/// The product is exact and is rounded once. NaN propagates as the canonical
/// NaN; the exceptional-value rules are IEEE 754's for the cases this spike
/// admits, stated here rather than inherited from a host type.
#[must_use]
pub fn multiply(left: Bf16, right: Bf16) -> ExactValue {
    exact_multiply(&left.to_exact(), &right.to_exact())
}

/// Multiplies two already-decoded exact values, exactly and without rounding.
///
/// The operand-decoding half of [`multiply`] is separated from its arithmetic
/// half because a promoted route composes *values*, not encodings: an operand
/// that reached this point through a widening has no BF16 encoding to be
/// re-decoded from, and re-encoding it to reuse [`multiply`] would insert the
/// very rounding the route exists to avoid.
#[must_use]
pub fn exact_multiply(left_value: &ExactValue, right_value: &ExactValue) -> ExactValue {
    match (left_value, right_value) {
        (ExactValue::Nan, _) | (_, ExactValue::Nan) => ExactValue::Nan,
        (ExactValue::Infinite { negative: a }, ExactValue::Infinite { negative: b }) => {
            ExactValue::Infinite { negative: a != b }
        }
        (ExactValue::Infinite { negative }, ExactValue::Finite { value, .. })
        | (ExactValue::Finite { value, .. }, ExactValue::Infinite { negative }) => {
            if value.is_zero() {
                // Infinity times zero is an invalid operation; its default
                // result is a quiet NaN.
                ExactValue::Nan
            } else {
                ExactValue::Infinite {
                    negative: negative != &value.is_negative(),
                }
            }
        }
        (
            ExactValue::Finite {
                value: a,
                zero_is_negative: a_zero_negative,
            },
            ExactValue::Finite {
                value: b,
                zero_is_negative: b_zero_negative,
            },
        ) => {
            let product = a.multiply(b);
            // The product's zero sign is the exclusive-or of the operand signs,
            // which for a zero operand is carried by its zero sign rather than
            // by the (unsigned) rational zero.
            let a_negative = a.is_negative() || (a.is_zero() && *a_zero_negative);
            let b_negative = b.is_negative() || (b.is_zero() && *b_zero_negative);
            ExactValue::Finite {
                value: product,
                zero_is_negative: a_negative != b_negative,
            }
        }
    }
}

/// Adds two BF16 values under this spike's exact semantics.
#[must_use]
pub fn add(left: Bf16, right: Bf16) -> ExactValue {
    exact_add(&left.to_exact(), &right.to_exact())
}

/// Adds two already-decoded exact values, exactly and without rounding.
///
/// Separated from [`add`] for the reason [`exact_multiply`] states.
#[must_use]
pub fn exact_add(left_value: &ExactValue, right_value: &ExactValue) -> ExactValue {
    match (left_value, right_value) {
        (ExactValue::Nan, _) | (_, ExactValue::Nan) => ExactValue::Nan,
        (ExactValue::Infinite { negative: a }, ExactValue::Infinite { negative: b }) => {
            if a == b {
                ExactValue::Infinite { negative: *a }
            } else {
                // Infinity minus infinity is invalid; the default is quiet NaN.
                ExactValue::Nan
            }
        }
        (ExactValue::Infinite { negative }, ExactValue::Finite { .. })
        | (ExactValue::Finite { .. }, ExactValue::Infinite { negative }) => ExactValue::Infinite {
            negative: *negative,
        },
        (
            ExactValue::Finite {
                value: a,
                zero_is_negative: a_zero_negative,
            },
            ExactValue::Finite {
                value: b,
                zero_is_negative: b_zero_negative,
            },
        ) => {
            let sum = a.add(b);
            // IEEE 754: the sum of two zeros is negative only when both are
            // negative (under round-to-nearest); a nonzero exact sum carries its
            // own sign, and an exact-zero sum of opposite-signed nonzeros is +0.
            let zero_is_negative = if a.is_zero() && b.is_zero() {
                *a_zero_negative && *b_zero_negative
            } else {
                false
            };
            ExactValue::Finite {
                value: sum,
                zero_is_negative,
            }
        }
    }
}
