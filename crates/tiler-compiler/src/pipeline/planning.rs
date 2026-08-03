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
    physical: &PhysicalAuthorities<'_>,
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
    let lowering = match resolve_lowering(semantic, verified) {
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
    //
    // Gated on the fused region being *spellable*, not merely on the program
    // being a reduction. The proof asserts `MultiplyThenAdd` as the fused
    // region's atomic operations, which is a positive claim about a region that
    // exists only for the affine prologue this vocabulary can fuse; a general
    // prologue has no fused region, and recording the claim anyway would put a
    // sound-proof label on a statement about nothing.
    let mut numerical = None;
    let mut numerical_cause = legality_cause;
    if crate::physical::fused_prologue_constants(verified).is_some()
        && let Some(candidate) = formation.whole_program_candidate()
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
    // the physical authorities, and only the subject varies here — so a repeat
    // is a re-derivation of a value already in hand. It repeats a lot: the governed
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
                    enumerate_frontier(verified, &subject, physical.providers(), physical.calls())
                        .map_err(|source| {
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
                                cause: cause.clone(),
                            })
                        }
                        crate::frontier::FrontierRejection::Unsynchronizable { cause, .. } => {
                            Some(PhysicalError::Synchronization {
                                region: region_id_of(cover, region),
                                cause: cause.clone(),
                            })
                        }
                        // A target verdict like the three above — the proposal
                        // was made and was not admitted — so it is attributed to
                        // this region rather than left to the frontier record.
                        // What separates it from `Unsynchronizable` is that the
                        // verdict rests on the absence of a fact, and the record
                        // it produces says so.
                        crate::frontier::FrontierRejection::SynchronizationUndeclared {
                            subject,
                            ..
                        } => Some(PhysicalError::UnrealizedSynchronization {
                            region: region_id_of(cover, region),
                            subject: *subject,
                        }),
                        // A reserved body variant, an unregistered opaque call,
                        // and an inapplicable proposal are not target verdicts
                        // and carry no rejection to attribute to this region.
                        // An unregistered call is a provider naming something
                        // that does not exist, which is the provider's fault
                        // rather than this target's limitation.
                        // An opaque refusal retains the exact call proposal and
                        // its own typed cause. Even target-derived causes belong
                        // to that call rather than to a scheduled region, so the
                        // frontier record owns their attribution.
                        // A declined strategy is likewise not a target verdict:
                        // nothing was proposed, so there is no candidate this
                        // region could be reported as having refused. Its typed
                        // reason is recorded by `record_frontier`, which is the
                        // authority for what the enumeration withheld.
                        crate::frontier::FrontierRejection::OpaqueCall { .. }
                        | crate::frontier::FrontierRejection::StrategyDeclined { .. }
                        | crate::frontier::FrontierRejection::UnsupportedVariant { .. }
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

