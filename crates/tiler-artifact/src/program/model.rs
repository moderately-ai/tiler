//! Artifact-program data model, read-only views, verified product, and identity.
//!
//! The vocabulary is what a runtime or a codec needs and nothing more: an
//! ordered named interface, a portfolio of complete plan variants each with its
//! applicability guard and declared target requirements, one executable entry
//! per program stage carrying the neutral ABI and launch contracts, backend
//! payload descriptors the entries are realized by, and the provenance the
//! packaged plan actually reached.
//!
//! Every entry keeps its verified [`VerifiedKernelProgram`] so a consumer reads
//! stages, values, views, allocations, and dependencies through the shared IR's
//! own views. Nothing here requires a consumer to link the compiler.
//!
//! Only [`super::ArtifactProgramBuilder::build`] can bind a draft into an
//! opaque [`VerifiedArtifactProgram`].

use tiler_ir::kernel::{AddressSpace, BufferAccess, CanonicalKernelIdentity, KernelType};
use tiler_ir::program::{
    ByteWindow, MaterializedValueRef, StageRef, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    FlushedZeroSign, NumericalPermission, NumericalRealization, ResourceRequirements, SubnormalMode,
};
use tiler_ir::semantic::{
    InputKey, OutputKey, ProviderIdentity, SemanticAdmissionProvenanceIdentity,
    SemanticDefinitionProjectionIdentity, SemanticGraphIdentity, SemanticIdentity,
};
use tiler_ir::shape::Shape;

use tiler_ir::identity::{push_len, push_slice};

use super::MAX_ARTIFACT_IDENTITY_BYTES;
use super::codec::{
    ArtifactEnvelope, EntryRow, NumericalFacts, PayloadContent, VariantRow, expression_keys,
    position as node_at,
};
use super::error::{ArtifactDiagnostic, ArtifactEntityKind, ForeignEnumSubject};
use super::expr::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue,
    AvailabilityPhase, ExprNode, evaluate,
};
use super::handles::PayloadId;
use super::keys::{
    BackendEntryKey, BackendKey, CapabilityKey, FeasibilityRuleSetRef, PayloadDigest,
    RepresentationKey, TargetProfileRef,
};

const ARTIFACT_DOMAIN: &[u8] = b"tiler.artifact-program.v1\0";
const STAGE_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.stage.v1\0";
const PAYLOAD_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.payload.v1\0";
const PROVIDER_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.provider.v1\0";
const DEFERRED_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.deferred.v1\0";

fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Converts a host index of a verified arena into its compact ordinal.
fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("a bounded verified arena fits u32")
}

/// Reads the stage one executable entry dispatches.
///
/// Entry positions are proven to be a bijection onto the program's stages
/// before the artifact is frozen, so the lookup is a verified invariant rather
/// than caller input.
fn stage_at(program: &VerifiedKernelProgram, entry: usize) -> StageRef<'_> {
    program
        .stages()
        .nth(entry)
        .expect("a verified entry names a stage of its own program")
}

pub(super) fn push_shape(bytes: &mut Vec<u8>, shape: &Shape) {
    push_len(bytes, shape.rank());
    for extent in shape.extents() {
        bytes.extend_from_slice(&extent.get().to_be_bytes());
    }
}

/// A governed `{major, minor}` schema version of one artifact component.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion {
    major: u16,
    minor: u16,
}

impl SchemaVersion {
    /// Declares one component schema version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the major version; a mismatch is incompatible.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor version; a reader must support at least this minor.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.major.to_be_bytes());
        bytes.extend_from_slice(&self.minor.to_be_bytes());
    }
}

/// The governed component schema versions one artifact program was written at.
///
/// Versioning is component-wise on purpose: widening the ABI expression
/// language must not force a reader that only inspects routing to reject the
/// artifact. A producer never chooses these; the builder stamps
/// [`ArtifactSchema::GOVERNED`], so the version always names the code that
/// actually assembled the model.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ArtifactSchema {
    program: SchemaVersion,
    abi_expression: SchemaVersion,
    guard_and_routing: SchemaVersion,
    target_requirement: SchemaVersion,
}

impl ArtifactSchema {
    /// The component versions this build of the crate produces.
    pub const GOVERNED: Self = Self {
        program: SchemaVersion::new(1, 0),
        abi_expression: SchemaVersion::new(1, 0),
        guard_and_routing: SchemaVersion::new(1, 0),
        target_requirement: SchemaVersion::new(1, 0),
    };

    /// Returns the artifact program schema version.
    #[must_use]
    pub const fn program(self) -> SchemaVersion {
        self.program
    }

    /// Returns the ABI expression language schema version.
    #[must_use]
    pub const fn abi_expression(self) -> SchemaVersion {
        self.abi_expression
    }

    /// Returns the guard and routing schema version.
    #[must_use]
    pub const fn guard_and_routing(self) -> SchemaVersion {
        self.guard_and_routing
    }

    /// Returns the target requirement schema version.
    #[must_use]
    pub const fn target_requirement(self) -> SchemaVersion {
        self.target_requirement
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        let Self {
            program,
            abi_expression,
            guard_and_routing,
            target_requirement,
        } = self;
        program.encode(bytes);
        abi_expression.encode(bytes);
        guard_and_routing.encode(bytes);
        target_requirement.encode(bytes);
    }
}

