//! Compiler-owned discharge of residual logical index-domain predicates.
//!
//! A structurally verified region may retain exact `Unknown` predicates without
//! becoming executable refinement evidence. This stage consumes that pending
//! state before cover enumeration. A trusted rule assesses each exact borrowed
//! obligation once; only an all-`Proved` result seals durable receipts and
//! completes refinement. `Disproved` and unsupported `Unknown` remain distinct
//! typed refusals.
//!
//! The receipts overlay the immutable verified region. They do not rewrite
//! `tiler-ir` verifier evidence, copy its predicate language, or re-drive the
//! lowering provider. Concrete host/device semantic enforcement is a separate
//! physical/runtime vertical; the initial compiler rule therefore preserves
//! every `Unknown` and fails closed.

#![allow(
    dead_code,
    reason = "the three-way discharge protocol is complete while the initial production authority deliberately constructs only Unknown; Proved and Disproved are exercised through private conformance authorities until a real proof or host-enforcement authority lands"
)]

use core::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    IndexDomainSoundProof, IndexDomainUnknownReason, UnknownIndexDomainPredicate,
    VerifiedIndexRegion,
};
use tiler_ir::semantic::ProviderIdentity;

use crate::legality::{IndexRefinement, PendingIndexRefinement, complete_pending_index_refinement};

/// Canonical identity tag for one sealed semantic-discharge receipt.
const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.compiler.index-domain-discharge-receipt.v1\0";

/// Versioned semantic identity of one proof or disproof rule.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainProofRuleKey(ProviderIdentity);

impl IndexDomainProofRuleKey {
    /// Creates a compiler-owned rule key.
    fn builtin(name: &str, version: u32) -> Self {
        Self(
            ProviderIdentity::new("tiler", name, version)
                .expect("compiler-owned discharge rule keys are valid"),
        )
    }

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

/// Complete identity of the trusted rule making one claim.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct IndexDomainDischargeAuthority {
    provider: ProviderIdentity,
    rule: IndexDomainProofRuleKey,
    revision: IndexDomainDischargeRevision,
}

impl IndexDomainDischargeAuthority {
    pub(crate) fn builtin(provider: &str, rule: &str, revision: u32) -> Self {
        Self {
            provider: ProviderIdentity::new("tiler", provider, 1)
                .expect("compiler-owned discharge provider identities are valid"),
            rule: IndexDomainProofRuleKey::builtin(rule, 1),
            revision: IndexDomainDischargeRevision::new(revision)
                .expect("compiler-owned discharge revisions are nonzero"),
        }
    }

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
    /// A sound named derivation over the complete predicate domain.
    Sound {
        proof: IndexDomainSoundProof,
        derivation: Box<[u8]>,
    },
    /// Exact evaluation of every point in a bounded finite domain.
    ExhaustiveFinite { points: u64, derivation: Box<[u8]> },
}

/// A typed semantic disproof claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexDomainDisproof {
    reason: &'static str,
    counterexample: Box<[u8]>,
}

impl IndexDomainDisproof {
    pub(crate) fn new(reason: &'static str, counterexample: impl Into<Box<[u8]>>) -> Self {
        Self {
            reason,
            counterexample: counterexample.into(),
        }
    }

    pub(crate) const fn reason(&self) -> &'static str {
        self.reason
    }
}

/// One trusted rule's total claim about one exact borrowed obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexDomainDischargeClaim {
    Proved(IndexDomainDischargeProof),
    Disproved(IndexDomainDisproof),
    Unknown(IndexDomainUnknownReason),
}

/// The private authority callback used by the initial discharge stage.
///
/// It is deliberately not a public extension seam. The production compiler has
/// only the fail-closed rule below; a public registry belongs with the first
/// independently installable authority and its reviewed resolution contract.
pub(crate) trait IndexDomainDischargeProvider {
    fn authority(&self) -> &IndexDomainDischargeAuthority;

    fn assess(
        &self,
        region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
    ) -> IndexDomainDischargeClaim;
}

/// One sealed proof receipt bound to an exact region and local obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorizedIndexDomainProof {
    obligation: UnknownIndexDomainPredicate,
    authority: IndexDomainDischargeAuthority,
    proof: IndexDomainDischargeProof,
    identity: Box<[u8]>,
}

impl AuthorizedIndexDomainProof {
    fn seal(
        region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
        authority: &IndexDomainDischargeAuthority,
        proof: IndexDomainDischargeProof,
    ) -> Self {
        let identity = encode_receipt(region, obligation, authority, &proof).into_boxed_slice();
        Self {
            obligation,
            authority: authority.clone(),
            proof,
            identity,
        }
    }

