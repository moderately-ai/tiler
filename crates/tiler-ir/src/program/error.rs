//! Typed errors for kernel-program construction and verification.
//!
//! Two error boundaries mirror the [`crate::schedule`] and [`crate::kernel`]
//! discipline. Insertion-time [`KernelProgramBuildError`] rejects locally
//! decidable malformed builder input, leaving the draft unchanged; the
//! consuming [`super::KernelProgramBuilder::build`] returns a recoverable
//! [`KernelProgramVerificationError`] carrying the whole-program
//! [`KernelProgramDiagnostic`] set and the intact builder.
//!
//! No variant erases its cause into a message: each names the rejected entity,
//! the exhausted resource with its attempted and permitted quantities, or the
//! expected and actual quantity a rule required.

use std::error::Error;
use std::fmt;

use crate::kernel::KernelType;
use crate::schedule::TensorRole;
use crate::semantic::{EncodedComponentRole, InputKey, OutputKey};
use crate::shape::Axis;

use super::KernelProgramBuilder;
use super::abi::{AbiEvaluationError, AbiType, AvailabilityPhase};
use super::alignment::{AlignmentGuarantee, AlignmentRequirement};
use super::model::{
    AllocationOwnership, MemorySpace, RoutingCommitState, SemanticOccurrence, StageAccessMode,
    StorageEncoding, StorageScalar, ValueRole,
};

/// One site at which a kernel program uses an ABI expression.
///
/// A use site fixes the value type an expression must produce, the latest
/// availability phase its roots may require, and whether it must be computable
/// from the bound semantic interface alone.
///
/// This names the use sites of a *program*. It is deliberately not
/// `tiler_artifact::program::AbiExprUse`, which names the use sites of an
/// artifact *variant* — that vocabulary additionally covers launch
/// preconditions and deferred feasibility predicates, neither of which a
/// target-neutral program owns. The two enumerations share three spellings and
/// are not the same subject; this crate is downstream of neither, so it names
/// the other in text rather than linking to it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProgramAbiUse {
    /// The guard deciding whether this program may be routed to.
    ApplicabilityGuard,
    /// The byte count one stage access may address through its view.
    AccessibleBytes,
    /// The total launch thread count of one stage.
    GridThreads,
    /// The per-workgroup thread count of one stage.
    ThreadsPerWorkgroup,
}

impl fmt::Display for ProgramAbiUse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A program-owned entity category used by typed handle and ambiguity errors.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProgramEntityKind {
    /// One program stage.
    Stage,
    /// One materialized program value.
    Value,
    /// One program storage allocation.
    Allocation,
    /// One byte view of a materialized value.
    View,
    /// One node of the program's ABI expression arena.
    AbiExpression,
}

impl fmt::Display for ProgramEntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A governed structural resource in the kernel-program profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProgramLimitKind {
    /// Stage count of one program.
    Stages,
    /// Materialized value count of one program.
    Values,
    /// Allocation count of one program.
    Allocations,
    /// View count of one program.
    Views,
    /// Dependency-edge count of one program.
    Dependencies,
    /// Split-reduction contract count of one program.
    PartialReductions,
    /// Publishing-copy contract count of one program.
    PublishingCopies,
    /// Staged-realization contract count of one program.
    StagedRealizations,
    /// Named-output count of one program.
    Outputs,
    /// Access count of one stage.
    StageAccesses,
    /// Covered semantic occurrence count of one stage.
    StageCoverage,
    /// ABI expression arena node count of one program.
    AbiExpressions,
    /// Routing-commit transition count of one program.
    RoutingCommitTransitions,
    /// Canonical identity bytes retained for one program.
    IdentityBytes,
}

