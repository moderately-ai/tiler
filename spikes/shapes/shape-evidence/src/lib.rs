//! Stable-Rust model of non-authoritative shape evidence.

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GRAPH_ID: AtomicU64 = AtomicU64::new(1);

/// Marker used by the spike's values.
pub enum F32 {}

/// A graph-owned typed value without Rust-side shape evidence.
pub struct Value<T> {
    graph: u64,
    index: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Value<T> {}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Value<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Value")
            .field("graph", &self.graph)
            .field("index", &self.index)
            .finish()
    }
}

impl<T> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.graph == other.graph && self.index == other.index
    }
}

impl<T> Eq for Value<T> {}

mod sealed {
    pub trait Sealed {}
}

/// A host-controlled class of checked, non-authoritative shape evidence.
pub trait ShapeEvidence: sealed::Sealed + 'static {
    /// Checks this evidence against canonical graph metadata.
    #[doc(hidden)]
    fn matches(extents: &[u64]) -> bool;
}

/// Evidence that a value has exactly `R` logical axes.
pub enum Rank<const R: usize> {}

impl<const R: usize> sealed::Sealed for Rank<R> {}

impl<const R: usize> ShapeEvidence for Rank<R> {
    fn matches(extents: &[u64]) -> bool {
        extents.len() == R
    }
}

/// A downstream-defined static-shape description.
///
/// Implementing this trait grants no evidence. The builder compares the
/// returned extents with authoritative graph metadata before producing a
/// refined handle.
pub trait StaticShapeSpec: 'static {
    /// Exact outermost-to-innermost extents.
    const EXTENTS: &'static [u64];
}

/// One static extent used by the tuple spelling comparison.
pub enum Dim<const N: u64> {}

/// A rank-zero static-shape family used by the owned-family spelling.
pub enum Dims0 {}

/// A rank-one static-shape family used by the owned-family spelling.
pub enum Dims1<const A: u64> {}

/// A rank-two static-shape family used by the owned-family spelling.
pub enum Dims2<const A: u64, const B: u64> {}

/// A rank-three static-shape family used by the direct-family comparison.
pub enum Dims3<const A: u64, const B: u64, const C: u64> {}

impl StaticShapeSpec for Dims0 {
    const EXTENTS: &'static [u64] = &[];
}

impl<const A: u64> StaticShapeSpec for Dims1<A> {
    const EXTENTS: &'static [u64] = &[A];
}

impl<const A: u64, const B: u64> StaticShapeSpec for Dims2<A, B> {
    const EXTENTS: &'static [u64] = &[A, B];
}

impl<const A: u64, const B: u64, const C: u64> StaticShapeSpec for Dims3<A, B, C> {
    const EXTENTS: &'static [u64] = &[A, B, C];
}

impl<const A: u64, const B: u64, const C: u64> StaticShapeSpec for (Dim<A>, Dim<B>, Dim<C>) {
    const EXTENTS: &'static [u64] = &[A, B, C];
}

/// Evidence that a value has the exact extents described by `S`.
pub struct Exact<S>(PhantomData<fn() -> S>);

impl<S: StaticShapeSpec> sealed::Sealed for Exact<S> {}

impl<S: StaticShapeSpec> ShapeEvidence for Exact<S> {
    fn matches(extents: &[u64]) -> bool {
        extents == S::EXTENTS
    }
}

/// Candidate owned exact-shape evidence for rank zero.
pub type StaticShape0 = Exact<Dims0>;
/// Candidate owned exact-shape evidence for rank one.
pub type StaticShape1<const A: u64> = Exact<Dims1<A>>;
/// Candidate owned exact-shape evidence for rank two.
pub type StaticShape2<const A: u64, const B: u64> = Exact<Dims2<A, B>>;
/// Candidate owned exact-shape evidence for rank three.
pub type StaticShape3<const A: u64, const B: u64, const C: u64> = Exact<Dims3<A, B, C>>;

