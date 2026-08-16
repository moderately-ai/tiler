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
//!     ExceptionalValueAssumption, ExecutionBinding, AccessOrdinal, KernelSchedule, LaunchPlan,
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
//! let verified = builder.build()?;
//! assert_eq!(verified.requirements().buffer_bindings, 2);
//! # Ok(())
//! # }
//! ```

mod blocked;
mod builder;
mod cooperative;
mod error;
mod handles;
mod model;
mod numerics;
mod parametric;
mod pointwise;
mod pointwise_bf16;
mod subgroup;
mod synchronization;
mod witness;

pub use blocked::{
    ExactCooperativeContraction, PredicatedCooperativeContraction,
    admit_exact_cooperative_contraction, admit_predicated_cooperative_contraction,
    prove_blocked_bijection, prove_blocked_predicated_cover,
};
pub use builder::ScheduledRegionBuilder;
pub use cooperative::{
    AntiDependencyEdge, ContributorArrival, CooperativePhase, CooperativeTile,
    LocalCoordinateSource, LocalCoordinates, ParticipantRange, ParticipantSpace, StagedElement,
    StagedRead, StagedSpan, StagedWrite, VisibilityEdge, WorkgroupStaging, workgroup_tree_tile,
};
pub use error::{
    BlockedWorkgroupRule, ContributorCoverageRule, ContributorError,
    CooperativeContractionAdmission, CooperativeTileRule, ElementCountOverflow, ScheduleBuildError,
    ScheduleComponent, ScheduleLimitKind, ScheduledRegionBuildError, ScheduledRegionDiagnostic,
};
pub use handles::{
    AccessOrdinal, BoundsWitnessId, OwnershipWitnessId, PhaseId, RegionId, StagingId, SyncPointId,
};
pub use model::{
    Access, AccessMode, AxisDecode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ContractionAxisSource, ContributorCoverage, ContributorOrder, ContributorPartition,
    ExecutionBinding, IndexArithmetic, IndexRegion, KernelSchedule, LaunchPlan, LogicalAccess,
    OwnershipProof, OwnershipProofKind, ReductionPaddingIdentity, ReductionPass, ReductionTopology,
    ResourceRequirements, ScalarProgram, ScheduledRegion, TailPolicy, TensorRole,
    VerifiedScheduledRegion, axes_are_canonical, broadcast_decodes_are_replicating,
    contributor_count, cooperative_local_memory_bytes, cooperative_tile, element_count,
    live_input_extents, partial_reduction_axis, partial_reduction_shape,
    reindex_decodes_are_bijective,
};
pub(crate) use model::{REGION_INDEX_ARITHMETIC, subnormal_freedom_of};
pub use numerics::{
    ApproximationEnvelope, ArithmeticType, BF16_NUMERICAL_CONTRACT_KEY_DOMAIN,
    Bf16NumericalContractKey, ExceptionalValueAssumption, F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
    F32NumericalContractKey, FlushedZeroSign, MaterializationRounding, NumericalContractKeyError,
    NumericalPermission, NumericalRealization, SubnormalFreedom, SubnormalMode,
    ValueDomainProvenance,
};
pub use parametric::{
    BroadcastTransformClass, ParametricBroadcastRule, classify_broadcast_transform,
    environment_proves_actual_widening, interpret_parametric_broadcast, mapping_names_a_symbol,
    parametric_broadcast, parametric_broadcast_read_is_admissible,
    replication_only_transform_is_admitted,
};
pub use pointwise::{
    MAX_POINTWISE_F32_EXPRESSION_NODES, PointwiseF32Expression,
    PointwiseF32ExpressionAdmissionError, PointwiseF32ExpressionBuildError,
    PointwiseF32ExpressionBuilder, PointwiseF32ExpressionDiagnostic, PointwiseF32Node,
    PointwiseF32NodeId, PointwiseF32Value,
};
pub use pointwise_bf16::{
    MAX_POINTWISE_BF16_EXPRESSION_NODES, PointwiseBf16Expression,
    PointwiseBf16ExpressionAdmissionError, PointwiseBf16ExpressionBuildError,
    PointwiseBf16ExpressionBuilder, PointwiseBf16ExpressionDiagnostic, PointwiseBf16Node,
    PointwiseBf16NodeId, PointwiseBf16Value,
};
pub use subgroup::{
    SubgroupRealizationError, SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth,
};
pub use synchronization::{
    ConvergenceEvidence, FencedSpaces, MemoryOrdering, SynchronizationKind,
    SynchronizationPlacement, SynchronizationPoint, SynchronizationRule, SynchronizationScope,
    SynchronizationSubject, required_subject,
};
pub use witness::{
    RealizationWitness, UnevaluableRealization, UnpinnedFreedomSite, UnrecordedFoldContraction,
};

