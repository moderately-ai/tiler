#![allow(
    clippy::wildcard_imports,
    reason = "this module is one half of `pipeline`, not a separate concept: it is a \
private child that exists so the root reads as the compilation story, and every name it \
uses is defined in that root. Enumerating them would restate fifty parent items and would \
have to be restated again on every change, for no reader benefit -- the glob is scoped to \
one parent whose contents are visible in the same directory"
)]

//! Transactional planning and alternative construction.
//!
//! Enumerating complete plans, building one program alternative from a
//! selected plan, and reducing a portfolio to its non-dominated set. The
//! boundary is the *transaction*: nothing here is observable until
//! `compile_target_with_explain` accepts the portfolio it returns.

use super::*;

/// Everything the complete-plan authorities produced for one target.
pub(super) struct CompletePlans {
    /// Every recognized occurrence's resolved capability and refinement evidence.
    pub(super) lowering: ResolvedLowering,
    pub(super) portfolio: SelectedPortfolio,
    /// One replayable fusion-legality proof per multi-occurrence region, keyed by
    /// the region occurrence it was derived for.
    pub(super) legality: std::collections::BTreeMap<
        crate::region::RegionOccurrenceIdentity,
        Box<FusionLegalityProof>,
    >,
    /// The whole-program strict-`f32` numerical equivalence proof, when a
    /// whole-program candidate exists and its fusion is legal.
    pub(super) numerical: Option<Box<FusionNumericalProof>>,
    /// Region subjects the frontier rejected as hard-infeasible on this target.
    pub(super) rejections: TargetRejections,
    /// The complete-plan selection record every alternative is caused by.
    pub(super) selection_record: ExplainRecordId,
}

