//! What a checked association mints, and what it leaves when it cannot.
//!
//! A verified check with no residual index-domain obligation mints an
//! [`IndexRefinementReceipt`] and, with it, the reached-only executable
//! coverage a consumer may publish. A check that succeeds structurally but
//! retains an obligation mints neither and returns a
//! [`PendingIndexRefinementReceipt`] instead, which carries no coverage
//! spelling at all: only completion produces one. Both artifacts are opaque —
//! their fields are minted here and read through accessors — which is why the
//! constructor and the revalidation that compares a receipt back against its
//! pending association stay in one file.

use crate::index::{
    CanonicalIndexRegionIdentity, CanonicalIndexRegionSequenceIdentity, ScalarAuthorityEvidence,
    UnknownIndexDomainPredicate, VerifiedIndexRegion, VerifiedIndexRegionSequence,
};
use crate::program::SemanticOccurrence;
use crate::semantic::SemanticGraphIdentity;

use super::binding::{OperandBinding, ResultBinding};
use super::error::IndexRefinementVerificationError;
use super::identity::{encode_executable_coverage_identity, encode_receipt_identity};
use super::proof::IndexRefinementDomainProof;
use super::registry::ResolvedIndexRealization;
use super::subject::IndexRefinementSubject;

/// Canonical identity of one checked occurrence-to-region receipt.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementReceiptIdentity(Box<[u8]>);

