//! Exact rational arithmetic and the nonnegative tolerance built on it.
//!
//! ADR 0042 requires that "every tolerance is a canonical exact nonnegative
//! number, initially an integer or rational, **never a host floating-point
//! literal**", and that the inclusive accuracy comparison be "evaluated exactly
//! or with certified bounds rather than by rounded floating-point division".
//! Both sentences are about the same hazard: a tolerance stored as an `f64` is
//! already a different number from the one written, and a comparison performed in
//! floating point can answer the accuracy question wrongly in the direction that
//! accepts a violating implementation.
//!
//! So this module is the exact-arithmetic floor the whole vocabulary stands on.
//! It exposes no machine-width or wrapping operation, exactly as
//! [`crate::index::IndexInteger`] exposes none, and for the same reason: a value
//! that can silently wrap is not an exact value.
//!
//! **Two types, deliberately.** [`ExactRational`] is a signed exact number — the
//! mathematical value of a candidate, the endpoint of a domain interval, a ULP
//! scale. [`ExactTolerance`] is the nonnegative subset, and it is a separate type
//! because "nonnegative" is a contract obligation ADR 0042 states rather than a
//! property a caller is trusted to preserve: a negative tolerance denotes an
//! unsatisfiable predicate, and constructing one must be a typed refusal rather
//! than a contract nothing can conform to.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

use crate::identity::push_len;
use crate::semantic::{AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView};

use super::error::{AccuracyAttributeSubject, AccuracyContractError, malformed};

/// Exact-rational record field carrying the canonical sign.
pub(super) const EXACT_RATIONAL_SIGN: AttributeFieldId = AttributeFieldId::new(1);
/// Exact-rational record field carrying the big-endian numerator magnitude.
pub(super) const EXACT_RATIONAL_NUMERATOR: AttributeFieldId = AttributeFieldId::new(2);
/// Exact-rational record field carrying the big-endian denominator magnitude.
pub(super) const EXACT_RATIONAL_DENOMINATOR: AttributeFieldId = AttributeFieldId::new(3);

/// Maximum big-endian magnitude bytes one decoded exact rational component may carry.
///
/// A decode boundary needs a bound, because an exact rational's magnitude is
/// unbounded by construction and a canonical attribute arrives from outside. The
/// value admits numbers far past any representable format's range while keeping
/// one component's decode work bounded.
pub const MAX_EXACT_RATIONAL_MAGNITUDE_BYTES: usize = 4_096;

/// Canonical sign of an exact rational.
///
/// Distinct from [`crate::index::IndexIntegerSign`], which is the sign of an
/// index-space integer: the two domains never mix, and a shared type would let a
/// coordinate be used where a tolerance belongs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExactSign {
    /// A strictly negative value.
    Negative,
    /// Zero, whose canonical magnitude is empty.
    Zero,
    /// A strictly positive value.
    Positive,
}

impl fmt::Display for ExactSign {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Negative => "negative",
            Self::Zero => "zero",
            Self::Positive => "positive",
        })
    }
}

/// A typed refusal of an exact rational or tolerance.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExactRationalError {
    /// A rational was constructed with a zero denominator.
    ZeroDenominator,
    /// A tolerance was constructed from a strictly negative value.
    NegativeTolerance {
        /// The rejected value.
        value: ExactRational,
    },
    /// A reciprocal or division by zero was requested.
    DivisionByZero,
    /// A square root of a strictly negative value was requested.
    NegativeSquareRoot {
        /// The rejected radicand.
        value: ExactRational,
    },
    /// A decoded magnitude exceeded the canonical component bound.
    MagnitudeTooLong {
        /// Actual byte count.
        bytes: usize,
    },
    /// A decoded magnitude carried a redundant leading zero byte.
    NoncanonicalLeadingZero,
    /// A decoded zero was paired with a nonempty numerator magnitude.
    NonemptyZeroMagnitude,
    /// A decoded nonzero sign was paired with an empty numerator magnitude.
    EmptyNonzeroMagnitude,
    /// A decoded value was not in lowest terms, so it is a second spelling of one number.
    NotInLowestTerms,
}

