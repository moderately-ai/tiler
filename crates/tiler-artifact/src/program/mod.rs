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
//! Every one of those subjects is a compilation **input**, including each
//! payload descriptor: its digest identifies the payload's compilation subject
//! — source, flags, resolved toolchain, entry mappings, obligations — and not
//! the emitted object. The artifact identity is therefore derivable *before*
//! the backend compiler runs, which is what an expansion cache needs on a miss,
//! and
//! [`ArtifactProgramBuilder::push_pending_payload`](crate::program::ArtifactProgramBuilder::push_pending_payload)
//! is the constructor that reaches it without an object. The pre-compilation
//! and post-compilation values are the same bytes rather than two subjects kept
//! in agreement; see
//! [`CanonicalArtifactProgramIdentity`](crate::program::CanonicalArtifactProgramIdentity)
//! for what that does and does not prove.
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
//! # What this layer declares that the program already states
//!
//! ADR 0068 and ADR 0070 place `AbiExpr` in `tiler_ir::program`, and ADR 0072
//! says complete program identity covers buffers, ABI, guards, and routing.
//! `complete-program-identity-with-abi-guards-and-routing` moved the entry ABI,
//! the applicability guard, and the routing-commit lifecycle down: a
//! [`tiler_ir::program::VerifiedKernelProgram`] now carries its own expression
//! arena, guard, per-stage launch, and per-access accessible range, and folds
//! each into `tiler.kernel-program.v2` identity.
//!
//! This crate still declares its own [`VariantSpec`](crate::program::VariantSpec)
//! ABI on its own arena,
//! under the separately versioned `guard_and_routing` schema, and validates it
//! against the same program facts. The two are not yet bound to each other:
//! nothing checks that a variant's accessible-byte *expression* is the one the
//! program states, only that both agree with the program's declared shapes. The
//! ticket `bind-the-artifact-variant-abi-to-the-program-abi` owns closing that,
//! and the reason it is separate is that the artifact layer additionally owns
//! launch preconditions, deferred predicates, and a portfolio's variant
//! priority — none of which a single target-neutral program can carry.
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
//!     MaterializedValueSpec, MemorySpace, RoutingCommitState, RoutingCommitTransition,
//!     SemanticOccurrence, StageAccess, StageAccessMode, StageLaunch, ValueRole,
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
//! # let read_bytes = plan.push_abi_root(AbiRoot::UnsignedLiteral(24))?;
//! # let write_bytes = plan.push_abi_root(AbiRoot::UnsignedLiteral(8))?;
//! # let grid_threads = plan.push_abi_root(AbiRoot::UnsignedLiteral(2))?;
//! # let threads_per_workgroup = plan.push_abi_root(AbiRoot::UnsignedLiteral(1))?;
//! # let program_guard = plan.push_abi_root(AbiRoot::BooleanLiteral(true))?;
//! # plan.applicability_guard(program_guard)?;
//! # plan.push_stage(
//! #     &kernel,
//! #     &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
//! #     &[
//! #         StageAccess { view: read, mode: StageAccessMode::Read, accessible_bytes: read_bytes },
//! #         StageAccess { view: write, mode: StageAccessMode::Write, accessible_bytes: write_bytes },
//! #     ],
//! #     StageLaunch { grid_threads, threads_per_workgroup },
//! # )?;
//! # plan.push_output(OutputKey::new("result")?, result)?;
//! # for (from, to, fallback_permitted) in [
//! #     (RoutingCommitState::Preflight, RoutingCommitState::Committed, true),
//! #     (RoutingCommitState::Committed, RoutingCommitState::Executing, false),
//! #     (RoutingCommitState::Executing, RoutingCommitState::Published, false),
//! # ] {
//! #     plan.push_routing_commit_transition(
//! #         RoutingCommitTransition { from, to, fallback_permitted },
//! #     )?;
//! # }
//! # let program = plan.build()?;
//! // Package that verified program as a one-variant artifact portfolio.
//! let provider = ProviderIdentity::new("tiler", "fused-serial-sum", 1)?;
//! let environment = CompilationEnvironment::new([provider.clone()])?;
//! let mut artifact = ArtifactProgramBuilder::new(&semantic, environment)?;
//! artifact.select_provider(SelectedProvider {
//!     provider,
//!     capability: CapabilityKey::new("tiler.capability.fused-serial-sum")?,
//!     capability_revision: 1,
//! })?;
//! let payload = artifact.push_payload(BackendPayloadDescriptor {
//!     backend: BackendKey::new("tiler.metal")?,
//!     representation: RepresentationKey::new("metallib")?,
//!     payload_schema: SchemaVersion::new(1, 0),
//!     digest: PayloadDigest::from_bytes([0xa1, 0xb2, 0xc3])?,
//!     // The payload's own compatibility contract, not the plan's: this names
//!     // what these bytes were built against, which a shared payload could not
//!     // otherwise state.
//!     compatibility: TargetProfileRef {
//!         key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//!         descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//!     },
//!     execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation,
//! })?;
//!
//! // The ABI's accessible ranges are formulas over the bound interface, not
//! // constants: `rows * columns * 4` input bytes and `rows * 4` output bytes.
//! // Both slots address their value from its first byte, so both offsets are
//! // the literal zero this plan's byte windows actually state.
//! let key = InputKey::new("input")?;
//!
//! artifact.push_variant(
//!     &program,
//!     VariantSpec {
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
//!                 BindingSpec { kind: BindingKind::Buffer },
//!                 BindingSpec { kind: BindingKind::Buffer },
//!             ],
//!             launch: LaunchSpec {
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
// Deliberately not re-exported. The delivered-realization record is a reviewed
// draft staged under ADR 0074 convention 7: its constructor and its reader are
// both public artifact surface that ADR 0075 reserves to Tom, and the module
// documentation records what is staged and what the wiring slice still owes.
mod realization;
mod verify;