/// Enumerates legal covers, proves their fusion legality, enumerates the local
/// implementation frontier of every cover region, and joins them into complete
/// physical plans.
///
/// The three authorities stay separate exactly as their contracts require:
/// [`enumerate_covers`] answers a strictly global legality question and chooses
/// no implementation; [`derive_fusion_legality`] decides whether a
/// multi-occurrence region may be realized as one fused region at all;
/// [`enumerate_frontier`] answers a strictly local feasibility question for one
/// region and target; and only [`select_physical_plans`] joins them.
#[allow(
    clippy::too_many_lines,
    reason = "keeps the cover, legality, frontier, and join stages and their phase-local failure contexts in one readable transaction"
)]
pub(super) fn enumerate_complete_plans(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    formation: &RegionFormationOutcome,
    explain: &mut ExplainWriter,
    root: ExplainRecordId,
    whole_program_record: Option<ExplainRecordId>,
) -> Result<CompletePlans, TargetFailure> {
    let budgets = verified.budgets();
    let contract = verified.numerical_contract();
    // Lowering-capability resolution precedes every cover: a cover is a claim
    // about how recognized occurrences are grouped, and grouping occurrences the
    // installed authority cannot lower at all would be enumerating plans nothing
    // could realize.
    let lowering = match resolve_lowering(semantic, verified, formation) {
        Ok(lowering) => lowering,
        Err(source) => {
            let cause = record_lowering_failure(explain, &source, root)?;
            return Err(lowering_failure(&source, cause));
        }
    };
    let lowering_record = record_lowering(explain, &lowering, root)?;
    let enumeration = enumerate_covers(semantic, budgets, formation).map_err(|source| {
        failure_at_source(
            source.into(),
            ExplainStage::CandidateEnumeration,
            record_cause(lowering_record),
        )
    })?;
    let cover_record = record_cover_enumeration(explain, &enumeration, lowering_record)?;

    let capabilities = FusionNumericalCapabilities::governed();
    let mut legality = std::collections::BTreeMap::new();
    let mut illegal = std::collections::BTreeSet::new();
    let mut legality_cause = cover_record;
    for cover in enumeration.covers() {
        for region in cover.regions() {
            if region.members().len() < 2
                || legality.contains_key(region.occurrence())
                || illegal.contains(region.occurrence())
            {
                continue;
            }
            let candidate = formation
                .candidates()
                .iter()
                .find(|candidate| candidate.occurrence() == region.occurrence())
                .ok_or_else(|| {
                    failure_at_source(
                        CompileError::InvalidCompilerOutput(CompilerOutputError::Cover(
                            CoverError::Structure {
                                rule: "cover-region-candidate",
                            },
                        )),
                        ExplainStage::CandidateEnumeration,
                        record_cause(cover_record),
                    )
                })?;
            let cause = if candidate.covers_whole_program() {
                whole_program_record.unwrap_or(legality_cause)
            } else {
                legality_cause
            };
            let outcome = derive_fusion_legality(
                semantic,
                budgets,
                contract,
                &capabilities,
                formation,
                candidate,
            )
            .map_err(|source| {
                failure_at_source(
                    source.into(),
                    ExplainStage::NumericalLegality,
                    record_cause(cover_record),
                )
            })?;
            legality_cause =
                record_fusion_legality(explain, &capabilities, candidate, &outcome, cause)?;
            match outcome {
                FusionLegality::Legal(proof) => {
                    legality.insert(region.occurrence().clone(), proof);
                }
                FusionLegality::Rejected(_) | FusionLegality::Unknown(_) => {
                    illegal.insert(region.occurrence().clone());
                }
            }
        }
    }

    // A whole-program candidate whose fusion is legal additionally carries the
    // strict-`f32` numerical equivalence proof the trace cites as a sound proof.
    let mut numerical = None;
    let mut numerical_cause = legality_cause;
    if let Some(candidate) = formation.whole_program_candidate()
        && !illegal.contains(candidate.occurrence())
    {
        let proof =
            prove_fused_numerics(formation.graph(), verified, candidate).map_err(|error| {
                failure_at_source(
                    error.into(),
                    ExplainStage::NumericalLegality,
                    record_cause(legality_cause),
                )
            })?;
        numerical_cause = record_numerical_equivalence(
            explain,
            verified,
            &lowering,
            candidate,
            &proof,
            legality_cause,
        )?;
        numerical = Some(Box::new(proof));
    }

    let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
    let mut sources = Vec::new();
    let mut rejections = TargetRejections::default();
    let mut frontier_cause = numerical_cause;
    let mut recorded_roles = std::collections::BTreeMap::new();
    // Covers every one of whose regions was proposed for, but at least one of
    // which the target refused. A reader expects those ruled out by feasibility
    // rather than by a missing capability, so each is noted in the terminal
    // ledger as an infeasible alternative.
    //
    // The identity is carried rather than its explain label because the label is
    // only wanted for the subset that survives the retention check below, while
    // the check itself wants exact bytes.
    let mut refused_covers: Vec<(&RegionCoverIdentity, TerminalCause)> = Vec::new();
    // One enumeration per distinct region subject, reused by every cover that
    // places that region.
    //
    // `enumerate_frontier` is a pure function of the request, the subject, and
    // the providers, and only the subject varies here — so a repeat is a
    // re-derivation of a value already in hand. It repeats a lot: the governed
    // five-operation program enumerates 48 times over 17 distinct subjects, and
    // the reduction region alone is enumerated 8 times because eight covers
    // place it.
    //
    // The key is the whole subject, not its role. Fourteen of those seventeen
    // subjects share the role `unrecognized` while covering different
    // occurrences, so a role-keyed memo would serve one region's frontier for
    // another's — the members are what the request-subject binding checks each
    // proposal against.
    //
    // A linear scan beats a map at this size and asks only for `PartialEq`,
    // which the subject already has.
    let mut frontiers_by_subject: Vec<(FrontierRegionSubject, ImplementationFrontier)> = Vec::new();
    for cover in enumeration.covers() {
        if cover
            .regions()
            .iter()
            .any(|region| illegal.contains(region.occurrence()))
        {
            continue;
        }
        let mut region_frontiers = Vec::with_capacity(cover.region_count());
        let mut proposed_everywhere = true;
        let mut refusal: Option<TerminalCause> = None;
        for region in cover.regions() {
            let role = region_role(verified, region.members());
            let subject = FrontierRegionSubject::new(role, region.members().to_vec());
            let frontier = if let Some((_, enumerated)) = frontiers_by_subject
                .iter()
                .find(|(seen, _)| *seen == subject)
            {
                enumerated.clone()
            } else {
                let enumerated =
                    enumerate_frontier(verified, &subject, &providers).map_err(|source| {
                        failure_at_source(
                            source.into(),
                            ExplainStage::IntrinsicScheduling,
                            record_cause(numerical_cause),
                        )
                    })?;
                frontiers_by_subject.push((subject.clone(), enumerated.clone()));
                enumerated
            };
            if frontier.admitted().is_empty() && frontier.rejections().is_empty() {
                proposed_everywhere = false;
            }
            // One region role yields one region subject, so its frontier and any
            // rejection it carries are recorded exactly once however many covers
            // place that same region.
            let first_sighting = !recorded_roles.contains_key(role);
            if first_sighting {
                frontier_cause = record_frontier(explain, role, &frontier, frontier_cause)?;
                for rejection in frontier.rejections() {
                    let error = match rejection {
                        crate::frontier::FrontierRejection::Infeasible {
                            axis,
                            required,
                            available,
                            ..
                        } => Some(PhysicalError::Target {
                            rule: axis,
                            region: region_id_of(cover, region),
                            required: *required,
                            available: *available,
                        }),
                        crate::frontier::FrontierRejection::Unhonourable { cause, .. } => {
                            Some(PhysicalError::Numerical {
                                region: region_id_of(cover, region),
                                cause: *cause,
                            })
                        }
                        // A reserved body variant and an inapplicable proposal
                        // are not target verdicts and carry no rejection to
                        // attribute to this region.
                        crate::frontier::FrontierRejection::UnsupportedVariant { .. }
                        | crate::frontier::FrontierRejection::NotApplicable { .. } => None,
                    };
                    if let Some(error) = error {
                        let cause = record_target_rejection(explain, &error, role, frontier_cause)?;
                        recorded_roles.insert(role, Some(cause));
                        rejections.push(TargetRejection { role, error, cause })?;
                    }
                }
                recorded_roles.entry(role).or_insert(None);
            }
            if let Some(Some(cause)) = recorded_roles.get(role) {
                refusal.get_or_insert(*cause);
            }
            region_frontiers.push(RegionFrontier::new(subject, frontier));
        }
        if proposed_everywhere && let Some(cause) = refusal {
            refused_covers.push((cover.identity(), cause));
        }
        sources.push(CoverFrontiers::new(cover, region_frontiers));
    }

    let portfolio =
        select_physical_plans(semantic, budgets, formation, &sources).map_err(|source| {
            failure_at_source(
                source.into(),
                ExplainStage::Selection,
                record_cause(frontier_cause),
            )
        })?;
    for (identity, cause) in refused_covers {
        if portfolio
            .plans()
            .iter()
            .all(|plan| plan.cover().identity() != identity)
        {
            note_infeasible_cover(explain, &identity.label(), Some(cause))?;
        }
    }
    let selection_record = record_plan_selection(explain, &portfolio, frontier_cause)?;
    Ok(CompletePlans {
        lowering,
        portfolio,
        legality,
        numerical,
        rejections,
        selection_record,
    })
}

