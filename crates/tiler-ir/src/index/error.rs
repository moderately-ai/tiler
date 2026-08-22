use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::semantic::ResolvedValueType;

use super::{
    DimensionId, IndexRegionBuilder, ScalarRegistryError, ScalarValueId, TensorAccessId, TensorId,
};
use crate::shape::{Axis, Shape};

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
    /// A write domain named a dimension that is not parallel.
    ///
    /// A write may iterate any subset of the region's parallel dimensions, so
    /// this refuses the reduction half alone: a write iterating a reduction
    /// dimension would store to one element once per reduced point.
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
    /// Output tensor and scalar value types differ.
    OutputTypeMismatch,
    /// One tensor was named as both the gather source and its index.
    ///
    /// The two operands are distinct semantic roles, and one handle cannot play
    /// both. Two handles referring to storage a future alias model proves
    /// equivalent are neither detected nor authorized here.
    GatherAliasedTensors {
        /// Handle supplied for both roles.
        tensor: TensorId,
    },
    /// The gather source is not a program input boundary.
    GatherSourceNotInput {
        /// Offending source handle.
        tensor: TensorId,
    },
    /// The gather index is not a program input boundary.
    GatherIndexNotInput {
        /// Offending index handle.
        tensor: TensorId,
    },
    /// The gather source is not exactly `tiler::f32@1`.
    GatherSourceNotF32 {
        /// Offending source handle.
        tensor: TensorId,
        /// The type it actually carries.
        actual: Arc<ResolvedValueType>,
    },
    /// The gather index is not exactly `tiler::u32@1`.
    ///
    /// A signed index is refused here rather than admitted and reinterpreted,
    /// because a signed index raises negative indexing, which this family does
    /// not answer.
    GatherIndexNotU32 {
        /// Offending index handle.
        tensor: TensorId,
        /// The type it actually carries.
        actual: Arc<ResolvedValueType>,
    },
    /// The gather source boundary shape was not authored wholly literal.
    ///
    /// Means `SourcedShape::as_static()` returned `None`. An environment that
    /// happens to determine every symbol does **not** turn authored sourced
    /// spelling into a literal boundary; sourced gather boundaries need their
    /// own accepted decision.
    GatherSourceShapeNotLiteral {
        /// Offending source handle.
        tensor: TensorId,
    },
    /// The gather index boundary shape was not authored wholly literal.
    GatherIndexShapeNotLiteral {
        /// Offending index handle.
        tensor: TensorId,
    },
    /// The gather source has rank zero and so has no axis to gather along.
    GatherSourceRankZero {
        /// Offending source handle.
        tensor: TensorId,
    },
    /// The gathered axis is not an axis of the source.
    GatherAxisOutOfRange {
        /// Supplied axis.
        axis: Axis,
        /// The source's rank.
        source_rank: usize,
    },
    /// The source-coordinate run does not supply every non-gathered source axis.
    GatherSourceCoordinateRank {
        /// Required count, one per source axis except the gathered one.
        expected: usize,
        /// Supplied count.
        actual: usize,
    },
    /// The index-coordinate run does not supply every index axis.
    GatherIndexCoordinateRank {
        /// Required count, one per index axis.
        expected: usize,
        /// Supplied count.
        actual: usize,
    },
    /// A supplied result-domain dimension's extent is not authored literal.
    ///
    /// Reports the **first** such dimension in the supplied order, so a caller
    /// repairs one cause at a time.
    GatherDomainExtentNotLiteral {
        /// Offending dimension handle.
        dimension: DimensionId,
    },
    /// The supplied result domain does not carry the derived result extents.
    ///
    /// Compared as a **multiset**: the order the domain is written in is not
    /// significant, and this never reports a domain that names the right
    /// extents in a different order. A gather's domain is a set, so the two
    /// fields below are rendered as shapes for legibility rather than because
    /// either one's axis order is part of the rule.
    ///
    /// Raised only after all three literal-shape refusals, so a nonliteral
    /// boundary is never reported as a shape disagreement.
    GatherDomainShape {
        /// Shape derived by splicing the index shape into the source at `axis`.
        expected: Shape,
        /// Extents the supplied domain declares, in the order it supplied them.
        actual: Shape,
    },
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

