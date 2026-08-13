//! The atomic subgroup realization a schedule can require of a target.
//!
//! **Labelled draft** under ADR 0075. Tom accepted the whole-subject *shape*
//! on 2026-08-11 — one checked subject over a literal width, an exact
//! arithmetic type, and an operation-specific transfer, matched only by
//! equality — and has not accepted this crate's exact type, constructor, or
//! error spelling.
//!
//! [`SubgroupRealizationSubject`] groups the three dimensions a target must
//! realize *together*. Each of them is separately true of some machine — a
//! 32-lane simdgroup, an `f32` register file, an in-range XOR shuffle — so a
//! target that declared them independently would let their conjunction be
//! inferred from facts none of which is about it. The subject is therefore
//! matched as one equality, and a feasibility authority never reads a field
//! of it in isolation.
//!
//! Combine order, result lane, coordinate mapping, contributor coverage,
//! activity, and padding identity remain schedule/intrinsic obligations and
//! are not duplicated here.

use super::numerics::ArithmeticType;

/// A literal subgroup width.
///
/// Zero is not a width: a combine tree over no lanes has no steps and no
/// lane identity to prove. Construction rejects it so a later encoder cannot
/// write a vacuous subject.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubgroupWidth(u32);

impl SubgroupWidth {
    /// Constructs a nonzero literal width.
    ///
    /// # Errors
    ///
    /// Returns [`SubgroupRealizationError::ZeroWidth`] when `lanes` is zero.
    pub const fn new(lanes: u32) -> Result<Self, SubgroupRealizationError> {
        if lanes == 0 {
            return Err(SubgroupRealizationError::ZeroWidth);
        }
        Ok(Self(lanes))
    }

    /// The width in lanes.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The register-transfer operation one subgroup realization performs.
///
/// Deliberately not `#[non_exhaustive]`: the identity encoder and the
/// subject's constructor map this totally, so a widened vocabulary is a
/// build error at each rather than a silently admitted transfer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubgroupTransfer {
    /// An in-range XOR shuffle: `source(lane) = lane xor mask` for every
    /// power-of-two mask in `1..width`.
    ///
    /// Every `lane xor mask` stays inside `0..width` exactly when `width` is
    /// a power of two at least 2, which is why
    /// [`SubgroupRealizationSubject::new`] refuses every other width for this
    /// transfer.
    InRangeXorShuffle,
}

impl SubgroupTransfer {
    /// Returns the canonical tag naming this transfer in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::InRangeXorShuffle => 0x01,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized transfer.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::InRangeXorShuffle),
            _ => None,
        }
    }

    /// Returns the stable identifier naming this transfer in an explanation.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::InRangeXorShuffle => "in-range-xor-shuffle",
        }
    }
}

/// Why one checked subgroup subject could not be formed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubgroupRealizationError {
    /// A width of zero names no lane set.
    ZeroWidth,
    /// The width is not a form this transfer can define.
    ///
    /// For [`SubgroupTransfer::InRangeXorShuffle`] that is any width that is
    /// not a power of two at least 2: width 1 has no mask, and a non-power of
    /// two leaves `lane xor mask` out of range.
    UnsupportedWidth,
    /// The transfer tag is not one this build can name.
    UndefinedTransfer,
}

impl SubgroupRealizationError {
    /// Returns the stable identifier naming this construction failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ZeroWidth => "subgroup-width-zero",
            Self::UnsupportedWidth => "subgroup-width-unsupported",
            Self::UndefinedTransfer => "subgroup-transfer-undefined",
        }
    }
}

impl std::fmt::Display for SubgroupRealizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rule())
    }
}

impl std::error::Error for SubgroupRealizationError {}

/// The complete realization one subgroup combine requires of a target.
///
/// **This is the atomic unit a target fact ranges over.** Its three dimensions
/// are matched as one value and never composed: a machine that executes 32
/// lanes, a machine that shuffles `f32`, and a machine that implements XOR
/// transfer are three true statements whose conjunction is not a statement
/// about any of them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubgroupRealizationSubject {
    width: SubgroupWidth,
    arithmetic: ArithmeticType,
    transfer: SubgroupTransfer,
}

