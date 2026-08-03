//! Compiler policy for IR-owned discharge of residual logical index-domain predicates.
//!
//! A structurally verified region may retain exact `Unknown` predicates without
//! becoming executable refinement evidence. This stage consumes that pending
//! state before cover enumeration. The compiler chooses only a bounded work
//! budget; `tiler-ir` owns and runs the closed exact evaluator, and only its
//! all-`Proved` result seals durable receipts. `Disproved` and unsupported
//! `Unknown` remain distinct typed refusals.
//!
//! The receipts overlay the immutable verified region. They do not rewrite
//! `tiler-ir` verifier evidence, copy its predicate language, or re-drive the
//! lowering provider. The compiler projects IR's read-only assessment into its
//! explain vocabulary; it supplies no callback, proof constructor, or authority
//! bytes. Dtype payloads, component layouts, and physical encodings cannot
//! affect an index-domain predicate and are never inspected.

use core::fmt;
use tiler_ir::index::{
    IndexDomainProofAssessment as IrIndexDomainProofAssessment,
    IndexDomainProofClaim as IrIndexDomainProofClaim,
    IndexDomainProofEvidence as IrIndexDomainProofEvidence,
    IndexDomainProofRefusalKind as IrIndexDomainProofRefusalKind, IndexDomainUnknownReason,
    UnknownIndexDomainPredicate,
};
use tiler_ir::semantic::ProviderIdentity;

use crate::legality::{
    IndexRefinement, PendingIndexRefinement, RefinementError, complete_pending_index_refinement,
};

const MAX_DISCHARGE_CELLS: u64 = tiler_ir::index::MAX_FINITE_DOMAIN_PROOF_CELLS;
const MAX_DISCHARGE_INTEGER_BYTES: u64 = tiler_ir::index::MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES;

/// Versioned semantic identity of one proof or disproof rule.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainProofRuleKey(ProviderIdentity);

impl IndexDomainProofRuleKey {
    /// Returns the canonical provider-shaped key.
    pub(crate) const fn identity(&self) -> &ProviderIdentity {
        &self.0
    }
}

/// Output-affecting revision of one discharge authority.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainDischargeRevision(u32);

impl IndexDomainDischargeRevision {
    /// Creates a nonzero revision.
    fn new(value: u32) -> Option<Self> {
        (value != 0).then_some(Self(value))
    }

    /// Returns the stored revision.
    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

/// Read-only compiler projection of IR's closed proof authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainDischargeAuthority {
    provider: ProviderIdentity,
    rule: IndexDomainProofRuleKey,
    revision: IndexDomainDischargeRevision,
}

impl IndexDomainDischargeAuthority {
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    pub(crate) const fn rule(&self) -> &IndexDomainProofRuleKey {
        &self.rule
    }

    pub(crate) const fn revision(&self) -> IndexDomainDischargeRevision {
        self.revision
    }
}

/// A proving basis a trusted discharge rule may claim.
///
/// Empirical evidence is absent by construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeProof {
    /// Exact evaluation of every point in a bounded finite domain.
    ExhaustiveFinite { points: u64, derivation: Box<[u8]> },
}

/// A typed semantic disproof claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDisproof {
    reason: Box<str>,
    point_ordinal: Option<u64>,
    counterexample: Box<[u8]>,
}

impl IndexDomainDisproof {
    pub(crate) fn new(reason: impl Into<Box<str>>, counterexample: impl Into<Box<[u8]>>) -> Self {
        Self {
            reason: reason.into(),
            point_ordinal: None,
            counterexample: counterexample.into(),
        }
    }

    fn with_point_ordinal(mut self, point_ordinal: u64) -> Self {
        self.point_ordinal = Some(point_ordinal);
        self
    }

    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }

    pub(crate) const fn point_ordinal(&self) -> Option<u64> {
        self.point_ordinal
    }
}

/// Compiler explain projection of IR's total claim for one exact obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeClaim {
    Proved(IndexDomainDischargeProof),
    Disproved(IndexDomainDisproof),
    Unknown(IndexDomainUnknownReason),
}

/// One exact assessment retained for explanation on refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDischargeAssessment {
    obligation: UnknownIndexDomainPredicate,
    authority: IndexDomainDischargeAuthority,
    claim: IndexDomainDischargeClaim,
}

