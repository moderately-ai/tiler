//! The required record of the numerical realization an artifact delivered.
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
//! # Shape, and why each part is shaped that way
//!
//! One record per artifact, because [`super::builder::ArtifactProgramBuilder`]'s
//! `check_subject` already pins one [`TargetProfileRef`] and one numerical
//! contract across the whole portfolio. The record holds four canonical slices:
//!
//! - **subjects** — one versioned [`NumericalPolicySubject`] per
//!   compiler-produced policy subject, sorted by canonical subject key. The only
//!   implemented family is [`ScalarArithmeticRecord`], which stores the
//!   resolved-type identity once and its eleven resolutions and dispositions in
//!   dense arrays indexed by [`NumericalDimension::index`].
//! - **obligations** — a sparse slice of locus-specific requirements, sorted by
//!   `(subject, dimension, locus)`. A `Required` disposition names a contiguous
//!   non-empty range of it, so a reader borrows the rows without allocating.
//! - **evidence** — deduplicated target-fact rows, referenced by index.
//! - **entry bindings** — one `(entry, subject)` association per packaged
//!   executable entry, which is what lets the neutral artifact cross-check a
//!   dtype-free [`NumericalRealization`] against a dtype-keyed record at all.
//!
//! # Required, not optional
//!
//! The record type admits no absence: there is no `Option`, no
//! `UnrecordedRealization` state, and no reader that can hand back an absent
//! one. Absence is not a permissive realization, and the way this record makes
//! that true is by never admitting absence in the first place rather than by
//! giving every caller a third state to rediscover. The superseded draft's
//! `require_recorded` was migration state that a required terminal record
//! contradicts, and it is gone with it.
//!
//! **And required of an artifact, not only of itself.** Every executable
//! artifact carries one:
//! [`ArtifactProgramBuilder::declare_realization`](super::ArtifactProgramBuilder::declare_realization)
//! is how a producer supplies it, `build` refuses a draft without one, the
//! record's profile is checked against the artifact's single
//! [`TargetProfileRef`], its resolutions are cross-checked against every
//! packaged entry's own realization statement, its canonical bytes are folded
//! into the artifact identity, and it crosses the envelope codec as one framed
//! run the decoder re-validates. [`super::VerifiedArtifactProgram::delivered_realization`]
//! and [`super::DecodedArtifact::delivered_realization`] are total readers.
//!
//! # Eleven named fields are eliminated
//!
//! One struct field per dimension would duplicate the dimension set in the type
//! system, force a public signature change for every dimension added, and make
//! the total accessor a match over eleven arms in each of the record, the
//! builder, and the codec. The dense array indexed by one shared exhaustive
//! match carries the same completeness — an array of length [`DIMENSION_COUNT`]
//! cannot be missing a dimension — with one place to change.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::numerics::{
    CANONICAL_DIMENSIONS, DIMENSION_COUNT, DimensionBehaviour, FactSourceProvenance,
    HonouringMeans, NumericalDimension, NumericalObligationKey, ScalarArithmeticSubjectIdentity,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    ExceptionalValueAssumption, NumericalPermission, NumericalRealization, SubnormalMode,
};

use super::keys::TargetProfileRef;

pub(super) mod codec;
#[cfg(test)]
mod tests;

/// Versioned domain separator of one delivered-realization record's canonical
/// bytes.
///
/// `v2` rather than `v1`: the superseded four-dimension, dtype-free,
/// opaque-means draft's `v1` bytes described a different record entirely, and
/// nothing that holds one of those may match one of these.
pub const DELIVERED_REALIZATION_DOMAIN: &[u8] =
    b"tiler.artifact-program.delivered-realization.v2\0";

/// The latest availability phase a delivered-realization fact may name.
///
/// A produced artifact rests on facts readable by the time it was produced, and
/// [`AvailabilityPhase::ArtifactEvidence`] is the last such phase. A means
/// declared readable only from live preflight onward was not relied on to
/// produce these bytes, so recording it as delivered would claim evidence that
/// does not exist.
pub const LATEST_DELIVERED_PHASE: AvailabilityPhase = AvailabilityPhase::ArtifactEvidence;

/// The governed family tag of one policy-subject record.
///
/// A versioned, tagged seam rather than a universal dtype enum. A future
/// integer, boolean, complex, decimal, quantized, MX, conversion, or
/// owner-defined contract family arrives as a new tag **with** its first
/// producer, consumer, behaviour schema, validation, identity, and lowering
/// evidence. Until then an unrecognized tag rejects fail-closed; it is never
/// skipped.
///
/// Deliberately **not** `#[non_exhaustive]` under ADR 0074 convention 5b: every
/// consumer maps it totally onto an identity tag, and a wildcard arm there would
/// have to invent a tag only the variant itself determines.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RecordFamily {
    /// The scalar floating-point arithmetic contract family.
    ScalarArithmetic,
}

impl RecordFamily {
    /// The governed wire tag naming this family.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::ScalarArithmetic => 0x01,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized family.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::ScalarArithmetic),
            _ => None,
        }
    }
}

/// Whether any packaged route requires a dimension, and where.
///
/// This is the refinement of ADR 0076 item 4's "each dimension's means" wording.
/// The earlier phrasing implies every dimension has a means, which would make an
/// unconsumed dimension carry a fabricated target fact. A disposition states the
/// honest thing instead: either the compiler produced `NotRequired` for every
/// packaged route, or it produced a non-empty canonical range of locus-specific
/// obligations, each with its own required behaviour and evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssessmentDisposition {
    /// No packaged route requires this dimension of this subject.
    ///
    /// **A producer assertion, and written explicitly.** The artifact cannot
    /// re-run the compiler's consumption analysis, so it cannot verify this.
    /// What it can do — and does — is refuse to let the claim be recoverable
    /// from silence: the encoding writes a disposition byte for every dimension,
    /// on the same reasoning `docs/artifact-abi.md` records for the
    /// synchronization realization, where "an entry requiring no realization
    /// writes `0x00` rather than nothing".
    NotRequired,
    /// A packaged route requires this dimension at the named obligation range.
    Required {
        /// First obligation index, into the record's canonical obligation slice.
        first: u32,
        /// Number of obligations, never zero.
        len: u32,
    },
}

