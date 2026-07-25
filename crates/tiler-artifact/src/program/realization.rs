#![allow(
    dead_code,
    reason = "the delivered-realization record is a reviewed draft staged under ADR 0074 convention 7: nothing constructs one yet, because the constructor a producer would call is part of the public artifact surface ADR 0075 reserves to Tom. `wire-the-delivered-realization-record-into-the-artifact` is the slice that consumes it, and it cannot land until that surface is accepted"
)]

//! The readable record of the numerical realization an artifact delivered.
//!
//! ADR 0076 item 4. A consumer comparing generated output against a CPU
//! reference reads this record rather than reconstructing the realization from
//! the request, the selected compiler flags, or the target's name. The
//! measurement that forces it is on the ADR: under `-fmetal-math-mode=relaxed`
//! the emitted module records `!"air.compile.fast_math_disable"` while every
//! floating-point operation in it carries a fast-math licence set, so a reader
//! inferring the realization from the module flag reads the opposite of the
//! truth.
//!
//! # What this record adds, and what it deliberately does not restate
//!
//! The *values* of the resolved contract are already carried. A packaged
//! artifact holds one [`tiler_ir::schedule::NumericalRealization`] for the whole
//! portfolio — [`super::builder::ArtifactProgramBuilder`] rejects a variant
//! whose contract differs from its siblings with
//! [`super::ArtifactBuildError::NumericalContractMismatch`] — and the envelope
//! codec writes all four of its dimensions. This record adds only what those
//! values cannot supply: **by what means** each dimension was honoured, at what
//! availability phase that means was declared, and **which profile** declared
//! it.
//!
//! It therefore does not restate a single behaviour. The behaviour each means
//! was declared for *is* the artifact's own resolved contract on that dimension,
//! read from the artifact, so there is no second copy that could disagree with
//! the first. That is also why there is no "actual versus requested" shape here:
//! ADR 0076 item 5 forbids delivering anything other than the declared contract,
//! so the delivered realization equals the declared one for every artifact that
//! exists. This record is the evidence that no downgrade occurred, not a channel
//! for reporting one, and a schema admitting a divergence would invite a future
//! implementation to fill it in.
//!
//! Nor is it a second authority over what identity commits to. `docs/artifact-abi.md`
//! already folds the numerical contract and the exact flags into artifact
//! identity, which is what makes two artifacts distinguishable. A digest is
//! comparable and not readable; this is the readable statement beside it.
//!
//! # Why the means arrives as opaque bytes
//!
//! The four means — honoured exactly by the target's own arithmetic, by exact
//! emulation the backend emits, only under a relaxation the contract already
//! authorizes, or not at all — are declared by `tiler_compiler::honourability`,
//! and ADR 0076 forbids a second authority restating those terms. This crate is
//! a sibling of `tiler-compiler`: both depend only on `tiler-ir` and neither can
//! see the other, so the vocabulary is not merely private but unreachable.
//!
//! It is not relocated into `tiler-ir` to solve that. `AGENTS.md` fixes
//! `tiler-ir` as the crate describing what tensor operations *mean*, not how a
//! device executes them, and target-honourability is a physical, target-aware
//! choice that the same document places in typed target profiles and feasibility
//! predicates. Moving it there for a sibling's convenience would densify a
//! physical choice into the semantic layer.
//!
//! So the means arrives the way every other identity this crate is not the
//! authority for arrives: as bytes another authority minted, which this crate
//! compares and encodes and never re-derives. `HonouringMeans::key` in
//! `tiler-compiler` mints exactly such a key — `"supported-exactly"`,
//! `"supported-with-exact-emulation"`, and so on. Comparing two keys for
//! equality is the whole of what identity validation needs, and it needs no
//! ability to interpret either.
//!
//! [`HonouringMeansKey`] offers no presentation `label()` for the reason ADR
//! 0074 convention 2 offers one at all: a label shortens a wide digest into
//! something a human can read. These bytes are already the readable form, so a
//! label would make the record *less* readable, not more.
//!
//! # Absence is `Unknown`, and `Unknown` rejects
//!
//! An artifact that records no realization has not recorded a permissive one.
//! [`require_recorded`] is the only reader of an optional record and it returns
//! [`UnrecordedRealization`] rather than any realization; there is no `Default`,
//! no `From`, and no accessor that manufactures a means. This is the third-class
//! treatment `carry-the-dtype-on-the-metal-subnormal-flush-fact` established for
//! an unstated dtype, for the same reason: an absent fact that reads as a
//! satisfied one is how a wrong tensor is delivered quietly.
//!
//! Within a record, completeness is structural rather than checked — the record
//! has one field per dimension, so it cannot be missing one. The builder is what
//! refuses to produce a partial record.
//!
//! # What this module does not yet do
//!
//! Nothing constructs a record. A producer reaches one through the artifact
//! builder, and a consumer reads one off a verified or decoded artifact; both of
//! those are public surface that ADR 0075 reserves to Tom, so this module is
//! staged crate-private under ADR 0074 convention 7 and
//! `wire-the-delivered-realization-record-into-the-artifact` owns the wiring.
//! [`DeliveredNumericalRealization::canonical_bytes`] is written and tested here
//! and is not yet folded into
//! [`super::CanonicalArtifactProgramIdentity`]; the envelope section that would
//! carry the record across the codec does not exist.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::push_slice;

