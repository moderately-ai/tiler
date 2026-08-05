//! The one shared scalar-arithmetic policy vocabulary.
//!
//! This module is the single authority for the governed numerical dimensions,
//! the behaviours they range over, the policy subject a declaration or an
//! obligation is keyed by, the means a target honours a behaviour by, the policy
//! locus a requirement arose at, and the structured provenance behind a
//! numerical fact. `tiler-compiler` and `tiler-artifact` both name these types by
//! re-export, so the vocabularies exist once and a widened one is a build error
//! at every total encoder and consumer in the workspace.
//!
//! # Why this vocabulary is sited here
//!
//! `record-delivered-numerical-realization` eliminated relocating the means
//! vocabulary into `tiler-ir` and chose opaque compiler-minted key bytes. That
//! elimination recorded its own reopening trigger: a consumer of the artifact
//! that must *reason over* the means rather than compare them. Two facts fire
//! it. ADR 0076 item 4 names exactly that consumer — one comparing generated
//! output against a CPU reference, which must know an emulated dimension from a
//! natively honoured one. And [`crate::numerics::HonouringMeans::label`] is **not
//! injective**: every
//! [`crate::numerics::HonouringMeans::SupportedOnlyUnderDeclaredRelaxation`] value returns
//! the same string whatever relaxation it names, so the opaque-key mechanism
//! cannot carry the record even for comparison. ADR 0076 records the
//! supersession.
//!
//! The siting follows an existing precedent rather than inventing one.
//! [`crate::program::abi::AvailabilityPhase`] is ADR 0043 target-fact
//! provenance vocabulary defined in [`crate::program::abi`], and both consumers
//! already name it by re-export. So does every behaviour vocabulary below:
//! [`crate::schedule::SubnormalMode`], [`crate::schedule::NumericalPermission`],
//! [`crate::schedule::ApproximationEnvelope`],
//! [`crate::schedule::ExceptionalValueAssumption`], and
//! [`crate::schedule::MaterializationRounding`] are all [`crate::schedule`]
//! types, and [`crate::numerics::DimensionBehaviour`] is a sum over exactly those five. The
//! relocation therefore moves no meaning into the semantic graph: this is a
//! contract-vocabulary module beside [`crate::schedule`], not inside
//! [`crate::semantic`], and the target-aware *assessment* — which profile
//! declares what, and how feasibility composes it — stays entirely in
//! `tiler_compiler::target`.

use std::error::Error;
use std::fmt;

use crate::identity::{push_len, push_slice};
use crate::program::SemanticOccurrence;
use crate::program::abi::AvailabilityPhase;
use crate::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode, ValueDomainProvenance,
};
use crate::semantic::{
    CanonicalField, CanonicalValue, CanonicalValueView, ResolvedValueType, SCALAR_TYPE_FACT_CLASS,
    SCALAR_TYPE_FACT_WIDTH_BITS, builtin_scalar_value_type_facts, builtin_scalar_value_types,
};

/// The number of governed scalar-arithmetic dimensions.
///
/// Exported so a dense per-dimension array is spelled once and a widened
/// vocabulary is a build error at every array literal rather than a silently
/// short one.
pub const DIMENSION_COUNT: usize = 11;

/// The behaviour space one numerical dimension ranges over.
///
/// A dimension's space is fixed by the dimension, and pairing a dimension with a
/// behaviour from another space is *malformed* rather than a verdict. Naming the
/// space once — rather than restating the pairing at each site that checks it —
/// is what keeps [`NumericalDimension::admits`] and [`DimensionBehaviour`] from
/// drifting apart as either vocabulary grows.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BehaviourSpace {
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

/// A governed dimension of the resolved scalar-arithmetic contract.
///
/// The vocabulary is bounded and deliberately not open: an honourability
/// predicate ranges over these typed dimensions rather than a free-form backend
/// property bag, for the same reason ADR 0043 gives for the capability axes.
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
/// distributivity permission is admitted: the canonical policy has no such
/// field, and whether to admit one at all is reserved to the decision that
/// admits a tensor-contraction family. Adding it here would convert a reserved
/// question into an implemented permission.
///
/// **Not every dimension here is one the region IR carries.**
/// [`crate::schedule::NumericalRealization`] carries eight of them, and the
/// contract is complete over all eleven because completeness is what makes an
/// unenumerated dimension fail closed. `tiler_compiler::policy` owns the rule
/// that keeps the difference safe: a dimension outside the realization may take
/// any resolution only while no admitted operation can consume it.
///
/// Deliberately **not** `#[non_exhaustive]` under ADR 0074 convention 5b: every
/// out-of-crate consumer maps it totally onto an identity tag, and a wildcard
/// arm there would have to invent a tag only the variant itself determines.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NumericalDimension {
    /// Treatment of subnormal operands before each arithmetic operation.
    InputSubnormals,
    /// Treatment of a newly produced subnormal arithmetic result.
    ResultSubnormals,
    /// Whether fused-multiply-add contraction is permitted.
    Contraction,
    /// Whether ordered reassociation — regrouping one same-operation operand
    /// sequence without changing its logical order — is permitted.
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

