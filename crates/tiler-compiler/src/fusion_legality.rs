//! Derived fusion-legality authority for one proposed region occurrence.
//!
//! Region formation proposes candidates; this module answers a different
//! question about one of them: whether implementing that region as a single
//! fused kernel preserves the request's numerical contract exactly. Unlike a
//! graph-shape recognizer or a fixed proof label, legality here is *derived*.
//! For every member operation the derivation resolves a per-operation numerical
//! capability (its fusion role), then discharges each numerical, effect, and
//! materialization obligation against that role, the reached semantic
//! definition, and the effective numerical policy. The result is one of three
//! typed outcomes:
//!
//! - [`FusionLegality::Legal`] carries replayable evidence: every obligation is
//!   discharged with a labelled [`FusionEvidenceClass`];
//! - [`FusionLegality::Rejected`] names the obligation a fused realization is
//!   proved to violate; and
//! - [`FusionLegality::Unknown`] names the obligation the bounded profile cannot
//!   yet establish, failing closed rather than approximating an accept.
//!
//! The proof separates two identities that must never be conflated, mirroring
//! the region and refinement authorities:
//!
//! - [`FusionLegalityContent`] is reusable and site/provider-independent: the
//!   canonical region-content identity, the numerical-contract key, the derived
//!   structural counts, and the ordered discharged obligations with their
//!   evidence classes. It contains no selected provider and no graph site.
//! - [`FusionLegalityProof`] binds that content to one exact occurrence: the
//!   region-occurrence identity, the reached semantic definitions, the selected
//!   fusion-capability provider, and the ordered value/access bindings.
//!
//! The five evidence classes named by the correctness contract — normative
//! guarantee, sound proof, exhaustive-finite, empirical, and unknown — are kept
//! distinct and are never collapsed into one another.
//!
//! Scope boundary: this authority derives legality of one candidate. It selects
//! no cover, chooses no physical implementation, schedules nothing, and costs
//! nothing. Every item is a reviewed *draft* boundary, not a stable compiler
//! API, until Tom accepts the exact interface.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::schedule::NumericalPermission;
use tiler_ir::semantic::{
    F32, FrozenSemanticRegistry, OpKey, OperationEffect, ProviderIdentity, SemanticProgram,
    add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};

use crate::region::{
    MemberOperationFacts, RegionCandidate, RegionContentIdentity, RegionError, RegionGraph,
    RegionOccurrenceIdentity, SemanticMemberId, verify_candidate,
};
use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

/// Canonical domain-separation tag for reusable fusion-legality content.
const CONTENT_IDENTITY_TAG: &[u8] = b"tiler.compiler.fusion-legality-content.v1\0";
/// Canonical domain-separation tag for one fusion-legality occurrence binding.
const OCCURRENCE_IDENTITY_TAG: &[u8] = b"tiler.compiler.fusion-legality-occurrence.v1\0";
/// Namespace of the governed compiler-owned fusion-capability provider.
const GOVERNED_PROVIDER_NAMESPACE: &str = "tiler";
/// Name of the governed compiler-owned fusion-capability provider.
const GOVERNED_PROVIDER_NAME: &str = "fusion-strict-f32";
/// Output-affecting revision of the governed fusion-capability provider.
const GOVERNED_PROVIDER_REVISION: u32 = 1;

/// The class of evidence that discharged, rejected, or failed to establish one
/// obligation.
///
/// The five classes are deliberately distinct maturity claims and are never
/// collapsed: a sound proof is not empirical, and an unknown is not a normative
/// guarantee. The bounded strict-`f32` profile constructs a subset; the
/// remaining classes are reserved so that a future obligation discharged by
/// finite enumeration or measurement declares itself honestly rather than
/// masquerading as a proof.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(
    dead_code,
    reason = "reserved evidence classes; the bounded profile discharges every obligation by checked invariant, and exhaustive-finite and empirical evidence stay distinct classes a later profile produces"
)]
pub(crate) enum FusionEvidenceClass {
    /// The reached operation's normative definition guarantees the property.
    NormativeGuarantee,
    /// Soundly derived from the verified region structure and numerical policy.
    SoundProof,
    /// Established by exhaustively enumerating a finite domain.
    ///
    /// Reserved: no bounded strict-`f32` obligation discharges this way yet, but
    /// the class is kept distinct so a future finite-domain proof declares
    /// itself honestly rather than masquerading as a sound proof.
    ExhaustiveFinite,
    /// Established only by empirical measurement under a named profile.
    ///
    /// Reserved: kept distinct so a future measured qualification cannot be
    /// mistaken for a proof or a normative guarantee.
    Empirical,
    /// The property could not be established in this bounded profile.
    Unknown,
}

#[allow(
    dead_code,
    reason = "reserved evidence classes; the bounded profile discharges every obligation by checked invariant, and exhaustive-finite and empirical evidence stay distinct classes a later profile produces"
)]
impl FusionEvidenceClass {
    /// Returns the stable identity tag shared by ordering and encoding.
    const fn tag(self) -> u8 {
        match self {
            Self::NormativeGuarantee => 1,
            Self::SoundProof => 2,
            Self::ExhaustiveFinite => 3,
            Self::Empirical => 4,
            Self::Unknown => 5,
        }
    }

    /// Returns the stable presentation name of the evidence class.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::NormativeGuarantee => "normative-guarantee",
            Self::SoundProof => "sound-proof",
            Self::ExhaustiveFinite => "exhaustive-finite",
            Self::Empirical => "empirical",
            Self::Unknown => "unknown",
        }
    }
}

/// The fusion role of one operation family, resolved from its capability.
///
/// The role is the per-operation capability the derivation consults instead of
/// recognizing a whole-graph shape. It fails closed: an operation family with no
/// registered role yields no fusion legality at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FusionOperationRole {
    /// A constant or boundary read: it contributes a value and no reordering,
    /// conversion, or reduction obligation of its own.
    ValueSource,
    /// A separate-rounding elementwise arithmetic operation.
    ElementwiseArithmetic,
    /// A strict lexicographic left-fold reduction with a defined identity.
    OrderedReduction,
}

