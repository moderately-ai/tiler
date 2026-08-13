//! The delivered-realization record's fixtures and its refusal evidence.
//!
//! Every check this module's record and codec define is perturbed once and
//! observed failing on its **exact** rule identifier — not merely observed
//! failing, because a perturbation that trips a neighbouring check reports a
//! pass for a check that was never exercised. The population is then counted
//! against both `ALL_RULES` inventories, so a rule added without a perturbation
//! fails this suite rather than quietly shrinking what has been watched
//! refusing.
//!
//! Two perturbation shapes are used, deliberately.
//!
//! - **Structural** perturbations rebuild a deliberately non-canonical record
//!   through `from_canonical_parts` and re-encode it with the one production
//!   encoder, so a perturbation cannot pass by disagreeing with the encoder it
//!   is supposed to be testing.
//! - **Tag** perturbations poke one byte at an offset computed from the record's
//!   own field widths, because an unknown tag is by construction a value no Rust
//!   value can hold.
//!
//! Seven shapes are **wire-only** — a behaviour from another space, an evidence
//! behaviour mismatch, incomplete provenance, a phase escape, an empty
//! `Required` range, a non-component locus carrying an ordinal, and an
//! unsupported fact-source provenance schema. The typed producer path cannot
//! express any of them, which is precisely why decode has to check them
//! independently rather than trusting that a record came from the builder.
//!
//! # Measurement boundary
//!
//! The evidence rows below are **checked synthetic** evidence. They are
//! structurally valid and they are not measurements, and one consequence must
//! not be read past: **no target profile in this tree declares an `f16`
//! honourability row**. The two-dtype fixture therefore proves a property of the
//! *record* — that it carries two dtypes' subnormal evidence without collision —
//! and proves nothing about any measured target.

use tiler_ir::numerics::{
    CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole, DIMENSION_COUNT,
    DimensionBehaviour, ExecutionEnvironmentIdentity, FACT_SOURCE_PROVENANCE_SCHEMA_VERSION,
    FactAuthority, FactEvidenceBasis, FactSourceProvenance, FactValidityScope, HonouringMeans,
    MeasurementContext, NumericalDimension, NumericalObligationKey, PolicyLocus,
    ProvenanceIdentity, RelaxationRequirement, ScalarArithmeticSubject,
};
use tiler_ir::program::SemanticOccurrence;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, NumericalRealization, SubnormalMode,
};
use tiler_ir::semantic::{ResolvedValueType, StrictAffineU4, TypeKey, complex_value_type};

use super::codec::{
    ArtifactCrossCheck, OrderedSubject, RealizationCodecError, ReferenceSubject, TagSubject,
    decode, validate_against_artifact,
};
use super::{
    AssessmentDisposition, DELIVERED_REALIZATION_DOMAIN, DeliveredRealizationBuilder,
    DeliveredRealizationError, DeliveredRealizationRecord, DispositionView, EntryPolicyBinding,
    EntryRealization, NumericalObligation, NumericalPolicySubject, ScalarArithmeticRecord,
    TargetEvidence, TargetEvidenceDeclaration,
};
use crate::program::keys::{TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// The governed nominal identity of one built-in scalar.
fn governed_scalar(name: &str) -> ResolvedValueType {
    ResolvedValueType::nominal(
        TypeKey::new("tiler", name, 1).expect("a governed catalog name is a valid identity"),
    )
}

/// The governed `f32` subject, built through the production constructor.
fn f32_subject() -> ScalarArithmeticSubject {
    ScalarArithmeticSubject::f32()
}

/// The governed `f16` subject, built through the production constructor.
fn f16_subject() -> ScalarArithmeticSubject {
    ScalarArithmeticSubject::new(ArithmeticType::F16, governed_scalar("f16"))
        .expect("the catalog registers f16 over tiler::f16@1")
}

/// The fixture target profile every record below is attributed to.
fn profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.delivered-realization.v1")
            .expect("a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02, 0x03])
            .expect("descriptor bytes"),
    }
}

/// A second profile, differing only in its key.
fn other_profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.delivered-realization.v2")
            .expect("a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02, 0x03])
            .expect("descriptor bytes"),
    }
}

/// A complete, structurally valid measured provenance statement.
fn measured_source(compiler_version: &str, platform_build: &str) -> FactSourceProvenance {
    FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.test.measured-authority.v1", 1),
        FactEvidenceBasis::Measurement {
            contexts: vec![MeasurementContext::new(
                vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::CodeGenerator,
                    "test-offline-compiler",
                    compiler_version,
                    None,
                )],
                ExecutionEnvironmentIdentity::new(
                    "test-platform",
                    "1.0",
                    platform_build,
                    "test-architecture",
                    "test-hardware",
                ),
            )],
        },
    )
}

/// A governed-guarantee provenance statement.
fn governed_source() -> FactSourceProvenance {
    FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        FactAuthority::GovernedProfile,
        FactValidityScope::PortableProfile,
        ProvenanceIdentity::new("tiler.test.governed-authority.v1", 1),
        FactEvidenceBasis::GovernedGuarantee {
            guarantee: ProvenanceIdentity::new("tiler.test.governed-guarantee.v1", 1),
        },
    )
}

