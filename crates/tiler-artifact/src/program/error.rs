//! Typed errors for artifact-program construction and verification.
//!
//! Two error boundaries mirror the [`tiler_ir::program`] discipline.
//! Insertion-time [`ArtifactBuildError`] rejects locally decidable malformed
//! builder input and leaves the draft unchanged; the consuming
//! [`super::ArtifactProgramBuilder::build`] returns a recoverable
//! [`ArtifactVerificationError`] carrying the whole-artifact
//! [`ArtifactDiagnostic`] set and the intact builder.
//!
//! [`RecordedArtifactIdentityError`] is a third and separate boundary: it
//! rejects a *host assertion* about which artifact a consumer expects, which is
//! neither an insertion nor a verification of anything this crate built.
//!
//! No variant erases its cause into a message: each names the rejected entity,
//! the exhausted resource with its attempted and permitted quantities, or the
//! expected and actual quantity a rule required.

use std::error::Error;
use std::fmt;

use tiler_ir::program::ValueRole;
use tiler_ir::semantic::ProviderIdentity;

use super::ArtifactProgramBuilder;
use super::expr::{
    AbiEvaluationError, AbiType, AvailabilityPhase, MAX_TARGET_PROPERTY_KEY_BYTES,
    TargetPropertyKeyError,
};
use super::model::ARTIFACT_DOMAIN_LABEL;
use super::requirement::{RouteRequirementError, RouteRequirementSubject};

/// An artifact-owned entity category used by typed handle and closure errors.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ArtifactEntityKind {
    /// One node of the ABI expression arena.
    Expression,
    /// One complete plan variant of the portfolio.
    Variant,
    /// One backend payload descriptor.
    Payload,
    /// One executable entry of a plan variant.
    Entry,
    /// One ABI binding of an executable entry.
    Binding,
    /// One selected capability provider.
    Provider,
    /// One live-device route requirement of a plan variant.
    RouteRequirement,
}

impl fmt::Display for ArtifactEntityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A governed structural resource in the artifact-program profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ArtifactLimitKind {
    /// Plan-variant count of one artifact program.
    Variants,
    /// Executable-entry count of one plan variant.
    Entries,
    /// ABI binding count of one executable entry.
    EntryBindings,
    /// Node count of the shared ABI expression arena.
    Expressions,
    /// Backend payload descriptor count of one artifact program.
    Payloads,
    /// Delivery-position count of one artifact program.
    DeliveryPositions,
    /// Selected capability-provider count of one artifact program.
    SelectedProviders,
    /// Available provider count of one compilation environment.
    EnvironmentProviders,
    /// Deferred feasibility predicate count of one plan variant.
    DeferredPredicates,
    /// Live-device route-requirement count of one plan variant.
    RouteRequirements,
    /// Launch precondition count of one executable entry.
    LaunchPreconditions,
    /// Bound input-axis extent count of one ABI fact environment.
    BoundInputExtents,
    /// Bound target-property count of one ABI fact environment.
    BoundTargetProperties,
    /// UTF-8 byte length of one governed key.
    GovernedKeyBytes,
    /// Byte length of one opaque identity received at a boundary.
    OpaqueIdentityBytes,
    /// Canonical identity bytes retained for one artifact program.
    IdentityBytes,
}

impl fmt::Display for ArtifactLimitKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A governed key or received opaque-identity subject of the artifact model.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ArtifactKeyKind {
    /// The governed backend family key of a payload.
    Backend,
    /// The governed executable-representation key of a payload.
    Representation,
    /// The governed declared target-profile key.
    TargetProfile,
    /// The governed feasibility rule-set key.
    FeasibilityRuleSet,
    /// The governed capability key one provider was selected for.
    Capability,
    /// The governed backend-scoped route-requirement key.
    RouteFeature,
    /// The governed target-property key an ABI expression root names.
    TargetProperty,
    /// The opaque backend entry key of one executable entry.
    BackendEntry,
    /// The opaque content digest of one backend payload.
    PayloadDigest,
    /// The opaque descriptor digest of one declared target profile.
    TargetProfileDescriptor,
}

