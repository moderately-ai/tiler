#![allow(
    clippy::wildcard_imports,
    reason = "this module is one half of `pipeline`, not a separate concept: it is a \
private child that exists so the root reads as the compilation story, and every name it \
uses is defined in that root. Enumerating them would restate fifty parent items and would \
have to be restated again on every change, for no reader benefit -- the glob is scoped to \
one parent whose contents are visible in the same directory"
)]

//! Explain-trace production.
//!
//! Every `record_*` step the compilation writes, separated by that
//! invariant rather than by size: nothing here decides anything. A function
//! in this module observes a decision the orchestration already made and
//! writes it to the explain record, so a change here can alter what a
//! reader is told and never what the compiler chose.

use super::*;

/// Records the bounded cover enumeration, its budget stops, and infeasibility.
pub(super) fn record_cover_enumeration(
    explain: &mut ExplainWriter,
    enumeration: &CoverEnumeration,
    root: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut cause = record_count_step(
        explain,
        "cover.enumeration.v1",
        SubjectKind::Candidate,
        "region-cover",
        ExplainStage::CandidateEnumeration,
        "cover.complete-and-legal",
        "cover-count",
        enumeration.covers().len(),
        root,
    )?;
    for stop in enumeration.budget_stops() {
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Candidate, "region-cover")?;
                Ok(explain.push_detail(
                    RuleRef::builtin("cover.enumeration.v1")?,
                    vec![subject],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::CandidateEnumeration,
                        resource: crate::explain::ResourceKey::new(stop.resource.key())?,
                        limit: stop.limit,
                        actual: stop.actual,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::Candidate,
            "region-cover",
            record_cause(cause),
        )?;
    }
    for infeasibility in enumeration.infeasibilities() {
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Candidate, "region-cover")?;
                Ok(explain.push_detail(
                    RuleRef::builtin("cover.enumeration.v1")?,
                    vec![subject],
                    ExplainEvent::DeferredCapability {
                        predicate: PredicateKey::new("cover.complete-and-legal")?,
                        reason: ReasonCode::new(infeasibility.reason())?,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::Candidate,
            "region-cover",
            record_cause(cause),
        )?;
    }
    Ok(cause)
}

/// Records one region's typed fusion-legality outcome.
///
/// A legal region is an admitted check attributed to the capability provider that
/// declared the member operations' fusion roles; a rejection is a disproved
/// numerical-legality check, and an unknown is a deferred capability. The three
/// stay distinct classes rather than collapsing into one "not fused" verdict.
pub(super) fn record_fusion_legality(
    explain: &mut ExplainWriter,
    capabilities: &FusionNumericalCapabilities,
    candidate: &RegionCandidate,
    outcome: &FusionLegality,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = candidate.label().to_owned();
    explain_step(
        (|| -> Result<_, CompileError> {
            let provider = ProviderRef::registered(capabilities.provider())?;
            let rule = RuleRef::provided("fusion.legality.v1", capabilities.revision(), provider)?;
            let subject = explain.subject(SubjectKind::Candidate, &key)?;
            let event = match outcome {
                FusionLegality::Legal(_) => ExplainEvent::Check {
                    stage: ExplainStage::NumericalLegality,
                    assessment: PredicateAssessment::proven(
                        "fusion.obligations-discharged",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    rejection: RejectionClass::NumericalIllegal,
                },
                FusionLegality::Rejected(rejection) => ExplainEvent::Check {
                    stage: ExplainStage::NumericalLegality,
                    assessment: PredicateAssessment::disproved(
                        "fusion.obligations-discharged",
                        ReasonCode::new(rejection.reason())?,
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    rejection: RejectionClass::NumericalIllegal,
                },
                FusionLegality::Unknown(unknown) => ExplainEvent::DeferredCapability {
                    predicate: PredicateKey::new("fusion.obligations-discharged")?,
                    reason: ReasonCode::new(unknown.reason())?,
                },
            };
            Ok(explain.push_detail(rule, vec![subject], event, vec![cause])?)
        })(),
        ExplainStage::NumericalLegality,
        SubjectKind::Candidate,
        &key,
        record_cause(cause),
    )
}

/// Records every recognized occurrence's resolved capability and its evidence.
///
/// Two records per occurrence at most, and they are deliberately different
/// classes. The [`ExplainStage::CapabilityResolution`] record is an admitted
/// checked invariant attributed to the resolved provider: the installed registry
/// resolved exactly one index-access capability for this occurrence. The
/// [`ExplainStage::KernelRefinement`] record is either the exhaustive finite
/// evidence that the provider's region realizes the occurrence, or — when the
/// exhaustive access proof could not afford the region — a typed budget stop
/// naming the resource, its limit, and the required amount, plus an explicit
/// `Unknown` assessment. The budget stop is never a rejection: nothing about the
/// region was disproved, the analysis stopped.
pub(super) fn record_lowering(
    explain: &mut ExplainWriter,
    lowering: &ResolvedLowering,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    for occurrence in lowering.occurrences() {
        let key = occurrence.subject_key();
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let provider = ProviderRef::lowering(occurrence.provider())?;
                let rule = RuleRef::provided(
                    "capability.index-access-resolution.v1",
                    occurrence.provider().capability_revision().get(),
                    provider,
                )?;
                let subject = explain.subject(SubjectKind::Capability, &key)?;
                Ok(explain.push_detail(
                    rule,
                    vec![subject],
                    ExplainEvent::Check {
                        stage: ExplainStage::CapabilityResolution,
                        assessment: PredicateAssessment::proven(
                            "capability.index-access-resolved",
                            EvidenceBasis::CheckedInvariant,
                        )?,
                        rejection: RejectionClass::IntrinsicInvalid,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CapabilityResolution,
            SubjectKind::Capability,
            &key,
            record_cause(cause),
        )?;
        cause = record_refinement(explain, occurrence, cause)?;
    }
    Ok(cause)
}

/// Records one occurrence's refinement evidence or its recorded proof gap.
pub(super) fn record_refinement(
    explain: &mut ExplainWriter,
    occurrence: &crate::lowering::OccurrenceLowering,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = occurrence.subject_key();
    match occurrence.evidence() {
        OccurrenceEvidence::Refined(refinement) => {
            let identity = refinement_label(refinement);
            explain_step(
                (|| -> Result<_, CompileError> {
                    let provider = ProviderRef::lowering(occurrence.provider())?;
                    let rule = RuleRef::provided(
                        "kernel.index-region-refinement.v1",
                        occurrence.provider().capability_revision().get(),
                        provider,
                    )?;
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    Ok(explain.push_detail(
                        rule,
                        vec![subject],
                        ExplainEvent::Check {
                            stage: ExplainStage::KernelRefinement,
                            assessment: PredicateAssessment::proven(
                                "kernel.index-region-refines-occurrence",
                                EvidenceBasis::ExhaustiveFinite,
                            )?
                            .with_fact(ExplainFact::new(
                                "refinement-identity",
                                FactValue::Identity(crate::explain::SubjectKey::new(identity)?),
                            )?)?,
                            rejection: RejectionClass::IntrinsicInvalid,
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::KernelRefinement,
                SubjectKind::Kernel,
                &key,
                record_cause(cause),
            )
        }
        OccurrenceEvidence::BudgetStopped(stop) => {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin("kernel.index-region-refinement.v1")?,
                        vec![subject],
                        ExplainEvent::BudgetStop {
                            stage: ExplainStage::KernelRefinement,
                            resource: crate::explain::ResourceKey::new(stop.resource_key())?,
                            limit: stop.limit,
                            actual: u64::try_from(stop.required).unwrap_or(u64::MAX),
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::KernelRefinement,
                SubjectKind::Kernel,
                &key,
                record_cause(cause),
            )?;
            explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin("kernel.index-region-refinement.v1")?,
                        vec![subject],
                        ExplainEvent::Check {
                            stage: ExplainStage::KernelRefinement,
                            assessment: PredicateAssessment::unknown(
                                "kernel.index-region-refines-occurrence",
                                ReasonCode::new("proof-budget-exhausted")?,
                            )?,
                            rejection: RejectionClass::IntrinsicInvalid,
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::KernelRefinement,
                SubjectKind::Kernel,
                &key,
                record_cause(cause),
            )
        }
    }
}

/// Returns the stable presentation label of one refinement occurrence identity.
///
/// The label is a presentation handle over the identity's trailing bytes, never
/// the identity itself: the canonical bytes stay in the retained
/// [`crate::legality::IndexRefinement`], which is what any downstream check
/// compares.
pub(super) fn refinement_label(refinement: &crate::legality::IndexRefinement) -> String {
    use std::fmt::Write as _;

    let bytes = refinement.identity().as_bytes();
    let tail = bytes.len().saturating_sub(8);
    let mut label = String::from("refinement:");
    for byte in &bytes[tail..] {
        let _ = write!(label, "{byte:02x}");
    }
    label
}

/// Records the whole-program strict-`f32` numerical equivalence sound proof.
///
/// The proof is attributed to the provider that lowers the reduction occurrence,
/// because that is the operation whose reassociation the proof forbids. A
/// program with no recognized reduction has no fused equivalence claim to make.
pub(super) fn record_numerical_equivalence(
    explain: &mut ExplainWriter,
    verified: &crate::request::VerifiedTargetRequest,
    lowering: &ResolvedLowering,
    candidate: &RegionCandidate,
    proof: &FusionNumericalProof,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = candidate.label().to_owned();
    explain_step(
        (|| -> Result<_, CompileError> {
            let reduction = verified.serial_sum().members.reduction();
            let provider = lowering
                .occurrences()
                .iter()
                .find(|occurrence| reduction.contains(&occurrence.member()))
                .map(crate::lowering::OccurrenceLowering::provider)
                .ok_or_else(|| {
                    CompileError::from(ProgramError::Structure {
                        rule: "reduction-provider-missing",
                    })
                })?;
            let provider_ref = ProviderRef::lowering(provider)?;
            let subject = explain.subject(SubjectKind::Candidate, &key)?;
            Ok(explain.push_detail(
                RuleRef::provided("fusion.strict-f32-equivalence", 1, provider_ref.clone())?,
                vec![subject],
                check(
                    ExplainStage::NumericalLegality,
                    "fusion.strict-f32-equivalence",
                    EvidenceBasis::SoundProof(VerifiedEvidenceRef::from_fusion_numerical(
                        verified,
                        proof,
                        provider_ref,
                    )?),
                )?,
                vec![cause],
            )?)
        })(),
        ExplainStage::NumericalLegality,
        SubjectKind::Candidate,
        &key,
        record_cause(cause),
    )
}

/// Records one region subject's bounded implementation frontier.
pub(super) fn record_frontier(
    explain: &mut ExplainWriter,
    role: &'static str,
    frontier: &crate::frontier::ImplementationFrontier,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = format!("region:{role}");
    let cause = record_count_step(
        explain,
        "frontier.enumeration.v1",
        SubjectKind::Schedule,
        &key,
        ExplainStage::IntrinsicScheduling,
        "frontier.locally-feasible",
        "admitted-count",
        frontier.admitted().len(),
        cause,
    )?;
    // Rejections were previously not recorded at all — only the admitted count
    // was. That was survivable while every rejection was either a reserved
    // variant or an inapplicable target, both of which a reader could infer from
    // an empty frontier. It stops being survivable once a proposal can be
    // refused for a reason specific to *it*: an opaque call refused for a
    // numerical mismatch and one refused because nothing registered it are
    // indistinguishable from "no provider proposed", and the fix is different in
    // each case.
    //
    // A count rather than a record per rejection. What the count delivers is
    // "something was refused here"; what it does NOT deliver is *why* — the
    // typed rejection and its stable reason code live only on the in-memory
    // `ImplementationFrontier` and never reach explain output, so the
    // unregistered-versus-mismatch distinction above is recoverable from the
    // frontier object, not from the trace. Carrying the reasons into typed
    // records is part of the explain cost-vocabulary work
    // (`emit-analytical-costs-through-the-typed-cost-vocabulary`).
    record_count_step(
        explain,
        "frontier.enumeration.v1",
        SubjectKind::Schedule,
        &key,
        ExplainStage::IntrinsicScheduling,
        "frontier.rejections-recorded",
        "rejected-count",
        frontier.rejections().len(),
        cause,
    )
}

/// Records the complete-plan join: how many valid plans the portfolio retained.
pub(super) fn record_plan_selection(
    explain: &mut ExplainWriter,
    portfolio: &SelectedPortfolio,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut cause = record_count_step(
        explain,
        "selection.complete-plan.v1",
        SubjectKind::KernelProgram,
        "portfolio",
        ExplainStage::CandidateEnumeration,
        "selection.plans-complete-and-composed",
        "plan-count",
        portfolio.plans().len(),
        cause,
    )?;
    cause = record_analytical_costs(explain, portfolio, cause)?;
    for stop in portfolio.budget_stops() {
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::KernelProgram, "portfolio")?;
                Ok(explain.push_detail(
                    RuleRef::builtin("selection.complete-plan.v1")?,
                    vec![subject],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::CandidateEnumeration,
                        resource: crate::explain::ResourceKey::new(stop.resource.key())?,
                        limit: stop.limit,
                        actual: stop.actual,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::KernelProgram,
            "portfolio",
            record_cause(cause),
        )?;
    }
    Ok(cause)
}

/// Reports each retained plan's analytical component costs.
///
/// These are reported and never pruned on. A [`ComponentCost`] is not a
/// `PhysicalCostEstimate`, carries its own governed model key, and never enters
/// a dominance comparison — see `crate::component_cost` for why admitting a
/// second key into dominance would silently turn Pareto pruning off.
///
/// Terms are grouped by evidence class because [`ExplainEvent::CostAssessment`]
/// carries one basis for every term it contains. Exact derivations therefore
/// share one checked-invariant record and modelled bounds share one assumption
/// record; putting both in one record would overstate the bound or understate
/// the exact values. An `Unknown` component is deliberately *not* emitted as a
/// zero. It is counted in the exact record instead, so a reader can tell how
/// much of the model is unmodelled rather than reading an unmodelled component
/// as a free one.
fn record_analytical_costs(
    explain: &mut ExplainWriter,
    portfolio: &SelectedPortfolio,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    for plan in portfolio.plans() {
        let analytical = analytical_plan_cost(plan);
        // Dispatch is reported under both models, so the two must agree. A
        // duplicated number that drifted would be worse than no number at all:
        // a calibration pass comparing a device measurement against the
        // analytical dispatch count would attribute the difference to the
        // device. Debug-only because this is an invariant of two summations
        // over the same selections, not a condition any input can produce.
        debug_assert_eq!(
            analytical
                .get(CostComponent::Dispatch)
                .map(super::ComponentCost::value),
            Some(CostValue::Exact(plan.cost().dispatch_count())),
            "the analytical dispatch count disagrees with the structural one"
        );
        // Every deliberate cross-region materialization is one handoff edge, and
        // every edge imposes at least one wait, so the ordering-constraint count
        // can never fall below the materialization count. It exceeds it exactly
        // when a produced value has more than one consumer. Asserting the
        // inequality rather than equality is what keeps this true for the
        // multi-consumer case instead of pinning the single-consumer one.
        // Guarded on the value being `Exact` so the assertion states what it
        // checks: if `Synchronization` ever stopped being exact, the old form
        // (`is_some_and` over the extraction) would have fired with a message
        // blaming the ordering-constraint count for what was actually an
        // evidence-class change — a misattributed failure. A non-exact value
        // here simply skips the cross-check rather than lying about why.
        if let Some(cost) = analytical.get(CostComponent::Synchronization)
            && let CostValue::Exact(sync) = cost.value()
        {
            debug_assert!(
                sync >= plan.cost().materialization_count(),
                "fewer ordering constraints than cross-region materializations"
            );
        }
        let subject_key = plan.identity().label();
        let (exact_terms, bounded_terms) = explain_step(
            (|| -> Result<_, CompileError> {
                let mut exact_terms = Vec::new();
                let mut bounded_terms = Vec::new();
                for component in analytical.components() {
                    match component.value() {
                        CostValue::Exact(value) => {
                            exact_terms.push(CostTerm::new(
                                format!(
                                    "{}.{}",
                                    component.component().key(),
                                    component.value().class()
                                ),
                                analytical_quantity(component.unit(), value),
                            )?);
                        }
                        // Both ends are reported rather than a midpoint: a
                        // bound is the claim, and a midpoint would present a
                        // modelled range as a point estimate nobody derived.
                        CostValue::Bounded { low, high } => {
                            for (suffix, value) in [("low", low), ("high", high)] {
                                bounded_terms.push(CostTerm::new(
                                    format!(
                                        "{}.{}.{suffix}",
                                        component.component().key(),
                                        component.value().class()
                                    ),
                                    analytical_quantity(component.unit(), value),
                                )?);
                            }
                        }
                        CostValue::Unknown => {}
                    }
                }
                exact_terms.push(CostTerm::new(
                    "cost.unmodelled-components",
                    Quantity::Count(
                        u64::try_from(CANONICAL_COMPONENTS.len() - analytical.known_count())
                            .unwrap_or(u64::MAX),
                    ),
                )?);
                Ok((exact_terms, bounded_terms))
            })(),
            ExplainStage::Costing,
            SubjectKind::KernelProgram,
            &subject_key,
            record_cause(cause),
        )?;
        for (basis, terms) in [
            (EvidenceBasis::CheckedInvariant, exact_terms),
            (EvidenceBasis::Assumption, bounded_terms),
        ] {
            if terms.is_empty() {
                continue;
            }
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::KernelProgram, &subject_key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(ANALYTICAL_MODEL_KEY)?,
                        vec![subject],
                        ExplainEvent::CostAssessment {
                            model: CostModelKey::new(ANALYTICAL_MODEL_KEY)?,
                            basis,
                            terms,
                            disposition: CostDisposition::Reported,
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::Costing,
                SubjectKind::KernelProgram,
                &subject_key,
                record_cause(cause),
            )?;
        }
    }
    Ok(cause)
}

/// Carries one analytical value through the quantity variant fixed by its
/// component. The exhaustive match keeps `CostUnit` and `Quantity` from
/// silently drifting when either vocabulary grows.
const fn analytical_quantity(unit: CostUnit, value: u64) -> Quantity {
    match unit {
        CostUnit::Bytes => Quantity::Bytes(value),
        CostUnit::Count => Quantity::Count(value),
        CostUnit::Operations => Quantity::Operations(value),
        CostUnit::Registers => Quantity::Registers(value),
        CostUnit::Nanoseconds => Quantity::Nanoseconds(value),
    }
}

/// Records one region subject's hard-infeasible target rejection.
///
/// A capability rejection keeps the quantitative feasibility record; a numerical
/// one takes the honourability record, which is the only shape that can carry a
/// dimension, a required behaviour, a declared means, an honoured alternative,
/// and a declaring profile.
pub(super) fn record_target_rejection(
    explain: &mut ExplainWriter,
    error: &PhysicalError,
    role: &'static str,
    cause: ExplainRecordId,
) -> Result<TerminalCause, TargetFailure> {
    let key = format!("region:{role}");
    let (rule_key, event) = match error {
        PhysicalError::Target {
            rule,
            required,
            available,
            ..
        } => (
            format!("target.{rule}"),
            (|| -> Result<_, CompileError> {
                Ok(ExplainEvent::Feasibility {
                    predicate: PredicateKey::new(*rule)?,
                    outcome: crate::explain::FeasibilityOutcome::Rejected(ReasonCode::new(
                        "target-infeasible",
                    )?),
                    required: target_quantity(rule, *required)?,
                    available: target_quantity(rule, *available)?,
                })
            })(),
        ),
        PhysicalError::Numerical { cause, .. } => (
            format!("target.{}", cause.dimension().key()),
            (|| -> Result<_, CompileError> {
                Ok(ExplainEvent::NumericalHonourability {
                    dimension: PredicateKey::new(cause.dimension().key())?,
                    required: ReasonCode::new(cause.required().key())?,
                    outcome: crate::explain::HonourabilityOutcome::Unhonourable {
                        means: ReasonCode::new(cause.means().key())?,
                        honoured: cause
                            .honoured()
                            .map(|honoured| ReasonCode::new(honoured.key()))
                            .transpose()?,
                    },
                    profile: crate::explain::SubjectKey::new(cause.profile().key())?,
                })
            })(),
        ),
        PhysicalError::Intrinsic { .. }
        | PhysicalError::Refinement { .. }
        | PhysicalError::ShapeProductOverflow { .. } => {
            unreachable!("target rejection records require a target-feasibility error")
        }
    };
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Region, &key)?;
            Ok(explain.push_causal_detail(
                RuleRef::builtin(rule_key)?,
                subject,
                &event?,
                vec![cause],
            )?)
        })(),
        ExplainStage::TargetFeasibility,
        SubjectKind::Region,
        &key,
        record_cause(cause),
    )
}

/// Notes one cover as an infeasible alternative in the terminal ledger.
pub(super) fn note_infeasible_cover(
    explain: &mut ExplainWriter,
    label: &str,
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Alternative, label)?;
            explain.note_infeasible_alternative(subject, cause)?;
            Ok(())
        })(),
        ExplainStage::Selection,
        SubjectKind::Alternative,
        label,
        cause,
    )
}

pub(super) fn record_target_admissions(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let profile = request.target_profile();
    for scheduled in &alternative.scheduled_regions {
        let region = scheduled.region();
        // Re-derive the admitted feasibility facts from the single feasibility
        // authority rather than a parallel check list, so the admitted trace
        // cannot drift from the decision that admitted the region. A verified
        // region has already proven feasible, so a non-proven verdict here is an
        // internal inconsistency and fails closed via the physical-error stage.
        let admitted = crate::physical::assess_region(
            region.index.id,
            scheduled.requirements(),
            request.numerical_contract().arithmetic,
            region.schedule.work_items,
            &profile,
        )
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, record_cause(cause))
        })?;
        let key = format!("{}/region:{}", alternative.stable_id, region.index.id.get());
        for predicate in admitted.predicates() {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Region, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(format!("target.{}", predicate.axis().key()))?,
                        vec![subject],
                        ExplainEvent::Feasibility {
                            predicate: PredicateKey::new(predicate.axis().key())?,
                            outcome: crate::explain::FeasibilityOutcome::Admitted,
                            required: predicate.required(),
                            available: predicate.available(),
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::TargetFeasibility,
                SubjectKind::Region,
                &key,
                record_cause(cause),
            )?;
        }
        // The admitted trace records the *means* of each honoured dimension, not
        // only that it was honoured. An emulated dimension is admitted and emits
        // different operations, so a trace that carried only the verdict would
        // leave a reader unable to tell one from native support.
        for honoured in admitted.honoured() {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Region, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(format!("target.{}", honoured.dimension().key()))?,
                        vec![subject],
                        ExplainEvent::NumericalHonourability {
                            dimension: PredicateKey::new(honoured.dimension().key())?,
                            required: ReasonCode::new(honoured.behaviour().key())?,
                            outcome: crate::explain::HonourabilityOutcome::Honoured {
                                means: ReasonCode::new(honoured.means().key())?,
                            },
                            profile: crate::explain::SubjectKey::new(honoured.profile().key())?,
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::TargetFeasibility,
                SubjectKind::Region,
                &key,
                record_cause(cause),
            )?;
        }
    }
    Ok(cause)
}

