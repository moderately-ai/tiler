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
use tiler_ir::program::abi::{AbiArenaTraversal, canonical_arena_traversal, compare_expr_nodes};

use super::MAX_ARTIFACT_IDENTITY_BYTES;
use super::codec::{
    ArtifactEnvelope, EntryRow, NumericalFacts, PayloadContent, VariantRow, position as node_at,
};
use super::error::{ArtifactDiagnostic, ArtifactEntityKind};
use super::expr::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue,
    AvailabilityPhase, ExprNode, evaluate,
};
use super::handles::PayloadId;
use super::keys::{
    BackendEntryKey, BackendKey, CapabilityKey, FeasibilityRuleSetRef, PayloadDigest,
    RepresentationKey, TargetProfileRef,
};

/// Versioned domain separator of one packaged artifact's canonical identity.
///
/// Raised to `v2` when each ABI binding gained the interface reference naming
/// what it addresses. A domain bump rather than a silent re-encoding: the field
/// landed *inside* the per-binding record, so a `v1` and a `v2` encoding of two
/// different artifacts could in principle produce equal bytes, and two artifacts
/// that are not the same artifact must never share an identity. Separating the
/// domains makes that impossible rather than unlikely.
///
/// Raised to `v3` when a selected provider's trailing fixed-width integer
/// stopped being a `u16` capability API version and became the `u32` capability
/// revision. The same argument applies with an additional edge: the width moved
/// as well as the meaning, so a `v2` provider key and a `v3` provider key of two
/// different selections can differ only in bytes a reader of either domain would
/// have consumed as something else. Retagging the domain is what makes the two
/// encodings incomparable instead of merely unlikely to collide.
/// Raised to `v4` when each variant gained its stage execution order and the
/// typed dependency edges that order discharges. The same argument as `v2` and
/// `v3`, and it is the only one that applies: the fields landed *inside* the
/// per-variant record, so a `v3` and a `v4` encoding of two different artifacts
/// could in principle produce equal bytes.
///
/// Note what is *not* the reason, because it looks like one. Two artifacts whose
/// stages run in different orders already differ under `v3`: [`push_variant`]
/// folds the variant's program-section bytes, and that section is the kernel
/// program's canonical identity, which the shared IR derives over its own
/// dependency graph. The new rows make the order *readable* by a consumer
/// holding only bytes; they do not make two orders distinguishable, which they
/// already were.
///
/// # Why this is a `v5` step
///
/// `v4` embedded a full copy of an expression's key at every use site, so a
/// node's encoding contained its whole subtree — quadratic on a chain and
/// doubling per level on a shared DAG. `v5` writes the arena **once**, in a
/// canonical numbering, and names every use by its fixed-width canonical
/// position. The identity is linear in arena size and stays exactly injective:
/// the arena section is complete and self-delimiting, so a position determines
/// its node as precisely as a full copy did.
///
/// The two expression sets a variant carries — deferred predicates and each
/// entry's launch preconditions — are ordered by structural comparison rather
/// than by key bytes, because the numbering is derived from the order they are
/// written in and an order derived from the numbering would be circular. That
/// changes their canonical order as well as their spelling, which is the second
/// reason this is a domain step and not a re-encoding.
const ARTIFACT_DOMAIN: &[u8] = b"tiler.artifact-program.v5\0";
const STAGE_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.stage.v1\0";
const PAYLOAD_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.payload.v1\0";
/// Versioned domain separator of one selected provider's canonical key.
///
/// `v2` for the same change that took [`ARTIFACT_DOMAIN`] to `v3`: this record's
/// trailing integer changed both width and meaning. Retagged here as well as
/// there because a provider key is also compared to its siblings on its own —
/// `encode_identity` sorts and deduplicates these keys — so the record needs to
/// be self-describing rather than relying on the enclosing domain.
const PROVIDER_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.provider.v2\0";
const DEFERRED_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.deferred.v1\0";

/// Width of the canonical length prefix [`push_len`] writes.
///
/// Named so the exact-capacity expressions below read as the encoding they
/// mirror. This is **not** a second definition of the framing rule — `push_len`
/// remains its sole writer. What holds this constant to that writer is the
/// `debug_assert_eq!` each presized encoder ends with, which fails the moment a
/// capacity expression and the bytes actually written disagree, rather than the
/// two agreeing only by inspection.
pub(super) const LENGTH_BYTES: usize = 8;