impl fmt::Display for ProgramLimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Failure during one transactional kernel-program builder insertion.
///
/// Every variant names a locally decidable well-formedness rule. Whole-program
/// coverage, dependency, lifetime, and storage obligations are a separate
/// boundary reported as a [`KernelProgramDiagnostic`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelProgramBuildError {
    /// No fresh builder ownership identity remained.
    BuilderIdentityExhausted,
    /// A builder-owned handle came from another builder.
    ForeignHandle {
        /// Category of rejected handle.
        entity: ProgramEntityKind,
    },
    /// A builder-owned handle did not identify a live entity.
    InvalidHandle {
        /// Category of rejected handle.
        entity: ProgramEntityKind,
    },
    /// A governed construction resource exceeded its limit.
    StructuralLimit {
        /// Governed resource.
        resource: ProgramLimitKind,
        /// Attempted quantity.
        actual: usize,
        /// Maximum admitted quantity.
        limit: usize,
    },
    /// A value claimed a program input the bound semantic program does not declare.
    UnknownProgramInput {
        /// Rejected interface key.
        key: InputKey,
    },
    /// A bound semantic interface extent names a declared `ShapeEnv` symbol.
    ///
    /// A kernel program's subject records the exact interface boundaries a
    /// physical realization must cover, and a covered boundary is a fixed
    /// quantity: the stage coverage, allocations, and views built over it are
    /// all sized. Refusing here is what makes "no symbolic program reaches a
    /// packaged artifact" a property of this type rather than a convention —
    /// every artifact is built from a verified kernel program, so a symbolic
    /// program cannot reach one and cannot ship with its shape-environment
    /// subject unrepresented in the artifact's three carried subjects.
    SymbolicInterfaceExtent {
        /// Rejected interface entry, named by its stable key.
        interface: String,
    },
    /// Two materialized values claimed the same program input.
    DuplicateProgramInput {
        /// Repeated interface key.
        key: InputKey,
    },
    /// A dense or component declaration disagreed with the semantic value type.
    UnexpectedComponentRole {
        /// Rejected role; `None` denotes a dense singleton declaration.
        role: Option<EncodedComponentRole>,
    },
    /// An internal temporary carried a component role without a logical-value group.
    UngroupedInternalComponent {
        /// Rejected stable component role.
        role: EncodedComponentRole,
    },
    /// An encoded interface type declared no physical components.
    EmptyEncodedComponentSet,
    /// A storage encoding cannot represent the declared physical scalar.
    StorageEncodingScalar {
        /// Rejected physical scalar.
        scalar: StorageScalar,
        /// Rejected encoding.
        encoding: StorageEncoding,
    },
    /// A kernel access type cannot address the declared physical storage.
    StorageAccessType {
        /// Physical scalar stored in the bytes.
        scalar: StorageScalar,
        /// Physical storage encoding.
        encoding: StorageEncoding,
        /// Kernel access type required by that physical contract.
        expected: KernelType,
        /// Kernel access type the producer declared.
        actual: KernelType,
    },
    /// A value's shape disagreed with the semantic interface entry it binds.
    InterfaceShapeMismatch {
        /// Entity whose declared shape was rejected.
        entity: ProgramEntityKind,
    },
    /// A value's role and origin disagree.
    ///
    /// A [`ValueRole::Input`] value is externally bound and must carry a
    /// program-input origin; a temporary or output is internally produced.
    ValueRoleOrigin {
        /// Rejected role.
        role: ValueRole,
    },
    /// A shape product or byte product exceeded `u64`.
    ElementCountOverflow,
    /// An allocation's alignment does not satisfy the bound value's requirement.
    AllocationAlignment {
        /// Alignment the value requires.
        required: AlignmentRequirement,
        /// Alignment the allocation provides.
        provided: AlignmentGuarantee,
    },
    /// A stage access addresses a view whose effective alignment does not
    /// satisfy the storage carrier's natural requirement.
    StageAccessAlignment {
        /// Ordered access position.
        position: usize,
        /// Natural alignment the carrier requires.
        required: AlignmentRequirement,
        /// Alignment the addressed view is statically guaranteed to provide.
        guaranteed: AlignmentGuarantee,
    },
    /// An allocation's memory space differs from the bound value's.
    AllocationMemorySpace {
        /// Memory space the value requires.
        required: MemorySpace,
        /// Memory space the allocation provides.
        provided: MemorySpace,
    },
    /// An allocation cannot hold the bound value's required bytes.
    AllocationCapacity {
        /// Bytes the value requires.
        required: u64,
        /// Bytes the allocation provides.
        capacity: u64,
    },
    /// An allocation's ownership contradicts the bound value's role.
    ///
    /// Externally owned storage binds program inputs; program-owned storage
    /// binds temporaries and outputs.
    AllocationOwnershipRole {
        /// Declared ownership.
        ownership: AllocationOwnership,
        /// Bound value role.
        role: ValueRole,
    },
    /// A view escaped the byte range of the value it addresses.
    ViewOutOfRange {
        /// First addressed byte.
        offset: u64,
        /// Addressed byte count.
        length: u64,
        /// Bytes the base value requires.
        value_bytes: u64,
    },
    /// A packed value was exposed through less than its complete byte range.
    PartialPackedView {
        /// First addressed byte.
        offset: u64,
        /// Addressed byte count.
        length: u64,
        /// Complete packed component byte count.
        value_bytes: u64,
    },
    /// A view with the same base value and byte window is already declared.
    ///
    /// Views are canonically deduplicated so consumers of one window share one
    /// view and the view arena stays a function of program content.
    DuplicateView,
    /// A covered occurrence is not an operation of the bound semantic program.
    CoverageOutOfRange {
        /// Rejected occurrence.
        occurrence: SemanticOccurrence,
        /// Operation count of the bound semantic program.
        operations: u32,
    },
    /// A semantic occurrence was covered more than once.
    ///
    /// Now that coverage carries evidence, a repeat is two refinement receipts
    /// claiming one occurrence. There is no reading under which that is
    /// harmless — either the two prove the same thing and one is redundant, or
    /// they disagree and the program does not say which one it rests on — so it
    /// is refused rather than resolved by keeping the last writer.
    DuplicateCoverage {
        /// Repeated occurrence.
        occurrence: SemanticOccurrence,
    },
    /// A coverage record's refinement was minted against another semantic graph.
    ForeignCoverageGraph {
        /// Occurrence the foreign receipt names in its own graph.
        occurrence: SemanticOccurrence,
    },
    /// A stage declared a different access count than its kernel signature.
    StageAccessArity {
        /// Buffer-parameter count of the bound kernel.
        expected: usize,
        /// Declared access count.
        actual: usize,
    },
    /// One stage access mode contradicts the kernel buffer it realizes.
    StageAccessMode {
        /// Ordered access position.
        position: usize,
        /// Mode the kernel buffer admits.
        expected: StageAccessMode,
        /// Declared mode.
        actual: StageAccessMode,
    },
    /// One stage access binds a value whose role cannot fill the buffer's.
    StageTensorRole {
        /// Ordered access position.
        position: usize,
        /// Tensor role the kernel buffer binds.
        expected: TensorRole,
        /// Program role of the bound materialized value.
        actual: ValueRole,
    },
    /// A stage buffer targeted the wrong semantic component role.
    StageComponentRole {
        /// Ordered buffer position.
        position: usize,
        /// Role required by the kernel signature.
        expected: Option<EncodedComponentRole>,
        /// Role carried by the materialized value.
        actual: Option<EncodedComponentRole>,
    },
    /// One stage access binds a value whose element type is not the buffer's.
    StageElementType {
        /// Ordered access position.
        position: usize,
        /// Element type the kernel buffer stores.
        expected: KernelType,
        /// Element type the bound value stores.
        actual: KernelType,
    },
    /// One stage access addresses a different element count than its buffer.
    StageElementCount {
        /// Ordered access position.
        position: usize,
        /// Elements the kernel buffer addresses.
        expected: u64,
        /// Elements the declared view addresses.
        actual: u64,
    },
    /// A required live input extent did not resolve to exactly one program input.
    ///
    /// The count is over distinct `(InputKey, axis)` owners reached through the
    /// kernel's checked buffer/access correspondence. Zero would drop a
    /// schedule-derived precondition; more than one would guess which caller
    /// extent governs it. Both are refused.
    RequiredInputExtentBinding {
        /// Region-local tensor role whose extent requires the predicate.
        tensor: TensorRole,
        /// Required logical input axis.
        axis: Axis,
        /// Distinct program input-axis owners found.
        matches: usize,
    },
    /// A dependency named one stage as both predecessor and successor.
    SelfDependency,
    /// An identical dependency edge is already declared.
    DuplicateDependency,
    /// A partial tensor is already split by another declared reduction.
    DuplicatePartialReduction,
    /// A published value is already written by another declared publishing copy.
    ///
    /// Two copies naming one published value would leave each unprovable against
    /// the other, exactly as two splits over one partial tensor would.
    DuplicatePublishingCopy,
    /// A consuming stage already continues that occurrence's realization.
    ///
    /// The key is the pair, not the stage: one dispatch may legitimately
    /// continue the realizations of several occurrences it claims stages of, and
    /// a fused staged region is exactly that. What has no reading is one
    /// dispatch continuing *one* occurrence twice — the two declarations would
    /// name two handed values for one stage boundary, leaving which value
    /// carries the realization undecided.
    DuplicateStagedRealization,
    /// A named output used a key the bound semantic program does not declare.
    UnknownOutputKey {
        /// Rejected interface key.
        key: OutputKey,
    },
    /// A named output key was declared more than once.
    DuplicateOutput {
        /// Repeated interface key.
        key: OutputKey,
    },
    /// A named output published a value whose role is not [`ValueRole::Output`].
    OutputValueRole {
        /// Rejected role.
        role: ValueRole,
    },
    /// An ABI operand does not have the value type its operation requires.
    AbiOperandType {
        /// Value type the operation requires.
        expected: AbiType,
        /// Value type the operand produces.
        actual: AbiType,
    },
    /// The two branches of an ABI conditional disagree in value type.
    AbiSelectBranchType {
        /// Value type of the branch taken when the condition holds.
        if_true: AbiType,
        /// Value type of the branch taken otherwise.
        if_false: AbiType,
    },
    /// An ABI expression does not have the value type its use site requires.
    AbiUseType {
        /// Rejected use site.
        use_site: ProgramAbiUse,
        /// Value type the use site requires.
        expected: AbiType,
        /// Value type the expression produces.
        actual: AbiType,
    },
    /// An ABI expression names a root later than its use site admits.
    AbiRootPhaseEscape {
        /// Rejected use site.
        use_site: ProgramAbiUse,
        /// Earliest phase at which the whole subtree becomes readable.
        available_at: AvailabilityPhase,
        /// Latest phase the use site admits.
        admitted_through: AvailabilityPhase,
    },
    /// An ABI expression reads a target fact where only interface facts are admitted.
    AbiNonInterfaceRoot {
        /// Rejected use site.
        use_site: ProgramAbiUse,
    },
    /// An interface-only ABI expression could not be evaluated against the
    /// program's own declared input shapes.
    AbiStaticEvaluation {
        /// Rejected use site.
        use_site: ProgramAbiUse,
        /// Typed evaluation failure.
        cause: AbiEvaluationError,
    },
    /// A stage access declares an accessible range its own view contradicts.
    AccessibleBytesDisagreement {
        /// Ordered access position.
        position: usize,
        /// Bytes the declared view addresses.
        expected: u64,
        /// Bytes the declared expression computes.
        actual: u64,
    },
    /// A stage's launch declares a workgroup width its own kernel contradicts.
    ThreadsPerWorkgroupDisagreement {
        /// Width the bound kernel requires.
        expected: u64,
        /// Width the declared expression computes.
        actual: u64,
    },
    /// A second applicability guard was declared for one program.
    DuplicateApplicabilityGuard,
    /// A routing-commit transition did not continue the ordered lifecycle.
    RoutingCommitOutOfOrder {
        /// State the next transition must leave.
        expected: RoutingCommitState,
        /// State the declared transition leaves.
        actual: RoutingCommitState,
    },
    /// A routing-commit transition permits fallback at or after commit.
    ///
    /// Fallback is admissible only while the program has done no work a
    /// fallback would have to undo, which is exactly the transition leaving
    /// [`RoutingCommitState::Preflight`].
    RoutingCommitFallbackAfterCommit {
        /// State the rejected transition leaves.
        from: RoutingCommitState,
    },
}

