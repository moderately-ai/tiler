//! Typed, non-erasing failures of artifact envelope encoding and decoding.
//!
//! Every variant names the boundary that rejected, in the vocabulary
//! `docs/artifact-abi.md` uses for it: framing and integrity, schema and
//! feature compatibility, canonical form, structural closure, or a
//! re-proven artifact-model obligation. A decoder never reinterprets a
//! corrupt or unsupported envelope as an applicability miss, and never
//! returns a partially validated model.
//!
//! Two variants deliberately wrap a *lower* layer's own error rather than
//! restating it: [`ArtifactCodecError::ModelRule`] carries the exact
//! [`ArtifactBuildError`] an insertion-time rule produced, and
//! [`ArtifactCodecError::ModelObligation`] carries the exact
//! [`ArtifactDiagnostic`] a whole-artifact obligation produced. Re-proving a
//! decoded envelope against the model's own rules must report the model's own
//! cause, or the codec would grow a second, drifting diagnostic vocabulary.

use std::error::Error;
use std::fmt;

use tiler_ir::program::ByteAlignmentError;
use tiler_ir::schedule::SubgroupRealizationError;
use tiler_ir::semantic::{BuildError, RegistryError, TypeIdentityError};
use tiler_ir::shape::{ShapeEnvSubjectError, ShapeError};

use super::super::error::{ArtifactBuildError, ArtifactDiagnostic, ProvenanceField};
use super::super::expr::AbiType;
use super::super::realization::codec::RealizationCodecError;

/// A governed component schema of the artifact model.
///
/// The component is identified by position in the manifest, so this vocabulary
/// is the reader's name for the position that disagreed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ComponentSchemaKind {
    /// The artifact program schema.
    Program,
    /// The ABI expression language schema.
    AbiExpression,
    /// The guard and routing schema.
    GuardAndRouting,
    /// The target requirement schema.
    TargetRequirement,
}

impl fmt::Display for ComponentSchemaKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A closed enumeration this codec reads from a wire tag.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum TagSubject {
    /// The canonical routing policy of the portfolio.
    RoutingPolicy,
    /// The transport category of one ABI binding.
    BindingKind,
    /// The execution policy of one backend payload.
    ExecutionPolicy,
    /// The type through which a kernel accesses one component or binding.
    ElementType,
    /// The ABI type of one live input-extent operand row.
    ExtentOperandType,
    /// The scalar carrier stored in physical memory.
    StorageScalar,
    /// Presence of a stable encoded-component role or component type.
    ComponentPresence,
    /// Complete physical storage encoding.
    StorageEncoding,
    /// Ordering of elements in one packed byte.
    PackedBitOrder,
    /// Required contents of unused packed tail bits.
    PackedTailRule,
    /// The logical address space of one binding.
    AddressSpace,
    /// The access mode of one binding.
    BufferAccess,
    /// What one ABI binding slot addresses.
    BindingTarget,
    /// The subnormal treatment of one numerical realization.
    SubnormalMode,
    /// A numerical transform permission of one numerical realization.
    NumericalPermission,
    /// The approximate-intrinsic envelope of one numerical realization.
    ApproximationEnvelope,
    /// An exceptional-value assumption and the provenance that supports it.
    ExceptionalValueAssumption,
    /// The index arithmetic one entry's dispatch record requires of a device.
    IndexArithmetic,
    /// The availability phase of a root fact or deferred predicate.
    AvailabilityPhase,
    /// The directional relation of a target-property requirement.
    TargetPropertyRequirementRelation,
    /// The kind of one live-device route requirement.
    RouteRequirementKind,
    /// The neutral live-device dimension one route resource row constrains.
    RouteResourceDimension,
    /// The node kind of one ABI expression arena entry.
    ExpressionNode,
    /// The typed root fact of one ABI expression.
    ExpressionRoot,
    /// One admitted unary ABI operation.
    UnaryOperation,
    /// One admitted binary ABI operation.
    BinaryOperation,
    /// Why one packaged stage must precede another.
    StageDependencyReason,
    /// The purpose of one envelope section.
    SectionKind,
    /// Whether an unrecognizing reader may skip one envelope section.
    SectionDisposition,
    /// Whether one backend payload carries its content in this envelope.
    PayloadContent,
    /// Whether one backend payload declares a target environment.
    TargetEnvironmentPresence,
    /// The plan-determinism scope one variant claims at one delivery position.
    PlanDeterminismScope,
    /// The operation kind of one entry's synchronization realization.
    SynchronizationKind,
    /// An invocation scope of one entry's synchronization realization.
    SynchronizationScope,
    /// The memory ordering of one entry's synchronization realization.
    MemoryOrdering,
    /// Whether one entry requires a synchronization realization at all.
    SynchronizationPresence,
    /// Whether the conditional subgroup-requirement block is present.
    SubgroupPresence,
    /// The arithmetic type of one subgroup realization.
    SubgroupArithmetic,
    /// The register transfer of one subgroup realization.
    SubgroupTransfer,
    /// A Boolean field encoded as one byte.
    Boolean,
    /// The platform shape one carried payload's provenance declares.
    ///
    /// Only the unversioned tag is admitted, and the tag is present only for a
    /// payload that carries it: the versioned-SDK shape is the *untagged*
    /// encoding, so a tag naming it would give one record two spellings.
    /// `super::payload` states why that matters for payload identity.
    PayloadPlatform,
    /// The source kind of one declared interface axis: a literal or a symbol.
    InterfaceExtentSource,
}

