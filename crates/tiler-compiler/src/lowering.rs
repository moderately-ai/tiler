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
//! - **Refinement is exhaustive finite evidence, attached when the proof budget
//!   affords it.** `IndexRegionBuilder::build` proves bounds and write ownership
//!   by enumerating the access domain whenever the cheaper interval proof fails
//!   or a write is not a proved coordinate permutation, charged against
//!   `tiler_ir::index::MAX_EXHAUSTIVE_PROOF_CELLS`. A region that exceeds it has
//!   not been disproved; the analysis simply stopped. Reporting that as a
//!   rejection would confuse an exhausted analysis budget with hard
//!   infeasibility, so the stage records a typed budget stop and an explicit
//!   `Unknown` gap and leaves the plan standing.
//!
//! An emitted region that is malformed, or that is well formed but does not
//! realize the occurrence, is a genuine rejection and does fail closed: the
//! artifact plan names the resolved provider as the occurrence's lowering
//! authority, and that claim must be true.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::index::{IndexRegionDiagnostic, ProofResource};
use tiler_ir::semantic::{OpKey, SemanticProgram};

use crate::capability::{LoweringResolveError, LoweringSignature, ResolvedLoweringCapability};
use crate::legality::{
    IndexRefinement, NumericalContractIdentity, OccurrenceOperand, OccurrenceResult,
    OccurrenceValueId, RefinementError, SemanticOccurrence, SemanticOccurrenceIdentity,
    refine_index_region,
};
use crate::region::{RegionFormationOutcome, SemanticMemberId};
use crate::request::{LoweringProviderIdentity, VerifiedTargetRequest};

/// The proof budget one occurrence's refinement exhausted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RefinementBudgetStop {
    /// The exhausted exhaustive-proof resource.
    pub(crate) resource: ProofResource,
    /// The governed limit that stopped the proof.
    pub(crate) limit: u64,
    /// The amount the proof would have required.
    pub(crate) required: u128,
}

impl RefinementBudgetStop {
    /// Returns the stable explain resource key of the exhausted resource.
    pub(crate) const fn resource_key(self) -> &'static str {
        match self.resource {
            ProofResource::Cells => "index-proof-cells",
            ProofResource::IntegerBytes => "index-proof-integer-bytes",
            // `ProofResource` is `#[non_exhaustive]`; an unrecognized resource
            // still names a real stop, so it is reported under a stable generic
            // key rather than being dropped from the trace.
            _ => "index-proof-resource",
        }
    }
}

/// The evidence one recognized occurrence's lowering rests on.
#[derive(Clone, Debug)]
pub(crate) enum OccurrenceEvidence {
    /// The provider's emitted region was proved to realize the occurrence.
    Refined(Box<IndexRefinement>),
    /// The exhaustive access proof stopped before the region could be verified.
    ///
    /// This is an `Unknown` gap, not a pass and not a rejection: no refinement
    /// evidence exists for the occurrence and none was disproved.
    BudgetStopped(RefinementBudgetStop),
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
    /// The resolved provider's region does not realize the occurrence.
    Refine {
        /// Recognized member whose refinement was refused.
        member: SemanticMemberId,
        /// Typed refinement cause.
        source: Arc<RefinementError>,
    },
}

impl LoweringError {
    /// Returns the recognized member the failure is attributed to.
    pub(crate) const fn member(&self) -> SemanticMemberId {
        match self {
            Self::Occurrence { member, .. }
            | Self::Resolve { member, .. }
            | Self::Refine { member, .. } => *member,
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
            Self::Refine { member, source } => write!(
                formatter,
                "compile.lowering.refinement: member {} was not realized: {source}",
                member.0
            ),
        }
    }
}

impl Error for LoweringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Occurrence { .. } => None,
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
                Self::Refine { member, source },
                Self::Refine {
                    member: other_member,
                    source: other_source,
                },
            ) => member == other_member && source == other_source,
            _ => false,
        }
    }
}

impl Eq for LoweringError {}

