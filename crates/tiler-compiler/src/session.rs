//! The public compiler boundary: compile a semantic program, read its plans.
//!
//! This is a **reviewed draft** under ADR 0075 and ADR 0074 convention 7. It is
//! the first surface over which a caller outside this crate can compile
//! anything at all, and until it existed nothing downstream — MSL emission,
//! offline compilation, bundle assembly, execution — could be reached from a
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
//! sealed, through [`CompileFailure::explain`]; before, the boundary discarded
//! it and reported [`CompileFailureClass::NoFeasiblePlan`] with nothing
//! attached, which `docs/compiler/optimizer.md` forbids in terms.
//!
//! Nor does it expose the request. The bounded profile admits exactly one
//! governed configuration — shape environment, numerical contract, budgets,
//! target profile, and installed lowering capabilities — and
//! [`compile_governed`] names that profile rather than letting a caller
//! assemble a request whose fields have no second admissible value yet.
//! Widening it belongs to `select-numerical-contract-and-compose-feasibility`
//! and to the capability-installation work, not to a default this surface
//! should quietly offer.
//!
//! # What a caller gets
//!
//! One [`Compilation`] per requested target profile, each carrying its retained
//! plan alternatives and the policy's selection. Both the fused and the
//! materialized alternative are exposed rather than the selected one alone,
//! because the offline slice compiles the selected program *and* keeps the
//! materialized program as its numerical reference; a selected-only surface
//! could not express that.

use std::fmt;

use tiler_ir::kernel::VerifiedKernel;
use tiler_ir::program::abi::ExprNode;
use tiler_ir::program::{StageRef, VerifiedKernelProgram};
use tiler_ir::semantic::{ProviderIdentity, SemanticProgram};

use crate::capability::FrozenLoweringCapabilityRegistry;
use crate::explain::VerifiedExplainTrace;
use crate::feasibility::FeasibilityRuleSetIdentity;
use crate::pipeline::{
    CompilationProduct, CompileError, ProgramAlternative, ProgramAlternativeKind,
    compile as compile_internal,
};
use crate::policy::NumericalPolicyPreset;
use crate::program::KernelProgram;
use crate::request::{
    CompilationRequest, CompilerCapabilitySnapshot, LoweringProviderIdentity,
    NumericalContractPreference, RequestError, StrictF32NumericalContract,
};
use tiler_ir::index::FrozenScalarRegistry;

/// Which boundary refused a compilation.
///
/// ADR 0074 convention 1: a typed enumeration rather than a boxed error, so a
/// caller branches on the boundary that refused instead of matching on text.
/// The classes are the compiler's own and are deliberately coarse — a caller
/// that needs the exact internal cause reads [`CompileFailure::explain`], which
/// is where causes are already typed and attributed.
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
    /// This is a hard target rejection, never an exhausted analysis budget.
    NoFeasiblePlan,
    /// A deterministic search or proof budget stopped the compilation.
    BudgetExhausted,
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

/// One target profile's compilation result.
#[derive(Clone, Debug)]
pub struct Compilation {
    stated_contracts: Vec<StrictF32NumericalContract>,
    resolved_contract: StrictF32NumericalContract,
    target_profile_key: &'static str,
    target_profile_descriptor: Vec<u8>,
    feasibility_rule_set: FeasibilityRuleSetIdentity,
    alternatives: Vec<ProgramAlternative>,
    selected_alternative_id: String,
    explain: VerifiedExplainTrace,
}

