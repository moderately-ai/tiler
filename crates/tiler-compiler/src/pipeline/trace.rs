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
use crate::physical::{AccessMode, AccessOrdinal};

/// Records the bounded cover enumeration, what it pruned, its budget stops, and
/// its infeasibility.
///
/// The two pruning channels stay separate because a reader acts differently on
/// them. A [`crate::cover::CoverRefusal`] is a *hard legality* answer about a
/// candidate the search reached — recorded as a disproved check, so its
/// disposition is a rejection — while a dominated cover is legal and merely
/// beaten, recorded as a cost assessment whose disposition says so. Collapsing
/// the two would tell a reader that a legal alternative was refused, or that a
/// refused one might be recovered by changing a cost.
pub(super) fn record_cover_enumeration(
    explain: &mut ExplainWriter,
    enumeration: &CoverEnumeration,
    root: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut cause = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Candidate, "region-cover")?;
            let assessment = PredicateAssessment::proven(
                "cover.complete-and-legal",
                EvidenceBasis::CheckedInvariant,
            )?
            .with_fact(ExplainFact::new(
                "cover-count",
                FactValue::Count(u64::try_from(enumeration.covers().len()).unwrap_or(u64::MAX)),
            )?)?
            .with_fact(ExplainFact::new(
                "cover-policy",
                FactValue::Identity(crate::explain::SubjectKey::new(enumeration.policy().key())?),
            )?)?
            // The statement that turns a budget-stopped enumeration into an
            // explainable *partial* result rather than a truncated one presented
            // as complete. It is emitted on every compile, not only when a
            // budget fires, because a reader must be able to see the claim was
            // made rather than infer it from a record's absence.
            .with_fact(ExplainFact::new(
                "search-exhaustive",
                FactValue::Boolean(enumeration.is_exhaustive()),
            )?)?;
            Ok(explain.push_detail(
                RuleRef::builtin("cover.enumeration.v1")?,
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::CandidateEnumeration,
                    assessment,
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                vec![root],
            )?)
        })(),
        ExplainStage::CandidateEnumeration,
        SubjectKind::Candidate,
        "region-cover",
        record_cause(root),
    )?;
    cause = record_cover_refusals(explain, enumeration, cause)?;
    cause = record_dominated_covers(explain, enumeration, cause)?;
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