/// The canonical policy by which a runtime chooses among applicable variants.
///
/// The bounded profile fixes [`RoutingPolicy::StablePriority`]; the builder
/// offers no setter, and the tag is folded into identity so a later
/// piecewise-cost or constraint-region policy is an explicit identity change
/// rather than a silent behavioural one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RoutingPolicy {
    /// Try variants in their declared order and take the first applicable one.
    ///
    /// Declaration order is therefore semantic and is retained in identity.
    StablePriority,
}

impl RoutingPolicy {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::StablePriority => 0x01,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized policy.
    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::StablePriority),
            _ => None,
        }
    }
}

/// How a backend payload's bytes reach an executable state on a device.
///
/// A payload must not claim [`ArtifactExecutionPolicy::NativeImage`] merely
/// because no source is compiled at run time; ahead-of-time output that still
/// needs device-specific pipeline creation is
/// [`ArtifactExecutionPolicy::RequiresDeviceTranslation`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArtifactExecutionPolicy {
    /// The payload bytes are directly loadable on a compatible device.
    NativeImage,
    /// The payload requires device-specific translation before execution.
    RequiresDeviceTranslation,
}

impl ArtifactExecutionPolicy {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::NativeImage => 0x01,
            Self::RequiresDeviceTranslation => 0x02,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized policy.
    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::NativeImage),
            0x02 => Some(Self::RequiresDeviceTranslation),
            _ => None,
        }
    }
}

/// The transport category one ABI binding uses.
///
/// The bounded profile carries buffers only. The category is explicit anyway so
/// a metadata block, an inline scalar, or an error record becomes a new variant
/// rather than an overloaded buffer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BindingKind {
    /// A whole addressable buffer range.
    Buffer,
}

impl BindingKind {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Buffer => 0x01,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized category.
    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Buffer),
            _ => None,
        }
    }
}

/// One capability provider the packaged plan actually reached.
///
/// ADR 0072: a provider that was available but not selected is
/// compilation-request environment, never packaged artifact identity. Only the
/// providers recorded here reach [`CanonicalArtifactProgramIdentity`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SelectedProvider {
    /// Identity and nonzero output-affecting revision of the provider.
    pub provider: ProviderIdentity,
    /// Governed capability key the provider was selected for.
    pub capability: CapabilityKey,
    /// Version of the capability API the selection was made against.
    pub capability_api_version: u16,
}

impl SelectedProvider {
    pub(super) fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PROVIDER_KEY_DOMAIN);
        push_slice(&mut bytes, self.provider.namespace().as_bytes());
        push_slice(&mut bytes, self.provider.name().as_bytes());
        bytes.extend_from_slice(&self.provider.revision().to_be_bytes());
        push_slice(&mut bytes, self.capability.as_str().as_bytes());
        bytes.extend_from_slice(&self.capability_api_version.to_be_bytes());
        bytes
    }
}

/// One backend payload the artifact's executable entries are realized by.
///
/// The neutral layer names a payload by governed keys, its schema, its exact
/// content digest, and its execution policy. It deliberately holds no backend
/// spelling: no symbol names, no binding indices, no platform triples.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BackendPayloadDescriptor {
    /// Governed backend family key.
    pub backend: BackendKey,
    /// Governed executable-representation key.
    pub representation: RepresentationKey,
    /// Schema version of the backend's own payload metadata.
    pub payload_schema: SchemaVersion,
    /// Exact content digest of the payload bytes.
    pub digest: PayloadDigest,
    /// Target profile this payload's own bytes were built against.
    ///
    /// This is the *payload's* compatibility contract, not the plan's. The two
    /// coincide only while a payload is realized by one variant, and nothing in
    /// this model requires that: entries cross-reference payloads by index, so
    /// two variants declaring different `TargetProfileRef`s may realize their
    /// entries through one payload. Without this field a loader would have to
    /// infer the payload's contract from whichever variant it happened to route
    /// to, which is exactly the inference this layer exists to forbid.
    pub compatibility: TargetProfileRef,
    /// How the payload reaches an executable state.
    pub execution_policy: ArtifactExecutionPolicy,
}

impl BackendPayloadDescriptor {
    pub(super) fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PAYLOAD_KEY_DOMAIN);
        push_slice(&mut bytes, self.backend.as_str().as_bytes());
        push_slice(&mut bytes, self.representation.as_str().as_bytes());
        self.payload_schema.encode(&mut bytes);
        push_slice(&mut bytes, self.digest.as_bytes());
        push_slice(&mut bytes, self.compatibility.key.as_str().as_bytes());
        push_slice(&mut bytes, self.compatibility.descriptor.as_bytes());
        bytes.push(self.execution_policy.tag());
        bytes
    }
}