impl IndexRefinementReceiptIdentity {
    /// Returns the canonical receipt bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Reached-only executable provenance minted with one completed refinement receipt.
///
/// ```compile_fail
/// use tiler_ir::index::IndexRefinementExecutableCoverageIdentity;
///
/// // Executable coverage is proof-derived; opaque bytes are not a constructor.
/// let _ = IndexRefinementExecutableCoverageIdentity(Box::new([]));
/// ```
///
/// ```compile_fail
/// use tiler_ir::index::IndexRefinementExecutableCoverageIdentity;
///
/// // Nor is there a byte-level conversion: a caller holding one receipt's
/// // coverage bytes cannot re-mint them onto another receipt's occurrence.
/// fn cross(bytes: &[u8]) -> IndexRefinementExecutableCoverageIdentity {
///     IndexRefinementExecutableCoverageIdentity::from(bytes)
/// }
/// ```
///
/// This identity deliberately excludes the complete semantic, scalar, and
/// realization-law registry snapshots retained by [`IndexRefinementReceiptIdentity`].
/// It retains the selected graph occurrence, numerical contract, realization
/// law and provider, the realization's regions, reached semantic/scalar/type
/// definition and admission projections, exact operand/result bindings, and
/// every residual proof identity. Callers may read these bytes but cannot
/// construct this type from bytes or independently supplied fields.
///
/// The operation key, ordered signature, host-canonical attributes, and operand
/// and result boundary shapes are not re-encoded: `tiler.semantic-graph.v3`
/// already writes each of them for every operation in canonical traversal
/// order, and [`IndexRefinementSubject::derive`] fixes the retained occurrence
/// to that same canonical ordinal. Encoding them a second time would restate
/// what the graph and occurrence pair already determines rather than close a
/// substitution the pair leaves open.
///
/// **The graph half of that pair is named by digest rather than restated**, as
/// of `v2` and ADR 0104: the record opens with a fixed-width governed digest of
/// the bound graph's identity instead of the identity itself. What the pair
/// determines is unchanged — two receipts for one occurrence ordinal of two
/// different graphs still mint different bytes — and what changes is that the
/// graph identity is no longer recoverable from these bytes, which nothing in
/// the workspace attempted. The restatement was the whole of kernel-program
/// identity's quadratic term — one graph identity per record, one record per
/// semantic operation — and folding it makes that curve linear;
/// `encode_executable_coverage_identity` carries the derivation, and the
/// measured constants are read from
/// [the identity-growth spike](../../../../../spikes/program-planning/identity-growth/README.md),
/// whose results index records which compiler tree each retained ladder
/// measured and the displacement between consecutive ones.
///
/// *Corrected 2026-08-08 by
/// [`correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix`](../../../../../tickets/correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix.md),
/// and dated beside rather than substituted because it was true when written* —
/// this sentence pointed at `docs/artifact-abi.md` for the measured constants,
/// and that contract did carry them from this comment's authoring at `d48a33af`
/// until `775d314f` on 2026-08-08, when it deliberately stopped carrying the
/// fit as a live value and named the spike as the standing authority. Three
/// spellings of one curve in four days, none pinned by a test, is the reasoning
/// it recorded for the change; repointing here rather than restating a
/// coefficient is the same reasoning applied to this comment. The derivation
/// half of the sentence is unchanged.
///
/// *Corrected 2026-08-08 by
/// [`step-the-coverage-identity-comment-s-stale-semantic-graph-domain`](../../../../../tickets/step-the-coverage-identity-comment-s-stale-semantic-graph-domain.md),
/// and dated beside rather than substituted because it was true when written* —
/// the not-re-encoded paragraph above named the graph domain
/// `tiler.semantic-graph.v2`, and
/// `GRAPH_DOMAIN` in `crates/tiler-ir/src/semantic/identity.rs` did read that
/// from this comment's authoring at `6d143a01` on 2026-08-04 until `26157836`
/// on 2026-08-07, when
/// [`carry-a-sourced-shape-on-semantic-values`](../../../../../tickets/carry-a-sourced-shape-on-semantic-values.md)
/// stepped it to `v3` and began writing every extent through
/// `SourcedShape::encode`. **Only the spelling moved.** `compute_graph_identity`
/// under `v3` still writes the operation key and host-canonical attributes via
/// `encode_operation`, the ordered operand and result signature, and each
/// result's boundary shape, once per operation in canonical traversal order; the
/// step changed how an extent is spelled and not which of these the graph
/// covers, so the delegation this paragraph rests on is unchanged and the
/// sentence needed its domain corrected rather than its claim withdrawn.
/// **The retired spelling `tiler.semantic-graph.v2` is quoted here and so stays
/// greppable**, which means a later hit on it in this file lands inside this
/// note rather than on a live claim.
///
/// [ADR 0104](../../../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)
/// quotes the retired sentence verbatim while rejecting an alternative, and
/// carries a note of its own recording that this comment "still says `v2`" and
/// that the stale text was therefore here rather than there. That note is what
/// this correction makes stale; repairing it is `contracts/decisions` and not
/// this scope, so it is reported rather than fixed in passing.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementExecutableCoverageIdentity(Box<[u8]>);

impl IndexRefinementExecutableCoverageIdentity {
    /// Returns the canonical reached-only executable-coverage bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Opaque checked binding of one semantic occurrence to one verified
/// realization.
///
/// A realization is an *ordered sequence* of verified regions, and every field
/// below that names a region names all of them. [`Self::final_stage`] and
/// [`Self::final_scalar_authority`] answer the last stage alone and never the
/// realization; a consumer that must see the whole chain reads
/// [`Self::regions`], [`Self::scalar_authorities`], or [`Self::realization`].
/// The accessors are named for what they return because a one-stage
/// realization makes stage and realization indistinguishable, and a reader who
/// learned an accessor there would otherwise carry that reading into the first
/// chain met. That is still most of what a reader sees — ten of
/// [`IndexRealizationLaw`]'s thirteen variants are
/// single-region — but it is no longer all of it: the standard semantic
/// authority registers staged laws for `tiler::rms-norm-f32@1` and
/// `tiler::softmax-f32@1`, so a chain reaches these accessors from the governed
/// vocabulary rather than only from a test registry.
///
/// [`IndexRealizationLaw`]: crate::index::IndexRealizationLaw
#[derive(Clone, Debug)]
pub struct IndexRefinementReceipt {
    graph: SemanticGraphIdentity,
    occurrence: SemanticOccurrence,
    /// Every region identity except the final stage's, in stage order.
    ///
    /// Split the way [`VerifiedIndexRegionSequence`] splits its stages, so the
    /// stage whose writes are the occurrence's results is a field rather than a
    /// lookup a reader has to establish cannot fail.
    leading_regions: Vec<CanonicalIndexRegionIdentity>,
    region: CanonicalIndexRegionIdentity,
    realization: CanonicalIndexRegionSequenceIdentity,
    leading_scalar_authorities: Vec<ScalarAuthorityEvidence>,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
    identity: IndexRefinementReceiptIdentity,
    executable_coverage_identity: IndexRefinementExecutableCoverageIdentity,
}

impl IndexRefinementReceipt {
    /// Returns the semantic graph this receipt binds.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraphIdentity {
        &self.graph
    }
    /// Returns the graph-local semantic occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> SemanticOccurrence {
        self.occurrence
    }
    /// Returns the final stage's verified-region identity.
    ///
    /// The final stage is the one whose writes are the occurrence's results. For
    /// a one-stage realization it is the only region, and its identity is the
    /// realization identity byte for byte. For a chain it identifies one stage
    /// of several and is **not** the realization: the leading stages leave no
    /// trace in it, so two chains that merely end alike agree here. Compare
    /// [`Self::realization`] to compare realizations, and read [`Self::regions`]
    /// for every stage in order.
    #[must_use]
    pub const fn final_stage(&self) -> &CanonicalIndexRegionIdentity {
        &self.region
    }
    /// Returns every verified-region identity in stage order.
    #[must_use]
    pub fn regions(&self) -> Vec<CanonicalIndexRegionIdentity> {
        let mut regions = self.leading_regions.clone();
        regions.push(self.region.clone());
        regions
    }
    /// Returns the exact canonical identity of the whole ordered realization.
    #[must_use]
    pub const fn realization(&self) -> &CanonicalIndexRegionSequenceIdentity {
        &self.realization
    }
    /// Returns the checked scalar authority bound to the final stage alone.
    ///
    /// A chain's stages reach their own scalar operations and need not overlap:
    /// the governed staged template's fold reaches the add and its pass reaches
    /// the multiply, and neither reaches the other's. So this is one stage's
    /// reached vocabulary, not the realization's;
    /// [`Self::scalar_authorities`] answers that, in stage order.
    #[must_use]
    pub const fn final_scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }
    /// Returns the checked scalar authority of every stage, in stage order.
    #[must_use]
    pub fn scalar_authorities(&self) -> Vec<ScalarAuthorityEvidence> {
        let mut authorities = self.leading_scalar_authorities.clone();
        authorities.push(self.scalar_authority.clone());
        authorities
    }
    /// Returns ordered operand-to-input bindings.
    ///
    /// An encoded logical operand contributes one binding for every component
    /// in its semantic contract order; an ordinary operand contributes one.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }
    /// Returns ordered result-to-output bindings.
    ///
    /// A result whose output is partitioned contributes one binding per
    /// partition member, so this is one entry per output root and a caller that
    /// needs one answer per result groups by [`ResultBinding::result`].
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }
    /// Returns independently verified residual-domain proofs.
    #[must_use]
    pub fn index_domain_proofs(&self) -> &[IndexRefinementDomainProof] {
        &self.index_domain_proofs
    }
    /// Returns the canonical receipt identity.
    #[must_use]
    pub const fn identity(&self) -> &IndexRefinementReceiptIdentity {
        &self.identity
    }
    /// Returns reached-only provenance suitable for executable coverage.
    ///
    /// Unlike [`Self::identity`], this subject excludes unused registry rows.
    /// Its only minting path is successful receipt completion.
    #[must_use]
    pub const fn executable_coverage_identity(&self) -> &IndexRefinementExecutableCoverageIdentity {
        &self.executable_coverage_identity
    }
}

