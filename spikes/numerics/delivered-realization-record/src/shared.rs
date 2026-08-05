//! **Proposed `tiler_ir::numerics`** — the one shared scalar-arithmetic policy
//! vocabulary.
//!
//! Nothing here is production code. Every item models an exact proposed public
//! signature for the destination named in its own documentation, and the module
//! compiles against the real `tiler-ir` vocabularies so the model is checked
//! rather than transcribed.
//!
//! # Why this vocabulary is sited in `tiler-ir`
//!
//! `record-delivered-numerical-realization` eliminated relocating the means
//! vocabulary into `tiler-ir` and chose opaque compiler-minted key bytes. That
//! elimination recorded its own reopening trigger: "a consumer of the artifact
//! that must *reason over* the means rather than compare them". Two facts fire
//! it. First, ADR 0076 item 4 names exactly that consumer — one comparing
//! generated output against a CPU reference, which must know an emulated
//! dimension from a natively honoured one. Second, and decisively,
//! `HonouringMeans::key` is **not injective**: every
//! `SupportedOnlyUnderDeclaredRelaxation` value returns the same forty bytes
//! whatever relaxation it names, so the opaque-key mechanism cannot carry the
//! record even for comparison.
//!
//! The siting follows an existing precedent rather than inventing one.
//! [`AvailabilityPhase`] is ADR 0043 target-fact provenance vocabulary, it is
//! defined in `tiler_ir::program::abi`, and `tiler-compiler` and `tiler-artifact`
//! both name it by re-export. So does every behaviour vocabulary below:
//! `SubnormalMode`, `NumericalPermission`, `ApproximationEnvelope`,
//! `ExceptionalValueAssumption`, and `MaterializationRounding` are all
//! `tiler_ir::schedule` types today, and `DimensionBehaviour` is a sum over
//! exactly those five. The relocation therefore moves no meaning into the
//! semantic graph: `tiler_ir::numerics` is a contract-vocabulary module beside
//! `tiler_ir::schedule`, not inside `tiler_ir::semantic`, and the target-aware
//! *assessment* — which profile declares what, and how feasibility composes it —
//! stays entirely in `tiler_compiler::target`.
//!
//! This is a public-boundary proposal. It is not accepted;
//! `accept-the-delivered-realization-artifact-surface` owns Tom's ratification.

use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::SemanticOccurrence;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode, ValueDomainProvenance,
};
use tiler_ir::semantic::ResolvedValueType;

/// The number of governed scalar-arithmetic dimensions.
///
/// Exported so a dense per-dimension array is spelled once and a widened
/// vocabulary is a build error at every array literal rather than a silently
/// short one.
pub const DIMENSION_COUNT: usize = 11;