/// The canonical dimension order: evaluation, reporting, encoding, and the dense
/// array index below are all this one order.
pub const CANONICAL_DIMENSIONS: [NumericalDimension; DIMENSION_COUNT] = [
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
    #[must_use]
    pub const fn key(self) -> &'static str {
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

    /// The governed wire tag naming this dimension.
    ///
    /// Written by an exhaustive match rather than read from the discriminant, so
    /// adding or reordering a dimension is a build error here instead of a
    /// silent change to every target profile descriptor ever produced (ADR 0074
    /// convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
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

    /// Resolves a governed wire tag, or `None` for an unrecognized dimension.
    ///
    /// The fail-closed decode half, written as its own exhaustive match rather
    /// than derived from [`Self::tag`], following [`AvailabilityPhase::from_tag`]'s
    /// established shape: a reader handed a tag this build has never been taught
    /// rejects rather than approximating.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::InputSubnormals),
            0x02 => Some(Self::ResultSubnormals),
            0x03 => Some(Self::Contraction),
            0x04 => Some(Self::Reassociation),
            0x05 => Some(Self::Permutation),
            0x06 => Some(Self::SignedZero),
            0x07 => Some(Self::ReciprocalTransform),
            0x08 => Some(Self::ApproximateIntrinsics),
            0x09 => Some(Self::NanAssumptions),
            0x0a => Some(Self::InfinityAssumptions),
            0x0b => Some(Self::MaterializationRounding),
            _ => None,
        }
    }

    /// The dense array index this dimension occupies.
    ///
    /// One exhaustive shared match. Every dense per-dimension array in the
    /// workspace indexes through this and no other mapping, so a widened
    /// vocabulary cannot leave one array indexed by an old position.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::InputSubnormals => 0,
            Self::ResultSubnormals => 1,
            Self::Contraction => 2,
            Self::Reassociation => 3,
            Self::Permutation => 4,
            Self::SignedZero => 5,
            Self::ReciprocalTransform => 6,
            Self::ApproximateIntrinsics => 7,
            Self::NanAssumptions => 8,
            Self::InfinityAssumptions => 9,
            Self::MaterializationRounding => 10,
        }
    }

    /// The behaviour space this dimension ranges over.
    #[must_use]
    pub const fn space(self) -> BehaviourSpace {
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
    /// A declaration, a requirement, or a decoded row pairing a dimension with
    /// another space's behaviour is *malformed* rather than a verdict, which is
    /// why every reader rejects it by name.
    #[must_use]
    pub const fn admits(self, behaviour: DimensionBehaviour) -> bool {
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

impl fmt::Display for NumericalDimension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
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
/// maximum accuracy envelope … **not a boolean**", so spelling it as a
/// permission would state no bound at all.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DimensionBehaviour {
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
    #[must_use]
    pub const fn space(self) -> BehaviourSpace {
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
    /// Exhaustive over every space, so widening any of them is a build error
    /// here rather than an unnamed behaviour in a rejection. The
    /// approximate-intrinsic arm returns the envelope's own versioned key, which
    /// *is* the name of that behaviour: two envelopes are two behaviours.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Subnormals(SubnormalMode::Preserve) => "preserve",
            Self::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            }) => "flush-to-zero.preserves-sign",
            Self::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
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
                    ValueDomainProvenance::CompilerProven => "assume-absent.compiler-proven",
                    ValueDomainProvenance::RuntimeValidated => "assume-absent.runtime-validated",
                    ValueDomainProvenance::CallerDeclaredUnvalidated => {
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
    /// are widened.
    ///
    /// The approximate-intrinsic arm writes the envelope's own tag, because the
    /// envelope *is* the behaviour: two profiles honouring different envelopes
    /// admit different requests and must not share a descriptor.
    pub fn encode(self, bytes: &mut Vec<u8>) {
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
                        bytes.push(value_domain_provenance_tag(provenance));
                    }
                }
            }
            Self::Rounding(rounding) => {
                bytes.push(0x05);
                bytes.push(materialization_rounding_tag(rounding));
            }
        }
    }

    /// Reads one canonically encoded behaviour, or `None` for an unknown tag.
    ///
    /// Total in both directions, so a decoder and an encoder cannot disagree
    /// about what a behaviour *is*. Returns the behaviour and the bytes
    /// consumed, because a behaviour is variable width: an exceptional-value
    /// assumption carrying provenance is three bytes and every other behaviour
    /// is two.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match bytes {
            [0x01, value, ..] => Some((Self::Subnormals(subnormal_from_tag(*value)?), 2)),
            [0x02, value, ..] => Some((Self::Transform(permission_from_tag(*value)?), 2)),
            [0x03, value, ..] => Some((
                Self::Approximation(approximation_envelope_from_tag(*value)?),
                2,
            )),
            [0x04, 0x01, ..] => Some((
                Self::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
                2,
            )),
            [0x04, 0x02, provenance, ..] => Some((
                Self::ExceptionalValue(ExceptionalValueAssumption::AssumeAbsent {
                    provenance: value_domain_provenance_from_tag(*provenance)?,
                }),
                3,
            )),
            [0x05, value, ..] => Some((
                Self::Rounding(materialization_rounding_from_tag(*value)?),
                2,
            )),
            _ => None,
        }
    }

    /// This behaviour's canonical bytes, as a comparable and orderable key.
    ///
    /// The encoding itself rather than a separate summary, so a widened
    /// behaviour space cannot make two distinct behaviours tie here while
    /// remaining distinct in the descriptor those same bytes build.
    #[must_use]
    pub fn canonical_key(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

/// The governed wire tag naming one subnormal resolution.
#[must_use]
pub const fn subnormal_tag(mode: SubnormalMode) -> u8 {
    match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    }
}

/// Resolves a governed subnormal tag, or `None` for an unrecognized one.
#[must_use]
pub const fn subnormal_from_tag(tag: u8) -> Option<SubnormalMode> {
    match tag {
        0x01 => Some(SubnormalMode::Preserve),
        0x02 => Some(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        }),
        0x03 => Some(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        }),
        _ => None,
    }
}

/// The governed wire tag naming one transform permission.
#[must_use]
pub const fn permission_tag(permission: NumericalPermission) -> u8 {
    match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    }
}

/// Resolves a governed transform-permission tag, or `None` for an unrecognized
/// one.
#[must_use]
pub const fn permission_from_tag(tag: u8) -> Option<NumericalPermission> {
    match tag {
        0x01 => Some(NumericalPermission::Forbidden),
        0x02 => Some(NumericalPermission::Permitted),
        _ => None,
    }
}

/// Resolves a governed approximation-envelope tag, or `None` for an
/// unrecognized one.
///
/// The fail-closed decode half of [`ApproximationEnvelope::tag`].
#[must_use]
pub const fn approximation_envelope_from_tag(tag: u8) -> Option<ApproximationEnvelope> {
    match tag {
        0x01 => Some(ApproximationEnvelope::Forbidden),
        0x02 => Some(ApproximationEnvelope::BackendElementary),
        _ => None,
    }
}

/// The governed wire tag naming one value-domain provenance.
#[must_use]
pub const fn value_domain_provenance_tag(provenance: ValueDomainProvenance) -> u8 {
    match provenance {
        ValueDomainProvenance::CompilerProven => 0x01,
        ValueDomainProvenance::RuntimeValidated => 0x02,
        ValueDomainProvenance::CallerDeclaredUnvalidated => 0x03,
    }
}

/// Resolves a governed value-domain-provenance tag, or `None` for an
/// unrecognized one.
#[must_use]
pub const fn value_domain_provenance_from_tag(tag: u8) -> Option<ValueDomainProvenance> {
    match tag {
        0x01 => Some(ValueDomainProvenance::CompilerProven),
        0x02 => Some(ValueDomainProvenance::RuntimeValidated),
        0x03 => Some(ValueDomainProvenance::CallerDeclaredUnvalidated),
        _ => None,
    }
}

/// The governed wire tag naming one materialization rounding.
#[must_use]
pub const fn materialization_rounding_tag(rounding: MaterializationRounding) -> u8 {
    match rounding {
        MaterializationRounding::NearestTiesToEven => 0x01,
    }
}

/// Resolves a governed materialization-rounding tag, or `None` for an
/// unrecognized one.
#[must_use]
pub const fn materialization_rounding_from_tag(tag: u8) -> Option<MaterializationRounding> {
    match tag {
        0x01 => Some(MaterializationRounding::NearestTiesToEven),
        _ => None,
    }
}

/// A locally decidable rejection of one scalar-arithmetic policy subject.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ScalarArithmeticSubjectError {
    /// The value type is not a registered governed scalar, states no format
    /// class and width, or disagrees with the arithmetic type's own registered
    /// format.
    UnvalidatedScalarArithmetic,
}

impl fmt::Display for ScalarArithmeticSubjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnvalidatedScalarArithmetic => {
                formatter.write_str("unvalidated-scalar-arithmetic")
            }
        }
    }
}

