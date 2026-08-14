//! The public compiler boundary: compile a semantic program, read its plans.
//!
//! This boundary is **accepted** under ADR 0075 and ADR 0074 convention 7 —
//! `session::compile`, [`CompileRequest`] and its installation methods,
//! [`InstalledCapabilities`], [`Compilation`] and its accessors, and the
//! [`CompileFailureClass`] vocabulary on 2026-08-05, and [`compile_governed`]
//! on 2026-08-06 as the returned exclusion once its error type widened.
//! Acceptance is not stabilization: this is accepted pre-alpha vocabulary,
//! not a published API with compatibility obligations. It is the first
//! surface over which a caller outside this crate can compile anything at
//! all, and until it existed nothing downstream — MSL emission, offline
//! compilation, bundle assembly, execution — could be reached from a
//! producer, because this crate's `pipeline` module is private and both its
//! entry point and its request type are `pub(crate)`.
//!
//! # What this boundary deliberately is not
//!
//! It is scoped to *reaching execution*, not to being the compiler's finished
//! API.
//!
//! `prototype-public-compiler-api` carried seven deferred public-surface
//! questions, and they are now settled rather than left open. Six of the
//! answers are that the surface stays as narrow as it already is, and each is
//! recorded at the item it governs so a later reader finds a decision instead
//! of an omission: a trace is never serialized or embedded ([`ExplainReport`]);
//! the rendered form is deterministic and total but not a parse target, nothing
//! in it is redacted, and there is no retention control left to expose
//! ([`ExplainReport::render`]); public enums follow ADR 0074 convention 5's
//! clause test and never a parallel versioned schema view
//! ([`CompileFailureClass`]); every identity this boundary emits is canonical
//! bytes and never a digest ([`Compilation::target_profile_descriptor`]); the
//! compiler alone mints an evidence receipt, which is why none is reachable
//! here at all; and the renderer header's request qualifier is a correlation
//! label, never an identity.
//!
//! The seventh — report completeness — is the one that required a change.
//! A failed compilation now returns the complete trace the compiler had already
//! sealed, through [`CompileFailure::explain`]; a target refusal that precedes
//! that trace boundary instead retains recoverable typed detail through
//! [`TargetCompileFailure::refusal`].
//!
//! The general [`CompileRequest`] exposes the choices that now have more than
//! one validated value: numerical-contract preference, installed lowering
//! authority, and an ordered [`TargetRequest`]. Shape environment and budgets
//! remain governed because they still have no second public admissible value.
//! [`compile_governed`] is the single-target convenience spelling of that same
//! path.
//!
//! # What a caller gets
//!
//! One [`TargetCompilationResult`] per requested target profile. Successful
//! slots contain a [`Compilation`] carrying retained plan alternatives and the
//! policy's selection; refused slots retain their exact profile and typed
//! refusal. Both the fused and the materialized alternative are exposed rather
//! than the selected one alone, because the offline slice compiles the selected
//! program *and* keeps the materialized program as its numerical reference; a
//! selected-only surface could not express that.

mod realization;

pub(crate) use realization::DeliveredRealizationEvidence;
pub use realization::{
    DeliveredRealizationView, SelectedEvidence, SelectedObligation, SelectedScalarArithmetic,
};

use std::{fmt, sync::Arc};

use tiler_ir::kernel::VerifiedKernel;
use tiler_ir::numerics::ScalarArithmeticSubjectIdentity;
use tiler_ir::program::abi::{AvailabilityPhase, ExprNode, PreparedEntryTargetRequirement};
use tiler_ir::program::{StageRef, VerifiedKernelProgram};
use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::{ProviderIdentity, ResolvedValueType, SemanticProgram};

use crate::capability::FrozenLoweringCapabilityRegistry;
pub use crate::explain::VerifiedCompilationExplain;
use crate::explain::VerifiedExplainTrace;
use crate::physical_provider::InstalledPhysicalProviders;
use crate::pipeline::{
    CompilationProduct, CompileError, NoFeasiblePlanError, ProgramAlternative,
    ProgramAlternativeKind, TargetCompilationOutcome, compile_with_physical_providers,
};
use crate::program::KernelProgram;
/// The budget vocabulary a [`CompileFailureClass::BudgetExhausted`] refusal
/// names. Defined beside the deterministic budgets themselves and re-exported
/// here because `session` is the one module a caller reads a refusal through.
pub use crate::request::{BudgetRefusal, BudgetResource};
use crate::request::{
    CompilationRequest, CompilerCapabilitySnapshot, ContractRejection,
    DTypeDispatchRefusalDisposition, ExceptionalValueDimensionKind, LoweringProviderIdentity,
    MAX_NUMERICAL_CONTRACT_PREFERENCES as INTERNAL_MAX_NUMERICAL_CONTRACT_PREFERENCES,
    NumericalContractPreference, RequestError, StrictF32NumericalContract,
};
use crate::target::feasibility::FeasibilityRuleSetIdentity;
use crate::target::{
    TargetNumericalRefusalEvidence, TargetProfile, TargetProfileKey, TargetRequest,
};
use crate::{
    program::ProgramError,
    target::honourability::{
        DimensionBehaviour, HonouringMeans, NumericalDimension, NumericalRefusalEvidence,
    },
};
use tiler_ir::index::FrozenScalarRegistry;

/// Maximum number of numerical contracts in one caller preference list.
pub const MAX_NUMERICAL_CONTRACT_PREFERENCES: usize = INTERNAL_MAX_NUMERICAL_CONTRACT_PREFERENCES;

/// Which boundary refused a compilation.
///
/// ADR 0074 convention 1: a typed enumeration rather than a boxed error, so a
/// caller branches on the boundary that refused instead of matching on text.
/// The classes are the compiler's own and are deliberately coarse. A
/// post-verification refusal carries its exact attributed trace through
/// [`CompileFailure::explain`]; a pre-trace target refusal carries typed,
/// recoverable detail through [`TargetCompileFailure::refusal`].
///
/// # Why `#[non_exhaustive]` and not a versioned schema view
///
/// ADR 0074's amended convention 5 decides this per type by asking what an
/// out-of-crate wildcard arm would have to do. Here a consumer only classifies
/// partially or forwards the value; no consumer maps it totally onto a derived
/// value, and none matches it to decide what it supports. That is clause 5a, so
/// the attribute applies and a later class lands additively.
///
/// A parallel versioned schema view was considered and eliminated rather than
/// deferred. It is a second, hand-maintained description of this enum, and
/// nothing keeps the two in agreement — which is convention 3's argument
/// against encoding a projection of an enum instead of the enum: a projection
/// cannot fail closed when its source grows. It also buys compatibility, and
/// ADR 0075 records that Tom rejected the compatibility premise outright while
/// no crate in this workspace is publishable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompileFailureClass {
    /// The request itself is malformed and no build would admit it.
    ///
    /// An empty target set, a duplicated profile, an unstated numerical
    /// contract. The action is to fix the request, and no amount of installed
    /// capability changes that.
    InvalidRequest {
        /// Stable diagnostic key of the refusing check.
        rule: &'static str,
    },
    /// The program is valid and no installed capability compiles it.
    ///
    /// The program is not wrong and the request is not wrong; *this build* does
    /// not cover it. The action is to install a provider or wait for coverage,
    /// which is a different action from fixing a request — and one a caller
    /// acquired something to do about when out-of-crate capability installation
    /// landed.
    UnsupportedCapability {
        /// Stable diagnostic key of the refusing check.
        rule: &'static str,
    },
    /// No plan was feasible for a requested target profile.
    ///
    /// This includes hard target rejections and mixed or structural planning
    /// failures for which the compiler cannot attribute the empty portfolio
    /// solely to missing installed vocabulary. A complete, non-empty search
    /// with at least one complete cover blocked by a non-partial vocabulary
    /// wall, and no cause beyond `UnspellableRegion` search noise, is instead
    /// [`Self::UnsupportedCapability`]. This class is never an exhausted
    /// analysis budget.
    NoFeasiblePlan,
    /// A deterministic budget stopped the compilation.
    ///
    /// **This is not [`Self::NoFeasiblePlan`] and must not be read as one.**
    /// That class says an exhaustive plan search retained no valid plan for the
    /// requested target; this one says a bound *this build* declares refused a
    /// demand, and nothing was proved about the program. Which of the two a
    /// budget refusal is within itself — a bound no further search escapes, or
    /// a search stopped before it finished — is [`BudgetResource::refusal`],
    /// and that answer is what gives `reported` its meaning: an exact completed
    /// count, a conservative planning envelope, or a truncated-search lower
    /// bound.
    ///
    /// The three fields are the refusal's own, carried rather than re-derived,
    /// so a caller names the exhausted resource without reading compiler source.
    /// A refusal raised before a target-qualified trace exists carries no
    /// [`CompileFailure::explain`] report by construction, and every
    /// program-scoped budget refuses there, so for those these fields are the
    /// only route rather than a convenience beside the trace.
    BudgetExhausted {
        /// Which budget refused.
        resource: BudgetResource,
        /// The declared limit that budget carries in this build.
        limit: u64,
        /// The value the compiler compared against `limit`.
        ///
        /// Read [`BudgetResource::refusal`] on `resource` before treating this
        /// as a required size. It is an exact completed count, a conservative
        /// planning envelope that a particular plan may undershoot, or a lower
        /// bound recorded where search stopped — never a uniform "actual".
        reported: u64,
    },
    /// The compiler produced output its own verifier refused.
    ///
    /// This is always a defect in Tiler rather than in the caller's program,
    /// and is reported as a distinct class so a caller never reports it as an
    /// unsupported program.
    InvalidCompilerOutput,
}

/// A refused compilation: which boundary refused, and why in full.
///
/// # Why a failure carries a report at all
///
/// `docs/compiler/optimizer.md` requires that "Every rejection records its
/// stage, stable reason code, rule/provider identity, affected operation/value
/// or candidate, failed predicate/evidence, and whether the result is a hard
/// rejection, safe deferral, budget stop, dominance pruning, or cost
/// disadvantage", and that explain output "never collapses these into 'not
/// fused.'" A [`CompileFailureClass::NoFeasiblePlan`] with nothing attached is
/// exactly that collapse, and the compiler had already sealed the trace that
/// answers it before this boundary threw it away.
///
/// # Why the report is complete or absent, and never partial
///
/// A sealed trace is complete by construction. A detail record that would
/// exceed the retained-trace ceiling fails the compilation closed with a typed
/// capacity error rather than being dropped, so there is no truncated form for
/// this surface to describe. "Partial" was never one of two shapes to choose
/// between; it is a shape the explain authority does not produce.
///
/// # Why absence is structural rather than best-effort
///
/// A request-qualified trace can only exist once a verified per-target request
/// does. Request verification, semantic output typing, numerical-contract
/// resolution, normalization, and target selection all run before that point
/// and fail with no trace to seal, so [`Self::explain`] returns `None` for
/// precisely those refusals. It is a statement about which phase refused, not
/// about whether the compiler bothered.
#[derive(Clone, Eq, PartialEq)]
pub struct CompileFailure {
    class: CompileFailureClass,
    explain: Option<VerifiedExplainTrace>,
}

impl CompileFailure {
    /// Returns which boundary refused the compilation.
    #[must_use]
    pub const fn class(&self) -> CompileFailureClass {
        self.class
    }

    /// Returns the compilation's complete explain trace, when one exists.
    ///
    /// `None` means the refusal happened before a target-qualified trace could
    /// be opened; this type's documentation names which phases those are.
    #[must_use]
    pub fn explain(&self) -> Option<ExplainReport<'_>> {
        self.explain.as_ref().map(ExplainReport)
    }
}

/// Renders the class and whether a trace is attached, never the trace itself.
///
/// A derived `Debug` would print every retained record. A caller formatting an
/// error with `{:?}` on a failure path is asking what refused, not for the
/// whole explanation, and both in-workspace consumers do exactly that. The
/// trace stays reachable deliberately, through [`CompileFailure::explain`].
impl fmt::Debug for CompileFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompileFailure")
            .field("class", &self.class)
            .field(
                "explain",
                &match &self.explain {
                    Some(trace) => format!("{} records", trace.records().len()),
                    None => "absent (refused before a target-qualified trace)".to_owned(),
                },
            )
            .finish()
    }
}

impl fmt::Display for CompileFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "compilation refused at {:?} ({})",
            self.class,
            if self.explain.is_some() {
                "complete explain trace available"
            } else {
                "before a target-qualified explain trace"
            }
        )
    }
}

impl std::error::Error for CompileFailure {}

/// One target profile's compilation result.
#[derive(Clone, Debug)]
pub struct Compilation {
    stated_contracts: Vec<StrictF32NumericalContract>,
    resolved_contract: StrictF32NumericalContract,
    offered_lowering_providers: Arc<[ProviderIdentity]>,
    offered_physical_providers: Arc<[ProviderIdentity]>,
    target_profile: TargetProfile,
    feasibility_rule_set: FeasibilityRuleSetIdentity,
    alternatives: Vec<ProgramAlternative>,
    selected_alternative_id: String,
    explain: VerifiedCompilationExplain,
}

/// Ordered outcomes for every target in one caller request.
#[derive(Clone, Debug)]
pub struct CompilationBatch {
    targets: Vec<TargetCompilationResult>,
}

impl CompilationBatch {
    /// Returns one outcome per requested target, in request order.
    #[must_use]
    pub fn targets(&self) -> impl ExactSizeIterator<Item = &TargetCompilationResult> {
        self.targets.iter()
    }

    /// Consumes the batch without separating an outcome from its profile.
    #[must_use]
    pub fn into_targets(self) -> Vec<TargetCompilationResult> {
        self.targets
    }
}

/// One target profile inseparably paired with its compilation or refusal.
#[derive(Clone, Debug)]
pub struct TargetCompilationResult {
    target_profile: TargetProfile,
    outcome: Result<Compilation, TargetCompileFailure>,
}

impl TargetCompilationResult {
    /// Returns the exact immutable profile this slot names.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    /// Borrows the target-local success or failure.
    ///
    /// # Errors
    ///
    /// Returns the refusal scoped to this target slot.
    pub const fn outcome(&self) -> Result<&Compilation, &TargetCompileFailure> {
        match &self.outcome {
            Ok(compilation) => Ok(compilation),
            Err(failure) => Err(failure),
        }
    }

    /// Consumes the slot while retaining the profile beside its outcome.
    pub fn into_parts(self) -> (TargetProfile, Result<Compilation, TargetCompileFailure>) {
        (self.target_profile, self.outcome)
    }
}

/// Exact scalar subject named by a target-local numerical refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetNumericalSubject {
    arithmetic: ArithmeticType,
    resolved_type: ResolvedValueType,
}

impl TargetNumericalSubject {
    /// Returns the arithmetic family whose behaviour was required.
    #[must_use]
    pub const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// Returns the complete resolved semantic type, including parameters and
    /// encoded components.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }
}

/// A dimension-safe numerical requirement that one target could not honour.
///
/// Every arm carries only its dimension's behaviour vocabulary, so an output
/// cannot pair (for example) contraction with a subnormal mode.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetNumericalRequirement {
    /// Input-subnormal handling.
    InputSubnormals {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required input-subnormal mode.
        required: SubnormalMode,
    },
    /// Result-subnormal handling.
    ResultSubnormals {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required result-subnormal mode.
        required: SubnormalMode,
    },
    /// Contraction permission.
    Contraction {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required contraction permission.
        required: NumericalPermission,
    },
    /// Reassociation permission.
    Reassociation {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required reassociation permission.
        required: NumericalPermission,
    },
    /// Operand-permutation permission.
    Permutation {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required operand-permutation permission.
        required: NumericalPermission,
    },
    /// Signed-zero transformation permission.
    SignedZero {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required signed-zero transformation permission.
        required: NumericalPermission,
    },
    /// Reciprocal-replacement permission.
    ReciprocalTransform {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required reciprocal-replacement permission.
        required: NumericalPermission,
    },
    /// Approximate-intrinsic envelope.
    ApproximateIntrinsics {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required approximation envelope.
        required: ApproximationEnvelope,
    },
    /// NaN assumption.
    NanAssumptions {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required NaN assumption.
        required: ExceptionalValueAssumption,
    },
    /// Infinity assumption.
    InfinityAssumptions {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required infinity assumption.
        required: ExceptionalValueAssumption,
    },
    /// Observable materialization rounding.
    MaterializationRounding {
        /// Exact scalar subject.
        subject: TargetNumericalSubject,
        /// Required materialization rounding.
        required: MaterializationRounding,
    },
}

impl TargetNumericalRequirement {
    /// Returns the exact scalar subject shared by every requirement arm.
    #[must_use]
    pub const fn subject(&self) -> &TargetNumericalSubject {
        match self {
            Self::InputSubnormals { subject, .. }
            | Self::ResultSubnormals { subject, .. }
            | Self::Contraction { subject, .. }
            | Self::Reassociation { subject, .. }
            | Self::Permutation { subject, .. }
            | Self::SignedZero { subject, .. }
            | Self::ReciprocalTransform { subject, .. }
            | Self::ApproximateIntrinsics { subject, .. }
            | Self::NanAssumptions { subject, .. }
            | Self::InfinityAssumptions { subject, .. }
            | Self::MaterializationRounding { subject, .. } => subject,
        }
    }
}

/// Why a stated contract entry did not resolve on one target.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetNumericalRefusalDisposition {
    /// The target declared a means that does not honour the requirement.
    DeclaredUnhonourable(Box<TargetDeclaredNumericalRefusal>),
    /// No declaration named the exact subject and requirement.
    Unknown,
    /// A matching declaration exists only at a later phase.
    Deferred {
        /// Earliest phase at which the declaration can participate.
        available_at: AvailabilityPhase,
    },
}

/// Exact declaration behind one target's numerical refusal.
///
/// It retains the checked fact that refused, so [`Self::evidence`] reports that
/// fact's own authority, validity scope, and measured compiler builds and
/// execution environments rather than a summary reconstructed at this boundary.
/// The retained fact is compiler-private and shared; nothing here exposes it or
/// admits an edited provenance record in its place.
///
/// The behaviour the *caller* required is one level up, on
/// [`TargetNumericalContractRejection::requirement`], and is deliberately not
/// restated here: it belongs to the caller's contract entry rather than to the
/// target's declaration, and every disposition — refused, absent, deferred —
/// answers for the same required behaviour. [`Self::declared`] is the different
/// question this type answers: which behaviour the refusing declaration speaks
/// about.
///
/// # The retained fact is not reachable as data
///
/// A caller reads it only through [`Self::evidence`], which borrows. There is no
/// field to take, replace, or hand back edited, so a refusal that reaches a
/// diagnostic always cites provenance some authority actually supplied.
///
/// ```compile_fail,E0616
/// # use tiler_compiler::session::TargetDeclaredNumericalRefusal;
/// fn tamper(refusal: &TargetDeclaredNumericalRefusal) {
///     let _ = &refusal.evidence;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDeclaredNumericalRefusal {
    subject: TargetNumericalSubject,
    declared: TargetNumericalRequirement,
    means: TargetNumericalDeclaredMeans,
    honoured: Option<TargetNumericalHonouredBehaviour>,
    target_profile: TargetProfileKey,
    evidence: NumericalRefusalEvidence,
}

impl TargetDeclaredNumericalRefusal {
    /// Returns the exact arithmetic and resolved type the declaration addresses.
    #[must_use]
    pub const fn subject(&self) -> &TargetNumericalSubject {
        &self.subject
    }

    /// Returns the exact behaviour the refusing declaration speaks about.
    #[must_use]
    pub const fn declared(&self) -> &TargetNumericalRequirement {
        &self.declared
    }

    /// Returns the means the profile declares for the required behaviour.
    #[must_use]
    pub const fn means(&self) -> &TargetNumericalDeclaredMeans {
        &self.means
    }

    /// Returns the behaviour this dimension honours unconditionally, if any.
    #[must_use]
    pub const fn honoured(&self) -> Option<&TargetNumericalHonouredBehaviour> {
        self.honoured.as_ref()
    }

    /// Returns the exact profile key that made the declaration.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfileKey {
        &self.target_profile
    }

    /// Returns a borrowed, read-only view of the exact fact that refused.
    ///
    /// This is where the refusal stops being a verdict and becomes evidence: it
    /// names the authority, the scope over which that authority's claim holds,
    /// and — for a measured claim — the exact compiler builds and execution
    /// environments the measurement rests on, which is what a caller compares
    /// against its own deployment before acting on the refusal.
    #[must_use]
    pub const fn evidence(&self) -> TargetNumericalRefusalEvidence<'_> {
        TargetNumericalRefusalEvidence::borrow(&self.evidence)
    }
}

