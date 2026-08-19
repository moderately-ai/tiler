#![allow(
    clippy::wildcard_imports,
    reason = "this module is one half of `pipeline`, not a separate concept: it is a \
private child that exists so the root reads as the compilation story, and every name it \
uses is defined in that root. Enumerating them would restate fifty parent items and would \
have to be restated again on every change, for no reader benefit -- the glob is scoped to \
one parent whose contents are visible in the same directory"
)]

//! Re-derivation of the retained portfolio from the program and its contents.
//!
//! These functions rebuild what the planning phase produced and require the
//! rebuild to reproduce the receipt exactly, so a tampered plan, cost,
//! program, or artifact receipt fails closed instead of being carried into a
//! compilation product.
//!
//! **Nothing here may reuse a planning intermediate.** The independence is
//! the mechanism: a verifier handed the value it is checking compares that
//! value to itself and can never say no. This is the single largest cost in
//! a compile -- a sampling profile put it at 23% of active self time -- and
//! it is deliberate rather than duplicated work.

use super::*;

/// Re-derives the retained portfolio from the program and its own contents.
///
/// The complete-plan authority re-verifies every plan's cover and re-assembles
/// each plan from its selections; this additionally re-derives each alternative's
/// KIR, kernel program, and artifact plan and requires them to reproduce the
/// receipt exactly. A tampered plan, cost, program, or artifact receipt therefore
/// fails closed instead of being carried into a compilation product.
pub(super) fn verify_portfolio(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    formation: &crate::region::RegionFormationOutcome,
    portfolio: &SelectedPortfolio,
    alternatives: &[ProgramAlternative],
    selected_id: &str,
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    if request.semantic_identity() != semantic.semantic_identity() {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-semantic-request-binding",
            }
            .into(),
            ExplainStage::RequestVerification,
            cause,
        ));
    }
    // The same legality contract planning enumerated under, re-stated from the
    // request's own resolved contract rather than carried over from planning: a
    // verifier handed the policy the planner used could not refuse a cover that
    // was legal only under a weaker one.
    verify_selected_portfolio(
        semantic,
        formation,
        crate::cover::CoverPolicy::governed(request.numerical_contract()),
        portfolio,
    )
    .map_err(|source| failure_at_source(source.into(), ExplainStage::Selection, cause))?;
    if alternatives.is_empty()
        || alternatives
            .iter()
            .map(|alternative| alternative.stable_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != alternatives.len()
    {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-identity",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    }
    // This is verifier-owned evidence, independently re-derived from the
    // request rather than borrowed from planning. One portfolio has one
    // semantic program and request, so reusing it across alternatives keeps
    // verification independent without multiplying the same proof work.
    let lowering = resolve_lowering(semantic, request).map_err(|_| {
        failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-refinement-resolution",
            }
            .into(),
            ExplainStage::CapabilityResolution,
            cause,
        )
    })?;
    for alternative in alternatives {
        verify_alternative(semantic, request, formation, alternative, &lowering, cause)?;
    }
    // The profile comes from the request rather than from an alternative's own
    // scheduled regions, which each carry a copy: a verifier handed the value the
    // candidate carries compares that value to itself and can never say no.
    let recomputed = select_non_dominated(portfolio, alternatives, request.target_profile())
        .map_err(|source| failure_at_source(source, ExplainStage::Selection, cause))?;
    if selected_id != recomputed
        || !alternatives
            .iter()
            .any(|item| item.stable_id == selected_id)
    {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-selection",
            }
            .into(),
            ExplainStage::Selection,
            cause,
        ));
    }
    Ok(())
}

