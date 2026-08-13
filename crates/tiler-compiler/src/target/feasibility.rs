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
//! # Two kinds of predicate, one verdict
//!
//! A proposal carries typed [`AxisRequirement`]s over the quantitative
//! [`CapabilityAxis`] space *and* typed
//! [`crate::target::honourability::NumericalRequirement`]s over the per-dimension
//! numerical-honourability space (ADR 0076 item 3). They are different
//! authorities — [`crate::target::honourability`] owns the second vocabulary, and
//! numerical honourability is deliberately not a `CapabilityAxis`, because
//! `SupportedWithExactEmulation` has no representation as a bound comparison —
//! but a candidate has exactly one feasibility verdict, so the two compose here
//! into one [`FeasibilityOutcome`] under the same precedence. The composition is
//! stated rather than implicit: a dimension honoured exactly or by emulation is a
//! satisfied hard predicate; a dimension honourable only under a relaxation the
//! caller's contract does not authorize is *disproved*, not deferred and not
//! unknown, because that authorization is known at
//! [`AvailabilityPhase::CompileProfile`] and cannot arrive later; a dimension
//! declared unhonourable is disproved; and a dimension the profile does not
//! speak to is [`FeasibilityOutcome::Unknown`] in ADR 0043's exact sense, which
//! is what makes an unenumerated dimension fail closed rather than default to
//! honoured.
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

use std::sync::Arc;
use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::{
    PreparedEntryTargetRequirement, TargetPropertyQuery, TargetPropertyRequirementRelation,
};

use tiler_ir::schedule::{ArithmeticType, SynchronizationSubject};

use crate::explain::Quantity;
pub(crate) use crate::target::TargetProfileIdentity;
use crate::target::honourability::{
    DeferredDimension, DimensionBehaviour, FactSourceProvenance, HonouredDimension, HonouringMeans,
    NumericalDimension, NumericalHonourabilityFact, NumericalRequirement, UndeclaredDimension,
    UnhonouredDimension, encode_honourability_facts,
};

/// Domain separator of a canonical target profile descriptor.
///
/// Trailing NUL so no descriptor can be a prefix of a differently-domained
/// encoding, matching the framing the rest of the workspace's identities use.
///
/// `v10` adds the synchronization-realization declaration. It is folded into the
/// descriptor rather than kept beside it because it decides verdicts exactly as a
/// bound does: two profiles sharing a key and differing only in which
/// realization they declare admit different candidates, and a `v9` descriptor
/// could not tell them apart.
///
/// `v9` distinguishes observed capability facts from executable later-phase
/// query schemas. A future measured value no longer masquerades as proof that a
/// runtime knows how to obtain that value.
///
/// `v8` retires the conflated index/address-width axis and adds independent
/// operation-complete unsigned-64 index arithmetic and device-address-width
/// facts. Tag `0x04` remains reserved.
///
/// `v7` retires the invented numeric barrier-capacity axis. Tag `0x08` remains
/// reserved, but a schedule with no synchronization now has no predicate to
/// prove.
///
/// `v6` because every numerical row now carries its complete resolved semantic
/// type. `v5` carried only the arithmetic class in each row, allowing two
/// distinct same-class subjects to share feasibility identity.
///
/// `v4` introduced each numerical fact's versioned
/// authority and its governed-guarantee or measured compiler/environment source.
/// `v3` distinguished per-dimension behaviours after the strict-arithmetic
/// boolean was retired, but two profiles resting on different measured builds
/// still collided under it.
const PROFILE_DESCRIPTOR_DOMAIN: &[u8] = b"tiler.target-profile.descriptor.v10\0";

/// Governed key of the feasibility rule set this authority applies.
///
/// The version suffix names the governed *vocabulary* the rules range over — the
/// ADR 0043 capability axes, availability phases, fact authorities, and validity
/// scopes, and now the ADR 0076 numerical dimensions, behaviours, and honouring
/// means — not an output-affecting revision within it. Widening that vocabulary
/// mints a new key because the rules would then decide predicates the old key
/// could not express; changing how the same terms are compared bumps
/// [`GOVERNED_FEASIBILITY_RULE_SET_REVISION`] instead. That is why the artifact
/// layer's `FeasibilityRuleSetRef` carries both a key and a revision rather than
/// one number.
///
/// `v5` adds the atomic synchronization-realization predicate. This is a
/// *vocabulary* widening rather than a revision: the rules now decide a
/// predicate `v4` could not express at all, and a `v4` assessment of a
/// synchronized candidate could only have been silent about it. Tag `0x08` of
/// the [`CapabilityAxis`] space stays retired — the new predicate is not a
/// quantitative axis, because a subject is matched by equality and has no bound
/// to compare, which is the same reason numerical honourability is not one.
///
/// `v4` adds typed target-property query paths to deferred capability
/// predicates. Under v3, any fact assigned a later phase created `Deferred`
/// despite carrying no executable query contract.
///
/// `v3` retires the conflated index/address-width predicate and adds independent
/// operation-complete unsigned-64 index arithmetic and device-address-width
/// predicates.
///
/// `v2` retires the numeric barrier-capacity predicate from the vocabulary.
/// `v1` could decide a predicate the corrected authority cannot express, so a
/// revision bump would violate the key's vocabulary boundary.
///
/// The family originally replaced
/// `tiler.feasibility.phased-capability-bounds.v1`, whose vocabulary was
/// capability bounds alone: it could neither express a per-dimension numerical
/// predicate nor decide one, and it named an axis (`strict-f32`) this rule set
/// no longer has.
const GOVERNED_FEASIBILITY_RULE_SET_KEY: &str =
    "tiler.feasibility.phased-capability-and-numerical-honourability.v5";

/// Nonzero output-affecting revision of the governed feasibility rule set.
///
/// Bumped when this module changes *how* a requirement is compared against a
/// bound or a declaration — an axis [`Relation`], [`satisfies`],
/// [`CapabilityAxis::admits`], [`authority_matches_phase`],
/// [`CheckedTargetProfile::resolve`]'s preference for the most refined available
/// fact, the mapping of a [`HonouringMeans`] onto a verdict in
/// [`CheckedTargetProfile::resolve_dimension`], or the outcome precedence in
/// [`CheckedTargetProfile::assess`]. It is deliberately *not* bumped when a
/// target profile's declared bounds or honourability change: those are the
/// profile's claims, and the profile's canonical descriptor already
/// distinguishes them.
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
///
/// This space is *quantitative*: every axis has a `u64` bound, a
/// [`Quantity`] unit, and a comparison [`Relation`]. Numerical behaviour is
/// deliberately not in it — see [`crate::target::honourability`] — because a bound
/// comparison can report whether an obligation is met and never by what means,
/// and the means is what an emulated dimension's emitted operations depend on.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CapabilityAxis {
    /// Threads dispatched along the launch grid axis.
    GridAxisThreads,
    /// Threads per workgroup.
    WorkgroupThreads,
    /// Distinct buffer bindings per kernel entry.
    BufferBindings,
    /// Availability of an explicitly addressable device memory space.
    DeviceAddressSpace,
    /// Explicitly staged local memory, in bytes.
    LocalMemoryBytes,
    /// Complete support for the governed unsigned-64 KIR index operation family.
    IndexArithmeticU64,
    /// Device address-model width in bits.
    DeviceAddressWidthBits,
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
    ///
    /// `0x04`, `0x06`, and `0x08` are retired tags, not free ones. They named
    /// the conflated index/address width, withdrawn `StrictF32Arithmetic`, and
    /// numeric barrier-count axes. Reusing one would let a descriptor mean
    /// something a reader of the retirement would not expect. New axes take the
    /// next unused value.
    const fn tag(self) -> u8 {
        match self {
            Self::GridAxisThreads => 0x01,
            Self::WorkgroupThreads => 0x02,
            Self::BufferBindings => 0x03,
            Self::DeviceAddressSpace => 0x05,
            Self::LocalMemoryBytes => 0x07,
            Self::IndexArithmeticU64 => 0x09,
            Self::DeviceAddressWidthBits => 0x0a,
        }
    }
}

/// How a candidate requirement is compared against a profile capability bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CapabilityRelation {
    /// Feasible iff `required <= available` (ceilings such as threads or bytes).
    AtMost,
    /// Feasible iff `required == available` (two-sided, such as address width).
    Exact,
    /// Boolean implication: a required capability must be supported. Feasible iff
    /// `required == 0 || available != 0`.
    Implies,
}

/// The canonical axis order. This is the single source of truth for evaluation
/// and reporting order, matching the derived [`CapabilityAxis`] ordering.
const CANONICAL_AXES: [CapabilityAxis; 7] = [
    CapabilityAxis::GridAxisThreads,
    CapabilityAxis::WorkgroupThreads,
    CapabilityAxis::BufferBindings,
    CapabilityAxis::DeviceAddressSpace,
    CapabilityAxis::LocalMemoryBytes,
    CapabilityAxis::IndexArithmeticU64,
    CapabilityAxis::DeviceAddressWidthBits,
];

impl CapabilityAxis {
    /// The governed canonical predicate key for this axis.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::GridAxisThreads => "grid-axis",
            Self::WorkgroupThreads => "threads-per-workgroup",
            Self::BufferBindings => "buffer-bindings",
            Self::DeviceAddressSpace => "device-memory",
            Self::LocalMemoryBytes => "local-memory-bytes",
            Self::IndexArithmeticU64 => "index-arithmetic-u64",
            Self::DeviceAddressWidthBits => "device-address-bits",
        }
    }

    const fn relation(self) -> CapabilityRelation {
        match self {
            Self::GridAxisThreads
            | Self::WorkgroupThreads
            | Self::BufferBindings
            | Self::LocalMemoryBytes => CapabilityRelation::AtMost,
            Self::DeviceAddressSpace | Self::IndexArithmeticU64 => CapabilityRelation::Implies,
            Self::DeviceAddressWidthBits => CapabilityRelation::Exact,
        }
    }

    /// Wraps a raw amount in this axis's governed quantity unit.
    pub(crate) const fn quantity(self, value: u64) -> Quantity {
        match self {
            Self::GridAxisThreads | Self::WorkgroupThreads => Quantity::Threads(value),
            Self::BufferBindings => Quantity::Bindings(value),
            Self::LocalMemoryBytes => Quantity::Bytes(value),
            Self::DeviceAddressSpace | Self::IndexArithmeticU64 => Quantity::Count(value),
            Self::DeviceAddressWidthBits => Quantity::Bits(value),
        }
    }

    /// Whether `value` is an admissible declaration for this axis.
    ///
    /// Boolean-capability axes admit only `0` or `1`; exact quantities must be
    /// positive. Ceilings admit any non-negative amount.
    const fn admits(self, value: u64) -> bool {
        match self.relation() {
            CapabilityRelation::Implies => value <= 1,
            CapabilityRelation::Exact => value > 0,
            CapabilityRelation::AtMost => true,
        }
    }
}

const fn satisfies(relation: CapabilityRelation, required: u64, available: u64) -> bool {
    match relation {
        CapabilityRelation::AtMost => required <= available,
        CapabilityRelation::Exact => required == available,
        CapabilityRelation::Implies => required == 0 || available != 0,
    }
}

const fn target_property_relation(
    relation: CapabilityRelation,
) -> TargetPropertyRequirementRelation {
    match relation {
        CapabilityRelation::AtMost => TargetPropertyRequirementRelation::ObservedAtLeastRequired,
        CapabilityRelation::Exact => TargetPropertyRequirementRelation::ObservedEqualsRequired,
        CapabilityRelation::Implies => TargetPropertyRequirementRelation::RequiredImpliesObserved,
    }
}

// The fact-provenance vocabulary is `tiler_ir::numerics`, not this module's.
// A capability fact and a numerical fact answer to the same authorities and the
// same validity scopes, and the delivered-realization record has to read both;
// naming one authority by re-export is what keeps the two from drifting. The
// out-of-declaration-order wire tags (`ExternalProfile` 0x06, `MeasuredProfile`
// 0x07, `MeasuredEnvironment` 0x05) are preserved there byte for byte, because
// renumbering them would silently restate every target-profile descriptor that
// declares a measured fact.
pub(crate) use tiler_ir::numerics::{FactAuthority, FactValidityScope};

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FactProvenance {
    profile: TargetProfileIdentity,
}

impl FactProvenance {
    /// Records that a fact was declared by `profile`.
    pub(crate) fn declared_by(profile: impl Into<TargetProfileIdentity>) -> Self {
        Self {
            profile: profile.into(),
        }
    }

    /// The profile that declared the fact.
    pub(crate) const fn profile(&self) -> &TargetProfileIdentity {
        &self.profile
    }
}

/// Whether a target realizes one complete synchronization subject.
///
/// Two valued rather than a presence marker, for the reason
/// [`crate::target::DTypeDispatchability`] is: a measured negative is a fact
/// worth recording, and a profile that could only stay silent about what it
/// cannot do would make "unsupported" and "unmeasured" one state. They are not —
/// the first is a typed rejection a caller can act on, the second is
/// [`FeasibilityOutcome::Unknown`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SynchronizationRealization {
    /// The target realizes exactly this subject.
    Realized,
    /// The target does not realize this subject.
    Unrealizable,
}

impl SynchronizationRealization {
    /// Returns the canonical tag naming this verdict in a descriptor encoding.
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::Realized => 0x01,
            Self::Unrealizable => 0x02,
        }
    }

    /// The stable identifier naming this verdict in an explanation.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Realized => "realized",
            Self::Unrealizable => "unrealizable",
        }
    }
}

/// One target's verdict on one complete synchronization subject, before it is
/// attributed to the profile declaring it.
///
/// **The subject is one value and is matched as one value.** That is the whole
/// content of atomicity: each of its five dimensions is separately true of some
/// realization on some machine — a device-memory fence, a subgroup-wide arrival,
/// an acquire-release ordering — so a profile declaring them independently would
/// let their conjunction be inferred from facts none of which is about it. There
/// is deliberately no accessor yielding one dimension of the subject, and
/// [`CheckedTargetProfile::resolve_synchronization`] compares the whole value
/// rather than reading a field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredSynchronizationRealization {
    subject: SynchronizationSubject,
    realization: SynchronizationRealization,
    source: Arc<FactSourceProvenance>,
}

impl DeclaredSynchronizationRealization {
    /// Declares one verdict over one complete subject.
    pub(crate) const fn new(
        subject: SynchronizationSubject,
        realization: SynchronizationRealization,
        source: Arc<FactSourceProvenance>,
    ) -> Self {
        Self {
            subject,
            realization,
            source,
        }
    }