impl fmt::Display for KernelProgramBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for KernelProgramBuildError {}

/// One deterministic whole-program verification failure.
///
/// Each variant names an obligation proven by
/// [`super::KernelProgramBuilder::build`].
/// [`KernelProgramDiagnostic::rule`] returns the stable rule identifier a
/// consumer can surface in an explanation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelProgramDiagnostic {
    /// The program declares no stage, value, allocation, or named output.
    EmptyProgram,
    /// The stages do not cover every operation of the bound semantic program.
    IncompleteCoverage {
        /// Distinct occurrences the stages cover.
        covered: u32,
        /// Operations the bound semantic program contains.
        required: u32,
    },
    /// An internally produced value has no defining stage.
    MissingWriter,
    /// An internally produced value has more than one defining stage.
    MultipleWriters,
    /// A stage writes an externally bound program input.
    ExternalValueWritten,
    /// A stage reads an internally produced value without a data dependency.
    MissingDataDependency,
    /// A dependency edge names a reason its two stages do not realize.
    MisattributedDependency,
    /// The stage dependency graph contains a cycle.
    DependencyCycle,
    /// Two values share an allocation that the first profile forbids aliasing.
    ForbiddenAlias,
    /// Two values sharing an allocation have overlapping logical lifetimes.
    ReuseLifetimeOverlap,
    /// Reused storage has no explicit handoff from the old value's final user.
    ReuseMissingHandoff,
    /// A user of the old value is not ordered before the new value's writer.
    ReuseLiveAlias,
    /// A declared value is never accessed by a stage or published as an output.
    UnusedValue,
    /// A declared view is never used by a stage access.
    UnusedView,
    /// A declared allocation binds no materialized value.
    UnusedAllocation,
    /// A semantic program output has no named program output.
    MissingNamedOutput,
    /// The published outputs do not follow the semantic interface's order.
    ///
    /// The published list *is* the program's ordered output interface, and that
    /// order belongs to the unforgeable semantic subject rather than to the
    /// producer: keys appear in the subject's declared order, each key's records
    /// are contiguous, and within one key the records follow the encoded
    /// contract's declared component order. A permuted or interleaved
    /// publication would otherwise be observable through
    /// [`VerifiedKernelProgram::outputs`](crate::program::VerifiedKernelProgram::outputs)
    /// while agreeing with the identity of the program it permuted.
    MisorderedNamedOutput {
        /// Declared position of the first record that leaves the interface order.
        position: usize,
    },
    /// An output-role value is not published as a named program output.
    UnboundOutputValue,
    /// A logical interface value did not materialize exactly its semantic components.
    IncompleteComponentSet,
    /// An encoded interface type declared no physical components.
    EmptyEncodedComponentSet,
    /// Two entities produced the same canonical key, so identity is ambiguous.
    AmbiguousCanonicalKey {
        /// Category of the colliding entities.
        entity: ProgramEntityKind,
    },
    /// The program declares no applicability guard.
    MissingApplicabilityGuard,
    /// An ABI expression is reachable from no use site of the program.
    ///
    /// Identity writes the reached arena once and names every use by canonical
    /// position, so a node no use reaches would be retained program state
    /// omitted by that traversal.
    UnreferencedAbiExpression,
    /// A stage covers no semantic occurrence and has no declared account.
    ///
    /// Three accounts exist, and all three are declarations rather than
    /// relaxations. A split reduction's final pass legitimately computes no
    /// operation of its own: the pass it combines already claims the reduction,
    /// and claiming it twice would double-cover the graph. A publishing copy's
    /// publisher is the same shape one fold up — the stage that computed the
    /// value already claims the occurrences, and the copy exists because
    /// [`ValueRole::Output`] is exclusive of the temporary a consumer reads
    /// across. A staged realization's consumer is the third: the occurrence it
    /// continues is claimed by the stage that began it, because coverage is an
    /// obligation of the occurrence and is discharged once. Any *other*
    /// uncovering stage is a dispatch the program cannot account for.
    ///
    /// [`ValueRole::Output`]: super::ValueRole::Output
    UncoveringStage,
    /// A split reduction's partial value is not initialized by its producer.
    ///
    /// Either no stage writes it, or the writer is not the stage the contract
    /// names — so the pass that reads the partials would read values some other
    /// stage produced, or none at all.
    PartialNotInitializedByProducer,
    /// A split reduction's result is not produced by its combiner.
    PartialResultNotProducedByCombiner,
    /// A split reduction's combiner never reads the partial values it names.
    PartialNotConsumedByCombiner,
    /// A split reduction stages its partials somewhere the program may not.
    ///
    /// Partials are internal to the split: staging them in an externally bound
    /// input or a published output would put a value the caller owns between
    /// the two passes.
    PartialNotMaterialized,
    /// A split reduction's partial extent is not its result extent per partition.
    PartialExtentMismatch,
    /// A split reduction covers no contributors, or its coverage overflows `u64`.
    PartialCoverageUnrepresentable,
    /// A publishing copy's source value is not initialized by the stage it names.
    ///
    /// Either no stage writes it, or the writer is not the source stage the
    /// contract names — so the publisher would copy values some other stage
    /// produced, or none at all.
    CopiedSourceNotInitializedBySourceStage,
    /// A publishing copy's publisher never reads the source value it names.
    CopiedSourceNotReadByPublisher,
    /// A publishing copy's published value is not written by its publisher.
    PublishedCopyNotWrittenByPublisher,
    /// A publishing copy publishes a value whose role is not [`ValueRole::Output`].
    ///
    /// The declaration exists to account for a dispatch that writes the program's
    /// *interface*; one writing a temporary is an ordinary covered pass and has
    /// no copy to declare.
    ///
    /// [`ValueRole::Output`]: super::ValueRole::Output
    PublishedCopyNotOutput,
    /// A publishing copy's two values do not carry the same element count.
    ///
    /// A copy publishes what it read. Two extents that disagree describe a
    /// reshape, a slice, or a reduction — none of which this declaration
    /// accounts for, and each of which is an operation some stage must cover.
    PublishedCopyExtentMismatch,
    /// A staged realization's handed value is not initialized by its producer.
    ///
    /// Either no stage writes it, or the writer is not the producing stage the
    /// contract names — so the consumer would continue from values some other
    /// stage produced, or none at all.
    HandedValueNotInitializedByProducer,
    /// A staged realization's consumer never reads the handed value it names.
    HandedValueNotReadByConsumer,
    /// A staged realization hands a value the program may not hand.
    ///
    /// The handed value is internal to the realization: handing an externally
    /// bound input or a published output would put a value the caller owns
    /// between two stages of one operation, exactly as staging a split's
    /// partials there would.
    HandedValueNotMaterialized,
    /// The declared staged realizations of one occurrence are not one ordered chain.
    ///
    /// A realization's stages run in order and each is computed once, so the
    /// declarations naming an occurrence must form an unbroken path from the
    /// stage that *covers* that occurrence — the one that began it — through one
    /// consumer at a time. A chain rooted at a stage that covers something else,
    /// two declarations continuing from one stage, a stage reached twice, and a
    /// declaration the walk never reaches all fail it, and each is a program
    /// whose later dispatches compute a stage nobody began, or one twice.
    StagedRealizationChainBroken,
    /// The routing-commit transitions do not span the whole ordered lifecycle.
    IncompleteRoutingCommitContract {
        /// Transitions the program declares.
        declared: usize,
        /// Transitions the ordered lifecycle requires.
        required: usize,
    },
    /// The fully encoded canonical identity exceeded its bound.
    IdentityLimit {
        /// Encoded byte count.
        bytes: usize,
        /// Maximum byte count.
        limit: usize,
    },
}

