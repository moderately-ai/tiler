//! Classified conformance evidence, and what each class may discharge.
//!
//! ADR 0042 records the reason this is a separate subject from the contract
//! itself: "A normative platform guarantee, an exhaustive finite-format test, a
//! proof, and a vendor table of maximum errors observed in non-exhaustive testing
//! do not establish the same claim. Tiler must not turn an implementation name or
//! an empirical measurement into an unstated portable error guarantee."
//!
//! So the class is on the evidence, not on the contract, and the split it makes
//! is a hard one:
//!
//! - **proof, exhaustive testing, and an applicable normative guarantee** may
//!   discharge a hard accuracy feasibility requirement;
//! - **empirical qualification** detects regressions and characterizes an
//!   implementation, and proves no unmeasured worst-case bound;
//! - **unknown** remains unknown and cannot satisfy a hard contract.
//!
//! [`ConformanceEvidenceClass::discharges_hard_requirement`] is the only place
//! that line is drawn, and [`ConformanceEvidence::discharge`] is the only way to
//! consume it — so a caller cannot reach a hard feasibility conclusion by reading
//! the class and deciding for itself.
//!
//! # Why every record carries nine fields
//!
//! ADR 0042: "Evidence records include their scope, target,
//! implementation/helper identity, toolchain, device where applicable, reference
//! oracle, corpus, and digest." An evidence record missing any of them is not a
//! weaker record — it is one whose claim cannot be reproduced or refuted, which
//! is the same as no record. The device is optional because a proof has none; the
//! oracle and the corpus are required exactly of the two classes that have one.

use std::error::Error;
use std::fmt;

use crate::identity::push_slice;
use crate::semantic::NormativeDefinitionRef;

/// Domain separator of a canonical conformance-evidence encoding.
const CONFORMANCE_EVIDENCE_DOMAIN: &[u8] = b"tiler.conformance-evidence.v1\0";

/// Maximum bytes one conformance-evidence digest may carry.
pub const MAX_CONFORMANCE_EVIDENCE_DIGEST_BYTES: usize = 128;

/// The provenance class of one conformance-evidence record.
///
/// Not `#[non_exhaustive]`: every consumer that decides feasibility matches this
/// exhaustively, so a new class is a build error at each such site rather than a
/// silently unclassified claim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConformanceEvidenceClass {
    /// A formal proof over the complete admitted domain.
    FormalProof,
    /// Exhaustive testing over the complete admitted finite input space.
    ExhaustiveFinite,
    /// An applicable normative specification or vendor guarantee.
    NormativeGuarantee,
    /// Empirical qualification under a named test corpus and environment.
    ///
    /// Detects regressions and characterizes an implementation. It does **not**
    /// prove an unmeasured worst-case bound, however carefully sampled, and
    /// [`Self::discharges_hard_requirement`] is `false` for exactly that reason.
    EmpiricalQualification,
    /// Nothing is established.
    ///
    /// A first-class class rather than an absent record, so that "we have not
    /// measured this" is a statement the graph can carry and explain rather than
    /// a gap that reads as a pass.
    Unknown,
}

impl ConformanceEvidenceClass {
    /// Every class, in canonical order.
    pub const ALL: [Self; 5] = [
        Self::FormalProof,
        Self::ExhaustiveFinite,
        Self::NormativeGuarantee,
        Self::EmpiricalQualification,
        Self::Unknown,
    ];

    /// Returns whether this class may discharge a hard accuracy feasibility requirement.
    ///
    /// The one definition of ADR 0042's line. Written as an exhaustive match
    /// rather than a negated set membership, so a new class must state its own
    /// answer instead of inheriting whichever side the default happened to be.
    #[must_use]
    pub const fn discharges_hard_requirement(self) -> bool {
        match self {
            Self::FormalProof | Self::ExhaustiveFinite | Self::NormativeGuarantee => true,
            Self::EmpiricalQualification | Self::Unknown => false,
        }
    }

    /// Returns whether this class requires a named oracle and corpus.
    const fn requires_oracle_and_corpus(self) -> bool {
        match self {
            Self::ExhaustiveFinite | Self::EmpiricalQualification => true,
            Self::FormalProof | Self::NormativeGuarantee | Self::Unknown => false,
        }
    }