/// The complete means declared for a required numerical behaviour.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetNumericalDeclaredMeans {
    /// The target's arithmetic realizes the behaviour directly.
    SupportedExactly,
    /// Additional emitted operations realize the behaviour exactly.
    SupportedWithExactEmulation,
    /// The means is available only under another caller-authorized requirement.
    SupportedOnlyUnderDeclaredRelaxation {
        /// Exact dimension-safe relaxation the caller must already authorize.
        required: TargetNumericalRequirement,
    },
    /// The target declares that it cannot realize the behaviour.
    Unsupported,
}

/// A dimension-safe behaviour that a refusing profile does honour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetNumericalHonouredBehaviour {
    /// Input-subnormal handling.
    InputSubnormals(SubnormalMode),
    /// Result-subnormal handling.
    ResultSubnormals(SubnormalMode),
    /// Contraction permission.
    Contraction(NumericalPermission),
    /// Reassociation permission.
    Reassociation(NumericalPermission),
    /// Operand-permutation permission.
    Permutation(NumericalPermission),
    /// Signed-zero transformation permission.
    SignedZero(NumericalPermission),
    /// Reciprocal-replacement permission.
    ReciprocalTransform(NumericalPermission),
    /// Approximate-intrinsic envelope.
    ApproximateIntrinsics(ApproximationEnvelope),
    /// NaN assumption.
    NanAssumptions(ExceptionalValueAssumption),
    /// Infinity assumption.
    InfinityAssumptions(ExceptionalValueAssumption),
    /// Observable materialization rounding.
    MaterializationRounding(MaterializationRounding),
}

/// One entry in the caller's numerical-contract order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetNumericalContractRejection {
    contract_key: &'static str,
    requirement: TargetNumericalRequirement,
    disposition: TargetNumericalRefusalDisposition,
}

impl TargetNumericalContractRejection {
    /// Returns the rejected contract key.
    #[must_use]
    pub const fn contract_key(&self) -> &'static str {
        self.contract_key
    }

    /// Returns the exact, dimension-safe requirement.
    #[must_use]
    pub const fn requirement(&self) -> &TargetNumericalRequirement {
        &self.requirement
    }

    /// Returns whether the exact fact refused, was absent, or was deferred.
    #[must_use]
    pub const fn disposition(&self) -> &TargetNumericalRefusalDisposition {
        &self.disposition
    }
}

/// Ordered numerical-contract refusal for one exact target profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetNumericalContractRefusal {
    target_profile: TargetProfileKey,
    rejections: Vec<TargetNumericalContractRejection>,
}

impl TargetNumericalContractRefusal {
    /// Returns the target profile that refused the caller's preference.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfileKey {
        &self.target_profile
    }

    /// Returns one rejection per stated contract, in the caller's exact order.
    #[must_use]
    pub fn rejections(&self) -> &[TargetNumericalContractRejection] {
        &self.rejections
    }
}

/// Why one exact program dtype cannot dispatch on a target at compile profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetDTypeRefusalDisposition {
    /// The profile explicitly refuses the exact type.
    Unsupported,
    /// No declaration names the exact type.
    Unknown,
    /// The first exact declaration becomes available only later.
    Deferred {
        /// Earliest phase at which the declaration can participate.
        available_at: AvailabilityPhase,
    },
}

/// Target-local dispatch refusal for one exact resolved program dtype.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDTypeDispatchRefusal {
    target_profile: TargetProfileKey,
    resolved_type: ResolvedValueType,
    disposition: TargetDTypeRefusalDisposition,
}

impl TargetDTypeDispatchRefusal {
    /// Returns the target profile that cannot dispatch the type.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfileKey {
        &self.target_profile
    }

    /// Returns the complete exact resolved semantic type.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns whether the exact fact refused, was absent, or was deferred.
    #[must_use]
    pub const fn disposition(&self) -> TargetDTypeRefusalDisposition {
        self.disposition
    }
}

/// Which half of an elementary realization failed to discharge.
///
/// Re-exported from the assessment authority so the public refusal and the
/// internal check cannot name different halves.
pub use crate::target::accuracy::ElementaryEvidenceHalf;

/// Why one elementary accuracy obligation went unmet on a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetElementaryAccuracyReason {
    /// No installed realization speaks about the operation.
    NoInstalledRealization,
    /// A realization was installed and could not be proved to refine the contract.
    Unrefined,
    /// A realization refined the contract, but a half's evidence cannot discharge.
    UndischargedEvidence {
        /// Which half failed to discharge.
        half: ElementaryEvidenceHalf,
        /// The evidence class that failed to discharge.
        class: tiler_ir::semantic::accuracy::ConformanceEvidenceClass,
    },
}

impl TargetElementaryAccuracyReason {
    /// Returns the stable diagnostic code naming this reason.
    #[must_use]
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::NoInstalledRealization => "accuracy.elementary.no-installed-realization",
            Self::Unrefined => "accuracy.elementary.unrefined-realization",
            Self::UndischargedEvidence { .. } => "accuracy.elementary.undischarged-evidence",
        }
    }
}

/// One declared elementary-realization candidate named by a structured refusal.
///
/// **Labelled draft** under ADR 0075. Present when the profile installed at
/// least one same-operation row; empty on `no-installed-realization`. Several
/// rows appear in canonical row order so the details cannot depend on
/// insertion order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetElementaryAccuracyCandidate {
    contract: tiler_ir::semantic::accuracy::AccuracyContract,
    producer_key: String,
    producer_revision: u32,
}

impl TargetElementaryAccuracyCandidate {
    /// Returns the verified contract this candidate declared.
    #[must_use]
    pub const fn contract(&self) -> &tiler_ir::semantic::accuracy::AccuracyContract {
        &self.contract
    }

    /// Returns the producer key of the compile-profile source that declared it.
    #[must_use]
    pub fn producer_key(&self) -> &str {
        &self.producer_key
    }

    /// Returns the producer revision of that source.
    #[must_use]
    pub const fn producer_revision(&self) -> u32 {
        self.producer_revision
    }
}

/// Target-local elementary-accuracy refusal for one registered family.
///
/// **Labelled draft** under ADR 0075 for the candidate-provenance field. Tom
/// accepted the three-way reason split on 2026-08-11 and has not accepted this
/// type's exact included surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetElementaryAccuracyRefusal {
    operation: tiler_ir::semantic::OpKey,
    target_profile: TargetProfileKey,
    reason: TargetElementaryAccuracyReason,
    candidates: Box<[TargetElementaryAccuracyCandidate]>,
}

impl TargetElementaryAccuracyRefusal {
    /// Returns the elementary family whose registered contract went unsatisfied.
    #[must_use]
    pub const fn operation(&self) -> &tiler_ir::semantic::OpKey {
        &self.operation
    }

    /// Returns the target profile that was asked.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfileKey {
        &self.target_profile
    }

    /// Returns why the obligation went unmet.
    #[must_use]
    pub const fn reason(&self) -> TargetElementaryAccuracyReason {
        self.reason
    }

    /// Returns declared same-operation candidates in canonical order.
    ///
    /// Empty when the profile installed no row for the operation.
    #[must_use]
    pub fn candidates(&self) -> &[TargetElementaryAccuracyCandidate] {
        &self.candidates
    }
}

/// Recoverable typed detail for a pre-trace target-local refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetCompileRefusal {
    /// No contract in the caller's stated order resolved on this target.
    NumericalContract(TargetNumericalContractRefusal),
    /// One exact program dtype could not dispatch at compile profile.
    DTypeDispatch(TargetDTypeDispatchRefusal),
    /// A registered elementary family has no admitting realization on this target.
    ElementaryAccuracy(TargetElementaryAccuracyRefusal),
}

/// A refusal scoped to one otherwise valid target-profile slot.
///
/// Every value [`CompilationBatch`] carries is exactly that: one slot's own
/// refusal, with the other slots' outcomes untouched beside it. [`compile`]
/// reports a failure that precedes slot construction as a [`CompileFailure`]
/// instead, because such a failure loses every requested target rather than
/// one. [`compile_governed`] requests exactly one target and so has no second
/// outcome to lose; it reports both in this type, and its documentation states
/// what that means for [`Self::refusal`].
///
/// The detail is boxed because it is both large and rare: a
/// [`TargetCompileRefusal`] is 128 bytes against the 80 of the failure that
/// always accompanies it, and only a refusal preceding the trace boundary
/// carries one at all. Inline, this type is 208 bytes in every slot of every
/// batch and in the `Err` of every `Result` that returns one; boxed it is 88,
/// inside the width `clippy::result_large_err` admits in a returned `Err`.
/// [`Self::refusal`] hands out the same borrow either way.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompileFailure {
    failure: CompileFailure,
    refusal: Option<Box<TargetCompileRefusal>>,
}

impl TargetCompileFailure {
    /// Reports a failure raised before any target slot was constructed.
    ///
    /// [`Self::refusal`] is `None` by construction rather than by omission: the
    /// typed detail is minted from a per-target rejection, and there is no
    /// rejected target here to mint it from.
    const fn before_any_target(failure: CompileFailure) -> Self {
        Self {
            failure,
            refusal: None,
        }
    }

    /// Returns which compiler boundary refused this target.
    #[must_use]
    pub const fn class(&self) -> CompileFailureClass {
        self.failure.class()
    }

    /// Returns the complete target-qualified explanation, when available.
    #[must_use]
    pub fn explain(&self) -> Option<ExplainReport<'_>> {
        self.failure.explain()
    }

    /// Returns recoverable typed detail for a pre-trace target refusal.
    ///
    /// Post-verification planning failures are explained by [`Self::explain`].
    #[must_use]
    pub const fn refusal(&self) -> Option<&TargetCompileRefusal> {
        match &self.refusal {
            Some(refusal) => Some(refusal),
            None => None,
        }
    }
}

impl fmt::Display for TargetCompileFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "target compilation refused: {:?}", self.class())
    }
}

impl std::error::Error for TargetCompileFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.failure)
    }
}

impl Compilation {
    /// The governed keys of the contracts this compilation was told to accept.
    ///
    /// Keys rather than [`NumericalContract`] values, and the reason survived the
    /// enum this used to argue against. While the type was a four-value
    /// enumeration, mapping a resolved contract back onto it needed an inverse of
    /// the resolution whose only total spelling absorbed an unrecognized key into
    /// some variant — a silently wrong answer about which numerics a program was
    /// compiled under. The composed type has a genuine inverse, so that hazard is
    /// gone; what remains is that ADR 0076 makes the *key* a contract's governed
    /// name, and a caller comparing what it stated against what an artifact or a
    /// cache entry records is comparing keys. A caller that wants the dimensions
    /// back reads them off the [`NumericalContract`] it stated, whose
    /// [`NumericalContract::key`] is this same string.
    ///
    /// In the caller's stated order, which is the order bound into the request
    /// subject. The first entry is not necessarily the one that was used — read
    /// [`Self::resolved_numerical_contract_key`] for that. Exposing both is the
    /// point: "what I would have accepted" and "what I got" are different facts,
    /// and a reader seeing only the second cannot tell a compilation that got
    /// its first choice from one that fell back.
    #[must_use]
    pub fn stated_numerical_contract_keys(
        &self,
    ) -> impl ExactSizeIterator<Item = &'static str> + '_ {
        self.stated_contracts.iter().map(|contract| contract.key)
    }

    /// The numerical contract this compilation actually resolved to.
    #[must_use]
    pub const fn resolved_numerical_contract_key(&self) -> &'static str {
        self.resolved_contract.key
    }

    /// Returns the complete frozen *lowering* provider set offered to this
    /// compilation.
    ///
    /// This is compilation-environment evidence rather than program identity:
    /// an artifact records only the providers its retained plan selected, but
    /// its builder must prove each selection belonged to the authority the
    /// compiler actually offered. Returning the compiler-minted set prevents an
    /// assembler from reconstructing that environment from the selected subset.
    ///
    /// A compilation enumerates two provider environments against two different
    /// rules — one lowering authority per occurrence, several physical
    /// implementations per region — and the second is
    /// [`Self::offered_physical_providers`]. Reading this set as *the* provider
    /// environment is what would make an installed physical provider look as
    /// though it had never been offered.
    #[must_use]
    pub fn offered_lowering_providers(&self) -> &[ProviderIdentity] {
        &self.offered_lowering_providers
    }

    /// Returns the complete frozen physical-provider environment this
    /// compilation enumerated against.
    ///
    /// Governed first, then the caller's in installation order, which is
    /// exactly the list
    /// ([`crate::physical_provider::InstalledPhysicalProviders`]) the frontier
    /// was asked to enumerate. Installation order is retained for reporting and
    /// decides nothing.
    ///
    /// **This is the *offered* half of the disclosure [ADR 0090] item 5 splits
    /// in two, and the split is the point.** A provider named here and absent
    /// from every alternative's
    /// [`PlanAlternative::selected_physical_providers`] was consulted and
    /// contributed to no retained plan — it proposed nothing, or its proposals
    /// were refused, or they lost on cost. A provider absent from *both* was
    /// never installed. Merging the two sets into one accessor would make those
    /// two findings — the two a composition failure most needs to tell apart —
    /// indistinguishable.
    ///
    /// Never empty: the governed provider is always asked, so an empty answer
    /// would be a compiler defect rather than an environment. That is what
    /// separates this from
    /// [`crate::physical_provider::InstalledPhysicalProviders::identities`],
    /// whose empty answer is the ordinary "the caller installed nothing".
    ///
    /// This is compilation-environment evidence and is deliberately *not* what
    /// reaches artifact provenance today: an artifact's compilation environment
    /// is built from [`Self::offered_lowering_providers`] alone. Whether a
    /// physical-provider row belongs there is a separate decision owned where
    /// the artifact type is defined.
    ///
    /// [ADR 0090]: https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md
    #[must_use]
    pub fn offered_physical_providers(&self) -> &[ProviderIdentity] {
        &self.offered_physical_providers
    }

    /// Returns the validated declared key of the target profile this result is for.
    #[must_use]
    pub fn target_profile_key(&self) -> &str {
        self.target_profile.profile_key().as_str()
    }

    /// Returns the exact immutable target profile retained by this compilation.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    /// Returns the canonical descriptor bytes of the profile this compilation
    /// was assessed against.
    ///
    /// ADR 0043 requires a declared target profile to carry both its validated
    /// key and its exact descriptor identity, because two profiles can advertise
    /// one key and admit different candidates — so a key alone is not evidence
    /// that anything here is legal on a device presenting it.
    ///
    /// These bytes *are* the descriptor identity rather than a hash of it, so a
    /// consumer wraps them in its own opaque-identity type. Emitting bytes
    /// avoids minting a digest here and avoids a second identity that would have
    /// to be kept in agreement with the bytes it summarizes.
    ///
    /// Sited here rather than on [`PlanAlternative`] because one compilation
    /// declares one profile: every retained alternative was assessed against
    /// this exact descriptor, and offering it per alternative would invite a
    /// reader to believe two alternatives of one compilation could differ.
    #[must_use]
    pub fn target_profile_descriptor(&self) -> &[u8] {
        self.target_profile.canonical_descriptor()
    }

    /// Returns the governed key of the feasibility rules this compilation was
    /// assessed under.
    ///
    /// The rules are a second identity beside the target profile rather than a
    /// field of it: one profile can be re-assessed under new rules and one rule
    /// set applies across profiles, so neither determines the other, and the
    /// artifact layer records them as two references for that reason.
    ///
    /// Minted by the compiler and handed over whole, like a capability key. The
    /// pair enters artifact identity under ADR 0072, so a consumer composing a
    /// key and a revision of its own would be a second derivation of one
    /// identity.
    #[must_use]
    pub fn feasibility_rule_set_key(&self) -> &str {
        self.feasibility_rule_set.key()
    }

    /// Returns the output-affecting revision of those feasibility rules.
    ///
    /// Always nonzero: zero is reserved for "unset" at the artifact boundary, so
    /// the compiler refuses to mint one rather than let an artifact record rules
    /// it was never assessed under.
    #[must_use]
    pub fn feasibility_rule_set_revision(&self) -> u32 {
        self.feasibility_rule_set.revision()
    }

    /// Returns every retained plan alternative, in the order the policy ranked.
    #[must_use]
    pub fn alternatives(&self) -> impl ExactSizeIterator<Item = PlanAlternative<'_>> {
        self.alternatives.iter().map(|alternative| PlanAlternative {
            compilation: self,
            alternative,
        })
    }

    /// Returns the alternative the selection policy chose.
    ///
    /// The portfolio always retains the selected identifier, so this cannot be
    /// absent for a compilation that succeeded; it returns an `Option` rather
    /// than panicking because that invariant belongs to the compiler and a
    /// public surface should not assert it into a caller's process.
    #[must_use]
    pub fn selected(&self) -> Option<PlanAlternative<'_>> {
        self.alternatives
            .iter()
            .find(|alternative| alternative.stable_id == self.selected_alternative_id)
            .map(|alternative| PlanAlternative {
                compilation: self,
                alternative,
            })
    }

    /// Returns the compilation's verified composite explanation.
    ///
    /// The value binds the top-level selection to every independently sealed
    /// semantic-candidate trace for every contract group the compiler evaluated.
    /// Candidates in lower-preference groups are represented by explicit
    /// contract-preference-pruned records rather than unevaluated traces.
    #[must_use]
    pub const fn explain(&self) -> &VerifiedCompilationExplain {
        &self.explain
    }
}

/// A read view over one retained plan alternative.
///
/// A borrowed view rather than an owned record, so this boundary commits to no
/// public field set while the compiler's internal plan representation is still
/// moving.
#[derive(Clone, Copy, Debug)]
pub struct PlanAlternative<'a> {
    compilation: &'a Compilation,
    alternative: &'a ProgramAlternative,
}

impl<'a> PlanAlternative<'a> {
    /// Returns the compilation that owns this retained plan.
    ///
    /// The owner link is load-bearing for downstream orchestration: accepting a
    /// free `(Compilation, PlanAlternative)` pair would let a caller combine a
    /// program from one compilation with the target profile, feasibility rules,
    /// and offered-provider environment of another.
    #[must_use]
    pub const fn compilation(self) -> &'a Compilation {
        self.compilation
    }

    /// Returns the alternative's stable identifier within its portfolio.
    #[must_use]
    pub fn stable_id(&self) -> &str {
        &self.alternative.stable_id
    }

    /// Returns whether one region covers the whole program.
    #[must_use]
    pub fn is_fused(&self) -> bool {
        self.alternative.kind == ProgramAlternativeKind::Fused
    }

    /// Returns the verified kernels this alternative dispatches.
    ///
    /// This is what a backend emits from. The kernels are already verified by
    /// `tiler-ir`'s own authority, so handing them out commits this boundary to
    /// no new guarantee of its own.
    #[must_use]
    pub fn kernels(&self) -> &[VerifiedKernel] {
        &self.alternative.kernels
    }

    /// Returns the lowering capabilities this alternative resolved.
    ///
    /// An artifact records which capabilities actually lowered its program;
    /// ADR 0072 folds them into complete program identity, so this is evidence
    /// rather than description.
    pub fn selected_capabilities(&self) -> impl ExactSizeIterator<Item = SelectedCapability<'_>> {
        self.alternative
            .artifact_plan
            .lowering_providers()
            .iter()
            .map(SelectedCapability)
    }

    /// Returns the physical providers whose implementations this plan selected.
    ///
    /// One entry per cover region, in canonical region-occurrence order, so a
    /// plan that mixes an installed provider's specialization of one region with
    /// the governed implementation of another says exactly that rather than
    /// naming a single winner.
    ///
    /// **The identity is the host's, not the provider's claim.** It is read once
    /// from the provider at enumeration and stamped by the frontier onto each
    /// admitted implementation; a proposal carries no identity field to forge,
    /// and an installed provider claiming the governed identity is refused
    /// before any compilation runs
    /// ([`crate::physical_provider::InstalledPhysicalProviders::installed`]).
    ///
    /// This is the *selected* half of the disclosure [ADR 0090] item 5 splits in
    /// two. The offered half — the complete frozen environment this compilation
    /// was given, including providers no retained plan chose — is
    /// [`Compilation::offered_physical_providers`]. Reading an installed
    /// provider's absence *here* as "never installed" is exactly the conflation
    /// that split exists to prevent: an identity offered and not selected was
    /// consulted and contributed nothing, and only the pair of accessors
    /// separates that from a provider that was never installed.
    ///
    /// [ADR 0090]: https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md
    pub fn selected_physical_providers(
        &self,
    ) -> impl ExactSizeIterator<Item = SelectedImplementation<'_>> {
        self.alternative
            .plan
            .selections()
            .iter()
            .map(SelectedImplementation)
    }

    /// Returns this alternative's ABI construction inputs.
    ///
    /// This is what an artifact assembler needs beyond the kernels: the guard,
    /// the accessible byte ranges, and the launch geometry, as expressions
    /// rather than as scalars.
    #[must_use]
    pub fn abi(&self) -> AbiConstruction<'_> {
        AbiConstruction(self.alternative.artifact_plan.verified_program())
    }

    /// Returns this alternative's complete delivered-realization evidence.
    ///
    /// ADR 0076 item 4's readable statement, in its compiler-side form: the
    /// policy subjects the checked request selected with their complete
    /// eleven-dimension contracts, and the locus-keyed obligations every
    /// packaged stage relies on, each carrying the exact checked fact that
    /// honours it — the declared behaviour, the structured means with its
    /// relaxation payload, and the complete provenance ADR 0076 item 3 governs.
    ///
    /// One view rather than three iterators, because a total translation has to
    /// cross-check subjects, coverage, obligation associations, and the evidence
    /// pool together; three iterators can be zipped wrongly.
    #[must_use]
    pub fn delivered_realization(&self) -> DeliveredRealizationView<'_> {
        DeliveredRealizationView::new(
            self.compilation.target_profile_key(),
            self.compilation.target_profile_descriptor(),
            &self.alternative.realization,
        )
    }

    /// Returns every compiler-minted prepared-entry target requirement, in exact program
    /// entry then predicate order.
    ///
    /// The entry ordinal is part of the requirement subject. Two prepared pipelines
    /// may report different values for the same property key, so an assembler
    /// must preserve each record rather than deduplicating by key.
    pub fn prepared_entry_target_requirements(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedEntryTargetRequirementRef<'_>> {
        self.alternative
            .artifact_plan
            .entry_deferred_predicates()
            .iter()
            .map(PreparedEntryTargetRequirementRef)
    }
}

