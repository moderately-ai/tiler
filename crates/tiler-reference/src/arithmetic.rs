//! Bounded exact signed-integer arithmetic for the index-region reference oracle.
//!
//! [`tiler_ir::index::IndexInteger`] deliberately exposes no arithmetic, so the
//! oracle evaluates index expressions over its canonical sign-and-magnitude
//! representation with this checked implementation. Keeping the oracle's
//! arithmetic independent of the arbitrary-precision library used by the
//! structural verifier also prevents one shared defect from making the oracle
//! agree with an incorrect coordinate.
//!
//! Every operation is exact. A result whose canonical magnitude exceeds the
//! caller's governed bound is rejected as [`MagnitudeExceeded`]; nothing here
//! saturates, truncates, or wraps.

use std::cmp::Ordering;

use tiler_ir::index::{IndexInteger, IndexIntegerSign};

/// One evaluated integer exceeded a governed canonical magnitude.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MagnitudeExceeded {
    /// Canonical magnitude bytes the rejected value required.
    pub(crate) required_bytes: usize,
}

/// An exact signed integer held as a sign and little-endian 64-bit limbs.
///
/// Zero has an empty limb sequence and a non-negative sign. Nonzero magnitudes
/// never retain a leading zero limb, so equality is exact and canonical.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ExactInteger {
    negative: bool,
    magnitude: Vec<u64>,
}

impl ExactInteger {
    /// Returns exact zero.
    pub(crate) const fn zero() -> Self {
        Self {
            negative: false,
            magnitude: Vec::new(),
        }
    }

    /// Returns one exact non-negative host integer.
    pub(crate) fn from_u64(value: u64) -> Self {
        Self {
            negative: false,
            magnitude: if value == 0 { Vec::new() } else { vec![value] },
        }
    }

    /// Decodes one canonical sign-and-big-endian-magnitude index integer.
    pub(crate) fn from_index_integer(value: &IndexInteger) -> Self {
        let (sign, magnitude) = value.to_sign_magnitude();
        let mut limbs = Vec::with_capacity(magnitude.len().div_ceil(8));
        let mut remaining = magnitude.as_slice();
        while !remaining.is_empty() {
            let split = remaining.len().saturating_sub(8);
            let (head, tail) = remaining.split_at(split);
            let mut limb = 0_u64;
            for byte in tail {
                limb = (limb << 8) | u64::from(*byte);
            }
            limbs.push(limb);
            remaining = head;
        }
        trim(&mut limbs);
        Self {
            negative: sign == IndexIntegerSign::Negative && !limbs.is_empty(),
            magnitude: limbs,
        }
    }

    /// Returns whether this value is exactly zero.
    pub(crate) fn is_zero(&self) -> bool {
        self.magnitude.is_empty()
    }

    /// Returns the exact canonical magnitude byte count.
    pub(crate) fn magnitude_bytes(&self) -> usize {
        usize::try_from(magnitude_bits(&self.magnitude).div_ceil(8)).unwrap_or(usize::MAX)
    }

    /// Returns this value when it fits an unsigned host integer.
    pub(crate) fn to_u64(&self) -> Option<u64> {
        match self.magnitude.as_slice() {
            _ if self.negative => None,
            [] => Some(0),
            [limb] => Some(*limb),
            _ => None,
        }
    }

    /// Adds two exact integers within a governed magnitude bound.
    pub(crate) fn checked_add(
        &self,
        addend: &Self,
        limit_bytes: usize,
    ) -> Result<Self, MagnitudeExceeded> {
        let sum = if self.negative == addend.negative {
            Self {
                negative: self.negative,
                magnitude: magnitude_add(&self.magnitude, &addend.magnitude),
            }
        } else {
            match magnitude_compare(&self.magnitude, &addend.magnitude) {
                Ordering::Equal => Self::zero(),
                Ordering::Greater => Self {
                    negative: self.negative,
                    magnitude: magnitude_subtract(&self.magnitude, &addend.magnitude),
                },
                Ordering::Less => Self {
                    negative: addend.negative,
                    magnitude: magnitude_subtract(&addend.magnitude, &self.magnitude),
                },
            }
        };
        sum.normalized().admit(limit_bytes)
    }

    /// Multiplies two exact integers within a governed magnitude bound.
    ///
    /// The product size is rejected from operand bit lengths before any
    /// quadratic work, so an over-limit multiplication is never performed.
    pub(crate) fn checked_mul(
        &self,
        factor: &Self,
        limit_bytes: usize,
    ) -> Result<Self, MagnitudeExceeded> {
        if self.is_zero() || factor.is_zero() {
            return Ok(Self::zero());
        }
        let upper_bits =
            magnitude_bits(&self.magnitude).saturating_add(magnitude_bits(&factor.magnitude));
        let limit_bits = u64::try_from(limit_bytes)
            .unwrap_or(u64::MAX)
            .saturating_mul(8);
        if upper_bits > limit_bits.saturating_add(1) {
            return Err(MagnitudeExceeded {
                required_bytes: usize::try_from(upper_bits.div_ceil(8)).unwrap_or(usize::MAX),
            });
        }
        Self {
            negative: self.negative != factor.negative,
            magnitude: magnitude_multiply(&self.magnitude, &factor.magnitude),
        }
        .normalized()
        .admit(limit_bytes)
    }

