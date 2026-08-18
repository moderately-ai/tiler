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

use std::sync::Arc;

use tiler_ir::kernel::{AddressSpace, BufferAccess, CanonicalKernelIdentity, KernelType};
use tiler_ir::program::abi::{PreparedEntryTargetRequirement, TargetPropertyRequirementRelation};
use tiler_ir::program::{
    AlignmentRequirement, ByteWindow, MaterializedValueRef, StageRef, StorageEncoding,
    StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, IndexArithmetic,
    MemoryOrdering, NumericalPermission, NumericalRealization, ResourceRequirements,
    SubgroupRealizationSubject, SubgroupTransfer, SubnormalMode, SynchronizationKind,
    SynchronizationScope, SynchronizationSubject, ValueDomainProvenance,
};
use tiler_ir::semantic::{
    EncodedComponentRole, InputKey, OpKey, OutputKey, ProviderIdentity,
    SemanticAdmissionProvenanceIdentity, SemanticDefinitionProjectionIdentity,
    SemanticGraphIdentity, SemanticIdentity,
};
use tiler_ir::shape::{Axis, Shape};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::{AbiArenaTraversal, canonical_arena_traversal, compare_expr_nodes};

use super::MAX_ARTIFACT_IDENTITY_BYTES;
use super::codec::{
    ArtifactEnvelope, DecodedExtentOperand, EntryRow, NumericalFacts, PayloadContent, VariantRow,
    canonical_entry_positions, position as node_at,
};
use super::error::{ArtifactDiagnostic, ArtifactEntityKind, RecordedArtifactIdentityError};
use super::expr::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue, ExprNode,
    evaluate,
};
use super::handles::PayloadId;
use super::keys::{
    BackendEntryKey, BackendKey, CapabilityFamilyKey, FeasibilityRuleSetRef, PayloadDigest,
    RepresentationKey, TargetProfileRef,
};
use super::realization::DeliveredRealizationRecord;
use super::requirement::{RouteRequirement, push_requirements};

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
///
/// # Why this is a `v6` step
///
/// A variant no longer carries its own applicability guard, launch geometry, or
/// per-binding accessible ranges: they are derived from the bound program. The
/// identity folds the program's expressions where it used to fold a caller's
/// restatement, so two artifacts that differed only because one producer wrote
/// `UnsignedLiteral(24)` where another wrote `rows * columns * 4` over the same
/// program are now one artifact — a change in what the identity means, not only
/// in how it is spelled.
///
/// # Why this is a `v7` step
///
/// Raised to `v7` when each ABI binding gained the accessible *offset* beside
/// its extent, so a slot addressing part of a value states where in it the range
/// starts. The same argument as `v2`: the field landed inside the per-binding
/// record, and a `v6` encoding of one artifact could otherwise equal a `v7`
/// encoding of another. Every artifact's identity bytes move at this step, which
/// is the intended consequence — a `v6` identity described a record that could
/// not express a placement, so an artifact carrying one is not the same subject
/// as the artifact carrying its `v7` restatement.
///
/// # Why this is a `v8` step
///
/// Raised to `v8` when the artifact stopped dropping four consumable numerical
/// dimensions: permutation, signed-zero elimination, NaN absence, and infinity
/// absence. Those fields landed inside both the resource-requirement and
/// numerical-realization records, so a `v7` encoding of one artifact could
/// otherwise equal a `v8` encoding of another. Retagging makes the old,
/// incomplete subject incomparable with the complete one.
///
/// # Why this is a `v9` step
///
/// Raised to `v9` when interface entries became logical values with ordered
/// producer-derived components and each binding gained its semantic component
/// role, physical storage scalar, complete storage encoding, and kernel access
/// type. A `v8` identity cannot distinguish schemes or physical encodings that
/// require different runtime bindings.
///
/// # Why this is a `v10` step
///
/// Raised to `v10` when the resource-requirement record stopped encoding a
/// numeric barrier count. The count was not a valid synchronization capability:
/// it carried no operation kind, participants, visibility, ordering, placement,
/// or convergence proof. Removing its four bytes changes the framing of every
/// following entry field, so the old and corrected subjects need incomparable
/// domains rather than two interpretations of one byte string.
///
/// # Why this is a `v11` step
///
/// A deferred obligation now names the exact prepared entry whose target
/// property must be observed and carries the complete query contract. The old
/// record named only a phase and a compile-time capability provider, so two
/// prepared entries sharing a property key could be answered by one global
/// value. The new subject is deliberately incomparable with that incomplete
/// route.
///
/// # Why this is a `v12` step
///
/// A variant now carries the additional requirements its selected route places
/// on a live device: neutral quantitative rows and backend-scoped qualitative
/// rows. The rows land inside the repeated variant record, so a `v11` encoding
/// of one artifact could otherwise equal a `v12` encoding of another — and the
/// subjects genuinely differ, because a `v11` identity described a route that
/// could not state a device precondition at all. Two artifacts alike in every
/// other respect but requiring different device capabilities are two artifacts,
/// and the domain step is what stops them comparing equal.
///
/// # Why this is a `v13` step
///
/// An executable entry is no longer realized by one backend payload but by one
/// payload per **delivery position** — the ordered slot a consumer's build
/// target resolves to. The entry record now writes a counted run of payload keys
/// where it wrote exactly one, so a `v12` encoding of one artifact could
/// otherwise equal a `v13` encoding of another, and the subjects genuinely
/// differ: a `v12` identity described an artifact that could carry only one
/// backend object per entry, so a two-position artifact and the one-position
/// artifact holding its first object are not the same artifact and must not
/// share bytes.
///
/// The count is folded rather than assumed, which is what makes that true. A
/// one-position artifact writes `1` and one key; a two-position artifact writes
/// `2` and two. Dropping one family from a selection therefore moves every
/// entry's bytes rather than only removing a payload from the sorted table, so a
/// cache holding the two-family artifact misses for the one-family one instead
/// of matching a subject that carries different objects.
///
/// # Why this is a `v14` step
///
/// An executable entry's fixed resource record now states the synchronization
/// realization its schedule requires, or states that it requires none. The field
/// lands inside a record every entry writes, ahead of its numerical fields, so a
/// `v13` encoding of one artifact could otherwise equal a `v14` encoding of
/// another. The subjects genuinely differ: a `v13` identity described an entry
/// that could not state a synchronization obligation at all, so an entry that
/// performs a fenced staged handoff and an entry that performs none were
/// indistinguishable in these bytes. They are different programs against
/// different target authorities, and a cache holding one must miss for the
/// other.
///
/// The **absence** is folded too, and that is the load-bearing half. Writing
/// nothing for an entry that synchronizes nothing would leave "no requirement"
/// recoverable from bytes that never stated it, so a later entry that gained one
/// could share identity with the one that had none.
///
/// # Why proof-bound stage coverage is not a `v15` step
///
/// Binding each covered occurrence to its refinement evidence moved
/// [`STAGE_KEY_DOMAIN`] to `v3` and moved this identity's *value* for every
/// artifact ever minted — a cache holding the old bytes misses, which is the
/// outcome a step exists to produce. It does not move this *domain*, and the
/// reason is per-tag injectivity at the one site the new bytes reach:
/// [`push_variant`] writes each entry's stage subject with `push_slice`, so the
/// complete stepped key including its own separator arrives length-framed. No
/// `v14` encoding of one artifact can equal a `v14` encoding of another across
/// the change, because the framed run they differ in is delimited and its first
/// bytes are the separator that names its grammar. This is the same reasoning
/// the canonical-coverage step recorded when it moved the stage key from `v1`
/// to `v2` and held this domain, and `docs/artifact-abi.md` carries it as the
/// artifact ledger's own rule rather than as a one-off.
///
/// # Why this is a `v16` step
///
/// Raised to `v16` when every entry's derived dispatch record gained its
/// index-arithmetic requirement. The tag lands *inside* the fixed
/// resource-requirement run, between the device-memory flag and the
/// synchronization record, so a `v15` reader handed `v16` bytes would read the
/// index-arithmetic tag as the synchronization presence byte and lose framing
/// for the rest of the entry. The step is required for meaning as well as
/// framing: the requirement is a fact about the verified program that no `v15`
/// field carried, so a consumer holding `v15` bytes could not reconstruct it —
/// the envelope carries no KIR operations to re-derive it from — and a cache
/// holding a `v15` identity must miss on the complete subject rather than
/// match it.
///
/// # Why this was a `v17` step
///
/// `v17` added the retained shape environment after the existing semantic
/// subjects. Its declarations and constraints are artifact meaning even when
/// no variant guard happens to read them, so two programs differing there can
/// no longer share the earlier incomplete identity.
///
/// # Why this was a `v18` step
///
/// A selected lowering capability now frames its governed family and exact
/// operation namespace, name, and semantic version independently. The `v17`
/// row carried one delimiter-composed text field, so two legal operations whose
/// dots fell on different namespace/name boundaries could share one artifact
/// identity. The structured record removes that collision; stepping the domain
/// makes every ambiguous earlier subject incomparable with the injective one.
///
/// # Why this is a `v19` step
///
/// Both per-entry numerical records — the fixed resource-requirement run and
/// the entry's own numerical facts — gained the reciprocal-transform permission
/// and the approximate-intrinsic envelope, written between the signed-zero tag
/// and the exceptional-value assumptions in canonical dimension order. The
/// bytes land inside records the rest of the identity trails, so every artifact
/// ever encoded maps to different bytes now; and two artifacts differing only
/// in an elementary freedom were one subject under `v18`, which is the
/// collision the step refuses: a cache holding a `v18` identity must miss
/// rather than match.
///
/// # Why this was a `v15` step
///
/// Raised to `v15` when every artifact gained the required delivered-realization
/// record and this identity began folding its canonical bytes. Two artifacts
/// that deliver one numerical contract by different **means** — one honouring a
/// dimension exactly, the other only under a declared relaxation — were
/// indistinguishable in `v14` bytes, because a means is not a behaviour and no
/// `v14` field carried one. They are different artifacts to any consumer
/// comparing generated output against a reference, so a cache holding one must
/// miss for the other, and the domain step is what makes the old subject
/// incomparable with the complete one rather than merely unlikely to collide.
///
/// Crate-visible because [`RecordedArtifactProgramIdentity::from_bytes`] admits
/// bytes by `starts_with` on this separator, so a governed domain that prefixed
/// it would let another subject's bytes be accepted as an artifact identity.
/// `crate::domains` enumerates it and checks that no such domain exists.
pub(crate) const ARTIFACT_DOMAIN: &[u8] = b"tiler.artifact-program.v19\0";

