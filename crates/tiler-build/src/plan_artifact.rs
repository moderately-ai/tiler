//! The backend-neutral seam between a checked compiler plan and one artifact.
//!
//! This is the build-time orchestration boundary [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 promotes: a backend supplies its payload and its entry declarations,
//! and everything a plan already decided is derived here rather than restated by
//! the producer. It names no backend, and this crate's Metal path is one caller
//! of it rather than its owner.
//!
//! # What is derived, and what is delegated
//!
//! The split is the contract, so it is written out rather than left to be
//! inferred from the code below. **Derived** from the owner-linked plan, and
//! therefore unforgeable by a producer: the target-profile reference and its
//! exact descriptor digest, the feasibility rule set and revision, the
//! compilation environment, every selected capability provider, every
//! compiler-minted deferred prepared-entry predicate, the executable entry
//! ordinal and its stage, and each entry's [`BackendEntryKey`] — which is the
//! stage kernel's own canonical identity and has no parameter here. The artifact
//! builder derives more still from the bound program: the applicability guard,
//! launch geometry, accessible offsets and extents, binding targets, element
//! types, address spaces, access modes, and alignment.
//!
//! **Delegated** to the backend, because no plan decides them: which payload the
//! entries resolve through, the transport category of each ABI binding, whether
//! a zero-thread launch skips its dispatch, and what must hold at launch time.
//! Those four are exactly what item 11 names moving into what a backend
//! supplies.
//!
//! # Why two closures rather than one declaration value
//!
//! [`assemble_plan_artifact`] calls `declare_entry` once per stage of the bound
//! program, in stage order. A producer therefore cannot declare more or fewer
//! entries than the plan has stages — the cardinality is structural rather than
//! checked, so there is no refusal to write and no path on which a surplus
//! declaration is silently dropped. Both closures take the builder, because a
//! payload and a launch precondition are both minted on it and a handle from
//! another builder is a typed refusal rather than a usable value.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    AbiExprId, ArtifactBuildError, ArtifactProgramBuilder, ArtifactVerificationError,
    BackendEntryKey, BackendEntryRef, BindingKind, BindingSpec, CapabilityKey,
    CompilationEnvironment, DeferredPredicateSpec, EntrySpec, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, LaunchSpec, PayloadId, SelectedProvider, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, VariantSpec, VerifiedArtifactProgram,
};
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::program::StageRef;
use tiler_ir::semantic::SemanticProgram;

use crate::realization::RealizationTranslationError;

/// What one backend declares for a single program stage's executable entry.
///
/// A caller-constructed leaf record with public fields, in the convention this
/// crate's callers already write. Every field is a statement no plan makes; a
/// field a plan *does* make is absent by design and derived instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendEntryDeclaration {
    /// Transport category of each ABI binding, in kernel buffer-parameter order.
    ///
    /// The count must equal the stage's binding count; a disagreement is the
    /// artifact builder's typed cardinality refusal rather than a truncation.
    pub bindings: Vec<BindingKind>,
    /// Whether a zero-thread launch skips the dispatch entirely.
    ///
    /// Declaring `false` on a stage whose launch is statically zero is a typed
    /// refusal, not a silent empty dispatch.
    pub zero_work_skips_dispatch: bool,
    /// Launch-instance preconditions, minted on the same builder.
    pub preconditions: Vec<AbiExprId>,
}

/// Why a checked plan did not assemble into one verified artifact.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlanArtifactError {
    /// The neutral artifact builder rejected one derived or declared statement.
    Build(ArtifactBuildError),
    /// Whole-artifact verification rejected the assembled program.
    Verification(ArtifactVerificationError),
    /// The plan's delivered-realization evidence did not translate.
    Realization(RealizationTranslationError),
}

impl fmt::Display for PlanArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(error) => write!(formatter, "artifact assembly failed: {error}"),
            Self::Verification(error) => write!(
                formatter,
                "whole-artifact verification failed: {:?}",
                error.diagnostics(),
            ),
            Self::Realization(error) => {
                write!(
                    formatter,
                    "delivered-realization translation failed: {error}"
                )
            }
        }
    }
}

impl Error for PlanArtifactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build(error) => Some(error),
            Self::Verification(error) => Some(error),
            Self::Realization(error) => Some(error),
        }
    }
}

impl From<ArtifactBuildError> for PlanArtifactError {
    fn from(error: ArtifactBuildError) -> Self {
        Self::Build(error)
    }
}

impl From<RealizationTranslationError> for PlanArtifactError {
    fn from(error: RealizationTranslationError) -> Self {
        Self::Realization(error)
    }
}