    /// Returns the Euclidean floor quotient and modulus for a positive divisor.
    ///
    /// The modulus is always in `0..divisor`, matching the IR's floor contract.
    pub(crate) fn div_mod_floor(&self, divisor: u64) -> Option<(Self, Self)> {
        if divisor == 0 {
            return None;
        }
        let (quotient, remainder) = magnitude_div_rem(&self.magnitude, divisor);
        if !self.negative {
            return Some((
                Self {
                    negative: false,
                    magnitude: quotient,
                }
                .normalized(),
                Self::from_u64(remainder),
            ));
        }
        if remainder == 0 {
            return Some((
                Self {
                    negative: true,
                    magnitude: quotient,
                }
                .normalized(),
                Self::zero(),
            ));
        }
        Some((
            Self {
                negative: true,
                magnitude: magnitude_add(&quotient, &[1]),
            }
            .normalized(),
            Self::from_u64(divisor - remainder),
        ))
    }

    fn normalized(mut self) -> Self {
        trim(&mut self.magnitude);
        if self.magnitude.is_empty() {
            self.negative = false;
        }
        self
    }

    fn admit(self, limit_bytes: usize) -> Result<Self, MagnitudeExceeded> {
        let bytes = self.magnitude_bytes();
        if bytes > limit_bytes {
            return Err(MagnitudeExceeded {
                required_bytes: bytes,
            });
        }
        Ok(self)
    }
}

fn trim(magnitude: &mut Vec<u64>) {
    while magnitude.last() == Some(&0) {
        magnitude.pop();
    }
}

fn magnitude_bits(magnitude: &[u64]) -> u64 {
    magnitude.split_last().map_or(0, |(top, rest)| {
        (rest.len() as u64)
            .saturating_mul(64)
            .saturating_add(u64::from(64 - top.leading_zeros()))
    })
}

fn magnitude_compare(left: &[u64], right: &[u64]) -> Ordering {
    left.len()
        .cmp(&right.len())
        .then_with(|| left.iter().rev().cmp(right.iter().rev()))
}

/// Splits one 128-bit intermediate into its low and high 64-bit halves.
fn split(value: u128) -> (u64, u64) {
    let low = u64::try_from(value & u128::from(u64::MAX)).expect("a masked u128 fits u64");
    let high = u64::try_from(value >> 64).expect("a shifted u128 fits u64");
    (low, high)
}

fn magnitude_add(left: &[u64], right: &[u64]) -> Vec<u64> {
    let (longer, shorter) = if left.len() >= right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let mut sum = Vec::with_capacity(longer.len() + 1);
    let mut carry = 0_u64;
    for (index, limb) in longer.iter().enumerate() {
        // At most one of the two additions can carry, because a carrying
        // limb sum leaves a value far below the maximum.
        let (partial, first) = limb.overflowing_add(shorter.get(index).copied().unwrap_or(0));
        let (value, second) = partial.overflowing_add(carry);
        sum.push(value);
        carry = u64::from(first || second);
    }
    if carry != 0 {
        sum.push(carry);
    }
    sum
}

/// Subtracts `right` from `left`, which the caller has proved is not smaller.
fn magnitude_subtract(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut difference = Vec::with_capacity(left.len());
    let mut borrow = 0_u64;
    for (index, limb) in left.iter().enumerate() {
        // At most one of the two subtractions can borrow, because a borrowing
        // limb leaves the maximum value.
        let (partial, first) = limb.overflowing_sub(borrow);
        let (value, second) = partial.overflowing_sub(right.get(index).copied().unwrap_or(0));
        difference.push(value);
        borrow = u64::from(first || second);
    }
    trim(&mut difference);
    difference
}

fn magnitude_multiply(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut product = vec![0_u64; left.len() + right.len()];
    for (offset, factor) in left.iter().enumerate() {
        let mut carry = 0_u64;
        for (index, other) in right.iter().enumerate() {
            let total = u128::from(*factor) * u128::from(*other)
                + u128::from(product[offset + index])
                + u128::from(carry);
            let (low, high) = split(total);
            product[offset + index] = low;
            carry = high;
        }
        product[offset + right.len()] = carry;
    }
    trim(&mut product);
    product
}

