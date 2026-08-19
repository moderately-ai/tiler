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
//! - **Complete coverage and the proof behind it.** Each stage carries
//!   [`CoveredOccurrence`](crate::program::CoveredOccurrence) records binding
//!   the semantic occurrences it claims to the reached-only executable-coverage
//!   identity of the completed index-refinement receipt that proved each one,
//!   and those occurrences are proven to be a disjoint partition of every
//!   operation of the bound graph.
//! - **Materializations and buffers.** Values, byte views, allocations, typed
//!   dependencies, and the ordered named outputs described below.
//! - **The entry ABI.** Each stage's launch geometry and each access's
//!   addressable byte range, as [`abi`](crate::program::abi) expressions rather than resolved
//!   numbers.
//! - **The applicability guard.** The predicate deciding whether this program
//!   may be routed to at all.
//! - **The routing-commit contract.** The ordered lifecycle from preflight to
//!   publication and, for each step, whether fallback is still permitted.
//!
//! The last three landed with `complete-program-identity-with-abi-guards-and-routing`, which moved the domain from `tiler.kernel-program.v1` to historical v2 because a v1 identity was blind to two programs that differed only in their guard, ABI, or fallback contract. Later encoding and ABI-completeness changes moved the same subject through v3 and v4 to v5, folding the declared split-reduction contracts moved it to v6, canonical semantic stage coverage moved it to v7, folding the published outputs in interface order rather than sorted by content moved it to v8, binding each covered occurrence to its reached-only refinement evidence moved it to v9, folding the declared publishing-copy contracts moved it to v10, and folding the declared staged-realization contracts moved it to v11. Complete canonical stage ownership — including exact split occurrences and nonzero continuation ordinals — moves the current domain to `tiler.kernel-program.v12`; [`CanonicalKernelProgramIdentity`](crate::program::CanonicalKernelProgramIdentity) documents each step.
//!
//! Every transient ordinal is excluded: builder insertion order, the program's
//! own stage/value/view/allocation/arena positions, and the planning `RegionId`
//! already excluded below. Cross-references are encoded by canonical content
//! key, so two structurally equal programs assembled in different orders share
//! identity bytes.
//!
//! # The published output order is the semantic subject's
//!
//! A program's published outputs are its **ordered output interface**, and that
//! order belongs to the semantic program the builder was opened against — not
//! to the producer that published them. Whole-program verification proves it:
//! the published records carry the semantic subject's output keys in the
//! subject's declared order, each key's records contiguous, and within one key
//! the component records follow the encoded contract's own declared component
//! order. Any other publication is refused as
//! [`KernelProgramDiagnostic::MisorderedNamedOutput`](crate::program::KernelProgramDiagnostic::MisorderedNamedOutput).
//!
//! Two consequences follow, and they are why the rule is stated here rather
//! than left to each consumer. First,
//! [`VerifiedKernelProgram::outputs`](crate::program::VerifiedKernelProgram::outputs)
//! is genuinely ordered, so a consumer projecting an ordered interface — an
//! artifact's published contract, a frontend returning several results — reads
//! that order instead of re-deriving it by key against the semantic program.
//! Second, identity folds the output list *in that order* rather than sorting
//! it, which is what the sibling dependency-edge and split-reduction sections
//! do. The asymmetry is deliberate and is the whole distinction: an edge or a
//! split **names** entities and is named by none, so where a producer declared
//! it carries no meaning identity should preserve; an output record **is**
//! named, positionally, by the caller's interface. Sorting it discarded exactly
//! the fact the semantic layer treats as identity — `semantic::identity`
//! encodes the output list in declaration order and seeds canonical value
//! numbering from it — and an artifact layer blind to a permutation the
//! semantic layer distinguishes cannot name what it produced.
//!
//! Note what this does *not* claim. The order carries no bits the identity did
//! not already determine, because the folded
//! [`SemanticGraphIdentity`](crate::semantic::SemanticGraphIdentity) fixes the
//! interface order and verification pins publication to it. It is folded for
//! the reason the routing-commit lifecycle is folded in lifecycle order:
//! identity states what the program is, and completeness of a fact must not
//! rest on a verifier rule staying exactly as strict as it is today.
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
//! lifetimes, an explicit handoff, and no live alias across it), and named
//! outputs that are complete and published in the semantic interface's order.
//!
//! It does **not** re-derive, at this layer, that a stage's kernel computes the
//! semantic operations it covers. That proof is the index-refinement verifier's
//! and arrives already made: each [`CoveredOccurrence`](crate::program::CoveredOccurrence)
//! is minted from a completed [`IndexRefinementReceipt`](crate::index::IndexRefinementReceipt),
//! and this layer proves the receipt belongs to the bound graph, retains its
//! reached-only evidence in identity, and separately proves the coverage is
//! complete and disjoint. What it still does not prove is that the *kernel* the
//! stage dispatches is the lowering of the region that receipt is about; that
//! remains the compiler's obligation.
//!
//! # A coverage record cannot be assembled from parts
//!
//! [`CoveredOccurrence`](crate::program::CoveredOccurrence) has private fields
//! and one constructor, which requires a completed receipt. Pairing an
//! occurrence with unrelated evidence is not a mistake to be caught; it does not
//! typecheck:
//!
//! ```compile_fail
//! use tiler_ir::program::{CoveredOccurrence, SemanticOccurrence};
//!
//! fn forge(
//!     refinement: tiler_ir::index::IndexRefinementExecutableCoverageIdentity,
//! ) -> CoveredOccurrence {
//!     CoveredOccurrence {
//!         graph: todo!(),
//!         occurrence: SemanticOccurrence::new(0),
//!         refinement,
//!     }
//! }
//! ```
//!
//! A proof gap has no spelling either. Only a *completed* receipt exposes the
//! executable-coverage projection, so a pending association cannot be turned
//! into coverage:
//!
//! ```compile_fail
//! use tiler_ir::index::PendingIndexRefinementReceipt;
//! use tiler_ir::program::CoveredOccurrence;
//!
//! fn coverage_from_a_gap(pending: &PendingIndexRefinementReceipt) -> CoveredOccurrence {
//!     CoveredOccurrence::from_receipt(pending)
//! }
//! ```
//!
//! The assembly example below therefore mints its coverage the only way
//! anything can. It builds a *candidate* index region — its own claim about
//! what the occurrence computes — and submits it to the refinement verifier,
//! which mints a receipt only when that candidate's canonical identity equals
//! the registered realization law's. The law's own realization is deliberately
//! not public, so an example cannot ask for the expected answer and hand it
//! straight back; the region below is written out, and it is checked against an
//! authority the example does not influence.
//!
//! That path is per occurrence, which is why the graph here is a single
//! elementwise operation rather than the fused five-operation reduction
//! `crate::program::tests` uses: the three steps would repeat verbatim four more
//! times without demonstrating anything this module owns. The suite covers the
//! multi-stage, multi-occurrence, partitioned-coverage case.
//!
//! ```
//! use tiler_ir::index::{
//!     DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry,
//!     IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
//!     IndexRegionBuilder, ScalarAttributes, TensorRole as IndexTensorRole, multiply_f32_scalar_op,
//! };
//! use tiler_ir::kernel::{KernelType, lower_scheduled_region};
//! use tiler_ir::program::abi::AbiRoot;
//! use tiler_ir::program::{
//!     AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
//!     CoveredOccurrence, KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec,
//!     MemorySpace, RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode,
//!     StageLaunch, StorageEncoding, StorageScalar, ValueRole,
//! };
//! use tiler_ir::schedule::{
//!     Access, AccessMode, ApproximationEnvelope, BoundsProof, BoundsProofKind, BoundsWitnessId,
//!     ExceptionalValueAssumption, ExecutionBinding, F32NumericalContractKey, AccessOrdinal,
//!     KernelSchedule, LaunchPlan, LogicalAccess, MaterializationRounding, NumericalPermission,
//!     NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
//!     PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, RegionProgram,
//!     ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
//! };
//! use tiler_ir::semantic::{F32, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder};
//! use tiler_ir::shape::{Extent, Shape};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // result = left * right, elementwise over a 2x3 pair: one graph operation.
//! let mut draft = SemanticProgramBuilder::try_standard()?;
//! let left = draft.input::<F32>(InputKey::new("left")?, Shape::from_dims([2, 3]))?;
//! let right = draft.input::<F32>(InputKey::new("right")?, Shape::from_dims([2, 3]))?;
//! let product = F32Multiply::apply(&mut draft, left, right)?;
//! draft.output(OutputKey::new("result")?, product)?;
//! let semantic = draft.build()?;
//!
//! // The occurrence's refinement subject, under the strict binary32 contract
//! // the kernel below realizes. The contract reaches the receipt's executable
//! // coverage, so it is part of what the evidence is about rather than a
//! // detail of how it was obtained.
//! let contract = F32NumericalContractKey::new(
//!     SubnormalMode::Preserve,
//!     SubnormalMode::Preserve,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     NumericalPermission::Forbidden,
//!     ApproximationEnvelope::Forbidden,
//!     ExceptionalValueAssumption::MakeNoAssumption,
//!     ExceptionalValueAssumption::MakeNoAssumption,
//!     MaterializationRounding::NearestTiesToEven,
//! )?
//! .into();
//! let scalars = FrozenScalarRegistry::standard()?;
//! let laws = FrozenIndexRealizationLawRegistry::from_semantic(
//!     semantic.semantic_registry().clone(),
//!     scalars.clone(),
//! )?;
//! let operation = semantic.operations().next().expect("one operation").id();
//! let subject = IndexRefinementSubject::derive(&semantic, operation, contract)?;
//!
//! // The candidate region, in canonical logical index form.
//! let mut region = IndexRegionBuilder::new(scalars.clone())?;
//! let rows = region.dimension(DomainRole::Parallel, Extent::new(2))?;
//! let columns = region.dimension(DomainRole::Parallel, Extent::new(3))?;
//! let point = [rows, columns];
//! let coordinate = [region.dimension_expr(rows)?, region.dimension_expr(columns)?];
//! let mut operands = Vec::new();
//! for boundary in subject.inputs() {
//!     operands.push(region.tensor(
//!         IndexTensorRole::Input,
//!         boundary.value_type().clone(),
//!         boundary.shape().clone(),
//!     )?);
//! }
//! let mut reads = Vec::new();
//! for position in subject.operands() {
//!     reads.push(region.read(operands[*position], &point, &coordinate)?);
//! }
//! let value = region
//!     .apply(multiply_f32_scalar_op(), ScalarAttributes::empty(), &reads)?
//!     .get(0)
//!     .expect("one product");
//! let destination = region.tensor(
//!     IndexTensorRole::Output,
//!     subject.results()[0].value_type().clone(),
//!     subject.results()[0].shape().clone(),
//! )?;
//! let write = region.write(destination, &point, &coordinate)?;
//! region.output(write, value)?;
//! let region = region.build()?;
//!
//! // The verifier is the only mint. The admitted authority bounds what scalar
//! // operations the realization may emit, and it is stated here rather than
//! // read off the candidate.
//! let authority = IndexRealizationAuthority::admit(
//!     semantic.semantic_registry(),
//!     &scalars,
//!     subject.operation().clone(),
//!     subject.signature().clone(),
//!     &[multiply_f32_scalar_op()],
//! )?;
//! let coverage: Vec<CoveredOccurrence> =
//!     match laws.resolve(&subject)?.verify(&authority, &region)? {
//!         IndexRefinementVerificationOutcome::Verified(receipt) => {
//!             vec![CoveredOccurrence::from_receipt(&receipt)]
//!         }
//!         IndexRefinementVerificationOutcome::Pending(_) => {
//!             panic!("a static elementwise region retains no residual index-domain obligation")
//!         }
//!     };
//!
//! // The physical schedule, lowered to its verified structured kernel. The
//! // ordered access list is the local coordinate space that lets a consumer
//! // bind buffers positionally.
//! let mut schedule = ScheduledRegionBuilder::new(RegionId::new(0));
//! schedule.iteration_shape(Shape::from_dims([2, 3]))?;
//! for ordinal in [0, 1] {
//!     schedule.push_access(Access {
//!         tensor: TensorRole::Input,
//!         component_role: None,
//!         mode: AccessMode::Read,
//!         map: LogicalAccess::LinearIdentity,
//!         bounds: BoundsWitnessId::new(ordinal),
//!         ownership: None,
//!     })?;
//!     schedule.push_bounds_proof(BoundsProof {
//!         id: BoundsWitnessId::new(ordinal),
//!         tensor: TensorRole::Input,
//!         component_role: None,
//!         kind: BoundsProofKind::LinearRange { element_count: 6 },
//!     })?;
//! }
//! schedule.push_access(Access {
//!     tensor: TensorRole::Output,
//!     component_role: None,
//!     mode: AccessMode::Write,
//!     map: LogicalAccess::LinearIdentity,
//!     bounds: BoundsWitnessId::new(2),
//!     ownership: Some(OwnershipWitnessId::new(0)),
//! })?;
//! schedule.push_bounds_proof(BoundsProof {
//!     id: BoundsWitnessId::new(2),
//!     tensor: TensorRole::Output,
//!     component_role: None,
//!     kind: BoundsProofKind::LinearRange { element_count: 6 },
//! })?;
//! schedule.ownership_proof(OwnershipProof {
//!     id: OwnershipWitnessId::new(0),
//!     tensor: TensorRole::Output,
//!     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 },
//! })?;
//! let mut expression = PointwiseF32ExpressionBuilder::new();
//! let first = expression.input(AccessOrdinal::new(0))?;
//! let second = expression.input(AccessOrdinal::new(1))?;
//! let root = expression.multiply(first, second)?;
//! schedule.program(RegionProgram::Numerical {
//!     scalar: ScalarProgram::PointwiseF32(expression.build(root)?),
//!     numerical: NumericalRealization::new(
//!         "tiler.doc.strict-f32",
//!         0x7fc0_0000,
//!         SubnormalMode::Preserve,
//!         SubnormalMode::Preserve,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         NumericalPermission::Forbidden,
//!         ApproximationEnvelope::Forbidden,
//!         ExceptionalValueAssumption::MakeNoAssumption,
//!         ExceptionalValueAssumption::MakeNoAssumption,
//!     ),
//! })?;
//! schedule.schedule(KernelSchedule {
//!     binding: ExecutionBinding::GlobalLinearInvocation,
//!     work_items: 6,
//!     threads_per_workgroup: 1,
//!     tail: TailPolicy::Exact,
//!     output_owner: OwnershipWitnessId::new(0),
//!     reduction: ReductionTopology::None,
//!     launch: LaunchPlan { grid_threads: 6, threads_per_workgroup: 1, zero_work_skips_dispatch: true },
//! })?;
//! let kernel = lower_scheduled_region(&schedule.build()?)?;
//!
//! // One stage covering every operation of that exact graph.
//! let mut plan = KernelProgramBuilder::new(&semantic)?;
//! let mut bound = Vec::new();
//! for key in ["left", "right"] {
//!     let allocation = plan.push_allocation(AllocationSpec {
//!         capacity_bytes: 24,
//!         alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
//!         memory_space: MemorySpace::Device,
//!         ownership: AllocationOwnership::External,
//!     })?;
//!     let value = plan.push_value(
//!         MaterializedValueSpec {
//!             origin: MaterializedOrigin::ProgramInput { key: InputKey::new(key)? },
//!             role: ValueRole::Input,
//!             shape: Shape::from_dims([2, 3]),
//!             storage_scalar: StorageScalar::F32,
//!             encoding: StorageEncoding::Unpacked,
//!             element_type: KernelType::F32,
//!             alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
//!             memory_space: MemorySpace::Device,
//!         },
//!         allocation,
//!     )?;
//!     bound.push(plan.push_whole_view(value)?);
//! }
//! let owned = plan.push_allocation(AllocationSpec {
//!     capacity_bytes: 24,
//!     alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
//!     memory_space: MemorySpace::Device,
//!     ownership: AllocationOwnership::Program,
//! })?;
//! let result = plan.push_value(
//!     MaterializedValueSpec {
//!         origin: MaterializedOrigin::Internal,
//!         role: ValueRole::Output,
//!         shape: Shape::from_dims([2, 3]),
//!         storage_scalar: StorageScalar::F32,
//!         encoding: StorageEncoding::Unpacked,
//!         element_type: KernelType::F32,
//!         alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
//!         memory_space: MemorySpace::Device,
//!     },
//!     owned,
//! )?;
//! let result_view = plan.push_whole_view(result)?;
//!
//! // The entry ABI: what each access may address, and how the stage launches.
//! // The bounded profile's shapes are static, so each is a literal; a dynamic
//! // subject would name `AbiRoot::InputExtent` here instead, with no change to
//! // the shape of this contract. All three slots address 24 bytes, and the
//! // arena stores that literal once.
//! let bytes = plan.push_abi_root(AbiRoot::UnsignedLiteral(24))?;
//! let grid_threads = plan.push_abi_root(AbiRoot::UnsignedLiteral(6))?;
//! let threads_per_workgroup = plan.push_abi_root(AbiRoot::UnsignedLiteral(1))?;
//! let guard = plan.push_abi_root(AbiRoot::BooleanLiteral(true))?;
//! plan.applicability_guard(guard)?;
//!
//! plan.push_stage(
//!     &kernel,
//!     &coverage,
//!     &[
//!         StageAccess { view: bound[0], mode: StageAccessMode::Read, accessible_bytes: bytes },
//!         StageAccess { view: bound[1], mode: StageAccessMode::Read, accessible_bytes: bytes },
//!         StageAccess { view: result_view, mode: StageAccessMode::Write, accessible_bytes: bytes },
//!     ],
//!     StageLaunch { grid_threads, threads_per_workgroup },
//! )?;
//! plan.push_output(OutputKey::new("result")?, result)?;
//!
//! // Fallback is still legal while nothing is committed, and never after.
//! for (from, to, fallback_permitted) in [
//!     (RoutingCommitState::Preflight, RoutingCommitState::Committed, true),
//!     (RoutingCommitState::Committed, RoutingCommitState::Executing, false),
//!     (RoutingCommitState::Executing, RoutingCommitState::Published, false),
//! ] {
//!     plan.push_routing_commit_transition(
//!         RoutingCommitTransition { from, to, fallback_permitted },
//!     )?;
//! }
//! let program = plan.build()?;
//!
//! assert_eq!(program.stages().len(), 1);
//! assert_eq!(program.execution_order().len(), 1);
//! assert_eq!(program.outputs().len(), 1);
//! // Three distinct unsigned literals — 24, 6, 1 — and the guard predicate.
//! assert_eq!(program.abi_expressions().len(), 4);
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
mod alignment;
mod builder;
mod contraction_witness;
mod error;
mod handles;
mod model;
mod verify;

