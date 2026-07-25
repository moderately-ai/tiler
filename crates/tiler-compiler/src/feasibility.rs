#![allow(
    dead_code,
    reason = "the feasibility authority itself is on the compile path through assess_region; what stays unconstructed is the reserved later-phase surface — artifact-evidence, device-runtime, prepared-kernel, and launch phases, their fact authorities and validity scopes, the deferred/unknown verdicts, and the feasible-set view — which no compile-profile assessment can reach"
)]

//! Typed, phased target-feasibility authority (ADR 0043).
//!
//! Target feasibility is a physical contract outside the semantic tensor graph.
//! This module owns the *hard* feasibility decision only: whether a candidate's
//! typed resource and capability requirements are provably satisfiable against an
//! immutable checked target profile. It deliberately has no notion of cost. A
//! cost estimate can neither prove nor disprove feasibility, and a disproved hard
//! predicate is never expressed as an expensive plan; the two authorities are
//! kept in different types so they cannot be confused (ADR 0043, AGENTS.md
//! "Keep hard feasibility separate from estimated cost").
//!
//! The authority computes one of exactly four outcomes for a candidate proposal,
//! with fixed precedence: any disproved hard predicate rejects; otherwise a
//! predicate with no admissible proof/query path is unknown; otherwise all
//! unresolved checks form one nonempty canonical deferred set grouped by phase;
//! otherwise, with no remaining checks, the candidate is proven. A proposal with
//! no requirements is vacuously [`FeasibilityOutcome::Proven`].
//!
//! Two governed identities meet here and are kept apart. A
//! [`TargetProfileIdentity`] names *what* a target declares, distinguished by
//! [`CheckedTargetProfile::canonical_descriptor`]; a
//! [`FeasibilityRuleSetIdentity`] names *how* this authority compares a
//! requirement against a declaration. Neither determines the other — one profile
//! can be re-assessed under new rules, and one rule set applies across profiles
//! — and the artifact layer records them as two independent references, so
//! fusing them into one key-and-version pair would make an artifact assert that
//! it was assessed under a rule set named after a target profile.
//!
//! Malformed profiles and malformed proposals are *intrinsic errors* surfaced at
//! construction time ([`FeasibilityError`]), never a feasibility outcome. A valid
//! but empty feasible set — no candidate proves feasible — is a distinct,
//! legitimate result ([`FeasibleSet`] with an empty admitted partition), not an
//! error and not [`FeasibilityOutcome::Unknown`].
//!
//! Determinism: identities, deferred-set ordering, and the disproved/unknown
//! predicate lists are all canonical. No map iteration order participates in any
//! observable value; facts and requirements are stored sorted by their typed
//! keys and every aggregate is emitted in a fixed canonical order.

use tiler_ir::identity::{push_len, push_slice};

use crate::explain::Quantity;

/// Domain separator of a canonical target profile descriptor.
///
/// Trailing NUL so no descriptor can be a prefix of a differently-domained
/// encoding, matching the framing the rest of the workspace's identities use.
///
/// `v2` because the encoding changed when the feasibility rule version left the
/// profile's identity: a descriptor is durable identity, so an encoding that no
/// longer means what `v1` meant must not be able to collide with one that does.
const PROFILE_DESCRIPTOR_DOMAIN: &[u8] = b"tiler.target-profile.descriptor.v2\0";

/// Governed key of the feasibility rule set this authority applies.
///
/// The `.v1` suffix names the governed *vocabulary* the rules range over — the
/// ADR 0043 capability axes, availability phases, fact authorities, and validity
/// scopes — not an output-affecting revision within it. Widening that vocabulary
/// mints a new key because the rules would then decide predicates the old key
/// could not express; changing how the same terms are compared bumps
/// [`GOVERNED_FEASIBILITY_RULE_SET_REVISION`] instead. That is why the artifact
/// layer's `FeasibilityRuleSetRef` carries both a key and a revision rather than
/// one number.
const GOVERNED_FEASIBILITY_RULE_SET_KEY: &str = "tiler.feasibility.phased-capability-bounds.v1";

/// Nonzero output-affecting revision of the governed feasibility rule set.
///
/// Bumped when this module changes *how* a requirement is compared against a
/// bound — an axis [`Relation`], [`satisfies`], [`CapabilityAxis::admits`],
/// [`authority_matches_phase`], [`CheckedTargetProfile::resolve`]'s preference
/// for the most refined available fact, or the outcome precedence in
/// [`CheckedTargetProfile::assess`]. It is deliberately *not* bumped when a
/// target profile's declared bounds change: a bound is the profile's claim, and
/// the profile's canonical descriptor already distinguishes it.
const GOVERNED_FEASIBILITY_RULE_SET_REVISION: u32 = 1;

/// The one feasibility rule set every [`CheckedTargetProfile::assess`] applies.
///
/// A `const` rather than a per-target derivation, because the rules are this
/// module's code and do not vary by target: exposing a `fn(target) -> rules`
/// would imply a variation that cannot exist and would invite a second
/// definition of one identity. A consumer recording which rules assessed a
/// variant reads this; it never composes a key and a number of its own.
pub(crate) const GOVERNED_FEASIBILITY_RULE_SET: FeasibilityRuleSetIdentity =
    match FeasibilityRuleSetIdentity::new(
        GOVERNED_FEASIBILITY_RULE_SET_KEY,
        GOVERNED_FEASIBILITY_RULE_SET_REVISION,
    ) {
        Some(identity) => identity,
        // A const panic is a build failure, so a malformed governed identity can
        // never reach an artifact.
        None => panic!("the governed feasibility rule set identity is malformed"),
    };

/// Ordered capability availability phases (ADR 0043).
///
/// Re-exported rather than redefined. This crate carried its own copy with the
/// same five variants in the same order until `relocate-abi-expressions-into-tiler-ir`;
/// nothing checked that the two agreed, so a phase added to one would have left
/// the other silently unable to express it — the compiler deferring to a phase
/// the artifact layer cannot name, or the reverse, with no diagnostic. One
/// governed vocabulary now has one definition.
pub(crate) use tiler_ir::program::abi::AvailabilityPhase;

/// A governed, typed capability axis.
///
/// The vocabulary is bounded and canonically encoded; feasibility predicates
/// range over these typed axes rather than a free-form backend property bag,
/// which per ADR 0043 cannot prove correctness. The derived ordering is the
/// canonical evaluation and reporting order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CapabilityAxis {
    /// Threads dispatched along the launch grid axis.
    GridAxisThreads,
    /// Threads per workgroup.
    WorkgroupThreads,
    /// Distinct buffer bindings per kernel entry.
    BufferBindings,
    /// Index/address width in bits.
    IndexWidthBits,
    /// Availability of an explicitly addressable device memory space.
    DeviceAddressSpace,
    /// Availability of strict IEEE-754 binary32 arithmetic.
    StrictF32Arithmetic,
    /// Explicitly staged local memory, in bytes.
    LocalMemoryBytes,
    /// Barrier/collective synchronization operations.
    Barriers,
}