pub(super) fn target_quantity(rule: &str, value: u64) -> Result<Quantity, ExplainError> {
    match rule {
        "grid-axis" | "threads-per-workgroup" => Ok(Quantity::Threads(value)),
        "buffer-bindings" => Ok(Quantity::Bindings(value)),
        "local-memory-bytes" => Ok(Quantity::Bytes(value)),
        "index-bits" | "device-memory" | "barriers" => Ok(Quantity::Count(value)),
        _ => Err(ExplainError::UnknownQuantityUnit),
    }
}

/// Records one retained alternative's per-layer admitted evidence.
pub(super) fn record_alternative_explain(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    root: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut boundary_causes = Vec::new();
    for scheduled in &alternative.scheduled_regions {
        let region_id = scheduled.region().index.id.get();
        let key = format!("{}/region:{region_id}", alternative.stable_id);
        let record = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Region, &key)?;
                Ok(explain.push_detail(
                    RuleRef::provided(
                        "compile.region.verified",
                        1,
                        ProviderRef::registered(&GovernedPhysicalProvider::identity())?,
                    )?,
                    vec![subject],
                    check(
                        ExplainStage::RegionFormation,
                        "region.semantic-coverage",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    vec![root],
                )?)
            })(),
            ExplainStage::RegionFormation,
            SubjectKind::Region,
            &key,
            record_cause(root),
        )?;
        boundary_causes.push(record);
    }
    let key = format!("{}/boundary", alternative.stable_id);
    let boundary = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Boundary, &key)?;
            Ok(explain.push_detail(
                RuleRef::builtin("compile.plan.boundary")?,
                vec![subject],
                check_with_count(
                    ExplainStage::RegionFormation,
                    "boundary.handoffs-satisfied",
                    "handoff-count",
                    alternative.plan.handoffs().len(),
                )?,
                boundary_causes,
            )?)
        })(),
        ExplainStage::RegionFormation,
        SubjectKind::Boundary,
        &key,
        record_cause(root),
    )?;
    let key = format!("{}/schedules", alternative.stable_id);
    let schedule = record_count_step(
        explain,
        "schedule.plan-regions",
        SubjectKind::Schedule,
        &key,
        ExplainStage::IntrinsicScheduling,
        "schedule.intrinsic-valid",
        "schedule-count",
        alternative.scheduled_regions.len(),
        boundary,
    )?;
    let target = record_target_admissions(explain, request, alternative, schedule)?;
    let key = format!("{}/kernels", alternative.stable_id);
    let kernel = record_count_step(
        explain,
        "kernel.plan-refinement",
        SubjectKind::Kernel,
        &key,
        ExplainStage::KernelRefinement,
        "kernel.exact-refinement",
        "kernel-count",
        alternative.kernels.len(),
        target,
    )?;
    let key = format!("{}/program", alternative.stable_id);
    let program = record_count_step(
        explain,
        "program.plan-verified",
        SubjectKind::KernelProgram,
        &key,
        ExplainStage::ProgramVerification,
        "program.verified",
        "stage-count",
        alternative.program.stage_count(),
        kernel,
    )?;
    let key = format!("{}/artifact", alternative.stable_id);
    record_count_step(
        explain,
        "artifact.plan-construction",
        SubjectKind::ArtifactPlan,
        &key,
        ExplainStage::ArtifactPlanning,
        "artifact.plan-verified",
        "provider-count",
        alternative.artifact_plan.lowering_providers().len(),
        program,
    )
}

