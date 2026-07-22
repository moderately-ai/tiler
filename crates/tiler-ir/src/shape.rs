//! Target-independent fixed shape vocabulary.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

mod evidence;

pub use evidence::{Rank, ShapeEvidence, ShapeExpectation, StaticShape};

// Governed implementation limit. Keep numeric limits private; typed dynamic
// failures expose both the rejected rank and the active limit.
const MAX_SHAPE_RANK: usize = 4_096;

/// Failure to construct a bounded target-independent shape.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShapeError {
    /// The supplied rank exceeded the governed bound.
    RankTooLarge {
        /// First rejected rank.
        rank: usize,
        /// Governed maximum.
        limit: usize,
    },
}

impl fmt::Display for ShapeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RankTooLarge { rank, limit } => {
                write!(
                    formatter,
                    "tensor rank {rank} exceeds governed limit {limit}"
                )
            }
        }
    }
}

impl Error for ShapeError {}

/// The size of one logical tensor axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Extent(u64);

impl Extent {
    /// Creates an extent. Zero represents an empty axis.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the extent as a fixed-width integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A zero-based logical tensor axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Axis(u32);

impl Axis {
    /// Creates a logical axis index.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the axis as a fixed-width integer.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A fixed-rank, fixed-extent tensor shape.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Shape(Vec<Extent>);

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[")?;
        for (index, extent) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{}", extent.get())?;
        }
        formatter.write_str("]")
    }
}

impl Shape {
    /// Creates a statically bounded shape from outermost to innermost extents.
    ///
    /// A fixed array beyond the governed rank profile is rejected during
    /// compilation. Use [`Self::try_new`] for dynamically produced extents.
    #[must_use]
    pub fn new<const N: usize>(extents: [Extent; N]) -> Self {
        const { assert!(N <= MAX_SHAPE_RANK, "shape array exceeds MAX_SHAPE_RANK") };
        Self(Vec::from(extents))
    }

    /// Creates a statically bounded shape from ordinary dimension values.
    ///
    /// A fixed array beyond the governed rank profile is rejected during
    /// compilation. Use [`Self::try_from_dims`] for dynamically produced dimensions.
    #[must_use]
    pub fn from_dims<const N: usize>(extents: [u64; N]) -> Self {
        const { assert!(N <= MAX_SHAPE_RANK, "shape array exceeds MAX_SHAPE_RANK") };
        Self(Vec::from(extents).into_iter().map(Extent::new).collect())
    }

    /// Tries to collect arbitrary extents into a bounded shape.
    ///
    /// The iterator is stopped after the first over-limit item; even an
    /// infinite iterator therefore returns a typed error after bounded work.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError::RankTooLarge`] when the rank exceeds the governed bound.
    pub fn try_new(extents: impl IntoIterator<Item = Extent>) -> Result<Self, ShapeError> {
        let mut retained = Vec::new();
        for extent in extents.into_iter().take(MAX_SHAPE_RANK.saturating_add(1)) {
            if retained.len() == MAX_SHAPE_RANK {
                return Err(ShapeError::RankTooLarge {
                    rank: MAX_SHAPE_RANK.saturating_add(1),
                    limit: MAX_SHAPE_RANK,
                });
            }
            retained.push(extent);
        }
        Ok(Self(retained))
    }

    /// Tries to collect arbitrary dimension values into a bounded shape.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError::RankTooLarge`] when the rank exceeds the governed bound.
    pub fn try_from_dims(extents: impl IntoIterator<Item = u64>) -> Result<Self, ShapeError> {
        Self::try_new(extents.into_iter().map(Extent::new))
    }

    /// Returns the logical rank.
    #[must_use]
    pub fn rank(&self) -> usize {
        self.0.len()
    }

    /// Returns the extents from outermost to innermost.
    #[must_use]
    pub fn extents(&self) -> &[Extent] {
        &self.0
    }

    /// Returns the dense logical element count when it fits this host.
    ///
    /// A zero extent always produces zero, even when another extent is not
    /// representable as `usize`.
    #[must_use]
    pub fn element_count(&self) -> Option<usize> {
        if self.0.iter().any(|extent| extent.get() == 0) {
            return Some(0);
        }
        self.0.iter().try_fold(1_usize, |count, extent| {
            let extent = usize::try_from(extent.get()).ok()?;
            count.checked_mul(extent)
        })
    }

    /// Returns a shape with the named logical axes removed.
    ///
    /// Callers are responsible for validating axis range and uniqueness when
    /// those distinctions affect their semantic contract.
    #[must_use]
    pub fn without_axes(&self, axes: &[Axis]) -> Self {
        let reduced: BTreeSet<usize> = axes
            .iter()
            .filter_map(|axis| usize::try_from(axis.get()).ok())
            .collect();
        Self(
            self.0
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(index, extent)| (!reduced.contains(&index)).then_some(extent))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Extent, MAX_SHAPE_RANK, Shape, ShapeError};

    #[test]
    fn zero_extent_makes_element_count_zero_regardless_of_order() {
        assert_eq!(Shape::from_dims([u64::MAX, 2, 0]).element_count(), Some(0));
        assert_eq!(Shape::from_dims([0, u64::MAX, 2]).element_count(), Some(0));
    }

    #[test]
    fn unrepresentable_nonempty_element_count_is_a_reference_boundary() {
        assert_eq!(Shape::from_dims([u64::MAX, 2]).element_count(), None);
    }

    #[test]
    fn fallible_iterator_construction_stops_infinite_rank() {
        assert_eq!(
            Shape::try_new(std::iter::repeat(Extent::new(1))),
            Err(ShapeError::RankTooLarge {
                rank: MAX_SHAPE_RANK + 1,
                limit: MAX_SHAPE_RANK,
            })
        );
    }
}