    /// The complete subject this declaration ranges over.
    pub(crate) const fn subject(&self) -> SynchronizationSubject {
        self.subject
    }

    /// The verdict this declaration states.
    pub(crate) const fn realization(&self) -> SynchronizationRealization {
        self.realization
    }

    /// The phase from which the declaration is available.
    pub(crate) fn phase(&self) -> AvailabilityPhase {
        self.source.phase()
    }

    /// The structured source qualifying the declaration.
    pub(crate) fn source_ref(&self) -> &FactSourceProvenance {
        &self.source
    }

    /// The canonical sort and uniqueness key: the subject and its phase.
    ///
    /// Deliberately *excluding* the verdict. A profile declaring one subject both
    /// realized and unrealizable at one phase has stated a contradiction, and
    /// keying on the verdict would let both rows coexist with whichever the sort
    /// put first deciding.
    pub(crate) fn sort_key(&self) -> (SynchronizationSubject, AvailabilityPhase) {
        (self.subject(), self.phase())
    }

    /// Attributes this declaration to the profile that makes it.
    pub(crate) fn attributed_to(
        self,
        profile: impl Into<TargetProfileIdentity>,
    ) -> SynchronizationRealizationFact {
        SynchronizationRealizationFact {
            declared: self,
            provenance: FactProvenance::declared_by(profile),
        }
    }
}

/// One atomic, provenance-bearing target fact about a synchronization subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SynchronizationRealizationFact {
    declared: DeclaredSynchronizationRealization,
    provenance: FactProvenance,
}

impl SynchronizationRealizationFact {
    /// The complete subject this fact ranges over.
    pub(crate) const fn subject(&self) -> SynchronizationSubject {
        self.declared.subject
    }

    /// Whether the target realizes that subject.
    pub(crate) const fn realization(&self) -> SynchronizationRealization {
        self.declared.realization
    }

    /// The phase from which this fact is available.
    pub(crate) fn phase(&self) -> AvailabilityPhase {
        self.declared.source.phase()
    }

    /// The authority class supplying it.
    pub(crate) fn authority(&self) -> FactAuthority {
        self.declared.source.authority()
    }

    /// The scope over which it is valid.
    pub(crate) fn validity(&self) -> FactValidityScope {
        self.declared.source.validity()
    }

    /// The structured source qualifying the claim.
    pub(crate) fn source(&self) -> &Arc<FactSourceProvenance> {
        &self.declared.source
    }

    /// The profile that declared it.
    pub(crate) const fn provenance(&self) -> &FactProvenance {
        &self.provenance
    }

    /// The canonical sort and uniqueness key: the subject and its phase.
    ///
    /// Deliberately *excluding* the verdict. A profile declaring one subject both
    /// realized and unrealizable at one phase has stated a contradiction, and
    /// keying on the verdict would let both rows coexist with whichever the sort
    /// put first deciding.
    fn sort_key(&self) -> (SynchronizationSubject, AvailabilityPhase) {
        self.declared.sort_key()
    }
}

/// A typed capability fact: a bound on one axis, available from a stated phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapabilityFact {
    axis: CapabilityAxis,
    bound: u64,
    phase: AvailabilityPhase,
    authority: FactAuthority,
    validity: FactValidityScope,
    provenance: FactProvenance,
}

/// One executable later-phase query path for a quantitative capability.
///
/// Unlike [`CapabilityFact`], this carries no available value. It says how an
/// exact runtime subject can produce that value before routing commits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapabilityQuery {
    axis: CapabilityAxis,
    query: TargetPropertyQuery,
}

impl CapabilityQuery {
    /// Associates one typed capability axis with its query contract.
    pub(crate) const fn new(axis: CapabilityAxis, query: TargetPropertyQuery) -> Self {
        Self { axis, query }
    }

    /// The quantitative axis this query can answer.
    pub(crate) const fn axis(&self) -> CapabilityAxis {
        self.axis
    }

    /// The complete governed query contract.
    pub(crate) const fn query(&self) -> &TargetPropertyQuery {
        &self.query
    }
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
    pub(crate) const fn axis(&self) -> CapabilityAxis {
        self.axis
    }

    /// The phase from which this fact is available.
    pub(crate) const fn phase(&self) -> AvailabilityPhase {
        self.phase
    }

    /// The authority vouching for this fact.
    pub(crate) const fn authority(&self) -> FactAuthority {
        self.authority
    }

    /// The scope over which this fact is valid.
    pub(crate) const fn validity(&self) -> FactValidityScope {
        self.validity
    }

    /// Where this fact came from.
    pub(crate) const fn provenance(&self) -> &FactProvenance {
        &self.provenance
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
    /// Canonical: sorted by axis, unique per axis.
    queries: Vec<CapabilityQuery>,
    /// Canonical: sorted by `(dimension, arithmetic, behaviour, phase)`, unique
    /// per tuple.
    honourability: Vec<NumericalHonourabilityFact>,
    /// Canonical: sorted by `(subject, phase)`, unique per pair.
    synchronization: Vec<SynchronizationRealizationFact>,
    /// The bounded canonical descriptor, derived once after validation.
    descriptor: Box<[u8]>,
}

impl CheckedTargetProfile {
    /// Builds a checked profile, validating it as an intrinsic contract.
    ///
    /// Rejects an empty identity key, a fact whose bound is inadmissible for its
    /// axis, a fact whose provenance names another profile, a fact whose declared
    /// authority contradicts its phase, and duplicate facts for the same
    /// `(axis, phase)`. Honourability facts are validated on the same terms: a
    /// behaviour outside its dimension's space, a foreign provenance, an
    /// authority contradicting its phase, and a duplicate
    /// `(dimension, behaviour, phase)` are all malformed.
    ///
    /// The two declarations live on one profile and share one identity because
    /// they are one target's claims about itself; the *authorities* that decide
    /// them are separate, which is what ADR 0076 item 3 requires. Splitting the
    /// declaration into two profile objects would mint a second identity that
    /// has to be kept in agreement with the first.
    pub(crate) fn new(
        identity: impl Into<TargetProfileIdentity>,
        facts: Vec<CapabilityFact>,
        honourability: Vec<NumericalHonourabilityFact>,
    ) -> Result<Self, FeasibilityError> {
        Self::new_with_queries(identity, facts, Vec::new(), honourability)
    }

    /// Builds a checked profile including executable later-phase query schemas.
    pub(crate) fn new_with_queries(
        identity: impl Into<TargetProfileIdentity>,
        facts: Vec<CapabilityFact>,
        queries: Vec<CapabilityQuery>,
        honourability: Vec<NumericalHonourabilityFact>,
    ) -> Result<Self, FeasibilityError> {
        Self::new_complete(identity, facts, queries, honourability, Vec::new())
    }

