//! The bounded, versioned, target-neutral artifact program model.
//!
//! A [`VerifiedArtifactProgram`](crate::program::VerifiedArtifactProgram) is the *packaged* form of a compilation: the
//! portfolio of complete plan variants a runtime may route among, each bound to
//! its verified [`tiler_ir::program::VerifiedKernelProgram`], plus everything a
//! runtime needs that the program layer deliberately does not model — the
//! neutral ABI, launch and guard expressions, declared target requirements,
//! reached provenance, and backend payload descriptors.
//!
//! Every public item here is a reviewed **draft** boundary (ADR 0074 §7).
//!
//! # Consumable without optimizer internals
//!
//! This module depends on `tiler-ir` and nothing else. A runtime binds inputs
//! through [`VerifiedArtifactProgram::inputs`](crate::program::VerifiedArtifactProgram::inputs),
//! evaluates a variant's
//! [`VariantRef::applicability_guard`](crate::program::VariantRef::applicability_guard),
//! reads each [`EntryRef`](crate::program::EntryRef)'s ABI bindings
//! and launch contract, resolves the backend entry through
//! [`EntryRef::payload`](crate::program::EntryRef::payload), and walks the plan's
//! stages, values, views,
//! allocations, and dependencies through the shared IR's own read views. At no
//! point does it need a region cover, a fusion alternative, a cost, a search
//! state, or any other compiler-owned object.
//!
//! # Identity, and the ADR 0072 line
//!
//! [`CanonicalArtifactProgramIdentity`](crate::program::CanonicalArtifactProgramIdentity)
//! folds the artifact's own subjects on
//! top of each variant's complete program identity: the governed component
//! schemas, the semantic graph and the *reached* definition and admission
//! provenance, the routing policy and every guard, the neutral ABI and launch
//! contracts, the declared target requirements and deferred predicates, the
//! payload descriptors and their entry mappings, and the capability providers
//! the plan actually selected.
//!
//! It excludes the compilation environment's unused remainder in the strongest
//! available way: those providers are never retained. A
//! [`CompilationEnvironment`](crate::program::CompilationEnvironment) is a
//! construction-time authority used to prove
//! that a selected provider was really offered, and
//! [`tiler_ir::semantic::SemanticIdentity`]'s registry-snapshot subject — which
//! *does* change when an unused provider changes — is deliberately not folded
//! in. A provider that was available and never used cannot invalidate an
//! otherwise identical artifact; a provider that was reached always can.
//!
//! # A live contract divergence this module works inside
//!
//! ADR 0068 and ADR 0070 place `AbiExpr` in `tiler_ir::program`, and ADR 0072
//! says complete program identity covers buffers, ABI, guards, and routing.
//! `tiler_ir::program` as merged covers none of ABI, guards, or routing:
//! `prototype-kernel-program-ir` scoped them here. That divergence is real, is
//! owned by the ticket `complete-program-identity-with-abi-guards-and-routing`,
//! and is not resolved here. The expression domain is written so it can move
//! wholesale; its own module documentation states exactly which half would move
//! and which stays.
//!
//! ```
//! use tiler_artifact::program::{
//!     AbiBinaryOp, AbiRoot, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
//!     BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec,
//!     CapabilityKey, CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey,
//!     FeasibilityRuleSetRef, LaunchSpec, PayloadDigest, RepresentationKey, SchemaVersion,
//!     SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
//!     VariantSpec,
//! };
//! use tiler_ir::kernel::{KernelType, lower_scheduled_region};
//! use tiler_ir::program::{
//!     AllocationOwnership, AllocationSpec, KernelProgramBuilder, MaterializedOrigin,
//!     MaterializedValueSpec, MemorySpace, SemanticOccurrence, StageAccess, StageAccessMode,
//!     ValueRole,
//! };
//! use tiler_ir::schedule::{
//!     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
//!     ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission,
//!     NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, RegionId,
//!     ReductionTopology, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
//!     TensorRole,
//! };
//! use tiler_ir::semantic::{
//!     F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ProviderIdentity,
//!     SemanticProgramBuilder, StrictSerialF32Sum,
//! };
//! use tiler_ir::shape::{Axis, Shape};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # // One fused serial-sum plan over a 2x3 input: the shared IR fixture.
//! # let mut draft = SemanticProgramBuilder::try_standard()?;
//! # let input = draft.input::<F32>(InputKey::new("input")?, Shape::from_dims([2, 3]))?;
//! # let scale = F32Constant::apply(&mut draft, 2.0_f32.to_bits())?;
//! # let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits())?;
//! # let product = F32Multiply::apply(&mut draft, input, scale)?;
//! # let mapped = F32Add::apply(&mut draft, product, bias)?;
//! # let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)])?;
//! # draft.output(OutputKey::new("result")?, sum)?;
//! # let semantic = draft.build()?;
//! # let axes = vec![Axis::new(1)];
//! # let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
//! # region.iteration_shape(Shape::from_dims([2]))?;
//! # region.push_access(Access {
//! #     tensor: TensorRole::Input,
//! #     mode: AccessMode::Read,
//! #     map: LogicalAccess::ReductionContributor {
//! #         input_shape: Shape::from_dims([2, 3]),
//! #         output_shape: Shape::from_dims([2]),
//! #         axes: axes.clone(),
//! #         order: ContributorOrder::OriginalAxisLexicographic,
//! #     },
//! #     bounds: BoundsWitnessId::new(0),
//! #     ownership: None,
//! # })?;
//! # region.push_access(Access {
//! #     tensor: TensorRole::Output,
//! #     mode: AccessMode::Write,
//! #     map: LogicalAccess::LinearIdentity,
//! #     bounds: BoundsWitnessId::new(1),
//! #     ownership: Some(OwnershipWitnessId::new(0)),
//! # })?;
//! # region.push_bounds_proof(BoundsProof {
//! #     id: BoundsWitnessId::new(0),
//! #     tensor: TensorRole::Input,
//! #     kind: BoundsProofKind::ReductionDomain {
//! #         input_shape: Shape::from_dims([2, 3]),
//! #         output_shape: Shape::from_dims([2]),
//! #         axes: axes.clone(),
//! #         order: ContributorOrder::OriginalAxisLexicographic,
//! #     },
//! # })?;
//! # region.push_bounds_proof(BoundsProof {
//! #     id: BoundsWitnessId::new(1),
//! #     tensor: TensorRole::Output,
//! #     kind: BoundsProofKind::LinearRange { element_count: 2 },
//! # })?;
//! # region.ownership_proof(OwnershipProof {
//! #     id: OwnershipWitnessId::new(0),
//! #     tensor: TensorRole::Output,
//! #     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
//! # })?;
//! # region.scalar_program(ScalarProgram::FusedMultiplyAddSerialSum {
//! #     scale_bits: 2.0_f32.to_bits(),
//! #     bias_bits: 1.0_f32.to_bits(),
//! #     axes: axes.clone(),
//! #     order: ContributorOrder::OriginalAxisLexicographic,
//! #     canonical_nan_bits: 0x7fc0_0000,
//! #     empty_identity_bits: 0.0_f32.to_bits(),
//! #     contraction: false,
//! # })?;
//! # region.numerical(NumericalRealization::new(
//! #     "tiler.doc.strict-f32",
//! #     0x7fc0_0000,
//! #     SubnormalMode::Preserve,
//! #     SubnormalMode::Preserve,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! # ))?;
//! # region.schedule(KernelSchedule {
//! #     binding: ExecutionBinding::GlobalLinearInvocation,
//! #     work_items: 2,
//! #     threads_per_workgroup: 1,
//! #     tail: TailPolicy::Exact,
//! #     output_owner: OwnershipWitnessId::new(0),
//! #     reduction: ReductionTopology::Serial {
//! #         axes,
//! #         order: ContributorOrder::OriginalAxisLexicographic,
//! #         permits_reassociation: false,
//! #         permits_permutation: false,
//! #     },
//! #     launch: LaunchPlan { grid_threads: 2, threads_per_workgroup: 1, zero_work_skips_dispatch: true },
//! # })?;
//! # let kernel = lower_scheduled_region(&region.build()?)?;
//! # let mut plan = KernelProgramBuilder::new(&semantic)?;
//! # let external = plan.push_allocation(AllocationSpec {
//! #     capacity_bytes: 24,
//! #     alignment: 4,
//! #     memory_space: MemorySpace::Device,
//! #     ownership: AllocationOwnership::External,
//! # })?;
//! # let owned = plan.push_allocation(AllocationSpec {
//! #     capacity_bytes: 8,
//! #     alignment: 4,
//! #     memory_space: MemorySpace::Device,
//! #     ownership: AllocationOwnership::Program,
//! # })?;
//! # let source = plan.push_value(
//! #     MaterializedValueSpec {
//! #         origin: MaterializedOrigin::ProgramInput { key: InputKey::new("input")? },
//! #         role: ValueRole::Input,
//! #         shape: Shape::from_dims([2, 3]),
//! #         element_type: KernelType::F32,
//! #         alignment: 4,
//! #         memory_space: MemorySpace::Device,
//! #     },
//! #     external,
//! # )?;
//! # let result = plan.push_value(
//! #     MaterializedValueSpec {
//! #         origin: MaterializedOrigin::Internal,
//! #         role: ValueRole::Output,
//! #         shape: Shape::from_dims([2]),
//! #         element_type: KernelType::F32,
//! #         alignment: 4,
//! #         memory_space: MemorySpace::Device,
//! #     },
//! #     owned,
//! # )?;
//! # let read = plan.push_whole_view(source)?;
//! # let write = plan.push_whole_view(result)?;
//! # plan.push_stage(
//! #     &kernel,
//! #     &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
//! #     &[
//! #         StageAccess { view: read, mode: StageAccessMode::Read },
//! #         StageAccess { view: write, mode: StageAccessMode::Write },
//! #     ],
//! # )?;
//! # plan.push_output(OutputKey::new("result")?, result)?;
//! # let program = plan.build()?;
//! // Package that verified program as a one-variant artifact portfolio.
//! let provider = ProviderIdentity::new("tiler", "fused-serial-sum", 1)?;
//! let environment = CompilationEnvironment::new([provider.clone()])?;
//! let mut artifact = ArtifactProgramBuilder::new(&semantic, environment)?;
//! artifact.select_provider(SelectedProvider {
//!     provider,
//!     capability: CapabilityKey::new("tiler.capability.fused-serial-sum")?,
//!     capability_api_version: 1,
//! })?;
//! let payload = artifact.push_payload(BackendPayloadDescriptor {
//!     backend: BackendKey::new("tiler.metal")?,
//!     representation: RepresentationKey::new("metallib")?,
//!     payload_schema: SchemaVersion::new(1, 0),
//!     digest: PayloadDigest::from_bytes([0xa1, 0xb2, 0xc3])?,
//!     execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation,
//! })?;
//!
//! // The ABI's accessible ranges are formulas over the bound interface, not
//! // constants: `rows * columns * 4` input bytes and `rows * 4` output bytes.
//! let key = InputKey::new("input")?;
//! let rows = artifact.push_root(AbiRoot::InputExtent { key: key.clone(), axis: Axis::new(0) })?;
//! let columns = artifact.push_root(AbiRoot::InputExtent { key, axis: Axis::new(1) })?;
//! let width = artifact.push_root(AbiRoot::UnsignedLiteral(4))?;
//! let elements = artifact.push_binary(AbiBinaryOp::CheckedMultiply, rows, columns)?;
//! let input_bytes = artifact.push_binary(AbiBinaryOp::CheckedMultiply, elements, width)?;
//! let output_bytes = artifact.push_binary(AbiBinaryOp::CheckedMultiply, rows, width)?;
//! let one = artifact.push_root(AbiRoot::UnsignedLiteral(1))?;
//! let always = artifact.push_root(AbiRoot::BooleanLiteral(true))?;
//!
//! artifact.push_variant(
//!     &program,
//!     VariantSpec {
//!         applicability_guard: always,
//!         target_profile: TargetProfileRef {
//!             key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//!             descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//!         },
//!         feasibility_rules: FeasibilityRuleSetRef {
//!             key: FeasibilityRuleSetKey::new("tiler.feasibility.baseline")?,
//!             revision: 1,
//!         },
//!         deferred_predicates: Vec::new(),
//!         entries: vec![EntrySpec {
//!             bindings: vec![
//!                 BindingSpec { kind: BindingKind::Buffer, accessible_bytes: input_bytes },
//!                 BindingSpec { kind: BindingKind::Buffer, accessible_bytes: output_bytes },
//!             ],
//!             launch: LaunchSpec {
//!                 grid_threads: rows,
//!                 threads_per_workgroup: one,
//!                 zero_work_skips_dispatch: true,
//!                 preconditions: Vec::new(),
//!             },
//!             implementation: BackendEntryRef {
//!                 payload,
//!                 entry_key: BackendEntryKey::from_bytes(b"fused")?,
//!             },
//!         }],
//!     },
//! )?;
//! let artifact = artifact.build()?;
//!
//! assert_eq!(artifact.variants().len(), 1);
//! assert_eq!(artifact.selected_providers().len(), 1);
//! assert_eq!(artifact.inputs().next().expect("one input").key().as_str(), "input");
//! // The artifact retains the exact verified program it packages.
//! assert_eq!(
//!     artifact.variants().next().expect("one variant").program().canonical_identity(),
//!     program.canonical_identity(),
//! );
//! # Ok(())
//! # }
//! ```

