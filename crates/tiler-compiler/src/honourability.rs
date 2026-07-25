#![allow(
    dead_code,
    reason = "the honourability authority itself is on the compile path through assess_region and assess_contract; what stays unconstructed is the reserved declaration surface no governed profile yet uses — the emulated, relaxation-conditional, and unsupported honouring means, the relaxation requirement that names an authorization, and the canonical dimension order — which only a target profile declaring something other than exact native support can produce, and which `declare-metal-numerical-honourability` is the first to reach"
)]

//! Per-dimension numerical honourability, a peer of the capability authority.
//!
//! ADR 0076 item 3. A target profile declares, for each dimension of the
//! resolved numerical contract it can be asked about, *which behaviour* it
//! honours and *by what means*. This module owns that vocabulary; the
//! composition of a declaration and a caller requirement into one ADR 0043
//! outcome lives beside the capability assessment in [`crate::feasibility`],
//! because a candidate has exactly one feasibility verdict and the two kinds of
//! predicate contribute to it together.
//!
//! # Why this is not a `CapabilityAxis`
//!
//! [`crate::feasibility::CapabilityAxis`] is a quantitative space: a `u64`
//! bound, a [`crate::explain::Quantity`] unit, and an `AtMost`/`Exact`/`Implies`
//! relation. Numerical honourability is not a quantity, and the decisive point
//! is that [`HonouringMeans::SupportedWithExactEmulation`] has no representation
//! as a bound comparison — emulation is honoured by *emitting different
//! operations*, so it changes the program rather than the verdict, and encoding
//! it as a satisfied `Implies` predicate would discard exactly the outcome that
//! carries work.
//!
//! # The honesty rule this vocabulary exists to enforce
//!
//! No authority may narrow, weaken, or substitute the caller's stated numerical
//! contract in order to make a target feasible (ADR 0076 item 5). Nothing here
//! computes a *nearest honourable* behaviour, and nothing ranks one behaviour
//! against another: a required behaviour is either declared honourable, declared
//! unhonourable, or not spoken to at all. The consequence is that the numerical
//! contract is not a search dimension — cost may rank implementations of one
//! contract and may never rank contracts against each other, because that would
//! price meaning.

use tiler_ir::schedule::{NumericalPermission, SubnormalMode};

use crate::feasibility::{
    AvailabilityPhase, FactAuthority, FactProvenance, FactValidityScope, TargetProfileIdentity,
};
use crate::request::{permission_tag, subnormal_tag};

/// A governed dimension of the resolved numerical contract.
///
/// The vocabulary is bounded and deliberately not open: an honourability
/// predicate ranges over these typed dimensions rather than a free-form backend
/// property bag, for the same reason ADR 0043 gives for the capability axes. The
/// derived ordering is the canonical evaluation and reporting order.
///
/// This enumerates the dimensions [`tiler_ir::schedule::NumericalRealization`]
/// carries as *declared behaviours*. The realization's `profile_key` and
/// canonical NaN bits are deliberately absent: the first names the governing
/// contract and the second is a produced value, and neither is a behaviour a
/// target declares honourability for.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum NumericalDimension {
    /// Treatment of subnormal operands before each arithmetic operation.
    InputSubnormals,
    /// Treatment of a newly produced subnormal arithmetic result.
    ResultSubnormals,
    /// Whether fused-multiply-add contraction is permitted.
    Contraction,
    /// Whether reduction reassociation is permitted.
    Reassociation,
}

/// The canonical dimension order. Single source of truth for evaluation and
/// reporting order, matching the derived [`NumericalDimension`] ordering.
pub(crate) const CANONICAL_DIMENSIONS: [NumericalDimension; 4] = [
    NumericalDimension::InputSubnormals,
    NumericalDimension::ResultSubnormals,
    NumericalDimension::Contraction,
    NumericalDimension::Reassociation,
];