impl AssessmentDisposition {
    /// The governed wire tag naming this disposition.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::NotRequired => 0x01,
            Self::Required { .. } => 0x02,
        }
    }
}

/// One dtype-wide scalar-arithmetic contract, complete over all eleven
/// dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarArithmeticRecord {
    subject: ScalarArithmeticSubjectIdentity,
    resolutions: [DimensionBehaviour; DIMENSION_COUNT],
    dispositions: [AssessmentDisposition; DIMENSION_COUNT],
}

impl ScalarArithmeticRecord {
    /// Assembles a contract from already-validated canonical parts.
    ///
    /// Reserved for the decoder and for the perturbation harness that rebuilds a
    /// deliberately non-canonical record to watch decode refuse it.
    pub(super) const fn from_canonical_parts(
        subject: ScalarArithmeticSubjectIdentity,
        resolutions: [DimensionBehaviour; DIMENSION_COUNT],
        dispositions: [AssessmentDisposition; DIMENSION_COUNT],
    ) -> Self {
        Self {
            subject,
            resolutions,
            dispositions,
        }
    }

    /// The policy subject this contract speaks for.
    #[must_use]
    pub const fn subject(&self) -> &ScalarArithmeticSubjectIdentity {
        &self.subject
    }

    /// The resolved behaviour of one dimension.
    ///
    /// Total: the dense array cannot be missing a dimension, so there is no
    /// dimension a reader can ask about and receive nothing for.
    #[must_use]
    pub const fn resolution(&self, dimension: NumericalDimension) -> DimensionBehaviour {
        self.resolutions[dimension.index()]
    }

    /// The assessment disposition of one dimension. Total, for the same reason.
    #[must_use]
    pub const fn disposition(&self, dimension: NumericalDimension) -> AssessmentDisposition {
        self.dispositions[dimension.index()]
    }
}

/// One versioned policy-subject record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NumericalPolicySubject {
    /// The scalar floating-point arithmetic contract family.
    ScalarArithmetic(ScalarArithmeticRecord),
}

impl NumericalPolicySubject {
    /// The family this record belongs to.
    #[must_use]
    pub const fn family(&self) -> RecordFamily {
        match self {
            Self::ScalarArithmetic(_) => RecordFamily::ScalarArithmetic,
        }
    }

    /// The canonical sort key of this record's subject.
    #[must_use]
    pub fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = vec![self.family().tag()];
        match self {
            Self::ScalarArithmetic(record) => record.subject.encode(&mut bytes),
        }
        bytes
    }

    /// The scalar-arithmetic record, or `None` for another family.
    ///
    /// The `Option` is load-bearing rather than redundant, and it is the
    /// versioned seam this record family exists to reserve: `ScalarArithmetic`
    /// is the only family implemented *today*, and an integer, boolean, complex,
    /// decimal, quantized, MX, conversion, or owner-defined family arrives as a
    /// second variant with its own producer and validation. Every consumer
    /// already handles the `None` arm, so adding one is a new variant rather
    /// than a signature change rippling through every call site.
    #[must_use]
    #[allow(
        clippy::unnecessary_wraps,
        reason = "one variant is today's state, not the vocabulary's shape; the note above is the reservation"
    )]
    pub const fn scalar_arithmetic(&self) -> Option<&ScalarArithmeticRecord> {
        match self {
            Self::ScalarArithmetic(record) => Some(record),
        }
    }
}

/// One locus-specific numerical obligation a packaged route relies on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumericalObligation {
    subject: u32,
    dimension: NumericalDimension,
    locus: NumericalObligationKey,
    required: DimensionBehaviour,
    evidence: u32,
}

impl NumericalObligation {
    /// Assembles an obligation from already-validated canonical parts.
    ///
    /// Reserved for the decoder and the perturbation harness.
    pub(super) const fn from_canonical_parts(
        subject: u32,
        dimension: NumericalDimension,
        locus: NumericalObligationKey,
        required: DimensionBehaviour,
        evidence: u32,
    ) -> Self {
        Self {
            subject,
            dimension,
            locus,
            required,
            evidence,
        }
    }

    /// Index of the policy subject this obligation is stated for.
    #[must_use]
    pub const fn subject(&self) -> u32 {
        self.subject
    }

    /// The dimension this obligation is stated on.
    #[must_use]
    pub const fn dimension(&self) -> NumericalDimension {
        self.dimension
    }

    /// The program occurrence and policy locus that produced it.
    #[must_use]
    pub const fn locus(&self) -> NumericalObligationKey {
        self.locus
    }

    /// The behaviour this locus requires.
    #[must_use]
    pub const fn required(&self) -> DimensionBehaviour {
        self.required
    }

    /// Index of the target evidence row that honours it.
    #[must_use]
    pub const fn evidence(&self) -> u32 {
        self.evidence
    }

