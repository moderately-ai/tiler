#![allow(
    dead_code,
    reason = "the honourability authority itself is on the compile path through assess_region and assess_contract; what stays unconstructed is the reserved declaration surface no governed profile yet uses — the emulated, relaxation-conditional, and unsupported honouring means, the relaxation requirement that names an authorization, the canonical dimension order, and the dimensions no admitted operation can consume — which only a target profile declaring something other than exact native support, or an operation vocabulary wider than this build's, can produce, and which `declare-metal-numerical-honourability` is the first to reach"
)]

//! Per-dimension, per-dtype numerical honourability, a peer of the capability
//! authority.
//!
//! ADR 0076 item 3. A target profile declares, for each dimension of the
//! resolved numerical contract it can be asked about *and for each arithmetic
//! type it can be asked about it in*, which behaviour it honours and *by what
//! means*. This module owns that vocabulary; the composition of a declaration and
//! a caller requirement into one ADR 0043 outcome lives beside the capability
//! assessment in [`crate::feasibility`], because a candidate has exactly one
//! feasibility verdict and the two kinds of predicate contribute to it together.
//!
//! # Why the key carries an arithmetic type
//!
//! **Measurement.** On one Apple row — same GPU, same math modes, modules
//! declaring `air.compile.denorms_disable` identically — `f32` arithmetic flushes
//! subnormals, `f16` arithmetic preserves them, and `bf16` flushes. So on that
//! one profile, [`NumericalDimension::InputSubnormals`] is honoured
//! [`HonouringMeans::SupportedExactly`] for `f16` and
//! [`HonouringMeans::Unsupported`] for `f32`.
//!
//! **Inference.** A declaration keyed by dimension alone therefore has to state
//! one of those two wrongly, and a preset assuming one behaviour per dimension
//! per profile assumes something already known to be false. The key is
//! `(dimension, arithmetic type)` for that reason, not for symmetry; an
//! arithmetic type a profile does not speak about is `Unknown` in ADR 0043's
//! exact sense and fails closed, exactly as an unenumerated dimension does.
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

use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, MaterializationRounding,
    NumericalPermission, SubnormalMode,
};

use crate::feasibility::{
    AvailabilityPhase, FactAuthority, FactProvenance, FactValidityScope, TargetProfileIdentity,
};
use crate::request::{permission_tag, subnormal_tag};

/// The behaviour space one numerical dimension ranges over.
///
/// A dimension's space is fixed by the dimension, and pairing a dimension with a
/// behaviour from another space is *malformed* rather than a verdict. Naming the
/// space once — rather than restating the pairing at each site that checks it —
/// is what keeps [`NumericalDimension::admits`] and [`DimensionBehaviour`] from
/// drifting apart as either vocabulary grows.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum BehaviourSpace {
    /// Resolutions of one subnormal dimension.
    Subnormals,
    /// Resolutions of one transform-permission dimension.
    Transform,
    /// Resolutions of the approximate-intrinsic accuracy envelope.
    Approximation,
    /// Resolutions of one exceptional-value assumption.
    ExceptionalValue,
    /// Resolutions of an observable materialization boundary's rounding.
    Rounding,
}