/// [`ARTIFACT_DOMAIN`] without its terminator, for rendering in a diagnostic.
///
/// Derived from the separator rather than written a second time. A domain bump
/// is a one-line edit above, and an error message that named the previous
/// version would be a defect no test could be expected to notice.
pub(super) const ARTIFACT_DOMAIN_LABEL: &str = {
    let (label, terminator) = ARTIFACT_DOMAIN.split_at(ARTIFACT_DOMAIN.len() - 1);
    assert!(
        terminator[0] == 0,
        "the artifact domain separator ends with its NUL terminator"
    );
    match str::from_utf8(label) {
        Ok(label) => label,
        Err(_) => panic!("the artifact domain separator is ASCII"),
    }
};

/// Versioned domain of one independently compared and serialized stage key.
///
/// `v2` changes the coverage ordinals from semantic-program storage order to
/// canonical semantic occurrence order. The payload layout is unchanged, so
/// the separator must step to prevent the same raw ordinal from retaining its
/// v1 spelling while naming another operation.
///
/// `v3` writes, after each occurrence ordinal, the length-framed reached-only
/// executable-coverage identity of the completed index-refinement receipt that
/// proved it. This is a record-layout change inside a repeated run: a `v2`
/// reader handed `v3` bytes would read the framed identity as the following
/// occurrence, so the separator steps rather than the field being appended
/// silently. This key is compared on its own and serialized into every entry
/// row, which is why it carries its own separator instead of relying on
/// [`ARTIFACT_DOMAIN`] — and why that domain does not step with it, as the note
/// there records.
///
/// `v4` replaces coverage-only ownership with one complete canonical owner:
/// realization stages write a tagged, counted run of proof-bound occurrence
/// ordinals (including split and staged continuations), while administrative
/// publishers write their exact named-output component claims. The tag and
/// count are part of the independently serialized subject, so an older reader
/// cannot determine which grammar follows the bound kernel identity. The stage
/// domain steps; [`ARTIFACT_DOMAIN`] does not, because each entry length-frames
/// this complete stepped key.
pub(crate) const STAGE_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.stage.v4\0";
/// Versioned domain separator of one carried payload descriptor's canonical key.
pub(crate) const PAYLOAD_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.payload.v1\0";
/// Versioned domain separator of one selected provider's canonical key.
///
/// `v2` for the same change that took [`ARTIFACT_DOMAIN`] to `v3`: this record's
/// trailing integer changed both width and meaning. Retagged here as well as
/// there because a provider key is also compared to its siblings on its own —
/// `encode_identity` sorts and deduplicates these keys — so the record needs to
/// be self-describing rather than relying on the enclosing domain.
///
/// `v3` replaces the delimiter-composed capability text with four independently
/// framed fields: governed family, operation namespace, operation name, and
/// semantic version. This key is sorted and deduplicated on its own, so its
/// domain steps independently with the enclosing artifact domain.
pub(crate) const PROVIDER_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.provider.v3\0";
/// Versioned domain separator of one deferred predicate's canonical key.
pub(crate) const DEFERRED_KEY_DOMAIN: &[u8] = b"tiler.artifact-program.deferred.v2\0";

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
        // `3.0` for the live-device route-requirement family. The target
        // requirement component moves independently of the manifest because its
        // governed *vocabulary* changed: a requirement is no longer only a
        // prepared-entry quantity, so a reader that understands `2.0` cannot
        // decide a `3.0` route even when it can frame the manifest around it.
        target_requirement: SchemaVersion::new(3, 0),
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
/// One value, and the vocabulary is an extension point rather than a claim
/// about what exists: [`Self::NativeImage`] says the target's own API loads
/// these bytes as they stand. A representation that is *not* loadable as it
/// stands — source a runtime would have to compile, or target IR some
/// translation step would have to turn into a loadable object — has no
/// spelling here, because nothing in Tiler delivers one and a value naming it
/// would assert a route that does not exist.
///
/// **This is not the question of whether a device does work of its own between
/// a load and a dispatch.** A metallib is a [`Self::NativeImage`] and still
/// undergoes native device translation during Metal pipeline creation. That
/// translation is a typed capability fact carrying its own availability phase,
/// authority, and provenance, and ADR 0086 disposes of it as `Unknown` on every
/// currently observable macOS host — in the capability layer, which is why that
/// decision adds no field to artifact identity. Delivery and authority are two
/// questions, and this enum answers only the first.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArtifactExecutionPolicy {
    /// The payload bytes are directly loadable on a compatible device.
    NativeImage,
}

