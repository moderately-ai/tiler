//! The delivered-realization record's canonical codec.
//!
//! Encode is [`super::DeliveredRealizationRecord::canonical_bytes`]; decode is
//! here. Decode is not the inverse of encode written twice; it is the place
//! every producer assertion is checked for the properties the artifact *can*
//! check. The distinction the trust boundary draws is stated at
//! [`validate_against_artifact`] and is load-bearing: an untrusted producer can
//! write a wholly self-consistent record, including a false `NotRequired`, and
//! nothing here upgrades a producer assertion into an independently proved
//! semantics.
//!
//! Every rejection is a typed [`RealizationCodecError`] naming what failed and
//! where. Unknown family, subject-kind, dimension, disposition, means,
//! provenance, locus, phase, authority, validity, or behaviour tags reject
//! **fail-closed** — an older reader never skips an unknown numerical family
//! while still calling the executable artifact validated.

use std::error::Error;
use std::fmt;

use tiler_ir::numerics::{
    CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole, DIMENSION_COUNT,
    DimensionBehaviour, ExecutionEnvironmentIdentity, FactAuthority, FactEvidenceBasis,
    FactSourceProvenance, FactValidityScope, HonouringMeans, MeasurementContext,
    NumericalDimension, NumericalObligationKey, PolicyLocus, ProvenanceIdentity,
    RelaxationRequirement, ScalarArithmeticSubjectIdentity,
};
use tiler_ir::program::SemanticOccurrence;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::ArithmeticType;

use super::super::keys::{TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef};
use super::{
    AssessmentDisposition, DELIVERED_REALIZATION_DOMAIN, DeliveredRealizationRecord,
    EntryPolicyBinding, EntryRealization, LATEST_DELIVERED_PHASE, NumericalObligation,
    NumericalPolicySubject, RecordFamily, ScalarArithmeticRecord, TargetEvidence,
    overlapping_behaviour,
};

/// The subject a decode rejection names.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TagSubject {
    /// A policy-subject record family.
    RecordFamily,
    /// A governed numerical dimension.
    Dimension,
    /// An assessment disposition.
    Disposition,
    /// A honouring means.
    HonouringMeans,
    /// A dimension behaviour.
    DimensionBehaviour,
    /// A policy locus.
    PolicyLocus,
    /// An arithmetic type.
    ArithmeticType,
    /// An availability phase.
    AvailabilityPhase,
    /// A fact authority.
    FactAuthority,
    /// A fact validity scope.
    FactValidityScope,
    /// A fact evidence basis.
    FactEvidenceBasis,
    /// A compiler build role.
    CompilerBuildRole,
}

/// The ordered table a canonicality rejection names.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum OrderedSubject {
    /// The policy-subject table.
    Subjects,
    /// The sparse obligation table.
    Obligations,
    /// The deduplicated evidence table.
    Evidence,
    /// The entry-binding table.
    EntryBindings,
}

/// The referenced table a dangling-reference rejection names.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ReferenceSubject {
    /// An obligation's policy subject.
    ObligationSubject,
    /// An obligation's target evidence.
    ObligationEvidence,
    /// An evidence row's policy subject.
    EvidenceSubject,
    /// An entry binding's policy subject.
    BindingSubject,
    /// A `Required` disposition's obligation range.
    DispositionRange,
}