impl fmt::Display for TagSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A collection whose canonical wire order and distinctness are load-bearing.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum OrderedSubject {
    /// The required-feature set.
    Feature,
    /// A stable key of the named program interface.
    InterfaceKey,
    /// The selected capability providers.
    Provider,
    /// The backend payload descriptors.
    Payload,
    /// The shared ABI expression arena.
    Expression,
    /// The deferred feasibility predicates of one plan variant.
    DeferredPredicate,
    /// The live-device route requirements of one plan variant.
    RouteRequirement,
    /// The launch preconditions of one executable entry.
    LaunchPrecondition,
    /// The executable entries of one plan variant.
    Entry,
    /// The named program outputs one ABI binding's target publishes.
    BindingTargetKey,
    /// The live input-extent operand rows of one executable entry.
    ExtentOperand,
    /// The framed envelope sections.
    Section,
    /// The entry mappings of one carried backend payload.
    PayloadEntryMapping,
    /// The stage dependency edges of one plan variant.
    StageDependency,
    /// The versioned tool components of one payload provenance record.
    ProvenanceComponent,
    /// The recorded target obligations of one carried backend payload.
    TargetObligation,
}

impl fmt::Display for OrderedSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A cross-reference one decoded row makes into another decoded table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ReferenceSubject {
    /// An ABI expression arena node.
    Expression,
    /// A backend payload descriptor.
    Payload,
    /// An envelope section.
    Section,
    /// An executable entry of one plan variant.
    Entry,
}

impl fmt::Display for ReferenceSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A governed parser or encoder budget of the artifact envelope.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CodecLimitKind {
    /// Total encoded envelope bytes.
    EnvelopeBytes,
    /// Canonical manifest bytes.
    ManifestBytes,
    /// Bytes of one framed section.
    SectionBytes,
    /// Framed section count.
    Sections,
    /// Required-feature count.
    Features,
    /// Named interface entries of the artifact.
    InterfaceEntries,
    /// Bytes of one received opaque identity subject.
    SubjectBytes,
    /// Plan-variant count.
    Variants,
    /// Executable-entry count of one plan variant.
    Entries,
    /// Stage-dependency edge count of one plan variant.
    ///
    /// Distinct from [`Self::Entries`] because the two are separately bounded —
    /// `MAX_VARIANT_ENTRIES` against `MAX_STAGE_DEPENDENCIES` — so reporting an
    /// edge overflow as an entry overflow names a limit the bytes did not
    /// exceed and sends a reader to the wrong number.
    StageDependencies,
    /// ABI binding count of one executable entry.
    EntryBindings,
    /// Live input-extent operand count of one executable entry.
    EntryExtents,
    /// Named-output count of one ABI binding's target.
    BindingTargetKeys,
    /// Node count of the shared ABI expression arena.
    Expressions,
    /// Backend payload descriptor count.
    Payloads,
    /// Delivery-position count of one executable entry's realization run.
    DeliveryPositions,
    /// Plan-determinism scope-cell count of one plan variant.
    PlanDeterminismScopeCells,
    /// Byte length of one declared target-environment descriptor.
    TargetEnvironmentDescriptorBytes,
    /// Selected capability-provider count.
    SelectedProviders,
    /// Deferred feasibility predicate count of one plan variant.
    DeferredPredicates,
    /// Live-device route-requirement count of one plan variant.
    RouteRequirements,
    /// Byte length of one backend feature requirement's canonical payload.
    RouteFeaturePayloadBytes,
    /// Launch precondition count of one executable entry.
    LaunchPreconditions,
    /// Rank of one declared interface shape.
    ShapeRank,
    /// Byte length of one encoded text run.
    TextBytes,
    /// Byte length of one carried payload's exact compiled source.
    PayloadSourceBytes,
    /// Entry-mapping count of one carried backend payload.
    PayloadEntryMappings,
    /// Transport-slot count of one entry mapping.
    EntryTransports,
    /// Versioned tool-component count of one payload provenance record.
    ProvenanceComponents,
    /// Compiler or linker flag count of one payload provenance record.
    ProvenanceFlags,
    /// Recorded target-obligation count of one carried backend payload.
    TargetObligations,
}