use super::expr::AvailabilityPhase;
use super::keys::TargetProfileRef;

/// Versioned domain separator of one delivered-realization record's canonical
/// bytes.
///
/// Self-describing rather than relying on an enclosing domain, because these
/// bytes are compared on their own: two artifacts that delivered the same
/// contract by different means differ here and nowhere else.
const DELIVERED_REALIZATION_DOMAIN: &[u8] = b"tiler.artifact-program.delivered-realization.v1\0";

/// Maximum byte length of one received honouring-means key.
///
/// **Measurement** on this checkout: the longest key
/// `tiler_compiler::honourability::HonouringMeans::key` mints is
/// `"supported-only-under-declared-relaxation"`, 40 bytes, so nothing a governed
/// profile can declare today approaches this bound.
///
/// The bound is this crate's only because no upstream authority publishes one —
/// the same position [`super::MAX_OPAQUE_IDENTITY_BYTES`] records for the target
/// profile descriptor digest, and the same remedy applies: the number that
/// admits every value a producer can legally mint belongs to the producer. It is
/// deliberately *not* [`super::MAX_OPAQUE_IDENTITY_BYTES`], because sharing one
/// bound across identities that share only a shape is what that constant's own
/// documentation warns against.
pub(crate) const MAX_HONOURING_MEANS_KEY_BYTES: usize = 256;

/// The latest availability phase a delivered-realization fact may name.
///
/// A produced artifact rests on facts readable by the time it was produced, and
/// [`AvailabilityPhase::ArtifactEvidence`] is the last such phase. This is the
/// exact complement of the boundary
/// [`super::ArtifactBuildError::NonDeferredPredicatePhase`] already draws from
/// the other side: that rule rejects a *deferred* predicate below
/// [`AvailabilityPhase::LiveDevicePreflight`], because a predicate decided at
/// packaging is not deferred. A means declared readable only from live preflight
/// onward was not relied on to produce these bytes, so recording it as delivered
/// would claim evidence that does not exist.
const LATEST_DELIVERED_PHASE: AvailabilityPhase = AvailabilityPhase::ArtifactEvidence;

/// One governed dimension of the resolved numerical contract.
///
/// These are the four behaviour dimensions
/// [`tiler_ir::schedule::NumericalRealization`] carries, which this crate
/// already projects field by field into its envelope's numerical facts. It names
/// dimensions of a shared-IR record, not the means of honouring them: the means
/// vocabulary stays in `tiler_compiler::honourability` and reaches this crate
/// only as opaque bytes, so this enum is not the second authority ADR 0076
/// forbids. The realization's profile key and canonical NaN bits are absent for
/// the reason the compiler's own projection gives: the first names the governing
/// contract and the second is a produced value, and a target declares
/// honourability for neither.
///
/// Deliberately **not** `#[non_exhaustive]`, under ADR 0074's amended convention
/// 5b. [`Self::tag`] maps it totally, and a wildcard arm there would have to
/// invent an identity byte that only the variant itself determines — the
/// failure convention 3 exists to prevent. A consumer that renders one line per
/// dimension is the same case.
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

/// The canonical dimension order for encoding and for reporting.
pub(crate) const CANONICAL_DIMENSIONS: [NumericalDimension; 4] = [
    NumericalDimension::InputSubnormals,
    NumericalDimension::ResultSubnormals,
    NumericalDimension::Contraction,
    NumericalDimension::Reassociation,
];

