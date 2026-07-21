//! Optional Rust-side evidence about canonical tensor shapes.

use super::Shape;
use std::fmt;

mod sealed {
    pub trait Sealed {}
}

/// A host-controlled, non-authoritative shape-evidence class.
///
/// Implementations are sealed because only Tiler's graph may decide which
/// claims can become proof-carrying semantic handles. Evidence is always
/// checked against canonical graph metadata before use.
pub trait ShapeEvidence: sealed::Sealed + 'static {
    /// Returns whether this evidence agrees with an authoritative shape.
    #[doc(hidden)]
    fn matches(shape: &Shape) -> bool;

    /// Describes the evidence for a typed mismatch diagnostic.
    #[doc(hidden)]
    fn expectation() -> ShapeExpectation;
}

/// A diagnostic description of requested Rust-side shape evidence.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ShapeExpectation {
    /// Exactly this many logical axes.
    Rank(usize),
    /// Exactly these outermost-to-innermost extents.
    Exact(Shape),
}

impl fmt::Display for ShapeExpectation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rank(rank) => write!(formatter, "rank {rank}"),
            Self::Exact(shape) => write!(formatter, "exact shape {shape}"),
        }
    }
}

/// Evidence that a tensor has exactly `RANK` logical axes.
pub struct Rank<const RANK: usize>;

impl<const RANK: usize> sealed::Sealed for Rank<RANK> {}

impl<const RANK: usize> ShapeEvidence for Rank<RANK> {
    fn matches(shape: &Shape) -> bool {
        shape.rank() == RANK
    }

    fn expectation() -> ShapeExpectation {
        ShapeExpectation::Rank(RANK)
    }
}

/// Exact static extents represented by one structural type at every rank.
///
/// `RANK` uses `usize` because it is an array length. Extents use Tiler's
/// fixed-width `u64` semantic domain and are ordered outermost to innermost.
pub struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;

impl<const RANK: usize, const EXTENTS: [u64; RANK]> sealed::Sealed for StaticShape<RANK, EXTENTS> {}

impl<const RANK: usize, const EXTENTS: [u64; RANK]> ShapeEvidence for StaticShape<RANK, EXTENTS> {
    fn matches(shape: &Shape) -> bool {
        shape.rank() == RANK
            && shape
                .extents()
                .iter()
                .zip(EXTENTS)
                .all(|(actual, expected)| actual.get() == expected)
    }

    fn expectation() -> ShapeExpectation {
        ShapeExpectation::Exact(Shape::from_dims(EXTENTS))
    }
}

#[cfg(test)]
mod tests {
    use super::{Rank, ShapeEvidence, ShapeExpectation, StaticShape};
    use crate::shape::Shape;
    use std::mem::size_of;

    #[test]
    fn rank_and_exact_evidence_match_only_their_claims() {
        type Matrix = StaticShape<2, { [2, 3] }>;

        assert!(Rank::<2>::matches(&Shape::from_dims([7, 11])));
        assert!(!Rank::<2>::matches(&Shape::from_dims([7])));
        assert!(Matrix::matches(&Shape::from_dims([2, 3])));
        assert!(!Matrix::matches(&Shape::from_dims([3, 2])));
        assert_eq!(
            Matrix::expectation(),
            ShapeExpectation::Exact(Shape::from_dims([2, 3]))
        );
    }

    #[test]
    fn evidence_types_are_zero_sized_at_arbitrary_rank() {
        assert_eq!(size_of::<Rank<64>>(), 0);
        assert_eq!(size_of::<StaticShape<0, { [] }>>(), 0);
        assert_eq!(size_of::<StaticShape<64, { [1; 64] }>>(), 0);
    }
}