impl Error for ScalarArithmeticSubjectError {}

/// One scalar-arithmetic policy subject: an arithmetic type paired with the
/// complete resolved semantic value type it computes in.
///
/// [`crate::semantic::TypeKey`] alone is insufficient and the reason is
/// structural rather than stylistic: [`ResolvedValueType`] has three families —
/// nominal, parameterized, and encoded-numeric — and two resolved types within
/// one definition family are distinguished only by their parameters or their
/// ordered encoded components. A subject keyed by the nominal spelling would
/// merge them.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArithmeticSubject {
    arithmetic: ArithmeticType,
    resolved_type: ResolvedValueType,
}

impl ScalarArithmeticSubject {
    /// Pairs one arithmetic type with the semantic value type it computes in.
    ///
    /// The association is proven from the governed built-in scalar catalog, not
    /// from the spelling of either argument: a similar-looking name is not
    /// evidence that an arithmetic type's semantics were ever defined over a
    /// value identity. [`ArithmeticType::canonical_type_key`] names the durable
    /// dtype identity of the arithmetic type, the catalog states which
    /// identities it registers and what format each one is, and this constructor
    /// admits the pair only when the value type's registered descriptor states
    /// the same format class and the same width as the arithmetic type's own
    /// registered descriptor does.
    ///
    /// Constructing a subject is not declaring a fact about it. A profile that
    /// declares no row for the subject leaves every dimension `Unknown` for it,
    /// exactly as it does for a dimension it never mentions.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic`]
    /// when the value type is not a registered governed scalar, when its
    /// descriptor states no format class and width — a logical predicate
    /// identity states a value cardinality instead, and has no format an
    /// arithmetic subject could match — or when either disagrees with the
    /// arithmetic type's own registered format. Disagreement is the case a pair
    /// of similar formats falls into: `tiler::u32@1` shares `f32`'s width and
    /// differs in class, while `tiler::f16@1` shares `f32`'s class and differs
    /// in width.
    pub fn new(
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
    ) -> Result<Self, ScalarArithmeticSubjectError> {
        // Both sides are read from the one catalog, so neither a compiler nor a
        // caller is the authority for what a format is. The arithmetic type's
        // own lookup failing is not reachable through any argument — every
        // variant names a registered identity, which a test in this crate pins —
        // and it stays a refusal rather than a panic so that a catalog and an
        // arithmetic vocabulary which have drifted apart refuse a subject
        // instead of admitting one no registry describes.
        let arithmetic_facts = registered_arithmetic_facts(arithmetic)
            .ok_or(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic)?;
        let subject_facts = builtin_scalar_value_type_facts(&resolved_type)
            .ok_or(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic)?;
        let arithmetic_format = registered_scalar_format(&arithmetic_facts)
            .ok_or(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic)?;
        let subject_format = registered_scalar_format(&subject_facts)
            .ok_or(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic)?;
        if subject_format != arithmetic_format {
            return Err(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic);
        }
        Ok(Self {
            arithmetic,
            resolved_type,
        })
    }

    /// The governed `tiler::f32@1` scalar-arithmetic subject.
    ///
    /// Kept beside [`Self::new`] because this pair is named at more call sites
    /// than every other combined and cannot fail, so those sites carry no
    /// unreachable error path.
    ///
    /// # Panics
    ///
    /// Panics only if the governed catalog stops registering `tiler::f32@1`,
    /// which a test in this crate pins.
    #[must_use]
    pub fn f32() -> Self {
        Self::new(ArithmeticType::F32, crate::semantic::F32::resolved_type())
            .expect("the governed F32 arithmetic subject is registered")
    }

    /// The arithmetic type this subject computes in.
    #[must_use]
    pub const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// The complete resolved semantic value type.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Appends this subject's canonical identity bytes.
    ///
    /// The full resolved-type canonical encoding is length-framed rather than
    /// summarized, which is what keeps a nominal `tiler::f32@1` subject, a
    /// parameterized one, and an encoded-numeric one three distinct identities.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.arithmetic.tag());
        push_slice(bytes, self.resolved_type.canonical_encoding().as_bytes());
    }

    /// This subject's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }

    /// Projects this subject into the identity a serialized record carries.
    #[must_use]
    pub fn identity(&self) -> ScalarArithmeticSubjectIdentity {
        ScalarArithmeticSubjectIdentity {
            arithmetic: self.arithmetic,
            resolved_type: self
                .resolved_type
                .canonical_encoding()
                .as_bytes()
                .to_vec()
                .into_boxed_slice(),
        }
    }
}

/// Returns the registered descriptor of the identity `arithmetic` names.
///
/// The arithmetic vocabulary states a durable dtype spelling and the built-in
/// scalar catalog states which spellings it registers and describes. Resolving
/// one through the other is what keeps this vocabulary from carrying a second
/// copy of a format's parameters: a copy is a second place for the format to be
/// wrong, and the copy is what a caller's pair would then be checked against.
///
/// `None` when no catalog row carries that spelling.
#[must_use]
pub fn registered_arithmetic_facts(arithmetic: ArithmeticType) -> Option<CanonicalValue> {
    builtin_scalar_value_types()
        .into_iter()
        .find(|value| {
            value
                .nominal_key()
                .is_some_and(|key| key.to_string() == arithmetic.canonical_type_key())
        })
        .as_ref()
        .and_then(builtin_scalar_value_type_facts)
}

/// Returns the registered value identity `arithmetic` names.
///
/// The complement of [`registered_arithmetic_facts`], reading the same catalog
/// row for its *key* rather than its descriptor. Every arithmetic type resolves
/// here — the vocabulary and the catalog are pinned to each other by a test in
/// this crate — so a numerical requirement can always be stated for the exact
/// value identity its width computes over, including the widths this build
/// registers no contract key for.
///
/// That totality is what a requirement needs and what a *contract* deliberately
/// does not get: a subject a profile can be asked about is not a contract a
/// caller may state.
///
/// `None` when no catalog row carries that spelling, which would mean the
/// arithmetic vocabulary and the catalog had drifted apart.
#[must_use]
pub fn registered_arithmetic_value_type(arithmetic: ArithmeticType) -> Option<ResolvedValueType> {
    builtin_scalar_value_types().into_iter().find_map(|value| {
        value
            .nominal_key()
            .filter(|key| key.to_string() == arithmetic.canonical_type_key())
            .map(|key| ResolvedValueType::nominal(key.clone()))
    })
}

/// Returns the format class and stated width of one registered scalar
/// descriptor.
///
/// `None` when the descriptor states neither, which is a real answer rather than
/// a malformed one: a logical-predicate row states a value cardinality and no
/// width at all, so there is no format for an arithmetic subject to agree with.
#[must_use]
pub fn registered_scalar_format(facts: &CanonicalValue) -> Option<(&str, u64)> {
    let CanonicalValueView::Record(fields) = facts.view() else {
        return None;
    };
    let field = |id| {
        fields
            .iter()
            .find(|field| field.id() == id)
            .map(CanonicalField::value)
    };
    let CanonicalValueView::Utf8(class) = field(SCALAR_TYPE_FACT_CLASS)?.view() else {
        return None;
    };
    let CanonicalValueView::Unsigned { bits, .. } = field(SCALAR_TYPE_FACT_WIDTH_BITS)?.view()
    else {
        return None;
    };
    Some((class, bits))
}