    pub(crate) const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }

    pub(crate) const fn authority(&self) -> &IndexDomainDischargeAuthority {
        &self.authority
    }

    pub(crate) const fn proof(&self) -> &IndexDomainDischargeProof {
        &self.proof
    }

    pub(crate) fn identity(&self) -> &[u8] {
        &self.identity
    }
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

/// Production rule until a real semantic enforcement or proof authority lands.
struct UnsupportedIndexDomainDischarge {
    authority: IndexDomainDischargeAuthority,
}

impl UnsupportedIndexDomainDischarge {
    fn governed() -> Self {
        Self {
            authority: IndexDomainDischargeAuthority::builtin(
                "compiler.index-domain-discharge",
                "index-domain-discharge-unsupported",
                1,
            ),
        }
    }
}

impl IndexDomainDischargeProvider for UnsupportedIndexDomainDischarge {
    fn authority(&self) -> &IndexDomainDischargeAuthority {
        &self.authority
    }

    fn assess(
        &self,
        _region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
    ) -> IndexDomainDischargeClaim {
        IndexDomainDischargeClaim::Unknown(obligation.reason())
    }
}

/// Runs the production discharge rule before executable planning.
pub(crate) fn discharge_pending_index_refinement(
    pending: PendingIndexRefinement,
) -> Result<IndexRefinement, IndexDomainDischargeRefusal> {
    discharge_with(&UnsupportedIndexDomainDischarge::governed(), pending)
}

pub(crate) fn discharge_with(
    provider: &dyn IndexDomainDischargeProvider,
    pending: PendingIndexRefinement,
) -> Result<IndexRefinement, IndexDomainDischargeRefusal> {
    let assessments = pending
        .obligations()
        .map(|obligation| IndexDomainDischargeAssessment {
            obligation,
            authority: provider.authority().clone(),
            claim: provider.assess(pending.region(), obligation),
        })
        .collect::<Vec<_>>();
    let kind = if assessments
        .iter()
        .any(|assessment| matches!(assessment.claim, IndexDomainDischargeClaim::Disproved(_)))
    {
        Some(IndexDomainDischargeRefusalKind::Disproved)
    } else if assessments
        .iter()
        .any(|assessment| matches!(assessment.claim, IndexDomainDischargeClaim::Unknown(_)))
    {
        Some(IndexDomainDischargeRefusalKind::Unknown)
    } else {
        None
    };
    if let Some(kind) = kind {
        return Err(IndexDomainDischargeRefusal {
            pending: Box::new(pending),
            assessments,
            kind,
        });
    }
    let receipts = assessments
        .into_iter()
        .map(|assessment| {
            let IndexDomainDischargeClaim::Proved(proof) = assessment.claim else {
                unreachable!("the refusal scan removed every non-proved assessment")
            };
            AuthorizedIndexDomainProof::seal(
                pending.region(),
                assessment.obligation,
                &assessment.authority,
                proof,
            )
        })
        .collect();
    Ok(complete_pending_index_refinement(pending, receipts))
}

fn encode_receipt(
    region: &VerifiedIndexRegion,
    obligation: UnknownIndexDomainPredicate,
    authority: &IndexDomainDischargeAuthority,
    proof: &IndexDomainDischargeProof,
) -> Vec<u8> {
    let mut output = RECEIPT_IDENTITY_TAG.to_vec();
    push_slice(&mut output, region.canonical_identity().as_bytes());
    push_slice(&mut output, obligation.canonical_local_key().as_bytes());
    encode_provider(&mut output, authority.provider());
    encode_provider(&mut output, authority.rule().identity());
    output.extend_from_slice(&authority.revision().get().to_be_bytes());
    match proof {
        IndexDomainDischargeProof::Sound { proof, derivation } => {
            output.push(1);
            output.push(match proof {
                IndexDomainSoundProof::VacuousEmptyDomain => 1,
                IndexDomainSoundProof::Interval => 2,
                IndexDomainSoundProof::ProvedExtentEquality => 3,
            });
            push_slice(&mut output, derivation);
        }
        IndexDomainDischargeProof::ExhaustiveFinite { points, derivation } => {
            output.push(2);
            output.extend_from_slice(&points.to_be_bytes());
            push_slice(&mut output, derivation);
        }
    }
    output
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_len(output, provider.namespace().len());
    output.extend_from_slice(provider.namespace().as_bytes());
    push_len(output, provider.name().len());
    output.extend_from_slice(provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

#[cfg(test)]
mod tests {
    // Stage tests are added with the compiler fixture once the private
    // transition is wired into `legality` and `lowering`.
}