/// A borrowed compiler-minted target requirement bound to one prepared entry.
#[derive(Clone, Copy, Debug)]
pub struct PreparedEntryTargetRequirementRef<'a>(&'a crate::program::EntryDeferredPredicate);

impl<'a> PreparedEntryTargetRequirementRef<'a> {
    /// Returns the zero-based program-entry ordinal whose prepared subject is queried.
    #[must_use]
    pub fn entry(self) -> u32 {
        self.0.entry()
    }

    /// Returns the governed capability-axis key used for diagnostics.
    #[must_use]
    pub fn capability_axis(self) -> &'static str {
        self.0.predicate().axis().key()
    }

    /// Returns the complete requirement without reconstructing any predicate.
    #[must_use]
    pub fn requirement(self) -> &'a PreparedEntryTargetRequirement {
        self.0.predicate().requirement()
    }
}

/// One capability the compiler resolved to lower part of a program.
///
/// The governed key is minted by the compiler and handed over whole. A caller
/// wraps it in its own key type rather than composing one from parts, because
/// the key enters artifact identity and two places deriving one identity is the
/// drift this boundary exists to prevent.
#[derive(Clone, Copy, Debug)]
pub struct SelectedCapability<'a>(&'a LoweringProviderIdentity);

impl<'a> SelectedCapability<'a> {
    /// Returns the identity of the provider that supplied the capability.
    #[must_use]
    pub fn provider(self) -> &'a ProviderIdentity {
        self.0.provider()
    }

    /// Returns the governed key of the resolved capability.
    #[must_use]
    pub fn capability_key(self) -> &'a str {
        self.0.capability_key()
    }

    /// Returns the capability's output-affecting revision.
    #[must_use]
    pub fn capability_revision(self) -> u32 {
        self.0.capability_revision().get()
    }
}

/// One cover region's selected physical implementation, with its provenance.
///
/// A borrowed view rather than an owned record, for the reason
/// [`PlanAlternative`] is one: the selection-level representation behind it is
/// still moving, and this boundary commits to no field set of it. The compiler
/// is its only constructor: an artifact assembler may forward the four subjects
/// in the accepted artifact projection, but cannot mint or replace the checked
/// occurrence-to-proposal binding.
///
/// ```compile_fail,E0423
/// use tiler_compiler::session::SelectedImplementation;
///
/// let _ = SelectedImplementation(());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SelectedImplementation<'a>(&'a crate::selection::RegionSelection);

impl<'a> SelectedImplementation<'a> {
    /// Returns the whole canonical identity of the cover-region occurrence.
    ///
    /// This is the occurrence side of the compiler-checked binding, not the
    /// scheduled body's reusable structural identity. Two equal bodies at two
    /// graph sites therefore keep two rows. The bytes are handed over whole:
    /// callers must not parse them into graph-local parts or substitute a
    /// presentation label for the collision-free identity.
    #[must_use]
    pub fn region_occurrence_identity(self) -> &'a [u8] {
        self.0.occurrence().as_bytes()
    }

    /// Returns the whole canonical identity of the admitted implementation.
    ///
    /// The frontier mints this identity only after host verification and folds
    /// the exact structural body subject, host-stamped provider, proposal kind,
    /// applicability, derived boundary contract, and deferred feasibility
    /// evidence. Cost and enumeration order are deliberately absent. A caller
    /// forwards these opaque bytes rather than reconstructing the identity from
    /// this view's readable provider and kind.
    #[must_use]
    pub fn implementation_proposal_identity(self) -> &'a [u8] {
        self.0.implementation().identity().as_bytes()
    }

    /// Returns the identity of the provider whose proposal was admitted.
    #[must_use]
    pub fn provider(self) -> &'a ProviderIdentity {
        self.0.implementation().provenance().provider()
    }

    /// Returns the provider's exact identity in bounded explain-subject form.
    ///
    /// The same rendering the explain trace carries, so a caller comparing what
    /// it installed against what a trace names is comparing one string rather
    /// than reconstructing one from parts.
    #[must_use]
    pub fn provider_explain_subject(self) -> &'a str {
        self.0
            .implementation()
            .provenance()
            .provider_explain_subject()
    }

    /// Returns the stable name of the admitted proposal's body kind.
    ///
    /// A stable code rather than the additive body enumeration, which stays
    /// crate-private: three of its four variants have no out-of-crate spelling
    /// to propose, so exporting the type would publish a vocabulary a caller
    /// could read and never write.
    #[must_use]
    pub fn proposal_kind(self) -> &'static str {
        self.0.implementation().provenance().kind().name()
    }
}

#[cfg(test)]
impl<'a> SelectedImplementation<'a> {
    /// Wraps one crate-private selection for subject-perturbation tests.
    ///
    /// Production has no corresponding constructor: this exists only so the
    /// selection authority can demonstrate that the public projection follows a
    /// deliberately moved occurrence while checked-plan verification refuses the
    /// same rebinding.
    pub(crate) const fn from_selection_for_test(
        selection: &'a crate::selection::RegionSelection,
    ) -> Self {
        Self(selection)
    }
}

/// A borrowed view of the ABI construction inputs of one plan alternative.
///
/// # Why this is a view of expressions rather than of numbers
///
/// ADR 0068 assigns construction of `AbiExpr` to `tiler-compiler`, and the
/// compiler derives these while it verifies the program's host preflight
/// contract. Handing out the derived *scalars* instead would make the assembler
/// re-derive an accessible byte range beside the compiler's own derivation, and
/// two derivations of one fact is exactly the drift this boundary exists to
/// prevent.
///
/// # Why an assembler does not replay them
///
/// `tiler-artifact` adopts the verified program's reachable ABI subgraph when a
/// variant is inserted and derives the artifact-owned handles itself. An
/// assembler may inspect these positions, but replaying them first would be a
/// duplicate traversal whose handles no variant uses.
///
/// # Where the decision is actually made
///
/// Since `complete-program-identity-with-abi-guards-and-routing`, every
/// accessor below reads the *verified program's* own ABI rather than a
/// compiler-side copy of it, and each is folded into
/// `CanonicalKernelProgramIdentity`. So this type is a convenience over
/// [`AbiConstruction::kernel_program`] and not a second authority: a consumer
/// that already holds the program can read the same facts from it directly.
///
/// The vocabulary is `tiler_ir::program::abi`, which both crates already
/// depend on, so nothing compiler-internal crosses this boundary and packaging
/// needs no `tiler-compiler` → `tiler-artifact` edge.
#[derive(Clone, Copy, Debug)]
pub struct AbiConstruction<'a>(&'a KernelProgram);

impl<'a> AbiConstruction<'a> {
    /// Returns the expression arena in canonical arena order.
    ///
    /// Every operand position is strictly smaller than the node that names it,
    /// so replaying front to back always has its operands already minted.
    #[must_use]
    pub fn expressions(self) -> &'a [ExprNode] {
        self.0.core().abi_expressions()
    }

    /// Returns the arena position of the guard deciding whether this
    /// alternative may be routed to.
    #[must_use]
    pub fn applicability_guard(self) -> u32 {
        self.0.core().applicability_guard()
    }

    /// Returns the verified target-neutral program this alternative packages.
    ///
    /// Already verified by `tiler-ir`'s own authority, so exposing it commits
    /// this boundary to no guarantee of its own.
    #[must_use]
    pub fn kernel_program(self) -> &'a VerifiedKernelProgram {
        self.0.core()
    }

    /// Returns one entry view per program stage, in stage order.
    pub fn entries(self) -> impl ExactSizeIterator<Item = AbiEntry<'a>> {
        self.0.core().stages().map(AbiEntry)
    }
}

/// A borrowed view of one stage's ABI and launch contract.
///
/// Each accessor returns an arena position into the same arena
/// [`AbiConstruction::expressions`] returns, never a resolved number.
#[derive(Clone, Copy, Debug)]
pub struct AbiEntry<'a>(StageRef<'a>);

impl<'a> AbiEntry<'a> {
    /// Returns the accessible byte range of each binding, in kernel
    /// buffer-parameter order.
    ///
    /// The order is the contract: `push_variant` matches bindings to kernel
    /// buffer parameters positionally, and a program stage's accesses are
    /// already in that order.
    #[must_use]
    pub fn accessible_bytes(self) -> impl ExactSizeIterator<Item = u32> + 'a {
        self.0
            .accesses()
            .map(tiler_ir::program::StageAccessRef::accessible_bytes)
    }

    /// Returns the total launch thread count of this entry.
    #[must_use]
    pub fn grid_threads(self) -> u32 {
        self.0.launch().grid_threads
    }

    /// Returns the workgroup width of this entry.
    #[must_use]
    pub fn threads_per_workgroup(self) -> u32 {
        self.0.launch().threads_per_workgroup
    }
}

/// An opaque handle to one compilation's complete explain trace.
///
/// Rendering is the only capability exposed, and that is now a settled shape
/// rather than a placeholder narrow enough to avoid settling anything.
///
/// # Not serialized, and not embedded in an artifact
///
/// `docs/compiler/optimizer.md` already states the half that governs artifacts:
/// "Canonical trace content is data and the renderer is presentation. Nothing
/// in this contract requires an explain trace to be serialized into an artifact
/// envelope, and the artifact contract does not carry one." Neither placement
/// survives inspection anyway. Inside artifact identity, a trace folds rule
/// keys, provider revisions and the explain schema version, so renaming a
/// reason code would change the identity of a program whose executable meaning
/// did not — invalidating every cache entry, which `docs/artifact-abi.md`
/// already rejects for the frozen registry snapshot on exactly that ground.
/// Outside it, `docs/artifact-abi.md` refuses a section no variant references
/// as `UnreferencedSection`, precisely so an envelope cannot carry bytes its
/// identity does not cover.
///
/// Serializing the canonical bytes at this boundary fails for a third reason:
/// those bytes *are* the trace's identity, and ADR 0074 convention 2 keeps a
/// canonical identity opaque and never re-derived by a consumer. A caller
/// parsing them would be a second derivation of what the trace means. The
/// producer-evidence use this would otherwise serve is already owned, with the
/// better shape, by the proof sidecar — where a sidecar names an artifact and
/// an artifact never names a sidecar.
///
/// The reconsideration trigger is the one `docs/compiler/optimizer.md` already
/// names: a second crate that must *read* canonical traces. Its answer is to
/// move the record vocabulary into `tiler-ir`, not to publish a byte format.
#[derive(Clone, Copy, Debug)]
pub struct ExplainReport<'a>(&'a VerifiedExplainTrace);

impl ExplainReport<'_> {
    /// Renders the trace in its deterministic text form.
    ///
    /// # What is guaranteed
    ///
    /// Rendering is **deterministic** — one trace renders to one string — and
    /// **total**: every retained record appears, in trace order. There is no
    /// filter, no bound, and no retention control to configure, because the
    /// explain authority refuses a compilation whose trace would not fit rather
    /// than dropping records from one that exists.
    ///
    /// # What is not
    ///
    /// The spelling. The rendered form is a diagnostic for a human reader and
    /// **not a parse target**; the leading `tiler-explain-v<N>` names the
    /// renderer version and changes when the rendering does. Committing to the
    /// text would create a second description of the trace that has to be kept
    /// in agreement with its canonical bytes, which is the duplicate-derivation
    /// hazard this whole boundary is shaped to avoid.
    ///
    /// The `request=<hex>` qualifier beside that version is a **correlation
    /// label**, not an identity: it is a 64-bit non-cryptographic fold of the
    /// request subject, so two distinct requests may share one. Reading it as
    /// an identifier — keying a cache or a lookup on it — is unsound in the
    /// silent direction, and ADR 0074 convention 2 states the rule it breaks: a
    /// short bounded label is presentation and is never an equality or dedup
    /// input.
    ///
    /// # Nothing here is redacted
    ///
    /// Every provider key and revision a trace attributes is either minted by
    /// Tiler or installed by this caller's own request — the writer refuses a
    /// rule attributed to any other provider — so there is no third party's
    /// detail present to withhold. Redacting one would also make a rejection
    /// unexplainable, which `docs/compiler/optimizer.md` forbids by requiring
    /// every rejection to record its rule and provider identity.
    #[must_use]
    pub fn render(&self) -> String {
        self.0.render()
    }
}

/// Splits one internal compile error into its public class and its trace.
impl From<CompileError> for CompileFailure {
    fn from(error: CompileError) -> Self {
        match error {
            // The one shape that carries a sealed trace, and the only place a
            // trace can reach a caller: the compiler opens a writer once a
            // verified per-target request exists and seals it on the way out.
            CompileError::Explained { source, explain } => Self {
                class: class_of(*source),
                explain: Some(explain),
            },
            other => Self {
                class: class_of(other),
                explain: None,
            },
        }
    }
}

/// Classifies one internal compile error into the public failure vocabulary.
///
/// Exhaustive at every level, with no wildcard arm: a new internal class must
/// be classified deliberately rather than absorbed into whichever public class
/// happened to sit under a catch-all. That matters here more than at most
/// mapping sites, because the two directions of a wrong absorption are not
/// symmetric — reporting a Tiler defect as an unsupported program tells a
/// caller to change a program that was fine, and reporting an unsupported
/// program as a Tiler defect sends a correct refusal to the wrong owner.
fn class_of(error: CompileError) -> CompileFailureClass {
    match error {
        // Unreachable rather than impossible: `compile_target` is the sole
        // construction site and never feeds its own result back into a wrapped
        // failure, but the type admits nesting, so the arm classifies the
        // innermost cause instead of leaving a wildcard to absorb it. The
        // caller above keeps the outer trace, which is the more complete one.
        CompileError::Explained { source, .. } => class_of(*source),
        // These two were one class, on the argument that both are statements
        // about the request rather than about Tiler, both carry the refusing
        // check's own key, and the internal distinction survives in the explain
        // trace. **That argument is preserved because it is true about
        // information and wrong about class.** Nothing was lost by merging
        // them; what was lost is the caller's ability to branch. Telling the two
        // apart meant matching `rule` against strings, which is the thing this
        // enum's own documentation says it exists to avoid, and the two imply
        // different actions — fix the request, versus install a provider.
        CompileError::InvalidRequest(cause) => CompileFailureClass::InvalidRequest {
            rule: rule_of(&cause),
        },
        CompileError::UnsupportedCapability(cause) => CompileFailureClass::UnsupportedCapability {
            rule: rule_of(&cause),
        },
        // Destructured rather than classified from the wrapper, because the
        // fields a caller acts on live on the inner error and this boundary is
        // where they were dropped. The wrapper keeps carrying the whole
        // `RequestError` so `Error::source` still yields it with its own type,
        // which convention 1 requires; the cost is that the payload type admits
        // a variant the one construction site never puts there.
        //
        // `From<RequestError>` routes `BudgetExceeded` to this variant and
        // nothing else, so the `else` arm is unreachable — and it is written
        // rather than unwrapped because a wrapper holding some other request
        // refusal would be Tiler mislabelling its own error, which is what
        // `InvalidCompilerOutput` reports. Inventing a resource, or panicking,
        // would each be worse: one is a silently wrong attribution and the
        // other refuses to report at all.
        CompileError::BudgetExhausted(cause) => {
            let RequestError::BudgetExceeded {
                resource,
                limit,
                reported,
            } = cause
            else {
                return CompileFailureClass::InvalidCompilerOutput;
            };
            CompileFailureClass::BudgetExhausted {
                resource,
                limit,
                reported,
            }
        }
        CompileError::NoFeasiblePlan(_) => CompileFailureClass::NoFeasiblePlan,
        CompileError::InvalidCompilerOutput(_) => CompileFailureClass::InvalidCompilerOutput,
    }
}

/// Returns the stable diagnostic key of one request refusal.
const fn rule_of(error: &RequestError) -> &'static str {
    match error {
        RequestError::UnsupportedRequestVersion => "compile.request.schema",
        RequestError::MismatchedShapeEnvironment => "compile.request.shape-environment",
        RequestError::EmptyTargetSet => "compile.request.targets.empty",
        RequestError::DuplicateTargetProfile => "compile.request.targets.duplicate",
        RequestError::UnverifiedTargetSelection => "compile.request.targets.selection",
        RequestError::UnstatedNumericalContract => "compile.request.numerics.unstated",
        RequestError::DuplicateNumericalContract => "compile.request.numerics.duplicate",
        RequestError::TooManyNumericalContracts { .. } => "compile.request.numerics.too-many",
        RequestError::NoResolvableNumericalContract { .. } => {
            "compile.request.numerics.unhonourable"
        }
        RequestError::DTypeNotDispatchable { .. } => "compile.request.dtype.dispatch",
        RequestError::UnrepresentableNumericalDimension { .. } => {
            "compile.request.numerics.unrepresentable"
        }
        RequestError::NoApplicableNumericalContract { .. } => {
            "compile.request.numerics.inapplicable"
        }
        // Reachable only if a budget refusal is ever wrapped as a request or
        // capability refusal, which `From<RequestError>` does not do. It stays
        // because `rule_of` is total over `RequestError` and the key exists;
        // `class_of` reports this refusal through its own typed fields instead.
        RequestError::BudgetExceeded { resource, .. } => resource.key(),
        RequestError::UnsupportedCapability { rule, .. }
        | RequestError::UnsupportedSymbolicExtent { rule, .. } => rule,
        // The refusing authority's own stable code, so the three findings it
        // distinguishes — no installed realization, an installed one that
        // could not be proved to refine, and a refining one whose evidence
        // cannot discharge — reach a caller as different keys.
        RequestError::UnrealizedElementaryAccuracy { reason, .. } => reason,
        RequestError::ShapeProductOverflow { role } => role,
    }
}