/// Byte length one [`push_slice`] call appends for a run of `len` bytes.
pub(super) const fn framed(len: usize) -> usize {
    LENGTH_BYTES + len
}

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
    /// Output-affecting revision of that capability, as the compiler minted it.
    ///
    /// Two revisions, not one. `docs/operation-extensions.md` fixes them as
    /// independent — "one provider may register several capabilities that move
    /// at different rates, and both revisions are retained wherever a lowering's
    /// provenance is recorded" — so [`Self::provider`]'s revision does not
    /// determine this one and folding only the first left a provider free to
    /// change what it emits and produce an identical artifact identity.
    ///
    /// Received, not derived: this is `tiler-compiler`'s
    /// `SelectedCapability::capability_revision`, carried whole. That crate
    /// documents it nonzero and this layer does not re-check it, exactly as
    /// [`FeasibilityRuleSetRef::revision`] beside it does not.
    pub capability_revision: u32,
}

impl SelectedProvider {
    /// Derives this selection's canonical content key.
    ///
    /// Destructured irrefutably, so a field added to this record fails to
    /// compile here rather than silently leaving artifact identity.
    /// [`ProviderIdentity`] is another crate's type and is read through its
    /// accessors instead; that crate owns the same obligation for its own
    /// fields.
    pub(super) fn canonical_key(&self) -> Vec<u8> {
        let Self {
            provider,
            capability,
            capability_revision,
        } = self;
        let exact = PROVIDER_KEY_DOMAIN.len()
            + framed(provider.namespace().len())
            + framed(provider.name().len())
            + size_of::<u32>()
            + framed(capability.as_str().len())
            + size_of::<u32>();
        let mut bytes = Vec::with_capacity(exact);
        bytes.extend_from_slice(PROVIDER_KEY_DOMAIN);
        push_slice(&mut bytes, provider.namespace().as_bytes());
        push_slice(&mut bytes, provider.name().as_bytes());
        bytes.extend_from_slice(&provider.revision().to_be_bytes());
        push_slice(&mut bytes, capability.as_str().as_bytes());
        bytes.extend_from_slice(&capability_revision.to_be_bytes());
        debug_assert_eq!(bytes.len(), exact, "provider key capacity is exact");
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
    /// Derives this descriptor's canonical content key.
    ///
    /// Every field is a compilation *input*: [`Self::digest`] is the identity
    /// of the payload's compilation subject rather than of the emitted object
    /// — `super::codec`'s payload module states and its decoder re-proves that
    /// — so this key, and therefore the artifact identity that folds it, is
    /// derivable before the backend compiler has run.
    ///
    /// Destructured irrefutably, so a field added to this record fails to
    /// compile here rather than silently leaving artifact identity.
    pub(super) fn canonical_key(&self) -> Vec<u8> {
        let Self {
            backend,
            representation,
            payload_schema,
            digest,
            compatibility,
            execution_policy,
        } = self;
        let TargetProfileRef {
            key: compatibility_key,
            descriptor: compatibility_descriptor,
        } = compatibility;
        // A `SchemaVersion` encodes as two `u16`s; the trailing byte is the
        // execution-policy tag.
        let exact = PAYLOAD_KEY_DOMAIN.len()
            + framed(backend.as_str().len())
            + framed(representation.as_str().len())
            + 2 * size_of::<u16>()
            + framed(digest.as_bytes().len())
            + framed(compatibility_key.as_str().len())
            + framed(compatibility_descriptor.as_bytes().len())
            + 1;
        let mut bytes = Vec::with_capacity(exact);
        bytes.extend_from_slice(PAYLOAD_KEY_DOMAIN);
        push_slice(&mut bytes, backend.as_str().as_bytes());
        push_slice(&mut bytes, representation.as_str().as_bytes());
        payload_schema.encode(&mut bytes);
        push_slice(&mut bytes, digest.as_bytes());
        push_slice(&mut bytes, compatibility_key.as_str().as_bytes());
        push_slice(&mut bytes, compatibility_descriptor.as_bytes());
        bytes.push(execution_policy.tag());
        debug_assert_eq!(bytes.len(), exact, "payload key capacity is exact");
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

/// The owned storage form of [`BindingTarget`]; see it for the design.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BindingTargetData {
    /// The externally bound tensor of one named program input.
    ProgramInput(InputKey),
    /// Storage published under every named program output listed.
    ///
    /// A list rather than one key because `push_output` rejects a repeated
    /// *key* and not a repeated *value*, so two named outputs may publish one
    /// program value. Carrying only one of them would leave a loader binding a
    /// second buffer for storage that is already bound, or leaving a declared
    /// output unwritten. Canonically ordered and never empty.
    ProgramOutput(Vec<OutputKey>),
    /// Storage the program produced for itself, which the runtime allocates.
    Internal,
}

impl BindingTargetData {
    /// Returns the program role this target implies.
    ///
    /// Derived rather than stored beside the target. The two encode one fact,
    /// and a stored copy could disagree with the reference it describes.
    pub(super) const fn value_role(&self) -> ValueRole {
        match self {
            Self::ProgramInput(_) => ValueRole::Input,
            Self::ProgramOutput(_) => ValueRole::Output,
            Self::Internal => ValueRole::Temporary,
        }
    }
}

/// What one ABI binding slot addresses.
///
/// # Why the interface, and not a program value
///
/// The obvious spelling — the materialized value the binding's stage access
/// reaches — has no durable name this layer can carry. A builder arena position
/// is exactly the transient fact artifact identity replaces with canonical
/// content keys everywhere else, and the shared IR's own canonical *value* key
/// is crate-private to `tiler_ir::program` with no read view publishing it.
///
/// What an artifact can name is the **semantic interface**, which the envelope
/// already carries and which artifact identity already folds. Every value an
/// artifact binding addresses is either one of those named entries or storage
/// the program produced for itself, and those are different instructions to a
/// loader rather than shades of one.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5c): a target
/// is an instruction to a loader, and a reader that gained a wildcard arm would
/// silently route a newly governed target class into whichever arm the wildcard
/// named. Adding a class must break the build at every reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingTarget<'a> {
    /// The externally bound tensor of one named program input.
    ///
    /// A loader binds the host buffer it was given for this interface key.
    ProgramInput(&'a InputKey),
    /// Storage published under every named program output listed.
    ///
    /// One buffer, published under each key. The list is canonically ordered
    /// and never empty; more than one key means the plan publishes one value
    /// under several names, not that there are several buffers.
    ProgramOutput(&'a [OutputKey]),
    /// Storage the program produced for itself.
    ///
    /// A loader allocates it rather than binding host data, sized by the
    /// binding's own accessible-byte expression. It carries no name, for the
    /// reason above, so two `Internal` slots are indistinguishable — which is
    /// why [`ArtifactBuildError::AliasedInternalBinding`] refuses to package an
    /// entry whose two bindings address one internal value rather than encode
    /// the ambiguity.
    ///
    /// [`ArtifactBuildError::AliasedInternalBinding`]: super::ArtifactBuildError::AliasedInternalBinding
    Internal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BindingData {
    pub(super) kind: BindingKind,
    pub(super) element_type: KernelType,
    pub(super) address_space: AddressSpace,
    pub(super) access: BufferAccess,
    pub(super) alignment: u32,
    pub(super) target: BindingTargetData,
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
/// It deliberately excludes three things. **Unused compilation-environment
/// providers** never enter it: only reached admission provenance and selected
/// capability providers do, so an artifact is not invalidated by a provider it
/// never used (ADR 0072). **Transient ordinals** never enter it: expression
/// arena positions, builder insertion order, and program-local stage positions
/// are all replaced by canonical content keys, so two structurally equal
/// artifacts assembled in different orders share bytes. Variant order is the
/// one retained order, because routing priority is meaning rather than
/// insertion. And **emitted backend object bytes** never enter it: a payload is
/// named by the digest of its compilation subject, so a non-reproducible linker
/// does not change what artifact this is.
///
/// # This is a pre-compilation subject
///
/// The last exclusion is the load-bearing one for anything that has to decide
/// *whether to compile*. Every fact folded here is a compilation **input**: the
/// component schemas, the semantic subjects, the interface, the selected
/// providers, each variant's complete program identity, its guards, routing,
/// ABI, launch contracts, declared target requirements and deferred predicates,
/// and — through each [`BackendPayloadDescriptor`]'s digest — that payload's
/// source, flags, resolved toolchain, entry mappings, and recorded obligations.
/// No output of a backend compiler is among them.
///
/// **Inference.** These bytes are therefore derivable before the backend
/// compiler runs, given only the metadata the caller must already hold to
/// invoke it, and
/// [`ArtifactProgramBuilder::push_pending_payload`](super::ArtifactProgramBuilder::push_pending_payload)
/// is the constructor that reaches them without an object. An expansion cache
/// needs its key on a *miss*; this value answers that, and it is the same value
/// the compiled artifact carries rather than a cheaper stand-in kept in
/// agreement with it. There is one identity authority over the artifact
/// subject, which is what ADR 0082 requires and what a second "pre-compilation"
/// encoding would have broken.
///
/// **What it is not evidence of.** It names the artifact and says nothing about
/// whether the compilation succeeded, what the compiler emitted, or whether any
/// object with these bytes exists. An artifact assembled from pending payloads
/// has this identity and carries no code; a published envelope additionally
/// carries the object under its own section digest, which is integrity rather
/// than identity. Equal identity implies equal bytes for the identity-bearing
/// part of an envelope and deliberately not for its object sections.
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

    /// Returns what this binding slot addresses.
    ///
    /// The reference the artifact carries, not a walk into the packaged
    /// program: [`Self::value`] resolves through the `VerifiedKernelProgram`
    /// and is therefore unavailable to anything holding only encoded bytes,
    /// which is exactly why this is encoded.
    ///
    /// It is nonetheless *derived* rather than declared. The builder reads it
    /// from the packaged program's own stage access, the same way it reads this
    /// binding's element type, address space, access mode, and alignment, so a
    /// producer cannot assert a correspondence its plan contradicts.
    #[must_use]
    pub fn target(self) -> BindingTarget<'a> {
        match &self.data().target {
            BindingTargetData::ProgramInput(key) => BindingTarget::ProgramInput(key),
            BindingTargetData::ProgramOutput(keys) => BindingTarget::ProgramOutput(keys),
            BindingTargetData::Internal => BindingTarget::Internal,
        }
    }

    /// Returns the program role of the bound value.
    ///
    /// Derived from [`Self::target`] rather than stored beside it: the two say
    /// one thing, and a stored copy is a second authority that can drift.
    #[must_use]
    pub fn value_role(self) -> ValueRole {
        self.data().target.value_role()
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
    // Each coverage entry is one `SemanticOccurrence`, a `u32` ordinal.
    let exact = STAGE_KEY_DOMAIN.len()
        + framed(stage.kernel().canonical_identity().as_bytes().len())
        + LENGTH_BYTES
        + stage.coverage().len() * size_of::<u32>();
    let mut bytes = Vec::with_capacity(exact);
    bytes.extend_from_slice(STAGE_KEY_DOMAIN);
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());
    push_len(&mut bytes, stage.coverage().len());
    for occurrence in stage.coverage() {
        bytes.extend_from_slice(&occurrence.get().to_be_bytes());
    }
    debug_assert_eq!(bytes.len(), exact, "stage key capacity is exact");
    bytes
}

// Each shared-IR vocabulary below has exactly one tag table, written as an
// adjacent forward and inverse pair. Two tables that agreed only by inspection
// would let an envelope decode into a plausible-but-wrong program, so the pair
// is kept in one place and pinned by an exhaustive round-trip test.

/// Encodes one shared-IR element type.
///
/// Infallible, and that is the point of the vocabulary not being
/// `#[non_exhaustive]`. This is a cross-crate total map into artifact identity,
/// so widening `KernelType` must stop the build *here*, at the encoder that has
/// to decide what the new variant's tag is. It previously rejected an
/// unrecognized variant at run time, which was sound and strictly weaker: a
/// widened enum would have silently made previously packageable artifacts
/// unpackageable instead of failing the build at the site that must decide.
pub(super) const fn element_type_tag(element_type: KernelType) -> u8 {
    match element_type {
        KernelType::Bool => 0x01,
        KernelType::Index => 0x02,
        KernelType::F32 => 0x03,
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

pub(super) const fn address_space_tag(address_space: AddressSpace) -> u8 {
    match address_space {
        AddressSpace::Device => 0x01,
        AddressSpace::Workgroup => 0x02,
        AddressSpace::InvocationPrivate => 0x03,
        AddressSpace::Constant => 0x04,
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

pub(super) const fn buffer_access_tag(access: BufferAccess) -> u8 {
    match access {
        BufferAccess::Read => 0x01,
        BufferAccess::Write => 0x02,
    }
}

pub(super) const fn buffer_access_from_tag(tag: u8) -> Option<BufferAccess> {
    match tag {
        0x01 => Some(BufferAccess::Read),
        0x02 => Some(BufferAccess::Write),
        _ => None,
    }
}

/// Writes one binding's interface reference into a canonical byte run.
///
/// Shared by the identity encoder and the envelope encoder so the two cannot
/// spell one reference two ways. The match is exhaustive with no wildcard arm
/// (ADR 0074 convention 3), so a widened target vocabulary stops the build here
/// rather than writing bytes a reader would misparse.
pub(super) fn push_binding_target(bytes: &mut Vec<u8>, target: &BindingTargetData) {
    match target {
        BindingTargetData::ProgramInput(key) => {
            bytes.push(BINDING_TARGET_PROGRAM_INPUT);
            push_slice(bytes, key.as_str().as_bytes());
        }
        BindingTargetData::ProgramOutput(keys) => {
            bytes.push(BINDING_TARGET_PROGRAM_OUTPUT);
            push_len(bytes, keys.len());
            for key in keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
        }
        BindingTargetData::Internal => bytes.push(BINDING_TARGET_INTERNAL),
    }
}

/// Why one packaged stage must precede another.
///
/// The shared IR's [`DependencyReasonView`](tiler_ir::program::DependencyReasonView)
/// narrowed to what an envelope can carry. The IR's arms name the *value* read
/// or the *allocation* reused; neither has a durable name at this layer — see
/// [`BindingTarget`]'s own account of why a program value is unnameable here —
/// so the envelope carries the obligation's kind and not its subject.
///
/// **That is a real loss and it is bounded.** A consumer learns that this edge
/// is a read-after-write rather than a storage reuse, which is what decides
/// whether reordering is a correctness violation or a liveness one. It does not
/// learn which buffer, and it does not need to: the bindings it already decodes
/// say what each entry addresses.
///
/// Deliberately not `#[non_exhaustive]`, matching every other governed
/// vocabulary this codec matches totally: widening it must break the build at
/// the encoder rather than silently encode one reason as another.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StageDependencyReason {
    /// The successor reads a value the predecessor fully initializes.
    Data,
    /// The successor reuses storage whose previous value the predecessor released.
    StorageHandoff,
}

impl StageDependencyReason {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Data => 0x01,
            Self::StorageHandoff => 0x02,
        }
    }

    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Data),
            0x02 => Some(Self::StorageHandoff),
            _ => None,
        }
    }
}

/// One ordering obligation between two entries of a variant.
///
/// Both positions index the variant's canonical entry table. Derived from the
/// packaged program's own `dependencies()`, never stated by a producer, so an
/// artifact cannot claim an ordering its plan does not prove.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StageDependencyData {
    pub(crate) predecessor: u32,
    pub(crate) successor: u32,
    pub(crate) reason: StageDependencyReason,
}

