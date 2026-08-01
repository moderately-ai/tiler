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
    verify_selected_portfolio(semantic, formation, portfolio)
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
    for alternative in alternatives {
        verify_alternative(semantic, request, formation, alternative, cause)?;
    }
    let recomputed = select_non_dominated(portfolio, alternatives)
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
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
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
    // The schedule is re-derived and compared exactly as before; only the copy
    // is gone. Once the stored regions are proven equal to the re-derivation,
    // they *are* the re-derivation, so the layers below verify against the
    // borrowed slice instead of against a duplicate of it.
    // `None` means the plan contains a body with no scheduled region, which the
    // alternative could not have been built from — so it is a binding failure
    // like any other mismatch, not a separate outcome.
    let Some(ordered) = plan_region_order(&alternative.plan) else {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-schedule-binding",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    };
    if alternative.scheduled_regions.len() != ordered.len()
        || alternative
            .scheduled_regions
            .iter()
            .zip(&ordered)
            .any(|(stored, derived)| stored != *derived)
    {
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
    let program = build_plan_program(semantic, request, alternative.kind, scheduled)
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
    let providers = crate::lowering::resolve_capabilities(semantic, request).map_err(|_| {
        failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-provider-resolution",
            }
            .into(),
            ExplainStage::CapabilityResolution,
            cause,
        )
    })?;
    verify_artifact_plan(
        &alternative.artifact_plan,
        semantic,
        request,
        scheduled,
        &kernels,
        &program,
        providers,
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
        // into a reduction. A reduced-elementwise request never reaches this arm
        // — its only whole-program region is the fused one, which exists exactly
        // when the affine equivalence proof does. The coverage check below is
        // what stands in its place, and it is not weaker: it requires the one
        // scheduled region to cover exactly the candidate's occurrences.
        (ProgramAlternativeKind::Fused, None)
            if request.pointwise().is_some() || request.contraction().is_some() =>
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

fn verify_whole_program_schedule_coverage(
    alternative: &ProgramAlternative,
    candidate: &crate::region::RegionCandidate,
) -> Result<(), CompileError> {
    if alternative.scheduled_regions.len() != 1
        || alternative.scheduled_regions[0].semantic_members() != candidate.members()
    {
        return Err(ProgramError::Structure {
            rule: "portfolio-candidate-schedule-binding",
        }
        .into());
    }
    Ok(())
}