fn magnitude_div_rem(magnitude: &[u64], divisor: u64) -> (Vec<u64>, u64) {
    let mut quotient = vec![0_u64; magnitude.len()];
    let mut remainder = 0_u128;
    let divisor = u128::from(divisor);
    for (index, limb) in magnitude.iter().enumerate().rev() {
        // The running remainder is below the divisor, so the quotient limb
        // and the next remainder both fit 64 bits.
        let current = (remainder << 64) | u128::from(*limb);
        quotient[index] = u64::try_from(current / divisor).expect("a bounded quotient fits u64");
        remainder = current % divisor;
    }
    trim(&mut quotient);
    (
        quotient,
        u64::try_from(remainder).expect("a remainder below a u64 divisor fits u64"),
    )
}

#[cfg(test)]
mod tests {
    use super::{ExactInteger, MagnitudeExceeded};
    use tiler_ir::index::{IndexInteger, IndexIntegerSign};

    const LIMIT: usize = 1024;

    fn exact(value: i128) -> ExactInteger {
        ExactInteger::from_index_integer(&IndexInteger::from_i128(value))
    }

    fn decode(sign: IndexIntegerSign, magnitude: &[u8]) -> ExactInteger {
        ExactInteger::from_index_integer(
            &IndexInteger::from_sign_magnitude(sign, magnitude).expect("canonical magnitude"),
        )
    }

    #[test]
    fn round_trips_canonical_sign_and_magnitude_across_limb_boundaries() {
        for bytes in 1..=24_usize {
            let mut magnitude = vec![0xff_u8; bytes];
            magnitude[0] = 0x80;
            let positive = decode(IndexIntegerSign::Positive, &magnitude);
            let negative = decode(IndexIntegerSign::Negative, &magnitude);
            assert_eq!(positive.magnitude_bytes(), bytes);
            assert_eq!(negative.magnitude_bytes(), bytes);
            assert_ne!(positive, negative);
            assert!(
                positive
                    .checked_add(&negative, LIMIT)
                    .expect("in-bounds sum")
                    .is_zero()
            );
            assert_eq!(
                negative
                    .checked_mul(&exact(-1), LIMIT)
                    .expect("in-bounds product"),
                positive
            );
        }
        assert!(exact(0).is_zero());
    }

    #[test]
    fn signed_addition_and_multiplication_stay_exact_through_cancellation() {
        let large = IndexInteger::from_sign_magnitude(IndexIntegerSign::Positive, &[0x01; 40])
            .expect("canonical magnitude");
        let positive = ExactInteger::from_index_integer(&large);
        let negative = positive
            .checked_mul(&exact(-1), LIMIT)
            .expect("in-bounds product");
        assert!(
            positive
                .checked_add(&negative, LIMIT)
                .expect("in-bounds sum")
                .is_zero()
        );
        assert_eq!(
            positive
                .checked_add(&negative, LIMIT)
                .and_then(|zero| zero.checked_add(&exact(7), LIMIT))
                .expect("in-bounds sum")
                .to_u64(),
            Some(7)
        );
        let squared = positive
            .checked_mul(&positive, LIMIT)
            .expect("in-bounds product");
        assert_eq!(squared.magnitude_bytes(), 79);
        assert_eq!(
            negative
                .checked_mul(&negative, LIMIT)
                .expect("in-bounds product"),
            squared
        );
    }

    #[test]
    fn euclidean_division_matches_the_floor_contract_for_negative_values() {
        for (value, divisor, quotient, modulus) in [
            (7_i128, 3_u64, 2_i128, 1_u64),
            (-7, 3, -3, 2),
            (-9, 3, -3, 0),
            (0, 5, 0, 0),
            (-1, 1, -1, 0),
        ] {
            let (actual_quotient, actual_modulus) = exact(value)
                .div_mod_floor(divisor)
                .expect("positive divisor");
            assert_eq!(actual_quotient, exact(quotient), "{value} / {divisor}");
            assert_eq!(
                actual_modulus.to_u64(),
                Some(modulus),
                "{value} % {divisor}"
            );
        }
        assert!(exact(1).div_mod_floor(0).is_none());
    }

    #[test]
    fn oversized_intermediates_reject_before_quadratic_work() {
        let large = IndexInteger::from_sign_magnitude(IndexIntegerSign::Positive, &[0xff; 64])
            .expect("canonical magnitude");
        let value = ExactInteger::from_index_integer(&large);
        assert_eq!(
            value.checked_mul(&value, 64),
            Err(MagnitudeExceeded {
                required_bytes: 128
            })
        );
        assert_eq!(
            value.checked_add(&value, 64),
            Err(MagnitudeExceeded { required_bytes: 65 })
        );
        assert!(value.checked_mul(&value, 128).is_ok());
    }
}