    /// The canonical sort key: subject, dimension, locus.
    #[must_use]
    pub fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = self.subject.to_be_bytes().to_vec();
        bytes.push(self.dimension.tag());
        self.locus.encode(&mut bytes);
        bytes
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            subject,
            dimension,
            locus,
            required,
            evidence,
        } = self;
        bytes.extend_from_slice(&subject.to_be_bytes());
        bytes.push(dimension.tag());
        locus.encode(bytes);
        required.encode(bytes);
        bytes.extend_from_slice(&evidence.to_be_bytes());
    }
}

/// One deduplicated target-fact row the record relies on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetEvidence {
    subject: u32,
    dimension: NumericalDimension,
    declared: DimensionBehaviour,
    means: HonouringMeans,
    profile: TargetProfileRef,
    source: FactSourceProvenance,
}

impl TargetEvidence {
    /// Assembles an evidence row from already-validated canonical parts.
    ///
    /// Reserved for the decoder and the perturbation harness.
    pub(super) const fn from_canonical_parts(
        subject: u32,
        dimension: NumericalDimension,
        declared: DimensionBehaviour,
        means: HonouringMeans,
        profile: TargetProfileRef,
        source: FactSourceProvenance,
    ) -> Self {
        Self {
            subject,
            dimension,
            declared,
            means,
            profile,
            source,
        }
    }

    /// Index of the policy subject this fact speaks about.
    #[must_use]
    pub const fn subject(&self) -> u32 {
        self.subject
    }

    /// The dimension this fact speaks about.
    #[must_use]
    pub const fn dimension(&self) -> NumericalDimension {
        self.dimension
    }

    /// The behaviour the declaring target speaks about.
    #[must_use]
    pub const fn declared(&self) -> DimensionBehaviour {
        self.declared
    }

    /// The structured means, relaxation payload included.
    ///
    /// Carried structurally rather than as a rendered key, which is the whole
    /// correction ADR 0076 records: [`HonouringMeans::label`] collapses every
    /// declared relaxation to one string, so a record that carried the label
    /// could not say *which* relaxation made a requirement honourable.
    #[must_use]
    pub const fn means(&self) -> &HonouringMeans {
        &self.means
    }

    /// The profile that declared this fact.
    #[must_use]
    pub const fn profile(&self) -> &TargetProfileRef {
        &self.profile
    }

    /// The complete structured provenance: phase, authority, validity scope,
    /// versioned authority identity, and the cited guarantee or the exact
    /// compiler builds and execution environments measured.
    #[must_use]
    pub const fn source(&self) -> &FactSourceProvenance {
        &self.source
    }

    /// The canonical sort and deduplication key.
    #[must_use]
    pub fn canonical_key(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.encode(&mut bytes);
        bytes
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            subject,
            dimension,
            declared,
            means,
            profile,
            source,
        } = self;
        bytes.extend_from_slice(&subject.to_be_bytes());
        bytes.push(dimension.tag());
        declared.encode(bytes);
        means.encode(bytes);
        push_slice(bytes, profile.key.as_str().as_bytes());
        push_slice(bytes, profile.descriptor.as_bytes());
        source.encode(bytes);
    }
}

/// The association binding one packaged executable entry to its policy subject.
///
/// **This is what makes the neutral cross-check possible at all.** An entry's
/// [`NumericalRealization`] carries eight behaviour dimensions and no arithmetic
/// type, so the artifact cannot derive from an entry which subject governs it.
/// The producer states the association and the artifact validates the encoding;
/// the compiler and `tiler-build` are what prove its semantic meaning, and this
/// record says so rather than implying it was checked here.
///
/// The entry ordinal names one packaged executable entry, and this record is
/// deliberately agnostic about which ordinal space that is: it carries the
/// association a producer states, and [`codec::validate_against_artifact`]
/// checks it against the entry sequence its caller supplies, in that caller's
/// own order.
///
/// The artifact wiring fixes the space, exactly as it does for
/// `DeferredPredicateData::entry`. A producer states a **flat declared** ordinal
/// over (variant declaration rank, declared entry ordinal), and
/// [`ArtifactProgramBuilder::build`](super::ArtifactProgramBuilder::build)
/// remaps it once into the **flat canonical** ordinal over (routing rank,
/// canonical stage-key entry) that every reader and the wire then carry. See
/// [`ArtifactProgramBuilder::declare_realization`](super::ArtifactProgramBuilder::declare_realization)
/// for why a producer states the declared space and never the canonical one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntryPolicyBinding {
    entry: u32,
    subject: u32,
}

impl EntryPolicyBinding {
    /// Binds one packaged entry ordinal to one policy-subject index.
    #[must_use]
    pub const fn new(entry: u32, subject: u32) -> Self {
        Self { entry, subject }
    }

    /// The packaged entry ordinal.
    #[must_use]
    pub const fn entry(self) -> u32 {
        self.entry
    }

    /// The policy-subject index.
    #[must_use]
    pub const fn subject(self) -> u32 {
        self.subject
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        let Self { entry, subject } = self;
        bytes.extend_from_slice(&entry.to_be_bytes());
        bytes.extend_from_slice(&subject.to_be_bytes());
    }
}

/// The numerical realization one artifact delivered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveredRealizationRecord {
    profile: TargetProfileRef,
    evidence: Box<[TargetEvidence]>,
    subjects: Box<[NumericalPolicySubject]>,
    obligations: Box<[NumericalObligation]>,
    bindings: Box<[EntryPolicyBinding]>,
}

impl DeliveredRealizationRecord {
    /// The declared target profile every fact in this record is attributed to.
    #[must_use]
    pub const fn profile(&self) -> &TargetProfileRef {
        &self.profile
    }

    /// The canonically ordered policy subjects.
    #[must_use]
    pub fn subjects(&self) -> &[NumericalPolicySubject] {
        &self.subjects
    }