/// Re-derives one alternative's structured, program, and artifact layers.
pub(super) fn verify_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    formation: &crate::region::RegionFormationOutcome,
    alternative: &ProgramAlternative,
    lowering: &ResolvedLowering,
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    if alternative.semantic.semantic_identity() != semantic.semantic_identity() {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-retained-semantic-binding",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    }
    if alternative.stable_id != alternative.identity.label()
        || alternative.structural_cost != alternative.plan.cost()
        || alternative.kind
            != ProgramAlternativeKind::of(
                alternative.plan.cover(),
                total_members(&alternative.plan),
            )
    {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-cost-or-identity",
            }
            .into(),
            ExplainStage::Costing,
            cause,
        ));
    }
    // The whole assembly description is re-derived from the alternative's own
    // plan and its stage order compared against the stored regions. A refusal
    // means the plan describes a program this assembler has no expression for,
    // which the alternative could not have been built from — so it is a binding
    // failure like any other mismatch, not a separate outcome.
    let Ok(assembly) = CoverAssembly::from_plan(semantic, &alternative.plan) else {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-schedule-binding",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    };
    if alternative.scheduled_regions.as_slice() != assembly.regions() {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-schedule-binding",
            }
            .into(),
            ExplainStage::IntrinsicScheduling,
            cause,
        ));
    }
    let scheduled = alternative.scheduled_regions.as_slice();
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, cause)
        })?;
    if alternative.kernels != kernels {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-kernel-binding",
            }
            .into(),
            ExplainStage::KernelRefinement,
            cause,
        ));
    }
    let program = build_plan_program(semantic, request, &assembly, lowering)
        .map_err(|error| failure_at_source(error, ExplainStage::ProgramVerification, cause))?;
    if alternative.program != program {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-program-binding",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    }
    // The plan's own recorded provenance is checked against the request's
    // installed registry rather than against itself, so a receipt naming a
    // provider the registry never resolved fails closed here.
    verify_artifact_plan_with_lowering(
        &alternative.artifact_plan,
        semantic,
        request,
        &assembly,
        &kernels,
        &program,
        lowering,
    )
    .map_err(|error| failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause))?;
    verify_equivalence(semantic, request, formation, alternative)
        .map_err(|source| failure_at_source(source, ExplainStage::NumericalLegality, cause))
}

/// Returns the number of semantic occurrences a plan's cover covers.
pub(super) fn total_members(plan: &SelectedPlan) -> u32 {
    u32::try_from(
        plan.cover()
            .regions()
            .iter()
            .map(|region| region.members().len())
            .sum::<usize>(),
    )
    .unwrap_or(u32::MAX)
}

/// Replays every retained numerical-equivalence and fusion-legality proof.
pub(super) fn verify_equivalence(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    formation: &crate::region::RegionFormationOutcome,
    alternative: &ProgramAlternative,
) -> Result<(), CompileError> {
    let capabilities = FusionNumericalCapabilities::governed();
    // Every multi-occurrence region must carry exactly one replayable legality
    // proof; a fused region without one would be an unproven fusion.
    let expected: Vec<usize> = alternative
        .plan
        .cover()
        .regions()
        .iter()
        .enumerate()
        .filter_map(|(position, region)| (region.members().len() > 1).then_some(position))
        .collect();
    if alternative
        .equivalence
        .legality
        .iter()
        .map(|(position, _)| *position)
        .collect::<Vec<_>>()
        != expected
    {
        return Err(ProgramError::Structure {
            rule: "portfolio-equivalence",
        }
        .into());
    }
    for (position, proof) in &alternative.equivalence.legality {
        let region =
            alternative
                .plan
                .cover()
                .regions()
                .get(*position)
                .ok_or(ProgramError::Structure {
                    rule: "portfolio-equivalence",
                })?;
        let candidate = formation
            .candidates()
            .iter()
            .find(|candidate| candidate.occurrence() == region.occurrence())
            .ok_or(ProgramError::Structure {
                rule: "portfolio-equivalence",
            })?;
        verify_fusion_legality(
            semantic,
            request.budgets(),
            request.numerical_contract(),
            &capabilities,
            formation,
            candidate,
            proof,
        )?;
    }
    match (
        alternative.kind,
        alternative.equivalence.numerical.as_deref(),
    ) {
        (ProgramAlternativeKind::Materialized, None) => Ok(()),
        // A whole-program strategy that carries no fused-numerics proof, because
        // there is no fusion to prove: its single region realizes one semantic
        // occurrence family directly rather than merging an elementwise prologue
        // into a reduction. The coverage check below is what stands in its place,
        // and it is not weaker: it requires the one scheduled region to cover
        // exactly the candidate's occurrences.
        //
        // **The condition is the declared-input contributor, stated as the fact
        // it means rather than inferred from two absences.** The exemption used
        // to read `try_serial_sum().is_none_or(|serial| serial.prologue.is_none())`,
        // and both halves were absences: a non-serial-sum output was exempt
        // through `is_none_or`'s vacuous truth, and a fold was exempt whenever
        // its `prologue` field was empty — which a fold whose contributors
        // another region *materializes* also is. Under the contributor source
        // that spelling would have exempted a produced sum from the numerical
        // replay while claiming to be about a fold that merges nothing.
        //
        // What the arm actually needs is that the whole-program region merges
        // nothing. `sum(x)` satisfies it: its region is the plain
        // `ScalarProgram::StrictSerialSum` folding a declared input, there is no
        // affine pair for a proof to be about, and `fused_prologue_constants`
        // answers `None` for it. A fold with a prologue does not — its
        // whole-program region really is a fusion — and neither does a fold over
        // a materialized producer, whose partition spans a producer region, an
        // optional continuation, and the fold. Both fall through to the proving
        // arm below rather than being re-derived a second way.
        //
        // A non-serial-sum output is exempted by a stated per-family rule rather
        // than by `is_none_or`: the pointwise and contraction arms publish from
        // one region that merges nothing, while a chain and a staged family are
        // several regions by construction and cannot classify `Fused` at all —
        // so naming them refuses rather than vacuously admits.
        // [`match-the-declared-input-contributor-in-the-fused-proof-exemption`](../../../tickets/match-the-declared-input-contributor-in-the-fused-proof-exemption.md)
        // owns this statement.
        (ProgramAlternativeKind::Fused, None)
            if request.normalized().outputs().iter().all(merges_nothing) =>
        {
            let candidate = formation.whole_program_candidate().ok_or({
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "portfolio-fused-region",
                    },
                ))
            })?;
            verify_whole_program_schedule_coverage(alternative, candidate)
        }
        (ProgramAlternativeKind::Fused, Some(proof)) => {
            let candidate = formation.whole_program_candidate().ok_or({
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "portfolio-fused-region",
                    },
                ))
            })?;
            verify_fused_numerics(formation.graph(), request, candidate, proof)?;
            verify_whole_program_schedule_coverage(alternative, candidate)
        }
        _ => Err(ProgramError::Structure {
            rule: "portfolio-equivalence",
        }
        .into()),
    }
}