/// The eleven strict resolutions, in canonical dimension order.
///
/// A dense array literal of exactly [`DIMENSION_COUNT`] entries: widening the
/// vocabulary is a build error here, which is the completeness property the
/// eleven named fields were eliminated in favour of.
fn strict_resolutions() -> [DimensionBehaviour; DIMENSION_COUNT] {
    [
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden),
        DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven),
    ]
}

/// The eleven resolutions of a flushing contract, in canonical dimension order.
fn flushing_resolutions() -> [DimensionBehaviour; DIMENSION_COUNT] {
    let mut resolutions = strict_resolutions();
    let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    });
    resolutions[NumericalDimension::InputSubnormals.index()] = flush;
    resolutions[NumericalDimension::ResultSubnormals.index()] = flush;
    resolutions
}

/// The scheduled realization matching [`strict_resolutions`] over the eight
/// dimensions a region carries.
const fn strict_realization() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.contract.v1",
        0x7fc0_0000,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

/// One evidence declaration honouring `behaviour` exactly.
fn exact_evidence(behaviour: DimensionBehaviour) -> TargetEvidenceDeclaration {
    TargetEvidenceDeclaration {
        declared: behaviour,
        means: HonouringMeans::SupportedExactly,
        profile: profile(),
        source: measured_source("1.0", "test-build-a"),
    }
}

/// One evidence declaration honouring `behaviour` only under a named relaxation.
///
/// The `relaxed_on` dimension and `relaxed_to` behaviour are the payload two
/// otherwise identical conditional means differ in — the exact distinction
/// [`HonouringMeans::label`] erases and this record preserves.
fn relaxation_evidence(
    behaviour: DimensionBehaviour,
    relaxed_on: NumericalDimension,
    relaxed_to: DimensionBehaviour,
) -> TargetEvidenceDeclaration {
    TargetEvidenceDeclaration {
        declared: behaviour,
        means: HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
            relaxation: RelaxationRequirement::new(
                f32_subject().identity(),
                relaxed_on,
                relaxed_to,
            ),
        },
        profile: profile(),
        source: governed_source(),
    }
}

/// One obligation key at the computation locus of an occurrence.
const fn computation(occurrence: u32) -> NumericalObligationKey {
    NumericalObligationKey::new(
        SemanticOccurrence::new(occurrence),
        PolicyLocus::Computation,
    )
}

/// One obligation key at the accumulator locus of an occurrence.
const fn accumulator(occurrence: u32) -> NumericalObligationKey {
    NumericalObligationKey::new(
        SemanticOccurrence::new(occurrence),
        PolicyLocus::Accumulator,
    )
}

/// One obligation key at an ordered component of an occurrence's compound value.
const fn component(occurrence: u32, ordinal: u32) -> NumericalObligationKey {
    NumericalObligationKey::component(SemanticOccurrence::new(occurrence), ordinal)
}

/// One obligation key at the observable materialization locus of an occurrence.
const fn materialization(occurrence: u32) -> NumericalObligationKey {
    NumericalObligationKey::new(
        SemanticOccurrence::new(occurrence),
        PolicyLocus::Materialization,
    )
}

/// The reference record: one `f32` subject over all eleven dimensions.
///
/// Input subnormals required at three distinct loci, two of which carry
/// different legal requirements; contraction required at one locus; result
/// subnormals required at one *component* locus carrying a nonzero ordinal —
/// the only locus that may, which is what makes it the row the malformed-key
/// perturbation retags; every other dimension `NotRequired`.
fn reference_record() -> DeliveredRealizationRecord {
    let subject = f32_subject();
    let resolutions = strict_resolutions();
    let preserve = resolutions[NumericalDimension::InputSubnormals.index()];
    let forbidden = resolutions[NumericalDimension::Contraction.index()];
    let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    });

    let mut builder = DeliveredRealizationBuilder::new(profile());
    builder
        .declare_scalar_arithmetic(subject.identity(), resolutions)
        .expect("the selected contract");

    // Three distinct loci on one (type, dimension). The materialization locus
    // legitimately tolerates a flush where the computation locus does not, which
    // is the case a dtype-wide ceiling alone would have collapsed.
    for (locus, required) in [
        (computation(0), preserve),
        (accumulator(0), preserve),
        (materialization(1), flush),
    ] {
        builder
            .require(
                &subject.identity(),
                NumericalDimension::InputSubnormals,
                locus,
                required,
                exact_evidence(required),
            )
            .expect("a well-formed obligation");
    }
    builder
        .require(
            &subject.identity(),
            NumericalDimension::Contraction,
            computation(0),
            forbidden,
            exact_evidence(forbidden),
        )
        .expect("a contraction obligation");
    builder
        .require(
            &subject.identity(),
            NumericalDimension::ResultSubnormals,
            component(2, 1),
            preserve,
            exact_evidence(preserve),
        )
        .expect("a component obligation");
    builder
        .bind_entry(0, &subject.identity())
        .expect("the one packaged entry");
    builder.build().expect("the reference record")
}

// ---------------------------------------------------------------------------
// The boundary's own properties
// ---------------------------------------------------------------------------