/// Maximum logical accesses admitted by one scheduled region.
pub const MAX_SCHEDULE_ACCESSES: usize = 4_096;
/// Maximum bounds proofs admitted by one scheduled region.
pub const MAX_SCHEDULE_BOUNDS_PROOFS: usize = 4_096;
/// Maximum participants admitted by one cooperative workgroup tile.
///
/// The bound exists so the tile's disjointness and coverage rules can be decided
/// by enumerating every addressed slot rather than by a modular argument over an
/// unbounded participant count. It is a verification bound, not a hardware
/// claim: nothing here asserts a target admits this many invocations per
/// workgroup, and a target profile's own workgroup-thread axis is what refuses
/// one that does not.
pub const MAX_COOPERATIVE_PARTICIPANTS: u64 = 4_096;
/// Maximum staging slots admitted across one cooperative workgroup tile.
///
/// Bounded for the reason [`MAX_COOPERATIVE_PARTICIPANTS`] is, and separately,
/// because coverage is decided over the slot space rather than the participant
/// space. The derived local-memory requirement is composed against a target's
/// declared workgroup memory by the feasibility authority; this bound never
/// stands in for that.
pub const MAX_COOPERATIVE_STAGING_SLOTS: u64 = 65_536;
/// Maximum participant dimensions admitted by one cooperative workgroup tile.
///
/// Deliberately not implied by [`MAX_COOPERATIVE_PARTICIPANTS`]: a space of unit
/// extents has a product of one at any rank, so the participant bound does not
/// bound the rank. What this bounds is the address sum a staged span evaluates
/// and the frame its encoding writes, not the enumeration — the enumeration
/// ranges over the extent product, which the participant bound already governs.
///
/// `3` and not more, because a threadgroup is at most three-dimensional on every
/// target this repository names and a fourth dimension would be a shape no
/// launch could declare. It is a verification bound rather than a hardware
/// claim, exactly as its siblings are.
///
/// Unlike its siblings this bound is *structural*: it sizes the inline arrays
/// [`ParticipantSpace`] and [`StagedSpan`] carry, so a rank above it is refused
/// by their constructors and never reaches the verifier. That is deliberate —
/// the ceiling is a property of the domain, so making it unrepresentable is
/// stronger than refusing it late, and raising it stays a one-constant edit plus
/// an identity recompute because the arrays sit behind those constructors.
pub const MAX_COOPERATIVE_PARTICIPANT_RANK: usize = 3;
/// Maximum phases admitted by one cooperative workgroup tile.
pub const MAX_COOPERATIVE_PHASES: usize = 64;
/// Maximum rounds one cooperative workgroup tile's phase sequence may execute.
///
/// Deliberately *not* an enumeration bound like the three above. Nothing walks a
/// tile's rounds: the phase sequence is verified once and the round count only
/// says how many times it repeats, so an unbounded count would not make
/// verification unbounded. What it bounds is arithmetic and emission — a
/// consumer multiplies the round count into its contributor coverage, and a
/// lowering emits it as a loop trip count — so the bound is what keeps a
/// declared round count from overflowing a product or naming a loop no launch
/// could finish. It is not a hardware claim; nothing here asserts a target can
/// run this many rounds.
pub const MAX_COOPERATIVE_ROUNDS: u64 = 65_536;
/// Maximum staged accesses admitted by one cooperative phase.
pub const MAX_COOPERATIVE_PHASE_ACCESSES: usize = 64;
/// Maximum synchronization points admitted by one cooperative workgroup tile.
///
/// A point sits at a boundary between consecutive phases or at the round
/// boundary, so a tile can need at most as many points as it has phases; the
/// bound is stated separately anyway, because the verifier enumerates points
/// against edges and a bound derived from another bound is one a reader has to
/// reconstruct.
pub const MAX_COOPERATIVE_SYNCHRONIZATION_POINTS: usize = 64;