/// Names every candidate the partition search refused, with its hard reason.
///
/// A refusal is a legality answer, so it is a disproved check and its
/// disposition is a rejection. The subject is the refused candidate itself —
/// the region occurrence that would have duplicated or been left unobserved, or
/// the named output with two producers — so the record names *what* was pruned
/// rather than only that something was.
fn record_cover_refusals(
    explain: &mut ExplainWriter,
    enumeration: &CoverEnumeration,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    for refusal in enumeration.refusals() {
        let key = refusal.subject_label();
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Candidate, &key)?;
                Ok(explain.push_detail(
                    RuleRef::builtin("cover.enumeration.v1")?,
                    vec![subject],
                    ExplainEvent::Check {
                        stage: ExplainStage::CandidateEnumeration,
                        assessment: PredicateAssessment::disproved(
                            "cover.complete-and-legal",
                            ReasonCode::new(refusal.reason())?,
                            EvidenceBasis::CheckedInvariant,
                        )?,
                        rejection: RejectionClass::IntrinsicInvalid,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::Candidate,
            &key,
            record_cause(cause),
        )?;
    }
    Ok(cause)
}

/// Names every legal cover another cover's estimate beat, and the one that beat
/// it.
///
/// Deliberately a cost assessment rather than a check: these covers are legal
/// and completely derived, and nothing about them was disproved. The record
/// carries both subjects — the pruned cover first, the dominating cover second —
/// so a reader does not have to re-derive the pairing from four numbers.
fn record_dominated_covers(
    explain: &mut ExplainWriter,
    enumeration: &CoverEnumeration,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    for (pruned, dominator) in enumeration.dominated() {
        let key = pruned.identity().label();
        let dominator_key = dominator.identity().label();
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let cost = pruned.cost();
                let subject = explain.subject(SubjectKind::Candidate, &key)?;
                let by = explain.subject(SubjectKind::Candidate, &dominator_key)?;
                Ok(explain.push_detail(
                    RuleRef::builtin(cost.model_key())?,
                    vec![subject, by],
                    ExplainEvent::CostAssessment {
                        model: CostModelKey::new(cost.model_key())?,
                        basis: EvidenceBasis::CheckedInvariant,
                        terms: vec![
                            CostTerm::new("region-count", Quantity::Count(cost.region_count()))?,
                            CostTerm::new(
                                "materialization-count",
                                Quantity::Count(cost.materialization_count()),
                            )?,
                            CostTerm::new(
                                "materialized-elements",
                                Quantity::Count(cost.materialized_elements()),
                            )?,
                            CostTerm::new(
                                "recomputed-elements",
                                Quantity::Operations(cost.recomputed_elements()),
                            )?,
                        ],
                        disposition: CostDisposition::Dominated,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::Costing,
            SubjectKind::Candidate,
            &key,
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
            let provider = ProviderRef::registered(capabilities.provider());
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
                let provider = ProviderRef::lowering(occurrence.provider());
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
                    let provider = ProviderRef::lowering(occurrence.provider());
                    let rule = RuleRef::provided(
                        "kernel.index-region-refinement.v1",
                        occurrence.provider().capability_revision().get(),
                        provider,
                    )?;
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    let assessment = PredicateAssessment::proven(
                        "kernel.index-region-refines-occurrence",
                        EvidenceBasis::ExhaustiveFinite,
                    )?
                    .with_fact(ExplainFact::new(
                        "refinement-identity",
                        FactValue::Identity(crate::explain::SubjectKey::new(identity)?),
                    )?)?;
                    let assessment = with_realization_facts(assessment, refinement.realization())?;
                    Ok(explain.push_detail(
                        rule,
                        vec![subject],
                        ExplainEvent::Check {
                            stage: ExplainStage::KernelRefinement,
                            assessment,
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
                    tiler_ir::index::IndexDomainProofEvidence::ExhaustiveFinite {
                        points, ..
                    } => (
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
                        proof.authority().rule().namespace(),
                        proof.authority().rule().name(),
                    ))?),
                )?)?
                .with_fact(ExplainFact::new(
                    "discharge-revision",
                    FactValue::Count(u64::from(proof.authority().revision())),
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

/// Names every region of an ordered realization and every value handed between
/// them.
///
/// **Emitted only for a chain, and that is the whole design.** A one-stage
/// realization's single region is already named by `refinement-identity`, whose
/// bytes *are* that region's for a one-stage sequence, so restating it would add
/// a fact carrying no information to every record every compilation writes. A
/// chain is the case where a reader can otherwise only infer the shape from the
/// dispatch count, which is exactly the inference this record exists to remove:
/// the stage population, each stage's own region, and each handed value's
/// producer, consumer, and element count are stated.
///
/// The per-stage keys are ordinal-suffixed rather than repeated, because a
/// reader resolving "which region is stage one" must not have to depend on the
/// order two same-named facts happen to be rendered in.
pub(super) fn with_realization_facts(
    assessment: PredicateAssessment,
    realization: &tiler_ir::index::VerifiedIndexRegionSequence,
) -> Result<PredicateAssessment, CompileError> {
    if realization.is_single_stage() {
        return Ok(assessment);
    }
    let mut assessment = assessment.with_fact(ExplainFact::new(
        "realization-stages",
        FactValue::Count(u64::try_from(realization.stage_count()).unwrap_or(u64::MAX)),
    )?)?;
    for (ordinal, stage) in realization.stages().enumerate() {
        assessment = assessment.with_fact(ExplainFact::new(
            format!("realization-stage-{ordinal}-region"),
            FactValue::Identity(crate::explain::SubjectKey::new(identity_label(
                "region",
                stage.canonical_identity().as_bytes(),
            ))?),
        )?)?;
    }
    for (ordinal, intermediate) in realization.intermediates().iter().enumerate() {
        assessment = assessment
            .with_fact(ExplainFact::new(
                format!("realization-intermediate-{ordinal}-producer"),
                FactValue::Count(u64::try_from(intermediate.producer()).unwrap_or(u64::MAX)),
            )?)?
            .with_fact(ExplainFact::new(
                format!("realization-intermediate-{ordinal}-consumer"),
                FactValue::Count(u64::try_from(intermediate.consumer()).unwrap_or(u64::MAX)),
            )?)?
            .with_fact(ExplainFact::new(
                format!("realization-intermediate-{ordinal}-elements"),
                FactValue::Count(
                    intermediate
                        .shape()
                        .element_count()
                        .and_then(|count| u64::try_from(count).ok())
                        .unwrap_or(u64::MAX),
                ),
            )?)?;
    }
    Ok(assessment)
}

/// Returns the stable presentation label of one refinement occurrence identity.
///
/// The label is a presentation handle over the identity's trailing bytes, never
/// the identity itself: the canonical bytes stay in the retained
/// [`crate::legality::IndexRefinement`], which is what any downstream check
/// compares.
pub(super) fn refinement_label(refinement: &crate::legality::IndexRefinement) -> String {
    identity_label("refinement", refinement.identity().as_bytes())
}

/// Renders one canonical identity's trailing bytes as a presentation handle.
///
/// A handle, never the identity: the canonical bytes stay in the retained
/// evidence, which is what any downstream check compares.
fn identity_label(kind: &str, bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let tail = bytes.len().saturating_sub(8);
    let mut label = format!("{kind}:");
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
            // The fold of the output this candidate implements, resolved from
            // the candidate's own occurrences rather than from "the" recognized
            // reduction: a program declaring several outputs has one partition
            // per output, and the proof forbids the reassociation of *this*
            // region's fold.
            let reduction = verified
                .output_for_region(candidate.members())
                .and_then(|(_, output)| output.try_serial_sum())
                .map(|serial| serial.members.reduction().to_vec())
                .ok_or_else(|| {
                    CompileError::from(ProgramError::Structure {
                        rule: "reduction-provider-missing",
                    })
                })?;
            let provider = lowering
                .occurrences()
                .iter()
                .find(|lowering| {
                    reduction
                        .iter()
                        .any(|atom| atom.member() == lowering.member())
                })
                .map(crate::lowering::OccurrenceLowering::provider)
                .ok_or_else(|| {
                    CompileError::from(ProgramError::Structure {
                        rule: "reduction-provider-missing",
                    })
                })?;
            let provider_ref = ProviderRef::lowering(provider);
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
///
/// **`key` is the region's own explain subject, not its role.** The role is a
/// four-valued presentation label and a cover places many regions under
/// `unrecognized`, so a role-keyed record merges regions covering different
/// occurrences into one subject a reader cannot disentangle. The caller passes
/// the bounded label of the region's canonical occurrence identity, which
/// region formation already proved pairwise distinct within one compilation, so
/// the record names exactly the region it is about. The role travels beside it
/// as a fact, which is what it was always good for.
pub(super) fn record_frontier(
    explain: &mut ExplainWriter,
    key: &str,
    role: &'static str,
    frontier: &crate::frontier::ImplementationFrontier,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut cause = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Schedule, key)?;
            Ok(explain.push_detail(
                RuleRef::builtin("frontier.enumeration.v1")?,
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::IntrinsicScheduling,
                    assessment: PredicateAssessment::proven(
                        "frontier.locally-feasible",
                        EvidenceBasis::CheckedInvariant,
                    )?
                    .with_fact(ExplainFact::new(
                        "admitted-count",
                        FactValue::Count(
                            u64::try_from(frontier.admitted().len()).unwrap_or(u64::MAX),
                        ),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "region-role",
                        FactValue::Identity(crate::explain::SubjectKey::new(role)?),
                    )?)?,
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                vec![cause],
            )?)
        })(),
        ExplainStage::IntrinsicScheduling,
        SubjectKind::Schedule,
        key,
        record_cause(cause),
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
                    record_declined_strategy(explain, key, provider, strategy, *decline, cause)?;
            }
            crate::frontier::FrontierRejection::Infeasible { .. }
            | crate::frontier::FrontierRejection::Unhonourable { .. }
            | crate::frontier::FrontierRejection::Unsynchronizable { .. }
            | crate::frontier::FrontierRejection::SynchronizationUndeclared { .. }
            | crate::frontier::FrontierRejection::UnrealizableSubgroup { .. }
            | crate::frontier::FrontierRejection::SubgroupUndeclared { .. }
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
        key,
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
/// The disposition is `Disproved` and never `Unknown`: every cause is decided
/// before any region is built — the first three from the request alone and the
/// fourth from the exact occurrences the cover grouped — so nothing further
/// could resolve it.
fn record_declined_strategy(
    explain: &mut ExplainWriter,
    key: &str,
    provider: &crate::frontier::PhysicalProviderProvenance,
    strategy: &'static str,
    cause: crate::frontier::StrategyDeclineCause,
    parent: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let stage = match cause {
        crate::frontier::StrategyDeclineCause::NumericalPermissionRefused { .. } => {
            ExplainStage::NumericalLegality
        }
        // The algebraic half of ADR 0014's two-fact rule: the failing source is
        // the operation's own declared capability, so it reports under
        // capability resolution rather than under the caller's numerical
        // contract — the two sources are never collapsed into one verdict.
        crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported { .. } => {
            ExplainStage::CapabilityResolution
        }
        crate::frontier::StrategyDeclineCause::NoAdmissibleShape { .. }
        | crate::frontier::StrategyDeclineCause::Unrepresentable { .. }
        // A missing region spelling is a scheduling-vocabulary fact, exactly as
        // an inadmissible shape is: the caller's numerical contract permits
        // this region, and nothing about the target refused it.
        | crate::frontier::StrategyDeclineCause::UnspellableRegion { .. }
        | crate::frontier::StrategyDeclineCause::TargetPolicyUndeclared { .. } => {
            ExplainStage::IntrinsicScheduling
        }
    };
    explain_step(
        (|| -> Result<_, CompileError> {
            let subjects = vec![
                explain.subject(SubjectKind::Schedule, key)?,
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
                crate::frontier::StrategyDeclineCause::AlgebraicCapabilityUnsupported {
                    dimension,
                } => assessment.with_fact(ExplainFact::new(
                    "withheld-dimension",
                    FactValue::Identity(crate::explain::SubjectKey::new(dimension)?),
                )?)?,
                crate::frontier::StrategyDeclineCause::NoAdmissibleShape { extent, .. } => {
                    assessment.with_fact(ExplainFact::new("extent", FactValue::Count(extent))?)?
                }
                crate::frontier::StrategyDeclineCause::Unrepresentable { .. }
                | crate::frontier::StrategyDeclineCause::TargetPolicyUndeclared { .. } => {
                    assessment
                }
                crate::frontier::StrategyDeclineCause::UnspellableRegion { covered, .. } => {
                    assessment.with_fact(ExplainFact::new(
                        "covered-occurrences",
                        FactValue::Count(u64::from(covered)),
                    )?)?
                }
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
                        RejectionClass::IntrinsicInvalid
                    },
                },
                vec![parent],
            )?)
        })(),
        stage,
        SubjectKind::Schedule,
        key,
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
                        means: ReasonCode::new(cause.means().label())?,
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
        crate::call_abi::BindingError::AccessOutOfRange { parameter, access } => {
            binding_parameter_assessment("opaque-call.binding.access-out-of-range", parameter)?
                .with_fact(access_fact(access)?)
        }
        crate::call_abi::BindingError::InOutRegionUnsupported { parameter, access } => {
            binding_parameter_assessment("opaque-call.binding.inout-region-unsupported", parameter)?
                .with_fact(access_fact(access)?)
        }
        crate::call_abi::BindingError::AccessModeMismatch {
            parameter,
            access,
            parameter_role,
            access_mode,
        } => binding_parameter_assessment("opaque-call.binding.access-mode-mismatch", parameter)?
            .with_fact(access_fact(access)?)?
            .with_fact(parameter_fact("parameter-role", parameter_role.key())?)?
            .with_fact(parameter_fact(
                "access-mode",
                match access_mode {
                    AccessMode::Read => "read",
                    AccessMode::Write => "write",
                },
            )?),
        crate::call_abi::BindingError::UnboundAccess(access) => PredicateAssessment::disproved(
            "opaque-call.binding-valid",
            ReasonCode::new("opaque-call.binding.unbound-access")?,
            EvidenceBasis::CheckedInvariant,
        )?
        .with_fact(access_fact(access)?),
        crate::call_abi::BindingError::AccessStorageDisagreement {
            access,
            first,
            second,
        } => PredicateAssessment::disproved(
            "opaque-call.binding-valid",
            ReasonCode::new("opaque-call.binding.access-storage-disagreement")?,
            EvidenceBasis::CheckedInvariant,
        )?
        .with_fact(access_fact(access)?)?
        .with_fact(parameter_fact("first-parameter", first)?)?
        .with_fact(parameter_fact("second-parameter", second)?),
    }
}

fn access_fact(access: AccessOrdinal) -> Result<ExplainFact, ExplainError> {
    ExplainFact::new("access", FactValue::Count(u64::from(access.get())))
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
    frontier_records: &[(&str, ExplainRecordId)],
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
    record_coverage_gaps(explain, portfolio, frontier_records, cause)?;
    Ok(cause)
}

/// Publishes each coverage gap: a region no implementation covered.
///
/// **The only authority that states the gap, and it used to be read by tests
/// alone.** `select_physical_plans` has always constructed a
/// [`crate::selection::PlanRejection::RegionUnimplemented`] for every cover
/// region with an empty admitted set, and nothing on the compile path ever
/// emitted one — so a caller reading the trace saw a cover that produced no plan
/// and no record saying which of its regions had nothing to implement it.
///
/// **One record per unimplemented region, carrying the count of covers it
/// blocked — not one record per (cover, region) pair.** The distinct rejection
/// grounds this stage decides are the regions: whether a region has an admitted
/// implementation follows from that region's own frontier, and `planning.rs`
/// enumerates and records each frontier exactly once "however many covers place
/// that same region". A per-cover record set restated one ground under a
/// different opaque `region-cover:` digest — a subject no other record in the
/// trace names, so a reader cannot resolve it to a partition — up to
/// `region_covers` times. `docs/compiler/optimizer.md` requires that explain
/// output "never collapses these into 'not fused'", and nothing is collapsed
/// here: every distinct stage, reason code, rule, provider, affected region,
/// predicate, evidence class, and disposition still has its own record. What
/// the `blocked-covers` fact replaces is a repetition of one identical tuple,
/// and it replaces it with the only thing that repetition said.
///
/// Each record is caused by *that region's own frontier record* rather than by
/// the selection chain, because the frontier enumeration is where the answer
/// was decided: following the cause reaches the declined strategy naming the
/// region-vocabulary wall. A region whose frontier record is not among
/// `frontier_records` cites the selection chain instead, which happens only for
/// a portfolio assembled outside the planning loop.
///
/// These are leaves rather than links in the returned chain: a coverage gap
/// explains covers that produced nothing, so hanging the retained
/// alternatives' attribution off it would say the plans that *were* built
/// followed from it.
///
/// The stage is `CandidateEnumeration` — the one `record_plan_selection`'s own
/// plan-count and budget-stop records already carry — and not `Selection`,
/// because the explain vocabulary deliberately admits no `Check` at that stage:
/// selection owns a typed event carrying an outcome a checked predicate cannot,
/// and a `Check` there would silently lose it. What is recorded here is why a
/// candidate cover contributed no complete plan, which is a fact about the
/// enumeration rather than about which plan was chosen.
fn record_coverage_gaps(
    explain: &mut ExplainWriter,
    portfolio: &SelectedPortfolio,
    frontier_records: &[(&str, ExplainRecordId)],
    fallback: ExplainRecordId,
) -> Result<(), TargetFailure> {
    for rejection in portfolio.rejections() {
        let crate::selection::PlanRejection::RegionUnimplemented {
            region,
            role,
            covers,
        } = rejection
        else {
            continue;
        };
        let cause = frontier_records
            .iter()
            .find(|(key, _)| key == region)
            .map_or(fallback, |(_, record)| *record);
        explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Schedule, region)?;
                Ok(explain.push_detail(
                    RuleRef::builtin("selection.region-coverage.v1")?,
                    vec![subject],
                    ExplainEvent::Check {
                        stage: ExplainStage::CandidateEnumeration,
                        assessment: PredicateAssessment::disproved(
                            "selection.region-implemented",
                            ReasonCode::new("region-unimplemented")?,
                            EvidenceBasis::CheckedInvariant,
                        )?
                        .with_fact(ExplainFact::new(
                            "region-role",
                            FactValue::Identity(crate::explain::SubjectKey::new(*role)?),
                        )?)?
                        .with_fact(ExplainFact::new(
                            "blocked-covers",
                            FactValue::Count(*covers),
                        )?)?,
                        rejection: RejectionClass::IntrinsicInvalid,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::Schedule,
            region,
            record_cause(cause),
        )?;
    }
    Ok(())
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
    key: &str,
    cause: ExplainRecordId,
) -> Result<TerminalCause, TargetFailure> {
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
                        means: ReasonCode::new(cause.means().label())?,
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
        PhysicalError::Synchronization { cause, .. } => (
            format!("target.synchronization.{}", cause.subject().kind.key()),
            (|| -> Result<_, CompileError> {
                Ok(synchronization_event(
                    cause.subject(),
                    crate::explain::SynchronizationOutcome::Unrealizable {
                        profile: crate::explain::SubjectKey::new(
                            cause.fact().provenance().profile().key(),
                        )?,
                    },
                )?)
            })(),
        ),
        // The undeclared outcome the explain vocabulary already carries, not a
        // second refusal spelling: nothing declared this subject, so there is no
        // profile to attribute the answer to, and naming a neighbouring one
        // would invite a reader to compose facts none of which is about it.
        PhysicalError::UnrealizedSynchronization { subject, .. } => (
            format!("target.synchronization.{}", subject.kind.key()),
            (|| -> Result<_, CompileError> {
                Ok(synchronization_event(
                    *subject,
                    crate::explain::SynchronizationOutcome::Undeclared,
                )?)
            })(),
        ),
        PhysicalError::Subgroup { cause, .. } => (
            format!("target.subgroup.{}", cause.subject().transfer().key()),
            (|| -> Result<_, CompileError> {
                Ok(subgroup_event(
                    cause.subject(),
                    crate::explain::SynchronizationOutcome::Unrealizable {
                        profile: crate::explain::SubjectKey::new(
                            cause.fact().provenance().profile().key(),
                        )?,
                    },
                )?)
            })(),
        ),
        PhysicalError::UnrealizedSubgroup { subject, .. } => (
            format!("target.subgroup.{}", subject.transfer().key()),
            (|| -> Result<_, CompileError> {
                Ok(subgroup_event(
                    *subject,
                    crate::explain::SynchronizationOutcome::Undeclared,
                )?)
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
            let subject = explain.subject(SubjectKind::Region, key)?;
            Ok(explain.push_causal_detail(
                RuleRef::builtin(rule_key)?,
                subject,
                &event?,
                vec![cause],
            )?)
        })(),
        ExplainStage::TargetFeasibility,
        SubjectKind::Region,
        key,
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
                        // Exhaustive over the typed deferred subject, so a new
                        // proof shape stops this producer at compile time
                        // instead of rendering under a borrowed spelling. The
                        // capability arm keeps the exact record it always
                        // wrote; the subgroup arm carries its complete atomic
                        // subject, never a width disguised as an axis.
                        use crate::target::feasibility::ExecutableDeferredTargetSubject;
                        let (rule, event) = match predicate.subject() {
                            ExecutableDeferredTargetSubject::CapabilityAxis(axis) => (
                                format!("target.{}", axis.key()),
                                ExplainEvent::DeferredTargetRequirement {
                                    entry,
                                    predicate: PredicateKey::new(axis.key())?,
                                    required: axis.quantity(predicate.requirement().required()),
                                    requirement: predicate.requirement().clone(),
                                },
                            ),
                            ExecutableDeferredTargetSubject::SubgroupWidthConfirmation(subject) => {
                                (
                                    "target.subgroup-width-confirmation".to_owned(),
                                    ExplainEvent::DeferredSubgroupWidthConfirmation {
                                        entry,
                                        width: subject.width().get(),
                                        arithmetic: subject.arithmetic(),
                                        transfer: ReasonCode::new(subject.transfer().key())?,
                                        requirement: predicate.requirement().clone(),
                                    },
                                )
                            }
                        };
                        let subject = explain.subject(SubjectKind::Region, &key)?;
                        Ok(explain.push_detail(
                            RuleRef::builtin(rule)?,
                            vec![subject],
                            event,
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
                                means: ReasonCode::new(honoured.means().label())?,
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
        // One record for the one atomic realization, and **no record at all**
        // when the region requires none. That absence is the canonical state, not
        // an omission: a zero-synchronization program consulted no target fact,
        // so a row saying so would be a check that never ran.
        if let Some(realized) = admitted.synchronization() {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Region, &key)?;
                    let event = synchronization_event(
                        realized.subject(),
                        crate::explain::SynchronizationOutcome::Realized {
                            profile: crate::explain::SubjectKey::new(
                                realized.fact().provenance().profile().key(),
                            )?,
                        },
                    )?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(format!(
                            "target.synchronization.{}",
                            realized.subject().kind.key()
                        ))?,
                        vec![subject],
                        event,
                        vec![cause],
                    )?)
                })(),
                ExplainStage::TargetFeasibility,
                SubjectKind::Region,
                &key,
                record_cause(cause),
            )?;
        }
        // The one atomic subgroup realization, on the synchronization block's
        // terms: one record for the whole subject, and no record at all when
        // the region requires none. On the compile path this admission never
        // stands alone — the deferred loop above already recorded the
        // prepared-width confirmation the same assessment minted.
        if let Some(realized) = admitted.subgroup() {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Region, &key)?;
                    let event = subgroup_event(
                        realized.subject(),
                        crate::explain::SynchronizationOutcome::Realized {
                            profile: crate::explain::SubjectKey::new(
                                realized.fact().provenance().profile().key(),
                            )?,
                        },
                    )?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(format!(
                            "target.subgroup.{}",
                            realized.subject().transfer().key()
                        ))?,
                        vec![subject],
                        event,
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

/// Builds the one explain record for a complete synchronization subject.
///
/// One helper for both the admitted and the refused path, so the rendered
/// subject cannot differ between them — a reader comparing a refusal against a
/// later admission is comparing the same six fields spelled the same way.
fn subgroup_event(
    subject: tiler_ir::schedule::SubgroupRealizationSubject,
    outcome: crate::explain::SynchronizationOutcome,
) -> Result<ExplainEvent, ExplainError> {
    Ok(ExplainEvent::SubgroupRealization {
        width: subject.width().get(),
        arithmetic: subject.arithmetic(),
        transfer: ReasonCode::new(subject.transfer().key())?,
        outcome,
    })
}

fn synchronization_event(
    subject: tiler_ir::schedule::SynchronizationSubject,
    outcome: crate::explain::SynchronizationOutcome,
) -> Result<ExplainEvent, ExplainError> {
    Ok(ExplainEvent::SynchronizationRealization {
        kind: ReasonCode::new(subject.kind.key())?,
        execution_scope: ReasonCode::new(subject.execution_scope.key())?,
        visibility_scope: ReasonCode::new(subject.visibility_scope.key())?,
        fences_workgroup: subject.fenced_spaces.workgroup,
        fences_device: subject.fenced_spaces.device,
        ordering: ReasonCode::new(subject.ordering.key())?,
        outcome,
    })
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
                        ProviderRef::registered(&GovernedPhysicalProvider::identity()),
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
    // The ADR 0013 IR witness verdict, reported through the existing
    // detail-event shape so the explain schema and renderer stay put. The
    // verdict is derived here from the verified program itself — never a
    // producer claim — and a refusal is a disproved check rather than a
    // compilation failure: the plan compiles, and its artifact simply cannot
    // claim plan determinism. Deliberately no envelope digest and no delivery
    // position: neither is known before publication, and printing a
    // not-yet-known value would be a fabricated fact.
    let key = format!("{}/program", alternative.stable_id);
    let determinism = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::KernelProgram, &key)?;
            let assessment =
                match tiler_ir::kernel::verify_plan_determinism(alternative.program.core()) {
                    Ok(_) => PredicateAssessment::proven(
                        "program.plan-determinism-witness",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    Err(refusal) => PredicateAssessment::disproved(
                        "program.plan-determinism-witness",
                        ReasonCode::new(plan_determinism_reason(&refusal))?,
                        EvidenceBasis::CheckedInvariant,
                    )?,
                };
            Ok(explain.push_detail(
                RuleRef::builtin("program.plan-determinism.v1")?,
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::ProgramVerification,
                    assessment,
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                vec![program],
            )?)
        })(),
        ExplainStage::ProgramVerification,
        SubjectKind::KernelProgram,
        &key,
        record_cause(program),
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
        determinism,
    )
}