    /// Returns the canonical spelling.
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::FormalProof => "formal-proof",
            Self::ExhaustiveFinite => "exhaustive-finite",
            Self::NormativeGuarantee => "normative-guarantee",
            Self::EmpiricalQualification => "empirical-qualification",
            Self::Unknown => "unknown",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::FormalProof => 1,
            Self::ExhaustiveFinite => 2,
            Self::NormativeGuarantee => 3,
            Self::EmpiricalQualification => 4,
            Self::Unknown => 5,
        }
    }
}

impl fmt::Display for ConformanceEvidenceClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spelling())
    }
}

/// Why one conformance-evidence record is invalid, or cannot discharge a requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConformanceEvidenceError {
    /// The record's digest was empty or over the canonical bound.
    MalformedDigest {
        /// Actual byte count.
        bytes: usize,
    },
    /// A class that measures something named no reference oracle.
    MissingReferenceOracle {
        /// The class that requires one.
        class: ConformanceEvidenceClass,
    },
    /// A class that measures something named no corpus.
    MissingCorpus {
        /// The class that requires one.
        class: ConformanceEvidenceClass,
    },
    /// The record's class cannot discharge a hard accuracy feasibility requirement.
    ///
    /// The fail-closed path. `Unknown` reaches it because nothing was established,
    /// and `EmpiricalQualification` reaches it because a measured maximum is not a
    /// worst-case bound — two different reasons for one refusal, kept apart by the
    /// class this variant carries.
    ClassCannotDischarge {
        /// The class that was offered.
        class: ConformanceEvidenceClass,
    },
}

impl ConformanceEvidenceError {
    /// Returns the stable provider diagnostic code naming this refusal.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::MalformedDigest { .. } => "accuracy.evidence.malformed-digest",
            Self::MissingReferenceOracle { .. } => "accuracy.evidence.missing-reference-oracle",
            Self::MissingCorpus { .. } => "accuracy.evidence.missing-corpus",
            Self::ClassCannotDischarge { .. } => "accuracy.evidence.class-cannot-discharge",
        }
    }
}

impl fmt::Display for ConformanceEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedDigest { bytes } => write!(
                formatter,
                "a conformance-evidence digest of {bytes} bytes is empty or exceeds {MAX_CONFORMANCE_EVIDENCE_DIGEST_BYTES}"
            ),
            Self::MissingReferenceOracle { class } => write!(
                formatter,
                "{class} evidence compares against a reference oracle and this record names none"
            ),
            Self::MissingCorpus { class } => write!(
                formatter,
                "{class} evidence is qualified by a corpus and this record names none"
            ),
            Self::ClassCannotDischarge { class } => write!(
                formatter,
                "{class} evidence cannot discharge a hard accuracy feasibility requirement"
            ),
        }
    }
}

impl Error for ConformanceEvidenceError {}

/// One classified conformance-evidence record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceEvidence {
    class: ConformanceEvidenceClass,
    scope: NormativeDefinitionRef,
    target: NormativeDefinitionRef,
    implementation_identity: NormativeDefinitionRef,
    toolchain: NormativeDefinitionRef,
    device: Option<NormativeDefinitionRef>,
    reference_oracle: Option<NormativeDefinitionRef>,
    corpus: Option<NormativeDefinitionRef>,
    digest: Vec<u8>,
}

