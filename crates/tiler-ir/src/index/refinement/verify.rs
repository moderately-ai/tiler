//! The checking pass itself.
//!
//! This is where a resolved law, an admitted lowering authority, and a
//! candidate realization are compared. The order is deliberate and stated in
//! the method documentation: the two authorities must describe the same
//! occurrence before the candidate is read at all, then the candidate's effect,
//! per-stage scalar containment, ordered interfaces, governed numerical
//! contract, and finally its exact canonical realization identity. Nothing here
//! approximates semantic equivalence — an alternate spelling is refused — and
//! nothing mints a receipt while a residual index-domain obligation remains.
//!
//! Completion is the second half: it assesses every retained obligation exactly
//! once against one shared ledger and either mints the receipt with its proofs
//! or refuses atomically, retaining every assessment for explanation.

use std::sync::Arc;

use crate::index::{VerifiedIndexRegion, VerifiedIndexRegionSequence};
use crate::semantic::OperationEffect;

use super::MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS;
use super::authority::IndexRealizationAuthority;
use super::binding::{bind_operands, bind_results};
use super::error::IndexRefinementVerificationError;
use super::finite::{IndexDomainProofLedger, assess_finite_domains_with};
use super::identity::encode_proof_identity;
use super::proof::{
    IndexDomainProofAssessment, IndexDomainProofAuthority, IndexDomainProofBudget,
    IndexDomainProofClaim, IndexDomainProofRefusal, IndexDomainProofRefusalKind,
    IndexRefinementDomainProof, IndexRefinementDomainProofIdentity,
};
use super::receipt::{
    IndexRefinementReceipt, IndexRefinementVerificationOutcome, PendingIndexRefinementReceipt,
    mint_receipt,
};
use super::registry::ResolvedIndexRealization;
use super::subject::IndexRefinementSubject;

