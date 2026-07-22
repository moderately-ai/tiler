use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::semantic::ResolvedValueType;

use super::{
    DimensionId, IndexRegionBuilder, ScalarRegistryError, ScalarValueId, TensorAccessId, TensorId,
};

/// A governed structural resource in the canonical index profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum IndexLimitKind {
    /// Domain-dimension count.
    DomainDimensions,
    /// Boundary-tensor count.
    BoundaryTensors,
    /// Rank of one boundary tensor.
    TensorRank,
    /// Canonical boundary-description bytes.
    BoundaryCanonicalBytes,
    /// Index-expression count.
    IndexExpressions,
    /// Operand count of one index expression.
    IndexExpressionOperands,
    /// Dependency depth of one index expression.
    IndexExpressionDepth,
    /// Canonical magnitude bytes of one exact index integer.
    IndexIntegerBytes,
    /// Canonical index-expression bytes.
    IndexCanonicalBytes,
    /// Logical tensor-access count.
    TensorAccesses,
    /// Canonical access-description bytes.
    AccessCanonicalBytes,
    /// Scalar-operation count.
    ScalarOperations,
    /// Scalar-value count.
    ScalarValues,
    /// Dependency depth of one scalar expression.
    ScalarExpressionDepth,
    /// Operand count of one scalar operation.
    ScalarOperands,
    /// Canonical scalar-SSA bytes.
    ScalarCanonicalBytes,
    /// Named output-root count.
    OutputRoots,
}

impl fmt::Display for IndexLimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Builder-owned or verified entity category used by typed errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexEntityKind {
    /// Domain dimension.
    Dimension,
    /// Boundary tensor.
    Tensor,
    /// Symbolic index expression.
    IndexExpression,
    /// Logical tensor access.
    TensorAccess,
    /// Scalar operation occurrence.
    ScalarOperation,
    /// Scalar SSA value.
    ScalarValue,
    /// Named output root.
    OutputRoot,
}

impl fmt::Display for IndexEntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Failure to resolve a verified handle against a region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerifiedIndexHandleError {
    /// The handle belongs to another verified region.
    ForeignRegion {
        /// Category of rejected handle.
        entity: IndexEntityKind,
    },
    /// The handle index does not identify a retained entity.
    InvalidHandle {
        /// Category of rejected handle.
        entity: IndexEntityKind,
    },
}

impl fmt::Display for VerifiedIndexHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for VerifiedIndexHandleError {}

/// Failure during one transactional builder insertion.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexBuildError {
    /// No fresh builder ownership identity remained.
    BuilderIdentityExhausted,
    /// A handle-representable entity count was exhausted.
    TooManyEntities {
        /// Category whose handle space was exhausted.
        entity: IndexEntityKind,
    },
    /// A builder-owned handle came from another builder.
    ForeignHandle {
        /// Category of rejected handle.
        entity: IndexEntityKind,
    },
    /// A builder-owned handle did not identify a live entity.
    InvalidHandle {
        /// Category of rejected handle.
        entity: IndexEntityKind,
    },
    /// An access domain repeated a dimension.
    DuplicateAccessDimension {
        /// Repeated dimension.
        dimension: DimensionId,
    },
    /// A coordinate depends on a dimension outside its access domain.
    CoordinateOutsideAccessDomain,
    /// Coordinate count differs from tensor rank.
    AccessRank {
        /// Tensor rank.
        expected: usize,
        /// Supplied coordinate count.
        actual: usize,
    },
    /// An output tensor was used as a read boundary.
    ReadFromOutput,
    /// An input tensor was used as a write boundary.
    WriteToInput,
    /// A write domain is not exactly the parallel region domain.
    InvalidWriteDomain,
    /// An output root referred to a read access.
    OutputUsesRead,
    /// Floor division or modulo used a zero divisor.
    NonPositiveDivisor,
    /// A reduction listed a non-reduction dimension.
    ExpectedReductionDimension {
        /// Rejected dimension.
        dimension: DimensionId,
    },
    /// A reduction listed one dimension more than once.
    DuplicateReductionDimension {
        /// Repeated dimension.
        dimension: DimensionId,
    },
    /// A scalar evaluation scope listed one dimension more than once.
    DuplicateEvaluationDimension {
        /// Repeated dimension.
        dimension: DimensionId,
    },
    /// A reduction had no reduction dimensions.
    EmptyReductionDimensions,
    /// A pointwise result retained a free reduction dimension.
    PointwiseDomainContainsReductionDimension {
        /// Free reduction dimension.
        dimension: DimensionId,
    },
    /// A reduction had no accumulator state.
    EmptyReductionState,
    /// A reducer body did not declare its yielded state.
    MissingReducerYield,
    /// A reducer body attempted to set its yielded state more than once.
    ReducerYieldAlreadySet,
    /// Reducer yielded-state arity differs from accumulator-state arity.
    ReducerYieldArity {
        /// Accumulator-state arity.
        expected: usize,
        /// Yielded-state arity.
        actual: usize,
    },
    /// One yielded reducer state has the wrong semantic type.
    ReducerYieldTypeMismatch {
        /// Ordered state position.
        position: usize,
        /// Initial accumulator type.
        expected: Arc<ResolvedValueType>,
        /// Yielded value type.
        actual: Arc<ResolvedValueType>,
    },
    /// More than one root writes the same output tensor.
    DuplicateOutputTensor,
    /// Output tensor and scalar value types differ.
    OutputTypeMismatch,
    /// Scalar authority rejected registration, typing, or application.
    ScalarAuthority(Arc<ScalarRegistryError>),
    /// A governed construction resource exceeded its limit.
    StructuralLimit {
        /// Governed resource.
        resource: IndexLimitKind,
        /// Attempted quantity.
        actual: u128,
        /// Maximum admitted quantity.
        limit: u128,
    },
}