impl CapabilityAxis {
    /// Returns the governed tag naming this axis in a canonical descriptor.
    ///
    /// Written by an exhaustive match rather than read from the discriminant,
    /// so adding or reordering an axis is a build error here instead of a
    /// silent change to every target profile descriptor ever produced
    /// (ADR 0074 convention 3). A descriptor is durable identity: a profile
    /// whose descriptor changed without its facts changing would claim to be a
    /// different profile.
    const fn tag(self) -> u8 {
        match self {
            Self::GridAxisThreads => 0x01,
            Self::WorkgroupThreads => 0x02,
            Self::BufferBindings => 0x03,
            Self::IndexWidthBits => 0x04,
            Self::DeviceAddressSpace => 0x05,
            Self::StrictF32Arithmetic => 0x06,
            Self::LocalMemoryBytes => 0x07,
            Self::Barriers => 0x08,
        }
    }
}

/// How a candidate requirement is compared against a profile capability bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Relation {
    /// Feasible iff `required <= available` (ceilings such as threads or bytes).
    AtMost,
    /// Feasible iff `required == available` (two-sided, such as index width).
    Exact,
    /// Boolean implication: a required capability must be supported. Feasible iff
    /// `required == 0 || available != 0`.
    Implies,
}

/// The canonical axis order. This is the single source of truth for evaluation
/// and reporting order, matching the derived [`CapabilityAxis`] ordering.
const CANONICAL_AXES: [CapabilityAxis; 8] = [
    CapabilityAxis::GridAxisThreads,
    CapabilityAxis::WorkgroupThreads,
    CapabilityAxis::BufferBindings,
    CapabilityAxis::IndexWidthBits,
    CapabilityAxis::DeviceAddressSpace,
    CapabilityAxis::StrictF32Arithmetic,
    CapabilityAxis::LocalMemoryBytes,
    CapabilityAxis::Barriers,
];

impl CapabilityAxis {
    /// The governed canonical predicate key for this axis.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::GridAxisThreads => "grid-axis",
            Self::WorkgroupThreads => "threads-per-workgroup",
            Self::BufferBindings => "buffer-bindings",
            Self::IndexWidthBits => "index-bits",
            Self::DeviceAddressSpace => "device-memory",
            Self::StrictF32Arithmetic => "strict-f32",
            Self::LocalMemoryBytes => "local-memory-bytes",
            Self::Barriers => "barriers",
        }
    }

    const fn relation(self) -> Relation {
        match self {
            Self::GridAxisThreads
            | Self::WorkgroupThreads
            | Self::BufferBindings
            | Self::LocalMemoryBytes
            | Self::Barriers => Relation::AtMost,
            Self::IndexWidthBits => Relation::Exact,
            Self::DeviceAddressSpace | Self::StrictF32Arithmetic => Relation::Implies,
        }
    }

    /// Wraps a raw amount in this axis's governed quantity unit.
    pub(crate) const fn quantity(self, value: u64) -> Quantity {
        match self {
            Self::GridAxisThreads | Self::WorkgroupThreads => Quantity::Threads(value),
            Self::BufferBindings => Quantity::Bindings(value),
            Self::LocalMemoryBytes => Quantity::Bytes(value),
            Self::IndexWidthBits
            | Self::DeviceAddressSpace
            | Self::StrictF32Arithmetic
            | Self::Barriers => Quantity::Count(value),
        }
    }

    /// Whether `value` is an admissible declaration for this axis.
    ///
    /// Boolean-capability axes admit only `0` or `1`; index width must be
    /// positive. Ceilings admit any non-negative amount.
    const fn admits(self, value: u64) -> bool {
        match self.relation() {
            Relation::Implies => value <= 1,
            Relation::Exact => value > 0,
            Relation::AtMost => true,
        }
    }
}

const fn satisfies(relation: Relation, required: u64, available: u64) -> bool {
    match relation {
        Relation::AtMost => required <= available,
        Relation::Exact => required == available,
        Relation::Implies => required == 0 || available != 0,
    }
}

/// The entity vouching for a capability fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactAuthority {
    /// A governed, conservative compile-time profile guarantee.
    GovernedProfile,
    /// Evidence attributed to a produced artifact.
    ArtifactEvidence,
    /// A live device runtime.
    DeviceRuntime,
    /// A prepared, specialized kernel.
    PreparedKernel,
    /// A concrete launch instance.
    LaunchInstance,
}

impl FactAuthority {
    /// Returns the governed tag naming this authority in a canonical descriptor.
    ///
    /// Exhaustive for the same reason as [`CapabilityAxis::tag`].
    const fn tag(self) -> u8 {
        match self {
            Self::GovernedProfile => 0x01,
            Self::ArtifactEvidence => 0x02,
            Self::DeviceRuntime => 0x03,
            Self::PreparedKernel => 0x04,
            Self::LaunchInstance => 0x05,
        }
    }
}

/// The scope over which a capability fact is valid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactValidityScope {
    /// Valid for any device matching the portable profile.
    PortableProfile,
    /// Valid for one device instance only.
    DeviceInstance,
    /// Valid for one prepared artifact only.
    PreparedArtifact,
    /// Valid for one launch instance only.
    LaunchInstance,
}

impl FactValidityScope {
    /// Returns the governed tag naming this scope in a canonical descriptor.
    ///
    /// Exhaustive for the same reason as [`CapabilityAxis::tag`].
    const fn tag(self) -> u8 {
        match self {
            Self::PortableProfile => 0x01,
            Self::DeviceInstance => 0x02,
            Self::PreparedArtifact => 0x03,
            Self::LaunchInstance => 0x04,
        }
    }
}

/// Identity of a checked target profile.
///
/// The key alone is *not* the profile's identity: ADR 0043 requires the exact
/// [`CheckedTargetProfile::canonical_descriptor`] beside it, because two
/// profiles can advertise one key and admit different candidates. This type
/// names the governed key; the descriptor distinguishes profiles sharing it.
///
/// It deliberately carries no version. The rules that decide predicates are
/// [`GOVERNED_FEASIBILITY_RULE_SET`], which is a property of this authority
/// rather than of any one profile: a rule change alters every profile's
/// verdicts at once, so recording it on each profile would claim per-profile
/// variation that cannot occur. The artifact layer keeps the two apart for the
/// same reason — one profile may be re-assessed under a new rule set, and one
/// rule set applies across profiles, so neither identity determines the other.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct TargetProfileIdentity {
    key: &'static str,
}