/// Checked association awaiting proof of retained index-domain obligations.
///
/// A pending association has no executable-coverage spelling. Only
/// [`ResolvedIndexRealization::complete`] discharges the retained obligations,
/// and only its success value carries an
/// [`IndexRefinementExecutableCoverageIdentity`]:
///
/// ```compile_fail
/// fn coverage(
///     pending: &tiler_ir::index::PendingIndexRefinementReceipt,
/// ) -> &tiler_ir::index::IndexRefinementExecutableCoverageIdentity {
///     pending.executable_coverage_identity()
/// }
/// ```
#[derive(Clone, Debug)]
pub struct PendingIndexRefinementReceipt {
    pub(super) resolution: ResolvedIndexRealization,
    /// Every stage's evidence except the final one's, in stage order.
    ///
    /// Split the same way [`VerifiedIndexRegionSequence`] splits its stages: a
    /// realization always has a final stage, so its evidence is a field rather
    /// than a lookup that could fail.
    pub(super) leading_scalar_authorities: Vec<ScalarAuthorityEvidence>,
    pub(super) scalar_authority: ScalarAuthorityEvidence,
    pub(super) operand_bindings: Vec<OperandBinding>,
    pub(super) result_bindings: Vec<ResultBinding>,
    pub(super) realization: VerifiedIndexRegionSequence,
}