/// The backend entry one executable entry is realized by.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendEntryRef {
    /// Payload descriptor that contains the entry.
    pub payload: PayloadId,
    /// Opaque backend entry key within that payload.
    pub entry_key: BackendEntryKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StoredBackendEntry {
    pub(super) payload: u32,
    pub(super) entry_key: BackendEntryKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BindingData {
    pub(super) kind: BindingKind,
    pub(super) element_type: KernelType,
    pub(super) address_space: AddressSpace,
    pub(super) access: BufferAccess,
    pub(super) alignment: u32,
    pub(super) value_role: ValueRole,
    pub(super) accessible_bytes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LaunchData {
    pub(super) grid_threads: u32,
    pub(super) threads_per_workgroup: u32,
    pub(super) zero_work_skips_dispatch: bool,
    pub(super) preconditions: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EntryData {
    pub(super) bindings: Vec<BindingData>,
    pub(super) launch: LaunchData,
    pub(super) implementation: StoredBackendEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DeferredPredicateData {
    pub(super) predicate: u32,
    pub(super) phase: AvailabilityPhase,
    pub(super) authority: ProviderIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct VariantData {
    pub(super) program: VerifiedKernelProgram,
    pub(super) guard: u32,
    pub(super) profile: TargetProfileRef,
    pub(super) feasibility_rules: FeasibilityRuleSetRef,
    pub(super) deferred: Vec<DeferredPredicateData>,
    pub(super) entries: Vec<EntryData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InterfaceEntryData<K> {
    pub(super) key: K,
    pub(super) shape: Shape,
    pub(super) element_type: KernelType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ArtifactProgramData {
    pub(super) schema: ArtifactSchema,
    pub(super) semantic: SemanticIdentity,
    pub(super) routing: RoutingPolicy,
    pub(super) inputs: Vec<InterfaceEntryData<InputKey>>,
    pub(super) outputs: Vec<InterfaceEntryData<OutputKey>>,
    pub(super) providers: Vec<SelectedProvider>,
    pub(super) payloads: Vec<BackendPayloadDescriptor>,
    /// Carried content of each payload, aligned with `payloads`.
    ///
    /// `None` is the descriptor-only payload this model always admitted: the
    /// artifact names a backend object it does not carry. `Some` carries the
    /// compilation subject and the emitted bytes.
    pub(super) payload_content: Vec<Option<PayloadContent>>,
    pub(super) expressions: Vec<ExprNode>,
    pub(super) expression_keys: Vec<Vec<u8>>,
    pub(super) expression_types: Vec<AbiType>,
    pub(super) variants: Vec<VariantData>,
}

/// Opaque canonical bytes identifying one verified artifact program.
///
/// The identity folds what the artifact layer *is*: its governed component
/// schemas, the semantic subject it realizes, the complete kernel program of
/// every packaged variant, the guards and routing that choose among them, the
/// neutral ABI and launch contracts of every entry, the declared target
/// requirements, the backend payload descriptors and entry mappings, and the
/// provenance the packaged plan actually reached.
///
/// It deliberately excludes two things. **Unused compilation-environment
/// providers** never enter it: only reached admission provenance and selected
/// capability providers do, so an artifact is not invalidated by a provider it
/// never used (ADR 0072). And **transient ordinals** never enter it: expression
/// arena positions, builder insertion order, and program-local stage positions
/// are all replaced by canonical content keys, so two structurally equal
/// artifacts assembled in different orders share bytes. Variant order is the
/// one retained order, because routing priority is meaning rather than
/// insertion.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalArtifactProgramIdentity(Vec<u8>);

impl CanonicalArtifactProgramIdentity {
    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An immutable, verified target-neutral artifact program.
///
/// Only [`super::ArtifactProgramBuilder::build`] produces one. Equality
/// compares the canonical identity.
#[derive(Clone)]
pub struct VerifiedArtifactProgram {
    pub(super) data: ArtifactProgramData,
    pub(super) identity: CanonicalArtifactProgramIdentity,
}

impl PartialEq for VerifiedArtifactProgram {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for VerifiedArtifactProgram {}

impl std::fmt::Debug for VerifiedArtifactProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerifiedArtifactProgram")
            .field("variants", &self.data.variants.len())
            .field("payloads", &self.data.payloads.len())
            .field("identity_bytes", &self.identity.0.len())
            .finish()
    }
}

impl VerifiedArtifactProgram {
    /// Returns the canonical identity of this artifact program.
    #[must_use]
    pub const fn canonical_identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the governed component schema versions this artifact was written at.
    #[must_use]
    pub const fn schema(&self) -> ArtifactSchema {
        self.data.schema
    }

    /// Returns the canonical graph identity of the semantic program realized.
    #[must_use]
    pub const fn semantic_graph_identity(&self) -> &SemanticGraphIdentity {
        self.data.semantic.graph()
    }

    /// Returns the provider-independent definitions the program reached.
    #[must_use]
    pub const fn reached_definitions(&self) -> &SemanticDefinitionProjectionIdentity {
        self.data.semantic.reached_definitions()
    }

    /// Returns the provider-attributed admission provenance the program reached.
    ///
    /// This is one of the two provenance subjects folded into artifact
    /// identity. The complete frozen registry snapshot is deliberately not:
    /// it is compilation-request provenance, and packaging it would let a
    /// provider the plan never used invalidate the artifact.
    #[must_use]
    pub const fn admission_provenance(&self) -> &SemanticAdmissionProvenanceIdentity {
        self.data.semantic.admission_provenance()
    }

    /// Returns the capability providers the packaged plan actually selected.
    #[must_use]
    pub fn selected_providers(&self) -> &[SelectedProvider] {
        &self.data.providers
    }

    /// Returns the backend payload descriptors in declaration order.
    ///
    /// Declaration order is presentation only: identity encodes the descriptors
    /// in canonical content order and cross-references them by content key, so
    /// two artifacts that declare the same payloads in different orders are
    /// identical.
    #[must_use]
    pub fn payloads(&self) -> &[BackendPayloadDescriptor] {
        &self.data.payloads
    }

    /// Returns the canonical routing policy of the portfolio.
    #[must_use]
    pub const fn routing_policy(&self) -> RoutingPolicy {
        self.data.routing
    }

    /// Returns the named program inputs in the semantic interface order.
    #[must_use]
    pub fn inputs(&self) -> impl ExactSizeIterator<Item = ArtifactInputRef<'_>> {
        (0..self.data.inputs.len()).map(move |input| ArtifactInputRef {
            artifact: self,
            input,
        })
    }

    /// Returns the named program outputs in the semantic interface order.
    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = ArtifactOutputRef<'_>> {
        (0..self.data.outputs.len()).map(move |output| ArtifactOutputRef {
            artifact: self,
            output,
        })
    }

    /// Returns the plan variants in routing priority order.
    #[must_use]
    pub fn variants(&self) -> impl ExactSizeIterator<Item = VariantRef<'_>> {
        (0..self.data.variants.len()).map(move |variant| VariantRef {
            artifact: self,
            variant,
        })
    }

    /// Returns every node of the shared ABI expression arena.
    ///
    /// The arena is canonically deduplicated and contains exactly the nodes
    /// reachable from a declared use site.
    #[must_use]
    pub fn expressions(&self) -> impl ExactSizeIterator<Item = AbiExprRef<'_>> {
        (0..self.data.expressions.len()).map(move |node| AbiExprRef {
            artifact: self,
            node,
        })
    }
}

/// A read-only view of one named program input of the artifact interface.
#[derive(Clone, Copy, Debug)]
pub struct ArtifactInputRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    input: usize,
}

impl<'a> ArtifactInputRef<'a> {
    /// Returns the stable interface key of the input.
    #[must_use]
    pub fn key(self) -> &'a InputKey {
        &self.artifact.data.inputs[self.input].key
    }

    /// Returns the logical tensor shape the input must be bound with.
    #[must_use]
    pub fn shape(self) -> &'a Shape {
        &self.artifact.data.inputs[self.input].shape
    }

    /// Returns the storage element type of the input.
    #[must_use]
    pub fn element_type(self) -> KernelType {
        self.artifact.data.inputs[self.input].element_type
    }
}