/// A typed rejection while decoding or validating a delivered-realization
/// record.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RealizationCodecError {
    /// The bytes did not open with the governed versioned domain.
    BadDomain,
    /// The bytes ended inside a record.
    Truncated {
        /// Bytes still required at the point the input ended.
        needed: usize,
    },
    /// Bytes remained after the record was complete.
    TrailingBytes {
        /// Number of unconsumed bytes.
        bytes: usize,
    },
    /// A governed tag this build has never been taught.
    UnknownTag {
        /// The vocabulary the tag was read for.
        subject: TagSubject,
        /// The tag byte read.
        tag: u8,
    },
    /// A table's rows were not in strictly increasing canonical order.
    NonCanonicalOrder {
        /// The table whose order was violated.
        subject: OrderedSubject,
        /// Index of the first row that did not increase.
        index: usize,
    },
    /// A reference named a row outside its table.
    DanglingReference {
        /// The reference that dangled.
        subject: ReferenceSubject,
        /// The index named.
        index: usize,
    },
    /// A subject's dimension rows were not the complete canonical sequence.
    IncompleteDimensionCoverage {
        /// The dimension expected at this position.
        expected: NumericalDimension,
        /// The dimension tag found.
        found: u8,
    },
    /// A behaviour belongs to another dimension's space.
    BehaviourSpaceMismatch {
        /// The dimension the behaviour was read for.
        dimension: NumericalDimension,
        /// The behaviour read.
        behaviour: DimensionBehaviour,
    },
    /// An obligation's required behaviour disagrees with its evidence.
    BehaviourMismatch {
        /// The dimension the mismatch was found on.
        dimension: NumericalDimension,
        /// The behaviour the obligation requires.
        required: DimensionBehaviour,
        /// The behaviour the evidence speaks about.
        declared: DimensionBehaviour,
    },
    /// A `Required` range did not exactly cover its dimension's obligations.
    DispositionCoverageMismatch {
        /// The dimension whose coverage disagreed.
        dimension: NumericalDimension,
        /// Obligations the range named.
        named: usize,
        /// Obligations the canonical slice actually holds.
        present: usize,
    },
    /// A `Required` range named no obligations.
    EmptyRequiredRange {
        /// The dimension whose range was empty.
        dimension: NumericalDimension,
    },
    /// A non-component locus carried a component ordinal.
    MalformedObligationKey {
        /// The malformed key.
        locus: NumericalObligationKey,
    },
    /// An evidence row's provenance is not complete and internally consistent.
    IncompleteProvenance {
        /// Index of the evidence row.
        index: usize,
    },
    /// Evidence was declared readable only after the artifact was produced.
    FactPhaseEscape {
        /// Earliest phase the declaration can be read from.
        available_at: AvailabilityPhase,
        /// Latest phase a produced artifact can have relied on.
        admitted_through: AvailabilityPhase,
    },
    /// A key or identity exceeded its bound, or was empty.
    MalformedIdentity {
        /// What was being read.
        subject: TagSubject,
    },
    /// The record names a profile other than the artifact's.
    ProfileMismatch {
        /// The profile the record names.
        recorded: Box<TargetProfileRef>,
        /// The profile the artifact pins.
        artifact: Box<TargetProfileRef>,
    },
    /// A packaged entry has no policy-subject binding.
    UnboundEntry {
        /// The entry ordinal.
        entry: u32,
    },
    /// A record resolution disagrees with an entry's own realization statement.
    OverlappingRealizationMismatch {
        /// The entry whose statement disagreed.
        entry: u32,
        /// The dimension the disagreement was found on.
        dimension: NumericalDimension,
        /// The behaviour the record states.
        recorded: DimensionBehaviour,
        /// The behaviour the entry states.
        entry_states: DimensionBehaviour,
    },
    /// The record declared no policy subject.
    NoSubjects,
}

impl RealizationCodecError {
    /// Every rule this vocabulary can report.
    ///
    /// Written out so the perturbation harness can check its coverage against a
    /// *named population* rather than against however many perturbations happen
    /// to exist. A rule added without a perturbation fails the harness here
    /// instead of quietly shrinking what has been watched refusing.
    pub const ALL_RULES: [&'static str; 19] = [
        "bad-realization-domain",
        "truncated-realization-record",
        "trailing-realization-bytes",
        "unknown-realization-tag",
        "non-canonical-realization-order",
        "dangling-realization-reference",
        "incomplete-dimension-coverage",
        "behaviour-space-mismatch",
        "evidence-behaviour-mismatch",
        "disposition-coverage-mismatch",
        "empty-required-range",
        "malformed-obligation-key",
        "incomplete-provenance",
        "means-fact-phase-escape",
        "malformed-realization-identity",
        "realization-profile-mismatch",
        "unbound-entry",
        "overlapping-realization-mismatch",
        "no-policy-subjects",
    ];