/// Expands exact extents into a library-owned static-shape evidence type.
#[macro_export]
macro_rules! static_shape {
    () => {
        $crate::StaticShape0
    };
    ($a:expr) => {
        $crate::StaticShape1<{ $a }>
    };
    ($a:expr, $b:expr) => {
        $crate::StaticShape2<{ $a }, { $b }>
    };
    ($a:expr, $b:expr, $c:expr) => {
        $crate::StaticShape3<{ $a }, { $b }, { $c }>
    };
}

/// A privately constructed typed value carrying checked shape evidence.
pub struct ShapedValue<T, E> {
    value: Value<T>,
    evidence: PhantomData<fn() -> E>,
}

impl<T, E> Copy for ShapedValue<T, E> {}

impl<T, E> Clone for ShapedValue<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

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

impl<T, E> ShapedValue<T, E> {
    /// Explicitly discards Rust-side evidence without changing the value.
    #[must_use]
    pub const fn weaken(self) -> Value<T> {
        self.value
    }
}

/// A statically selected reduction axis.
pub struct Axis<const A: usize>;

/// Two statically selected reduction axes.
pub struct Axes2<const A: usize, const B: usize>;

/// Predicate proved by a same-shape witness.
pub enum SameShape {}

/// A private graph-owned proof about an ordered pair of values.
#[derive(Debug)]
pub struct ShapeWitness<P> {
    graph: u64,
    left: u32,
    right: u32,
    predicate: PhantomData<fn() -> P>,
}

/// Failure at the checked shape-evidence boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapeError {
    /// A value or witness belongs to a different graph.
    ForeignGraph,
    /// Requested evidence disagrees with canonical graph metadata.
    EvidenceMismatch,
    /// Pointwise operands have different canonical shapes.
    PointwiseMismatch,
    /// A witness names different subjects than the operation arguments.
    WitnessSubjects,
}

impl fmt::Display for ShapeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignGraph => formatter.write_str("value or witness belongs to another graph"),
            Self::EvidenceMismatch => {
                formatter.write_str("shape evidence disagrees with canonical graph metadata")
            }
            Self::PointwiseMismatch => {
                formatter.write_str("pointwise operands have different canonical shapes")
            }
            Self::WitnessSubjects => formatter.write_str("shape witness has different subjects"),
        }
    }
}

impl Error for ShapeError {}