impl ExactRationalError {
    /// Returns the stable provider diagnostic code naming this refusal.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::ZeroDenominator => "accuracy.rational.zero-denominator",
            Self::NegativeTolerance { .. } => "accuracy.tolerance.negative",
            Self::DivisionByZero => "accuracy.rational.division-by-zero",
            Self::NegativeSquareRoot { .. } => "accuracy.rational.negative-square-root",
            Self::MagnitudeTooLong { .. } => "accuracy.rational.magnitude-too-long",
            Self::NoncanonicalLeadingZero => "accuracy.rational.noncanonical-leading-zero",
            Self::NonemptyZeroMagnitude => "accuracy.rational.nonempty-zero-magnitude",
            Self::EmptyNonzeroMagnitude => "accuracy.rational.empty-nonzero-magnitude",
            Self::NotInLowestTerms => "accuracy.rational.not-in-lowest-terms",
        }
    }
}

impl fmt::Display for ExactRationalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDenominator => {
                formatter.write_str("an exact rational requires a nonzero denominator")
            }
            Self::NegativeTolerance { value } => {
                write!(
                    formatter,
                    "a tolerance must be nonnegative, and {value} is not"
                )
            }
            Self::DivisionByZero => formatter.write_str("exact division by zero is undefined"),
            Self::NegativeSquareRoot { value } => {
                write!(formatter, "the square root of {value} is not real")
            }
            Self::MagnitudeTooLong { bytes } => write!(
                formatter,
                "an exact rational component has {bytes} magnitude bytes, exceeding {MAX_EXACT_RATIONAL_MAGNITUDE_BYTES}"
            ),
            Self::NoncanonicalLeadingZero => {
                formatter.write_str("an exact rational magnitude cannot carry a leading zero byte")
            }
            Self::NonemptyZeroMagnitude => {
                formatter.write_str("a zero numerator must use an empty canonical magnitude")
            }
            Self::EmptyNonzeroMagnitude => {
                formatter.write_str("a nonzero numerator requires a canonical magnitude")
            }
            Self::NotInLowestTerms => formatter.write_str(
                "an exact rational must be in lowest terms, or one number has two spellings",
            ),
        }
    }
}

impl Error for ExactRationalError {}

/// An exact signed rational number in lowest terms.
///
/// The denominator is strictly positive and coprime with the numerator's
/// magnitude, so **one number has exactly one representation** — which is what
/// lets the canonical encoding be an identity rather than a serialization. Every
/// operation below re-normalizes, so that invariant is a property of the type
/// rather than of its callers.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExactRational {
    numerator: BigInt,
    denominator: BigUint,
}

