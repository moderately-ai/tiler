use std::fmt;

use num_bigint::{BigInt, Sign};

/// Canonical sign of an exact index integer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexIntegerSign {
    /// A strictly negative integer.
    Negative,
    /// Zero, whose canonical magnitude is empty.
    Zero,
    /// A strictly positive integer.
    Positive,
}

/// Failure to decode a canonical sign-and-magnitude index integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexIntegerDecodeError {
    /// Zero was paired with a nonempty magnitude.
    NonemptyZeroMagnitude,
    /// A nonzero sign was paired with an empty magnitude.
    EmptyNonzeroMagnitude,
    /// A magnitude contained a redundant leading zero byte.
    NoncanonicalLeadingZero,
}

/// An exact signed mathematical integer used by canonical index arithmetic.
///
/// This type deliberately exposes no machine-width or wrapping operations.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexInteger(pub(super) BigInt);

impl IndexInteger {
    /// Creates an exact integer from a signed fixed-width host value.
    #[must_use]
    pub fn from_i128(value: i128) -> Self {
        Self(BigInt::from(value))
    }

    /// Creates an exact integer from an unsigned fixed-width host value.
    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        Self(BigInt::from(value))
    }

    /// Decodes one lossless canonical sign-and-big-endian-magnitude value.
    ///
    /// Zero uses an empty magnitude. Nonzero magnitudes are nonempty and have
    /// no leading zero byte.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the supplied representation is not canonical.
    pub fn from_sign_magnitude(
        sign: IndexIntegerSign,
        magnitude: &[u8],
    ) -> Result<Self, IndexIntegerDecodeError> {
        match sign {
            IndexIntegerSign::Zero if !magnitude.is_empty() => {
                return Err(IndexIntegerDecodeError::NonemptyZeroMagnitude);
            }
            IndexIntegerSign::Negative | IndexIntegerSign::Positive if magnitude.is_empty() => {
                return Err(IndexIntegerDecodeError::EmptyNonzeroMagnitude);
            }
            _ => {}
        }
        if magnitude.first() == Some(&0) {
            return Err(IndexIntegerDecodeError::NoncanonicalLeadingZero);
        }
        let sign = match sign {
            IndexIntegerSign::Negative => Sign::Minus,
            IndexIntegerSign::Zero => Sign::NoSign,
            IndexIntegerSign::Positive => Sign::Plus,
        };
        Ok(Self(BigInt::from_bytes_be(sign, magnitude)))
    }

    /// Returns the lossless canonical sign and big-endian magnitude.
    #[must_use]
    pub fn to_sign_magnitude(&self) -> (IndexIntegerSign, Vec<u8>) {
        let (sign, mut magnitude) = self.0.to_bytes_be();
        let sign = match sign {
            Sign::Minus => IndexIntegerSign::Negative,
            Sign::NoSign => {
                magnitude.clear();
                IndexIntegerSign::Zero
            }
            Sign::Plus => IndexIntegerSign::Positive,
        };
        (sign, magnitude)
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        let (sign, magnitude) = self.to_sign_magnitude();
        output.push(match sign {
            IndexIntegerSign::Negative => 0,
            IndexIntegerSign::Zero => 1,
            IndexIntegerSign::Positive => 2,
        });
        output.extend_from_slice(
            &u64::try_from(magnitude.len())
                .expect("supported integer magnitude fits u64")
                .to_be_bytes(),
        );
        output.extend_from_slice(&magnitude);
    }
}

impl From<i128> for IndexInteger {
    fn from(value: i128) -> Self {
        Self::from_i128(value)
    }
}

impl From<u64> for IndexInteger {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl fmt::Display for IndexIntegerDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NonemptyZeroMagnitude => "zero must use an empty canonical magnitude",
            Self::EmptyNonzeroMagnitude => "a nonzero integer requires a canonical magnitude",
            Self::NoncanonicalLeadingZero => {
                "an index-integer magnitude cannot contain a leading zero"
            }
        })
    }
}

impl std::error::Error for IndexIntegerDecodeError {}

impl fmt::Debug for IndexInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for IndexInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