impl FusionOperationRole {
    const fn is_arithmetic(self) -> bool {
        matches!(self, Self::ElementwiseArithmetic)
    }

    const fn is_reduction(self) -> bool {
        matches!(self, Self::OrderedReduction)
    }

    const fn is_value_source(self) -> bool {
        matches!(self, Self::ValueSource)
    }
}

/// A compiler-owned registry of per-operation fusion numerical capabilities.
///
/// It maps an operation family key to the fusion role the governed provider
/// declares for it. Resolution is a checked lookup, not a graph-shape match, so
/// coverage grows one operation at a time and any unregistered family fails
/// closed to [`FusionLegality::Unknown`].
#[derive(Clone, Debug)]
pub(crate) struct FusionNumericalCapabilities {
    provider: ProviderIdentity,
    revision: u32,
    roles: BTreeMap<OpKey, FusionOperationRole>,
}

impl FusionNumericalCapabilities {
    /// Builds the governed strict-`f32` fusion-capability registry.
    ///
    /// The governed provider declares the initial profile's constant,
    /// elementwise arithmetic, and strict serial-sum reduction roles. No other
    /// operation family has a fusion capability.
    #[must_use]
    pub(crate) fn governed() -> Self {
        let provider = ProviderIdentity::new(
            GOVERNED_PROVIDER_NAMESPACE,
            GOVERNED_PROVIDER_NAME,
            GOVERNED_PROVIDER_REVISION,
        )
        .expect("the governed fusion-capability provider identity is valid");
        let mut roles = BTreeMap::new();
        roles.insert(constant_f32_op(), FusionOperationRole::ValueSource);
        roles.insert(
            multiply_f32_op(),
            FusionOperationRole::ElementwiseArithmetic,
        );
        roles.insert(add_f32_op(), FusionOperationRole::ElementwiseArithmetic);
        roles.insert(
            strict_serial_sum_f32_op(),
            FusionOperationRole::OrderedReduction,
        );
        Self {
            provider,
            revision: GOVERNED_PROVIDER_REVISION,
            roles,
        }
    }

    /// Returns the provider that declared these capabilities.
    #[must_use]
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the output-affecting revision of the capability source.
    #[must_use]
    pub(crate) const fn revision(&self) -> u32 {
        self.revision
    }

    fn classify(&self, key: &OpKey) -> Option<FusionOperationRole> {
        self.roles.get(key).copied()
    }

    /// Builds the governed registry without one operation family's capability.
    ///
    /// This exercises the fail-closed path where a member operation has no
    /// registered fusion capability.
    #[cfg(test)]
    fn governed_without(excluded: &OpKey) -> Self {
        let mut capabilities = Self::governed();
        capabilities.roles.remove(excluded);
        capabilities
    }
}

/// One numerical, effect, or materialization obligation a fused realization must
/// satisfy.
///
/// The obligations are per-operation-derived rather than a fixed proof label.
/// Reassociation and operand permutation are separate obligations because a
/// permission or capability for one is not evidence for the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FusionObligation {
    /// Every member operation has a resolved fusion capability.
    OperationCapabilitiesResolved,
    /// Every member operation is referentially transparent.
    ReferentialTransparency,
    /// No observable conversion/materialization boundary is silently removed.
    ConversionBoundaryPreservation,
    /// The separate-rounding contract is preserved; contraction stays authorized.
    ArithmeticContraction,
    /// NaN canonicalization, signed zero, and subnormal handling are preserved.
    ExceptionalValues,
    /// Each reduction's identity and empty-domain result are defined.
    ReductionIdentityAndEmptyDomain,
    /// Each reduction's contributor order satisfies the semantic order contract.
    ReductionContributorOrder,
    /// Reassociation legality is established independently of permutation.
    ReductionReassociation,
    /// Operand-permutation legality is established independently of reassociation.
    ReductionOperandPermutation,
}

impl FusionObligation {
    /// Returns the stable rule key of this obligation.
    pub(crate) const fn rule(self) -> &'static str {
        match self {
            Self::OperationCapabilitiesResolved => "fusion.capabilities-resolved",
            Self::ReferentialTransparency => "fusion.referential-transparency",
            Self::ConversionBoundaryPreservation => "fusion.conversion-boundary",
            Self::ArithmeticContraction => "fusion.arithmetic-contraction",
            Self::ExceptionalValues => "fusion.exceptional-values",
            Self::ReductionIdentityAndEmptyDomain => "fusion.reduction-identity-empty-domain",
            Self::ReductionContributorOrder => "fusion.reduction-contributor-order",
            Self::ReductionReassociation => "fusion.reduction-reassociation",
            Self::ReductionOperandPermutation => "fusion.reduction-operand-permutation",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::OperationCapabilitiesResolved => 1,
            Self::ReferentialTransparency => 2,
            Self::ConversionBoundaryPreservation => 3,
            Self::ArithmeticContraction => 4,
            Self::ExceptionalValues => 5,
            Self::ReductionIdentityAndEmptyDomain => 6,
            Self::ReductionContributorOrder => 7,
            Self::ReductionReassociation => 8,
            Self::ReductionOperandPermutation => 9,
        }
    }
}

/// The assessment of one derived obligation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObligationAssessment {
    /// The obligation holds for a fused realization.
    Discharged,
    /// A fused realization is proved to violate the obligation.
    Rejected {
        /// Stable reason code.
        reason: &'static str,
    },
    /// The obligation cannot be established in this bounded profile.
    Unknown {
        /// Stable reason code.
        reason: &'static str,
    },
}

impl ObligationAssessment {
    const fn tag(self) -> u8 {
        match self {
            Self::Discharged => 1,
            Self::Rejected { .. } => 2,
            Self::Unknown { .. } => 3,
        }
    }

    const fn reason(self) -> Option<&'static str> {
        match self {
            Self::Discharged => None,
            Self::Rejected { reason } | Self::Unknown { reason } => Some(reason),
        }
    }
}