/// Governed wire tag of a binding addressing a named program input.
pub(super) const BINDING_TARGET_PROGRAM_INPUT: u8 = 0x01;
/// Governed wire tag of a binding addressing named program output storage.
pub(super) const BINDING_TARGET_PROGRAM_OUTPUT: u8 = 0x02;
/// Governed wire tag of a binding addressing program-internal storage.
pub(super) const BINDING_TARGET_INTERNAL: u8 = 0x03;

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
/// or [`ArtifactDiagnostic::IdentityLimit`] when the encoding exceeds its
/// bound.
///
/// It no longer refuses an unrecognized shared-IR variant, because there can no
/// longer be one: `KernelType`, `AddressSpace`, and `BufferAccess` are not
/// `#[non_exhaustive]`, so widening any of them stops the build at the tag
/// tables below rather than compiling into a run-time rejection.
pub(super) fn encode_identity(
    envelope: &ArtifactEnvelope,
) -> Result<CanonicalArtifactProgramIdentity, ArtifactDiagnostic> {
    let orders: Vec<VariantOrder> = envelope
        .variants()
        .iter()
        .map(|variant| variant_order(envelope.expressions(), variant))
        .collect();
    let arena = canonical_arena_traversal(
        envelope.expressions(),
        identity_use_sites(envelope, &orders),
    );
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
    push_interface(&mut bytes, envelope);
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
    // The arena is written once, here, and every reference below is a canonical
    // position into it. That is what makes the identity linear in arena size:
    // the previous encoding embedded a node's whole subtree at every use.
    arena.encode(envelope.expressions(), &mut bytes);
    push_len(&mut bytes, envelope.variants().len());
    for (variant, order) in envelope.variants().iter().zip(&orders) {
        push_variant(&mut bytes, envelope, &arena, variant, order, &payload_keys)?;
    }
    if bytes.len() > MAX_ARTIFACT_IDENTITY_BYTES {
        return Err(ArtifactDiagnostic::IdentityLimit {
            bytes: bytes.len(),
            limit: MAX_ARTIFACT_IDENTITY_BYTES,
        });
    }
    Ok(CanonicalArtifactProgramIdentity(bytes))
}

