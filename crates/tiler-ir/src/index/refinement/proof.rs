//! The residual-domain proof vocabulary a receipt retains.
//!
//! A verified region may leave an index-domain obligation undischarged, and
//! this is the vocabulary in which such an obligation is answered: who claimed
//! it, what the claim was, what evidence or counterexample stands behind it, and
//! what a completed receipt seals. It is kept apart from [`super::finite`]
//! deliberately — this file is what a receipt carries and a consumer reads,
//! while that one is one authority's way of producing it, and a second
//! authority would join there rather than here.

use core::fmt;
use std::error::Error;
use std::sync::Arc;

use crate::index::{IndexDomainUnknownReason, ProofResource, UnknownIndexDomainPredicate};
use crate::semantic::ProviderIdentity;

use super::error::IndexRefinementVerificationError;
use super::{
    MAX_DOMAIN_EVIDENCE_BYTES, MAX_FINITE_DOMAIN_PROOF_CELLS, MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
};

/// Complete identity of one trusted residual-domain proof authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexDomainProofAuthority {
    provider: ProviderIdentity,
    rule: ProviderIdentity,
    revision: u32,
}

impl IndexDomainProofAuthority {
    pub(super) fn exact_finite() -> Self {
        Self {
            provider: ProviderIdentity::new("tiler", "ir-index-domain-proof", 1)
                .expect("the IR proof provider identity is canonical"),
            rule: ProviderIdentity::new("tiler", "exact-finite-index-domain-enumeration", 1)
                .expect("the IR proof rule identity is canonical"),
            revision: 1,
        }
    }

    /// Returns the proof provider identity.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }
    /// Returns the versioned proof rule identity.
    #[must_use]
    pub const fn rule(&self) -> &ProviderIdentity {
        &self.rule
    }
    /// Returns the output-affecting authority revision.
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }
}

/// Proof evidence produced by IR's closed residual-domain algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexDomainProofEvidence {
    /// Exact evaluation of every point in a bounded finite domain.
    #[non_exhaustive]
    ExhaustiveFinite {
        /// Number of evaluated domain points.
        points: u64,
        /// Authority-owned canonical derivation bytes.
        derivation: Box<[u8]>,
    },
}

/// Bounded policy input to IR's closed exact-finite proof algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexDomainProofBudget {
    max_cells: u64,
    max_integer_bytes: u64,
}

impl IndexDomainProofBudget {
    /// Creates a nonzero budget no larger than IR's hard proof bound.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::InvalidDomainProofBudget`]
    /// with the exact resource, supplied value, and hard limit when either
    /// limit is zero or exceeds its corresponding
    /// [`MAX_FINITE_DOMAIN_PROOF_CELLS`] or
    /// [`MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES`] hard bound.
    pub fn try_new(
        max_cells: u64,
        max_integer_bytes: u64,
    ) -> Result<Self, IndexRefinementVerificationError> {
        if max_cells == 0 || max_cells > MAX_FINITE_DOMAIN_PROOF_CELLS {
            return Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: ProofResource::Cells,
                actual: max_cells,
                limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
            });
        }
        if max_integer_bytes == 0 || max_integer_bytes > MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES {
            return Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: ProofResource::IntegerBytes,
                actual: max_integer_bytes,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            });
        }
        Ok(Self {
            max_cells,
            max_integer_bytes,
        })
    }

    /// Returns the maximum cumulative structural and evaluation work cells.
    ///
    /// This includes domain and extent resolution, expression planning,
    /// coordinate initialization/advance, DAG traversal, memo clearing, and
    /// predicate evaluation—not merely expression nodes.
    #[must_use]
    pub const fn max_cells(self) -> u64 {
        self.max_cells
    }

    /// Returns the maximum cumulative integer-byte work the proof may perform.
    #[must_use]
    pub const fn max_integer_bytes(self) -> u64 {
        self.max_integer_bytes
    }
}

/// A typed exact counterexample from IR's closed domain evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDomainDisproof {
    reason: Box<str>,
    point_ordinal: Option<u64>,
    counterexample: Box<[u8]>,
}

impl IndexDomainDisproof {
    /// Creates a bounded disproof payload.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal for empty or oversized evidence.
    pub(super) fn new(
        reason: impl Into<Box<str>>,
        counterexample: impl Into<Box<[u8]>>,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let reason = reason.into();
        let counterexample = counterexample.into();
        if reason.is_empty()
            || reason.len() > MAX_DOMAIN_EVIDENCE_BYTES
            || counterexample.is_empty()
            || counterexample.len() > MAX_DOMAIN_EVIDENCE_BYTES
        {
            return Err(IndexRefinementVerificationError::InvalidDomainProofEvidence);
        }
        Ok(Self {
            reason,
            point_ordinal: None,
            counterexample,
        })
    }

