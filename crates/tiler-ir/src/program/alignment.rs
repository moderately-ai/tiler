//! Checked byte-alignment vocabulary for physical program values and allocations.
//!
//! **Accepted public surface.** Tom accepted this exact spelling on 2026-08-13
//! under [`accept-the-typed-byte-alignment-surface`]. The 2026-08-12 model
//! acceptance is the earlier packet; this label is the included/excluded Rust
//! surface.
//!
//! [`accept-the-typed-byte-alignment-surface`]: ../../../../../tickets/accept-the-typed-byte-alignment-surface.md
//!
//! Construction is fallible. Zero and non-powers of two are typed errors.
//! There is no `Default`, no unchecked public constructor, and no rounding,
//! clamp, or integer sentinel. Subsumption lives only on
//! [`AlignmentGuarantee::satisfies`]. Natural requirements come from
//! [`StorageScalar::byte_width`]; this module does not keep a second carrier
//! width table.

use std::fmt;

use super::model::StorageScalar;

/// Why a byte count cannot be a byte alignment.
///
/// **Accepted public surface.** The two cases are distinct because they
/// fail for different reasons: zero is not a divisor of anything, and a
/// non-power of two would make divisibility stop being the alignment relation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ByteAlignmentError {
    /// The count was zero.
    Zero,
    /// The count was positive but not a power of two.
    NotPowerOfTwo {
        /// Rejected byte count.
        bytes: u32,
    },
}

impl fmt::Display for ByteAlignmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Zero => formatter.write_str("byte alignment cannot be zero"),
            Self::NotPowerOfTwo { bytes } => {
                write!(
                    formatter,
                    "byte alignment {bytes} is not a positive power of two"
                )
            }
        }
    }
}

impl std::error::Error for ByteAlignmentError {}

/// A checked positive power-of-two byte alignment.
///
/// **Accepted public surface.** This is the single authority for the
/// quantity. APIs that state a *direction* wrap it in
/// [`AlignmentRequirement`] or [`AlignmentGuarantee`] so a caller cannot
/// reverse the comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteAlignment(u32);

impl ByteAlignment {
    /// Builds an alignment, rejecting anything that is not a positive power of two.
    ///
    /// Alignment subsumption is divisibility, and divisibility is a partial
    /// order over powers of two but not over arbitrary integers: with a
    /// guarantee of 12 and a requirement of 8, neither divides the other and a
    /// value 12-byte aligned is not 8-byte aligned, so admitting non-powers of
    /// two would make the relation quietly wrong rather than merely unusual.
    ///
    /// # Errors
    ///
    /// Returns [`ByteAlignmentError::Zero`] for zero and
    /// [`ByteAlignmentError::NotPowerOfTwo`] for any other non-power of two.
    pub const fn new(bytes: u32) -> Result<Self, ByteAlignmentError> {
        if bytes == 0 {
            Err(ByteAlignmentError::Zero)
        } else if !bytes.is_power_of_two() {
            Err(ByteAlignmentError::NotPowerOfTwo { bytes })
        } else {
            Ok(Self(bytes))
        }
    }

    /// The natural alignment of one unpacked element of `scalar`.
    ///
    /// [`StorageScalar::byte_width`] is the derivation, and deliberately the only
    /// one. A width table here would be a second place for a carrier's width to
    /// be stated, and the two would agree exactly until the day a carrier was
    /// added to one of them.
    ///
    /// The argument is the *carrier*: a sub-byte logical element reaches memory
    /// inside a carrier under a bit-packed encoding, and what the first element
    /// requires is that carrier's alignment. Deriving an alignment from the
    /// logical width instead would ask a four-bit element for a whole-byte
    /// power of two and get zero, which [`Self::new`] refuses.
    ///
    /// # Panics
    ///
    /// Panics if a storage carrier's byte width is not a positive power of two
    /// within `u32`, which would make it unrepresentable as an alignment at all.
    /// No carrier in the vocabulary is such a width, and
    /// `every_storage_carrier_has_a_representable_alignment` checks the whole
    /// vocabulary rather than leaving that a claim about the carriers that
    /// exist now.
    #[must_use]
    pub fn natural_for(scalar: StorageScalar) -> Self {
        u32::try_from(scalar.byte_width())
            .ok()
            .and_then(|bytes| Self::new(bytes).ok())
            .expect("a storage carrier's byte width is a positive power of two within `u32`")
    }