/// One obligation, its assessment, and the class of evidence behind it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DerivedObligation {
    obligation: FusionObligation,
    assessment: ObligationAssessment,
    evidence: FusionEvidenceClass,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl DerivedObligation {
    /// Returns the obligation this record assesses.
    pub(crate) const fn obligation(&self) -> FusionObligation {
        self.obligation
    }

    /// Returns the obligation's assessment.
    pub(crate) const fn assessment(&self) -> ObligationAssessment {
        self.assessment
    }

    /// Returns the class of evidence behind the assessment.
    pub(crate) const fn evidence(&self) -> FusionEvidenceClass {
        self.evidence
    }

    fn discharged(obligation: FusionObligation, evidence: FusionEvidenceClass) -> Self {
        Self {
            obligation,
            assessment: ObligationAssessment::Discharged,
            evidence,
        }
    }

    fn rejected(obligation: FusionObligation, reason: &'static str) -> Self {
        Self {
            obligation,
            assessment: ObligationAssessment::Rejected { reason },
            evidence: FusionEvidenceClass::SoundProof,
        }
    }

    fn unknown(obligation: FusionObligation, reason: &'static str) -> Self {
        Self {
            obligation,
            assessment: ObligationAssessment::Unknown { reason },
            evidence: FusionEvidenceClass::Unknown,
        }
    }
}

/// Site-independent structural counts of one region's derived computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FusionRegionStructure {
    /// Number of member operations.
    members: u32,
    /// Number of value-source members (constants and boundary reads).
    value_sources: u32,
    /// Number of elementwise arithmetic members.
    arithmetic: u32,
    /// Number of ordered-reduction members.
    reductions: u32,
    /// Number of boundary input values.
    boundary_inputs: u32,
    /// Number of retained boundary outputs.
    retained_outputs: u32,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionRegionStructure {
    /// Returns the number of member operations.
    pub(crate) const fn member_count(&self) -> u32 {
        self.members
    }

    /// Returns the number of ordered-reduction members.
    pub(crate) const fn reduction_count(&self) -> u32 {
        self.reductions
    }

    fn encode(&self, output: &mut Vec<u8>) {
        for field in [
            self.members,
            self.value_sources,
            self.arithmetic,
            self.reductions,
            self.boundary_inputs,
            self.retained_outputs,
        ] {
            output.extend_from_slice(&field.to_be_bytes());
        }
    }
}

/// Collision-free identity of reusable fusion-legality content.
///
/// Two occurrences of the same region content, discharged under the same
/// numerical contract, share these bytes. The graph site, selected provider,
/// and reached admission provenance are deliberately absent.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FusionLegalityContentIdentity(Vec<u8>);

impl FusionLegalityContentIdentity {
    /// Returns the canonical content bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Collision-free identity of one fusion-legality occurrence binding.
///
/// This is reusable content plus the exact graph site, the reached semantic
/// definitions, the selected provider, and the ordered value bindings.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FusionLegalityIdentity(Vec<u8>);

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionLegalityIdentity {
    /// Returns the canonical occurrence-binding bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Reusable, site- and provider-independent fusion-legality content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionLegalityContent {
    region_content: RegionContentIdentity,
    numerical_contract_key: &'static str,
    structure: FusionRegionStructure,
    obligations: Vec<DerivedObligation>,
    identity: FusionLegalityContentIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionLegalityContent {
    /// Returns the canonical region-content identity this legality is over.
    pub(crate) const fn region_content(&self) -> &RegionContentIdentity {
        &self.region_content
    }

    /// Returns the numerical-contract key the obligations were discharged under.
    pub(crate) const fn numerical_contract_key(&self) -> &'static str {
        self.numerical_contract_key
    }

    /// Returns the site-independent structural counts.
    pub(crate) const fn structure(&self) -> &FusionRegionStructure {
        &self.structure
    }

    /// Returns the ordered discharged obligations with their evidence classes.
    pub(crate) fn obligations(&self) -> &[DerivedObligation] {
        &self.obligations
    }

    /// Returns the reusable content identity.
    pub(crate) const fn identity(&self) -> &FusionLegalityContentIdentity {
        &self.identity
    }
}

/// One reached semantic definition an occurrence binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReachedDefinition {
    operation: OpKey,
    normative_definition: String,
    effect_tag: u8,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl ReachedDefinition {
    /// Returns the reached operation family key.
    pub(crate) const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the reached normative-definition reference.
    pub(crate) fn normative_definition(&self) -> &str {
        &self.normative_definition
    }
}

/// One retained boundary output bound to its exact producer occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedOutputBinding {
    value: u32,
    producer: u32,
    result_position: u32,
    named_result: bool,
    external_consumers: bool,
}

/// The ordered value/access mapping of one region occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionValueBindings {
    boundary_inputs: Vec<u32>,
    retained_outputs: Vec<RetainedOutputBinding>,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionValueBindings {
    /// Returns the graph-local boundary input value ordinals.
    pub(crate) fn boundary_inputs(&self) -> &[u32] {
        &self.boundary_inputs
    }

    /// Returns the ordered retained-output bindings.
    pub(crate) fn retained_outputs(&self) -> &[RetainedOutputBinding] {
        &self.retained_outputs
    }
}

