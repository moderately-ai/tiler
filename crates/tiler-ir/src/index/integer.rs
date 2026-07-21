use std::fmt;

use num_bigint::{BigInt, Sign};

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

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        let (sign, magnitude) = self.0.to_bytes_be();
        output.push(match sign {
            Sign::Minus => 0,
            Sign::NoSign => 1,
            Sign::Plus => 2,
        });
        output.extend_from_slice(
            &u64::try_from(magnitude.len())
                .expect("supported integer magnitude fits u64")
                .to_be_bytes(),
        );
        output.extend_from_slice(&magnitude);
    }
}

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