impl ExactRational {
    /// Returns exact zero.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            numerator: BigInt::zero(),
            denominator: BigUint::one(),
        }
    }

    /// Returns exact one.
    #[must_use]
    pub fn one() -> Self {
        Self {
            numerator: BigInt::one(),
            denominator: BigUint::one(),
        }
    }

    /// Creates an exact rational from a signed fixed-width host integer.
    #[must_use]
    pub fn from_integer(value: i128) -> Self {
        Self {
            numerator: BigInt::from(value),
            denominator: BigUint::one(),
        }
    }

    /// Creates an exact rational from a signed fixed-width numerator and denominator.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError::ZeroDenominator`] when `denominator` is zero.
    pub fn from_ratio(numerator: i128, denominator: i128) -> Result<Self, ExactRationalError> {
        if denominator == 0 {
            return Err(ExactRationalError::ZeroDenominator);
        }
        let sign = if denominator < 0 { -1 } else { 1 };
        Ok(Self::normalize(
            BigInt::from(numerator) * sign,
            BigInt::from(denominator).magnitude().clone(),
        ))
    }

    /// Returns the exact value of two raised to `exponent`, for any signed exponent.
    #[must_use]
    pub fn power_of_two(exponent: i32) -> Self {
        Self::one().scale_by_power_of_two(exponent)
    }

    /// Returns the exact value of one finite binary32 value.
    ///
    /// `None` for a NaN or an infinity, which have no exact rational value. This
    /// is the boundary a reference oracle crosses to compare a candidate's
    /// mathematical value `z` against an exact or enclosed reference `r`.
    ///
    /// # Panics
    ///
    /// Panics only if a binary32 exponent field stops fitting in an `i32` or its
    /// significand stops fitting in an `i128`, which the format's fixed widths
    /// make unreachable. The conversions are checked rather than cast so that a
    /// future widening fails loudly here instead of silently truncating a value.
    #[must_use]
    pub fn from_f32(value: f32) -> Option<Self> {
        let bits = value.to_bits();
        let biased_exponent = (bits >> 23) & 0xff;
        let trailing = bits & 0x007f_ffff;
        if biased_exponent == 0xff {
            return None;
        }
        let (significand, exponent) = if biased_exponent == 0 {
            (u128::from(trailing), -149_i32)
        } else {
            (
                u128::from(trailing | 0x0080_0000),
                i32::try_from(biased_exponent).expect("an eight-bit field fits i32") - 150,
            )
        };
        let magnitude = Self::from_integer(
            i128::try_from(significand).expect("a binary32 significand fits i128"),
        )
        .scale_by_power_of_two(exponent);
        Some(if bits >> 31 == 1 {
            magnitude.negate()
        } else {
            magnitude
        })
    }

    /// Decodes one canonical sign-and-magnitude rational.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError`] for an over-long magnitude, a redundant
    /// leading zero, a sign that disagrees with the numerator magnitude, a zero
    /// denominator, or a value that is not already in lowest terms. The last is a
    /// refusal rather than a renormalization for the reason
    /// [`crate::semantic::ContractionStructureError::NonCanonicalNumbering`]
    /// records: the encoding is the identity, so admitting a second spelling of
    /// one number would give one number two identities.
    pub fn from_sign_magnitude_ratio(
        sign: ExactSign,
        numerator_magnitude: &[u8],
        denominator_magnitude: &[u8],
    ) -> Result<Self, ExactRationalError> {
        validate_magnitude(numerator_magnitude)?;
        validate_magnitude(denominator_magnitude)?;
        match sign {
            ExactSign::Zero if !numerator_magnitude.is_empty() => {
                return Err(ExactRationalError::NonemptyZeroMagnitude);
            }
            ExactSign::Negative | ExactSign::Positive if numerator_magnitude.is_empty() => {
                return Err(ExactRationalError::EmptyNonzeroMagnitude);
            }
            _ => {}
        }
        if denominator_magnitude.is_empty() {
            return Err(ExactRationalError::ZeroDenominator);
        }
        let denominator = BigUint::from_bytes_be(denominator_magnitude);
        if denominator.is_zero() {
            return Err(ExactRationalError::ZeroDenominator);
        }
        let magnitude = BigUint::from_bytes_be(numerator_magnitude);
        // Through [`reduction_divisor`] rather than `gcd` for the same answer at
        // bounded cost. This is a decode boundary, so the widest dyadic pair
        // [`MAX_EXACT_RATIONAL_MAGNITUDE_BYTES`] admits is one an outside caller
        // may present, and the general algorithm's work on it is quadratic in a
        // width the bound deliberately allows to be large.
        if reduction_divisor(&magnitude, &denominator) != BigUint::one() {
            return Err(ExactRationalError::NotInLowestTerms);
        }
        if magnitude.is_zero() && denominator != BigUint::one() {
            return Err(ExactRationalError::NotInLowestTerms);
        }
        let numerator = BigInt::from_biguint(
            match sign {
                ExactSign::Negative => Sign::Minus,
                ExactSign::Zero => Sign::NoSign,
                ExactSign::Positive => Sign::Plus,
            },
            magnitude,
        );
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// Returns the canonical sign and the big-endian numerator and denominator magnitudes.
    #[must_use]
    pub fn to_sign_magnitude_ratio(&self) -> (ExactSign, Vec<u8>, Vec<u8>) {
        let (sign, magnitude) = self.numerator.to_bytes_be();
        let (sign, magnitude) = match sign {
            Sign::Minus => (ExactSign::Negative, magnitude),
            Sign::NoSign => (ExactSign::Zero, Vec::new()),
            Sign::Plus => (ExactSign::Positive, magnitude),
        };
        (sign, magnitude, self.denominator.to_bytes_be())
    }

    /// Returns the canonical sign of this value.
    #[must_use]
    pub fn sign(&self) -> ExactSign {
        match self.numerator.sign() {
            Sign::Minus => ExactSign::Negative,
            Sign::NoSign => ExactSign::Zero,
            Sign::Plus => ExactSign::Positive,
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
        self.numerator.is_negative()
    }

    /// Returns the exact magnitude.
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            numerator: self.numerator.abs(),
            denominator: self.denominator.clone(),
        }
    }

    /// Returns the exact additive inverse.
    #[must_use]
    pub fn negate(&self) -> Self {
        Self {
            numerator: -self.numerator.clone(),
            denominator: self.denominator.clone(),
        }
    }

    /// Returns the exact sum.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::normalize(
            &self.numerator * to_int(&other.denominator)
                + &other.numerator * to_int(&self.denominator),
            &self.denominator * &other.denominator,
        )
    }

    /// Returns the exact difference.
    #[must_use]
    pub fn subtract(&self, other: &Self) -> Self {
        self.add(&other.negate())
    }

    /// Returns the exact product.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self::normalize(
            &self.numerator * &other.numerator,
            &self.denominator * &other.denominator,
        )
    }

    /// Returns the exact quotient.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError::DivisionByZero`] when `other` is zero.
    pub fn divide(&self, other: &Self) -> Result<Self, ExactRationalError> {
        Ok(self.multiply(&other.reciprocal()?))
    }

    /// Returns the exact multiplicative inverse.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError::DivisionByZero`] when this value is zero.
    pub fn reciprocal(&self) -> Result<Self, ExactRationalError> {
        if self.is_zero() {
            return Err(ExactRationalError::DivisionByZero);
        }
        let (sign, magnitude, denominator) = self.to_sign_magnitude_ratio();
        let numerator = BigInt::from_biguint(
            match sign {
                ExactSign::Negative => Sign::Minus,
                ExactSign::Zero => Sign::NoSign,
                ExactSign::Positive => Sign::Plus,
            },
            BigUint::from_bytes_be(&denominator),
        );
        Ok(Self {
            numerator,
            denominator: BigUint::from_bytes_be(&magnitude),
        })
    }

    /// Returns this value multiplied by two raised to `exponent`.
    ///
    /// Exact for any signed exponent, which is what makes every ULP scale in this
    /// vocabulary an exact number rather than a rounded one.
    #[must_use]
    pub fn scale_by_power_of_two(&self, exponent: i32) -> Self {
        let shift = u64::from(exponent.unsigned_abs());
        if exponent >= 0 {
            Self::normalize(&self.numerator << shift, self.denominator.clone())
        } else {
            Self::normalize(self.numerator.clone(), &self.denominator << shift)
        }
    }

    /// Returns this value raised to a nonnegative integer power.
    #[must_use]
    pub fn power(&self, exponent: u32) -> Self {
        Self::normalize(self.numerator.pow(exponent), self.denominator.pow(exponent))
    }

    /// Returns the largest multiple of `2^-fraction_bits` that does not exceed this value.
    ///
    /// Outward rounding onto a fixed binary grid is how a certified enclosure
    /// stays bounded: exact rational arithmetic is exact but its magnitudes grow
    /// without limit, and an enclosure that is *widened* onto a grid is still an
    /// enclosure while an unbounded one is a resource hazard.
    ///
    /// # Panics
    ///
    /// Panics when `fraction_bits` exceeds `i32::MAX`, which is far past any grid
    /// a bounded machine could allocate. Checked rather than cast so that a
    /// nonsensical width fails here instead of silently negating.
    #[must_use]
    pub fn floor_to_binary_grid(&self, fraction_bits: u32) -> Self {
        let scaled = &self.numerator << u64::from(fraction_bits);
        let quotient = scaled.div_floor(&to_int(&self.denominator));
        Self::normalize(quotient, BigUint::one())
            .scale_by_power_of_two(-i32::try_from(fraction_bits).expect("a grid width fits i32"))
    }

    /// Returns the smallest multiple of `2^-fraction_bits` that is not below this value.
    ///
    /// # Panics
    ///
    /// Panics under the same unreachable condition as
    /// [`Self::floor_to_binary_grid`].
    #[must_use]
    pub fn ceil_to_binary_grid(&self, fraction_bits: u32) -> Self {
        let scaled = &self.numerator << u64::from(fraction_bits);
        let quotient = scaled.div_ceil(&to_int(&self.denominator));
        Self::normalize(quotient, BigUint::one())
            .scale_by_power_of_two(-i32::try_from(fraction_bits).expect("a grid width fits i32"))
    }

    /// Returns a rigorous enclosure of this value's square root on a binary grid.
    ///
    /// The returned pair `(lower, upper)` satisfies `lower^2 <= self` and
    /// `upper^2 > self` by construction, both endpoints being multiples of
    /// `2^-fraction_bits`. It is an *enclosure*, never an approximation: the
    /// caller may narrow it by raising `fraction_bits` and may never assume it is
    /// tight.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError::NegativeSquareRoot`] for a strictly negative
    /// radicand.
    ///
    /// # Panics
    ///
    /// Panics under the same unreachable condition as
    /// [`Self::floor_to_binary_grid`].
    pub fn sqrt_enclosure(&self, fraction_bits: u32) -> Result<(Self, Self), ExactRationalError> {
        if self.is_negative() {
            return Err(ExactRationalError::NegativeSquareRoot {
                value: self.clone(),
            });
        }
        let shift = u64::from(fraction_bits) * 2;
        let scaled = (self.numerator.magnitude() << shift).div_floor(&self.denominator);
        let root = scaled.sqrt();
        let exponent = -i32::try_from(fraction_bits).expect("a grid width fits i32");
        let lower = Self::normalize(BigInt::from(root.clone()), BigUint::one())
            .scale_by_power_of_two(exponent);
        let upper = Self::normalize(BigInt::from(root + BigUint::one()), BigUint::one())
            .scale_by_power_of_two(exponent);
        Ok((lower, upper))
    }

    /// Returns `k` such that `2^k <= |self| < 2^(k+1)`, or `None` at zero.
    ///
    /// # Panics
    ///
    /// Panics when a magnitude's bit length or the resulting exponent leaves
    /// `i64` or `i32`. Both are unreachable for a value built through this
    /// module's constructors, which bound every decoded magnitude by
    /// [`MAX_EXACT_RATIONAL_MAGNITUDE_BYTES`]; checked rather than cast so an
    /// unbounded value fails here instead of returning a wrapped exponent that
    /// every ULP scale derived from it would then be silently wrong about.
    #[must_use]
    pub fn floor_log2_abs(&self) -> Option<i32> {
        if self.is_zero() {
            return None;
        }
        let numerator_bits = i64::try_from(self.numerator.magnitude().bits())
            .expect("a bounded magnitude's bit length fits i64");
        let denominator_bits = i64::try_from(self.denominator.bits())
            .expect("a bounded magnitude's bit length fits i64");
        // `bits()` returns one more than the floor of the base-two logarithm, so
        // the difference brackets the answer to within one and one comparison
        // decides which side it is on.
        let estimate = numerator_bits - denominator_bits;
        let exponent = if self.cmp_abs_power_of_two(estimate) == Ordering::Less {
            estimate - 1
        } else {
            estimate
        };
        Some(i32::try_from(exponent).expect("a bounded exact rational's exponent fits i32"))
    }

    /// Returns whether this value's magnitude is exactly a power of two.
    #[must_use]
    pub fn is_power_of_two_abs(&self) -> bool {
        let magnitude = self.numerator.magnitude();
        if magnitude.is_zero() {
            return false;
        }
        // In lowest terms at most one side can carry a factor of two, so exactly
        // one of these two shapes is possible for a power of two.
        (power_of_two_exponent(magnitude).is_some() && self.denominator.is_one())
            || (magnitude.is_one() && power_of_two_exponent(&self.denominator).is_some())
    }

    /// Compares `|self|` against `2^exponent`.
    fn cmp_abs_power_of_two(&self, exponent: i64) -> Ordering {
        let magnitude = self.numerator.magnitude();
        if exponent >= 0 {
            let shift = u64::try_from(exponent).expect("a nonnegative exponent fits u64");
            magnitude.cmp(&(&self.denominator << shift))
        } else {
            let shift = exponent.unsigned_abs();
            (magnitude << shift).cmp(&self.denominator)
        }
    }

    fn normalize(numerator: BigInt, denominator: BigUint) -> Self {
        debug_assert!(
            !denominator.is_zero(),
            "normalization requires a nonzero denominator"
        );
        if numerator.is_zero() {
            return Self::zero();
        }
        let divisor = reduction_divisor(numerator.magnitude(), &denominator);
        if let Some(exponent) = power_of_two_exponent(&divisor) {
            // Both components are exact multiples of `2^exponent` — that is what
            // makes it their common divisor — so the shift is the exact quotient
            // on the signed side too, where `>>` would otherwise round toward
            // negative infinity and disagree with the division it replaces.
            return Self {
                numerator: numerator >> exponent,
                denominator: denominator >> exponent,
            };
        }
        Self {
            numerator: numerator / to_int(&divisor),
            denominator: denominator / divisor,
        }
    }

    /// Returns the canonical attribute value carrying this exact number.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::CanonicalBound`] when a magnitude exceeds
    /// a canonical payload bound.
    pub fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let (sign, numerator, denominator) = self.to_sign_magnitude_ratio();
        Ok(CanonicalValue::record([
            CanonicalField::new(
                EXACT_RATIONAL_SIGN,
                CanonicalValue::unsigned_u8(match sign {
                    ExactSign::Negative => 0,
                    ExactSign::Zero => 1,
                    ExactSign::Positive => 2,
                }),
            ),
            CanonicalField::new(
                EXACT_RATIONAL_NUMERATOR,
                CanonicalValue::bytes_owned(numerator)?,
            ),
            CanonicalField::new(
                EXACT_RATIONAL_DENOMINATOR,
                CanonicalValue::bytes_owned(denominator)?,
            ),
        ])?)
    }

    /// Decodes one exact number exactly as an attribute carries it.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for a malformed record, and the exact
    /// canonicality refusals [`Self::from_sign_magnitude_ratio`] reports.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::ExactRational);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let [sign, numerator, denominator] = fields else {
            return Err(subject());
        };
        if sign.id() != EXACT_RATIONAL_SIGN
            || numerator.id() != EXACT_RATIONAL_NUMERATOR
            || denominator.id() != EXACT_RATIONAL_DENOMINATOR
        {
            return Err(subject());
        }
        let (
            CanonicalValueView::Unsigned { bits, .. },
            CanonicalValueView::Bytes(numerator),
            CanonicalValueView::Bytes(denominator),
        ) = (
            sign.value().view(),
            numerator.value().view(),
            denominator.value().view(),
        )
        else {
            return Err(subject());
        };
        let sign = match bits {
            0 => ExactSign::Negative,
            1 => ExactSign::Zero,
            2 => ExactSign::Positive,
            _ => return Err(subject()),
        };
        Ok(Self::from_sign_magnitude_ratio(
            sign,
            numerator,
            denominator,
        )?)
    }

    /// Appends the collision-free canonical encoding of this value.
    pub(crate) fn encode(&self, output: &mut Vec<u8>) {
        let (sign, numerator, denominator) = self.to_sign_magnitude_ratio();
        output.push(match sign {
            ExactSign::Negative => 0,
            ExactSign::Zero => 1,
            ExactSign::Positive => 2,
        });
        push_len(output, numerator.len());
        output.extend_from_slice(&numerator);
        push_len(output, denominator.len());
        output.extend_from_slice(&denominator);
    }
}