/// Records every exact claim made by the named semantic-discharge stage.
fn record_semantic_discharge_refusal(
    explain: &mut ExplainWriter,
    source: &LoweringError,
    refusal: &IndexDomainDischargeRefusal,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    use std::fmt::Write as _;

    let key = format!("occurrence:{}", source.member().0);
    for (ordinal, discharge) in refusal.assessments().iter().enumerate() {
        let obligation = discharge.obligation();
        let (verifier_reason, verifier_resource) = match obligation.reason() {
            tiler_ir::index::IndexDomainUnknownReason::InsufficientFacts => {
                ("index-domain-insufficient-facts", None)
            }
            tiler_ir::index::IndexDomainUnknownReason::UnsupportedFragment => {
                ("index-domain-unsupported-fragment", None)
            }
            tiler_ir::index::IndexDomainUnknownReason::ResourceLimit {
                resource,
                required,
                limit,
            } => (
                "index-domain-proof-resource-limit",
                Some((resource, required, limit)),
            ),
        };
        let predicate_kind = match obligation.predicate() {
            tiler_ir::index::IndexDomainPredicate::NonNegative { .. } => {
                "index-domain.non-negative"
            }
            tiler_ir::index::IndexDomainPredicate::LessThanExtent { .. } => {
                "index-domain.less-than-extent"
            }
        };
        let mut obligation_key = String::from("obligation:");
        for byte in obligation.canonical_local_key().as_bytes() {
            write!(obligation_key, "{byte:02x}").expect("writing to a String cannot fail");
        }
        let ordinal = u64::try_from(ordinal).expect("index-region obligations are host bounded");
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Kernel, &key)?;
                let (mut assessment, proof_basis, discharge_resource) = match discharge.claim() {
                    IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::Sound {
                        ..
                    }) => (
                        PredicateAssessment::proven(
                            format!("kernel.index-domain-obligation.{ordinal}"),
                            EvidenceBasis::CheckedInvariant,
                        )?,
                        Some("sound-proof"),
                        None,
                    ),
                    IndexDomainDischargeClaim::Proved(
                        IndexDomainDischargeProof::ExhaustiveFinite { .. },
                    ) => (
                        PredicateAssessment::proven(
                            format!("kernel.index-domain-obligation.{ordinal}"),
                            EvidenceBasis::ExhaustiveFinite,
                        )?,
                        Some("exhaustive-finite"),
                        None,
                    ),
                    IndexDomainDischargeClaim::Disproved(disproof) => (
                        PredicateAssessment::disproved(
                            format!("kernel.index-domain-obligation.{ordinal}"),
                            ReasonCode::new(disproof.reason())?,
                            EvidenceBasis::CheckedInvariant,
                        )?,
                        Some("semantic-counterexample"),
                        None,
                    ),
                    IndexDomainDischargeClaim::Unknown(reason) => {
                        let (reason, resource) = match reason {
                            tiler_ir::index::IndexDomainUnknownReason::InsufficientFacts => {
                                ("index-domain-insufficient-facts", None)
                            }
                            tiler_ir::index::IndexDomainUnknownReason::UnsupportedFragment => {
                                ("index-domain-unsupported-fragment", None)
                            }
                            tiler_ir::index::IndexDomainUnknownReason::ResourceLimit {
                                resource,
                                required,
                                limit,
                            } => (
                                "index-domain-proof-resource-limit",
                                Some((*resource, *required, *limit)),
                            ),
                        };
                        (
                            PredicateAssessment::unknown(
                                format!("kernel.index-domain-obligation.{ordinal}"),
                                ReasonCode::new(reason)?,
                            )?,
                            None,
                            resource,
                        )
                    }
                };
                assessment = assessment
                    .with_fact(ExplainFact::new(
                        "obligation-ordinal",
                        FactValue::Count(ordinal),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "obligation-key",
                        FactValue::Identity(crate::explain::SubjectKey::new(&obligation_key)?),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "predicate-kind",
                        FactValue::Identity(crate::explain::SubjectKey::new(predicate_kind)?),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "verifier-unknown-reason",
                        FactValue::Identity(crate::explain::SubjectKey::new(verifier_reason)?),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "discharge-provider",
                        FactValue::Identity(crate::explain::SubjectKey::new(format!(
                            "{}.{}",
                            discharge.authority().provider().namespace(),
                            discharge.authority().provider().name(),
                        ))?),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "discharge-rule",
                        FactValue::Identity(crate::explain::SubjectKey::new(format!(
                            "{}.{}",
                            discharge.authority().rule().identity().namespace(),
                            discharge.authority().rule().identity().name(),
                        ))?),
                    )?)?
                    .with_fact(ExplainFact::new(
                        "discharge-revision",
                        FactValue::Count(u64::from(discharge.authority().revision().get())),
                    )?)?;
                if let Some(proof_basis) = proof_basis {
                    assessment = assessment.with_fact(ExplainFact::new(
                        "evidence-basis",
                        FactValue::Identity(crate::explain::SubjectKey::new(proof_basis)?),
                    )?)?;
                }
                if let IndexDomainDischargeClaim::Disproved(disproof) = discharge.claim()
                    && let Some(point_ordinal) = disproof.point_ordinal()
                {
                    assessment = assessment.with_fact(ExplainFact::new(
                        "counterexample-point-ordinal",
                        FactValue::Count(point_ordinal),
                    )?)?;
                }
                if let Some((resource, required, limit)) = verifier_resource {
                    let resource = match resource {
                        tiler_ir::index::ProofResource::Cells => "index-proof-cells",
                        tiler_ir::index::ProofResource::IntegerBytes => "index-proof-integer-bytes",
                    };
                    assessment = assessment
                        .with_fact(ExplainFact::new(
                            "verifier-proof-resource",
                            FactValue::Identity(crate::explain::SubjectKey::new(resource)?),
                        )?)?
                        .with_fact(ExplainFact::new(
                            "verifier-proof-required-upper-64",
                            FactValue::Count(
                                u64::try_from(required >> 64)
                                    .expect("the upper half of u128 fits u64"),
                            ),
                        )?)?
                        .with_fact(ExplainFact::new(
                            "verifier-proof-required-lower-64",
                            FactValue::Count(
                                u64::try_from(required & u128::from(u64::MAX))
                                    .expect("the lower half of u128 fits u64"),
                            ),
                        )?)?
                        .with_fact(ExplainFact::new(
                            "verifier-proof-limit",
                            FactValue::Count(limit),
                        )?)?;
                }
                if let Some((resource, required, limit)) = discharge_resource {
                    let resource = match resource {
                        tiler_ir::index::ProofResource::Cells => "index-proof-cells",
                        tiler_ir::index::ProofResource::IntegerBytes => "index-proof-integer-bytes",
                    };
                    assessment = assessment
                        .with_fact(ExplainFact::new(
                            "proof-resource",
                            FactValue::Identity(crate::explain::SubjectKey::new(resource)?),
                        )?)?
                        .with_fact(ExplainFact::new(
                            "proof-required-upper-64",
                            FactValue::Count(
                                u64::try_from(required >> 64)
                                    .expect("the upper half of u128 fits u64"),
                            ),
                        )?)?
                        .with_fact(ExplainFact::new(
                            "proof-required-lower-64",
                            FactValue::Count(
                                u64::try_from(required & u128::from(u64::MAX))
                                    .expect("the lower half of u128 fits u64"),
                            ),
                        )?)?
                        .with_fact(ExplainFact::new("proof-limit", FactValue::Count(limit))?)?;
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
            &key,
            record_cause(cause),
        )?;
    }
    Ok(cause)
}