#[test]
fn a_recognized_value_type_is_not_a_calibrated_arithmetic_subject() {
    // The two governed pairs these fixtures use are admitted.
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F32, governed_scalar("f32")).is_ok());
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F16, governed_scalar("f16")).is_ok());

    // Every recognized non-scalar-float identity is refused for every arithmetic
    // type. The population is counted so a table that silently shrank could not
    // read as a pass.
    let rows: Vec<(&str, ResolvedValueType)> = vec![
        ("bool", governed_scalar("bool")),
        ("integer i32", governed_scalar("i32")),
        ("decimal64", governed_scalar("decimal64")),
        (
            "complex over f32",
            complex_value_type(&governed_scalar("f32")).expect("a governed complex identity"),
        ),
        ("strict-affine u4", StrictAffineU4::resolved_type()),
        ("mx element f8e4m3fn", governed_scalar("f8e4m3fn")),
    ];
    let mut refusals = 0_usize;
    for (name, value_type) in &rows {
        for arithmetic in ArithmeticType::ALL {
            assert!(
                ScalarArithmeticSubject::new(arithmetic, value_type.clone()).is_err(),
                "{name} must not calibrate a {arithmetic:?} arithmetic subject"
            );
            refusals += 1;
        }
    }
    assert_eq!(
        refusals,
        rows.len() * ArithmeticType::ALL.len(),
        "the refusal count must equal the population walked"
    );

    // An owner-namespaced identity no catalog registers cannot create a subject
    // either — recognition of a namespace is not evidence of calibration.
    let owner = ResolvedValueType::nominal(
        TypeKey::new("acme", "posit16", 1).expect("an owner namespace is a valid identity"),
    );
    for arithmetic in ArithmeticType::ALL {
        assert!(ScalarArithmeticSubject::new(arithmetic, owner.clone()).is_err());
    }

    // A width or class that merely resembles the arithmetic type is still
    // refused: `tiler::u32@1` is exactly f32's width and a different class,
    // while `tiler::f16@1` is exactly f32's class and a different width.
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F32, governed_scalar("u32")).is_err());
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F32, governed_scalar("f16")).is_err());
}

#[test]
fn a_complete_subject_answers_for_every_dimension_and_keeps_its_loci_apart() {
    let record = reference_record();
    assert_total_coverage(&record);

    let subject = f32_subject();
    let view = record
        .scalar_arithmetic(&subject.identity())
        .expect("the declared subject resolves");

    let mut not_required = 0_usize;
    let mut required = 0_usize;
    for dimension in CANONICAL_DIMENSIONS {
        match view.assessment(dimension) {
            DispositionView::NotRequired => not_required += 1,
            DispositionView::Required(rows) => {
                assert!(!rows.is_empty(), "a required range is never empty");
                required += 1;
            }
        }
    }
    assert_eq!(
        not_required + required,
        DIMENSION_COUNT,
        "every dimension has exactly one disposition"
    );
    assert!(
        required >= 1 && not_required >= 1,
        "the fixture covers both"
    );

    // Three distinct loci on one (subject, dimension), each with its own
    // required behaviour and evidence reference.
    let DispositionView::Required(rows) = view.assessment(NumericalDimension::InputSubnormals)
    else {
        panic!("input subnormals is required in this fixture");
    };
    assert_eq!(
        rows.len(),
        3,
        "three distinct loci on one (type, dimension)"
    );
    let mut loci: Vec<_> = rows.iter().map(NumericalObligation::locus).collect();
    loci.sort_unstable();
    loci.dedup();
    assert_eq!(loci.len(), 3, "the three loci are distinct");

    // The two f32 loci with *different* legal requirements did not collapse.
    let behaviours: Vec<_> = rows.iter().map(NumericalObligation::required).collect();
    assert!(
        behaviours.iter().any(|value| *value != behaviours[0]),
        "two loci with different legal requirements must not collapse"
    );
    for row in rows {
        assert_eq!(
            view.evidence_for(row).declared(),
            row.required(),
            "each obligation's evidence speaks about its own required behaviour"
        );
    }
}

#[test]
fn one_record_carries_two_dtypes_subnormal_evidence_without_collision() {
    let f32_subject = f32_subject();
    let f16_subject = f16_subject();

    let mut builder = DeliveredRealizationBuilder::new(profile());
    builder
        .declare_scalar_arithmetic(f32_subject.identity(), flushing_resolutions())
        .expect("the f32 contract flushes");
    builder
        .declare_scalar_arithmetic(f16_subject.identity(), strict_resolutions())
        .expect("the f16 contract preserves");

    let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    });
    let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
    builder
        .require(
            &f32_subject.identity(),
            NumericalDimension::InputSubnormals,
            computation(0),
            flush,
            exact_evidence(flush),
        )
        .expect("the f32 row");
    builder
        .require(
            &f16_subject.identity(),
            NumericalDimension::InputSubnormals,
            computation(1),
            preserve,
            exact_evidence(preserve),
        )
        .expect("the f16 row");
    let record = builder.build().expect("a two-subject record");

    let f32_view = record
        .scalar_arithmetic(&f32_subject.identity())
        .expect("the f32 subject resolves");
    let f16_view = record
        .scalar_arithmetic(&f16_subject.identity())
        .expect("the f16 subject resolves");
    assert_eq!(
        f32_view.resolution(NumericalDimension::InputSubnormals),
        flush
    );
    assert_eq!(
        f16_view.resolution(NumericalDimension::InputSubnormals),
        preserve
    );
    assert_eq!(record.evidence().len(), 2, "two dtypes, two evidence rows");
    assert_eq!(
        decode(&record.canonical_bytes()).expect("the two-subject record round-trips"),
        record,
    );
}