/// The resolved numerical contract a caller states, composed from its
/// independent dimensions.
///
/// Stating one is **required**, not defaulted. A contract decides what the
/// program *means*, so the choice belongs to the caller and no authority below
/// may narrow, weaken, or substitute it to make a target feasible: an
/// unhonourable request is a typed rejection naming the dimension, the
/// arithmetic type, the required behaviour, the behaviour the target declares,
/// and the declaring profile — never a downgrade and never a cost.
///
/// # Composed, because the axes were already decided independent
///
/// This was a four-value preset enumeration. Every axis it spanned had already
/// been decided independent — ADR 0011 holds that one permission never implies
/// another, ADR 0014 split ordered regrouping from contributor permutation on
/// evidence, ADR 0080 added a third independent dimension — and the target side
/// already declares honourability and refuses per dimension. The enumeration was
/// the one point-shaped surface left, and it produced its predictable failure the
/// first time real hardware needed a corner no preset named: Apple `f32`
/// arithmetic flushes subnormals in every measured math mode, both reassociating
/// presets required them preserved, and so no parallel reduction was statable on
/// the one measured Apple row — for want of a contract a caller could name, not
/// for want of a target fact.
///
/// A caller now resolves each dimension directly through
/// [`NumericalContractBuilder`], and the combination is a stated contract rather
/// than a filed blocker.
///
/// # Omission never widens
///
/// A composition starts at [`NumericalContractBuilder::strict_f32`], every
/// dimension at its strict resolution, and a caller resolves the ones it means to
/// move. An unstated dimension is therefore forbidden rather than unconstrained,
/// and a dimension added to the vocabulary later arrives forbidden in every
/// contract written before it existed.
///
/// # Statable exceeds tested, permanently
///
/// The number of statable contracts is the size of the dimension space, and this
/// build's conformance evidence covers a handful of points in it. That gap is
/// permanent and is not closed by narrowing what a caller may say — it is closed
/// per dimension, at the target: feasibility assesses every dimension of a stated
/// contract against a profile's measured honourability declaration, and an
/// unmeasured resolution is `Unknown` rather than assumed, so an untested
/// combination fails closed with a typed refusal naming the dimension. Two
/// further gates sit before it — a self-contradictory vector is refused by
/// [`NumericalContractBuilder::build`], and a dimension whose stated resolution
/// no scheduled region can record is refused by the request boundary with the
/// dimension, the behaviour this build realizes, and the operation that would
/// consume it.
///
/// # One arithmetic type
///
/// Every resolution is stated for exactly one [`ArithmeticType`], and the
/// contract carries which. Subnormal behaviour is measurably per-dtype — one
/// Apple row flushes in `f32`, preserves in `f16`, and flushes in `bf16` — so a
/// contract that spoke for every width at once would be stating something
/// already known to be false for one of them.
///
/// The width is never defaulted and never inferred from the program: a
/// composition starts at [`NumericalContractBuilder::strict_f32`] or
/// [`NumericalContractBuilder::strict_bf16`], each of which names its width in
/// its own name, and there is no width-free entry point for an omission to fall
/// through. The subject a target is then asked about is derived from that width
/// through the governed scalar catalog, so a contract and a profile row speak
/// about the same value identity by construction rather than by coincidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NumericalContract {
    /// The arithmetic type every resolution below is stated for.
    arithmetic: ArithmeticType,
    /// The canonical arithmetic NaN pattern of that width.
    ///
    /// Carried beside the width rather than looked up at resolution, because it
    /// is part of what the contract *is*: two contracts resolving the same
    /// dimensions and producing different NaN patterns are different contracts
    /// (ADR 0076 item 6), and the key encodes it for exactly that reason.
    canonical_arithmetic_nan_bits: u32,
    input_subnormals: SubnormalMode,
    result_subnormals: SubnormalMode,
    contraction: NumericalPermission,
    reassociation: NumericalPermission,
    permutation: NumericalPermission,
    signed_zero: NumericalPermission,
    reciprocal_transform: NumericalPermission,
    approximate_intrinsics: ApproximationEnvelope,
    nan_assumptions: ExceptionalValueAssumption,
    infinity_assumptions: ExceptionalValueAssumption,
    materialization_rounding: MaterializationRounding,
}

impl NumericalContract {
    /// Every freedom refused; subnormals preserved on both dimensions.
    ///
    /// Not deliverable on any governed Apple family, whose `f32` arithmetic
    /// flushes subnormals in every math mode. A caller states it when it needs
    /// preservation and would rather not run than run wrong.
    pub const STRICT_F32: Self = NumericalContractBuilder::strict_f32().resolved();

    /// Strict, except that both subnormal dimensions flush to the
    /// sign-preserving zero.
    ///
    /// This is what Apple hardware measurably delivers, so stating it makes
    /// running there a choice the caller made rather than a compromise made on
    /// its behalf. It widens exactly two dimensions: accepting flushing does not
    /// thereby accept reassociated sums.
    ///
    /// `PreservesSign` because that is what the hardware measurably does —
    /// `0x80400000 * 2.0f` returns `0x80000000` — and a contract must name which
    /// zero it accepts, since the two zeros are observably different results.
    pub const FLUSH_SUBNORMALS_TO_ZERO_F32: Self = NumericalContractBuilder::strict_f32()
        .input_subnormals(SIGN_PRESERVING_FLUSH)
        .result_subnormals(SIGN_PRESERVING_FLUSH)
        .resolved();

    /// Subnormals preserved, and the reshaping freedoms this build can express
    /// authorized: fused-multiply-add contraction, ordered reassociation of one
    /// same-operation operand sequence, reciprocal replacement of division, and
    /// approximate elementary functions within a named accuracy envelope.
    ///
    /// Operand permutation, signed-zero elimination, and assuming NaNs or
    /// infinities absent are deliberately *not* authorized. Each is a freedom an
    /// admitted operation could consume, so widening one here would broaden this
    /// contract's established meaning rather than record a new one.
    pub const RELAXED_F32: Self = NumericalContractBuilder::strict_f32()
        .contraction(NumericalPermission::Permitted)
        .reassociation(NumericalPermission::Permitted)
        .reciprocal_transform(NumericalPermission::Permitted)
        .approximate_intrinsics(ApproximationEnvelope::BackendElementary)
        .resolved();

    /// Strict, except that ordered regrouping of one same-operation operand
    /// sequence is authorized — a reduction's contributor sequence included.
    ///
    /// This is what a caller states to make a split reduction a legal
    /// implementation of its program while keeping every rounding boundary the
    /// strict reading has. It is not a narrower [`Self::RELAXED_F32`]: contracts
    /// are not ordered by strength, and the difference is which observable
    /// results the caller has agreed to. Contraction in particular stays
    /// forbidden, which ADR 0015 makes an independent choice — permission to
    /// regroup an operand sequence is not permission to fuse a multiply into an
    /// add.
    pub const REASSOCIATE_F32: Self = NumericalContractBuilder::strict_f32()
        .reassociation(NumericalPermission::Permitted)
        .resolved();

    /// Sign-preserving subnormal flushing **and** ordered regrouping.
    ///
    /// **The corner the preset enumeration could not name.** Every parallel
    /// reduction strategy regroups the declared contributor sequence, and the
    /// measured Apple `f32` row flushes subnormals in every math mode, so this is
    /// the contract under which a split or a workgroup tree is a legal
    /// implementation of a program on that hardware. Under the four-preset
    /// enumeration it was unstatable, and the gap read as a missing target fact
    /// rather than as a missing contract.
    ///
    /// It widens exactly three dimensions, and each is an independent statement:
    /// contraction, permutation, signed-zero elimination, reciprocal
    /// replacement, approximate intrinsics, and both exceptional-value
    /// assumptions stay at their strict resolution.
    pub const FLUSH_AND_REASSOCIATE_F32: Self = NumericalContractBuilder::strict_f32()
        .input_subnormals(SIGN_PRESERVING_FLUSH)
        .result_subnormals(SIGN_PRESERVING_FLUSH)
        .reassociation(NumericalPermission::Permitted)
        .resolved();

    /// Every freedom refused in `bf16`; subnormals preserved on both dimensions.
    ///
    /// **Not [`Self::STRICT_F32`] read in another width, and the difference is
    /// measured rather than stylistic.** The two resolve every dimension
    /// identically and are still different contracts with different keys,
    /// because the behaviour they require is required of different arithmetic:
    /// on one measured Apple row `f32` flushes, `f16` preserves, and `bf16`
    /// flushes, so a target's answer to this contract is not its answer to the
    /// `f32` one and neither may be read off the other.
    ///
    /// A caller states it when it needs `bf16` gradual underflow preserved and
    /// would rather not run than run wrong. The measured macOS Apple9 row
    /// declares a sign-preserving flush for `bf16`, so that profile refuses this
    /// contract by name.
    pub const STRICT_BF16: Self = NumericalContractBuilder::strict_bf16().resolved();

    /// Strict `bf16`, except that both subnormal dimensions flush to the
    /// sign-preserving zero.
    ///
    /// The `bf16` counterpart of [`Self::FLUSH_SUBNORMALS_TO_ZERO_F32`], and a
    /// separately measured claim rather than a translation of it: the two widths
    /// are declared by independent rows on a target profile, and the measured
    /// Apple row that flushes in both preserves in `f16`.
    ///
    /// It widens exactly two dimensions; accepting flushing does not thereby
    /// accept reassociated sums.
    pub const FLUSH_SUBNORMALS_TO_ZERO_BF16: Self = NumericalContractBuilder::strict_bf16()
        .input_subnormals(SIGN_PRESERVING_FLUSH)
        .result_subnormals(SIGN_PRESERVING_FLUSH)
        .resolved();

    /// The arithmetic type every resolution in this contract is stated for.
    ///
    /// Reported because a caller branching on a refusal needs to distinguish
    /// "this target cannot preserve `bf16` subnormals" from the same sentence
    /// about `f32`, and the refusal names the width precisely so it can.
    #[must_use]
    pub const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
    }

    /// The treatment of subnormal operands before arithmetic.
    #[must_use]
    pub const fn input_subnormals(self) -> SubnormalMode {
        self.input_subnormals
    }

    /// The treatment of newly produced subnormal results.
    #[must_use]
    pub const fn result_subnormals(self) -> SubnormalMode {
        self.result_subnormals
    }

    /// Whether fusing a multiply into an adjacent add is permitted.
    #[must_use]
    pub const fn contraction(self) -> NumericalPermission {
        self.contraction
    }

    /// Whether ordered regrouping of one same-operation operand sequence is
    /// permitted.
    #[must_use]
    pub const fn reassociation(self) -> NumericalPermission {
        self.reassociation
    }

    /// Whether changing a reduction's logical contributor order is permitted.
    #[must_use]
    pub const fn permutation(self) -> NumericalPermission {
        self.permutation
    }

    /// Whether eliminating the two signed zeros' distinction is permitted.
    #[must_use]
    pub const fn signed_zero(self) -> NumericalPermission {
        self.signed_zero
    }

    /// Whether replacing a division by a reciprocal multiplication is permitted.
    #[must_use]
    pub const fn reciprocal_transform(self) -> NumericalPermission {
        self.reciprocal_transform
    }

    /// The maximum accuracy envelope an approximate intrinsic may consume.
    #[must_use]
    pub const fn approximate_intrinsics(self) -> ApproximationEnvelope {
        self.approximate_intrinsics
    }

    /// Whether NaN operands may be assumed absent, and on what evidence.
    #[must_use]
    pub const fn nan_assumptions(self) -> ExceptionalValueAssumption {
        self.nan_assumptions
    }

    /// Whether infinite operands may be assumed absent, and on what evidence.
    #[must_use]
    pub const fn infinity_assumptions(self) -> ExceptionalValueAssumption {
        self.infinity_assumptions
    }

    /// The rounding an observable materialization boundary applies.
    #[must_use]
    pub const fn materialization_rounding(self) -> MaterializationRounding {
        self.materialization_rounding
    }

    /// The canonical, injective key identifying this contract.
    ///
    /// **Derived from the dimension vector, not chosen.** Two contracts that
    /// resolve any dimension differently have different keys, and two callers
    /// that resolve every dimension alike produce the same key — which is what
    /// lets an artifact, a cache entry, and an explain trace name the contract
    /// they were produced under without a table of names that could never have
    /// covered the space. This is the same string
    /// [`Compilation::resolved_numerical_contract_key`] reports.
    #[must_use]
    pub fn key(self) -> &'static str {
        self.resolve().key
    }

    /// Resolves this stated contract into the complete internal contract.
    ///
    /// `pub(crate)` because the named constants above are the **one** spelling of
    /// each named contract: the internal constructors in `request` resolve
    /// through them rather than repeating a vector, so a named point cannot be
    /// widened in one place and not the other.
    pub(crate) fn resolve(self) -> StrictF32NumericalContract {
        StrictF32NumericalContract {
            arithmetic: self.arithmetic,
            canonical_arithmetic_nan_bits: self.canonical_arithmetic_nan_bits,
            input_subnormals: self.input_subnormals,
            result_subnormals: self.result_subnormals,
            contraction: self.contraction,
            reassociation: self.reassociation,
            permutation: self.permutation,
            signed_zero: self.signed_zero,
            reciprocal_transform: self.reciprocal_transform,
            approximate_intrinsics: self.approximate_intrinsics,
            nan_assumptions: self.nan_assumptions,
            infinity_assumptions: self.infinity_assumptions,
            materialization_rounding: self.materialization_rounding,
            // The strict resolution of the *stated* width, so the base a
            // composition is completed from names the same arithmetic the
            // composition does. Reading the base off the governed `f32` contract
            // would give a `bf16` statement an `f32` NaN pattern.
            ..crate::policy::strict_contract(self.arithmetic, self.canonical_arithmetic_nan_bits)
        }
        .keyed()
    }
}

/// The flush behaviour the measured Apple row delivers in `f32` and in `bf16`.
///
/// One constant for both widths because it names a *behaviour*, not a target
/// answer: which widths deliver it is a per-dtype profile row, and the measured
/// Apple `f16` row deliberately does not.
const SIGN_PRESERVING_FLUSH: SubnormalMode = SubnormalMode::FlushToZero {
    zero_sign: FlushedZeroSign::PreservesSign,
};

/// Resolves a numerical contract one dimension at a time.
///
/// **Checked construction, so an incoherent contract is not a value that
/// exists.** [`Self::build`] is the only way to reach a [`NumericalContract`]
/// that is not one of the named constants, and it refuses a self-contradictory
/// vector by name. A caller therefore never holds a contract that has not been
/// assessed for coherence, and no boundary below has to re-derive the question.
///
/// Every resolver is `const` and consumes and returns the builder, so a
/// composition reads as the strict contract with the dimensions the caller moved
/// named beside it — which is exactly the sentence a reader has to be able to
/// write about a program's meaning.
///
/// ```
/// use tiler_compiler::session::{NumericalContract, NumericalContractBuilder};
/// use tiler_ir::schedule::{FlushedZeroSign, NumericalPermission, SubnormalMode};
///
/// let flush = SubnormalMode::FlushToZero {
///     zero_sign: FlushedZeroSign::PreservesSign,
/// };
/// let composed = NumericalContractBuilder::strict_f32()
///     .input_subnormals(flush)
///     .result_subnormals(flush)
///     .reassociation(NumericalPermission::Permitted)
///     .build()
///     .expect("flushing and regrouping are independent dimensions");
/// assert_eq!(composed, NumericalContract::FLUSH_AND_REASSOCIATE_F32);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NumericalContractBuilder(NumericalContract);

impl NumericalContractBuilder {
    /// Starts an `f32` composition with every dimension at its strict
    /// resolution.
    ///
    /// Every entry point is per width and says so in its name. Starting from a
    /// laxer contract would make an omitted dimension inherit a freedom the
    /// caller never stated, which is the one direction a numerical default must
    /// not have; and there is deliberately no width-free entry point, because an
    /// omitted *width* is the same failure one level up — a contract that spoke
    /// for every float width at once would state something the measurements
    /// already refute for one of them.
    #[must_use]
    pub const fn strict_f32() -> Self {
        Self::strict(
            ArithmeticType::F32,
            tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
        )
    }

    /// Starts a `bf16` composition with every dimension at its strict
    /// resolution.
    ///
    /// The `bf16` sibling of [`Self::strict_f32`], and a separate statement
    /// rather than a mode of it: the two produce different contracts with
    /// different keys even when every dimension is resolved alike, because they
    /// require their behaviour of different arithmetic and a target declares the
    /// two independently.
    ///
    /// **Statable, and now planned.** The semantic registry admits
    /// `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and
    /// `tiler::add-bf16@1`; a profile can declare measured `bf16`
    /// honourability; and each of the three families carries both a governed
    /// index-access lowering and a governed fusion-capability row, so a `bf16`
    /// program whose profile dispatches the dtype and whose contract that
    /// profile honours is recognized, planned, and selected — for a region
    /// covering one occurrence and for one covering several. In
    /// `crates/tiler-compiler/tests/bf16_numerical_contract.rs`,
    /// `a_flush_accepting_bf16_contract_reaches_a_selected_plan` asserts the
    /// first and
    /// `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` the
    /// second, where its predecessor asserted a refusal.
    ///
    /// **It is still not general support, and a fused `bf16` region proves less
    /// than that it fused.** Two boundaries survive the widening and are named
    /// here rather than left to be read off the outcome.
    ///
    /// *Reassociation is not proved at this width; it is merely never
    /// required.* `BF16_FACT_REASSOCIATION_PERMITTED` is `false` and no
    /// registered `bf16` family declares an algebraic capability, so nothing in
    /// this build establishes that regrouping preserves meaning against an
    /// 8-bit significand — the question stays open at the operation vocabulary
    /// instead of being answered by a region that happened to fuse.
    ///
    /// *The four reduction obligations discharge vacuously, over an empty
    /// population.* `tiler-ir` registers no `bf16` family carrying a fold at
    /// all, so there is no `bf16` contributor sequence for an identity, an
    /// empty domain, an order, or a regrouping to be about. That is evidence
    /// that no reduction was present, never that a `bf16` reduction would be
    /// legal.
    ///
    /// The wall moved rather than vanished: a `bf16` region under a
    /// contraction-*permitting* contract still stops, because nothing in a
    /// bounded profile establishes that a fused `bf16` body will decline a
    /// permission the contract granted.
    /// `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall`
    /// asserts that boundary in the same file.
    ///
    /// **On the one profile this build ships, a `bf16` request does not reach
    /// fusion legality at all — it is refused a phase earlier.** The
    /// authoritative macOS Apple9 ledger declares `bf16` dispatchability and the
    /// two `bf16` subnormal tables and nothing else; its seven reshaping and
    /// exceptional-value rows are each stated for the `f32` subject. So a
    /// flush-accepting `bf16` contract clears the measured subnormal dimensions
    /// and then meets an *undeclared* one — contraction, canonically first among
    /// the remaining consumable dimensions — whose disposition is `Unknown`, and
    /// the target refuses at numerical resolution before any cover is
    /// enumerated. That the refusal names contraction rather than a subnormal
    /// dimension is itself the evidence that the flush was honoured.
    /// `the_measured_subnormal_rows_alone_leave_the_remaining_dimensions_unknown`
    /// asserts that exact rejection against a profile restating the ledger's own
    /// rows, so widening the ledger moves that test rather than passing
    /// silently. Reaching a plan on this target needs the missing `bf16` rows
    /// measured, not a further optimizer widening.
    #[must_use]
    pub const fn strict_bf16() -> Self {
        Self::strict(
            ArithmeticType::Bf16,
            tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS as u32,
        )
    }

    /// The strict resolution of every dimension, for one stated width.
    ///
    /// Private, and the one place the strict vector is spelled: two per-width
    /// copies would be two places for "strict" to drift, and the whole
    /// fail-closed argument rests on every entry point starting from the same
    /// resolution.
    const fn strict(arithmetic: ArithmeticType, canonical_arithmetic_nan_bits: u32) -> Self {
        Self(NumericalContract {
            arithmetic,
            canonical_arithmetic_nan_bits,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
            permutation: NumericalPermission::Forbidden,
            signed_zero: NumericalPermission::Forbidden,
            reciprocal_transform: NumericalPermission::Forbidden,
            approximate_intrinsics: ApproximationEnvelope::Forbidden,
            nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            materialization_rounding: MaterializationRounding::NearestTiesToEven,
        })
    }

    /// Resolves the treatment of subnormal operands before arithmetic.
    #[must_use]
    pub const fn input_subnormals(mut self, mode: SubnormalMode) -> Self {
        self.0.input_subnormals = mode;
        self
    }

    /// Resolves the treatment of newly produced subnormal results.
    ///
    /// Independent of [`Self::input_subnormals`] (ADR 0019): a target that
    /// couples the two in one execution mode declares that coupling on its own
    /// profile and never collapses the semantic dimensions here.
    #[must_use]
    pub const fn result_subnormals(mut self, mode: SubnormalMode) -> Self {
        self.0.result_subnormals = mode;
        self
    }

    /// Resolves whether fusing a multiply into an adjacent add is permitted.
    #[must_use]
    pub const fn contraction(mut self, permission: NumericalPermission) -> Self {
        self.0.contraction = permission;
        self
    }

    /// Resolves whether ordered regrouping of one same-operation operand
    /// sequence is permitted.
    ///
    /// This is the dimension every parallel reduction strategy consumes: a
    /// multi-pass split and a single-workgroup tree both regroup the declared
    /// contributor sequence while retaining its leaves and their order.
    #[must_use]
    pub const fn reassociation(mut self, permission: NumericalPermission) -> Self {
        self.0.reassociation = permission;
        self
    }

    /// Resolves whether changing a reduction's logical contributor order is
    /// permitted.
    ///
    /// Independent of [`Self::reassociation`] (ADR 0014). A permuted contributor
    /// sequence folded strictly left to right is a well-defined different sum
    /// that consumes no regrouping at all, so neither permission carries the
    /// other.
    #[must_use]
    pub const fn permutation(mut self, permission: NumericalPermission) -> Self {
        self.0.permutation = permission;
        self
    }

    /// Resolves whether eliminating the two signed zeros' distinction is
    /// permitted.
    #[must_use]
    pub const fn signed_zero(mut self, permission: NumericalPermission) -> Self {
        self.0.signed_zero = permission;
        self
    }

    /// Resolves whether replacing a division by a reciprocal multiplication is
    /// permitted.
    #[must_use]
    pub const fn reciprocal_transform(mut self, permission: NumericalPermission) -> Self {
        self.0.reciprocal_transform = permission;
        self
    }