impl KernelProgramDiagnostic {
    /// Returns the stable verification-rule identifier for this diagnostic.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::EmptyProgram => "empty-program",
            Self::IncompleteCoverage { .. } => "incomplete-coverage",
            Self::MissingWriter => "missing-writer",
            Self::MultipleWriters => "multiple-writers",
            Self::ExternalValueWritten => "external-value-written",
            Self::MissingDataDependency => "missing-data-dependency",
            Self::MisattributedDependency => "misattributed-dependency",
            Self::DependencyCycle => "dependency-cycle",
            Self::ForbiddenAlias => "forbidden-alias",
            Self::ReuseLifetimeOverlap => "reuse-lifetime-overlap",
            Self::ReuseMissingHandoff => "reuse-missing-handoff",
            Self::ReuseLiveAlias => "reuse-live-alias",
            Self::UnusedValue => "unused-value",
            Self::UnusedView => "unused-view",
            Self::UnusedAllocation => "unused-allocation",
            Self::MissingNamedOutput => "missing-named-output",
            Self::MisorderedNamedOutput { .. } => "misordered-named-output",
            Self::UnboundOutputValue => "unbound-output-value",
            Self::IncompleteComponentSet => "incomplete-component-set",
            Self::EmptyEncodedComponentSet => "empty-encoded-component-set",
            Self::AmbiguousCanonicalKey { .. } => "ambiguous-canonical-key",
            Self::MissingApplicabilityGuard => "missing-applicability-guard",
            Self::UnreferencedAbiExpression => "unreferenced-abi-expression",
            Self::UncoveringStage => "uncovering-stage",
            Self::PartialNotInitializedByProducer => "partial-not-initialized-by-producer",
            Self::PartialResultNotProducedByCombiner => "partial-result-not-produced-by-combiner",
            Self::PartialNotConsumedByCombiner => "partial-not-consumed-by-combiner",
            Self::PartialNotMaterialized => "partial-not-materialized",
            Self::PartialExtentMismatch => "partial-extent-mismatch",
            Self::PartialCoverageUnrepresentable => "partial-coverage-unrepresentable",
            Self::CopiedSourceNotInitializedBySourceStage => {
                "copied-source-not-initialized-by-source-stage"
            }
            Self::CopiedSourceNotReadByPublisher => "copied-source-not-read-by-publisher",
            Self::PublishedCopyNotWrittenByPublisher => "published-copy-not-written-by-publisher",
            Self::PublishedCopyNotOutput => "published-copy-not-output",
            Self::PublishedCopyExtentMismatch => "published-copy-extent-mismatch",
            Self::HandedValueNotInitializedByProducer => "handed-value-not-initialized-by-producer",
            Self::HandedValueNotReadByConsumer => "handed-value-not-read-by-consumer",
            Self::HandedValueNotMaterialized => "handed-value-not-materialized",
            Self::StagedRealizationChainBroken => "staged-realization-chain-broken",
            Self::IncompleteRoutingCommitContract { .. } => "incomplete-routing-commit-contract",
            Self::IdentityLimit { .. } => "identity-limit",
        }
    }
}