impl NumericalDimension {
    /// The governed canonical predicate key for this dimension.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::InputSubnormals => "numerics.input-subnormals",
            Self::ResultSubnormals => "numerics.result-subnormals",
            Self::Contraction => "numerics.contraction",
            Self::Reassociation => "numerics.reassociation",
        }
    }

    /// Returns the governed tag naming this dimension in a canonical descriptor.
    ///
    /// Written by an exhaustive match rather than read from the discriminant, so
    /// adding or reordering a dimension is a build error here instead of a
    /// silent change to every target profile descriptor ever produced (ADR 0074
    /// convention 3).
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::InputSubnormals => 0x01,
            Self::ResultSubnormals => 0x02,
            Self::Contraction => 0x03,
            Self::Reassociation => 0x04,
        }
    }

    /// Whether `behaviour` is a value this dimension can take.
    ///
    /// A dimension's behaviour space is fixed by the dimension: the two
    /// subnormal dimensions range over [`SubnormalMode`] and the two transform
    /// dimensions over [`NumericalPermission`]. A declaration or a requirement
    /// pairing a dimension with the other space is malformed, never a verdict.
    pub(crate) const fn admits(self, behaviour: DimensionBehaviour) -> bool {
        matches!(
            (self, behaviour),
            (
                Self::InputSubnormals | Self::ResultSubnormals,
                DimensionBehaviour::Subnormals(_)
            ) | (
                Self::Contraction | Self::Reassociation,
                DimensionBehaviour::Transform(_)
            )
        )
    }
}

/// One behaviour a numerical dimension can take.
///
/// The two arms are the two behaviour spaces the governed dimensions range
/// over. They are kept as one type so a requirement, a declaration, and a
/// rejection can all name "the behaviour on this dimension" without the caller
/// switching on the dimension first; [`NumericalDimension::admits`] is what
/// keeps a subnormal behaviour off a transform dimension.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DimensionBehaviour {
    /// A resolution of one subnormal dimension.
    Subnormals(SubnormalMode),
    /// A resolution of one transform-permission dimension.
    Transform(NumericalPermission),
}

impl DimensionBehaviour {
    /// The governed canonical key naming this behaviour.
    ///
    /// Exhaustive over both spaces, so widening either vocabulary is a build
    /// error here rather than an unnamed behaviour in a rejection.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Subnormals(SubnormalMode::Preserve) => "preserve",
            Self::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: tiler_ir::schedule::FlushedZeroSign::PreservesSign,
            }) => "flush-to-zero.preserves-sign",
            Self::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: tiler_ir::schedule::FlushedZeroSign::AlwaysPositive,
            }) => "flush-to-zero.always-positive",
            Self::Transform(NumericalPermission::Forbidden) => "forbidden",
            Self::Transform(NumericalPermission::Permitted) => "permitted",
        }
    }

    /// The canonical two-byte tag of this behaviour: its space, then its value.
    ///
    /// The space byte is what keeps `Subnormals` and `Transform` values from
    /// colliding once both spaces are widened; the value byte reuses the
    /// governed request-subject tags so one encoding of a behaviour exists in
    /// this crate rather than two that must be kept in agreement.
    pub(crate) const fn tag(self) -> [u8; 2] {
        match self {
            Self::Subnormals(mode) => [0x01, subnormal_tag(mode)],
            Self::Transform(permission) => [0x02, permission_tag(permission)],
        }
    }
}