impl ArtifactExecutionPolicy {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::NativeImage => 0x01,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized policy.
    ///
    /// Tag `0x02` named a retired `RequiresDeviceTranslation` and is **never
    /// reassigned**. It resolves to `None` like any tag this vocabulary does
    /// not define, so a stream carrying it is refused by subject and tag rather
    /// than read as some other policy.
    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::NativeImage),
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

/// One structured lowering capability at the neutral artifact boundary.
///
/// Family and operation remain separately typed and are framed independently
/// wherever this subject enters canonical bytes. There is deliberately no
/// composite text spelling.
///
/// ```compile_fail,E0277
/// use std::fmt::Display;
/// use tiler_artifact::program::LoweringCapabilitySubject;
///
/// fn requires_display<T: Display>() {}
/// requires_display::<LoweringCapabilitySubject>();
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LoweringCapabilitySubject {
    /// Governed lowering-family key.
    pub family: CapabilityFamilyKey,
    /// Exact semantic operation key.
    pub operation: OpKey,
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
    /// Exact lowering capability the provider was selected for.
    pub capability: LoweringCapabilitySubject,
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
            + framed(capability.family.as_str().len())
            + framed(capability.operation.namespace().len())
            + framed(capability.operation.name().len())
            + size_of::<u32>()
            + size_of::<u32>();
        let mut bytes = Vec::with_capacity(exact);
        bytes.extend_from_slice(PROVIDER_KEY_DOMAIN);
        push_slice(&mut bytes, provider.namespace().as_bytes());
        push_slice(&mut bytes, provider.name().as_bytes());
        bytes.extend_from_slice(&provider.revision().to_be_bytes());
        push_slice(&mut bytes, capability.family.as_str().as_bytes());
        push_slice(&mut bytes, capability.operation.namespace().as_bytes());
        push_slice(&mut bytes, capability.operation.name().as_bytes());
        bytes.extend_from_slice(&capability.operation.semantic_version().to_be_bytes());
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

/// The backend entries one executable entry is realized by, in delivery order.
///
/// # One entry, several objects
///
/// A **delivery position** is the ordered slot a consumer's build target
/// resolves to — not a device property, and not a plan alternative. Two
/// positions are one compilation, one plan, one kernel program, and two
/// compiled objects, so what varies across them is the object and never the
/// entry: [`Self::entry_key`] is stated once and every position's payload is
/// looked up by it. An artifact that wanted a different *entry* per position
/// would be describing two programs, which is a portfolio of variants rather
/// than a delivery.
///
/// [`Self::payloads`] is positional against the artifact's delivery positions:
/// position `p` of every entry names the object a consumer resolving to `p`
/// loads. The order is meaning and is retained in identity; the artifact layer
/// deliberately does not name what a position *is*, because "macOS" and "iOS"
/// are consumer-target vocabulary a target-neutral artifact must not carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendEntryRef {
    /// Payload descriptors that contain the entry, one per delivery position.
    ///
    /// Never empty, and the same length for every entry of the artifact; the
    /// builder refuses both with [`ArtifactBuildError::EmptyDelivery`] and
    /// [`ArtifactBuildError::DeliveryCardinality`].
    ///
    /// [`ArtifactBuildError::EmptyDelivery`]: super::ArtifactBuildError::EmptyDelivery
    /// [`ArtifactBuildError::DeliveryCardinality`]: super::ArtifactBuildError::DeliveryCardinality
    pub payloads: Vec<PayloadId>,
    /// Opaque backend entry key, the same within every one of those payloads.
    pub entry_key: BackendEntryKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StoredBackendEntry {
    pub(super) payloads: Vec<u32>,
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
/// reaches — has no durable name this layer can carry. A program value's builder
/// arena position is a transient implementation detail, while artifact identity
/// uses content-derived orderings where its other entity references need stable
/// names. The shared IR's own canonical *value* key is crate-private to
/// `tiler_ir::program` with no read view publishing it.
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
    /// A loader allocates it rather than binding host data. The binding's own
    /// accessible offset and extent bound what the entry reaches, so the
    /// allocation must span at least their sum; the value's total size is a
    /// program fact the envelope does not carry. It carries no name, for the
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
    pub(super) storage_scalar: StorageScalar,
    pub(super) access_type: KernelType,
    pub(super) component_role: Option<EncodedComponentRole>,
    pub(super) encoding: StorageEncoding,
    pub(super) address_space: AddressSpace,
    pub(super) access: BufferAccess,
    pub(super) alignment: AlignmentRequirement,
    pub(super) target: BindingTargetData,
    pub(super) accessible_offset: u32,
    pub(super) accessible_bytes: u32,
}

/// One live input-extent operand an executable entry declares.
///
/// The packaged spelling of a structured-kernel [`tiler_ir::kernel::InputExtentParameter`].
/// The live *value* is not stored; only the program-interface root is. Runtime
/// binds the value from the same [`super::AbiFacts`] used for range and launch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtentOperandData {
    pub(super) key: InputKey,
    pub(super) axis: Axis,
    pub(super) value_type: AbiType,
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
    pub(super) input_extents: Vec<ExtentOperandData>,
    pub(super) launch: LaunchData,
    pub(super) implementation: StoredBackendEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DeferredPredicateData {
    pub(super) predicate: u32,
    pub(super) requirement: PreparedEntryTargetRequirement,
    pub(super) entry: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct VariantData {
    pub(super) program: VerifiedKernelProgram,
    pub(super) guard: u32,
    pub(super) profile: TargetProfileRef,
    pub(super) feasibility_rules: FeasibilityRuleSetRef,
    pub(super) deferred: Vec<DeferredPredicateData>,
    /// Additional requirements this variant's route places on a live device.
    ///
    /// Held in declaration order and canonicalized once, where the envelope is
    /// projected, exactly as the deferred set is.
    pub(super) route_requirements: Vec<RouteRequirement>,
    pub(super) entries: Vec<EntryData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InterfaceEntryData<K> {
    pub(super) key: K,
    pub(super) shape: Shape,
    pub(super) logical_type: Vec<u8>,
    pub(super) components: Vec<InterfaceComponentData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InterfaceComponentData {
    pub(super) role: Option<EncodedComponentRole>,
    pub(super) shape: Shape,
    pub(super) resolved_type: Option<Vec<u8>>,
    pub(super) storage_scalar: StorageScalar,
    pub(super) access_type: KernelType,
    pub(super) encoding: StorageEncoding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ArtifactProgramData {
    pub(super) schema: ArtifactSchema,
    pub(super) semantic: SemanticIdentity,
    pub(super) retained: super::retained::RetainedShapeEnvironment,
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
    pub(super) expression_types: Vec<AbiType>,
    pub(super) variants: Vec<VariantData>,
    /// The numerical realization this artifact delivered.
    ///
    /// Not an `Option`: every executable artifact rests on declared honouring
    /// means, so the absence a migration would need is refused at
    /// [`super::ArtifactProgramBuilder::build`] rather than carried here as a
    /// third state every reader would have to rediscover.
    ///
    /// Entry bindings are in the **flat canonical** packaged-entry space —
    /// `build` remaps the producer's declared ordinals once, so this record, the
    /// envelope's, and a decoded artifact's are one value rather than three
    /// spellings of it.
    pub(super) realization: DeliveredRealizationRecord,
}

/// Maps each declared packaged-entry ordinal to its canonical position.
///
/// Flat over (variant, entry). Variant order is routing priority and is
/// retained, so only entry positions within a variant move, and the rule that
/// moves them is [`canonical_entry_positions`]'s rather than a second definition
/// of it.
pub(super) fn packaged_entry_positions(variants: &[VariantData]) -> Vec<u32> {
    let mut flat = Vec::new();
    let mut base = 0_u32;
    for variant in variants {
        let stage_keys: Vec<Vec<u8>> = variant
            .program
            .stages()
            .map(|stage| stage_key(&variant.program, stage))
            .collect();
        let entry_of = canonical_entry_positions(&stage_keys);
        flat.extend(entry_of.iter().map(|canonical| base + canonical));
        base += ordinal(entry_of.len());
    }
    flat
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
/// never used (ADR 0072). **Transient ordinals** never enter it: expressions are
/// written once in canonical arena order and referenced by canonical position;
/// providers and payloads are ordered by canonical key, payload references carry
/// that key, and an entry carries its stage's canonical key. Two structurally
/// equal artifacts assembled in different builder or program-local stage orders
/// therefore share bytes. Variant order is the one retained order, because
/// routing priority is meaning rather than insertion. And **emitted backend
/// object bytes** never enter it: a payload is named by the digest of its
/// compilation subject, so a non-reproducible linker does not change what
/// artifact this is.
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

/// An artifact identity a consumer *recorded*, stated as the one it expects.
///
/// [`CanonicalArtifactProgramIdentity`] can only be held by code that built or
/// decoded an artifact, because this crate's encoder is its only constructor.
/// That rule is right — nobody should mint an artifact identity — and it leaves
/// the cold-consumer case unrepresentable: a producer writes the identity beside
/// the cached bytes, a separate process reads it back, and what it holds is a
/// byte string rather than a derivation. This type is that byte string, named.
///
/// # It is an assertion, not evidence
///
/// Read this before treating one as proof of anything. The two types are
/// deliberately distinct and deliberately not convertible: an encoder-derived
/// identity was computed from content this crate validated, and a recorded one
/// is what somebody wrote down. Equal bytes mean the same thing in both cases —
/// the comparison is byte equality either way — but the *warrant* behind the
/// bytes is not the same. A runtime's program-mismatch rejection carries the two
/// sides as two types for exactly this reason, and no conversion between them is
/// offered in either direction.
///
/// So a consumer that recorded the wrong bytes gets a mismatch it can act on,
/// and a consumer that recorded bytes re-read from the very artifact it is about
/// to load has checked nothing. Neither this type nor any check below can tell
/// those apart; only the provenance of the recording can.
///
/// # What the constructor does prove
///
/// [`Self::from_bytes`] rejects empty input, input above
/// [`MAX_ARTIFACT_IDENTITY_BYTES`], and bytes whose leading frame is not the
/// current artifact-identity domain separator. The last is syntax and type
/// separation: it distinguishes an artifact identity from a kernel identity, a
/// content digest, a cache key, or an identity recorded under a superseded
/// domain — all of which are byte strings a caller might plausibly have to hand.
/// It is not a claim that the remainder is canonical, that it decodes, or that
/// any artifact with these bytes exists.
///
/// Storage is shared and immutable, because the governed bound is 64 MiB and a
/// retry loop or a mismatch error should not copy that.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordedArtifactProgramIdentity(Arc<[u8]>);

impl RecordedArtifactProgramIdentity {
    /// States recorded bytes as the artifact identity a consumer expects.
    ///
    /// # Errors
    ///
    /// Returns [`RecordedArtifactIdentityError::Empty`] for empty bytes,
    /// [`RecordedArtifactIdentityError::TooLong`] above
    /// [`MAX_ARTIFACT_IDENTITY_BYTES`], or
    /// [`RecordedArtifactIdentityError::ForeignDomain`] when the leading frame
    /// is not this build's artifact-identity domain separator.
    pub fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, RecordedArtifactIdentityError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(RecordedArtifactIdentityError::Empty);
        }
        // Bounded before the domain frame is inspected, so a hostile length is
        // refused by the cheap check rather than after reading into it.
        if value.len() > MAX_ARTIFACT_IDENTITY_BYTES {
            return Err(RecordedArtifactIdentityError::TooLong {
                bytes: value.len(),
                limit: MAX_ARTIFACT_IDENTITY_BYTES,
            });
        }
        if !value.starts_with(ARTIFACT_DOMAIN) {
            return Err(RecordedArtifactIdentityError::ForeignDomain { bytes: value.len() });
        }
        Ok(Self(Arc::from(value)))
    }

    /// Returns the recorded identity bytes.
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

    /// Returns the numerical realization this artifact delivered.
    ///
    /// A **total** reader: there is no absent state to rediscover, because the
    /// builder refuses a draft that declared no record. A consumer comparing
    /// generated output against a CPU reference reads the honouring means here
    /// rather than reconstructing it from the request, the selected compiler
    /// flags, or the target's name — ADR 0076 item 4's measurement is that every
    /// such proxy can state the opposite of the truth.
    ///
    /// Entry bindings name the flat canonical packaged-entry ordinal: variants
    /// in routing priority order, and within each variant its entries in the
    /// canonical stage-key order [`VariantRef::entries`] reports.
    #[must_use]
    pub const fn delivered_realization(&self) -> &DeliveredRealizationRecord {
        &self.data.realization
    }

    /// Returns how many delivery positions this artifact carries a payload for.
    ///
    /// One for the ordinary single-target artifact. A consumer resolves exactly
    /// one of these positions from its own build target, and every executable
    /// entry names one payload at each — see [`BackendEntryRef`] for what a
    /// position is and what it deliberately is not.
    ///
    /// Derived from the entry table rather than stored beside it, because the
    /// two would be one fact written twice and a stored copy could disagree with
    /// the realizations it describes. The builder refuses an entry that does not
    /// agree, so every entry answers the same number.
    #[must_use]
    pub fn delivery_positions(&self) -> usize {
        delivery_positions(&self.data)
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

    /// Returns the canonical logical resolved-type encoding.
    #[must_use]
    pub fn resolved_type_encoding(self) -> &'a [u8] {
        &self.artifact.data.inputs[self.input].logical_type
    }

    /// Returns the producer-owned physical components in semantic order.
    #[must_use]
    pub fn components(self) -> impl ExactSizeIterator<Item = InterfaceComponentRef<'a>> {
        self.artifact.data.inputs[self.input]
            .components
            .iter()
            .map(InterfaceComponentRef)
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

    /// Returns the canonical logical resolved-type encoding.
    #[must_use]
    pub fn resolved_type_encoding(self) -> &'a [u8] {
        &self.artifact.data.outputs[self.output].logical_type
    }

    /// Returns the producer-owned physical components in semantic order.
    #[must_use]
    pub fn components(self) -> impl ExactSizeIterator<Item = InterfaceComponentRef<'a>> {
        self.artifact.data.outputs[self.output]
            .components
            .iter()
            .map(InterfaceComponentRef)
    }
}

/// One physical component of a logical artifact interface value.
#[derive(Clone, Copy, Debug)]
pub struct InterfaceComponentRef<'a>(&'a InterfaceComponentData);

impl<'a> InterfaceComponentRef<'a> {
    /// Returns the stable semantic role, or `None` for a dense singleton.
    #[must_use]
    pub fn role(self) -> Option<EncodedComponentRole> {
        self.0.role
    }

    /// Returns the physical tensor shape.
    #[must_use]
    pub fn shape(self) -> &'a Shape {
        &self.0.shape
    }

    /// Returns the semantic component type encoding, absent for a dense singleton.
    #[must_use]
    pub fn resolved_type_encoding(self) -> Option<&'a [u8]> {
        self.0.resolved_type.as_deref()
    }

    /// Returns the scalar carrier stored in physical memory.
    #[must_use]
    pub fn storage_scalar(self) -> StorageScalar {
        self.0.storage_scalar
    }

    /// Returns the type through which kernels access the stored component.
    #[must_use]
    pub fn access_type(self) -> KernelType {
        self.0.access_type
    }

    /// Returns the complete storage encoding.
    #[must_use]
    pub fn storage_encoding(self) -> StorageEncoding {
        self.0.encoding
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

    /// Returns the additional live-device requirements of this variant's route.
    ///
    /// Empty is a state rather than an absence: a route consuming no additional
    /// requirement declares none. This layer cannot decide whether a row is
    /// *missing*, because only the producer holds the exhaustive declaration of
    /// what its selected payload uses.
    #[must_use]
    pub fn route_requirements(self) -> &'a [RouteRequirement] {
        &self.data().route_requirements
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

    /// Returns the complete target-property query contract.
    #[must_use]
    pub fn requirement(self) -> &'a PreparedEntryTargetRequirement {
        &self.data().requirement
    }

    /// Returns the exact prepared entry whose property must be observed.
    #[must_use]
    pub fn entry(self) -> EntryRef<'a> {
        EntryRef {
            artifact: self.artifact,
            variant: self.variant,
            entry: position(self.data().entry),
        }
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

    /// **Draft surface, not yet accepted.** See [`DecodedExtentOperand`].
    ///
    /// Returns the live input-extent operand rows in canonical declaration order.
    #[must_use]
    pub fn extent_operands(self) -> impl ExactSizeIterator<Item = DecodedExtentOperand<'a>> {
        self.data().input_extents.iter().map(DecodedExtentOperand)
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

    /// Returns the backend payload realizing this entry at one delivery position.
    ///
    /// `None` for a position this artifact declares no payload at; see
    /// [`VerifiedArtifactProgram::delivery_positions`] for how many there are.
    /// A position argument rather than a bare accessor because an artifact
    /// carrying several objects has no *default* one — picking the first would
    /// hand a consumer the object built for somebody else's target.
    #[must_use]
    pub fn payload(self, delivery: usize) -> Option<&'a BackendPayloadDescriptor> {
        let payload = *self.data().implementation.payloads.get(delivery)?;
        Some(&self.artifact.data.payloads[position(payload)])
    }

    /// Returns every payload realizing this entry, in delivery-position order.
    #[must_use]
    pub fn payloads(self) -> impl ExactSizeIterator<Item = &'a BackendPayloadDescriptor> {
        let artifact = self.artifact;
        self.data()
            .implementation
            .payloads
            .iter()
            .map(move |payload| &artifact.data.payloads[position(*payload)])
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

    /// Returns the scalar carrier stored in physical memory.
    #[must_use]
    pub fn storage_scalar(self) -> StorageScalar {
        self.data().storage_scalar
    }

    /// Returns the type through which the kernel accesses this binding.
    #[must_use]
    pub fn access_type(self) -> KernelType {
        self.data().access_type
    }

    /// Returns the addressed semantic component role, absent for a dense singleton.
    #[must_use]
    pub fn component_role(self) -> Option<EncodedComponentRole> {
        self.data().component_role
    }

    /// Returns the complete physical storage encoding.
    #[must_use]
    pub fn storage_encoding(self) -> StorageEncoding {
        self.data().encoding
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
    pub fn alignment(self) -> AlignmentRequirement {
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

    /// Returns the first addressed byte of the bound value, as an expression.
    ///
    /// Together with [`Self::accessible_bytes`] this is the exact range the
    /// entry reaches. A slot may address part of its value, so the offset is a
    /// placement a loader must honour rather than a field that is always zero.
    #[must_use]
    pub fn accessible_offset(self) -> AbiExprRef<'a> {
        AbiExprRef {
            artifact: self.artifact,
            node: position(self.data().accessible_offset),
        }
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
/// The key is the shared IR's own complete stage subject — the exact bound
/// kernel and either proof-bound realization claims (including continuations)
/// or exact named-output component publication claims — so an artifact entry
/// cross-references a stage by content rather than by the program's declaration
/// position. Two stages that differ in any owner claim are different here as
/// well as in the kernel program.
///
/// This is the second, independent writer of that subject. It is deliberately
/// not shared with `tiler_ir`'s own stage encoder: a widening the two disagree
/// about must be a visible divergence between two readable encoders rather than
/// a silent agreement neither states.
pub(super) fn stage_key(program: &VerifiedKernelProgram, stage: StageRef<'_>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(STAGE_KEY_DOMAIN);
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());
    let realization = realization_claims(program, stage);
    let publication = publication_claims(program, stage);
    match (realization.is_empty(), publication.is_empty()) {
        (false, true) => {
            bytes.push(0x01);
            push_len(&mut bytes, realization.len());
            for (ordinal, covered) in realization {
                bytes.extend_from_slice(&ordinal.to_be_bytes());
                bytes.extend_from_slice(&covered.occurrence().get().to_be_bytes());
                push_slice(&mut bytes, covered.refinement().as_bytes());
            }
        }
        (true, false) => {
            bytes.push(0x02);
            push_len(&mut bytes, publication.len());
            for (key, role) in publication {
                push_slice(&mut bytes, key.as_str().as_bytes());
                push_component_role(&mut bytes, role);
            }
        }
        // A verified kernel program has exactly one closed owner per stage;
        // reaching either branch is a shared-IR verifier defect, not a fallback.
        _ => unreachable!("verified program carries one complete stage owner"),
    }
    bytes
}

fn realization_claims<'a>(
    program: &'a VerifiedKernelProgram,
    stage: StageRef<'a>,
) -> Vec<(u32, tiler_ir::program::CoveredOccurrence)> {
    let mut claims: Vec<_> = stage
        .coverage()
        .iter()
        .cloned()
        .map(|covered| (0, covered))
        .collect();
    let mut occurrences: Vec<_> = program
        .partial_reductions()
        .filter(|split| split.combiner() == stage)
        .map(tiler_ir::program::PartialReductionRef::occurrence)
        .chain(
            program
                .staged_realizations()
                .filter(|row| row.consumer() == stage)
                .map(tiler_ir::program::StagedRealizationRef::occurrence),
        )
        .collect();
    occurrences.sort_unstable();
    occurrences.dedup();
    for occurrence in occurrences {
        let (root, proof) = program
            .stages()
            .find_map(|candidate| {
                candidate
                    .coverage()
                    .iter()
                    .find(|covered| covered.occurrence() == occurrence)
                    .cloned()
                    .map(|covered| (candidate, covered))
            })
            .expect("verified owner has a proof-bound root");
        let mut current = root;
        let mut ordinal = 0_u32;
        while let Some(next) = program
            .partial_reductions()
            .filter(|split| split.occurrence() == occurrence)
            .map(|split| (split.producer(), split.combiner()))
            .chain(
                program
                    .staged_realizations()
                    .filter(|row| row.occurrence() == occurrence)
                    .map(|row| (row.producer(), row.consumer())),
            )
            .find(|(producer, _)| *producer == current)
            .map(|(_, consumer)| consumer)
        {
            ordinal = ordinal.saturating_add(1);
            if next == stage {
                claims.push((ordinal, proof.clone()));
                break;
            }
            current = next;
        }
    }
    claims.sort_by(|left, right| {
        left.1
            .occurrence()
            .get()
            .cmp(&right.1.occurrence().get())
            .then_with(|| left.0.cmp(&right.0))
    });
    claims
}