fn to_int(value: &BigUint) -> BigInt {
    BigInt::from(value.clone())
}

/// Returns `k` when `value` is exactly `2^k`, and `None` otherwise.
fn power_of_two_exponent(value: &BigUint) -> Option<u64> {
    let bits = value.bits();
    (bits != 0 && value.trailing_zeros() == Some(bits - 1)).then(|| bits - 1)
}

/// Returns `gcd(magnitude, denominator)`, by a shift when the denominator is dyadic.
///
/// Identical to [`Integer::gcd`] at every input, including the zero magnitude,
/// where both answer `denominator`. It exists because the general algorithm's
/// *cost* on a dyadic denominator is out of all proportion to its answer:
/// `num-bigint`'s `BigUint::gcd` is Stein's binary algorithm, which shifts the
/// denominator down to one and then subtracts and shifts the magnitude down to
/// zero, so it runs a loop proportional to the *magnitude's* bit length over
/// operands whose word count is proportional to that length — quadratic work for
/// a result that is `2^min(v2(magnitude), k)` and therefore one trailing-zero
/// count away.
///
/// **Measurement.** Over 401 `certified_exp_f32` arguments spanning `[-40, 40]`
/// at the binary32 corpus precision, 42,220 of 66,050 normalizations (63.9 %)
/// carried a power-of-two denominator, and they accounted for 5,915,630 of
/// 9,410,857 Stein iterations (62.9 %). Over 256 `rsqrt_enclosure` radicands it
/// was every one of 1,280 normalizations and all 133,797 iterations. The reason
/// the share is that high is structural rather than incidental: a certified
/// enclosure rounds every intermediate outward onto a binary grid, so every
/// value it carries past that point has a power-of-two denominator, and so does
/// every product of two of them.
///
/// The population census remains reproducible from `### Reproducing the census`
/// in `tickets/bound-the-exact-rational-gcd-cost-in-certified-enclosures.md`;
/// its temporary harness runs with
/// `cargo nextest run -p tiler-reference --test gcd_census_temp --no-capture`.
///
/// **Regression-guard decision (2026-08-10).** No admissible deterministic
/// in-tree test distinguishes this shift from the value-identical general gcd in
/// release builds. A test-only path observer can be satisfied from a
/// `debug_assert` while release executes gcd; a source-text census pins one Rust
/// spelling rather than the mechanism; timing is host-sensitive; and a permanent
/// dependency or hot-path counter adds the cost this branch exists to avoid.
/// Preserve the explicit branch and rerun the cited operand-population census
/// when changing it rather than treating value tests as performance evidence.
///
/// The symmetric case — a dyadic *magnitude* against an odd denominator — is not
/// taken, because the same census measured it at 480 calls (0.7 %) carrying 6,168
/// iterations (0.066 %). It would be a branch whose cost is paid on every call to
/// remove work that is not there.
fn reduction_divisor(magnitude: &BigUint, denominator: &BigUint) -> BigUint {
    let Some(exponent) = power_of_two_exponent(denominator) else {
        return magnitude.gcd(denominator);
    };
    let Some(twos) = magnitude.trailing_zeros() else {
        // `gcd(0, d)` is `d`, which the shift below cannot express.
        return denominator.clone();
    };
    BigUint::one() << exponent.min(twos)
}