/// Returns the governed reason code of one plan-determinism refusal.
///
/// A wildcard is required across the crate boundary — the refusal vocabulary
/// is `#[non_exhaustive]` — and classifies a later-admitted class as
/// unclassified rather than inventing a name for it.
fn plan_determinism_reason(refusal: &tiler_ir::kernel::PlanDeterminismRefusal) -> &'static str {
    use tiler_ir::kernel::PlanDeterminismRefusal;
    match refusal {
        PlanDeterminismRefusal::UnfixedContributorArrival { .. } => {
            "plan-determinism.unfixed-contributor-arrival"
        }
        PlanDeterminismRefusal::OutputAffectingAtomic { .. } => {
            "plan-determinism.output-affecting-atomic"
        }
        PlanDeterminismRefusal::RuntimeDependentSelection { .. } => {
            "plan-determinism.runtime-dependent-selection"
        }
        PlanDeterminismRefusal::UnverifiedOpaqueStage { .. } => {
            "plan-determinism.unverified-opaque-stage"
        }
        _ => "plan-determinism.unclassified",
    }
}

pub(super) fn record_cost_and_selection(
    alternatives: &[ProgramAlternative],
    selected_alternative_id: &str,
    causes: &[(String, ExplainRecordId)],
    profile: &crate::request::TargetProfile,
    explain: &mut ExplainWriter,
) -> Result<(), TargetFailure> {
    // Scored once for the whole portfolio rather than per alternative, because a
    // partial scoring decides nothing: `measured_scores` refuses the comparison
    // unless every candidate states a work span, and the record below must report
    // the same set the selector compared.
    let scores = measured_scores(alternatives, profile);
    for alternative in alternatives {
        let cost = alternative.structural_cost;
        let cause = causes
            .iter()
            .find_map(|(id, cause)| (*id == alternative.stable_id).then_some(*cause));
        let (subject, structural_record) = explain_step(
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
                let record = explain.push_detail(
                    RuleRef::builtin(STRUCTURAL_COST_MODEL_KEY)?,
                    vec![subject.clone()],
                    ExplainEvent::CostAssessment {
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
        // The measured record, when the profile declares the row this selector
        // reads. **It names the deciding term and both sides of the `max`**, not
        // merely that the alternative was selected: `work-steps` is the total
        // fold work, `span-steps` is the critical path already scaled by the
        // declared row so the two are directly comparable, and `fold-steps` is the
        // per-stage maximum of the two summed. A reader can see which side
        // decided, which is the whole content of the model.
        //
        // `EvidenceBasis::Assumption` rather than `CheckedInvariant`: the work and
        // span counts are exact, but the row that combines them is a fitted
        // quantity determined to about a factor of four, so the total is a
        // modelled preference. `PredicateAssessment` refuses that basis by
        // construction, which is why this is a cost record and not a check.
        let alternative_score = scores.as_deref().and_then(|portfolio_scores| {
            portfolio_scores
                .iter()
                .find(|score| score.alternative.stable_id == alternative.stable_id)
        });
        let measured_record = match alternative_score {
            None => None,
            Some(score) => {
                let assessment = score.assessment;
                let disposition = if alternative.stable_id == selected_alternative_id {
                    CostDisposition::Retained
                } else {
                    CostDisposition::HigherCost
                };
                Some(explain_step(
                    (|| -> Result<_, CompileError> {
                        let subject = explain
                            .subject(SubjectKind::Alternative, alternative.stable_id.as_str())?;
                        Ok(explain.push_detail(
                            RuleRef::builtin(MEASURED_FOLD_STEP_MODEL_KEY)?,
                            vec![subject],
                            ExplainEvent::CostAssessment {
                                model: CostModelKey::new(MEASURED_FOLD_STEP_MODEL_KEY)?,
                                basis: EvidenceBasis::Assumption,
                                terms: vec![
                                    CostTerm::new(
                                        "saturated-parallel-fold-steps",
                                        Quantity::Operations(
                                            assessment.saturated_parallel_fold_steps,
                                        ),
                                    )?,
                                    CostTerm::new(
                                        "work-steps",
                                        Quantity::Operations(assessment.work_steps),
                                    )?,
                                    CostTerm::new(
                                        "span-steps",
                                        Quantity::Operations(assessment.span_steps),
                                    )?,
                                    CostTerm::new(
                                        "fold-steps",
                                        Quantity::Operations(assessment.fold_steps),
                                    )?,
                                ],
                                disposition,
                            },
                            vec![structural_record],
                        )?)
                    })(),
                    ExplainStage::Costing,
                    SubjectKind::Alternative,
                    alternative.stable_id.as_str(),
                    record_cause(structural_record),
                )?)
            }
        };
        // The selection row cites the measured record where one exists, so the
        // reason the winner won is on the causal path from the verdict rather than
        // beside it.
        let cost_record = TerminalCause::from_record(measured_record.unwrap_or(structural_record));
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
        CompilationRequest, CostUnit, ExplainEvent, ExplainStage, FactValue, Quantity,
        analytical_quantity, compile, opaque_call_rejection_event, target_quantity,
    };
    use crate::explain::{ExplainDisposition, ExplainError};
    use crate::frontier::{OpaqueCallRejectionCause, WorkResolutionError};
    use crate::physical::{AccessOrdinal, PhysicalError};
    use crate::pipeline::{CompileError, NoFeasiblePlanError};
    use crate::request::{StrictF32NumericalContract, TargetProfile};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ResolvedValueType,
        SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum, TypeKey,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn reassociating_reduction(contributors: u64) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(
                InputKey::new("input").unwrap(),
                Shape::from_dims([1, contributors]),
            )
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn flush_and_reassociate_request(program: &SemanticProgram) -> CompilationRequest<'_> {
        let mut request = CompilationRequest::governed_under(
            program,
            StrictF32NumericalContract::governed_flush_and_reassociate(),
        );
        request.target_profiles = vec![TargetProfile::flush_only_for_test(
            "tiler.target.flush-and-reassociate-decline-test.v1",
        )];
        request
    }

    /// An unsplittable extent is an ordinary intrinsic strategy decline, not a
    /// candidate-applicability verdict and not malformed compiler output.
    #[test]
    fn no_admissible_partition_is_retained_and_does_not_abort_compilation() {
        let program = reassociating_reduction(2);
        let product = compile(flush_and_reassociate_request(&program))
            .expect("the composed contract reaches physical enumeration");
        let compiled = product.targets[0]
            .compiled()
            .expect("the serial alternative survives both parallel declines");
        // Restricted to the two *reduction* strategies by the name each decline
        // carries. Every region a cover places that this vocabulary cannot
        // spell also declines the serial baseline, and those declines are the
        // point of the region-general provider — counting them here would make
        // this test's number a fact about the program's cover space instead of
        // about the two parallel strategies it is asserting on.
        let parallel = |record: &&crate::explain::ExplainRecord| {
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                return false;
            };
            assessment.facts().iter().any(|fact| {
                fact.key().as_str() == "strategy"
                    && matches!(
                        fact.value(),
                        FactValue::Identity(key)
                            if key.as_str() == crate::physical::MULTI_PASS_SPLIT_STRATEGY
                                || key.as_str()
                                    == crate::physical::SINGLE_WORKGROUP_TREE_STRATEGY
                    )
            })
        };
        let declines = compiled
            .explain
            .records()
            .iter()
            .filter(|record| {
                record.rule().key().as_str() == "frontier.strategy-decline.v1"
                    && record.event().disposition() == ExplainDisposition::RejectedIntrinsic
            })
            .filter(parallel)
            .collect::<Vec<_>>();
        assert_eq!(
            declines.len(),
            2,
            "both unsplittable strategies are explained"
        );
        assert!(declines.iter().any(|record| {
            matches!(
                record.event(),
                ExplainEvent::Check { assessment, .. }
                    if assessment.reason().is_some_and(|reason| {
                        reason.as_str() == "no-admissible-partition"
                    })
            )
        }));
        assert!(declines.iter().all(|record| {
            matches!(
                record.event(),
                ExplainEvent::Check { assessment, .. }
                    if assessment.reason().is_some_and(|reason| {
                        reason.as_str() == "no-admissible-partition"
                            || reason.as_str() == "no-admissible-participant-count"
                            || reason.as_str() == "qualified-width-policy-undeclared"
                    })
            )
        }));
    }

    /// The decline record must not mask the next honest result: a prime extent
    /// above the target's grid bound has no feasible serial fallback.
    #[test]
    fn prime_extent_reaches_the_grid_axis_feasibility_refusal() {
        let program = reassociating_reduction(5);
        let product = compile(flush_and_reassociate_request(&program))
            .expect("a target-local refusal remains an ordered outcome");
        let CompileError::Explained { source, explain } = product.targets[0]
            .failure()
            .expect("five threads exceed the profile's four-thread grid bound")
        else {
            panic!("the target-local refusal retains its explanation")
        };
        assert!(matches!(
            source.as_ref(),
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target {
                rule: "grid-axis",
                required: 5,
                available: 4,
                ..
            }))
        ));
        assert!(explain.records().iter().any(|record| {
            matches!(
                record.event(),
                ExplainEvent::Feasibility {
                    predicate,
                    required: Quantity::Threads(5),
                    available: Quantity::Threads(4),
                    ..
                } if predicate.as_str() == "grid-axis"
            )
        }));
    }

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
                    crate::call_abi::BindingError::AccessStorageDisagreement {
                        access: AccessOrdinal::FIRST,
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
            crate::call_abi::BindingError::AccessStorageDisagreement {
                access: AccessOrdinal::FIRST,
                first: "left",
                second: "right",
            },
        ));
        let ExplainEvent::Check { assessment, .. } = event.expect("reportable") else {
            panic!("binding fault was not a checked refusal");
        };
        assert_eq!(
            assessment.reason().map(crate::explain::ReasonCode::as_str),
            Some("opaque-call.binding.access-storage-disagreement")
        );
        assert_eq!(assessment.facts().len(), 3);
        assert!(matches!(
            assessment.facts()[1].value(),
            FactValue::Identity(value) if value.as_str() == "left"
        ));
        assert!(matches!(
            assessment.facts()[2].value(),
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
        use tiler_ir::schedule::{
            ArithmeticType, NumericalPermission, RegionNumericalRequirements, ResourceRequirements,
        };

        let realization = StrictF32NumericalContract::governed().realization();
        let capability = match crate::physical::assess_resources(
            ResourceRequirements {
                buffer_bindings: u32::MAX,
                threads_per_workgroup: 1,
                local_memory_bytes: 0,
                requires_device_memory: true,
                index_arithmetic: tiler_ir::schedule::IndexArithmetic::CompleteU64,
                synchronization: None,
                subgroup: None,
                numerical: RegionNumericalRequirements::FloatingPoint {
                    input_subnormals: realization.input_subnormals,
                    result_subnormals: realization.result_subnormals,
                    contraction: realization.contraction,
                    reassociation: realization.reassociation,
                    permutation: realization.permutation,
                    signed_zero: realization.signed_zero,
                    reciprocal_transform: realization.reciprocal_transform,
                    approximate_intrinsics: realization.approximate_intrinsics,
                    nan_assumptions: realization.nan_assumptions,
                    infinity_assumptions: realization.infinity_assumptions,
                },
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