    /// The stable rule identifier a consumer can surface.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::BadDomain => "bad-realization-domain",
            Self::Truncated { .. } => "truncated-realization-record",
            Self::TrailingBytes { .. } => "trailing-realization-bytes",
            Self::UnknownTag { .. } => "unknown-realization-tag",
            Self::NonCanonicalOrder { .. } => "non-canonical-realization-order",
            Self::DanglingReference { .. } => "dangling-realization-reference",
            Self::IncompleteDimensionCoverage { .. } => "incomplete-dimension-coverage",
            Self::BehaviourSpaceMismatch { .. } => "behaviour-space-mismatch",
            Self::BehaviourMismatch { .. } => "evidence-behaviour-mismatch",
            Self::DispositionCoverageMismatch { .. } => "disposition-coverage-mismatch",
            Self::EmptyRequiredRange { .. } => "empty-required-range",
            Self::MalformedObligationKey { .. } => "malformed-obligation-key",
            Self::IncompleteProvenance { .. } => "incomplete-provenance",
            Self::FactPhaseEscape { .. } => "means-fact-phase-escape",
            Self::MalformedIdentity { .. } => "malformed-realization-identity",
            Self::ProfileMismatch { .. } => "realization-profile-mismatch",
            Self::UnboundEntry { .. } => "unbound-entry",
            Self::OverlappingRealizationMismatch { .. } => "overlapping-realization-mismatch",
            Self::NoSubjects => "no-policy-subjects",
        }
    }
}

impl fmt::Display for RealizationCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {self:?}", self.rule())
    }
}

impl Error for RealizationCodecError {}

type Decoded<T> = Result<T, RealizationCodecError>;

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize) -> Decoded<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(RealizationCodecError::Truncated { needed: len })?;
        if end > self.bytes.len() {
            return Err(RealizationCodecError::Truncated {
                needed: end - self.bytes.len(),
            });
        }
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn byte(&mut self) -> Decoded<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Decoded<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn len(&mut self) -> Decoded<usize> {
        let bytes = self.take(8)?;
        let value = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        usize::try_from(value).map_err(|_| RealizationCodecError::Truncated { needed: usize::MAX })
    }

    fn slice(&mut self) -> Decoded<&'a [u8]> {
        let len = self.len()?;
        self.take(len)
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    /// Reads one behaviour, consuming exactly the bytes it occupies.
    ///
    /// The peek-then-advance shape exists because a behaviour is variable width:
    /// an exceptional-value assumption carrying provenance is three bytes and
    /// every other behaviour is two, so the decoder cannot take a fixed run.
    fn behaviour(&mut self) -> Decoded<DimensionBehaviour> {
        let rest = &self.bytes[self.offset..];
        let (behaviour, width) = DimensionBehaviour::decode(rest).ok_or_else(|| {
            rest.first()
                .map_or(RealizationCodecError::Truncated { needed: 1 }, |tag| {
                    RealizationCodecError::UnknownTag {
                        subject: TagSubject::DimensionBehaviour,
                        tag: *tag,
                    }
                })
        })?;
        self.offset += width;
        Ok(behaviour)
    }
}

fn tag<T>(value: Option<T>, subject: TagSubject, raw: u8) -> Decoded<T> {
    value.ok_or(RealizationCodecError::UnknownTag { subject, tag: raw })
}

fn strictly_increasing<T>(
    rows: &[T],
    subject: OrderedSubject,
    key: impl Fn(&T) -> Vec<u8>,
) -> Decoded<()> {
    for index in 1..rows.len() {
        if key(&rows[index - 1]) >= key(&rows[index]) {
            return Err(RealizationCodecError::NonCanonicalOrder { subject, index });
        }
    }
    Ok(())
}

