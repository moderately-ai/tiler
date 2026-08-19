//! The frozen realization-law authority and the resolution it mints.
//!
//! A law registry is the semantic snapshot that owns every realization law
//! together with the scalar snapshot those laws are interpreted under, frozen
//! inseparably so a law cannot be read against a vocabulary it was not written
//! for. Resolving a subject against it produces a [`ResolvedIndexRealization`]:
//! the one place a law, its provider, its revision, and an exact subject are
//! bound together, and the value every later check is stated against.

use core::fmt;
use std::sync::Arc;

use crate::identity::push_slice;
use crate::index::{
    CanonicalScalarRegistrySnapshotIdentity, FrozenScalarRegistry, IndexRealizationLaw,
    VerifiedIndexRegionSequence,
};
use crate::semantic::{
    FrozenSemanticRegistry, OpKey, ProviderIdentity, SemanticRegistrySnapshotIdentity,
};

use super::LAW_REGISTRY_IDENTITY_TAG;
use super::error::IndexRefinementVerificationError;
use super::identity::{encode_op_key, encode_provider, encode_resolution_identity};
use super::subject::IndexRefinementSubject;

/// Canonical identity of one frozen semantic realization-law registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRealizationLawRegistryIdentity(Box<[u8]>);

impl IndexRealizationLawRegistryIdentity {
    /// Returns the canonical registry identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub(super) struct FrozenIndexRealizationLawRegistryData {
    pub(super) semantic: FrozenSemanticRegistry,
    pub(super) scalars: FrozenScalarRegistry,
    identity: IndexRealizationLawRegistryIdentity,
}

/// Immutable semantic-provider-bound logical realization-law authority.
#[derive(Clone)]
pub struct FrozenIndexRealizationLawRegistry(pub(super) Arc<FrozenIndexRealizationLawRegistryData>);

impl fmt::Debug for FrozenIndexRealizationLawRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenIndexRealizationLawRegistry")
            .field("identity", &self.0.identity)
            .finish_non_exhaustive()
    }
}

impl PartialEq for FrozenIndexRealizationLawRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for FrozenIndexRealizationLawRegistry {}

pub(super) fn semantic_authorities_cohere(
    semantic: &FrozenSemanticRegistry,
    scalar_semantic: &FrozenSemanticRegistry,
) -> bool {
    semantic.snapshot_identity() == scalar_semantic.snapshot_identity()
        && semantic.encode_index_realization_law_sidecar()
            == scalar_semantic.encode_index_realization_law_sidecar()
}

impl FrozenIndexRealizationLawRegistry {
    /// Derives the law snapshot inseparably retained by one semantic registry.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch`]
    /// when `scalars` was built over a different semantic authority, including
    /// one with equal semantic snapshot bytes but different realization laws.
    pub fn from_semantic(
        semantic: FrozenSemanticRegistry,
        scalars: FrozenScalarRegistry,
    ) -> Result<Self, IndexRefinementVerificationError> {
        if !semantic_authorities_cohere(&semantic, scalars.semantic_authority()) {
            return Err(IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch);
        }
        let mut identity = Vec::new();
        identity.extend_from_slice(LAW_REGISTRY_IDENTITY_TAG);
        push_slice(&mut identity, semantic.snapshot_identity().as_bytes());
        push_slice(&mut identity, scalars.snapshot_identity().as_bytes());
        identity.extend_from_slice(&semantic.encode_index_realization_law_sidecar());
        Ok(Self(Arc::new(FrozenIndexRealizationLawRegistryData {
            semantic,
            scalars,
            identity: IndexRealizationLawRegistryIdentity(identity.into_boxed_slice()),
        })))
    }

    /// Returns the exact canonical registry identity.
    #[must_use]
    pub fn identity(&self) -> &IndexRealizationLawRegistryIdentity {
        &self.0.identity
    }

