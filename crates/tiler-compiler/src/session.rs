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

use crate::explain::VerifiedExplainTrace;
use crate::feasibility::FeasibilityRuleSetIdentity;
use crate::pipeline::{
    CompilationProduct, CompileError, ProgramAlternative, ProgramAlternativeKind,
    compile as compile_internal,
};
use crate::program::KernelProgram;
use crate::request::{
    CompilationRequest, LoweringProviderIdentity, RequestError, StrictF32NumericalContract,
};

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
    /// The request or the program is outside the admitted profile.
    ///
    /// The program is not wrong; this build does not compile it.
    Unsupported {
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
    target_profile_key: &'static str,
    target_profile_descriptor: Vec<u8>,
    feasibility_rule_set: FeasibilityRuleSetIdentity,
    alternatives: Vec<ProgramAlternative>,
    selected_alternative_id: String,
    explain: VerifiedExplainTrace,
}

impl Compilation {
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
        // Both are statements about the request rather than about Tiler, and
        // both carry the refusing check's own key, so they classify the same
        // way; the internal distinction between a malformed request and an
        // unsupported capability is preserved in the explain trace.
        CompileError::InvalidRequest(cause) | CompileError::UnsupportedCapability(cause) => {
            CompileFailureClass::Unsupported {
                rule: rule_of(&cause),
            }
        }
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
        RequestError::BudgetExceeded { resource, .. } => resource,
        RequestError::UnsupportedCapability { rule, .. } => rule,
        RequestError::ShapeProductOverflow { role } => role,
    }
}

/// A numerical contract this build registers.
///
/// Stating one is **required**, not defaulted. These are two different
/// contracts rather than a strict setting and a relaxed one: each carries its
/// own versioned key, so the same program under each has different canonical
/// identities, artifacts, and cache entries. The choice belongs to the caller
/// because it decides what the program *means*, and no authority below may
/// narrow, weaken, or substitute it to make a target feasible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NumericalContract {
    /// Subnormals preserved on both dimensions; no contraction, no
    /// reassociation.
    ///
    /// Not deliverable on any governed Apple family, whose `f32` arithmetic
    /// flushes subnormals in every math mode. A caller states it when it needs
    /// preservation and would rather not run than run wrong.
    StrictF32,
    /// Subnormals flushed to the sign-preserving zero; no contraction, no
    /// reassociation.
    ///
    /// This is what Apple hardware measurably delivers, so stating it makes
    /// running there a choice the caller made rather than a compromise made on
    /// its behalf. It widens exactly one dimension: accepting flushing does not
    /// thereby accept reassociated sums.
    FlushSubnormalsToZeroF32,
}

impl NumericalContract {
    fn resolve(self) -> StrictF32NumericalContract {
        match self {
            Self::StrictF32 => StrictF32NumericalContract::governed(),
            Self::FlushSubnormalsToZeroF32 => StrictF32NumericalContract::governed_flush_to_zero(),
        }
    }
}

/// Compiles one semantic program under a stated numerical contract.
///
/// # Errors
///
/// Returns a [`CompileFailure`] naming the class of boundary that refused. An
/// unsupported program, an infeasible target, an exhausted budget, and invalid
/// compiler output are kept distinct: the first three are statements about the
/// request, and the last is a defect in Tiler.
pub fn compile_governed(
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<Vec<Compilation>, CompileFailure> {
    let product = compile_internal(CompilationRequest::governed_under(
        program,
        contract.resolve(),
    ))?;
    Ok(into_compilations(product))
}

fn into_compilations(product: CompilationProduct) -> Vec<Compilation> {
    product
        .targets
        .into_iter()
        .map(|target| Compilation {
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
        CompilationRequest, CompileFailure, CompileFailureClass, NumericalContract,
        StrictF32NumericalContract, compile_governed, compile_internal,
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

    /// The failure vocabulary keeps a Tiler defect distinct from a refused
    /// program, so a caller never reports one as the other.
    #[test]
    fn the_failure_classes_are_distinct() {
        assert_ne!(
            CompileFailureClass::NoFeasiblePlan,
            CompileFailureClass::InvalidCompilerOutput,
        );
        assert_ne!(
            CompileFailureClass::BudgetExhausted,
            CompileFailureClass::Unsupported { rule: "any" },
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
            matches!(failure.class(), CompileFailureClass::Unsupported { .. }),
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
        assert!(
            descriptor.len() <= 1_024,
            "the descriptor fits the artifact boundary's opaque-identity bound: {} bytes",
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