/// Decodes one delivered-realization record from canonical bytes.
///
/// # Errors
///
/// Returns a [`RealizationCodecError`] for a malformed, truncated,
/// non-canonical, duplicated, dangling, unknown-tagged, behaviour-mismatched, or
/// incomplete-provenance record.
pub fn decode(bytes: &[u8]) -> Decoded<DeliveredRealizationRecord> {
    let mut cursor = Cursor::new(bytes);
    let domain = cursor.take(DELIVERED_REALIZATION_DOMAIN.len())?;
    if domain != DELIVERED_REALIZATION_DOMAIN {
        return Err(RealizationCodecError::BadDomain);
    }
    let profile = decode_profile(&mut cursor)?;

    let evidence_count = cursor.len()?;
    let mut evidence = Vec::with_capacity(evidence_count.min(1_024));
    for _ in 0..evidence_count {
        evidence.push(decode_evidence(&mut cursor)?);
    }
    strictly_increasing(
        &evidence,
        OrderedSubject::Evidence,
        TargetEvidence::canonical_key,
    )?;

    let subject_count = cursor.len()?;
    if subject_count == 0 {
        return Err(RealizationCodecError::NoSubjects);
    }
    let mut subjects = Vec::with_capacity(subject_count.min(1_024));
    for _ in 0..subject_count {
        subjects.push(decode_subject(&mut cursor)?);
    }
    strictly_increasing(
        &subjects,
        OrderedSubject::Subjects,
        NumericalPolicySubject::canonical_key,
    )?;

    let obligation_count = cursor.len()?;
    let mut obligations = Vec::with_capacity(obligation_count.min(4_096));
    for _ in 0..obligation_count {
        obligations.push(decode_obligation(&mut cursor)?);
    }
    strictly_increasing(
        &obligations,
        OrderedSubject::Obligations,
        NumericalObligation::canonical_key,
    )?;

    let binding_count = cursor.len()?;
    let mut bindings = Vec::with_capacity(binding_count.min(4_096));
    for _ in 0..binding_count {
        let entry = cursor.u32()?;
        let subject = cursor.u32()?;
        bindings.push(EntryPolicyBinding::new(entry, subject));
    }
    strictly_increasing(&bindings, OrderedSubject::EntryBindings, |binding| {
        let mut key = binding.entry().to_be_bytes().to_vec();
        key.extend_from_slice(&binding.subject().to_be_bytes());
        key
    })?;

    if cursor.remaining() != 0 {
        return Err(RealizationCodecError::TrailingBytes {
            bytes: cursor.remaining(),
        });
    }

    check_references(&subjects, &obligations, &evidence, &bindings)?;
    Ok(DeliveredRealizationRecord::from_canonical_parts(
        profile,
        evidence,
        subjects,
        obligations,
        bindings,
    ))
}

fn decode_profile(cursor: &mut Cursor<'_>) -> Decoded<TargetProfileRef> {
    let key = cursor.slice()?;
    let key = std::str::from_utf8(key)
        .ok()
        .and_then(|key| TargetProfileKey::new(key).ok())
        .ok_or(RealizationCodecError::MalformedIdentity {
            subject: TagSubject::RecordFamily,
        })?;
    let descriptor = cursor.slice()?;
    let descriptor = TargetProfileDescriptorDigest::from_bytes(descriptor).map_err(|_| {
        RealizationCodecError::MalformedIdentity {
            subject: TagSubject::RecordFamily,
        }
    })?;
    Ok(TargetProfileRef { key, descriptor })
}

fn decode_subject_identity(cursor: &mut Cursor<'_>) -> Decoded<ScalarArithmeticSubjectIdentity> {
    let raw = cursor.byte()?;
    let arithmetic = tag(
        ArithmeticType::from_tag(raw),
        TagSubject::ArithmeticType,
        raw,
    )?;
    let resolved = cursor.slice()?;
    ScalarArithmeticSubjectIdentity::from_parts(arithmetic, resolved).ok_or(
        RealizationCodecError::MalformedIdentity {
            subject: TagSubject::ArithmeticType,
        },
    )
}

fn decode_dimension(cursor: &mut Cursor<'_>) -> Decoded<NumericalDimension> {
    let raw = cursor.byte()?;
    tag(
        NumericalDimension::from_tag(raw),
        TagSubject::Dimension,
        raw,
    )
}

fn decode_locus(cursor: &mut Cursor<'_>) -> Decoded<NumericalObligationKey> {
    let occurrence = SemanticOccurrence::new(cursor.u32()?);
    let raw = cursor.byte()?;
    let locus = tag(PolicyLocus::from_tag(raw), TagSubject::PolicyLocus, raw)?;
    let component = cursor.u32()?;
    let key = match locus {
        PolicyLocus::Component => NumericalObligationKey::component(occurrence, component),
        other => {
            let key = NumericalObligationKey::new(occurrence, other);
            if component != 0 {
                // Reconstructing the malformed key rather than dropping the
                // ordinal is what lets the rejection name what it read: a
                // decoder that silently normalized here would accept two wire
                // forms of one row and break the canonical-order check below.
                return Err(RealizationCodecError::MalformedObligationKey {
                    locus: NumericalObligationKey::component(occurrence, component),
                });
            }
            key
        }
    };
    if !key.is_well_formed() {
        return Err(RealizationCodecError::MalformedObligationKey { locus: key });
    }
    Ok(key)
}