    /// Builds a checked profile including its synchronization declaration.
    pub(crate) fn new_complete(
        identity: impl Into<TargetProfileIdentity>,
        facts: Vec<CapabilityFact>,
        queries: Vec<CapabilityQuery>,
        honourability: Vec<NumericalHonourabilityFact>,
        synchronization: Vec<SynchronizationRealizationFact>,
    ) -> Result<Self, FeasibilityError> {
        let identity = identity.into();
        let mut facts = facts;
        let mut queries = queries;
        let mut honourability = honourability;
        let mut synchronization = synchronization;
        for fact in &synchronization {
            if fact.provenance().profile() != &identity {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "synchronization-provenance",
                });
            }
            if !authority_matches_phase(fact.authority(), fact.phase()) {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "synchronization-authority",
                });
            }
            if !fact.source().is_valid() {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "synchronization-source",
                });
            }
            // A fence naming no memory domain publishes nothing, so no handoff
            // could consume it and a realization of one would be a permission
            // for an operation with no effect.
            if fact.subject().fenced_spaces.is_empty() {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "synchronization-subject",
                });
            }
        }
        synchronization.sort_by_key(SynchronizationRealizationFact::sort_key);
        let mut exact_duplicate = false;
        let mut contradiction = false;
        for pair in synchronization.windows(2) {
            if pair[0].sort_key() != pair[1].sort_key() {
                continue;
            }
            if pair[0].realization() == pair[1].realization() {
                exact_duplicate = true;
            } else {
                contradiction = true;
            }
        }
        if exact_duplicate {
            return Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-synchronization",
            });
        }
        if contradiction {
            return Err(FeasibilityError::MalformedProfile {
                rule: "contradictory-synchronization",
            });
        }
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
        if queries
            .iter()
            .any(|query| query.query.available_at() != AvailabilityPhase::PreparedKernelPreflight)
        {
            return Err(FeasibilityError::MalformedProfile {
                rule: "query-phase",
            });
        }
        for fact in &honourability {
            if !fact.dimension().admits(fact.behaviour()) {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "declaration-behaviour",
                });
            }
            if let HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { relaxation } =
                fact.means()
                && !relaxation.dimension().admits(relaxation.behaviour())
            {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "declaration-relaxation",
                });
            }
            if fact.provenance().profile() != &identity {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "declaration-provenance",
                });
            }
            if !authority_matches_phase(fact.authority(), fact.phase()) {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "declaration-authority",
                });
            }
            if !fact.source().is_valid() {
                return Err(FeasibilityError::MalformedProfile {
                    rule: "declaration-source",
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
        queries.sort_by_key(|query| query.axis);
        if queries.windows(2).any(|pair| pair[0].axis == pair[1].axis) {
            return Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-query",
            });
        }
        if queries
            .iter()
            .any(|query| facts.iter().any(|fact| fact.axis == query.axis))
        {
            return Err(FeasibilityError::MalformedProfile {
                rule: "fact-query-conflict",
            });
        }
        honourability.sort_by_key(NumericalHonourabilityFact::sort_key);
        if honourability
            .windows(2)
            .any(|pair| pair[0].sort_key() == pair[1].sort_key())
        {
            return Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-declaration",
            });
        }
        let descriptor = canonical_profile_descriptor(
            &identity,
            &facts,
            &queries,
            &honourability,
            &synchronization,
        );
        let descriptor_length = descriptor.len();
        if descriptor_length > MAX_TARGET_PROFILE_DESCRIPTOR_BYTES {
            return Err(FeasibilityError::DescriptorTooLong {
                key: identity.key().to_owned(),
                actual: descriptor_length,
            });
        }
        Ok(Self {
            identity,
            facts,
            queries,
            honourability,
            synchronization,
            descriptor: descriptor.into_boxed_slice(),
        })
    }

    /// The governed identity of this profile.
    pub(crate) const fn identity(&self) -> &TargetProfileIdentity {
        &self.identity
    }

    /// The checked capability facts, in canonical order.
    pub(crate) fn facts(&self) -> &[CapabilityFact] {
        &self.facts
    }

    /// The executable quantitative query schemas, in canonical axis order.
    pub(crate) fn queries(&self) -> &[CapabilityQuery] {
        &self.queries
    }

    /// The checked numerical honourability declaration, in canonical order.
    pub(crate) fn honourability(&self) -> &[NumericalHonourabilityFact] {
        &self.honourability
    }

    /// The checked synchronization-realization declaration, in canonical order.
    pub(crate) fn synchronization(&self) -> &[SynchronizationRealizationFact] {
        &self.synchronization
    }

    /// Resolves one complete synchronization subject against this profile.
    ///
    /// **The match is one equality over the whole subject**, which is what makes
    /// the fact atomic rather than composable. A profile carrying a fact for a
    /// subject differing in any one dimension resolves this subject as
    /// [`SynchronizationResolution::NoPath`], however many of its dimensions
    /// some other fact happens to state — so a caller can never assemble a
    /// permission out of rows about neighbouring realizations.
    ///
    /// A fact admissible only from a later phase also resolves as `NoPath`
    /// rather than as a deferral, and the difference from the quantitative axes
    /// is the reason: a deferred axis carries a
    /// [`TargetPropertyQuery`] that says how a runtime obtains the value before
    /// routing commits, and no query vocabulary can ask a device "do you order a
    /// workgroup-scoped acquire-release fence over threadgroup memory". Deferring
    /// without a query contract would be a promise nothing can keep, so the
    /// unresolved case is `Unknown` and fails closed.
    fn resolve_synchronization(
        &self,
        subject: SynchronizationSubject,
        available_phase: AvailabilityPhase,
    ) -> SynchronizationResolution {
        let mut resolved: Option<&SynchronizationRealizationFact> = None;
        for fact in &self.synchronization {
            if fact.subject() != subject || fact.phase() > available_phase {
                continue;
            }
            // Prefer the most refined fact already available, exactly as the
            // quantitative axes do.
            resolved = Some(match resolved {
                Some(current) if current.phase() >= fact.phase() => current,
                _ => fact,
            });
        }
        match resolved {
            None => SynchronizationResolution::NoPath,
            Some(fact) => match fact.realization() {
                SynchronizationRealization::Realized => {
                    SynchronizationResolution::Realized(RealizedSynchronization {
                        subject,
                        fact: fact.clone(),
                    })
                }
                SynchronizationRealization::Unrealizable => {
                    SynchronizationResolution::Unrealizable(UnrealizableSynchronization {
                        subject,
                        fact: fact.clone(),
                    })
                }
            },
        }
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
    /// The identity key; every capability fact's axis, bound, phase, authority,
    /// and validity scope; and every honourability fact's dimension, behaviour,
    /// means, phase, authority, and validity scope — the whole of what makes one
    /// profile admit a candidate another rejects. Both declarations are already
    /// in the canonical order the constructor enforces and are unique per key,
    /// so the encoding is a function of the profile rather than of the order it
    /// was declared in.
    ///
    /// The honourability declaration is *inside* the descriptor, not beside it,
    /// because it decides verdicts exactly as a capability bound does: two
    /// profiles sharing a key and differing only in which subnormal behaviour
    /// they honour admit different requests, and a descriptor that could not
    /// tell them apart would let one artifact claim it was assessed against the
    /// other. That is the same defect the descriptor exists to prevent for
    /// bounds.
    ///
    /// A fact's declaring-profile citation ([`FactProvenance`]) is excluded: it
    /// cites this profile's own identity, so folding it in would make the
    /// descriptor depend on a value derived from the descriptor's own subject.
    /// The structured source supplied by that declarer is included and
    /// deduplicated. It is not circular: it names the authority, validity, and
    /// evidence basis that qualify the claim, not the descriptor being minted.
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
    pub(crate) fn canonical_descriptor(&self) -> &[u8] {
        &self.descriptor
    }

    /// Resolves one axis against the facts available through `available_phase`.
    fn resolve(&self, axis: CapabilityAxis, available_phase: AvailabilityPhase) -> AxisResolution {
        let mut now: Option<CapabilityFact> = None;
        for fact in self.facts.iter().filter(|fact| fact.axis == axis) {
            if fact.phase <= available_phase {
                // Prefer the most refined fact already available.
                now = Some(match now {
                    Some(current) if current.phase >= fact.phase => current,
                    _ => fact.clone(),
                });
            }
        }
        let later = self
            .queries
            .iter()
            .find(|query| query.axis == axis && query.query.available_at() > available_phase)
            .cloned();
        match (now, later) {
            (Some(fact), _) => AxisResolution::Now(fact.bound),
            (None, Some(query)) => AxisResolution::Later(query),
            (None, None) => AxisResolution::NoPath,
        }
    }

    /// Resolves one required behaviour against the honourability declaration
    /// available through `available_phase`.
    ///
    /// `authorized` is the caller's own contract, projected per dimension. It is
    /// read for exactly one purpose: deciding whether a
    /// [`HonouringMeans::SupportedOnlyUnderDeclaredRelaxation`] declaration is
    /// satisfied by a relaxation the caller has *already stated*. Nothing here
    /// may add, widen, or substitute an authorization — the caller's contract is
    /// an input to this decision, never an output of it (ADR 0076 item 5).
    fn resolve_dimension(
        &self,
        requirement: &NumericalRequirement,
        authorized: &[NumericalRequirement],
        available_phase: AvailabilityPhase,
    ) -> DimensionResolution {
        let dimension = requirement.dimension();
        let arithmetic = requirement.arithmetic();
        let resolved_type = requirement.resolved_type();
        let required = requirement.behaviour();
        let mut now: Option<NumericalHonourabilityFact> = None;
        let mut later: Option<AvailabilityPhase> = None;
        // The arithmetic type is part of the match, not a filter applied
        // afterwards: a fact about a neighbouring type is measurably not a
        // substitute — one Apple profile flushes subnormals in `f32` and
        // preserves them in `f16` — so a declaration for another type leaves this
        // one undeclared rather than partially answered.
        for fact in self.honourability.iter().filter(|fact| {
            fact.dimension() == dimension
                && fact.arithmetic() == arithmetic
                && fact.resolved_type() == resolved_type
                && fact.behaviour() == required
        }) {
            if fact.phase() <= available_phase {
                // Prefer the most refined declaration already available.
                now = Some(match now {
                    Some(current) if current.phase() >= fact.phase() => current,
                    _ => fact.clone(),
                });
            } else {
                later = Some(match later {
                    Some(phase) if phase <= fact.phase() => phase,
                    _ => fact.phase(),
                });
            }
        }
        let Some(fact) = now else {
            return match later {
                Some(phase) => DimensionResolution::Later(DeferredDimension::new(
                    dimension,
                    arithmetic,
                    resolved_type.clone(),
                    required,
                    phase,
                )),
                None => DimensionResolution::NoPath(UndeclaredDimension::new(
                    dimension,
                    arithmetic,
                    resolved_type.clone(),
                    required,
                )),
            };
        };
        let honoured = match fact.means() {
            HonouringMeans::SupportedExactly | HonouringMeans::SupportedWithExactEmulation => true,
            // The authorization is known now and cannot arrive later, so an
            // unauthorized relaxation disproves rather than defers.
            HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { relaxation } => {
                authorized.iter().any(|stated| {
                    // The relaxation carries its subject as the serialized
                    // identity, because that is the only form that survives the
                    // codec into the delivered-realization record. Comparing the
                    // stated requirement's own canonical encoding against it is
                    // exact: `canonical_encoding` is collision free, so equal
                    // bytes are the same resolved type.
                    stated.dimension() == relaxation.dimension()
                        && stated.arithmetic() == relaxation.subject().arithmetic()
                        && stated.resolved_type().canonical_encoding().as_bytes()
                            == relaxation.subject().resolved_type_identity()
                        && stated.behaviour() == relaxation.behaviour()
                })
            }
            HonouringMeans::Unsupported => false,
        };
        if honoured {
            DimensionResolution::Honoured(HonouredDimension::new(fact))
        } else {
            // The refusing fact is retained whole rather than summarized into
            // dimension, means, and profile key. Everything a caller needs to
            // decide whether the refusal applies to its own deployment — the
            // authority, the validity scope, the compiler builds and execution
            // environments a measurement rests on — lives in that fact and
            // nowhere else, and the profile identity it already carries is the
            // one this profile validated at construction.
            let alternative =
                self.honoured_alternative(dimension, arithmetic, resolved_type, available_phase);
            DimensionResolution::Unhonoured(UnhonouredDimension::new(fact, required, alternative))
        }
    }

    /// The canonical-first behaviour on `dimension`, in `arithmetic`, this
    /// profile honours unconditionally, when it honours one at all.
    ///
    /// Reported in a rejection so a caller can see which contract this target
    /// would accept. A conditional means is excluded because whether it honours
    /// anything depends on the request, so it is not an alternative the profile
    /// offers on its own.
    ///
    /// The arithmetic type is matched rather than ignored: reporting a behaviour
    /// honoured in another dtype would tell the caller a contract is available
    /// that this dimension does not in fact offer for the type it asked about.
    fn honoured_alternative(
        &self,
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        resolved_type: &tiler_ir::semantic::ResolvedValueType,
        available_phase: AvailabilityPhase,
    ) -> Option<DimensionBehaviour> {
        self.honourability
            .iter()
            .find(|fact| {
                fact.dimension() == dimension
                    && fact.arithmetic() == arithmetic
                    && fact.resolved_type() == resolved_type
                    && fact.phase() <= available_phase
                    && fact.is_unconditionally_honoured()
            })
            .map(NumericalHonourabilityFact::behaviour)
    }

    /// Assesses one candidate proposal against this profile.
    ///
    /// `available_phase` is the phase up to which facts are known; the compiler's
    /// static assessment uses [`AvailabilityPhase::CompileProfile`]. The result is
    /// always exactly one of the four outcomes; malformed inputs cannot reach here
    /// because both the profile and the proposal are validated at construction.
    ///
    /// Capability predicates, numerical-honourability predicates, and the one
    /// synchronization-realization predicate are assessed by their own rules and
    /// then composed under one precedence, so a candidate that is both too large
    /// and numerically unhonourable has one verdict rather than two.
    ///
    /// The synchronization predicate is composed and never decomposed. Its
    /// subject resolves as one equality, so a candidate is never partly
    /// synchronization-feasible, and no path here reads a dimension of it.
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
                AxisResolution::Later(query) => {
                    let requirement = PreparedEntryTargetRequirement::new(
                        query.query,
                        requirement.required,
                        target_property_relation(axis.relation()),
                    )
                    .expect("a checked profile declares a phase- and axis-valid query");
                    deferred.push(DeferredPredicate { axis, requirement });
                }
                AxisResolution::NoPath => unknown.push(UnknownPredicate { axis, required }),
            }
        }
        let mut honoured = Vec::new();
        let mut unhonoured = Vec::new();
        let mut undeclared = Vec::new();
        let mut deferred_dimensions = Vec::new();
        for requirement in &proposal.numerical {
            match self.resolve_dimension(requirement, &proposal.numerical, available_phase) {
                DimensionResolution::Honoured(record) => honoured.push(record),
                DimensionResolution::Unhonoured(record) => unhonoured.push(record),
                DimensionResolution::Later(record) => deferred_dimensions.push(record),
                DimensionResolution::NoPath(record) => undeclared.push(record),
            }
        }
        // The synchronization requirement, resolved as one atomic subject. A
        // candidate that requires none skips this entirely, which is what keeps
        // the absence canonical: no predicate is resolved, so no evidence, no
        // rejection, and no unknown mentions synchronization at all.
        let mut realized = None;
        let mut unrealizable = None;
        let mut unknown_synchronization = None;
        if let Some(subject) = proposal.synchronization {
            match self.resolve_synchronization(subject, available_phase) {
                SynchronizationResolution::Realized(record) => realized = Some(record),
                SynchronizationResolution::Unrealizable(record) => unrealizable = Some(record),
                SynchronizationResolution::NoPath => {
                    unknown_synchronization = Some(UnknownSynchronization { subject });
                }
            }
        }
        // Precedence: rejected, then unknown, then deferred, then proven.
        if !disproved.is_empty() || !unhonoured.is_empty() || unrealizable.is_some() {
            return FeasibilityOutcome::Rejected(Rejection {
                disproved,
                unhonourable: unhonoured,
                synchronization: unrealizable,
            });
        }
        if !unknown.is_empty() || !undeclared.is_empty() || unknown_synchronization.is_some() {
            return FeasibilityOutcome::Unknown(UnknownSet {
                predicates: unknown,
                dimensions: undeclared,
                synchronization: unknown_synchronization,
            });
        }
        if !deferred.is_empty() || !deferred_dimensions.is_empty() {
            deferred.sort_by(|left, right| {
                left.phase()
                    .cmp(&right.phase())
                    .then(left.axis.cmp(&right.axis))
            });
            deferred_dimensions.sort_by(|left, right| {
                left.phase()
                    .cmp(&right.phase())
                    .then(left.dimension().cmp(&right.dimension()))
            });
            return FeasibilityOutcome::Deferred(DeferredSet {
                proven: ProvenEvidence {
                    predicates: proven,
                    honoured,
                    synchronization: realized,
                },
                predicates: deferred,
                dimensions: deferred_dimensions,
            });
        }
        FeasibilityOutcome::Proven(ProvenEvidence {
            predicates: proven,
            honoured,
            synchronization: realized,
        })
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
                FeasibilityOutcome::Proven(evidence) => {
                    set.proven.push((proposal.candidate, evidence));
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

fn canonical_profile_descriptor(
    identity: &TargetProfileIdentity,
    facts: &[CapabilityFact],
    queries: &[CapabilityQuery],
    honourability: &[NumericalHonourabilityFact],
    synchronization: &[SynchronizationRealizationFact],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, PROFILE_DESCRIPTOR_DOMAIN);
    push_slice(&mut bytes, identity.key().as_bytes());
    push_len(&mut bytes, facts.len());
    for fact in facts {
        bytes.push(fact.axis.tag());
        bytes.extend_from_slice(&fact.bound.to_be_bytes());
        bytes.push(fact.phase.tag());
        bytes.push(fact.authority.tag());
        bytes.push(fact.validity.tag());
    }
    push_len(&mut bytes, queries.len());
    for query in queries {
        bytes.push(query.axis.tag());
        push_slice(&mut bytes, &query.query.canonical_bytes());
    }
    encode_honourability_facts(&mut bytes, honourability);
    // The synchronization declaration is *inside* the descriptor, for the reason
    // the honourability declaration is: it decides verdicts exactly as a bound
    // does, so two profiles sharing a key and differing only in which
    // realization they declare admit different candidates, and a descriptor that
    // could not tell them apart would let one artifact claim it was assessed
    // against the other.
    push_len(&mut bytes, synchronization.len());
    for fact in synchronization {
        let subject = fact.subject();
        bytes.push(subject.kind.tag());
        bytes.push(subject.execution_scope.tag());
        bytes.push(subject.visibility_scope.tag());
        bytes.push(u8::from(subject.fenced_spaces.workgroup));
        bytes.push(u8::from(subject.fenced_spaces.device));
        bytes.push(subject.ordering.tag());
        bytes.push(fact.realization().tag());
        bytes.push(fact.phase().tag());
        bytes.push(fact.authority().tag());
        bytes.push(fact.validity().tag());
    }
    bytes
}

/// Whether a fact authority is consistent with the phase it is available from.
const fn authority_matches_phase(authority: FactAuthority, phase: AvailabilityPhase) -> bool {
    matches!(
        (authority, phase),
        (
            FactAuthority::GovernedProfile
                | FactAuthority::ExternalProfile
                | FactAuthority::MeasuredProfile,
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
    Later(CapabilityQuery),
    /// No admissible proof/query path exists for the axis.
    NoPath,
}

/// The three ways one synchronization subject resolves against a profile.
///
/// Three, not four: there is no `Later`. See
/// [`CheckedTargetProfile::resolve_synchronization`] for why a fact with no
/// query contract cannot be deferred.
enum SynchronizationResolution {
    /// A fact available now declares the target realizes exactly this subject.
    Realized(RealizedSynchronization),
    /// A fact available now declares the target does not realize it.
    Unrealizable(UnrealizableSynchronization),
    /// Nothing available now speaks to this exact subject.
    NoPath,
}

/// A synchronization subject a target declares it realizes, with its evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RealizedSynchronization {
    subject: SynchronizationSubject,
    fact: SynchronizationRealizationFact,
}

impl RealizedSynchronization {
    /// The complete subject the target realizes.
    pub(crate) const fn subject(&self) -> SynchronizationSubject {
        self.subject
    }

    /// The whole attributed fact that established it.
    pub(crate) const fn fact(&self) -> &SynchronizationRealizationFact {
        &self.fact
    }
}

/// A synchronization subject a target declares it cannot realize.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnrealizableSynchronization {
    subject: SynchronizationSubject,
    fact: SynchronizationRealizationFact,
}

impl UnrealizableSynchronization {
    /// The complete subject the candidate required.
    pub(crate) const fn subject(&self) -> SynchronizationSubject {
        self.subject
    }

    /// The whole refusing fact, retained rather than summarized.
    ///
    /// A caller reporting the refusal needs the provenance behind it: "this
    /// profile, on this measurement, says no" is actionable and "no" is not.
    pub(crate) const fn fact(&self) -> &SynchronizationRealizationFact {
        &self.fact
    }
}

/// A synchronization subject no available fact speaks to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnknownSynchronization {
    subject: SynchronizationSubject,
}

impl UnknownSynchronization {
    /// The complete subject the candidate required and nothing declared.
    pub(crate) const fn subject(self) -> SynchronizationSubject {
        self.subject
    }
}

