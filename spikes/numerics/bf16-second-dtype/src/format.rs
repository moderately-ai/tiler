//! A parameterized binary interchange format, and the rounding it fixes.
//!
//! # Why this exists beside the BF16 oracle rather than replacing it
//!
//! [`crate::bf16`] states BF16's value set and its single rounding directly, and
//! it is the module the retained corpus is evidence about. The computation and
//! accumulator question needs a *second* format in the same run — a route that
//! widens a BF16 operand, evaluates at binary32, and rounds back — so the
//! rounding rule has to be expressible at two precisions at once.
//!
//! This module is therefore a generalization *checked against* the specific one
//! rather than a substitute for it: [`crate::promotion`]'s first stage requires
//! `BinaryFormat::BF16` to agree with `bf16::round_to_nearest_even` over a named
//! population before any later stage's answer is read. A generic rounder that
//! quietly disagreed with the trusted one would otherwise make every promoted
//! route a statement about this file instead of about BF16.
//!
//! # What a `BinaryFormat` is and is not
//!
//! It is exactly the four parameters an IEEE 754 binary interchange format fixes
//! that this spike's arithmetic reads — sign presence, exponent width, trailing
//! significand width, and the bias derived from the exponent width. It is **not**
//! a dtype identity: `tiler::bf16@1`'s identity is the governed catalog row, and
//! `BinaryFormat::BF16`'s parameters are checked against that row by
//! [`crate::seams::descriptor_agreement_seam`]. Two formats with equal
//! parameters are not thereby the same dtype, which is exactly what
//! `docs/numerical-semantics.md` says about structural descriptions and
//! identity.

use std::cmp::Ordering;

use num_bigint::BigInt;
use num_traits::Zero;

use crate::bf16::{ExactValue, Rational, TieRule, exceeds};

/// One binary interchange format's parameters.
///
/// Every field is a positive claim about a format family. The `name` is for
/// diagnostics only and participates in no comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BinaryFormat {
    /// Diagnostic name, never an identity.
    pub name: &'static str,
    /// Exponent field width in bits.
    pub exponent_bits: u32,
    /// Trailing significand field width in bits.
    pub trailing_significand_bits: u32,
}

impl BinaryFormat {
    /// `tiler::bf16@1`'s parameters: 8 exponent bits, 7 trailing significand bits.
    pub const BF16: Self = Self {
        name: "bf16",
        exponent_bits: 8,
        trailing_significand_bits: 7,
    };

    /// IEEE 754 binary32: 8 exponent bits, 23 trailing significand bits.
    ///
    /// The exponent width is the **same** as BF16's, which is the whole reason
    /// the widening below is exact and total and the reason a widened BF16
    /// subnormal is still a subnormal. Binary16's is not, and
    /// [`Self::BINARY16`] is carried so that contrast is checkable rather than
    /// asserted.
    pub const BINARY32: Self = Self {
        name: "binary32",
        exponent_bits: 8,
        trailing_significand_bits: 23,
    };

    /// IEEE 754 binary16: 5 exponent bits, 10 trailing significand bits.
    ///
    /// Present only as the contrast case. No BF16 route in this spike uses it as
    /// an intermediate; [`crate::promotion`] uses it as a perturbation, where its
    /// narrower exponent range must break a route that binary32 carries.
    pub const BINARY16: Self = Self {
        name: "binary16",
        exponent_bits: 5,
        trailing_significand_bits: 10,
    };

    /// Precision in bits: the trailing significand plus the implicit leading bit.
    #[must_use]
    pub const fn precision(&self) -> u32 {
        self.trailing_significand_bits + 1
    }

    /// Total encoded width: one sign bit, the exponent field, the trailing field.
    #[must_use]
    pub const fn width_bits(&self) -> u32 {
        1 + self.exponent_bits + self.trailing_significand_bits
    }

    /// Largest exponent of a finite value, which is also the encoding's bias.
    #[must_use]
    pub const fn max_exponent(&self) -> i32 {
        (1 << (self.exponent_bits - 1)) - 1
    }

    /// Smallest exponent of a normal value.
    #[must_use]
    pub const fn min_exponent(&self) -> i32 {
        1 - self.max_exponent()
    }

    /// Exponent of the least significant bit of the smallest subnormal.
    #[must_use]
    pub const fn min_subnormal_exponent(&self) -> i32 {
        self.min_exponent() - self.significand_offset()
    }

    const fn significand_offset(&self) -> i32 {
        self.trailing_significand_bits.cast_signed()
    }

    /// The largest finite magnitude this format represents, exactly.
    #[must_use]
    pub fn largest_finite(&self) -> Rational {
        let significand = (1_i64 << self.precision()) - 1;
        Rational::from_integer(significand)
            .scale_by_power_of_two(self.max_exponent() - self.significand_offset())
    }

    /// The magnitude at or above which round-to-nearest produces an infinity.
    ///
    /// The midpoint between the largest finite value and the first absent power
    /// of two, exactly as [`crate::bf16::overflow_threshold`] states it for BF16.
    /// The boundary is inclusive: a magnitude *equal* to this value overflows.
    #[must_use]
    pub fn overflow_threshold(&self) -> Rational {
        let significand = (1_i64 << self.precision()) * 2 - 1;
        Rational::from_integer(significand)
            .scale_by_power_of_two(self.max_exponent() - self.significand_offset() - 1)
    }