fn validate_magnitude(magnitude: &[u8]) -> Result<(), ExactRationalError> {
    if magnitude.len() > MAX_EXACT_RATIONAL_MAGNITUDE_BYTES {
        return Err(ExactRationalError::MagnitudeTooLong {
            bytes: magnitude.len(),
        });
    }
    if magnitude.first() == Some(&0) {
        return Err(ExactRationalError::NoncanonicalLeadingZero);
    }
    Ok(())
}

impl Ord for ExactRational {
    fn cmp(&self, other: &Self) -> Ordering {
        // Cross-multiplied rather than divided: both denominators are strictly
        // positive, so the comparison is exact and the sign is preserved.
        (&self.numerator * to_int(&other.denominator))
            .cmp(&(&other.numerator * to_int(&self.denominator)))
    }
}

impl PartialOrd for ExactRational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for ExactRational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ExactRational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator.is_one() {
            self.numerator.fmt(formatter)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// An exact nonnegative tolerance.
///
/// The type ADR 0042's "every tolerance is a canonical exact nonnegative number"
/// names. Nonnegativity is enforced at construction rather than documented,
/// because a negative tolerance is not a tight contract — it is an unsatisfiable
/// one, and a contract nothing can conform to is a defect that must be refused
/// where it is written rather than discovered where it is checked.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExactTolerance(ExactRational);

impl ExactTolerance {
    /// Returns the exact zero tolerance.
    #[must_use]
    pub fn zero() -> Self {
        Self(ExactRational::zero())
    }