    /// The canonically ordered sparse obligation rows.
    #[must_use]
    pub fn obligations(&self) -> &[NumericalObligation] {
        &self.obligations
    }

    /// The deduplicated target-evidence rows.
    #[must_use]
    pub fn evidence(&self) -> &[TargetEvidence] {
        &self.evidence
    }

    /// The entry-to-subject associations, sorted by packaged entry ordinal.
    #[must_use]
    pub fn bindings(&self) -> &[EntryPolicyBinding] {
        &self.bindings
    }

    /// Resolves one scalar-arithmetic subject's view.
    ///
    /// Allocation-free for the caller. The subject slice is canonically sorted,
    /// so the lookup is a binary search over subject keys.
    #[must_use]
    pub fn scalar_arithmetic(
        &self,
        subject: &ScalarArithmeticSubjectIdentity,
    ) -> Option<ScalarArithmeticView<'_>> {
        let mut key = vec![RecordFamily::ScalarArithmetic.tag()];
        subject.encode(&mut key);
        let index = self
            .subjects
            .binary_search_by(|candidate| candidate.canonical_key().cmp(&key))
            .ok()?;
        self.subjects[index]
            .scalar_arithmetic()
            .map(|record| ScalarArithmeticView {
                record,
                obligations: &self.obligations,
                evidence: &self.evidence,
            })
    }

    /// Returns this record's canonical bytes.
    ///
    /// Domain-separated, length-prefixed through `tiler_ir::identity`'s single
    /// definition of the framing, and free of any ordinal that is not itself a
    /// canonical reference: each subject, obligation, and evidence row writes its
    /// own tags, and the tables are written before the rows that reference them.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DELIVERED_REALIZATION_DOMAIN);
        push_slice(&mut bytes, self.profile.key.as_str().as_bytes());
        push_slice(&mut bytes, self.profile.descriptor.as_bytes());

        push_len(&mut bytes, self.evidence.len());
        for row in &self.evidence {
            row.encode(&mut bytes);
        }

        push_len(&mut bytes, self.subjects.len());
        for subject in &self.subjects {
            bytes.push(subject.family().tag());
            match subject {
                NumericalPolicySubject::ScalarArithmetic(record) => {
                    record.subject.encode(&mut bytes);
                    for dimension in CANONICAL_DIMENSIONS {
                        bytes.push(dimension.tag());
                        record.resolution(dimension).encode(&mut bytes);
                    }
                    for dimension in CANONICAL_DIMENSIONS {
                        bytes.push(dimension.tag());
                        let disposition = record.disposition(dimension);
                        bytes.push(disposition.tag());
                        if let AssessmentDisposition::Required { first, len } = disposition {
                            bytes.extend_from_slice(&first.to_be_bytes());
                            bytes.extend_from_slice(&len.to_be_bytes());
                        }
                    }
                }
            }
        }

        push_len(&mut bytes, self.obligations.len());
        for row in &self.obligations {
            row.encode(&mut bytes);
        }

        push_len(&mut bytes, self.bindings.len());
        for binding in &self.bindings {
            binding.encode(&mut bytes);
        }
        bytes
    }

    /// Rewrites every entry binding through a packaged-entry position map.
    ///
    /// `positions[declared]` is the canonical packaged-entry ordinal of the
    /// declared one. The result is re-sorted, because a remap does not preserve
    /// the canonical `(entry, subject)` order the record's own encoder requires
    /// — which is exactly why this is a rebuild rather than an in-place edit.
    ///
    /// Nothing else moves: subjects, obligations, evidence, and the derived
    /// dispositions are all stated in spaces the entry order does not touch.
    ///
    /// # Errors
    ///
    /// Returns the declared ordinal a binding named when it lies outside the
    /// artifact's packaged-entry range.
    pub(super) fn remap_entries(&self, positions: &[u32]) -> Result<Self, u32> {
        let mut bindings = Vec::with_capacity(self.bindings.len());
        for binding in &self.bindings {
            let declared = binding.entry();
            let canonical = positions
                .get(usize::try_from(declared).expect("u32 fits every supported host usize"))
                .ok_or(declared)?;
            bindings.push(EntryPolicyBinding::new(*canonical, binding.subject()));
        }
        bindings.sort_unstable();
        Ok(Self {
            profile: self.profile.clone(),
            evidence: self.evidence.clone(),
            subjects: self.subjects.clone(),
            obligations: self.obligations.clone(),
            bindings: bindings.into_boxed_slice(),
        })
    }

    /// Assembles a record from already-canonical parts.
    ///
    /// Reserved for the decoder, which has to rebuild a record from bytes it has
    /// separately validated, and for the perturbation harness that rebuilds a
    /// deliberately non-canonical one to watch decode refuse it. It is not a
    /// second producer path: [`DeliveredRealizationBuilder`] is the only way an
    /// out-of-crate caller reaches a record.
    pub(super) fn from_canonical_parts(
        profile: TargetProfileRef,
        evidence: Vec<TargetEvidence>,
        subjects: Vec<NumericalPolicySubject>,
        obligations: Vec<NumericalObligation>,
        bindings: Vec<EntryPolicyBinding>,
    ) -> Self {
        Self {
            profile,
            evidence: evidence.into_boxed_slice(),
            subjects: subjects.into_boxed_slice(),
            obligations: obligations.into_boxed_slice(),
            bindings: bindings.into_boxed_slice(),
        }
    }
}

/// A borrowed view of one scalar-arithmetic subject's complete contract.
#[derive(Clone, Copy, Debug)]
pub struct ScalarArithmeticView<'a> {
    record: &'a ScalarArithmeticRecord,
    obligations: &'a [NumericalObligation],
    evidence: &'a [TargetEvidence],
}