impl TargetProfileIdentity {
    /// Constructs a target profile identity from a governed key.
    pub(crate) const fn new(key: &'static str) -> Self {
        Self { key }
    }

    /// The governed target profile key.
    pub(crate) const fn key(self) -> &'static str {
        self.key
    }
}

/// Identity of the feasibility rule set a candidate was assessed under.
///
/// Separate from [`TargetProfileIdentity`] because the two answer different
/// questions: a profile declares *what* a target can do, and a rule set governs
/// *how* a requirement is compared against that declaration. Fusing them into
/// one key-and-version pair would make an artifact assert that it was assessed
/// under a rule set named after a target profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FeasibilityRuleSetIdentity {
    key: &'static str,
    revision: u32,
}

impl FeasibilityRuleSetIdentity {
    /// Constructs a rule set identity, rejecting a malformed one.
    ///
    /// Returns [`None`] for an empty key or a zero revision. Zero is reserved
    /// for "unset" at the artifact boundary, so admitting it here would let an
    /// artifact record rules it was never assessed under.
    pub(crate) const fn new(key: &'static str, revision: u32) -> Option<Self> {
        if key.is_empty() || revision == 0 {
            return None;
        }
        Some(Self { key, revision })
    }

    /// The governed rule set key.
    pub(crate) const fn key(self) -> &'static str {
        self.key
    }

    /// The nonzero output-affecting revision of the rule set.
    pub(crate) const fn revision(self) -> u32 {
        self.revision
    }
}

/// Provenance of a capability fact: which target profile declared it.
///
/// It names the **profile**, not the rule set. A capability fact is a *bound* —
/// "at most one thread per workgroup" — and a profile is the authority that
/// declares a bound. The rule set governs how a requirement is compared against
/// that bound; it neither supplies nor admits the bound itself, so citing it
/// here would attribute the claim to something that never made it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactProvenance {
    profile: TargetProfileIdentity,
}

impl FactProvenance {
    /// Records that a fact was declared by `profile`.
    pub(crate) const fn declared_by(profile: TargetProfileIdentity) -> Self {
        Self { profile }
    }
}

/// A typed capability fact: a bound on one axis, available from a stated phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CapabilityFact {
    axis: CapabilityAxis,
    bound: u64,
    phase: AvailabilityPhase,
    authority: FactAuthority,
    validity: FactValidityScope,
    provenance: FactProvenance,
}

impl CapabilityFact {
    /// Constructs a capability fact.
    pub(crate) const fn new(
        axis: CapabilityAxis,
        bound: u64,
        phase: AvailabilityPhase,
        authority: FactAuthority,
        validity: FactValidityScope,
        provenance: FactProvenance,
    ) -> Self {
        Self {
            axis,
            bound,
            phase,
            authority,
            validity,
            provenance,
        }
    }

    /// The axis this fact bounds.
    pub(crate) const fn axis(self) -> CapabilityAxis {
        self.axis
    }

    /// The phase from which this fact is available.
    pub(crate) const fn phase(self) -> AvailabilityPhase {
        self.phase
    }

    /// The authority vouching for this fact.
    pub(crate) const fn authority(self) -> FactAuthority {
        self.authority
    }

    /// The scope over which this fact is valid.
    pub(crate) const fn validity(self) -> FactValidityScope {
        self.validity
    }

    /// Where this fact came from.
    pub(crate) const fn provenance(self) -> FactProvenance {
        self.provenance
    }
}

/// An immutable checked target profile with a key and a canonical descriptor.
///
/// Constructed only through [`CheckedTargetProfile::new`], which rejects
/// malformed declarations as intrinsic errors. There are no mutators: once
/// checked, the facts and identity are fixed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedTargetProfile {
    identity: TargetProfileIdentity,
    /// Canonical: sorted by `(axis, phase)`, unique per `(axis, phase)`.
    facts: Vec<CapabilityFact>,
}