/// Replayable evidence that one region occurrence fuses legally.
///
/// It binds reusable [`FusionLegalityContent`] to the exact occurrence: the
/// region-occurrence identity, the reached semantic definitions, the selected
/// fusion-capability provider, and the ordered value bindings. Holding one is
/// evidence that the derivation discharged every obligation for *this* site, not
/// merely that a candidate exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionLegalityProof {
    content: FusionLegalityContent,
    region_occurrence: RegionOccurrenceIdentity,
    registry_snapshot: Box<[u8]>,
    reached_definitions: Vec<ReachedDefinition>,
    provider: ProviderIdentity,
    provider_revision: u32,
    value_bindings: FusionValueBindings,
    identity: FusionLegalityIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionLegalityProof {
    /// Returns the reusable, site-independent content.
    pub(crate) const fn content(&self) -> &FusionLegalityContent {
        &self.content
    }

    /// Returns the graph-occurrence identity this proof is bound to.
    pub(crate) const fn region_occurrence(&self) -> &RegionOccurrenceIdentity {
        &self.region_occurrence
    }

    /// Returns the reached semantic definitions in region-local order.
    pub(crate) fn reached_definitions(&self) -> &[ReachedDefinition] {
        &self.reached_definitions
    }

    /// Returns the selected fusion-capability provider.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the ordered value/access bindings of this occurrence.
    pub(crate) const fn value_bindings(&self) -> &FusionValueBindings {
        &self.value_bindings
    }

    /// Returns the occurrence-binding identity that pins this realization.
    pub(crate) const fn identity(&self) -> &FusionLegalityIdentity {
        &self.identity
    }
}

/// A candidate proved to violate an obligation as a fused realization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionRejection {
    obligation: FusionObligation,
    reason: &'static str,
    region: String,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionRejection {
    /// Returns the violated obligation.
    pub(crate) const fn obligation(&self) -> FusionObligation {
        self.obligation
    }

    /// Returns the stable reason code.
    pub(crate) const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for FusionRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}: {} rejected",
            self.obligation.rule(),
            self.reason,
            self.region
        )
    }
}

/// A candidate whose fused legality the bounded profile cannot establish.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FusionUnknown {
    obligation: FusionObligation,
    reason: &'static str,
    region: String,
}

#[allow(
    dead_code,
    reason = "reviewed proof-record accessor exercised by this authority's own tests; the compile path replays a proof by equality rather than by reading its parts"
)]
impl FusionUnknown {
    /// Returns the obligation that could not be established.
    pub(crate) const fn obligation(&self) -> FusionObligation {
        self.obligation
    }

    /// Returns the stable reason code.
    pub(crate) const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for FusionUnknown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}: {} unknown",
            self.obligation.rule(),
            self.reason,
            self.region
        )
    }
}

/// The typed outcome of deriving fusion legality for one candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FusionLegality {
    /// The candidate fuses legally, with replayable evidence.
    Legal(Box<FusionLegalityProof>),
    /// A fused realization is proved to violate an obligation.
    Rejected(FusionRejection),
    /// The bounded profile cannot establish a required obligation.
    Unknown(FusionUnknown),
}

/// A fault in fusion-legality derivation, distinct from a legality outcome.
///
/// These are invalid compiler input or output — a forged candidate that fails
/// re-derivation, or a verified program whose operation lacks a registry
/// definition — not the legal `Rejected`/`Unknown` outcomes above.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FusionLegalityError {
    /// The candidate failed re-derivation from its own exact contents.
    Region(RegionError),
    /// The derivation observed invalid compiler state.
    Structure {
        /// Stable rule code.
        rule: &'static str,
    },
}

impl FusionLegalityError {
    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Region(error) => error.reason(),
            Self::Structure { rule } => rule,
        }
    }
}

impl fmt::Display for FusionLegalityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(error) => error.fmt(formatter),
            Self::Structure { rule } => {
                write!(formatter, "fusion.legality.structure.{rule}")
            }
        }
    }
}

impl Error for FusionLegalityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Region(error) => Some(error),
            Self::Structure { .. } => None,
        }
    }
}

impl From<RegionError> for FusionLegalityError {
    fn from(value: RegionError) -> Self {
        Self::Region(value)
    }
}

/// The complete derivation of one member operation.
struct MemberDerivation {
    role: FusionOperationRole,
    reached: ReachedDefinition,
    pure: bool,
    homogeneous: bool,
}

/// Derives fusion legality for one region candidate.
///
/// The candidate is re-derived from the graph before anything else, so a forged
/// or stale candidate fails closed. Each member operation's fusion capability is
/// then resolved and its obligations discharged against the reached semantic
/// definition and the numerical policy. The result is a legal proof, a typed
/// rejection, or a typed unknown; a hard rejection dominates an unknown so the
/// most certain failure is reported.
///
/// # Errors
///
/// Returns a [`FusionLegalityError`] when the candidate does not re-derive or a
/// member operation lacks a semantic-registry definition. A legality outcome
/// (`Rejected`/`Unknown`) is a successful `Ok`, not an error.
pub(crate) fn derive_fusion_legality(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    capabilities: &FusionNumericalCapabilities,
    candidate: &RegionCandidate,
) -> Result<FusionLegality, FusionLegalityError> {
    let graph = RegionGraph::from_program(program)?;
    verify_candidate(&graph, budgets, contract, candidate)?;
    let registry = program.semantic_registry();

    let ordered = ordered_members(&graph, candidate)?;
    let governed_dtype = F32::resolved_type().canonical_encoding();
    let governed_dtype = governed_dtype.as_bytes();

    // An unresolved capability makes the whole derivation unknown before any
    // role-dependent obligation is evaluated: without a role the reduction and
    // arithmetic obligations cannot be soundly derived.
    let mut members = Vec::with_capacity(ordered.len());
    for member in &ordered {
        match derive_member(&graph, registry, capabilities, governed_dtype, *member)? {
            Some(derivation) => members.push(derivation),
            None => {
                return Ok(FusionLegality::Unknown(FusionUnknown {
                    obligation: FusionObligation::OperationCapabilitiesResolved,
                    reason: "unsupported-operation-capability",
                    region: candidate.label().to_owned(),
                }));
            }
        }
    }

    let obligations = derive_obligations(&members, contract);
    if let Some(rejected) = first_rejection(&obligations, candidate) {
        return Ok(FusionLegality::Rejected(rejected));
    }
    if let Some(unknown) = first_unknown(&obligations, candidate) {
        return Ok(FusionLegality::Unknown(unknown));
    }

    let structure = region_structure(candidate, &members);
    let content = assemble_content(candidate, contract, structure, obligations);
    let proof = assemble_proof(candidate, capabilities, registry, &members, content);
    Ok(FusionLegality::Legal(Box::new(proof)))
}