    /// Resolves the maximum accuracy envelope an approximate intrinsic may
    /// consume.
    ///
    /// An envelope rather than a boolean, because an unbounded approximation is
    /// not a contract a reference evaluation or a backend intrinsic can be
    /// checked against.
    #[must_use]
    pub const fn approximate_intrinsics(mut self, envelope: ApproximationEnvelope) -> Self {
        self.0.approximate_intrinsics = envelope;
        self
    }

    /// Resolves whether NaN operands may be assumed absent, and on what
    /// evidence.
    #[must_use]
    pub const fn nan_assumptions(mut self, assumption: ExceptionalValueAssumption) -> Self {
        self.0.nan_assumptions = assumption;
        self
    }

    /// Resolves whether infinite operands may be assumed absent, and on what
    /// evidence.
    #[must_use]
    pub const fn infinity_assumptions(mut self, assumption: ExceptionalValueAssumption) -> Self {
        self.0.infinity_assumptions = assumption;
        self
    }

    /// Resolves the rounding an observable materialization boundary applies.
    #[must_use]
    pub const fn materialization_rounding(mut self, rounding: MaterializationRounding) -> Self {
        self.0.materialization_rounding = rounding;
        self
    }

    /// Assesses the composed vector and returns the contract it states.
    ///
    /// # Errors
    ///
    /// Returns [`IncoherentNumericalContract`] when the composed dimensions
    /// contradict each other. The enumeration is small and its derivation —
    /// including the combinations that were considered and are *not*
    /// contradictions — is on that type.
    pub fn build(self) -> Result<NumericalContract, IncoherentNumericalContract> {
        // Assessed on the resolved internal contract rather than on the public
        // vector, so the coherence rule has exactly one implementation and an
        // internally constructed contract is held to the same statement.
        crate::request::coherence(&self.0.resolve()).map_err(|cause| match cause {
            crate::request::IncoherentContract::UnfoundedValueDomainProvenance {
                dimension,
                provenance,
            } => IncoherentNumericalContract::UnfoundedValueDomainProvenance {
                dimension: match dimension {
                    ExceptionalValueDimensionKind::Nan => ExceptionalValueDimension::Nan,
                    ExceptionalValueDimensionKind::Infinity => ExceptionalValueDimension::Infinity,
                },
                provenance,
            },
        })?;
        Ok(self.0)
    }

    /// Returns the composed contract without assessing it.
    ///
    /// `const`, and reachable only from this module's own named constants, which
    /// resolve no exceptional-value assumption at all and therefore cannot be
    /// incoherent. `named_contracts_are_coherent` drives every one of them
    /// through [`Self::build`] so the claim is checked rather than asserted.
    const fn resolved(self) -> NumericalContract {
        self.0
    }
}

/// Which exceptional value an absence was stated about.
///
/// A typed pair rather than the internal dimension vocabulary, because these are
/// the only two dimensions an absence can be stated on and a caller branching on
/// the refusal should not have to match a wider set than the refusal can carry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExceptionalValueDimension {
    /// NaN operands.
    Nan,
    /// Infinite operands.
    Infinity,
}

impl ExceptionalValueDimension {
    /// The stable diagnostic key naming this dimension.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Nan => "numerics.nan-assumptions",
            Self::Infinity => "numerics.infinity-assumptions",
        }
    }
}

/// Why a composed dimension vector is not a contract this build will hold.
///
/// **Enumerated, not discovered.** Composition lets a caller state combinations
/// a four-value enumeration could not, so the combinations that are *not*
/// contracts are named here rather than found in the field. The enumeration is
/// deliberately small, and the eliminations are stated so a reader can refute the
/// list rather than only read it.
///
/// **What survives.** Exactly one: a contract may not assert a value-domain
/// absence on evidence it is not the author of. Compiler-proven provenance names
/// a conclusion this compiler reaches from verified producers, constants, or
/// analysis; runtime-validated provenance names a guard that runs before any
/// plan relying on it executes, which this build neither emits nor checks. A
/// caller-stated absence therefore carries
/// [`tiler_ir::schedule::ValueDomainProvenance::CallerDeclaredUnvalidated`], and
/// a caller asserting either of the other two is claiming somebody else's
/// evidence.
///
/// **What was eliminated.** Assuming NaNs absent does not contradict the
/// canonical arithmetic NaN pattern (one governs an operand, the other a produced
/// value). Assuming one exceptional value absent and not the other is independent
/// by ADR 0011. Permitted signed-zero elimination does not contradict a
/// sign-preserving flush, because the flush's zero sign is carried on the
/// behaviour precisely so no permission can leave it unspecified; nor does a
/// forbidden signed-zero elimination contradict a flush to always-positive zero,
/// because a declared flush is a stated result rather than a rewrite. Permitted
/// contraction with forbidden reassociation is ADR 0015's separation, and
/// permitted permutation with forbidden reassociation is ADR 0014's.
///
/// **What this is not.** A contract this build cannot *realize* is a different
/// refusal — [`CompileFailureClass::InvalidRequest`] with rule
/// `compile.request.numerics.unrepresentable`, naming the dimension, the
/// behaviour this build realizes, and the operation that would consume it — and a
/// contract a *target* cannot honour is a third, reported per dimension by
/// [`TargetCompileRefusal::NumericalContract`]. Coherence is about the statement
/// alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IncoherentNumericalContract {
    /// A stated absence claims provenance the caller cannot be the author of.
    UnfoundedValueDomainProvenance {
        /// The exceptional-value dimension the absence was stated on.
        dimension: ExceptionalValueDimension,
        /// The provenance class the composition asserted.
        provenance: tiler_ir::schedule::ValueDomainProvenance,
    },
}

impl fmt::Display for IncoherentNumericalContract {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnfoundedValueDomainProvenance {
                dimension,
                provenance,
            } => write!(
                formatter,
                "{} states an absence on {} provenance, which a caller is not the author of",
                dimension.key(),
                match provenance {
                    tiler_ir::schedule::ValueDomainProvenance::CompilerProven => "compiler-proven",
                    tiler_ir::schedule::ValueDomainProvenance::RuntimeValidated =>
                        "runtime-validated",
                    tiler_ir::schedule::ValueDomainProvenance::CallerDeclaredUnvalidated =>
                        "caller-declared-unvalidated",
                }
            ),
        }
    }
}

impl std::error::Error for IncoherentNumericalContract {}

/// The installed lowering authority a compilation resolves occurrences through.
///
/// This is the half of the compiler boundary that was missing. Everything needed
/// to *build* a registry was already public — [`crate::capability`]'s builder,
/// the provider traits, `LoweringSignature`, `ProviderIdentity` — and nothing
/// could install one, so an out-of-crate provider could be written and never
/// reached the compile path. ADR 0078 item 4 names that asymmetry and states its
/// closing condition as exactly this: a public path by which a caller supplies
/// its own [`FrozenLoweringCapabilityRegistry`] to a compilation.
///
/// An opaque wrapper rather than the internal snapshot, so the request model
/// behind it stays private and the caller's obligation is the one that matters:
/// pairing the installed lowering registry with the scalar authority it was
/// registered against. Request preflight derives realization-law authority from
/// the exact program semantic registry and refuses unless the lowering/scalar
/// pair has full semantic and law-sidecar coherence with that program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledCapabilities(CompilerCapabilitySnapshot);

impl InstalledCapabilities {
    /// The lowering capabilities this build ships.
    #[must_use]
    pub fn governed() -> Self {
        Self(CompilerCapabilitySnapshot::governed())
    }

    /// A caller's lowering registry with its scalar and semantic authority.
    ///
    /// Request preflight derives realization-law authority from the program's
    /// exact semantic registry; callers cannot replace the law that defines an
    /// operation while installing its lowering. A mismatched pair is refused
    /// rather than silently reconciled.
    #[must_use]
    pub fn installed(
        lowering: FrozenLoweringCapabilityRegistry,
        scalars: FrozenScalarRegistry,
    ) -> Self {
        Self(CompilerCapabilitySnapshot::new(lowering, scalars))
    }
}

/// One compilation a caller composes.
///
/// Built and consumed: [`compile`] takes it by value, so a request cannot be
/// submitted twice or mutated after the compiler has begun reading it.
///
/// The inputs a caller may state are deliberately fewer than the internal
/// request carries. Budgets stay internal because they admit exactly one
/// governed value today. The shape environment is the program's own and is
/// not caller-supplied: two environments over one program is the ambiguity
/// the program constructor exists to prevent. Target declaration is accepted
/// only through [`crate::target::TargetProfileBuilder`], which validates and
/// freezes the whole profile before it can enter this request.
#[derive(Clone, Debug)]
pub struct CompileRequest<'a> {
    program: &'a SemanticProgram,
    contracts: Vec<NumericalContract>,
    targets: TargetRequest,
    capabilities: InstalledCapabilities,
    physical: InstalledPhysicalProviders<'a>,
}

impl<'a> CompileRequest<'a> {
    /// States the program to compile and the contract it means.
    ///
    /// Capabilities default to [`InstalledCapabilities::governed`], which is
    /// what makes [`compile_governed`] expressible through this same path.
    #[must_use]
    pub fn new(
        program: &'a SemanticProgram,
        contract: NumericalContract,
        targets: TargetRequest,
    ) -> Self {
        Self {
            program,
            contracts: vec![contract],
            targets,
            capabilities: InstalledCapabilities::governed(),
            physical: InstalledPhysicalProviders::governed(),
        }
    }

    /// States an ordered preference over several acceptable contracts.
    ///
    /// **Why the compiler needs the list rather than the caller retrying.** A
    /// caller that compiled under the strictest contract, saw a refusal, and
    /// tried the next one would get the same answer only by accident: the
    /// compiler would never have seen the alternatives, so it could not record
    /// which were acceptable, could not bind them into the request subject, and
    /// could not tell a reader which one it resolved to. The stated list is part
    /// of what the compilation *is*, not a retry policy outside it — two
    /// requests accepting different fallbacks are different requests even when
    /// they resolve identically.
    ///
    /// Order is the caller's and is preserved into request identity, so a
    /// reordered list is a different subject.
    ///
    /// # Errors
    ///
    /// Returns a [`CompileFailure`] classed
    /// [`CompileFailureClass::InvalidRequest`] for an empty list. There is no
    /// default and no implicit strictest reading: a request stating no contract
    /// does not compile.
    pub fn preferring(
        program: &'a SemanticProgram,
        contracts: impl IntoIterator<Item = NumericalContract>,
        targets: TargetRequest,
    ) -> Result<Self, CompileFailure> {
        let contracts: Vec<NumericalContract> = contracts
            .into_iter()
            .take(MAX_NUMERICAL_CONTRACT_PREFERENCES + 1)
            .collect();
        if contracts.is_empty() {
            return Err(CompileFailure::from(CompileError::InvalidRequest(
                RequestError::UnstatedNumericalContract,
            )));
        }
        if contracts.len() > MAX_NUMERICAL_CONTRACT_PREFERENCES {
            return Err(CompileFailure::from(CompileError::InvalidRequest(
                RequestError::TooManyNumericalContracts {
                    actual: contracts.len(),
                    max: MAX_NUMERICAL_CONTRACT_PREFERENCES,
                },
            )));
        }
        if contracts
            .iter()
            .enumerate()
            .any(|(index, contract)| contracts[..index].contains(contract))
        {
            return Err(CompileFailure::from(CompileError::InvalidRequest(
                RequestError::DuplicateNumericalContract,
            )));
        }
        Ok(Self {
            program,
            contracts,
            targets,
            capabilities: InstalledCapabilities::governed(),
            physical: InstalledPhysicalProviders::governed(),
        })
    }

    /// Installs the lowering authority this compilation resolves through.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: InstalledCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Installs the physical-implementation providers this compilation enumerates.
    ///
    /// **The asymmetry with [`Self::with_capabilities`] is the point and is not
    /// an oversight.** An installed lowering registry *replaces* the governed
    /// one, because exactly one authority may say what an occurrence means and
    /// two claimants are a contradiction. An installed physical provider is
    /// *added to* the governed one, because several correct implementations of
    /// one verified region are alternatives the compiler retains side by side
    /// and ranks on cost. Neither rule may be generalized to the other seam.
    ///
    /// Installation is not selection. A provider installed here is asked about
    /// every region subject, and each body it proposes is re-verified, checked
    /// against the request-subject binding, and decided feasible or not by this
    /// host before it can compete. It may then contribute to no retained plan,
    /// which the *pair* of [`Compilation::offered_physical_providers`] and
    /// [`PlanAlternative::selected_physical_providers`] tells apart from never
    /// having been asked — offered and unselected against absent from both.
    /// Neither accessor alone draws that distinction.
    #[must_use]
    pub fn with_physical_providers(mut self, physical: InstalledPhysicalProviders<'a>) -> Self {
        self.physical = physical;
        self
    }
}

/// Compiles one caller-composed request.
///
/// # Errors
///
/// Returns a [`CompileFailure`] naming the class of boundary that refused. See
/// [`CompileFailureClass`] for what each class means and which of them are
/// statements about the request rather than about Tiler.
pub fn compile(request: CompileRequest<'_>) -> Result<CompilationBatch, CompileFailure> {
    let CompileRequest {
        program,
        contracts,
        targets,
        capabilities,
        physical,
    } = request;
    let stated: Vec<_> = contracts
        .iter()
        .map(|contract| contract.resolve())
        .collect();
    let preference = NumericalContractPreference::ordered(stated)
        .map_err(|error| CompileFailure::from(CompileError::InvalidRequest(error)))?;
    let offered_lowering_providers: Arc<[ProviderIdentity]> =
        Arc::from(capabilities.0.lowering().providers());
    // Minted from the same value the frontier is handed below, so the reported
    // environment cannot name a provider the enumeration was not given.
    let offered_physical_providers: Arc<[ProviderIdentity]> =
        Arc::from(physical.offered_identities());
    let mut internal = CompilationRequest::governed_preferring(program, preference);
    let expected_targets = targets.profiles().to_vec();
    internal.target_profiles = targets.into_profiles();
    internal.capabilities = capabilities.0;
    let product = compile_with_physical_providers(internal, physical.providers())?;
    into_compilation_batch(
        product,
        &expected_targets,
        &offered_lowering_providers,
        &offered_physical_providers,
    )
    .map_err(CompileFailure::from)
}

/// Compiles one semantic program under a stated numerical contract.
///
/// It is the bounded convenience profile and not a second compile path: it
/// composes the same [`CompileRequest`] a caller would and calls the same
/// [`compile`]. One path rather than two is what stops the convenient one and
/// the general one from drifting, and expressing this wrapper through the
/// general surface is the cheapest proof that surface is usable at all.
///
/// The convenience is the *success* shape. Because the composed request names
/// exactly one target, the returned [`CompilationBatch`] has exactly one slot,
/// and this function unwraps it to that slot's [`Compilation`] rather than
/// making every caller destructure a batch it did not compose. That unwrapping
/// does not extend to the refusal, which is returned whole.
///
/// # Errors
///
/// Returns the single governed target's [`TargetCompileFailure`], so a refusal
/// arriving here carries what the same refusal carries through [`compile`]:
/// [`class`](TargetCompileFailure::class), the sealed
/// [`explain`](TargetCompileFailure::explain) trace when the refusal followed
/// the trace boundary, and the typed pre-trace
/// [`refusal`](TargetCompileFailure::refusal) detail when it preceded it — a
/// numerical-contract or dtype-dispatch rejection naming its exact subject.
///
/// A failure raised before the target slot exists — a request the compiler
/// refuses as a whole, or this function's own governed-cardinality check — is
/// reported in the same type carrying no refusal detail. One target is
/// requested, so there is no distinction left for a second error type to draw:
/// either way the governed compilation did not happen, and the class names
/// which boundary said so.
///
/// The classes stay distinct as [`compile`] mints them: an unsupported program,
/// an infeasible target, and an exhausted budget are statements about the
/// request, while invalid compiler output is a defect in Tiler.
///
/// # Panics
///
/// Panics only if the compiler's built-in governed profile violates its own
/// construction invariants.
pub fn compile_governed(
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<Compilation, TargetCompileFailure> {
    let targets = TargetRequest::new([TargetProfile::governed()])
        .expect("the governed singleton target request is valid");
    let mut batch = compile(CompileRequest::new(program, contract, targets))
        .map_err(TargetCompileFailure::before_any_target)?
        .into_targets();
    if batch.len() != 1 {
        return Err(TargetCompileFailure::before_any_target(
            CompileFailure::from(CompileError::InvalidCompilerOutput(
                crate::pipeline::CompilerOutputError::Program(
                    crate::program::ProgramError::Structure {
                        rule: "public-governed-target-cardinality",
                    },
                ),
            )),
        ));
    }
    let (_, outcome) = batch
        .pop()
        .expect("the governed target cardinality was checked")
        .into_parts();
    outcome
}

fn public_elementary_accuracy_reason(
    reason: &'static str,
    half: Option<ElementaryEvidenceHalf>,
    class: Option<tiler_ir::semantic::accuracy::ConformanceEvidenceClass>,
) -> Result<TargetElementaryAccuracyReason, CompileError> {
    match (reason, half, class) {
        ("accuracy.elementary.no-installed-realization", None, None) => {
            Ok(TargetElementaryAccuracyReason::NoInstalledRealization)
        }
        ("accuracy.elementary.unrefined-realization", None, None) => {
            Ok(TargetElementaryAccuracyReason::Unrefined)
        }
        ("accuracy.elementary.undischarged-evidence", Some(half), Some(class)) => {
            Ok(TargetElementaryAccuracyReason::UndischargedEvidence { half, class })
        }
        _ => Err(CompileError::InvalidCompilerOutput(
            crate::pipeline::CompilerOutputError::Program(
                crate::program::ProgramError::Structure {
                    rule: "public-elementary-accuracy-refusal",
                },
            ),
        )),
    }
}

fn target_compile_failure(error: CompileError) -> Result<TargetCompileFailure, CompileError> {
    let refusal = match target_request_refusal(&error) {
        Some(RequestError::NoResolvableNumericalContract {
            target_profile,
            rejections,
        }) => Some(TargetCompileRefusal::NumericalContract(
            TargetNumericalContractRefusal {
                target_profile: target_profile.clone(),
                rejections: rejections
                    .iter()
                    .map(public_numerical_rejection)
                    .collect::<Result<Vec<_>, _>>()?,
            },
        )),
        Some(RequestError::DTypeNotDispatchable {
            target_profile,
            resolved_type,
            disposition,
        }) => Some(TargetCompileRefusal::DTypeDispatch(
            TargetDTypeDispatchRefusal {
                target_profile: target_profile.clone(),
                resolved_type: resolved_type.as_ref().clone(),
                disposition: match disposition {
                    DTypeDispatchRefusalDisposition::Unsupported => {
                        TargetDTypeRefusalDisposition::Unsupported
                    }
                    DTypeDispatchRefusalDisposition::Deferred { available_at } => {
                        TargetDTypeRefusalDisposition::Deferred {
                            available_at: *available_at,
                        }
                    }
                    DTypeDispatchRefusalDisposition::Unknown => {
                        TargetDTypeRefusalDisposition::Unknown
                    }
                },
            },
        )),
        Some(RequestError::UnrealizedElementaryAccuracy {
            operation,
            target_profile,
            reason,
            undischarged_half,
            undischarged_class,
            candidates,
        }) => Some(TargetCompileRefusal::ElementaryAccuracy(
            TargetElementaryAccuracyRefusal {
                operation: operation.clone(),
                target_profile: target_profile.clone(),
                reason: public_elementary_accuracy_reason(
                    reason,
                    *undischarged_half,
                    *undischarged_class,
                )?,
                candidates: candidates
                    .iter()
                    .map(|candidate| TargetElementaryAccuracyCandidate {
                        contract: candidate.contract().clone(),
                        producer_key: candidate.producer_key().to_owned(),
                        producer_revision: candidate.producer_revision(),
                    })
                    .collect(),
            },
        )),
        Some(
            RequestError::UnsupportedRequestVersion
            | RequestError::MismatchedShapeEnvironment
            | RequestError::EmptyTargetSet
            | RequestError::DuplicateTargetProfile
            | RequestError::UnverifiedTargetSelection
            | RequestError::UnstatedNumericalContract
            | RequestError::DuplicateNumericalContract
            | RequestError::TooManyNumericalContracts { .. }
            | RequestError::UnrepresentableNumericalDimension { .. }
            // No structured refusal, and for the reason the unrepresentable
            // dimension has none: no target was consulted, so there is no
            // target-local answer to carry. It reaches a caller as
            // [`CompileFailureClass::InvalidRequest`] under
            // `compile.request.numerics.inapplicable`, and the `Display` names
            // the program's arithmetic beside every stated contract's.
            | RequestError::NoApplicableNumericalContract { .. }
            | RequestError::BudgetExceeded { .. }
            | RequestError::UnsupportedCapability { .. }
            | RequestError::UnsupportedSymbolicExtent { .. }
            | RequestError::ShapeProductOverflow { .. },
        )
        | None => None,
    };
    Ok(TargetCompileFailure {
        failure: CompileFailure::from(error),
        refusal: refusal.map(Box::new),
    })
}

fn target_request_refusal(error: &CompileError) -> Option<&RequestError> {
    match error {
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(error))
        | CompileError::UnsupportedCapability(
            error @ RequestError::UnrealizedElementaryAccuracy { .. },
        ) => Some(error),
        CompileError::Explained { source, .. } => target_request_refusal(source),
        CompileError::InvalidRequest(_)
        | CompileError::UnsupportedCapability(_)
        | CompileError::BudgetExhausted(_)
        | CompileError::NoFeasiblePlan(
            NoFeasiblePlanError::Physical(_) | NoFeasiblePlanError::Selection(_),
        )
        | CompileError::InvalidCompilerOutput(_) => None,
    }
}