impl CheckedTargetProfile {
    /// Builds a checked profile, validating it as an intrinsic contract.
    ///
    /// Rejects an empty identity key, a fact whose bound is inadmissible for its
    /// axis, a fact whose provenance names another profile, a fact whose declared
    /// authority contradicts its phase, and duplicate facts for the same
    /// `(axis, phase)`.
    pub(crate) fn new(
        identity: TargetProfileIdentity,
        mut facts: Vec<CapabilityFact>,
    ) -> Result<Self, FeasibilityError> {
        if identity.key().is_empty() {
            return Err(FeasibilityError::MalformedProfile { rule: "identity" });
        }
        for fact in &facts {
            if !fact.axis.admits(fact.bound) {
                return Err(FeasibilityError::MalformedProfile { rule: "fact-bound" });
            }
            if fact.provenance.profile != identity {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "fact-provenance",
                });
            }
            if !authority_matches_phase(fact.authority, fact.phase) {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "fact-authority",
                });
            }
        }
        facts.sort_by(|left, right| {
            left.axis
                .cmp(&right.axis)
                .then(left.phase.cmp(&right.phase))
        });
        if facts
            .windows(2)
            .any(|pair| pair[0].axis == pair[1].axis && pair[0].phase == pair[1].phase)
        {
            return Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-fact",
            });
        }
        Ok(Self { identity, facts })
    }

    /// The governed identity of this profile.
    pub(crate) const fn identity(&self) -> TargetProfileIdentity {
        self.identity
    }

    /// The checked capability facts, in canonical order.
    pub(crate) fn facts(&self) -> &[CapabilityFact] {
        &self.facts
    }

    /// Returns this profile's canonical descriptor bytes.
    ///
    /// These bytes *are* the profile's descriptor identity, not a hash of it.
    /// `tiler_artifact::program::TargetProfileDescriptorDigest` is a bounded
    /// opaque identity rather than a fixed-width digest, so a consumer wraps
    /// these directly. Emitting bytes rather than a hash avoids introducing a
    /// digest algorithm here and avoids a second identity that would have to be
    /// kept in agreement with the bytes it summarizes.
    ///
    /// ADR 0043 is why this exists at all: a profile *key* is not evidence that
    /// a variant is legal on a device advertising that key, because two
    /// profiles can share a key and differ in their facts. The descriptor is
    /// what distinguishes them.
    ///
    /// # What it covers, and what it deliberately does not
    ///
    /// The identity key and every fact's axis, bound, phase, authority, and
    /// validity scope — the whole of what makes one profile admit a candidate
    /// another rejects. Facts are already in the canonical `(axis, phase)` order
    /// the constructor enforces and are unique per pair, so the encoding is a
    /// function of the profile rather than of the order it was declared in.
    ///
    /// A fact's [`FactProvenance`] is excluded: it cites this profile's own
    /// identity, so folding it in would make the descriptor depend on a value
    /// derived from the descriptor's own subject.
    ///
    /// The feasibility rule set is excluded, and that exclusion is load-bearing
    /// rather than an omission. [`Self::assess`] is a function of the facts this
    /// descriptor covers and of rules that are the same for every profile:
    /// [`Self::resolve`] reads only each fact's axis, phase, and bound, and the
    /// comparison it feeds is [`CapabilityAxis::relation`], a function of the
    /// axis alone. So two profiles with equal descriptors return equal verdicts
    /// for every proposal and phase — the invariant the discarded profile
    /// *version* used to state, now discharged by the facts themselves. A rule
    /// change moves every profile's verdicts at once and is recorded beside the
    /// descriptor as [`GOVERNED_FEASIBILITY_RULE_SET`]; folding it in here would
    /// make a profile appear to have changed when only the rules did.
    pub(crate) fn canonical_descriptor(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, PROFILE_DESCRIPTOR_DOMAIN);
        push_slice(&mut bytes, self.identity.key().as_bytes());
        push_len(&mut bytes, self.facts.len());
        for fact in &self.facts {
            bytes.push(fact.axis.tag());
            bytes.extend_from_slice(&fact.bound.to_be_bytes());
            bytes.push(fact.phase.tag());
            bytes.push(fact.authority.tag());
            bytes.push(fact.validity.tag());
        }
        bytes
    }

    /// Resolves one axis against the facts available through `available_phase`.
    fn resolve(&self, axis: CapabilityAxis, available_phase: AvailabilityPhase) -> AxisResolution {
        let mut now: Option<CapabilityFact> = None;
        let mut later: Option<AvailabilityPhase> = None;
        for fact in self.facts.iter().filter(|fact| fact.axis == axis) {
            if fact.phase <= available_phase {
                // Prefer the most refined fact already available.
                now = Some(match now {
                    Some(current) if current.phase >= fact.phase => current,
                    _ => *fact,
                });
            } else {
                // Track the earliest phase that can supply the fact.
                later = Some(match later {
                    Some(phase) if phase <= fact.phase => phase,
                    _ => fact.phase,
                });
            }
        }
        match (now, later) {
            (Some(fact), _) => AxisResolution::Now(fact.bound),
            (None, Some(phase)) => AxisResolution::Later(phase),
            (None, None) => AxisResolution::NoPath,
        }
    }

    /// Assesses one candidate proposal against this profile.
    ///
    /// `available_phase` is the phase up to which facts are known; the compiler's
    /// static assessment uses [`AvailabilityPhase::CompileProfile`]. The result is
    /// always exactly one of the four outcomes; malformed inputs cannot reach here
    /// because both the profile and the proposal are validated at construction.
    pub(crate) fn assess(
        &self,
        proposal: &FeasibilityProposal,
        available_phase: AvailabilityPhase,
    ) -> FeasibilityOutcome {
        let mut proven = Vec::new();
        let mut disproved = Vec::new();
        let mut deferred = Vec::new();
        let mut unknown = Vec::new();
        for requirement in &proposal.requirements {
            let axis = requirement.axis;
            let required = axis.quantity(requirement.required);
            match self.resolve(axis, available_phase) {
                AxisResolution::Now(bound) => {
                    let resolved = ResolvedPredicate {
                        axis,
                        required,
                        available: axis.quantity(bound),
                    };
                    if satisfies(axis.relation(), requirement.required, bound) {
                        proven.push(resolved);
                    } else {
                        disproved.push(resolved);
                    }
                }
                AxisResolution::Later(phase) => deferred.push(DeferredPredicate {
                    axis,
                    required,
                    phase,
                }),
                AxisResolution::NoPath => unknown.push(UnknownPredicate { axis, required }),
            }
        }
        // Precedence: rejected, then unknown, then deferred, then proven.
        if !disproved.is_empty() {
            return FeasibilityOutcome::Rejected(Rejection { disproved });
        }
        if !unknown.is_empty() {
            return FeasibilityOutcome::Unknown(UnknownSet {
                predicates: unknown,
            });
        }
        if !deferred.is_empty() {
            deferred.sort_by(|left, right| {
                left.phase
                    .cmp(&right.phase)
                    .then(left.axis.cmp(&right.axis))
            });
            return FeasibilityOutcome::Deferred(DeferredSet {
                predicates: deferred,
            });
        }
        FeasibilityOutcome::Proven(proven)
    }

    /// Assesses a set of candidate proposals, partitioning them by outcome.
    ///
    /// An empty admitted partition is a valid, legitimate result: it means no
    /// candidate proves feasible for this target, which the caller reports as
    /// unsupported rather than as an error or as uncertainty.
    pub(crate) fn assess_set(
        &self,
        proposals: &[FeasibilityProposal],
        available_phase: AvailabilityPhase,
    ) -> FeasibleSet {
        let mut set = FeasibleSet::default();
        for proposal in proposals {
            match self.assess(proposal, available_phase) {
                FeasibilityOutcome::Proven(predicates) => {
                    set.proven.push((proposal.candidate, predicates));
                }
                FeasibilityOutcome::Deferred(deferred) => {
                    set.deferred.push((proposal.candidate, deferred));
                }
                FeasibilityOutcome::Rejected(rejection) => {
                    set.rejected.push((proposal.candidate, rejection));
                }
                FeasibilityOutcome::Unknown(unknown) => {
                    set.unknown.push((proposal.candidate, unknown));
                }
            }
        }
        set
    }
}

/// Whether a fact authority is consistent with the phase it is available from.
const fn authority_matches_phase(authority: FactAuthority, phase: AvailabilityPhase) -> bool {
    matches!(
        (authority, phase),
        (
            FactAuthority::GovernedProfile,
            AvailabilityPhase::CompileProfile
        ) | (
            FactAuthority::ArtifactEvidence,
            AvailabilityPhase::ArtifactEvidence
        ) | (
            FactAuthority::DeviceRuntime,
            AvailabilityPhase::LiveDevicePreflight
        ) | (
            FactAuthority::PreparedKernel,
            AvailabilityPhase::PreparedKernelPreflight
        ) | (
            FactAuthority::LaunchInstance,
            AvailabilityPhase::LaunchPreflight
        )
    )
}

enum AxisResolution {
    /// A fact is available now with this bound.
    Now(u64),
    /// No fact is available now, but one is admissible from this later phase.
    Later(AvailabilityPhase),
    /// No admissible proof/query path exists for the axis.
    NoPath,
}