/// The behaviour space one numerical dimension ranges over.
///
/// Relocated verbatim from `tiler_compiler::target::honourability`. Naming the
/// space once is what keeps [`NumericalDimension::admits`] and
/// [`DimensionBehaviour`] from drifting as either vocabulary grows.
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
/// **This is the single authority.** The proposal deletes
/// `tiler_compiler::target::honourability::NumericalDimension` and
/// `tiler_artifact::program::realization::NumericalDimension` and re-exports this
/// type in their place, so the eleven-case set exists once and a twelfth case is
/// a build error at every total encoder and consumer in the workspace.
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
    /// Whether ordered reassociation of one same-operation operand sequence is
    /// permitted.
    Reassociation,
    /// Whether operand permutation — changing logical contributor order — is
    /// permitted.
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
    /// than derived from [`Self::tag`], following
    /// [`AvailabilityPhase::from_tag`]'s established shape: a reader handed a tag
    /// this build has never been taught rejects rather than approximating.
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
    /// One exhaustive shared match, which is what the ticket's "dense
    /// dimension-indexed arrays whose index conversion is one exhaustive shared
    /// match" names. Every dense array in the record family indexes through this
    /// and no other mapping, so a widened vocabulary cannot leave one array
    /// indexed by an old position.
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
    /// A row pairing a dimension with another space's behaviour is *malformed*
    /// rather than a verdict, which is why decode rejects it by name.
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
/// Relocated verbatim from `tiler_compiler::target::honourability`. Every arm's
/// payload is already a `tiler_ir::schedule` type, so this sum introduces no new
/// meaning into `tiler-ir` — it names a choice over five vocabularies that crate
/// already owns.
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

    /// Appends this behaviour's canonical bytes: its space, then its value.
    ///
    /// The space byte keeps two spaces' values from colliding once both widen.
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
                        bytes.push(match provenance {
                            ValueDomainProvenance::CompilerProven => 0x01,
                            ValueDomainProvenance::RuntimeValidated => 0x02,
                            ValueDomainProvenance::CallerDeclaredUnvalidated => 0x03,
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

    /// Reads one canonically encoded behaviour, or `None` for an unknown tag.
    ///
    /// Total in both directions, so a decoder and an encoder cannot disagree
    /// about what a behaviour *is*. Returns the behaviour and the bytes consumed.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match bytes {
            [0x01, value, ..] => Some((Self::Subnormals(subnormal_from_tag(*value)?), 2)),
            [0x02, value, ..] => Some((Self::Transform(permission_from_tag(*value)?), 2)),
            [0x03, 0x01, ..] => Some((Self::Approximation(ApproximationEnvelope::Forbidden), 2)),
            [0x03, 0x02, ..] => Some((
                Self::Approximation(ApproximationEnvelope::BackendElementary),
                2,
            )),
            [0x04, 0x01, ..] => Some((
                Self::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
                2,
            )),
            [0x04, 0x02, provenance, ..] => {
                let provenance = match provenance {
                    0x01 => ValueDomainProvenance::CompilerProven,
                    0x02 => ValueDomainProvenance::RuntimeValidated,
                    0x03 => ValueDomainProvenance::CallerDeclaredUnvalidated,
                    _ => return None,
                };
                Some((
                    Self::ExceptionalValue(ExceptionalValueAssumption::AssumeAbsent { provenance }),
                    3,
                ))
            }
            [0x05, 0x01, ..] => Some((
                Self::Rounding(MaterializationRounding::NearestTiesToEven),
                2,
            )),
            _ => None,
        }
    }

    /// This behaviour's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_key(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }
}

const fn subnormal_tag(mode: SubnormalMode) -> u8 {
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

const fn subnormal_from_tag(tag: u8) -> Option<SubnormalMode> {
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

const fn permission_tag(permission: NumericalPermission) -> u8 {
    match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    }
}

const fn permission_from_tag(tag: u8) -> Option<NumericalPermission> {
    match tag {
        0x01 => Some(NumericalPermission::Forbidden),
        0x02 => Some(NumericalPermission::Permitted),
        _ => None,
    }
}

/// One scalar-arithmetic policy subject: an arithmetic type paired with the
/// complete resolved semantic value type it computes in.
///
/// **Proposed relocation of `tiler_compiler::target::ScalarArithmetic`.** The
/// production type is already public and already validates the pair against the
/// governed built-in scalar catalog, which lives in `tiler-ir`; only its siting
/// changes, so `tiler_compiler::target::ScalarArithmetic` becomes a re-export and
/// `ScalarArithmetic::new`'s validation is unmoved.
///
/// `TypeKey` alone is insufficient and the reason is structural rather than
/// stylistic: [`ResolvedValueType`] has three families — nominal, parameterized,
/// and encoded-numeric — and two resolved types within one definition family are
/// distinguished only by their parameters or their ordered encoded components.
/// A subject keyed by the nominal spelling would merge them.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArithmeticSubject {
    arithmetic: ArithmeticType,
    resolved_type: ResolvedValueType,
}

