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
//! # Why the leaf descriptors expose fields
//!
//! This module's leaf descriptors — [`crate::schedule::IndexRegion`],
//! [`crate::schedule::Access`], [`crate::schedule::KernelSchedule`],
//! [`crate::schedule::BoundsProof`], and [`crate::schedule::OwnershipProof`] —
//! are `pub`-field value data, while the sibling [`crate::index`] module reaches
//! its data through view accessors. That is deliberate, and the two are not
//! comparable the way they first appear: they sit on opposite sides of a
//! verification boundary.
//!
//! `tiler_ir::index`'s public type *is* the verified product, so it must be
//! opaque and hand out views. This module's verified product is
//! [`crate::schedule::VerifiedScheduledRegion`], which is equally opaque —
//! private fields, a `pub(super)` constructor, and read-only accessors.
//! [`crate::schedule::ScheduledRegion`] is the *unverified proposal* submitted
//! to `ScheduledRegionBuilder::from_region`, and the read-only borrow that
//! `VerifiedScheduledRegion::region` hands back. The honest comparison is
//! `VerifiedScheduledRegion` against
//! [`crate::index::VerifiedIndexRegion`], and both are opaque; comparing
//! `ScheduledRegion`'s fields against `VerifiedIndexRegion`'s accessors compares
//! an input to an output.
//!
//! Nothing here maintains a field-level invariant between calls: every
//! descriptor is a closed enum, a [`crate::shape::Shape`], or a fixed-width bit
//! pattern, and every invariant relating them is a whole-region property the
//! intrinsic verifier proves at `build`. Accessors would therefore add ceremony
//! without moving a check earlier. Struct-literal construction also earns
//! something accessors would cost: adding a descriptor field is a compile error
//! at every construction site, so a new physical fact cannot be silently
//! defaulted by a producer that has not been taught about it.
//!
//! A consumer can of course clone a borrowed `ScheduledRegion`, edit a field,
//! and resubmit it — which is exactly why `from_region` re-verifies rather than
//! trusting its input.
//!
//! ```
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId,
//!     ExceptionalValueAssumption, ExecutionBinding, InputOrdinal, KernelSchedule, LaunchPlan,
//!     LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
//!     OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, RegionId,
//!     ReductionTopology, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
//!     TensorRole,
//! };
//! use tiler_ir::shape::Shape;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
//! builder.iteration_shape(Shape::from_dims([4]))?;
//! builder.push_access(Access {
//!     tensor: TensorRole::Input { ordinal: InputOrdinal::FIRST },
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
//!     tensor: TensorRole::Input { ordinal: InputOrdinal::FIRST },
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
//! let input = expression.input(InputOrdinal::FIRST)?;
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
mod pointwise;

pub use builder::ScheduledRegionBuilder;
pub use error::{
    ContributorError, ElementCountOverflow, ScheduleBuildError, ScheduleComponent,
    ScheduleLimitKind, ScheduledRegionBuildError, ScheduledRegionDiagnostic,
};
pub use handles::{BoundsWitnessId, InputOrdinal, OwnershipWitnessId, RegionId};
pub(crate) use model::subnormal_freedom_of;
pub use model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ContributorOrder, ContributorPartition, ExecutionBinding, IndexRegion, KernelSchedule,
    LaunchPlan, LogicalAccess, OwnershipProof, OwnershipProofKind, ReductionPass,
    ReductionTopology, ResourceRequirements, ScalarProgram, ScheduledRegion, TailPolicy,
    TensorRole, VerifiedScheduledRegion, axes_are_canonical, contributor_count, element_count,
    partial_reduction_axis, partial_reduction_shape,
};
pub use numerics::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, NumericalRealization, SubnormalFreedom,
    SubnormalMode, ValueDomainProvenance,
};
pub use pointwise::{
    MAX_POINTWISE_F32_EXPRESSION_NODES, PointwiseF32Expression,
    PointwiseF32ExpressionAdmissionError, PointwiseF32ExpressionBuildError,
    PointwiseF32ExpressionBuilder, PointwiseF32ExpressionDiagnostic, PointwiseF32Node,
    PointwiseF32NodeId, PointwiseF32Value,
};

/// Maximum logical accesses admitted by one scheduled region.
pub const MAX_SCHEDULE_ACCESSES: usize = 4_096;
/// Maximum bounds proofs admitted by one scheduled region.
pub const MAX_SCHEDULE_BOUNDS_PROOFS: usize = 4_096;