#[test]
fn two_conditional_means_differing_only_in_relaxation_stay_distinct() {
    let behaviour = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
    let permitted = DimensionBehaviour::Transform(NumericalPermission::Permitted);
    let first = relaxation_evidence(behaviour, NumericalDimension::Contraction, permitted);
    let second = relaxation_evidence(behaviour, NumericalDimension::Reassociation, permitted);

    assert_eq!(
        first.means.label(),
        second.means.label(),
        "the presentation label is documented as non-injective, and it is"
    );
    assert_ne!(
        first.means.canonical_key(),
        second.means.canonical_key(),
        "the canonical identity must distinguish two relaxations"
    );
    assert_ne!(first.means, second.means, "and so must equality");

    let subject = f32_subject();
    let build = |evidence| {
        let mut builder = DeliveredRealizationBuilder::new(profile());
        builder
            .declare_scalar_arithmetic(subject.identity(), strict_resolutions())
            .expect("a contract");
        builder
            .require(
                &subject.identity(),
                NumericalDimension::InputSubnormals,
                computation(0),
                behaviour,
                evidence,
            )
            .expect("an obligation");
        builder.build().expect("a record")
    };
    let left = build(first);
    assert_ne!(
        left.canonical_bytes(),
        build(second).canonical_bytes(),
        "two records differing only in a relaxation payload must not share bytes"
    );

    let decoded = decode(&left.canonical_bytes()).expect("a conditional means round-trips");
    let HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { relaxation } =
        decoded.evidence()[0].means()
    else {
        panic!("the decoded means must still be the conditional one");
    };
    assert_eq!(
        relaxation.dimension(),
        NumericalDimension::Contraction,
        "a reader can still say which relaxation made the requirement honourable"
    );
}

#[test]
fn declaration_order_does_not_reach_the_wire_and_duplicates_reject() {
    let subject = f32_subject();
    let strict = strict_resolutions();
    let preserve = strict[NumericalDimension::InputSubnormals.index()];
    let forbidden = strict[NumericalDimension::Contraction.index()];

    let declare = |order: [usize; 4]| {
        let mut builder = DeliveredRealizationBuilder::new(profile());
        builder
            .declare_scalar_arithmetic(subject.identity(), strict)
            .expect("a contract");
        let calls: [(
            NumericalDimension,
            NumericalObligationKey,
            DimensionBehaviour,
        ); 4] = [
            (
                NumericalDimension::InputSubnormals,
                computation(2),
                preserve,
            ),
            (
                NumericalDimension::InputSubnormals,
                accumulator(1),
                preserve,
            ),
            (NumericalDimension::Contraction, computation(0), forbidden),
            (
                NumericalDimension::InputSubnormals,
                computation(0),
                preserve,
            ),
        ];
        for index in order {
            let (dimension, locus, behaviour) = calls[index];
            builder
                .require(
                    &subject.identity(),
                    dimension,
                    locus,
                    behaviour,
                    exact_evidence(behaviour),
                )
                .expect("a well-formed obligation");
        }
        builder.build().expect("a record")
    };

    let forward = declare([0, 1, 2, 3]);
    assert_eq!(
        forward.canonical_bytes(),
        declare([3, 2, 1, 0]).canonical_bytes(),
        "declaration order must not reach the wire"
    );
    // Evidence identical across four obligations deduplicates to one row per
    // (subject, dimension, behaviour) triple the rows actually cite.
    assert_eq!(
        forward.evidence().len(),
        2,
        "identical evidence is carried once"
    );

    let mut builder = DeliveredRealizationBuilder::new(profile());
    builder
        .declare_scalar_arithmetic(subject.identity(), strict)
        .expect("a contract");
    builder
        .require(
            &subject.identity(),
            NumericalDimension::InputSubnormals,
            computation(0),
            preserve,
            exact_evidence(preserve),
        )
        .expect("a first obligation");
    assert_eq!(
        builder
            .require(
                &subject.identity(),
                NumericalDimension::InputSubnormals,
                computation(0),
                preserve,
                exact_evidence(preserve),
            )
            .expect_err("a duplicate")
            .rule(),
        "obligation-redeclared",
    );
    // The rejected insertion left the draft unchanged.
    assert!(builder.build().is_ok(), "the draft survived the rejection");
}

#[test]
fn a_selected_contract_with_no_obligations_is_still_a_complete_subject() {
    let subject = f32_subject();
    let mut builder = DeliveredRealizationBuilder::new(profile());
    builder
        .declare_scalar_arithmetic(subject.identity(), strict_resolutions())
        .expect("a selected contract");
    let record = builder
        .build()
        .expect("a complete subject with no obligations");

    assert_eq!(record.subjects().len(), 1);
    assert!(record.obligations().is_empty());
    let view = record
        .scalar_arithmetic(&subject.identity())
        .expect("the subject resolves");
    for dimension in CANONICAL_DIMENSIONS {
        assert_eq!(
            view.assessment(dimension),
            DispositionView::NotRequired,
            "{dimension} is NotRequired, and says so"
        );
        assert!(dimension.admits(view.resolution(dimension)));
    }

    // The `NotRequired` claims are written rather than recoverable from silence.
    let bytes = record.canonical_bytes();
    assert!(
        bytes.len() > DIMENSION_COUNT * 2,
        "every disposition writes its own byte"
    );
    assert_eq!(decode(&bytes).expect("it round-trips"), record);
}