/// A read-only view of one named program output of the artifact interface.
#[derive(Clone, Copy, Debug)]
pub struct ArtifactOutputRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    output: usize,
}

impl<'a> ArtifactOutputRef<'a> {
    /// Returns the stable interface key of the output.
    #[must_use]
    pub fn key(self) -> &'a OutputKey {
        &self.artifact.data.outputs[self.output].key
    }

    /// Returns the logical tensor shape the output is published with.
    #[must_use]
    pub fn shape(self) -> &'a Shape {
        &self.artifact.data.outputs[self.output].shape
    }

    /// Returns the storage element type of the output.
    #[must_use]
    pub fn element_type(self) -> KernelType {
        self.artifact.data.outputs[self.output].element_type
    }
}

/// A read-only view of one complete plan variant.
#[derive(Clone, Copy, Debug)]
pub struct VariantRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    variant: usize,
}

impl<'a> VariantRef<'a> {
    /// Returns the zero-based routing rank; lower is tried first.
    #[must_use]
    pub fn routing_rank(self) -> usize {
        self.variant
    }

    /// Returns the complete verified kernel program this variant executes.
    #[must_use]
    pub fn program(self) -> &'a VerifiedKernelProgram {
        &self.data().program
    }

    /// Returns the applicability guard evaluated before this variant is chosen.
    #[must_use]
    pub fn applicability_guard(self) -> AbiExprRef<'a> {
        AbiExprRef {
            artifact: self.artifact,
            node: position(self.data().guard),
        }
    }

    /// Returns the declared target profile this variant was assessed against.
    #[must_use]
    pub fn target_profile(self) -> &'a TargetProfileRef {
        &self.data().profile
    }

    /// Returns the feasibility rule set this variant was assessed under.
    #[must_use]
    pub fn feasibility_rules(self) -> &'a FeasibilityRuleSetRef {
        &self.data().feasibility_rules
    }

    /// Returns the deferred feasibility predicates of this variant.
    #[must_use]
    pub fn deferred_predicates(self) -> impl ExactSizeIterator<Item = DeferredPredicateRef<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        (0..self.data().deferred.len()).map(move |predicate| DeferredPredicateRef {
            artifact,
            variant,
            predicate,
        })
    }

    /// Returns the executable entries, one per stage of the variant's program.
    #[must_use]
    pub fn entries(self) -> impl ExactSizeIterator<Item = EntryRef<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        (0..self.data().entries.len()).map(move |entry| EntryRef {
            artifact,
            variant,
            entry,
        })
    }

    fn data(self) -> &'a VariantData {
        &self.artifact.data.variants[self.variant]
    }
}

/// A read-only view of one deferred feasibility predicate.
#[derive(Clone, Copy, Debug)]
pub struct DeferredPredicateRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    variant: usize,
    predicate: usize,
}

impl<'a> DeferredPredicateRef<'a> {
    /// Returns the predicate that must hold before routing commits.
    #[must_use]
    pub fn predicate(self) -> AbiExprRef<'a> {
        AbiExprRef {
            artifact: self.artifact,
            node: position(self.data().predicate),
        }
    }

    /// Returns the phase at which the predicate becomes decidable.
    #[must_use]
    pub fn phase(self) -> AvailabilityPhase {
        self.data().phase
    }

    /// Returns the selected provider that must answer the query.
    #[must_use]
    pub fn authority(self) -> &'a ProviderIdentity {
        &self.data().authority
    }

    fn data(self) -> &'a DeferredPredicateData {
        &self.artifact.data.variants[self.variant].deferred[self.predicate]
    }
}

