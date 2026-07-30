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
//! [`crate::honourability::NumericalRequirement`]s over the per-dimension
//! numerical-honourability space (ADR 0076 item 3). They are different
//! authorities — [`crate::honourability`] owns the second vocabulary, and
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

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::schedule::ArithmeticType;

use crate::explain::Quantity;
use crate::honourability::{
    DeferredDimension, DimensionBehaviour, HonouredDimension, HonouringMeans, NumericalDimension,
    NumericalHonourabilityFact, NumericalRequirement, UndeclaredDimension, UnhonouredDimension,
    encode_honourability_facts,
};
pub(crate) use crate::target::TargetProfileIdentity;

/// Domain separator of a canonical target profile descriptor.
///
/// Trailing NUL so no descriptor can be a prefix of a differently-domained
/// encoding, matching the framing the rest of the workspace's identities use.
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
const PROFILE_DESCRIPTOR_DOMAIN: &[u8] = b"tiler.target-profile.descriptor.v7\0";

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
    "tiler.feasibility.phased-capability-and-numerical-honourability.v2";

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
/// deliberately not in it — see [`crate::honourability`] — because a bound
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
    /// Index/address width in bits.
    IndexWidthBits,
    /// Availability of an explicitly addressable device memory space.
    DeviceAddressSpace,
    /// Explicitly staged local memory, in bytes.
    LocalMemoryBytes,
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
    /// `0x06` and `0x08` are retired tags, not free ones. They named the
    /// withdrawn `StrictF32Arithmetic` and numeric barrier-count axes. Reusing
    /// either would let a descriptor mean something a reader of the retirement
    /// would not expect. New axes take the next unused value.
    const fn tag(self) -> u8 {
        match self {
            Self::GridAxisThreads => 0x01,
            Self::WorkgroupThreads => 0x02,
            Self::BufferBindings => 0x03,
            Self::IndexWidthBits => 0x04,
            Self::DeviceAddressSpace => 0x05,
            Self::LocalMemoryBytes => 0x07,
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
const CANONICAL_AXES: [CapabilityAxis; 6] = [
    CapabilityAxis::GridAxisThreads,
    CapabilityAxis::WorkgroupThreads,
    CapabilityAxis::BufferBindings,
    CapabilityAxis::IndexWidthBits,
    CapabilityAxis::DeviceAddressSpace,
    CapabilityAxis::LocalMemoryBytes,
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
            Self::LocalMemoryBytes => "local-memory-bytes",
        }
    }

    const fn relation(self) -> Relation {
        match self {
            Self::GridAxisThreads
            | Self::WorkgroupThreads
            | Self::BufferBindings
            | Self::LocalMemoryBytes => Relation::AtMost,
            Self::IndexWidthBits => Relation::Exact,
            Self::DeviceAddressSpace => Relation::Implies,
        }
    }

    /// Wraps a raw amount in this axis's governed quantity unit.
    pub(crate) const fn quantity(self, value: u64) -> Quantity {
        match self {
            Self::GridAxisThreads | Self::WorkgroupThreads => Quantity::Threads(value),
            Self::BufferBindings => Quantity::Bindings(value),
            Self::LocalMemoryBytes => Quantity::Bytes(value),
            Self::IndexWidthBits | Self::DeviceAddressSpace => Quantity::Count(value),
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FactAuthority {
    /// A governed, conservative compile-time profile guarantee.
    GovernedProfile,
    /// A named external producer's normative target-family declaration.
    ///
    /// This is available at the compile-profile phase but is not a compiler
    /// proof. Its source record carries both the producer identity and the
    /// versioned specification or guarantee it relies on.
    ExternalProfile,
    /// An empirical compiler-profile measurement tied to exact compiler builds
    /// and execution environments.
    MeasuredProfile,
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
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::GovernedProfile => 0x01,
            Self::ExternalProfile => 0x06,
            Self::MeasuredProfile => 0x07,
            Self::ArtifactEvidence => 0x02,
            Self::DeviceRuntime => 0x03,
            Self::PreparedKernel => 0x04,
            Self::LaunchInstance => 0x05,
        }
    }
}

/// The scope over which a capability fact is valid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FactValidityScope {
    /// Valid for any device matching the portable profile.
    PortableProfile,
    /// Valid only for the exact measured compiler/environment population.
    MeasuredEnvironment,
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
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::PortableProfile => 0x01,
            Self::MeasuredEnvironment => 0x05,
            Self::DeviceInstance => 0x02,
            Self::PreparedArtifact => 0x03,
            Self::LaunchInstance => 0x04,
        }
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
    /// Canonical: sorted by `(dimension, arithmetic, behaviour, phase)`, unique
    /// per tuple.
    honourability: Vec<NumericalHonourabilityFact>,
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
        let identity = identity.into();
        let mut facts = facts;
        let mut honourability = honourability;
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
        honourability.sort_by_key(NumericalHonourabilityFact::sort_key);
        if honourability
            .windows(2)
            .any(|pair| pair[0].sort_key() == pair[1].sort_key())
        {
            return Err(FeasibilityError::MalformedProfile {
                rule: "duplicate-declaration",
            });
        }
        let descriptor = canonical_profile_descriptor(&identity, &facts, &honourability);
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
            honourability,
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

    /// The checked numerical honourability declaration, in canonical order.
    pub(crate) fn honourability(&self) -> &[NumericalHonourabilityFact] {
        &self.honourability
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
        let mut later: Option<AvailabilityPhase> = None;
        for fact in self.facts.iter().filter(|fact| fact.axis == axis) {
            if fact.phase <= available_phase {
                // Prefer the most refined fact already available.
                now = Some(match now {
                    Some(current) if current.phase >= fact.phase => current,
                    _ => fact.clone(),
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
                    stated.dimension() == relaxation.dimension()
                        && stated.arithmetic() == relaxation.arithmetic()
                        && stated.resolved_type() == relaxation.resolved_type()
                        && stated.behaviour() == relaxation.behaviour()
                })
            }
            HonouringMeans::Unsupported => false,
        };
        if honoured {
            DimensionResolution::Honoured(HonouredDimension::new(fact))
        } else {
            DimensionResolution::Unhonoured(UnhonouredDimension::new(
                dimension,
                arithmetic,
                resolved_type.clone(),
                required,
                fact.means(),
                self.honoured_alternative(dimension, arithmetic, resolved_type, available_phase),
                self.identity.clone(),
            ))
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
    /// Capability predicates and numerical-honourability predicates are assessed
    /// by their own rules and then composed under one precedence, so a candidate
    /// that is both too large and numerically unhonourable has one verdict rather
    /// than two.
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
        // Precedence: rejected, then unknown, then deferred, then proven.
        if !disproved.is_empty() || !unhonoured.is_empty() {
            return FeasibilityOutcome::Rejected(Rejection {
                disproved,
                unhonourable: unhonoured,
            });
        }
        if !unknown.is_empty() || !undeclared.is_empty() {
            return FeasibilityOutcome::Unknown(UnknownSet {
                predicates: unknown,
                dimensions: undeclared,
            });
        }
        if !deferred.is_empty() || !deferred_dimensions.is_empty() {
            deferred.sort_by(|left, right| {
                left.phase
                    .cmp(&right.phase)
                    .then(left.axis.cmp(&right.axis))
            });
            deferred_dimensions.sort_by(|left, right| {
                left.phase()
                    .cmp(&right.phase())
                    .then(left.dimension().cmp(&right.dimension()))
            });
            return FeasibilityOutcome::Deferred(DeferredSet {
                predicates: deferred,
                dimensions: deferred_dimensions,
            });
        }
        FeasibilityOutcome::Proven(ProvenEvidence {
            predicates: proven,
            honoured,
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
    honourability: &[NumericalHonourabilityFact],
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
    encode_honourability_facts(&mut bytes, honourability);
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
    Later(AvailabilityPhase),
    /// No admissible proof/query path exists for the axis.
    NoPath,
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
        let mut requirements = requirements;
        let mut numerical = numerical;
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

    /// Whether this evidence records no check at all.
    pub(crate) fn is_empty(&self) -> bool {
        self.predicates.is_empty() && self.honoured.is_empty()
    }
}

/// The nonempty disproved predicates that reject a candidate, canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Rejection {
    disproved: Vec<ResolvedPredicate>,
    unhonourable: Vec<UnhonouredDimension>,
}

/// The canonical representative cause of one rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RejectionCause {
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
    /// At least one of the two sets is nonempty by construction, so this never
    /// panics.
    pub(crate) fn representative(&self) -> RejectionCause {
        self.unhonourable.first().map_or_else(
            || RejectionCause::Capability(self.disproved[0]),
            |cause| RejectionCause::Numerical(cause.clone()),
        )
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
    /// Canonical: sorted by `(phase, axis)`.
    predicates: Vec<DeferredPredicate>,
    /// Canonical: sorted by `(phase, dimension)`.
    dimensions: Vec<DeferredDimension>,
}

impl DeferredSet {
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
            .map(|predicate| predicate.phase())
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
/// crate's `MAX_OPAQUE_IDENTITY_BYTES` is a codec resource ceiling, and a
/// producer minting past it would publish a descriptor no reader could carry.
/// Nothing checks the two against each other and nothing can — neither crate
/// depends on the other, and no library crate depends on both — so the
/// relationship is held by this comment and by review. **Raising this bound
/// requires checking the artifact ceiling in the same change.**
pub(crate) const MAX_TARGET_PROFILE_DESCRIPTOR_BYTES: usize = 1_024;

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

    use crate::honourability::{
        CompilerBuildIdentity, CompilerBuildRole, DeclaredBehaviour, ExecutionEnvironmentIdentity,
        FactSourceProvenance, MeasurementContext, ProvenanceIdentity, RelaxationRequirement,
    };
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
                compile_fact(id, CapabilityAxis::LocalMemoryBytes, 0),
            ],
            baseline_honourability(id),
        )
        .unwrap()
    }

    /// The dimensions this module's synthetic fixture speaks about.
    ///
    /// Deliberately its own list rather than
    /// [`crate::honourability::CANONICAL_DIMENSIONS`]. These tests exercise the
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
        let later =
            CheckedTargetProfile::new(id, deferred_facts, baseline_honourability(id)).unwrap();
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
                AxisRequirement::new(CapabilityAxis::IndexWidthBits, 64),
                AxisRequirement::new(CapabilityAxis::DeviceAddressSpace, 1),
                AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, 0),
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
                .map(|p| p.axis())
                .collect::<Vec<_>>(),
            CANONICAL_AXES.to_vec()
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
                    NumericalDimension::Reassociation,
                    ArithmeticType::F32,
                    F32::resolved_type(),
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
                    NumericalDimension::Reassociation,
                    ArithmeticType::F32,
                    F32::resolved_type(),
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
                AxisRequirement::new(CapabilityAxis::IndexWidthBits, 64),
                AxisRequirement::new(CapabilityAxis::DeviceAddressSpace, 1),
                AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, 0),
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
            Vec::new(),
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
                            NumericalDimension::Reassociation,
                            ArithmeticType::F32,
                            F32::resolved_type(),
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
    fn oversized_structured_fact_source_is_refused_at_profile_admission() {
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
                        format!("test-hardware-{}", "x".repeat(64)),
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

        assert!(matches!(
            CheckedTargetProfile::new(id, Vec::new(), declaration_from_source(id, source)),
            Err(FeasibilityError::DescriptorTooLong { key, actual })
                if key == BASELINE_KEY && actual > MAX_TARGET_PROFILE_DESCRIPTOR_BYTES
        ));
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
        assert_eq!(axes, CANONICAL_AXES.to_vec());
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
            "tiler.feasibility.phased-capability-and-numerical-honourability.v2"
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