impl fmt::Display for ArtifactKeyKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// One field of a carried payload's provenance record.
///
/// Which of these a payload owes is a property of the shape its provenance
/// declares rather than of a backend this crate knows. Every payload owes its
/// toolchain, target, family, and language, and a role and a version for every
/// tool component it lists. The last four are owed only by a payload that
/// declares [`PayloadPlatform::VersionedSdk`](super::PayloadPlatform::VersionedSdk);
/// a payload that declares
/// [`PayloadPlatform::Unversioned`](super::PayloadPlatform::Unversioned) owes
/// none of them and may not state one.
///
/// Compiler and linker flags are deliberately absent. Their order is meaning
/// and an empty list is a legitimate invocation, so no flag is owed and an
/// emptiness rule over them would reject real compilations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProvenanceField {
    /// The governed key of the toolchain family that produced the payload.
    Toolchain,
    /// The normalized target the payload was compiled for.
    Target,
    /// The artifact family the payload belongs to.
    Family,
    /// The source language standard the payload was compiled under.
    Language,
    /// The governed role of one listed tool component.
    ToolComponentRole,
    /// The reported version of one listed tool component.
    ToolComponentVersion,
    /// The requested platform deployment minimum.
    DeploymentMinimum,
    /// The canonical selector of the SDK the payload was compiled against.
    SdkName,
    /// The canonical version of that SDK.
    SdkVersion,
    /// The build identifier of that SDK.
    SdkBuild,
}

impl fmt::Display for ProvenanceField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// The artifact-model role an expression is required to serve.
///
/// A use site fixes both the value type an expression must have and the latest
/// availability phase its roots may require.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AbiExprUse {
    /// The first addressed byte of one ABI binding, within the value it binds.
    AccessibleOffset,
    /// The accessible byte range of one ABI binding.
    AccessibleBytes,
    /// The total launch thread count of one executable entry.
    LaunchThreads,
    /// The per-workgroup thread count of one executable entry.
    ThreadsPerWorkgroup,
    /// One launch-instance precondition of an executable entry.
    LaunchPrecondition,
    /// The applicability guard of one plan variant.
    ApplicabilityGuard,
    /// One deferred feasibility predicate of a plan variant.
    DeferredPredicate,
}

