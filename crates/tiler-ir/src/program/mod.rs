//! Target-neutral kernel-program IR, its whole-program verifier, and identity.
//!
//! A [`VerifiedKernelProgram`](crate::program::VerifiedKernelProgram) is the
//! compiler's *execution intent* for one
//! semantic program: an acyclic stage DAG whose stages dispatch exact verified
//! structured kernels, over checked materialized values, byte views, and
//! storage allocations with proven lifetimes and handoffs, ordered by typed
//! dependencies, publishing an ordered list of named outputs, and covering the
//! bound semantic graph completely. It is placed in this crate by ADR 0070 and
//! constructed under the ADR 0071 checked-builder discipline.
//!
//! It is deliberately **not** the artifact manifest and not a codec. A later
//! artifact-facing projection owns packaged admission, selected-provider
//! provenance, the wire encoding of the ABI, and a portfolio's variant
//! priority; this layer owns only what a program *is*.
//!
//! # Identity carries the ADR 0072 layers
//!
//! [`CanonicalKernelProgramIdentity`](crate::program::CanonicalKernelProgramIdentity)
//! folds every subject ADR 0072 assigns to complete program identity, together
//! with the program structure that binds them:
//!
//! - **Semantic graph identity.** The canonical
//!   [`SemanticGraphIdentity`](crate::semantic::SemanticGraphIdentity) of the
//!   program being realized — meaning only, never provider provenance.
//! - **Bound implementations.** Each stage's
//!   [`CanonicalKernelIdentity`](crate::kernel::CanonicalKernelIdentity), which
//!   already folds the exact
//!   [`CanonicalScheduledRegionIdentity`](crate::schedule::CanonicalScheduledRegionIdentity)
//!   it refines. A program identity therefore changes when any selected
//!   refinement changes, at any structural layer.
//! - **Complete coverage.** The semantic occurrences each stage claims, proven
//!   to be a disjoint partition of every operation of the bound graph.
//! - **Materializations and buffers.** Values, byte views, allocations, typed
//!   dependencies, and named outputs.
//! - **The entry ABI.** Each stage's launch geometry and each access's
//!   addressable byte range, as [`abi`](crate::program::abi) expressions rather than resolved
//!   numbers.
//! - **The applicability guard.** The predicate deciding whether this program
//!   may be routed to at all.
//! - **The routing-commit contract.** The ordered lifecycle from preflight to
//!   publication and, for each step, whether fallback is still permitted.
//!
//! The last three landed with `complete-program-identity-with-abi-guards-and-routing`, which moved the domain from `tiler.kernel-program.v1` to historical v2 because a v1 identity was blind to two programs that differed only in their guard, ABI, or fallback contract. Later encoding and ABI-completeness changes moved the same subject through v3 and v4 to the current `tiler.kernel-program.v5`; [`CanonicalKernelProgramIdentity`](crate::program::CanonicalKernelProgramIdentity) documents each step.
//!
//! Every transient ordinal is excluded: builder insertion order, the program's
//! own stage/value/view/allocation/arena positions, and the planning `RegionId`
//! already excluded below. Cross-references are encoded by canonical content
//! key, so two structurally equal programs assembled in different orders share
//! identity bytes.
//!
//! # What the verifier proves
//!
//! Insertion-time checks reject handle forgery, cross-builder handles,
//! interface disagreement, invalid alignment, insufficient or mismatched
//! allocations, out-of-range views, stage/kernel signature disagreement,
//! out-of-range or repeated coverage, self and duplicate dependencies, unknown
//! or repeated output keys, mistyped or phase-escaping ABI expressions, an
//! accessible range or workgroup width the program's own view or kernel
//! contradicts, a second applicability guard, and a routing-commit step that
//! breaks the lifecycle order or permits fallback after commit. Whole-program
//! verification then proves unique writers, no write to an externally bound
//! input, complete and disjoint semantic coverage, unambiguous canonical keys,
//! an acyclic dependency graph with a data edge behind every read, dependency
//! edges that state obligations their stages actually realize, no unused value,
//! view, allocation, or ABI expression, a declared applicability guard, a
//! routing-commit contract carried to publication, the baseline aliasing
//! contract, the conservative storage-reuse contract (non-overlapping
//! lifetimes, an explicit handoff, and no live alias across it), and complete
//! named-output coverage.
//!
//! It does **not** prove that a stage's kernel computes the semantic operations
//! it covers. Coverage here is a structural completeness and disjointness
//! obligation; semantic equivalence remains compiler-owned refinement evidence.
//!
//! ```
//! use tiler_ir::kernel::{KernelType, lower_scheduled_region};
//! use tiler_ir::program::abi::AbiRoot;
//! use tiler_ir::program::{
//!     AllocationOwnership, AllocationSpec, KernelProgramBuilder, MaterializedOrigin,
//!     MaterializedValueSpec, MemorySpace, RoutingCommitState, RoutingCommitTransition,
//!     SemanticOccurrence, StageAccess, StageAccessMode, StageLaunch, StorageEncoding,
//!     StorageScalar, ValueRole,
//! };
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
//!     ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
//!     NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
//!     OwnershipWitnessId, RegionId, ReductionTopology, ScalarProgram, ScheduledRegionBuilder,
//!     SubnormalMode, TailPolicy, TensorRole,
//! };
//! use tiler_ir::semantic::{
//!     F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
//!     StrictSerialF32Sum,
//! };
//! use tiler_ir::shape::{Axis, Shape};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // result = strict_serial_sum(input * 2.0 + 1.0, axis 1): five graph operations.
//! let mut draft = SemanticProgramBuilder::try_standard()?;
//! let input = draft.input::<F32>(InputKey::new("input")?, Shape::from_dims([2, 3]))?;
//! let scale = F32Constant::apply(&mut draft, 2.0_f32.to_bits())?;
//! let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits())?;
//! let product = F32Multiply::apply(&mut draft, input, scale)?;
//! let mapped = F32Add::apply(&mut draft, product, bias)?;
//! let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)])?;
//! draft.output(OutputKey::new("result")?, sum)?;
//! let semantic = draft.build()?;
//!
//! // One fused scheduled region, lowered to its verified structured kernel.
//! let axes = vec![Axis::new(1)];
//! let contributor = LogicalAccess::ReductionContributor {
//!     input_shape: Shape::from_dims([2, 3]),
//!     output_shape: Shape::from_dims([2]),
//!     axes: axes.clone(),
//!     order: ContributorOrder::OriginalAxisLexicographic,
//! };
//! let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
//! region.iteration_shape(Shape::from_dims([2]))?;
//! region.push_access(Access {
//!     tensor: TensorRole::Input,
//!     component_role: None,
//!     mode: AccessMode::Read,
//!     map: contributor,
//!     bounds: BoundsWitnessId::new(0),
//!     ownership: None,
//! })?;
//! region.push_access(Access {
//!     tensor: TensorRole::Output,
//!     component_role: None,
//!     mode: AccessMode::Write,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(1),
//!     ownership: Some(OwnershipWitnessId::new(0)),
//! })?;
//! region.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(0),
//!     tensor: TensorRole::Input,
//!     component_role: None,
//!     kind: BoundsProofKind::ReductionDomain {
//!         input_shape: Shape::from_dims([2, 3]),
//!         output_shape: Shape::from_dims([2]),
//!         axes: axes.clone(),
//!         order: ContributorOrder::OriginalAxisLexicographic,
//!     },
//! })?;
//! region.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(1),
//!     tensor: TensorRole::Output,
//!     component_role: None,
//!     kind: BoundsProofKind::LinearRange { element_count: 2 },
//! })?;
//! region.ownership_proof(OwnershipProof {
//!     id: OwnershipWitnessId::new(0),
//!     tensor: TensorRole::Output,
//!     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
//! })?;
//! region.scalar_program(ScalarProgram::FusedMultiplyAddSerialSum {
//!     scale_bits: 2.0_f32.to_bits(),
//!     bias_bits: 1.0_f32.to_bits(),
//!     axes: axes.clone(),
//!     order: ContributorOrder::OriginalAxisLexicographic,
//!     canonical_nan_bits: 0x7fc0_0000,
//!     empty_identity_bits: 0.0_f32.to_bits(),
//!     contraction: false,
//! })?;
//! region.numerical(NumericalRealization::new(
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
//! region.schedule(KernelSchedule {
//!     binding: ExecutionBinding::GlobalLinearInvocation,
//!     work_items: 2,
//!     threads_per_workgroup: 1,
//!     tail: TailPolicy::Exact,
//!     output_owner: OwnershipWitnessId::new(0),
//!     reduction: ReductionTopology::Serial {
//!         axes,
//!         order: ContributorOrder::OriginalAxisLexicographic,
//!         permits_reassociation: false,
//!         permits_permutation: false,
//!     },
//!     launch: LaunchPlan { grid_threads: 2, threads_per_workgroup: 1, zero_work_skips_dispatch: true },
//! })?;
//! let kernel = lower_scheduled_region(&region.build()?)?;
//!
//! // One stage covering every operation of that exact graph.
//! let mut program = KernelProgramBuilder::new(&semantic)?;
//! let external = program.push_allocation(AllocationSpec {
//!     capacity_bytes: 24,
//!     alignment: 4,
//!     memory_space: MemorySpace::Device,
//!     ownership: AllocationOwnership::External,
//! })?;
//! let owned = program.push_allocation(AllocationSpec {
//!     capacity_bytes: 8,
//!     alignment: 4,
//!     memory_space: MemorySpace::Device,
//!     ownership: AllocationOwnership::Program,
//! })?;
//! let source = program.push_value(
//!     MaterializedValueSpec {
//!         origin: MaterializedOrigin::ProgramInput { key: InputKey::new("input")? },
//!         role: ValueRole::Input,
//!         shape: Shape::from_dims([2, 3]),
//!         storage_scalar: StorageScalar::F32,
//!         encoding: StorageEncoding::Unpacked,
//!         element_type: KernelType::F32,
//!         alignment: 4,
//!         memory_space: MemorySpace::Device,
//!     },
//!     external,
//! )?;
//! let result = program.push_value(
//!     MaterializedValueSpec {
//!         origin: MaterializedOrigin::Internal,
//!         role: ValueRole::Output,
//!         shape: Shape::from_dims([2]),
//!         storage_scalar: StorageScalar::F32,
//!         encoding: StorageEncoding::Unpacked,
//!         element_type: KernelType::F32,
//!         alignment: 4,
//!         memory_space: MemorySpace::Device,
//!     },
//!     owned,
//! )?;
//! let read = program.push_whole_view(source)?;
//! let write = program.push_whole_view(result)?;
//!
//! // The entry ABI: what each access may address, and how the stage launches.
//! // The bounded profile's shapes are static, so each is a literal; a dynamic
//! // subject would name `AbiRoot::InputExtent` here instead, with no change to
//! // the shape of this contract.
//! let read_bytes = program.push_abi_root(AbiRoot::UnsignedLiteral(24))?;
//! let write_bytes = program.push_abi_root(AbiRoot::UnsignedLiteral(8))?;
//! let grid_threads = program.push_abi_root(AbiRoot::UnsignedLiteral(2))?;
//! let threads_per_workgroup = program.push_abi_root(AbiRoot::UnsignedLiteral(1))?;
//! let guard = program.push_abi_root(AbiRoot::BooleanLiteral(true))?;
//! program.applicability_guard(guard)?;
//!
//! program.push_stage(
//!     &kernel,
//!     &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
//!     &[
//!         StageAccess { view: read, mode: StageAccessMode::Read, accessible_bytes: read_bytes },
//!         StageAccess { view: write, mode: StageAccessMode::Write, accessible_bytes: write_bytes },
//!     ],
//!     StageLaunch { grid_threads, threads_per_workgroup },
//! )?;
//! program.push_output(OutputKey::new("result")?, result)?;
//!
//! // Fallback is still legal while nothing is committed, and never after.
//! for (from, to, fallback_permitted) in [
//!     (RoutingCommitState::Preflight, RoutingCommitState::Committed, true),
//!     (RoutingCommitState::Committed, RoutingCommitState::Executing, false),
//!     (RoutingCommitState::Executing, RoutingCommitState::Published, false),
//! ] {
//!     program.push_routing_commit_transition(
//!         RoutingCommitTransition { from, to, fallback_permitted },
//!     )?;
//! }
//! let program = program.build()?;
//!
//! assert_eq!(program.stages().len(), 1);
//! assert_eq!(program.execution_order().len(), 1);
//! assert_eq!(program.outputs().len(), 1);
//! // Four distinct unsigned literals — 24, 8, 2, 1 — and the guard predicate.
//! assert_eq!(program.abi_expressions().len(), 5);
//! assert_eq!(program.routing_commit_contract().len(), 3);
//! // The program retains the exact bound implementation it was verified against.
//! assert_eq!(
//!     program.stages().next().expect("one stage").kernel().canonical_identity(),
//!     kernel.canonical_identity(),
//! );
//! # Ok(())
//! # }
//! ```

