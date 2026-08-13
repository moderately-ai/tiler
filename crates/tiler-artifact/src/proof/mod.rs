//! The separate, versioned proof-case evidence sidecar.
//!
//! A producer that compiles an artifact also knows what the artifact is
//! *supposed to compute*, because it can evaluate the same semantic program
//! through the target-independent reference evaluator. This module is the
//! bounded container that carries that knowledge beside an artifact: stable
//! case keys, bit-preserving input bytes, normative expected output bytes, the
//! identities of the three authorities that make the expectation meaningful,
//! content digests over every payload, and an exact association with the one
//! envelope the cases are about.
//!
//! # Public boundary status
//!
//! This is an **accepted facade**, promoted from the crate-private draft form
//! of ADR 0074 convention 7 on Tom's review of 2026-07-25. The surface is the
//! producer's builder, the reader, the case vocabulary, the read views, and the
//! four typed rejection vocabularies with their classification: exactly what a
//! case written by one crate needs in order to be verified by another, because
//! the producer and the runner are different crates by construction.
//!
//! The wire form itself is not public. The framing magic, the four versioned
//! domain separators, the schema versions, the manifest encoder, and the
//! identity deriver stay private, so an out-of-crate caller cannot digest a
//! subject under one of this container's domains or assemble bytes the reader
//! did not derive. [`docs/artifact-abi.md`] records the format normatively;
//! broadening the surface to expose it would mean deciding, first, what an
//! out-of-crate producer of these bytes may claim about them.
//!
//! [`docs/artifact-abi.md`]: https://github.com/moderately-ai/tiler/blob/main/docs/artifact-abi.md
//!
//! # This is not artifact semantics, and the separation is structural
//!
//! `tiler.research.workspace.prototype-crate-layout-and-msrv` records the rule
//! this module implements: "The sidecar is not part of artifact semantics."
//! Nothing in [`crate::program`] references this module, no envelope section
//! carries a proof case, and an artifact decodes, validates, and dispatches
//! with no sidecar present. The dependency runs one way — a sidecar names an
//! artifact, an artifact never names a sidecar — which is what makes proof data
//! deletable without changing what a program means.
//!
//! The separation is also why the two containers share exactly two things and
//! no more: the governed digest algorithm, which `docs/artifact-abi.md`
//! requires every digest use in this crate to name explicitly rather than
//! choose locally, and the envelope digest function, which *is* the
//! association. Framing, schema, vocabulary, limits, and failure classification
//! are this module's own.
//!
//! # What a consumer may conclude, and what it may not
//!
//! A validated sidecar is evidence of **integrity and association**: these are
//! the exact bytes a producer wrote, and they name exactly one artifact.
//!
//! It is not evidence of **authenticity**. Every digest and identity in the
//! container is derived from the container's own content, so a forger that
//! rewrites a case recomputes them all and the result validates. What protects
//! a proof run is that the expected bytes are compared against a device
//! readback: a forged expectation makes a correct device fail the comparison,
//! which is a loud result rather than a silent one. A consumer must therefore
//! treat sidecar payloads as *test data* — the runtime-proof ticket says
//! exactly this — and never as a semantic authority, a fallback value, or an
//! input to routing.
//!
//! # The two association strengths
//!
//! [`DecodedProofSidecar::bind_to_envelope`](crate::proof::DecodedProofSidecar::bind_to_envelope)
//! is available to a consumer that
//! holds only bytes. It re-derives the envelope digest over the exact bytes
//! supplied, decodes them, and compares the re-derived artifact identity with
//! the one the sidecar recorded.
//!
//! [`DecodedProofSidecar::bind_to_artifact`](crate::proof::DecodedProofSidecar::bind_to_artifact)
//! is available to a consumer that
//! holds the verified artifact program. It additionally re-proves every
//! structural obligation the builder proved — that the sidecar binds exactly
//! the artifact's declared inputs and outputs in the artifact's own interface
//! order, that every case's payload length is a whole number of elements of the
//! declared shape, and that all cases agree on each interface entry's length.
//!
//! The weaker check is not a weaker *association*: both prove the same artifact
//! identity, and the artifact identity already folds the ordered named
//! interface. The difference is that the stronger one re-proves the obligations
//! locally instead of inheriting them through an identity comparison, which is
//! what a consumer wants when the sidecar was written by an older producer.
//!
//! # Limits
//!
//! Every bound below is checked with exact arithmetic before any allocation
//! proportional to it, in both directions. The producer projects the encoded
//! identity, manifest, framed-payload stream, and complete sidecar and refuses
//! before cloning a payload, hashing, reserving, or appending. The reader
//! refuses a declared count before reserving for it. A size that is not
//! representable on the host is refused as unrepresentable rather than wrapped
//! or truncated.
//!
//! # A case one crate writes, verified by another
//!
//! This example is the reason the facade is public, so it is also the test of
//! it: a doctest compiles as its own crate, and naming an item that is not
//! `pub` fails to compile rather than failing an assertion. The artifact
//! assembly is hidden because [`crate::program`] documents it; what is shown is
//! the producer writing evidence beside the artifact it just built, and a
//! consumer holding bytes verifying it.
//!
//! That hidden assembly is real rather than a stub: its stage coverage is
//! minted by the refinement verifier, exactly as [`crate::program`]'s own
//! walk-through spells out. A sidecar names an artifact, so there has to be an
//! artifact before there is anything to write evidence beside.
//!
//! ```
//! use tiler_artifact::proof::{
//!     ProofCaseKey, ProofCaseSpec, ProofNumericalIdentity, ProofProvenance,
//!     ProofReferenceIdentity, ProofSidecarBuilder, decode_proof_sidecar,
//! };
//! use tiler_ir::semantic::{InputKey, OutputKey};
//! # use tiler_artifact::program::{
//! #     ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey, BackendEntryRef,
//! #     BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec, CapabilityKey,
//! #     CompilationEnvironment, DeliveredRealizationBuilder, DimensionBehaviour, EntrySpec,
//! #     FactSourceProvenance, FeasibilityRuleSetKey, FeasibilityRuleSetRef, HonouringMeans,
//! #     LaunchSpec, NumericalDimension, NumericalObligationKey, PayloadDigest, PolicyLocus,
//! #     ProvenanceIdentity, RepresentationKey, ScalarArithmeticSubject, SchemaVersion,
//! #     SelectedProvider, SemanticOccurrence, TargetEvidenceDeclaration,
//! #     TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, VariantSpec,
//! # };
//! # use tiler_ir::semantic::ProviderIdentity;
//! # use tiler_ir::index::{
//! #     DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry,
//! #     IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
//! #     IndexRegionBuilder, ScalarAttributes, TensorRole as IndexTensorRole, multiply_f32_scalar_op,
//! # };
//! # use tiler_ir::kernel::{KernelType, lower_scheduled_region};
//! # use tiler_ir::program::abi::AbiRoot;
//! # use tiler_ir::program::{
//! #     AllocationOwnership, AllocationSpec, CoveredOccurrence, KernelProgramBuilder,
//! #     MaterializedOrigin, MaterializedValueSpec, MemorySpace, RoutingCommitState,
//! #     RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch, StorageEncoding,
//! #     StorageScalar, ValueRole,
//! # };
//! # use tiler_ir::schedule::{
//! #     Access, AccessMode, ApproximationEnvelope, BoundsProof, BoundsProofKind, BoundsWitnessId,
//! #     ExceptionalValueAssumption, ExecutionBinding, F32NumericalContractKey, InputOrdinal,
//! #     KernelSchedule, LaunchPlan, LogicalAccess, MaterializationRounding, NumericalPermission,
//! #     NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
//! #     PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, ScalarProgram,
//! #     ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
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
//! #         alignment: 4,
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
//! #             alignment: 4,
//! #             memory_space: MemorySpace::Device,
//! #         },
//! #         allocation,
//! #     )?;
//! #     bound.push(plan.push_whole_view(value)?);
//! # }
//! # let owned = plan.push_allocation(AllocationSpec {
//! #     capacity_bytes: 24,
//! #     alignment: 4,
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
//! #         alignment: 4,
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
//! # // Package that verified program as a one-variant artifact portfolio.
//! # let provider = ProviderIdentity::new("tiler", "elementwise-multiply", 1)?;
//! # let environment = CompilationEnvironment::new([provider.clone()])?;
//! # let mut artifact = ArtifactProgramBuilder::new(&semantic, environment)?;
//! # artifact.select_provider(SelectedProvider {
//! #     provider,
//! #     capability: CapabilityKey::new("tiler.capability.elementwise-multiply")?,
//! #     capability_revision: 1,
//! # })?;
//! # let payload = artifact.push_payload(BackendPayloadDescriptor {
//! #     backend: BackendKey::new("tiler.metal")?,
//! #     representation: RepresentationKey::new("metallib")?,
//! #     payload_schema: SchemaVersion::new(1, 0),
//! #     digest: PayloadDigest::from_bytes([0xa1, 0xb2, 0xc3])?,
//! #     // The payload's own compatibility contract, not the plan's: this names
//! #     // what these bytes were built against, which a shared payload could not
//! #     // otherwise state.
//! #     compatibility: TargetProfileRef {
//! #         key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//! #         descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//! #     },
//! #     execution_policy: ArtifactExecutionPolicy::NativeImage,
//! # })?;
//! #
//! # // The variant declares only artifact-owned facts. Every ABI binding below —
//! # // its target, accessible offset and extent, storage scalar, access mode, and
//! # // alignment — is replayed from the verified program rather than restated
//! # // here, which is why one buffer slot per program access is all this spells.
//! # artifact.push_variant(
//! #     &program,
//! #     VariantSpec {
//! #         target_profile: TargetProfileRef {
//! #             key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//! #             descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//! #         },
//! #         feasibility_rules: FeasibilityRuleSetRef {
//! #             key: FeasibilityRuleSetKey::new("tiler.feasibility.baseline")?,
//! #             revision: 1,
//! #         },
//! #         deferred_predicates: Vec::new(),
//! #         entries: vec![EntrySpec {
//! #             bindings: vec![
//! #                 BindingSpec { kind: BindingKind::Buffer },
//! #                 BindingSpec { kind: BindingKind::Buffer },
//! #                 BindingSpec { kind: BindingKind::Buffer },
//! #             ],
//! #             launch: LaunchSpec {
//! #                 zero_work_skips_dispatch: true,
//! #                 preconditions: Vec::new(),
//! #             },
//! #             // One delivery position, because this artifact is built for one
//! #             // consumer target. A second position would name a second
//! #             // compiled object realizing this same entry.
//! #             implementation: BackendEntryRef {
//! #                 payloads: vec![payload],
//! #                 entry_key: BackendEntryKey::from_bytes(b"multiply")?,
//! #             },
//! #         }],
//! #     },
//! # )?;
//! # // Every executable artifact carries the numerical realization it
//! # // delivered; `crate::program`'s walk-through is where that record is
//! # // built through the typed producer path and read back.
//! # let subject = ScalarArithmeticSubject::f32().identity();
//! # let profile = TargetProfileRef {
//! #     key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//! #     descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//! # };
//! # let mut realization = DeliveredRealizationBuilder::new(profile.clone());
//! # realization.declare_scalar_arithmetic(subject.clone(), [
//! #     DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
//! #     DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
//! #     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #     DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden),
//! #     DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
//! #     DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
//! #     DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven),
//! # ])?;
//! # realization.require(
//! #     &subject,
//! #     NumericalDimension::Contraction,
//! #     NumericalObligationKey::new(SemanticOccurrence::new(0), PolicyLocus::Computation),
//! #     DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #     TargetEvidenceDeclaration {
//! #         declared: DimensionBehaviour::Transform(NumericalPermission::Forbidden),
//! #         means: HonouringMeans::SupportedExactly,
//! #         profile: profile.clone(),
//! #         source: FactSourceProvenance::governed(
//! #             ProvenanceIdentity::new("tiler.prototype-target-neutral-baseline.v1", 1),
//! #             ProvenanceIdentity::new("tiler.guarantee.strict-f32", 1),
//! #         ),
//! #     },
//! # )?;
//! # realization.bind_entry(0, &subject)?;
//! # artifact.declare_realization(realization.build()?)?;
//! # let artifact = artifact.build()?;
//! // The producer holds the artifact and the graph it reference-evaluated. The
//! // association is not supplied: the builder encodes the artifact itself and
//! // digests those exact bytes.
//! let mut draft = ProofSidecarBuilder::new(
//!     &artifact,
//!     ProofProvenance {
//!         semantic_graph: semantic.semantic_identity().graph().clone(),
//!         numerical: ProofNumericalIdentity::from_bytes(b"tiler.doc.strict-f32")?,
//!         reference: ProofReferenceIdentity::from_bytes(b"tiler.doc.reference-evaluator.v1")?,
//!     },
//! )?;
//! draft.push_case(ProofCaseSpec {
//!     key: ProofCaseKey::new("canonical-nan")?,
//!     inputs: vec![
//!         (InputKey::new("left")?, vec![0x7f; 6 * 4]),
//!         (InputKey::new("right")?, vec![0x00; 6 * 4]),
//!     ],
//!     expected: vec![(OutputKey::new("result")?, vec![0x80; 6 * 4])],
//! })?;
//! let sidecar = draft.build()?;
//! let sidecar_bytes = sidecar.encode()?;
//! let envelope_bytes = artifact.encode()?;
//!
//! // The consumer holds bytes and takes nothing on the producer's word: the
//! // reader re-derives the identity, and the association is re-derived from the
//! // envelope bytes the caller itself supplied.
//! let decoded = decode_proof_sidecar(&sidecar_bytes)?;
//! assert_eq!(decoded.identity(), sidecar.canonical_identity());
//! assert_eq!(decoded.re_encode()?, sidecar_bytes);
//! decoded.bind_to_envelope(&envelope_bytes)?;
//!
//! // Payloads are bit-preserving, which is what makes a bitwise readback
//! // comparison downstream mean anything.
//! let case = decoded
//!     .case(&ProofCaseKey::new("canonical-nan")?)
//!     .expect("the case is present");
//! assert_eq!(
//!     case.expected().next().expect("one expectation").bytes(),
//!     [0x80_u8; 24].as_slice(),
//! );
//!
//! // The stronger check is available to a consumer that holds the program it
//! // compiled — here, the producer validating its own output. It proves the
//! // same association and re-proves the interface obligations locally.
//! decoded.bind_to_artifact(&artifact)?;
//! # Ok(())
//! # }
//! ```