/// Assembles one verified artifact from a checked plan and a backend's payload.
///
/// The plan is owner-linked: every compilation-wide fact is read through
/// [`PlanAlternative::compilation`], so a caller cannot pair a plan from one
/// compilation with another compilation's target profile or provider
/// environment. `semantic` remains an explicit input because the compiler does
/// not retain the graph; the artifact builder verifies it against the plan's
/// target-neutral program before returning.
///
/// `declare_payload` runs once, before any entry, and receives the *derived*
/// target-profile reference — a backend states which payloads it carries, never
/// which target the artifact declares compatibility with. It returns them in
/// **delivery order**: one payload per delivery position, which is the ordered
/// slot a consumer's build target resolves to. One selection produces one
/// envelope carrying one payload per built family, so several families are
/// several entries in that run rather than several artifacts. The order is the
/// backend's statement and this function never reorders it; an empty run and a
/// run that disagrees with a sibling entry are the artifact builder's typed
/// refusals.
///
/// `declare_entry` runs once per stage in stage order and receives that stage,
/// so a backend reads the kernel, its accesses, and its launch geometry to
/// decide the four statements that are its own. It does **not** decide payloads:
/// every entry is realized by the same delivery-ordered run, because two
/// delivery positions are one plan compiled twice rather than two plans.
///
/// # This does not validate the payload's bytes, and cannot
///
/// A payload's `code` is opaque to every check performed here. Validating it
/// from bytes before the routing commit is the backend's own obligation under
/// [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
/// item 8, and no amount of assembly-side checking substitutes for it.
///
/// # Errors
///
/// Returns [`PlanArtifactError::Build`] for a rejected declaration — a foreign
/// handle, a binding-count disagreement with the stage, a launch precondition
/// reading a fact unavailable at launch preflight, a duplicate payload —
/// [`PlanArtifactError::Realization`] when the plan's own delivered-realization
/// evidence does not translate, and [`PlanArtifactError::Verification`] with
/// every whole-artifact diagnostic when the assembled program does not verify.
///
/// # Panics
///
/// Panics if the bound program holds more than [`u32::MAX`] stages. The
/// artifact model's own entry bound is four orders of magnitude below that, so
/// the conversion is infallible for any plan this function can package; it is an
/// assertion rather than a silent truncation because a truncated count would
/// leave a packaged entry with no policy-subject binding and the artifact would
/// then be refused for the wrong reason.
pub fn assemble_plan_artifact(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    declare_payload: impl FnOnce(
        &mut ArtifactProgramBuilder,
        TargetProfileRef,
    ) -> Result<Vec<PayloadId>, ArtifactBuildError>,
    mut declare_entry: impl FnMut(
        &mut ArtifactProgramBuilder,
        StageRef<'_>,
    ) -> Result<BackendEntryDeclaration, ArtifactBuildError>,
) -> Result<VerifiedArtifactProgram, PlanArtifactError> {
    let compilation = plan.compilation();
    let profile = target_profile(compilation)?;
    let rules = feasibility_rules(compilation)?;
    let environment =
        CompilationEnvironment::new(compilation.offered_lowering_providers().iter().cloned())?;
    let mut builder = ArtifactProgramBuilder::new(semantic, environment)?;

    for selected in plan.selected_capabilities() {
        builder.select_provider(SelectedProvider {
            provider: selected.provider().clone(),
            capability: CapabilityKey::new(selected.capability_key())?,
            capability_revision: selected.capability_revision(),
        })?;
    }

    let payload_ids = declare_payload(&mut builder, profile.clone())?;
    let program = plan.abi().kernel_program();
    let deferred_predicates = plan
        .prepared_entry_target_requirements()
        .map(|requirement| DeferredPredicateSpec {
            requirement: requirement.requirement().clone(),
            entry: requirement.entry(),
        })
        .collect();
    let mut entries = Vec::with_capacity(program.stages().len());
    for stage in program.stages() {
        let declared = declare_entry(&mut builder, stage)?;
        entries.push(EntrySpec {
            bindings: declared
                .bindings
                .into_iter()
                .map(|kind| BindingSpec { kind })
                .collect(),
            launch: LaunchSpec {
                zero_work_skips_dispatch: declared.zero_work_skips_dispatch,
                preconditions: declared.preconditions,
            },
            implementation: BackendEntryRef {
                // The same delivery-ordered payload run for every entry, so
                // position `p` names one object across the whole plan by
                // construction rather than by a check: a backend states which
                // objects it built, in which order, once.
                payloads: payload_ids.clone(),
                // The stage kernel's own canonical identity, with no parameter a
                // producer could supply instead. A backend states which *symbol*
                // realizes this key in its payload's entry mapping, and a
                // mapping that omits the key is refused when the envelope is
                // decoded.
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )?,
            },
        });
    }

    let packaged_entries = u32::try_from(entries.len()).expect("a bounded entry table fits u32");
    builder.push_variant(
        program,
        VariantSpec {
            target_profile: profile.clone(),
            feasibility_rules: rules,
            deferred_predicates,
            entries,
        },
    )?;
    // The delivered realization, transcribed from the plan's own compiler
    // evidence. Declared after the variant so the entry count it binds is the
    // one the artifact actually packaged, and derived rather than delegated
    // because no backend may state a numerical fact: the two `declare_*`
    // closures above receive no way to reach this.
    builder.declare_realization(crate::realization::translate(
        plan.delivered_realization(),
        &profile,
        packaged_entries,
    )?)?;
    builder.build().map_err(PlanArtifactError::Verification)
}

fn target_profile(compilation: &Compilation) -> Result<TargetProfileRef, ArtifactBuildError> {
    Ok(TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())?,
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )?,
    })
}

fn feasibility_rules(
    compilation: &Compilation,
) -> Result<FeasibilityRuleSetRef, ArtifactBuildError> {
    Ok(FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())?,
        revision: compilation.feasibility_rule_set_revision(),
    })
}