/// Maximum byte length of one carried resolved-type canonical identity.
///
/// [`crate::semantic`]'s own bound governs the *payload* a resolved type may
/// carry; this bounds the framed canonical encoding a serialized record admits,
/// and it belongs beside the reader that must refuse the rest.
pub const MAX_RESOLVED_TYPE_IDENTITY_BYTES: usize = 64 * 1_024;

/// The serialized identity of one scalar-arithmetic policy subject.
///
/// # Why a record carries bytes rather than a reconstructed type
///
/// [`ResolvedValueType::canonical_encoding`] is **one-way**: this crate
/// publishes the collision-free encoder and no decoder, and the accepted policy
/// behind that is `own-the-numerical-realization-profile-key`'s — decoding
/// yields a *dispatch record* rather than reconstructed compiler IR, so nothing
/// converts one back.
///
/// That is not a limitation to work around, it is the right shape. The exact
/// canonical bytes **are** the full resolved-type identity: they are collision
/// free by construction and their leading family discriminant distinguishes a
/// nominal, a parameterized, and an encoded-numeric type, so a record
/// distinguishes all three without claiming any of them inhabits the
/// scalar-arithmetic schema.
///
/// The arithmetic type is carried as a decodable tag beside them because a
/// consumer must be able to read *which dtype* a record speaks for; that is the
/// whole point of the dtype key, and a wholly opaque subject would reinstate the
/// defect the delivered-realization record exists to correct.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArithmeticSubjectIdentity {
    arithmetic: ArithmeticType,
    resolved_type: Box<[u8]>,
}

impl ScalarArithmeticSubjectIdentity {
    /// Wraps canonical resolved-type identity bytes minted by this crate.
    ///
    /// Returns `None` for empty bytes or bytes beyond
    /// [`MAX_RESOLVED_TYPE_IDENTITY_BYTES`].
    #[must_use]
    pub fn from_parts(arithmetic: ArithmeticType, resolved_type: &[u8]) -> Option<Self> {
        if resolved_type.is_empty() || resolved_type.len() > MAX_RESOLVED_TYPE_IDENTITY_BYTES {
            return None;
        }
        Some(Self {
            arithmetic,
            resolved_type: resolved_type.into(),
        })
    }

    /// The arithmetic type this subject computes in.
    #[must_use]
    pub const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// The exact canonical resolved-type identity bytes.
    #[must_use]
    pub fn resolved_type_identity(&self) -> &[u8] {
        &self.resolved_type
    }

    /// Appends this identity's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.arithmetic.tag());
        push_slice(bytes, &self.resolved_type);
    }

    /// This identity's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

/// A behaviour the caller's contract must already state for a conditional means.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RelaxationRequirement {
    subject: ScalarArithmeticSubjectIdentity,
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
}

impl RelaxationRequirement {
    /// Names the subject, dimension, and behaviour a caller must already have
    /// authorized.
    ///
    /// The subject is carried as its serialized identity rather than the rich
    /// [`ScalarArithmeticSubject`], because a relaxation payload has to survive
    /// the codec: a delivered-realization record must be able to state *which*
    /// relaxation made a requirement honourable after a round trip, and only the
    /// identity form decodes.
    #[must_use]
    pub const fn new(
        subject: ScalarArithmeticSubjectIdentity,
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
    ) -> Self {
        Self {
            subject,
            dimension,
            behaviour,
        }
    }

    /// The subject the authorization must be stated for.
    #[must_use]
    pub const fn subject(&self) -> &ScalarArithmeticSubjectIdentity {
        &self.subject
    }

    /// The dimension the authorization must be stated on.
    #[must_use]
    pub const fn dimension(&self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour that dimension must already be resolved to.
    #[must_use]
    pub const fn behaviour(&self) -> DimensionBehaviour {
        self.behaviour
    }

    /// Appends this requirement's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            subject,
            dimension,
            behaviour,
        } = self;
        subject.encode(bytes);
        bytes.push(dimension.tag());
        behaviour.encode(bytes);
    }
}

/// The four means by which a target may honour a required behaviour.
///
/// This is the vocabulary `docs/numerical-semantics.md` already names under
/// "Backend numerical feasibility"; no term is invented here. The distinction
/// that forces a separate authority from the capability axes is that
/// [`Self::SupportedWithExactEmulation`] is honoured by *emitting different
/// operations*, which a bound comparison cannot express.
///
/// # Presentation and identity are separate
///
/// [`Self::label`] is the presentation string and is documented as **not**
/// injective: two conditional means differing only in their relaxation return
/// the same string. [`Self::encode`] is the identity, and it carries the
/// relaxation payload. Nothing compares, encodes, or keys on the label — ADR
/// 0074 convention 2 draws exactly this line, and ADR 0076 records why a record
/// that carried the label instead could not answer the question it exists to
/// make answerable.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HonouringMeans {
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
    /// The presentation string naming this means.
    ///
    /// **Presentation only, and deliberately not injective.** Two conditional
    /// means differing in their relaxation return the same string. Nothing
    /// compares, encodes, or keys on this value; [`Self::encode`] is the
    /// identity and [`Self::canonical_key`] is the comparable form.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::SupportedExactly => "supported-exactly",
            Self::SupportedWithExactEmulation => "supported-with-exact-emulation",
            Self::SupportedOnlyUnderDeclaredRelaxation { .. } => {
                "supported-only-under-declared-relaxation"
            }
            Self::Unsupported => "unsupported",
        }
    }

    /// The governed wire tag naming this means.
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::SupportedExactly => 0x01,
            Self::SupportedWithExactEmulation => 0x02,
            Self::SupportedOnlyUnderDeclaredRelaxation { .. } => 0x03,
            Self::Unsupported => 0x04,
        }
    }

    /// Appends this means's complete canonical bytes, relaxation payload
    /// included.
    ///
    /// The conditional arm carries its relaxation, because two profiles whose
    /// declarations differ only in *which* relaxation they require admit
    /// different requests and must not share a descriptor.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        if let Self::SupportedOnlyUnderDeclaredRelaxation { relaxation } = self {
            relaxation.encode(bytes);
        }
    }

    /// This means's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

/// The policy position within one program occurrence that produced a
/// requirement.
///
/// ADR 0011's per-operation restrictions attach to a position, not to a dtype:
/// one `f32` operation's accumulator and its observable result can carry
/// different legal requirements, and a record keyed by type alone would collapse
/// them into whichever was written last.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PolicyLocus {
    /// An operand read before the operation applies.
    Input,
    /// The operation's own arithmetic.
    Computation,
    /// A reduction or fold accumulator.
    Accumulator,
    /// The operation's produced value.
    Result,
    /// One ordered component of a compound encoded value.
    Component,
    /// An observable materialization boundary.
    Materialization,
}