/// Which whole-region gather obligation one access failed.
///
/// `#[non_exhaustive]` publicly so a later admitted obligation is additive for
/// downstream readers, while every match inside `tiler-ir` stays total — the
/// crate must prove it considered each rule, and a caller must not assume the
/// list is closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherAccessRule {
    /// The source is not a program input boundary.
    SourceRole,
    /// The index is not a program input boundary.
    IndexRole,
    /// The source is not exactly `tiler::f32@1`.
    SourceType,
    /// The index is not exactly `tiler::u32@1`.
    IndexType,
    /// The source boundary shape is not authored wholly literal.
    SourceShapeLiteral,
    /// The index boundary shape is not authored wholly literal.
    IndexShapeLiteral,
    /// The source has rank zero.
    SourceRank,
    /// The gathered axis is not an axis of the source.
    Axis,
    /// The source-coordinate arity is wrong.
    SourceCoordinateRank,
    /// The index-coordinate arity is wrong.
    IndexCoordinateRank,
    /// A domain dimension's extent is not authored literal.
    DomainExtentLiteral,
    /// The declared domain does not carry the derived result extents.
    ///
    /// Compared as a multiset, exactly as
    /// [`IndexBuildError::GatherDomainShape`] compares it, so a domain naming
    /// the right extents in a different order is admitted by both surfaces.
    DomainShape,
    /// A source coordinate leaves the access domain.
    SourceCoordinateScope,
    /// An index coordinate leaves the access domain.
    IndexCoordinateScope,
    /// The retained bounds resolution does not match the access.
    BoundsResolution,
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
    /// A write was not proved total and injective.
    WriteOwnershipNotProven {
        /// Unproved write access.
        access: TensorAccessId,
    },
    /// The write roots over one output leave part of it unwritten.
    ///
    /// Raised by either partition mechanism: interval reasoning found the
    /// disjoint partitions' volumes short of the boundary's element count, or
    /// the joint enumeration finished with an element no root reached. A
    /// partition that does not cover leaves bytes whose contents no proof in
    /// this region establishes.
    OutputPartitionUncovered {
        /// Output boundary whose partition is incomplete.
        tensor: TensorId,
    },
    /// Interval reasoning proved two roots' coordinate ranges intersect.
    ///
    /// Distinct from [`Self::OutputPartitionDoubleWritten`] because it is
    /// decided over the ranges without visiting an element: no separating axis
    /// exists for the pair, so the two rectangles share at least one
    /// coordinate.
    OutputPartitionRangesOverlap {
        /// Output boundary whose roots overlap.
        tensor: TensorId,
    },
    /// The joint enumeration observed one element written by two roots.
    ///
    /// The enumerated counterpart of [`Self::OutputPartitionRangesOverlap`],
    /// reported where interval reasoning could not place every root and the
    /// shared bitset found the collision instead.
    OutputPartitionDoubleWritten {
        /// Output boundary whose roots collide.
        tensor: TensorId,
    },
    /// Whole-region revalidation refused one gather access.
    ///
    /// Owns corruption and any future internal construction. The builder's
    /// structured errors win for caller input; this is the later verifier's
    /// owner and deliberately does **not** collapse into
    /// [`Self::CoordinateOutOfBounds`], because invocation-required data is not
    /// an observed bad coordinate.
    GatherAccess {
        /// Offending access.
        access: TensorAccessId,
        /// The rule it violated.
        rule: GatherAccessRule,
    },
    /// A reachable scalar value retained an unreduced dimension.
    FreeReductionDimension {
        /// Invalid scalar value.
        value: ScalarValueId,
        /// Free reduction dimension.
        dimension: DimensionId,
    },
    /// An output root's value varies along a parallel dimension its write does
    /// not iterate.
    ///
    /// **Accepted boundary** (Tom, 2026-08-06). Added with the relaxation that
    /// lets a write declare a subset of the region's parallel dimensions; the
    /// variant, its name, and its fields were accepted as one boundary. The
    /// acceptance record is `accept-the-sub-domain-write-domain-surface`.
    ///
    /// The value-side counterpart of
    /// [`IndexBuildError::CoordinateOutsideAccessDomain`]. A coordinate has
    /// never been allowed to name a dimension outside its access domain; while
    /// every write iterated every parallel dimension the same restriction on
    /// the *stored value* held for free, and a subset domain is exactly what
    /// stops it holding. It is refused rather than interpreted because both
    /// available readings are wrong: evaluating the root once per point of the
    /// omitted dimension stores several values to one element, and picking one
    /// point of it stores a value nothing in the region selected.
    ///
    /// Distinct from [`Self::FreeReductionDimension`], which is the reduction
    /// case and stays reported under its own name: a reduction dimension is
    /// never in a write's domain, so folding the two would report every
    /// unreduced value as a domain mismatch.
    ValueDimensionOutsideWriteDomain {
        /// The write root whose domain does not supply the dimension.
        access: TensorAccessId,
        /// The stored value that varies along it.
        value: ScalarValueId,
        /// The parallel dimension the write does not iterate.
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProofResource {
    /// Cumulative planning and evaluation cells: domain and extent resolution,
    /// expression nodes and edges, predicates, coordinates, and memo work.
    Cells,
    /// Conservative arbitrary-precision integer work and transient bytes.
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