fn publication_claims(
    program: &VerifiedKernelProgram,
    stage: StageRef<'_>,
) -> Vec<(OutputKey, Option<EncodedComponentRole>)> {
    let mut claims: Vec<_> = program
        .publishing_copies()
        .filter(|copy| copy.publisher() == stage)
        .map(|copy| {
            let published = copy.published();
            let output = program
                .outputs()
                .find(|output| output.value() == published)
                .expect("verified publishing owner names one output component");
            (output.key().clone(), published.component_role())
        })
        .collect();
    claims.sort_by(|left, right| {
        left.0
            .as_str()
            .cmp(right.0.as_str())
            .then_with(|| left.1.cmp(&right.1))
    });
    claims
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
        KernelType::U8 => 0x04,
        KernelType::I32 => 0x05,
        // Appended: every earlier tag keeps its value, so no artifact the
        // earlier vocabulary could encode maps to different bytes.
        KernelType::Bf16 => 0x06,
        // Appended: every earlier artifact element-type byte remains exact.
        KernelType::U32 => 0x07,
    }
}

pub(super) const fn element_type_from_tag(tag: u8) -> Option<KernelType> {
    match tag {
        0x01 => Some(KernelType::Bool),
        0x02 => Some(KernelType::Index),
        0x03 => Some(KernelType::F32),
        0x04 => Some(KernelType::U8),
        0x05 => Some(KernelType::I32),
        0x06 => Some(KernelType::Bf16),
        0x07 => Some(KernelType::U32),
        _ => None,
    }
}

