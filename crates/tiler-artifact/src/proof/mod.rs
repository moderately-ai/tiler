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
//! Every bound below is checked before any allocation proportional to it, in
//! both directions: the encoder refuses to write a container a reader would
//! not admit, and the reader refuses a declared count before reserving for it.
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
//! `ignore`d for the reason [`crate::program`]'s own walk-through is: the
//! hidden artifact assembly needs proof-derived stage coverage, which no
//! documentation example can produce.
//!
//! ```ignore
//! use tiler_artifact::proof::{
//!     ProofCaseKey, ProofCaseSpec, ProofNumericalIdentity, ProofProvenance,
//!     ProofReferenceIdentity, ProofSidecarBuilder, decode_proof_sidecar,
//! };
//! use tiler_ir::semantic::{InputKey, OutputKey};
//! # use tiler_artifact::program::{
//! #     AbiBinaryOp, AbiRoot, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
//! #     BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec,
//! #     CapabilityKey, CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey,
//! #     FeasibilityRuleSetRef, LaunchSpec, PayloadDigest, RepresentationKey, SchemaVersion,
//! #     SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
//! #     VariantSpec,
//! # };
//! # use tiler_ir::kernel::{KernelType, lower_scheduled_region};
//! # use tiler_ir::program::{
//! #     AllocationOwnership, AllocationSpec, CoveredOccurrence, KernelProgramBuilder,
//! #     MaterializedOrigin, MaterializedValueSpec, MemorySpace, RoutingCommitState,
//! #     RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch, StorageEncoding,
//! #     StorageScalar, ValueRole,
//! # };
//! # use tiler_ir::schedule::{
//! #     Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
//! #     ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
//! #     NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
//! #     OwnershipWitnessId, RegionId, ReductionTopology, ScalarProgram, ScheduledRegionBuilder,
//! #     InputOrdinal, SubnormalMode, TailPolicy, TensorRole,
//! # };
//! # use tiler_ir::semantic::{
//! #     F32, F32Add, F32Constant, F32Multiply, ProviderIdentity, SemanticProgramBuilder,
//! #     StrictSerialF32Sum,
//! # };
//! # use tiler_ir::shape::{Axis, Shape};
//! #
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # // One fused serial-sum plan over a 2x3 input, packaged as a one-variant
//! # // artifact: `crate::program`'s own documented assembly, verbatim.
//! # let mut draft = SemanticProgramBuilder::try_standard()?;
//! # let input = draft.input::<F32>(InputKey::new("input")?, Shape::from_dims([2, 3]))?;
//! # let scale = F32Constant::apply(&mut draft, 2.0_f32.to_bits())?;
//! # let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits())?;
//! # let product = F32Multiply::apply(&mut draft, input, scale)?;
//! # let mapped = F32Add::apply(&mut draft, product, bias)?;
//! # let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)])?;
//! # draft.output(OutputKey::new("result")?, sum)?;
//! # let semantic = draft.build()?;
//! # let coverage: Vec<CoveredOccurrence> = proof_derived_coverage(&semantic);
//! # let axes = vec![Axis::new(1)];
//! # let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
//! # region.iteration_shape(Shape::from_dims([2]))?;
//! # region.push_access(Access {
//! #     tensor: TensorRole::Input { ordinal: InputOrdinal::FIRST },
//! #     component_role: None,
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
//! #     component_role: None,
//! #     mode: AccessMode::Write,
//! #     map: LogicalAccess::LinearIdentity,
//! #     bounds: BoundsWitnessId::new(1),
//! #     ownership: Some(OwnershipWitnessId::new(0)),
//! # })?;
//! # region.push_bounds_proof(BoundsProof {
//! #     id: BoundsWitnessId::new(0),
//! #     tensor: TensorRole::Input { ordinal: InputOrdinal::FIRST },
//! #     component_role: None,
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
//! #     component_role: None,
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
//! #     NumericalPermission::Forbidden,
//! #     NumericalPermission::Forbidden,
//! #     ExceptionalValueAssumption::MakeNoAssumption,
//! #     ExceptionalValueAssumption::MakeNoAssumption,
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
//! #         storage_scalar: StorageScalar::F32,
//! #         element_type: KernelType::F32,
//! #         encoding: StorageEncoding::Unpacked,
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
//! #         storage_scalar: StorageScalar::F32,
//! #         element_type: KernelType::F32,
//! #         encoding: StorageEncoding::Unpacked,
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
//! #     &coverage,
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
//! # let provider = ProviderIdentity::new("tiler", "fused-serial-sum", 1)?;
//! # let environment = CompilationEnvironment::new([provider.clone()])?;
//! # let mut builder = ArtifactProgramBuilder::new(&semantic, environment)?;
//! # builder.select_provider(SelectedProvider {
//! #     provider,
//! #     capability: CapabilityKey::new("tiler.capability.fused-serial-sum")?,
//! #     capability_revision: 1,
//! # })?;
//! # let profile = TargetProfileRef {
//! #     key: TargetProfileKey::new("tiler.prototype-target-neutral-baseline.v1")?,
//! #     descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])?,
//! # };
//! # let payload = builder.push_payload(BackendPayloadDescriptor {
//! #     backend: BackendKey::new("tiler.metal")?,
//! #     representation: RepresentationKey::new("metallib")?,
//! #     payload_schema: SchemaVersion::new(1, 0),
//! #     digest: PayloadDigest::from_bytes([0xa1, 0xb2, 0xc3])?,
//! #     compatibility: profile.clone(),
//! #     execution_policy: ArtifactExecutionPolicy::NativeImage,
//! # })?;
//! # let key = InputKey::new("input")?;
//! # builder.push_variant(
//! #     &program,
//! #     VariantSpec {
//! #         target_profile: profile,
//! #         feasibility_rules: FeasibilityRuleSetRef {
//! #             key: FeasibilityRuleSetKey::new("tiler.feasibility.baseline")?,
//! #             revision: 1,
//! #         },
//! #         deferred_predicates: Vec::new(),
//! #         entries: vec![EntrySpec {
//! #             bindings: vec![
//! #                 BindingSpec { kind: BindingKind::Buffer },
//! #                 BindingSpec { kind: BindingKind::Buffer },
//! #             ],
//! #             launch: LaunchSpec {
//! #                 zero_work_skips_dispatch: true,
//! #                 preconditions: Vec::new(),
//! #             },
//! #             implementation: BackendEntryRef {
//! #                 payloads: vec![payload],
//! #                 entry_key: BackendEntryKey::from_bytes(b"fused")?,
//! #             },
//! #         }],
//! #     },
//! # )?;
//! # let artifact = builder.build()?;
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
//!     inputs: vec![(InputKey::new("input")?, vec![0x7f; 6 * 4])],
//!     expected: vec![(OutputKey::new("result")?, vec![0x80; 2 * 4])],
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
//!     [0x80_u8; 8].as_slice(),
//! );
//!
//! // The stronger check is available to a consumer that holds the program it
//! // compiled — here, the producer validating its own output. It proves the
//! // same association and re-proves the interface obligations locally.
//! decoded.bind_to_artifact(&artifact)?;
//! # Ok(())
//! # }
//! ```

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
/// Maximum bytes of one case payload — one input or one expected output.
pub const MAX_PROOF_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
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