/// The canonical order of one variant's deferred predicates, as positions.
///
/// Shared by the identity encoder, the codec that stores them, and the
/// validator that re-checks the stored order, so the three cannot drift into
/// three definitions of "canonical".
///
/// The expression leads, then the facts that make two predicates over one
/// expression distinct — a predicate is not determined by its expression alone,
/// so ordering by that alone would leave ties and the order would depend on the
/// input permutation.
pub(super) fn canonical_deferred_order(
    nodes: &[ExprNode],
    deferred: &[DeferredPredicateData],
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..deferred.len()).collect();
    order.sort_by(|left, right| {
        let (left, right) = (&deferred[*left], &deferred[*right]);
        compare_expr_nodes(nodes, left.predicate, right.predicate)
            .then_with(|| left.phase.tag().cmp(&right.phase.tag()))
            .then_with(|| left.authority.namespace().cmp(right.authority.namespace()))
            .then_with(|| left.authority.name().cmp(right.authority.name()))
            .then_with(|| left.authority.revision().cmp(&right.authority.revision()))
    });
    order
}

/// The canonical order of one entry's launch preconditions.
pub(super) fn canonical_precondition_order(nodes: &[ExprNode], preconditions: &[u32]) -> Vec<u32> {
    let mut ordered = preconditions.to_vec();
    ordered.sort_by(|left, right| compare_expr_nodes(nodes, *left, *right));
    ordered
}

