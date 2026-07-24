//! Typed errors for artifact-program construction and verification.
//!
//! Two error boundaries mirror the [`tiler_ir::program`] discipline.
//! Insertion-time [`ArtifactBuildError`] rejects locally decidable malformed
//! builder input and leaves the draft unchanged; the consuming
//! [`super::ArtifactProgramBuilder::build`] returns a recoverable
//! [`ArtifactVerificationError`] carrying the whole-artifact
//! [`ArtifactDiagnostic`] set and the intact builder.
//!
//! No variant erases its cause into a message: each names the rejected entity,
//! the exhausted resource with its attempted and permitted quantities, or the
//! expected and actual quantity a rule required.

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::ProviderIdentity;

use super::ArtifactProgramBuilder;
use super::expr::{AbiEvaluationError, AbiType, AvailabilityPhase};

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
    /// Selected capability-provider count of one artifact program.
    SelectedProviders,
    /// Available provider count of one compilation environment.
    EnvironmentProviders,
    /// Deferred feasibility predicate count of one plan variant.
    DeferredPredicates,
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

/// The artifact-model role an expression is required to serve.
///
/// A use site fixes both the value type an expression must have and the latest
/// availability phase its roots may require.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AbiExprUse {
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
    /// A deferred predicate named a query authority that was never selected.
    UnselectedDeferredAuthority {
        /// Rejected provider identity.
        provider: Box<ProviderIdentity>,
    },
    /// An identical backend payload descriptor is already declared.
    DuplicatePayload,
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
    /// An entry declared a different binding count than its kernel signature.
    BindingCardinality {
        /// Ordered entry position.
        entry: usize,
        /// Buffer-parameter count of the stage's bound kernel.
        expected: usize,
        /// Declared ABI binding count.
        actual: usize,
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
    /// A predicate was declared deferred at a phase that is already decided.
    ///
    /// A predicate decidable from the compile profile or the artifact's own
    /// evidence is proven or rejected before packaging; recording it as
    /// deferred would claim a runtime query that never happens.
    NonDeferredPredicatePhase {
        /// Rejected phase.
        phase: AvailabilityPhase,
    },
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
            Self::BuilderIdentityExhausted
            | Self::ForeignHandle { .. }
            | Self::InvalidHandle { .. }
            | Self::StructuralLimit { .. }
            | Self::EmptyKey { .. }
            | Self::KeyTooLong { .. }
            | Self::ProviderNotAvailable { .. }
            | Self::DuplicateSelectedProvider { .. }
            | Self::UnselectedDeferredAuthority { .. }
            | Self::DuplicatePayload
            | Self::OperandType { .. }
            | Self::SelectBranchType { .. }
            | Self::ExpressionType { .. }
            | Self::NonDeferredPredicatePhase { .. }
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
            | Self::BindingCardinality { .. }
            | Self::AccessibleBytesDisagreement { .. }
            | Self::LaunchDisagreement { .. }
            | Self::DuplicateVariant => None,
        }
    }
}

/// A shared-IR enum this crate must encode but cannot match exhaustively.
///
/// `tiler_ir::kernel::KernelType`, `tiler_ir::kernel::AddressSpace`, and
/// `tiler_ir::kernel::BufferAccess` are `#[non_exhaustive]`, so a widened
/// variant cannot break this crate's encoder at compile time the way ADR 0074
/// §3 intends. Rejecting the artifact is the only remaining fail-closed
/// behaviour: an unrecognized variant must never share identity bytes with a
/// recognized one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ForeignEnumSubject {
    /// A structured-kernel element type.
    KernelType,
    /// A structured-kernel governed address space.
    AddressSpace,
    /// A structured-kernel buffer access mode.
    BufferAccess,
}

impl fmt::Display for ForeignEnumSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

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
    /// Two entities produced the same canonical key, so identity is ambiguous.
    AmbiguousCanonicalKey {
        /// Category of the colliding entities.
        entity: ArtifactEntityKind,
    },
    /// A shared-IR enum presented a variant this encoder does not recognize.
    ///
    /// Unreachable while `tiler-ir`'s element-type, address-space, and
    /// buffer-access vocabularies are exactly the ones enumerated here, and it
    /// is therefore not covered by a test. It exists because those enums are
    /// `#[non_exhaustive]`: widening one cannot break this cross-crate encoder
    /// at compile time, so the only fail-closed behaviour left is to reject.
    UnrecognizedForeignVariant {
        /// Shared-IR enum whose variant was not recognized.
        subject: ForeignEnumSubject,
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
            Self::AmbiguousCanonicalKey { .. } => "ambiguous-canonical-key",
            Self::UnrecognizedForeignVariant { .. } => "unrecognized-foreign-variant",
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