/// A governed dimension of the resolved numerical contract.
///
/// The vocabulary is bounded and deliberately not open: an honourability
/// predicate ranges over these typed dimensions rather than a free-form backend
/// property bag, for the same reason ADR 0043 gives for the capability axes. The
/// derived ordering is the canonical evaluation and reporting order.
///
/// # What is here and why
///
/// These are the dimensions `docs/numerical-semantics.md` names as the granular
/// policy: subnormal input and result handling, contraction, reassociation,
/// operand permutation, signed-zero distinction, reciprocal replacement,
/// approximate intrinsics, NaN and infinity assumptions, and the rounding an
/// observable materialization boundary applies. No term is invented here.
///
/// **Distributivity is deliberately absent.** `docs/numerical-semantics.md`
/// records it as a third numerical dimension and then states that no
/// distributivity permission is admitted: the canonical policy has no such field,
/// and whether to admit one at all is reserved to the decision that admits a
/// tensor-contraction family. Adding it here would convert a reserved question
/// into an implemented permission.
///
/// **Not every dimension here is one the region IR carries.**
/// [`tiler_ir::schedule::NumericalRealization`] carries four of them, and the
/// contract is complete over all of them because completeness is what makes an
/// unenumerated dimension fail closed. [`crate::policy`] owns the rule that keeps
/// the difference safe: a dimension outside the realization may take any
/// resolution only while no admitted operation can consume it, and a contract
/// that resolves one otherwise is rejected by name rather than compiled under a
/// realization that never mentioned it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum NumericalDimension {
    /// Treatment of subnormal operands before each arithmetic operation.
    InputSubnormals,
    /// Treatment of a newly produced subnormal arithmetic result.
    ResultSubnormals,
    /// Whether fused-multiply-add contraction is permitted.
    Contraction,
    /// Whether reduction reassociation — regrouping while preserving logical
    /// operand order — is permitted.
    Reassociation,
    /// Whether operand permutation — changing logical contributor order — is
    /// permitted.
    ///
    /// Independent of [`Self::Reassociation`]: granting one never grants the
    /// other, and a physical schedule proves the two properties separately.
    Permutation,
    /// Whether eliminating the distinction between the two signed zeros is
    /// permitted.
    SignedZero,
    /// Whether replacing a division by a reciprocal multiplication is permitted.
    ReciprocalTransform,
    /// The maximum accuracy envelope approximate intrinsics may consume.
    ApproximateIntrinsics,
    /// Whether NaN operands may be assumed absent, and on what evidence.
    NanAssumptions,
    /// Whether infinite operands may be assumed absent, and on what evidence.
    InfinityAssumptions,
    /// The rounding an observable materialization boundary applies.
    MaterializationRounding,
}

/// The canonical dimension order. Single source of truth for evaluation and
/// reporting order, matching the derived [`NumericalDimension`] ordering.
pub(crate) const CANONICAL_DIMENSIONS: [NumericalDimension; 11] = [
    NumericalDimension::InputSubnormals,
    NumericalDimension::ResultSubnormals,
    NumericalDimension::Contraction,
    NumericalDimension::Reassociation,
    NumericalDimension::Permutation,
    NumericalDimension::SignedZero,
    NumericalDimension::ReciprocalTransform,
    NumericalDimension::ApproximateIntrinsics,
    NumericalDimension::NanAssumptions,
    NumericalDimension::InfinityAssumptions,
    NumericalDimension::MaterializationRounding,
];

impl NumericalDimension {
    /// The governed canonical predicate key for this dimension.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::InputSubnormals => "numerics.input-subnormals",
            Self::ResultSubnormals => "numerics.result-subnormals",
            Self::Contraction => "numerics.contraction",
            Self::Reassociation => "numerics.reassociation",
            Self::Permutation => "numerics.permutation",
            Self::SignedZero => "numerics.signed-zero",
            Self::ReciprocalTransform => "numerics.reciprocal-transform",
            Self::ApproximateIntrinsics => "numerics.approximate-intrinsics",
            Self::NanAssumptions => "numerics.nan-assumptions",
            Self::InfinityAssumptions => "numerics.infinity-assumptions",
            Self::MaterializationRounding => "numerics.materialization-rounding",
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
            Self::Permutation => 0x05,
            Self::SignedZero => 0x06,
            Self::ReciprocalTransform => 0x07,
            Self::ApproximateIntrinsics => 0x08,
            Self::NanAssumptions => 0x09,
            Self::InfinityAssumptions => 0x0a,
            Self::MaterializationRounding => 0x0b,
        }
    }

    /// The behaviour space this dimension ranges over.
    pub(crate) const fn space(self) -> BehaviourSpace {
        match self {
            Self::InputSubnormals | Self::ResultSubnormals => BehaviourSpace::Subnormals,
            Self::Contraction
            | Self::Reassociation
            | Self::Permutation
            | Self::SignedZero
            | Self::ReciprocalTransform => BehaviourSpace::Transform,
            Self::ApproximateIntrinsics => BehaviourSpace::Approximation,
            Self::NanAssumptions | Self::InfinityAssumptions => BehaviourSpace::ExceptionalValue,
            Self::MaterializationRounding => BehaviourSpace::Rounding,
        }
    }

    /// Whether `behaviour` is a value this dimension can take.
    ///
    /// A declaration or a requirement pairing a dimension with another space's
    /// behaviour is malformed, never a verdict.
    pub(crate) const fn admits(self, behaviour: DimensionBehaviour) -> bool {
        matches!(
            (self.space(), behaviour.space()),
            (BehaviourSpace::Subnormals, BehaviourSpace::Subnormals)
                | (BehaviourSpace::Transform, BehaviourSpace::Transform)
                | (BehaviourSpace::Approximation, BehaviourSpace::Approximation)
                | (
                    BehaviourSpace::ExceptionalValue,
                    BehaviourSpace::ExceptionalValue
                )
                | (BehaviourSpace::Rounding, BehaviourSpace::Rounding)
        )
    }
}