/// Re-derives one legal proof and requires it to equal the retained evidence.
///
/// # Errors
///
/// Returns a [`FusionLegalityError`] when the candidate does not re-derive, or a
/// [`FusionLegalityError::Structure`] when the re-derivation is not a legal proof
/// equal to `proof`.
pub(crate) fn verify_fusion_legality(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    capabilities: &FusionNumericalCapabilities,
    candidate: &RegionCandidate,
    proof: &FusionLegalityProof,
) -> Result<(), FusionLegalityError> {
    match derive_fusion_legality(program, budgets, contract, capabilities, candidate)? {
        FusionLegality::Legal(expected) if expected.as_ref() == proof => Ok(()),
        _ => Err(FusionLegalityError::Structure {
            rule: "legality-proof-subject",
        }),
    }
}

/// Orders the candidate's members by content-derived canonical position.
fn ordered_members(
    graph: &RegionGraph,
    candidate: &RegionCandidate,
) -> Result<Vec<SemanticMemberId>, FusionLegalityError> {
    let mut keyed = Vec::with_capacity(candidate.members().len());
    for member in candidate.members() {
        keyed.push((graph.member_canonical_position(*member)?, *member));
    }
    keyed.sort_by_key(|(position, _)| *position);
    Ok(keyed.into_iter().map(|(_, member)| member).collect())
}

/// Derives one member's role, reached definition, purity, and type homogeneity.
///
/// Returns `Ok(None)` when the member's operation has no fusion capability.
fn derive_member(
    graph: &RegionGraph,
    registry: &FrozenSemanticRegistry,
    capabilities: &FusionNumericalCapabilities,
    governed_dtype: &[u8],
    member: SemanticMemberId,
) -> Result<Option<MemberDerivation>, FusionLegalityError> {
    let facts = graph.member_operation_facts(member)?;
    let definition =
        registry
            .operation_definition(facts.key())
            .ok_or(FusionLegalityError::Structure {
                rule: "missing-operation-definition",
            })?;
    // The derived graph purity must agree with the reached definition's effect;
    // a disagreement is invalid compiler state, not a legality outcome.
    if facts.is_pure() != matches!(definition.effect(), OperationEffect::Pure) {
        return Err(FusionLegalityError::Structure {
            rule: "effect-disagreement",
        });
    }
    let Some(role) = capabilities.classify(facts.key()) else {
        return Ok(None);
    };
    let reached = ReachedDefinition {
        operation: facts.key().clone(),
        normative_definition: definition.normative_definition().as_str().to_owned(),
        effect_tag: effect_tag(definition.effect()),
    };
    Ok(Some(MemberDerivation {
        role,
        reached,
        pure: facts.is_pure(),
        homogeneous: member_is_homogeneous(&facts, governed_dtype),
    }))
}

/// Returns whether every operand and result type is the governed dtype.
fn member_is_homogeneous(facts: &MemberOperationFacts<'_>, governed_dtype: &[u8]) -> bool {
    facts
        .operand_type_encodings()
        .iter()
        .chain(facts.result_type_encodings())
        .all(|encoding| *encoding == governed_dtype)
}

/// Discharges every obligation for the resolved members under the contract.
fn derive_obligations(
    members: &[MemberDerivation],
    contract: StrictF32NumericalContract,
) -> Vec<DerivedObligation> {
    let mut obligations = Vec::new();

    // Every member resolved a capability, or the caller returned unknown earlier.
    obligations.push(DerivedObligation::discharged(
        FusionObligation::OperationCapabilitiesResolved,
        FusionEvidenceClass::SoundProof,
    ));

    obligations.push(if members.iter().all(|member| member.pure) {
        DerivedObligation::discharged(
            FusionObligation::ReferentialTransparency,
            FusionEvidenceClass::SoundProof,
        )
    } else {
        DerivedObligation::rejected(FusionObligation::ReferentialTransparency, "impure-member")
    });

    obligations.push(if members.iter().all(|member| member.homogeneous) {
        DerivedObligation::discharged(
            FusionObligation::ConversionBoundaryPreservation,
            FusionEvidenceClass::SoundProof,
        )
    } else {
        DerivedObligation::unknown(
            FusionObligation::ConversionBoundaryPreservation,
            "unproven-conversion-preservation",
        )
    });

    // Contraction: the initial arithmetic contract keeps separate roundings and
    // no member role requires a fused multiply-add. A policy that permitted
    // contraction is not realizable in this profile and is left unknown.
    obligations.push(
        if matches!(contract.contraction, NumericalPermission::Forbidden) {
            DerivedObligation::discharged(
                FusionObligation::ArithmeticContraction,
                FusionEvidenceClass::NormativeGuarantee,
            )
        } else {
            DerivedObligation::unknown(
                FusionObligation::ArithmeticContraction,
                "unrealized-contraction",
            )
        },
    );

    // Exceptional values: NaN canonicalization, signed zero, and subnormal
    // handling must survive fusion.
    //
    // The subnormal dimensions do **not** constrain this, whatever their
    // resolution. `docs/numerical-semantics.md` defines both as per-operation
    // rules — "input flushing treats an existing subnormal operand as zero
    // before arithmetic" and "result flushing replaces a newly produced
    // subnormal result with zero". A materialization boundary is a store and a
    // load: neither is arithmetic and neither produces a newly produced result,
    // so removing one neither adds nor removes a flush. The fused and
    // materialized forms perform the same arithmetic under the same
    // per-operation rule, so their exceptional-value behaviour agrees.
    //
    // Requiring `Preserve` here was the strict contract's assumption rather
    // than this obligation's content, and it deferred every fused candidate
    // under any flush contract — costing the fused alternative for a reason the
    // contract does not state.
    //
    // The canonical NaN pattern *is* constrained, and stays. It is a per-result
    // rewrite the fused body must still apply at every arithmetic boundary,
    // which `emit_reduction` and `emit_scale_bias` are what realize.
    //
    // A boundary that genuinely carries semantics is guarded separately:
    // `ConversionBoundaryPreservation` above discharges only when every member
    // is homogeneous, so a removed dtype-conversion boundary is refused there
    // rather than here.
    let governed = StrictF32NumericalContract::governed();
    let exceptional_ok =
        contract.canonical_arithmetic_nan_bits == governed.canonical_arithmetic_nan_bits;
    obligations.push(if exceptional_ok {
        DerivedObligation::discharged(
            FusionObligation::ExceptionalValues,
            FusionEvidenceClass::NormativeGuarantee,
        )
    } else {
        DerivedObligation::unknown(
            FusionObligation::ExceptionalValues,
            "unproven-exceptional-values",
        )
    });

    push_reduction_obligations(&mut obligations, members, contract);
    obligations
}