pub(super) const fn storage_scalar_tag(storage_scalar: StorageScalar) -> u8 {
    match storage_scalar {
        StorageScalar::U8 => 0x01,
        StorageScalar::F32 => 0x02,
        // Appended, for the reason `element_type_tag` states.
        StorageScalar::Bf16 => 0x03,
        StorageScalar::U32 => 0x04,
    }
}

pub(super) const fn storage_scalar_from_tag(tag: u8) -> Option<StorageScalar> {
    match tag {
        0x01 => Some(StorageScalar::U8),
        0x02 => Some(StorageScalar::F32),
        0x03 => Some(StorageScalar::Bf16),
        0x04 => Some(StorageScalar::U32),
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

pub(super) fn push_component_role(bytes: &mut Vec<u8>, role: Option<EncodedComponentRole>) {
    match role {
        None => bytes.push(0x00),
        Some(role) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&role.get().to_be_bytes());
        }
    }
}

pub(super) fn push_storage_encoding(bytes: &mut Vec<u8>, encoding: StorageEncoding) {
    match encoding {
        StorageEncoding::Unpacked => bytes.push(0x01),
        StorageEncoding::BitPacked(packed) => {
            bytes.push(0x02);
            bytes.push(packed.element_bits());
            bytes.push(match packed.bit_order() {
                tiler_ir::program::PackedBitOrder::LeastSignificantElementFirst => 0x01,
                tiler_ir::program::PackedBitOrder::MostSignificantElementFirst => 0x02,
            });
            bytes.push(match packed.tail() {
                tiler_ir::program::PackedTailRule::Zero => 0x01,
            });
        }
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

/// This crate's own tag table for the approximate-intrinsic envelope.
///
/// A second, independent copy of the schedule's table by design, exactly as the
/// subnormal and permission tables are: the two identities are different
/// subjects, and a shared encoder would let one domain's step silently move the
/// other's bytes. Admitting a third envelope is a build error at both.
pub(super) const fn approximation_envelope_tag(envelope: ApproximationEnvelope) -> u8 {
    match envelope {
        ApproximationEnvelope::Forbidden => 0x01,
        ApproximationEnvelope::BackendElementary => 0x02,
    }
}

pub(super) const fn approximation_envelope_from_tag(tag: u8) -> Option<ApproximationEnvelope> {
    match tag {
        0x01 => Some(ApproximationEnvelope::Forbidden),
        0x02 => Some(ApproximationEnvelope::BackendElementary),
        _ => None,
    }
}

/// The governed tag table of the index-arithmetic requirement vocabulary.
///
/// A forward and inverse pair kept in one place, like every other enumeration
/// this envelope encodes: the forward half is an exhaustive match, so widening
/// [`IndexArithmetic`] stops the build here rather than compiling into a
/// run-time rejection, and the inverse half refuses an unassigned byte before
/// the requirement it names is compared against a device.
pub(super) const fn index_arithmetic_tag(index_arithmetic: IndexArithmetic) -> u8 {
    match index_arithmetic {
        IndexArithmetic::CompleteU64 => 0x01,
    }
}

pub(super) const fn index_arithmetic_from_tag(tag: u8) -> Option<IndexArithmetic> {
    match tag {
        0x01 => Some(IndexArithmetic::CompleteU64),
        _ => None,
    }
}

/// Encodes one exceptional-value assumption and its provenance in one tag.
///
/// The flattened table keeps the record fixed-width while remaining injective:
/// the three provenance classes are semantically distinct and therefore receive
/// distinct tags rather than sharing an `AssumeAbsent` tag with trailing data.
pub(super) const fn exceptional_assumption_tag(assumption: ExceptionalValueAssumption) -> u8 {
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption => 0x01,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CompilerProven,
        } => 0x02,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::RuntimeValidated,
        } => 0x03,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        } => 0x04,
    }
}