    /// Decodes one bit pattern of this format into its exact mathematical value.
    ///
    /// The bit pattern occupies the low [`Self::width_bits`] bits; anything above
    /// them is ignored rather than rejected, because every caller here builds the
    /// pattern from a literal of the exact width.
    ///
    /// # Panics
    ///
    /// Panics when the format's significand does not fit `i64`, which no format
    /// this spike constructs approaches.
    #[must_use]
    pub fn decode(&self, bits: u64) -> ExactValue {
        let sign_negative = (bits >> (self.width_bits() - 1)) & 1 == 1;
        let biased = (bits >> self.trailing_significand_bits) & ((1 << self.exponent_bits) - 1);
        let trailing = bits & ((1 << self.trailing_significand_bits) - 1);
        let max_biased = (1 << self.exponent_bits) - 1;
        if biased == max_biased {
            return if trailing == 0 {
                ExactValue::Infinite {
                    negative: sign_negative,
                }
            } else {
                ExactValue::Nan
            };
        }
        if biased == 0 && trailing == 0 {
            return ExactValue::Finite {
                value: Rational::zero(),
                zero_is_negative: sign_negative,
            };
        }
        let (significand, exponent) = if biased == 0 {
            (trailing, self.min_subnormal_exponent())
        } else {
            (
                trailing | (1 << self.trailing_significand_bits),
                i32::try_from(biased).expect("a biased exponent fits i32")
                    - self.max_exponent()
                    - self.significand_offset(),
            )
        };
        let significand = i64::try_from(significand).expect("a significand fits i64");
        let magnitude = Rational::from_integer(significand).scale_by_power_of_two(exponent);
        ExactValue::Finite {
            value: if sign_negative {
                magnitude.negate()
            } else {
                magnitude
            },
            zero_is_negative: false,
        }
    }

    /// Returns whether a decoded pattern of this format is a subnormal.
    #[must_use]
    pub fn is_subnormal_encoding(&self, bits: u64) -> bool {
        let biased = (bits >> self.trailing_significand_bits) & ((1 << self.exponent_bits) - 1);
        let trailing = bits & ((1 << self.trailing_significand_bits) - 1);
        biased == 0 && trailing != 0
    }

    /// Rounds an exact value into this format's value set, returning the
    /// rounded value — still exact, so a second rounding can consume it.
    ///
    /// This deliberately returns an [`ExactValue`] rather than an encoding. A
    /// promoted route is *two* roundings applied to one exact value, and
    /// re-encoding between them would add a third boundary the contract under
    /// test does not have.
    #[must_use]
    pub fn round(&self, value: &ExactValue, tie: TieRule) -> ExactValue {
        match value {
            ExactValue::Nan => ExactValue::Nan,
            ExactValue::Infinite { negative } => ExactValue::Infinite {
                negative: *negative,
            },
            ExactValue::Finite {
                value,
                zero_is_negative,
            } => {
                if value.is_zero() {
                    return ExactValue::Finite {
                        value: Rational::zero(),
                        zero_is_negative: *zero_is_negative,
                    };
                }
                let negative = value.is_negative();
                let magnitude = value.abs();
                // `exceeds(threshold, magnitude)` is `magnitude < threshold`; the
                // negation makes the boundary inclusive, which is the decided
                // rule rather than an accident of the comparison direction.
                if !exceeds(&self.overflow_threshold(), &magnitude) {
                    return ExactValue::Infinite { negative };
                }
                let rounded = self.round_magnitude(&magnitude, tie);
                ExactValue::Finite {
                    value: if negative {
                        rounded.negate()
                    } else {
                        rounded.clone()
                    },
                    // Only meaningful when the rounding underflowed to zero; a
                    // nonzero result must report `false` so this agrees with a
                    // decoded encoding's own representation of the same value.
                    zero_is_negative: negative && rounded.is_zero(),
                }
            }
        }
    }

    fn round_magnitude(&self, magnitude: &Rational, tie: TieRule) -> Rational {
        let mut exponent = self.binade_exponent(magnitude).max(self.min_exponent());
        let mut quanta = self.round_quanta(magnitude, exponent, tie);
        // Rounding up can carry into the next binade, which is a legal single
        // step and needs the exponent incremented once.
        if quanta >= (1_i64 << self.precision()) {
            exponent += 1;
            quanta = self.round_quanta(magnitude, exponent, tie);
        }
        Rational::from_integer(quanta).scale_by_power_of_two(exponent - self.significand_offset())
    }

    fn binade_exponent(&self, magnitude: &Rational) -> i32 {
        let mut low = self.min_subnormal_exponent() - 1;
        let mut high = self.max_exponent() + 2;
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            if exceeds(
                &Rational::from_integer(1).scale_by_power_of_two(middle),
                magnitude,
            ) {
                high = middle;
            } else {
                low = middle;
            }
        }
        low
    }

    fn round_quanta(&self, magnitude: &Rational, exponent: i32, tie: TieRule) -> i64 {
        let quantum_exponent = exponent - self.significand_offset();
        let scaled = magnitude.scale_by_power_of_two(-quantum_exponent);
        let numerator = scaled.numerator();
        let denominator = scaled.denominator();
        let floor = numerator / denominator;
        let remainder = numerator - &floor * denominator;
        let twice_remainder: BigInt = &remainder * 2;
        let rounded = match twice_remainder.cmp(denominator) {
            Ordering::Greater => &floor + 1,
            Ordering::Less => floor.clone(),
            Ordering::Equal => match tie {
                TieRule::ToEven if (&floor % 2_i32).is_zero() => floor.clone(),
                TieRule::ToEven | TieRule::AwayFromZero => &floor + 1,
            },
        };
        i64::try_from(rounded).expect("a quantum count of a format this spike uses fits i64")
    }
}