    /// The alignment in bytes.
    #[must_use]
    pub const fn bytes(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ByteAlignment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-byte", self.0)
    }
}

/// The minimum byte alignment an access or value requires.
///
/// **Accepted public surface.** Opaque so it cannot be compared as if it
/// were a guarantee. The only satisfaction check is
/// [`AlignmentGuarantee::satisfies`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AlignmentRequirement(ByteAlignment);

impl AlignmentRequirement {
    /// Builds a requirement from a checked alignment quantity.
    #[must_use]
    pub const fn from_alignment(alignment: ByteAlignment) -> Self {
        Self(alignment)
    }

    /// Builds a requirement, rejecting anything that is not a positive power of two.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`ByteAlignment::new`].
    pub const fn new(bytes: u32) -> Result<Self, ByteAlignmentError> {
        match ByteAlignment::new(bytes) {
            Ok(alignment) => Ok(Self(alignment)),
            Err(error) => Err(error),
        }
    }

    /// The natural requirement of one unpacked element of `scalar`.
    #[must_use]
    pub fn natural_for(scalar: StorageScalar) -> Self {
        Self(ByteAlignment::natural_for(scalar))
    }

    /// The checked quantity this requirement names.
    #[must_use]
    pub const fn alignment(self) -> ByteAlignment {
        self.0
    }

    /// The requirement in bytes.
    #[must_use]
    pub const fn bytes(self) -> u32 {
        self.0.bytes()
    }
}

impl fmt::Display for AlignmentRequirement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// The byte alignment an allocation or view is statically guaranteed to provide.
///
/// **Accepted public surface.** Opaque so it cannot be compared as if it
/// were a requirement. [`Self::satisfies`] is the only comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AlignmentGuarantee(ByteAlignment);

impl AlignmentGuarantee {
    /// Builds a guarantee from a checked alignment quantity.
    #[must_use]
    pub const fn from_alignment(alignment: ByteAlignment) -> Self {
        Self(alignment)
    }

    /// Builds a guarantee, rejecting anything that is not a positive power of two.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`ByteAlignment::new`].
    pub const fn new(bytes: u32) -> Result<Self, ByteAlignmentError> {
        match ByteAlignment::new(bytes) {
            Ok(alignment) => Ok(Self(alignment)),
            Err(error) => Err(error),
        }
    }

    /// The natural guarantee of one unpacked element of `scalar`.
    #[must_use]
    pub fn natural_for(scalar: StorageScalar) -> Self {
        Self(ByteAlignment::natural_for(scalar))
    }

    /// The checked quantity this guarantee names.
    #[must_use]
    pub const fn alignment(self) -> ByteAlignment {
        self.0
    }

    /// The guarantee in bytes.
    #[must_use]
    pub const fn bytes(self) -> u32 {
        self.0.bytes()
    }

    /// Whether this guaranteed alignment discharges `required`.
    ///
    /// Subsumption is divisibility: a 16-byte guarantee satisfies a 4-byte
    /// requirement, and the converse does not hold.
    #[must_use]
    pub const fn satisfies(self, required: AlignmentRequirement) -> bool {
        self.bytes().is_multiple_of(required.bytes())
    }

    /// The alignment still guaranteed after advancing `offset` bytes.
    ///
    /// Offset zero preserves the base guarantee. A nonzero offset returns the
    /// greatest power-of-two guarantee common to the base and the offset: the
    /// largest power of two that divides both. The arithmetic is the minimum of
    /// the two trailing-zero counts, which stays inside `u32` because the base
    /// is already a positive power of two no larger than `1 << 31`.
    #[must_use]
    pub const fn after_offset(self, offset: u64) -> Self {
        if offset == 0 {
            return self;
        }
        let base_zeros = self.bytes().trailing_zeros();
        let offset_zeros = offset.trailing_zeros();
        let common_zeros = if offset_zeros < base_zeros {
            offset_zeros
        } else {
            base_zeros
        };
        Self(ByteAlignment(1 << common_zeros))
    }
}