pub(super) const fn exceptional_assumption_from_tag(tag: u8) -> Option<ExceptionalValueAssumption> {
    match tag {
        0x01 => Some(ExceptionalValueAssumption::MakeNoAssumption),
        0x02 => Some(ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CompilerProven,
        }),
        0x03 => Some(ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::RuntimeValidated,
        }),
        0x04 => Some(ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        }),
        _ => None,
    }
}

/// This crate's own tag table for the synchronization operation kind.
///
/// A second, independent copy of the schedule's table by design, exactly as the
/// subnormal and permission tables are: the two identities are different
/// subjects, and a shared encoder would let one domain's step silently move the
/// other's bytes. Adding a kind is a build error at both.
pub(super) const fn synchronization_kind_tag(kind: SynchronizationKind) -> u8 {
    match kind {
        SynchronizationKind::ControlBarrier => 0x01,
        SynchronizationKind::AsynchronousCopy => 0x02,
        SynchronizationKind::SplitPhaseBarrier => 0x03,
        SynchronizationKind::Collective => 0x04,
        SynchronizationKind::Atomic => 0x05,
        SynchronizationKind::InterDispatchDependency => 0x06,
    }
}

pub(super) const fn synchronization_kind_from_tag(tag: u8) -> Option<SynchronizationKind> {
    match tag {
        0x01 => Some(SynchronizationKind::ControlBarrier),
        0x02 => Some(SynchronizationKind::AsynchronousCopy),
        0x03 => Some(SynchronizationKind::SplitPhaseBarrier),
        0x04 => Some(SynchronizationKind::Collective),
        0x05 => Some(SynchronizationKind::Atomic),
        0x06 => Some(SynchronizationKind::InterDispatchDependency),
        _ => None,
    }
}

pub(super) const fn synchronization_scope_tag(scope: SynchronizationScope) -> u8 {
    match scope {
        SynchronizationScope::Subgroup => 0x01,
        SynchronizationScope::Workgroup => 0x02,
        SynchronizationScope::Device => 0x03,
    }
}

pub(super) const fn synchronization_scope_from_tag(tag: u8) -> Option<SynchronizationScope> {
    match tag {
        0x01 => Some(SynchronizationScope::Subgroup),
        0x02 => Some(SynchronizationScope::Workgroup),
        0x03 => Some(SynchronizationScope::Device),
        _ => None,
    }
}