#[test]
fn the_three_resolved_type_identity_families_stay_distinct() {
    let nominal = governed_scalar("f32");
    let parameterized = complex_value_type(&nominal).expect("a complex identity");
    let encoded = StrictAffineU4::resolved_type();

    let mut identities: Vec<Vec<u8>> = [&nominal, &parameterized, &encoded]
        .into_iter()
        .map(|value| value.canonical_encoding().as_bytes().to_vec())
        .collect();
    identities.sort();
    identities.dedup();
    assert_eq!(
        identities.len(),
        3,
        "nominal, parameterized, and encoded identities are three subjects"
    );

    // Two nominal identities differing only in namespace stay distinct, so an
    // owner-namespaced type can never alias a governed one.
    let owner =
        ResolvedValueType::nominal(TypeKey::new("acme", "f32", 1).expect("an owner namespace"));
    assert_ne!(
        owner.canonical_encoding().as_bytes(),
        nominal.canonical_encoding().as_bytes(),
        "a namespace is part of identity"
    );

    // None of the three inhabits the scalar-arithmetic schema.
    for arithmetic in ArithmeticType::ALL {
        assert!(ScalarArithmeticSubject::new(arithmetic, parameterized.clone()).is_err());
        assert!(ScalarArithmeticSubject::new(arithmetic, encoded.clone()).is_err());
    }
}

#[test]
fn the_record_round_trips_exactly_and_agrees_with_the_entry_it_binds() {
    let record = reference_record();
    let bytes = record.canonical_bytes();
    let decoded = decode(&bytes).expect("the record decodes");
    assert_eq!(decoded, record, "decode is the inverse of encode");
    assert_eq!(
        decoded.canonical_bytes(),
        bytes,
        "and re-encoding reproduces the exact bytes"
    );

    assert_eq!(
        decoded.evidence()[0].source().schema_version(),
        FACT_SOURCE_PROVENANCE_SCHEMA_VERSION,
        "the admitted schema survives decode"
    );

    let profile = profile();
    let entries = [EntryRealization::of(strict_realization())];
    validate_against_artifact(
        &decoded,
        &ArtifactCrossCheck {
            profile: &profile,
            entries: &entries,
        },
    )
    .expect("the record agrees with the artifact that would carry it");
}

#[test]
fn an_unsupported_provenance_schema_is_refused_before_the_body_is_read() {
    let record = reference_record();
    let bytes = record.canonical_bytes();
    let decoded = decode(&bytes).expect("the current-schema record decodes");
    assert_eq!(decoded, record);
    assert_eq!(decoded.canonical_bytes(), bytes);
    assert_eq!(
        decoded.evidence()[0].source().schema_version(),
        FACT_SOURCE_PROVENANCE_SCHEMA_VERSION
    );

    let at = first_provenance_schema_offset(&record);
    assert_eq!(
        &bytes[at..at + 4],
        FACT_SOURCE_PROVENANCE_SCHEMA_VERSION.to_be_bytes(),
        "the computed offset must land on the schema word"
    );

    let unknown = poke_provenance_schema(&bytes, &record, 1);
    let error = decode(&unknown).expect_err("schema 1 is unknown");
    assert_eq!(
        error,
        RealizationCodecError::UnknownProvenanceSchema { version: 1 }
    );
    assert_eq!(
        error.to_string(),
        "unsupported-provenance-schema: UnknownProvenanceSchema { version: 1 }"
    );

    let newer = poke_provenance_schema(&bytes, &record, 4);
    let error = decode(&newer).expect_err("schema 4 is newer");
    assert_eq!(
        error,
        RealizationCodecError::NewerProvenanceSchema { version: 4 }
    );
    assert_eq!(
        error.to_string(),
        "unsupported-provenance-schema: NewerProvenanceSchema { version: 4 }"
    );

    // Trash the first body byte after the schema word. If dispatch still
    // interpreted the body, this would be an unknown phase tag; the schema
    // refusal must win.
    let mut newer_and_broken = poke_provenance_schema(&bytes, &record, 4);
    newer_and_broken[at + 4] ^= 0xff;
    let error = decode(&newer_and_broken).expect_err("schema is refused before the body");
    assert_eq!(
        error,
        RealizationCodecError::NewerProvenanceSchema { version: 4 },
        "a damaged body must not be interpreted after an unsupported schema, got {error}"
    );

    let retired = RealizationCodecError::RetiredProvenanceSchema { version: 2 };
    assert_eq!(retired.rule(), "unsupported-provenance-schema");
    assert_eq!(
        retired.to_string(),
        "unsupported-provenance-schema: RetiredProvenanceSchema { version: 2 }"
    );
}