mod builder;
mod codec;
mod error;
mod expr;
mod facts;
mod handles;
mod keys;
mod model;
mod verify;

pub use builder::{
    ArtifactProgramBuilder, BindingSpec, CompilationEnvironment, DeferredPredicateSpec, EntrySpec,
    LaunchSpec, VariantSpec,
};
pub use error::{
    AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind, ArtifactKeyKind,
    ArtifactLimitKind, ArtifactVerificationError, ForeignEnumSubject,
};
pub use expr::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue,
    AvailabilityPhase,
};
pub use facts::{
    AbiBindingError, AbiFactBinder, MAX_BOUND_INPUT_EXTENTS, MAX_BOUND_TARGET_PROPERTIES,
};
pub use handles::{AbiExprId, PayloadId, VariantId};
pub use keys::{
    BackendEntryKey, BackendKey, CapabilityKey, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
    MAX_GOVERNED_KEY_BYTES, MAX_OPAQUE_IDENTITY_BYTES, PayloadDigest, RepresentationKey,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, TargetPropertyKey,
};
pub use model::{
    AbiExprRef, AbiExprView, ArtifactExecutionPolicy, ArtifactInputRef, ArtifactOutputRef,
    ArtifactSchema, BackendEntryRef, BackendPayloadDescriptor, BindingKind, BindingRef,
    CanonicalArtifactProgramIdentity, DeferredPredicateRef, EntryRef, RoutingPolicy, SchemaVersion,
    SelectedProvider, VariantRef, VerifiedArtifactProgram,
};