impl fmt::Display for AbiExprUse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// Failure during one transactional artifact-program builder insertion.
///
/// Every variant names a locally decidable well-formedness rule. Whole-artifact
/// closure, provenance, and identity obligations are a separate boundary
/// reported as an [`ArtifactDiagnostic`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArtifactBuildError {
    /// No fresh builder ownership identity remained.
    BuilderIdentityExhausted,
    /// A builder-owned handle came from another builder.
    ForeignHandle {
        /// Category of rejected handle.
        entity: ArtifactEntityKind,
    },
    /// A builder-owned handle did not identify a live entity.
    InvalidHandle {
        /// Category of rejected handle.
        entity: ArtifactEntityKind,
    },
    /// A replayed arena named a position outside the arena it came from.
    ///
    /// Raised by `ArtifactProgramBuilder::adopt_abi`. A verified program's arena
    /// cannot contain one, so this reports a caller replaying an arena and a
    /// root list that do not belong together rather than a malformed program.
    ExpressionOutOfRange {
        /// The out-of-range source position.
        position: u32,
    },
    /// A governed construction resource exceeded its limit.
    StructuralLimit {
        /// Governed resource.
        resource: ArtifactLimitKind,
        /// Attempted quantity.
        actual: usize,
        /// Maximum admitted quantity.
        limit: usize,
    },
    /// A governed key or received identity was empty.
    EmptyKey {
        /// Rejected key subject.
        kind: ArtifactKeyKind,
    },
    /// A governed key or received identity exceeded its byte bound.
    KeyTooLong {
        /// Rejected key subject.
        kind: ArtifactKeyKind,
        /// Attempted byte length.
        bytes: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
    /// A governed key carried a byte outside the governed-key alphabet.
    ///
    /// Raised only for a governed key, never for a received opaque identity:
    /// those are another authority's derived bytes, which this crate carries
    /// rather than spells. `super::keys` states the alphabet and why it is the
    /// same one `tiler_compiler::target::TargetProfileKey` admits.
    NoncanonicalKeyByte {
        /// Rejected key subject.
        kind: ArtifactKeyKind,
        /// Zero-based byte offset of the refused byte.
        index: usize,
        /// Refused byte value.
        value: u8,
    },
    /// A provider was selected that the compilation environment never offered.
    ///
    /// An artifact may only attribute work to authority it was actually given.
    ProviderNotAvailable {
        /// Rejected provider identity.
        provider: Box<ProviderIdentity>,
    },
    /// An identical provider selection is already recorded.
    DuplicateSelectedProvider {
        /// Repeated provider identity.
        provider: Box<ProviderIdentity>,
    },
    /// An identical backend payload descriptor is already declared.
    DuplicatePayload,
    /// A carried payload's provenance left a field its declared shape owes empty.
    ///
    /// Raised where the payload's compilation identity is derived, so a subject
    /// that does not state what it claims to state has no identity rather than a
    /// weaker one. The same rule runs again on decode, because an artifact's
    /// bytes arrive from a producer this process never ran.
    ///
    /// The field is named rather than the backend: a backend states which
    /// provenance shape describes its toolchain, and the shape is what fixes the
    /// obligation. That is the whole of what generalizing this record beyond the
    /// Apple-shaped one changed — a Metal payload owes every field it owed
    /// before, and a backend with no SDK owes none of the four SDK-shaped ones
    /// instead of minting values with no referent on its target.
    IncompletePayloadProvenance {
        /// The owed field that carried no value.
        field: ProvenanceField,
    },
    /// An operand had the wrong value type for the operation applied to it.
    OperandType {
        /// Value type the operation requires.
        expected: AbiType,
        /// Value type the operand has.
        actual: AbiType,
    },
    /// A conditional selection's two branches disagree on value type.
    SelectBranchType {
        /// Value type of the branch taken when the predicate holds.
        if_true: AbiType,
        /// Value type of the branch taken otherwise.
        if_false: AbiType,
    },
    /// An expression had the wrong value type for the artifact use site.
    ExpressionType {
        /// Use site that rejected the expression.
        use_site: AbiExprUse,
        /// Value type the use site requires.
        expected: AbiType,
        /// Value type the expression has.
        actual: AbiType,
    },
    /// An expression root is unavailable at the phase its use site evaluates in.
    RootPhaseEscape {
        /// Use site that rejected the root.
        use_site: AbiExprUse,
        /// Earliest phase at which the root can be read.
        available_at: AvailabilityPhase,
        /// Latest phase the use site admits.
        admitted_through: AvailabilityPhase,
    },
    /// A size or launch expression named a root outside the bound interface.
    ///
    /// The bounded profile requires accessible ranges and launch geometry to be
    /// computable from the bound semantic environment alone, before any
    /// device-dependent query.
    NonInterfaceRoot {
        /// Use site that rejected the root.
        use_site: AbiExprUse,
    },
    /// A plan variant realizes a different semantic graph than the artifact's.
    SemanticSubjectMismatch,
    /// A plan variant does not publish the artifact's named interface.
    ///
    /// Either it fails to materialize a declared program input, fails to
    /// publish a declared output, binds one at a shape the semantic interface
    /// contradicts, or disagrees with a sibling variant's projection. A runtime
    /// must be able to bind every declared name against every routable variant.
    InterfaceMismatch,
    /// A plan variant declared a different numerical realization than its siblings.
    NumericalContractMismatch,
    /// A plan variant declared a different target profile than its siblings.
    TargetProfileMismatch,
    /// A plan variant declared a different entry count than its program has stages.
    EntryCardinality {
        /// Stage count of the variant's verified kernel program.
        expected: usize,
        /// Declared executable-entry count.
        actual: usize,
    },
    /// An executable entry names no backend payload at all.
    ///
    /// An entry is realized once per delivery position, and an artifact declares
    /// at least one position: every entry is executable on the consumer targets
    /// the artifact was built for, or the artifact carries an entry no consumer
    /// can dispatch. Distinct from [`Self::DeliveryCardinality`] because the
    /// first entry declared has no sibling to disagree with, so an empty
    /// realization there would otherwise establish an artifact with zero
    /// positions rather than be refused.
    EmptyDelivery {
        /// Ordered entry position.
        entry: usize,
    },
    /// An entry names a different payload count than the artifact's delivery positions.
    ///
    /// The count is fixed by the first entry any variant declares and every
    /// later entry must agree, because a consumer resolves one delivery position
    /// for the whole artifact: an entry with fewer positions would leave that
    /// consumer with no object for an entry its route must dispatch, and one
    /// with more would carry a payload no position selects.
    DeliveryCardinality {
        /// Ordered entry position.
        entry: usize,
        /// Delivery-position count the artifact already established.
        expected: usize,
        /// Payload count this entry declared.
        actual: usize,
    },
    /// An entry declared a different binding count than its kernel signature.
    BindingCardinality {
        /// Ordered entry position.
        entry: usize,
        /// Buffer-parameter count of the stage's bound kernel.
        expected: usize,
        /// Declared ABI binding count.
        actual: usize,
    },
    /// A binding's addressed value has no interface reference the artifact can carry.
    ///
    /// The program role and the value's origin disagree about whether the bytes
    /// enter across the public interface: an externally bound tensor recorded as
    /// an internal temporary would be allocated by a loader instead of bound,
    /// and an internally produced value recorded as a program input would be
    /// bound from host data the plan never reads.
    ///
    /// Unreachable for a verified program, and therefore not covered by a test.
    /// `tiler_ir::program::KernelProgramBuilder::push_value`'s `check_origin`
    /// admits exactly `(ProgramInput, Input)`, `(Internal, Temporary)` and
    /// `(Internal, Output)` and rejects the other three pairs with
    /// `KernelProgramBuildError::ValueRoleOrigin`. This exists because that
    /// correspondence is enforced by another crate's builder rather than by a
    /// type this one can match on, so refusing is the only fail-closed
    /// behaviour left if the guarantee ever moves.
    ///
    /// The artifact layer was once in this same position for `KernelType`,
    /// `AddressSpace`, and `BufferAccess`, and it no longer is: those enums
    /// dropped `#[non_exhaustive]`, so widening one is a build error at this
    /// crate's encoder rather than a run-time refusal. That remedy is
    /// unavailable here, because the constraint is a *builder rule* in another
    /// crate rather than a variant set this one could match exhaustively.
    UnnameableBindingTarget {
        /// Ordered entry position.
        entry: usize,
        /// Ordered binding position within that entry.
        binding: usize,
        /// Program role the addressed value declares.
        role: ValueRole,
        /// Whether the addressed value originates at the program interface.
        external_origin: bool,
    },
    /// Two bindings of one entry address the same program-internal value.
    ///
    /// Internal storage reaches the artifact without a durable name, so two
    /// such bindings encode identically and a loader cannot tell one shared
    /// buffer from two independent ones. Allocating twice for one value is a
    /// silently wrong dispatch, so the artifact refuses to package what its
    /// record cannot distinguish.
    ///
    /// Scoped to one entry deliberately. Two *entries* sharing a temporary is
    /// what a temporary is for — the two-stage partial-window fixture packages
    /// and decodes exactly that — and refusing it would make every multi-stage
    /// plan unpackageable.
    ///
    /// Unreachable for a verified program at this maturity, and therefore not
    /// covered by a test. Two bindings of one entry are two accesses of one
    /// stage, and `KernelProgramBuilder::push_stage` pins each access's mode to
    /// its buffer parameter's, so of any two one reads and the other writes. A
    /// stage that both defined and read one value would need a data dependency
    /// from its defining stage to its reading stage — `verify_dependencies`
    /// rejects a read without one as `MissingDataDependency` — and
    /// `push_data_dependency` rejects a stage naming itself as
    /// `SelfDependency`. So the two accesses of one stage always address
    /// different values.
    ///
    /// It is retained rather than deleted for the reason
    /// [`Self::UnnameableBindingTarget`] is: the guarantee is another crate's
    /// builder rule rather than something this crate's types express, and an
    /// entry whose slots a loader cannot tell apart must fail closed if that
    /// rule ever moves.
    AliasedInternalBinding {
        /// Ordered entry position.
        entry: usize,
        /// Ordered position of the binding that repeats an earlier target.
        binding: usize,
        /// Ordered position of the earlier binding addressing the same value.
        aliases: usize,
    },
    /// A binding's accessible-byte expression disagrees with its addressed range.
    AccessibleBytesDisagreement {
        /// Ordered entry position.
        entry: usize,
        /// Ordered binding position within that entry.
        binding: usize,
        /// Byte count the program's byte view addresses.
        expected: u64,
        /// Byte count the declared expression computes.
        actual: u64,
    },
    /// A launch expression disagrees with the bound kernel's requirements.
    LaunchDisagreement {
        /// Ordered entry position.
        entry: usize,
        /// Quantity the bound kernel requires.
        expected: u64,
        /// Quantity the declared expression computes.
        actual: u64,
    },
    /// A statically evaluated size or launch expression failed.
    StaticEvaluation {
        /// Use site whose expression failed.
        use_site: AbiExprUse,
        /// Typed evaluation failure.
        cause: AbiEvaluationError,
    },
    /// A deferred target-property query names a phase this artifact profile cannot execute.
    UnsupportedDeferredQueryPhase {
        /// Unsupported query phase.
        phase: AvailabilityPhase,
    },
    /// A deferred implication requirement is outside the Boolean quantity domain.
    DeferredImplicationRequirementNotBoolean {
        /// Rejected required quantity.
        required: u64,
    },
    /// A deferred target-property query names no executable entry.
    DeferredQueryEntryOutOfRange {
        /// Rejected declared entry ordinal.
        entry: u32,
        /// Entry count of the variant.
        entries: usize,
    },
    /// A deferred predicate does not read exactly the target-property query it names.
    DeferredQueryPredicateMismatch,
    /// An entry declares a zero-thread launch without a zero-work policy.
    ZeroWorkPolicy {
        /// Ordered entry position.
        entry: usize,
    },
    /// The same deferred predicate, phase, and authority appear twice.
    DuplicateDeferredPredicate,
    /// The same launch precondition appears twice in one entry.
    DuplicateLaunchPrecondition {
        /// Ordered entry position.
        entry: usize,
    },
    /// A plan variant with the same program and applicability guard exists.
    DuplicateVariant,
    /// Two live-device route requirements of one variant constrain one subject.
    ///
    /// Contradictory rather than redundant: two rows naming one subject state
    /// two answers to one question, and nothing in the artifact can say which
    /// the producer meant.
    DuplicateRouteRequirementSubject {
        /// The subject both rows named.
        subject: Box<RouteRequirementSubject>,
    },
    /// A route requirement was rejected by the route-requirement vocabulary.
    InvalidRouteRequirement {
        /// Typed cause from that vocabulary.
        cause: RouteRequirementError,
    },
}