    /// Returns the semantic snapshot that owns every realization law.
    #[must_use]
    pub fn semantic_snapshot(&self) -> &SemanticRegistrySnapshotIdentity {
        self.0.semantic.snapshot_identity()
    }

    /// Returns the scalar snapshot under which every law is interpreted.
    #[must_use]
    pub fn scalar_snapshot(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        self.0.scalars.snapshot_identity()
    }

    /// Returns whether the law registered for one operation family realizes a
    /// region *sequence* rather than a single region.
    ///
    /// **Accepted public surface** — by Tom on 2026-08-06 at the live session's
    /// decision round, as-is with no exclusion;
    /// [`accept-the-registered-family-region-sequence-query`](../../../../../tickets/accept-the-registered-family-region-sequence-query.md)
    /// records the provenance.
    ///
    /// **The question is about the registered law and nothing else, which is why
    /// it takes an operation key rather than a subject.** A caller that already
    /// holds an [`IndexRefinementSubject`] asks
    /// [`ResolvedIndexRealization::realizes_region_sequence`] through
    /// [`Self::resolve`] and gets the same answer off the same registry row; a
    /// caller classifying a *program* — deciding whether an occurrence is one
    /// region's worth of work or a staged one — has no subject to derive,
    /// because deriving one requires a numerical contract and the classification
    /// does not depend on one. [`super::IndexRealizationLaw`]'s own predicate
    /// reads the variant alone, so answering it here from the operation key is
    /// the same fact read from the same place rather than a second derivation.
    ///
    /// Answers `false` for an operation the registry carries no law for. That is
    /// the fail-closed direction and not an approximation: an occurrence with no
    /// registered law realizes no region sequence this authority can describe,
    /// and refinement reports the absent law by name when the occurrence is
    /// lowered.
    ///
    /// [`super::IndexRealizationLaw`]: crate::index::IndexRealizationLaw
    #[must_use]
    pub fn family_realizes_region_sequence(&self, operation: &OpKey) -> bool {
        self.0
            .semantic
            .index_realization_law(operation)
            .is_some_and(|registered| registered.law.realizes_region_sequence())
    }

    /// Returns the law registered for one operation family, if it carries one.
    ///
    /// **Accepted public surface** — by Tom on 2026-08-06 at the live session's
    /// decision round, as-is with no exclusion;
    /// [`accept-the-registered-family-realization-law-query`](../../../../../tickets/accept-the-registered-family-realization-law-query.md)
    /// records the provenance.
    ///
    /// **Why a caller needs the law itself and not a predicate over it.** A
    /// physical planner spelling a staged family's stage has to know *what that
    /// stage computes* — which axes it folds, which payload its epilogue carries —
    /// and that is the law's content. Deriving it from the operation key instead
    /// would key the planner to a family, so a second family registering this law
    /// would need a second arm for one template; deriving it from the shapes is
    /// not possible at all, because a `[2, 2]` input reduced to `[2]` names two
    /// different reductions. Answering with the closed typed law is what lets a
    /// consumer be written against the *vocabulary* — one arm per law, a
    /// fail-closed wildcard for the rest — which is the same discipline this
    /// module's own interpretation follows.
    ///
    /// It takes an operation key for the reason
    /// [`Self::family_realizes_region_sequence`] does, reads the same registry row
    /// that method and [`Self::resolve`] read, and is deliberately *not* a
    /// resolution: it performs no contract check, no authority projection, and no
    /// realization, so it answers what is registered rather than what a subject
    /// may have. A caller acting on the answer still resolves.
    ///
    /// `None` for an operation the registry carries no law for, which is the
    /// fail-closed direction: an occurrence with no registered law has no
    /// realization this authority describes.
    #[must_use]
    pub fn family_realization_law(&self, operation: &OpKey) -> Option<&IndexRealizationLaw> {
        self.0
            .semantic
            .index_realization_law(operation)
            .map(|registered| &registered.law)
    }