/// One behaviour a numerical dimension can take.
///
/// The arms are the behaviour spaces the governed dimensions range over. They
/// are kept as one type so a requirement, a declaration, and a rejection can all
/// name "the behaviour on this dimension" without the caller switching on the
/// dimension first; [`NumericalDimension::admits`] is what keeps a subnormal
/// behaviour off a transform dimension.
///
/// Each arm's payload is the space `docs/numerical-semantics.md` requires for
/// that dimension rather than a uniform permission. The approximate-intrinsic
/// arm is the load-bearing case: that contract says the dimension "resolves to a
/// maximum accuracy envelope … **not a boolean**", so spelling it as a permission
/// would state no bound at all.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DimensionBehaviour {
    /// A resolution of one subnormal dimension.
    Subnormals(SubnormalMode),
    /// A resolution of one transform-permission dimension.
    Transform(NumericalPermission),
    /// A resolution of the approximate-intrinsic accuracy envelope.
    Approximation(ApproximationEnvelope),
    /// A resolution of one exceptional-value assumption.
    ExceptionalValue(ExceptionalValueAssumption),
    /// A resolution of a materialization boundary's rounding.
    Rounding(MaterializationRounding),
}

impl DimensionBehaviour {
    /// The space this behaviour belongs to.
    pub(crate) const fn space(self) -> BehaviourSpace {
        match self {
            Self::Subnormals(_) => BehaviourSpace::Subnormals,
            Self::Transform(_) => BehaviourSpace::Transform,
            Self::Approximation(_) => BehaviourSpace::Approximation,
            Self::ExceptionalValue(_) => BehaviourSpace::ExceptionalValue,
            Self::Rounding(_) => BehaviourSpace::Rounding,
        }
    }