/// The four means by which a target may honour a required behaviour.
///
/// This is the vocabulary `docs/numerical-semantics.md` already names under
/// "Backend numerical feasibility"; no term is invented here. The distinction
/// that forces a separate authority is that emulation is honoured by *emitting
/// different operations*, which a bound comparison cannot express.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HonouringMeans {
    /// The target's own arithmetic realizes the behaviour.
    SupportedExactly,
    /// The backend emits additional operations that realize the behaviour
    /// exactly. The verdict is satisfied; the emitted program differs.
    SupportedWithExactEmulation,
    /// The behaviour is realized only when the caller's contract already
    /// authorizes the named relaxation on another dimension.
    ///
    /// This is *not* permission to relax the contract. The relaxation must
    /// already be stated in the same request, and when it is not, the predicate
    /// is disproved rather than deferred: the caller's authorization is known at
    /// [`AvailabilityPhase::CompileProfile`] and cannot arrive later.
    SupportedOnlyUnderDeclaredRelaxation {
        /// The behaviour the caller's contract must already state, and where.
        relaxation: RelaxationRequirement,
    },
    /// The target cannot realize the behaviour by any means it declares.
    Unsupported,
}

impl HonouringMeans {
    /// The governed canonical key naming this means.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::SupportedExactly => "supported-exactly",
            Self::SupportedWithExactEmulation => "supported-with-exact-emulation",
            Self::SupportedOnlyUnderDeclaredRelaxation { .. } => {
                "supported-only-under-declared-relaxation"
            }
            Self::Unsupported => "unsupported",
        }
    }

    /// The governed tag naming this means in a canonical descriptor.
    const fn tag(self) -> u8 {
        match self {
            Self::SupportedExactly => 0x01,
            Self::SupportedWithExactEmulation => 0x02,
            Self::SupportedOnlyUnderDeclaredRelaxation { .. } => 0x03,
            Self::Unsupported => 0x04,
        }
    }

    /// Appends this means to a canonical descriptor.
    ///
    /// The conditional arm carries its relaxation, because two profiles whose
    /// declarations differ only in *which* relaxation they require admit
    /// different requests and must not share a descriptor.
    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        if let Self::SupportedOnlyUnderDeclaredRelaxation { relaxation } = self {
            bytes.push(relaxation.dimension.tag());
            bytes.extend_from_slice(&relaxation.behaviour.tag());
        }
    }
}

/// A behaviour the caller's contract must already state for a conditional means.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RelaxationRequirement {
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
}

impl RelaxationRequirement {
    /// Names the dimension and behaviour a caller must already have authorized.
    pub(crate) const fn new(dimension: NumericalDimension, behaviour: DimensionBehaviour) -> Self {
        Self {
            dimension,
            behaviour,
        }
    }

    /// The dimension the authorization must be stated on.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour that dimension must already be resolved to.
    pub(crate) const fn behaviour(self) -> DimensionBehaviour {
        self.behaviour
    }
}

/// One line of a target profile's honourability declaration, before provenance.
///
/// A profile states these as `&'static` data; [`NumericalHonourabilityFact`] is
/// what a checked profile holds, after each line has been attributed to the
/// declaring profile's identity. The split mirrors how a
/// [`crate::feasibility::CapabilityFact`]'s provenance is bound at checking time
/// rather than restated by every declarer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredBehaviour {
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
    means: HonouringMeans,
    phase: AvailabilityPhase,
    authority: FactAuthority,
    validity: FactValidityScope,
}

impl DeclaredBehaviour {
    /// Declares how a target honours one behaviour of one dimension.
    pub(crate) const fn new(
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
        phase: AvailabilityPhase,
        authority: FactAuthority,
        validity: FactValidityScope,
    ) -> Self {
        Self {
            dimension,
            behaviour,
            means,
            phase,
            authority,
            validity,
        }
    }

    /// Declares a compile-time governed-profile honourability guarantee.
    ///
    /// The overwhelmingly common shape: a portable profile fact known before any
    /// artifact exists. A later-phase declaration states its phase explicitly.
    pub(crate) const fn compile_profile(
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
    ) -> Self {
        Self::new(
            dimension,
            behaviour,
            means,
            AvailabilityPhase::CompileProfile,
            FactAuthority::GovernedProfile,
            FactValidityScope::PortableProfile,
        )
    }