impl Compilation {
    /// The governed keys of the contracts this compilation was told to accept.
    ///
    /// Keys rather than the public [`NumericalContract`] enum, and deliberately:
    /// mapping a resolved contract back onto that enum needs an inverse of
    /// `resolve`, and the only total spelling of it absorbs an unrecognized key
    /// into one of the two variants — a silently wrong answer about which
    /// numerics a program was compiled under. ADR 0076 makes the key a
    /// contract's governed name, so it is the value that identifies one.
    ///
    /// In the caller's stated order, which is the order bound into the request
    /// subject. The first entry is not necessarily the one that was used — read
    /// [`Self::resolved_numerical_contract_key`] for that. Exposing both is the
    /// point: "what I would have accepted" and "what I got" are different facts,
    /// and a reader seeing only the second cannot tell a compilation that got
    /// its first choice from one that fell back.
    #[must_use]
    pub fn stated_numerical_contract_keys(&self) -> Vec<&'static str> {
        self.stated_contracts
            .iter()
            .map(|contract| contract.key)
            .collect()
    }

    /// The numerical contract this compilation actually resolved to.
    #[must_use]
    pub const fn resolved_numerical_contract_key(&self) -> &'static str {
        self.resolved_contract.key
    }

    /// Returns the governed key of the target profile this result is for.
    #[must_use]
    pub fn target_profile_key(&self) -> &str {
        self.target_profile_key
    }

    /// Returns the canonical descriptor bytes of the profile this compilation
    /// was assessed against.
    ///
    /// ADR 0043 requires a declared target profile to carry both its governed
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
        &self.target_profile_descriptor
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
        self.alternatives.iter().map(PlanAlternative)
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
            .map(PlanAlternative)
    }

    /// Returns the compilation's typed explain trace.
    #[must_use]
    pub fn explain(&self) -> ExplainReport<'_> {
        ExplainReport(&self.explain)
    }
}

/// A read view over one retained plan alternative.
///
/// A borrowed view rather than an owned record, so this boundary commits to no
/// public field set while the compiler's internal plan representation is still
/// moving.
#[derive(Clone, Copy, Debug)]
pub struct PlanAlternative<'a>(&'a ProgramAlternative);

impl PlanAlternative<'_> {
    /// Returns the alternative's stable identifier within its portfolio.
    #[must_use]
    pub fn stable_id(&self) -> &str {
        &self.0.stable_id
    }

    /// Returns whether one region covers the whole program.
    #[must_use]
    pub fn is_fused(&self) -> bool {
        self.0.kind == ProgramAlternativeKind::Fused
    }

    /// Returns the verified kernels this alternative dispatches.
    ///
    /// This is what a backend emits from. The kernels are already verified by
    /// `tiler-ir`'s own authority, so handing them out commits this boundary to
    /// no new guarantee of its own.
    #[must_use]
    pub fn kernels(&self) -> &[VerifiedKernel] {
        &self.0.kernels
    }

    /// Returns the lowering capabilities this alternative resolved.
    ///
    /// An artifact records which capabilities actually lowered its program;
    /// ADR 0072 folds them into complete program identity, so this is evidence
    /// rather than description.
    pub fn selected_capabilities(&self) -> impl ExactSizeIterator<Item = SelectedCapability<'_>> {
        self.0
            .artifact_plan
            .lowering_providers()
            .iter()
            .map(SelectedCapability)
    }

    /// Returns this alternative's ABI construction inputs.
    ///
    /// This is what an artifact assembler needs beyond the kernels: the guard,
    /// the accessible byte ranges, and the launch geometry, as expressions
    /// rather than as scalars.
    #[must_use]
    pub fn abi(&self) -> AbiConstruction<'_> {
        AbiConstruction(self.0.artifact_plan.verified_program())
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
/// # Why the assembler still has to replay them
///
/// `tiler-artifact`'s builder owns its own arena and mints owner-bound handles
/// into it, so these nodes are transliterated onto that arena rather than moved.
/// That replay is mechanical and position-preserving — it introduces no second
/// derivation, because the *decision* about what each expression says was made
/// upstream of here.
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
        CompileError::BudgetExhausted(_) => CompileFailureClass::BudgetExhausted,
        CompileError::NoFeasiblePlan(_) => CompileFailureClass::NoFeasiblePlan,
        CompileError::InvalidCompilerOutput(_) => CompileFailureClass::InvalidCompilerOutput,
    }
}