impl IndexDomainDischargeAssessment {
    pub(crate) const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }

    pub(crate) const fn authority(&self) -> &IndexDomainDischargeAuthority {
        &self.authority
    }

    pub(crate) const fn claim(&self) -> &IndexDomainDischargeClaim {
        &self.claim
    }
}

/// Why semantic discharge refused one otherwise-conforming realization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeRefusalKind {
    Disproved,
    Unknown,
}

/// Atomic refusal retaining every canonical assessment and the exact pending state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDischargeRefusal {
    pending: Box<PendingIndexRefinement>,
    assessments: Vec<IndexDomainDischargeAssessment>,
    kind: IndexDomainDischargeRefusalKind,
}

impl IndexDomainDischargeRefusal {
    #[allow(
        dead_code,
        reason = "the pending state is retained to prove atomic refusal and inspected by conformance tests; production explanation consumes the exact assessments instead"
    )]
    pub(crate) const fn pending(&self) -> &PendingIndexRefinement {
        &self.pending
    }

    pub(crate) fn assessments(&self) -> &[IndexDomainDischargeAssessment] {
        &self.assessments
    }

    pub(crate) const fn kind(&self) -> IndexDomainDischargeRefusalKind {
        self.kind
    }
}

impl fmt::Display for IndexDomainDischargeRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} index-domain obligation(s) reached semantic discharge as {:?}",
            self.assessments.len(),
            self.kind
        )
    }
}

pub(crate) enum IndexDomainDischargeError {
    Domain(IndexDomainDischargeRefusal),
    Refinement(RefinementError),
}

/// Runs the production discharge rule before executable planning.
pub(crate) fn discharge_pending_index_refinement(
    pending: PendingIndexRefinement,
) -> Result<IndexRefinement, IndexDomainDischargeError> {
    let budget = tiler_ir::index::IndexDomainProofBudget::try_new(
        MAX_DISCHARGE_CELLS,
        MAX_DISCHARGE_INTEGER_BYTES,
    )
    .expect("the governed compiler proof budgets are within IR's hard bounds");
    let completed =
        tiler_ir::index::ResolvedIndexRealization::complete(pending.ir_receipt(), budget);
    let (ir_receipt, ir_assessments) = match completed {
        Ok(completed) => completed,
        Err(refusal) => {
            let kind = match refusal.kind() {
                IrIndexDomainProofRefusalKind::Disproved => {
                    IndexDomainDischargeRefusalKind::Disproved
                }
                IrIndexDomainProofRefusalKind::Unknown => IndexDomainDischargeRefusalKind::Unknown,
            };
            return Err(IndexDomainDischargeError::Domain(
                IndexDomainDischargeRefusal {
                    pending: Box::new(pending),
                    assessments: refusal
                        .assessments()
                        .iter()
                        .map(convert_ir_assessment)
                        .collect(),
                    kind,
                },
            ));
        }
    };
    debug_assert!(
        ir_assessments
            .iter()
            .all(|assessment| matches!(assessment.claim(), IrIndexDomainProofClaim::Proved(_)))
    );
    complete_pending_index_refinement(pending, ir_receipt)
        .map_err(IndexDomainDischargeError::Refinement)
}

fn convert_ir_assessment(
    assessment: &IrIndexDomainProofAssessment,
) -> IndexDomainDischargeAssessment {
    let claim = match assessment.claim() {
        IrIndexDomainProofClaim::Proved(IrIndexDomainProofEvidence::ExhaustiveFinite {
            points,
            derivation,
            ..
        }) => IndexDomainDischargeClaim::Proved(IndexDomainDischargeProof::ExhaustiveFinite {
            points: *points,
            derivation: derivation.clone(),
        }),
        IrIndexDomainProofClaim::Disproved(disproof) => {
            let mut converted =
                IndexDomainDisproof::new(disproof.reason(), disproof.counterexample());
            if let Some(point) = disproof.point_ordinal() {
                converted = converted.with_point_ordinal(point);
            }
            IndexDomainDischargeClaim::Disproved(converted)
        }
        IrIndexDomainProofClaim::Unknown(reason) => IndexDomainDischargeClaim::Unknown(*reason),
    };
    let authority = IndexDomainDischargeAuthority {
        provider: assessment.authority().provider().clone(),
        rule: IndexDomainProofRuleKey(assessment.authority().rule().clone()),
        revision: IndexDomainDischargeRevision::new(assessment.authority().revision())
            .expect("IR proof revisions are nonzero"),
    };
    IndexDomainDischargeAssessment {
        obligation: assessment.obligation(),
        authority,
        claim,
    }
}
