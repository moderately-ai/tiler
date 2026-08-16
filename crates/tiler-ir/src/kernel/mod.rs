//! Backend-consumable structured kernel IR, its verifier, and its identity.
//!
//! A [`VerifiedKernel`](crate::kernel::VerifiedKernel) is the typed,
//! target-neutral, executable-shaped refinement of one
//! [`crate::schedule::VerifiedScheduledRegion`] adopted by ADR 0048 and placed
//! in this crate by ADR 0070. Construction follows the ADR 0071
//! checked-builder discipline: a public transactional
//! [`KernelBuilder`](crate::kernel::KernelBuilder) with private storage, a
//! consuming [`KernelBuilder::build`](crate::kernel::KernelBuilder::build) that
//! runs whole-kernel verification, and an opaque verified product that exposes
//! read-only meaning and a canonical identity retaining the exact identity of
//! the schedule it refines.
//!
//! # Why a backend needs nothing else
//!
//! The layer exists so a backend never reconstructs graph-specific semantics.
//! Every fact a translation needs is either a signature field or an explicit
//! operation:
//!
//! - **Typed values.** Every SSA value has a resolved
//!   [`KernelType`](crate::kernel::KernelType); there are no untyped registers
//!   and no implicit widening.
//! - **Address spaces.** Each
//!   [`BufferParameter`](crate::kernel::BufferParameter) names a governed
//!   [`AddressSpace`](crate::kernel::AddressSpace), its element type, its
//!   access mode, and the exact number of addressable elements.
//! - **Explicit indexing.** Element offsets are computed by ordinary index
//!   operations over admitted launch builtins and loop induction variables. A
//!   reduction contributor address is emitted as the row-major linearization of
//!   the scheduled access, so a backend never re-derives an access relation.
//! - **Loads and stores.** Every memory effect names its buffer, its offset,
//!   and the schedule witness that authorizes it: a
//!   [`crate::schedule::BoundsWitnessId`] for range evidence and a
//!   [`crate::schedule::OwnershipWitnessId`] for the owning commit.
//! - **Conversions.** A numerical-contract normalization is an explicit
//!   [`ConvertOp`](crate::kernel::ConvertOp), not an implicit rule a backend
//!   must remember to apply.
//! - **Loops.** A serial reduction is a bounded
//!   [`OperationView::SerialLoop`](crate::kernel::OperationView::SerialLoop)
//!   with an explicit trip count and a typed loop-carried accumulator, never an
//!   opaque reduce whose order a backend chooses.
//! - **Predicates.** Iteration-domain guarding is an explicit
//!   [`OperationView::Predicated`](crate::kernel::OperationView::Predicated)
//!   region, so tail behaviour is visible rather than implied by a launch
//!   geometry. A scalar
//!   [`OperationView::GuardedLoad`](crate::kernel::OperationView::GuardedLoad)
//!   is the value-producing form for a padded launch: true performs the
//!   bounds-witnessed load, false performs no memory access and returns the
//!   supplied inactive value.
//! - **Effects and barriers.** Memory effects are ordered, and a
//!   [`BarrierSpec`](crate::kernel::BarrierSpec) names execution scope, memory
//!   scope, fenced address spaces, and ordering separately even where one
//!   target builtin combines them.
//!
//! # What the verifier proves
//!
//! Insertion-time checks reject handle forgery, cross-builder handles,
//! out-of-scope uses, type mismatches, buffer access-mode violations,
//! undeclared builtins, non-constant or zero divisors, malformed loop ranges,
//! and yield arity or type mismatches. Whole-kernel verification then proves
//! signature agreement with the scheduled accesses, address-space legality,
//! admitted-builtin agreement with the execution binding, numerical and
//! resource agreement, predicate dominance of every effect, bounds and
//! ownership witness provenance, exactly-once output coverage, effect ordering
//! ending at the owning commit, the barrier obligation, the reduction contract,
//! and finally that the body is the canonical refinement of the exact scheduled
//! region.
//!
//! ```
//! use tiler_ir::kernel::{OperationView, lower_scheduled_region};
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId,
//!     ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan,
//!     LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
//!     OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, RegionId,
//!     ReductionTopology, ScalarProgram, ScheduledRegionBuilder, SubnormalMode,
//!     AccessOrdinal, TailPolicy, TensorRole,
//! };
//! use tiler_ir::shape::Shape;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
//! builder.iteration_shape(Shape::from_dims([4]))?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Input,
//!     component_role: None,
//!     mode: AccessMode::Read,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(0),
//!     ownership: None,
//! })?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Intermediate,
//!     component_role: None,
//!     mode: AccessMode::Write,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(1),
//!     ownership: Some(OwnershipWitnessId::new(0)),
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(0),
//!     tensor: TensorRole::Input,
//!     component_role: None,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(1),
//!     tensor: TensorRole::Intermediate,
//!     component_role: None,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.ownership_proof(OwnershipProof {
//!     id: OwnershipWitnessId::new(0),
//!     tensor: TensorRole::Intermediate,
//!     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
//! })?;
//! let mut expression = PointwiseF32ExpressionBuilder::new();
//! let input = expression.input(AccessOrdinal::FIRST)?;
//! let scale = expression.constant(2.0_f32.to_bits())?;
//! let product = expression.multiply(input, scale)?;
//! let bias = expression.constant(1.0_f32.to_bits())?;
//! let root = expression.add(product, bias)?;
//! builder.scalar_program(ScalarProgram::PointwiseF32(expression.build(root)?))?;
//! builder.numerical(NumericalRealization::new(
//!     "tiler.doc.strict-f32",
//!     0x7fc0_0000,
//!     SubnormalMode::Preserve,
//!     SubnormalMode::Preserve,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     ExceptionalValueAssumption::MakeNoAssumption,
//!     ExceptionalValueAssumption::MakeNoAssumption,
//! ))?;
//! builder.schedule(KernelSchedule {
//!     binding: ExecutionBinding::GlobalLinearInvocation,
//!     work_items: 4,
//!     threads_per_workgroup: 1,
//!     tail: TailPolicy::Exact,
//!     output_owner: OwnershipWitnessId::new(0),
//!     reduction: ReductionTopology::None,
//!     launch: LaunchPlan { grid_threads: 4, threads_per_workgroup: 1, zero_work_skips_dispatch: true },
//! })?;
//! let scheduled = builder.build()?;
//!
//! let kernel = lower_scheduled_region(&scheduled)?;
//! assert_eq!(kernel.buffers().len(), 2);
//! // The whole body is one predicated region: tail behaviour is explicit.
//! let guarded = kernel
//!     .body()
//!     .operations()
//!     .filter(|operation| matches!(operation.view(), OperationView::Predicated { .. }))
//!     .count();
//! assert_eq!(guarded, 1);
//! # Ok(())
//! # }
//! ```