/// Resolves and refines every recognized occurrence's index-access lowering.
///
/// Resolution runs for all occurrences and fails closed. Refinement runs for
/// each resolved capability and attaches occurrence-bound evidence, degrading to
/// a recorded proof-budget stop — never to a rejection — when the exhaustive
/// access proof cannot afford the region.
///
/// # Errors
///
/// Returns [`LoweringError`] when the recognized program cannot be projected
/// into occurrence facts, when a capability is missing or contended, or when a
/// resolved provider's region does not realize its occurrence.
pub(crate) fn resolve_lowering(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    formation: &RegionFormationOutcome,
) -> Result<ResolvedLowering, LoweringError> {
    let capabilities = request.capabilities();
    let contract = NumericalContractIdentity::from_key(request.numerical_contract().key);
    let mut occurrences = Vec::new();
    for member in request.normalized().all_members() {
        let identity = singleton_occurrence_identity(formation, member)?;
        let occurrence = project_occurrence(semantic, member, &contract, identity)?;
        let resolved = resolve_occurrence(capabilities, &occurrence, member)?;
        let evidence = refine(&resolved, &occurrence, capabilities.scalars(), member)?;
        occurrences.push(OccurrenceLowering {
            member,
            operation: occurrence.operation().clone(),
            provider: LoweringProviderIdentity::new(
                resolved.provider().clone(),
                governed_capability_key(&resolved),
                resolved.revision(),
            ),
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
    let contract = NumericalContractIdentity::from_key(request.numerical_contract().key);
    let mut providers = Vec::new();
    for member in request.normalized().all_members() {
        // Resolution never reads the semantic-source identity, so re-deriving it
        // here would be an unused cost; the empty source keeps this path
        // independent of region formation.
        let occurrence = project_occurrence(
            semantic,
            member,
            &contract,
            SemanticOccurrenceIdentity::from_bytes(Vec::new()),
        )?;
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
    occurrence: &SemanticOccurrence,
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

/// Returns the semantic-source identity region formation gave one member.
fn singleton_occurrence_identity(
    formation: &RegionFormationOutcome,
    member: SemanticMemberId,
) -> Result<SemanticOccurrenceIdentity, LoweringError> {
    // The singleton region candidate for this member is the semantic source
    // identity the whole compilation already agreed on; refinement never invents
    // its own naming for the same occurrence.
    formation
        .candidates()
        .iter()
        .find(|candidate| candidate.members() == [member])
        .map(|candidate| {
            SemanticOccurrenceIdentity::from_bytes(candidate.occurrence().as_bytes().to_vec())
        })
        .ok_or(LoweringError::Occurrence {
            rule: "singleton-candidate",
            member,
        })
}

/// Refines one resolved capability, degrading a proof-budget stop to a gap.
fn refine(
    resolved: &ResolvedLoweringCapability,
    occurrence: &SemanticOccurrence,
    scalars: &tiler_ir::index::FrozenScalarRegistry,
    member: SemanticMemberId,
) -> Result<OccurrenceEvidence, LoweringError> {
    match refine_index_region(resolved, occurrence, scalars) {
        Ok(refinement) => Ok(OccurrenceEvidence::Refined(Box::new(refinement))),
        Err(RefinementError::Build { diagnostics }) => match proof_budget_stop(&diagnostics) {
            Some(stop) => Ok(OccurrenceEvidence::BudgetStopped(stop)),
            None => Err(LoweringError::Refine {
                member,
                source: Arc::new(RefinementError::Build { diagnostics }),
            }),
        },
        Err(source) => Err(LoweringError::Refine {
            member,
            source: Arc::new(source),
        }),
    }
}

/// Returns the proof-budget stop when that is the *only* thing verification found.
///
/// A budget stop next to any other diagnostic is not a budget stop: the region
/// was independently rejected, and reporting the pair as an unknown gap would
/// hide a real refusal behind an exhausted analysis.
fn proof_budget_stop(diagnostics: &[IndexRegionDiagnostic]) -> Option<RefinementBudgetStop> {
    let mut stop = None;
    for diagnostic in diagnostics {
        let IndexRegionDiagnostic::ProofResourceLimit {
            resource,
            required,
            limit,
        } = diagnostic
        else {
            return None;
        };
        stop.get_or_insert(RefinementBudgetStop {
            resource: *resource,
            limit: *limit,
            required: *required,
        });
    }
    stop
}

/// Projects one recognized member into the occurrence refinement is bound to.
fn project_occurrence(
    semantic: &SemanticProgram,
    member: SemanticMemberId,
    contract: &NumericalContractIdentity,
    identity: SemanticOccurrenceIdentity,
) -> Result<SemanticOccurrence, LoweringError> {
    let malformed = |rule: &'static str| LoweringError::Occurrence { rule, member };
    let ordinal = usize::try_from(member.0).map_err(|_| malformed("member-ordinal"))?;
    let operation = semantic
        .operations()
        .nth(ordinal)
        .ok_or_else(|| malformed("member-ordinal"))?;
    let definition = semantic
        .semantic_registry()
        .operation_definition(operation.key())
        .ok_or_else(|| malformed("operation-definition"))?;
    // The occurrence-local value names only have to distinguish aliases, so they
    // are first-occurrence positions rather than graph ordinals. Two operands
    // carrying one graph value therefore lower to one input boundary without the
    // refinement authority ever seeing a graph-local identifier.
    let mut seen: Vec<tiler_ir::semantic::ValueId> = Vec::new();
    let mut operands = Vec::new();
    for value in operation.operands() {
        let reference = semantic.value(value).map_err(|_| malformed("operand"))?;
        let shape = semantic.shape(value).map_err(|_| malformed("operand"))?;
        let local = seen
            .iter()
            .position(|seen| *seen == value)
            .unwrap_or_else(|| {
                seen.push(value);
                seen.len() - 1
            });
        let local = u32::try_from(local).map_err(|_| malformed("operand-alias"))?;
        operands.push(OccurrenceOperand::new(
            OccurrenceValueId(local),
            reference.resolved_type().clone(),
            shape.clone(),
        ));
    }
    let mut results = Vec::new();
    for value in operation.results() {
        let reference = semantic.value(value).map_err(|_| malformed("result"))?;
        let shape = semantic.shape(value).map_err(|_| malformed("result"))?;
        results.push(OccurrenceResult::new(
            reference.resolved_type().clone(),
            shape.clone(),
        ));
    }
    Ok(SemanticOccurrence::new(
        operation.key().clone(),
        operands,
        results,
        operation.attributes().clone(),
        definition.effect(),
        contract.clone(),
        identity,
    ))
}

/// Derives the exact resolution signature of one occurrence.
fn occurrence_signature(
    occurrence: &SemanticOccurrence,
    member: SemanticMemberId,
) -> Result<LoweringSignature, LoweringError> {
    LoweringSignature::new(
        occurrence
            .operands()
            .iter()
            .map(|operand| operand.value_type().clone()),
        occurrence
            .results()
            .iter()
            .map(|result| result.value_type().clone()),
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