impl NumericalDimension {
    /// Returns the governed wire tag naming this dimension.
    ///
    /// Written by an exhaustive match rather than read from the discriminant, so
    /// adding or reordering a dimension is a build error here instead of a
    /// silent re-encoding of every record ever produced (ADR 0074 convention 3).
    #[must_use]
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::InputSubnormals => 0x01,
            Self::ResultSubnormals => 0x02,
            Self::Contraction => 0x03,
            Self::Reassociation => 0x04,
        }
    }
}

impl fmt::Display for NumericalDimension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// A locally decidable rejection of one received honouring-means key.
///
/// Its own type rather than a variant of [`DeliveredRealizationError`], because
/// the wrapping constructor does not know which dimension it is being minted
/// for; the builder attributes it when it does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum HonouringMeansKeyError {
    /// The key had no bytes.
    Empty,
    /// The key exceeded its byte bound.
    TooLong {
        /// Attempted byte length.
        bytes: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
}

impl fmt::Display for HonouringMeansKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for HonouringMeansKeyError {}

/// The opaque key of the means by which a target honours one behaviour.
///
/// The bytes are treated as opaque: this crate compares and encodes them, and
/// never re-derives or interprets them. `tiler_compiler::honourability`'s
/// `HonouringMeans::key` is the authority that mints one, and the wrapping
/// constructor here is a statement that this crate is not.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HonouringMeansKey(Box<[u8]>);

impl HonouringMeansKey {
    /// Wraps opaque means-key bytes minted by the declaring authority.
    ///
    /// # Errors
    ///
    /// Returns [`HonouringMeansKeyError::Empty`] for empty bytes, or
    /// [`HonouringMeansKeyError::TooLong`] beyond
    /// [`MAX_HONOURING_MEANS_KEY_BYTES`].
    pub(crate) fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, HonouringMeansKeyError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(HonouringMeansKeyError::Empty);
        }
        if value.len() > MAX_HONOURING_MEANS_KEY_BYTES {
            return Err(HonouringMeansKeyError::TooLong {
                bytes: value.len(),
                limit: MAX_HONOURING_MEANS_KEY_BYTES,
            });
        }
        Ok(Self(value.into()))
    }

    /// Returns the opaque means-key bytes.
    #[must_use]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// The target fact one dimension of the delivered contract was honoured by.
///
/// It carries the two pieces of ADR 0076 item 3's provenance discipline that a
/// target-neutral artifact can state: the means, opaque, and the availability
/// phase the declaration was readable from. The phase is [`AvailabilityPhase`],
/// which `tiler-ir` defines and both this crate and `tiler-compiler` import, so
/// it is one shared vocabulary rather than a restatement of one.
///
/// The fact's authority and validity scope are **not** carried, and their
/// absence is a boundary rather than a decision: `tiler_compiler::feasibility`
/// keeps `FactAuthority` and `FactValidityScope` crate-private with no minting
/// API, so nothing this crate could carry exists to be carried. ADR 0076 item 4
/// additionally requires the validity scope to identify the compiler build and
/// execution environment the behaviour was measured on; no type in the workspace
/// expresses that today. `carry-the-honourability-fact-provenance-into-the-artifact-record`
/// owns closing both, and a placeholder field is deliberately not reserved for
/// them, because a field a producer cannot fill is the producer-less placeholder
/// this repository has repeatedly had to retract.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct HonouredDimensionFact {
    means: HonouringMeansKey,
    available_at: AvailabilityPhase,
}

impl HonouredDimensionFact {
    /// The opaque key of the means the target honoured this dimension by.
    #[must_use]
    pub(crate) fn means(&self) -> &HonouringMeansKey {
        &self.means
    }

    /// The availability phase the declaration this fact rests on was readable
    /// from.
    #[must_use]
    pub(crate) const fn available_at(&self) -> AvailabilityPhase {
        self.available_at
    }
}

/// The numerical realization one artifact actually delivered.
///
/// One record per artifact, because the contract and the assessed target profile
/// are both artifact-wide: [`super::builder::ArtifactProgramBuilder`] rejects a
/// variant disagreeing with its siblings on either.
///
/// Complete by construction — one field per dimension of
/// [`tiler_ir::schedule::NumericalRealization`] — so [`Self::honoured`] is total
/// and there is no dimension a reader can ask about and receive nothing for.
/// Only [`DeliveredRealizationBuilder::build`] produces one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeliveredNumericalRealization {
    profile: TargetProfileRef,
    input_subnormals: HonouredDimensionFact,
    result_subnormals: HonouredDimensionFact,
    contraction: HonouredDimensionFact,
    reassociation: HonouredDimensionFact,
}

