//! The bounded, versioned, target-neutral artifact program model.
//!
//! A [`VerifiedArtifactProgram`](crate::program::VerifiedArtifactProgram) is the *packaged* form of a compilation: the
//! portfolio of complete plan variants a runtime may route among, each bound to
//! its verified [`tiler_ir::program::VerifiedKernelProgram`], plus everything a
//! runtime needs that the program layer deliberately does not model — the
//! neutral ABI, launch and guard expressions, declared target requirements,
//! reached provenance, and backend payload descriptors.
//!
//! The artifact model is a reviewed **draft** boundary (ADR 0074 §7). The narrower codec capability — encoding a verified artifact and decoding bytes into a validated read view — is accepted; its envelope, row, encoder, decoder, identity-derivation, and wire-layout machinery remains private.
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
//! contracts, the declared target requirements, deferred predicates, and
//! live-device route requirements, the
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
//! `complete-program-identity-with-abi-guards-and-routing` moved the entry ABI, the applicability guard, and the routing-commit lifecycle down: a [`tiler_ir::program::VerifiedKernelProgram`] carries its own expression arena, guard, per-stage launch, and per-access accessible byte range. Historical v2 first folded those facts; later encoding, ABI-completeness, split-reduction, canonical-coverage, published-output-order, proof-bound-coverage, publishing-copy, and staged-realization changes moved the current domain to `tiler.kernel-program.v11`.
//!
//! Artifact construction now replays that exact program ABI onto the artifact arena and derives the guard, launch geometry, accessible byte offset and extent, binding target, component role, storage scalar and encoding, kernel access type, access mode, address space, and alignment from the verified program. [`VariantSpec`](crate::program::VariantSpec) supplies only artifact-owned facts: target and feasibility references, typed [`PreparedEntryTargetRequirement`](crate::program::PreparedEntryTargetRequirement) values associated with program-entry ordinals, binding transport kinds, launch preconditions and zero-work policy, and backend entry selection. The builder mints each executable deferred predicate from the whole checked requirement, so an assembler cannot reverse its comparison or replace the requirement's exact-entry query with a global property observation. The low-level builder validates the producer assertion and entry range but cannot authenticate that an arbitrary caller preserved the compiler's requirement-to-entry association; the ordinary `tiler-build` translation provides that stronger guarantee by forwarding the compiler's borrowed view without reconstruction. This is one ABI authority with an explicit producer trust boundary, not two ABIs kept in agreement; the artifact layer still owns portfolio priority and the predicates no single target-neutral program can carry.
//!
//! A variant's live-device route requirements arrive through [`ArtifactProgramBuilder::require_route`](crate::program::ArtifactProgramBuilder::require_route) rather than through `VariantSpec`, because they state what the *emitted payload* consumes and are known only after backend emission — to a different producer stage from the one that assembles a variant. See [`RouteRequirement`](crate::program::RouteRequirement) for the derivability test that decides what may be declared there and what stays a derived requirement.
//!
//! The walk-through below packages a real verified kernel program, and its
//! hidden preamble assembles one. Stage coverage is proof-derived — each record
//! needs a completed [`tiler_ir::index::IndexRefinementReceipt`], which only the
//! refinement verifier mints — so the preamble derives the occurrence's subject,
//! builds a candidate index region, and submits the pair through
//! `tiler_ir::index`. It reaches the verifier by that door rather than by
//! compiling the graph: a `tiler-compiler` dev-dependency would make the
//! preamble four lines, but `tiler-runtime`'s
//! `the_consumer_links_no_compiler_emitter_or_build_provider` refuses it,
//! because `Cargo.lock` merges normal and development edges per package and
//! ADR 0081 item 2 fixes the consumer closure at `[tiler-artifact]`. The graph is
//! one elementwise operation for the same reason `tiler_ir::program`'s own
//! walk-through is: the receipt path is per occurrence, and this module's
//! subject is what happens *after* a program is verified.
//!
//! ```
//! use tiler_artifact::program::{
//!     ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey, BackendEntryRef,
//!     BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec, CapabilityKey,
//!     CompilationEnvironment, DeliveredRealizationBuilder, DimensionBehaviour, DispositionView,
//!     EntrySpec, FactSourceProvenance, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
//!     HonouringMeans, LaunchSpec, NumericalDimension, NumericalObligationKey, PayloadDigest,
//!     PolicyLocus, ProvenanceIdentity, RepresentationKey, ScalarArithmeticSubject, SchemaVersion,
//!     SelectedProvider, SemanticOccurrence, TargetEvidenceDeclaration,
//!     TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, VariantSpec,
//! };
//! use tiler_ir::schedule::{
//!     ApproximationEnvelope, ExceptionalValueAssumption, MaterializationRounding,
//!     NumericalPermission, SubnormalMode,
//! };
//! use tiler_ir::semantic::{InputKey, OutputKey, ProviderIdentity};
//! # use tiler_ir::index::{
//! #     DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry,
//! #     IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
//! #     IndexRegionBuilder, ScalarAttributes, TensorRole as IndexTensorRole, multiply_f32_scalar_op,
//! # };
//! # use tiler_ir::kernel::{KernelType, lower_scheduled_region};
//! # use tiler_ir::program::abi::AbiRoot;
//! # use tiler_ir::program::{
//! #     AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
//! #     CoveredOccurrence, KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec,
//! #     MemorySpace, RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode,
//! #     StageLaunch, StorageEncoding, StorageScalar, ValueRole,
//! # };
//! # use tiler_ir::schedule::{
//! #     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExecutionBinding,
//! #     F32NumericalContractKey, InputOrdinal, KernelSchedule, LaunchPlan, LogicalAccess,
//! #     NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
//! #     PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, ScalarProgram,
//! #     ScheduledRegionBuilder, TailPolicy, TensorRole,
//! # };
//! # use tiler_ir::semantic::{F32, F32Multiply, SemanticProgramBuilder};
//! # use tiler_ir::shape::{Extent, Shape};
//! #
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # // One elementwise product over a 2x3 pair: `tiler_ir::program`'s own
//! # // documented assembly, verbatim.
//! # let mut draft = SemanticProgramBuilder::try_standard()?;
//! # let left = draft.input::<F32>(InputKey::new("left")?, Shape::from_dims([2, 3]))?;
//! # let right = draft.input::<F32>(InputKey::new("right")?, Shape::from_dims([2, 3]))?;
//! # let product = F32Multiply::apply(&mut draft, left, right)?;
//! # draft.output(OutputKey::new("result")?, product)?;
//! # let semantic = draft.build()?;
//! # let contract = F32NumericalContractKey::new(
//! #     SubnormalMode::Preserve,
//! #     SubnormalMode::Preserve,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     ApproximationEnvelope::Forbidden,
//! #     ExceptionalValueAssumption::MakeNoAssumption,
//! #     ExceptionalValueAssumption::MakeNoAssumption,
//! #     MaterializationRounding::NearestTiesToEven,
//! # )?
//! # .into();
//! # let scalars = FrozenScalarRegistry::standard()?;
//! # let laws = FrozenIndexRealizationLawRegistry::from_semantic(
//! #     semantic.semantic_registry().clone(),
//! #     scalars.clone(),
//! # )?;
//! # let operation = semantic.operations().next().expect("one operation").id();
//! # let subject = IndexRefinementSubject::derive(&semantic, operation, contract)?;
//! # let mut region = IndexRegionBuilder::new(scalars.clone())?;
//! # let rows = region.dimension(DomainRole::Parallel, Extent::new(2))?;
//! # let columns = region.dimension(DomainRole::Parallel, Extent::new(3))?;
//! # let point = [rows, columns];
//! # let coordinate = [region.dimension_expr(rows)?, region.dimension_expr(columns)?];
//! # let mut operands = Vec::new();
//! # for boundary in subject.inputs() {
//! #     operands.push(region.tensor(
//! #         IndexTensorRole::Input,
//! #         boundary.value_type().clone(),
//! #         boundary.shape().clone(),
//! #     )?);
//! # }
//! # let mut reads = Vec::new();
//! # for position in subject.operands() {
//! #     reads.push(region.read(operands[*position], &point, &coordinate)?);
//! # }
//! # let value = region
//! #     .apply(multiply_f32_scalar_op(), ScalarAttributes::empty(), &reads)?
//! #     .get(0)
//! #     .expect("one product");
//! # let destination = region.tensor(
//! #     IndexTensorRole::Output,
//! #     subject.results()[0].value_type().clone(),
//! #     subject.results()[0].shape().clone(),
//! # )?;
//! # let write = region.write(destination, &point, &coordinate)?;
//! # region.output(write, value)?;
//! # let region = region.build()?;
//! # let authority = IndexRealizationAuthority::admit(
//! #     semantic.semantic_registry(),
//! #     &scalars,
//! #     subject.operation().clone(),
//! #     subject.signature().clone(),
//! #     &[multiply_f32_scalar_op()],
//! # )?;
//! # let coverage: Vec<CoveredOccurrence> =
//! #     match laws.resolve(&subject)?.verify(&authority, &region)? {
//! #         IndexRefinementVerificationOutcome::Verified(receipt) => {
//! #             vec![CoveredOccurrence::from_receipt(&receipt)]
//! #         }
//! #         IndexRefinementVerificationOutcome::Pending(_) => {
//! #             panic!("a static elementwise region retains no residual obligation")
//! #         }
//! #     };
//! # let mut schedule = ScheduledRegionBuilder::new(RegionId::new(0));
//! # schedule.iteration_shape(Shape::from_dims([2, 3]))?;
//! # for ordinal in [0, 1] {
//! #     schedule.push_access(Access {
//! #         tensor: TensorRole::Input { ordinal: InputOrdinal::new(ordinal) },
//! #         component_role: None,
//! #         mode: AccessMode::Read,
//! #         map: LogicalAccess::LinearIdentity,
//! #         bounds: BoundsWitnessId::new(ordinal),
//! #         ownership: None,
//! #     })?;
//! #     schedule.push_bounds_proof(BoundsProof {
//! #         id: BoundsWitnessId::new(ordinal),
//! #         tensor: TensorRole::Input { ordinal: InputOrdinal::new(ordinal) },
//! #         component_role: None,
//! #         kind: BoundsProofKind::LinearRange { element_count: 6 },
//! #     })?;
//! # }
//! # schedule.push_access(Access {
//! #     tensor: TensorRole::Output,
//! #     component_role: None,
//! #     mode: AccessMode::Write,
//! #     map: LogicalAccess::LinearIdentity,
//! #     bounds: BoundsWitnessId::new(2),
//! #     ownership: Some(OwnershipWitnessId::new(0)),
//! # })?;
//! # schedule.push_bounds_proof(BoundsProof {
//! #     id: BoundsWitnessId::new(2),
//! #     tensor: TensorRole::Output,
//! #     component_role: None,
//! #     kind: BoundsProofKind::LinearRange { element_count: 6 },
//! # })?;
//! # schedule.ownership_proof(OwnershipProof {
//! #     id: OwnershipWitnessId::new(0),
//! #     tensor: TensorRole::Output,
//! #     kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 },
//! # })?;
//! # let mut expression = PointwiseF32ExpressionBuilder::new();
//! # let first = expression.input(InputOrdinal::new(0))?;
//! # let second = expression.input(InputOrdinal::new(1))?;
//! # let root = expression.multiply(first, second)?;
//! # schedule.scalar_program(ScalarProgram::PointwiseF32(expression.build(root)?))?;
//! # schedule.numerical(NumericalRealization::new(
//! #     "tiler.doc.strict-f32",
//! #     0x7fc0_0000,
//! #     SubnormalMode::Preserve,
//! #     SubnormalMode::Preserve,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     ExceptionalValueAssumption::MakeNoAssumption,
//! #     ExceptionalValueAssumption::MakeNoAssumption,
//! # ))?;
//! # schedule.schedule(KernelSchedule {
//! #     binding: ExecutionBinding::GlobalLinearInvocation,
//! #     work_items: 6,
//! #     threads_per_workgroup: 1,
//! #     tail: TailPolicy::Exact,
//! #     output_owner: OwnershipWitnessId::new(0),
//! #     reduction: ReductionTopology::None,
//! #     launch: LaunchPlan { grid_threads: 6, threads_per_workgroup: 1, zero_work_skips_dispatch: true },
//! # })?;
//! # let kernel = lower_scheduled_region(&schedule.build()?)?;
//! # let mut plan = KernelProgramBuilder::new(&semantic)?;
//! # let mut bound = Vec::new();
//! # for key in ["left", "right"] {
//! #     let allocation = plan.push_allocation(AllocationSpec {
//! #         capacity_bytes: 24,
//! #         alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
//! #         memory_space: MemorySpace::Device,
//! #         ownership: AllocationOwnership::External,
//! #     })?;
//! #     let value = plan.push_value(
//! #         MaterializedValueSpec {
//! #             origin: MaterializedOrigin::ProgramInput { key: InputKey::new(key)? },
//! #             role: ValueRole::Input,
//! #             shape: Shape::from_dims([2, 3]),
//! #             storage_scalar: StorageScalar::F32,
//! #             element_type: KernelType::F32,
//! #             encoding: StorageEncoding::Unpacked,
//! #             alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
//! #             memory_space: MemorySpace::Device,
//! #         },
//! #         allocation,
//! #     )?;
//! #     bound.push(plan.push_whole_view(value)?);
//! # }
//! # let owned = plan.push_allocation(AllocationSpec {
//! #     capacity_bytes: 24,
//! #     alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
//! #     memory_space: MemorySpace::Device,
//! #     ownership: AllocationOwnership::Program,
//! # })?;
//! # let result = plan.push_value(
//! #     MaterializedValueSpec {
//! #         origin: MaterializedOrigin::Internal,
//! #         role: ValueRole::Output,
//! #         shape: Shape::from_dims([2, 3]),
//! #         storage_scalar: StorageScalar::F32,
//! #         element_type: KernelType::F32,
//! #         encoding: StorageEncoding::Unpacked,
//! #         alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
//! #         memory_space: MemorySpace::Device,
//! #     },
//! #     owned,
//! # )?;
//! # let result_view = plan.push_whole_view(result)?;
//! # let bytes = plan.push_abi_root(AbiRoot::UnsignedLiteral(24))?;
//! # let grid_threads = plan.push_abi_root(AbiRoot::UnsignedLiteral(6))?;
//! # let threads_per_workgroup = plan.push_abi_root(AbiRoot::UnsignedLiteral(1))?;
//! # let program_guard = plan.push_abi_root(AbiRoot::BooleanLiteral(true))?;
//! # plan.applicability_guard(program_guard)?;
//! # plan.push_stage(
//! #     &kernel,
//! #     &coverage,
//! #     &[
//! #         StageAccess { view: bound[0], mode: StageAccessMode::Read, accessible_bytes: bytes },
//! #         StageAccess { view: bound[1], mode: StageAccessMode::Read, accessible_bytes: bytes },
//! #         StageAccess { view: result_view, mode: StageAccessMode::Write, accessible_bytes: bytes },
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
//! let provider = ProviderIdentity::new("tiler", "elementwise-multiply", 1)?;
//! let environment = CompilationEnvironment::new([provider.clone()])?;
//! let mut artifact = ArtifactProgramBuilder::new(&semantic, environment)?;
//! artifact.select_provider(SelectedProvider {
//!     provider,
//!     capability: CapabilityKey::new("tiler.capability.elementwise-multiply")?,
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
//!     execution_policy: ArtifactExecutionPolicy::NativeImage,
//! })?;
//!
//! // The variant declares only artifact-owned facts. Every ABI binding below —
//! // its target, accessible offset and extent, storage scalar, access mode, and
//! // alignment — is replayed from the verified program rather than restated
//! // here, which is why one buffer slot per program access is all this spells.
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
//!                 BindingSpec { kind: BindingKind::Buffer },
//!             ],
//!             launch: LaunchSpec {
//!                 zero_work_skips_dispatch: true,
//!                 preconditions: Vec::new(),
//!             },
//!             // One delivery position, because this artifact is built for one
//!             // consumer target. A second position would name a second
//!             // compiled object realizing this same entry.
//!             implementation: BackendEntryRef {
//!                 payloads: vec![payload],
//!                 entry_key: BackendEntryKey::from_bytes(b"multiply")?,
//!             },
//!         }],
//!     },
//! )?;
//!
//! // Every executable artifact carries the numerical realization it delivered,
//! // and the record is built through the typed producer path rather than from
//! // opaque means bytes: the governed `f32` policy subject, its eleven resolved
//! // behaviours in canonical dimension order, and — for the one dimension this
//! // packaged route consumes — the locus that requires it, the behaviour that
//! // locus requires, and the structured evidence honouring it.
//! //
//! // The strict contract this program schedules forbids contraction, so
//! // `SupportedExactly` is the honest means: this target honours the
//! // requirement as it stands rather than under a declared relaxation. A
//! // producer that cannot say which it is has no business writing either.
//! let subject = ScalarArithmeticSubject::f32().identity();
//! let mut realization = DeliveredRealizationBuilder::new(TargetProfileRef {
//!     key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//!     descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//! });
//! realization.declare_scalar_arithmetic(subject.clone(), [
//!     DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
//!     DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!     DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden),
//!     DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
//!     DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
//!     DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven),
//! ])?;
//! realization.require(
//!     &subject,
//!     NumericalDimension::Contraction,
//!     NumericalObligationKey::new(SemanticOccurrence::new(0), PolicyLocus::Computation),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!     TargetEvidenceDeclaration {
//!         declared: DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//!         means: HonouringMeans::SupportedExactly,
//!         profile: TargetProfileRef {
//!             key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//!             descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//!         },
//!         source: FactSourceProvenance::governed(
//!             ProvenanceIdentity::new("tiler.prototype-target-neutral-baseline.v1", 1),
//!             ProvenanceIdentity::new("tiler.guarantee.strict-f32", 1),
//!         ),
//!     },
//! )?;
//! // The one packaged entry, at flat declared ordinal 0: one variant, one stage.
//! realization.bind_entry(0, &subject)?;
//! artifact.declare_realization(realization.build()?)?;
//!
//! let artifact = artifact.build()?;
//!
//! // A consumer reads the delivered means from the artifact rather than
//! // inferring it from the request or the target's name.
//! let delivered = artifact
//!     .delivered_realization()
//!     .scalar_arithmetic(&subject)
//!     .expect("the packaged f32 contract");
//! assert_eq!(
//!     delivered.resolution(NumericalDimension::Contraction),
//!     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! );
//! match delivered.assessment(NumericalDimension::Contraction) {
//!     DispositionView::Required(obligations) => assert_eq!(
//!         delivered.evidence_for(&obligations[0]).means(),
//!         &HonouringMeans::SupportedExactly,
//!     ),
//!     DispositionView::NotRequired => panic!("this route requires contraction to be forbidden"),
//! }
//! // A dimension no packaged route consumes carries no fabricated target fact.
//! assert_eq!(
//!     delivered.assessment(NumericalDimension::ReciprocalTransform),
//!     DispositionView::NotRequired,
//! );
//!
//! assert_eq!(artifact.variants().len(), 1);
//! assert_eq!(artifact.delivery_positions(), 1);
//! assert_eq!(artifact.selected_providers().len(), 1);
//! // The published interface is the semantic subject's, in its declared order.
//! let keys: Vec<String> = artifact
//!     .inputs()
//!     .map(|input| input.key().as_str().to_owned())
//!     .collect();
//! assert_eq!(keys, ["left", "right"]);
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
mod realization;
mod requirement;
#[cfg(test)]
mod tag_injectivity;
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
// Re-exported for the reason [`BufferAccess`] is: `BindingRef::alignment` and
// `DecodedBinding::alignment` hand back an `AlignmentRequirement`, and
// `tiler-runtime` must be able to name that type without taking a direct
// `tiler-ir` dependency (ADR 0081).
pub use tiler_ir::program::abi::{
    PreparedEntryTargetRequirement, PreparedEntryTargetRequirementError,
    TargetPropertyProviderIdentity, TargetPropertyQuery, TargetPropertyQueryError,
    TargetPropertyRequirementRelation,
};
pub use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, ByteAlignment, ByteAlignmentError,
};
// Re-exported for the reason [`BufferAccess`] is: a delivered-realization
// obligation is keyed by one, `NumericalObligationKey::occurrence` hands one
// back, and `NumericalObligationKey::new` takes one — so a consumer whose
// dependency closure is fixed at `[tiler-artifact]` under ADR 0081 could neither
// read nor state a locus without it.
pub use tiler_ir::program::SemanticOccurrence;