impl<'a> ScalarArithmeticView<'a> {
    /// The policy subject.
    #[must_use]
    pub const fn subject(self) -> &'a ScalarArithmeticSubjectIdentity {
        &self.record.subject
    }

    /// The resolved behaviour of one dimension. Total.
    #[must_use]
    pub const fn resolution(self, dimension: NumericalDimension) -> DimensionBehaviour {
        self.record.resolution(dimension)
    }

    /// The assessment of one dimension, borrowing its obligations in place.
    ///
    /// Allocation-free: a `Required` disposition names a contiguous range of the
    /// record's own canonical slice, so the view is a subslice rather than a
    /// gathered vector.
    #[must_use]
    pub fn assessment(self, dimension: NumericalDimension) -> DispositionView<'a> {
        match self.record.disposition(dimension) {
            AssessmentDisposition::NotRequired => DispositionView::NotRequired,
            AssessmentDisposition::Required { first, len } => {
                let first = first as usize;
                let end = first + len as usize;
                DispositionView::Required(&self.obligations[first..end])
            }
        }
    }

    /// The evidence row one obligation references.
    #[must_use]
    pub fn evidence_for(self, obligation: &NumericalObligation) -> &'a TargetEvidence {
        &self.evidence[obligation.evidence as usize]
    }
}

/// Whether a dimension is required, and by which obligations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispositionView<'a> {
    /// No packaged route requires this dimension. A compiler assertion.
    NotRequired,
    /// The non-empty canonical obligation range that requires it.
    Required(&'a [NumericalObligation]),
}

/// One target-fact declaration a producer offers for an obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetEvidenceDeclaration {
    /// The behaviour the declaring target speaks about.
    pub declared: DimensionBehaviour,
    /// The structured means, relaxation payload included.
    pub means: HonouringMeans,
    /// The profile that declared it.
    pub profile: TargetProfileRef,
    /// The complete structured provenance.
    pub source: FactSourceProvenance,
}

#[derive(Clone, Debug)]
struct DeclaredObligation {
    subject: ScalarArithmeticSubjectIdentity,
    dimension: NumericalDimension,
    locus: NumericalObligationKey,
    required: DimensionBehaviour,
    evidence: TargetEvidenceDeclaration,
}

/// A transactional builder for one delivered-realization record.
///
/// Declarations arrive in **arbitrary order**. The builder validates each on
/// insertion, leaves the draft unchanged when it rejects, and canonicalizes once
/// in the consuming [`Self::build`]. A producer call site therefore never has to
/// reproduce wire ordering, which is the property that keeps the exhaustive
/// `tiler-build` translation a straight walk over the compiler's evidence rather
/// than a sort the translator has to get right.
#[derive(Clone, Debug)]
pub struct DeliveredRealizationBuilder {
    profile: TargetProfileRef,
    subjects: Vec<(
        ScalarArithmeticSubjectIdentity,
        [DimensionBehaviour; DIMENSION_COUNT],
    )>,
    obligations: Vec<DeclaredObligation>,
    bindings: Vec<(u32, ScalarArithmeticSubjectIdentity)>,
}

impl DeliveredRealizationBuilder {
    /// Opens a draft attributed to the profile that declares its facts.
    #[must_use]
    pub fn new(profile: TargetProfileRef) -> Self {
        Self {
            profile,
            subjects: Vec::new(),
            obligations: Vec::new(),
            bindings: Vec::new(),
        }
    }

    /// Declares one complete scalar-arithmetic contract.
    ///
    /// The resolutions arrive as a dense array in [`CANONICAL_DIMENSIONS`]
    /// order, so a producer cannot omit a dimension: the array's length is the
    /// vocabulary's, checked by the compiler rather than at run time.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveredRealizationError::SubjectRedeclared`] for a subject
    /// already declared, or
    /// [`DeliveredRealizationError::ResolutionSpaceMismatch`] when a resolution
    /// belongs to another dimension's behaviour space.
    pub fn declare_scalar_arithmetic(
        &mut self,
        subject: ScalarArithmeticSubjectIdentity,
        resolutions: [DimensionBehaviour; DIMENSION_COUNT],
    ) -> Result<(), DeliveredRealizationError> {
        if self.subjects.iter().any(|(known, _)| known == &subject) {
            return Err(DeliveredRealizationError::SubjectRedeclared {
                subject: Box::new(subject),
            });
        }
        for dimension in CANONICAL_DIMENSIONS {
            let behaviour = resolutions[dimension.index()];
            if !dimension.admits(behaviour) {
                return Err(DeliveredRealizationError::ResolutionSpaceMismatch {
                    dimension,
                    behaviour,
                });
            }
        }
        self.subjects.push((subject, resolutions));
        Ok(())
    }