impl DeliveredNumericalRealization {
    /// The declared target profile that declared every means in this record.
    #[must_use]
    pub(crate) const fn profile(&self) -> &TargetProfileRef {
        &self.profile
    }

    /// The fact one dimension was honoured by.
    #[must_use]
    pub(crate) const fn honoured(&self, dimension: NumericalDimension) -> &HonouredDimensionFact {
        match dimension {
            NumericalDimension::InputSubnormals => &self.input_subnormals,
            NumericalDimension::ResultSubnormals => &self.result_subnormals,
            NumericalDimension::Contraction => &self.contraction,
            NumericalDimension::Reassociation => &self.reassociation,
        }
    }

    /// Returns this record's canonical bytes.
    ///
    /// Domain-separated, length-prefixed through `tiler_ir::identity`'s single
    /// definition of the framing, and free of any ordinal: each dimension writes
    /// the tag its own [`NumericalDimension::tag`] arm states, so the encoding
    /// does not depend on declaration order even though it is produced in it.
    #[must_use]
    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DELIVERED_REALIZATION_DOMAIN);
        push_slice(&mut bytes, self.profile.key.as_str().as_bytes());
        push_slice(&mut bytes, self.profile.descriptor.as_bytes());
        for dimension in CANONICAL_DIMENSIONS {
            let fact = self.honoured(dimension);
            bytes.push(dimension.tag());
            push_slice(&mut bytes, fact.means.as_bytes());
            bytes.push(fact.available_at.tag());
        }
        bytes
    }
}

/// A transactional builder for one delivered-realization record.
///
/// Each declaration is checked on insertion and leaves the draft unchanged when
/// rejected; only the consuming [`Self::build`] yields a record, so a record
/// that exists went through every check.
///
/// The failure does not return the builder. ADR 0058's rationale for recoverable
/// ownership is that a large arena-backed draft must be correctable rather than
/// discarded; this draft is four slots and a profile reference, so recovering it
/// would carry the cost of that convention with none of its benefit.
#[derive(Clone, Debug)]
pub(crate) struct DeliveredRealizationBuilder {
    profile: TargetProfileRef,
    input_subnormals: Option<HonouredDimensionFact>,
    result_subnormals: Option<HonouredDimensionFact>,
    contraction: Option<HonouredDimensionFact>,
    reassociation: Option<HonouredDimensionFact>,
}

impl DeliveredRealizationBuilder {
    /// Opens a draft attributed to the profile that declares its means.
    #[must_use]
    pub(crate) fn new(profile: TargetProfileRef) -> Self {
        Self {
            profile,
            input_subnormals: None,
            result_subnormals: None,
            contraction: None,
            reassociation: None,
        }
    }

    /// Records the means one dimension of the delivered contract was honoured
    /// by.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveredRealizationError::MeansKey`] for a malformed key,
    /// [`DeliveredRealizationError::FactPhaseEscape`] for a declaration that was
    /// not readable when the artifact was produced, or
    /// [`DeliveredRealizationError::DimensionRedeclared`] for a dimension
    /// already recorded. Restating a dimension is a dropped fact rather than a
    /// correction, so it is refused instead of taken last-wins.
    pub(crate) fn declare(
        &mut self,
        dimension: NumericalDimension,
        means: impl AsRef<[u8]>,
        available_at: AvailabilityPhase,
    ) -> Result<(), DeliveredRealizationError> {
        if self.slot(dimension).is_some() {
            return Err(DeliveredRealizationError::DimensionRedeclared { dimension });
        }
        if available_at > LATEST_DELIVERED_PHASE {
            return Err(DeliveredRealizationError::FactPhaseEscape {
                dimension,
                available_at,
                admitted_through: LATEST_DELIVERED_PHASE,
            });
        }
        let means = HonouringMeansKey::from_bytes(means)
            .map_err(|cause| DeliveredRealizationError::MeansKey { dimension, cause })?;
        *self.slot_mut(dimension) = Some(HonouredDimensionFact {
            means,
            available_at,
        });
        Ok(())
    }