/// A variant's expression-bearing sets, put in content-derived order once.
///
/// Neither set is canonicalized when a variant is built — `check_deferred` and
/// the launch-precondition loop preserve the caller's declaration order and
/// only reject duplicates — so this is where their order stops depending on
/// which obligation a producer happened to enumerate first.
///
/// **It is computed before the arena is numbered, and that is the point.** The
/// numbering is a function of the root order, the root order is the order these
/// sets are written in, and so an order derived from canonical IDs would be
/// circular. [`compare_expr_nodes`] needs no numbering, which is what breaks
/// the cycle.
struct VariantOrder {
    /// Deferred predicate positions, in canonical order.
    deferred: Vec<usize>,
    /// Per entry, its launch-precondition arena nodes in canonical order.
    preconditions: Vec<Vec<u32>>,
}

/// Orders one variant's deferred predicates and launch preconditions.
fn variant_order(nodes: &[ExprNode], variant: &VariantRow) -> VariantOrder {
    let deferred = canonical_deferred_order(nodes, &variant.deferred);
    let preconditions = variant
        .entries
        .iter()
        .map(|entry| canonical_precondition_order(nodes, &entry.launch.preconditions))
        .collect();
    VariantOrder {
        deferred,
        preconditions,
    }
}

/// Every arena node the identity names, in the order the identity writes it.
///
/// The order is the numbering, so this must stay in step with `push_variant`
/// and `push_entry`. It is one function rather than a walk beside each writer
/// because a root list that disagreed with the write order would still produce
/// a *valid* identity — just not the one a decoder re-deriving it would get.
fn identity_use_sites(envelope: &ArtifactEnvelope, orders: &[VariantOrder]) -> Vec<u32> {
    let mut sites = Vec::new();
    for (variant, order) in envelope.variants().iter().zip(orders) {
        sites.push(variant.guard);
        sites.extend(
            order
                .deferred
                .iter()
                .map(|index| variant.deferred[*index].predicate),
        );
        for (entry, preconditions) in variant.entries.iter().zip(&order.preconditions) {
            sites.extend(
                entry
                    .bindings
                    .iter()
                    .map(|binding| binding.accessible_bytes),
            );
            sites.push(entry.launch.grid_threads);
            sites.push(entry.launch.threads_per_workgroup);
            sites.extend(preconditions.iter().copied());
        }
    }
    // Every remaining arena position, in arena order, so the numbering is total.
    //
    // **This is not reachable for a valid artifact and is not a fallback that
    // hides one.** Validation requires every expression to be reached by a use
    // site, so for anything that decodes these add nothing and the numbering is
    // exactly the use-site one. They exist because this encoder runs *before*
    // validation — `an_expression_no_use_site_reaches_is_rejected` and
    // `an_empty_portfolio_is_rejected` both derive an identity for an envelope
    // that is about to be refused — and naming an unreached node would
    // otherwise panic inside the traversal instead of letting the typed
    // rejection be returned. `tiler-ir` needs no such tail because verification
    // precedes identity there.
    sites.extend(0..ordinal(envelope.expressions().len()));
    sites
}