    /// Declares one locus-specific obligation and the evidence that honours it.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveredRealizationError::UnknownSubject`] when no contract
    /// was declared for the subject,
    /// [`DeliveredRealizationError::ObligationRedeclared`] for a repeated
    /// `(subject, dimension, locus)`,
    /// [`DeliveredRealizationError::MalformedObligationKey`] for a non-component
    /// locus carrying a component ordinal,
    /// [`DeliveredRealizationError::BehaviourSpaceMismatch`] when the required or
    /// declared behaviour belongs to another space,
    /// [`DeliveredRealizationError::EvidenceBehaviourMismatch`] when the evidence
    /// speaks about a behaviour other than the one required,
    /// [`DeliveredRealizationError::EvidenceProfileMismatch`] when the evidence
    /// names a profile other than the record's,
    /// [`DeliveredRealizationError::IncompleteProvenance`] for provenance that is
    /// not complete and internally consistent, and
    /// [`DeliveredRealizationError::FactPhaseEscape`] for evidence readable only
    /// after this artifact was produced.
    pub fn require(
        &mut self,
        subject: &ScalarArithmeticSubjectIdentity,
        dimension: NumericalDimension,
        locus: NumericalObligationKey,
        required: DimensionBehaviour,
        evidence: TargetEvidenceDeclaration,
    ) -> Result<(), DeliveredRealizationError> {
        if !self.subjects.iter().any(|(known, _)| known == subject) {
            return Err(DeliveredRealizationError::UnknownSubject {
                subject: Box::new(subject.clone()),
            });
        }
        if !locus.is_well_formed() {
            return Err(DeliveredRealizationError::MalformedObligationKey { locus });
        }
        if !dimension.admits(required) {
            return Err(DeliveredRealizationError::BehaviourSpaceMismatch {
                dimension,
                behaviour: required,
            });
        }
        if !dimension.admits(evidence.declared) {
            return Err(DeliveredRealizationError::BehaviourSpaceMismatch {
                dimension,
                behaviour: evidence.declared,
            });
        }
        if evidence.declared != required {
            return Err(DeliveredRealizationError::EvidenceBehaviourMismatch {
                dimension,
                required,
                declared: evidence.declared,
            });
        }
        if evidence.profile != self.profile {
            return Err(DeliveredRealizationError::EvidenceProfileMismatch {
                dimension,
                declared_by: Box::new(evidence.profile),
            });
        }
        if !evidence.source.is_valid() {
            return Err(DeliveredRealizationError::IncompleteProvenance { dimension });
        }
        if evidence.source.phase() > LATEST_DELIVERED_PHASE {
            return Err(DeliveredRealizationError::FactPhaseEscape {
                dimension,
                available_at: evidence.source.phase(),
                admitted_through: LATEST_DELIVERED_PHASE,
            });
        }
        if self.obligations.iter().any(|declared| {
            &declared.subject == subject
                && declared.dimension == dimension
                && declared.locus == locus
        }) {
            return Err(DeliveredRealizationError::ObligationRedeclared { dimension, locus });
        }
        self.obligations.push(DeclaredObligation {
            subject: subject.clone(),
            dimension,
            locus,
            required,
            evidence,
        });
        Ok(())
    }

    /// Binds one packaged entry ordinal to the policy subject governing it.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveredRealizationError::UnknownSubject`] for an undeclared
    /// subject, or [`DeliveredRealizationError::EntryRebound`] when the entry is
    /// already bound.
    pub fn bind_entry(
        &mut self,
        entry: u32,
        subject: &ScalarArithmeticSubjectIdentity,
    ) -> Result<(), DeliveredRealizationError> {
        if !self.subjects.iter().any(|(known, _)| known == subject) {
            return Err(DeliveredRealizationError::UnknownSubject {
                subject: Box::new(subject.clone()),
            });
        }
        if self.bindings.iter().any(|(known, _)| *known == entry) {
            return Err(DeliveredRealizationError::EntryRebound { entry });
        }
        self.bindings.push((entry, subject.clone()));
        Ok(())
    }

    /// Freezes the declarations into a canonical record.
    ///
    /// Sorts the subject, evidence, obligation, and binding tables once,
    /// deduplicates evidence, derives every dimension's disposition from the
    /// obligations declared for it, and resolves each obligation's subject and
    /// evidence reference into a canonical index.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveredRealizationError::NoSubjects`] when nothing was
    /// declared. A selected scalar contract always produces one complete
    /// subject, so an empty record would mean no contract was selected at all.
    ///
    /// # Panics
    ///
    /// Panics if a declared obligation or binding names a subject the draft does
    /// not hold, or if a table outgrows [`u32::MAX`] rows. Neither is reachable:
    /// [`Self::require`] and [`Self::bind_entry`] each refuse an unknown subject
    /// on insertion, so a draft that reaches here holds every subject its rows
    /// name, and the tables are bounded far below `u32::MAX`. They are assertions
    /// rather than silent fallbacks because a resolved index that quietly missed
    /// would attribute an obligation to the wrong dtype.
    pub fn build(mut self) -> Result<DeliveredRealizationRecord, DeliveredRealizationError> {
        if self.subjects.is_empty() {
            return Err(DeliveredRealizationError::NoSubjects);
        }

        self.subjects.sort_by_key(|entry| entry.0.canonical_key());
        let subject_index = |subject: &ScalarArithmeticSubjectIdentity| -> u32 {
            let key = subject.canonical_key();
            let index = self
                .subjects
                .binary_search_by(|candidate| candidate.0.canonical_key().cmp(&key))
                .expect("every declared obligation names a declared subject");
            u32::try_from(index).expect("a bounded subject table fits u32")
        };

        // Evidence is deduplicated before the obligations reference it, so two
        // obligations honoured by one fact share a row rather than duplicating
        // its measurement contexts.
        let mut evidence: Vec<TargetEvidence> = self
            .obligations
            .iter()
            .map(|declared| TargetEvidence {
                subject: subject_index(&declared.subject),
                dimension: declared.dimension,
                declared: declared.evidence.declared,
                means: declared.evidence.means.clone(),
                profile: declared.evidence.profile.clone(),
                source: declared.evidence.source.clone(),
            })
            .collect();
        evidence.sort_by_key(TargetEvidence::canonical_key);
        evidence.dedup_by_key(|row| row.canonical_key());

        let mut obligations: Vec<NumericalObligation> = self
            .obligations
            .iter()
            .map(|declared| {
                let subject = subject_index(&declared.subject);
                let row = TargetEvidence {
                    subject,
                    dimension: declared.dimension,
                    declared: declared.evidence.declared,
                    means: declared.evidence.means.clone(),
                    profile: declared.evidence.profile.clone(),
                    source: declared.evidence.source.clone(),
                };
                let key = row.canonical_key();
                let index = evidence
                    .binary_search_by_key(&key, TargetEvidence::canonical_key)
                    .expect("every obligation's evidence was collected into the table");
                NumericalObligation {
                    subject,
                    dimension: declared.dimension,
                    locus: declared.locus,
                    required: declared.required,
                    evidence: u32::try_from(index).expect("a bounded evidence table fits u32"),
                }
            })
            .collect();
        obligations.sort_by_key(NumericalObligation::canonical_key);

        // Dispositions are derived from the canonical obligation slice rather
        // than declared, so a `Required` range is contiguous by construction and
        // cannot name a row that is not there.
        let subjects: Vec<NumericalPolicySubject> = self
            .subjects
            .iter()
            .enumerate()
            .map(|(index, (subject, resolutions))| {
                let index = u32::try_from(index).expect("a bounded subject table fits u32");
                let mut dispositions = [AssessmentDisposition::NotRequired; DIMENSION_COUNT];
                for dimension in CANONICAL_DIMENSIONS {
                    let first = obligations
                        .iter()
                        .position(|row| row.subject == index && row.dimension == dimension);
                    if let Some(first) = first {
                        let len = obligations[first..]
                            .iter()
                            .take_while(|row| row.subject == index && row.dimension == dimension)
                            .count();
                        dispositions[dimension.index()] = AssessmentDisposition::Required {
                            first: u32::try_from(first).expect("a bounded slice fits u32"),
                            len: u32::try_from(len).expect("a bounded slice fits u32"),
                        };
                    }
                }
                NumericalPolicySubject::ScalarArithmetic(ScalarArithmeticRecord {
                    subject: subject.clone(),
                    resolutions: *resolutions,
                    dispositions,
                })
            })
            .collect();

        let mut bindings: Vec<EntryPolicyBinding> = self
            .bindings
            .iter()
            .map(|(entry, subject)| EntryPolicyBinding::new(*entry, subject_index(subject)))
            .collect();
        bindings.sort_unstable();

        Ok(DeliveredRealizationRecord {
            profile: self.profile,
            evidence: evidence.into_boxed_slice(),
            subjects: subjects.into_boxed_slice(),
            obligations: obligations.into_boxed_slice(),
            bindings: bindings.into_boxed_slice(),
        })
    }
}