/// Records why one occurrence's lowering could not be established.
///
/// The three classes stay distinct. An absent capability is a deferred
/// capability: the installed authority was never extended to this occurrence. A
/// contended capability is a disproved checked predicate: two extensions
/// contradict each other, which is a defect in the installed authority rather
/// than a gap in it. A refused refinement is a disproved refinement predicate at
/// the kernel stage: a provider was resolved, drove the canonical builder, and
/// the emitted region does not realize the occurrence.
pub(super) fn record_lowering_failure(
    explain: &mut ExplainWriter,
    source: &LoweringError,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = format!("occurrence:{}", source.member().0);
    let (stage, subject_kind) = match source {
        LoweringError::Refine { .. } => (ExplainStage::KernelRefinement, SubjectKind::Kernel),
        LoweringError::Occurrence { .. } | LoweringError::Resolve { .. } => {
            (ExplainStage::CapabilityResolution, SubjectKind::Capability)
        }
    };
    let reason = source.reason();
    let missing = source.is_missing();
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(subject_kind, &key)?;
            let event = if missing {
                ExplainEvent::DeferredCapability {
                    predicate: PredicateKey::new("capability.index-access-resolved")?,
                    reason: ReasonCode::new(reason)?,
                }
            } else {
                ExplainEvent::Check {
                    stage,
                    assessment: PredicateAssessment::disproved(
                        match stage {
                            ExplainStage::KernelRefinement => {
                                "kernel.index-region-refines-occurrence"
                            }
                            _ => "capability.index-access-resolved",
                        },
                        ReasonCode::new(reason)?,
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    rejection: RejectionClass::IntrinsicInvalid,
                }
            };
            Ok(explain.push_detail(
                RuleRef::builtin("capability.index-access-resolution.v1")?,
                vec![subject],
                event,
                vec![cause],
            )?)
        })(),
        stage,
        subject_kind,
        &key,
        record_cause(cause),
    )
}