impl PendingIndexRefinementReceipt {
    /// Returns the checked semantic occurrence.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        self.resolution.subject()
    }
    /// Returns the exact retained final-stage verified region.
    ///
    /// For a one-stage realization this is the only region, and evaluating it
    /// evaluates the occurrence. For a chain it is one stage of several and
    /// evaluating it does not: at least one of its
    /// input boundaries reads the value the preceding stage handed on, which no
    /// operand named by [`Self::operand_bindings`] carries. A consumer that can
    /// run exactly one region must therefore establish that
    /// [`Self::realization`] has exactly one stage before it runs this one —
    /// otherwise it runs part of a realization and reports the result as the
    /// occurrence's.
    #[must_use]
    pub const fn final_stage(&self) -> &VerifiedIndexRegion {
        self.realization.final_stage()
    }
    /// Returns the exact retained ordered realization.
    #[must_use]
    pub const fn realization(&self) -> &VerifiedIndexRegionSequence {
        &self.realization
    }
    /// Returns the final stage's checked scalar authority evidence alone.
    ///
    /// Each stage carries its own reached vocabulary, so for a chain this omits
    /// every scalar operation only a leading stage reaches;
    /// [`Self::scalar_authorities`] answers the whole realization, in stage
    /// order.
    #[must_use]
    pub const fn final_scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }
    /// Returns every stage's checked scalar authority evidence, in stage order.
    #[must_use]
    pub fn scalar_authorities(&self) -> Vec<ScalarAuthorityEvidence> {
        let mut authorities = self.leading_scalar_authorities.clone();
        authorities.push(self.scalar_authority.clone());
        authorities
    }
    /// Returns ordered operand bindings, expanding encoded components in their
    /// semantic contract order.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }
    /// Returns ordered result bindings, one per output root.
    ///
    /// A partitioned result contributes one binding per member; see
    /// [`IndexRefinementReceipt::result_bindings`].
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }
    /// Returns every exact residual obligation, in stage order and within a
    /// stage in canonical region order.
    ///
    /// An obligation is region-local, so a caller reading this flat sequence
    /// needs [`Self::staged_obligations`] to know which stage each belongs to;
    /// the flat order is what a completed receipt's proofs are aligned against.
    #[must_use]
    pub fn obligations(&self) -> impl ExactSizeIterator<Item = UnknownIndexDomainPredicate> + '_ {
        self.staged_obligations()
            .into_iter()
            .map(|(_, obligation)| obligation)
    }

    /// Returns every residual obligation paired with the stage that retains it.
    #[must_use]
    pub fn staged_obligations(&self) -> Vec<(usize, UnknownIndexDomainPredicate)> {
        self.realization
            .stages()
            .enumerate()
            .flat_map(|(stage, region)| {
                region
                    .unknown_index_domain_predicates()
                    .map(move |obligation| (stage, obligation))
            })
            .collect()
    }

    /// Revalidates that a completed receipt was minted from this exact pending
    /// association and its canonical residual obligations.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when any occurrence, region, authority,
    /// interface, proof, or identity field was crossed with another pending
    /// association.
    pub fn verify_completion(
        &self,
        receipt: &IndexRefinementReceipt,
    ) -> Result<(), IndexRefinementVerificationError> {
        let subject = self.subject();
        if receipt.graph != subject.graph || receipt.occurrence != subject.occurrence {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.realization != *self.realization.identity() {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        let bound_regions = receipt.regions();
        if bound_regions.len() != self.realization.stage_count()
            || bound_regions
                .iter()
                .zip(self.realization.stages())
                .any(|(bound, stage)| bound != stage.canonical_identity())
        {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.scalar_authorities() != self.scalar_authorities() {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.operand_bindings != self.operand_bindings {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.result_bindings != self.result_bindings {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        let obligations = self.staged_obligations();
        if receipt.index_domain_proofs.len() != obligations.len()
            || receipt.index_domain_proofs.iter().zip(obligations).any(
                |(proof, (stage, obligation))| {
                    proof.obligation != obligation || proof.stage != stage
                },
            )
        {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        let expected = encode_receipt_identity(
            subject,
            &self.resolution,
            &self.realization,
            &self.scalar_authorities(),
            &receipt.index_domain_proofs,
        );
        if receipt.identity.as_bytes() != expected {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        Ok(())
    }
}

impl PartialEq for PendingIndexRefinementReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.resolution == other.resolution
            && self.leading_scalar_authorities == other.leading_scalar_authorities
            && self.scalar_authority == other.scalar_authority
            && self.operand_bindings == other.operand_bindings
            && self.result_bindings == other.result_bindings
            && self.realization == other.realization
    }
}

impl Eq for PendingIndexRefinementReceipt {}

/// Result of checking the dependency-neutral refinement association.
#[derive(Clone, Debug)]
#[must_use]
pub enum IndexRefinementVerificationOutcome {
    /// All obligations are discharged and a receipt was minted.
    Verified(Box<IndexRefinementReceipt>),
    /// The association is checked, but residual obligations grant no permission.
    Pending(Box<PendingIndexRefinementReceipt>),
}

pub(super) fn mint_receipt(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    realization: &VerifiedIndexRegionSequence,
    scalar_authorities: Vec<ScalarAuthorityEvidence>,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
) -> IndexRefinementReceipt {
    let identity = encode_receipt_identity(
        subject,
        resolution,
        realization,
        &scalar_authorities,
        &index_domain_proofs,
    );
    let executable_coverage_identity = encode_executable_coverage_identity(
        subject,
        resolution,
        realization,
        &scalar_authorities,
        &operand_bindings,
        &result_bindings,
        &index_domain_proofs,
    );
    let mut leading_scalar_authorities = scalar_authorities;
    let Some(scalar_authority) = leading_scalar_authorities.pop() else {
        unreachable!("a realization has a final stage and therefore its evidence")
    };
    IndexRefinementReceipt {
        graph: subject.graph.clone(),
        occurrence: subject.occurrence,
        leading_regions: realization
            .leading_stages()
            .iter()
            .map(|stage| stage.canonical_identity().clone())
            .collect(),
        region: realization.final_stage().canonical_identity().clone(),
        realization: realization.identity().clone(),
        leading_scalar_authorities,
        scalar_authority,
        operand_bindings,
        result_bindings,
        index_domain_proofs,
        identity: IndexRefinementReceiptIdentity(identity.into_boxed_slice()),
        executable_coverage_identity: IndexRefinementExecutableCoverageIdentity(
            executable_coverage_identity.into_boxed_slice(),
        ),
    }
}
