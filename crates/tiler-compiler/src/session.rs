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
//! API. `prototype-public-compiler-api` carries seven deferred public-surface
//! questions — report completeness, trace serialization and embedding, renderer
//! and redaction guarantees, enum exhaustiveness, evidence-receipt minting,
//! identity spelling, and header stability. **None of them is answered here.**
//! Explain is exposed as an opaque handle with one rendering method, which is
//! the narrowest shape that answers none of those questions by default; a
//! caller can see a trace and cannot yet serialize, embed, redact, or key on
//! one. Answering them by omission is exactly what that ticket exists to
//! prevent.
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

use tiler_ir::kernel::VerifiedKernel;
use tiler_ir::semantic::SemanticProgram;

use crate::explain::VerifiedExplainTrace;
use crate::pipeline::{
    CompilationProduct, CompileError, ProgramAlternative, ProgramAlternativeKind,
    compile as compile_internal,
};
use crate::request::{CompilationRequest, RequestError};

/// Why a compilation did not produce plans.
///
/// ADR 0074 convention 1: a typed enumeration rather than a boxed error, so a
/// caller branches on the boundary that refused instead of matching on text.
/// The classes are the compiler's own and are deliberately coarse at this
/// boundary — a caller that needs the exact internal cause reads the explain
/// trace, which is where causes are already typed and attributed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompileFailure {
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

/// One target profile's compilation result.
#[derive(Clone, Debug)]
pub struct Compilation {
    target_profile_key: &'static str,
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
}

/// An opaque handle to one compilation's explain trace.
///
/// Deliberately minimal. Rendering is the only capability exposed, because
/// every richer one — serializing, embedding in an artifact, redacting provider
/// detail, keying a cache on trace identity — is a deferred question owned by
/// `prototype-public-compiler-api`, and offering it here would answer that
/// question by default.
#[derive(Clone, Copy, Debug)]
pub struct ExplainReport<'a>(&'a VerifiedExplainTrace);

impl ExplainReport<'_> {
    /// Renders the trace in its deterministic text form.
    ///
    /// The rendered form is **not** a stable contract at this boundary; it is a
    /// diagnostic for a human reader. Do not parse it.
    #[must_use]
    pub fn render(&self) -> String {
        self.0.render()
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
impl From<CompileError> for CompileFailure {
    fn from(error: CompileError) -> Self {
        match error {
            // An explained failure wraps its own cause; the trace is not
            // exposed on the error path, because whether a failed compilation
            // returns a partial report is one of the deferred questions this
            // boundary must not answer by default.
            CompileError::Explained { source, .. } => Self::from(*source),
            // Both are statements about the request rather than about Tiler,
            // and both carry the refusing check's own key, so they classify the
            // same way; the internal distinction between a malformed request
            // and an unsupported capability is preserved in the explain trace.
            CompileError::InvalidRequest(cause) | CompileError::UnsupportedCapability(cause) => {
                Self::Unsupported {
                    rule: rule_of(&cause),
                }
            }
            CompileError::BudgetExhausted(_) => Self::BudgetExhausted,
            CompileError::NoFeasiblePlan(_) => Self::NoFeasiblePlan,
            CompileError::InvalidCompilerOutput(_) => Self::InvalidCompilerOutput,
        }
    }
}

/// Returns the stable diagnostic key of one request refusal.
const fn rule_of(error: &RequestError) -> &'static str {
    match error {
        RequestError::UnsupportedRequestVersion => "compile.request.schema",
        RequestError::EmptyTargetSet => "compile.request.targets.empty",
        RequestError::DuplicateTargetProfile => "compile.request.targets.duplicate",
        RequestError::UnverifiedTargetSelection => "compile.request.targets.selection",
        RequestError::BudgetExceeded { resource, .. } => resource,
        RequestError::UnsupportedCapability { rule, .. } => rule,
        RequestError::ShapeProductOverflow { role } => role,
    }
}

/// Compiles one semantic program under the governed strict contract.
///
/// # Errors
///
/// Returns a [`CompileFailure`] naming the class of boundary that refused. An
/// unsupported program, an infeasible target, an exhausted budget, and invalid
/// compiler output are kept distinct: the first three are statements about the
/// request, and the last is a defect in Tiler.
pub fn compile_governed(program: &SemanticProgram) -> Result<Vec<Compilation>, CompileFailure> {
    let product = compile_internal(CompilationRequest::governed(program))?;
    Ok(into_compilations(product))
}

fn into_compilations(product: CompilationProduct) -> Vec<Compilation> {
    product
        .targets
        .into_iter()
        .map(|target| Compilation {
            target_profile_key: target.target_profile_key,
            selected_alternative_id: target.portfolio.selection.selected_alternative_id,
            alternatives: target.portfolio.alternatives,
            explain: target.explain,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CompileFailure, compile_governed};
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
        let compilations = compile_governed(&program).expect("the governed program compiles");
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
        let compilations = compile_governed(&program).expect("the governed program compiles");
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
        let compilations = compile_governed(&program).expect("the governed program compiles");
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
            CompileFailure::NoFeasiblePlan,
            CompileFailure::InvalidCompilerOutput,
        );
        assert_ne!(
            CompileFailure::BudgetExhausted,
            CompileFailure::Unsupported { rule: "any" },
        );
    }
}
