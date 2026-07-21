use std::error::Error;
use std::fmt;

use super::{DimensionId, IndexRegionBuilder, ScalarExprId, TensorAccessId, TensorId};

/// Builder-owned entity categories used by typed diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexEntityKind {
    /// An iteration-domain dimension.
    Dimension,
    /// A tensor at the region boundary.
    Tensor,
    /// An exact index expression.
    IndexExpression,
    /// A logical tensor access.
    TensorAccess,
    /// A scalar expression.
    ScalarExpression,
    /// An output root.
    OutputRoot,
}

impl fmt::Display for IndexEntityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Dimension => "domain dimension",
            Self::Tensor => "boundary tensor",
            Self::IndexExpression => "index expression",
            Self::TensorAccess => "tensor access",
            Self::ScalarExpression => "scalar expression",
            Self::OutputRoot => "output root",
        })
    }
}

/// Failure during one transactional index-region insertion.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexBuildError {
    /// The process-local builder identity space was exhausted.
    BuilderIdentityExhausted,
    /// An arena exhausted its fixed-width handle space.
    TooManyEntities {
        /// Arena category that overflowed its compact index.
        entity: IndexEntityKind,
    },
    /// A handle belongs to another builder.
    ForeignHandle {
        /// Rejected handle category.
        entity: IndexEntityKind,
    },
    /// A handle has the correct owner but no local entity.
    InvalidHandle {
        /// Rejected handle category.
        entity: IndexEntityKind,
    },
    /// An access domain repeated a dimension.
    DuplicateAccessDimension {
        /// Repeated dimension.
        dimension: DimensionId,
    },
    /// A coordinate depends on a dimension outside the access domain.
    CoordinateOutsideAccessDomain,
    /// An access coordinate count disagrees with tensor rank.
    AccessRank {
        /// Authoritative logical tensor rank.
        expected: usize,
        /// Supplied coordinate count.
        actual: usize,
    },
    /// A read was requested from an output boundary tensor.
    ReadFromOutput,
    /// A write was requested to an input boundary tensor.
    WriteToInput,
    /// An ordinary write domain was not exactly the parallel domain.
    InvalidWriteDomain,
    /// A scalar load referenced a write access.
    LoadFromWrite,
    /// An output root referenced a read access.
    OutputUsesRead,
    /// A scalar primitive or tensor requires the first profile's F32 type.
    ScalarTypeMismatch,
    /// A floor-div or modulo divisor was zero.
    NonPositiveDivisor,
    /// A reduction dimension was not declared with reduction role.
    ExpectedReductionDimension {
        /// Rejected dimension.
        dimension: DimensionId,
    },
    /// A reduction dimension was repeated.
    DuplicateReductionDimension {
        /// Repeated dimension.
        dimension: DimensionId,
    },
    /// The authoritative structural budget was exceeded.
    StructuralLimit {
        /// Bounded entity category.
        entity: IndexEntityKind,
        /// Authoritative maximum count or byte budget.
        limit: usize,
    },
}

impl fmt::Display for IndexBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuilderIdentityExhausted => {
                formatter.write_str("index-region builder identity space is exhausted")
            }
            Self::TooManyEntities { entity } => write!(formatter, "too many {entity} entities"),
            Self::ForeignHandle { entity } => {
                write!(
                    formatter,
                    "{entity} handle belongs to another index builder"
                )
            }
            Self::InvalidHandle { entity } => {
                write!(
                    formatter,
                    "{entity} handle is invalid in this index builder"
                )
            }
            Self::DuplicateAccessDimension { dimension } => {
                write!(formatter, "access domain repeats dimension {dimension:?}")
            }
            Self::CoordinateOutsideAccessDomain => formatter.write_str(
                "an access coordinate depends on a dimension outside its evaluation domain",
            ),
            Self::AccessRank { expected, actual } => write!(
                formatter,
                "access rank {actual} does not match tensor rank {expected}"
            ),
            Self::ReadFromOutput => {
                formatter.write_str("an output boundary tensor cannot be read in this profile")
            }
            Self::WriteToInput => formatter.write_str("an input boundary tensor cannot be written"),
            Self::InvalidWriteDomain => formatter.write_str(
                "an ordinary write domain must be exactly the declared parallel dimensions",
            ),
            Self::LoadFromWrite => formatter.write_str("a scalar load requires a read access"),
            Self::OutputUsesRead => formatter.write_str("an output root requires a write access"),
            Self::ScalarTypeMismatch => {
                formatter.write_str("the first scalar profile requires governed F32 values")
            }
            Self::NonPositiveDivisor => {
                formatter.write_str("Euclidean division and modulo require a positive divisor")
            }
            Self::ExpectedReductionDimension { dimension } => {
                write!(formatter, "{dimension:?} is not a reduction dimension")
            }
            Self::DuplicateReductionDimension { dimension } => {
                write!(formatter, "reduction dimension {dimension:?} is repeated")
            }
            Self::StructuralLimit { entity, limit } => {
                write!(formatter, "{entity} exceeds authoritative limit {limit}")
            }
        }
    }
}