    /// Binds this declaration to the profile that declared it.
    pub(crate) const fn attributed_to(
        self,
        profile: TargetProfileIdentity,
    ) -> NumericalHonourabilityFact {
        NumericalHonourabilityFact {
            declaration: self,
            provenance: FactProvenance::declared_by(profile),
        }
    }

    /// Appends this declaration's canonical bytes.
    ///
    /// The one encoding of a declared behaviour in this crate. A checked profile
    /// descriptor and a request subject both reach it, so a widened vocabulary
    /// cannot change one and leave the other reading the old shape.
    pub(crate) fn encode_declaration(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.dimension.tag());
        bytes.extend_from_slice(&self.behaviour.tag());
        self.means.encode(bytes);
        bytes.push(self.phase.tag());
        bytes.push(self.authority.tag());
        bytes.push(self.validity.tag());
    }
}

/// A typed honourability fact: how one behaviour of one dimension is honoured.
///
/// It carries the same provenance discipline a
/// [`crate::feasibility::CapabilityFact`] does — an availability phase, a fact
/// authority, a validity scope, and the declaring profile's identity — so a
/// rejection can name where the claim came from (ADR 0076 item 3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericalHonourabilityFact {
    declaration: DeclaredBehaviour,
    provenance: FactProvenance,
}

impl NumericalHonourabilityFact {
    /// The dimension this fact speaks about.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.declaration.dimension
    }

    /// The behaviour of that dimension this fact speaks about.
    pub(crate) const fn behaviour(self) -> DimensionBehaviour {
        self.declaration.behaviour
    }

    /// The means by which the behaviour is honoured, if it is.
    pub(crate) const fn means(self) -> HonouringMeans {
        self.declaration.means
    }

    /// The phase from which this fact is available.
    pub(crate) const fn phase(self) -> AvailabilityPhase {
        self.declaration.phase
    }

    /// The authority vouching for this fact.
    pub(crate) const fn authority(self) -> FactAuthority {
        self.declaration.authority
    }

    /// Where this fact came from.
    pub(crate) const fn provenance(self) -> FactProvenance {
        self.provenance
    }

    /// The canonical sort key: dimension, then behaviour, then phase.
    pub(crate) const fn sort_key(self) -> (u8, [u8; 2], AvailabilityPhase) {
        (
            self.declaration.dimension.tag(),
            self.declaration.behaviour.tag(),
            self.declaration.phase,
        )
    }

    /// Whether this fact declares the behaviour honoured without conditions.
    ///
    /// A conditional means is deliberately excluded: whether it honours anything
    /// depends on the request, so it is not an alternative the profile *offers*.
    pub(crate) const fn is_unconditionally_honoured(self) -> bool {
        matches!(
            self.declaration.means,
            HonouringMeans::SupportedExactly | HonouringMeans::SupportedWithExactEmulation
        )
    }

    /// Appends this fact's declaration to a canonical profile descriptor.
    ///
    /// The provenance is excluded for the same reason a capability fact's is: it
    /// cites the descriptor's own subject.
    pub(crate) fn encode_declaration(&self, bytes: &mut Vec<u8>) {
        self.declaration.encode_declaration(bytes);
    }
}

/// A candidate requirement: the behaviour the caller's contract needs on one
/// dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericalRequirement {
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
}

impl NumericalRequirement {
    /// Requires `behaviour` on `dimension`.
    pub(crate) const fn new(dimension: NumericalDimension, behaviour: DimensionBehaviour) -> Self {
        Self {
            dimension,
            behaviour,
        }
    }

    /// The dimension this requirement ranges over.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour the contract requires.
    pub(crate) const fn behaviour(self) -> DimensionBehaviour {
        self.behaviour
    }
}

/// A dimension whose required behaviour the target honours, and by what means.
///
/// The means is retained rather than collapsed to a boolean because it is what
/// an artifact record and a cost model both need: an emulated dimension is
/// honoured by emitted operations, which is work that a satisfied predicate
/// alone would hide.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HonouredDimension {
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
    means: HonouringMeans,
    profile: TargetProfileIdentity,
}

