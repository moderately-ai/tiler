//! Every proposed validation check, perturbed once and observed failing.
//!
//! A check that has never been watched saying no is a check whose population is
//! unknown. Each perturbation below names the check it targets, produces the one
//! wire image or builder call that must break it, and asserts the **exact** rule
//! identifier — not merely that something failed, because a perturbation that
//! trips a different check reports a pass for a check that was never exercised.
//!
//! Two shapes are used, deliberately:
//!
//! - **Structural perturbations** rebuild a deliberately non-canonical record
//!   through `from_canonical_parts` and re-encode it with the one production
//!   encoder. Reversing a table or dangling an index this way exercises the real
//!   `canonical_bytes`, so a perturbation cannot pass by disagreeing with the
//!   encoder it is supposed to be testing.
//! - **Tag perturbations** poke one byte at a computed offset, because an unknown
//!   tag is by construction a value no Rust value can hold.

use crate::codec::{
    ArtifactCrossCheck, OrderedSubject, RealizationCodecError, ReferenceSubject, TagSubject,
    decode, validate_against_artifact,
};
use crate::fixtures;
use crate::record::{
    DELIVERED_REALIZATION_DOMAIN, DeliveredRealizationBuilder, DeliveredRealizationRecord,
    EntryPolicyBinding, NumericalObligation, TargetEvidence,
};
use crate::shared::{
    CANONICAL_DIMENSIONS, DimensionBehaviour, FactSourceProvenance, NumericalDimension,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{NumericalPermission, SubnormalMode};

/// One perturbation's outcome.
pub struct Perturbation {
    /// What was perturbed.
    pub name: &'static str,
    /// The rule identifier the check reported.
    pub observed: String,
}

/// Splits a record into its canonical parts so a perturbation can rebuild it.
fn parts(
    record: &DeliveredRealizationRecord,
) -> (
    Vec<TargetEvidence>,
    Vec<crate::record::NumericalPolicySubject>,
    Vec<NumericalObligation>,
    Vec<EntryPolicyBinding>,
) {
    (
        record.evidence().to_vec(),
        record.subjects().to_vec(),
        record.obligations().to_vec(),
        record.bindings().to_vec(),
    )
}

/// The byte offset at which the subject table's first family tag sits.
///
/// Computed from the record's own leading field widths rather than searched for,
/// so a perturbation cannot silently start poking the wrong byte when a field
/// width changes: the offsets below are wrong loudly, as a decode that reports an
/// unexpected rule, rather than quietly.
fn subject_family_offset(record: &DeliveredRealizationRecord) -> usize {
    DELIVERED_REALIZATION_DOMAIN.len()
        + 8
        + record.profile().key.as_str().len()
        + 8
        + record.profile().descriptor.as_bytes().len()
        + 8 // evidence count
        + evidence_span(record)
        + 8 // subject count
}

/// Total encoded width of the evidence table's rows.
///
/// Measured through `canonical_key`, which *is* the row's encoding rather than a
/// summary of it, so the span cannot drift from what the encoder writes.
fn evidence_span(record: &DeliveredRealizationRecord) -> usize {
    record
        .evidence()
        .iter()
        .map(|row| row.canonical_key().len())
        .sum()
}

/// The byte offset of the first evidence row's honouring-means tag.
fn first_means_offset(record: &DeliveredRealizationRecord) -> usize {
    let head = DELIVERED_REALIZATION_DOMAIN.len()
        + 8
        + record.profile().key.as_str().len()
        + 8
        + record.profile().descriptor.as_bytes().len()
        + 8; // evidence count
    let row = record.evidence().first().expect("a row to perturb");
    let mut behaviour = Vec::new();
    row.declared().encode(&mut behaviour);
    head + 4 /* subject index */ + 1 /* dimension tag */ + behaviour.len()
}

fn rule_of(error: &RealizationCodecError) -> String {
    error.rule().to_owned()
}

fn expect_decode_failure(name: &'static str, bytes: &[u8], expected: &str) -> Perturbation {
    let error = decode(bytes).expect_err(name);
    let observed = rule_of(&error);
    assert_eq!(
        observed, expected,
        "{name} tripped the wrong check: {error}"
    );
    Perturbation { name, observed }
}

/// Runs every perturbation, returning what each check reported.
///
/// # Panics
///
/// Panics when any perturbation fails to trip its own check, or trips another
/// one. That is the whole product: a silent pass here would be a check nothing
/// has watched refuse.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "the population is the point: one function naming every perturbation is what makes the count checkable against the check list, and splitting it would let a dropped case pass unnoticed"
)]
pub fn run(record: &DeliveredRealizationRecord) -> Vec<Perturbation> {
    let mut observed = Vec::new();
    let bytes = record.canonical_bytes();
    assert!(decode(&bytes).is_ok(), "the baseline record must decode");

    // --- framing -----------------------------------------------------------
    let mut corrupt = bytes.clone();
    corrupt[0] ^= 0xff;
    observed.push(expect_decode_failure(
        "domain separator",
        &corrupt,
        "bad-realization-domain",
    ));

    observed.push(expect_decode_failure(
        "truncated record",
        &bytes[..bytes.len() - 1],
        "truncated-realization-record",
    ));

    let mut trailing = bytes.clone();
    trailing.push(0x00);
    observed.push(expect_decode_failure(
        "trailing bytes",
        &trailing,
        "trailing-realization-bytes",
    ));

    // --- unknown tags, each fail-closed ------------------------------------
    let family = subject_family_offset(record);
    let mut corrupt = bytes.clone();
    corrupt[family] = 0xee;
    let error = decode(&corrupt).expect_err("an unknown record family");
    assert_eq!(
        error,
        RealizationCodecError::UnknownTag {
            subject: TagSubject::RecordFamily,
            tag: 0xee,
        },
        "an unknown family must reject rather than be skipped"
    );
    observed.push(Perturbation {
        name: "unknown record family",
        observed: rule_of(&error),
    });

    let mut corrupt = bytes.clone();
    corrupt[family + 1] = 0xee;
    observed.push(expect_decode_failure(
        "unknown arithmetic type",
        &corrupt,
        "unknown-realization-tag",
    ));

    // The first dimension tag sits after the family tag, the arithmetic tag, and
    // the framed resolved-type identity.
    let identity_len = record.subjects()[0]
        .scalar_arithmetic()
        .expect("a scalar subject")
        .subject()
        .resolved_type_identity()
        .len();
    let first_dimension = family + 1 + 1 + 8 + identity_len;
    let mut corrupt = bytes.clone();
    corrupt[first_dimension] = 0xee;
    observed.push(expect_decode_failure(
        "unknown dimension tag",
        &corrupt,
        "incomplete-dimension-coverage",
    ));

    let mut corrupt = bytes.clone();
    corrupt[first_dimension + 1] = 0xee;
    observed.push(expect_decode_failure(
        "unknown behaviour space",
        &corrupt,
        "unknown-realization-tag",
    ));

    assert!(
        !record.evidence().is_empty(),
        "the baseline fixture must carry evidence for the means perturbation to reach"
    );
    {
        let means = first_means_offset(record);
        let mut corrupt = bytes.clone();
        corrupt[means] = 0xee;
        observed.push(expect_decode_failure(
            "unknown honouring means",
            &corrupt,
            "unknown-realization-tag",
        ));
    }

    // --- canonical order ---------------------------------------------------
    let (evidence, subjects, obligations, bindings) = parts(record);
    assert!(
        obligations.len() >= 2,
        "the order perturbations need at least two obligation rows"
    );
    {
        let mut shuffled = obligations.clone();
        shuffled.reverse();
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            evidence.clone(),
            subjects.clone(),
            shuffled,
            bindings.clone(),
        )
        .canonical_bytes();
        let error = decode(&image).expect_err("shuffled obligations");
        assert!(
            matches!(
                error,
                RealizationCodecError::NonCanonicalOrder {
                    subject: OrderedSubject::Obligations,
                    ..
                }
            ),
            "shuffled obligations must reject as non-canonical, got {error}"
        );
        observed.push(Perturbation {
            name: "shuffled obligations",
            observed: rule_of(&error),
        });

        let mut duplicated = obligations.clone();
        duplicated[1] = duplicated[0].clone();
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            evidence.clone(),
            subjects.clone(),
            duplicated,
            bindings.clone(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "duplicate obligation",
            &image,
            "non-canonical-realization-order",
        ));
    }

    assert!(
        evidence.len() >= 2,
        "the evidence-order perturbation needs at least two rows"
    );
    {
        let mut shuffled = evidence.clone();
        shuffled.reverse();
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            shuffled,
            subjects.clone(),
            obligations.clone(),
            bindings.clone(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "shuffled evidence",
            &image,
            "non-canonical-realization-order",
        ));
    }

    // --- missing rows ------------------------------------------------------
    let image = DeliveredRealizationRecord::from_canonical_parts(
        record.profile().clone(),
        evidence.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .canonical_bytes();
    observed.push(expect_decode_failure(
        "missing policy subjects",
        &image,
        "no-policy-subjects",
    ));

    // --- dangling references ----------------------------------------------
    {
        let first = obligations.first().expect("a row to dangle");
        let dangling = NumericalObligation::from_canonical_parts(
            u32::try_from(subjects.len()).expect("a small table"),
            first.dimension(),
            first.locus(),
            first.required(),
            first.evidence(),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            evidence.clone(),
            subjects.clone(),
            vec![dangling],
            bindings.clone(),
        )
        .canonical_bytes();
        let error = decode(&image).expect_err("a dangling obligation subject");
        assert!(
            matches!(
                error,
                RealizationCodecError::DanglingReference {
                    subject: ReferenceSubject::ObligationSubject,
                    ..
                }
            ),
            "a dangling subject must be named, got {error}"
        );
        observed.push(Perturbation {
            name: "dangling obligation subject",
            observed: rule_of(&error),
        });

        let dangling = NumericalObligation::from_canonical_parts(
            first.subject(),
            first.dimension(),
            first.locus(),
            first.required(),
            u32::try_from(evidence.len()).expect("a small table"),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            evidence.clone(),
            subjects.clone(),
            vec![dangling],
            bindings.clone(),
        )
        .canonical_bytes();
        let error = decode(&image).expect_err("a dangling evidence reference");
        assert!(
            matches!(
                error,
                RealizationCodecError::DanglingReference {
                    subject: ReferenceSubject::ObligationEvidence,
                    ..
                }
            ),
            "a dangling evidence reference must be named, got {error}"
        );
        observed.push(Perturbation {
            name: "dangling evidence reference",
            observed: rule_of(&error),
        });
    }

    {
        let binding = bindings.first().expect("a binding to dangle");
        let dangling = EntryPolicyBinding::new(
            binding.entry(),
            u32::try_from(subjects.len()).expect("a small table"),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            evidence.clone(),
            subjects.clone(),
            obligations.clone(),
            vec![dangling],
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "dangling entry binding",
            &image,
            "dangling-realization-reference",
        ));
    }

    // --- disposition coverage ---------------------------------------------
    // Dropping every obligation while the subject still claims `Required` is the
    // exact shape a producer would reach by translating dispositions separately
    // from the rows they name.
    assert!(
        !obligations.is_empty(),
        "a required range needs rows to lose"
    );
    {
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            evidence.clone(),
            subjects.clone(),
            Vec::new(),
            bindings.clone(),
        )
        .canonical_bytes();
        let error = decode(&image).expect_err("a required range over no obligations");
        assert!(
            matches!(
                error,
                RealizationCodecError::DispositionCoverageMismatch { .. }
                    | RealizationCodecError::DanglingReference {
                        subject: ReferenceSubject::DispositionRange,
                        ..
                    }
            ),
            "an uncovered required range must reject, got {error}"
        );
        observed.push(Perturbation {
            name: "required range over no obligations",
            observed: rule_of(&error),
        });
    }

    // A range that is perfectly in-bounds and names another dimension's rows.
    // Dropping the rows entirely (above) trips the bounds check first, and
    // retargeting the obligations alone trips the space or association check, so
    // the coverage check needs a record whose every *local* invariant holds and
    // whose meaning is still wrong. Evidence and obligations both move to one
    // unused dimension: each row is well formed, each association is exact, the
    // ranges stay in bounds — and the subject still claims dimensions that now
    // have no rows. That is precisely the shape a producer reaches by carrying
    // dispositions separately from the rows they name.
    {
        let forbidden = DimensionBehaviour::Transform(NumericalPermission::Forbidden);
        let moved_evidence = vec![TargetEvidence::from_canonical_parts(
            0,
            NumericalDimension::Reassociation,
            forbidden,
            crate::shared::HonouringMeans::SupportedExactly,
            record.profile().clone(),
            fixtures::measured_source("1.0", "spike-build-a"),
        )];
        let retargeted: Vec<NumericalObligation> = (0..obligations.len())
            .map(|index| {
                NumericalObligation::from_canonical_parts(
                    0,
                    NumericalDimension::Reassociation,
                    fixtures::computation(u32::try_from(index).expect("a small table") + 100),
                    forbidden,
                    0,
                )
            })
            .collect();
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            moved_evidence,
            subjects.clone(),
            retargeted,
            bindings.clone(),
        )
        .canonical_bytes();
        let error = decode(&image).expect_err("an in-bounds range over the wrong rows");
        assert!(
            matches!(
                error,
                RealizationCodecError::DispositionCoverageMismatch { .. }
            ),
            "a range naming the wrong rows must reject as a coverage mismatch, got {error}"
        );
        observed.push(Perturbation {
            name: "required range naming another dimension's rows",
            observed: rule_of(&error),
        });
    }

    // --- a locus shape no Rust value can hold ------------------------------
    // `NumericalObligationKey::new` forces a zero ordinal on every non-component
    // locus, so the malformed row is unrepresentable in the type system and only
    // the wire can carry one. The perturbation retags a component locus as a
    // computation locus while its ordinal stays nonzero.
    {
        let component = obligations
            .iter()
            .find(|row| row.locus().component_ordinal() != 0)
            .expect("the baseline fixture carries a component-locus obligation");
        let mut needle = Vec::new();
        component.locus().encode(&mut needle);
        let at = bytes
            .windows(needle.len())
            .position(|window| window == needle.as_slice())
            .expect("the component locus is encoded in the record");
        let mut corrupt = bytes.clone();
        // Byte 4 of the nine-byte locus run is the locus tag.
        corrupt[at + 4] = crate::shared::PolicyLocus::Computation.tag();
        observed.push(expect_decode_failure(
            "non-component locus carrying an ordinal",
            &corrupt,
            "malformed-obligation-key",
        ));
    }

    // --- wire-only rejections the builder cannot produce -------------------
    // Each shape below is unrepresentable through the typed producer path, so
    // only a hostile or corrupted wire image can carry one. That is exactly why
    // decode has to check them independently rather than trusting that a record
    // came from the builder.
    {
        let subject_index = 0_u32;
        let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
        let forbidden = DimensionBehaviour::Transform(NumericalPermission::Forbidden);

        // A behaviour from another dimension's space.
        let evidence_row = TargetEvidence::from_canonical_parts(
            subject_index,
            NumericalDimension::Contraction,
            preserve,
            crate::shared::HonouringMeans::SupportedExactly,
            record.profile().clone(),
            fixtures::measured_source("1.0", "spike-build-a"),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            vec![evidence_row],
            subjects.clone(),
            Vec::new(),
            Vec::new(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "wire behaviour from another space",
            &image,
            "behaviour-space-mismatch",
        ));

        // An obligation whose evidence speaks about a different behaviour.
        let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: tiler_ir::schedule::FlushedZeroSign::PreservesSign,
        });
        let evidence_row = TargetEvidence::from_canonical_parts(
            subject_index,
            NumericalDimension::InputSubnormals,
            flush,
            crate::shared::HonouringMeans::SupportedExactly,
            record.profile().clone(),
            fixtures::measured_source("1.0", "spike-build-a"),
        );
        let obligation = NumericalObligation::from_canonical_parts(
            subject_index,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            preserve,
            0,
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            vec![evidence_row],
            subjects.clone(),
            vec![obligation],
            Vec::new(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "wire evidence behaviour mismatch",
            &image,
            "evidence-behaviour-mismatch",
        ));

        // Provenance whose authority triple names no readable moment.
        let evidence_row = TargetEvidence::from_canonical_parts(
            subject_index,
            NumericalDimension::Contraction,
            forbidden,
            crate::shared::HonouringMeans::SupportedExactly,
            record.profile().clone(),
            FactSourceProvenance::new(
                AvailabilityPhase::CompileProfile,
                crate::shared::FactAuthority::LaunchInstance,
                crate::shared::FactValidityScope::MeasuredEnvironment,
                crate::shared::ProvenanceIdentity::new("tiler.spike.measured-authority.v1", 1),
                crate::shared::FactEvidenceBasis::Measurement {
                    contexts: Vec::new(),
                },
            ),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            vec![evidence_row],
            subjects.clone(),
            Vec::new(),
            Vec::new(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "wire incomplete provenance",
            &image,
            "incomplete-provenance",
        ));

        // Evidence readable only from a phase after packaging.
        let evidence_row = TargetEvidence::from_canonical_parts(
            subject_index,
            NumericalDimension::Contraction,
            forbidden,
            crate::shared::HonouringMeans::SupportedExactly,
            record.profile().clone(),
            FactSourceProvenance::new(
                AvailabilityPhase::LiveDevicePreflight,
                crate::shared::FactAuthority::DeviceRuntime,
                crate::shared::FactValidityScope::DeviceInstance,
                crate::shared::ProvenanceIdentity::new("tiler.spike.device-authority.v1", 1),
                crate::shared::FactEvidenceBasis::Measurement {
                    contexts: vec![crate::shared::MeasurementContext::new(
                        vec![crate::shared::CompilerBuildIdentity::new(
                            crate::shared::CompilerBuildRole::RuntimeCompiler,
                            "spike-runtime-compiler",
                            "1.0",
                            None,
                        )],
                        crate::shared::ExecutionEnvironmentIdentity::new(
                            "spike-platform",
                            "1.0",
                            "spike-build-a",
                            "spike-architecture",
                            "spike-hardware",
                        ),
                    )],
                },
            ),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            vec![evidence_row],
            subjects.clone(),
            Vec::new(),
            Vec::new(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "wire fact phase escape",
            &image,
            "means-fact-phase-escape",
        ));
    }

    // A `Required` range naming zero rows. The builder derives ranges from the
    // obligations that exist, so it can never mint one; only a wire image can.
    {
        let scalar = subjects[0]
            .scalar_arithmetic()
            .expect("the only implemented family");
        let mut resolutions = [DimensionBehaviour::Subnormals(SubnormalMode::Preserve); 11];
        let mut dispositions = [crate::record::AssessmentDisposition::NotRequired; 11];
        for dimension in CANONICAL_DIMENSIONS {
            resolutions[dimension.index()] = scalar.resolution(dimension);
        }
        dispositions[NumericalDimension::InputSubnormals.index()] =
            crate::record::AssessmentDisposition::Required { first: 0, len: 0 };
        let empty = crate::record::NumericalPolicySubject::ScalarArithmetic(
            crate::record::ScalarArithmeticRecord::from_canonical_parts(
                scalar.subject().clone(),
                resolutions,
                dispositions,
            ),
        );
        let image = DeliveredRealizationRecord::from_canonical_parts(
            record.profile().clone(),
            Vec::new(),
            vec![empty],
            Vec::new(),
            Vec::new(),
        )
        .canonical_bytes();
        observed.push(expect_decode_failure(
            "empty required range",
            &image,
            "empty-required-range",
        ));
    }

    // A profile key the governed grammar refuses. Poked rather than built,
    // because `TargetProfileKey::new` is what refuses it.
    {
        let mut corrupt = bytes.clone();
        let key_at = DELIVERED_REALIZATION_DOMAIN.len() + 8;
        corrupt[key_at] = b'A';
        observed.push(expect_decode_failure(
            "malformed profile key",
            &corrupt,
            "malformed-realization-identity",
        ));
    }

    // --- artifact cross-checks --------------------------------------------
    let decoded = decode(&bytes).expect("the baseline record decodes");
    let other = fixtures::other_profile();
    let entries = [fixtures::strict_realization()];
    let error = validate_against_artifact(
        &decoded,
        &ArtifactCrossCheck {
            profile: &other,
            entries: &entries,
        },
    )
    .expect_err("a profile mismatch");
    assert_eq!(rule_of(&error), "realization-profile-mismatch");
    observed.push(Perturbation {
        name: "artifact profile mismatch",
        observed: rule_of(&error),
    });

    let profile = fixtures::profile();
    let mut divergent = fixtures::strict_realization();
    divergent.contraction = NumericalPermission::Permitted;
    let entries = [divergent];
    let error = validate_against_artifact(
        &decoded,
        &ArtifactCrossCheck {
            profile: &profile,
            entries: &entries,
        },
    )
    .expect_err("an overlapping realization mismatch");
    assert_eq!(rule_of(&error), "overlapping-realization-mismatch");
    observed.push(Perturbation {
        name: "overlapping realization mismatch",
        observed: rule_of(&error),
    });

    let entries = [
        fixtures::strict_realization(),
        fixtures::strict_realization(),
    ];
    let error = validate_against_artifact(
        &decoded,
        &ArtifactCrossCheck {
            profile: &profile,
            entries: &entries,
        },
    )
    .expect_err("an unbound entry");
    assert_eq!(rule_of(&error), "unbound-entry");
    observed.push(Perturbation {
        name: "unbound packaged entry",
        observed: rule_of(&error),
    });

    // --- builder-side checks ----------------------------------------------
    observed.extend(builder_perturbations());

    // The population is counted against a named inventory rather than against
    // however many perturbations happen to exist. A rule added without a
    // perturbation fails here, which is the difference between "every check has
    // been watched saying no" and "every check that had a perturbation did".
    let seen: Vec<&str> = observed
        .iter()
        .map(|entry| entry.observed.as_str())
        .collect();
    let mut unexercised = Vec::new();
    for rule in RealizationCodecError::ALL_RULES {
        if !seen.contains(&rule) {
            unexercised.push(rule);
        }
    }
    for rule in crate::record::DeliveredRealizationError::ALL_RULES {
        if !seen.contains(&rule) {
            unexercised.push(rule);
        }
    }
    assert!(
        unexercised.is_empty(),
        "these rules were never watched refusing: {unexercised:?}"
    );
    observed
}