fn public_numerical_rejection(
    rejection: &ContractRejection,
) -> Result<TargetNumericalContractRejection, CompileError> {
    let subject = TargetNumericalSubject {
        arithmetic: rejection.arithmetic(),
        resolved_type: rejection.resolved_type().clone(),
    };
    // The serialized form of the same subject, so a declared relaxation's
    // identity can be compared against it without reconstructing a resolved type
    // from one-way canonical bytes.
    let rejection_subject = ScalarArithmeticSubjectIdentity::from_parts(
        rejection.arithmetic(),
        rejection.resolved_type().canonical_encoding().as_bytes(),
    )
    .ok_or_else(|| {
        CompileError::InvalidCompilerOutput(crate::pipeline::CompilerOutputError::Program(
            ProgramError::Structure {
                rule: "public-numerical-subject-identity",
            },
        ))
    })?;
    let requirement =
        public_numerical_requirement(rejection.dimension(), rejection.required(), subject.clone())?;
    let disposition = match rejection {
        ContractRejection::Unhonourable { cause, .. } => {
            let means = match cause.means() {
                HonouringMeans::SupportedExactly => TargetNumericalDeclaredMeans::SupportedExactly,
                HonouringMeans::SupportedWithExactEmulation => {
                    TargetNumericalDeclaredMeans::SupportedWithExactEmulation
                }
                HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { relaxation } => {
                    // The relaxation carries its subject as the serialized
                    // identity, which is one-way: nothing reconstructs a
                    // `ResolvedValueType` from it. The profile builder refuses a
                    // relaxation naming a subject other than the one its
                    // declaration speaks about, so the rejection's own subject
                    // *is* the relaxation's -- but that is checked here rather
                    // than assumed, because a builder that stopped enforcing it
                    // would otherwise make this boundary report a requirement
                    // stated for the wrong dtype.
                    if relaxation.subject() != &rejection_subject {
                        return Err(CompileError::InvalidCompilerOutput(
                            crate::pipeline::CompilerOutputError::Program(
                                ProgramError::Structure {
                                    rule: "public-numerical-relaxation-subject",
                                },
                            ),
                        ));
                    }
                    TargetNumericalDeclaredMeans::SupportedOnlyUnderDeclaredRelaxation {
                        required: public_numerical_requirement(
                            relaxation.dimension(),
                            relaxation.behaviour(),
                            subject.clone(),
                        )?,
                    }
                }
                HonouringMeans::Unsupported => TargetNumericalDeclaredMeans::Unsupported,
            };
            let honoured = cause
                .honoured()
                .map(|behaviour| public_honoured_behaviour(rejection.dimension(), behaviour))
                .transpose()?;
            let declared = public_numerical_requirement(
                rejection.dimension(),
                cause.declared(),
                subject.clone(),
            )?;
            TargetNumericalRefusalDisposition::DeclaredUnhonourable(Box::new(
                TargetDeclaredNumericalRefusal {
                    subject,
                    declared,
                    means,
                    honoured,
                    target_profile: cause.profile().public_key().clone(),
                    // Borrowed from the retained fact rather than rebuilt: the
                    // conversion above narrows the declaration to what the
                    // dimension-safe requirement vocabulary can spell, and the
                    // evidence is the part that must not be narrowed at all.
                    evidence: cause.evidence(),
                },
            ))
        }
        ContractRejection::Undeclared { .. } => TargetNumericalRefusalDisposition::Unknown,
        ContractRejection::Deferred { cause, .. } => TargetNumericalRefusalDisposition::Deferred {
            available_at: cause.phase(),
        },
    };
    Ok(TargetNumericalContractRejection {
        contract_key: rejection.contract_key(),
        requirement,
        disposition,
    })
}

fn public_numerical_requirement(
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
    subject: TargetNumericalSubject,
) -> Result<TargetNumericalRequirement, CompileError> {
    let requirement = match (dimension, behaviour) {
        (NumericalDimension::InputSubnormals, DimensionBehaviour::Subnormals(required)) => {
            TargetNumericalRequirement::InputSubnormals { subject, required }
        }
        (NumericalDimension::ResultSubnormals, DimensionBehaviour::Subnormals(required)) => {
            TargetNumericalRequirement::ResultSubnormals { subject, required }
        }
        (NumericalDimension::Contraction, DimensionBehaviour::Transform(required)) => {
            TargetNumericalRequirement::Contraction { subject, required }
        }
        (NumericalDimension::Reassociation, DimensionBehaviour::Transform(required)) => {
            TargetNumericalRequirement::Reassociation { subject, required }
        }
        (NumericalDimension::Permutation, DimensionBehaviour::Transform(required)) => {
            TargetNumericalRequirement::Permutation { subject, required }
        }
        (NumericalDimension::SignedZero, DimensionBehaviour::Transform(required)) => {
            TargetNumericalRequirement::SignedZero { subject, required }
        }
        (NumericalDimension::ReciprocalTransform, DimensionBehaviour::Transform(required)) => {
            TargetNumericalRequirement::ReciprocalTransform { subject, required }
        }
        (
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(required),
        ) => TargetNumericalRequirement::ApproximateIntrinsics { subject, required },
        (NumericalDimension::NanAssumptions, DimensionBehaviour::ExceptionalValue(required)) => {
            TargetNumericalRequirement::NanAssumptions { subject, required }
        }
        (
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(required),
        ) => TargetNumericalRequirement::InfinityAssumptions { subject, required },
        (NumericalDimension::MaterializationRounding, DimensionBehaviour::Rounding(required)) => {
            TargetNumericalRequirement::MaterializationRounding { subject, required }
        }
        _ => {
            return Err(CompileError::InvalidCompilerOutput(
                crate::pipeline::CompilerOutputError::Program(ProgramError::Structure {
                    rule: "public-numerical-requirement-shape",
                }),
            ));
        }
    };
    Ok(requirement)
}

fn public_honoured_behaviour(
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
) -> Result<TargetNumericalHonouredBehaviour, CompileError> {
    let behaviour = match (dimension, behaviour) {
        (NumericalDimension::InputSubnormals, DimensionBehaviour::Subnormals(value)) => {
            TargetNumericalHonouredBehaviour::InputSubnormals(value)
        }
        (NumericalDimension::ResultSubnormals, DimensionBehaviour::Subnormals(value)) => {
            TargetNumericalHonouredBehaviour::ResultSubnormals(value)
        }
        (NumericalDimension::Contraction, DimensionBehaviour::Transform(value)) => {
            TargetNumericalHonouredBehaviour::Contraction(value)
        }
        (NumericalDimension::Reassociation, DimensionBehaviour::Transform(value)) => {
            TargetNumericalHonouredBehaviour::Reassociation(value)
        }
        (NumericalDimension::Permutation, DimensionBehaviour::Transform(value)) => {
            TargetNumericalHonouredBehaviour::Permutation(value)
        }
        (NumericalDimension::SignedZero, DimensionBehaviour::Transform(value)) => {
            TargetNumericalHonouredBehaviour::SignedZero(value)
        }
        (NumericalDimension::ReciprocalTransform, DimensionBehaviour::Transform(value)) => {
            TargetNumericalHonouredBehaviour::ReciprocalTransform(value)
        }
        (NumericalDimension::ApproximateIntrinsics, DimensionBehaviour::Approximation(value)) => {
            TargetNumericalHonouredBehaviour::ApproximateIntrinsics(value)
        }
        (NumericalDimension::NanAssumptions, DimensionBehaviour::ExceptionalValue(value)) => {
            TargetNumericalHonouredBehaviour::NanAssumptions(value)
        }
        (NumericalDimension::InfinityAssumptions, DimensionBehaviour::ExceptionalValue(value)) => {
            TargetNumericalHonouredBehaviour::InfinityAssumptions(value)
        }
        (NumericalDimension::MaterializationRounding, DimensionBehaviour::Rounding(value)) => {
            TargetNumericalHonouredBehaviour::MaterializationRounding(value)
        }
        _ => {
            return Err(CompileError::InvalidCompilerOutput(
                crate::pipeline::CompilerOutputError::Program(ProgramError::Structure {
                    rule: "public-numerical-honoured-shape",
                }),
            ));
        }
    };
    Ok(behaviour)
}

