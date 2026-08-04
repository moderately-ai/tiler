//! The compile path's lowering-capability resolution and refinement stage.
//!
//! Every recognized semantic occurrence resolves exactly one
//! [`LoweringFamily::IndexAccess`] capability through the frozen registry the
//! request carries, and the resolved provider is then driven through
//! [`refine_index_region`] to prove it realizes that occurrence. The two answer
//! different questions and are kept apart accordingly:
//!
//! - **Resolution is unconditional and fails closed.** A missing or contended
//!   capability means the installed authority cannot lower a program it was
//!   handed, which is a typed compile error with an explainable cause. There is
//!   no approximate provider and no silent default.
//! - **Refinement requires complete evidence.** `IndexRegionBuilder::build` may
//!   return a structurally verified region with exact residual semantic
//!   predicates, but such a region is not refinement evidence. Until a later
//!   semantic-discharge stage proves those predicates, lowering fails closed
//!   before an executable frontier or artifact can be formed.
//!
//! An emitted region that is malformed, or that is well formed but does not
//! realize the occurrence, is a genuine rejection and does fail closed: the
//! artifact plan names the resolved provider as the occurrence's lowering
//! authority, and that claim must be true.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::index::{IndexRefinementSubject, NumericalContractIdentity};
use tiler_ir::program::SemanticOccurrence;
use tiler_ir::semantic::{OpKey, SemanticProgram};

use crate::capability::{LoweringResolveError, LoweringSignature, ResolvedLoweringCapability};
use crate::index_discharge::{
    IndexDomainDischargeError, IndexDomainDischargeRefusal, IndexDomainDischargeRefusalKind,
    discharge_pending_index_refinement,
};
use crate::legality::{
    IndexRefinement, IndexRefinementOutcome, RefinementError, refine_index_region,
};
use crate::region::SemanticMemberId;
use crate::request::{LoweringProviderIdentity, VerifiedTargetRequest};

/// The evidence one recognized occurrence's lowering rests on.
#[derive(Clone, Debug)]
pub(crate) enum OccurrenceEvidence {
    /// The provider's emitted region was proved to realize the occurrence.
    Refined(Box<IndexRefinement>),
}

/// One recognized occurrence, its resolved capability, and its evidence.
#[derive(Clone, Debug)]
pub(crate) struct OccurrenceLowering {
    member: SemanticMemberId,
    operation: OpKey,
    provider: LoweringProviderIdentity,
    evidence: OccurrenceEvidence,
}

impl OccurrenceLowering {
    /// Returns the recognized member this lowering realizes.
    pub(crate) const fn member(&self) -> SemanticMemberId {
        self.member
    }

    /// Returns the resolved provider and capability revision.
    pub(crate) const fn provider(&self) -> &LoweringProviderIdentity {
        &self.provider
    }

    /// Returns the occurrence-bound refinement evidence or its recorded gap.
    pub(crate) const fn evidence(&self) -> &OccurrenceEvidence {
        &self.evidence
    }

    /// Returns the canonical semantic occurrence proved by this lowering.
    pub(crate) const fn canonical_occurrence(&self) -> SemanticOccurrence {
        match &self.evidence {
            OccurrenceEvidence::Refined(refinement) => refinement.receipt().occurrence(),
        }
    }

    /// Returns the stable explain subject key of this occurrence.
    pub(crate) fn subject_key(&self) -> String {
        format!("occurrence:{}/{}", self.member.0, self.operation)
    }
}

/// Every recognized occurrence's resolved lowering, in ascending member order.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedLowering {
    occurrences: Vec<OccurrenceLowering>,
}

impl ResolvedLowering {
    /// Returns the per-occurrence lowerings in ascending member order.
    pub(crate) fn occurrences(&self) -> &[OccurrenceLowering] {
        &self.occurrences
    }

    /// Resolves one dense storage member in O(1).
    pub(crate) fn occurrence(&self, member: SemanticMemberId) -> Option<&OccurrenceLowering> {
        self.occurrences
            .get(usize::try_from(member.0).expect("u32 fits every supported host usize"))
            .filter(|occurrence| occurrence.member == member)
    }

    /// Returns the distinct resolved providers in canonical ascending order.
    ///
    /// This is the lowering provenance an artifact plan records: several
    /// occurrences of one family resolve one capability and contribute one
    /// entry, while one provider owning two capabilities at different revisions
    /// contributes two.
    pub(crate) fn providers(&self) -> Vec<LoweringProviderIdentity> {
        let mut providers: Vec<_> = self
            .occurrences
            .iter()
            .map(|occurrence| occurrence.provider.clone())
            .collect();
        providers.sort_unstable();
        providers.dedup();
        providers
    }
}