/// Classifies the ABI domain's own key rejection into this crate's vocabulary.
///
/// ADR 0068 gives `tiler-ir` the expression domain, including validating a
/// target-property key, and gives this crate failure classification. So the
/// domain rejects with its own typed error and this conversion is where that
/// becomes an artifact diagnostic — rather than the domain importing this
/// crate's error type, which would be the dependency inversion the ADR exists
/// to prevent.
impl From<TargetPropertyKeyError> for ArtifactBuildError {
    fn from(error: TargetPropertyKeyError) -> Self {
        match error {
            TargetPropertyKeyError::Empty => Self::EmptyKey {
                kind: ArtifactKeyKind::TargetProperty,
            },
            TargetPropertyKeyError::TooLong { bytes } => Self::KeyTooLong {
                kind: ArtifactKeyKind::TargetProperty,
                bytes,
                limit: MAX_TARGET_PROPERTY_KEY_BYTES,
            },
        }
    }
}

impl fmt::Display for ArtifactBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ArtifactBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StaticEvaluation { cause, .. } => Some(cause),
            Self::InvalidRouteRequirement { cause } => Some(cause),
            Self::BuilderIdentityExhausted
            | Self::ForeignHandle { .. }
            | Self::InvalidHandle { .. }
            | Self::ExpressionOutOfRange { .. }
            | Self::StructuralLimit { .. }
            | Self::EmptyKey { .. }
            | Self::KeyTooLong { .. }
            | Self::NoncanonicalKeyByte { .. }
            | Self::ProviderNotAvailable { .. }
            | Self::DuplicateSelectedProvider { .. }
            | Self::DuplicatePayload
            | Self::IncompletePayloadProvenance { .. }
            | Self::OperandType { .. }
            | Self::SelectBranchType { .. }
            | Self::ExpressionType { .. }
            | Self::UnsupportedDeferredQueryPhase { .. }
            | Self::DeferredImplicationRequirementNotBoolean { .. }
            | Self::DeferredQueryEntryOutOfRange { .. }
            | Self::DeferredQueryPredicateMismatch
            | Self::ZeroWorkPolicy { .. }
            | Self::DuplicateDeferredPredicate
            | Self::DuplicateLaunchPrecondition { .. }
            | Self::RootPhaseEscape { .. }
            | Self::NonInterfaceRoot { .. }
            | Self::SemanticSubjectMismatch
            | Self::InterfaceMismatch
            | Self::NumericalContractMismatch
            | Self::TargetProfileMismatch
            | Self::EntryCardinality { .. }
            | Self::EmptyDelivery { .. }
            | Self::DeliveryCardinality { .. }
            | Self::BindingCardinality { .. }
            | Self::UnnameableBindingTarget { .. }
            | Self::AliasedInternalBinding { .. }
            | Self::AccessibleBytesDisagreement { .. }
            | Self::LaunchDisagreement { .. }
            | Self::DuplicateRouteRequirementSubject { .. }
            | Self::DuplicateVariant => None,
        }
    }
}