impl fmt::Display for KernelProgramDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for KernelProgramDiagnostic {}

/// Recoverable failure from consuming whole-program verification.
///
/// Carries the deterministic diagnostics and returns the intact builder through
/// [`KernelProgramVerificationError::into_parts`] so a caller can amend and
/// retry without rebuilding the draft.
#[derive(Debug)]
pub struct KernelProgramVerificationError {
    pub(super) builder: Box<KernelProgramBuilder>,
    pub(super) diagnostics: Vec<KernelProgramDiagnostic>,
}

impl KernelProgramVerificationError {
    /// Returns all deterministic diagnostics in stable order.
    #[must_use]
    pub fn diagnostics(&self) -> &[KernelProgramDiagnostic] {
        &self.diagnostics
    }

    /// Recovers the intact builder and its diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (KernelProgramBuilder, Vec<KernelProgramDiagnostic>) {
        (*self.builder, self.diagnostics)
    }
}

impl fmt::Display for KernelProgramVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "kernel-program verification failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}
impl Error for KernelProgramVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.diagnostics.first().map(|diagnostic| diagnostic as _)
    }
}

pub(super) fn invalid_handle(entity: ProgramEntityKind, foreign: bool) -> KernelProgramBuildError {
    if foreign {
        KernelProgramBuildError::ForeignHandle { entity }
    } else {
        KernelProgramBuildError::InvalidHandle { entity }
    }
}