    /// Creates a tolerance from a nonnegative fixed-width host integer.
    #[must_use]
    pub fn from_integer(value: u64) -> Self {
        Self(ExactRational::from_integer(i128::from(value)))
    }

    /// Creates a tolerance from a nonnegative fixed-width ratio.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError`] for a zero denominator or a negative value.
    pub fn from_ratio(numerator: i128, denominator: i128) -> Result<Self, ExactRationalError> {
        Self::try_from_rational(ExactRational::from_ratio(numerator, denominator)?)
    }

    /// Admits one exact rational as a tolerance.
    ///
    /// # Errors
    ///
    /// Returns [`ExactRationalError::NegativeTolerance`] for a strictly negative
    /// value.
    pub fn try_from_rational(value: ExactRational) -> Result<Self, ExactRationalError> {
        if value.is_negative() {
            return Err(ExactRationalError::NegativeTolerance { value });
        }
        Ok(Self(value))
    }

    /// Returns the exact value of this tolerance.
    #[must_use]
    pub const fn value(&self) -> &ExactRational {
        &self.0
    }

    /// Returns the canonical attribute value carrying this tolerance.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::CanonicalBound`] when a magnitude exceeds
    /// a canonical payload bound.
    pub fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        self.0.to_canonical_value()
    }

    /// Decodes one tolerance exactly as an attribute carries it.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for a malformed record, a non-canonical
    /// exact number, or a strictly negative value.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        Ok(Self::try_from_rational(
            ExactRational::from_canonical_value(value)?,
        )?)
    }

    pub(crate) fn encode(&self, output: &mut Vec<u8>) {
        self.0.encode(output);
    }
}

impl fmt::Display for ExactTolerance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