/// Failure while stating recorded bytes as an expected artifact identity.
///
/// Its own boundary rather than an [`ArtifactBuildError`] variant, because
/// nothing is being built: no draft is amended, no entity is inserted, and the
/// caller holds no builder to recover. A consumer that reads an identity beside
/// cached bytes and finds it unusable is diagnosing its own recording, and the
/// three answers it can act on — nothing was recorded, the recording is beyond
/// what this build admits, and the recording is of something else — are all
/// here.
///
/// `#[non_exhaustive]` because a later check on a recorded assertion lands as a
/// new variant rather than by widening one of these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RecordedArtifactIdentityError {
    /// No bytes were recorded.
    ///
    /// Distinct from the domain rejection below: an empty recording is a
    /// producer that wrote nothing, not one that wrote the wrong thing.
    Empty,
    /// The recorded bytes exceed what any artifact identity may occupy.
    TooLong {
        /// Recorded byte length.
        bytes: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
    /// The leading frame is not this build's artifact-identity domain.
    ///
    /// The recorded bytes are some other identity, digest, or key, or an
    /// artifact identity from a superseded domain. Recognizing the domain is
    /// syntax and type separation and proves nothing about the remainder.
    ForeignDomain {
        /// Recorded byte length. The bytes themselves are not carried: the
        /// governed bound is 64 MiB, and an error is not a place to copy one.
        bytes: usize,
    },
}