impl PolicyLocus {
    /// The governed wire tag naming this locus.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Input => 0x01,
            Self::Computation => 0x02,
            Self::Accumulator => 0x03,
            Self::Result => 0x04,
            Self::Component => 0x05,
            Self::Materialization => 0x06,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized locus.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Input),
            0x02 => Some(Self::Computation),
            0x03 => Some(Self::Accumulator),
            0x04 => Some(Self::Result),
            0x05 => Some(Self::Component),
            0x06 => Some(Self::Materialization),
            _ => None,
        }
    }

    /// The governed canonical key naming this locus in an explanation.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Input => "locus.input",
            Self::Computation => "locus.computation",
            Self::Accumulator => "locus.accumulator",
            Self::Result => "locus.result",
            Self::Component => "locus.component",
            Self::Materialization => "locus.materialization",
        }
    }
}

/// The canonical key identifying where in the program one obligation arose.
///
/// The occurrence is [`SemanticOccurrence`], the same graph-local ordinal
/// [`crate::program::CoveredOccurrence`] uses, so the obligation and the stage
/// coverage that implements it name the position the same way. The `component`
/// ordinal is present only for [`PolicyLocus::Component`] and is otherwise zero,
/// which a producer enforces rather than leaving to a convention.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NumericalObligationKey {
    occurrence: SemanticOccurrence,
    locus: PolicyLocus,
    component: u32,
}

impl NumericalObligationKey {
    /// The encoded width of one obligation key: occurrence, locus tag, ordinal.
    ///
    /// Stated so a reader computing an offset into an encoded row reads the
    /// width from the type that defines it rather than restating nine.
    pub const ENCODED_BYTES: usize = 4 + 1 + 4;

    /// Names one non-component policy locus of one program occurrence.
    #[must_use]
    pub const fn new(occurrence: SemanticOccurrence, locus: PolicyLocus) -> Self {
        Self {
            occurrence,
            locus,
            component: 0,
        }
    }

    /// Names one ordered component of one program occurrence's compound value.
    #[must_use]
    pub const fn component(occurrence: SemanticOccurrence, component: u32) -> Self {
        Self {
            occurrence,
            locus: PolicyLocus::Component,
            component,
        }
    }

    /// The graph-local occurrence this obligation arose at.
    #[must_use]
    pub const fn occurrence(self) -> SemanticOccurrence {
        self.occurrence
    }

    /// The policy position within that occurrence.
    #[must_use]
    pub const fn locus(self) -> PolicyLocus {
        self.locus
    }

    /// The ordered component ordinal, zero for every non-component locus.
    #[must_use]
    pub const fn component_ordinal(self) -> u32 {
        self.component
    }

    /// Whether the component ordinal agrees with the locus.
    ///
    /// A non-component locus carrying a nonzero ordinal is malformed rather than
    /// a verdict: the ordinal would enter the canonical key and make two
    /// obligations at one position two rows.
    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        matches!(self.locus, PolicyLocus::Component) || self.component == 0
    }

    /// Appends this key's canonical bytes.
    pub fn encode(self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.occurrence.get().to_be_bytes());
        bytes.push(self.locus.tag());
        bytes.extend_from_slice(&self.component.to_be_bytes());
    }

    /// This key's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_key(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

impl fmt::Display for NumericalObligationKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}@{}#{}",
            self.locus.key(),
            self.occurrence.get(),
            self.component
        )
    }
}

// ---------------------------------------------------------------------------
// Structured fact provenance
// ---------------------------------------------------------------------------

/// The class of authority vouching for one numerical fact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactAuthority {
    /// A governed, conservative compile-time profile guarantee.
    GovernedProfile,
    /// A named external producer's normative target-family declaration.
    ///
    /// Available at the compile-profile phase but not a compiler proof. Its
    /// source record carries both the producer identity and the versioned
    /// specification or guarantee it relies on.
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
    /// The governed wire tag naming this authority.
    ///
    /// **The tags are deliberately not in declaration order, and every reader
    /// and writer must preserve them byte for byte.** `ExternalProfile` and
    /// `MeasuredProfile` were inserted after `0x02`–`0x05` had already been
    /// committed to every target-profile descriptor, so they carry `0x06` and
    /// `0x07`. Renumbering them into declaration order would silently change
    /// every profile descriptor that declares a measured fact — an
    /// identity-domain step, and one that would be invisible in a diff that only
    /// looked tidier.
    #[must_use]
    pub const fn tag(self) -> u8 {
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

    /// Resolves a governed wire tag, or `None` for an unrecognized authority.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::GovernedProfile),
            0x06 => Some(Self::ExternalProfile),
            0x07 => Some(Self::MeasuredProfile),
            0x02 => Some(Self::ArtifactEvidence),
            0x03 => Some(Self::DeviceRuntime),
            0x04 => Some(Self::PreparedKernel),
            0x05 => Some(Self::LaunchInstance),
            _ => None,
        }
    }

    /// The governed canonical key naming this authority in an explanation.
    ///
    /// Exhaustive for the same reason as [`Self::tag`]: a rejection that cannot
    /// name the authority vouching for the refusing fact is not explainable.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::GovernedProfile => "governed-profile",
            Self::ExternalProfile => "external-profile",
            Self::MeasuredProfile => "measured-profile",
            Self::ArtifactEvidence => "artifact-evidence",
            Self::DeviceRuntime => "device-runtime",
            Self::PreparedKernel => "prepared-kernel",
            Self::LaunchInstance => "launch-instance",
        }
    }
}

/// The scope over which one numerical fact remains valid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactValidityScope {
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
    /// The governed wire tag naming this scope.
    ///
    /// `MeasuredEnvironment` carries `0x05` for the same reason
    /// [`FactAuthority::tag`] records: it was inserted second in declaration
    /// order after `0x02`–`0x04` were already committed. The declaration order
    /// and the tag order are preserved exactly.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::PortableProfile => 0x01,
            Self::MeasuredEnvironment => 0x05,
            Self::DeviceInstance => 0x02,
            Self::PreparedArtifact => 0x03,
            Self::LaunchInstance => 0x04,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized scope.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::PortableProfile),
            0x05 => Some(Self::MeasuredEnvironment),
            0x02 => Some(Self::DeviceInstance),
            0x03 => Some(Self::PreparedArtifact),
            0x04 => Some(Self::LaunchInstance),
            _ => None,
        }
    }

    /// The governed canonical key naming this scope in an explanation.
    ///
    /// Exhaustive for the same reason as [`Self::tag`]: a refusal whose validity
    /// scope is unnamed cannot be acted on, because a reader cannot tell a
    /// portable claim from one true of one measured population.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::PortableProfile => "portable-profile",
            Self::MeasuredEnvironment => "measured-environment",
            Self::DeviceInstance => "device-instance",
            Self::PreparedArtifact => "prepared-artifact",
            Self::LaunchInstance => "launch-instance",
        }
    }
}