/// Pushes the four reduction obligations, kept independent per ADR 0014.
fn push_reduction_obligations(
    obligations: &mut Vec<DerivedObligation>,
    members: &[MemberDerivation],
    contract: StrictF32NumericalContract,
) {
    let has_reduction = members.iter().any(|member| member.role.is_reduction());

    // Identity/empty-domain and contributor order rest on the ordered-reduction
    // role's normative definition. With no reduction the obligation is
    // vacuously discharged as a structural fact.
    let reduction_class = if has_reduction {
        FusionEvidenceClass::NormativeGuarantee
    } else {
        FusionEvidenceClass::SoundProof
    };
    obligations.push(DerivedObligation::discharged(
        FusionObligation::ReductionIdentityAndEmptyDomain,
        reduction_class,
    ));
    obligations.push(DerivedObligation::discharged(
        FusionObligation::ReductionContributorOrder,
        reduction_class,
    ));

    // Reassociation is a policy permission over the ordered-reduction role.
    obligations.push(
        if matches!(contract.reassociation, NumericalPermission::Forbidden) {
            DerivedObligation::discharged(
                FusionObligation::ReductionReassociation,
                FusionEvidenceClass::SoundProof,
            )
        } else {
            DerivedObligation::unknown(
                FusionObligation::ReductionReassociation,
                "unproven-reassociation",
            )
        },
    );

    // Operand permutation is independent: the ordered left fold fixes operand
    // order, so no permutation is used. It is derived from the role, not from a
    // separate contract permission field.
    obligations.push(DerivedObligation::discharged(
        FusionObligation::ReductionOperandPermutation,
        FusionEvidenceClass::SoundProof,
    ));
}

/// Returns the first rejected obligation as a typed rejection.
fn first_rejection(
    obligations: &[DerivedObligation],
    candidate: &RegionCandidate,
) -> Option<FusionRejection> {
    obligations
        .iter()
        .find_map(|derived| match derived.assessment {
            ObligationAssessment::Rejected { reason } => Some(FusionRejection {
                obligation: derived.obligation,
                reason,
                region: candidate.label().to_owned(),
            }),
            _ => None,
        })
}

/// Returns the first unknown obligation as a typed unknown.
fn first_unknown(
    obligations: &[DerivedObligation],
    candidate: &RegionCandidate,
) -> Option<FusionUnknown> {
    obligations
        .iter()
        .find_map(|derived| match derived.assessment {
            ObligationAssessment::Unknown { reason } => Some(FusionUnknown {
                obligation: derived.obligation,
                reason,
                region: candidate.label().to_owned(),
            }),
            _ => None,
        })
}

/// Computes the site-independent structural counts of the region.
fn region_structure(
    candidate: &RegionCandidate,
    members: &[MemberDerivation],
) -> FusionRegionStructure {
    let count = |predicate: fn(FusionOperationRole) -> bool| {
        u32::try_from(
            members
                .iter()
                .filter(|member| predicate(member.role))
                .count(),
        )
        .unwrap_or(u32::MAX)
    };
    FusionRegionStructure {
        members: u32::try_from(members.len()).unwrap_or(u32::MAX),
        value_sources: count(FusionOperationRole::is_value_source),
        arithmetic: count(FusionOperationRole::is_arithmetic),
        reductions: count(FusionOperationRole::is_reduction),
        boundary_inputs: u32::try_from(candidate.boundary_inputs().len()).unwrap_or(u32::MAX),
        retained_outputs: u32::try_from(candidate.retained_outputs().len()).unwrap_or(u32::MAX),
    }
}

/// Assembles reusable content and its canonical identity.
fn assemble_content(
    candidate: &RegionCandidate,
    contract: StrictF32NumericalContract,
    structure: FusionRegionStructure,
    obligations: Vec<DerivedObligation>,
) -> FusionLegalityContent {
    let region_content = candidate.content().clone();
    let identity = encode_content_identity(&region_content, contract.key, &structure, &obligations);
    FusionLegalityContent {
        region_content,
        numerical_contract_key: contract.key,
        structure,
        obligations,
        identity,
    }
}

/// Assembles the occurrence binding and its canonical identity.
fn assemble_proof(
    candidate: &RegionCandidate,
    capabilities: &FusionNumericalCapabilities,
    registry: &FrozenSemanticRegistry,
    members: &[MemberDerivation],
    content: FusionLegalityContent,
) -> FusionLegalityProof {
    let reached_definitions = members
        .iter()
        .map(|member| member.reached.clone())
        .collect::<Vec<_>>();
    let value_bindings = value_bindings(candidate);
    let registry_snapshot = registry
        .snapshot_identity()
        .as_bytes()
        .to_vec()
        .into_boxed_slice();
    let identity = encode_occurrence_identity(
        &content,
        candidate.occurrence(),
        &registry_snapshot,
        &reached_definitions,
        capabilities,
        &value_bindings,
    );
    FusionLegalityProof {
        content,
        region_occurrence: candidate.occurrence().clone(),
        registry_snapshot,
        reached_definitions,
        provider: capabilities.provider().clone(),
        provider_revision: capabilities.revision(),
        value_bindings,
        identity,
    }
}