/// Maximum plan variants admitted by one artifact program.
pub const MAX_ARTIFACT_VARIANTS: usize = 64;
/// Maximum executable entries admitted by one plan variant.
pub const MAX_VARIANT_ENTRIES: usize = 4_096;
/// Maximum ABI bindings admitted by one executable entry.
pub const MAX_ENTRY_BINDINGS: usize = 64;
/// Maximum nodes admitted by one shared ABI expression arena.
pub const MAX_ABI_EXPRESSIONS: usize = 4_096;
/// Maximum backend payload descriptors admitted by one artifact program.
pub const MAX_ARTIFACT_PAYLOADS: usize = 16;
/// Maximum selected capability providers admitted by one artifact program.
pub const MAX_SELECTED_PROVIDERS: usize = 256;
/// Maximum available providers admitted by one compilation environment.
pub const MAX_ENVIRONMENT_PROVIDERS: usize = 4_096;
/// Maximum deferred feasibility predicates admitted by one plan variant.
pub const MAX_DEFERRED_PREDICATES: usize = 64;
/// Maximum launch preconditions admitted by one executable entry.
pub const MAX_LAUNCH_PRECONDITIONS: usize = 32;
/// Maximum size of the final canonical artifact-program identity.
pub const MAX_ARTIFACT_IDENTITY_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
mod tests;