/// A read-only view of one executable entry.
#[derive(Clone, Copy, Debug)]
pub struct EntryRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    variant: usize,
    entry: usize,
}

impl<'a> EntryRef<'a> {
    /// Returns the program stage this entry dispatches.
    #[must_use]
    pub fn stage(self) -> StageRef<'a> {
        stage_at(
            &self.artifact.data.variants[self.variant].program,
            self.entry,
        )
    }

    /// Returns the canonical identity of the kernel this entry realizes.
    #[must_use]
    pub fn kernel_identity(self) -> &'a CanonicalKernelIdentity {
        self.stage().kernel().canonical_identity()
    }

    /// Returns the exact resource requirements the bound kernel proved.
    #[must_use]
    pub fn resources(self) -> ResourceRequirements {
        self.stage().kernel().requirements()
    }

    /// Returns the numerical realization the bound kernel preserves.
    #[must_use]
    pub fn numerical(self) -> NumericalRealization {
        self.stage().kernel().numerical()
    }

    /// Returns the ABI bindings in kernel buffer-parameter order.
    #[must_use]
    pub fn bindings(self) -> impl ExactSizeIterator<Item = BindingRef<'a>> {
        let artifact = self.artifact;
        let variant = self.variant;
        let entry = self.entry;
        (0..self.data().bindings.len()).map(move |binding| BindingRef {
            artifact,
            variant,
            entry,
            binding,
        })
    }

    /// Returns the total launch thread count expression.
    #[must_use]
    pub fn launch_threads(self) -> AbiExprRef<'a> {
        self.expression(self.data().launch.grid_threads)
    }

    /// Returns the per-workgroup thread count expression.
    #[must_use]
    pub fn threads_per_workgroup(self) -> AbiExprRef<'a> {
        self.expression(self.data().launch.threads_per_workgroup)
    }

    /// Returns whether a zero-work launch skips the dispatch entirely.
    #[must_use]
    pub fn zero_work_skips_dispatch(self) -> bool {
        self.data().launch.zero_work_skips_dispatch
    }

    /// Returns the launch-instance preconditions of this entry.
    #[must_use]
    pub fn launch_preconditions(self) -> impl ExactSizeIterator<Item = AbiExprRef<'a>> {
        let artifact = self.artifact;
        self.data()
            .launch
            .preconditions
            .iter()
            .map(move |node| AbiExprRef {
                artifact,
                node: position(*node),
            })
    }

    /// Returns the backend payload descriptor this entry is realized by.
    #[must_use]
    pub fn payload(self) -> &'a BackendPayloadDescriptor {
        &self.artifact.data.payloads[position(self.data().implementation.payload)]
    }

    /// Returns the opaque backend entry key within that payload.
    #[must_use]
    pub fn backend_entry_key(self) -> &'a BackendEntryKey {
        &self.data().implementation.entry_key
    }

    fn expression(self, node: u32) -> AbiExprRef<'a> {
        AbiExprRef {
            artifact: self.artifact,
            node: position(node),
        }
    }

    fn data(self) -> &'a EntryData {
        &self.artifact.data.variants[self.variant].entries[self.entry]
    }
}

/// A read-only view of one ABI binding of an executable entry.
#[derive(Clone, Copy, Debug)]
pub struct BindingRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    variant: usize,
    entry: usize,
    binding: usize,
}

impl<'a> BindingRef<'a> {
    /// Returns the zero-based ABI slot; the order is the kernel signature's.
    #[must_use]
    pub fn slot(self) -> usize {
        self.binding
    }

    /// Returns the transport category of the binding.
    #[must_use]
    pub fn kind(self) -> BindingKind {
        self.data().kind
    }

    /// Returns the storage element type addressed through the binding.
    #[must_use]
    pub fn element_type(self) -> KernelType {
        self.data().element_type
    }

    /// Returns the logical address space the binding requires.
    #[must_use]
    pub fn address_space(self) -> AddressSpace {
        self.data().address_space
    }

    /// Returns whether the entry reads or writes through the binding.
    #[must_use]
    pub fn access(self) -> BufferAccess {
        self.data().access
    }

    /// Returns the byte alignment the bound storage must satisfy.
    #[must_use]
    pub fn alignment(self) -> u32 {
        self.data().alignment
    }

    /// Returns the program role of the bound value.
    #[must_use]
    pub fn value_role(self) -> ValueRole {
        self.data().value_role
    }

    /// Returns the minimum accessible byte range expression.
    #[must_use]
    pub fn accessible_bytes(self) -> AbiExprRef<'a> {
        AbiExprRef {
            artifact: self.artifact,
            node: position(self.data().accessible_bytes),
        }
    }

    /// Returns the materialized program value this binding addresses.
    #[must_use]
    pub fn value(self) -> MaterializedValueRef<'a> {
        self.access_ref().view().value()
    }

    /// Returns the byte window of that value the binding addresses.
    #[must_use]
    pub fn window(self) -> ByteWindow {
        self.access_ref().view().window()
    }

    fn access_ref(self) -> tiler_ir::program::StageAccessRef<'a> {
        EntryRef {
            artifact: self.artifact,
            variant: self.variant,
            entry: self.entry,
        }
        .stage()
        .accesses()
        .nth(self.binding)
        .expect("a verified binding names an access of its own stage")
    }

    fn data(self) -> &'a BindingData {
        &self.artifact.data.variants[self.variant].entries[self.entry].bindings[self.binding]
    }
}