pub use builder::{
    ArtifactProgramBuilder, BindingSpec, CompilationEnvironment, DeferredPredicateSpec, EntrySpec,
    LaunchSpec, VariantSpec,
};
// Re-exported because this module's own public accessors return it.
// `DecodedBinding::access` hands back a `BufferAccess`, and until now a consumer
// could only name that type by taking its own `tiler-ir` dependency — which
// `tiler-runtime` deliberately does not have, its closure being a decided
// property under ADR 0081. A public method whose return type its callers cannot
// spell is unusable, so the type travels with the accessor that produces it.
pub use tiler_ir::kernel::BufferAccess;

pub use codec::{
    ArtifactCodecFailure, DecodedArtifact, DecodedBinding, DecodedDeferredPredicate, DecodedEntry,
    DecodedExpr, DecodedInput, DecodedNumerical, DecodedOutput, DecodedStageDependency,
    DecodedVariant, PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadProvenance,
    PayloadSdkIdentity, PayloadTargetObligation, SectionPurpose, SectionView, ToolComponent,
    decode_artifact,
};
// The governed digest algorithm, which `docs/artifact-abi.md` requires every
// digest use to name explicitly rather than choose locally.
//
// Promoted for `tiler-cache` on Tom's decision of 2026-07-25
// (`decide-the-expansion-cache-owner-and-digest-authority`). The expansion
// cache validates a stored bundle's section digests on every hit (ADR 0050),
// and the alternative — a hash function local to that crate — would make it a
// second identity authority over the same subject. The promotion is
// deliberately the algorithm and the opaque digest alone: `digest_parts` and
// [`envelope_digest`] stay crate-private, so an outside caller can digest a
// subject under its own domain and cannot construct an envelope association.
pub use codec::{DIGEST_BYTES, Digest, DigestAlgorithm};
// [`envelope_digest`] *is* the proof sidecar's association with an envelope, and
// nothing outside this crate has a use for it. Named re-exports rather than a
// crate-visible `mod codec`, so the codec's working vocabulary stays confined to
// this module.
pub(crate) use codec::envelope_digest;
// The envelope's three governed digest domains, reachable only under test.
// `crate::proof::tests` checks the no-domain-prefixes-another property over the
// *union* of both containers' domains rather than per container, because the
// property is global: one algorithm hashes both, so a domain added to either
// one could silently merge two subjects across the boundary.
#[cfg(test)]
pub(crate) use codec::{ENVELOPE_DIGEST_DOMAIN, MANIFEST_DIGEST_DOMAIN, SECTION_DIGEST_DOMAIN};
pub use error::{
    AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind, ArtifactKeyKind,
    ArtifactLimitKind, ArtifactVerificationError,
};
pub use expr::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue,
    AvailabilityPhase, MAX_TARGET_PROPERTY_KEY_BYTES, TargetPropertyKey, TargetPropertyKeyError,
};
pub use facts::{
    AbiBindingError, AbiFactBinder, MAX_BOUND_INPUT_EXTENTS, MAX_BOUND_TARGET_PROPERTIES,
};
pub use handles::{AbiExprId, PayloadId, VariantId};
pub use keys::{
    BackendEntryKey, BackendKey, CapabilityKey, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
    MAX_GOVERNED_KEY_BYTES, MAX_OPAQUE_IDENTITY_BYTES, PayloadDigest, RepresentationKey,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
pub use model::{
    AbiExprRef, AbiExprView, ArtifactExecutionPolicy, ArtifactInputRef, ArtifactOutputRef,
    ArtifactSchema, BackendEntryRef, BackendPayloadDescriptor, BindingKind, BindingRef,
    BindingTarget, CanonicalArtifactProgramIdentity, DeferredPredicateRef, EntryRef, RoutingPolicy,
    SchemaVersion, SelectedProvider, StageDependencyReason, VariantRef, VerifiedArtifactProgram,
};

/// Maximum plan variants admitted by one artifact program.
pub const MAX_ARTIFACT_VARIANTS: usize = 64;
/// Maximum executable entries admitted by one plan variant.
pub const MAX_VARIANT_ENTRIES: usize = 4_096;
/// Maximum stage dependency edges admitted by one plan variant.
///
/// A dependency graph over `n` entries admits at most `n * (n - 1) / 2` distinct
/// edges per reason, which for [`MAX_VARIANT_ENTRIES`] is far beyond anything a
/// plan produces. The bound is stated as a budget a parser can allocate against
/// before reading, not as a claim about plan shape: a decoder must refuse a
/// hostile count before it allocates for it.
pub const MAX_STAGE_DEPENDENCIES: usize = 65_536;
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

// Crate-visible under `cfg(test)` so `crate::proof`'s tests can package the
// same real verified artifact these fixtures build, instead of growing a second
// copy of a 400-line semantic-and-kernel fixture that could drift from this one.
#[cfg(test)]
pub(crate) mod tests;
