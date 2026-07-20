use std::sync::Arc;

use crate::shape::{Axis, Shape};

use super::handles::{GraphId, OperationId, OperationIndex, ValueId, ValueIndex};
use super::interface::InputIndex;
use super::types::ResolvedValueType;

/// The bounded profile's canonical quiet NaN produced by arithmetic.
pub const CANONICAL_F32_ARITHMETIC_NAN_BITS: u32 = 0x7fc0_0000;

/// A zero-based result position on a semantic operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResultIndex(u32);

impl ResultIndex {
    pub(super) const ZERO: Self = Self(0);

    /// Returns the fixed-width operation-result position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ValueDefinition {
    Input {
        input_index: InputIndex,
    },
    OperationResult {
        operation: OperationIndex,
        result_index: ResultIndex,
    },
}

/// The unique definition of a semantic value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Definition {
    /// An ordered program input.
    Input {
        /// Zero-based position in the program input interface.
        input_index: InputIndex,
    },
    /// One ordered result of an operation.
    OperationResult {
        /// Defining graph-owned operation.
        operation: OperationId,
        /// Zero-based result position on that operation.
        result_index: ResultIndex,
    },
}

#[derive(Clone, Debug)]
pub(super) struct ValueData {
    pub(super) definition: ValueDefinition,
    pub(super) shape: Shape,
    pub(super) resolved_type: Arc<ResolvedValueType>,
}

/// A borrowed typed value in a semantic program.
#[derive(Clone, Copy, Debug)]
pub struct ValueRef<'a> {
    pub(super) owner: GraphId,
    pub(super) index: ValueIndex,
    pub(super) value: &'a ValueData,
}

impl ValueRef<'_> {
    /// Returns the graph-owned value handle.
    #[must_use]
    pub const fn id(&self) -> ValueId {
        ValueId {
            owner: self.owner,
            index: self.index,
        }
    }

    /// Returns the value's unique definition.
    #[must_use]
    pub const fn definition(&self) -> Definition {
        match self.value.definition {
            ValueDefinition::Input { input_index } => Definition::Input { input_index },
            ValueDefinition::OperationResult {
                operation,
                result_index,
            } => Definition::OperationResult {
                operation: OperationId {
                    owner: self.owner,
                    index: operation,
                },
                result_index,
            },
        }
    }

    /// Returns the statically verified shape. Every current value is f32.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.value.shape
    }

    /// Returns the complete shape-independent semantic value type.
    #[must_use]
    pub fn resolved_type(&self) -> &ResolvedValueType {
        &self.value.resolved_type
    }
}

/// One atomic operation family supported by the bounded semantic profile.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationKind {
    /// A rank-zero f32 constant represented by exact IEEE-754 bits.
    ConstantF32 {
        /// Exact IEEE-754 binary32 payload.
        bits: u32,
    },
    /// Elementwise f32 multiplication with rank-zero scalar broadcast.
    MultiplyF32,
    /// Elementwise f32 addition with rank-zero scalar broadcast.
    AddF32,
    /// Strict serial f32 Sum with canonical, nonempty axes.
    StrictSerialSumF32 {
        /// Unique, strictly ascending input axes.
        axes: Vec<Axis>,
    },
}

impl OperationKind {
    /// Returns exact constant bits when this is a scalar constant.
    #[must_use]
    pub const fn constant_f32_bits(&self) -> Option<u32> {
        match self {
            Self::ConstantF32 { bits } => Some(*bits),
            _ => None,
        }
    }

    /// Returns canonical reduction axes when this is strict serial Sum.
    #[must_use]
    pub fn reduction_axes(&self) -> Option<&[Axis]> {
        match self {
            Self::StrictSerialSumF32 { axes } => Some(axes),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct OperationData {
    pub(super) kind: OperationKind,
    pub(super) operands: Vec<ValueIndex>,
    pub(super) results: Vec<ValueIndex>,
}

/// A borrowed atomic operation in a semantic program.
#[derive(Clone, Copy, Debug)]
pub struct OperationRef<'a> {
    pub(super) owner: GraphId,
    pub(super) index: OperationIndex,
    pub(super) operation: &'a OperationData,
}

impl OperationRef<'_> {
    /// Returns the graph-owned operation handle.
    #[must_use]
    pub const fn id(&self) -> OperationId {
        OperationId {
            owner: self.owner,
            index: self.index,
        }
    }

    /// Returns the atomic semantic operation family and attributes.
    #[must_use]
    pub const fn kind(&self) -> &OperationKind {
        &self.operation.kind
    }

    /// Returns operands in semantic order.
    #[must_use]
    pub fn operands(&self) -> impl ExactSizeIterator<Item = ValueId> + DoubleEndedIterator + '_ {
        self.operation
            .operands
            .iter()
            .copied()
            .map(|index| ValueId {
                owner: self.owner,
                index,
            })
    }

    /// Returns results in semantic order.
    #[must_use]
    pub fn results(&self) -> impl ExactSizeIterator<Item = ValueId> + DoubleEndedIterator + '_ {
        self.operation.results.iter().copied().map(|index| ValueId {
            owner: self.owner,
            index,
        })
    }
}