/// Maximum UTF-8 byte length of one descriptive provenance field.
pub const MAX_PROVENANCE_TEXT_BYTES: usize = 256;
/// Maximum compiler builds admitted in one measurement context.
pub const MAX_COMPILER_BUILDS_PER_CONTEXT: usize = 16;
/// Maximum measurement contexts admitted by one evidence row.
pub const MAX_MEASUREMENT_CONTEXTS_PER_SOURCE: usize = 64;

/// A versioned identity naming an authority or governed guarantee.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProvenanceIdentity {
    key: String,
    revision: u32,
}

impl ProvenanceIdentity {
    /// Constructs a versioned identity. Validation happens at the record
    /// boundary, so a malformed one is rejected where it is read rather than
    /// where it is spelled.
    #[must_use]
    pub fn new(key: impl Into<String>, revision: u32) -> Self {
        Self {
            key: key.into(),
            revision,
        }
    }

    /// Whether this identity is well formed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        valid_key(&self.key) && self.revision != 0
    }

    /// The governed key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The nonzero revision.
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    /// Appends this identity's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        // Destructured rather than field-accessed, here and in every encoder
        // below: a field added to a provenance record is then a build error at
        // the encoder that would otherwise have silently omitted it, which is
        // the only thing keeping a rejection's canonical identity complete.
        let Self { key, revision } = self;
        push_slice(bytes, key.as_bytes());
        bytes.extend_from_slice(&revision.to_be_bytes());
    }

    /// Renders this identity into an explanation.
    pub fn render(&self, output: &mut String) {
        use std::fmt::Write as _;
        let Self { key, revision } = self;
        let _ = write!(output, "{key}@{revision}");
    }
}

/// The role one compiler build performed in a measured execution.
///
/// Roles are semantic pipeline positions rather than vendor executable names:
/// several roles may resolve to one binary, and one vendor may split a role
/// across binaries without changing what the record says that build did.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompilerBuildRole {
    /// The compiler frontend.
    Frontend,
    /// The optimizer.
    Optimizer,
    /// An intermediate translator.
    IntermediateTranslator,
    /// The code generator.
    CodeGenerator,
    /// The assembler.
    Assembler,
    /// The linker.
    Linker,
    /// A runtime compiler.
    RuntimeCompiler,
    /// A versioned provider-defined role not yet in the governed common set.
    ProviderDefined(ProvenanceIdentity),
}

impl CompilerBuildRole {
    /// Whether this role is well formed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        match self {
            Self::ProviderDefined(identity) => identity.is_valid(),
            Self::Frontend
            | Self::Optimizer
            | Self::IntermediateTranslator
            | Self::CodeGenerator
            | Self::Assembler
            | Self::Linker
            | Self::RuntimeCompiler => true,
        }
    }

    /// The governed wire tag naming this role.
    ///
    /// Deliberately not in declaration order: `IntermediateTranslator` carries
    /// `0x07` because it was inserted after `0x01`–`0x06` had been committed.
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::Frontend => 0x01,
            Self::Optimizer => 0x02,
            Self::CodeGenerator => 0x03,
            Self::Assembler => 0x04,
            Self::Linker => 0x05,
            Self::RuntimeCompiler => 0x06,
            Self::IntermediateTranslator => 0x07,
            Self::ProviderDefined(_) => 0xff,
        }
    }

    /// Appends this role's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        if let Self::ProviderDefined(identity) = self {
            identity.encode(bytes);
        }
    }

    /// Renders this role into an explanation.
    pub fn render(&self, output: &mut String) {
        match self {
            Self::Frontend => output.push_str("frontend"),
            Self::Optimizer => output.push_str("optimizer"),
            Self::CodeGenerator => output.push_str("code-generator"),
            Self::Assembler => output.push_str("assembler"),
            Self::Linker => output.push_str("linker"),
            Self::RuntimeCompiler => output.push_str("runtime-compiler"),
            Self::IntermediateTranslator => output.push_str("intermediate-translator"),
            Self::ProviderDefined(identity) => {
                output.push_str("provider-defined:");
                identity.render(output);
            }
        }
    }
}

/// One compiler component build participating in a measured fact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompilerBuildIdentity {
    role: CompilerBuildRole,
    implementation: String,
    version: String,
    build: Option<String>,
}

impl CompilerBuildIdentity {
    /// Names one compiler build by role, implementation, version, and build.
    #[must_use]
    pub fn new(
        role: CompilerBuildRole,
        implementation: impl Into<String>,
        version: impl Into<String>,
        build: Option<String>,
    ) -> Self {
        Self {
            role,
            implementation: implementation.into(),
            version: version.into(),
            build,
        }
    }

    /// Whether this build identity is well formed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.role.is_valid()
            && valid_key(&self.implementation)
            && valid_text(&self.version)
            && self.build.as_deref().is_none_or(valid_text)
    }

    /// The role this build performed.
    #[must_use]
    pub const fn role(&self) -> &CompilerBuildRole {
        &self.role
    }

    /// The implementation key.
    #[must_use]
    pub fn implementation(&self) -> &str {
        &self.implementation
    }

    /// The version string.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The optional build string.
    #[must_use]
    pub fn build(&self) -> Option<&str> {
        self.build.as_deref()
    }

    /// Appends this build identity's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            role,
            implementation,
            version,
            build,
        } = self;
        role.encode(bytes);
        push_slice(bytes, implementation.as_bytes());
        push_slice(bytes, version.as_bytes());
        bytes.push(u8::from(build.is_some()));
        if let Some(build) = build {
            push_slice(bytes, build.as_bytes());
        }
    }

    /// Renders this build identity into an explanation.
    pub fn render(&self, output: &mut String) {
        use std::fmt::Write as _;
        let Self {
            role,
            implementation,
            version,
            build,
        } = self;
        role.render(output);
        let _ = write!(output, "={implementation}@{version}");
        if let Some(build) = build {
            let _ = write!(output, "+{build}");
        }
    }

    /// This build identity's canonical bytes, as a comparable key.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

/// The execution environment on which one numerical behaviour was measured.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExecutionEnvironmentIdentity {
    platform: String,
    platform_version: String,
    platform_build: String,
    architecture: String,
    hardware: String,
}

impl ExecutionEnvironmentIdentity {
    /// Names one execution environment.
    #[must_use]
    pub fn new(
        platform: impl Into<String>,
        platform_version: impl Into<String>,
        platform_build: impl Into<String>,
        architecture: impl Into<String>,
        hardware: impl Into<String>,
    ) -> Self {
        Self {
            platform: platform.into(),
            platform_version: platform_version.into(),
            platform_build: platform_build.into(),
            architecture: architecture.into(),
            hardware: hardware.into(),
        }
    }