/// The four ways one numerical dimension resolves against a declaration.
enum DimensionResolution {
    /// A declaration available now honours the required behaviour.
    Honoured(HonouredDimension),
    /// A declaration available now refuses the required behaviour, either
    /// outright or because a relaxation it names is unauthorized.
    Unhonoured(UnhonouredDimension),
    /// No declaration is available now, but one is admissible from a later phase.
    Later(DeferredDimension),
    /// Nothing declares the required behaviour at any phase.
    NoPath(UndeclaredDimension),
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
///
/// The numerical requirements are the caller's resolved contract projected per
/// dimension, not a preference or a ceiling to negotiate against. Exactly one
/// behaviour per dimension may be required, which is what makes the set usable
/// as the authorization a conditional honouring means is checked against.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FeasibilityProposal {
    candidate: &'static str,
    /// Canonical: sorted by axis, unique per axis.
    requirements: Vec<AxisRequirement>,
    /// Canonical: sorted by dimension, unique per dimension.
    numerical: Vec<NumericalRequirement>,
    /// The one complete synchronization realization this candidate requires.
    ///
    /// `None` is the canonical absence, and it is what makes a
    /// zero-synchronization candidate *feasible* against a profile that declares
    /// nothing about synchronization rather than unknown against it: no
    /// requirement is composed, so no predicate is resolved, so no explain row is
    /// produced and no target fact is consulted. It is deliberately not a `Vec`:
    /// a candidate requires one realization however many times it performs it,
    /// and a count would be the barrier-count capacity `v7` retired.
    synchronization: Option<SynchronizationSubject>,
}

impl FeasibilityProposal {
    /// Builds a checked proposal, validating it as an intrinsic contract.
    ///
    /// Rejects an empty candidate identity, a requirement whose amount is
    /// inadmissible for its axis, a numerical requirement whose behaviour is
    /// outside its dimension's space, and duplicate requirements for the same
    /// axis or the same dimension.
    pub(crate) fn new(
        candidate: &'static str,
        requirements: Vec<AxisRequirement>,
        numerical: Vec<NumericalRequirement>,
    ) -> Result<Self, FeasibilityError> {
        Self::new_with_synchronization(candidate, requirements, numerical, None)
    }

    /// Builds a checked proposal that also requires a synchronization
    /// realization.
    pub(crate) fn new_with_synchronization(
        candidate: &'static str,
        requirements: Vec<AxisRequirement>,
        numerical: Vec<NumericalRequirement>,
        synchronization: Option<SynchronizationSubject>,
    ) -> Result<Self, FeasibilityError> {
        let mut requirements = requirements;
        let mut numerical = numerical;
        // A requirement to fence nothing publishes nothing, so no handoff could
        // consume it; asking a target to realize one would consume an authority
        // for an operation with no effect.
        if synchronization.is_some_and(|subject| subject.fenced_spaces.is_empty()) {
            return Err(FeasibilityError::MalformedProposal {
                rule: "requirement-synchronization",
            });
        }
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
        for requirement in &numerical {
            if !requirement.dimension().admits(requirement.behaviour()) {
                return Err(FeasibilityError::MalformedProposal {
                    rule: "requirement-behaviour",
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
        // Unique per `(dimension, arithmetic type)`, not per dimension: one
        // program may require preservation on the input-subnormal dimension in
        // one dtype and flushing in another, which is a well-formed contract on
        // measured hardware and not a duplicate.
        numerical.sort_by_key(NumericalRequirement::subject);
        if numerical
            .windows(2)
            .any(|pair| pair[0].subject() == pair[1].subject())
        {
            return Err(FeasibilityError::MalformedProposal {
                rule: "duplicate-dimension",
            });
        }
        Ok(Self {
            candidate,
            requirements,
            numerical,
            synchronization,
        })
    }

    /// The stable candidate identity.
    pub(crate) const fn candidate(&self) -> &'static str {
        self.candidate
    }

    /// The numerical requirements, in canonical dimension order.
    pub(crate) fn numerical(&self) -> &[NumericalRequirement] {
        &self.numerical
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeferredPredicate {
    axis: CapabilityAxis,
    requirement: PreparedEntryTargetRequirement,
}

impl DeferredPredicate {
    /// The axis this predicate ranges over.
    pub(crate) const fn axis(&self) -> CapabilityAxis {
        self.axis
    }

    /// The required quantity.
    pub(crate) fn required(&self) -> Quantity {
        self.axis.quantity(self.requirement.required())
    }

    /// The earliest phase that can resolve the predicate.
    pub(crate) const fn phase(&self) -> AvailabilityPhase {
        self.requirement.query().available_at()
    }

    /// The complete executable target-property requirement.
    pub(crate) const fn requirement(&self) -> &PreparedEntryTargetRequirement {
        &self.requirement
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

/// The evidence a proven candidate carries.
///
/// Both halves are retained because they are different claims: the resolved
/// predicates say a bound was met, and the honoured dimensions say *by what
/// means* the numerical contract was honoured. The means cannot be recovered
/// from the predicates, and an emulated dimension is honoured by emitted
/// operations, so a consumer that kept only the verdict would lose the work.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProvenEvidence {
    predicates: Vec<ResolvedPredicate>,
    honoured: Vec<HonouredDimension>,
    /// The one realization fact that authorized this candidate's synchronization,
    /// or `None` when it requires none.
    synchronization: Option<RealizedSynchronization>,
}

impl ProvenEvidence {
    /// The resolved capability predicates, in canonical axis order.
    pub(crate) fn predicates(&self) -> &[ResolvedPredicate] {
        &self.predicates
    }

    /// The honoured numerical dimensions, in canonical dimension order.
    pub(crate) fn honoured(&self) -> &[HonouredDimension] {
        &self.honoured
    }

    /// The synchronization realization that authorized this candidate, if any.
    ///
    /// `None` for a candidate that requires none. That is an absence and not a
    /// vacuous proof: a consumer rendering evidence emits no row for it, which is
    /// what keeps a zero-synchronization program's explanation free of a
    /// manufactured zero.
    pub(crate) const fn synchronization(&self) -> Option<&RealizedSynchronization> {
        self.synchronization.as_ref()
    }

    /// Whether this evidence records no check at all.
    pub(crate) fn is_empty(&self) -> bool {
        self.predicates.is_empty() && self.honoured.is_empty() && self.synchronization.is_none()
    }
}

/// The nonempty disproved predicates that reject a candidate, canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Rejection {
    disproved: Vec<ResolvedPredicate>,
    unhonourable: Vec<UnhonouredDimension>,
    synchronization: Option<UnrealizableSynchronization>,
}

/// The canonical representative cause of one rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RejectionCause {
    /// A synchronization realization the target declares it cannot provide.
    Synchronization(UnrealizableSynchronization),
    /// A numerical dimension the target declares it cannot honour as required.
    Numerical(UnhonouredDimension),
    /// A capability bound the candidate exceeds.
    Capability(ResolvedPredicate),
}

impl Rejection {
    /// The canonical representative cause: the first unhonourable dimension when
    /// there is one, otherwise the first disproved capability predicate.
    ///
    /// Numerical rejections come first deliberately. A capability rejection says
    /// this *plan* does not fit and another plan might; an unhonourable
    /// dimension says the target cannot compute what the caller asked for, which
    /// no amount of re-planning changes, because the numerical contract is not a
    /// search dimension. Reporting the cause that re-planning cannot fix is the
    /// more useful of the two.
    ///
    /// A refused synchronization realization comes first of all, for a stronger
    /// form of the same reason: it says the target cannot *order* what this
    /// program's dataflow requires, and no re-planning within this strategy
    /// changes that — a different strategy is a different candidate with a
    /// different proposal.
    ///
    /// At least one of the three is nonempty by construction, so this never
    /// panics.
    pub(crate) fn representative(&self) -> RejectionCause {
        if let Some(cause) = &self.synchronization {
            return RejectionCause::Synchronization(cause.clone());
        }
        self.unhonourable.first().map_or_else(
            || RejectionCause::Capability(self.disproved[0]),
            |cause| RejectionCause::Numerical(cause.clone()),
        )
    }

    /// The refused synchronization realization, when one caused the rejection.
    pub(crate) const fn synchronization(&self) -> Option<&UnrealizableSynchronization> {
        self.synchronization.as_ref()
    }

    /// All disproved capability predicates, in canonical axis order.
    pub(crate) fn disproved(&self) -> &[ResolvedPredicate] {
        &self.disproved
    }

    /// All unhonourable numerical dimensions, in canonical dimension order.
    pub(crate) fn unhonourable(&self) -> &[UnhonouredDimension] {
        &self.unhonourable
    }
}

/// One nonempty canonical deferred set, grouped by phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeferredSet {
    /// Checks already proven before the remaining queries can run.
    proven: ProvenEvidence,
    /// Canonical: sorted by `(phase, axis)`.
    predicates: Vec<DeferredPredicate>,
    /// Canonical: sorted by `(phase, dimension)`.
    dimensions: Vec<DeferredDimension>,
}

impl DeferredSet {
    /// The evidence already established before the deferred checks resolve.
    pub(crate) const fn proven(&self) -> &ProvenEvidence {
        &self.proven
    }

    /// The deferred capability predicates, canonical `(phase, axis)` order.
    pub(crate) fn predicates(&self) -> &[DeferredPredicate] {
        &self.predicates
    }

    /// The deferred numerical dimensions, canonical `(phase, dimension)` order.
    pub(crate) fn dimensions(&self) -> &[DeferredDimension] {
        &self.dimensions
    }

    /// The distinct phases the deferred checks resolve at, ascending.
    pub(crate) fn phases(&self) -> Vec<AvailabilityPhase> {
        let mut phases: Vec<AvailabilityPhase> = self
            .predicates
            .iter()
            .map(DeferredPredicate::phase)
            .chain(self.dimensions.iter().map(DeferredDimension::phase))
            .collect();
        phases.sort_unstable();
        phases.dedup();
        phases
    }
}

/// The nonempty set of predicates with no admissible proof/query path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnknownSet {
    predicates: Vec<UnknownPredicate>,
    dimensions: Vec<UndeclaredDimension>,
    synchronization: Option<UnknownSynchronization>,
}

impl UnknownSet {
    /// The unknown capability predicates, in canonical axis order.
    pub(crate) fn predicates(&self) -> &[UnknownPredicate] {
        &self.predicates
    }

    /// The numerical dimensions the profile does not speak to, canonical order.
    pub(crate) fn dimensions(&self) -> &[UndeclaredDimension] {
        &self.dimensions
    }

    /// The synchronization realization no available fact speaks to, if any.
    ///
    /// This is where a profile carrying facts about *neighbouring* subjects
    /// lands: none of them equals the required subject, so none of them resolves
    /// it, and the candidate is unknown rather than composed into feasible.
    pub(crate) const fn synchronization(&self) -> Option<&UnknownSynchronization> {
        self.synchronization.as_ref()
    }
}

/// The four target-feasibility outcomes (ADR 0043).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FeasibilityOutcome {
    /// Every check resolved and is satisfied; the candidate may enter the
    /// executable frontier. Carries the resolved predicates and the honoured
    /// numerical dimensions in canonical order.
    Proven(ProvenEvidence),
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
    proven: Vec<(&'static str, ProvenEvidence)>,
    deferred: Vec<(&'static str, DeferredSet)>,
    rejected: Vec<(&'static str, Rejection)>,
    unknown: Vec<(&'static str, UnknownSet)>,
}

impl FeasibleSet {
    /// The proven (admitted) candidates and their evidence.
    pub(crate) fn proven(&self) -> &[(&'static str, ProvenEvidence)] {
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
/// Maximum byte length of one target profile's canonical descriptor.
///
/// **This crate is the declaring authority, so this crate publishes the bound
/// and refuses where a descriptor is minted.** The descriptor's bytes *are* the
/// profile's identity rather than a hash of them, so it grows with every
/// capability and honourability fact a profile declares — the axis, bound,
/// phase, authority, and validity scope of each. A profile declaring too many
/// must be refused by whoever can name the profile, not by a downstream reader
/// that can only report a length.
///
/// **Measurement** on this checkout: the standard
/// `tiler.prototype-target-neutral-baseline.v1` descriptor is 480 bytes, and it
/// does not vary with the program because it is a property of the profile.
///
/// **Why this number.** It is the largest value `tiler-artifact` will hold: that
/// crate's own `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` is the matching 64 KiB
/// admission ceiling, and a producer minting past it would publish a descriptor
/// no reader could carry.
/// Nothing checks the two against each other and nothing can — neither crate
/// depends on the other, and no library crate depends on both — so the
/// relationship is held by this comment and by review. **Raising this bound
/// requires checking the artifact ceiling in the same change.**
pub(crate) const MAX_TARGET_PROFILE_DESCRIPTOR_BYTES: usize = 64 * 1_024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FeasibilityError {
    /// A target profile was declared inconsistently.
    MalformedProfile { rule: &'static str },
    /// A profile declares more facts than its canonical descriptor may carry.
    DescriptorTooLong {
        /// The profile whose descriptor exceeded the bound.
        key: String,
        /// The length it reached.
        actual: usize,
    },
    /// A candidate proposal was declared inconsistently.
    MalformedProposal { rule: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::target::honourability::{
        CompilerBuildIdentity, CompilerBuildRole, DeclaredBehaviour, ExecutionEnvironmentIdentity,
        FactSourceProvenance, MeasurementContext, ProvenanceIdentity,
    };
    use tiler_ir::numerics::{RelaxationRequirement, ScalarArithmeticSubject};
    use tiler_ir::program::abi::{TargetPropertyKey, TargetPropertyProviderIdentity};
    use tiler_ir::schedule::{FlushedZeroSign, NumericalPermission, SubnormalMode};
    use tiler_ir::semantic::F32;

    const BASELINE_KEY: &str = "tiler.test.baseline.v1";

    const PRESERVE: DimensionBehaviour = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
    const FLUSH_SIGNED: DimensionBehaviour =
        DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        });
    const FLUSH_POSITIVE: DimensionBehaviour =
        DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        });
    const FORBIDDEN: DimensionBehaviour =
        DimensionBehaviour::Transform(NumericalPermission::Forbidden);
    const PERMITTED: DimensionBehaviour =
        DimensionBehaviour::Transform(NumericalPermission::Permitted);

    fn identity() -> &'static TargetProfileIdentity {
        static IDENTITY: std::sync::OnceLock<TargetProfileIdentity> = std::sync::OnceLock::new();
        IDENTITY.get_or_init(|| TargetProfileIdentity::new(BASELINE_KEY))
    }

    fn measured_source(authority: FactAuthority) -> Arc<FactSourceProvenance> {
        measured_source_with(authority, "1.0", "build-1")
    }

    fn measured_source_with(
        authority: FactAuthority,
        compiler_version: &str,
        platform_build: &str,
    ) -> Arc<FactSourceProvenance> {
        Arc::new(FactSourceProvenance::measured(
            AvailabilityPhase::LiveDevicePreflight,
            authority,
            FactValidityScope::DeviceInstance,
            ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
            vec![MeasurementContext::new(
                vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::RuntimeCompiler,
                    "test-compiler",
                    compiler_version,
                    Some("build-1".to_owned()),
                )],
                ExecutionEnvironmentIdentity::new(
                    "test-platform",
                    "1.0",
                    platform_build,
                    "test-architecture",
                    "test-hardware",
                ),
            )],
        ))
    }

    fn compile_fact(
        id: &TargetProfileIdentity,
        axis: CapabilityAxis,
        bound: u64,
    ) -> CapabilityFact {
        CapabilityFact::new(
            axis,
            bound,
            AvailabilityPhase::CompileProfile,
            FactAuthority::GovernedProfile,
            FactValidityScope::PortableProfile,
            FactProvenance::declared_by(id),
        )
    }

    fn capability_query(axis: CapabilityAxis, phase: AvailabilityPhase) -> CapabilityQuery {
        CapabilityQuery::new(
            axis,
            TargetPropertyQuery::new(
                TargetPropertyKey::new(format!("tiler.test.query.{}", axis.key())).unwrap(),
                phase,
                TargetPropertyProviderIdentity::new("tiler", "test-target-properties", 1).unwrap(),
            )
            .unwrap(),
        )
    }

    fn declares(
        id: &TargetProfileIdentity,
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
    ) -> NumericalHonourabilityFact {
        DeclaredBehaviour::compile_profile(
            dimension,
            ArithmeticType::F32,
            F32::resolved_type(),
            behaviour,
            means,
        )
        .attributed_to(id)
    }

    /// The baseline's honourability declaration: strict everywhere, exactly.
    fn baseline_honourability(id: &TargetProfileIdentity) -> Vec<NumericalHonourabilityFact> {
        vec![
            declares(
                id,
                NumericalDimension::InputSubnormals,
                PRESERVE,
                HonouringMeans::SupportedExactly,
            ),
            declares(
                id,
                NumericalDimension::ResultSubnormals,
                PRESERVE,
                HonouringMeans::SupportedExactly,
            ),
            declares(
                id,
                NumericalDimension::Contraction,
                FORBIDDEN,
                HonouringMeans::SupportedExactly,
            ),
            declares(
                id,
                NumericalDimension::Reassociation,
                FORBIDDEN,
                HonouringMeans::SupportedExactly,
            ),
        ]
    }

    // ---- The atomic synchronization-realization fact -----------------------
    //
    // Every fixture below is the same complete subject: a control barrier every
    // workgroup invocation arrives at, publishing workgroup-wide, fencing
    // workgroup memory alone, ordered acquire-release. The perturbation tests
    // change exactly one of its five dimensions, so a refusal names the
    // dimension the change touched rather than a difference the fixture carried.

    /// The realization the cooperative tile's staged handoff requires.
    const REQUIRED_SUBJECT: SynchronizationSubject = SynchronizationSubject {
        kind: tiler_ir::schedule::SynchronizationKind::ControlBarrier,
        execution_scope: tiler_ir::schedule::SynchronizationScope::Workgroup,
        visibility_scope: tiler_ir::schedule::SynchronizationScope::Workgroup,
        fenced_spaces: tiler_ir::schedule::FencedSpaces {
            workgroup: true,
            device: false,
        },
        ordering: tiler_ir::schedule::MemoryOrdering::AcquireRelease,
    };

    /// The five one-dimension neighbours of [`REQUIRED_SUBJECT`].
    ///
    /// Named rather than spelled inline because they serve two tests: each is a
    /// mismatch the authority must refuse, and *together* they are the
    /// composition hazard — a profile declaring all five realizes every
    /// dimension of the required subject somewhere and the required subject
    /// nowhere.
    fn neighbouring_subjects() -> [(&'static str, SynchronizationSubject); 5] {
        use tiler_ir::schedule::{MemoryOrdering, SynchronizationKind, SynchronizationScope};
        [
            (
                "operation kind",
                SynchronizationSubject {
                    kind: SynchronizationKind::Collective,
                    ..REQUIRED_SUBJECT
                },
            ),
            (
                "arrival scope",
                SynchronizationSubject {
                    execution_scope: SynchronizationScope::Subgroup,
                    ..REQUIRED_SUBJECT
                },
            ),
            (
                "publication scope",
                SynchronizationSubject {
                    visibility_scope: SynchronizationScope::Device,
                    ..REQUIRED_SUBJECT
                },
            ),
            (
                "fenced domains",
                SynchronizationSubject {
                    fenced_spaces: tiler_ir::schedule::FencedSpaces {
                        workgroup: true,
                        device: true,
                    },
                    ..REQUIRED_SUBJECT
                },
            ),
            (
                "ordering",
                SynchronizationSubject {
                    ordering: MemoryOrdering::SequentiallyConsistent,
                    ..REQUIRED_SUBJECT
                },
            ),
        ]
    }

    fn synchronization_fact(
        id: &TargetProfileIdentity,
        subject: SynchronizationSubject,
        realization: SynchronizationRealization,
    ) -> SynchronizationRealizationFact {
        DeclaredSynchronizationRealization::new(
            subject,
            realization,
            crate::target::honourability::governed_profile_source(),
        )
        .attributed_to(id)
    }

    /// The baseline plus a declaration over exactly `facts`.
    fn synchronizing_profile(facts: Vec<SynchronizationRealizationFact>) -> CheckedTargetProfile {
        let id = identity();
        CheckedTargetProfile::new_complete(
            id,
            vec![
                compile_fact(id, CapabilityAxis::GridAxisThreads, 65_535),
                compile_fact(id, CapabilityAxis::WorkgroupThreads, 8),
                compile_fact(id, CapabilityAxis::BufferBindings, 2),
                compile_fact(id, CapabilityAxis::DeviceAddressSpace, 1),
                compile_fact(id, CapabilityAxis::LocalMemoryBytes, 4_096),
                compile_fact(id, CapabilityAxis::IndexArithmeticU64, 1),
            ],
            Vec::new(),
            baseline_honourability(id),
            facts,
        )
        .unwrap()
    }

    /// A candidate requiring exactly [`REQUIRED_SUBJECT`], and nothing else new.
    fn synchronizing_proposal() -> FeasibilityProposal {
        FeasibilityProposal::new_with_synchronization(
            "tiler.test.synchronized",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 4)],
            Vec::new(),
            Some(REQUIRED_SUBJECT),
        )
        .unwrap()
    }

    /// A zero-synchronization candidate is feasible against a profile that
    /// declares nothing about synchronization.
    ///
    /// The absence is canonical, and this proves all three halves of that: the
    /// proposal composes no requirement, the profile carries no fact, and the
    /// proven evidence carries no synchronization record for a consumer to
    /// render. A vacuous "zero barriers required, zero available" predicate
    /// would have made the same program *report* a check it never performed.
    #[test]
    fn a_zero_synchronization_candidate_needs_no_synchronization_fact() {
        let profile = synchronizing_profile(Vec::new());
        assert!(profile.synchronization().is_empty());
        let proposal = FeasibilityProposal::new(
            "tiler.test.unsynchronized",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 4)],
            Vec::new(),
        )
        .unwrap();
        assert!(proposal.synchronization.is_none());
        let FeasibilityOutcome::Proven(evidence) =
            profile.assess(&proposal, AvailabilityPhase::CompileProfile)
        else {
            panic!("a candidate requiring no synchronization is feasible");
        };
        assert!(
            evidence.synchronization().is_none(),
            "a candidate that required no realization was credited with one"
        );
    }