pub use alignment::{AlignmentGuarantee, AlignmentRequirement, ByteAlignment, ByteAlignmentError};
pub use builder::KernelProgramBuilder;
pub use contraction_witness::{ContractionF32PlanWitness, ContractionF32PlanWitnessError};
pub use error::{
    KernelProgramBuildError, KernelProgramDiagnostic, KernelProgramVerificationError,
    ProgramAbiUse, ProgramEntityKind, ProgramLimitKind,
};
pub use handles::{AbiExprId, AllocationId, MaterializedValueId, StageId, ViewId};
pub use model::{
    AllocationOwnership, AllocationRef, AllocationSpec, BitPackedEncoding, ByteWindow,
    CanonicalKernelProgramIdentity, CoveredOccurrence, DependencyReasonView, DependencyRef,
    MaterializedComponentSpec, MaterializedOrigin, MaterializedValueRef, MaterializedValueSpec,
    MemorySpace, PackedBitOrder, PackedTailRule, PartialReduction, PartialReductionRef,
    ProgramOutputRef, PublishingCopy, PublishingCopyRef, RoutingCommitState,
    RoutingCommitTransition, SemanticOccurrence, StageAccess, StageAccessMode, StageAccessRef,
    StageLaunch, StageLaunchView, StageRef, StagedRealization, StagedRealizationRef,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram, ViewRef,
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
/// Maximum publishing-copy contracts admitted by one kernel program.
///
/// One contract per published value at most, so the output ceiling bounds it.
pub const MAX_PROGRAM_PUBLISHING_COPIES: usize = 4_096;
/// Maximum staged-realization contracts admitted by one kernel program.
///
/// One contract per consuming stage and continued occurrence at most, so the
/// stage ceiling bounds a program whose stages each continue one realization.
pub const MAX_PROGRAM_STAGED_REALIZATIONS: usize = 4_096;
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