    /// Freezes the declarations into a complete record.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveredRealizationError::UndeclaredDimension`] naming the
    /// first dimension in canonical order that nothing declared. A partial
    /// record is refused rather than produced, because a record read as complete
    /// while missing a dimension states that nothing honoured it.
    pub(crate) fn build(self) -> Result<DeliveredNumericalRealization, DeliveredRealizationError> {
        for dimension in CANONICAL_DIMENSIONS {
            if self.slot(dimension).is_none() {
                return Err(DeliveredRealizationError::UndeclaredDimension { dimension });
            }
        }
        let Self {
            profile,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
        } = self;
        Ok(DeliveredNumericalRealization {
            profile,
            input_subnormals: input_subnormals.expect("the loop above proved every slot declared"),
            result_subnormals: result_subnormals
                .expect("the loop above proved every slot declared"),
            contraction: contraction.expect("the loop above proved every slot declared"),
            reassociation: reassociation.expect("the loop above proved every slot declared"),
        })
    }

    fn slot(&self, dimension: NumericalDimension) -> Option<&HonouredDimensionFact> {
        match dimension {
            NumericalDimension::InputSubnormals => self.input_subnormals.as_ref(),
            NumericalDimension::ResultSubnormals => self.result_subnormals.as_ref(),
            NumericalDimension::Contraction => self.contraction.as_ref(),
            NumericalDimension::Reassociation => self.reassociation.as_ref(),
        }
    }

    fn slot_mut(&mut self, dimension: NumericalDimension) -> &mut Option<HonouredDimensionFact> {
        match dimension {
            NumericalDimension::InputSubnormals => &mut self.input_subnormals,
            NumericalDimension::ResultSubnormals => &mut self.result_subnormals,
            NumericalDimension::Contraction => &mut self.contraction,
            NumericalDimension::Reassociation => &mut self.reassociation,
        }
    }
}

/// A typed rejection while recording a delivered numerical realization.
///
/// Every variant names the dimension it rejected; none erases its cause into a
/// message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum DeliveredRealizationError {
    /// A received means key was malformed.
    MeansKey {
        /// Dimension the key was offered for.
        dimension: NumericalDimension,
        /// Typed key rejection.
        cause: HonouringMeansKeyError,
    },
    /// A means was declared readable only after the artifact was produced.
    FactPhaseEscape {
        /// Dimension the declaration was offered for.
        dimension: NumericalDimension,
        /// Earliest phase the declaration can be read from.
        available_at: AvailabilityPhase,
        /// Latest phase a produced artifact can have relied on.
        admitted_through: AvailabilityPhase,
    },
    /// The same dimension was declared twice.
    DimensionRedeclared {
        /// Dimension that was declared twice.
        dimension: NumericalDimension,
    },
    /// A dimension of the resolved contract has no recorded means.
    UndeclaredDimension {
        /// First dimension in canonical order that nothing declared.
        dimension: NumericalDimension,
    },
}

impl DeliveredRealizationError {
    /// Returns the stable rule identifier a consumer can surface.
    #[must_use]
    pub(crate) const fn rule(self) -> &'static str {
        match self {
            Self::MeansKey { .. } => "malformed-means-key",
            Self::FactPhaseEscape { .. } => "means-fact-phase-escape",
            Self::DimensionRedeclared { .. } => "dimension-redeclared",
            Self::UndeclaredDimension { .. } => "undeclared-dimension",
        }
    }
}

impl fmt::Display for DeliveredRealizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for DeliveredRealizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MeansKey { cause, .. } => Some(cause),
            Self::FactPhaseEscape { .. }
            | Self::DimensionRedeclared { .. }
            | Self::UndeclaredDimension { .. } => None,
        }
    }
}

/// An artifact that records no delivered numerical realization.
///
/// The `Unknown` third class, kept distinct from every rejection in
/// [`DeliveredRealizationError`]: those say a record was offered and refused,
/// and this says nothing was offered at all. It is deliberately not a variant of
/// that enum, so a caller matching on a malformed record cannot absorb an absent
/// one, and there is no value it can be turned into.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UnrecordedRealization;

impl UnrecordedRealization {
    /// The stable rule identifier a consumer can surface.
    ///
    /// An associated constant rather than the `rule()` accessor
    /// [`DeliveredRealizationError`] and [`super::ArtifactDiagnostic`] carry: the
    /// rejection has no data to vary over, so a method would take a `self` it
    /// could not read.
    pub(crate) const RULE: &'static str = "unrecorded-delivered-realization";
}

impl fmt::Display for UnrecordedRealization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(Self::RULE)
    }
}

impl Error for UnrecordedRealization {}