/// Attributes a lowering-stage failure to its exact phase and subject.
///
/// Resolution failures belong to [`ExplainStage::CapabilityResolution`] and
/// refinement refusals to [`ExplainStage::KernelRefinement`]; both are reported
/// as unsupported capabilities rather than as target infeasibility, because the
/// installed authority is what could not lower the program.
pub(super) fn lowering_failure(source: &LoweringError, cause: ExplainRecordId) -> TargetFailure {
    let stage = match source {
        LoweringError::Refine { .. } => ExplainStage::KernelRefinement,
        LoweringError::Occurrence { .. } | LoweringError::Resolve { .. } => {
            ExplainStage::CapabilityResolution
        }
    };
    target_failure(
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: source.reason(),
        }),
        stage,
        format!("lowering-{}", source.reason()),
        SubjectKind::Capability,
        format!("occurrence:{}", source.member().0),
        record_cause(cause),
    )
}

/// Returns the planning ordinal a cover region's implementation will carry.
///
/// The ordinal is presentation only; a rejected proposal has no verified region,
/// so the region subject's position in the cover is the stable coordinate to
/// attribute the rejection to.
pub(super) fn region_id_of(
    cover: &RegionCover,
    region: &crate::cover::CoverRegion,
) -> crate::physical::RegionId {
    let position = cover
        .regions()
        .iter()
        .position(|candidate| candidate.occurrence() == region.occurrence())
        .unwrap_or(0);
    crate::physical::RegionId::new(u32::try_from(position).unwrap_or(u32::MAX))
}

/// Assembles one retained complete plan into KIR, a kernel program, and a plan.
pub(super) fn build_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    plan: &SelectedPlan,
    kind: ProgramAlternativeKind,
    plans: &CompletePlans,
    cause: Option<&TerminalCause>,
) -> Result<ProgramAlternative, TargetFailure> {
    let CompletePlans {
        lowering,
        legality,
        numerical,
        ..
    } = plans;
    let scheduled = plan_regions(plan);
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, cause.copied())
        })?;
    let program = build_plan_program(semantic, verified, kind, &scheduled).map_err(|error| {
        failure_at_source(error, ExplainStage::ProgramVerification, cause.copied())
    })?;
    assert_kernels_match_program(verified, &scheduled, &program, &kernels).map_err(|error| {
        failure_at_source(
            error.into(),
            ExplainStage::ProgramVerification,
            cause.copied(),
        )
    })?;
    let artifact_plan = build_artifact_plan(
        semantic,
        verified,
        &scheduled,
        &kernels,
        &program,
        lowering.providers(),
    )
    .map_err(|error| {
        failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause.copied())
    })?;
    let equivalence = EquivalenceEvidence {
        legality: plan
            .cover()
            .regions()
            .iter()
            .enumerate()
            .filter_map(|(position, region)| {
                legality
                    .get(region.occurrence())
                    .map(|proof| (position, proof.clone()))
            })
            .collect(),
        numerical: match kind {
            ProgramAlternativeKind::Fused => numerical.clone(),
            ProgramAlternativeKind::Materialized => None,
        },
    };
    Ok(ProgramAlternative {
        stable_id: plan.identity().label(),
        kind,
        plan: plan.clone(),
        scheduled_regions: scheduled,
        kernels,
        program,
        artifact_plan,
        structural_cost: plan.cost(),
        equivalence,
    })
}