impl fmt::Display for RecordedArtifactIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str(
                "artifact.recorded-identity: no bytes were recorded as the expected artifact \
                 identity",
            ),
            Self::TooLong { bytes, limit } => write!(
                formatter,
                "artifact.recorded-identity: {bytes} recorded byte(s) exceed the {limit}-byte \
                 bound on an artifact identity",
            ),
            Self::ForeignDomain { bytes } => write!(
                formatter,
                "artifact.recorded-identity: {bytes} recorded byte(s) do not lead with the \
                 `{ARTIFACT_DOMAIN_LABEL}` domain, so they are not an artifact identity this \
                 build can be asked about",
            ),
        }
    }
}

impl Error for RecordedArtifactIdentityError {}

/// One deterministic whole-artifact verification failure.
///
/// Each variant names an obligation proven by
/// [`super::ArtifactProgramBuilder::build`]. [`ArtifactDiagnostic::rule`]
/// returns the stable rule identifier a consumer can surface in an explanation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArtifactDiagnostic {
    /// The artifact packages no plan variant.
    EmptyPortfolio,
    /// The artifact attributes its plan to no selected provider.
    MissingSelectedProvider,
    /// An expression node is not reachable from any declared use site.
    UnusedExpression,
    /// A backend payload descriptor realizes no executable entry.
    UnusedPayload,
    /// Two executable entries claim the same backend entry of one payload.
    DuplicateBackendEntry,
    /// One backend payload realizes entries at two different delivery positions.
    ///
    /// A delivery position is what a consumer's build target resolves to, and
    /// each position is meant to be one backend object built for that target. A
    /// payload reached from two positions makes the artifact carry fewer objects
    /// than it declares positions, so two consumer targets would load one
    /// object while the artifact claims to have built one for each.
    ///
    /// The neutral layer cannot decide *which* target a payload was built for —
    /// that is a backend fact a producer holds — so this refuses the shape that
    /// makes the question unanswerable rather than answering it.
    AmbiguousPayloadDelivery {
        /// Canonical position of the payload reached from two delivery positions.
        payload: u32,
    },
    /// Two entities produced the same canonical key, so identity is ambiguous.
    AmbiguousCanonicalKey {
        /// Category of the colliding entities.
        entity: ArtifactEntityKind,
    },
    /// The fully encoded canonical identity exceeded its bound.
    IdentityLimit {
        /// Encoded byte count.
        bytes: usize,
        /// Maximum byte count.
        limit: usize,
    },
}