impl fmt::Display for AlignmentGuarantee {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AlignmentGuarantee, AlignmentRequirement, ByteAlignment, ByteAlignmentError, StorageScalar,
    };

    const CARRIERS: [StorageScalar; core::mem::variant_count::<StorageScalar>()] =
        [StorageScalar::U8, StorageScalar::F32, StorageScalar::Bf16];

    #[test]
    fn every_storage_carrier_has_a_representable_alignment() {
        for scalar in CARRIERS {
            match scalar {
                StorageScalar::U8 | StorageScalar::F32 | StorageScalar::Bf16 => {}
            }
            let width = scalar.byte_width();
            assert!(
                width != 0 && width.is_power_of_two() && u32::try_from(width).is_ok(),
                "{scalar:?} has byte width {width}, which cannot be a byte alignment"
            );
            let required = AlignmentRequirement::natural_for(scalar);
            assert_eq!(
                u64::from(required.bytes()),
                width,
                "{scalar:?} derived a requirement that is not its own byte width"
            );
            assert_eq!(
                AlignmentGuarantee::natural_for(scalar).bytes(),
                required.bytes(),
                "{scalar:?} derived a guarantee that is not its own byte width"
            );
        }
    }

    #[test]
    fn construction_refuses_zero_and_non_powers_of_two_and_admits_powers_of_two() {
        assert_eq!(ByteAlignment::new(0), Err(ByteAlignmentError::Zero));
        assert_eq!(AlignmentRequirement::new(0), Err(ByteAlignmentError::Zero));
        assert_eq!(AlignmentGuarantee::new(0), Err(ByteAlignmentError::Zero));

        assert_eq!(
            ByteAlignment::new(3),
            Err(ByteAlignmentError::NotPowerOfTwo { bytes: 3 })
        );
        assert_eq!(
            AlignmentRequirement::new(3),
            Err(ByteAlignmentError::NotPowerOfTwo { bytes: 3 })
        );
        assert_eq!(
            AlignmentGuarantee::new(3),
            Err(ByteAlignmentError::NotPowerOfTwo { bytes: 3 })
        );

        let four = ByteAlignment::new(4).expect("4 is a power of two");
        assert_eq!(four.bytes(), 4);
        assert_eq!(
            AlignmentRequirement::new(4).map(AlignmentRequirement::bytes),
            Ok(4)
        );
        assert_eq!(
            AlignmentGuarantee::new(4).map(AlignmentGuarantee::bytes),
            Ok(4)
        );

        let eight = ByteAlignment::new(8).expect("8 is a power of two");
        assert_eq!(eight.bytes(), 8);

        let largest = 1_u32 << 31;
        let max = ByteAlignment::new(largest).expect("2^31 is a power of two");
        assert_eq!(max.bytes(), largest);
        assert_eq!(
            AlignmentRequirement::new(largest).map(AlignmentRequirement::bytes),
            Ok(largest)
        );
        assert_eq!(
            AlignmentGuarantee::new(largest).map(AlignmentGuarantee::bytes),
            Ok(largest)
        );
    }

    #[test]
    fn satisfaction_is_one_directional_divisibility() {
        let sixteen = AlignmentGuarantee::new(16).expect("16 is a power of two");
        let four = AlignmentRequirement::natural_for(StorageScalar::F32);
        let sixteen_required = AlignmentRequirement::new(16).expect("16 is a power of two");
        let four_guaranteed = AlignmentGuarantee::natural_for(StorageScalar::F32);

        assert!(sixteen.satisfies(four));
        assert!(!four_guaranteed.satisfies(sixteen_required));
        assert!(four_guaranteed.satisfies(four));
        assert!(sixteen.satisfies(sixteen_required));
    }

    #[test]
    fn after_offset_of_a_sixteen_byte_base_follows_the_accepted_derivation() {
        let base = AlignmentGuarantee::new(16).expect("16 is a power of two");
        assert_eq!(base.after_offset(0).bytes(), 16);
        assert_eq!(base.after_offset(4).bytes(), 4);
        assert_eq!(base.after_offset(8).bytes(), 8);
        assert_eq!(base.after_offset(16).bytes(), 16);
        assert_eq!(base.after_offset(20).bytes(), 4);
    }
}