pub(super) fn record_lowering_failure(
    explain: &mut ExplainWriter,
    source: &LoweringError,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    if let Some((_, refusal)) = source.semantic_discharge() {
        return record_semantic_discharge_refusal(explain, source, refusal, cause);
    }
    let key = format!("occurrence:{}", source.member().0);
    let (stage, subject_kind) = match source {
        LoweringError::Refine { .. } => (ExplainStage::KernelRefinement, SubjectKind::Kernel),
        LoweringError::SemanticDischarge { .. } => {
            (ExplainStage::SemanticDischarge, SubjectKind::Kernel)
        }
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
                            ExplainStage::SemanticDischarge => {
                                "kernel.index-domain-obligations-discharged"
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
/// refinement refusals belong to [`ExplainStage::KernelRefinement`], while
/// residual semantic claims belong to [`ExplainStage::SemanticDischarge`].
pub(super) fn lowering_failure(source: &LoweringError, cause: ExplainRecordId) -> TargetFailure {
    let stage = match source {
        LoweringError::Refine { .. } => ExplainStage::KernelRefinement,
        LoweringError::SemanticDischarge { .. } => ExplainStage::SemanticDischarge,
        LoweringError::Occurrence { .. } | LoweringError::Resolve { .. } => {
            ExplainStage::CapabilityResolution
        }
    };
    let phase = match source {
        LoweringError::SemanticDischarge { .. } => "semantic-discharge",
        _ => "lowering",
    };
    let error = if matches!(
        source,
        LoweringError::SemanticDischarge { refusal, .. }
            if semantic_discharge_is_invalid(refusal.kind())
    ) {
        CompileError::InvalidCompilerOutput(CompilerOutputError::Lowering(source.clone()))
    } else {
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase,
            rule: source.reason(),
        })
    };
    target_failure(
        error,
        stage,
        format!("lowering-{}", source.reason()),
        SubjectKind::Capability,
        format!("occurrence:{}", source.member().0),
        record_cause(cause),
    )
}

/// Returns whether a discharge outcome proves the emitted lowering invalid.
const fn semantic_discharge_is_invalid(
    kind: crate::index_discharge::IndexDomainDischargeRefusalKind,
) -> bool {
    match kind {
        crate::index_discharge::IndexDomainDischargeRefusalKind::Disproved => true,
        crate::index_discharge::IndexDomainDischargeRefusalKind::Unknown => false,
    }
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
#[cfg(test)]
pub(super) fn build_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    plan: &SelectedPlan,
    kind: ProgramAlternativeKind,
    plans: &CompletePlans,
    cause: Option<&TerminalCause>,
) -> Result<ProgramAlternative, TargetFailure> {
    build_alternative_for_origin(
        semantic,
        verified,
        SemanticAlternativeOwner {
            origin: SemanticAlternativeOrigin::Baseline,
            key: "semantic:baseline",
        },
        plan,
        kind,
        plans,
        cause,
    )
}

pub(super) fn build_alternative_for_origin(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    owner: SemanticAlternativeOwner<'_>,
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
    // A plan containing an opaque call has no scheduled region for it, and
    // lowering one is not implemented. Refusing here is what stops the plan
    // being lowered as though the call were not in it.
    let scheduled = plan_regions(plan).ok_or_else(|| {
        failure_at_source(
            CompileError::from(ProgramError::Structure {
                rule: "unlowerable-opaque-body",
            }),
            ExplainStage::ProgramVerification,
            cause.copied(),
        )
    })?;
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
    let identity = ProgramAlternativeIdentity::new(owner.origin, semantic, verified, plan);
    Ok(ProgramAlternative {
        stable_id: identity.label(),
        identity,
        owner_key: owner.key.to_owned(),
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
/// Returns `None` when any selection has no scheduled region.
///
/// **Filtering here would be silently wrong**, and became so the moment opaque
/// calls were admittable. A plan of one scheduled region and one opaque call
/// filters to a single region, which `build_plan_program` then matches as a
/// *fused* program — producing a kernel that omits the call's work entirely and
/// reports success. Nothing downstream compares the region count against the
/// selection count, so the omission would surface as a wrong result rather than
/// an error.
///
/// Lowering an opaque call is genuinely not implemented; declining to order a
/// plan containing one is how that is said, and the caller turns it into a typed
/// refusal.
///
/// A **subprogram contributes every one of its stages**, not one region standing
/// for it. Taking the first would be the same silent omission the paragraph
/// above describes, one dispatch instead of one region: the assembled program
/// would compute a partial reduction and publish it as the result.
pub(super) fn plan_region_order(plan: &SelectedPlan) -> Option<Vec<&VerifiedScheduledRegion>> {
    let mut regions: Vec<&VerifiedScheduledRegion> = Vec::with_capacity(plan.selections().len());
    for selection in plan.selections() {
        regions.extend(selection.implementation().scheduled_stages()?);
    }
    regions.sort_by_key(|region| region.region().index.id.get());
    Some(regions)
}

/// Returns an owned copy of a plan's scheduled regions in planning order.
pub(super) fn plan_regions(plan: &SelectedPlan) -> Option<Vec<VerifiedScheduledRegion>> {
    Some(plan_region_order(plan)?.into_iter().cloned().collect())
}

/// Assembles the verified kernel program for one plan shape.
///
/// The bounded profile implements exactly three program shapes: a one-region
/// fused program, a two-region materialized program, and the three-region split
/// whose reduction is realized by a partial and a final pass. Any other retained
/// plan shape is invalid compiler output and rejects explicitly rather than
/// being approximated by the closest implemented assembly.
///
/// The split shares the materialized *kind* because a kind is a property of the
/// cover — the split refines one of that cover's regions into two dispatches and
/// changes no grouping — so the region count is what selects the assembly. A
/// separate kind would have to be derivable from the cover, and it is not.
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
        (ProgramAlternativeKind::Materialized, [_, _, _]) => {
            build_split_kernel_program(semantic, verified, scheduled).map_err(CompileError::from)
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

#[cfg(test)]
mod tests {
    use super::semantic_discharge_is_invalid;
    use crate::index_discharge::IndexDomainDischargeRefusalKind;

    #[test]
    fn only_disproved_semantic_discharge_is_invalid_compiler_output() {
        assert!(semantic_discharge_is_invalid(
            IndexDomainDischargeRefusalKind::Disproved
        ));
        assert!(!semantic_discharge_is_invalid(
            IndexDomainDischargeRefusalKind::Unknown
        ));
    }
}