/// Resolves an artifact's delivered-realization record, refusing an absent one.
///
/// The only reader of an optional record. An artifact recording nothing is
/// `Unknown` and rejects here rather than reading as permissive; no `Default`,
/// `From`, or fallback exists that could manufacture a realization for it.
///
/// # Errors
///
/// Returns [`UnrecordedRealization`] when no record was carried.
pub(crate) fn require_recorded(
    recorded: Option<&DeliveredNumericalRealization>,
) -> Result<&DeliveredNumericalRealization, UnrecordedRealization> {
    recorded.ok_or(UnrecordedRealization)
}

#[cfg(test)]
mod tests {
    use super::{
        CANONICAL_DIMENSIONS, DELIVERED_REALIZATION_DOMAIN, DeliveredNumericalRealization,
        DeliveredRealizationBuilder, DeliveredRealizationError, HonouringMeansKey,
        HonouringMeansKeyError, MAX_HONOURING_MEANS_KEY_BYTES, NumericalDimension,
        UnrecordedRealization, require_recorded,
    };
    use crate::program::expr::AvailabilityPhase;
    use crate::program::keys::{TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef};

    /// The four keys `tiler_compiler::honourability::HonouringMeans::key` mints.
    ///
    /// Copied as opaque test input, not as a vocabulary: nothing in this crate
    /// interprets them, and the tests below compare bytes rather than meaning.
    const MEANS: [&[u8]; 4] = [
        b"supported-exactly",
        b"supported-with-exact-emulation",
        b"supported-only-under-declared-relaxation",
        b"unsupported",
    ];