/// A read-only view of one node of the shared ABI expression arena.
#[derive(Clone, Copy, Debug)]
pub struct AbiExprRef<'a> {
    artifact: &'a VerifiedArtifactProgram,
    node: usize,
}

impl<'a> AbiExprRef<'a> {
    /// Returns the value type this expression produces.
    #[must_use]
    pub fn value_type(self) -> AbiType {
        self.artifact.data.expression_types[self.node]
    }

    /// Returns the typed structure of this expression node.
    #[must_use]
    pub fn view(self) -> AbiExprView<'a> {
        match &self.artifact.data.expressions[self.node] {
            ExprNode::Root(root) => AbiExprView::Root(root),
            ExprNode::Unary { op, operand } => AbiExprView::Unary {
                op: *op,
                operand: self.sibling(*operand),
            },
            ExprNode::Binary { op, left, right } => AbiExprView::Binary {
                op: *op,
                left: self.sibling(*left),
                right: self.sibling(*right),
            },
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => AbiExprView::Select {
                condition: self.sibling(*condition),
                if_true: self.sibling(*if_true),
                if_false: self.sibling(*if_false),
            },
        }
    }

    /// Evaluates this expression against an already-bound fact environment.
    ///
    /// # Errors
    ///
    /// Returns [`AbiEvaluationError`] for an unbound root, checked-arithmetic
    /// overflow or underflow, a zero divisor, an inexact exact division, or a
    /// narrowing that does not fit.
    pub fn evaluate(self, facts: &AbiFacts) -> Result<AbiValue, AbiEvaluationError> {
        evaluate(&self.artifact.data.expressions, ordinal(self.node), facts)
    }

    fn sibling(self, node: u32) -> Self {
        Self {
            artifact: self.artifact,
            node: position(node),
        }
    }
}

/// The typed structure of one ABI expression node.
#[derive(Clone, Copy, Debug)]
pub enum AbiExprView<'a> {
    /// A typed root fact.
    Root(&'a AbiRoot),
    /// One unary operation over an earlier node.
    Unary {
        /// Operation applied.
        op: AbiUnaryOp,
        /// Operand expression.
        operand: AbiExprRef<'a>,
    },
    /// One binary operation over two earlier nodes.
    Binary {
        /// Operation applied.
        op: AbiBinaryOp,
        /// Left operand expression.
        left: AbiExprRef<'a>,
        /// Right operand expression.
        right: AbiExprRef<'a>,
    },
    /// A conditional selection that evaluates only the branch it takes.
    Select {
        /// Predicate deciding the branch.
        condition: AbiExprRef<'a>,
        /// Branch taken when the predicate holds.
        if_true: AbiExprRef<'a>,
        /// Branch taken otherwise.
        if_false: AbiExprRef<'a>,
    },
}

/// Derives the canonical content key of one program stage.
///
/// The key is the shared IR's own stage subject — the exact bound kernel and
/// the semantic occurrences it covers — so an artifact entry cross-references a
/// stage by content rather than by the program's declaration position.
pub(super) fn stage_key(stage: StageRef<'_>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(STAGE_KEY_DOMAIN);
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());
    push_len(&mut bytes, stage.coverage().len());
    for occurrence in stage.coverage() {
        bytes.extend_from_slice(&occurrence.get().to_be_bytes());
    }
    bytes
}

// Each shared-IR vocabulary below has exactly one tag table, written as an
// adjacent forward and inverse pair. Two tables that agreed only by inspection
// would let an envelope decode into a plausible-but-wrong program, so the pair
// is kept in one place and pinned by an exhaustive round-trip test.

pub(super) fn element_type_tag(element_type: KernelType) -> Result<u8, ArtifactDiagnostic> {
    // `KernelType` is `#[non_exhaustive]`, so a widened variant cannot break
    // this cross-crate encoder at compile time the way ADR 0074 §3 intends.
    // Rejecting is the only remaining fail-closed behaviour: an unrecognized
    // element type must never share identity bytes with a recognized one.
    match element_type {
        KernelType::Bool => Ok(0x01),
        KernelType::Index => Ok(0x02),
        KernelType::F32 => Ok(0x03),
        _ => Err(ArtifactDiagnostic::UnrecognizedForeignVariant {
            subject: ForeignEnumSubject::KernelType,
        }),
    }
}

pub(super) const fn element_type_from_tag(tag: u8) -> Option<KernelType> {
    match tag {
        0x01 => Some(KernelType::Bool),
        0x02 => Some(KernelType::Index),
        0x03 => Some(KernelType::F32),
        _ => None,
    }
}

pub(super) fn address_space_tag(address_space: AddressSpace) -> Result<u8, ArtifactDiagnostic> {
    match address_space {
        AddressSpace::Device => Ok(0x01),
        AddressSpace::Workgroup => Ok(0x02),
        AddressSpace::InvocationPrivate => Ok(0x03),
        AddressSpace::Constant => Ok(0x04),
        _ => Err(ArtifactDiagnostic::UnrecognizedForeignVariant {
            subject: ForeignEnumSubject::AddressSpace,
        }),
    }
}