pub use codec::{
    ArtifactCodecFailure, DecodedArtifact, DecodedBinding, DecodedComponent,
    DecodedDeferredPredicate, DecodedEntry, DecodedExpr, DecodedInput, DecodedNumerical,
    DecodedOutput, DecodedStageDependency, DecodedVariant, PayloadContent, PayloadEntryMapping,
    PayloadMetadata, PayloadPlatform, PayloadProvenance, PayloadSdkIdentity,
    PayloadTargetObligation, SectionPurpose, SectionView, ToolComponent, decode_artifact,
};
// The governed digest algorithm, which `docs/artifact-abi.md` requires every
// digest use to name explicitly rather than choose locally.
//
// Promoted for `tiler-cache` on Tom's decision of 2026-07-25
// (`decide-the-expansion-cache-owner-and-digest-authority`). The expansion
// cache validates a stored bundle's section digests on every hit (ADR 0050),
// and the alternative — a hash function local to that crate — would make it a
// second identity authority over the same subject.
//
// This crate owned the algorithm until ADR 0104, which needed it in `tiler-ir`
// — the crate every other member depends on, so the *consumer* could not be
// moved the way ADR 0082 moved the cache. Tom decided on 2026-08-06 that the
// governed digest is its own bottom crate, `tiler-digest`, and this re-export is
// what keeps `tiler_artifact::program::{DIGEST_BYTES, Digest, DigestAlgorithm}`
// resolving for every consumer that already used it. The surface stays
// deliberately narrow: the algorithm and the opaque digest, with the general
// parts-digest this crate carried gone rather than promoted across the boundary
// and [`envelope_digest`] crate-private here, so an outside caller can digest a
// subject under an explicit domain — plain or qualified — and cannot express
// the ambiguous concatenation the general form put on its caller, nor construct
// an envelope association.
pub use codec::{DIGEST_BYTES, Digest, DigestAlgorithm};
// [`envelope_digest`] *is* the proof sidecar's association with an envelope, and
// nothing outside this crate has a use for it. Named re-exports rather than a
// crate-visible `mod codec`, so the codec's working vocabulary stays confined to
// this module.
pub(crate) use codec::envelope_digest;
// The envelope's seven governed domains and this module's seven identity
// domains. Thirteen are crate-visible only under `cfg(test)`: these seven,
// the artifact-identity separator and four key domains re-exported from `model`,
// and `ROUTE_REQUIREMENT_DOMAIN`.
// `DELIVERED_REALIZATION_DOMAIN` is the exception — it is `pub`, re-exported
// below beside the record whose canonical bytes it opens, so its value is
// observable to a consumer. Observable is not accepted: it carries this
// module's reviewed-draft boundary status (see the crate header, ADR 0075)
// until Tom accepts the exact surface. `crate::domains` checks the
// no-domain-prefixes-another property over the *union* of every domain the crate
// admits rather than per container, because the property is global: one
// algorithm hashes them all in one process, so a domain added anywhere could
// silently merge two subjects across a boundary.
#[cfg(test)]
pub(crate) use codec::{
    ENVELOPE_DIGEST_DOMAIN, IDENTITY_DIGEST_DOMAIN, MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN,
    PAYLOAD_IDENTITY_DOMAIN, PAYLOAD_METADATA_DOMAIN, SECTION_DIGEST_DOMAIN,
};
pub use error::{
    AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind, ArtifactKeyKind,
    ArtifactLimitKind, ArtifactVerificationError, ProvenanceField, RecordedArtifactIdentityError,
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
    MAX_GOVERNED_KEY_BYTES, MAX_OPAQUE_IDENTITY_BYTES, MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
    PayloadDigest, RepresentationKey, RouteFeatureKey, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef,
};
#[cfg(test)]
pub(crate) use model::{
    ARTIFACT_DOMAIN, DEFERRED_KEY_DOMAIN, PAYLOAD_KEY_DOMAIN, PROVIDER_KEY_DOMAIN, STAGE_KEY_DOMAIN,
};
pub use model::{
    AbiExprRef, AbiExprView, ArtifactExecutionPolicy, ArtifactInputRef, ArtifactOutputRef,
    ArtifactSchema, BackendEntryRef, BackendPayloadDescriptor, BindingKind, BindingRef,
    BindingTarget, CanonicalArtifactProgramIdentity, DeferredPredicateRef, EntryRef,
    InterfaceComponentRef, RecordedArtifactProgramIdentity, RoutingPolicy, SchemaVersion,
    SelectedProvider, StageDependencyReason, VariantRef, VerifiedArtifactProgram,
};
pub use realization::codec::{
    ArtifactCrossCheck, OrderedSubject as RealizationOrderedSubject, RealizationCodecError,
    ReferenceSubject as RealizationReferenceSubject, TagSubject as RealizationTagSubject,
    decode as decode_realization, validate_against_artifact,
};
pub use realization::{
    AssessmentDisposition, DELIVERED_REALIZATION_DOMAIN, DeliveredRealizationBuilder,
    DeliveredRealizationError, DeliveredRealizationRecord, DispositionView, EntryPolicyBinding,
    EntryRealization, LATEST_DELIVERED_PHASE, NumericalObligation, NumericalPolicySubject,
    RecordFamily, ScalarArithmeticRecord, ScalarArithmeticView, TargetEvidence,
    TargetEvidenceDeclaration, overlapping_behaviour,
};
#[cfg(test)]
pub(crate) use requirement::ROUTE_REQUIREMENT_DOMAIN;
pub use requirement::{
    BackendFeatureRequirement, MAX_ROUTE_FEATURE_PAYLOAD_BYTES, RouteRequirement,
    RouteRequirementError, RouteRequirementSubject, RouteResourceDimension,
    RouteResourceRequirement,
};
// The one shared scalar-arithmetic policy vocabulary, named by re-export rather
// than restated. `tiler-compiler` names the same types the same way, so the
// dimension set, the behaviour spaces, the means, the locus, and the structured
// provenance exist once in the workspace and a widened one is a build error at
// every total encoder rather than a silent divergence between two copies. It
// travels with the accessors that produce and consume it for the reason
// [`BufferAccess`] does: a public method whose types its callers cannot spell is
// unusable, and `tiler-runtime`'s dependency closure is fixed at
// `[tiler-artifact]` under ADR 0081.
//
// Reattached 2026-08-08. This block sat above `pub use requirement::{…}` from
// `8bfcd432`, the commit that wrote it, so it never annotated the item it
// describes: the route-requirement types name no dimension set, behaviour
// space, means, or locus. Nothing was removed from the requirement re-export —
// it never carried a rationale of its own.
pub use tiler_ir::numerics::{
    BehaviourSpace, CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole,
    DIMENSION_COUNT, DimensionBehaviour, ExecutionEnvironmentIdentity, FactAuthority,
    FactEvidenceBasis, FactSourceProvenance, FactValidityScope, HonouringMeans, MeasurementContext,
    NumericalDimension, NumericalObligationKey, PolicyLocus, ProvenanceIdentity,
    RelaxationRequirement, ScalarArithmeticSubject, ScalarArithmeticSubjectError,
    ScalarArithmeticSubjectIdentity,
};
// [`DimensionBehaviour`]'s own payloads, and the arithmetic type a subject
// identity names. Re-exported for the reason [`BufferAccess`] is, and now
// unavoidably so: every artifact carries a delivered-realization record, so a
// consumer that reads one matches `DimensionBehaviour` and reaches these, and a
// consumer that produces one constructs them. A record whose behaviours its
// callers cannot spell would be unusable from a closure ADR 0081 item 2 fixes at
// `[tiler-artifact]`.
//
// [`DecodedNumerical`]'s accessors already returned three of them —
// `SubnormalMode`, `NumericalPermission`, and `ExceptionalValueAssumption` —
// which is the same gap one layer down; it is closed here rather than left for
// the next reader to rediscover. They are named rather than counted because a
// bare number is what went wrong here, and a name can be checked against the
// accessor list without re-deriving one.
//
// Corrected 2026-08-08. That sentence read "already returned four of them" from
// `002b1d63`, the commit that wrote it, and was wrong there too: at that commit
// `DecodedNumerical` carried the same ten accessors it carries now, returning
// those three re-exported types plus `&str` and `u32`, which are not among
// them. No reading of it yields four. The retired wording is quoted so the
// correction is legible, which also keeps it greppable — a later hit on
// "returned four of them" finds this note, and proves the string is present,
// not that the claim stands.
pub use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode, ValueDomainProvenance,
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
/// Maximum delivery positions admitted by one artifact program.
///
/// Equal to [`MAX_ARTIFACT_PAYLOADS`] rather than chosen separately, because a
/// position is a distinct backend object: no payload may realize entries at two
/// positions, so an artifact can never have more positions than it has payloads
/// and a larger bound would admit a count no artifact could satisfy. It is
/// stated anyway rather than left implicit, because a decoder must refuse a
/// hostile count before it allocates against it.
pub const MAX_DELIVERY_POSITIONS: usize = MAX_ARTIFACT_PAYLOADS;
/// Maximum selected capability providers admitted by one artifact program.
pub const MAX_SELECTED_PROVIDERS: usize = 256;
/// Maximum available providers admitted by one compilation environment.
pub const MAX_ENVIRONMENT_PROVIDERS: usize = 4_096;
/// Maximum deferred feasibility predicates admitted by one plan variant.
pub const MAX_DEFERRED_PREDICATES: usize = 64;
/// Maximum live-device route requirements admitted by one plan variant.
///
/// Subjects are distinct within a variant, so this bounds how many *different*
/// device preconditions one route may state. It is a budget a parser can
/// allocate against before reading rather than a claim about plan shape.
pub const MAX_ROUTE_REQUIREMENTS: usize = 64;
/// Maximum launch preconditions admitted by one executable entry.
pub const MAX_LAUNCH_PRECONDITIONS: usize = 32;
/// Maximum size of the final canonical artifact-program identity.
pub const MAX_ARTIFACT_IDENTITY_BYTES: usize = 64 * 1024 * 1024;

// Crate-visible under `cfg(test)` so `crate::proof`'s tests can package the
// same real verified artifact these fixtures build, instead of growing a second
// copy of a 400-line semantic-and-kernel fixture that could drift from this one.
#[cfg(test)]
pub(crate) mod tests;