/// Asserts that every dimension of every subject is reachable and total.
///
/// The dense array makes a missing dimension unrepresentable; this is the
/// runtime witness of that compile-time property, and it counts its population
/// rather than trusting the loop ran.
fn assert_total_coverage(record: &DeliveredRealizationRecord) {
    let mut counted = 0_usize;
    for subject in record.subjects() {
        let scalar = subject
            .scalar_arithmetic()
            .expect("the only implemented family");
        for dimension in CANONICAL_DIMENSIONS {
            assert!(
                dimension.admits(scalar.resolution(dimension)),
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

// ---------------------------------------------------------------------------
// The perturbation table
// ---------------------------------------------------------------------------

/// One perturbation's outcome.
struct Perturbation {
    /// What was perturbed.
    name: &'static str,
    /// The rule identifier the check reported.
    observed: String,
}

/// Splits a record into its canonical parts so a perturbation can rebuild it.
fn parts(
    record: &DeliveredRealizationRecord,
) -> (
    Vec<TargetEvidence>,
    Vec<NumericalPolicySubject>,
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

/// The byte offset at which the subject table's first family tag sits.
///
/// Computed from the record's own leading field widths rather than searched for,
/// so a perturbation cannot silently start poking the wrong byte when a field
/// width changes: the offsets below go wrong loudly, as a decode that reports an
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

/// The byte offset of the first evidence row's fact-source provenance schema.
///
/// Derived from the row's own encoding minus its source tail, so a field
/// inserted ahead of the source moves this offset with it rather than leaving
/// the perturbation poking a neighbouring byte.
fn first_provenance_schema_offset(record: &DeliveredRealizationRecord) -> usize {
    let head = DELIVERED_REALIZATION_DOMAIN.len()
        + 8
        + record.profile().key.as_str().len()
        + 8
        + record.profile().descriptor.as_bytes().len()
        + 8; // evidence count
    let row = record.evidence().first().expect("a row to perturb");
    let row_bytes = row.canonical_key();
    let mut source = Vec::new();
    row.source().encode(&mut source);
    assert!(
        row_bytes.ends_with(source.as_slice()),
        "source.encode is the tail of the evidence row"
    );
    head + row_bytes.len() - source.len()
}

fn poke_provenance_schema(
    bytes: &[u8],
    record: &DeliveredRealizationRecord,
    version: u32,
) -> Vec<u8> {
    let at = first_provenance_schema_offset(record);
    let mut corrupt = bytes.to_vec();
    corrupt[at..at + 4].copy_from_slice(&version.to_be_bytes());
    corrupt
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

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "the population is the point: one test naming every perturbation is what makes the count checkable against the two rule inventories, and splitting it would let a dropped case pass unnoticed"
)]
fn every_check_is_watched_refusing_on_its_own_rule() {
    let record = reference_record();
    let mut observed = Vec::new();
    let bytes = record.canonical_bytes();
    assert!(decode(&bytes).is_ok(), "the baseline record must decode");

    // --- framing ---------------------------------------------------------
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

    // --- unknown tags, each fail-closed ----------------------------------
    let family = subject_family_offset(&record);
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

    // The first dimension tag sits after the family tag, the arithmetic tag,
    // and the framed resolved-type identity.
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
    let mut corrupt = bytes.clone();
    corrupt[first_means_offset(&record)] = 0xee;
    observed.push(expect_decode_failure(
        "unknown honouring means",
        &corrupt,
        "unknown-realization-tag",
    ));

    // --- canonical order --------------------------------------------------
    let (evidence, subjects, obligations, bindings) = parts(&record);
    assert!(
        obligations.len() >= 2,
        "the order perturbations need at least two obligation rows"
    );
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

    assert!(
        evidence.len() >= 2,
        "the evidence-order perturbation needs at least two rows"
    );
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

    // --- missing rows -----------------------------------------------------
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

    // --- dangling references ---------------------------------------------
    let first = obligations.first().expect("a row to dangle").clone();
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

    let binding = *bindings.first().expect("a binding to dangle");
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

    // --- disposition coverage --------------------------------------------
    // Dropping every obligation while the subject still claims `Required` is
    // the exact shape a producer would reach by translating dispositions
    // separately from the rows they name.
    assert!(
        !obligations.is_empty(),
        "a required range needs rows to lose"
    );
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

    // A range that is perfectly in-bounds and names another dimension's rows.
    // Dropping the rows entirely (above) trips the bounds check first, and
    // retargeting the obligations alone trips the space or association check,
    // so the coverage check needs a record whose every *local* invariant holds
    // and whose meaning is still wrong. Evidence and obligations both move to
    // one unused dimension: each row is well formed, each association is exact,
    // the ranges stay in bounds — and the subject still claims dimensions that
    // now have no rows.
    let forbidden = DimensionBehaviour::Transform(NumericalPermission::Forbidden);
    let moved_evidence = vec![TargetEvidence::from_canonical_parts(
        0,
        NumericalDimension::Reassociation,
        forbidden,
        HonouringMeans::SupportedExactly,
        record.profile().clone(),
        measured_source("1.0", "test-build-a"),
    )];
    let retargeted: Vec<NumericalObligation> = (0..obligations.len())
        .map(|index| {
            NumericalObligation::from_canonical_parts(
                0,
                NumericalDimension::Reassociation,
                computation(u32::try_from(index).expect("a small table") + 100),
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

    // --- a locus shape no Rust value can hold -----------------------------
    // `NumericalObligationKey::new` forces a zero ordinal on every
    // non-component locus, so the malformed row is unrepresentable in the type
    // system and only the wire can carry one. The perturbation retags a
    // component locus as a computation locus while its ordinal stays nonzero.
    let component_row = obligations
        .iter()
        .find(|row| row.locus().component_ordinal() != 0)
        .expect("the baseline fixture carries a component-locus obligation");
    let mut needle = Vec::new();
    component_row.locus().encode(&mut needle);
    let at = bytes
        .windows(needle.len())
        .position(|window| window == needle.as_slice())
        .expect("the component locus is encoded in the record");
    let mut corrupt = bytes.clone();
    // Byte 4 of the nine-byte locus run is the locus tag.
    corrupt[at + 4] = PolicyLocus::Computation.tag();
    observed.push(expect_decode_failure(
        "non-component locus carrying an ordinal",
        &corrupt,
        "malformed-obligation-key",
    ));

    // --- wire-only rejections the builder cannot produce -------------------
    let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);

    // A behaviour from another dimension's space.
    let image = DeliveredRealizationRecord::from_canonical_parts(
        record.profile().clone(),
        vec![TargetEvidence::from_canonical_parts(
            0,
            NumericalDimension::Contraction,
            preserve,
            HonouringMeans::SupportedExactly,
            record.profile().clone(),
            measured_source("1.0", "test-build-a"),
        )],
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
        zero_sign: FlushedZeroSign::PreservesSign,
    });
    let image = DeliveredRealizationRecord::from_canonical_parts(
        record.profile().clone(),
        vec![TargetEvidence::from_canonical_parts(
            0,
            NumericalDimension::InputSubnormals,
            flush,
            HonouringMeans::SupportedExactly,
            record.profile().clone(),
            measured_source("1.0", "test-build-a"),
        )],
        subjects.clone(),
        vec![NumericalObligation::from_canonical_parts(
            0,
            NumericalDimension::InputSubnormals,
            computation(0),
            preserve,
            0,
        )],
        Vec::new(),
    )
    .canonical_bytes();
    observed.push(expect_decode_failure(
        "wire evidence behaviour mismatch",
        &image,
        "evidence-behaviour-mismatch",
    ));

    // Provenance whose authority triple names no readable moment.
    let image = DeliveredRealizationRecord::from_canonical_parts(
        record.profile().clone(),
        vec![TargetEvidence::from_canonical_parts(
            0,
            NumericalDimension::Contraction,
            forbidden,
            HonouringMeans::SupportedExactly,
            record.profile().clone(),
            FactSourceProvenance::new(
                AvailabilityPhase::CompileProfile,
                FactAuthority::LaunchInstance,
                FactValidityScope::MeasuredEnvironment,
                ProvenanceIdentity::new("tiler.test.measured-authority.v1", 1),
                FactEvidenceBasis::Measurement {
                    contexts: Vec::new(),
                },
            ),
        )],
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

    // Only the schema word of an otherwise valid record. The body is the
    // current grammar; the old decoder would have discarded the number,
    // reconstructed through `new`, and accepted the row as schema 3.
    let unknown = poke_provenance_schema(&bytes, &record, 1);
    let error = decode(&unknown).expect_err("an unknown provenance schema");
    assert_eq!(
        error,
        RealizationCodecError::UnknownProvenanceSchema { version: 1 },
        "schema 1 has never had a decoder, got {error}"
    );
    observed.push(Perturbation {
        name: "unknown provenance schema",
        observed: rule_of(&error),
    });

    let newer = poke_provenance_schema(&bytes, &record, 4);
    let error = decode(&newer).expect_err("a newer provenance schema");
    assert_eq!(
        error,
        RealizationCodecError::NewerProvenanceSchema { version: 4 },
        "schema 4 is newer than this decoder, got {error}"
    );
    observed.push(Perturbation {
        name: "newer provenance schema",
        observed: rule_of(&error),
    });

    // Evidence readable only from a phase after packaging.
    let image = DeliveredRealizationRecord::from_canonical_parts(
        record.profile().clone(),
        vec![TargetEvidence::from_canonical_parts(
            0,
            NumericalDimension::Contraction,
            forbidden,
            HonouringMeans::SupportedExactly,
            record.profile().clone(),
            device_source(),
        )],
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

    // A `Required` range naming zero rows. The builder derives ranges from the
    // obligations that exist, so it can never mint one; only a wire image can.
    let scalar = subjects[0]
        .scalar_arithmetic()
        .expect("the only implemented family");
    let mut resolutions = [preserve; DIMENSION_COUNT];
    let mut dispositions = [AssessmentDisposition::NotRequired; DIMENSION_COUNT];
    for dimension in CANONICAL_DIMENSIONS {
        resolutions[dimension.index()] = scalar.resolution(dimension);
    }
    dispositions[NumericalDimension::InputSubnormals.index()] =
        AssessmentDisposition::Required { first: 0, len: 0 };
    let image = DeliveredRealizationRecord::from_canonical_parts(
        record.profile().clone(),
        Vec::new(),
        vec![NumericalPolicySubject::ScalarArithmetic(
            ScalarArithmeticRecord::from_canonical_parts(
                scalar.subject().clone(),
                resolutions,
                dispositions,
            ),
        )],
        Vec::new(),
        Vec::new(),
    )
    .canonical_bytes();
    observed.push(expect_decode_failure(
        "empty required range",
        &image,
        "empty-required-range",
    ));

    // A profile key the governed grammar refuses. Poked rather than built,
    // because `TargetProfileKey::new` is what refuses it.
    let mut corrupt = bytes.clone();
    corrupt[DELIVERED_REALIZATION_DOMAIN.len() + 8] = b'A';
    observed.push(expect_decode_failure(
        "malformed profile key",
        &corrupt,
        "malformed-realization-identity",
    ));

    // --- artifact cross-checks --------------------------------------------
    let decoded = decode(&bytes).expect("the baseline record decodes");
    let other = other_profile();
    let entries = [EntryRealization::of(strict_realization())];
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

    let profile = profile();
    let mut divergent = strict_realization();
    divergent.contraction = NumericalPermission::Permitted;
    let entries = [EntryRealization::of(divergent)];
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
        EntryRealization::of(strict_realization()),
        EntryRealization::of(strict_realization()),
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
    for rule in DeliveredRealizationError::ALL_RULES {
        if !seen.contains(&rule) {
            unexercised.push(rule);
        }
    }
    assert!(
        unexercised.is_empty(),
        "these rules were never watched refusing: {unexercised:?}"
    );

    let mut distinct: Vec<&str> = seen.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        observed.len(),
        40,
        "the perturbation population is stated, so a dropped case is a failure rather than a smaller pass: {:?}",
        observed.iter().map(|entry| entry.name).collect::<Vec<_>>()
    );
    assert_eq!(
        distinct.len(),
        26,
        "40 perturbations cover all 26 distinct rule identifiers the two vocabularies define"
    );
}

/// A complete measured provenance readable only from live device preflight.
fn device_source() -> FactSourceProvenance {
    FactSourceProvenance::new(
        AvailabilityPhase::LiveDevicePreflight,
        FactAuthority::DeviceRuntime,
        FactValidityScope::DeviceInstance,
        ProvenanceIdentity::new("tiler.test.device-authority.v1", 1),
        FactEvidenceBasis::Measurement {
            contexts: vec![MeasurementContext::new(
                vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::RuntimeCompiler,
                    "test-runtime-compiler",
                    "1.0",
                    None,
                )],
                ExecutionEnvironmentIdentity::new(
                    "test-platform",
                    "1.0",
                    "test-build-a",
                    "test-architecture",
                    "test-hardware",
                ),
            )],
        },
    )
}

/// The builder's own refusals, each watched failing.
#[allow(
    clippy::too_many_lines,
    reason = "the population is the point, exactly as it is for the decode table: one function naming every builder refusal is what the coverage assertion counts against"
)]
fn builder_perturbations() -> Vec<Perturbation> {
    let mut observed = Vec::new();
    let subject = f32_subject().identity();
    let strict = strict_resolutions();

    let mut builder = DeliveredRealizationBuilder::new(profile());
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
    let mut builder = DeliveredRealizationBuilder::new(profile());
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
    let mut builder = DeliveredRealizationBuilder::new(profile());
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
            strict_input,
            exact_evidence(strict_input),
        )
        .expect_err("an obligation before any contract");
    assert_eq!(error.rule(), "unknown-policy-subject");
    observed.push(Perturbation {
        name: "obligation for an undeclared subject",
        observed: error.rule().to_owned(),
    });

    let mut base = DeliveredRealizationBuilder::new(profile());
    base.declare_scalar_arithmetic(subject.clone(), strict)
        .expect("a well-formed contract");

    // A repeated (subject, dimension, locus).
    let mut builder = base.clone();
    builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
            strict_input,
            exact_evidence(strict_input),
        )
        .expect("a well-formed obligation");
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
            strict_input,
            exact_evidence(strict_input),
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
        zero_sign: FlushedZeroSign::PreservesSign,
    });
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
            strict_input,
            exact_evidence(other_behaviour),
        )
        .expect_err("evidence about another behaviour");
    assert_eq!(error.rule(), "evidence-behaviour-mismatch");
    observed.push(Perturbation {
        name: "evidence behaviour mismatch",
        observed: error.rule().to_owned(),
    });

    // Evidence naming another profile.
    let mut builder = base.clone();
    let mut foreign = exact_evidence(strict_input);
    foreign.profile = other_profile();
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
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
    let mut incomplete = exact_evidence(strict_input);
    incomplete.source = FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        FactAuthority::LaunchInstance,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.test.measured-authority.v1", 1),
        FactEvidenceBasis::Measurement {
            contexts: Vec::new(),
        },
    );
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
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
    let mut late = exact_evidence(strict_input);
    late.source = device_source();
    let error = builder
        .require(
            &subject,
            NumericalDimension::InputSubnormals,
            computation(0),
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
        .bind_entry(0, &f16_subject().identity())
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
    let error = DeliveredRealizationBuilder::new(profile())
        .build()
        .expect_err("a record with no subject");
    assert_eq!(error.rule(), "no-policy-subjects");
    observed.push(Perturbation {
        name: "no policy subjects",
        observed: error.rule().to_owned(),
    });

    observed
}