impl fmt::Display for IndexBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for IndexBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ScalarAuthority(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<ScalarRegistryError> for IndexBuildError {
    fn from(value: ScalarRegistryError) -> Self {
        Self::ScalarAuthority(Arc::new(value))
    }
}

/// One deterministic whole-region verification failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRegionDiagnostic {
    /// No named output root was declared.
    NoOutputs,
    /// A declared output tensor has no root.
    MissingOutputTensor {
        /// Missing tensor.
        tensor: TensorId,
    },
    /// A declared input tensor is unreachable from outputs.
    UnusedInputTensor {
        /// Unused tensor.
        tensor: TensorId,
    },
    /// A declared domain dimension is unreachable from outputs.
    UnusedDomainDimension {
        /// Unused dimension.
        dimension: DimensionId,
    },
    /// Exhaustive evaluation found an out-of-bounds coordinate.
    CoordinateOutOfBounds {
        /// Invalid access.
        access: TensorAccessId,
    },
    /// Bounds could not be proved within governed resources.
    BoundsNotProven {
        /// Unproved access.
        access: TensorAccessId,
    },
    /// A write was not proved total and injective.
    WriteOwnershipNotProven {
        /// Unproved write access.
        access: TensorAccessId,
    },
    /// A reachable scalar value retained an unreduced dimension.
    FreeReductionDimension {
        /// Invalid scalar value.
        value: ScalarValueId,
        /// Free reduction dimension.
        dimension: DimensionId,
    },
    /// A finite proof exceeded a governed resource budget.
    ProofResourceLimit {
        /// Exhausted proof resource.
        resource: ProofResource,
        /// Required amount.
        required: u128,
        /// Configured limit.
        limit: u64,
    },
    /// The fully encoded canonical identity exceeded its bound.
    CanonicalIdentityLimit {
        /// Encoded byte count.
        bytes: usize,
        /// Maximum byte count.
        limit: usize,
    },
}

/// Exhaustive-proof resource governed by a limit diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProofResource {
    /// Evaluated expression cells across enumerated points.
    Cells,
    /// Conservative integer storage and dense ownership-bitset bytes.
    IntegerBytes,
}

impl fmt::Display for IndexRegionDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for IndexRegionDiagnostic {}

/// Recoverable failure from consuming whole-region verification.
#[derive(Debug)]
pub struct IndexRegionBuildError {
    pub(super) builder: Box<IndexRegionBuilder>,
    pub(super) diagnostics: Vec<IndexRegionDiagnostic>,
}

impl IndexRegionBuildError {
    /// Returns all deterministic diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[IndexRegionDiagnostic] {
        &self.diagnostics
    }
    /// Recovers the intact builder and diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (IndexRegionBuilder, Vec<IndexRegionDiagnostic>) {
        (*self.builder, self.diagnostics)
    }
}

impl fmt::Display for IndexRegionBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "index-region verification failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}
impl Error for IndexRegionBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.diagnostics.first().map(|diagnostic| diagnostic as _)
    }
}

pub(super) fn invalid_handle(entity: IndexEntityKind, foreign: bool) -> IndexBuildError {
    if foreign {
        IndexBuildError::ForeignHandle { entity }
    } else {
        IndexBuildError::InvalidHandle { entity }
    }
}