    fn profile() -> TargetProfileRef {
        TargetProfileRef {
            key: TargetProfileKey::new("tiler.test.profile.v1").expect("a governed profile key"),
            descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])
                .expect("descriptor bytes"),
        }
    }

    fn complete() -> DeliveredRealizationBuilder {
        let mut builder = DeliveredRealizationBuilder::new(profile());
        for (dimension, means) in CANONICAL_DIMENSIONS.into_iter().zip(MEANS) {
            builder
                .declare(dimension, means, AvailabilityPhase::CompileProfile)
                .expect("a well-formed declaration");
        }
        builder
    }

    #[test]
    fn a_complete_record_answers_for_every_dimension() {
        let record = complete().build().expect("a complete record");
        for (dimension, means) in CANONICAL_DIMENSIONS.into_iter().zip(MEANS) {
            assert_eq!(record.honoured(dimension).means().as_bytes(), means);
            assert_eq!(
                record.honoured(dimension).available_at(),
                AvailabilityPhase::CompileProfile,
            );
        }
        assert_eq!(record.profile(), &profile());
    }

    #[test]
    fn every_dimension_must_be_declared_before_a_record_exists() {
        for omitted in CANONICAL_DIMENSIONS {
            let mut builder = DeliveredRealizationBuilder::new(profile());
            for (dimension, means) in CANONICAL_DIMENSIONS.into_iter().zip(MEANS) {
                if dimension == omitted {
                    continue;
                }
                builder
                    .declare(dimension, means, AvailabilityPhase::CompileProfile)
                    .expect("a well-formed declaration");
            }
            assert_eq!(
                builder.build(),
                Err(DeliveredRealizationError::UndeclaredDimension { dimension: omitted }),
            );
        }
    }

    #[test]
    fn an_absent_record_is_unknown_and_rejects() {
        let error = require_recorded(None).expect_err("an absent record rejects");
        assert_eq!(error, UnrecordedRealization);
        assert_eq!(error.to_string(), "unrecorded-delivered-realization");
        assert_eq!(
            UnrecordedRealization::RULE,
            "unrecorded-delivered-realization"
        );

        let record = complete().build().expect("a complete record");
        assert_eq!(
            require_recorded(Some(&record)).expect("a recorded realization"),
            &record,
        );
    }

    #[test]
    fn a_dimension_cannot_be_restated() {
        let mut builder = complete();
        assert_eq!(
            builder.declare(
                NumericalDimension::Contraction,
                b"supported-exactly",
                AvailabilityPhase::CompileProfile,
            ),
            Err(DeliveredRealizationError::DimensionRedeclared {
                dimension: NumericalDimension::Contraction
            }),
        );
        // The rejected insertion left the draft unchanged.
        let record = builder.build().expect("the draft survived the rejection");
        assert_eq!(
            record
                .honoured(NumericalDimension::Contraction)
                .means()
                .as_bytes(),
            MEANS[2],
        );
    }

    #[test]
    fn a_means_readable_only_after_packaging_is_refused() {
        for available_at in [
            AvailabilityPhase::LiveDevicePreflight,
            AvailabilityPhase::PreparedKernelPreflight,
            AvailabilityPhase::LaunchPreflight,
        ] {
            let mut builder = DeliveredRealizationBuilder::new(profile());
            assert_eq!(
                builder.declare(
                    NumericalDimension::InputSubnormals,
                    b"supported-exactly",
                    available_at,
                ),
                Err(DeliveredRealizationError::FactPhaseEscape {
                    dimension: NumericalDimension::InputSubnormals,
                    available_at,
                    admitted_through: AvailabilityPhase::ArtifactEvidence,
                }),
            );
        }
    }

    #[test]
    fn artifact_evidence_is_admitted() {
        let mut builder = DeliveredRealizationBuilder::new(profile());
        builder
            .declare(
                NumericalDimension::InputSubnormals,
                b"supported-with-exact-emulation",
                AvailabilityPhase::ArtifactEvidence,
            )
            .expect("a fact the artifact itself evidences");
    }

    #[test]
    fn a_malformed_means_key_names_its_dimension_and_keeps_its_cause() {
        let mut builder = DeliveredRealizationBuilder::new(profile());
        assert_eq!(
            builder.declare(
                NumericalDimension::Reassociation,
                b"",
                AvailabilityPhase::CompileProfile,
            ),
            Err(DeliveredRealizationError::MeansKey {
                dimension: NumericalDimension::Reassociation,
                cause: HonouringMeansKeyError::Empty,
            }),
        );
        let long = vec![b'k'; MAX_HONOURING_MEANS_KEY_BYTES + 1];
        assert_eq!(
            builder.declare(
                NumericalDimension::Reassociation,
                &long,
                AvailabilityPhase::CompileProfile,
            ),
            Err(DeliveredRealizationError::MeansKey {
                dimension: NumericalDimension::Reassociation,
                cause: HonouringMeansKeyError::TooLong {
                    bytes: MAX_HONOURING_MEANS_KEY_BYTES + 1,
                    limit: MAX_HONOURING_MEANS_KEY_BYTES,
                },
            }),
        );
        assert!(HonouringMeansKey::from_bytes(vec![b'k'; MAX_HONOURING_MEANS_KEY_BYTES]).is_ok());
    }

    #[test]
    fn the_longest_key_the_declaring_authority_mints_fits() {
        // The measurement recorded on `MAX_HONOURING_MEANS_KEY_BYTES`.
        assert!(
            MEANS
                .iter()
                .all(|means| means.len() <= MAX_HONOURING_MEANS_KEY_BYTES)
        );
        assert_eq!(
            MEANS.iter().map(|means| means.len()).max(),
            Some("supported-only-under-declared-relaxation".len()),
        );
    }

    #[test]
    fn canonical_bytes_are_domain_separated_and_length_framed() {
        let bytes = complete()
            .build()
            .expect("a complete record")
            .canonical_bytes();
        assert!(bytes.starts_with(DELIVERED_REALIZATION_DOMAIN));
        // The framing width is spelled out independently of the encoder, so a
        // change to `tiler_ir::identity`'s prefix fails here.
        let framed = &bytes[DELIVERED_REALIZATION_DOMAIN.len()..];
        let key = "tiler.test.profile.v1";
        assert_eq!(
            framed[..8],
            u64::try_from(key.len()).expect("a short key").to_be_bytes(),
        );
        assert_eq!(&framed[8..8 + key.len()], key.as_bytes());
    }

    #[test]
    fn a_record_differing_only_in_one_means_differs_in_canonical_bytes() {
        let baseline = complete()
            .build()
            .expect("a complete record")
            .canonical_bytes();
        let mut builder = DeliveredRealizationBuilder::new(profile());
        for (index, (dimension, means)) in CANONICAL_DIMENSIONS.into_iter().zip(MEANS).enumerate() {
            let means: &[u8] = if index == 0 { b"unsupported" } else { means };
            builder
                .declare(dimension, means, AvailabilityPhase::CompileProfile)
                .expect("a well-formed declaration");
        }
        assert_ne!(
            builder
                .build()
                .expect("a complete record")
                .canonical_bytes(),
            baseline,
        );
    }

    #[test]
    fn a_record_differing_only_in_one_phase_differs_in_canonical_bytes() {
        let baseline = complete()
            .build()
            .expect("a complete record")
            .canonical_bytes();
        let mut builder = DeliveredRealizationBuilder::new(profile());
        for (index, (dimension, means)) in CANONICAL_DIMENSIONS.into_iter().zip(MEANS).enumerate() {
            let phase = if index == 0 {
                AvailabilityPhase::ArtifactEvidence
            } else {
                AvailabilityPhase::CompileProfile
            };
            builder
                .declare(dimension, means, phase)
                .expect("a well-formed declaration");
        }
        assert_ne!(
            builder
                .build()
                .expect("a complete record")
                .canonical_bytes(),
            baseline,
        );
    }

    #[test]
    fn a_record_differing_only_in_its_declaring_profile_differs_in_canonical_bytes() {
        let baseline = complete()
            .build()
            .expect("a complete record")
            .canonical_bytes();
        let other = TargetProfileRef {
            key: TargetProfileKey::new("tiler.test.profile.v2").expect("a governed profile key"),
            descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02])
                .expect("descriptor bytes"),
        };
        let mut builder = DeliveredRealizationBuilder::new(other);
        for (dimension, means) in CANONICAL_DIMENSIONS.into_iter().zip(MEANS) {
            builder
                .declare(dimension, means, AvailabilityPhase::CompileProfile)
                .expect("a well-formed declaration");
        }
        assert_ne!(
            builder
                .build()
                .expect("a complete record")
                .canonical_bytes(),
            baseline,
        );
    }

    #[test]
    fn two_dimensions_swapping_their_means_do_not_share_canonical_bytes() {
        // The property the per-dimension tag protects: a positional encoding
        // alone would make these two records differ, but a reader could not say
        // which dimension carried which means.
        let mut swapped = DeliveredRealizationBuilder::new(profile());
        let order = [MEANS[1], MEANS[0], MEANS[2], MEANS[3]];
        for (dimension, means) in CANONICAL_DIMENSIONS.into_iter().zip(order) {
            swapped
                .declare(dimension, means, AvailabilityPhase::CompileProfile)
                .expect("a well-formed declaration");
        }
        let swapped = swapped.build().expect("a complete record");
        assert_ne!(
            swapped.canonical_bytes(),
            complete()
                .build()
                .expect("a complete record")
                .canonical_bytes(),
        );
        for (dimension, expected) in CANONICAL_DIMENSIONS.into_iter().zip(order) {
            assert_eq!(swapped.honoured(dimension).means().as_bytes(), expected);
        }
    }

    #[test]
    fn every_dimension_tag_is_distinct() {
        let mut tags: Vec<u8> = CANONICAL_DIMENSIONS
            .into_iter()
            .map(NumericalDimension::tag)
            .collect();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), CANONICAL_DIMENSIONS.len());
    }

    #[test]
    fn every_rejection_names_a_stable_rule() {
        let rules = [
            DeliveredRealizationError::MeansKey {
                dimension: NumericalDimension::Contraction,
                cause: HonouringMeansKeyError::Empty,
            }
            .rule(),
            DeliveredRealizationError::FactPhaseEscape {
                dimension: NumericalDimension::Contraction,
                available_at: AvailabilityPhase::LaunchPreflight,
                admitted_through: AvailabilityPhase::ArtifactEvidence,
            }
            .rule(),
            DeliveredRealizationError::DimensionRedeclared {
                dimension: NumericalDimension::Contraction,
            }
            .rule(),
            DeliveredRealizationError::UndeclaredDimension {
                dimension: NumericalDimension::Contraction,
            }
            .rule(),
        ];
        let mut unique = rules.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), rules.len());
    }

    #[test]
    fn a_malformed_key_rejection_preserves_its_cause_through_source() {
        use std::error::Error as _;

        let error = DeliveredRealizationError::MeansKey {
            dimension: NumericalDimension::Contraction,
            cause: HonouringMeansKeyError::Empty,
        };
        let source = error.source().expect("the key rejection is preserved");
        assert!(source.is::<HonouringMeansKeyError>());
    }

    #[test]
    fn a_record_is_not_constructible_without_the_builder() {
        // A compile-time property stated as a runtime witness: the record's
        // fields are private, so the only path to one is `build`, and the only
        // path to a `HonouredDimensionFact` is through `declare`.
        let record: DeliveredNumericalRealization = complete().build().expect("a complete record");
        assert_eq!(
            record
                .honoured(NumericalDimension::InputSubnormals)
                .means()
                .as_bytes(),
            MEANS[0]
        );
    }
}