/// Names one arena node by its canonical position, at fixed width.
///
/// Eight big-endian bytes are self-delimiting, so this replaces a
/// length-prefixed key without losing framing.
fn push_abi_reference(bytes: &mut Vec<u8>, arena: &AbiArenaTraversal, node: u32) {
    bytes.extend_from_slice(&arena.canonical_id(node).to_be_bytes());
}

fn push_interface(bytes: &mut Vec<u8>, envelope: &ArtifactEnvelope) {
    push_len(bytes, envelope.inputs().len());
    for input in envelope.inputs() {
        push_slice(bytes, input.key.as_str().as_bytes());
        push_shape(bytes, &input.shape);
        bytes.push(element_type_tag(input.element_type));
    }
    push_len(bytes, envelope.outputs().len());
    for output in envelope.outputs() {
        push_slice(bytes, output.key.as_str().as_bytes());
        push_shape(bytes, &output.shape);
        bytes.push(element_type_tag(output.element_type));
    }
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
    arena: &AbiArenaTraversal,
    variant: &VariantRow,
    order: &VariantOrder,
    payload_keys: &[Vec<u8>],
) -> Result<(), ArtifactDiagnostic> {
    push_slice(
        bytes,
        &envelope.sections()[node_at(variant.program_section)].bytes,
    );
    push_abi_reference(bytes, arena, variant.guard);
    push_slice(bytes, variant.profile.key.as_str().as_bytes());
    push_slice(bytes, variant.profile.descriptor.as_bytes());
    push_slice(bytes, variant.feasibility_rules.key.as_str().as_bytes());
    bytes.extend_from_slice(&variant.feasibility_rules.revision.to_be_bytes());
    // Already in canonical order, so this writes rather than sorts: the order
    // was fixed before the arena was numbered, because the numbering depends on
    // it. Distinctness is still proven, since equal keys would make the order
    // unrecoverable rather than merely arbitrary.
    let deferred: Vec<Vec<u8>> = order
        .deferred
        .iter()
        .map(|index| deferred_key(arena, &variant.deferred[*index]))
        .collect();
    if deferred.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ArtifactDiagnostic::AmbiguousCanonicalKey {
            entity: ArtifactEntityKind::Variant,
        });
    }
    push_len(bytes, deferred.len());
    for key in &deferred {
        push_slice(bytes, key);
    }
    push_len(bytes, variant.entries.len());
    for (entry, preconditions) in variant.entries.iter().zip(&order.preconditions) {
        push_slice(bytes, entry.stage.as_bytes());
        push_entry(bytes, arena, entry, preconditions, payload_keys)?;
    }
    // The order is meaning, so it is folded as stated rather than sorted: two
    // artifacts whose stages run in different orders are different artifacts.
    // The edges are a set and are folded in canonical order, because which
    // obligation the producer happened to enumerate first is not.
    push_len(bytes, variant.execution_order.len());
    for entry in &variant.execution_order {
        bytes.extend_from_slice(&entry.to_be_bytes());
    }
    push_len(bytes, variant.dependencies.len());
    for edge in &variant.dependencies {
        let StageDependencyData {
            predecessor,
            successor,
            reason,
        } = edge;
        bytes.extend_from_slice(&predecessor.to_be_bytes());
        bytes.extend_from_slice(&successor.to_be_bytes());
        bytes.push(reason.tag());
    }
    Ok(())
}