    /// The exact matching fact, and only it, admits the synchronized candidate.
    #[test]
    fn an_exactly_matching_realization_admits_a_synchronized_candidate() {
        let profile = synchronizing_profile(vec![synchronization_fact(
            identity(),
            REQUIRED_SUBJECT,
            SynchronizationRealization::Realized,
        )]);
        let FeasibilityOutcome::Proven(evidence) =
            profile.assess(&synchronizing_proposal(), AvailabilityPhase::CompileProfile)
        else {
            panic!("the exactly matching realization admits the candidate");
        };
        let realized = evidence
            .synchronization()
            .expect("the admitted evidence names the realization it consumed");
        assert_eq!(realized.subject(), REQUIRED_SUBJECT);
        // The evidence retains the whole attributed fact, so a consumer can say
        // *which* authority permitted the operation rather than only that one
        // did.
        assert_eq!(realized.fact().provenance().profile().key(), BASELINE_KEY);
        assert_eq!(realized.fact().authority(), FactAuthority::GovernedProfile);
    }

    /// A profile that declares nothing about the subject is `Unknown`.
    #[test]
    fn a_missing_realization_is_unknown_rather_than_admitted() {
        let profile = synchronizing_profile(Vec::new());
        let FeasibilityOutcome::Unknown(unknown) =
            profile.assess(&synchronizing_proposal(), AvailabilityPhase::CompileProfile)
        else {
            panic!("a candidate whose realization nothing declares is unknown");
        };
        assert_eq!(
            unknown
                .synchronization()
                .expect("the unknown set names the unresolved subject")
                .subject(),
            REQUIRED_SUBJECT
        );
        assert!(unknown.predicates().is_empty());
        assert!(unknown.dimensions().is_empty());
    }

    /// A declared refusal is a typed rejection, not an unknown.
    #[test]
    fn a_declared_unrealizable_subject_rejects_by_name() {
        let profile = synchronizing_profile(vec![synchronization_fact(
            identity(),
            REQUIRED_SUBJECT,
            SynchronizationRealization::Unrealizable,
        )]);
        let FeasibilityOutcome::Rejected(rejection) =
            profile.assess(&synchronizing_proposal(), AvailabilityPhase::CompileProfile)
        else {
            panic!("a declared refusal rejects the candidate");
        };
        let RejectionCause::Synchronization(cause) = rejection.representative() else {
            panic!("the representative cause names the synchronization refusal");
        };
        assert_eq!(cause.subject(), REQUIRED_SUBJECT);
        assert!(rejection.disproved().is_empty());
        assert!(rejection.unhonourable().is_empty());
    }

    /// One dimension changed is one refusal: the match is over the whole value.
    #[test]
    fn a_realization_differing_in_any_one_dimension_satisfies_nothing() {
        for (dimension, neighbour) in neighbouring_subjects() {
            assert_ne!(neighbour, REQUIRED_SUBJECT, "{dimension} did not change");
            let profile = synchronizing_profile(vec![synchronization_fact(
                identity(),
                neighbour,
                SynchronizationRealization::Realized,
            )]);
            assert!(
                matches!(
                    profile.assess(&synchronizing_proposal(), AvailabilityPhase::CompileProfile),
                    FeasibilityOutcome::Unknown(_)
                ),
                "a realization differing only in {dimension} satisfied the requirement"
            );
        }
    }

    /// **The composition hazard, demonstrated refused.**
    ///
    /// Every dimension of the required subject is realized by *some* fact in
    /// this profile: it declares a collective, a subgroup arrival, a device-wide
    /// publication, a workgroup-and-device fence, and a sequentially consistent
    /// ordering, each realized. Every component of the conjunction is therefore
    /// separately true of this target, and the conjunction is declared nowhere.
    ///
    /// A per-dimension authority would admit the candidate here. This one does
    /// not, and the outcome is `Unknown` rather than a rejection because the
    /// profile has not refused anything — it has simply never been asked about
    /// the realization the program needs.
    #[test]
    fn independently_true_component_facts_compose_into_no_permission() {
        let facts: Vec<_> = neighbouring_subjects()
            .into_iter()
            .map(|(_, subject)| {
                synchronization_fact(identity(), subject, SynchronizationRealization::Realized)
            })
            .collect();
        assert_eq!(facts.len(), 5);
        let profile = synchronizing_profile(facts);
        // Each dimension of the required subject appears, realized, somewhere.
        for (dimension, neighbour) in neighbouring_subjects() {
            assert!(
                profile.synchronization().iter().any(|fact| {
                    fact.subject() == neighbour
                        && fact.realization() == SynchronizationRealization::Realized
                }),
                "the fixture does not realize the {dimension} neighbour"
            );
        }
        let FeasibilityOutcome::Unknown(unknown) =
            profile.assess(&synchronizing_proposal(), AvailabilityPhase::CompileProfile)
        else {
            panic!("five true component facts must not compose into one permission");
        };
        assert_eq!(
            unknown
                .synchronization()
                .expect("the unknown set names the unresolved subject")
                .subject(),
            REQUIRED_SUBJECT
        );
    }