impl ConformanceEvidence {
    /// Assembles one evidence record.
    ///
    /// # Errors
    ///
    /// Returns [`ConformanceEvidenceError`] for a malformed digest, or when a
    /// class that measures something names no oracle or no corpus. Those two are
    /// refusals rather than optional fields because a measurement whose oracle or
    /// corpus is unstated cannot be reproduced, and an irreproducible measurement
    /// is not evidence.
    #[allow(
        clippy::too_many_arguments,
        reason = "ADR 0042 enumerates the record's fields and every one is an explicit required argument, so widening the record breaks every constructor instead of silently defaulting a new obligation"
    )]
    pub fn new(
        class: ConformanceEvidenceClass,
        scope: NormativeDefinitionRef,
        target: NormativeDefinitionRef,
        implementation_identity: NormativeDefinitionRef,
        toolchain: NormativeDefinitionRef,
        device: Option<NormativeDefinitionRef>,
        reference_oracle: Option<NormativeDefinitionRef>,
        corpus: Option<NormativeDefinitionRef>,
        digest: impl AsRef<[u8]>,
    ) -> Result<Self, ConformanceEvidenceError> {
        let digest = digest.as_ref();
        if digest.is_empty() || digest.len() > MAX_CONFORMANCE_EVIDENCE_DIGEST_BYTES {
            return Err(ConformanceEvidenceError::MalformedDigest {
                bytes: digest.len(),
            });
        }
        if class.requires_oracle_and_corpus() {
            if reference_oracle.is_none() {
                return Err(ConformanceEvidenceError::MissingReferenceOracle { class });
            }
            if corpus.is_none() {
                return Err(ConformanceEvidenceError::MissingCorpus { class });
            }
        }
        Ok(Self {
            class,
            scope,
            target,
            implementation_identity,
            toolchain,
            device,
            reference_oracle,
            corpus,
            digest: digest.to_vec(),
        })
    }

    /// Returns the provenance class.
    #[must_use]
    pub const fn class(&self) -> ConformanceEvidenceClass {
        self.class
    }

    /// Returns the scope this record covers.
    #[must_use]
    pub const fn scope(&self) -> &NormativeDefinitionRef {
        &self.scope
    }

    /// Returns the target this record is about.
    #[must_use]
    pub const fn target(&self) -> &NormativeDefinitionRef {
        &self.target
    }

    /// Returns the implementation or helper identity this record measured.
    #[must_use]
    pub const fn implementation_identity(&self) -> &NormativeDefinitionRef {
        &self.implementation_identity
    }

    /// Returns the toolchain the record was produced under.
    #[must_use]
    pub const fn toolchain(&self) -> &NormativeDefinitionRef {
        &self.toolchain
    }

    /// Returns the device, where the record has one.
    #[must_use]
    pub const fn device(&self) -> Option<&NormativeDefinitionRef> {
        self.device.as_ref()
    }

    /// Returns the reference oracle, where the record has one.
    #[must_use]
    pub const fn reference_oracle(&self) -> Option<&NormativeDefinitionRef> {
        self.reference_oracle.as_ref()
    }

    /// Returns the corpus, where the record has one.
    #[must_use]
    pub const fn corpus(&self) -> Option<&NormativeDefinitionRef> {
        self.corpus.as_ref()
    }

    /// Returns the record's digest.
    #[must_use]
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }

    /// Discharges a hard accuracy feasibility requirement with this record.
    ///
    /// # Errors
    ///
    /// Returns [`ConformanceEvidenceError::ClassCannotDischarge`] for empirical
    /// and unknown evidence. This is the fail-closed path ADR 0042 requires:
    /// "Unknown behavior remains unknown and cannot satisfy a hard contract."
    pub fn discharge(&self) -> Result<HardAccuracyDischarge<'_>, ConformanceEvidenceError> {
        if self.class.discharges_hard_requirement() {
            Ok(HardAccuracyDischarge { evidence: self })
        } else {
            Err(ConformanceEvidenceError::ClassCannotDischarge { class: self.class })
        }
    }

    /// Returns the domain-separated canonical encoding of this record.
    #[must_use]
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, CONFORMANCE_EVIDENCE_DOMAIN);
        bytes.push(self.class.tag());
        for field in [
            Some(&self.scope),
            Some(&self.target),
            Some(&self.implementation_identity),
            Some(&self.toolchain),
            self.device.as_ref(),
            self.reference_oracle.as_ref(),
            self.corpus.as_ref(),
        ] {
            match field {
                None => bytes.push(0),
                Some(reference) => {
                    bytes.push(1);
                    push_slice(&mut bytes, reference.as_str().as_bytes());
                }
            }
        }
        push_slice(&mut bytes, &self.digest);
        bytes
    }
}

/// Evidence that a hard accuracy feasibility requirement is discharged.
///
/// There is no constructor other than [`ConformanceEvidence::discharge`], so
/// holding one is evidence that the class was checked — a caller cannot assemble
/// the conclusion from an empirical or unknown record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HardAccuracyDischarge<'a> {
    evidence: &'a ConformanceEvidence,
}

impl<'a> HardAccuracyDischarge<'a> {
    /// Returns the record that discharged the requirement.
    #[must_use]
    pub const fn evidence(self) -> &'a ConformanceEvidence {
        self.evidence
    }
}

#[cfg(test)]
mod tag_injectivity_tests {
    use std::mem::variant_count;

    use super::ConformanceEvidenceClass;

    #[test]
    fn the_conformance_evidence_tag_table_is_injective_over_its_variant_set() {
        const CLASSES: [ConformanceEvidenceClass; variant_count::<ConformanceEvidenceClass>()] =
            ConformanceEvidenceClass::ALL;
        crate::exhaustive_injectivity::assert_tag_table(
            "ConformanceEvidenceClass::tag",
            &CLASSES,
            ConformanceEvidenceClass::tag,
        );
    }
}