/// A candidate requirement: a bound the candidate needs on one axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AxisRequirement {
    axis: CapabilityAxis,
    required: u64,
}

impl AxisRequirement {
    /// Constructs a requirement of `required` on `axis`.
    pub(crate) const fn new(axis: CapabilityAxis, required: u64) -> Self {
        Self { axis, required }
    }
}

/// A candidate proposal: the typed requirements one implementation places on a
/// target. This is the concrete, bounded predicate form the authority evaluates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FeasibilityProposal {
    candidate: &'static str,
    /// Canonical: sorted by axis, unique per axis.
    requirements: Vec<AxisRequirement>,
}

impl FeasibilityProposal {
    /// Builds a checked proposal, validating it as an intrinsic contract.
    ///
    /// Rejects an empty candidate identity, a requirement whose amount is
    /// inadmissible for its axis, and duplicate requirements for the same axis.
    pub(crate) fn new(
        candidate: &'static str,
        mut requirements: Vec<AxisRequirement>,
    ) -> Result<Self, FeasibilityError> {
        if candidate.is_empty() {
            return Err(FeasibilityError::MalformedProposal {
                rule: "candidate-id",
            });
        }
        for requirement in &requirements {
            if !requirement.axis.admits(requirement.required) {
                return Err(FeasibilityError::MalformedProposal {
                    rule: "requirement-amount",
                });
            }
        }
        requirements.sort_by_key(|requirement| requirement.axis);
        if requirements
            .windows(2)
            .any(|pair| pair[0].axis == pair[1].axis)
        {
            return Err(FeasibilityError::MalformedProposal {
                rule: "duplicate-requirement",
            });
        }
        Ok(Self {
            candidate,
            requirements,
        })
    }

    /// The stable candidate identity.
    pub(crate) const fn candidate(&self) -> &'static str {
        self.candidate
    }
}

/// A predicate resolved against an available fact, retaining both quantities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedPredicate {
    axis: CapabilityAxis,
    required: Quantity,
    available: Quantity,
}

impl ResolvedPredicate {
    /// The axis this predicate ranges over.
    pub(crate) const fn axis(self) -> CapabilityAxis {
        self.axis
    }

    /// The required quantity.
    pub(crate) const fn required(self) -> Quantity {
        self.required
    }

    /// The available quantity that resolved the predicate.
    pub(crate) const fn available(self) -> Quantity {
        self.available
    }
}

/// A predicate whose resolving fact is admissible only from a later phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeferredPredicate {
    axis: CapabilityAxis,
    required: Quantity,
    phase: AvailabilityPhase,
}

impl DeferredPredicate {
    /// The axis this predicate ranges over.
    pub(crate) const fn axis(self) -> CapabilityAxis {
        self.axis
    }

    /// The required quantity.
    pub(crate) const fn required(self) -> Quantity {
        self.required
    }

    /// The earliest phase that can resolve the predicate.
    pub(crate) const fn phase(self) -> AvailabilityPhase {
        self.phase
    }
}

/// A predicate with no admissible proof/query path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnknownPredicate {
    axis: CapabilityAxis,
    required: Quantity,
}

impl UnknownPredicate {
    /// The axis this predicate ranges over.
    pub(crate) const fn axis(self) -> CapabilityAxis {
        self.axis
    }

    /// The required quantity.
    pub(crate) const fn required(self) -> Quantity {
        self.required
    }
}

/// The nonempty disproved predicates that reject a candidate, canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Rejection {
    disproved: Vec<ResolvedPredicate>,
}

impl Rejection {
    /// The canonical representative disproved predicate (first in axis order).
    ///
    /// The disproved set is nonempty by construction, so this never panics.
    pub(crate) fn representative(&self) -> ResolvedPredicate {
        self.disproved[0]
    }

    /// All disproved predicates, in canonical axis order.
    pub(crate) fn disproved(&self) -> &[ResolvedPredicate] {
        &self.disproved
    }
}

/// One nonempty canonical deferred set, grouped by phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeferredSet {
    /// Canonical: sorted by `(phase, axis)`.
    predicates: Vec<DeferredPredicate>,
}

impl DeferredSet {
    /// The deferred predicates, canonical `(phase, axis)` order.
    pub(crate) fn predicates(&self) -> &[DeferredPredicate] {
        &self.predicates
    }

    /// The distinct phases the deferred checks resolve at, ascending.
    pub(crate) fn phases(&self) -> Vec<AvailabilityPhase> {
        let mut phases: Vec<AvailabilityPhase> = self
            .predicates
            .iter()
            .map(|predicate| predicate.phase())
            .collect();
        phases.dedup();
        phases
    }
}

/// The nonempty set of predicates with no admissible proof/query path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnknownSet {
    predicates: Vec<UnknownPredicate>,
}

impl UnknownSet {
    /// The unknown predicates, in canonical axis order.
    pub(crate) fn predicates(&self) -> &[UnknownPredicate] {
        &self.predicates
    }
}

/// The four target-feasibility outcomes (ADR 0043).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FeasibilityOutcome {
    /// Every check resolved and is satisfied; the candidate may enter the
    /// executable frontier. Carries the resolved predicates in canonical order.
    Proven(Vec<ResolvedPredicate>),
    /// Some checks are unresolved but admissible from a later phase.
    Deferred(DeferredSet),
    /// At least one hard predicate is disproved.
    Rejected(Rejection),
    /// At least one predicate has no admissible proof/query path. An unknown
    /// candidate may remain in search/explain state but cannot enter an
    /// executable frontier or manifest.
    Unknown(UnknownSet),
}