impl Error for IndexBuildError {}

/// One whole-region verification failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRegionDiagnostic {
    /// The region has no output root.
    NoOutputs,
    /// A declared output tensor has no ordinary write root.
    MissingOutputTensor {
        /// Unwritten output tensor.
        tensor: TensorId,
    },
    /// A declared reduction dimension was never bound by a reachable reduction.
    UnusedDomainDimension {
        /// Unused reduction dimension.
        dimension: DimensionId,
    },
    /// An access coordinate could not be proven in bounds.
    BoundsNotProven {
        /// Access whose bounds remain unknown.
        access: TensorAccessId,
    },
    /// Sound interval or finite evidence proved an access coordinate out of bounds.
    CoordinateOutOfBounds {
        /// Access disproved by a concrete point.
        access: TensorAccessId,
    },
    /// An ordinary write was not proven complete and unique.
    WriteOwnershipNotProven {
        /// Write whose coverage or uniqueness remains unknown.
        access: TensorAccessId,
    },
    /// One output tensor was assigned more than one ordinary write root.
    DuplicateOutputTensor {
        /// Repeated output tensor.
        tensor: TensorId,
    },
    /// A reduction variable reaches an output without a reduction binder.
    FreeReductionDimension {
        /// Output scalar expression.
        expression: ScalarExprId,
        /// Unbound reduction dimension.
        dimension: DimensionId,
    },
    /// A reduction binder names a dimension unused by its body.
    UnusedReductionDimension {
        /// Reduction scalar expression.
        expression: ScalarExprId,
        /// Named but unused reduction dimension.
        dimension: DimensionId,
    },
    /// Exhaustive proof would exceed the governed work or memory cap.
    ProofResourceLimit {
        /// Governed resource that would be exceeded.
        resource: ProofResource,
        /// Required cell count or conservative estimated integer-byte count.
        required: u128,
        /// Governed resource cap.
        limit: u64,
    },
    /// The final canonical identity exceeded its governed byte limit.
    CanonicalIdentityLimit {
        /// Required canonical bytes.
        bytes: usize,
        /// Governed byte limit.
        limit: usize,
    },
}

/// Exhaustive-proof resource governed by a limit diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProofResource {
    /// Evaluated coordinate/expression cells.
    Cells,
    /// Estimated aggregate arbitrary-precision integer bytes.
    IntegerBytes,
}

impl fmt::Display for IndexRegionDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOutputs => formatter.write_str("an index region requires at least one output"),
            Self::MissingOutputTensor { tensor } => {
                write!(formatter, "output tensor {tensor:?} has no write root")
            }
            Self::UnusedDomainDimension { dimension } => {
                write!(
                    formatter,
                    "reduction dimension {dimension:?} is never bound"
                )
            }
            Self::BoundsNotProven { access } => {
                write!(formatter, "bounds were not proved for access {access:?}")
            }
            Self::CoordinateOutOfBounds { access } => {
                write!(
                    formatter,
                    "access {access:?} has an out-of-bounds coordinate"
                )
            }
            Self::WriteOwnershipNotProven { access } => write!(
                formatter,
                "complete unique ownership was not proved for write {access:?}"
            ),
            Self::DuplicateOutputTensor { tensor } => {
                write!(
                    formatter,
                    "output tensor {tensor:?} has more than one write root"
                )
            }
            Self::FreeReductionDimension {
                expression,
                dimension,
            } => write!(
                formatter,
                "scalar expression {expression:?} leaves reduction dimension {dimension:?} free"
            ),
            Self::UnusedReductionDimension {
                expression,
                dimension,
            } => write!(
                formatter,
                "scalar expression {expression:?} binds unused reduction dimension {dimension:?}"
            ),
            Self::ProofResourceLimit {
                resource,
                required,
                limit,
            } => write!(
                formatter,
                "proof requires {required} {resource}, exceeding governed cap {limit}"
            ),
            Self::CanonicalIdentityLimit { bytes, limit } => write!(
                formatter,
                "canonical index-region identity requires {bytes} bytes, exceeding limit {limit}"
            ),
        }
    }
}

impl fmt::Display for ProofResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Cells => "evaluation cells",
            Self::IntegerBytes => "estimated integer bytes",
        })
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "index-region verification failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for IndexRegionBuildError {}

pub(super) fn invalid_handle(entity: IndexEntityKind, foreign: bool) -> IndexBuildError {
    if foreign {
        IndexBuildError::ForeignHandle { entity }
    } else {
        IndexBuildError::InvalidHandle { entity }
    }
}