/// A failure to resolve or refine one recognized occurrence's lowering.
#[derive(Clone, Debug)]
pub(crate) enum LoweringError {
    /// The recognized program could not be projected into occurrence facts.
    Occurrence {
        /// Stable rule identifier of the malformed projection.
        rule: &'static str,
        /// Recognized member the projection failed for.
        member: SemanticMemberId,
    },
    /// No installed capability lowers the occurrence, or more than one does.
    Resolve {
        /// Recognized member whose capability failed to resolve.
        member: SemanticMemberId,
        /// Typed registry cause.
        source: Box<LoweringResolveError>,
    },
    /// The resolved provider's region could not establish refinement evidence.
    Refine {
        /// Recognized member whose refinement was refused.
        member: SemanticMemberId,
        /// Resolved provider and capability revision that emitted the region.
        provider: LoweringProviderIdentity,
        /// Typed refinement cause.
        source: Arc<RefinementError>,
    },
    /// Named semantic discharge refused one or more residual predicates.
    SemanticDischarge {
        /// Recognized member whose realization failed semantic discharge.
        member: SemanticMemberId,
        /// Resolved provider and capability revision that emitted the region.
        provider: LoweringProviderIdentity,
        /// Exact typed assessments and retained pending state.
        refusal: Box<IndexDomainDischargeRefusal>,
    },
}

impl LoweringError {
    /// Returns the recognized member the failure is attributed to.
    pub(crate) const fn member(&self) -> SemanticMemberId {
        match self {
            Self::Occurrence { member, .. }
            | Self::Resolve { member, .. }
            | Self::Refine { member, .. }
            | Self::SemanticDischarge { member, .. } => *member,
        }
    }

    /// Returns the stable reason code of this failure class.
    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::Occurrence { rule, .. } => rule,
            Self::Resolve { source, .. } => match **source {
                LoweringResolveError::MissingCapability { .. } => "missing-capability",
                LoweringResolveError::AmbiguousCapability { .. } => "ambiguous-capability",
            },
            Self::SemanticDischarge { refusal, .. } => match refusal.kind() {
                IndexDomainDischargeRefusalKind::Disproved => "index-domain-disproved",
                IndexDomainDischargeRefusalKind::Unknown => "index-domain-discharge-unsupported",
            },
            Self::Refine { .. } => "refinement-refused",
        }
    }

    /// Returns whether the installed authority holds no capability at all.
    ///
    /// An absent capability and a contended one are different findings: the
    /// first says the authority was never extended to this occurrence, the
    /// second says two extensions contradict each other. Only the first is a
    /// deferred capability.
    pub(crate) fn is_missing(&self) -> bool {
        matches!(
            self,
            Self::Resolve { source, .. }
                if matches!(**source, LoweringResolveError::MissingCapability { .. })
        )
    }

    /// Returns the exact semantic-discharge refusal, when residuals stopped lowering.
    pub(crate) fn semantic_discharge(
        &self,
    ) -> Option<(&LoweringProviderIdentity, &IndexDomainDischargeRefusal)> {
        let Self::SemanticDischarge {
            provider, refusal, ..
        } = self
        else {
            return None;
        };
        Some((provider, refusal))
    }
}

impl fmt::Display for LoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occurrence { rule, member } => write!(
                formatter,
                "compile.lowering.occurrence.{rule}: member {} is not a lowerable occurrence",
                member.0
            ),
            Self::Resolve { member, source } => write!(
                formatter,
                "compile.lowering.capability: member {} did not resolve: {source}",
                member.0
            ),
            Self::Refine { member, source, .. } => write!(
                formatter,
                "compile.lowering.refinement: member {} was not realized: {source}",
                member.0
            ),
            Self::SemanticDischarge {
                member, refusal, ..
            } => write!(
                formatter,
                "compile.lowering.semantic-discharge: member {} was refused: {}",
                member.0, refusal
            ),
        }
    }
}

impl Error for LoweringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Occurrence { .. } | Self::SemanticDischarge { .. } => None,
            Self::Resolve { source, .. } => Some(source.as_ref()),
            Self::Refine { source, .. } => Some(source.as_ref()),
        }
    }
}