/// The partition of a proposal set by outcome.
///
/// The admitted (`proven`) partition may legitimately be empty; that is a valid
/// result distinct from a malformed-input error and from an unknown candidate.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FeasibleSet {
    proven: Vec<(&'static str, Vec<ResolvedPredicate>)>,
    deferred: Vec<(&'static str, DeferredSet)>,
    rejected: Vec<(&'static str, Rejection)>,
    unknown: Vec<(&'static str, UnknownSet)>,
}

impl FeasibleSet {
    /// The proven (admitted) candidates and their resolved predicates.
    pub(crate) fn proven(&self) -> &[(&'static str, Vec<ResolvedPredicate>)] {
        &self.proven
    }

    /// The deferred candidates.
    pub(crate) fn deferred(&self) -> &[(&'static str, DeferredSet)] {
        &self.deferred
    }

    /// The rejected candidates.
    pub(crate) fn rejected(&self) -> &[(&'static str, Rejection)] {
        &self.rejected
    }

    /// The unknown candidates.
    pub(crate) fn unknown(&self) -> &[(&'static str, UnknownSet)] {
        &self.unknown
    }

    /// Whether no candidate proves feasible. A legitimate, non-error result.
    pub(crate) fn admitted_is_empty(&self) -> bool {
        self.proven.is_empty()
    }
}

/// An intrinsic error in a target profile or candidate proposal.
///
/// Distinct from every feasibility outcome: a malformed input is a contract
/// violation, not a statement about whether a candidate is feasible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FeasibilityError {
    /// A target profile was declared inconsistently.
    MalformedProfile { rule: &'static str },
    /// A candidate proposal was declared inconsistently.
    MalformedProposal { rule: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASELINE_KEY: &str = "tiler.test.baseline.v1";

    fn identity() -> TargetProfileIdentity {
        TargetProfileIdentity::new(BASELINE_KEY)
    }

    fn compile_fact(id: TargetProfileIdentity, axis: CapabilityAxis, bound: u64) -> CapabilityFact {
        CapabilityFact::new(
            axis,
            bound,
            AvailabilityPhase::CompileProfile,
            FactAuthority::GovernedProfile,
            FactValidityScope::PortableProfile,
            FactProvenance::declared_by(id),
        )
    }

    /// The bounded serial-Sum baseline: every axis resolvable at compile time.
    fn baseline_profile() -> CheckedTargetProfile {
        let id = identity();
        CheckedTargetProfile::new(
            id,
            vec![
                compile_fact(id, CapabilityAxis::GridAxisThreads, 65_535),
                compile_fact(id, CapabilityAxis::WorkgroupThreads, 1),
                compile_fact(id, CapabilityAxis::BufferBindings, 2),
                compile_fact(id, CapabilityAxis::IndexWidthBits, 64),
                compile_fact(id, CapabilityAxis::DeviceAddressSpace, 1),
                compile_fact(id, CapabilityAxis::StrictF32Arithmetic, 1),
                compile_fact(id, CapabilityAxis::LocalMemoryBytes, 0),
                compile_fact(id, CapabilityAxis::Barriers, 0),
            ],
        )
        .unwrap()
    }

    /// The descriptor distinguishes profiles a key alone cannot tell apart.
    ///
    /// ADR 0043's whole reason for requiring a descriptor beside the key is
    /// that two profiles can advertise one key and admit different candidates.
    /// Each pair below shares a key and must not share a descriptor.
    #[test]
    fn the_canonical_profile_descriptor_separates_profiles_sharing_a_key() {
        let baseline = baseline_profile();
        let descriptor = baseline.canonical_descriptor();

        assert_eq!(
            descriptor,
            baseline_profile().canonical_descriptor(),
            "the descriptor is a function of the profile, not of when it was built",
        );
        assert!(
            descriptor.starts_with(&(PROFILE_DESCRIPTOR_DOMAIN.len() as u64).to_be_bytes()),
            "the descriptor is domain-separated and length-framed",
        );

        // A differing bound on one axis: same key, same axes, different profile.
        let id = identity();
        let mut facts: Vec<_> = baseline.facts().to_vec();
        facts[0] = compile_fact(id, CapabilityAxis::GridAxisThreads, 1_024);
        let narrower = CheckedTargetProfile::new(id, facts).unwrap();
        assert_eq!(narrower.identity().key(), baseline.identity().key());
        assert_ne!(
            narrower.canonical_descriptor(),
            descriptor,
            "a profile that admits fewer candidates must not share a descriptor",
        );

        // A fact moved to a later phase: same key, same axes, same bounds, but
        // the profile now defers where the baseline proved. This is the case the
        // discarded profile *version* was supposed to catch and never could,
        // because nothing forced a declarer to bump it; the facts carry it.
        let mut deferred_facts: Vec<_> = baseline.facts().to_vec();
        deferred_facts[0] = CapabilityFact::new(
            CapabilityAxis::GridAxisThreads,
            65_535,
            AvailabilityPhase::LiveDevicePreflight,
            FactAuthority::DeviceRuntime,
            FactValidityScope::DeviceInstance,
            FactProvenance::declared_by(id),
        );
        let later = CheckedTargetProfile::new(id, deferred_facts).unwrap();
        assert_eq!(later.identity().key(), baseline.identity().key());
        assert_ne!(
            later.canonical_descriptor(),
            descriptor,
            "a profile that resolves an axis at a later phase must not share a descriptor",
        );
        assert!(matches!(
            later.assess(
                &baseline_proposal("candidate:baseline", 6),
                AvailabilityPhase::CompileProfile,
            ),
            FeasibilityOutcome::Deferred(_),
        ));

        // The consumer wraps these bytes directly rather than hashing them, so
        // they must fit the artifact layer's opaque-identity bound. If a profile
        // ever outgrows it, that is when a digest becomes a real decision with a
        // real reason -- and it fails here rather than silently truncating.
        assert!(
            descriptor.len() <= 1_024,
            "descriptor is {} bytes, past the governed opaque-identity bound",
            descriptor.len(),
        );
    }

    fn baseline_proposal(candidate: &'static str, grid_threads: u64) -> FeasibilityProposal {
        FeasibilityProposal::new(
            candidate,
            vec![
                AxisRequirement::new(CapabilityAxis::GridAxisThreads, grid_threads),
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1),
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
                AxisRequirement::new(CapabilityAxis::IndexWidthBits, 64),
                AxisRequirement::new(CapabilityAxis::DeviceAddressSpace, 1),
                AxisRequirement::new(CapabilityAxis::StrictF32Arithmetic, 1),
                AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, 0),
                AxisRequirement::new(CapabilityAxis::Barriers, 0),
            ],
        )
        .unwrap()
    }

    #[test]
    fn availability_phases_are_totally_ordered_by_earliness() {
        assert!(AvailabilityPhase::CompileProfile < AvailabilityPhase::ArtifactEvidence);
        assert!(AvailabilityPhase::ArtifactEvidence < AvailabilityPhase::LiveDevicePreflight);
        assert!(
            AvailabilityPhase::LiveDevicePreflight < AvailabilityPhase::PreparedKernelPreflight
        );
        assert!(AvailabilityPhase::PreparedKernelPreflight < AvailabilityPhase::LaunchPreflight);
    }

    #[test]
    fn baseline_candidate_is_proven_with_canonical_resolved_predicates() {
        let outcome = baseline_profile().assess(
            &baseline_proposal("candidate:baseline", 6),
            AvailabilityPhase::CompileProfile,
        );
        let FeasibilityOutcome::Proven(predicates) = outcome else {
            panic!("baseline candidate must prove feasible");
        };
        assert_eq!(
            predicates.iter().map(|p| p.axis()).collect::<Vec<_>>(),
            CANONICAL_AXES.to_vec()
        );
        let grid = predicates[0];
        assert_eq!(grid.required(), Quantity::Threads(6));
        assert_eq!(grid.available(), Quantity::Threads(65_535));
    }

    #[test]
    fn empty_proposal_is_vacuously_proven() {
        let outcome = baseline_profile().assess(
            &FeasibilityProposal::new("candidate:empty", Vec::new()).unwrap(),
            AvailabilityPhase::CompileProfile,
        );
        assert_eq!(outcome, FeasibilityOutcome::Proven(Vec::new()));
    }

    #[test]
    fn a_disproved_hard_predicate_rejects_with_a_canonical_representative() {
        let outcome = baseline_profile().assess(
            &baseline_proposal("candidate:oversized", 140_000),
            AvailabilityPhase::CompileProfile,
        );
        let FeasibilityOutcome::Rejected(rejection) = outcome else {
            panic!("oversized grid must reject");
        };
        assert_eq!(
            rejection.representative().axis(),
            CapabilityAxis::GridAxisThreads
        );
        assert_eq!(
            rejection.representative().required(),
            Quantity::Threads(140_000)
        );
        assert_eq!(
            rejection.representative().available(),
            Quantity::Threads(65_535)
        );
    }

    #[test]
    fn rejection_takes_precedence_over_unknown_and_deferred() {
        // One axis is disproved, one is unknown (no fact declared), one is
        // deferred (declared only at a later phase). Rejection must win.
        let id = identity();
        let profile = CheckedTargetProfile::new(
            id,
            vec![
                compile_fact(id, CapabilityAxis::GridAxisThreads, 4),
                CapabilityFact::new(
                    CapabilityAxis::BufferBindings,
                    8,
                    AvailabilityPhase::LiveDevicePreflight,
                    FactAuthority::DeviceRuntime,
                    FactValidityScope::DeviceInstance,
                    FactProvenance::declared_by(id),
                ),
            ],
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:mixed",
            vec![
                AxisRequirement::new(CapabilityAxis::GridAxisThreads, 9),
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1),
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
            ],
        )
        .unwrap();
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Rejected(_)
        ));
    }

    #[test]
    fn unknown_takes_precedence_over_deferred() {
        let id = identity();
        let profile = CheckedTargetProfile::new(
            id,
            vec![CapabilityFact::new(
                CapabilityAxis::BufferBindings,
                8,
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
                FactProvenance::declared_by(id),
            )],
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:unknown-and-deferred",
            vec![
                // No fact for WorkgroupThreads at all -> unknown.
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1),
                // BufferBindings only at a later phase -> deferred.
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
            ],
        )
        .unwrap();
        let outcome = profile.assess(&proposal, AvailabilityPhase::CompileProfile);
        let FeasibilityOutcome::Unknown(unknown) = outcome else {
            panic!("an unknown predicate outranks a deferred one");
        };
        assert_eq!(
            unknown
                .predicates()
                .iter()
                .map(|p| p.axis())
                .collect::<Vec<_>>(),
            vec![CapabilityAxis::WorkgroupThreads]
        );
    }

    #[test]
    fn unresolved_checks_form_one_canonical_deferred_set_grouped_by_phase() {
        let id = identity();
        let profile = CheckedTargetProfile::new(
            id,
            vec![
                // WorkgroupThreads resolvable only at a prepared-kernel preflight.
                CapabilityFact::new(
                    CapabilityAxis::WorkgroupThreads,
                    256,
                    AvailabilityPhase::PreparedKernelPreflight,
                    FactAuthority::PreparedKernel,
                    FactValidityScope::PreparedArtifact,
                    FactProvenance::declared_by(id),
                ),
                // BufferBindings resolvable at the earlier live-device preflight.
                CapabilityFact::new(
                    CapabilityAxis::BufferBindings,
                    8,
                    AvailabilityPhase::LiveDevicePreflight,
                    FactAuthority::DeviceRuntime,
                    FactValidityScope::DeviceInstance,
                    FactProvenance::declared_by(id),
                ),
            ],
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:deferred",
            vec![
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 64),
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
            ],
        )
        .unwrap();
        let outcome = profile.assess(&proposal, AvailabilityPhase::CompileProfile);
        let FeasibilityOutcome::Deferred(deferred) = outcome else {
            panic!("later-phase facts must defer");
        };
        // Grouped by phase, ascending: LiveDevicePreflight before
        // PreparedKernelPreflight, independent of requirement authoring order.
        assert_eq!(
            deferred
                .predicates()
                .iter()
                .map(|p| p.phase())
                .collect::<Vec<_>>(),
            vec![
                AvailabilityPhase::LiveDevicePreflight,
                AvailabilityPhase::PreparedKernelPreflight,
            ]
        );
        assert_eq!(
            deferred.phases(),
            vec![
                AvailabilityPhase::LiveDevicePreflight,
                AvailabilityPhase::PreparedKernelPreflight,
            ]
        );
    }

    #[test]
    fn a_deferred_fact_resolves_once_its_phase_is_available() {
        let id = identity();
        let profile = CheckedTargetProfile::new(
            id,
            vec![CapabilityFact::new(
                CapabilityAxis::WorkgroupThreads,
                256,
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
                FactProvenance::declared_by(id),
            )],
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:resolves-later",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 64)],
        )
        .unwrap();
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Deferred(_)
        ));
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::LiveDevicePreflight),
            FeasibilityOutcome::Proven(_)
        ));
    }

    #[test]
    fn a_required_axis_with_no_fact_is_unknown() {
        let id = identity();
        let profile = CheckedTargetProfile::new(
            id,
            vec![compile_fact(id, CapabilityAxis::GridAxisThreads, 4)],
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:unprovable",
            vec![AxisRequirement::new(CapabilityAxis::Barriers, 1)],
        )
        .unwrap();
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::LaunchPreflight),
            FeasibilityOutcome::Unknown(_)
        ));
    }

    #[test]
    fn an_empty_feasible_set_is_a_valid_result_not_an_error() {
        let profile = baseline_profile();
        let rejected = baseline_proposal("candidate:too-big", 10_000_000);
        let set = profile.assess_set(
            std::slice::from_ref(&rejected),
            AvailabilityPhase::CompileProfile,
        );
        assert!(set.admitted_is_empty());
        assert_eq!(set.rejected().len(), 1);
        assert_eq!(set.rejected()[0].0, "candidate:too-big");
    }

    #[test]
    fn a_feasible_set_partitions_candidates_by_outcome() {
        let profile = baseline_profile();
        let proven = baseline_proposal("candidate:ok", 6);
        let rejected = baseline_proposal("candidate:big", 10_000_000);
        let set = profile.assess_set(&[proven, rejected], AvailabilityPhase::CompileProfile);
        assert_eq!(set.proven().len(), 1);
        assert_eq!(set.proven()[0].0, "candidate:ok");
        assert_eq!(set.rejected().len(), 1);
        assert!(!set.admitted_is_empty());
    }

    #[test]
    fn malformed_profiles_are_intrinsic_errors_not_outcomes() {
        let id = identity();
        assert_eq!(
            CheckedTargetProfile::new(TargetProfileIdentity::new(""), Vec::new()),
            Err(FeasibilityError::MalformedProfile { rule: "identity" })
        );
        // A boolean-capability axis with a non-boolean bound is malformed.
        assert_eq!(
            CheckedTargetProfile::new(
                id,
                vec![compile_fact(id, CapabilityAxis::DeviceAddressSpace, 2)],
            ),
            Err(FeasibilityError::MalformedProfile { rule: "fact-bound" })
        );
        // Two facts for the same axis and phase are malformed.
        assert_eq!(
            CheckedTargetProfile::new(
                id,
                vec![
                    compile_fact(id, CapabilityAxis::GridAxisThreads, 4),
                    compile_fact(id, CapabilityAxis::GridAxisThreads, 8),
                ],
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-fact"
            })
        );
        // A fact whose authority contradicts its phase is malformed.
        assert_eq!(
            CheckedTargetProfile::new(
                id,
                vec![CapabilityFact::new(
                    CapabilityAxis::GridAxisThreads,
                    4,
                    AvailabilityPhase::LiveDevicePreflight,
                    FactAuthority::GovernedProfile,
                    FactValidityScope::PortableProfile,
                    FactProvenance::declared_by(id),
                )],
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "fact-authority"
            })
        );
        // A fact whose provenance names a different profile is malformed.
        let other = TargetProfileIdentity::new("tiler.test.other.v1");
        assert_eq!(
            CheckedTargetProfile::new(id, vec![compile_fact(other, CapabilityAxis::Barriers, 0)]),
            Err(FeasibilityError::MalformedProfile {
                rule: "fact-provenance"
            })
        );
    }

    #[test]
    fn malformed_proposals_are_intrinsic_errors() {
        assert_eq!(
            FeasibilityProposal::new("", Vec::new()),
            Err(FeasibilityError::MalformedProposal {
                rule: "candidate-id"
            })
        );
        assert_eq!(
            FeasibilityProposal::new(
                "candidate:dup",
                vec![
                    AxisRequirement::new(CapabilityAxis::GridAxisThreads, 4),
                    AxisRequirement::new(CapabilityAxis::GridAxisThreads, 8),
                ],
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "duplicate-requirement"
            })
        );
        assert_eq!(
            FeasibilityProposal::new(
                "candidate:bad-bool",
                vec![AxisRequirement::new(CapabilityAxis::StrictF32Arithmetic, 5)],
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "requirement-amount"
            })
        );
    }

    #[test]
    fn checked_profile_exposes_canonical_facts_and_its_governed_identity() {
        let profile = baseline_profile();
        assert_eq!(profile.identity(), identity());
        assert_eq!(profile.identity().key(), BASELINE_KEY);
        // Facts are sorted into canonical axis order regardless of input order.
        let axes: Vec<_> = profile.facts().iter().map(|fact| fact.axis()).collect();
        assert_eq!(axes, CANONICAL_AXES.to_vec());
    }

    /// The two identities are separate values a consumer records separately.
    ///
    /// Before the split, one `key`/`version` pair carried a profile key and a
    /// rule revision, so a consumer building the artifact layer's two
    /// independent references had to name the rule set after the profile. The
    /// assertions below are that neither identity is recoverable from, or equal
    /// to, the other, and that the rule set's revision is a real nonzero value
    /// rather than one a consumer would have to invent.
    #[test]
    fn the_profile_and_rule_set_identities_are_independent_and_complete() {
        let profile = baseline_profile();
        let rules = GOVERNED_FEASIBILITY_RULE_SET;

        assert_ne!(rules.key(), profile.identity().key());
        assert!(rules.revision() > 0);
        assert!(!rules.key().is_empty());

        // The descriptor is the profile's identity beside its key, and it does
        // not carry the rule set: a consumer that recorded only the descriptor
        // would silently claim rule-set independence it does not have, which is
        // why the rule set is a second recorded reference rather than a field.
        let descriptor = profile.canonical_descriptor();
        assert!(
            !descriptor
                .windows(rules.key().len())
                .any(|window| window == rules.key().as_bytes()),
            "the profile descriptor must not encode the rule set key",
        );

        // A malformed rule set identity is rejected rather than defaulted, so
        // the reserved "unset" revision cannot reach an artifact.
        assert_eq!(FeasibilityRuleSetIdentity::new("tiler.rules.v1", 0), None);
        assert_eq!(FeasibilityRuleSetIdentity::new("", 1), None);
    }

    /// Equal descriptors imply equal verdicts, which is what lets the profile
    /// version go away.
    ///
    /// The discarded `ProfileIdentity::version` documented the invariant "two
    /// profiles that would evaluate predicates differently must not share a
    /// version". Nothing enforced it — a declarer could reuse a version for a
    /// changed profile. The descriptor discharges it structurally, because
    /// assessment reads exactly the axis, phase, and bound the descriptor
    /// encodes and otherwise only `CapabilityAxis::relation`, which is a
    /// function of the axis rather than of the profile.
    #[test]
    fn profiles_sharing_a_descriptor_return_the_same_verdicts() {
        let id = identity();
        let rebuilt = CheckedTargetProfile::new(
            id,
            // Declared in a deliberately different order from `baseline_profile`.
            baseline_profile()
                .facts()
                .iter()
                .rev()
                .map(|fact| compile_fact(id, fact.axis, fact.bound))
                .collect(),
        )
        .unwrap();
        assert_eq!(
            rebuilt.canonical_descriptor(),
            baseline_profile().canonical_descriptor(),
        );
        for threads in [0, 6, 65_535, 65_536, 10_000_000] {
            let proposal = baseline_proposal("candidate:probe", threads);
            for phase in [
                AvailabilityPhase::CompileProfile,
                AvailabilityPhase::LiveDevicePreflight,
                AvailabilityPhase::LaunchPreflight,
            ] {
                assert_eq!(
                    rebuilt.assess(&proposal, phase),
                    baseline_profile().assess(&proposal, phase),
                );
            }
        }
    }
}