    /// Whether this environment identity is well formed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        valid_key(&self.platform)
            && valid_text(&self.platform_version)
            && valid_text(&self.platform_build)
            && valid_key(&self.architecture)
            && valid_text(&self.hardware)
    }

    /// The platform key.
    #[must_use]
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// The platform version.
    #[must_use]
    pub fn platform_version(&self) -> &str {
        &self.platform_version
    }

    /// The platform build.
    #[must_use]
    pub fn platform_build(&self) -> &str {
        &self.platform_build
    }

    /// The architecture key.
    #[must_use]
    pub fn architecture(&self) -> &str {
        &self.architecture
    }

    /// The hardware description.
    #[must_use]
    pub fn hardware(&self) -> &str {
        &self.hardware
    }

    /// Appends this environment identity's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            platform,
            platform_version,
            platform_build,
            architecture,
            hardware,
        } = self;
        for field in [
            platform,
            platform_version,
            platform_build,
            architecture,
            hardware,
        ] {
            push_slice(bytes, field.as_bytes());
        }
    }

    /// Renders this environment identity into an explanation.
    pub fn render(&self, output: &mut String) {
        use std::fmt::Write as _;
        let Self {
            platform,
            platform_version,
            platform_build,
            architecture,
            hardware,
        } = self;
        let _ = write!(
            output,
            "{platform}/{platform_version}/{platform_build}/{architecture}/{hardware}"
        );
    }
}

/// One measured compiler-build set paired with the environment it executed in.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MeasurementContext {
    compiler_builds: Vec<CompilerBuildIdentity>,
    environment: ExecutionEnvironmentIdentity,
}

impl MeasurementContext {
    /// Pairs a compiler-build set with its execution environment, canonicalizing
    /// the build order.
    #[must_use]
    pub fn new(
        mut compiler_builds: Vec<CompilerBuildIdentity>,
        environment: ExecutionEnvironmentIdentity,
    ) -> Self {
        compiler_builds.sort_by_key(CompilerBuildIdentity::canonical_bytes);
        Self {
            compiler_builds,
            environment,
        }
    }

    /// Whether this context is well formed and complete.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.compiler_builds.is_empty()
            && self.compiler_builds.len() <= MAX_COMPILER_BUILDS_PER_CONTEXT
            && self
                .compiler_builds
                .iter()
                .all(CompilerBuildIdentity::is_valid)
            && strictly_increasing(
                &self.compiler_builds,
                CompilerBuildIdentity::canonical_bytes,
            )
            && self.environment.is_valid()
    }

    /// The canonically ordered compiler builds.
    #[must_use]
    pub fn compiler_builds(&self) -> &[CompilerBuildIdentity] {
        &self.compiler_builds
    }

    /// The execution environment.
    #[must_use]
    pub const fn environment(&self) -> &ExecutionEnvironmentIdentity {
        &self.environment
    }

    /// Appends this context's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            compiler_builds,
            environment,
        } = self;
        push_len(bytes, compiler_builds.len());
        for build in compiler_builds {
            build.encode(bytes);
        }
        environment.encode(bytes);
    }

    /// Renders this context into an explanation.
    pub fn render(&self, output: &mut String) {
        let Self {
            compiler_builds,
            environment,
        } = self;
        output.push_str("env=");
        environment.render(output);
        output.push_str(";builds=");
        for (index, build) in compiler_builds.iter().enumerate() {
            if index != 0 {
                output.push(',');
            }
            build.render(output);
        }
    }

    /// This context's canonical bytes, as a comparable key.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

/// Why the authority may make the fact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactEvidenceBasis {
    /// A normative guarantee, not an empirical claim.
    GovernedGuarantee {
        /// The versioned guarantee cited.
        guarantee: ProvenanceIdentity,
    },
    /// A normative or specification-backed guarantee attributed to an external
    /// target-profile producer.
    ExternalGuarantee {
        /// The versioned external reference cited.
        reference: ProvenanceIdentity,
    },
    /// One or more exact, independently readable measurement contexts.
    Measurement {
        /// The canonically ordered measurement contexts.
        contexts: Vec<MeasurementContext>,
    },
}

impl FactEvidenceBasis {
    /// The governed wire tag naming this basis.
    ///
    /// Deliberately not in declaration order: `ExternalGuarantee` carries `0x03`
    /// because it was inserted after `0x01` and `0x02` had been committed to
    /// every target-profile descriptor.
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::GovernedGuarantee { .. } => 0x01,
            Self::Measurement { .. } => 0x02,
            Self::ExternalGuarantee { .. } => 0x03,
        }
    }

    /// Appends this basis's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::GovernedGuarantee { guarantee } => guarantee.encode(bytes),
            Self::ExternalGuarantee { reference } => reference.encode(bytes),
            Self::Measurement { contexts } => {
                push_len(bytes, contexts.len());
                for context in contexts {
                    context.encode(bytes);
                }
            }
        }
    }
}

/// Version of the structured numerical-fact provenance vocabulary.
pub const FACT_SOURCE_PROVENANCE_SCHEMA_VERSION: u32 = 3;

/// Structured, versioned provenance shared by numerical evidence rows.
///
/// It carries every field ADR 0076 item 3 requires and item 4 inherits:
/// availability phase, measured-fact authority, validity scope, versioned
/// authority identity, and either a cited guarantee or the exact compiler builds
/// and execution environments the behaviour was measured on.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FactSourceProvenance {
    schema_version: u32,
    phase: AvailabilityPhase,
    authority: FactAuthority,
    validity: FactValidityScope,
    authority_identity: ProvenanceIdentity,
    basis: FactEvidenceBasis,
}

impl FactSourceProvenance {
    /// Assembles one provenance statement, canonicalizing measurement order.
    #[must_use]
    pub fn new(
        phase: AvailabilityPhase,
        authority: FactAuthority,
        validity: FactValidityScope,
        authority_identity: ProvenanceIdentity,
        basis: FactEvidenceBasis,
    ) -> Self {
        let basis = match basis {
            FactEvidenceBasis::Measurement { mut contexts } => {
                contexts.sort_by_key(MeasurementContext::canonical_bytes);
                FactEvidenceBasis::Measurement { contexts }
            }
            other => other,
        };
        Self {
            schema_version: FACT_SOURCE_PROVENANCE_SCHEMA_VERSION,
            phase,
            authority,
            validity,
            authority_identity,
            basis,
        }
    }

    /// A compile-profile governed-guarantee statement.
    #[must_use]
    pub fn governed(authority_identity: ProvenanceIdentity, guarantee: ProvenanceIdentity) -> Self {
        Self::new(
            AvailabilityPhase::CompileProfile,
            FactAuthority::GovernedProfile,
            FactValidityScope::PortableProfile,
            authority_identity,
            FactEvidenceBasis::GovernedGuarantee { guarantee },
        )
    }

    /// A compile-profile externally guaranteed statement.
    #[must_use]
    pub fn externally_guaranteed(
        authority_identity: ProvenanceIdentity,
        reference: ProvenanceIdentity,
    ) -> Self {
        Self::new(
            AvailabilityPhase::CompileProfile,
            FactAuthority::ExternalProfile,
            FactValidityScope::PortableProfile,
            authority_identity,
            FactEvidenceBasis::ExternalGuarantee { reference },
        )
    }

    /// A measured statement over one or more exact contexts.
    #[must_use]
    pub fn measured(
        phase: AvailabilityPhase,
        authority: FactAuthority,
        validity: FactValidityScope,
        authority_identity: ProvenanceIdentity,
        contexts: Vec<MeasurementContext>,
    ) -> Self {
        Self::new(
            phase,
            authority,
            validity,
            authority_identity,
            FactEvidenceBasis::Measurement { contexts },
        )
    }