impl SubgroupRealizationSubject {
    /// Constructs one checked subject.
    ///
    /// # Errors
    ///
    /// Returns [`SubgroupRealizationError::UnsupportedWidth`] when `transfer`
    /// cannot define a realization at `width`, and
    /// [`SubgroupRealizationError::UndefinedTransfer`] is reserved for a tag
    /// this build does not name (the public constructor only accepts a typed
    /// transfer).
    pub const fn new(
        width: SubgroupWidth,
        arithmetic: ArithmeticType,
        transfer: SubgroupTransfer,
    ) -> Result<Self, SubgroupRealizationError> {
        if !transfer_defines_width(transfer, width) {
            return Err(SubgroupRealizationError::UnsupportedWidth);
        }
        Ok(Self {
            width,
            arithmetic,
            transfer,
        })
    }

    /// The literal width this realization executes.
    #[must_use]
    pub const fn width(self) -> SubgroupWidth {
        self.width
    }

    /// The exact arithmetic type carried through the transfer.
    #[must_use]
    pub const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
    }

    /// The register-transfer operation this realization performs.
    #[must_use]
    pub const fn transfer(self) -> SubgroupTransfer {
        self.transfer
    }

    /// Appends the subject's canonical identity bytes.
    pub fn encode(self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.width.get().to_be_bytes());
        bytes.push(self.arithmetic.tag());
        bytes.push(self.transfer.tag());
    }
}

const fn transfer_defines_width(transfer: SubgroupTransfer, width: SubgroupWidth) -> bool {
    match transfer {
        SubgroupTransfer::InRangeXorShuffle => {
            let lanes = width.get();
            lanes >= 2 && lanes.is_power_of_two()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn width(lanes: u32) -> SubgroupWidth {
        SubgroupWidth::new(lanes).expect("nonzero width")
    }

    #[test]
    fn zero_width_is_rejected() {
        assert_eq!(
            SubgroupWidth::new(0),
            Err(SubgroupRealizationError::ZeroWidth)
        );
        assert_eq!(
            SubgroupRealizationError::ZeroWidth.rule(),
            "subgroup-width-zero"
        );
    }

    #[test]
    fn xor_shuffle_rejects_width_one() {
        assert_eq!(
            SubgroupRealizationSubject::new(
                width(1),
                ArithmeticType::F32,
                SubgroupTransfer::InRangeXorShuffle
            ),
            Err(SubgroupRealizationError::UnsupportedWidth)
        );
        assert_eq!(
            SubgroupRealizationError::UnsupportedWidth.rule(),
            "subgroup-width-unsupported"
        );
    }

    #[test]
    fn xor_shuffle_rejects_non_power_of_two_width() {
        assert_eq!(
            SubgroupRealizationSubject::new(
                width(24),
                ArithmeticType::F32,
                SubgroupTransfer::InRangeXorShuffle
            ),
            Err(SubgroupRealizationError::UnsupportedWidth)
        );
    }

    #[test]
    fn xor_shuffle_accepts_power_of_two_widths_at_least_two() {
        for lanes in [2, 4, 8, 16, 32, 64, 128] {
            let subject = SubgroupRealizationSubject::new(
                width(lanes),
                ArithmeticType::F32,
                SubgroupTransfer::InRangeXorShuffle,
            )
            .expect("power-of-two width at least 2 defines an XOR shuffle");
            assert_eq!(subject.width().get(), lanes);
            assert_eq!(subject.arithmetic(), ArithmeticType::F32);
            assert_eq!(subject.transfer(), SubgroupTransfer::InRangeXorShuffle);
        }
    }

    #[test]
    fn every_arithmetic_type_defines_an_xor_shuffle() {
        for arithmetic in ArithmeticType::ALL {
            SubgroupRealizationSubject::new(
                width(32),
                arithmetic,
                SubgroupTransfer::InRangeXorShuffle,
            )
            .expect("XOR shuffle is defined for every exact arithmetic type");
        }
    }

    #[test]
    fn unknown_transfer_tag_is_undefined() {
        assert_eq!(SubgroupTransfer::from_tag(0x02), None);
        assert_eq!(
            SubgroupRealizationError::UndefinedTransfer.rule(),
            "subgroup-transfer-undefined"
        );
    }

    #[test]
    fn whole_subject_equality_is_the_only_match() {
        let required = SubgroupRealizationSubject::new(
            width(32),
            ArithmeticType::F32,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .unwrap();
        let wider = SubgroupRealizationSubject::new(
            width(64),
            ArithmeticType::F32,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .unwrap();
        let bf16 = SubgroupRealizationSubject::new(
            width(32),
            ArithmeticType::Bf16,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .unwrap();
        assert_ne!(required, wider);
        assert_ne!(required, bf16);
        assert_eq!(required, required);
    }
}