/// Extracts the ordered value/access mapping from the candidate.
fn value_bindings(candidate: &RegionCandidate) -> FusionValueBindings {
    let boundary_inputs = candidate
        .boundary_inputs()
        .iter()
        .map(|value| value.0)
        .collect();
    let retained_outputs = candidate
        .retained_outputs()
        .iter()
        .map(|output| RetainedOutputBinding {
            value: output.value.0,
            producer: output.producer.0,
            result_position: output.result_position,
            named_result: output.named_result,
            external_consumers: output.external_consumers,
        })
        .collect();
    FusionValueBindings {
        boundary_inputs,
        retained_outputs,
    }
}

fn encode_content_identity(
    region_content: &RegionContentIdentity,
    contract_key: &str,
    structure: &FusionRegionStructure,
    obligations: &[DerivedObligation],
) -> FusionLegalityContentIdentity {
    let mut bytes = CONTENT_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region_content.as_bytes());
    push_slice(&mut bytes, contract_key.as_bytes());
    structure.encode(&mut bytes);
    push_len(&mut bytes, obligations.len());
    for derived in obligations {
        bytes.push(derived.obligation.tag());
        bytes.push(derived.assessment.tag());
        bytes.push(derived.evidence.tag());
        push_slice(
            &mut bytes,
            derived.assessment.reason().unwrap_or("").as_bytes(),
        );
    }
    FusionLegalityContentIdentity(bytes)
}

fn encode_occurrence_identity(
    content: &FusionLegalityContent,
    occurrence: &RegionOccurrenceIdentity,
    registry_snapshot: &[u8],
    reached_definitions: &[ReachedDefinition],
    capabilities: &FusionNumericalCapabilities,
    value_bindings: &FusionValueBindings,
) -> FusionLegalityIdentity {
    let mut bytes = OCCURRENCE_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, content.identity.as_bytes());
    push_slice(&mut bytes, occurrence.as_bytes());
    push_slice(&mut bytes, registry_snapshot);
    push_len(&mut bytes, reached_definitions.len());
    for reached in reached_definitions {
        encode_op_key(&mut bytes, &reached.operation);
        push_slice(&mut bytes, reached.normative_definition.as_bytes());
        bytes.push(reached.effect_tag);
    }
    encode_provider(&mut bytes, capabilities.provider());
    bytes.extend_from_slice(&capabilities.revision().to_be_bytes());
    push_len(&mut bytes, value_bindings.boundary_inputs.len());
    for input in &value_bindings.boundary_inputs {
        bytes.extend_from_slice(&input.to_be_bytes());
    }
    push_len(&mut bytes, value_bindings.retained_outputs.len());
    for output in &value_bindings.retained_outputs {
        bytes.extend_from_slice(&output.value.to_be_bytes());
        bytes.extend_from_slice(&output.producer.to_be_bytes());
        bytes.extend_from_slice(&output.result_position.to_be_bytes());
        bytes.push(u8::from(output.named_result));
        bytes.push(u8::from(output.external_consumers));
    }
    FusionLegalityIdentity(bytes)
}