/// Derives the canonical content key of one deferred feasibility predicate.
pub(super) fn deferred_key(
    arena: &AbiArenaTraversal,
    predicate: &DeferredPredicateData,
) -> Vec<u8> {
    let exact = DEFERRED_KEY_DOMAIN.len()
        + size_of::<u64>()
        + 1
        + framed(predicate.authority.namespace().len())
        + framed(predicate.authority.name().len())
        + size_of::<u32>();
    let mut bytes = Vec::with_capacity(exact);
    bytes.extend_from_slice(DEFERRED_KEY_DOMAIN);
    push_abi_reference(&mut bytes, arena, predicate.predicate);
    bytes.push(predicate.phase.tag());
    push_slice(&mut bytes, predicate.authority.namespace().as_bytes());
    push_slice(&mut bytes, predicate.authority.name().as_bytes());
    bytes.extend_from_slice(&predicate.authority.revision().to_be_bytes());
    debug_assert_eq!(bytes.len(), exact, "deferred key capacity is exact");
    bytes
}

fn push_entry(
    bytes: &mut Vec<u8>,
    arena: &AbiArenaTraversal,
    entry: &EntryRow,
    preconditions: &[u32],
    payload_keys: &[Vec<u8>],
) -> Result<(), ArtifactDiagnostic> {
    push_resources(bytes, entry.resources);
    push_numerical(bytes, &entry.numerical);
    push_len(bytes, entry.bindings.len());
    for binding in &entry.bindings {
        bytes.push(binding.kind.tag());
        bytes.push(element_type_tag(binding.element_type));
        bytes.push(address_space_tag(binding.address_space));
        bytes.push(buffer_access_tag(binding.access));
        bytes.extend_from_slice(&binding.alignment.to_be_bytes());
        push_binding_target(bytes, &binding.target);
        push_abi_reference(bytes, arena, binding.accessible_bytes);
    }
    push_abi_reference(bytes, arena, entry.launch.grid_threads);
    push_abi_reference(bytes, arena, entry.launch.threads_per_workgroup);
    bytes.push(u8::from(entry.launch.zero_work_skips_dispatch));
    // Canonically ordered before the numbering, as the deferred set is.
    // Distinctness is the builder's, which rejects a duplicate precondition, so
    // two equal canonical IDs here would be a builder defect rather than input.
    if preconditions
        .windows(2)
        .any(|pair| arena.canonical_id(pair[0]) == arena.canonical_id(pair[1]))
    {
        return Err(ArtifactDiagnostic::AmbiguousCanonicalKey {
            entity: ArtifactEntityKind::Expression,
        });
    }
    push_len(bytes, preconditions.len());
    for node in preconditions {
        push_abi_reference(bytes, arena, *node);
    }
    push_slice(bytes, &payload_keys[node_at(entry.payload)]);
    push_slice(bytes, entry.entry_key.as_bytes());
    Ok(())
}