impl fmt::Display for CodecLimitKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A typed failure of artifact envelope encoding, decoding, or validation.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: this is a rejection
/// vocabulary that a consumer forwards or partially classifies, never a
/// vocabulary any crate maps totally, so a new boundary lands additively.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum ArtifactCodecError {
    /// The encoding ran out of bytes before a field was complete.
    Truncated {
        /// Bytes the field required.
        needed: usize,
        /// Bytes that remained.
        available: usize,
    },
    /// Bytes remained after the last framed section.
    TrailingBytes {
        /// Count of unconsumed bytes.
        count: usize,
    },
    /// Bytes remained after the last canonical manifest field.
    TrailingManifestBytes {
        /// Count of unconsumed manifest bytes.
        count: usize,
    },
    /// The fixed framing magic did not match.
    BadMagic,
    /// The canonical manifest did not open with its versioned domain tag.
    BadManifestDomain,
    /// A carried payload's metadata did not open with its versioned domain tag.
    BadPayloadMetadataDomain,
    /// The payload-metadata schema is not the one this reader implements.
    UnsupportedPayloadMetadataSchema {
        /// Encoded major version.
        major: u16,
        /// Encoded minor version.
        minor: u16,
    },
    /// A section descriptor claims a skip permission its purpose does not have.
    ///
    /// The disposition is carried for a reader that does not recognize the
    /// purpose. A reader that *does* recognize it owns the answer, so a
    /// descriptor disagreeing with it is asserting a skip permission rather
    /// than reporting one.
    SectionDispositionMismatch {
        /// Ordered section identifier.
        section: u32,
        /// Disposition tag the descriptor declared.
        declared: u8,
        /// Disposition tag the recognized purpose carries.
        expected: u8,
    },
    /// A section's declared content schema is not the one its purpose carries.
    UnsupportedSectionSchema {
        /// Ordered section identifier.
        section: u32,
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// A section reference names a section whose governed purpose is wrong.
    ///
    /// Resolving a reference to an existing section is not enough. A payload's
    /// compilation subject, its object bytes, and a variant's kernel-program
    /// subject are three governed purposes, and each is a well-formed section
    /// with a verifying digest. Reading one as another would load an artifact
    /// whose executable half had been replaced by another section of its own
    /// envelope, with no framing or integrity check failing.
    SectionPurposeMismatch {
        /// Ordered section identifier the reference named.
        section: u32,
        /// Governed purpose tag the reference requires.
        expected: u8,
        /// Governed purpose tag the named section carries.
        actual: u8,
    },
    /// A carried payload's declared digest is not the identity of its subject.
    ///
    /// The descriptor's content digest is re-derived from the exact
    /// payload-metadata bytes on every decode, so a payload cannot claim a
    /// compilation subject it does not carry.
    PayloadIdentityMismatch {
        /// Ordered payload identifier.
        payload: u32,
    },
    /// The declared total length disagreed with the supplied byte run.
    TotalLengthMismatch {
        /// Length the header declared.
        declared: u64,
        /// Length actually supplied.
        actual: u64,
    },
    /// The envelope framing format is not the one this reader implements.
    UnsupportedEnvelopeFormat {
        /// Encoded major version.
        major: u16,
        /// Encoded minor version.
        minor: u16,
    },
    /// The canonical byte-encoding profile is not the one this reader implements.
    UnsupportedCanonicalEncoding {
        /// Encoded major version.
        major: u16,
        /// Encoded minor version.
        minor: u16,
    },
    /// The neutral manifest schema is not the one this reader implements.
    UnsupportedManifestSchema {
        /// Encoded major version.
        major: u16,
        /// Encoded minor version.
        minor: u16,
    },
    /// One governed component schema is not one this reader implements.
    UnsupportedComponentSchema {
        /// Component whose schema disagreed.
        component: ComponentSchemaKind,
        /// Encoded major version.
        major: u16,
        /// Encoded minor version.
        minor: u16,
    },
    /// The digest algorithm tag is not one this reader implements.
    ///
    /// A reader never infers an algorithm from a digest width.
    UnsupportedDigestAlgorithm {
        /// Encoded algorithm tag.
        tag: u8,
    },
    /// The envelope requires a feature this reader does not implement.
    UnsupportedRequiredFeature {
        /// Governed feature key that was required.
        feature: String,
    },
    /// A governed parser or encoder budget was exceeded.
    Limit {
        /// Governed resource.
        resource: CodecLimitKind,
        /// Attempted quantity.
        actual: u64,
        /// Maximum admitted quantity.
        limit: u64,
    },
    /// The manifest bytes do not match the digest the header declares.
    ManifestDigestMismatch,
    /// One framed section does not match the digest its descriptor declares.
    SectionDigestMismatch {
        /// Ordered section identifier.
        section: u32,
    },
    /// One framed section length disagrees with its descriptor.
    SectionLengthMismatch {
        /// Ordered section identifier.
        section: u32,
        /// Length the descriptor declared.
        declared: u64,
        /// Length the framing supplied.
        framed: u64,
    },
    /// The header's section count disagrees with the manifest's section table.
    SectionCountMismatch {
        /// Count the header declared.
        header: usize,
        /// Count the manifest described.
        manifest: usize,
    },
    /// A framed section identifier is not its canonical position.
    NonCanonicalSectionId {
        /// Ordered position in the framing.
        position: usize,
        /// Identifier the framing declared.
        declared: u32,
    },
    /// A set-meaning collection is not in canonical order.
    NonCanonicalOrder {
        /// Collection that was out of order.
        subject: OrderedSubject,
    },
    /// A set-meaning collection repeats an item.
    DuplicateItem {
        /// Collection that repeated an item.
        subject: OrderedSubject,
    },
    /// A payload that declares no platform SDK stated a platform field anyway.
    ///
    /// A canonical-form rejection rather than a completeness one, and the
    /// direction matters: this reports a *stated* field where the declared shape
    /// owes none, while [`ArtifactBuildError::IncompletePayloadProvenance`]
    /// reports an owed field left empty.
    ///
    /// It exists because the unversioned shape writes the versioned shape's
    /// field positions as pinned zeroes and appends a tag after the record —
    /// which is what let the platform block be added without moving any
    /// already-encodable payload's bytes. Accepting a tagged encoding that
    /// filled those positions would give one record two spellings, and payload
    /// identity is exactly those bytes. Normalizing the extra values away would
    /// be worse than refusing: it would silently discard a producer's claim.
    PlatformFieldWithoutPlatform {
        /// The platform field the encoding stated.
        field: ProvenanceField,
    },
    /// A variant's stated execution order is not a permutation of its entries.
    ///
    /// The order must name every entry exactly once. A short, long, or repeating
    /// order would leave a stage unsequenced or sequenced twice, and a consumer
    /// following it would dispatch a program the artifact does not describe.
    StageOrderNotAPermutation {
        /// Routing rank of the variant whose order is malformed.
        variant: u64,
        /// Executable entries the variant declares.
        entries: u64,
        /// Positions the stated order names.
        stated: u64,
    },
    /// A stage dependency edge is not discharged by the stated execution order.
    ///
    /// The edge names an obligation the packaged program proved — the successor
    /// reads what the predecessor writes, or reuses storage it released — so an
    /// order that runs the successor first is not a different valid schedule but
    /// a contradiction of the artifact's own dependency graph.
    StageDependencyOutOfOrder {
        /// Routing rank of the variant carrying the edge.
        variant: u64,
        /// Entry position that must precede.
        predecessor: u64,
        /// Entry position that must follow.
        successor: u64,
    },
    /// A stage dependency edge orders an entry against itself.
    StageDependencyOnItself {
        /// Routing rank of the variant carrying the edge.
        variant: u64,
        /// Entry position naming itself on both sides.
        entry: u64,
    },
    /// Re-encoding the fully understood manifest did not reproduce its bytes.
    NonCanonicalManifest,
    /// A framed section is referenced by no plan variant.
    ///
    /// An unreferenced section changes the envelope's bytes without changing
    /// the artifact's identity, which would give one artifact two byte
    /// identities.
    UnreferencedSection {
        /// Ordered section identifier.
        section: u32,
    },
    /// A binding addressing program output storage names no output.
    ///
    /// An empty list would read as "bind nothing" to a slot the kernel writes
    /// through, which is a silently unwritten output rather than a refusal.
    EmptyBindingTarget,
    /// A binding's target names an interface entry the artifact does not declare.
    ///
    /// The target is the one dispatch fact a decoder cannot re-derive, because
    /// the program that established it is carried only as identity bytes. What
    /// *is* decidable from the manifest alone is that the name it uses exists,
    /// and checking it is what stops a forged envelope directing a slot at a
    /// buffer the interface never mentions.
    UnknownBindingTargetKey {
        /// Governed interface key the target named.
        key: String,
        /// Whether the target claimed a program input rather than an output.
        input: bool,
    },
    /// A logical interface entry has no components or repeats a component role.
    MalformedInterfaceComponents,
    /// A binding names a role absent from its target interface value.
    UnknownBindingTargetComponent {
        /// Stable component role, or `None` for a dense singleton.
        role: Option<u32>,
    },
    /// A binding's carrier, encoding, or access type disagrees with its interface component.
    BindingComponentMismatch,
    /// A binding's kernel access type is incompatible with its physical storage.
    BindingAccessTypeMismatch,
    /// A carried payload maps no backend entry for an executable entry it realizes.
    ///
    /// The mapping is what turns a neutral backend entry key into the symbol a
    /// loader resolves, so an entry whose key is unmapped is one a consumer
    /// holding only these bytes cannot dispatch. Refusing at load is the
    /// fail-closed form; returning a record with an unreachable entry would
    /// move the failure to the loader with less to say about it.
    UnmappedBackendEntry {
        /// Ordered payload identifier.
        payload: u32,
    },
    /// An entry mapping declares a different transport count than the entry's
    /// bindings plus live-extent operand rows.
    ///
    /// `transports[i]` is the backend transport slot ABI binding `i` occupies
    /// and the following slots are the live-extent operands in declaration
    /// order, so a shorter list leaves operands unplaceable and a longer one
    /// places operands that do not exist. Either way the correspondence a
    /// loader binds through is not total, which is not a thing to approximate.
    EntryTransportCardinality {
        /// Ordered payload identifier.
        payload: u32,
        /// ABI binding count the executable entry declares.
        bindings: usize,
        /// Live input-extent operand count the executable entry declares.
        extents: usize,
        /// Transport-slot count the payload's entry mapping declares.
        transports: usize,
    },
    /// A live-extent operand names an input the artifact interface does not declare.
    UnknownExtentOperandKey {
        /// Stable input key the row named.
        key: String,
    },
    /// A live-extent operand names an axis the bound input does not have.
    ExtentOperandAxis {
        /// Stable input key the row named.
        key: String,
        /// Axis the row named.
        axis: u32,
        /// Rank of the named input.
        rank: usize,
    },
    /// A live-extent operand row is not the unsigned quantity the Metal `eN` ABI binds.
    ExtentOperandType {
        /// Stable input key the row named.
        key: String,
        /// Axis the row named.
        axis: u32,
    },
    /// A live-extent operand row names an interface axis the artifact fixes.
    ///
    /// A fixed semantic axis must not acquire a caller-selected extent. The
    /// same combination is refused at construction as
    /// `ArtifactBuildError::ExtentOperandStaticAxis`; this is the re-proof on
    /// bytes no builder wrote.
    ///
    /// **Per-axis since `tiler.artifact-program.v21`, not blanket.** The
    /// decoded interface grammar used to carry only literal extents, so every
    /// axis a row could name was fixed and every row was refused by this rule.
    /// The grammar now spells each axis literal-or-symbol, so the rule asks the
    /// question it was always about: a row over a *literal* axis is refused and
    /// names that axis's one extent, while a row over a symbolic axis is the
    /// case the row exists for and passes to the association checks.
    ExtentOperandStaticAxis {
        /// Stable input key the row named.
        key: String,
        /// Axis the row named.
        axis: u32,
        /// The one extent the published interface fixes for that axis.
        extent: u64,
    },
    /// A declared interface axis named a scope or symbol name the shape
    /// vocabulary refuses to construct.
    InvalidInterfaceSymbol {
        /// Why the constructor refused.
        cause: tiler_ir::shape::ShapeEnvError,
    },
    /// A declared interface axis names a symbol the retained environment does
    /// not declare.
    ///
    /// The two spellings of one boundary — the per-axis interface symbol and
    /// the retained environment's declarations — must agree, and this is the
    /// direction in which the interface could name a symbol from nowhere. A
    /// consumer resolving that axis would have no binding to resolve it
    /// through, so the artifact is refused rather than loaded with an axis
    /// nothing can bind.
    UndeclaredInterfaceSymbol {
        /// Stable interface key whose axis named the symbol.
        key: String,
        /// Axis that named it.
        axis: u32,
        /// The scoped symbol the axis named, as `scope::name`.
        symbol: String,
    },
    /// Two different symbols name one axis: the one the environment roots there
    /// and the one the published interface spells there.
    ///
    /// The retained environment roots a symbol at an exact `(input, axis)`, and
    /// that input's interface entry independently names its axes. Where both
    /// name a *symbol* at the same axis it must be the same symbol; otherwise a
    /// consumer resolving the root binds a quantity the interface calls
    /// something else. Refused rather than resolved by precedence: silently
    /// preferring one spelling would let a producer publish a boundary its own
    /// environment contradicts.
    ///
    /// Deliberately narrower than "the rooted axis must be symbolic". An
    /// environment may root a symbol at a statically known dimension — the
    /// symbol is then simply determined there — and
    /// `tiler.artifact-program.v17` pins that such an artifact is
    /// representable and identity-bearing.
    RootedAxisDisagreement {
        /// Stable input key the environment roots at.
        key: String,
        /// Root axis the environment names.
        axis: u32,
        /// The scoped symbol the environment roots there, as `scope::name`.
        rooted: String,
        /// The scoped symbol the interface spells at that axis.
        declared: String,
    },
    /// A payload mapping places a live-extent operand on a slot that is not the
    /// next buffer index after the tensor table.
    ExtentOperandTransport {
        /// Ordered payload identifier.
        payload: u32,
        /// Operand position within the entry's extent list.
        operand: usize,
        /// Transport slot the mapping declared.
        declared: u32,
        /// Slot the accepted Metal `eN` ABI requires (`binding_count + ordinal`).
        expected: u32,
    },
    /// The declared required-feature set is not the one the content implies.
    ///
    /// The set is derived, never asserted, so a producer cannot understate what
    /// a reader must implement to use the artifact correctly.
    DeclaredFeatureMismatch,
    /// The identity the manifest declares is not the identity of its content.
    ///
    /// The manifest declares its identity as a digest under
    /// `tiler.artifact-envelope.identity-digest.v1`, so this compares the digest
    /// of the decoder's own derivation against the carried one. The refused set
    /// is the same one the carried preimage refused: the check has always been
    /// on whether a producer's two derivations of one artifact agree, never on
    /// the wire against the world, so no sibling variant separates a "digest
    /// disagreed" case from an "identity disagreed" one — there is one case.
    ArtifactIdentityMismatch,
    /// A second conditional subgroup block followed the complete first block.
    ///
    /// The resource carrier is a singleton. Treating a duplicate as the next
    /// field's length would hide a second claim inside an absurd text count,
    /// while accepting either block would leave two encodings for one model.
    DuplicateSubgroupRequirement,
    /// A subgroup subject was framed completely but its checked constructor
    /// rejected the stated width and transfer combination.
    InvalidSubgroupRealization {
        /// Typed rejection from the shared schedule vocabulary.
        cause: SubgroupRealizationError,
    },
    /// A closed enumeration presented a tag this reader does not implement.
    UnknownTag {
        /// Enumeration whose tag was rejected.
        subject: TagSubject,
        /// Rejected tag.
        tag: u8,
    },
    /// A cross-reference named a row outside its table.
    MissingReference {
        /// Table the reference names.
        subject: ReferenceSubject,
        /// Rejected index.
        index: u64,
    },
    /// An expression operand does not precede the node that uses it.
    ///
    /// The arena is acyclic by construction; an operand at or after its own
    /// node would make it a graph a reader could not evaluate in one pass.
    ExpressionOperandOrder {
        /// Node whose operand was rejected.
        node: u64,
        /// Rejected operand position.
        operand: u64,
    },
    /// An expression operand has the wrong value type for its operation.
    ExpressionOperandType {
        /// Node whose operand was rejected.
        node: u64,
        /// Value type the operation requires.
        expected: AbiType,
        /// Value type the operand has.
        actual: AbiType,
    },
    /// A conditional selection's branches disagree on value type.
    ExpressionSelectBranchType {
        /// Node whose branches disagreed.
        node: u64,
        /// Value type of the branch taken when the predicate holds.
        if_true: AbiType,
        /// Value type of the branch taken otherwise.
        if_false: AbiType,
    },
    /// A text run was not valid UTF-8.
    InvalidText,
    /// A governed artifact key was rejected by its own validating constructor.
    InvalidGovernedKey {
        /// Typed rejection from the key constructor.
        cause: ArtifactBuildError,
    },
    /// A structured operation key was rejected by its own validating constructor.
    InvalidOperationKey {
        /// Typed rejection from the shared semantic identity vocabulary.
        cause: TypeIdentityError,
    },
    /// A stable interface key was rejected by its own validating constructor.
    InvalidInterfaceKey {
        /// Typed rejection from the shared-IR key constructor.
        cause: BuildError,
    },
    /// A provider identity was rejected by its own validating constructor.
    InvalidProviderIdentity {
        /// Typed rejection from the shared-IR registry.
        cause: RegistryError,
    },
    /// A declared target environment was rejected by its own grammar.
    ///
    /// The neutral decoder's half of the provider-versioned declaration:
    /// generic bounds and the zero-major refusal. Semantic provider validation
    /// remains unavailable to a neutral decoder by design.
    InvalidTargetEnvironment {
        /// Typed rejection from the declaration grammar.
        cause: super::super::environment::TargetEnvironmentDeclarationError,
    },
    /// A declared interface shape was rejected by its own constructor.
    InvalidShape {
        /// Typed rejection from the shared shape vocabulary.
        cause: ShapeError,
    },
    /// A binding alignment was rejected by the shared checked constructor.
    InvalidAlignment {
        /// Typed rejection from the shared alignment vocabulary.
        cause: ByteAlignmentError,
    },
    /// A decoded row violates an artifact-model insertion-time rule.
    ModelRule {
        /// The model's own typed rejection.
        cause: Box<ArtifactBuildError>,
    },
    /// A decoded envelope violates a whole-artifact obligation.
    ModelObligation {
        /// The model's own typed diagnostic.
        cause: ArtifactDiagnostic,
    },
    /// The artifact model refused to derive an identity for the decoded content.
    IdentityDerivation {
        /// The model's own typed diagnostic.
        cause: ArtifactDiagnostic,
    },
    /// The framed retained shape environment did not decode.
    ///
    /// Canonical order, table closure, unknown source or relation tags, and
    /// identity-byte mismatch are all decided here before a view exists, so an
    /// unsupported future tag cannot become an ignored runtime row.
    RetainedShapeEnvironment {
        /// The subject's own typed rejection.
        cause: Box<ShapeEnvSubjectError>,
    },
    /// The framed delivered-realization record did not decode.
    ///
    /// Distinct from [`Self::ModelObligation`], which carries the record's
    /// disagreement with the *artifact* around it. This one is the record
    /// failing on its own terms — a bad domain, a non-canonical table, a
    /// dangling reference, an unknown family or provenance tag — before any
    /// cross-check could run.
    DeliveredRealization {
        /// The record codec's own typed rejection.
        cause: Box<RealizationCodecError>,
    },
}

