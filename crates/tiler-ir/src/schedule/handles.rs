//! Opaque region-local reference newtypes for the scheduled-region IR.
//!
//! Each durable reference is a layer-specific newtype backed by a compact
//! `u32`, per ADR 0071. They identify a region and its intra-region proof
//! witnesses without exposing an editable arena position. Canonical identity is
//! independent of these transient ordinals (see
//! [`super::model::CanonicalScheduledRegionIdentity`]).

/// A scheduled-region planning ordinal.
///
/// This correlates a scheduled region with the structured kernel and program
/// stage that refine it. It is a transient planning handle and is deliberately
/// excluded from canonical scheduled-region identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegionId(u32);

impl RegionId {
    /// Wraps a planning ordinal.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the planning ordinal.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A region-local reference to a bounds proof witness.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundsWitnessId(u32);

impl BoundsWitnessId {
    /// Wraps a bounds-witness ordinal.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the bounds-witness ordinal.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A region-local reference to a write-ownership proof witness.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OwnershipWitnessId(u32);

impl OwnershipWitnessId {
    /// Wraps an ownership-witness ordinal.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the ownership-witness ordinal.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
