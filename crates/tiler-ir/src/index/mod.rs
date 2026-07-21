//! Canonical target-independent iteration, access, and scalar-region IR.
//!
//! Logical access maps end at tensor coordinates. Allocation views, strides,
//! byte addressing, target widths, execution scopes, and tail predicates belong
//! to later verified physical layers.

mod builder;
mod error;
mod handles;
mod integer;
mod model;

pub use builder::IndexRegionBuilder;
pub use error::{
    IndexBuildError, IndexEntityKind, IndexRegionBuildError, IndexRegionDiagnostic, ProofResource,
};
pub use handles::{
    BoundsWitnessId, DimensionId, IndexExprId, ScalarExprId, TensorAccessId, TensorId,
    VerifiedDimensionId, VerifiedIndexExprId, VerifiedScalarExprId, VerifiedTensorAccessId,
    VerifiedTensorId, WriteOwnershipWitnessId,
};
pub use integer::IndexInteger;
pub use model::{
    AccessMode, BoundsProofView, CanonicalIndexRegionIdentity, ContributorOrder,
    DomainDimensionRef, DomainDimensions, DomainRole, IndexExprClass, IndexExprRef, IndexExprView,
    IndexExpressions, LinearTermRef, LinearTerms, OutputRef, Outputs, ScalarEvaluationDimensions,
    ScalarExprRef, ScalarExprView, ScalarExpressions, SemanticRegionIdentity, TensorAccessRef,
    TensorAccesses, TensorRef, TensorRole, Tensors, VerifiedDimensionIds, VerifiedIndexExprIds,
    VerifiedIndexRegion, WriteOwnershipProofView,
};

/// Maximum domain dimensions in one canonical index region.
pub const MAX_DOMAIN_DIMENSIONS: usize = 1_024;
/// Maximum ordered input and output tensor boundaries in one region.
pub const MAX_BOUNDARY_TENSORS: usize = 16_384;
/// Maximum rank of one boundary tensor in the static profile.
pub const MAX_TENSOR_RANK: usize = 1_024;
/// Maximum aggregate canonical type-and-shape bytes across tensor boundaries.
pub const MAX_BOUNDARY_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum exact index-expression nodes in one canonical index region.
pub const MAX_INDEX_EXPRESSIONS: usize = 65_536;
/// Maximum operands consumed by one variadic index-expression insertion.
pub const MAX_INDEX_EXPRESSION_OPERANDS: usize = 4_096;
/// Maximum total canonical bytes retained across index expressions.
pub const MAX_INDEX_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum logical tensor accesses in one canonical index region.
pub const MAX_TENSOR_ACCESSES: usize = 16_384;
/// Maximum total canonical bytes retained across logical accesses.
pub const MAX_ACCESS_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum scalar-expression nodes in one canonical index region.
pub const MAX_SCALAR_EXPRESSIONS: usize = 65_536;
/// Maximum total canonical bytes retained across scalar expressions.
pub const MAX_SCALAR_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum scalar-expression nesting depth admitted by the first profile.
pub const MAX_SCALAR_EXPRESSION_DEPTH: usize = 256;
/// Maximum ordered output roots in one canonical index region.
pub const MAX_OUTPUT_ROOTS: usize = 4_096;
/// Maximum coordinate cells retained or evaluated by exhaustive proof fallback.
pub const MAX_EXHAUSTIVE_PROOF_CELLS: u64 = 1_048_576;
/// Maximum aggregate estimated integer bytes processed by exhaustive proof.
pub const MAX_EXHAUSTIVE_PROOF_BYTES: u64 = 64 * 1024 * 1024;
/// Maximum final canonical index-region identity bytes.
pub const MAX_INDEX_REGION_IDENTITY_BYTES: usize = 72 * 1024 * 1024;