    /// The governed canonical key naming this behaviour.
    ///
    /// Exhaustive over every space, so widening any of them is a build error here
    /// rather than an unnamed behaviour in a rejection. The approximate-intrinsic
    /// arm returns the envelope's own versioned key, which *is* the name of that
    /// behaviour: two envelopes are two behaviours.
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
            // Delegated rather than restated: the envelope vocabulary owns its
            // own key strings, and a second spelling here could be renamed alone.
            Self::Approximation(envelope) => envelope.key(),
            Self::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption) => {
                "make-no-assumption"
            }
            Self::ExceptionalValue(ExceptionalValueAssumption::AssumeAbsent { provenance }) => {
                match provenance {
                    tiler_ir::schedule::ValueDomainProvenance::CompilerProven => {
                        "assume-absent.compiler-proven"
                    }
                    tiler_ir::schedule::ValueDomainProvenance::RuntimeValidated => {
                        "assume-absent.runtime-validated"
                    }
                    tiler_ir::schedule::ValueDomainProvenance::CallerDeclaredUnvalidated => {
                        "assume-absent.caller-declared-unvalidated"
                    }
                }
            }
            Self::Rounding(MaterializationRounding::NearestTiesToEven) => "nearest-ties-to-even",
        }
    }

    /// Appends this behaviour's canonical bytes: its space, then its value.
    ///
    /// The space byte is what keeps two spaces' values from colliding once both
    /// are widened; the subnormal and transform value bytes reuse the governed
    /// request-subject tags so one encoding of those behaviours exists in this
    /// crate rather than two that must be kept in agreement.
    ///
    /// The approximate-intrinsic arm writes the envelope's own tag, because the
    /// envelope *is* the behaviour: two profiles honouring different envelopes
    /// admit different requests and must not share a descriptor. The envelope is
    /// a governed closed vocabulary rather than a free-form key, so one byte
    /// distinguishes every one of them and adding another is a build error at
    /// [`ApproximationEnvelope::tag`] rather than a silently colliding encoding.
    pub(crate) fn encode(self, bytes: &mut Vec<u8>) {
        match self {
            Self::Subnormals(mode) => {
                bytes.push(0x01);
                bytes.push(subnormal_tag(mode));
            }
            Self::Transform(permission) => {
                bytes.push(0x02);
                bytes.push(permission_tag(permission));
            }
            Self::Approximation(envelope) => {
                bytes.push(0x03);
                bytes.push(envelope.tag());
            }
            Self::ExceptionalValue(assumption) => {
                bytes.push(0x04);
                match assumption {
                    ExceptionalValueAssumption::MakeNoAssumption => bytes.push(0x01),
                    ExceptionalValueAssumption::AssumeAbsent { provenance } => {
                        bytes.push(0x02);
                        bytes.push(match provenance {
                            tiler_ir::schedule::ValueDomainProvenance::CompilerProven => 0x01,
                            tiler_ir::schedule::ValueDomainProvenance::RuntimeValidated => 0x02,
                            tiler_ir::schedule::ValueDomainProvenance::CallerDeclaredUnvalidated => {
                                0x03
                            }
                        });
                    }
                }
            }
            Self::Rounding(rounding) => {
                bytes.push(0x05);
                bytes.push(match rounding {
                    MaterializationRounding::NearestTiesToEven => 0x01,
                });
            }
        }
    }

    /// This behaviour's canonical bytes, as a comparable and orderable key.
    ///
    /// Used where a behaviour has to be sorted or compared for uniqueness. It is
    /// the encoding itself rather than a separate summary, so a widened behaviour
    /// space cannot make two distinct behaviours tie here while remaining
    /// distinct in the descriptor those same bytes build.
    pub(crate) fn canonical_key(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
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
            bytes.push(relaxation.arithmetic.tag());
            relaxation.behaviour.encode(bytes);
        }
    }
}

/// A behaviour the caller's contract must already state for a conditional means.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RelaxationRequirement {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    behaviour: DimensionBehaviour,
}

impl RelaxationRequirement {
    /// Names the dimension, arithmetic type, and behaviour a caller must already
    /// have authorized.
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        behaviour: DimensionBehaviour,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            behaviour,
        }
    }

    /// The dimension the authorization must be stated on.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type the authorization must be stated for.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
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
    arithmetic: ArithmeticType,
    behaviour: DimensionBehaviour,
    means: HonouringMeans,
    phase: AvailabilityPhase,
    authority: FactAuthority,
    validity: FactValidityScope,
}

impl DeclaredBehaviour {
    /// Declares how a target honours one behaviour of one dimension, in one
    /// arithmetic type.
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
        phase: AvailabilityPhase,
        authority: FactAuthority,
        validity: FactValidityScope,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
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
        arithmetic: ArithmeticType,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
    ) -> Self {
        Self::new(
            dimension,
            arithmetic,
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
        bytes.push(self.arithmetic.tag());
        self.behaviour.encode(bytes);
        self.means.encode(bytes);
        bytes.push(self.phase.tag());
        bytes.push(self.authority.tag());
        bytes.push(self.validity.tag());
    }
}