impl PartialEq for LoweringError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Occurrence { rule, member },
                Self::Occurrence {
                    rule: other_rule,
                    member: other_member,
                },
            ) => rule == other_rule && member == other_member,
            (
                Self::Resolve { member, source },
                Self::Resolve {
                    member: other_member,
                    source: other_source,
                },
            ) => member == other_member && source == other_source,
            (
                Self::Refine {
                    member,
                    provider,
                    source,
                },
                Self::Refine {
                    member: other_member,
                    provider: other_provider,
                    source: other_source,
                },
            ) => member == other_member && provider == other_provider && source == other_source,
            (
                Self::SemanticDischarge {
                    member,
                    provider,
                    refusal,
                },
                Self::SemanticDischarge {
                    member: other_member,
                    provider: other_provider,
                    refusal: other_refusal,
                },
            ) => member == other_member && provider == other_provider && refusal == other_refusal,
            _ => false,
        }
    }
}

impl Eq for LoweringError {}

/// Resolves and refines every recognized occurrence's index-access lowering.
///
/// Resolution runs for all occurrences and fails closed. Refinement runs for
/// each resolved capability and attaches occurrence-bound evidence. A
/// structurally verified region with unresolved semantic predicates is not
/// refinement evidence and fails closed.
///
/// # Errors
///
/// Returns [`LoweringError`] when the recognized program cannot be projected
/// into occurrence facts, when a capability is missing or contended, or when a
/// resolved provider's region does not realize its occurrence.
pub(crate) fn resolve_lowering(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
) -> Result<ResolvedLowering, LoweringError> {
    let capabilities = request.capabilities();
    let contract = NumericalContractIdentity::try_from_key(request.numerical_contract().key)
        .expect("verified compiler contract keys satisfy the IR bound");
    let mut occurrences = Vec::new();
    for member in request.normalized().all_members() {
        let occurrence = project_occurrence(semantic, member, &contract)?;
        let resolved = resolve_occurrence(capabilities, &occurrence, member)?;
        let provider = LoweringProviderIdentity::new(
            resolved.provider().clone(),
            governed_capability_key(&resolved),
            resolved.revision(),
        );
        let evidence = refine(
            &resolved,
            &occurrence,
            request.realization_laws(),
            capabilities.scalars(),
            member,
            provider.clone(),
        )?;
        occurrences.push(OccurrenceLowering {
            member,
            operation: occurrence.operation().clone(),
            provider,
            evidence,
        });
    }
    Ok(ResolvedLowering { occurrences })
}

/// Re-derives only the lowering provenance every recognized occurrence resolves.
///
/// This is the cheap half of [`resolve_lowering`]: it repeats the registry
/// resolution without driving any provider, so an artifact receipt can be
/// checked against the installed authority without re-emitting and re-verifying
/// every region.
///
/// # Errors
///
/// Returns [`LoweringError`] when the recognized program cannot be projected
/// into occurrence facts or when a capability is missing or contended.
pub(crate) fn resolve_capabilities(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
) -> Result<Vec<LoweringProviderIdentity>, LoweringError> {
    let capabilities = request.capabilities();
    let contract = NumericalContractIdentity::try_from_key(request.numerical_contract().key)
        .expect("verified compiler contract keys satisfy the IR bound");
    let mut providers = Vec::new();
    for member in request.normalized().all_members() {
        let occurrence = project_occurrence(semantic, member, &contract)?;
        let resolved = resolve_occurrence(capabilities, &occurrence, member)?;
        providers.push(LoweringProviderIdentity::new(
            resolved.provider().clone(),
            governed_capability_key(&resolved),
            resolved.revision(),
        ));
    }
    providers.sort_unstable();
    providers.dedup();
    Ok(providers)
}

/// Resolves the exact index-access capability one occurrence requires.
fn resolve_occurrence(
    capabilities: &crate::request::CompilerCapabilitySnapshot,
    occurrence: &IndexRefinementSubject,
    member: SemanticMemberId,
) -> Result<ResolvedLoweringCapability, LoweringError> {
    let signature = occurrence_signature(occurrence, member)?;
    capabilities
        .lowering()
        .resolve_index_access(occurrence.operation(), &signature)
        .map_err(|source| LoweringError::Resolve {
            member,
            source: Box::new(source),
        })
}

/// Refines one resolved capability.
fn refine(
    resolved: &ResolvedLoweringCapability,
    occurrence: &IndexRefinementSubject,
    realizations: &tiler_ir::index::FrozenIndexRealizationLawRegistry,
    scalars: &tiler_ir::index::FrozenScalarRegistry,
    member: SemanticMemberId,
    provider: LoweringProviderIdentity,
) -> Result<OccurrenceEvidence, LoweringError> {
    match refine_index_region(resolved, occurrence, realizations, scalars).map_err(|source| {
        LoweringError::Refine {
            member,
            provider: provider.clone(),
            source: Arc::new(source),
        }
    })? {
        IndexRefinementOutcome::Refined(refinement) => Ok(OccurrenceEvidence::Refined(refinement)),
        IndexRefinementOutcome::Pending(pending) => {
            match discharge_pending_index_refinement(*pending) {
                Ok(refinement) => Ok(OccurrenceEvidence::Refined(Box::new(refinement))),
                Err(IndexDomainDischargeError::Domain(refusal)) => {
                    Err(LoweringError::SemanticDischarge {
                        member,
                        provider,
                        refusal: Box::new(refusal),
                    })
                }
                Err(IndexDomainDischargeError::Refinement(source)) => Err(LoweringError::Refine {
                    member,
                    provider,
                    source: Arc::new(source),
                }),
            }
        }
    }
}