/// Returns whether one recognized output's whole-program region would merge no
/// occurrences, and therefore has no fusion for a numerical proof to be about.
///
/// **Exhaustive over the output vocabulary, and over the fold's contributor
/// source within it, so a widening is a build error here rather than a silently
/// widened exemption.** Answering `false` costs a compilation nothing: the
/// alternative takes the ordinary `portfolio-equivalence` proof path, which is
/// the fail-closed direction and the one the exemption exists to be narrower
/// than.
///
/// - A pointwise output and a contraction publish from one region computing one
///   recognized family; neither merges anything.
/// - A fold merges nothing exactly when it reads a declared input directly. A
///   pointwise prologue is a merge — that is what the fused scalar program
///   *is* — and a materialized contributor is a partition of several regions,
///   whose whole-program classification would be a claim about a cover this
///   compiler does not assemble.
/// - A chain and a staged family are several regions by construction, so a
///   `Fused` receipt over one is forged; refusing here is what keeps the forged
///   receipt on the replaying path instead of the exempt one.
fn merges_nothing(output: &crate::request::NormalizedOutput) -> bool {
    use crate::request::{NormalizedOutput, SerialSumContributor};

    match output {
        NormalizedOutput::Pointwise(_) | NormalizedOutput::Contraction(_) => true,
        NormalizedOutput::SerialSum(serial) => match &serial.contributor {
            SerialSumContributor::DeclaredInput(_) => true,
            SerialSumContributor::PointwisePrologue { .. }
            | SerialSumContributor::Materialized(_) => false,
        },
        NormalizedOutput::Epilogue(_) | NormalizedOutput::Staged(_) => false,
    }
}

/// Requires a whole-program plan's dispatches to cover its candidate exactly.
///
/// **Stated over the dispatches together rather than over one of them**, because
/// a cover placing every operation in one region does not fix how many dispatches
/// realize it. This used to require exactly one scheduled region, which was
/// equivalent while every whole-program cover region was realized by a single
/// kernel: a split reduction always covered two occurrences — a prologue and its
/// fold — so its cover had two regions and the plan was classified `Materialized`
/// before reaching here. `sum(x)` has one occurrence, so a two-dispatch split of it
/// is a *whole-program* cover realized by a subprogram, and the old shape check
/// rejected that as malformed compiler output.
///
/// What the obligation always was survives intact: the dispatches' claims
/// *partition* the candidate's occurrences. It is decided by
/// [`crate::region::chain_realizes_subject`], which is the one place the rule is
/// written — the physical frontier owes the identical obligation for a
/// subprogram it admits, and two spellings of one rule are two rules to keep in
/// agreement. A dispatch claiming an occurrence twice, one claiming an
/// occurrence outside the candidate, one continuing a stage nothing computed,
/// and a plan leaving an occurrence unclaimed all fail it.
fn verify_whole_program_schedule_coverage(
    alternative: &ProgramAlternative,
    candidate: &crate::region::RegionCandidate,
) -> Result<(), CompileError> {
    let mut claimed: Vec<crate::region::SemanticStage> = alternative
        .scheduled_regions
        .iter()
        .flat_map(|region| region.semantic_members().iter().copied())
        .collect();
    if !crate::region::chain_realizes_subject(&mut claimed, candidate.members()) {
        return Err(ProgramError::Structure {
            rule: "portfolio-candidate-schedule-binding",
        }
        .into());
    }
    Ok(())
}