/// Returns a plan's verified scheduled regions in ascending planning order,
/// borrowed from the plan rather than copied out of it.
///
/// A plan's selections are in canonical occurrence order, which is content
/// derived rather than execution ordered. Downstream program assembly consumes
/// producers before consumers, so the regions are ordered by the planning ordinal
/// the request-subject binding already pinned for each recognized region.
///
/// Ordering is the whole of what this derives, so it sorts references. A caller
/// that must *own* the result asks for it through [`plan_regions`]; a caller
/// that only compares against regions it already holds does not pay for a copy.
pub(super) fn plan_region_order(plan: &SelectedPlan) -> Vec<&VerifiedScheduledRegion> {
    let mut regions: Vec<&VerifiedScheduledRegion> = plan
        .selections()
        .iter()
        // An opaque call contributes no scheduled region. Filtered rather than
        // rejected here because this is an ordering helper, not an admission
        // check — the stage that must *lower* a plan is where an unlowerable
        // body rejects, and doing it twice would put the refusal in the place
        // with less to say about it.
        .filter_map(|selection| selection.implementation().scheduled())
        .collect();
    regions.sort_by_key(|region| region.region().index.id.get());
    regions
}

/// Returns an owned copy of a plan's scheduled regions in planning order.
pub(super) fn plan_regions(plan: &SelectedPlan) -> Vec<VerifiedScheduledRegion> {
    plan_region_order(plan).into_iter().cloned().collect()
}

/// Assembles the verified kernel program for one plan shape.
///
/// The bounded profile implements exactly two program shapes: a one-region fused
/// program and a two-region materialized program. Any other retained plan shape
/// is invalid compiler output and rejects explicitly rather than being
/// approximated by the closest implemented assembly.
pub(super) fn build_plan_program(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    kind: ProgramAlternativeKind,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, CompileError> {
    match (kind, scheduled) {
        (ProgramAlternativeKind::Fused, [region]) => {
            build_fused_kernel_program(semantic, verified, region).map_err(CompileError::from)
        }
        (ProgramAlternativeKind::Materialized, [_, _]) => {
            build_kernel_program(semantic, verified, scheduled).map_err(CompileError::from)
        }
        _ => Err(CompileError::from(ProgramError::Structure {
            rule: "unsupported-plan-shape",
        })),
    }
}

/// Returns the identity of the first structurally non-dominated alternative.
///
/// Domination is the Pareto relation the selection authority already computed
/// over exact structural counts; it is never a scalar latency total order. When
/// several plans are mutually non-dominated the canonical identity order breaks
/// the tie deterministically, so the choice is reproducible without inventing a
/// preference between incomparable trade-offs.
///
/// The match is on the plan identities themselves rather than on their explain
/// labels. A label is a 64-bit digest of those bytes, so matching on it asked a
/// weaker question than the one intended — two distinct plans that collided
/// would have compared equal — and it had to allocate a `String` per retained
/// plan to ask it. Comparing the identities directly is both the stronger check
/// and the one that allocates nothing; the borrowed `stable_id` returned here is
/// the label the matched alternative already computed once at construction.
pub(super) fn select_non_dominated<'alternatives>(
    portfolio: &SelectedPortfolio,
    alternatives: &'alternatives [ProgramAlternative],
) -> Result<&'alternatives str, CompileError> {
    let retained = portfolio.non_dominated();
    let selected = retained.iter().find_map(|plan| {
        alternatives
            .iter()
            .find(|alternative| alternative.plan.identity() == plan.identity())
    });
    selected
        .map(|alternative| alternative.stable_id.as_str())
        .ok_or(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "portfolio-empty",
            }),
        ))
}