    /// Attaches the exact enumerated point ordinal of the counterexample.
    #[must_use]
    pub(super) fn with_point_ordinal(mut self, point_ordinal: u64) -> Self {
        self.point_ordinal = Some(point_ordinal);
        self
    }
    /// Returns the stable reason code.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    /// Returns the optional exact point ordinal.
    #[must_use]
    pub const fn point_ordinal(&self) -> Option<u64> {
        self.point_ordinal
    }

    /// Returns the authority-owned canonical counterexample bytes.
    #[must_use]
    pub fn counterexample(&self) -> &[u8] {
        &self.counterexample
    }
}

/// IR's total claim about one exact residual obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexDomainProofClaim {
    /// The obligation is proved.
    Proved(IndexDomainProofEvidence),
    /// The obligation has an exact counterexample.
    Disproved(IndexDomainDisproof),
    /// The verifier cannot prove or disprove the obligation.
    Unknown(IndexDomainUnknownReason),
}

/// One exact assessment retained for success identity or refusal explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDomainProofAssessment {
    pub(super) obligation: UnknownIndexDomainPredicate,
    pub(super) authority: Arc<IndexDomainProofAuthority>,
    pub(super) claim: IndexDomainProofClaim,
}

impl IndexDomainProofAssessment {
    /// Returns the exact region-owned obligation.
    #[must_use]
    pub const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }
    /// Returns the authority that made the claim.
    #[must_use]
    pub fn authority(&self) -> &IndexDomainProofAuthority {
        &self.authority
    }
    /// Returns the verifier's total claim.
    #[must_use]
    pub const fn claim(&self) -> &IndexDomainProofClaim {
        &self.claim
    }
}

/// One IR-sealed residual-domain proof retained by a refinement receipt.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementDomainProofIdentity(pub(super) Box<[u8]>);

impl IndexRefinementDomainProofIdentity {
    /// Returns the canonical proof identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One IR-sealed residual-domain proof retained by a refinement receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementDomainProof {
    pub(super) stage: usize,
    pub(super) obligation: UnknownIndexDomainPredicate,
    pub(super) authority: Arc<IndexDomainProofAuthority>,
    pub(super) proof: IndexDomainProofEvidence,
    pub(super) identity: IndexRefinementDomainProofIdentity,
}

impl IndexRefinementDomainProof {
    /// Returns the ordered realization stage that retained the obligation.
    ///
    /// An obligation is region-local, so this is what says which region its
    /// handles resolve against.
    #[must_use]
    pub const fn stage(&self) -> usize {
        self.stage
    }
    /// Returns the exact region-owned obligation that was proved.
    #[must_use]
    pub const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }
    /// Returns the authority that proved the obligation.
    #[must_use]
    pub fn authority(&self) -> &IndexDomainProofAuthority {
        &self.authority
    }
    /// Returns the retained proof basis.
    #[must_use]
    pub const fn proof(&self) -> &IndexDomainProofEvidence {
        &self.proof
    }
    /// Returns the canonical proof identity.
    #[must_use]
    pub const fn identity(&self) -> &IndexRefinementDomainProofIdentity {
        &self.identity
    }
}

/// Whether one atomic completion pass found a disproof or an unknown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexDomainProofRefusalKind {
    /// At least one exact obligation was disproved.
    Disproved,
    /// No obligation was disproved and at least one remained unknown.
    Unknown,
}

/// Atomic refusal retaining every canonical assessment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDomainProofRefusal {
    pub(super) assessments: Vec<IndexDomainProofAssessment>,
    pub(super) kind: IndexDomainProofRefusalKind,
}

impl IndexDomainProofRefusal {
    /// Returns all assessments in canonical obligation order.
    #[must_use]
    pub fn assessments(&self) -> &[IndexDomainProofAssessment] {
        &self.assessments
    }
    /// Returns the fail-closed refusal class.
    #[must_use]
    pub const fn kind(&self) -> IndexDomainProofRefusalKind {
        self.kind
    }
}

impl fmt::Display for IndexDomainProofRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} index-domain obligation(s) reached IR proof completion as {:?}",
            self.assessments.len(),
            self.kind
        )
    }
}

impl Error for IndexDomainProofRefusal {}