fn decode_means(cursor: &mut Cursor<'_>) -> Decoded<HonouringMeans> {
    let raw = cursor.byte()?;
    match raw {
        0x01 => Ok(HonouringMeans::SupportedExactly),
        0x02 => Ok(HonouringMeans::SupportedWithExactEmulation),
        0x03 => {
            let subject = decode_subject_identity(cursor)?;
            let dimension = decode_dimension(cursor)?;
            let behaviour = cursor.behaviour()?;
            if !dimension.admits(behaviour) {
                return Err(RealizationCodecError::BehaviourSpaceMismatch {
                    dimension,
                    behaviour,
                });
            }
            Ok(HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
                relaxation: RelaxationRequirement::new(subject, dimension, behaviour),
            })
        }
        0x04 => Ok(HonouringMeans::Unsupported),
        _ => Err(RealizationCodecError::UnknownTag {
            subject: TagSubject::HonouringMeans,
            tag: raw,
        }),
    }
}

fn decode_provenance_identity(cursor: &mut Cursor<'_>) -> Decoded<ProvenanceIdentity> {
    let key = cursor.slice()?;
    let key = std::str::from_utf8(key).map_err(|_| RealizationCodecError::MalformedIdentity {
        subject: TagSubject::FactAuthority,
    })?;
    let revision = cursor.u32()?;
    Ok(ProvenanceIdentity::new(key, revision))
}

fn decode_text(cursor: &mut Cursor<'_>, subject: TagSubject) -> Decoded<String> {
    let bytes = cursor.slice()?;
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| RealizationCodecError::MalformedIdentity { subject })
}

fn decode_compiler_build(cursor: &mut Cursor<'_>) -> Decoded<CompilerBuildIdentity> {
    let raw = cursor.byte()?;
    let role = match raw {
        0x01 => CompilerBuildRole::Frontend,
        0x02 => CompilerBuildRole::Optimizer,
        0x03 => CompilerBuildRole::CodeGenerator,
        0x04 => CompilerBuildRole::Assembler,
        0x05 => CompilerBuildRole::Linker,
        0x06 => CompilerBuildRole::RuntimeCompiler,
        0x07 => CompilerBuildRole::IntermediateTranslator,
        0xff => CompilerBuildRole::ProviderDefined(decode_provenance_identity(cursor)?),
        _ => {
            return Err(RealizationCodecError::UnknownTag {
                subject: TagSubject::CompilerBuildRole,
                tag: raw,
            });
        }
    };
    let implementation = decode_text(cursor, TagSubject::CompilerBuildRole)?;
    let version = decode_text(cursor, TagSubject::CompilerBuildRole)?;
    let has_build = cursor.byte()?;
    let build = match has_build {
        0 => None,
        1 => Some(decode_text(cursor, TagSubject::CompilerBuildRole)?),
        _ => {
            return Err(RealizationCodecError::UnknownTag {
                subject: TagSubject::CompilerBuildRole,
                tag: has_build,
            });
        }
    };
    Ok(CompilerBuildIdentity::new(
        role,
        implementation,
        version,
        build,
    ))
}

fn decode_environment(cursor: &mut Cursor<'_>) -> Decoded<ExecutionEnvironmentIdentity> {
    let platform = decode_text(cursor, TagSubject::FactValidityScope)?;
    let platform_version = decode_text(cursor, TagSubject::FactValidityScope)?;
    let platform_build = decode_text(cursor, TagSubject::FactValidityScope)?;
    let architecture = decode_text(cursor, TagSubject::FactValidityScope)?;
    let hardware = decode_text(cursor, TagSubject::FactValidityScope)?;
    Ok(ExecutionEnvironmentIdentity::new(
        platform,
        platform_version,
        platform_build,
        architecture,
        hardware,
    ))
}

