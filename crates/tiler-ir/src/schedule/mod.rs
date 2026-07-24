//! Target-neutral scheduled-region IR, its intrinsic verifier, and identity.
//!
//! A `ScheduledRegion` pairs a bounded `IndexRegion` with a normalized
//! `KernelSchedule` and is the first-class, serializable, verifiable physical
//! representation adopted by ADR 0007. Construction follows the ADR 0071
//! checked-builder discipline: a public transactional `ScheduledRegionBuilder`
//! with private storage, a consuming `ScheduledRegionBuilder::build` that runs
//! whole-region intrinsic verification, and an opaque
//! `VerifiedScheduledRegion` that exposes read-only meaning and a canonical
//! identity independent of transient planning ordinals.
//!
//! The intrinsic verifier proves domain coverage, output ownership and race
//! freedom, tail and launch legality, bounds-proof refinement, reduction
//! contributor and order legality, numerical/access agreement, and zero-domain
//! behaviour. It runs before any feasibility query and derives the
//! `ResourceRequirements` that a separate target-feasibility authority
//! consumes. This module owns no target profile, no feasibility decision, no
//! cost model, and no semantic-graph correlation; those remain compiler-owned.
//!
//! ```
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, ExecutionBinding, KernelSchedule,
//!     LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
//!     OwnershipProofKind, BoundsWitnessId, OwnershipWitnessId, RegionId, ReductionTopology,
//!     ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
//! };
//! use tiler_ir::shape::Shape;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
//! builder.iteration_shape(Shape::from_dims([4]))?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Input,
//!     mode: AccessMode::Read,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(0),
//!     ownership: None,
//! })?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Intermediate,
//!     mode: AccessMode::Write,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(1),
//!     ownership: Some(OwnershipWitnessId::new(0)),
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(0),
//!     tensor: TensorRole::Input,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(1),
//!     tensor: TensorRole::Intermediate,
//!     kind: BoundsProofKind::LinearRange { element_count: 4 },
//! })?;
//! builder.ownership_proof(OwnershipProof {
//!     id: OwnershipWitnessId::new(0),
//!     tensor: TensorRole::Intermediate,
//!     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
//! })?;
//! builder.scalar_program(ScalarProgram::MultiplyThenAdd {
//!     scale_bits: 2.0_f32.to_bits(),
//!     bias_bits: 1.0_f32.to_bits(),
//!     canonical_nan_bits: 0x7fc0_0000,
//!     contraction: false,
//! })?;
//! builder.numerical(NumericalRealization::new(
//!     "tiler.doc.strict-f32",
//!     0x7fc0_0000,
//!     SubnormalMode::Preserve,
//!     SubnormalMode::Preserve,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
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
//! let verified = builder.build()?;
//! assert_eq!(verified.requirements().buffer_bindings, 2);
//! # Ok(())
//! # }
//! ```

mod builder;
mod error;
mod handles;
mod model;
mod numerics;

pub use builder::ScheduledRegionBuilder;
pub use error::{
    ContributorError, ElementCountOverflow, ScheduleBuildError, ScheduleComponent,
    ScheduleLimitKind, ScheduledRegionBuildError, ScheduledRegionDiagnostic,
};
pub use handles::{BoundsWitnessId, OwnershipWitnessId, RegionId};
pub use model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ContributorOrder, ExecutionBinding, IndexRegion, KernelSchedule, LaunchPlan, LogicalAccess,
    OwnershipProof, OwnershipProofKind, ReductionTopology, ResourceRequirements, ScalarProgram,
    ScheduledRegion, TailPolicy, TensorRole, VerifiedScheduledRegion, axes_are_canonical,
    contributor_count, element_count,
};
pub use numerics::{FlushedZeroSign, NumericalPermission, NumericalRealization, SubnormalMode};

/// Maximum logical accesses admitted by one scheduled region.
pub const MAX_SCHEDULE_ACCESSES: usize = 4_096;
/// Maximum bounds proofs admitted by one scheduled region.
pub const MAX_SCHEDULE_BOUNDS_PROOFS: usize = 4_096;
