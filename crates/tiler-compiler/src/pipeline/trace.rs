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
/// [`ExplainStage::KernelRefinement`] record is the exhaustive finite evidence
/// that the provider's region realizes the occurrence. An unresolved semantic
/// predicate fails before a `ResolvedLowering` exists and therefore cannot be
/// recorded here as if it supported an executable plan.
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

/// Records one occurrence's refinement evidence.
pub(super) fn record_refinement(
    explain: &mut ExplainWriter,
    occurrence: &crate::lowering::OccurrenceLowering,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = occurrence.subject_key();
    match occurrence.evidence() {
        OccurrenceEvidence::Refined(refinement) => {
            let identity = refinement_label(refinement);
            let refinement_cause = explain_step(
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
            )?;
            record_semantic_discharge_proofs(explain, &key, refinement, refinement_cause)
        }
    }
}

/// Records every sealed residual-domain proof before cover enumeration.
fn record_semantic_discharge_proofs(
    explain: &mut ExplainWriter,
    key: &str,
    refinement: &crate::legality::IndexRefinement,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    use std::fmt::Write as _;

    for (ordinal, proof) in refinement
        .content()
        .index_domain_proofs()
        .iter()
        .enumerate()
    {
        let ordinal = u64::try_from(ordinal).expect("index-region obligations are host bounded");
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Kernel, key)?;
                let mut obligation_key = String::from("obligation:");
                for byte in proof.obligation().canonical_local_key().as_bytes() {
                    write!(obligation_key, "{byte:02x}").expect("writing to a String cannot fail");
                }
                let (basis, proof_kind, points) = match proof.proof() {
                    IndexDomainDischargeProof::Sound { .. } => (
                        EvidenceBasis::SoundProof(VerifiedEvidenceRef::from_index_domain(
                            &subject, proof,
                        )),
                        "sound-proof",
                        None,
                    ),
                    IndexDomainDischargeProof::ExhaustiveFinite { points, .. } => (
                        EvidenceBasis::ExhaustiveFinite,
                        "exhaustive-finite",
                        Some(*points),
                    ),
                };
                let predicate_kind = match proof.obligation().predicate() {
                    tiler_ir::index::IndexDomainPredicate::NonNegative { .. } => {
                        "index-domain.non-negative"
                    }
                    tiler_ir::index::IndexDomainPredicate::LessThanExtent { .. } => {
                        "index-domain.less-than-extent"
                    }
                };
                let mut assessment = PredicateAssessment::proven(
                    format!("kernel.index-domain-obligation.{ordinal}"),
                    basis,
                )?
                .with_fact(ExplainFact::new(
                    "obligation-ordinal",
                    FactValue::Count(ordinal),
                )?)?
                .with_fact(ExplainFact::new(
                    "obligation-key",
                    FactValue::Identity(crate::explain::SubjectKey::new(obligation_key)?),
                )?)?
                .with_fact(ExplainFact::new(
                    "predicate-kind",
                    FactValue::Identity(crate::explain::SubjectKey::new(predicate_kind)?),
                )?)?
                .with_fact(ExplainFact::new(
                    "evidence-basis",
                    FactValue::Identity(crate::explain::SubjectKey::new(proof_kind)?),
                )?)?
                .with_fact(ExplainFact::new(
                    "discharge-provider",
                    FactValue::Identity(crate::explain::SubjectKey::new(format!(
                        "{}.{}",
                        proof.authority().provider().namespace(),
                        proof.authority().provider().name(),
                    ))?),
                )?)?
                .with_fact(ExplainFact::new(
                    "discharge-rule",
                    FactValue::Identity(crate::explain::SubjectKey::new(format!(
                        "{}.{}",
                        proof.authority().rule().identity().namespace(),
                        proof.authority().rule().identity().name(),
                    ))?),
                )?)?
                .with_fact(ExplainFact::new(
                    "discharge-revision",
                    FactValue::Count(u64::from(proof.authority().revision().get())),
                )?)?;
                if let Some(points) = points {
                    assessment = assessment.with_fact(ExplainFact::new(
                        "exhaustive-points",
                        FactValue::Count(points),
                    )?)?;
                }
                Ok(explain.push_detail(
                    RuleRef::builtin("index-domain.semantic-discharge.v1")?,
                    vec![subject],
                    ExplainEvent::Check {
                        stage: ExplainStage::SemanticDischarge,
                        assessment,
                        rejection: RejectionClass::IntrinsicInvalid,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::SemanticDischarge,
            SubjectKind::Kernel,
            key,
            record_cause(cause),
        )?;
    }
    Ok(cause)
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
    let mut cause = record_count_step(
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
    for rejection in frontier.rejections() {
        match rejection {
            crate::frontier::FrontierRejection::OpaqueCall {
                provider,
                proposal,
                cause: rejection,
            } => {
                cause =
                    record_opaque_call_rejection(explain, provider, proposal, rejection, cause)?;
            }
            crate::frontier::FrontierRejection::StrategyDeclined {
                provider,
                strategy,
                cause: decline,
            } => {
                cause =
                    record_declined_strategy(explain, role, provider, strategy, *decline, cause)?;
            }
            crate::frontier::FrontierRejection::Infeasible { .. }
            | crate::frontier::FrontierRejection::Unhonourable { .. }
            | crate::frontier::FrontierRejection::UnsupportedVariant { .. }
            | crate::frontier::FrontierRejection::NotApplicable { .. } => {}
        }
    }
    // The summaries remain present beside the detail records: consumers can
    // answer "how many?" without reconstructing it from event classes, while
    // each opaque-call refusal and each declined strategy above retains the
    // typed answer to "why?".
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

/// Records one strategy a provider considered for this region and withheld.
///
/// This is the record that makes an *absence* readable. Every other frontier
/// record answers "why was this candidate not admitted"; a reader of those alone
/// cannot tell a request whose extents admit no balanced split from one whose
/// provider never implemented splitting, because both enumerate exactly the
/// serial alternative. The typed cause and its exact fact — the refused
/// numerical dimension, or the contributor extent that admitted no partition —
/// are what separate them.
///
/// The disposition is `Disproved` and never `Unknown`: each cause is decided
/// from the request alone, before any region exists, so nothing further could
/// resolve it.
fn record_declined_strategy(
    explain: &mut ExplainWriter,
    role: &'static str,
    provider: &crate::frontier::PhysicalProviderProvenance,
    strategy: &'static str,
    cause: crate::frontier::StrategyDeclineCause,
    parent: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = format!("region:{role}");
    let stage = match cause {
        crate::frontier::StrategyDeclineCause::NumericalPermissionRefused { .. } => {
            ExplainStage::NumericalLegality
        }
        crate::frontier::StrategyDeclineCause::NoAdmissibleShape { .. }
        | crate::frontier::StrategyDeclineCause::Unrepresentable { .. } => {
            ExplainStage::IntrinsicScheduling
        }
    };
    explain_step(
        (|| -> Result<_, CompileError> {
            let subjects = vec![
                explain.subject(SubjectKind::Schedule, &key)?,
                explain.subject(SubjectKind::Provider, provider.explain_subject())?,
            ];
            let mut assessment = PredicateAssessment::disproved(
                "frontier.strategy-offered",
                ReasonCode::new(cause.reason())?,
                EvidenceBasis::CheckedInvariant,
            )?
            .with_fact(ExplainFact::new(
                "strategy",
                FactValue::Identity(crate::explain::SubjectKey::new(strategy)?),
            )?)?;
            assessment = match cause {
                crate::frontier::StrategyDeclineCause::NumericalPermissionRefused { dimension } => {
                    assessment.with_fact(ExplainFact::new(
                        "refused-dimension",
                        FactValue::Identity(crate::explain::SubjectKey::new(dimension)?),
                    )?)?
                }
                crate::frontier::StrategyDeclineCause::NoAdmissibleShape { extent, .. } => {
                    assessment.with_fact(ExplainFact::new("extent", FactValue::Count(extent))?)?
                }
                crate::frontier::StrategyDeclineCause::Unrepresentable { .. } => assessment,
            };
            Ok(explain.push_detail(
                RuleRef::builtin("frontier.strategy-decline.v1")?,
                subjects,
                ExplainEvent::Check {
                    stage,
                    assessment,
                    rejection: if stage == ExplainStage::NumericalLegality {
                        RejectionClass::NumericalIllegal
                    } else {
                        RejectionClass::NotApplicable
                    },
                },
                vec![parent],
            )?)
        })(),
        stage,
        SubjectKind::Schedule,
        &key,
        record_cause(parent),
    )
}

/// Records one exact opaque-call refusal without reclassifying it as cost.
///
/// Both subjects are governed identities. The proposal spelling includes the
/// exact call and ordered bindings and is bounded at construction; provider
/// provenance uses the semantic provider's complete namespace, name, and
/// output-affecting revision rather than the rule provider's presentation key.
fn record_opaque_call_rejection(
    explain: &mut ExplainWriter,
    provider: &crate::frontier::PhysicalProviderProvenance,
    proposal: &crate::call_registry::OpaqueCallProposal,
    rejection: &crate::frontier::OpaqueCallRejectionCause,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let call_key = proposal.subject();
    let provider_key = provider.explain_subject();
    let (rule_key, event) = opaque_call_rejection_event(rejection);
    let stage = event
        .as_ref()
        .map_or(ExplainStage::IntrinsicScheduling, ExplainEvent::stage);
    explain_step(
        (|| -> Result<_, CompileError> {
            let subjects = vec![
                explain.subject(SubjectKind::OpaqueCall, &call_key)?,
                explain.subject(SubjectKind::Provider, provider_key)?,
            ];
            Ok(explain.push_detail(RuleRef::builtin(rule_key?)?, subjects, event?, vec![cause])?)
        })(),
        stage,
        SubjectKind::OpaqueCall,
        &call_key,
        record_cause(cause),
    )
}

/// Maps every typed opaque-call cause to its truthful explain event.
fn opaque_call_rejection_event(
    rejection: &crate::frontier::OpaqueCallRejectionCause,
) -> (
    Result<String, ExplainError>,
    Result<ExplainEvent, CompileError>,
) {
    use crate::frontier::OpaqueCallRejectionCause;

    match rejection {
        OpaqueCallRejectionCause::NotApplicable { target_profile_key } => (
            Ok("opaque-call.applicability.v1".to_owned()),
            (|| {
                Ok(ExplainEvent::Check {
                    stage: ExplainStage::CandidateEnumeration,
                    assessment: PredicateAssessment::disproved(
                        "opaque-call.applicable",
                        ReasonCode::new("opaque-call.not-applicable")?,
                        EvidenceBasis::CheckedInvariant,
                    )?
                    .with_fact(ExplainFact::new(
                        "target-profile",
                        FactValue::Identity(crate::explain::SubjectKey::new(target_profile_key)?),
                    )?)?,
                    rejection: RejectionClass::NotApplicable,
                })
            })(),
        ),
        OpaqueCallRejectionCause::Unregistered => (
            Ok("opaque-call.registration.v1".to_owned()),
            opaque_call_check_event(
                PredicateAssessment::disproved(
                    "opaque-call.registered",
                    ReasonCode::new("opaque-call.unregistered").expect("governed reason"),
                    EvidenceBasis::CheckedInvariant,
                ),
                ExplainStage::CapabilityResolution,
            ),
        ),
        OpaqueCallRejectionCause::MalformedBinding(fault) => (
            Ok("opaque-call.binding.v1".to_owned()),
            opaque_call_check_event(
                binding_assessment(*fault),
                ExplainStage::IntrinsicScheduling,
            ),
        ),
        OpaqueCallRejectionCause::ContractUnderivable(fault) => (
            Ok("opaque-call.contract.v1".to_owned()),
            opaque_call_check_event(
                PredicateAssessment::disproved(
                    "opaque-call.contract-derivable",
                    ReasonCode::new(guarantee_reason(*fault)).expect("governed reason"),
                    EvidenceBasis::CheckedInvariant,
                ),
                ExplainStage::IntrinsicScheduling,
            ),
        ),
        OpaqueCallRejectionCause::NumericalContractMismatch => (
            Ok("opaque-call.numerical-contract.v1".to_owned()),
            opaque_call_check_event(
                PredicateAssessment::disproved(
                    "opaque-call.numerical-contract-matches",
                    ReasonCode::new("opaque-call.numerical-contract-mismatch")
                        .expect("governed reason"),
                    EvidenceBasis::CheckedInvariant,
                ),
                ExplainStage::NumericalLegality,
            ),
        ),
        OpaqueCallRejectionCause::WorkUnresolvable(fault) => {
            let (assessment, stage) = work_resolution_assessment(*fault);
            (
                Ok("opaque-call.work-resolution.v1".to_owned()),
                opaque_call_check_event(assessment, stage),
            )
        }
        OpaqueCallRejectionCause::TargetInfeasible(predicate) => (
            Ok(format!("target.{}", predicate.axis().key())),
            (|| {
                Ok(ExplainEvent::Feasibility {
                    predicate: PredicateKey::new(predicate.axis().key())?,
                    outcome: crate::explain::FeasibilityOutcome::Rejected(ReasonCode::new(
                        "target-infeasible",
                    )?),
                    required: predicate.required(),
                    available: predicate.available(),
                })
            })(),
        ),
        OpaqueCallRejectionCause::TargetUnhonourable(cause) => (
            Ok(format!("target.{}", cause.dimension().key())),
            (|| {
                Ok(ExplainEvent::NumericalHonourability {
                    dimension: PredicateKey::new(cause.dimension().key())?,
                    arithmetic: cause.arithmetic(),
                    resolved_type: cause.resolved_type().clone(),
                    required: ReasonCode::new(cause.required().key())?,
                    outcome: crate::explain::HonourabilityOutcome::Unhonourable {
                        means: ReasonCode::new(cause.means().key())?,
                        honoured: cause
                            .honoured()
                            .map(|honoured| ReasonCode::new(honoured.key()))
                            .transpose()?,
                        // The exact fact the feasibility authority refused on,
                        // carried rather than rebuilt: an explanation that
                        // re-derived it from the means and profile key would
                        // assert provenance no authority supplied.
                        evidence: cause.evidence(),
                    },
                    profile: crate::explain::SubjectKey::new(cause.profile().key())?,
                })
            })(),
        ),
    }
}

fn opaque_call_check_event(
    assessment: Result<PredicateAssessment, ExplainError>,
    stage: ExplainStage,
) -> Result<ExplainEvent, CompileError> {
    Ok(ExplainEvent::Check {
        stage,
        assessment: assessment?,
        rejection: if stage == ExplainStage::NumericalLegality {
            RejectionClass::NumericalIllegal
        } else {
            RejectionClass::IntrinsicInvalid
        },
    })
}

fn binding_assessment(
    fault: crate::call_abi::BindingError,
) -> Result<PredicateAssessment, ExplainError> {
    match fault {
        crate::call_abi::BindingError::UnboundParameter(parameter) => {
            binding_parameter_assessment("opaque-call.binding.unbound-parameter", parameter)
        }
        crate::call_abi::BindingError::UnknownParameter(parameter) => {
            binding_parameter_assessment("opaque-call.binding.unknown-parameter", parameter)
        }
        crate::call_abi::BindingError::ParameterBoundTwice(parameter) => {
            binding_parameter_assessment("opaque-call.binding.parameter-bound-twice", parameter)
        }
        crate::call_abi::BindingError::RoleStorageDisagreement { first, second } => {
            PredicateAssessment::disproved(
                "opaque-call.binding-valid",
                ReasonCode::new("opaque-call.binding.role-storage-disagreement")?,
                EvidenceBasis::CheckedInvariant,
            )?
            .with_fact(parameter_fact("first-parameter", first)?)?
            .with_fact(parameter_fact("second-parameter", second)?)
        }
    }
}

fn binding_parameter_assessment(
    reason: &'static str,
    parameter: &'static str,
) -> Result<PredicateAssessment, ExplainError> {
    PredicateAssessment::disproved(
        "opaque-call.binding-valid",
        ReasonCode::new(reason)?,
        EvidenceBasis::CheckedInvariant,
    )?
    .with_fact(parameter_fact("parameter", parameter)?)
}

fn parameter_fact(key: &'static str, parameter: &'static str) -> Result<ExplainFact, ExplainError> {
    ExplainFact::new(
        key,
        FactValue::Identity(crate::explain::SubjectKey::new(parameter)?),
    )
}

const fn guarantee_reason(fault: crate::call_declaration::GuaranteeError) -> &'static str {
    match fault {
        crate::call_declaration::GuaranteeError::NotAWrite => "opaque-call.contract.not-a-write",
        crate::call_declaration::GuaranteeError::AmbiguousWriteDomain => {
            "opaque-call.contract.ambiguous-write-domain"
        }
    }
}

fn work_resolution_assessment(
    fault: crate::frontier::WorkResolutionError,
) -> (Result<PredicateAssessment, ExplainError>, ExplainStage) {
    match fault {
        crate::frontier::WorkResolutionError::UnknownParameter(parameter) => (
            PredicateAssessment::disproved(
                "opaque-call.work-resolvable",
                ReasonCode::new("opaque-call.work.unknown-parameter").expect("governed reason"),
                EvidenceBasis::CheckedInvariant,
            )
            .and_then(|assessment| assessment.with_fact(parameter_fact("parameter", parameter)?)),
            ExplainStage::CapabilityResolution,
        ),
        crate::frontier::WorkResolutionError::IntermediateShapeUnavailable { parameter } => (
            PredicateAssessment::deferred(
                "opaque-call.work-resolvable",
                ReasonCode::new("opaque-call.work.intermediate-shape-unavailable")
                    .expect("governed reason"),
            )
            .and_then(|assessment| assessment.with_fact(parameter_fact("parameter", parameter)?)),
            ExplainStage::CapabilityResolution,
        ),
    }
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
                    arithmetic: cause.arithmetic(),
                    resolved_type: cause.resolved_type().clone(),
                    required: ReasonCode::new(cause.required().key())?,
                    outcome: crate::explain::HonourabilityOutcome::Unhonourable {
                        means: ReasonCode::new(cause.means().key())?,
                        honoured: cause
                            .honoured()
                            .map(|honoured| ReasonCode::new(honoured.key()))
                            .transpose()?,
                        evidence: cause.evidence(),
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
    alternative: &ProgramAlternative,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    for (entry, scheduled) in alternative.scheduled_regions.iter().enumerate() {
        let region = scheduled.region();
        // Explain observes the exact admission evidence that entered the
        // frontier. Re-assessing here would create a second authority and could
        // either lose its deferred obligations or disagree with the decision
        // whose consequences the rest of this alternative already retains.
        let admitted = scheduled.admission();
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
        if let Some(deferred) = admitted.deferred() {
            let entry = u32::try_from(entry).expect("program stage counts are bounded below u32");
            for predicate in deferred.predicates() {
                cause = explain_step(
                    (|| -> Result<_, CompileError> {
                        let subject = explain.subject(SubjectKind::Region, &key)?;
                        Ok(explain.push_detail(
                            RuleRef::builtin(format!("target.{}", predicate.axis().key()))?,
                            vec![subject],
                            ExplainEvent::DeferredTargetRequirement {
                                entry,
                                predicate: PredicateKey::new(predicate.axis().key())?,
                                required: predicate.required(),
                                requirement: predicate.requirement().clone(),
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
                            arithmetic: honoured.arithmetic(),
                            resolved_type: honoured.resolved_type().clone(),
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
        "index-arithmetic-u64" | "device-memory" => Ok(Quantity::Count(value)),
        "device-address-bits" => Ok(Quantity::Bits(value)),
        _ => Err(ExplainError::UnknownQuantityUnit),
    }
}

/// Records one retained alternative's per-layer admitted evidence.
pub(super) fn record_alternative_explain(
    explain: &mut ExplainWriter,
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
    let target = record_target_admissions(explain, alternative, schedule)?;
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
    use super::{
        CostUnit, ExplainEvent, ExplainStage, FactValue, Quantity, analytical_quantity,
        opaque_call_rejection_event, target_quantity,
    };
    use crate::explain::{ExplainDisposition, ExplainError};
    use crate::frontier::{OpaqueCallRejectionCause, WorkResolutionError};
    use tiler_ir::semantic::{ResolvedValueType, TypeKey};

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

    /// Retired capability axes cannot survive as explain-only quantities.
    #[test]
    fn retired_target_axes_are_not_target_quantities() {
        for retired in ["barriers", "index-bits"] {
            assert_eq!(
                target_quantity(retired, 0),
                Err(ExplainError::UnknownQuantityUnit)
            );
        }
    }

    #[test]
    fn index_arithmetic_and_device_address_width_have_distinct_quantities() {
        assert_eq!(
            target_quantity("index-arithmetic-u64", 1),
            Ok(Quantity::Count(1))
        );
        assert_eq!(
            target_quantity("device-address-bits", 64),
            Ok(Quantity::Bits(64))
        );
    }

    #[test]
    fn every_non_target_opaque_call_cause_has_one_typed_event() {
        let cases = [
            (
                OpaqueCallRejectionCause::NotApplicable {
                    target_profile_key: crate::request::TargetProfileKey::governed(
                        "tiler.target.test",
                    ),
                },
                "opaque-call.applicability.v1",
                ExplainStage::CandidateEnumeration,
                ExplainDisposition::NotApplicable,
            ),
            (
                OpaqueCallRejectionCause::Unregistered,
                "opaque-call.registration.v1",
                ExplainStage::CapabilityResolution,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::MalformedBinding(
                    crate::call_abi::BindingError::UnboundParameter("x"),
                ),
                "opaque-call.binding.v1",
                ExplainStage::IntrinsicScheduling,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::MalformedBinding(
                    crate::call_abi::BindingError::UnknownParameter("unknown"),
                ),
                "opaque-call.binding.v1",
                ExplainStage::IntrinsicScheduling,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::MalformedBinding(
                    crate::call_abi::BindingError::ParameterBoundTwice("x"),
                ),
                "opaque-call.binding.v1",
                ExplainStage::IntrinsicScheduling,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::MalformedBinding(
                    crate::call_abi::BindingError::RoleStorageDisagreement {
                        first: "x",
                        second: "y",
                    },
                ),
                "opaque-call.binding.v1",
                ExplainStage::IntrinsicScheduling,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::ContractUnderivable(
                    crate::call_declaration::GuaranteeError::AmbiguousWriteDomain,
                ),
                "opaque-call.contract.v1",
                ExplainStage::IntrinsicScheduling,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::NumericalContractMismatch,
                "opaque-call.numerical-contract.v1",
                ExplainStage::NumericalLegality,
                ExplainDisposition::RejectedNumerical,
            ),
            (
                OpaqueCallRejectionCause::WorkUnresolvable(WorkResolutionError::UnknownParameter(
                    "count",
                )),
                "opaque-call.work-resolution.v1",
                ExplainStage::CapabilityResolution,
                ExplainDisposition::RejectedIntrinsic,
            ),
            (
                OpaqueCallRejectionCause::WorkUnresolvable(
                    WorkResolutionError::IntermediateShapeUnavailable {
                        parameter: "scratch",
                    },
                ),
                "opaque-call.work-resolution.v1",
                ExplainStage::CapabilityResolution,
                ExplainDisposition::DeferredUnsupported,
            ),
        ];

        for (cause, expected_rule, expected_stage, expected_disposition) in cases {
            let (rule, event) = opaque_call_rejection_event(&cause);
            let event = event.expect("the typed cause maps without loss");
            assert_eq!(rule.unwrap(), expected_rule);
            assert_eq!(event.stage(), expected_stage);
            assert_eq!(event.disposition(), expected_disposition);
            assert!(
                !matches!(event, ExplainEvent::CostAssessment { .. }),
                "a rejection never becomes cost evidence"
            );
        }
    }

    #[test]
    fn binding_and_work_fault_payloads_are_typed_facts() {
        let (_, event) = opaque_call_rejection_event(&OpaqueCallRejectionCause::MalformedBinding(
            crate::call_abi::BindingError::RoleStorageDisagreement {
                first: "left",
                second: "right",
            },
        ));
        let ExplainEvent::Check { assessment, .. } = event.expect("reportable") else {
            panic!("binding fault was not a checked refusal");
        };
        assert_eq!(
            assessment.reason().map(crate::explain::ReasonCode::as_str),
            Some("opaque-call.binding.role-storage-disagreement")
        );
        assert_eq!(assessment.facts().len(), 2);
        assert!(matches!(
            assessment.facts()[0].value(),
            FactValue::Identity(value) if value.as_str() == "left"
        ));
        assert!(matches!(
            assessment.facts()[1].value(),
            FactValue::Identity(value) if value.as_str() == "right"
        ));

        let (_, event) = opaque_call_rejection_event(&OpaqueCallRejectionCause::WorkUnresolvable(
            WorkResolutionError::UnknownParameter("count"),
        ));
        let ExplainEvent::Check { assessment, .. } = event.expect("reportable") else {
            panic!("work fault was not a checked refusal");
        };
        assert_eq!(
            assessment.reason().map(crate::explain::ReasonCode::as_str),
            Some("opaque-call.work.unknown-parameter")
        );
        assert!(matches!(
            assessment.facts()[0].value(),
            FactValue::Identity(value) if value.as_str() == "count"
        ));
    }

    #[test]
    fn target_opaque_call_causes_preserve_the_authority_payloads() {
        use crate::physical::ResourceVerdict;
        use crate::request::{StrictF32NumericalContract, TargetProfile};
        use crate::target::feasibility::RejectionCause;
        use crate::target::honourability::{
            DeclaredBehaviour, DimensionBehaviour, HonouringMeans, NumericalDimension,
            UnhonouredDimension, governed_profile_source,
        };
        use tiler_ir::schedule::{ArithmeticType, NumericalPermission, ResourceRequirements};

        let realization = StrictF32NumericalContract::governed().realization();
        let capability = match crate::physical::assess_resources(
            ResourceRequirements {
                buffer_bindings: u32::MAX,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                requires_device_memory: true,
                input_subnormals: realization.input_subnormals,
                result_subnormals: realization.result_subnormals,
                contraction: realization.contraction,
                reassociation: realization.reassociation,
                permutation: realization.permutation,
                signed_zero: realization.signed_zero,
                nan_assumptions: realization.nan_assumptions,
                infinity_assumptions: realization.infinity_assumptions,
            },
            ArithmeticType::F32,
            1,
            &TargetProfile::governed(),
        )
        .expect_err("the binding requirement exceeds the governed profile")
        {
            ResourceVerdict::Rejected(RejectionCause::Capability(predicate)) => predicate,
            other => panic!("expected a typed capability refusal, got {other:?}"),
        };
        let (_, event) =
            opaque_call_rejection_event(&OpaqueCallRejectionCause::TargetInfeasible(capability));
        let event = event.expect("reportable");
        assert!(matches!(
            &event,
            ExplainEvent::Feasibility {
                required: Quantity::Bindings(required),
                available: Quantity::Bindings(available),
                ..
            } if *required == u64::from(u32::MAX) && *available == 4
        ));

        let required = DimensionBehaviour::Transform(NumericalPermission::Permitted);
        let unhonourable = UnhonouredDimension::new(
            DeclaredBehaviour::new(
                NumericalDimension::Contraction,
                ArithmeticType::F16,
                ResolvedValueType::nominal(TypeKey::new("test", "f16", 1).unwrap()),
                required,
                HonouringMeans::Unsupported,
                governed_profile_source(),
            )
            .attributed_to(crate::target::feasibility::TargetProfileIdentity::new(
                "tiler.test.profile.v1",
            )),
            required,
            Some(DimensionBehaviour::Transform(
                NumericalPermission::Forbidden,
            )),
        );
        let carried = unhonourable.evidence();
        let (_, event) = opaque_call_rejection_event(
            &OpaqueCallRejectionCause::TargetUnhonourable(unhonourable),
        );
        let event = event.expect("reportable");
        assert!(matches!(
            &event,
            ExplainEvent::NumericalHonourability {
                dimension,
                arithmetic: ArithmeticType::F16,
                required,
                outcome: crate::explain::HonourabilityOutcome::Unhonourable {
                    means,
                    honoured: Some(honoured),
                    ..
                },
                profile,
                ..
            } if dimension.as_str() == "numerics.contraction"
                && required.as_str() == "permitted"
                && means.as_str() == "unsupported"
                && honoured.as_str() == "forbidden"
                && profile.as_str() == "tiler.test.profile.v1"
        ));
        // The refusing fact reached the explain event itself, not a copy the
        // event rebuilt from the means and the profile key.
        let ExplainEvent::NumericalHonourability {
            outcome: crate::explain::HonourabilityOutcome::Unhonourable { evidence, .. },
            ..
        } = &event
        else {
            panic!("an unhonourable opaque-call cause records an unhonourable outcome");
        };
        assert!(evidence.cites_same_fact(&carried));
    }
}