impl ResolvedIndexRealization {
    /// Checks the occurrence and region together, minting no receipt while a
    /// residual index-domain obligation remains.
    ///
    /// The candidate must be the exact canonical region constructed by the
    /// registered semantic law. Semantic equivalence is not approximated here:
    /// an alternate logical spelling is refused and may become a physical
    /// alternative only after this semantic association is established.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`IndexRefinementVerificationError::SemanticRealizationLawRefused`] under
    /// `staged-law-requires-region-sequence` when the registered law realizes a
    /// region *sequence*, which one region cannot satisfy — [`Self::verify_sequence`]
    /// is the method for those. This is the first thing checked, before anything
    /// looks at `region` or at `lowering`. Otherwise this method is
    /// [`Self::verify_sequence`] over a one-stage sequence and returns exactly
    /// what that method returns.
    pub fn verify(
        &self,
        lowering: &IndexRealizationAuthority,
        region: &VerifiedIndexRegion,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        if self.law.realizes_region_sequence() {
            // Named before anything else looks at the region. A staged law's
            // final stage reads a value no occurrence input carries, so the
            // ordinary interface check would refuse a lone region by naming that
            // boundary — a true statement that sends a reader to the provider's
            // tensor list instead of to the arity of what it was asked for.
            return Err(
                IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(self.subject.operation().clone()),
                    rule: "staged-law-requires-region-sequence",
                },
            );
        }
        self.verify_sequence(
            lowering,
            &VerifiedIndexRegionSequence::single(region.clone()),
        )
    }

    /// Checks the occurrence against an ordered multi-region realization.
    ///
    /// The candidate must be the exact canonical region *sequence* the
    /// registered law constructs: the stages in order, each stage's own
    /// canonical region identity, and the source every stage input is bound to.
    /// A truncated chain, a reordered one, and one whose stages are individually
    /// correct but wired differently each render a different sequence identity
    /// and are refused for that reason alone.
    ///
    /// [`Self::verify`] is this method over a one-stage sequence, and a one-stage
    /// sequence's identity is its region's identity, so the two paths agree
    /// byte for byte on everything single-region verification ever accepted.
    ///
    /// # Errors
    ///
    /// Checked in body order. First, that `lowering` and this resolution
    /// describe the same occurrence: a disagreeing operation, attribute set,
    /// numerical contract, graph or occurrence, capability signature, or
    /// projected semantic authority is refused before the candidate is read at
    /// all. Then a typed refusal when effect, scalar authority, ordered tensor
    /// interfaces, the governed numerical contract, or the realized sequence
    /// disagree.
    ///
    /// The sequence comparison reports region identities when both the
    /// expectation and the candidate are single-stage, and the whole-sequence
    /// identities otherwise.
    pub fn verify_sequence(
        &self,
        lowering: &IndexRealizationAuthority,
        realization: &VerifiedIndexRegionSequence,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        let subject = &self.subject;
        check_lowering_authority(subject, self, lowering)?;
        if subject.effect != OperationEffect::Pure {
            return Err(IndexRefinementVerificationError::EffectNotIndexable {
                effect: subject.effect,
            });
        }
        // Per stage, so the containment check covers the union of everything the
        // realization reaches. A stage reaching an unadmitted scalar operation
        // refuses the whole realization: the admission is what the emitted
        // program is checked against, and a chain is one program.
        let stage_authority = |stage: &VerifiedIndexRegion| {
            let evidence = self
                .registry
                .0
                .scalars
                .revalidate_region(stage)
                .map_err(|source| {
                    IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
                })?;
            if evidence.scalar_snapshot() != self.registry.0.scalars.snapshot_identity() {
                return Err(IndexRefinementVerificationError::ScalarSnapshotMismatch);
            }
            if evidence
                .reached_operations()
                .iter()
                .any(|reached| !lowering.emitted_scalar_operations.contains(reached))
            {
                return Err(IndexRefinementVerificationError::ScalarAuthorityConformance);
            }
            Ok(evidence)
        };
        let mut leading_scalar_authorities = Vec::with_capacity(realization.leading_stages().len());
        for stage in realization.leading_stages() {
            leading_scalar_authorities.push(stage_authority(stage)?);
        }
        let scalar_authority = stage_authority(realization.final_stage())?;
        let mut scalar_authorities = leading_scalar_authorities.clone();
        scalar_authorities.push(scalar_authority.clone());
        let operand_bindings = bind_operands(subject, realization)?;
        let result_bindings = bind_results(subject, realization.final_stage())?;
        if !self.law.accepts_numerical_contract(subject) {
            return Err(IndexRefinementVerificationError::NumericalContractNotGoverned);
        }
        let expected = self
            .law
            .realize_sequence(subject, &self.registry.0.scalars)
            .map_err(
                |source| IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(subject.operation().clone()),
                    rule: source.rule(),
                },
            )?;
        if expected.identity() != realization.identity() {
            // A one-stage expectation against a one-stage candidate reports the
            // region identities the single-region refusal has always reported;
            // anything else reports the whole chain, because naming one stage of
            // a mismatched chain would hide which part disagreed.
            return Err(
                if expected.is_single_stage() && realization.is_single_stage() {
                    IndexRefinementVerificationError::SemanticRealizationMismatch {
                        expected: expected.final_stage().canonical_identity().clone(),
                        actual: realization.final_stage().canonical_identity().clone(),
                    }
                } else {
                    IndexRefinementVerificationError::SemanticRealizationSequenceMismatch {
                        expected: expected.identity().clone(),
                        actual: realization.identity().clone(),
                    }
                },
            );
        }
        let residual_obligations = realization
            .stages()
            .map(|stage| stage.unknown_index_domain_predicates().len())
            .try_fold(0_usize, usize::checked_add)
            .unwrap_or(usize::MAX);
        check_residual_obligation_count(residual_obligations)?;
        if residual_obligations != 0 {
            return Ok(IndexRefinementVerificationOutcome::Pending(Box::new(
                PendingIndexRefinementReceipt {
                    resolution: self.clone(),
                    leading_scalar_authorities,
                    scalar_authority,
                    operand_bindings,
                    result_bindings,
                    realization: realization.clone(),
                },
            )));
        }
        Ok(IndexRefinementVerificationOutcome::Verified(Box::new(
            mint_receipt(
                subject,
                self,
                realization,
                scalar_authorities,
                operand_bindings,
                result_bindings,
                Vec::new(),
            ),
        )))
    }

    /// Assesses every retained obligation exactly once and mints the receipt.
    ///
    /// A disproved or unknown obligation consumes no pending state and mints no
    /// receipt. The caller retains its clone if it needs diagnostics or retry.
    ///
    /// # Errors
    ///
    /// Returns an atomic refusal retaining every canonical assessment when any
    /// obligation is disproved or unsupported.
    pub fn complete(
        pending: &PendingIndexRefinementReceipt,
        budget: IndexDomainProofBudget,
    ) -> Result<(IndexRefinementReceipt, Vec<IndexDomainProofAssessment>), IndexDomainProofRefusal>
    {
        let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
        // One ledger for the whole realization: the caller funded one budget,
        // and a per-stage ledger would silently multiply it by the stage count.
        let mut ledger = IndexDomainProofLedger::new(budget);
        // Each retained obligation stays paired with the region its handles
        // resolve against, so nothing downstream re-derives that association.
        let mut owners: Vec<(usize, &VerifiedIndexRegion)> = Vec::new();
        let mut assessments = Vec::new();
        for (stage, region) in pending.realization.stages().enumerate() {
            let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
            let claims = assess_finite_domains_with(region, &obligations, &mut ledger);
            for (obligation, claim) in obligations.into_iter().zip(claims) {
                owners.push((stage, region));
                assessments.push(IndexDomainProofAssessment {
                    obligation,
                    authority: authority.clone(),
                    claim,
                });
            }
        }
        let assessments = retain_complete_assessments(assessments)?;
        let mut proofs = Vec::with_capacity(assessments.len());
        for (assessment, (stage, region)) in assessments.iter().zip(&owners) {
            let IndexDomainProofClaim::Proved(proof) = &assessment.claim else {
                unreachable!("the refusal scans removed every non-proof claim")
            };
            proofs.push(IndexRefinementDomainProof {
                stage: *stage,
                obligation: assessment.obligation,
                authority: assessment.authority.clone(),
                proof: proof.clone(),
                identity: IndexRefinementDomainProofIdentity(
                    encode_proof_identity(
                        region,
                        assessment.obligation,
                        &assessment.authority,
                        proof,
                    )
                    .into_boxed_slice(),
                ),
            });
        }
        Ok((
            mint_receipt(
                pending.resolution.subject(),
                &pending.resolution,
                &pending.realization,
                pending.scalar_authorities(),
                pending.operand_bindings.clone(),
                pending.result_bindings.clone(),
                proofs,
            ),
            assessments,
        ))
    }
}