pub(super) const fn address_space_from_tag(tag: u8) -> Option<AddressSpace> {
    match tag {
        0x01 => Some(AddressSpace::Device),
        0x02 => Some(AddressSpace::Workgroup),
        0x03 => Some(AddressSpace::InvocationPrivate),
        0x04 => Some(AddressSpace::Constant),
        _ => None,
    }
}

pub(super) fn buffer_access_tag(access: BufferAccess) -> Result<u8, ArtifactDiagnostic> {
    match access {
        BufferAccess::Read => Ok(0x01),
        BufferAccess::Write => Ok(0x02),
        _ => Err(ArtifactDiagnostic::UnrecognizedForeignVariant {
            subject: ForeignEnumSubject::BufferAccess,
        }),
    }
}

pub(super) const fn buffer_access_from_tag(tag: u8) -> Option<BufferAccess> {
    match tag {
        0x01 => Some(BufferAccess::Read),
        0x02 => Some(BufferAccess::Write),
        _ => None,
    }
}

pub(super) const fn value_role_tag(role: ValueRole) -> u8 {
    match role {
        ValueRole::Input => 0x01,
        ValueRole::Temporary => 0x02,
        ValueRole::Output => 0x03,
    }
}

pub(super) const fn value_role_from_tag(tag: u8) -> Option<ValueRole> {
    match tag {
        0x01 => Some(ValueRole::Input),
        0x02 => Some(ValueRole::Temporary),
        0x03 => Some(ValueRole::Output),
        _ => None,
    }
}

/// Encodes one subnormal dimension.
///
/// A flush names the zero it produces, so the two flush behaviours receive
/// distinct tags: they are different values, and collapsing them here would
/// encode an artifact that a decoder could not distinguish from one delivering
/// the other zero.
pub(super) const fn subnormal_tag(mode: SubnormalMode) -> u8 {
    match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    }
}

pub(super) const fn subnormal_from_tag(tag: u8) -> Option<SubnormalMode> {
    match tag {
        0x01 => Some(SubnormalMode::Preserve),
        0x02 => Some(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        }),
        0x03 => Some(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        }),
        _ => None,
    }
}

pub(super) const fn permission_tag(permission: NumericalPermission) -> u8 {
    match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    }
}

pub(super) const fn permission_from_tag(tag: u8) -> Option<NumericalPermission> {
    match tag {
        0x01 => Some(NumericalPermission::Forbidden),
        0x02 => Some(NumericalPermission::Permitted),
        _ => None,
    }
}

pub(super) fn push_resources(bytes: &mut Vec<u8>, resources: ResourceRequirements) {
    let ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup,
        local_memory_bytes,
        barriers,
        requires_device_memory,
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
    } = resources;
    bytes.extend_from_slice(&buffer_bindings.to_be_bytes());
    bytes.extend_from_slice(&threads_per_workgroup.to_be_bytes());
    bytes.extend_from_slice(&local_memory_bytes.to_be_bytes());
    bytes.extend_from_slice(&barriers.to_be_bytes());
    bytes.push(u8::from(requires_device_memory));
    bytes.push(subnormal_tag(input_subnormals));
    bytes.push(subnormal_tag(result_subnormals));
    bytes.push(permission_tag(contraction));
    bytes.push(permission_tag(reassociation));
}

pub(super) fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalFacts) {
    let NumericalFacts {
        profile_key,
        canonical_arithmetic_nan_bits,
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
    } = numerical;
    push_slice(bytes, profile_key.as_bytes());
    bytes.extend_from_slice(&canonical_arithmetic_nan_bits.to_be_bytes());
    bytes.push(subnormal_tag(*input_subnormals));
    bytes.push(subnormal_tag(*result_subnormals));
    bytes.push(permission_tag(*contraction));
    bytes.push(permission_tag(*reassociation));
}

/// Encodes the canonical identity of one packaged artifact program.
///
/// The subject is the canonical envelope, not the builder's draft storage, so
/// the identity a producer stamps and the identity a decoder re-derives come
/// from one encoder rather than from two that agree by inspection.
///
/// # Errors
///
/// Returns [`ArtifactDiagnostic::AmbiguousCanonicalKey`] when two payloads,
/// providers, deferred predicates, or launch preconditions produce equal keys,
/// [`ArtifactDiagnostic::UnrecognizedForeignVariant`] for a shared-IR variant
/// this encoder does not recognize, or
/// [`ArtifactDiagnostic::IdentityLimit`] when the encoding exceeds its bound.
pub(super) fn encode_identity(
    envelope: &ArtifactEnvelope,
) -> Result<CanonicalArtifactProgramIdentity, ArtifactDiagnostic> {
    let keys = expression_keys(envelope.expressions());
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ARTIFACT_DOMAIN);
    envelope.schema().encode(&mut bytes);
    bytes.push(envelope.routing_policy().tag());
    push_slice(&mut bytes, envelope.semantic().graph.as_bytes());
    push_slice(
        &mut bytes,
        envelope.semantic().reached_definitions.as_bytes(),
    );
    push_slice(
        &mut bytes,
        envelope.semantic().admission_provenance.as_bytes(),
    );
    push_interface(&mut bytes, envelope)?;
    push_sorted_keys(
        &mut bytes,
        envelope
            .providers()
            .iter()
            .map(SelectedProvider::canonical_key),
        ArtifactEntityKind::Provider,
    )?;
    let payload_keys: Vec<Vec<u8>> = envelope
        .payloads()
        .iter()
        .map(BackendPayloadDescriptor::canonical_key)
        .collect();
    push_sorted_keys(
        &mut bytes,
        payload_keys.iter().cloned(),
        ArtifactEntityKind::Payload,
    )?;
    push_len(&mut bytes, envelope.variants().len());
    for variant in envelope.variants() {
        push_variant(&mut bytes, envelope, &keys, variant, &payload_keys)?;
    }
    if bytes.len() > MAX_ARTIFACT_IDENTITY_BYTES {
        return Err(ArtifactDiagnostic::IdentityLimit {
            bytes: bytes.len(),
            limit: MAX_ARTIFACT_IDENTITY_BYTES,
        });
    }
    Ok(CanonicalArtifactProgramIdentity(bytes))
}