impl HonouredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
        profile: TargetProfileIdentity,
    ) -> Self {
        Self {
            dimension,
            behaviour,
            means,
            profile,
        }
    }

    /// The dimension honoured.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour the contract required.
    pub(crate) const fn behaviour(self) -> DimensionBehaviour {
        self.behaviour
    }

    /// The means by which the target honours it.
    pub(crate) const fn means(self) -> HonouringMeans {
        self.means
    }

    /// The profile that declared the honouring means.
    pub(crate) const fn profile(self) -> TargetProfileIdentity {
        self.profile
    }
}

/// A dimension the target declares it cannot honour as required.
///
/// This is the rejection shape ADR 0076 item 5 requires, and it is what replaces
/// `strict-f32: required 1, available 0`: the dimension, the required behaviour,
/// the behaviour the target does declare, the means the profile offers for the
/// required behaviour, and the declaring profile's identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnhonouredDimension {
    dimension: NumericalDimension,
    required: DimensionBehaviour,
    means: HonouringMeans,
    honoured: Option<DimensionBehaviour>,
    profile: TargetProfileIdentity,
}

impl UnhonouredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        required: DimensionBehaviour,
        means: HonouringMeans,
        honoured: Option<DimensionBehaviour>,
        profile: TargetProfileIdentity,
    ) -> Self {
        Self {
            dimension,
            required,
            means,
            honoured,
            profile,
        }
    }

    /// The dimension the contract could not be honoured on.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour the caller's contract required.
    pub(crate) const fn required(self) -> DimensionBehaviour {
        self.required
    }

    /// The means the profile declares for the required behaviour.
    pub(crate) const fn means(self) -> HonouringMeans {
        self.means
    }

    /// The behaviour on this dimension the profile does honour unconditionally,
    /// in canonical order, when it honours one at all.
    ///
    /// It is reported so a caller can see what contract this target would
    /// accept. It is never substituted for the stated one: only the caller may
    /// change what its program means (ADR 0076 item 5).
    pub(crate) const fn honoured(self) -> Option<DimensionBehaviour> {
        self.honoured
    }

    /// The profile that declared the means.
    pub(crate) const fn profile(self) -> TargetProfileIdentity {
        self.profile
    }
}

/// A dimension the profile does not speak to at all.
///
/// ADR 0043's `Unknown` in its exact sense — no admissible proof or query path —
/// and the clause that makes an unenumerated dimension fail closed instead of
/// defaulting to honoured. A profile that enumerates the dimension but not the
/// *required behaviour* is the same case for the same reason: nothing declared
/// says how that behaviour would be realized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UndeclaredDimension {
    dimension: NumericalDimension,
    required: DimensionBehaviour,
}

impl UndeclaredDimension {
    pub(crate) const fn new(dimension: NumericalDimension, required: DimensionBehaviour) -> Self {
        Self {
            dimension,
            required,
        }
    }

    /// The dimension nothing available declares.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour the caller's contract required.
    pub(crate) const fn required(self) -> DimensionBehaviour {
        self.required
    }
}

/// A dimension whose declaration is admissible only from a later phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeferredDimension {
    dimension: NumericalDimension,
    required: DimensionBehaviour,
    phase: AvailabilityPhase,
}

impl DeferredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        required: DimensionBehaviour,
        phase: AvailabilityPhase,
    ) -> Self {
        Self {
            dimension,
            required,
            phase,
        }
    }

    /// The dimension whose declaration is not yet available.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour the caller's contract required.
    pub(crate) const fn required(self) -> DimensionBehaviour {
        self.required
    }

    /// The earliest phase that can supply the declaration.
    pub(crate) const fn phase(self) -> AvailabilityPhase {
        self.phase
    }
}