/// The builder's own refusals, each watched failing.
#[allow(
    clippy::too_many_lines,
    reason = "the population is the point, exactly as it is for `run`: one function naming every builder refusal is what the coverage assertion counts against"
)]
fn builder_perturbations() -> Vec<Perturbation> {
    let mut observed = Vec::new();
    let subject = fixtures::f32_subject().identity();
    let strict = fixtures::strict_resolutions();

    let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
    builder
        .declare_scalar_arithmetic(subject.clone(), strict)
        .expect("a well-formed contract");
    let error = builder
        .declare_scalar_arithmetic(subject.clone(), strict)
        .expect_err("a redeclared subject");
    assert_eq!(error.rule(), "subject-redeclared");
    observed.push(Perturbation {
        name: "subject redeclared",
        observed: error.rule().to_owned(),
    });

    // A resolution from the wrong behaviour space.
    let mut wrong = strict;
    wrong[NumericalDimension::Contraction.index()] =
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
    let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
    let error = builder
        .declare_scalar_arithmetic(subject.clone(), wrong)
        .expect_err("a resolution from another space");
    assert_eq!(error.rule(), "resolution-space-mismatch");
    observed.push(Perturbation {
        name: "resolution space mismatch",
        observed: error.rule().to_owned(),
    });

    let strict_input = strict[NumericalDimension::InputSubnormals.index()];

    // An obligation for an undeclared subject.
    let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            fixtures::exact_evidence(strict_input),
        )
        .expect_err("an obligation before any contract");
    assert_eq!(error.rule(), "unknown-policy-subject");
    observed.push(Perturbation {
        name: "obligation for an undeclared subject",
        observed: error.rule().to_owned(),
    });

    let mut base = DeliveredRealizationBuilder::new(fixtures::profile());
    base.declare_scalar_arithmetic(subject.clone(), strict)
        .expect("a well-formed contract");

    // A repeated (subject, dimension, locus).
    let mut builder = base.clone();
    builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            fixtures::exact_evidence(strict_input),
        )
        .expect("a well-formed obligation");
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            fixtures::exact_evidence(strict_input),
        )
        .expect_err("a repeated locus");
    assert_eq!(error.rule(), "obligation-redeclared");
    observed.push(Perturbation {
        name: "obligation redeclared at one locus",
        observed: error.rule().to_owned(),
    });

    // Evidence speaking about another behaviour.
    let mut builder = base.clone();
    let other_behaviour = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: tiler_ir::schedule::FlushedZeroSign::PreservesSign,
    });
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            fixtures::exact_evidence(other_behaviour),
        )
        .expect_err("evidence about another behaviour");
    assert_eq!(error.rule(), "evidence-behaviour-mismatch");
    observed.push(Perturbation {
        name: "evidence behaviour mismatch",
        observed: error.rule().to_owned(),
    });

    // Evidence naming another profile.
    let mut builder = base.clone();
    let mut foreign = fixtures::exact_evidence(strict_input);
    foreign.profile = fixtures::other_profile();
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            foreign,
        )
        .expect_err("evidence from another profile");
    assert_eq!(error.rule(), "evidence-profile-mismatch");
    observed.push(Perturbation {
        name: "evidence profile mismatch",
        observed: error.rule().to_owned(),
    });

    // Provenance that is structurally incomplete: a measurement basis whose
    // authority triple names no readable moment.
    let mut builder = base.clone();
    let mut incomplete = fixtures::exact_evidence(strict_input);
    incomplete.source = FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        crate::shared::FactAuthority::LaunchInstance,
        crate::shared::FactValidityScope::MeasuredEnvironment,
        crate::shared::ProvenanceIdentity::new("tiler.spike.measured-authority.v1", 1),
        crate::shared::FactEvidenceBasis::Measurement {
            contexts: Vec::new(),
        },
    );
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            incomplete,
        )
        .expect_err("incomplete provenance");
    assert_eq!(error.rule(), "incomplete-provenance");
    observed.push(Perturbation {
        name: "incomplete provenance",
        observed: error.rule().to_owned(),
    });

    // Evidence readable only after the artifact was produced.
    let mut builder = base.clone();
    let mut late = fixtures::exact_evidence(strict_input);
    late.source = FactSourceProvenance::new(
        AvailabilityPhase::LaunchPreflight,
        crate::shared::FactAuthority::LaunchInstance,
        crate::shared::FactValidityScope::LaunchInstance,
        crate::shared::ProvenanceIdentity::new("tiler.spike.launch-authority.v1", 1),
        crate::shared::FactEvidenceBasis::Measurement {
            contexts: vec![crate::shared::MeasurementContext::new(
                vec![crate::shared::CompilerBuildIdentity::new(
                    crate::shared::CompilerBuildRole::RuntimeCompiler,
                    "spike-runtime-compiler",
                    "1.0",
                    None,
                )],
                crate::shared::ExecutionEnvironmentIdentity::new(
                    "spike-platform",
                    "1.0",
                    "spike-build-a",
                    "spike-architecture",
                    "spike-hardware",
                ),
            )],
        },
    );
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            strict_input,
            late,
        )
        .expect_err("a phase escape");
    assert_eq!(error.rule(), "means-fact-phase-escape");
    observed.push(Perturbation {
        name: "fact phase escape",
        observed: error.rule().to_owned(),
    });

    // An entry bound to a subject no contract declared.
    let mut builder = base.clone();
    let error = builder
        .bind_entry(0, &fixtures::f16_subject().identity())
        .expect_err("an entry bound to an undeclared subject");
    assert_eq!(error.rule(), "unknown-policy-subject");
    observed.push(Perturbation {
        name: "entry bound to an undeclared subject",
        observed: error.rule().to_owned(),
    });

    // A rebound entry.
    let mut builder = base.clone();
    builder.bind_entry(0, &subject).expect("a first binding");
    let error = builder
        .bind_entry(0, &subject)
        .expect_err("a rebound entry");
    assert_eq!(error.rule(), "entry-rebound");
    observed.push(Perturbation {
        name: "entry rebound",
        observed: error.rule().to_owned(),
    });

    // An empty record.
    let error = DeliveredRealizationBuilder::new(fixtures::profile())
        .build()
        .expect_err("a record with no subject");
    assert_eq!(error.rule(), "no-policy-subjects");
    observed.push(Perturbation {
        name: "no policy subjects",
        observed: error.rule().to_owned(),
    });

    observed
}

/// Asserts that every dimension of every subject is reachable and total.
///
/// # Panics
///
/// Panics when a dimension is missing, which the dense array makes
/// unrepresentable — the assertion is the runtime witness of a compile-time
/// property, and it counts its population rather than trusting the loop ran.
pub fn assert_total_coverage(record: &DeliveredRealizationRecord) {
    let mut counted = 0_usize;
    for subject in record.subjects() {
        let scalar = subject
            .scalar_arithmetic()
            .expect("the only implemented family");
        for dimension in CANONICAL_DIMENSIONS {
            let behaviour = scalar.resolution(dimension);
            assert!(
                dimension.admits(behaviour),
                "{dimension} carries a behaviour from another space"
            );
            counted += 1;
        }
    }
    assert_eq!(
        counted,
        record.subjects().len() * CANONICAL_DIMENSIONS.len(),
        "coverage counted a different population than it walked"
    );
}