/// Returns the stable diagnostic key of one request refusal.
const fn rule_of(error: &RequestError) -> &'static str {
    match error {
        RequestError::UnsupportedRequestVersion => "compile.request.schema",
        RequestError::EmptyTargetSet => "compile.request.targets.empty",
        RequestError::DuplicateTargetProfile => "compile.request.targets.duplicate",
        RequestError::UnverifiedTargetSelection => "compile.request.targets.selection",
        RequestError::UnstatedNumericalContract => "compile.request.numerics.unstated",
        RequestError::NoResolvableNumericalContract { .. } => {
            "compile.request.numerics.unhonourable"
        }
        RequestError::UnrepresentableNumericalDimension { .. } => {
            "compile.request.numerics.unrepresentable"
        }
        RequestError::BudgetExceeded { resource, .. } => resource,
        RequestError::UnsupportedCapability { rule, .. } => rule,
        RequestError::ShapeProductOverflow { role } => role,
    }
}

/// A named numerical policy preset this build registers.
///
/// Stating one is **required**, not defaulted. These are different *contracts*
/// rather than one contract at three strictness settings: each carries its own
/// versioned key, so the same program under each has different canonical
/// identities, artifacts, and cache entries. The choice belongs to the caller
/// because it decides what the program *means*, and no authority below may
/// narrow, weaken, or substitute it to make a target feasible.
///
/// **Naming a laxer preset is not a way to make a strict program compile.** It
/// states a different program, which feasibility then assesses on its own terms;
/// an unhonourable request is a typed rejection naming the dimension, the
/// arithmetic type, the required behaviour, the behaviour the target declares,
/// and the declaring profile — never a downgrade and never a cost.
///
/// Every preset resolves one arithmetic type, `f32`, and says so in its key.
/// Subnormal behaviour is measurably per-dtype — one Apple row flushes in `f32`
/// and preserves in `f16` — so a preset that spoke for every width at once would
/// be stating something already known to be false.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NumericalContract {
    /// Every freedom refused; subnormals preserved on both dimensions.
    ///
    /// Not deliverable on any governed Apple family, whose `f32` arithmetic
    /// flushes subnormals in every math mode. A caller states it when it needs
    /// preservation and would rather not run than run wrong.
    StrictF32,
    /// Strict, except that both subnormal dimensions flush to the
    /// sign-preserving zero.
    ///
    /// This is what Apple hardware measurably delivers, so stating it makes
    /// running there a choice the caller made rather than a compromise made on
    /// its behalf. It widens exactly two dimensions: accepting flushing does not
    /// thereby accept reassociated sums.
    FlushSubnormalsToZeroF32,
    /// Subnormals preserved, and the reshaping freedoms this build can express
    /// authorized: fused-multiply-add contraction, reduction reassociation,
    /// reciprocal replacement of division, and approximate elementary functions
    /// within a named accuracy envelope.
    ///
    /// Operand permutation, signed-zero elimination, and assuming NaNs or
    /// infinities absent are deliberately *not* authorized. Each is a freedom an
    /// admitted operation could consume and none is carried by the region IR, so
    /// two programs differing only there would share one identity; stating one is
    /// refused by name rather than compiled ambiguously.
    RelaxedF32,
}

impl NumericalContract {
    /// Resolves this preset into the complete contract it names.
    ///
    /// Routed through the internal preset table rather than naming each
    /// constructor again, so the public spelling and the registered contract set
    /// cannot drift apart: a preset added to one and not the other fails to
    /// compile here.
    const fn resolve(self) -> StrictF32NumericalContract {
        self.preset().contract()
    }

    /// The internal preset this public name denotes.
    const fn preset(self) -> NumericalPolicyPreset {
        match self {
            Self::StrictF32 => NumericalPolicyPreset::Strict,
            Self::FlushSubnormalsToZeroF32 => NumericalPolicyPreset::FlushSubnormalsToZero,
            Self::RelaxedF32 => NumericalPolicyPreset::Relaxed,
        }
    }
}

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
/// pairing a registry with the scalar authority its capabilities were registered
/// against. The request boundary re-checks that pairing and refuses a mismatched
/// pair rather than resolving through an authority the capabilities were never
/// admitted under.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledCapabilities(CompilerCapabilitySnapshot);