/// The eight numerical dimensions one packaged entry's own realization states.
///
/// # Why the cross-check subject is its own record
///
/// The record is compared against an entry's realization on **both** sides of
/// the codec, and the two sides hold different values for the same eight facts.
/// A builder holds the shared IR's [`NumericalRealization`], whose contract key
/// is a `&'static str` a compiling build chose. A decoder holds an owned-key
/// dispatch record, whose contract key arrived as bytes — the split
/// `super::codec`'s `NumericalFacts` documents as decided rather than pending.
/// Naming the eight behaviours once lets one exhaustive
/// [`overlapping_behaviour`] serve both, instead of two matches that could
/// drift.
///
/// Neither the contract key nor the canonical NaN bit pattern is carried: the
/// record states behaviours, and a cross-check that compared a key would be
/// comparing the profile the two sides already agree on through
/// [`DeliveredRealizationRecord::profile`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntryRealization {
    /// Treatment of subnormal inputs.
    pub input_subnormals: SubnormalMode,
    /// Treatment of subnormal results.
    pub result_subnormals: SubnormalMode,
    /// Whether contraction is permitted.
    pub contraction: NumericalPermission,
    /// Whether ordered reassociation is permitted.
    pub reassociation: NumericalPermission,
    /// Whether reduction contributors may be permuted.
    pub permutation: NumericalPermission,
    /// Whether observable signed-zero distinctions may be eliminated.
    pub signed_zero: NumericalPermission,
    /// Whether NaN values may be assumed absent.
    pub nan_assumptions: ExceptionalValueAssumption,
    /// Whether infinity values may be assumed absent.
    pub infinity_assumptions: ExceptionalValueAssumption,
}

impl EntryRealization {
    /// Projects the shared IR's scheduled realization onto its eight behaviours.
    ///
    /// The destructuring is exhaustive and field-named, so widening
    /// [`NumericalRealization`] to a ninth consumable dimension is a build error
    /// here rather than a cross-check that silently stops covering it.
    #[must_use]
    pub const fn of(realization: NumericalRealization) -> Self {
        let NumericalRealization {
            profile_key: _,
            canonical_arithmetic_nan_bits: _,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            nan_assumptions,
            infinity_assumptions,
        } = realization;
        Self {
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            nan_assumptions,
            infinity_assumptions,
        }
    }
}

/// The behaviour one entry's [`EntryRealization`] states on a dimension.
///
/// `None` for the three dimensions the scheduled realization does not carry —
/// reciprocal transform, approximate intrinsics, and materialization rounding.
/// Written as one exhaustive match so widening the entry statement to a ninth
/// dimension is a build error here rather than a cross-check that silently stops
/// covering it.
#[must_use]
pub const fn overlapping_behaviour(
    dimension: NumericalDimension,
    realization: EntryRealization,
) -> Option<DimensionBehaviour> {
    match dimension {
        NumericalDimension::InputSubnormals => {
            Some(DimensionBehaviour::Subnormals(realization.input_subnormals))
        }
        NumericalDimension::ResultSubnormals => Some(DimensionBehaviour::Subnormals(
            realization.result_subnormals,
        )),
        NumericalDimension::Contraction => {
            Some(DimensionBehaviour::Transform(realization.contraction))
        }
        NumericalDimension::Reassociation => {
            Some(DimensionBehaviour::Transform(realization.reassociation))
        }
        NumericalDimension::Permutation => {
            Some(DimensionBehaviour::Transform(realization.permutation))
        }
        NumericalDimension::SignedZero => {
            Some(DimensionBehaviour::Transform(realization.signed_zero))
        }
        NumericalDimension::NanAssumptions => Some(DimensionBehaviour::ExceptionalValue(
            realization.nan_assumptions,
        )),
        NumericalDimension::InfinityAssumptions => Some(DimensionBehaviour::ExceptionalValue(
            realization.infinity_assumptions,
        )),
        NumericalDimension::ReciprocalTransform
        | NumericalDimension::ApproximateIntrinsics
        | NumericalDimension::MaterializationRounding => None,
    }
}