pub(super) const fn memory_ordering_tag(ordering: MemoryOrdering) -> u8 {
    match ordering {
        MemoryOrdering::Relaxed => 0x01,
        MemoryOrdering::AcquireRelease => 0x02,
        MemoryOrdering::SequentiallyConsistent => 0x03,
    }
}

pub(super) const fn memory_ordering_from_tag(tag: u8) -> Option<MemoryOrdering> {
    match tag {
        0x01 => Some(MemoryOrdering::Relaxed),
        0x02 => Some(MemoryOrdering::AcquireRelease),
        0x03 => Some(MemoryOrdering::SequentiallyConsistent),
        _ => None,
    }
}

/// Encodes the synchronization realization one entry requires, or its absence.
///
/// A presence tag ahead of a fixed-width subject. The absence is written rather
/// than omitted because more fields follow it in the resource record, and it is
/// written *at all* because "this entry performs no synchronization" has to be a
/// recorded fact: an entry that later gains one must not share identity with the
/// entry that did not, and a reader must not be able to recover "no requirement"
/// from bytes that never stated it.
pub(super) fn push_synchronization(bytes: &mut Vec<u8>, subject: Option<SynchronizationSubject>) {
    match subject {
        None => bytes.push(0x00),
        Some(subject) => {
            bytes.push(0x01);
            bytes.push(synchronization_kind_tag(subject.kind));
            bytes.push(synchronization_scope_tag(subject.execution_scope));
            bytes.push(synchronization_scope_tag(subject.visibility_scope));
            bytes.push(u8::from(subject.fenced_spaces.workgroup));
            bytes.push(u8::from(subject.fenced_spaces.device));
            bytes.push(memory_ordering_tag(subject.ordering));
        }
    }
}

/// Opens the artifact resource record's conditional subgroup tail.
///
/// Zero is deliberately not an absence tag. An absent row writes nothing, and
/// the next field's bounded `u64` text length always supplies a zero high byte.
/// A present row claims the otherwise-impossible nonzero byte and appends the
/// fixed-width public subject encoding.
pub(super) const SUBGROUP_REQUIREMENT_BLOCK_TAG: u8 = 0x01;

/// Resolves the transfer tag at the artifact decoder's sole consuming seam.
///
/// The public schedule surface deliberately exposes no raw transfer tag or
/// inverse. Forward bytes remain owned by
/// [`SubgroupRealizationSubject::encode`]; this private inverse only recognizes
/// bytes that have already crossed the artifact boundary. The test-sized
/// transfer population derives every claimed byte through that public encoder
/// and checks this function over all 256 inputs, so vocabulary growth cannot
/// leave the literal below as a silently partial second authority.
pub(super) const fn subgroup_transfer_from_tag(tag: u8) -> Option<SubgroupTransfer> {
    match tag {
        0x01 => Some(SubgroupTransfer::InRangeXorShuffle),
        _ => None,
    }
}

/// Appends one present subgroup subject, while preserving every absent byte.
///
/// This conditional tail is injective because it is the final resource field
/// and the following numerical profile key has a `u64` byte length bounded by
/// `MAX_TEXT_BYTES = 4096`. Every valid legacy continuation therefore begins
/// with zero. Absence consumes none of that continuation; presence begins with
/// the otherwise-impossible nonzero block tag and six fixed subject bytes. An
/// old reader interprets that nonzero byte as the high byte of the text length
/// and refuses it as over-limit rather than misreading the row.
pub(super) fn push_subgroup_requirement(
    bytes: &mut Vec<u8>,
    subject: Option<SubgroupRealizationSubject>,
) {
    if let Some(subject) = subject {
        bytes.push(SUBGROUP_REQUIREMENT_BLOCK_TAG);
        subject.encode(bytes);
    }
}

/// Encodes one entry's resource requirements into the artifact grammar.
///
/// # Errors
///
/// Returns [`ArtifactDiagnostic::BitPreservingCopyResources`] for the copy
/// numerical arm: the grammar carries the ten floating-point rows every
/// existing artifact wrote, and the copy's tagged entry row plus its schema
/// step belong to the accepted delivery-state boundary. The `FloatingPoint`
/// arm writes byte-for-byte what the ten flat fields wrote, so every existing
/// artifact's bytes and identity stay exact under the current schema.
pub(super) fn push_resources(
    bytes: &mut Vec<u8>,
    resources: ResourceRequirements,
) -> Result<(), ArtifactDiagnostic> {
    let ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup,
        local_memory_bytes,
        requires_device_memory,
        index_arithmetic,
        synchronization,
        subgroup,
        numerical,
    } = resources;
    let tiler_ir::schedule::RegionNumericalRequirements::FloatingPoint {
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        reciprocal_transform,
        approximate_intrinsics,
        nan_assumptions,
        infinity_assumptions,
    } = numerical
    else {
        return Err(ArtifactDiagnostic::BitPreservingCopyResources);
    };
    bytes.extend_from_slice(&buffer_bindings.to_be_bytes());
    bytes.extend_from_slice(&threads_per_workgroup.to_be_bytes());
    bytes.extend_from_slice(&local_memory_bytes.to_be_bytes());
    bytes.push(u8::from(requires_device_memory));
    bytes.push(index_arithmetic_tag(index_arithmetic));
    push_synchronization(bytes, synchronization);
    bytes.push(subnormal_tag(input_subnormals));
    bytes.push(subnormal_tag(result_subnormals));
    bytes.push(permission_tag(contraction));
    bytes.push(permission_tag(reassociation));
    bytes.push(permission_tag(permutation));
    bytes.push(permission_tag(signed_zero));
    bytes.push(permission_tag(reciprocal_transform));
    bytes.push(approximation_envelope_tag(approximate_intrinsics));
    bytes.push(exceptional_assumption_tag(nan_assumptions));
    bytes.push(exceptional_assumption_tag(infinity_assumptions));
    push_subgroup_requirement(bytes, subgroup);
    Ok(())
}

pub(super) fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalFacts) {
    let NumericalFacts {
        profile_key,
        canonical_arithmetic_nan_bits,
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        reciprocal_transform,
        approximate_intrinsics,
        nan_assumptions,
        infinity_assumptions,
    } = numerical;
    push_slice(bytes, profile_key.as_bytes());
    bytes.extend_from_slice(&canonical_arithmetic_nan_bits.to_be_bytes());
    bytes.push(subnormal_tag(*input_subnormals));
    bytes.push(subnormal_tag(*result_subnormals));
    bytes.push(permission_tag(*contraction));
    bytes.push(permission_tag(*reassociation));
    bytes.push(permission_tag(*permutation));
    bytes.push(permission_tag(*signed_zero));
    bytes.push(permission_tag(*reciprocal_transform));
    bytes.push(approximation_envelope_tag(*approximate_intrinsics));
    bytes.push(exceptional_assumption_tag(*nan_assumptions));
    bytes.push(exceptional_assumption_tag(*infinity_assumptions));
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
    push_slice(&mut bytes, envelope.semantic().retained_shape.as_bytes());
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
    // The delivered-realization record, folded last and framed. Two artifacts
    // delivering one contract by different means are not the same artifact, so
    // the means, the provenance, and the locus-keyed obligations all enter the
    // identity — through the record's own domain-separated canonical encoder
    // rather than through a second spelling of it here.
    push_slice(&mut bytes, &envelope.realization().canonical_bytes());
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
            .then_with(|| left.entry.cmp(&right.entry))
            .then_with(|| left.requirement.cmp(&right.requirement))
    });
    order
}