fn decode_provenance(cursor: &mut Cursor<'_>) -> Decoded<FactSourceProvenance> {
    // The schema version is read and discarded rather than checked here: a
    // provenance statement whose schema this build does not implement fails the
    // `is_valid` completeness check in `check_references`, which is the one place
    // provenance validity is decided, and deciding it twice would let the two
    // answers drift.
    let _schema = cursor.u32()?;
    let raw_phase = cursor.byte()?;
    let phase = tag(
        AvailabilityPhase::from_tag(raw_phase),
        TagSubject::AvailabilityPhase,
        raw_phase,
    )?;
    let raw_authority = cursor.byte()?;
    let authority = tag(
        FactAuthority::from_tag(raw_authority),
        TagSubject::FactAuthority,
        raw_authority,
    )?;
    let raw_validity = cursor.byte()?;
    let validity = tag(
        FactValidityScope::from_tag(raw_validity),
        TagSubject::FactValidityScope,
        raw_validity,
    )?;
    let authority_identity = decode_provenance_identity(cursor)?;
    let raw_basis = cursor.byte()?;
    let basis = match raw_basis {
        0x01 => FactEvidenceBasis::GovernedGuarantee {
            guarantee: decode_provenance_identity(cursor)?,
        },
        0x03 => FactEvidenceBasis::ExternalGuarantee {
            reference: decode_provenance_identity(cursor)?,
        },
        0x02 => {
            let count = cursor.len()?;
            let mut contexts = Vec::with_capacity(count.min(64));
            for _ in 0..count {
                let builds = cursor.len()?;
                let mut compiler_builds = Vec::with_capacity(builds.min(16));
                for _ in 0..builds {
                    compiler_builds.push(decode_compiler_build(cursor)?);
                }
                let environment = decode_environment(cursor)?;
                contexts.push(MeasurementContext::new(compiler_builds, environment));
            }
            FactEvidenceBasis::Measurement { contexts }
        }
        _ => {
            return Err(RealizationCodecError::UnknownTag {
                subject: TagSubject::FactEvidenceBasis,
                tag: raw_basis,
            });
        }
    };
    Ok(FactSourceProvenance::new(
        phase,
        authority,
        validity,
        authority_identity,
        basis,
    ))
}

fn decode_evidence(cursor: &mut Cursor<'_>) -> Decoded<TargetEvidence> {
    let subject = cursor.u32()?;
    let dimension = decode_dimension(cursor)?;
    let declared = cursor.behaviour()?;
    if !dimension.admits(declared) {
        return Err(RealizationCodecError::BehaviourSpaceMismatch {
            dimension,
            behaviour: declared,
        });
    }
    let means = decode_means(cursor)?;
    let profile = decode_profile(cursor)?;
    let source = decode_provenance(cursor)?;
    Ok(TargetEvidence::from_canonical_parts(
        subject, dimension, declared, means, profile, source,
    ))
}

fn decode_subject(cursor: &mut Cursor<'_>) -> Decoded<NumericalPolicySubject> {
    let raw = cursor.byte()?;
    let family = tag(RecordFamily::from_tag(raw), TagSubject::RecordFamily, raw)?;
    match family {
        RecordFamily::ScalarArithmetic => {
            let subject = decode_subject_identity(cursor)?;
            let mut resolutions = [None; DIMENSION_COUNT];
            for expected in CANONICAL_DIMENSIONS {
                let found = cursor.byte()?;
                if found != expected.tag() {
                    return Err(RealizationCodecError::IncompleteDimensionCoverage {
                        expected,
                        found,
                    });
                }
                let behaviour = cursor.behaviour()?;
                if !expected.admits(behaviour) {
                    return Err(RealizationCodecError::BehaviourSpaceMismatch {
                        dimension: expected,
                        behaviour,
                    });
                }
                resolutions[expected.index()] = Some(behaviour);
            }
            let mut dispositions = [AssessmentDisposition::NotRequired; DIMENSION_COUNT];
            for expected in CANONICAL_DIMENSIONS {
                let found = cursor.byte()?;
                if found != expected.tag() {
                    return Err(RealizationCodecError::IncompleteDimensionCoverage {
                        expected,
                        found,
                    });
                }
                let raw = cursor.byte()?;
                dispositions[expected.index()] = match raw {
                    0x01 => AssessmentDisposition::NotRequired,
                    0x02 => {
                        let first = cursor.u32()?;
                        let len = cursor.u32()?;
                        if len == 0 {
                            return Err(RealizationCodecError::EmptyRequiredRange {
                                dimension: expected,
                            });
                        }
                        AssessmentDisposition::Required { first, len }
                    }
                    _ => {
                        return Err(RealizationCodecError::UnknownTag {
                            subject: TagSubject::Disposition,
                            tag: raw,
                        });
                    }
                };
            }
            let resolutions = resolutions.map(|slot| {
                slot.expect("the loop above filled every dimension or returned an error")
            });
            Ok(NumericalPolicySubject::ScalarArithmetic(
                ScalarArithmeticRecord::from_canonical_parts(subject, resolutions, dispositions),
            ))
        }
    }
}