fn push_interface(
    bytes: &mut Vec<u8>,
    envelope: &ArtifactEnvelope,
) -> Result<(), ArtifactDiagnostic> {
    push_len(bytes, envelope.inputs().len());
    for input in envelope.inputs() {
        push_slice(bytes, input.key.as_str().as_bytes());
        push_shape(bytes, &input.shape);
        bytes.push(element_type_tag(input.element_type)?);
    }
    push_len(bytes, envelope.outputs().len());
    for output in envelope.outputs() {
        push_slice(bytes, output.key.as_str().as_bytes());
        push_shape(bytes, &output.shape);
        bytes.push(element_type_tag(output.element_type)?);
    }
    Ok(())
}

/// Writes a set-meaning collection in canonical key order, proving distinctness.
///
/// Order independence is the point: the artifact's identity must not depend on
/// the order a producer happened to declare providers or payloads in, and equal
/// keys would make that order unrecoverable rather than merely arbitrary.
fn push_sorted_keys(
    bytes: &mut Vec<u8>,
    keys: impl Iterator<Item = Vec<u8>>,
    entity: ArtifactEntityKind,
) -> Result<(), ArtifactDiagnostic> {
    let mut keys: Vec<Vec<u8>> = keys.collect();
    keys.sort_unstable();
    if keys.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ArtifactDiagnostic::AmbiguousCanonicalKey { entity });
    }
    push_len(bytes, keys.len());
    for key in &keys {
        push_slice(bytes, key);
    }
    Ok(())
}

fn push_variant(
    bytes: &mut Vec<u8>,
    envelope: &ArtifactEnvelope,
    keys: &[Vec<u8>],
    variant: &VariantRow,
    payload_keys: &[Vec<u8>],
) -> Result<(), ArtifactDiagnostic> {
    push_slice(
        bytes,
        &envelope.sections()[node_at(variant.program_section)].bytes,
    );
    push_slice(bytes, &keys[node_at(variant.guard)]);
    push_slice(bytes, variant.profile.key.as_str().as_bytes());
    push_slice(bytes, variant.profile.descriptor.as_bytes());
    push_slice(bytes, variant.feasibility_rules.key.as_str().as_bytes());
    bytes.extend_from_slice(&variant.feasibility_rules.revision.to_be_bytes());
    push_sorted_keys(
        bytes,
        variant
            .deferred
            .iter()
            .map(|predicate| deferred_key(keys, predicate)),
        ArtifactEntityKind::Variant,
    )?;
    push_len(bytes, variant.entries.len());
    for entry in &variant.entries {
        push_slice(bytes, entry.stage.as_bytes());
        push_entry(bytes, keys, entry, payload_keys)?;
    }
    Ok(())
}

/// Derives the canonical content key of one deferred feasibility predicate.
pub(super) fn deferred_key(keys: &[Vec<u8>], predicate: &DeferredPredicateData) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(DEFERRED_KEY_DOMAIN);
    push_slice(&mut bytes, &keys[node_at(predicate.predicate)]);
    bytes.push(predicate.phase.tag());
    push_slice(&mut bytes, predicate.authority.namespace().as_bytes());
    push_slice(&mut bytes, predicate.authority.name().as_bytes());
    bytes.extend_from_slice(&predicate.authority.revision().to_be_bytes());
    bytes
}

fn push_entry(
    bytes: &mut Vec<u8>,
    keys: &[Vec<u8>],
    entry: &EntryRow,
    payload_keys: &[Vec<u8>],
) -> Result<(), ArtifactDiagnostic> {
    push_resources(bytes, entry.resources);
    push_numerical(bytes, &entry.numerical);
    push_len(bytes, entry.bindings.len());
    for binding in &entry.bindings {
        bytes.push(binding.kind.tag());
        bytes.push(element_type_tag(binding.element_type)?);
        bytes.push(address_space_tag(binding.address_space)?);
        bytes.push(buffer_access_tag(binding.access)?);
        bytes.extend_from_slice(&binding.alignment.to_be_bytes());
        bytes.push(value_role_tag(binding.value_role));
        push_slice(bytes, &keys[node_at(binding.accessible_bytes)]);
    }
    push_slice(bytes, &keys[node_at(entry.launch.grid_threads)]);
    push_slice(bytes, &keys[node_at(entry.launch.threads_per_workgroup)]);
    bytes.push(u8::from(entry.launch.zero_work_skips_dispatch));
    push_sorted_keys(
        bytes,
        entry
            .launch
            .preconditions
            .iter()
            .map(|node| keys[node_at(*node)].clone()),
        ArtifactEntityKind::Expression,
    )?;
    push_slice(bytes, &payload_keys[node_at(entry.payload)]);
    push_slice(bytes, entry.entry_key.as_bytes());
    Ok(())
}