    /// Resolves one semantic-provider-bound law from an exact subject.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when no governed contract capability exists or
    /// the subject came from another semantic authority.
    pub fn resolve(
        &self,
        subject: &IndexRefinementSubject,
    ) -> Result<ResolvedIndexRealization, IndexRefinementVerificationError> {
        let registry_row = self
            .0
            .semantic
            .encode_index_realization_law_row_for(&subject.operation);
        if registry_row != subject.realization_law_row {
            return Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch);
        }
        let registered = self
            .0
            .semantic
            .index_realization_law(&subject.operation)
            .ok_or(IndexRefinementVerificationError::MissingRealizationLaw)?;
        let actual = self
            .0
            .semantic
            .project_operation_occurrence_authority(
                &subject.operation,
                subject.signature.operands.iter(),
                subject.signature.results.iter(),
                &subject.attributes,
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        if actual != subject.semantic_authority {
            return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
        }
        if actual.registry_snapshot() != subject.semantic_authority.registry_snapshot() {
            return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
        }
        let mut law_identity = Vec::new();
        encode_op_key(&mut law_identity, &subject.operation);
        encode_provider(&mut law_identity, &registered.provider);
        law_identity.extend_from_slice(&registered.revision.to_be_bytes());
        registered.law.encode(&mut law_identity);
        let identity =
            encode_resolution_identity(&law_identity, &subject.identity).into_boxed_slice();
        Ok(ResolvedIndexRealization {
            registry: self.clone(),
            law: registered.law.clone(),
            provider: registered.provider.clone(),
            revision: registered.revision,
            subject: subject.clone(),
            identity,
        })
    }
}

/// One sealed independent-verifier resolution for an exact semantic subject.
#[derive(Clone)]
pub struct ResolvedIndexRealization {
    pub(super) registry: FrozenIndexRealizationLawRegistry,
    pub(super) law: IndexRealizationLaw,
    pub(super) provider: ProviderIdentity,
    pub(super) revision: u32,
    pub(super) subject: IndexRefinementSubject,
    pub(super) identity: Box<[u8]>,
}

impl fmt::Debug for ResolvedIndexRealization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedIndexRealization")
            .field("registry", &self.registry)
            .field("law", &self.law)
            .field("provider", &self.provider)
            .field("revision", &self.revision)
            .field("subject", &self.subject)
            .field("identity", &self.identity)
            .finish()
    }
}

impl PartialEq for ResolvedIndexRealization {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for ResolvedIndexRealization {}

impl ResolvedIndexRealization {
    /// Returns the exact governed subject.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        &self.subject
    }
    /// Returns whether this law realizes a region *sequence*.
    ///
    /// The cheap half of [`Self::realize_sequence`]'s answer: a consumer that
    /// only wants single-region occurrences filtered out asks this before
    /// paying for a realization it would discard.
    #[must_use]
    pub const fn realizes_region_sequence(&self) -> bool {
        self.law.realizes_region_sequence()
    }

    /// Realizes the resolved law's canonical region sequence for its subject.
    ///
    /// The same realization refinement performs internally when it compares a
    /// provider's emission against the law, over the same law, subject, and
    /// frozen scalar authority this resolution already binds — exposed so a
    /// consumer that needs the realization's *shape* (its stage count, each
    /// stage's reads, and the handed values) reads it from the one authority
    /// that owns it instead of deriving a second account of the law.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::SemanticRealizationLawRefused`]
    /// carrying the law's own refusal rule when the subject does not realize —
    /// the identical refusal refinement reports for the same subject.
    pub fn realize_sequence(
        &self,
    ) -> Result<VerifiedIndexRegionSequence, IndexRefinementVerificationError> {
        self.law
            .realize_sequence(&self.subject, &self.registry.0.scalars)
            .map_err(
                |source| IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(self.subject.operation().clone()),
                    rule: source.rule(),
                },
            )
    }
    /// Returns the independent verifier provider.
    #[must_use]
    pub fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }
    /// Returns the independent verifier revision.
    #[must_use]
    pub fn revision(&self) -> u32 {
        self.revision
    }
}