pub(super) fn record_cost_and_selection(
    alternatives: &[ProgramAlternative],
    selected_alternative_id: &str,
    causes: &[(String, ExplainRecordId)],
    explain: &mut ExplainWriter,
) -> Result<(), TargetFailure> {
    for alternative in alternatives {
        let cost = alternative.structural_cost;
        let cause = causes
            .iter()
            .find_map(|(id, cause)| (*id == alternative.stable_id).then_some(*cause));
        let (subject, cost_record) = explain_step(
            (|| -> Result<_, CompileError> {
                let subject =
                    explain.subject(SubjectKind::Alternative, alternative.stable_id.as_str())?;
                let terms = vec![
                    CostTerm::new("dispatch-count", Quantity::Count(cost.dispatch_count()))?,
                    CostTerm::new(
                        "launched-threads",
                        Quantity::Threads(cost.launched_threads()),
                    )?,
                    CostTerm::new("temporary-bytes", Quantity::Bytes(cost.temporary_bytes()))?,
                    CostTerm::new(
                        "materialization-count",
                        Quantity::Count(cost.materialization_count()),
                    )?,
                ];
                let record = explain.push_causal_detail(
                    RuleRef::builtin(STRUCTURAL_COST_MODEL_KEY)?,
                    subject.clone(),
                    &ExplainEvent::CostAssessment {
                        model: CostModelKey::new(STRUCTURAL_COST_MODEL_KEY)?,
                        basis: EvidenceBasis::CheckedInvariant,
                        terms,
                        disposition: CostDisposition::Retained,
                    },
                    optional_cause(cause),
                )?;
                Ok((subject, record))
            })(),
            ExplainStage::Costing,
            SubjectKind::Alternative,
            alternative.stable_id.as_str(),
            cause.map(TerminalCause::from_record),
        )?;
        let outcome = if alternative.stable_id == selected_alternative_id {
            SelectionOutcome::Selected
        } else if alternatives
            .iter()
            .find(|item| item.stable_id == selected_alternative_id)
            .is_some_and(|selected| {
                selected
                    .structural_cost
                    .dominates(&alternative.structural_cost)
            })
        {
            SelectionOutcome::Dominated
        } else {
            SelectionOutcome::NotSelectedTradeoff
        };
        explain_step(
            explain
                .note_selection(subject, outcome, Some(cost_record))
                .map_err(Into::into),
            ExplainStage::Selection,
            SubjectKind::Alternative,
            alternative.stable_id.as_str(),
            Some(cost_record),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CostUnit, Quantity, analytical_quantity};

    /// Every analytical unit maps to its namesake typed quantity.
    ///
    /// This includes the two currently unmodelled components, so producing
    /// their first value cannot fall back to a dimensionless count.
    #[test]
    fn every_analytical_unit_has_a_typed_quantity() {
        assert_eq!(analytical_quantity(CostUnit::Bytes, 1), Quantity::Bytes(1));
        assert_eq!(analytical_quantity(CostUnit::Count, 2), Quantity::Count(2));
        assert_eq!(
            analytical_quantity(CostUnit::Operations, 3),
            Quantity::Operations(3)
        );
        assert_eq!(
            analytical_quantity(CostUnit::Registers, 4),
            Quantity::Registers(4)
        );
        assert_eq!(
            analytical_quantity(CostUnit::Nanoseconds, 5),
            Quantity::Nanoseconds(5)
        );
    }
}