pub(super) fn retain_complete_assessments(
    assessments: Vec<IndexDomainProofAssessment>,
) -> Result<Vec<IndexDomainProofAssessment>, IndexDomainProofRefusal> {
    let kind = if assessments
        .iter()
        .any(|assessment| matches!(assessment.claim, IndexDomainProofClaim::Disproved(_)))
    {
        Some(IndexDomainProofRefusalKind::Disproved)
    } else if assessments
        .iter()
        .any(|assessment| matches!(assessment.claim, IndexDomainProofClaim::Unknown(_)))
    {
        Some(IndexDomainProofRefusalKind::Unknown)
    } else {
        None
    };
    match kind {
        Some(kind) => Err(IndexDomainProofRefusal { assessments, kind }),
        None => Ok(assessments),
    }
}

pub(super) fn check_residual_obligation_count(
    actual: usize,
) -> Result<(), IndexRefinementVerificationError> {
    if actual > MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS {
        return Err(
            IndexRefinementVerificationError::ResidualObligationsTooLarge {
                actual,
                limit: MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS,
            },
        );
    }
    Ok(())
}

pub(super) fn check_lowering_authority(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    lowering: &IndexRealizationAuthority,
) -> Result<(), IndexRefinementVerificationError> {
    let resolved = &resolution.subject;
    if subject.operation != resolved.operation {
        return Err(IndexRefinementVerificationError::OperationMismatch);
    }
    if subject.attributes != resolved.attributes {
        return Err(IndexRefinementVerificationError::AttributeMismatch);
    }
    if subject.numerical_contract != resolved.numerical_contract {
        return Err(IndexRefinementVerificationError::NumericalContractMismatch);
    }
    if subject.graph != resolved.graph || subject.occurrence != resolved.occurrence {
        return Err(IndexRefinementVerificationError::OccurrenceMismatch);
    }
    if subject.signature != resolved.signature
        || subject.effect != resolved.effect
        || subject.identity != resolved.identity
    {
        return Err(IndexRefinementVerificationError::CapabilitySignatureMismatch);
    }
    if lowering.operation != subject.operation {
        return Err(IndexRefinementVerificationError::OperationMismatch);
    }
    if lowering.signature != subject.signature {
        return Err(IndexRefinementVerificationError::CapabilitySignatureMismatch);
    }
    let lowering_occurrence = lowering
        .semantic_registry
        .project_operation_occurrence_authority(
            &subject.operation,
            subject.signature.operands.iter(),
            subject.signature.results.iter(),
            &subject.attributes,
        )
        .map_err(|source| IndexRefinementVerificationError::SemanticAuthority(Arc::new(source)))?;
    if lowering_occurrence != subject.semantic_authority
        || lowering.semantic_registry.snapshot_identity()
            != resolution.registry.0.semantic.snapshot_identity()
    {
        return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
    }
    if lowering.realization_law_row != subject.realization_law_row {
        return Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch);
    }
    if lowering.scalar_registry.snapshot_identity()
        != resolution.registry.0.scalars.snapshot_identity()
    {
        return Err(IndexRefinementVerificationError::ScalarSnapshotMismatch);
    }
    Ok(())
}