    /// A fact only available later is unknown, never deferred.
    ///
    /// Deferral means "a runtime can obtain this before routing commits", and a
    /// synchronization fact carries no query contract that could. Admitting it
    /// as deferred would be a promise nothing can keep.
    #[test]
    fn a_later_phase_realization_is_unknown_rather_than_deferred() {
        let id = identity();
        let later = DeclaredSynchronizationRealization::new(
            REQUIRED_SUBJECT,
            SynchronizationRealization::Realized,
            measured_source(FactAuthority::DeviceRuntime),
        )
        .attributed_to(id);
        assert_eq!(later.phase(), AvailabilityPhase::LiveDevicePreflight);
        let profile = synchronizing_profile(vec![later]);
        assert!(matches!(
            profile.assess(&synchronizing_proposal(), AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
        // And it *is* resolvable once that phase is reached, which is what makes
        // the compile-time refusal a phase decision rather than a dead branch.
        assert!(matches!(
            profile.assess(
                &synchronizing_proposal(),
                AvailabilityPhase::LiveDevicePreflight
            ),
            FeasibilityOutcome::Proven(_)
        ));
    }

    /// The declaration is part of the profile's identity.
    #[test]
    fn a_synchronization_declaration_moves_the_profile_descriptor() {
        let bare = synchronizing_profile(Vec::new());
        let realized = synchronizing_profile(vec![synchronization_fact(
            identity(),
            REQUIRED_SUBJECT,
            SynchronizationRealization::Realized,
        )]);
        let refused = synchronizing_profile(vec![synchronization_fact(
            identity(),
            REQUIRED_SUBJECT,
            SynchronizationRealization::Unrealizable,
        )]);
        assert_ne!(bare.canonical_descriptor(), realized.canonical_descriptor());
        assert_ne!(
            realized.canonical_descriptor(),
            refused.canonical_descriptor(),
            "two profiles that answer one subject differently shared a descriptor"
        );
        // And every dimension of the subject moves it, which is what stops one
        // artifact claiming it was assessed against a neighbouring realization.
        for (dimension, neighbour) in neighbouring_subjects() {
            let other = synchronizing_profile(vec![synchronization_fact(
                identity(),
                neighbour,
                SynchronizationRealization::Realized,
            )]);
            assert_ne!(
                realized.canonical_descriptor(),
                other.canonical_descriptor(),
                "the {dimension} dimension does not reach the descriptor"
            );
        }
    }

    /// A profile cannot answer one subject twice at one phase.
    ///
    /// Exact restatement and same-key contradiction are independent refusals:
    /// neither is "sort and keep the first", and the two error rules are
    /// distinct so a later reader cannot collapse them.
    #[test]
    fn an_exact_duplicate_synchronization_declaration_is_malformed() {
        let id = identity();
        assert!(matches!(
            CheckedTargetProfile::new_complete(
                id,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![
                    synchronization_fact(
                        id,
                        REQUIRED_SUBJECT,
                        SynchronizationRealization::Realized
                    ),
                    synchronization_fact(
                        id,
                        REQUIRED_SUBJECT,
                        SynchronizationRealization::Realized
                    ),
                ],
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-synchronization"
            })
        ));
    }

    #[test]
    fn a_contradictory_synchronization_declaration_is_malformed() {
        let id = identity();
        for (first, second) in [
            (
                SynchronizationRealization::Realized,
                SynchronizationRealization::Unrealizable,
            ),
            (
                SynchronizationRealization::Unrealizable,
                SynchronizationRealization::Realized,
            ),
        ] {
            assert!(
                matches!(
                    CheckedTargetProfile::new_complete(
                        id,
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        vec![
                            synchronization_fact(id, REQUIRED_SUBJECT, first),
                            synchronization_fact(id, REQUIRED_SUBJECT, second),
                        ],
                    ),
                    Err(FeasibilityError::MalformedProfile {
                        rule: "contradictory-synchronization"
                    })
                ),
                "sort order must not choose a winner between {first:?} then {second:?}"
            );
        }
    }

    /// Insertion order is not identity: two checked populations that differ
    /// only in declaration order encode one descriptor and store one order.
    #[test]
    fn checked_synchronization_rows_canonicalize_independently_of_insertion_order() {
        let id = identity();
        let (_, neighbour) = neighbouring_subjects()[0];
        let first =
            synchronization_fact(id, REQUIRED_SUBJECT, SynchronizationRealization::Realized);
        let second = synchronization_fact(id, neighbour, SynchronizationRealization::Unrealizable);
        let forward = synchronizing_profile(vec![first.clone(), second.clone()]);
        let reverse = synchronizing_profile(vec![second, first]);
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        let expected = if REQUIRED_SUBJECT < neighbour {
            [REQUIRED_SUBJECT, neighbour]
        } else {
            [neighbour, REQUIRED_SUBJECT]
        };
        for profile in [&forward, &reverse] {
            let subjects: Vec<_> = profile
                .synchronization()
                .iter()
                .map(SynchronizationRealizationFact::subject)
                .collect();
            assert_eq!(subjects, expected);
        }
    }

    /// Distinct phases of one subject coexist; their declaration order does not
    /// move the checked descriptor.
    #[test]
    fn checked_synchronization_rows_at_distinct_phases_are_order_independent() {
        let id = identity();
        let compile =
            synchronization_fact(id, REQUIRED_SUBJECT, SynchronizationRealization::Realized);
        let later = DeclaredSynchronizationRealization::new(
            REQUIRED_SUBJECT,
            SynchronizationRealization::Unrealizable,
            measured_source(FactAuthority::DeviceRuntime),
        )
        .attributed_to(id);
        assert_ne!(compile.phase(), later.phase());
        let forward = synchronizing_profile(vec![compile.clone(), later.clone()]);
        let reverse = synchronizing_profile(vec![later, compile]);
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        for profile in [&forward, &reverse] {
            let phases: Vec<_> = profile
                .synchronization()
                .iter()
                .map(SynchronizationRealizationFact::phase)
                .collect();
            assert_eq!(
                phases,
                [
                    AvailabilityPhase::CompileProfile,
                    AvailabilityPhase::LiveDevicePreflight
                ]
            );
        }
    }

    /// A fence over no memory domain publishes nothing, in both directions.
    #[test]
    fn a_subject_that_fences_nothing_is_malformed() {
        let vacuous = SynchronizationSubject {
            fenced_spaces: tiler_ir::schedule::FencedSpaces::NONE,
            ..REQUIRED_SUBJECT
        };
        assert!(matches!(
            CheckedTargetProfile::new_complete(
                identity(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![synchronization_fact(
                    identity(),
                    vacuous,
                    SynchronizationRealization::Realized
                )],
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "synchronization-subject"
            })
        ));
        assert!(matches!(
            FeasibilityProposal::new_with_synchronization(
                "tiler.test.vacuous",
                Vec::new(),
                Vec::new(),
                Some(vacuous),
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "requirement-synchronization"
            })
        ));
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
                compile_fact(id, CapabilityAxis::DeviceAddressSpace, 1),
                compile_fact(id, CapabilityAxis::LocalMemoryBytes, 0),
                compile_fact(id, CapabilityAxis::IndexArithmeticU64, 1),
            ],
            baseline_honourability(id),
        )
        .unwrap()
    }

    /// The dimensions this module's synthetic fixture speaks about.
    ///
    /// Deliberately its own list rather than
    /// [`crate::target::honourability::CANONICAL_DIMENSIONS`]. These tests exercise the
    /// *authority* — how one declaration and one requirement compose into a
    /// verdict — over a fixture that declares four dimensions, and that is a
    /// complete test of the authority whatever the governed contract's dimension
    /// count happens to be. Asserting against the production list instead would
    /// make every added dimension fail here for a reason that has nothing to do
    /// with what these tests check, and would quietly turn them into a second
    /// pin on the vocabulary that `crate::policy`'s tests already own.
    const FIXTURE_DIMENSIONS: [NumericalDimension; 4] = [
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Contraction,
        NumericalDimension::Reassociation,
    ];

    /// The strict contract, projected per dimension.
    fn strict_requirements() -> Vec<NumericalRequirement> {
        vec![
            NumericalRequirement::new(
                NumericalDimension::InputSubnormals,
                ArithmeticType::F32,
                F32::resolved_type(),
                PRESERVE,
            ),
            NumericalRequirement::new(
                NumericalDimension::ResultSubnormals,
                ArithmeticType::F32,
                F32::resolved_type(),
                PRESERVE,
            ),
            NumericalRequirement::new(
                NumericalDimension::Contraction,
                ArithmeticType::F32,
                F32::resolved_type(),
                FORBIDDEN,
            ),
            NumericalRequirement::new(
                NumericalDimension::Reassociation,
                ArithmeticType::F32,
                F32::resolved_type(),
                FORBIDDEN,
            ),
        ]
    }

    /// A profile with no honourability declaration at all.
    fn capability_only_profile(facts: Vec<CapabilityFact>) -> CheckedTargetProfile {
        CheckedTargetProfile::new(identity(), facts, Vec::new()).unwrap()
    }

    /// The descriptor distinguishes profiles a key alone cannot tell apart.
    ///
    /// ADR 0043's whole reason for requiring a descriptor beside the key is
    /// that two profiles can advertise one key and admit different candidates.
    /// Each pair below shares a key and must not share a descriptor.
    #[test]
    fn the_canonical_profile_descriptor_separates_profiles_sharing_a_key() {
        let baseline = baseline_profile();
        let descriptor = baseline.canonical_descriptor().to_vec();
        let id = identity();

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
        let mut facts: Vec<_> = baseline.facts().to_vec();
        facts[0] = compile_fact(id, CapabilityAxis::GridAxisThreads, 1_024);
        let narrower = CheckedTargetProfile::new(id, facts, baseline_honourability(id)).unwrap();
        assert_eq!(narrower.identity().key(), baseline.identity().key());
        assert_ne!(
            narrower.canonical_descriptor(),
            descriptor,
            "a profile that admits fewer candidates must not share a descriptor",
        );

        // An available fact replaced by an exact prepared-entry query: same key
        // and axis, but the profile now defers where the baseline proved.
        let mut deferred_facts: Vec<_> = baseline.facts().to_vec();
        deferred_facts.retain(|fact| fact.axis() != CapabilityAxis::GridAxisThreads);
        let later = CheckedTargetProfile::new_with_queries(
            id,
            deferred_facts,
            vec![capability_query(
                CapabilityAxis::GridAxisThreads,
                AvailabilityPhase::PreparedKernelPreflight,
            )],
            baseline_honourability(id),
        )
        .unwrap();
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

        // The honourability declaration is *inside* the descriptor. These three
        // profiles share a key and every capability bound, and differ only in
        // what they say about one numerical dimension. Each admits a different
        // set of contracts, so none may share a descriptor with another — the
        // defect the retired boolean axis could not even express.
        let mut emulated = baseline_honourability(id);
        emulated[0] = declares(
            id,
            NumericalDimension::InputSubnormals,
            PRESERVE,
            HonouringMeans::SupportedWithExactEmulation,
        );
        let mut refusing = baseline_honourability(id);
        refusing[0] = declares(
            id,
            NumericalDimension::InputSubnormals,
            PRESERVE,
            HonouringMeans::Unsupported,
        );
        let mut descriptors = vec![descriptor.clone()];
        for declaration in [emulated, refusing] {
            let variant =
                CheckedTargetProfile::new(id, baseline.facts().to_vec(), declaration).unwrap();
            assert_eq!(variant.identity().key(), baseline.identity().key());
            assert!(
                !descriptors
                    .iter()
                    .any(|descriptor| descriptor == variant.canonical_descriptor()),
                "profiles declaring different honouring means must not share a descriptor",
            );
            descriptors.push(variant.canonical_descriptor().to_vec());
        }

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
                AxisRequirement::new(CapabilityAxis::DeviceAddressSpace, 1),
                AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, 0),
                AxisRequirement::new(CapabilityAxis::IndexArithmeticU64, 1),
            ],
            strict_requirements(),
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
        let FeasibilityOutcome::Proven(evidence) = outcome else {
            panic!("baseline candidate must prove feasible");
        };
        assert_eq!(
            evidence
                .predicates()
                .iter()
                .copied()
                .map(ResolvedPredicate::axis)
                .collect::<Vec<_>>(),
            CANONICAL_AXES
                .into_iter()
                .filter(|axis| *axis != CapabilityAxis::DeviceAddressWidthBits)
                .collect::<Vec<_>>()
        );
        let grid = evidence.predicates()[0];
        assert_eq!(grid.required(), Quantity::Threads(6));
        assert_eq!(grid.available(), Quantity::Threads(65_535));
        // Composition case 1: every dimension honoured exactly is a satisfied
        // hard predicate, and the *means* survives into the evidence rather than
        // collapsing into the verdict.
        assert_eq!(
            evidence
                .honoured()
                .iter()
                .map(|honoured| (honoured.dimension(), honoured.means()))
                .collect::<Vec<_>>(),
            FIXTURE_DIMENSIONS
                .iter()
                .map(|dimension| (*dimension, HonouringMeans::SupportedExactly))
                .collect::<Vec<_>>(),
        );
        for honoured in evidence.honoured() {
            assert_eq!(honoured.profile(), identity());
        }
    }

    /// Composition case 1b: emulation is proven, and says so.
    ///
    /// This is the outcome a boolean capability axis cannot carry. An emulated
    /// dimension is *satisfied* — the candidate is proven — but it is honoured by
    /// emitting different operations, so the verdict alone would discard the
    /// work. The assertion is that the evidence distinguishes it from native
    /// support while both prove.
    #[test]
    fn an_emulated_dimension_proves_and_retains_its_means() {
        let id = identity();
        let mut declaration = baseline_honourability(id);
        declaration[1] = declares(
            id,
            NumericalDimension::ResultSubnormals,
            PRESERVE,
            HonouringMeans::SupportedWithExactEmulation,
        );
        let profile =
            CheckedTargetProfile::new(id, baseline_profile().facts().to_vec(), declaration)
                .unwrap();
        let outcome = profile.assess(
            &baseline_proposal("candidate:emulated", 6),
            AvailabilityPhase::CompileProfile,
        );
        let FeasibilityOutcome::Proven(evidence) = outcome else {
            panic!("an emulated dimension is honoured, so the candidate proves");
        };
        let result = evidence
            .honoured()
            .iter()
            .find(|honoured| honoured.dimension() == NumericalDimension::ResultSubnormals)
            .expect("the result-subnormal dimension is honoured");
        assert_eq!(result.means(), HonouringMeans::SupportedWithExactEmulation);
        let input = evidence
            .honoured()
            .iter()
            .find(|honoured| honoured.dimension() == NumericalDimension::InputSubnormals)
            .expect("the input-subnormal dimension is honoured");
        assert_eq!(input.means(), HonouringMeans::SupportedExactly);
        assert_ne!(input.means(), result.means());
    }

    /// Composition case 2: a declared refusal is a disproved hard predicate, and
    /// the rejection names the five things a boolean axis could not.
    #[test]
    fn a_declared_unhonourable_dimension_rejects_with_the_full_shape() {
        let id = identity();
        let mut declaration = baseline_honourability(id);
        declaration[0] = declares(
            id,
            NumericalDimension::InputSubnormals,
            PRESERVE,
            HonouringMeans::Unsupported,
        );
        // The target does honour sign-preserving flushing, so the rejection can
        // report a behaviour this target would accept without ever substituting
        // it for the one the caller stated.
        declaration.push(declares(
            id,
            NumericalDimension::InputSubnormals,
            FLUSH_SIGNED,
            HonouringMeans::SupportedExactly,
        ));
        let profile =
            CheckedTargetProfile::new(id, baseline_profile().facts().to_vec(), declaration)
                .unwrap();
        let outcome = profile.assess(
            &baseline_proposal("candidate:preserving", 6),
            AvailabilityPhase::CompileProfile,
        );
        let FeasibilityOutcome::Rejected(rejection) = outcome else {
            panic!("a declared-unhonourable dimension disproves a hard predicate");
        };
        let RejectionCause::Numerical(cause) = rejection.representative() else {
            panic!("the representative cause is the numerical one");
        };
        assert_eq!(cause.dimension(), NumericalDimension::InputSubnormals);
        assert_eq!(cause.required(), PRESERVE);
        assert_eq!(cause.means(), HonouringMeans::Unsupported);
        assert_eq!(cause.honoured(), Some(FLUSH_SIGNED));
        assert_eq!(cause.profile(), identity());
        assert!(rejection.disproved().is_empty());
        assert_eq!(rejection.unhonourable(), [cause]);
    }

    /// Composition case 3: a relaxation the caller did not authorize is
    /// *disproved*, never deferred and never unknown.
    ///
    /// The caller's authorization is known at the compile profile and cannot
    /// arrive later, so deferring it would promise a resolution that no later
    /// phase can supply. The same declaration proves once the caller states the
    /// relaxation, which is what makes this a check of the caller's contract
    /// rather than a permission the authority granted itself.
    #[test]
    fn an_unauthorized_relaxation_is_disproved_and_authorizing_it_proves() {
        let id = identity();
        let mut declaration = baseline_honourability(id);
        declaration[0] = declares(
            id,
            NumericalDimension::InputSubnormals,
            PRESERVE,
            HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
                relaxation: RelaxationRequirement::new(
                    ScalarArithmeticSubject::f32().identity(),
                    NumericalDimension::Reassociation,
                    PERMITTED,
                ),
            },
        );
        declaration.push(declares(
            id,
            NumericalDimension::Reassociation,
            PERMITTED,
            HonouringMeans::SupportedExactly,
        ));
        let profile =
            CheckedTargetProfile::new(id, baseline_profile().facts().to_vec(), declaration)
                .unwrap();

        // Reassociation forbidden: the relaxation is unauthorized.
        let strict = baseline_proposal("candidate:strict", 6);
        let FeasibilityOutcome::Rejected(rejection) =
            profile.assess(&strict, AvailabilityPhase::CompileProfile)
        else {
            panic!("an unauthorized relaxation disproves rather than defers");
        };
        let RejectionCause::Numerical(cause) = rejection.representative() else {
            panic!("the cause is numerical");
        };
        assert_eq!(cause.dimension(), NumericalDimension::InputSubnormals);
        assert_eq!(
            cause.means(),
            HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
                relaxation: RelaxationRequirement::new(
                    ScalarArithmeticSubject::f32().identity(),
                    NumericalDimension::Reassociation,
                    PERMITTED,
                ),
            }
        );

        // The same declaration, with the caller stating the relaxation.
        let mut numerical = strict_requirements();
        numerical[3] = NumericalRequirement::new(
            NumericalDimension::Reassociation,
            ArithmeticType::F32,
            F32::resolved_type(),
            PERMITTED,
        );
        let authorized = FeasibilityProposal::new(
            "candidate:authorized",
            vec![
                AxisRequirement::new(CapabilityAxis::GridAxisThreads, 6),
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1),
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
                AxisRequirement::new(CapabilityAxis::DeviceAddressSpace, 1),
                AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, 0),
                AxisRequirement::new(CapabilityAxis::IndexArithmeticU64, 1),
            ],
            numerical,
        )
        .unwrap();
        assert!(matches!(
            profile.assess(&authorized, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Proven(_),
        ));
    }

    /// Composition case 4, and the one most likely to be an accidental pass: a
    /// dimension the profile does not speak to is `Unknown`, not honoured.
    ///
    /// Three shapes of silence are checked, because they fail closed for the
    /// same reason and an implementation could easily get one right and another
    /// wrong: a profile with no declaration at all, a profile that declares
    /// three of the four dimensions, and a profile that declares the dimension
    /// but not the behaviour required. None may prove, and none may reject —
    /// `Unknown` is a third class, and reporting it as a rejection would assert
    /// knowledge the profile never supplied.
    #[test]
    fn an_unenumerated_dimension_is_unknown_and_never_honoured_by_default() {
        let id = identity();
        let facts = baseline_profile().facts().to_vec();

        // (a) Nothing declared at all.
        let silent = capability_only_profile(facts.clone());
        let outcome = silent.assess(
            &baseline_proposal("candidate:silent", 6),
            AvailabilityPhase::CompileProfile,
        );
        let FeasibilityOutcome::Unknown(unknown) = outcome else {
            panic!("a profile that declares no honourability cannot prove a contract");
        };
        assert!(unknown.predicates().is_empty());
        assert_eq!(
            unknown
                .dimensions()
                .iter()
                .map(UndeclaredDimension::dimension)
                .collect::<Vec<_>>(),
            FIXTURE_DIMENSIONS.to_vec(),
        );

        // (b) Three of four dimensions declared: the fourth is unknown alone.
        let mut partial = baseline_honourability(id);
        partial.remove(2);
        let partial = CheckedTargetProfile::new(id, facts.clone(), partial).unwrap();
        let FeasibilityOutcome::Unknown(unknown) = partial.assess(
            &baseline_proposal("candidate:partial", 6),
            AvailabilityPhase::CompileProfile,
        ) else {
            panic!("an undeclared dimension outranks every proven one");
        };
        assert_eq!(
            unknown
                .dimensions()
                .iter()
                .map(UndeclaredDimension::dimension)
                .collect::<Vec<_>>(),
            vec![NumericalDimension::Contraction],
        );

        // (c) The dimension is declared, but not for the behaviour required.
        // Silence about a behaviour is silence, not a refusal: the profile has
        // said nothing about preservation, so nothing may be inferred from its
        // having spoken about flushing.
        let mut behaviour_gap = baseline_honourability(id);
        behaviour_gap[0] = declares(
            id,
            NumericalDimension::InputSubnormals,
            FLUSH_SIGNED,
            HonouringMeans::SupportedExactly,
        );
        let behaviour_gap = CheckedTargetProfile::new(id, facts, behaviour_gap).unwrap();
        let FeasibilityOutcome::Unknown(unknown) = behaviour_gap.assess(
            &baseline_proposal("candidate:behaviour-gap", 6),
            AvailabilityPhase::CompileProfile,
        ) else {
            panic!("an undeclared behaviour is unknown, not honoured and not refused");
        };
        assert_eq!(
            unknown
                .dimensions()
                .iter()
                .map(|dimension| (dimension.dimension(), dimension.required()))
                .collect::<Vec<_>>(),
            vec![(NumericalDimension::InputSubnormals, PRESERVE)],
        );
    }

    /// A honourability declaration available only from a later phase defers.
    ///
    /// The governed compile-profile declaration never reaches this, but the
    /// phase machinery is the same one the capability axes use, and a profile
    /// whose device runtime supplies the declaration must not be treated as
    /// silent. Deferred and `Unknown` are different claims.
    #[test]
    fn a_later_phase_honourability_declaration_defers_then_resolves() {
        let id = identity();
        let mut declaration = baseline_honourability(id);
        let source = measured_source(FactAuthority::DeviceRuntime);
        declaration[0] = DeclaredBehaviour::new(
            NumericalDimension::InputSubnormals,
            ArithmeticType::F32,
            F32::resolved_type(),
            PRESERVE,
            HonouringMeans::SupportedExactly,
            Arc::clone(&source),
        )
        .attributed_to(id);
        let profile =
            CheckedTargetProfile::new(id, baseline_profile().facts().to_vec(), declaration)
                .unwrap();
        let proposal = baseline_proposal("candidate:late-declaration", 6);
        let FeasibilityOutcome::Deferred(deferred) =
            profile.assess(&proposal, AvailabilityPhase::CompileProfile)
        else {
            panic!("a later-phase declaration defers");
        };
        assert!(deferred.predicates().is_empty());
        assert_eq!(
            deferred
                .dimensions()
                .iter()
                .map(|dimension| (dimension.dimension(), dimension.phase()))
                .collect::<Vec<_>>(),
            vec![(
                NumericalDimension::InputSubnormals,
                AvailabilityPhase::LiveDevicePreflight
            )],
        );
        assert_eq!(
            deferred.phases(),
            vec![AvailabilityPhase::LiveDevicePreflight]
        );
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::LiveDevicePreflight),
            FeasibilityOutcome::Proven(ref evidence)
                if evidence.honoured().iter().any(|honoured| {
                    honoured.dimension() == NumericalDimension::InputSubnormals
                        && honoured.fact().source() == source.as_ref()
                }),
        ));
    }

    /// A rejection reports the numerical cause first, because it is the one
    /// re-planning cannot fix.
    #[test]
    fn a_numerical_cause_represents_a_rejection_that_is_also_capability_infeasible() {
        let id = identity();
        let mut declaration = baseline_honourability(id);
        declaration[3] = declares(
            id,
            NumericalDimension::Reassociation,
            FORBIDDEN,
            HonouringMeans::Unsupported,
        );
        let profile =
            CheckedTargetProfile::new(id, baseline_profile().facts().to_vec(), declaration)
                .unwrap();
        let FeasibilityOutcome::Rejected(rejection) = profile.assess(
            &baseline_proposal("candidate:both", 10_000_000),
            AvailabilityPhase::CompileProfile,
        ) else {
            panic!("both predicates are disproved, so the candidate rejects");
        };
        assert_eq!(rejection.disproved().len(), 1);
        assert_eq!(rejection.unhonourable().len(), 1);
        let RejectionCause::Numerical(cause) = rejection.representative() else {
            panic!("the numerical cause represents the rejection");
        };
        assert_eq!(cause.dimension(), NumericalDimension::Reassociation);
    }

    #[test]
    fn empty_proposal_is_vacuously_proven() {
        let outcome = baseline_profile().assess(
            &FeasibilityProposal::new("candidate:empty", Vec::new(), Vec::new()).unwrap(),
            AvailabilityPhase::CompileProfile,
        );
        let FeasibilityOutcome::Proven(evidence) = outcome else {
            panic!("a proposal with no requirements is vacuously proven");
        };
        assert!(evidence.is_empty());
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
        let RejectionCause::Capability(predicate) = rejection.representative() else {
            panic!("no numerical dimension is unhonourable here");
        };
        assert_eq!(predicate.axis(), CapabilityAxis::GridAxisThreads);
        assert_eq!(predicate.required(), Quantity::Threads(140_000));
        assert_eq!(predicate.available(), Quantity::Threads(65_535));
    }

    #[test]
    fn rejection_takes_precedence_over_unknown_and_deferred() {
        // One axis is disproved, one is unknown (no fact declared), one is
        // deferred (declared only at a later phase). Rejection must win.
        let id = identity();
        let profile = CheckedTargetProfile::new_with_queries(
            id,
            vec![compile_fact(id, CapabilityAxis::GridAxisThreads, 4)],
            vec![capability_query(
                CapabilityAxis::BufferBindings,
                AvailabilityPhase::PreparedKernelPreflight,
            )],
            Vec::new(),
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:mixed",
            vec![
                AxisRequirement::new(CapabilityAxis::GridAxisThreads, 9),
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1),
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
            ],
            Vec::new(),
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
        let profile = CheckedTargetProfile::new_with_queries(
            id,
            Vec::new(),
            vec![capability_query(
                CapabilityAxis::BufferBindings,
                AvailabilityPhase::PreparedKernelPreflight,
            )],
            Vec::new(),
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:unknown-and-deferred",
            vec![
                // No fact or query for WorkgroupThreads at all -> unknown.
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1),
                // BufferBindings has an executable later query -> deferred.
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
            ],
            Vec::new(),
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
    fn unresolved_checks_form_one_canonical_prepared_deferred_set() {
        let id = identity();
        let profile = CheckedTargetProfile::new_with_queries(
            id,
            Vec::new(),
            vec![
                capability_query(
                    CapabilityAxis::WorkgroupThreads,
                    AvailabilityPhase::PreparedKernelPreflight,
                ),
                capability_query(
                    CapabilityAxis::BufferBindings,
                    AvailabilityPhase::PreparedKernelPreflight,
                ),
            ],
            Vec::new(),
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:deferred",
            vec![
                AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 64),
                AxisRequirement::new(CapabilityAxis::BufferBindings, 2),
            ],
            Vec::new(),
        )
        .unwrap();
        let outcome = profile.assess(&proposal, AvailabilityPhase::CompileProfile);
        let FeasibilityOutcome::Deferred(deferred) = outcome else {
            panic!("executable later-phase queries must defer");
        };
        // Canonical axis order, independent of requirement authoring order.
        assert_eq!(
            deferred
                .predicates()
                .iter()
                .map(DeferredPredicate::axis)
                .collect::<Vec<_>>(),
            vec![
                CapabilityAxis::WorkgroupThreads,
                CapabilityAxis::BufferBindings,
            ]
        );
        assert_eq!(
            deferred.phases(),
            vec![AvailabilityPhase::PreparedKernelPreflight]
        );
    }

    #[test]
    fn a_non_prepared_entry_query_is_rejected_by_the_checked_profile() {
        let id = identity();
        assert_eq!(
            CheckedTargetProfile::new_with_queries(
                id,
                Vec::new(),
                vec![capability_query(
                    CapabilityAxis::WorkgroupThreads,
                    AvailabilityPhase::LiveDevicePreflight,
                )],
                Vec::new(),
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "query-phase",
            })
        );
    }

    #[test]
    fn a_later_observation_is_unknown_until_its_value_is_available() {
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
            Vec::new(),
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:resolves-later",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 64)],
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
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
            Vec::new(),
        )
        .unwrap();
        let proposal = FeasibilityProposal::new(
            "candidate:unprovable",
            vec![AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, 1)],
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            profile.assess(&proposal, AvailabilityPhase::LaunchPreflight),
            FeasibilityOutcome::Unknown(_)
        ));
    }

    #[test]
    fn arithmetic_support_and_address_width_are_independent_predicates() {
        let id = identity();
        let current = baseline_profile();
        let proposal = baseline_proposal("candidate:current-index-operations", 6);
        let FeasibilityOutcome::Proven(evidence) =
            current.assess(&proposal, AvailabilityPhase::CompileProfile)
        else {
            panic!("the current KIR requires arithmetic support but no address width");
        };
        assert!(
            evidence
                .predicates()
                .iter()
                .any(
                    |predicate| predicate.axis() == CapabilityAxis::IndexArithmeticU64
                        && predicate.required() == Quantity::Count(1)
                        && predicate.available() == Quantity::Count(1)
                )
        );
        assert!(
            evidence
                .predicates()
                .iter()
                .all(|predicate| predicate.axis() != CapabilityAxis::DeviceAddressWidthBits)
        );

        let address_requirement = FeasibilityProposal::new(
            "candidate:requires-64-bit-addresses",
            vec![AxisRequirement::new(
                CapabilityAxis::DeviceAddressWidthBits,
                64,
            )],
            Vec::new(),
        )
        .unwrap();
        let FeasibilityOutcome::Unknown(unknown) =
            current.assess(&address_requirement, AvailabilityPhase::CompileProfile)
        else {
            panic!("an absent address-width authority is unknown");
        };
        assert!(matches!(
            unknown.predicates(),
            [predicate]
                if predicate.axis() == CapabilityAxis::DeviceAddressWidthBits
                    && predicate.required() == Quantity::Bits(64)
        ));

        let address_32 = CheckedTargetProfile::new(
            id,
            vec![compile_fact(id, CapabilityAxis::DeviceAddressWidthBits, 32)],
            Vec::new(),
        )
        .unwrap();
        let FeasibilityOutcome::Rejected(rejection) =
            address_32.assess(&address_requirement, AvailabilityPhase::CompileProfile)
        else {
            panic!("an explicit 32-bit address model rejects a 64-bit requirement");
        };
        assert!(matches!(
            rejection.disproved(),
            [predicate]
                if predicate.axis() == CapabilityAxis::DeviceAddressWidthBits
                    && predicate.required() == Quantity::Bits(64)
                    && predicate.available() == Quantity::Bits(32)
        ));

        let arithmetic_requirement = FeasibilityProposal::new(
            "candidate:requires-u64-index-arithmetic",
            vec![AxisRequirement::new(CapabilityAxis::IndexArithmeticU64, 1)],
            Vec::new(),
        )
        .unwrap();
        let missing_arithmetic = CheckedTargetProfile::new(id, Vec::new(), Vec::new()).unwrap();
        let FeasibilityOutcome::Unknown(unknown) =
            missing_arithmetic.assess(&arithmetic_requirement, AvailabilityPhase::CompileProfile)
        else {
            panic!("missing u64 index-arithmetic authority is unknown");
        };
        assert!(matches!(
            unknown.predicates(),
            [predicate]
                if predicate.axis() == CapabilityAxis::IndexArithmeticU64
                    && predicate.required() == Quantity::Count(1)
        ));

        let no_arithmetic = CheckedTargetProfile::new(
            id,
            vec![compile_fact(id, CapabilityAxis::IndexArithmeticU64, 0)],
            Vec::new(),
        )
        .unwrap();
        let FeasibilityOutcome::Rejected(rejection) =
            no_arithmetic.assess(&arithmetic_requirement, AvailabilityPhase::CompileProfile)
        else {
            panic!("explicitly unavailable u64 index arithmetic rejects the KIR family");
        };
        assert!(matches!(
            rejection.disproved(),
            [predicate]
                if predicate.axis() == CapabilityAxis::IndexArithmeticU64
                    && predicate.required() == Quantity::Count(1)
                    && predicate.available() == Quantity::Count(0)
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
        // Identity spelling is now rejected by `TargetProfileKey` before an
        // attributed checked fact can exist; this boundary still rejects the
        // malformed capability and attribution combinations it owns.
        // A boolean-capability axis with a non-boolean bound is malformed.
        assert_eq!(
            CheckedTargetProfile::new(
                id,
                vec![compile_fact(id, CapabilityAxis::DeviceAddressSpace, 2)],
                Vec::new(),
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
                Vec::new(),
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
                Vec::new(),
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "fact-authority"
            })
        );
        // A fact whose provenance names a different profile is malformed.
        let other = TargetProfileIdentity::new("tiler.test.other.v1");
        assert_eq!(
            CheckedTargetProfile::new(
                id,
                vec![compile_fact(&other, CapabilityAxis::LocalMemoryBytes, 0,)],
                Vec::new()
            ),
            Err(FeasibilityError::MalformedProfile {
                rule: "fact-provenance"
            })
        );
    }

    /// A malformed honourability declaration is an intrinsic error too.
    ///
    /// Each rule below is a way a declaration could claim something the
    /// vocabulary cannot mean. None is a verdict about a candidate: a profile
    /// that pairs a subnormal behaviour with a transform dimension has not
    /// declared a target unable to honour anything, it has failed to declare.
    #[test]
    fn malformed_honourability_declarations_are_intrinsic_errors() {
        let id = identity();
        let other = TargetProfileIdentity::new("tiler.test.other.v1");
        for (declaration, rule) in [
            (
                vec![declares(
                    id,
                    NumericalDimension::Contraction,
                    FLUSH_POSITIVE,
                    HonouringMeans::SupportedExactly,
                )],
                "declaration-behaviour",
            ),
            (
                vec![declares(
                    id,
                    NumericalDimension::InputSubnormals,
                    PRESERVE,
                    HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
                        relaxation: RelaxationRequirement::new(
                            ScalarArithmeticSubject::f32().identity(),
                            NumericalDimension::Reassociation,
                            PRESERVE,
                        ),
                    },
                )],
                "declaration-relaxation",
            ),
            (
                vec![declares(
                    &other,
                    NumericalDimension::InputSubnormals,
                    PRESERVE,
                    HonouringMeans::SupportedExactly,
                )],
                "declaration-provenance",
            ),
            (
                vec![
                    DeclaredBehaviour::new(
                        NumericalDimension::InputSubnormals,
                        ArithmeticType::F32,
                        F32::resolved_type(),
                        PRESERVE,
                        HonouringMeans::SupportedExactly,
                        measured_source(FactAuthority::GovernedProfile),
                    )
                    .attributed_to(id),
                ],
                "declaration-authority",
            ),
            (
                vec![
                    declares(
                        id,
                        NumericalDimension::InputSubnormals,
                        PRESERVE,
                        HonouringMeans::SupportedExactly,
                    ),
                    declares(
                        id,
                        NumericalDimension::InputSubnormals,
                        PRESERVE,
                        HonouringMeans::Unsupported,
                    ),
                ],
                "duplicate-declaration",
            ),
        ] {
            assert_eq!(
                CheckedTargetProfile::new(id, Vec::new(), declaration),
                Err(FeasibilityError::MalformedProfile { rule }),
            );
        }
    }

    fn declaration_from_source(
        id: &TargetProfileIdentity,
        source: Arc<FactSourceProvenance>,
    ) -> Vec<NumericalHonourabilityFact> {
        vec![
            DeclaredBehaviour::new(
                NumericalDimension::InputSubnormals,
                ArithmeticType::F32,
                F32::resolved_type(),
                PRESERVE,
                HonouringMeans::SupportedExactly,
                source,
            )
            .attributed_to(id),
        ]
    }

    #[test]
    fn malformed_structured_fact_sources_are_intrinsic_errors() {
        let id = identity();
        let context = |builds| {
            MeasurementContext::new(
                builds,
                ExecutionEnvironmentIdentity::new(
                    "test-platform",
                    "1.0",
                    "build-1",
                    "test-architecture",
                    "test-hardware",
                ),
            )
        };
        let build = || {
            CompilerBuildIdentity::new(
                CompilerBuildRole::RuntimeCompiler,
                "test-compiler",
                "1.0",
                Some("build-1".to_owned()),
            )
        };
        let measured = |authority_identity, contexts| {
            Arc::new(FactSourceProvenance::measured(
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
                authority_identity,
                contexts,
            ))
        };

        let invalid = [
            measured(
                ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 0),
                vec![context(vec![build()])],
            ),
            measured(
                ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
                Vec::new(),
            ),
            measured(
                ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
                vec![context(Vec::new())],
            ),
            measured(
                ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
                vec![context(vec![build(), build()])],
            ),
            measured(
                ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
                vec![context(vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::RuntimeCompiler,
                    "Test Compiler",
                    "1.0",
                    None,
                )])],
            ),
            measured(
                ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
                vec![context(vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::RuntimeCompiler,
                    "test-compiler",
                    " 1.0",
                    None,
                )])],
            ),
            Arc::new(FactSourceProvenance::governed(
                ProvenanceIdentity::new("tiler.test.governed-authority.v1", 1),
                ProvenanceIdentity::new("tiler.test.guarantee.v1", 0),
            )),
        ];

        for source in invalid {
            assert_eq!(
                CheckedTargetProfile::new(id, Vec::new(), declaration_from_source(id, source),),
                Err(FeasibilityError::MalformedProfile {
                    rule: "declaration-source",
                }),
            );
        }
    }

    #[test]
    fn structured_fact_source_is_canonical_and_identity_relevant() {
        let id = identity();
        let baseline = baseline_profile();
        let build_one = measured_source_with(FactAuthority::DeviceRuntime, "1.0", "build-1");
        let same_build = measured_source_with(FactAuthority::DeviceRuntime, "1.0", "build-1");
        let build_two = measured_source_with(FactAuthority::DeviceRuntime, "2.0", "build-1");
        let environment_two = measured_source_with(FactAuthority::DeviceRuntime, "1.0", "build-2");

        let descriptor = |source| {
            let mut declarations = baseline_honourability(id);
            declarations[0] = declaration_from_source(id, source)
                .pop()
                .expect("one declaration");
            CheckedTargetProfile::new(id, baseline.facts().to_vec(), declarations)
                .expect("structured source is valid")
                .canonical_descriptor()
                .to_vec()
        };

        assert_eq!(
            descriptor(Arc::clone(&build_one)),
            descriptor(same_build),
            "allocation identity must not enter canonical profile identity",
        );
        assert_ne!(
            descriptor(Arc::clone(&build_one)),
            descriptor(build_two),
            "the compiler build qualifies the numerical fact",
        );
        assert_ne!(
            descriptor(build_one),
            descriptor(environment_two),
            "the execution environment qualifies the numerical fact",
        );

        let mut reversed = baseline_honourability(id);
        reversed.reverse();
        assert_eq!(
            CheckedTargetProfile::new(id, baseline.facts().to_vec(), reversed)
                .unwrap()
                .canonical_descriptor(),
            baseline.canonical_descriptor(),
            "declaration encounter order must not enter identity",
        );
    }

    #[test]
    fn maximally_wide_valid_structured_fact_source_fits_the_profile_bound() {
        let id = identity();
        let contexts = (0..16)
            .map(|index| {
                MeasurementContext::new(
                    vec![CompilerBuildIdentity::new(
                        CompilerBuildRole::RuntimeCompiler,
                        "test-compiler",
                        format!("1.{index}"),
                        Some(format!("compiler-build-{index}")),
                    )],
                    ExecutionEnvironmentIdentity::new(
                        "test-platform",
                        "1.0",
                        format!("platform-build-{index}"),
                        "test-architecture",
                        format!("test-hardware-{}", "x".repeat(128)),
                    ),
                )
            })
            .collect();
        let source = Arc::new(FactSourceProvenance::measured(
            AvailabilityPhase::LiveDevicePreflight,
            FactAuthority::DeviceRuntime,
            FactValidityScope::DeviceInstance,
            ProvenanceIdentity::new("tiler.test.measurement-authority.v1", 1),
            contexts,
        ));

        let profile =
            CheckedTargetProfile::new(id, Vec::new(), declaration_from_source(id, source)).unwrap();
        assert!(profile.canonical_descriptor().len() <= MAX_TARGET_PROFILE_DESCRIPTOR_BYTES);
    }

    #[test]
    fn malformed_proposals_are_intrinsic_errors() {
        assert_eq!(
            FeasibilityProposal::new("", Vec::new(), Vec::new()),
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
                Vec::new(),
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "duplicate-requirement"
            })
        );
        assert_eq!(
            FeasibilityProposal::new(
                "candidate:bad-bool",
                vec![AxisRequirement::new(CapabilityAxis::DeviceAddressSpace, 5)],
                Vec::new(),
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "requirement-amount"
            })
        );
        assert_eq!(
            FeasibilityProposal::new(
                "candidate:bad-behaviour",
                Vec::new(),
                vec![NumericalRequirement::new(
                    NumericalDimension::Contraction,
                    ArithmeticType::F32,
                    F32::resolved_type(),
                    PRESERVE
                )],
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "requirement-behaviour"
            })
        );
        // One behaviour per dimension: a proposal stating two would make the
        // authorization set a contract can be checked against ambiguous.
        assert_eq!(
            FeasibilityProposal::new(
                "candidate:two-behaviours",
                Vec::new(),
                vec![
                    NumericalRequirement::new(
                        NumericalDimension::Contraction,
                        ArithmeticType::F32,
                        F32::resolved_type(),
                        FORBIDDEN,
                    ),
                    NumericalRequirement::new(
                        NumericalDimension::Contraction,
                        ArithmeticType::F32,
                        F32::resolved_type(),
                        PERMITTED,
                    ),
                ],
            ),
            Err(FeasibilityError::MalformedProposal {
                rule: "duplicate-dimension"
            })
        );
    }

    #[test]
    fn checked_profile_exposes_canonical_facts_and_its_governed_identity() {
        let profile = baseline_profile();
        assert_eq!(profile.identity(), identity());
        assert_eq!(profile.identity().key(), BASELINE_KEY);
        // Facts are sorted into canonical axis order regardless of input order.
        let axes: Vec<_> = profile.facts().iter().map(CapabilityFact::axis).collect();
        assert_eq!(
            axes,
            CANONICAL_AXES
                .into_iter()
                .filter(|axis| *axis != CapabilityAxis::DeviceAddressWidthBits)
                .collect::<Vec<_>>()
        );
        let dimensions: Vec<_> = profile
            .honourability()
            .iter()
            .map(NumericalHonourabilityFact::dimension)
            .collect();
        assert_eq!(dimensions, FIXTURE_DIMENSIONS.to_vec());
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
        assert_eq!(
            rules.key(),
            "tiler.feasibility.phased-capability-and-numerical-honourability.v5"
        );
        assert_eq!(rules.revision(), 1);

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
    /// encodes, the dimension, behaviour, means, and phase it encodes for each
    /// declaration, and otherwise only `CapabilityAxis::relation`, which is a
    /// function of the axis rather than of the profile.
    #[test]
    fn profiles_sharing_a_descriptor_return_the_same_verdicts() {
        let id = identity();
        let baseline = baseline_profile();
        let mut reversed_declaration = baseline.honourability().to_vec();
        reversed_declaration.reverse();
        let rebuilt = CheckedTargetProfile::new(
            id,
            // Declared in a deliberately different order from `baseline_profile`.
            baseline
                .facts()
                .iter()
                .rev()
                .map(|fact| compile_fact(id, fact.axis, fact.bound))
                .collect(),
            reversed_declaration,
        )
        .unwrap();
        assert_eq!(
            rebuilt.canonical_descriptor(),
            baseline.canonical_descriptor(),
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
                    baseline.assess(&proposal, phase),
                );
            }
        }
    }
}
