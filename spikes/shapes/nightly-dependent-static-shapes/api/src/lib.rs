#![feature(min_adt_const_params)]
#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]
//! Minimal authority-preserving model for dependent-array shape evidence.

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GRAPH_ID: AtomicU64 = AtomicU64::new(1);

/// One canonical structural exact-shape evidence family for every rank.
pub struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;

mod sealed {
    pub trait Sealed {}
}

/// Host-controlled, non-authoritative shape evidence checked against a graph.
pub trait ShapeEvidence: sealed::Sealed + 'static {
    /// Returns whether this evidence agrees with canonical extents.
    #[doc(hidden)]
    fn matches(extents: &[u64]) -> bool;
}

impl<const RANK: usize, const EXTENTS: [u64; RANK]> sealed::Sealed for StaticShape<RANK, EXTENTS> {}

impl<const RANK: usize, const EXTENTS: [u64; RANK]> ShapeEvidence for StaticShape<RANK, EXTENTS> {
    fn matches(extents: &[u64]) -> bool {
        extents == EXTENTS
    }
}

/// Marker used by the conformance graph.
pub enum F32 {}

/// A graph-owned typed value without Rust-side shape evidence.
pub struct Value<T> {
    graph: u64,
    index: u32,
    semantic_identity: u64,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Value<T> {}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Value<T> {
    /// Returns the evidence-independent identity used by this bounded model.
    #[must_use]
    pub const fn semantic_identity(self) -> u64 {
        self.semantic_identity
    }
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

impl<T, E> ShapedValue<T, E> {
    /// Explicitly discards Rust-side evidence without changing the value.
    #[must_use]
    pub const fn weaken(self) -> Value<T> {
        self.value
    }

    /// Returns the same identity as the underlying unrefined value.
    #[must_use]
    pub const fn semantic_identity(self) -> u64 {
        self.value.semantic_identity()
    }
}

/// Failure at the checked shape-evidence boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapeError {
    /// The value belongs to a different graph.
    ForeignGraph,
    /// The requested evidence disagrees with canonical graph metadata.
    EvidenceMismatch,
}

impl fmt::Display for ShapeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignGraph => formatter.write_str("value belongs to another graph"),
            Self::EvidenceMismatch => {
                formatter.write_str("shape evidence disagrees with canonical graph metadata")
            }
        }
    }
}

impl Error for ShapeError {}

/// Minimal authoritative graph used only by the conformance fixture.
#[derive(Debug)]
pub struct Builder {
    graph: u64,
    shapes: Vec<Vec<u64>>,
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
        }
    }

    /// Adds one canonically shaped input.
    ///
    /// # Panics
    ///
    /// Panics if this bounded fixture exceeds `u32::MAX` values.
    #[must_use]
    pub fn input<T>(&mut self, extents: impl IntoIterator<Item = u64>) -> Value<T> {
        let index = u32::try_from(self.shapes.len()).expect("bounded conformance graph");
        self.shapes.push(extents.into_iter().collect());
        Value {
            graph: self.graph,
            index,
            semantic_identity: u64::from(index),
            marker: PhantomData,
        }
    }

    /// Checks and attaches optional Rust evidence without changing graph state.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError::ForeignGraph`] for a value from another builder or
    /// [`ShapeError::EvidenceMismatch`] when `E` disagrees with canonical
    /// extents.
    pub fn refine<T, E: ShapeEvidence>(
        &self,
        value: Value<T>,
    ) -> Result<ShapedValue<T, E>, ShapeError> {
        if value.graph != self.graph {
            return Err(ShapeError::ForeignGraph);
        }
        let extents = self
            .shapes
            .get(value.index as usize)
            .ok_or(ShapeError::ForeignGraph)?;
        if !E::matches(extents) {
            return Err(ShapeError::EvidenceMismatch);
        }
        Ok(ShapedValue {
            value,
            evidence: PhantomData,
        })
    }
}