impl ScalarArithmeticSubject {
    /// Pairs one arithmetic type with the semantic value type it computes in.
    ///
    /// The spike models the production validator's *outcome* rather than
    /// re-implementing it: `crate::fixtures` builds every subject through the
    /// real `tiler_compiler::target::ScalarArithmetic`, so a pair this
    /// constructor accepts is one the catalog already admitted.
    #[must_use]
    pub const fn new(arithmetic: ArithmeticType, resolved_type: ResolvedValueType) -> Self {
        Self {
            arithmetic,
            resolved_type,
        }
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

/// The serialized identity of one scalar-arithmetic policy subject.
///
/// # Why the record carries bytes rather than a reconstructed type
///
/// [`ResolvedValueType::canonical_encoding`] is **one-way**: `tiler-ir` publishes
/// the collision-free encoder and no decoder, and the accepted policy behind that
/// is `own-the-numerical-realization-profile-key`'s — decoding yields a *dispatch
/// record* rather than reconstructed compiler IR, so nothing converts one back.
///
/// That is not a limitation to work around, it is the right shape. The exact
/// canonical bytes **are** the full resolved-type identity: they are collision
/// free by construction and their leading family discriminant distinguishes a
/// nominal, a parameterized, and an encoded-numeric type, so the record
/// distinguishes all three without claiming any of them inhabits the
/// scalar-arithmetic schema. The artifact compares and encodes these bytes and
/// never re-derives them — the same ignorance that keeps every other identity
/// this crate is not the authority for consumer-agnostic.
///
/// The arithmetic type is carried as a decodable tag beside them because a
/// consumer must be able to read *which dtype* a record speaks for; that is the
/// whole point of the dtype key, and a wholly opaque subject would reinstate the
/// defect this record exists to correct.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArithmeticSubjectIdentity {
    arithmetic: ArithmeticType,
    resolved_type: Box<[u8]>,
}

/// Maximum byte length of one carried resolved-type canonical identity.
///
/// `tiler_ir::semantic::MAX_RESOLVED_TYPE_BYTES` bounds the *payload* a resolved
/// type may carry; this bounds the framed canonical encoding a record admits, and
/// it is this crate's own for the reason `MAX_OPAQUE_IDENTITY_BYTES` records —
/// the number that admits every value a producer can legally mint belongs beside
/// the reader that must refuse the rest.
pub const MAX_RESOLVED_TYPE_IDENTITY_BYTES: usize = 64 * 1_024;

impl ScalarArithmeticSubjectIdentity {
    /// Wraps canonical resolved-type identity bytes minted by `tiler-ir`.
    ///
    /// # Errors
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

/// Resolves a governed arithmetic-type tag, or `None` for an unrecognized one.
///
/// The fail-closed decode half of [`ArithmeticType::tag`], which `tiler-ir`
/// publishes without an inverse. The proposal adds `ArithmeticType::from_tag`
/// beside `tag` so one exhaustive pair lives in the defining crate; this function
/// is the spike's stand-in for it.
#[must_use]
pub const fn arithmetic_from_tag(tag: u8) -> Option<ArithmeticType> {
    match tag {
        0x01 => Some(ArithmeticType::F16),
        0x02 => Some(ArithmeticType::Bf16),
        0x03 => Some(ArithmeticType::F32),
        0x04 => Some(ArithmeticType::F64),
        _ => None,
    }
}

/// A behaviour the caller's contract must already state for a conditional means.
///
/// Relocated from `tiler_compiler::target::honourability::RelaxationRequirement`.
/// Every field is shared vocabulary once the three types above move, so the
/// relocation adds nothing to `tiler-ir` that is not already there.
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
    /// the codec: the record must be able to state *which* relaxation made a
    /// requirement honourable after a round trip, and only the identity form
    /// decodes.
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

    fn encode(&self, bytes: &mut Vec<u8>) {
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
/// Relocated from `tiler_compiler::target::honourability::HonouringMeans`, with
/// **one correction**, which is the reason the artifact cannot receive this as
/// opaque key bytes.
///
/// # The corrected defect
///
/// `HonouringMeans::key` returns `"supported-only-under-declared-relaxation"` for
/// every conditional value, whatever relaxation it names, so two means differing
/// only in their relaxation payload mint identical key bytes. The staged artifact
/// draft carries exactly those bytes as its whole record of the means, so a
/// reader of that draft cannot tell which relaxation made a requirement
/// honourable — and two artifacts honouring one contract under different
/// relaxations would share the record.
///
/// The correction separates the two roles ADR 0074 convention 2 distinguishes.
/// [`Self::label`] is the presentation string, documented as **not** injective
/// and never encoded; [`Self::encode`] is the identity, and it carries the
/// relaxation payload. The production encoder already had this right — the
/// conditional arm of `HonouringMeans::encode` writes the relaxation — so the
/// correction renames the non-injective accessor to say what it is and stops the
/// artifact record reading identity out of it.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HonouringMeans {
    /// The target's own arithmetic realizes the behaviour.
    SupportedExactly,
    /// The backend emits additional operations that realize the behaviour
    /// exactly. The verdict is satisfied; the emitted program differs.
    SupportedWithExactEmulation,
    /// The behaviour is realized only when the caller's contract already
    /// authorizes the named relaxation on another dimension.
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
    /// compares, encodes, or keys on this value; [`Self::encode`] is the identity
    /// and [`Self::canonical_key`] is the comparable form.
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
/// `CoveredOccurrence` uses, so the obligation and the stage coverage that
/// implements it name the position the same way. The `component` ordinal is
/// present only for [`PolicyLocus::Component`] and is otherwise zero, which the
/// builder enforces rather than leaving to a convention.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NumericalObligationKey {
    occurrence: SemanticOccurrence,
    locus: PolicyLocus,
    component: u32,
}

impl NumericalObligationKey {
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
///
/// Relocated from `tiler_compiler::target::feasibility::FactAuthority`, which is
/// `pub(crate)` today. `carry-the-honourability-fact-provenance-into-the-artifact-record`
/// recorded the consequence: "nothing this crate could carry exists to be
/// carried". Relocation is what makes the artifact record's provenance readable
/// without a second recognizer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactAuthority {
    /// A governed target-neutral profile.
    GovernedProfile,
    /// An externally supplied target profile.
    ExternalProfile,
    /// A measured target profile.
    MeasuredProfile,
    /// Evidence recorded in the artifact itself.
    ArtifactEvidence,
    /// A live device runtime.
    DeviceRuntime,
    /// A prepared kernel.
    PreparedKernel,
    /// One concrete launch instance.
    LaunchInstance,
}

impl FactAuthority {
    /// The governed wire tag naming this authority.
    ///
    /// **The tags are deliberately not in declaration order, and the relocation
    /// must preserve them byte for byte.** `ExternalProfile` and
    /// `MeasuredProfile` were inserted after `0x02`–`0x05` had already been
    /// committed to every target-profile descriptor, so they carry `0x06` and
    /// `0x07`. Renumbering them into declaration order during the move would
    /// silently change `tiler.target-profile.descriptor.v10` for every profile
    /// that declares a measured fact — an identity-domain step this ticket is
    /// forbidden to take, and one that would be invisible in a diff that only
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