    /// Whether this provenance statement is complete and internally consistent.
    ///
    /// The phase/authority/validity triple is checked against a closed table
    /// rather than each field independently, because the three coordinates are
    /// one claim: a `CompileProfile` fact vouched for by a `LaunchInstance`
    /// authority names no readable moment.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.schema_version == FACT_SOURCE_PROVENANCE_SCHEMA_VERSION
            && self.authority_identity.is_valid()
            && match &self.basis {
                FactEvidenceBasis::GovernedGuarantee { guarantee } => {
                    self.authority == FactAuthority::GovernedProfile && guarantee.is_valid()
                }
                FactEvidenceBasis::ExternalGuarantee { reference } => {
                    self.authority == FactAuthority::ExternalProfile && reference.is_valid()
                }
                FactEvidenceBasis::Measurement { contexts } => {
                    matches!(
                        (self.phase, self.authority, self.validity),
                        (
                            AvailabilityPhase::CompileProfile,
                            FactAuthority::MeasuredProfile,
                            FactValidityScope::MeasuredEnvironment,
                        ) | (
                            AvailabilityPhase::ArtifactEvidence,
                            FactAuthority::ArtifactEvidence,
                            FactValidityScope::PreparedArtifact,
                        ) | (
                            AvailabilityPhase::LiveDevicePreflight,
                            FactAuthority::DeviceRuntime,
                            FactValidityScope::DeviceInstance,
                        ) | (
                            AvailabilityPhase::PreparedKernelPreflight,
                            FactAuthority::PreparedKernel,
                            FactValidityScope::PreparedArtifact,
                        ) | (
                            AvailabilityPhase::LaunchPreflight,
                            FactAuthority::LaunchInstance,
                            FactValidityScope::LaunchInstance,
                        )
                    ) && !contexts.is_empty()
                        && contexts.len() <= MAX_MEASUREMENT_CONTEXTS_PER_SOURCE
                        && contexts.iter().all(MeasurementContext::is_valid)
                        && strictly_increasing(contexts, MeasurementContext::canonical_bytes)
                }
            }
    }

    /// The provenance schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// The phase from which this fact is available.
    #[must_use]
    pub const fn phase(&self) -> AvailabilityPhase {
        self.phase
    }

    /// The authority vouching for this fact.
    #[must_use]
    pub const fn authority(&self) -> FactAuthority {
        self.authority
    }

    /// The scope over which this fact remains valid.
    #[must_use]
    pub const fn validity(&self) -> FactValidityScope {
        self.validity
    }

    /// The versioned identity of the vouching authority.
    #[must_use]
    pub const fn authority_identity(&self) -> &ProvenanceIdentity {
        &self.authority_identity
    }

    /// The evidence basis.
    #[must_use]
    pub const fn basis(&self) -> &FactEvidenceBasis {
        &self.basis
    }

    /// Appends this statement's canonical bytes.
    pub fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            schema_version,
            phase,
            authority,
            validity,
            authority_identity,
            basis,
        } = self;
        bytes.extend_from_slice(&schema_version.to_be_bytes());
        bytes.push(phase.tag());
        bytes.push(authority.tag());
        bytes.push(validity.tag());
        authority_identity.encode(bytes);
        basis.encode(bytes);
    }

    /// Renders the complete source statement into an explanation.
    ///
    /// Every field the canonical encoding covers is spelled here too: a reader
    /// of the rendered trace and a reader of the identity bytes must be able to
    /// see the same claim, or the rendering is a summary of evidence rather than
    /// the evidence.
    pub fn render(&self, output: &mut String) {
        use std::fmt::Write as _;
        let Self {
            schema_version,
            phase,
            authority,
            validity,
            authority_identity,
            basis,
        } = self;
        let _ = write!(
            output,
            "source-schema={schema_version}:phase={}:authority={}:validity={}:authority-identity=",
            phase_key(*phase),
            authority.key(),
            validity.key()
        );
        authority_identity.render(output);
        match basis {
            FactEvidenceBasis::GovernedGuarantee { guarantee } => {
                output.push_str(":basis=governed-guarantee:");
                guarantee.render(output);
            }
            FactEvidenceBasis::ExternalGuarantee { reference } => {
                output.push_str(":basis=external-guarantee:");
                reference.render(output);
            }
            FactEvidenceBasis::Measurement { contexts } => {
                let _ = write!(output, ":basis=measurement:contexts={}", contexts.len());
                for context in contexts {
                    output.push_str(":[");
                    context.render(output);
                    output.push(']');
                }
            }
        }
    }

    /// This statement's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

/// The governed canonical key naming one availability phase in an explanation.
///
/// Written here rather than read from [`AvailabilityPhase::tag`] because a
/// rendered explanation names phases in words; exhaustive so a widened phase
/// vocabulary is a build error rather than an unnamed phase in a refusal.
#[must_use]
pub const fn phase_key(phase: AvailabilityPhase) -> &'static str {
    match phase {
        AvailabilityPhase::CompileProfile => "compile-profile",
        AvailabilityPhase::ArtifactEvidence => "artifact-evidence",
        AvailabilityPhase::LiveDevicePreflight => "live-device-preflight",
        AvailabilityPhase::PreparedKernelPreflight => "prepared-kernel-preflight",
        AvailabilityPhase::LaunchPreflight => "launch-preflight",
    }
}

fn valid_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROVENANCE_TEXT_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROVENANCE_TEXT_BYTES
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
}

fn strictly_increasing<T>(values: &[T], canonical_bytes: impl Fn(&T) -> Vec<u8>) -> bool {
    values
        .windows(2)
        .all(|pair| canonical_bytes(&pair[0]) < canonical_bytes(&pair[1]))
}

/// A compile-time witness that the dense arrays and the vocabulary agree.
const _: () = {
    assert!(CANONICAL_DIMENSIONS.len() == DIMENSION_COUNT);
    // Every dimension's dense index is its position in canonical order, which is
    // what lets one exhaustive match serve every dense array in the workspace.
    let mut index = 0;
    while index < DIMENSION_COUNT {
        assert!(CANONICAL_DIMENSIONS[index].index() == index);
        index += 1;
    }
};

/// A compile-time witness that no two dimension tags collide and each decodes
/// back to itself.
const _: () = {
    let mut left = 0;
    while left < DIMENSION_COUNT {
        let dimension = CANONICAL_DIMENSIONS[left];
        match NumericalDimension::from_tag(dimension.tag()) {
            Some(resolved) => assert!(resolved.tag() == dimension.tag()),
            None => panic!("every governed dimension tag resolves"),
        }
        let mut right = left + 1;
        while right < DIMENSION_COUNT {
            assert!(CANONICAL_DIMENSIONS[left].tag() != CANONICAL_DIMENSIONS[right].tag());
            right += 1;
        }
        left += 1;
    }
};

#[cfg(test)]
mod tests;
