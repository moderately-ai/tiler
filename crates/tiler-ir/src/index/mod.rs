//! Canonical target-independent iteration, access, and scalar SSA IR.

mod builder;
mod error;
mod handles;
mod integer;
mod model;
mod scalar;

pub use builder::{
    IndexRegionBuilder, ReducerScalarResults, ReducerScalarValueId, ScalarReducerBodyBuilder,
    ScalarResults,
};
pub use error::{
    IndexBuildError, IndexEntityKind, IndexLimitKind, IndexRegionBuildError, IndexRegionDiagnostic,
    ProofResource, VerifiedIndexHandleError,
};
pub use handles::{
    DimensionId, IndexExprId, ScalarOperationId, ScalarResultIndex, ScalarValueId, TensorAccessId,
    TensorId, VerifiedDimensionId, VerifiedIndexExprId, VerifiedReducerBodyOperationId,
    VerifiedReducerBodyValueId, VerifiedScalarOperationId, VerifiedScalarValueId,
    VerifiedTensorAccessId, VerifiedTensorId,
};
pub use integer::IndexInteger;
pub use integer::{IndexIntegerDecodeError, IndexIntegerSign};
pub use model::{
    AccessMode, BoundsProofView, CanonicalIndexRegionIdentity, DomainDimensionRef, DomainRole,
    IndexExprClass, IndexExprRef, IndexExprView, LinearTermRef, LinearTerms, OutputRef,
    ReducerBodyOperationRef, ReducerBodyValueDefinitionView, ReducerBodyValueRef,
    ReductionTraversal, ScalarOperationKindRef, ScalarOperationRef, ScalarReducerBodyRef,
    ScalarReductionRef, ScalarValueDefinitionView, ScalarValueRef, TensorAccessRef, TensorRef,
    TensorRole, VerifiedIndexRegion, WriteOwnershipProofView,
};
pub use scalar::{
    CanonicalScalarDefinitionProjection, CanonicalScalarRegistrySnapshotIdentity,
    FrozenScalarRegistry, ScalarAdmissionProvenanceIdentity, ScalarApplicationRejection,
    ScalarArity, ScalarAttributeField, ScalarAttributeSchema, ScalarAttributes,
    ScalarAuthorityEvidence, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
    ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
    ScalarOperationInferencer, ScalarRegistryBuilder, ScalarRegistryError,
};

/// Maximum dimensions admitted by one region.
pub const MAX_DOMAIN_DIMENSIONS: usize = 1_024;
/// Maximum input and output tensor boundaries admitted by one region.
pub const MAX_BOUNDARY_TENSORS: usize = 16_384;
/// Maximum rank of one boundary tensor.
pub const MAX_TENSOR_RANK: usize = 1_024;
/// Maximum canonical bytes retained for all tensor boundaries.
pub const MAX_BOUNDARY_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum index-expression occurrences admitted before canonical compaction.
pub const MAX_INDEX_EXPRESSIONS: usize = 65_536;
/// Maximum operands admitted by one normalized index expression.
pub const MAX_INDEX_EXPRESSION_OPERANDS: usize = 4_096;
/// Maximum dependency depth of one index expression.
pub const MAX_INDEX_EXPRESSION_DEPTH: u32 = 256;
/// Maximum canonical sign-magnitude bytes retained for one exact index integer.
pub const MAX_INDEX_INTEGER_BYTES: usize = 1024 * 1024;
/// Maximum canonical bytes retained for index expressions.
pub const MAX_INDEX_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum logical tensor accesses admitted by one region.
pub const MAX_TENSOR_ACCESSES: usize = 16_384;
/// Maximum canonical bytes retained for tensor accesses.
pub const MAX_ACCESS_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum scalar SSA values admitted by one region or reducer body.
pub const MAX_SCALAR_EXPRESSIONS: usize = 65_536;
/// Maximum dependency depth of scalar SSA in one region or reducer body.
pub const MAX_SCALAR_EXPRESSION_DEPTH: u32 = 256;
/// Maximum operands admitted by one scalar operation.
pub const MAX_SCALAR_OPERANDS: usize = 4_096;
/// Maximum canonical bytes retained for scalar SSA.
pub const MAX_SCALAR_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum named output roots admitted by one region.
pub const MAX_OUTPUT_ROOTS: usize = 4_096;
/// Maximum expression-evaluation cells used by exhaustive access proofs.
pub const MAX_EXHAUSTIVE_PROOF_CELLS: u64 = 1_048_576;
/// Maximum governed integer and dense-bitset bytes used by exhaustive proofs.
pub const MAX_EXHAUSTIVE_PROOF_BYTES: u64 = 64 * 1024 * 1024;
/// Maximum size of the final canonical region identity.
pub const MAX_INDEX_REGION_IDENTITY_BYTES: usize = 72 * 1024 * 1024;