mod builder;
mod error;
mod handles;
mod lower;
mod model;
mod verify;

pub use builder::{KernelBuilder, SerialLoopParameters, SerialLoopResults};
pub use error::{
    KernelBuildError, KernelComponent, KernelDiagnostic, KernelEntityKind, KernelLimitKind,
    KernelLoweringError, KernelVerificationError, VerifiedKernelHandleError,
};
pub use handles::{
    KernelBufferId, KernelInputExtentId, KernelStagingId, KernelValueId, VerifiedBufferId,
    VerifiedInputExtentId, VerifiedStagingId, VerifiedValueId,
};
pub use lower::lower_scheduled_region;
pub use model::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BlockRef, BufferAccess, BufferParameter,
    Builtin, CanonicalKernelIdentity, CompareOp, ConvertOp, ExecutionScope, InputExtentParameter,
    KernelConstant, KernelType, LoopBound, MemoryScope, OperationRef, OperationView,
    PackedExtractOp, SerialLoopRef, SerialLoopSpec, StagingParameter, UnaryOp, VerifiedKernel,
};

/// Maximum buffer parameters admitted by one kernel signature.
pub const MAX_KERNEL_BUFFERS: usize = 64;
/// Maximum workgroup staging allocations declared by one kernel.
pub const MAX_KERNEL_STAGING: usize = 64;
/// Maximum live input-extent operands admitted by one kernel signature.
pub const MAX_KERNEL_INPUT_EXTENTS: usize = 16;
/// Maximum launch builtins admitted by one kernel signature.
pub const MAX_KERNEL_ADMITTED_BUILTINS: usize = 16;
/// Maximum structured SSA values admitted by one kernel.
pub const MAX_KERNEL_VALUES: usize = 65_536;
/// Maximum structured operations admitted by one kernel.
pub const MAX_KERNEL_OPERATIONS: usize = 65_536;
/// Maximum structured blocks admitted by one kernel.
pub const MAX_KERNEL_BLOCKS: usize = 4_096;
/// Maximum lexical nesting depth of structured blocks.
pub const MAX_KERNEL_BLOCK_DEPTH: usize = 64;
/// Maximum loop-carried accumulators admitted by one structured loop.
pub const MAX_KERNEL_LOOP_ACCUMULATORS: usize = 64;
/// Maximum size of the final canonical kernel identity.
pub const MAX_KERNEL_IDENTITY_BYTES: usize = 16 * 1024 * 1024;

#[cfg(test)]
mod tests;
