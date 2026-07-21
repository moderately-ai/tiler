use std::fmt;
use std::marker::PhantomData;

use super::{Value, ValueId};
use crate::shape::{Rank, StaticShape};

/// A privately constructed typed value carrying checked shape evidence.
///
/// The evidence is an authoring capability only. It does not alter the
/// underlying semantic value or participate in canonical identity.
#[repr(transparent)]
pub struct ShapedValue<T, E> {
    value: Value<T>,
    evidence: PhantomData<fn() -> E>,
}

impl<T, E> ShapedValue<T, E> {
    pub(super) const fn from_verified(value: Value<T>) -> Self {
        Self {
            value,
            evidence: PhantomData,
        }
    }

    /// Explicitly discards Rust-side shape evidence.
    #[must_use]
    pub const fn weaken(self) -> Value<T> {
        self.value
    }
}

impl<T, const RANK: usize, const EXTENTS: [u64; RANK]> ShapedValue<T, StaticShape<RANK, EXTENTS>> {
    /// Discards exact extents while retaining statically proved rank.
    #[must_use]
    pub const fn forget_extents(self) -> ShapedValue<T, Rank<RANK>> {
        ShapedValue::from_verified(self.value)
    }
}

impl<T, E> Clone for ShapedValue<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for ShapedValue<T, E> {}

impl<T, E> fmt::Debug for ShapedValue<T, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ShapedValue")
            .field(&self.value)
            .finish()
    }
}

impl<T, E> PartialEq for ShapedValue<T, E> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T, E> Eq for ShapedValue<T, E> {}

impl<T, E> std::hash::Hash for ShapedValue<T, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A host-controlled predicate that a graph-owned shape witness may prove.
pub trait ShapePredicate: sealed::Sealed + 'static {}

/// The predicate that two ordered semantic values have equal canonical shapes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SameShape;

impl sealed::Sealed for SameShape {}
impl ShapePredicate for SameShape {}

/// A graph-owned proof about an ordered set of semantic values.
///
/// Construction and validation remain private to the owning builder or
/// completed program. Witnesses are transient capabilities, not durable
/// semantic identities.
pub struct ShapeWitness<P: ShapePredicate> {
    pub(super) owner: super::handles::GraphId,
    pub(super) left: ValueId,
    pub(super) right: ValueId,
    predicate: PhantomData<fn() -> P>,
}

impl<P: ShapePredicate> ShapeWitness<P> {
    pub(super) const fn from_verified(
        owner: super::handles::GraphId,
        left: ValueId,
        right: ValueId,
    ) -> Self {
        Self {
            owner,
            left,
            right,
            predicate: PhantomData,
        }
    }
}

impl<P: ShapePredicate> Clone for ShapeWitness<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ShapePredicate> Copy for ShapeWitness<P> {}

impl<P: ShapePredicate> fmt::Debug for ShapeWitness<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ShapeWitness")
            .field("left", &self.left)
            .field("right", &self.right)
            .finish_non_exhaustive()
    }
}