/// Encodes one observable effect class into fusion-legality identity.
///
/// Exhaustive with no wildcard arm (ADR 0074 convention 3): a second effect
/// must choose its own tag at this site as a compile error, because a wildcard
/// would give two structurally distinct occurrences the same identity bytes.
/// That is only expressible because `OperationEffect` deliberately carries no
/// `#[non_exhaustive]`, which is what convention 5b decides for a vocabulary an
/// out-of-crate encoder maps totally.
const fn effect_tag(effect: OperationEffect) -> u8 {
    match effect {
        OperationEffect::Pure => 1,
    }
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::{
        DerivedObligation, FusionEvidenceClass, FusionLegality, FusionLegalityError,
        FusionNumericalCapabilities, FusionObligation, ObligationAssessment,
        derive_fusion_legality, verify_fusion_legality,
    };
    use crate::region::{RegionCandidate, form_region_candidates};
    use crate::request::{DeterministicBudgets, StrictF32NumericalContract};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum, add_f32_op,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn serial_sum_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
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

    fn square_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), product)
            .unwrap();
        builder.build().unwrap()
    }

    fn whole_program_candidate(program: &SemanticProgram) -> RegionCandidate {
        let outcome = form_region_candidates(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap();
        outcome
            .whole_program_candidate()
            .expect("a connected program has a whole-program region")
            .clone()
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        !needle.is_empty()
            && haystack
                .windows(needle.len())
                .any(|window| window == needle)
    }

    #[test]
    fn whole_program_serial_sum_is_legal_with_replayable_evidence() {
        let program = serial_sum_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) =
            derive_fusion_legality(&program, budgets, contract, &capabilities, &candidate).unwrap()
        else {
            panic!("the governed serial sum fuses legally");
        };

        // Replay reproduces the exact proof.
        verify_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &candidate,
            &proof,
        )
        .unwrap();

        // Every obligation is discharged with a labelled evidence class, and the
        // reduction obligations carry a normative guarantee, not a bare label.
        assert!(
            proof
                .content()
                .obligations()
                .iter()
                .all(|derived| matches!(derived.assessment(), ObligationAssessment::Discharged))
        );
        let reduction = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ReductionContributorOrder)
            .unwrap();
        assert_eq!(
            reduction.evidence(),
            FusionEvidenceClass::NormativeGuarantee
        );
        assert_eq!(proof.content().structure().reduction_count(), 1);

        // The reached definitions cover every member and name the reduction's
        // normative definition.
        assert_eq!(
            proof.reached_definitions().len(),
            usize::try_from(proof.content().structure().member_count()).unwrap()
        );
        assert!(proof.reached_definitions().iter().any(|reached| {
            reached
                .normative_definition()
                .contains("strict-serial-sum-f32")
        }));

        // The occurrence binds the ordered value/access mapping.
        assert_eq!(proof.value_bindings().retained_outputs().len(), 1);
    }

    #[test]
    fn content_identity_excludes_provider_and_occurrence() {
        let program = serial_sum_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) =
            derive_fusion_legality(&program, budgets, contract, &capabilities, &candidate).unwrap()
        else {
            panic!("legal");
        };

        let content_bytes = proof.content().identity().as_bytes();
        let occurrence_bytes = proof.identity().as_bytes();

        // Content and occurrence identities are distinct.
        assert_ne!(content_bytes, occurrence_bytes);
        // Pure content contains neither the selected provider nor the graph site.
        assert!(!contains(content_bytes, proof.provider().name().as_bytes()));
        assert!(!contains(
            content_bytes,
            proof.region_occurrence().as_bytes()
        ));
        // The occurrence binding is content plus the site and the provider.
        assert!(contains(occurrence_bytes, content_bytes));
        assert!(contains(
            occurrence_bytes,
            proof.provider().name().as_bytes()
        ));
        assert!(contains(
            occurrence_bytes,
            proof.region_occurrence().as_bytes()
        ));
    }

    #[test]
    fn a_pure_pointwise_square_is_legal_with_no_reduction() {
        let program = square_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(proof) =
            derive_fusion_legality(&program, budgets, contract, &capabilities, &candidate).unwrap()
        else {
            panic!("a pure square fuses legally");
        };
        assert_eq!(proof.content().structure().reduction_count(), 0);
        // The reduction obligations are vacuously discharged as sound structural
        // facts, distinct from the normative guarantee a real reduction carries.
        let order = proof
            .content()
            .obligations()
            .iter()
            .find(|derived| derived.obligation() == FusionObligation::ReductionContributorOrder)
            .unwrap();
        assert_eq!(order.evidence(), FusionEvidenceClass::SoundProof);
    }

    #[test]
    fn an_unregistered_operation_capability_is_unknown() {
        let program = serial_sum_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        // Drop the add capability so a member operation has no fusion role.
        let capabilities = FusionNumericalCapabilities::governed_without(&add_f32_op());

        let FusionLegality::Unknown(unknown) =
            derive_fusion_legality(&program, budgets, contract, &capabilities, &candidate).unwrap()
        else {
            panic!("a missing capability fails closed to unknown");
        };
        assert_eq!(
            unknown.obligation(),
            FusionObligation::OperationCapabilitiesResolved
        );
        assert_eq!(unknown.reason(), "unsupported-operation-capability");
    }

    #[test]
    fn a_contract_with_foreign_nan_bits_is_unknown() {
        let program = serial_sum_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let capabilities = FusionNumericalCapabilities::governed();
        // Keep the governed contract key (so the candidate re-derives) but demand
        // a NaN pattern the governed operations do not produce.
        let mut contract = StrictF32NumericalContract::governed();
        contract.canonical_arithmetic_nan_bits ^= 1;

        let FusionLegality::Unknown(unknown) =
            derive_fusion_legality(&program, budgets, contract, &capabilities, &candidate).unwrap()
        else {
            panic!("a foreign NaN contract cannot be proved");
        };
        assert_eq!(unknown.obligation(), FusionObligation::ExceptionalValues);
        assert_eq!(unknown.reason(), "unproven-exceptional-values");
    }

    #[test]
    fn a_forged_proof_fails_replay() {
        let program = serial_sum_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        let FusionLegality::Legal(mut proof) =
            derive_fusion_legality(&program, budgets, contract, &capabilities, &candidate).unwrap()
        else {
            panic!("legal");
        };
        // Tamper with the recorded provider revision.
        proof.provider_revision += 1;
        let error = verify_fusion_legality(
            &program,
            budgets,
            contract,
            &capabilities,
            &candidate,
            &proof,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            FusionLegalityError::Structure {
                rule: "legality-proof-subject"
            }
        ));
    }

    #[test]
    fn a_candidate_from_another_graph_fails_re_derivation() {
        let program = serial_sum_program();
        let candidate = whole_program_candidate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        let capabilities = FusionNumericalCapabilities::governed();

        // A structurally different program yields a different graph, so the
        // stored occurrence identity no longer re-derives.
        let other = square_program();
        let error = derive_fusion_legality(&other, budgets, contract, &capabilities, &candidate)
            .unwrap_err();
        assert!(matches!(error, FusionLegalityError::Region(_)));
    }

    #[test]
    fn the_five_evidence_classes_stay_distinct() {
        let classes = [
            FusionEvidenceClass::NormativeGuarantee,
            FusionEvidenceClass::SoundProof,
            FusionEvidenceClass::ExhaustiveFinite,
            FusionEvidenceClass::Empirical,
            FusionEvidenceClass::Unknown,
        ];
        let mut names: Vec<&str> = classes.iter().map(|class| class.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), classes.len());
    }

    #[test]
    fn a_rejection_reports_its_exact_obligation_and_reason() {
        // The reject disposition is a fail-closed guard: the governed strict-f32
        // vocabulary (only pure effects, forbidden permissions, preserved
        // subnormals) cannot express an illegal-but-valid program, so this
        // exercises the typed rejection surface directly.
        let rejection = super::FusionRejection {
            obligation: FusionObligation::ReferentialTransparency,
            reason: "impure-member",
            region: "region:0000000000000000".to_owned(),
        };
        assert_eq!(
            rejection.obligation(),
            FusionObligation::ReferentialTransparency
        );
        assert_eq!(rejection.reason(), "impure-member");
        assert_eq!(
            rejection.to_string(),
            "fusion.referential-transparency.impure-member: region:0000000000000000 rejected"
        );
    }

    #[test]
    fn discharged_obligations_are_never_rejected_or_unknown() {
        let discharged = DerivedObligation::discharged(
            FusionObligation::ArithmeticContraction,
            FusionEvidenceClass::NormativeGuarantee,
        );
        assert!(matches!(
            discharged.assessment(),
            ObligationAssessment::Discharged
        ));
    }
}