fn into_compilation_batch(
    product: CompilationProduct,
    expected_targets: &[TargetProfile],
    offered_lowering_providers: &Arc<[ProviderIdentity]>,
    offered_physical_providers: &Arc<[ProviderIdentity]>,
) -> Result<CompilationBatch, CompileError> {
    if product.targets.len() != expected_targets.len() {
        return Err(CompileError::InvalidCompilerOutput(
            crate::pipeline::CompilerOutputError::Program(
                crate::program::ProgramError::Structure {
                    rule: "public-target-outcome-cardinality",
                },
            ),
        ));
    }
    for (expected, outcome) in expected_targets.iter().zip(&product.targets) {
        let actual = outcome.target_profile();
        if actual.profile_key() != expected.profile_key()
            || actual.canonical_descriptor() != expected.canonical_descriptor()
        {
            return Err(CompileError::InvalidCompilerOutput(
                crate::pipeline::CompilerOutputError::Program(
                    crate::program::ProgramError::Structure {
                        rule: "public-target-outcome-binding",
                    },
                ),
            ));
        }
    }
    let targets = product
        .targets
        .into_iter()
        .map(|outcome| match outcome {
            TargetCompilationOutcome::Compiled(target) => {
                let target_profile = target.target_profile.clone();
                let compilation = Compilation {
                    stated_contracts: target.stated_contracts,
                    resolved_contract: target.resolved_contract,
                    offered_lowering_providers: Arc::clone(offered_lowering_providers),
                    offered_physical_providers: Arc::clone(offered_physical_providers),
                    target_profile: target.target_profile,
                    feasibility_rule_set: target.feasibility_rule_set,
                    selected_alternative_id: target.portfolio.selection.selected_alternative_id,
                    alternatives: target.portfolio.alternatives,
                    explain: target.compilation_explain,
                };
                Ok(TargetCompilationResult {
                    target_profile,
                    outcome: Ok(compilation),
                })
            }
            TargetCompilationOutcome::Rejected {
                target_profile,
                failure,
            } => Ok(TargetCompilationResult {
                target_profile,
                outcome: Err(target_compile_failure(failure)?),
            }),
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    Ok(CompilationBatch { targets })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        BudgetRefusal, BudgetResource, CompilationRequest, CompileFailure, CompileFailureClass,
        CompileRequest, NumericalContract, StrictF32NumericalContract, TargetCompileRefusal,
        TargetElementaryAccuracyReason, TargetNumericalRefusalDisposition,
        TargetNumericalRequirement, compile, compile_governed,
    };
    use crate::pipeline::compile as compile_internal;
    use crate::target::{TargetProfile, TargetRequest};
    use tiler_ir::program::abi::{ExprNode, TargetPropertyRequirementRelation};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, F32Silu, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn governed_targets() -> TargetRequest {
        TargetRequest::new([TargetProfile::governed()]).unwrap()
    }

    /// Builds the bounded profile's scale-then-reduce program.
    ///
    /// Constructed through `tiler-ir`'s own public builder rather than borrowed
    /// from another module's test helpers, so these cases exercise the same
    /// surface an out-of-crate caller would use to reach this boundary.
    fn semantic_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 1]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    /// Builds the bare reduction program: one input, one strict serial sum.
    ///
    /// **The program whose honoured contraction no packaged occurrence
    /// consumes.** It is [`semantic_program`] with the scaling multiply and the
    /// bias add removed, so the only occurrence a plan covers is the fold, and
    /// `policy::operation_capabilities`'s reduction row omits contraction
    /// because a strict serial sum's per-contributor step is `accumulator +
    /// contributor` with no product for a fused multiply-add to act on. The
    /// dimension is still *asked of the target*: `physical::region_proposal`
    /// carries every dimension the region realization states on every
    /// candidate, so the plan holds an honoured contraction fact and no row
    /// naming it.
    fn bare_reduction_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 3]))
            .unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    /// The boundary compiles a program and hands out emittable kernels.
    ///
    /// This is the property the surface exists for: before it, no caller
    /// outside this crate could obtain a `VerifiedKernel` at all, so no backend
    /// could emit and nothing could execute.
    #[test]
    fn a_governed_program_compiles_to_alternatives_carrying_kernels() {
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        assert!(!compilation.target_profile_key().is_empty());

        let alternatives: Vec<_> = compilation.alternatives().collect();
        assert!(
            alternatives.len() >= 2,
            "the bounded profile retains a fused and a materialized alternative",
        );
        assert!(
            alternatives.iter().any(PlanAlternativeExt::fused),
            "a fused alternative is retained",
        );
        assert!(
            alternatives.iter().any(|plan| !plan.is_fused()),
            "the materialized reference alternative is retained",
        );
        for plan in &alternatives {
            assert!(
                std::ptr::eq(plan.compilation(), &raw const compilation),
                "every alternative retains the exact compilation that owns it",
            );
            assert!(
                !plan.kernels().is_empty(),
                "{} dispatches at least one kernel",
                plan.stable_id(),
            );
            for selected in plan.selected_capabilities() {
                assert!(
                    compilation
                        .offered_lowering_providers()
                        .contains(selected.provider()),
                    "a selected lowering provider must belong to the offered lowering environment",
                );
            }
            for selected in plan.selected_physical_providers() {
                assert!(
                    compilation
                        .offered_physical_providers()
                        .contains(selected.provider()),
                    "a selected physical provider must belong to the offered physical environment",
                );
            }
        }
    }

    /// **The two offered environments are disjoint subjects, not one set split.**
    ///
    /// A governed compilation offers a lowering registry and one governed
    /// physical provider, and neither identity appears in the other's set. The
    /// populations are counted rather than shape-asserted, so a future second
    /// governed physical provider cannot make this silently assert less, and the
    /// disjointness is what stops a reader treating either accessor as a partial
    /// view of one environment.
    #[test]
    fn the_offered_lowering_and_physical_environments_are_separate_populations() {
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");

        let lowering = compilation.offered_lowering_providers();
        let physical = compilation.offered_physical_providers();
        assert!(
            !lowering.is_empty(),
            "the governed compilation offered no lowering provider",
        );
        assert_eq!(
            physical.len(),
            1,
            "a governed compilation is offered exactly the governed physical provider: {physical:?}",
        );
        for identity in physical {
            assert!(
                !lowering.contains(identity),
                "{identity} appears in both offered environments",
            );
        }
    }

    trait PlanAlternativeExt {
        fn fused(&self) -> bool;
    }

    impl PlanAlternativeExt for super::PlanAlternative<'_> {
        fn fused(&self) -> bool {
            self.is_fused()
        }
    }

    /// A selected plan honours every realized dimension and delivers a row for
    /// each one this program consumes.
    ///
    /// The four dimensions `region_proposal` used to drop — permutation, signed
    /// zero, NaN assumptions, infinity assumptions — are named rather than
    /// counted, so omitting any one production projection fails this test
    /// without the assertion changing.
    #[test]
    fn a_selected_plan_delivers_every_realized_dimension() {
        use std::collections::BTreeSet;
        use tiler_ir::numerics::NumericalDimension;

        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        let plan = compilation.selected().expect("a selected alternative");
        let honoured: BTreeSet<NumericalDimension> = plan
            .alternative
            .plan
            .honoured()
            .iter()
            .map(crate::target::honourability::HonouredDimension::dimension)
            .collect();
        let realized: BTreeSet<NumericalDimension> =
            crate::policy::REALIZED_DIMENSIONS.into_iter().collect();
        assert_eq!(
            honoured, realized,
            "region feasibility must ask the target about every realized dimension",
        );

        let delivered: BTreeSet<NumericalDimension> = plan
            .delivered_realization()
            .obligations()
            .map(crate::session::SelectedObligation::dimension)
            .collect();
        assert_eq!(
            delivered, realized,
            "the scale-then-reduce program consumes every realized dimension, so \
             none may be missing from selected delivered evidence",
        );
        for dimension in [
            NumericalDimension::Permutation,
            NumericalDimension::SignedZero,
            NumericalDimension::NanAssumptions,
            NumericalDimension::InfinityAssumptions,
        ] {
            assert!(
                delivered.contains(&dimension),
                "{} must reach selected delivered evidence",
                dimension.key(),
            );
        }
    }

    /// Every retained plan offers one complete eleven-dimension policy subject.
    ///
    /// The completeness is what makes an unenumerated dimension fail closed: a
    /// subject that could be short would let a record be silent about a
    /// dimension, and silence is what ADR 0076 item 4 exists to remove.
    #[test]
    fn every_retained_plan_offers_one_complete_scalar_arithmetic_subject() {
        use tiler_ir::numerics::{CANONICAL_DIMENSIONS, DimensionBehaviour, NumericalDimension};
        use tiler_ir::schedule::ArithmeticType;

        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        for plan in compilation.alternatives() {
            let view = plan.delivered_realization();
            assert_eq!(view.profile_key(), compilation.target_profile_key());
            assert_eq!(
                view.profile_descriptor(),
                compilation.target_profile_descriptor(),
            );
            let subjects: Vec<_> = view.scalar_arithmetic().collect();
            assert_eq!(
                subjects.len(),
                1,
                "one selected scalar contract yields one subject",
            );
            let contract = subjects[0];
            assert_eq!(contract.subject().arithmetic(), ArithmeticType::F32);
            // The strict resolution of every governed dimension, read through
            // the dense array rather than through eleven named getters.
            for dimension in CANONICAL_DIMENSIONS {
                let behaviour = contract.resolution(dimension);
                assert!(
                    dimension.admits(behaviour),
                    "{dimension} resolves inside its own behaviour space",
                );
                assert_eq!(
                    contract.resolutions()[dimension.index()],
                    behaviour,
                    "the dense array and the accessor are one lookup",
                );
            }
            assert_eq!(
                contract.resolution(NumericalDimension::Contraction),
                DimensionBehaviour::Transform(tiler_ir::schedule::NumericalPermission::Forbidden),
                "the strict contract forbids contraction",
            );
        }
    }

    /// Every obligation names the offered subject, a covered occurrence, and the
    /// exact checked fact that honours it.
    ///
    /// Each coordinate is asserted rather than the count alone: an obligation
    /// pointing at a neighbouring dimension's fact, or at an occurrence the
    /// packaged program does not cover, would otherwise read as evidence.
    #[test]
    fn every_obligation_carries_its_own_checked_evidence() {
        use tiler_ir::numerics::{HonouringMeans, NumericalDimension, PolicyLocus};
        use tiler_ir::program::abi::AvailabilityPhase;

        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        let plan = compilation.selected().expect("a selected alternative");
        let view = plan.delivered_realization();
        let contract = view
            .scalar_arithmetic()
            .next()
            .expect("one offered subject");
        let subject = contract.subject().clone();
        let covered: Vec<u32> = plan
            .abi()
            .kernel_program()
            .stages()
            .flat_map(|stage| {
                stage
                    .coverage()
                    .iter()
                    .map(|record| record.occurrence().get())
                    .collect::<Vec<_>>()
            })
            .collect();

        let obligations: Vec<_> = view.obligations().collect();
        assert!(
            !obligations.is_empty(),
            "a strict contract over admitted operations places obligations on the target",
        );
        for obligation in obligations {
            assert_eq!(obligation.subject(), &subject);
            // The locus is founded on the dimension's own definition, so it is
            // asserted per dimension rather than against one constant. The two
            // reshaping freedoms are the pair that genuinely depends on the
            // operation — a fold puts them at its accumulator and pointwise
            // arithmetic at its own computation — so only those admit two
            // answers here; `the_locus_follows_the_operation_at_the_occurrence`
            // is what pins which occurrence takes which.
            let locus = obligation.locus().locus();
            match obligation.dimension() {
                NumericalDimension::InputSubnormals => assert_eq!(locus, PolicyLocus::Input),
                NumericalDimension::ResultSubnormals => assert_eq!(locus, PolicyLocus::Result),
                NumericalDimension::Permutation => assert_eq!(locus, PolicyLocus::Accumulator),
                NumericalDimension::SignedZero
                | NumericalDimension::NanAssumptions
                | NumericalDimension::InfinityAssumptions => {
                    assert_eq!(locus, PolicyLocus::Computation);
                }
                NumericalDimension::Contraction | NumericalDimension::Reassociation => assert!(
                    matches!(locus, PolicyLocus::Computation | PolicyLocus::Accumulator),
                    "{} is founded on the arithmetic or on a fold, never elsewhere",
                    obligation.dimension(),
                ),
                dimension @ (NumericalDimension::ReciprocalTransform
                | NumericalDimension::ApproximateIntrinsics
                | NumericalDimension::MaterializationRounding) => {
                    panic!("{dimension} is unconsumable, so no route founds an obligation on it")
                }
            }
            assert!(obligation.locus().is_well_formed());
            assert!(
                covered.contains(&obligation.locus().occurrence().get()),
                "an obligation names an occurrence the packaged program covers",
            );
            assert!(obligation.dimension().admits(obligation.required()));

            let evidence = obligation.evidence();
            // Not `declared() == required()`: today's `HonouredDimension`
            // resolves both from one field, so that comparison could not fail.
            // The subject's dense resolution is materialized from the resolved
            // contract and the obligation's behaviour from the retained fact, so
            // this comparison crosses two independently built structures.
            assert_eq!(
                evidence.declared(),
                contract.resolution(obligation.dimension()),
                "the retained fact speaks about the behaviour the contract resolved",
            );
            assert_eq!(evidence.profile_key(), compilation.target_profile_key());
            assert_eq!(
                evidence.profile_descriptor(),
                compilation.target_profile_descriptor(),
            );
            assert!(
                evidence.source().is_valid(),
                "every carried provenance is complete and internally consistent",
            );
            assert!(
                evidence.source().phase() <= AvailabilityPhase::ArtifactEvidence,
                "a produced artifact rests only on facts readable by the time it exists",
            );
            // The means is read structurally, not through its non-injective
            // presentation label: the label is what a record could not carry.
            assert!(matches!(
                evidence.means(),
                HonouringMeans::SupportedExactly
                    | HonouringMeans::SupportedWithExactEmulation
                    | HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { .. }
            ));
        }
    }

    /// One dimension's locus follows the operation realized at the occurrence.
    ///
    /// **The capability this ticket adds, stated as the difference between two
    /// positions of one program.** `semantic_program` applies two constants, a
    /// pointwise multiply, a pointwise add, and a strict serial sum. Ordered
    /// reassociation acts on all three arithmetic families, but not in the same
    /// place: the sum folds a contributor sequence, so its regrouping is a
    /// property of the accumulator, while the multiply and the add have no fold
    /// and regroup their own arithmetic. A producer keyed by dimension alone —
    /// or pinned at one locus, as this one was — states both as the same
    /// position and cannot tell an accumulator obligation from a computation
    /// one, which is precisely the collapse ADR 0011 names.
    ///
    /// The constants are the other half: a constant retains its declared bit
    /// pattern, so no arithmetic freedom acts on it anywhere, and it carries no
    /// row rather than a `Computation` row asserting a position nothing founds.
    #[test]
    fn the_locus_follows_the_operation_at_the_occurrence() {
        use std::collections::{BTreeMap, BTreeSet};
        use tiler_ir::numerics::{NumericalDimension, PolicyLocus};

        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        let plan = compilation.selected().expect("a selected alternative");
        let view = plan.delivered_realization();

        let covered: BTreeSet<u32> = plan
            .abi()
            .kernel_program()
            .stages()
            .flat_map(|stage| {
                stage
                    .coverage()
                    .iter()
                    .map(|record| record.occurrence().get())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            covered.len(),
            5,
            "two constants, a multiply, an add, and a sum are packaged",
        );

        // Every row, keyed the way the record keys it. Counted rather than
        // sampled: a locus derivation that emitted nothing would otherwise pass
        // every `matches!` below without ever running.
        let rows: Vec<(u32, NumericalDimension, PolicyLocus)> = view
            .obligations()
            .map(|obligation| {
                (
                    obligation.locus().occurrence().get(),
                    obligation.dimension(),
                    obligation.locus().locus(),
                )
            })
            .collect();
        assert_eq!(
            rows.len(),
            21,
            "eight honoured dimensions over three arithmetic occurrences, less the \
             one contraction the fold cannot consume and the two permutation rows \
             the pointwise multiply and add cannot consume",
        );

        // The constants consume nothing, so they are covered and unobligated.
        let obligated: BTreeSet<u32> = rows.iter().map(|(occurrence, _, _)| *occurrence).collect();
        assert!(obligated.is_subset(&covered));
        assert_eq!(
            obligated.len(),
            3,
            "the two constants carry no row: no arithmetic freedom acts on a \
             retained bit pattern, so there is no position to name",
        );

        // The load-bearing comparison: one dimension, three occurrences, two
        // different founded positions.
        let reassociation: BTreeMap<u32, PolicyLocus> = rows
            .iter()
            .filter(|(_, dimension, _)| *dimension == NumericalDimension::Reassociation)
            .map(|(occurrence, _, locus)| (*occurrence, *locus))
            .collect();
        assert_eq!(
            reassociation.len(),
            3,
            "every arithmetic occurrence regroups something",
        );
        let folding: Vec<u32> = reassociation
            .iter()
            .filter(|(_, locus)| **locus == PolicyLocus::Accumulator)
            .map(|(occurrence, _)| *occurrence)
            .collect();
        let pointwise: Vec<u32> = reassociation
            .iter()
            .filter(|(_, locus)| **locus == PolicyLocus::Computation)
            .map(|(occurrence, _)| *occurrence)
            .collect();
        assert_eq!(
            folding.len(),
            1,
            "the strict serial sum is the one fold this program packages, and its \
             regrouping is a property of its accumulator",
        );
        assert_eq!(
            pointwise.len(),
            2,
            "the multiply and the add have no fold, so they regroup their own \
             arithmetic",
        );
        let fold = folding[0];

        // Cross-checked against a second dimension rather than restated. The
        // occurrence whose regrouping sits at an accumulator is the same one
        // consuming no contraction, because a strict serial sum's step is
        // `accumulator + contributor` with no product for a fused multiply-add
        // to act on. Two independently derived row sets agreeing on which
        // occurrence folds is what makes the split a property of the operation
        // rather than of iteration order.
        let contraction: Vec<u32> = rows
            .iter()
            .filter(|(_, dimension, _)| *dimension == NumericalDimension::Contraction)
            .map(|(occurrence, _, locus)| {
                assert_eq!(*locus, PolicyLocus::Computation);
                *occurrence
            })
            .collect::<BTreeSet<u32>>()
            .into_iter()
            .collect();
        assert!(!contraction.contains(&fold));
        assert_eq!(
            contraction, pointwise,
            "the occurrences that can fuse a multiply into an add are exactly the \
             two whose regrouping is not an accumulator's",
        );
    }

    /// Two loci of one occurrence carry genuinely different obligations.
    ///
    /// **The case the single-locus producer could not express, and the reason
    /// the record keys a position at all.** The contract below flushes subnormal
    /// *operands* while preserving subnormal *results* — one arithmetic type,
    /// two positions, two different legal requirements. Keyed by dtype alone the
    /// second statement overwrites the first; pinned at one locus, both rows
    /// land on `Computation` and the record says one position requires two
    /// contradictory subnormal behaviours. Keyed by the founded position they
    /// are the operand read and the produced value, and each is separately
    /// checkable against the target fact that honours it.
    #[test]
    fn two_loci_of_one_occurrence_carry_different_obligations() {
        use crate::session::NumericalContractBuilder;
        use std::collections::BTreeMap;
        use tiler_ir::numerics::{DimensionBehaviour, NumericalDimension, PolicyLocus};
        use tiler_ir::schedule::{FlushedZeroSign, SubnormalMode};

        let flush = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        };
        let contract = NumericalContractBuilder::strict_f32()
            .input_subnormals(flush)
            .build()
            .expect("flushing operands while preserving results is coherent");
        assert_ne!(
            flush,
            SubnormalMode::Preserve,
            "the two dimensions genuinely differ, or this proves nothing",
        );

        let program = semantic_program();
        let compilation =
            compile_governed(&program, contract).expect("the asymmetric contract compiles");
        let plan = compilation.selected().expect("a selected alternative");
        let view = plan.delivered_realization();

        // Every subnormal row, by occurrence and position.
        let mut by_occurrence: BTreeMap<u32, BTreeMap<PolicyLocus, DimensionBehaviour>> =
            BTreeMap::new();
        for obligation in view.obligations() {
            if !matches!(
                obligation.dimension(),
                NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals
            ) {
                continue;
            }
            let previous = by_occurrence
                .entry(obligation.locus().occurrence().get())
                .or_default()
                .insert(obligation.locus().locus(), obligation.required());
            assert!(
                previous.is_none(),
                "one position states one requirement per dimension",
            );
        }
        assert_eq!(
            by_occurrence.len(),
            3,
            "the three arithmetic occurrences each read operands and produce a result",
        );

        for (occurrence, loci) in &by_occurrence {
            assert_eq!(
                loci.len(),
                2,
                "occurrence {occurrence} carries an operand read and a produced value",
            );
            let input = loci
                .get(&PolicyLocus::Input)
                .expect("the operand read is founded by `InputSubnormals`");
            let result = loci
                .get(&PolicyLocus::Result)
                .expect("the produced value is founded by `ResultSubnormals`");
            assert_eq!(*input, DimensionBehaviour::Subnormals(flush));
            assert_eq!(
                *result,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
            );
            // The whole point: same occurrence, same dtype, different demands.
            assert_ne!(
                input, result,
                "occurrence {occurrence} requires different behaviour at its two \
                 positions, which a dtype-wide ceiling cannot state and a \
                 single-locus producer collapsed onto one key",
            );
        }
    }

    /// An honoured dimension no covered occurrence consumes carries no row.
    ///
    /// **The producer half of the newly reachable `NotRequired` disposition.**
    /// Narrowing rows to consuming occurrences made an empty row set possible
    /// for a dimension the target was asked about and *did* honour, which the
    /// artifact builder derives as `NotRequired` — the one producer assertion
    /// the neutral artifact cannot re-check. Before the narrowing every
    /// honoured dimension had a row at every covered occurrence, so this state
    /// could not be produced at all.
    ///
    /// The two halves are asserted separately because either alone would be
    /// consistent with a defect. The honoured facts come from the retained
    /// plan, so contraction being among them is what rules out "nothing asked";
    /// the row set comes from the delivered-realization view, so its emptiness
    /// on that dimension is what makes the disposition `NotRequired` rather
    /// than `Required`. A producer that had simply dropped the requirement
    /// would fail the first assertion, and one that still emitted an unfounded
    /// computation row would fail the second.
    ///
    /// [`semantic_program`] is compiled beside it under the identical contract
    /// and profile and *does* carry contraction rows, so the difference is the
    /// program's operations rather than the contract, the target, or the
    /// dimension.
    #[test]
    fn an_honoured_dimension_no_covered_occurrence_consumes_carries_no_row() {
        use std::collections::BTreeSet;
        use tiler_ir::numerics::{NumericalDimension, PolicyLocus};

        let program = bare_reduction_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the bare reduction compiles");
        let plan = compilation.selected().expect("a selected alternative");

        // The dimensions the target was asked about and answered for. Read from
        // the retained plan's own honoured facts rather than from the contract,
        // because the claim under test is that the target *did* honour
        // contraction here and the row set is empty anyway.
        let honoured: BTreeSet<NumericalDimension> = plan
            .alternative
            .plan
            .honoured()
            .iter()
            .map(crate::target::honourability::HonouredDimension::dimension)
            .collect();
        assert_eq!(
            honoured,
            BTreeSet::from([
                NumericalDimension::InputSubnormals,
                NumericalDimension::ResultSubnormals,
                NumericalDimension::Contraction,
                NumericalDimension::Reassociation,
                NumericalDimension::Permutation,
                NumericalDimension::SignedZero,
                NumericalDimension::NanAssumptions,
                NumericalDimension::InfinityAssumptions,
            ]),
            "the region proposal asks every candidate about every dimension the \
             region realization states, so the honoured set is a property of \
             the profile and the contract rather than of this program",
        );

        let covered: BTreeSet<u32> = plan
            .abi()
            .kernel_program()
            .stages()
            .flat_map(|stage| {
                stage
                    .coverage()
                    .iter()
                    .map(|record| record.occurrence().get())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            covered.len(),
            1,
            "the fold is the only occurrence this program packages",
        );

        let rows: Vec<(u32, NumericalDimension, PolicyLocus)> = plan
            .delivered_realization()
            .obligations()
            .map(|obligation| {
                (
                    obligation.locus().occurrence().get(),
                    obligation.dimension(),
                    obligation.locus().locus(),
                )
            })
            .collect();
        assert_eq!(
            rows.len(),
            7,
            "seven of the eight honoured dimensions act on the one covered \
             occurrence: {rows:?}",
        );
        let stated: BTreeSet<NumericalDimension> =
            rows.iter().map(|(_, dimension, _)| *dimension).collect();
        assert_eq!(
            stated,
            BTreeSet::from([
                NumericalDimension::InputSubnormals,
                NumericalDimension::ResultSubnormals,
                NumericalDimension::Reassociation,
                NumericalDimension::Permutation,
                NumericalDimension::SignedZero,
                NumericalDimension::NanAssumptions,
                NumericalDimension::InfinityAssumptions,
            ]),
        );
        assert!(
            !stated.contains(&NumericalDimension::Contraction),
            "the strict serial sum's step is `accumulator + contributor`, so no \
             product exists for a fused multiply-add to act on and the honoured \
             contraction fact founds no position in this program",
        );

        // The capability table is what decides that, and it is read here rather
        // than assumed from the empty row set.
        let fold = crate::policy::operation_capability(
            &tiler_ir::semantic::OpKey::new("tiler", "strict-serial-sum-f32", 1)
                .expect("a governed key"),
        )
        .expect("the strict serial sum is admitted");
        assert!(!fold.can_consume(NumericalDimension::Contraction));
        assert!(
            crate::policy::operation_capability(&tiler_ir::semantic::multiply_f32_op())
                .expect("the multiply is admitted")
                .can_consume(NumericalDimension::Contraction),
            "contraction is consumable by some admitted operation, so this is a \
             property of the reduction rather than of the dimension",
        );

        // The same contract on the same profile, over a program that does
        // contain a pointwise multiply and add.
        let consuming = compile_governed(&semantic_program(), NumericalContract::STRICT_F32)
            .expect("the scale-then-reduce program compiles");
        let contraction_rows = consuming
            .selected()
            .expect("a selected alternative")
            .delivered_realization()
            .obligations()
            .filter(|obligation| obligation.dimension() == NumericalDimension::Contraction)
            .count();
        assert_eq!(
            contraction_rows, 2,
            "the multiply and the add each found a contraction position, so the \
             empty set above is the program's doing",
        );
    }

    /// Every named contract this module publishes survives its own assessment.
    ///
    /// **The claim [`super::NumericalContractBuilder::resolved`] makes about
    /// itself, checked rather than asserted.** That constructor is `const` and
    /// skips coherence, so each named constant reaches a caller unassessed on
    /// the argument that none of them resolves an exceptional-value assumption.
    /// Driving each through
    /// [`super::NumericalContractBuilder::build`] — the one gate a composed
    /// contract passes — is what turns that argument into a checked fact, and it
    /// wraps the constant itself rather than respelling its dimension vector, so
    /// a constant that moved is assessed in its moved form.
    ///
    /// The population is counted rather than iterated over silently: a named
    /// contract added without a row here would leave this test passing about the
    /// seven it still names.
    #[test]
    fn named_contracts_are_coherent() {
        let named = [
            ("STRICT_F32", NumericalContract::STRICT_F32),
            (
                "FLUSH_SUBNORMALS_TO_ZERO_F32",
                NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            ),
            ("RELAXED_F32", NumericalContract::RELAXED_F32),
            ("REASSOCIATE_F32", NumericalContract::REASSOCIATE_F32),
            (
                "FLUSH_AND_REASSOCIATE_F32",
                NumericalContract::FLUSH_AND_REASSOCIATE_F32,
            ),
            ("STRICT_BF16", NumericalContract::STRICT_BF16),
            (
                "FLUSH_SUBNORMALS_TO_ZERO_BF16",
                NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
            ),
        ];
        assert_eq!(
            named.len(),
            7,
            "a named contract was added or removed without moving this row",
        );
        for (name, contract) in named {
            let assessed = super::NumericalContractBuilder(contract)
                .build()
                .unwrap_or_else(|cause| panic!("{name} is not a coherent contract: {cause}"));
            assert_eq!(
                assessed, contract,
                "{name} did not survive its own assessment unchanged",
            );
        }
    }

    /// The selected alternative is one of the retained ones.
    #[test]
    fn the_selection_names_a_retained_alternative() {
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        let selected = compilation.selected().expect("a selected alternative");
        assert!(
            compilation
                .alternatives()
                .any(|plan| plan.stable_id() == selected.stable_id()),
        );
    }

    /// A successful compilation carries a renderable composite explanation.
    #[test]
    fn a_compilation_renders_its_explain_trace() {
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect("the governed program compiles");
        assert_eq!(compilation.explain().semantic_candidate_count(), 1);
        let rendered = compilation.explain().render();
        assert!(
            rendered.starts_with("tiler-compilation-explain-v1 "),
            "the composite explanation renders in its deterministic form",
        );
    }

    /// Which of ADR 0069's five classes the public surface can actually produce.
    ///
    /// **Reachability is a property worth pinning, not an assumption.** Three of
    /// the five are reached from the public surface by tests in this module; the
    /// other two are recorded below with the reason, because a class nothing can
    /// produce is a different claim from one that is merely untested.
    ///
    /// - `UnsupportedCapability` — reached by this test's identity program, and
    ///   again by `a_refusal_before_the_trace_boundary_carries_no_trace`.
    /// - `NoFeasiblePlan` — reached by
    ///   `target_outcomes_preserve_request_order_cardinality_and_profile_identity`
    ///   through a profile that declares no strict-`f32` behaviour.
    /// - `BudgetExhausted` — reached by a program that exceeds a deterministic
    ///   budget. `RequestError::BudgetExceeded` is still its sole carrier, and
    ///   it is raised from two places: `check_program_budgets`, for a program
    ///   whose *size* a bound refuses before any target compiles, and the empty
    ///   portfolio, for a target whose *analysis* a bound truncated before it
    ///   reached a plan. `tests/region_search_budget_coverage.rs` reaches the
    ///   first from this surface with a sixty-three-operation chain, one past
    ///   `semantic_operations`.
    ///
    ///   **The second is no longer reachable through this surface, and that is
    ///   a consequence of a decision rather than a gap.**
    ///   `derive-the-region-shape-budgets-from-the-declaration` made
    ///   `region_members` and `region_live_values` the same formulas
    ///   `semantic_operations` and `semantic_values` are, so a program large
    ///   enough for a region bound to truncate its analysis is refused for its
    ///   *size* first — which is the envelope disagreement that ticket
    ///   dissolved. Reaching the empty portfolio now needs a caller-stated
    ///   budget set, which this surface deliberately does not admit;
    ///   `crate::pipeline::tests::a_region_shape_budget_below_the_only_implementable_cover_reports_the_budget`
    ///   drives it one layer down and is what keeps that path measured.
    /// - `InvalidCompilerOutput` — **unreachable by construction from a valid
    ///   call, deliberately.** It reports that Tiler's own verifier refused
    ///   Tiler's own output, so reaching it from the public surface would mean
    ///   shipping the defect it exists to report.
    /// - `InvalidRequest` — reached by
    ///   `a_stated_preference_is_carried_and_both_halves_are_readable`, whose
    ///   empty preference list states no contract. It was recorded here as
    ///   unreachable while `compile` built the whole request structure itself
    ///   and a caller supplied only a program, a contract, and capabilities;
    ///   the caller now states an ordered contract preference and its own
    ///   target profiles, so the structural facts this class reports — an
    ///   unstated, duplicated, or overlong contract list, an empty or
    ///   duplicated target set — are the caller's to get wrong.
    #[test]
    fn the_reachable_failure_classes_are_reached_from_the_public_surface() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 3]))
            .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), input)
            .unwrap();
        let program = builder.build().unwrap();

        let failure = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect_err("the bounded profile does not admit an identity program");
        assert!(
            matches!(
                failure.class(),
                CompileFailureClass::UnsupportedCapability { .. }
            ),
            "a valid program no installed capability covers is an unsupported \
             capability, not a malformed request: {:?}",
            failure.class(),
        );
    }

    /// A stated preference reaches the compiler, and both halves are readable.
    ///
    /// **The fallback has to be visible to the compiler, not applied by the
    /// caller.** A caller that compiled under the strict contract, saw a
    /// refusal, and retried under the relaxed one would get an answer the
    /// compiler never recorded as a preference: the stated list would not be in
    /// the request subject, so two requests accepting different fallbacks would
    /// be indistinguishable, and no reader could tell which contract was used.
    #[test]
    fn a_stated_preference_is_carried_and_both_halves_are_readable() {
        let program = semantic_program();
        let request = CompileRequest::preferring(
            &program,
            [
                NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
                NumericalContract::STRICT_F32,
            ],
            governed_targets(),
        )
        .expect("a non-empty preference is admitted");
        let compilations = compile(request).expect("the governed program compiles");
        let compilation = compilations
            .targets()
            .next()
            .expect("one governed target")
            .outcome()
            .expect("the governed target compiles");

        assert_eq!(
            compilation.stated_numerical_contract_keys().len(),
            2,
            "the whole stated list is retained, not only the winner",
        );
        assert_eq!(
            compilation
                .stated_numerical_contract_keys()
                .next()
                .expect("the request stated a contract"),
            compilation.resolved_numerical_contract_key(),
            "the first acceptable contract is the one resolved on this profile",
        );

        // Order is the caller's, so the reversed list states a different
        // preference even though it names the same two contracts.
        let reversed = CompileRequest::preferring(
            &program,
            [
                NumericalContract::STRICT_F32,
                NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            ],
            governed_targets(),
        )
        .expect("a non-empty preference is admitted");
        let reversed = compile(reversed).expect("the governed program compiles");
        let reversed = reversed
            .targets()
            .next()
            .expect("one governed target")
            .outcome()
            .expect("the governed target compiles");
        assert_ne!(
            reversed
                .stated_numerical_contract_keys()
                .collect::<Vec<_>>(),
            compilation
                .stated_numerical_contract_keys()
                .collect::<Vec<_>>(),
            "a reordered preference is a different stated list",
        );

        // An empty list has no default and no implicit strictest reading.
        let empty = CompileRequest::preferring(&program, [], governed_targets())
            .expect_err("an empty preference states no contract");
        assert!(matches!(
            empty.class(),
            CompileFailureClass::InvalidRequest { .. }
        ));
    }

    #[test]
    fn target_outcomes_preserve_request_order_cardinality_and_profile_identity() {
        let program = semantic_program();
        let supported = TargetProfile::governed_with_key_for_test("test.supported-target.v1");
        let unsupported =
            TargetProfile::without_numerical_declarations_for_test("test.unsupported-target.v1");
        let targets = TargetRequest::new([supported.clone(), unsupported.clone()]).unwrap();

        let batch = compile(CompileRequest::new(
            &program,
            NumericalContract::STRICT_F32,
            targets,
        ))
        .expect("a target-local refusal does not discard the other target");
        let outcomes: Vec<_> = batch.targets().collect();
        assert_eq!(outcomes.len(), 2);
        assert_eq!(outcomes[0].target_profile(), &supported);
        assert_eq!(outcomes[1].target_profile(), &unsupported);

        let compilation = outcomes[0]
            .outcome()
            .expect("the first profile supports strict f32");
        assert_eq!(compilation.target_profile(), outcomes[0].target_profile());
        let failure = outcomes[1]
            .outcome()
            .expect_err("the sparse profile has no strict-f32 declaration");
        assert_eq!(failure.class(), CompileFailureClass::NoFeasiblePlan);
        assert!(
            failure.explain().is_none(),
            "contract honourability is checked before a target trace is opened"
        );
        let TargetCompileRefusal::NumericalContract(refusal) = failure
            .refusal()
            .expect("a pre-trace contract refusal retains typed detail")
        else {
            panic!("the refusal must retain numerical-contract detail");
        };
        assert_eq!(refusal.target_profile(), unsupported.profile_key());
        let [rejection] = refusal.rejections() else {
            panic!("the one-entry preference has exactly one rejection");
        };
        assert_eq!(
            rejection.contract_key(),
            StrictF32NumericalContract::governed().key
        );
        assert_eq!(
            rejection.disposition(),
            &TargetNumericalRefusalDisposition::Unknown
        );
        let TargetNumericalRequirement::InputSubnormals { subject, required } =
            rejection.requirement()
        else {
            panic!("canonical-first strict refusal is input subnormals");
        };
        assert_eq!(subject.resolved_type(), &F32::resolved_type());
        assert_eq!(*required, tiler_ir::schedule::SubnormalMode::Preserve);
    }

    #[test]
    fn public_conversion_rejects_mutated_target_cardinality_and_binding() {
        let program = semantic_program();
        let first = TargetProfile::governed_with_key_for_test("test.binding-first.v1");
        let second = TargetProfile::governed_with_key_for_test("test.binding-second.v1");
        let expected = vec![first.clone(), second.clone()];
        let product = || {
            let mut request = CompilationRequest::governed_under(
                &program,
                StrictF32NumericalContract::governed(),
            );
            request.target_profiles = expected.clone();
            compile_internal(request).expect("both governed-equivalent profiles compile")
        };
        // Both offered environments are empty throughout: these cases perturb
        // the target-outcome correspondence, which is decided before either set
        // is attached to a compilation, so a populated one would only obscure
        // which rule refused.
        let assert_rule = |failure: crate::pipeline::CompileError, expected_rule| {
            assert!(matches!(
                failure,
                crate::pipeline::CompileError::InvalidCompilerOutput(
                    crate::pipeline::CompilerOutputError::Program(
                        crate::program::ProgramError::Structure { rule }
                    )
                ) if rule == expected_rule
            ));
        };

        let mut missing_product = product();
        missing_product.targets.pop();
        let missing = super::into_compilation_batch(
            missing_product,
            &expected,
            &Arc::from([]),
            &Arc::from([]),
        )
        .expect_err("a missing target outcome is invalid compiler output");
        assert_rule(missing, "public-target-outcome-cardinality");

        let mut extra = product();
        extra.targets.push(extra.targets[0].clone());
        let extra = super::into_compilation_batch(extra, &expected, &Arc::from([]), &Arc::from([]))
            .expect_err("an extra target outcome is invalid compiler output");
        assert_rule(extra, "public-target-outcome-cardinality");

        let mut swapped = product();
        swapped.targets.swap(0, 1);
        let swapped =
            super::into_compilation_batch(swapped, &expected, &Arc::from([]), &Arc::from([]))
                .expect_err("equal-cardinality swapped outcomes are invalid compiler output");
        assert_rule(swapped, "public-target-outcome-binding");

        let mut substituted = product();
        match &mut substituted.targets[0] {
            crate::pipeline::TargetCompilationOutcome::Compiled(target) => {
                target.target_profile = second.clone();
            }
            crate::pipeline::TargetCompilationOutcome::Rejected { target_profile, .. } => {
                *target_profile = second.clone();
            }
        }
        let substituted =
            super::into_compilation_batch(substituted, &expected, &Arc::from([]), &Arc::from([]))
                .expect_err("a substituted target key is invalid compiler output");
        assert_rule(substituted, "public-target-outcome-binding");

        let same_key_changed_descriptor =
            TargetProfile::with_grid_axis_limit_for_test(first.profile_key().as_str(), 65_534);
        let mut mismatched = product();
        match &mut mismatched.targets[0] {
            crate::pipeline::TargetCompilationOutcome::Compiled(target) => {
                target.target_profile = same_key_changed_descriptor;
            }
            crate::pipeline::TargetCompilationOutcome::Rejected { target_profile, .. } => {
                *target_profile = same_key_changed_descriptor;
            }
        }
        let mismatched =
            super::into_compilation_batch(mismatched, &expected, &Arc::from([]), &Arc::from([]))
                .expect_err("a same-key descriptor substitution is invalid compiler output");
        assert_rule(mismatched, "public-target-outcome-binding");
    }

    /// The failure vocabulary keeps a Tiler defect distinct from a refused
    /// program, so a caller never reports one as the other.
    #[test]
    fn the_failure_classes_are_distinct() {
        // ADR 0069's five, pairwise. Listing them here rather than asserting two
        // pairs is what makes a future collapse visible: a class merged back
        // into another fails this, where two spot checks would not notice.
        let classes = [
            CompileFailureClass::InvalidRequest { rule: "any" },
            CompileFailureClass::UnsupportedCapability { rule: "any" },
            CompileFailureClass::NoFeasiblePlan,
            CompileFailureClass::BudgetExhausted {
                resource: BudgetResource::SemanticOperations,
                limit: 1,
                reported: 2,
            },
            CompileFailureClass::InvalidCompilerOutput,
        ];
        for (index, left) in classes.iter().enumerate() {
            for right in &classes[index + 1..] {
                assert_ne!(left, right, "two failure classes compare equal");
            }
        }

        // `BudgetExhausted` is the one class whose payload a caller may compare
        // rather than only read, so its fields must participate in equality: a
        // refusal on one budget must not equal a refusal on another, and the two
        // quantities must separate two refusals on the same budget. Without
        // this, carrying the fields would still let a consumer treat every
        // budget refusal as one value, which is the collapse the class exists to
        // avoid one level down.
        let exhausted = |resource, limit, reported| CompileFailureClass::BudgetExhausted {
            resource,
            limit,
            reported,
        };
        assert_ne!(
            exhausted(BudgetResource::SemanticOperations, 62, 63),
            exhausted(BudgetResource::SemanticValues, 62, 63),
        );
        assert_ne!(
            exhausted(BudgetResource::SemanticOperations, 62, 63),
            exhausted(BudgetResource::SemanticOperations, 62, 64),
        );
        assert_ne!(
            exhausted(BudgetResource::SemanticOperations, 62, 63),
            exhausted(BudgetResource::SemanticOperations, 61, 63),
        );

        // The two that were one class carry the same rule key and are still
        // distinct, which is the whole point of splitting them: a caller must
        // not have to read the key to learn which action applies.
        assert_ne!(
            CompileFailureClass::InvalidRequest { rule: "same" },
            CompileFailureClass::UnsupportedCapability { rule: "same" },
        );

        // Public regressions for the two provenances the request-gate chain
        // test cannot reach: a search stop's reported value is a lower bound,
        // and a planning envelope is not an exact demand. Constructed here
        // because only program-scoped exact-demand rows are reachable through
        // `compile_governed` today.
        assert_eq!(
            BudgetResource::RegionExpansions.refusal(),
            BudgetRefusal::SearchLowerBound,
        );
        assert_eq!(
            BudgetResource::HostExpressionNodes.refusal(),
            BudgetRefusal::PlanningUpperBound,
        );
    }

    /// A target compilation that fails hands the caller its complete trace.
    ///
    /// This is the property the boundary previously lacked: the compiler sealed
    /// a trace on the failure path and `From<CompileError>` discarded it, so a
    /// caller receiving `NoFeasiblePlan` had no way to learn which predicate
    /// rejected which alternative — the collapse `docs/compiler/optimizer.md`
    /// forbids.
    ///
    /// It drives the internal request rather than [`compile_governed`] because
    /// reaching a *post-request* failure needs a target the governed profile
    /// does not expose. That is a statement about the bounded profile, not
    /// about the mapping, and the mapping is what this test covers.
    ///
    /// The refusal is a disproved target predicate rather than an exhausted
    /// budget, deliberately: `NoFeasiblePlan` is the class a hard target
    /// rejection carries, and the predicate is the thing the caller could not
    /// read before the trace travelled with the failure.
    #[test]
    fn a_target_failure_carries_its_complete_trace() {
        let program = semantic_program();
        let mut request = CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed_flush_to_zero(),
        );
        // A single-element grid axis is a limit no plan for this program can
        // satisfy, and the frontier disproves it per region.
        request.target_profiles[0] =
            crate::request::TargetProfile::governed_with_grid_axis_limit(1);
        let product =
            compile_internal(request).expect("target-local failure is retained as an outcome");
        let failure = CompileFailure::from(
            product.targets[0]
                .failure()
                .expect("a one-element grid axis admits no plan")
                .clone(),
        );

        assert_eq!(failure.class(), CompileFailureClass::NoFeasiblePlan);
        let rendered = failure
            .explain()
            .expect("a post-request failure retains its sealed trace")
            .render();
        assert!(rendered.starts_with("tiler-explain-v8 "));
        assert!(
            rendered.contains("compiler-failure"),
            "the trace names the terminal failure: {rendered}",
        );
        assert!(
            rendered.contains("region.formation.v1"),
            "the trace names the stage whose candidates were judged: {rendered}",
        );
    }

    /// A refusal that precedes the trace boundary says so, rather than
    /// pretending a trace was withheld.
    ///
    /// The `None` is structural: request verification runs before any
    /// `ExplainWriter` exists, so there is nothing to seal. Asserting it here
    /// keeps the two cases distinguishable at the public surface.
    #[test]
    fn a_refusal_before_the_trace_boundary_carries_no_trace() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 1]))
            .unwrap();
        // An identity program: outside the bounded profile's admitted subject,
        // and refused while verifying the request.
        builder
            .output(OutputKey::new("result").unwrap(), input)
            .unwrap();
        let program = builder.build().unwrap();

        let failure = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect_err("the bounded profile does not admit an identity program");
        assert!(
            matches!(
                failure.class(),
                CompileFailureClass::UnsupportedCapability { .. }
            ),
            "unexpected class: {:?}",
            failure.class(),
        );
        assert!(failure.explain().is_none());
    }

    /// The public structured refusal names a missing declared row.
    ///
    /// The governed profile does not grandfather its three Metal records.
    /// Until those rows can discharge both evidence halves, a `SiLU` program
    /// against the governed profile is `no-installed-realization`.
    #[test]
    fn compile_governed_refuses_silu_as_no_installed_realization() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let activated = F32Silu::apply(&mut builder, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), activated)
            .unwrap();
        let program = builder.build().unwrap();

        let failure = compile_governed(&program, NumericalContract::STRICT_F32)
            .expect_err("the governed profile declares no elementary realization");
        assert_eq!(
            failure.class(),
            CompileFailureClass::UnsupportedCapability {
                rule: "accuracy.elementary.no-installed-realization",
            }
        );
        assert!(failure.explain().is_none());
        let TargetCompileRefusal::ElementaryAccuracy(refusal) = failure
            .refusal()
            .expect("a missing elementary row retains structured detail")
        else {
            panic!(
                "expected an elementary-accuracy refusal, got {:?}",
                failure.refusal()
            );
        };
        assert_eq!(refusal.operation(), &tiler_ir::semantic::silu_f32_op());
        assert_eq!(
            refusal.reason(),
            TargetElementaryAccuracyReason::NoInstalledRealization
        );
        assert!(refusal.candidates().is_empty());
        assert_eq!(
            refusal.reason().diagnostic_code(),
            "accuracy.elementary.no-installed-realization"
        );
    }

    /// The boundary names both assessment identities an artifact must record.
    ///
    /// These are the two halves an assembler needs beside the capability pair:
    /// `TargetProfileRef` wants the profile key and its exact descriptor, and
    /// `FeasibilityRuleSetRef` wants the rule set key and its revision. Both are
    /// asserted from `Compilation` alone, because that is where a consumer has
    /// to be able to reach them without holding an alternative.
    #[test]
    fn a_compilation_names_its_target_profile_and_its_feasibility_rules() {
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32).unwrap();

        // Both halves of the profile reference, neither invented by a consumer.
        assert!(!compilation.target_profile_key().is_empty());
        let descriptor = compilation.target_profile_descriptor();
        assert!(!descriptor.is_empty(), "a descriptor identity is carried");
        // Against the constant this crate publishes, not a literal restating it.
        // The bound is now enforced where a descriptor is minted, so this is a
        // regression guard on the governed profile's size rather than the only
        // thing standing between an oversized profile and a packaging failure.
        assert!(
            descriptor.len() <= crate::target::feasibility::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
            "the governed descriptor fits the bound this crate publishes: {} bytes",
            descriptor.len(),
        );

        // The rule set is a second identity, not a restatement of the profile.
        // Asserting the two keys differ is the point: fusing them would make an
        // artifact claim it was assessed under rules named after a target.
        let rules = compilation.feasibility_rule_set_key();
        assert!(
            rules.starts_with("tiler.feasibility."),
            "unexpected rule set key spelling: {rules}",
        );
        assert_ne!(rules, compilation.target_profile_key());
        assert!(
            compilation.feasibility_rule_set_revision() > 0,
            "zero is reserved for unset at the artifact boundary",
        );

        // Compilation-invariant, not per-alternative: the surface offers one
        // value and every retained alternative was assessed against it.
        assert!(compilation.alternatives().len() >= 2);
    }

    /// The boundary names every capability that lowered, and its ABI inputs.
    ///
    /// Both halves are asserted together because they are the two things an
    /// artifact assembler needs and neither was reachable before: a capability
    /// key it can record without inventing one, and expressions it can replay
    /// without re-deriving one.
    #[test]
    fn an_alternative_names_its_capabilities_and_exposes_its_abi_inputs() {
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::STRICT_F32).unwrap();
        let selected = compilation.selected().expect("a selected alternative");

        // Every resolved capability is named by a governed key, never blank.
        // The spelling is the family and the operation it lowers; the provider
        // is recorded beside it rather than inside it, so two providers of one
        // operation share a key and are still told apart.
        let keys: Vec<_> = selected
            .selected_capabilities()
            .map(|capability| capability.capability_key().to_owned())
            .collect();
        assert!(!keys.is_empty(), "a compiled plan resolved some capability");
        for key in &keys {
            assert!(
                key.starts_with("tiler.capability.index-access.tiler."),
                "unexpected capability key spelling: {key}",
            );
            // Split rather than match a literal suffix: the assertion is that
            // the key ends in a parseable operation version, not that this
            // operation happens to be at version one.
            let version = key.rsplit('.').next().expect("a key has segments");
            assert!(
                version
                    .strip_prefix('v')
                    .is_some_and(|digits| digits.parse::<u32>().is_ok()),
                "key omits the operation version: {key}",
            );
        }
        assert!(
            keys.contains(
                &"tiler.capability.index-access.tiler.strict-serial-sum-f32.v1".to_owned()
            ),
            "the reduction's capability is named: {keys:?}",
        );
        for capability in selected.selected_capabilities() {
            assert_eq!(capability.provider().namespace(), "tiler");
            assert!(capability.capability_revision() > 0);
        }

        // The ABI arena is replayable: every operand precedes the node naming
        // it, which is what lets an assembler mint handles in one forward pass.
        let abi = selected.abi();
        let arena = abi.expressions();
        assert!(!arena.is_empty());
        for (position, node) in arena.iter().enumerate() {
            let position = u32::try_from(position).unwrap();
            match node {
                ExprNode::Root(_) => {}
                ExprNode::Unary { operand, .. } => assert!(*operand < position),
                ExprNode::Binary { left, right, .. } => {
                    assert!(*left < position && *right < position);
                }
                ExprNode::Select {
                    condition,
                    if_true,
                    if_false,
                } => {
                    assert!(*condition < position);
                    assert!(*if_true < position && *if_false < position);
                }
            }
        }

        // Every position the entries name is inside the arena they name it in.
        let bound = u32::try_from(arena.len()).unwrap();
        assert!(abi.applicability_guard() < bound);
        assert_eq!(abi.entries().len(), selected.kernels().len());
        for entry in abi.entries() {
            assert!(entry.grid_threads() < bound);
            assert!(entry.threads_per_workgroup() < bound);
            for accessible in entry.accessible_bytes() {
                assert!(accessible < bound);
            }
        }

        let deferred: Vec<_> = selected.prepared_entry_target_requirements().collect();
        assert_eq!(deferred.len(), selected.kernels().len());
        for (entry, query) in deferred.iter().enumerate() {
            assert_eq!(query.entry(), u32::try_from(entry).unwrap());
            assert_eq!(query.capability_axis(), "threads-per-workgroup");
            let requirement = query.requirement();
            assert_eq!(requirement.required(), 1);
            assert_eq!(
                requirement.relation(),
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            );
            assert_eq!(
                requirement.query().available_at(),
                tiler_ir::program::abi::AvailabilityPhase::PreparedKernelPreflight
            );
            assert_eq!(
                requirement.query().key().as_str(),
                "tiler.target.prepared-entry.max-threads-per-workgroup.v1"
            );
            assert_eq!(requirement.query().provider().namespace(), "tiler");
            assert_eq!(
                requirement.query().provider().name(),
                "prepared-entry-properties"
            );
            assert_eq!(requirement.query().provider().revision(), 1);
        }
    }
}