/// Proves one predicate reads the exact target-property query it declares.
pub(super) fn deferred_predicate_matches_requirement(
    nodes: &[ExprNode],
    predicate: u32,
    requirement: &PreparedEntryTargetRequirement,
) -> bool {
    let unsigned = |node: u32, expected: u64| {
        matches!(
            nodes.get(position(node)),
            Some(ExprNode::Root(AbiRoot::UnsignedLiteral(value))) if *value == expected
        )
    };
    let property = |node: u32| {
        matches!(
            nodes.get(position(node)),
            Some(ExprNode::Root(AbiRoot::TargetProperty { key, phase }))
                if key == requirement.query().key()
                    && *phase == requirement.query().available_at()
        )
    };
    let binary = |node: u32, expected: AbiBinaryOp| match nodes.get(position(node)) {
        Some(ExprNode::Binary { op, left, right }) if *op == expected => Some((*left, *right)),
        _ => None,
    };
    match requirement.relation() {
        TargetPropertyRequirementRelation::ObservedAtLeastRequired => {
            let Some((left, right)) = binary(predicate, AbiBinaryOp::LessOrEqual) else {
                return false;
            };
            unsigned(left, requirement.required()) && property(right)
        }
        TargetPropertyRequirementRelation::ObservedEqualsRequired => {
            let Some((left, right)) = binary(predicate, AbiBinaryOp::Equal) else {
                return false;
            };
            unsigned(left, requirement.required()) && property(right)
        }
        TargetPropertyRequirementRelation::RequiredImpliesObserved => {
            let Some((required_is_zero, observed_nonzero)) = binary(predicate, AbiBinaryOp::Or)
            else {
                return false;
            };
            let Some((required, zero)) = binary(required_is_zero, AbiBinaryOp::Equal) else {
                return false;
            };
            let Some((one, observed)) = binary(observed_nonzero, AbiBinaryOp::LessOrEqual) else {
                return false;
            };
            unsigned(required, requirement.required())
                && unsigned(zero, 0)
                && unsigned(one, 1)
                && property(observed)
        }
    }
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
                    .flat_map(|binding| [binding.accessible_offset, binding.accessible_bytes]),
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
        push_interface_components(bytes, input);
    }
    push_len(bytes, envelope.outputs().len());
    for output in envelope.outputs() {
        push_slice(bytes, output.key.as_str().as_bytes());
        push_shape(bytes, &output.shape);
        push_interface_components(bytes, output);
    }
}

fn push_interface_components<K>(bytes: &mut Vec<u8>, entry: &InterfaceEntryData<K>) {
    push_slice(bytes, &entry.logical_type);
    push_len(bytes, entry.components.len());
    for component in &entry.components {
        push_component_role(bytes, component.role);
        push_shape(bytes, &component.shape);
        match &component.resolved_type {
            None => bytes.push(0x00),
            Some(value_type) => {
                bytes.push(0x01);
                push_slice(bytes, value_type);
            }
        }
        bytes.push(storage_scalar_tag(component.storage_scalar));
        push_storage_encoding(bytes, component.encoding);
        bytes.push(element_type_tag(component.access_type));
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
    // Already in canonical order, for the same reason and by the same shared
    // definition. Distinctness is proven here as well as at construction: the
    // envelope this encodes may have been decoded rather than built, and a
    // decoder's own duplicate check is a different code path from the builder's.
    if variant
        .route_requirements
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err(ArtifactDiagnostic::AmbiguousCanonicalKey {
            entity: ArtifactEntityKind::RouteRequirement,
        });
    }
    push_requirements(bytes, &variant.route_requirements);
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
        + size_of::<u32>()
        + framed(predicate.requirement.canonical_bytes().len());
    let mut bytes = Vec::with_capacity(exact);
    bytes.extend_from_slice(DEFERRED_KEY_DOMAIN);
    push_abi_reference(&mut bytes, arena, predicate.predicate);
    bytes.extend_from_slice(&predicate.entry.to_be_bytes());
    push_slice(&mut bytes, &predicate.requirement.canonical_bytes());
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
    push_resources(bytes, entry.resources)?;
    push_numerical(bytes, &entry.numerical);
    push_len(bytes, entry.bindings.len());
    for binding in &entry.bindings {
        bytes.push(binding.kind.tag());
        bytes.push(storage_scalar_tag(binding.storage_scalar));
        bytes.push(element_type_tag(binding.access_type));
        push_component_role(bytes, binding.component_role);
        push_storage_encoding(bytes, binding.encoding);
        bytes.push(address_space_tag(binding.address_space));
        bytes.push(buffer_access_tag(binding.access));
        bytes.extend_from_slice(&binding.alignment.bytes().to_be_bytes());
        push_binding_target(bytes, &binding.target);
        push_abi_reference(bytes, arena, binding.accessible_offset);
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
    // The delivery-position count is folded by writing it, not assumed from the
    // enclosing record: two artifacts that differ only in how many families they
    // were built for differ here, in every entry, rather than only in the sorted
    // payload table. Order is meaning — position `p` is what a consumer's build
    // target resolves to — so the keys are written as stated rather than sorted.
    push_len(bytes, entry.payloads.len());
    for payload in &entry.payloads {
        push_slice(bytes, &payload_keys[node_at(*payload)]);
    }
    push_slice(bytes, entry.entry_key.as_bytes());
    push_input_extents(bytes, &entry.input_extents);
    Ok(())
}

/// Presence tag of a nonempty live input-extent operand list.
///
/// Written after the backend entry key, and written as nothing at all when the
/// list is empty. The key is a framed slice, so a reader that has consumed it
/// is at the next field. `0xfe` cannot be the high byte of the next `push_len`
/// (execution order or the next entry's stage key), so a nonempty list cannot
/// be re-read as that length. Empty writes nothing, so previously encodable
/// entries keep the bytes they encoded before this list existed.
pub(super) const INPUT_EXTENT_BLOCK_TAG: u8 = 0xfe;

pub(super) const fn abi_type_tag(ty: AbiType) -> u8 {
    match ty {
        AbiType::Unsigned => 0x01,
        AbiType::Boolean => 0x02,
    }
}

pub(super) fn abi_type_from_tag(tag: u8) -> Option<AbiType> {
    match tag {
        0x01 => Some(AbiType::Unsigned),
        0x02 => Some(AbiType::Boolean),
        _ => None,
    }
}

fn push_input_extents(bytes: &mut Vec<u8>, extents: &[ExtentOperandData]) {
    if extents.is_empty() {
        return;
    }
    bytes.push(INPUT_EXTENT_BLOCK_TAG);
    push_len(bytes, extents.len());
    for operand in extents {
        push_slice(bytes, operand.key.as_str().as_bytes());
        bytes.extend_from_slice(&operand.axis.get().to_be_bytes());
        bytes.push(abi_type_tag(operand.value_type));
    }
}

/// Reads how many delivery positions one artifact's entries are realized at.
///
/// The first entry answers for all of them: `push_variant` refuses an entry
/// whose realization count disagrees, and `codec::validate` re-proves the same
/// agreement for an envelope decoded from bytes no builder wrote. Zero is
/// reachable only for a draft with no variant or no entry, which
/// [`ArtifactDiagnostic::EmptyPortfolio`] and
/// [`ArtifactBuildError::EntryCardinality`] already refuse.
///
/// [`ArtifactBuildError::EntryCardinality`]: super::ArtifactBuildError::EntryCardinality
pub(super) fn delivery_positions(data: &ArtifactProgramData) -> usize {
    data.variants
        .first()
        .and_then(|variant| variant.entries.first())
        .map_or(0, |entry| entry.implementation.payloads.len())
}