/// A typed rejection while recording a delivered numerical realization.
///
/// Every variant names what it rejected and why; none erases its cause into a
/// message.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DeliveredRealizationError {
    /// Nothing was declared, so no contract was selected.
    NoSubjects,
    /// The same policy subject was declared twice.
    SubjectRedeclared {
        /// The subject that was declared twice.
        subject: Box<ScalarArithmeticSubjectIdentity>,
    },
    /// An obligation or binding named a subject nothing declared.
    UnknownSubject {
        /// The subject that was named.
        subject: Box<ScalarArithmeticSubjectIdentity>,
    },
    /// A resolution belongs to another dimension's behaviour space.
    ResolutionSpaceMismatch {
        /// The dimension the resolution was offered for.
        dimension: NumericalDimension,
        /// The behaviour offered.
        behaviour: DimensionBehaviour,
    },
    /// A required or declared behaviour belongs to another dimension's space.
    BehaviourSpaceMismatch {
        /// The dimension the behaviour was offered for.
        dimension: NumericalDimension,
        /// The behaviour offered.
        behaviour: DimensionBehaviour,
    },
    /// The same `(subject, dimension, locus)` was declared twice.
    ObligationRedeclared {
        /// The dimension the obligation was declared on.
        dimension: NumericalDimension,
        /// The locus that was declared twice.
        locus: NumericalObligationKey,
    },
    /// A non-component locus carried a component ordinal.
    MalformedObligationKey {
        /// The malformed key.
        locus: NumericalObligationKey,
    },
    /// The evidence speaks about a behaviour other than the one required.
    EvidenceBehaviourMismatch {
        /// The dimension the mismatch was found on.
        dimension: NumericalDimension,
        /// The behaviour the locus requires.
        required: DimensionBehaviour,
        /// The behaviour the evidence speaks about.
        declared: DimensionBehaviour,
    },
    /// The evidence names a profile other than the record's.
    EvidenceProfileMismatch {
        /// The dimension the mismatch was found on.
        dimension: NumericalDimension,
        /// The profile the evidence named.
        declared_by: Box<TargetProfileRef>,
    },
    /// The evidence's provenance is not complete and internally consistent.
    IncompleteProvenance {
        /// The dimension the incomplete evidence was offered for.
        dimension: NumericalDimension,
    },
    /// Evidence was declared readable only after the artifact was produced.
    FactPhaseEscape {
        /// The dimension the declaration was offered for.
        dimension: NumericalDimension,
        /// Earliest phase the declaration can be read from.
        available_at: AvailabilityPhase,
        /// Latest phase a produced artifact can have relied on.
        admitted_through: AvailabilityPhase,
    },
    /// The same packaged entry was bound to a subject twice.
    EntryRebound {
        /// The entry ordinal that was bound twice.
        entry: u32,
    },
}

impl DeliveredRealizationError {
    /// Every rule this vocabulary can report.
    ///
    /// The builder-side counterpart of
    /// [`codec::RealizationCodecError::ALL_RULES`], and it exists for the same
    /// reason: a perturbation harness counts its coverage against a *named
    /// population* rather than against however many perturbations happen to
    /// exist, so a rule added without a perturbation fails the harness rather
    /// than quietly shrinking what has been watched refusing.
    pub const ALL_RULES: [&'static str; 12] = [
        "no-policy-subjects",
        "subject-redeclared",
        "unknown-policy-subject",
        "resolution-space-mismatch",
        "behaviour-space-mismatch",
        "obligation-redeclared",
        "malformed-obligation-key",
        "evidence-behaviour-mismatch",
        "evidence-profile-mismatch",
        "incomplete-provenance",
        "means-fact-phase-escape",
        "entry-rebound",
    ];

    /// The stable rule identifier a consumer can surface.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::NoSubjects => "no-policy-subjects",
            Self::SubjectRedeclared { .. } => "subject-redeclared",
            Self::UnknownSubject { .. } => "unknown-policy-subject",
            Self::ResolutionSpaceMismatch { .. } => "resolution-space-mismatch",
            Self::BehaviourSpaceMismatch { .. } => "behaviour-space-mismatch",
            Self::ObligationRedeclared { .. } => "obligation-redeclared",
            Self::MalformedObligationKey { .. } => "malformed-obligation-key",
            Self::EvidenceBehaviourMismatch { .. } => "evidence-behaviour-mismatch",
            Self::EvidenceProfileMismatch { .. } => "evidence-profile-mismatch",
            Self::IncompleteProvenance { .. } => "incomplete-provenance",
            Self::FactPhaseEscape { .. } => "means-fact-phase-escape",
            Self::EntryRebound { .. } => "entry-rebound",
        }
    }
}

impl fmt::Display for DeliveredRealizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {self:?}", self.rule())
    }
}

impl Error for DeliveredRealizationError {}