mod budget;
mod builder;
mod codec;
mod model;

pub use builder::{
    ProofBuildError, ProofCaseSpec, ProofDirection, ProofInterfaceError, ProofProvenance,
    ProofSidecarBuilder,
};
pub use codec::{
    DecodedProofSidecar, ProofAssociationError, ProofCodecError, ProofFailureClass,
    ProofLimitExceeded, ProofLimitKind, ProofOrderedSubject, decode_proof_sidecar,
};
pub use model::{
    CanonicalProofSidecarIdentity, ProofCaseKey, ProofCaseKeyError, ProofCaseRef,
    ProofNumericalIdentity, ProofPayloadRef, ProofReferenceIdentity, ProofSemanticSubject,
    ProofSubjectError, VerifiedProofSidecar,
};
// This container's four governed domains, reachable under test so
// `crate::domains` can hold the no-prefix check over the crate's whole set. Two
// are digest arguments and two are framing tags opening a canonical byte run;
// `docs/artifact-abi.md` names all four under "The sidecar's four governed
// domains".
#[cfg(test)]
pub(crate) use codec::{
    IDENTITY_DOMAIN, MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN, PAYLOAD_DIGEST_DOMAIN,
};

/// Maximum proof cases admitted by one sidecar.
pub const MAX_PROOF_CASES: usize = 256;
/// Maximum UTF-8 byte length of one stable proof-case key.
pub const MAX_PROOF_CASE_KEY_BYTES: usize = 256;
/// Maximum named interface entries one sidecar binds per direction.
///
/// Deliberately equal to the artifact model's own interface bound: a sidecar
/// binds one payload per declared entry, so a looser bound here would admit a
/// container no artifact could ever associate with.
pub const MAX_PROOF_INTERFACE_ENTRIES: usize = 4_096;
/// Maximum bytes of one received opaque provenance subject.
pub const MAX_PROOF_SUBJECT_BYTES: usize = 1_024;
/// Maximum bytes of the sidecar's canonical manifest.
pub const MAX_PROOF_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
/// Maximum bytes of one complete encoded sidecar.
pub const MAX_PROOF_SIDECAR_BYTES: usize = 256 * 1024 * 1024;
/// Maximum bytes of the derived canonical sidecar identity.
pub const MAX_PROOF_IDENTITY_BYTES: usize = 8 * 1024 * 1024;

#[cfg(test)]
mod tests;
