//! Target-independent fixed shape vocabulary.

use std::collections::BTreeSet;
use std::fmt;

mod evidence;

pub use evidence::{Rank, ShapeEvidence, ShapeExpectation, StaticShape};

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
    /// Creates a shape from outermost to innermost extents.
    #[must_use]
    pub fn new(extents: impl IntoIterator<Item = Extent>) -> Self {
        Self(extents.into_iter().collect())
    }

    /// Creates a shape from ordinary dimension values.
    #[must_use]
    pub fn from_dims(extents: impl IntoIterator<Item = u64>) -> Self {
        Self::new(extents.into_iter().map(Extent::new))
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
    use super::Shape;

    #[test]
    fn zero_extent_makes_element_count_zero_regardless_of_order() {
        assert_eq!(Shape::from_dims([u64::MAX, 2, 0]).element_count(), Some(0));
        assert_eq!(Shape::from_dims([0, u64::MAX, 2]).element_count(), Some(0));
    }

    #[test]
    fn unrepresentable_nonempty_element_count_is_a_reference_boundary() {
        assert_eq!(Shape::from_dims([u64::MAX, 2]).element_count(), None);
    }
}