pub mod abi;
mod builder;
mod error;
mod handles;
mod model;
mod verify;

pub use builder::KernelProgramBuilder;
pub use error::{
    KernelProgramBuildError, KernelProgramDiagnostic, KernelProgramVerificationError,
    ProgramAbiUse, ProgramEntityKind, ProgramLimitKind,
};
pub use handles::{AbiExprId, AllocationId, MaterializedValueId, StageId, ViewId};
pub use model::{
    AllocationOwnership, AllocationRef, AllocationSpec, BitPackedEncoding, ByteWindow,
    CanonicalKernelProgramIdentity, DependencyReasonView, DependencyRef, MaterializedComponentSpec,
    MaterializedOrigin, MaterializedValueRef, MaterializedValueSpec, MemorySpace, PackedBitOrder,
    PackedTailRule, PartialReduction, PartialReductionRef, ProgramOutputRef, RoutingCommitState,
    RoutingCommitTransition, SemanticOccurrence, StageAccess, StageAccessMode, StageAccessRef,
    StageLaunch, StageLaunchView, StageRef, StorageEncoding, StorageScalar, ValueRole,
    VerifiedKernelProgram, ViewRef,
};

/// Maximum stages admitted by one kernel program.
pub const MAX_PROGRAM_STAGES: usize = 4_096;
/// Maximum materialized values admitted by one kernel program.
pub const MAX_PROGRAM_VALUES: usize = 65_536;
/// Maximum byte views admitted by one kernel program.
pub const MAX_PROGRAM_VIEWS: usize = 65_536;
/// Maximum storage allocations admitted by one kernel program.
pub const MAX_PROGRAM_ALLOCATIONS: usize = 65_536;
/// Maximum typed dependency edges admitted by one kernel program.
pub const MAX_PROGRAM_DEPENDENCIES: usize = 262_144;
/// Maximum split-reduction contracts admitted by one kernel program.
///
/// One contract per stage pair at most, so the stage ceiling bounds it.
pub const MAX_PROGRAM_PARTIAL_REDUCTIONS: usize = 4_096;
/// Maximum named outputs admitted by one kernel program.
pub const MAX_PROGRAM_OUTPUTS: usize = 4_096;
/// Maximum accesses admitted by one program stage.
pub const MAX_STAGE_ACCESSES: usize = 64;
/// Maximum semantic occurrences one stage may cover.
pub const MAX_STAGE_COVERAGE: usize = 65_536;
/// Maximum ABI expression arena nodes admitted by one kernel program.
pub const MAX_PROGRAM_ABI_EXPRESSIONS: usize = 4_096;
/// Maximum size of the final canonical kernel-program identity.
pub const MAX_PROGRAM_IDENTITY_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
mod tests;