/// Projects one recognized member into the occurrence refinement is bound to.
fn project_occurrence(
    semantic: &SemanticProgram,
    member: SemanticMemberId,
    contract: &NumericalContractIdentity,
) -> Result<IndexRefinementSubject, LoweringError> {
    let operation = semantic
        .operations()
        .nth(usize::try_from(member.0).expect("u32 fits every supported host usize"))
        .ok_or(LoweringError::Occurrence {
            rule: "refinement-subject-selector",
            member,
        })?
        .id();
    IndexRefinementSubject::derive(semantic, operation, contract.clone()).map_err(|_| {
        LoweringError::Occurrence {
            rule: "refinement-subject",
            member,
        }
    })
}

/// Derives the exact resolution signature of one occurrence.
fn occurrence_signature(
    occurrence: &IndexRefinementSubject,
    member: SemanticMemberId,
) -> Result<LoweringSignature, LoweringError> {
    LoweringSignature::new(
        occurrence.signature().operands().iter().cloned(),
        occurrence.signature().results().iter().cloned(),
    )
    .map_err(|_| LoweringError::Occurrence {
        rule: "signature-bound",
        member,
    })
}

/// Mints the governed key of one resolved lowering capability.
///
/// The spelling names the capability family and the exact semantic operation
/// family it lowers, including that operation's semantic version, so two
/// versions of one operation never share a key.
///
/// # What is deliberately not in the key, and what keeps that safe
///
/// The resolved **signature** is excluded. A capability is registered under
/// family, operation, signature, and provider (`capability.rs`'s
/// `LoweringCapabilityKey`), so two capabilities from one provider differing
/// only in signature would mint the same key here. Signatures are unbounded
/// structural values and a governed key is bounded at 256 bytes, so folding one
/// in would either truncate — silently colliding, which is worse than the
/// conflation because it would *look* distinguishing — or require a digest,
/// which is a second identity that must be kept in agreement with the signature
/// it summarizes.
///
/// The exclusion is therefore kept, and the assumption it rests on is enforced
/// rather than recorded: every consumer stores the provider beside this key, so
/// the pair names one capability exactly while one provider registers one
/// signature per family and operation, and
/// `LoweringRegistryError::ConflatedCapabilityKey` refuses the registration that
/// would make that false. Two *different* providers may still differ in
/// signature for one operation, because the recorded provider distinguishes them.
///
/// The consequence is a real restriction: a provider cannot register per-shape
/// or per-attribute signatures for one operation family. Admitting those means
/// deciding a bounded signature encoding for this key first, which is a decision
/// someone makes rather than a property that quietly stops holding
/// (`resolve-capability-key-signature-conflation`).
///
/// # This composition can mint a key the artifact layer refuses
///
/// The two interpolated components come from an `OpKey`, whose validator
/// (`tiler_ir::semantic::types`) admits ASCII *alphanumeric* — uppercase
/// included — and 255 bytes per component. `tiler_artifact::program`'s governed
/// keys admit ASCII lowercase within 256 bytes total. So a legal `OpKey` such
/// as `Acme::MyOp`, registered through the public `register_scalar_lowering`,
/// composes a capability key that `CapabilityKey::new` refuses at packaging
/// time, and two long components compose one past the byte bound. This function
/// is infallible and cannot report either, so the refusal lands at the
/// packaging call rather than at the registration that caused it.
///
/// Refusing is correct — an uppercase key would compare unequal to the one a
/// reader sees — but the site is wrong, and choosing between narrowing the
/// operation-identity grammar and making this composition fallible is a public
/// boundary decision. `reconcile-the-operation-identity-and-governed-key-grammars`
/// owns it.
fn governed_capability_key(resolved: &ResolvedLoweringCapability) -> String {
    let operation = resolved.operation();
    format!(
        "tiler.capability.{}.{}.{}.v{}",
        resolved.family().key_token(),
        operation.namespace(),
        operation.name(),
        operation.semantic_version(),
    )
}