impl InstalledCapabilities {
    /// The lowering capabilities this build ships.
    #[must_use]
    pub fn governed() -> Self {
        Self(CompilerCapabilitySnapshot::governed())
    }

    /// A caller's own registry, with the scalar authority it was frozen against.
    ///
    /// The two are taken together rather than separately because they are only
    /// meaningful as a pair: every resolved provider emits against, and is
    /// revalidated under, that scalar snapshot. Supplying a registry frozen over
    /// one authority and a different authority beside it is refused at the
    /// request boundary, not silently reconciled.
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
/// request carries. Budgets, the shape environment, and target-profile
/// *declaration* stay internal: the first two admit exactly one governed value
/// today, and declaring a target profile is a validation job rather than a
/// visibility change — `express-metal-honourability-in-the-shared-form` is the
/// ticket that needs it and it is not delivered here.
#[derive(Clone, Debug)]
pub struct CompileRequest<'a> {
    program: &'a SemanticProgram,
    contracts: Vec<NumericalContract>,
    capabilities: InstalledCapabilities,
}

impl<'a> CompileRequest<'a> {
    /// States the program to compile and the contract it means.
    ///
    /// Capabilities default to [`InstalledCapabilities::governed`], which is
    /// what makes [`compile_governed`] expressible through this same path.
    #[must_use]
    pub fn new(program: &'a SemanticProgram, contract: NumericalContract) -> Self {
        Self {
            program,
            contracts: vec![contract],
            capabilities: InstalledCapabilities::governed(),
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
    ) -> Result<Self, CompileFailure> {
        let contracts: Vec<NumericalContract> = contracts.into_iter().collect();
        if contracts.is_empty() {
            return Err(CompileFailure::from(CompileError::InvalidRequest(
                RequestError::UnstatedNumericalContract,
            )));
        }
        Ok(Self {
            program,
            contracts,
            capabilities: InstalledCapabilities::governed(),
        })
    }

    /// Installs the lowering authority this compilation resolves through.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: InstalledCapabilities) -> Self {
        self.capabilities = capabilities;
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
pub fn compile(request: CompileRequest<'_>) -> Result<Vec<Compilation>, CompileFailure> {
    let CompileRequest {
        program,
        contracts,
        capabilities,
    } = request;
    let stated: Vec<_> = contracts
        .iter()
        .map(|contract| contract.resolve())
        .collect();
    let preference = NumericalContractPreference::ordered(stated)
        .map_err(|error| CompileFailure::from(CompileError::InvalidRequest(error)))?;
    let mut internal = CompilationRequest::governed_preferring(program, preference);
    internal.capabilities = capabilities.0;
    let product = compile_internal(internal)?;
    Ok(into_compilations(product))
}

/// Compiles one semantic program under a stated numerical contract.
///
/// # Errors
///
/// Returns a [`CompileFailure`] naming the class of boundary that refused. An
/// unsupported program, an infeasible target, an exhausted budget, and invalid
/// compiler output are kept distinct: the first three are statements about the
/// request, and the last is a defect in Tiler.
/// It is the bounded convenience profile and not a second compile path: it
/// composes the same [`CompileRequest`] a caller would and calls the same
/// [`compile`]. One path rather than two is what stops the convenient one and
/// the general one from drifting, and expressing this wrapper through the
/// general surface is the cheapest proof that surface is usable at all.
pub fn compile_governed(
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<Vec<Compilation>, CompileFailure> {
    compile(CompileRequest::new(program, contract))
}

fn into_compilations(product: CompilationProduct) -> Vec<Compilation> {
    product
        .targets
        .into_iter()
        .map(|target| Compilation {
            stated_contracts: target.stated_contracts,
            resolved_contract: target.resolved_contract,
            target_profile_key: target.target_profile_key,
            target_profile_descriptor: target.target_profile_descriptor,
            feasibility_rule_set: target.feasibility_rule_set,
            selected_alternative_id: target.portfolio.selection.selected_alternative_id,
            alternatives: target.portfolio.alternatives,
            explain: target.explain,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        CompilationRequest, CompileFailure, CompileFailureClass, CompileRequest, NumericalContract,
        StrictF32NumericalContract, compile, compile_governed, compile_internal,
    };
    use tiler_ir::program::abi::ExprNode;
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

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

    /// The boundary compiles a program and hands out emittable kernels.
    ///
    /// This is the property the surface exists for: before it, no caller
    /// outside this crate could obtain a `VerifiedKernel` at all, so no backend
    /// could emit and nothing could execute.
    #[test]
    fn a_governed_program_compiles_to_alternatives_carrying_kernels() {
        let program = semantic_program();
        let compilations = compile_governed(&program, NumericalContract::StrictF32)
            .expect("the governed program compiles");
        assert_eq!(compilations.len(), 1);
        let compilation = &compilations[0];
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
                !plan.kernels().is_empty(),
                "{} dispatches at least one kernel",
                plan.stable_id(),
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

    /// The selected alternative is one of the retained ones.
    #[test]
    fn the_selection_names_a_retained_alternative() {
        let program = semantic_program();
        let compilations = compile_governed(&program, NumericalContract::StrictF32)
            .expect("the governed program compiles");
        let compilation = &compilations[0];
        let selected = compilation.selected().expect("a selected alternative");
        assert!(
            compilation
                .alternatives()
                .any(|plan| plan.stable_id() == selected.stable_id()),
        );
    }

    /// A successful compilation carries a renderable trace.
    #[test]
    fn a_compilation_renders_its_explain_trace() {
        let program = semantic_program();
        let compilations = compile_governed(&program, NumericalContract::StrictF32)
            .expect("the governed program compiles");
        let rendered = compilations[0].explain().render();
        assert!(
            rendered.starts_with("tiler-explain-v2 "),
            "the trace renders in its deterministic form",
        );
    }

    /// Which of ADR 0069's five classes the public surface can actually produce.
    ///
    /// **Reachability is a property worth pinning, not an assumption.** Two of
    /// the five are reached here from `compile_governed`; the other three are
    /// recorded below with the reason, because a class nothing can produce is a
    /// different claim from one that is merely untested.
    ///
    /// - `UnsupportedCapability` — reached by
    ///   `an_identity_program_is_refused_as_unsupported`.
    /// - `NoFeasiblePlan` — reached by the target-failure test above.
    /// - `BudgetExhausted` — reached only by a program that exceeds a
    ///   deterministic budget. `RequestError::BudgetExceeded` is its sole
    ///   source and the governed budgets admit every program this profile
    ///   compiles, so producing one means building a program specifically to
    ///   exceed them.
    /// - `InvalidCompilerOutput` — **unreachable by construction from a valid
    ///   call, deliberately.** It reports that Tiler's own verifier refused
    ///   Tiler's own output, so reaching it from the public surface would mean
    ///   shipping the defect it exists to report.
    /// - `InvalidRequest` — **unreachable from today's public surface**, and
    ///   this is the interesting one. Its five sources are all structural facts
    ///   about the request — an unsupported schema version, an empty or
    ///   duplicated target set, an unverified target selection, an unstated
    ///   contract — and `compile` builds that structure itself through
    ///   `CompilationRequest::governed_under`. A caller supplies a program, a
    ///   contract, and capabilities, none of which can produce any of them.
    ///   The class is still correct and still distinct: it becomes reachable
    ///   the moment a caller can declare its own target profiles, which is
    ///   `admit-a-caller-declared-target-profile`.
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

        let failure = compile_governed(&program, NumericalContract::StrictF32)
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
                NumericalContract::FlushSubnormalsToZeroF32,
                NumericalContract::StrictF32,
            ],
        )
        .expect("a non-empty preference is admitted");
        let compilations = compile(request).expect("the governed program compiles");
        let compilation = compilations.first().expect("one governed target");

        assert_eq!(
            compilation.stated_numerical_contract_keys().len(),
            2,
            "the whole stated list is retained, not only the winner",
        );
        assert_eq!(
            compilation.stated_numerical_contract_keys()[0],
            compilation.resolved_numerical_contract_key(),
            "the first acceptable contract is the one resolved on this profile",
        );

        // Order is the caller's, so the reversed list states a different
        // preference even though it names the same two contracts.
        let reversed = CompileRequest::preferring(
            &program,
            [
                NumericalContract::StrictF32,
                NumericalContract::FlushSubnormalsToZeroF32,
            ],
        )
        .expect("a non-empty preference is admitted");
        let reversed = compile(reversed).expect("the governed program compiles");
        assert_ne!(
            reversed[0].stated_numerical_contract_keys(),
            compilation.stated_numerical_contract_keys(),
            "a reordered preference is a different stated list",
        );

        // An empty list has no default and no implicit strictest reading.
        let empty = CompileRequest::preferring(&program, [])
            .expect_err("an empty preference states no contract");
        assert!(matches!(
            empty.class(),
            CompileFailureClass::InvalidRequest { .. }
        ));
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
            CompileFailureClass::BudgetExhausted,
            CompileFailureClass::InvalidCompilerOutput,
        ];
        for (index, left) in classes.iter().enumerate() {
            for right in &classes[index + 1..] {
                assert_ne!(left, right, "two failure classes compare equal");
            }
        }

        // The two that were one class carry the same rule key and are still
        // distinct, which is the whole point of splitting them: a caller must
        // not have to read the key to learn which action applies.
        assert_ne!(
            CompileFailureClass::InvalidRequest { rule: "same" },
            CompileFailureClass::UnsupportedCapability { rule: "same" },
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
    /// reaching a *post-request* failure needs a budget the governed profile
    /// does not expose. That is a statement about the bounded profile, not
    /// about the mapping, and the mapping is what this test covers.
    #[test]
    fn a_target_failure_carries_its_complete_trace() {
        let program = semantic_program();
        let mut request = CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed_flush_to_zero(),
        );
        // No per-seed growth leaves only singleton candidates, which the
        // bounded profile implements for no region, so every plan depends on a
        // region that was never formed.
        request.budgets.region_candidates_per_seed = 0;
        let failure = CompileFailure::from(
            compile_internal(request).expect_err("a zero region budget has no complete plan"),
        );

        assert_eq!(failure.class(), CompileFailureClass::NoFeasiblePlan);
        let rendered = failure
            .explain()
            .expect("a post-request failure retains its sealed trace")
            .render();
        assert!(rendered.starts_with("tiler-explain-v2 "));
        assert!(
            rendered.contains("compiler-failure"),
            "the trace names the terminal failure: {rendered}",
        );
        assert!(
            rendered.contains("region.formation.v1"),
            "the trace names the rule that refused: {rendered}",
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

        let failure = compile_governed(&program, NumericalContract::StrictF32)
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
        let compilations = compile_governed(&program, NumericalContract::StrictF32).unwrap();
        let compilation = compilations.first().expect("one governed target");

        // Both halves of the profile reference, neither invented by a consumer.
        assert!(!compilation.target_profile_key().is_empty());
        let descriptor = compilation.target_profile_descriptor();
        assert!(!descriptor.is_empty(), "a descriptor identity is carried");
        // Against the constant this crate publishes, not a literal restating it.
        // The bound is now enforced where a descriptor is minted, so this is a
        // regression guard on the governed profile's size rather than the only
        // thing standing between an oversized profile and a packaging failure.
        assert!(
            descriptor.len() <= crate::feasibility::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
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
        let compilations = compile_governed(&program, NumericalContract::StrictF32).unwrap();
        let compilation = compilations.first().expect("one governed target");
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
    }
}