fn decode_obligation(cursor: &mut Cursor<'_>) -> Decoded<NumericalObligation> {
    let subject = cursor.u32()?;
    let dimension = decode_dimension(cursor)?;
    let locus = decode_locus(cursor)?;
    let required = cursor.behaviour()?;
    if !dimension.admits(required) {
        return Err(RealizationCodecError::BehaviourSpaceMismatch {
            dimension,
            behaviour: required,
        });
    }
    let evidence = cursor.u32()?;
    Ok(NumericalObligation::from_canonical_parts(
        subject, dimension, locus, required, evidence,
    ))
}

/// Checks every reference, coverage range, and evidence association.
#[allow(
    clippy::too_many_lines,
    reason = "one function walking every reference class is what makes the population checkable against the `ReferenceSubject` vocabulary; splitting it by table would let a class be dropped without any single function looking short"
)]
fn check_references(
    subjects: &[NumericalPolicySubject],
    obligations: &[NumericalObligation],
    evidence: &[TargetEvidence],
    bindings: &[EntryPolicyBinding],
) -> Decoded<()> {
    for (index, row) in evidence.iter().enumerate() {
        if row.subject() as usize >= subjects.len() {
            return Err(RealizationCodecError::DanglingReference {
                subject: ReferenceSubject::EvidenceSubject,
                index: row.subject() as usize,
            });
        }
        if !row.source().is_valid() {
            return Err(RealizationCodecError::IncompleteProvenance { index });
        }
        if row.source().phase() > LATEST_DELIVERED_PHASE {
            return Err(RealizationCodecError::FactPhaseEscape {
                available_at: row.source().phase(),
                admitted_through: LATEST_DELIVERED_PHASE,
            });
        }
    }

    for row in obligations {
        if row.subject() as usize >= subjects.len() {
            return Err(RealizationCodecError::DanglingReference {
                subject: ReferenceSubject::ObligationSubject,
                index: row.subject() as usize,
            });
        }
        let Some(fact) = evidence.get(row.evidence() as usize) else {
            return Err(RealizationCodecError::DanglingReference {
                subject: ReferenceSubject::ObligationEvidence,
                index: row.evidence() as usize,
            });
        };
        // The association is checked in all three coordinates. Two of them have
        // been wrong in a hand-built fixture before the check existed: an
        // obligation pointing at a neighbouring dimension's fact, and one
        // pointing at the right dimension of the wrong subject.
        if fact.subject() != row.subject() || fact.dimension() != row.dimension() {
            return Err(RealizationCodecError::DanglingReference {
                subject: ReferenceSubject::ObligationEvidence,
                index: row.evidence() as usize,
            });
        }
        if fact.declared() != row.required() {
            return Err(RealizationCodecError::BehaviourMismatch {
                dimension: row.dimension(),
                required: row.required(),
                declared: fact.declared(),
            });
        }
    }

    for binding in bindings {
        if binding.subject() as usize >= subjects.len() {
            return Err(RealizationCodecError::DanglingReference {
                subject: ReferenceSubject::BindingSubject,
                index: binding.subject() as usize,
            });
        }
    }

    for (index, subject) in subjects.iter().enumerate() {
        let index = u32::try_from(index).expect("a bounded subject table fits u32");
        let Some(record) = subject.scalar_arithmetic() else {
            continue;
        };
        for dimension in CANONICAL_DIMENSIONS {
            let present = obligations
                .iter()
                .filter(|row| row.subject() == index && row.dimension() == dimension)
                .count();
            match record.disposition(dimension) {
                AssessmentDisposition::NotRequired => {
                    if present != 0 {
                        return Err(RealizationCodecError::DispositionCoverageMismatch {
                            dimension,
                            named: 0,
                            present,
                        });
                    }
                }
                AssessmentDisposition::Required { first, len } => {
                    let first = first as usize;
                    let len = len as usize;
                    let end = first
                        .checked_add(len)
                        .filter(|end| *end <= obligations.len())
                        .ok_or(RealizationCodecError::DanglingReference {
                            subject: ReferenceSubject::DispositionRange,
                            index: first,
                        })?;
                    // The range must be exactly the rows for this
                    // `(subject, dimension)` — not merely a valid slice. A range
                    // that is in-bounds but names another dimension's rows would
                    // otherwise decode cleanly and report the wrong obligations.
                    if len != present
                        || obligations[first..end]
                            .iter()
                            .any(|row| row.subject() != index || row.dimension() != dimension)
                    {
                        return Err(RealizationCodecError::DispositionCoverageMismatch {
                            dimension,
                            named: len,
                            present,
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

/// The artifact-level facts a decoded record is cross-checked against.
#[derive(Clone, Debug)]
pub struct ArtifactCrossCheck<'a> {
    /// The one target profile the artifact pins across its portfolio.
    pub profile: &'a TargetProfileRef,
    /// Each packaged entry's own numerical realization statement, by canonical
    /// packaged entry ordinal.
    pub entries: &'a [EntryRealization],
}

/// Validates a record against the artifact that carries it.
///
/// # What this proves, and what it deliberately does not
///
/// It proves three things the neutral artifact genuinely can: the record's
/// profile equals the artifact's single [`TargetProfileRef`]; every packaged
/// entry references an existing policy subject; and the record's eight
/// overlapping scalar resolutions equal every bound entry's own widened
/// realization statement. A mismatch on any of them rejects.
///
/// It proves nothing about whether the compiler's consumption analysis was
/// correct. **An untrusted producer can write a self-consistent record,
/// including a false `NotRequired`**, and every check here would pass. The
/// layering is: the compiler proves the policy subject, the obligation loci, the
/// required behaviours, and the `NotRequired` claims from the checked plan;
/// `tiler-build` proves its translation agrees with that compiler view; and this
/// function proves the encoded associations. Ordinary checked production goes
/// through `tiler_build::realization::translate`;
/// [`ArtifactProgramBuilder::declare_realization`](super::super::ArtifactProgramBuilder::declare_realization)
/// is the low-level seam a producer reaches directly, and it accepts typed
/// producer assertions, which its own documentation says.
///
/// `entries` is read in the caller's own order and the record's binding ordinals
/// are interpreted in that same order. This function does not decide which
/// ordinal space a packaged entry lives in; its caller does.
///
/// # Errors
///
/// Returns [`RealizationCodecError::ProfileMismatch`],
/// [`RealizationCodecError::UnboundEntry`], or
/// [`RealizationCodecError::OverlappingRealizationMismatch`].
///
/// # Panics
///
/// Panics if `artifact.entries` holds more than [`u32::MAX`] entries. Every
/// artifact bound governing a packaged entry count is far below that, so the
/// conversion is infallible for any portfolio this crate can construct; it is an
/// assertion rather than a silent truncation because a truncated ordinal would
/// bind an entry to another entry's subject.
pub fn validate_against_artifact(
    record: &DeliveredRealizationRecord,
    artifact: &ArtifactCrossCheck<'_>,
) -> Decoded<()> {
    if record.profile() != artifact.profile {
        return Err(RealizationCodecError::ProfileMismatch {
            recorded: Box::new(record.profile().clone()),
            artifact: Box::new(artifact.profile.clone()),
        });
    }
    for (entry, realization) in artifact.entries.iter().enumerate() {
        let entry = u32::try_from(entry).expect("a bounded entry table fits u32");
        let binding = record
            .bindings()
            .iter()
            .find(|binding| binding.entry() == entry)
            .ok_or(RealizationCodecError::UnboundEntry { entry })?;
        let subject = &record.subjects()[binding.subject() as usize];
        let Some(record_row) = subject.scalar_arithmetic() else {
            continue;
        };
        for dimension in CANONICAL_DIMENSIONS {
            let Some(entry_states) = overlapping_behaviour(dimension, *realization) else {
                continue;
            };
            let recorded = record_row.resolution(dimension);
            if recorded != entry_states {
                return Err(RealizationCodecError::OverlappingRealizationMismatch {
                    entry,
                    dimension,
                    recorded,
                    entry_states,
                });
            }
        }
    }
    Ok(())
}