impl fmt::Display for ArtifactCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ArtifactCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidGovernedKey { cause } => Some(cause),
            Self::InvalidTargetEnvironment { cause } => Some(cause),
            Self::InvalidOperationKey { cause } => Some(cause),
            Self::InvalidInterfaceKey { cause } => Some(cause),
            Self::InvalidInterfaceSymbol { cause } => Some(cause),
            Self::InvalidProviderIdentity { cause } => Some(cause),
            Self::InvalidShape { cause } => Some(cause),
            Self::InvalidAlignment { cause } => Some(cause),
            Self::InvalidSubgroupRealization { cause } => Some(cause),
            Self::ModelRule { cause } => Some(cause.as_ref()),
            Self::ModelObligation { cause } | Self::IdentityDerivation { cause } => Some(cause),
            Self::DeliveredRealization { cause } => Some(cause.as_ref()),
            Self::RetainedShapeEnvironment { cause } => Some(cause.as_ref()),
            Self::Truncated { .. }
            | Self::TrailingBytes { .. }
            | Self::TrailingManifestBytes { .. }
            | Self::BadMagic
            | Self::BadManifestDomain
            | Self::BadPayloadMetadataDomain
            | Self::UnsupportedPayloadMetadataSchema { .. }
            | Self::SectionPurposeMismatch { .. }
            | Self::SectionDispositionMismatch { .. }
            | Self::UnsupportedSectionSchema { .. }
            | Self::PayloadIdentityMismatch { .. }
            | Self::TotalLengthMismatch { .. }
            | Self::UnsupportedEnvelopeFormat { .. }
            | Self::UnsupportedCanonicalEncoding { .. }
            | Self::UnsupportedManifestSchema { .. }
            | Self::UnsupportedComponentSchema { .. }
            | Self::UnsupportedDigestAlgorithm { .. }
            | Self::UnsupportedRequiredFeature { .. }
            | Self::Limit { .. }
            | Self::ManifestDigestMismatch
            | Self::SectionDigestMismatch { .. }
            | Self::SectionLengthMismatch { .. }
            | Self::SectionCountMismatch { .. }
            | Self::NonCanonicalSectionId { .. }
            | Self::NonCanonicalOrder { .. }
            | Self::DuplicateItem { .. }
            | Self::PlatformFieldWithoutPlatform { .. }
            | Self::StageOrderNotAPermutation { .. }
            | Self::StageDependencyOutOfOrder { .. }
            | Self::StageDependencyOnItself { .. }
            | Self::NonCanonicalManifest
            | Self::UnreferencedSection { .. }
            | Self::EmptyBindingTarget
            | Self::UnknownBindingTargetKey { .. }
            | Self::MalformedInterfaceComponents
            | Self::UnknownBindingTargetComponent { .. }
            | Self::BindingComponentMismatch
            | Self::BindingAccessTypeMismatch
            | Self::UnmappedBackendEntry { .. }
            | Self::EntryTransportCardinality { .. }
            | Self::UnknownExtentOperandKey { .. }
            | Self::ExtentOperandAxis { .. }
            | Self::ExtentOperandType { .. }
            | Self::ExtentOperandStaticAxis { .. }
            | Self::UndeclaredInterfaceSymbol { .. }
            | Self::RootedAxisDisagreement { .. }
            | Self::ExtentOperandTransport { .. }
            | Self::DeclaredFeatureMismatch
            | Self::ArtifactIdentityMismatch
            | Self::DuplicateSubgroupRequirement
            | Self::UnknownTag { .. }
            | Self::MissingReference { .. }
            | Self::ExpressionOperandOrder { .. }
            | Self::ExpressionOperandType { .. }
            | Self::ExpressionSelectBranchType { .. }
            | Self::InvalidText => None,
        }
    }
}

impl From<ShapeEnvSubjectError> for ArtifactCodecError {
    fn from(cause: ShapeEnvSubjectError) -> Self {
        Self::RetainedShapeEnvironment {
            cause: Box::new(cause),
        }
    }
}

impl From<ArtifactBuildError> for ArtifactCodecError {
    fn from(cause: ArtifactBuildError) -> Self {
        Self::ModelRule {
            cause: Box::new(cause),
        }
    }
}

/// Rejects a quantity that exceeds a governed codec budget.
pub(super) fn codec_limit(
    actual: usize,
    limit: usize,
    resource: CodecLimitKind,
) -> Result<(), ArtifactCodecError> {
    if actual > limit {
        return Err(ArtifactCodecError::Limit {
            resource,
            actual: u64::try_from(actual).expect("supported usize fits u64"),
            limit: u64::try_from(limit).expect("supported usize fits u64"),
        });
    }
    Ok(())
}