/// Minimal authoritative graph used by the feasibility spike.
#[derive(Debug)]
pub struct Builder {
    graph: u64,
    shapes: Vec<Vec<u64>>,
    operations: Vec<(u32, u32)>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates an independent graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: NEXT_GRAPH_ID.fetch_add(1, Ordering::Relaxed),
            shapes: Vec::new(),
            operations: Vec::new(),
        }
    }

    /// Adds one canonically shaped input.
    ///
    /// # Panics
    ///
    /// Panics if this bounded spike is asked to hold more than `u32::MAX`
    /// values.
    #[must_use]
    pub fn input<T>(&mut self, extents: impl IntoIterator<Item = u64>) -> Value<T> {
        let index = u32::try_from(self.shapes.len()).expect("bounded spike graph");
        self.shapes.push(extents.into_iter().collect());
        Value {
            graph: self.graph,
            index,
            marker: PhantomData,
        }
    }

    /// Checks and attaches evidence without changing canonical graph state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign value or mismatched evidence.
    pub fn refine<T, E: ShapeEvidence>(
        &self,
        value: Value<T>,
    ) -> Result<ShapedValue<T, E>, ShapeError> {
        let shape = self.shape(value)?;
        if !E::matches(shape) {
            return Err(ShapeError::EvidenceMismatch);
        }
        Ok(ShapedValue {
            value,
            evidence: PhantomData,
        })
    }

    /// Admits a pointwise operation through the canonical unrefined path.
    ///
    /// # Errors
    ///
    /// Returns a typed error for foreign or differently shaped operands.
    pub fn pointwise<T>(
        &mut self,
        left: Value<T>,
        right: Value<T>,
    ) -> Result<Value<T>, ShapeError> {
        let left_shape = self.shape(left)?;
        let right_shape = self.shape(right)?;
        if left_shape != right_shape {
            return Err(ShapeError::PointwiseMismatch);
        }
        let result_shape = left_shape.to_vec();
        self.operations.push((left.index, right.index));
        Ok(self.input(result_shape))
    }

    /// Propagates equal evidence after using and rechecking the canonical path.
    ///
    /// # Errors
    ///
    /// Returns any canonical admission or result-refinement error.
    pub fn pointwise_shaped<T, E: ShapeEvidence>(
        &mut self,
        left: ShapedValue<T, E>,
        right: ShapedValue<T, E>,
    ) -> Result<ShapedValue<T, E>, ShapeError> {
        let result = self.pointwise(left.weaken(), right.weaken())?;
        self.refine(result)
    }

    /// Reduces one compile-time-checked axis and conservatively returns an
    /// unrefined value because stable Rust cannot generally spell `Rank<R-1>`.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the input belongs to another graph.
    pub fn reduce_axis<T, const R: usize, const A: usize>(
        &mut self,
        input: ShapedValue<T, Rank<R>>,
        _: Axis<A>,
    ) -> Result<Value<T>, ShapeError> {
        const { assert!(A < R, "reduction axis is out of range") };
        self.reduce(input.weaken(), &[A])
    }

    /// Reduces two distinct compile-time-checked axes and returns an unrefined
    /// value at the stable-Rust boundary.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the input belongs to another graph.
    pub fn reduce_axes2<T, const R: usize, const A: usize, const B: usize>(
        &mut self,
        input: ShapedValue<T, Rank<R>>,
        _: Axes2<A, B>,
    ) -> Result<Value<T>, ShapeError> {
        const { assert!(A < R && B < R, "reduction axis is out of range") };
        const { assert!(A != B, "reduction axes must be unique") };
        self.reduce(input.weaken(), &[A, B])
    }

    /// Proves canonical shape equality for an ordered value pair.
    ///
    /// # Errors
    ///
    /// Returns a typed error for foreign values or unequal shapes.
    pub fn prove_same_shape<L, R>(
        &self,
        left: Value<L>,
        right: Value<R>,
    ) -> Result<ShapeWitness<SameShape>, ShapeError> {
        if self.shape(left)? != self.shape(right)? {
            return Err(ShapeError::PointwiseMismatch);
        }
        Ok(ShapeWitness {
            graph: self.graph,
            left: left.index,
            right: right.index,
            predicate: PhantomData,
        })
    }

    /// Checks a same-shape witness against its graph and exact subjects.
    ///
    /// # Errors
    ///
    /// Returns a typed error for foreign or mismatched proof capabilities.
    pub fn check_same_shape_witness<L, R>(
        &self,
        witness: &ShapeWitness<SameShape>,
        left: Value<L>,
        right: Value<R>,
    ) -> Result<(), ShapeError> {
        self.shape(left)?;
        self.shape(right)?;
        if witness.graph != self.graph {
            return Err(ShapeError::ForeignGraph);
        }
        if witness.left != left.index || witness.right != right.index {
            return Err(ShapeError::WitnessSubjects);
        }
        Ok(())
    }

    /// Returns a small canonical-state fingerprint which excludes evidence.
    ///
    /// # Panics
    ///
    /// Panics only on a hypothetical host whose `usize` cannot fit in `u64`.
    #[must_use]
    pub fn canonical_fingerprint(&self) -> Vec<u64> {
        let mut words = vec![u64::try_from(self.shapes.len()).expect("bounded spike graph")];
        for shape in &self.shapes {
            words.push(u64::try_from(shape.len()).expect("bounded spike rank"));
            words.extend(shape);
        }
        words.push(u64::try_from(self.operations.len()).expect("bounded spike graph"));
        for (left, right) in &self.operations {
            words.extend([u64::from(*left), u64::from(*right)]);
        }
        words
    }

    fn shape<T>(&self, value: Value<T>) -> Result<&[u64], ShapeError> {
        if value.graph != self.graph {
            return Err(ShapeError::ForeignGraph);
        }
        usize::try_from(value.index)
            .ok()
            .and_then(|index| self.shapes.get(index))
            .map(Vec::as_slice)
            .ok_or(ShapeError::ForeignGraph)
    }

    fn reduce<T>(&mut self, input: Value<T>, axes: &[usize]) -> Result<Value<T>, ShapeError> {
        let shape = self.shape(input)?;
        let result: Vec<_> = shape
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(axis, extent)| (!axes.contains(&axis)).then_some(extent))
            .collect();
        Ok(self.input(result))
    }
}