/// A typed honourability fact: how one behaviour of one dimension is honoured,
/// in one arithmetic type.
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

    /// The arithmetic type this fact speaks about.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.declaration.arithmetic
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

    /// The canonical sort key: dimension, arithmetic type, behaviour, phase.
    ///
    /// The behaviour contributes its canonical bytes rather than a fixed-width
    /// tag, because one behaviour space is variable-width: two distinct accuracy
    /// envelopes would tie under a tag, and the duplicate check this key feeds
    /// would then reject a profile that declared both.
    pub(crate) fn sort_key(self) -> (u8, u8, Vec<u8>, AvailabilityPhase) {
        (
            self.declaration.dimension.tag(),
            self.declaration.arithmetic.tag(),
            self.declaration.behaviour.canonical_key(),
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
/// dimension, in one arithmetic type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericalRequirement {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    behaviour: DimensionBehaviour,
}

impl NumericalRequirement {
    /// Requires `behaviour` on `dimension`, for `arithmetic`.
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        behaviour: DimensionBehaviour,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            behaviour,
        }
    }

    /// The dimension this requirement ranges over.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type this requirement is stated for.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
    }

    /// The behaviour the contract requires.
    pub(crate) const fn behaviour(self) -> DimensionBehaviour {
        self.behaviour
    }

    /// The canonical key this requirement is unique under.
    pub(crate) const fn subject(self) -> (NumericalDimension, ArithmeticType) {
        (self.dimension, self.arithmetic)
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
    arithmetic: ArithmeticType,
    behaviour: DimensionBehaviour,
    means: HonouringMeans,
    profile: TargetProfileIdentity,
}

impl HonouredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
        profile: TargetProfileIdentity,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            behaviour,
            means,
            profile,
        }
    }

    /// The dimension honoured.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type it is honoured in.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
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
/// `strict-f32: required 1, available 0`: the dimension, the arithmetic type, the
/// required behaviour, the behaviour the target does declare, the means the
/// profile offers for the required behaviour, and the declaring profile's
/// identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnhonouredDimension {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    required: DimensionBehaviour,
    means: HonouringMeans,
    honoured: Option<DimensionBehaviour>,
    profile: TargetProfileIdentity,
}

impl UnhonouredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        required: DimensionBehaviour,
        means: HonouringMeans,
        honoured: Option<DimensionBehaviour>,
        profile: TargetProfileIdentity,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
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

    /// The arithmetic type it could not be honoured in.
    ///
    /// Reported because the same dimension can be honoured in one type and
    /// unhonourable in another on one profile: a rejection that named only the
    /// dimension would be false about the other type.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
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

/// A dimension the profile does not speak to at all, in the required arithmetic
/// type.
///
/// ADR 0043's `Unknown` in its exact sense — no admissible proof or query path —
/// and the clause that makes an unenumerated dimension fail closed instead of
/// defaulting to honoured. A profile that enumerates the dimension but not the
/// *required behaviour* is the same case for the same reason, and so is one that
/// enumerates both but only for another arithmetic type: nothing declared says
/// how that behaviour would be realized in this one, and a neighbouring type's
/// fact is measurably not a substitute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UndeclaredDimension {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    required: DimensionBehaviour,
}

impl UndeclaredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        required: DimensionBehaviour,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            required,
        }
    }

    /// The dimension nothing available declares.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type nothing available declares it for.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
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
    arithmetic: ArithmeticType,
    required: DimensionBehaviour,
    phase: AvailabilityPhase,
}

impl DeferredDimension {
    pub(crate) const fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        required: DimensionBehaviour,
        phase: AvailabilityPhase,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            required,
            phase,
        }
    }

    /// The dimension whose declaration is not yet available.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type whose declaration is not yet available.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
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