    /// The governed canonical key naming this authority.
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
///
/// Relocated from `tiler_compiler::target::feasibility::FactValidityScope`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactValidityScope {
    /// Valid for every environment the profile is portable across.
    PortableProfile,
    /// Valid only on the exact measured environment.
    MeasuredEnvironment,
    /// Valid for one bound device instance.
    DeviceInstance,
    /// Valid for the prepared artifact.
    PreparedArtifact,
    /// Valid for one launch instance.
    LaunchInstance,
}

impl FactValidityScope {
    /// The governed wire tag naming this scope.
    ///
    /// `MeasuredEnvironment` carries `0x05` for the same reason
    /// [`FactAuthority::tag`] records: it was inserted second in declaration
    /// order after `0x02`–`0x04` were already committed. The declaration order
    /// and the tag order are preserved exactly as production states them.
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

    /// The governed canonical key naming this scope.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::PortableProfile => "portable-profile",
            Self::MeasuredEnvironment => "measured-environment",
            Self::PreparedArtifact => "prepared-artifact",
            Self::DeviceInstance => "device-instance",
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
    /// boundary.
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

    fn encode(&self, bytes: &mut Vec<u8>) {
        let Self { key, revision } = self;
        push_slice(bytes, key.as_bytes());
        bytes.extend_from_slice(&revision.to_be_bytes());
    }
}

/// The role one compiler build performed in a measured execution.
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

    fn encode(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Frontend => bytes.push(0x01),
            Self::Optimizer => bytes.push(0x02),
            Self::CodeGenerator => bytes.push(0x03),
            Self::Assembler => bytes.push(0x04),
            Self::Linker => bytes.push(0x05),
            Self::RuntimeCompiler => bytes.push(0x06),
            Self::IntermediateTranslator => bytes.push(0x07),
            Self::ProviderDefined(identity) => {
                bytes.push(0xff);
                identity.encode(bytes);
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

    fn encode(&self, bytes: &mut Vec<u8>) {
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

    fn canonical_bytes(&self) -> Vec<u8> {
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

    fn encode(&self, bytes: &mut Vec<u8>) {
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

    fn encode(&self, bytes: &mut Vec<u8>) {
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

    fn canonical_bytes(&self) -> Vec<u8> {
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
    /// A specification-backed guarantee attributed to an external producer.
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
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::GovernedGuarantee { .. } => 0x01,
            Self::Measurement { .. } => 0x02,
            Self::ExternalGuarantee { .. } => 0x03,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
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
/// Relocated from `tiler_compiler::target::honourability::FactSourceProvenance`.
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
    /// Assembles one provenance statement.
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

    /// This statement's canonical bytes, as a comparable and orderable key.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
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