/// Forces one evidence type to be checked by a workload.
#[must_use]
pub fn exercise_evidence<E: ShapeEvidence>(extents: &[u64]) -> bool {
    E::matches(extents)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TwoByThree;
    impl StaticShapeSpec for TwoByThree {
        const EXTENTS: &'static [u64] = &[2, 3];
    }

    #[test]
    fn refinement_is_checked_and_identity_neutral() {
        let mut builder = Builder::new();
        let value = builder.input::<F32>([2, 3]);
        let before = builder.canonical_fingerprint();
        let ranked = builder.refine::<_, Rank<2>>(value).unwrap();
        let exact = builder.refine::<_, Exact<TwoByThree>>(value).unwrap();
        assert_eq!(ranked.weaken(), exact.weaken());
        assert_eq!(builder.canonical_fingerprint(), before);
        assert!(matches!(
            builder.refine::<_, Rank<3>>(value),
            Err(ShapeError::EvidenceMismatch)
        ));
    }

    #[test]
    fn pointwise_evidence_uses_one_canonical_admission_path() {
        let mut plain = Builder::new();
        let left = plain.input::<F32>([2, 3]);
        let right = plain.input::<F32>([2, 3]);
        plain.pointwise(left, right).unwrap();

        let mut shaped = Builder::new();
        let left = shaped.input::<F32>([2, 3]);
        let right = shaped.input::<F32>([2, 3]);
        let left = shaped.refine::<_, Exact<TwoByThree>>(left).unwrap();
        let right = shaped.refine::<_, Exact<TwoByThree>>(right).unwrap();
        shaped.pointwise_shaped(left, right).unwrap();
        assert_eq!(
            plain.canonical_fingerprint(),
            shaped.canonical_fingerprint()
        );
    }

    #[test]
    fn reduction_checks_axes_and_weakens_result() {
        let mut builder = Builder::new();
        let input = builder.input::<F32>([2, 3, 4]);
        let input = builder.refine::<_, Rank<3>>(input).unwrap();
        let result = builder.reduce_axes2(input, Axes2::<0, 2>).unwrap();
        assert_eq!(builder.shape(result).unwrap(), &[3]);
    }

    #[test]
    fn witnesses_are_graph_and_subject_bound() {
        let mut first = Builder::new();
        let left = first.input::<F32>([2]);
        let right = first.input::<F32>([2]);
        let witness = first.prove_same_shape(left, right).unwrap();
        first
            .check_same_shape_witness(&witness, left, right)
            .unwrap();
        assert_eq!(
            first.check_same_shape_witness(&witness, right, left),
            Err(ShapeError::WitnessSubjects)
        );

        let mut second = Builder::new();
        let second_left = second.input::<F32>([2]);
        let second_right = second.input::<F32>([2]);
        assert_eq!(
            second.check_same_shape_witness(&witness, second_left, second_right),
            Err(ShapeError::ForeignGraph)
        );
    }
}