impl ArtifactDiagnostic {
    /// Returns the stable verification-rule identifier for this diagnostic.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::EmptyPortfolio => "empty-portfolio",
            Self::MissingSelectedProvider => "missing-selected-provider",
            Self::UnusedExpression => "unused-expression",
            Self::UnusedPayload => "unused-payload",
            Self::DuplicateBackendEntry => "duplicate-backend-entry",
            Self::AmbiguousPayloadDelivery { .. } => "ambiguous-payload-delivery",
            Self::AmbiguousCanonicalKey { .. } => "ambiguous-canonical-key",
            Self::IdentityLimit { .. } => "identity-limit",
        }
    }
}

impl fmt::Display for ArtifactDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.rule())
    }
}

impl Error for ArtifactDiagnostic {}

/// Recoverable failure from consuming whole-artifact verification.
///
/// Carries the deterministic diagnostics and returns the intact builder through
/// [`ArtifactVerificationError::into_parts`] so a caller can amend and retry
/// without rebuilding the draft.
#[derive(Debug)]
pub struct ArtifactVerificationError {
    pub(super) builder: Box<ArtifactProgramBuilder>,
    pub(super) diagnostics: Vec<ArtifactDiagnostic>,
}

impl ArtifactVerificationError {
    /// Returns all deterministic diagnostics in stable order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ArtifactDiagnostic] {
        &self.diagnostics
    }

    /// Recovers the intact builder and its diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (ArtifactProgramBuilder, Vec<ArtifactDiagnostic>) {
        (*self.builder, self.diagnostics)
    }
}

impl fmt::Display for ArtifactVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "artifact-program verification failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for ArtifactVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.diagnostics.first().map(|diagnostic| diagnostic as _)
    }
}

pub(super) fn invalid_handle(entity: ArtifactEntityKind, foreign: bool) -> ArtifactBuildError {
    if foreign {
        ArtifactBuildError::ForeignHandle { entity }
    } else {
        ArtifactBuildError::InvalidHandle { entity }
    }
}

pub(super) fn limit(
    actual: usize,
    limit: usize,
    resource: ArtifactLimitKind,
) -> Result<(), ArtifactBuildError> {
    if actual > limit {
        return Err(ArtifactBuildError::StructuralLimit {
            resource,
            actual,
            limit,
        });
    }
    Ok(())
}
