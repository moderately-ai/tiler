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

/// Which of a region's boundary input tensors one access or scalar leaf names.
///
/// The ordinal is *region-local and positional*: `0` is the first input tensor
/// the region reads, `1` the second, and a region reading `n` inputs uses every
/// ordinal in `0..n` exactly once. It is deliberately not an interface key and
/// not a semantic value — a caller that renames its inputs must not change the
/// kernel a region compiles to, and a scheduled region carries no semantic
/// correlation at all (ADR 0070). Binding an ordinal to a named program input is
/// the program layer's job, positionally, through its stage accesses.
///
/// One tensor's *components* share an ordinal and are separated by
/// [`crate::semantic::EncodedComponentRole`]: a component is schema data of one
/// tensor, never an independent operand position.
///
/// Unlike [`RegionId`], this ordinal *is* part of canonical identity: it says
/// which tensor a read addresses, and two regions that read their inputs in
/// different orders compute different things.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputOrdinal(u32);

impl InputOrdinal {
    /// The first input tensor a region reads.
    pub const FIRST: Self = Self(0);

    /// Wraps a region-local input ordinal.
    #[must_use]
    pub const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }

    /// Returns the region-local input ordinal.
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
