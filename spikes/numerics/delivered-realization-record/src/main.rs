//! Compile-checked design packet for the required delivered-realization record.
//!
//! Run it from this directory; `README.md` records the exact invocation. The
//! binary's only product is a verdict: every stage that fails exits non-zero with
//! the stage named, and there is no partial success.
//!
//! Nothing here is production code. No file under `crates/` is modified, no
//! public boundary is promoted, no identity or schema domain is advanced, and no
//! pinned value is rebaselined. The proposal goes to
//! `accept-the-delivered-realization-artifact-surface` for Tom's ratification and
//! is **not accepted** by having compiled.

#![allow(
    dead_code,
    reason = "the packet's product is an exact proposed public surface, so every accessor Tom is being asked to ratify is spelled whether or not a fixture happens to call it. Dropping the unexercised ones would make the review packet a description of the fixtures rather than of the boundary, and the accessor a reviewer most needs to see is the one nothing calls yet"
)]

mod codec;
mod compiler_view;
mod fixtures;
mod perturb;
mod record;
mod shared;
mod translate;

use std::process::ExitCode;

use tiler_compiler::target::ScalarArithmetic;
use tiler_ir::schedule::{ArithmeticType, FlushedZeroSign, SubnormalMode};
use tiler_ir::semantic::{ResolvedValueType, StrictAffineU4, TypeKey, complex_value_type};

use codec::{ArtifactCrossCheck, decode, validate_against_artifact};
use compiler_view::{DeliveredRealizationView, SelectedEvidence, SelectedObligation};
use record::{
    AssessmentDisposition, DeliveredRealizationBuilder, DeliveredRealizationRecord,
    DispositionView, NumericalObligation,
};
use shared::{
    CANONICAL_DIMENSIONS, DIMENSION_COUNT, DimensionBehaviour, HonouringMeans, NumericalDimension,
    ScalarArithmeticSubject,
};

fn main() -> ExitCode {
    let stages: [(&str, fn()); 10] = [
        ("subject-validation", subject_validation),
        ("complete-eleven-dimension-subject", complete_subject),
        ("two-dtype-evidence", two_dtype_evidence),
        ("relaxation-payload-distinct", relaxation_payload_distinct),
        ("builder-canonicalizes", builder_canonicalizes),
        ("zero-obligation-subject", zero_obligation_subject),
        ("resolved-type-identity-families", identity_families),
        ("canonical-round-trip", round_trip),
        ("build-translation", build_translation),
        ("perturbations", perturbations),
    ];
    for (name, stage) in stages {
        println!("stage {name}");
        stage();
    }
    println!("\nall {} stages passed", stages.len());
    ExitCode::SUCCESS
}

/// A recognized value type is not a calibrated arithmetic subject.
///
/// The production validator is the authority, and every refusal below is its
/// answer rather than this spike's.
fn subject_validation() {
    // The two governed pairs this packet uses are admitted.
    assert!(
        ScalarArithmetic::new(ArithmeticType::F32, fixtures::governed_scalar("f32")).is_ok(),
        "f32 arithmetic over tiler::f32@1 is a governed subject"
    );
    assert!(
        ScalarArithmetic::new(ArithmeticType::F16, fixtures::governed_scalar("f16")).is_ok(),
        "f16 arithmetic over tiler::f16@1 is a governed subject"
    );

    // Every recognized non-scalar-float identity is refused for every arithmetic
    // type. The population is counted so a table that silently shrank could not
    // read as a pass.
    let rows = fixtures::non_subject_value_types();
    assert!(
        rows.len() >= 6,
        "the refusal population must cover bool, integer, decimal, complex, strict-affine, and MX"
    );
    let mut refusals = 0_usize;
    for (name, value_type) in &rows {
        for arithmetic in ArithmeticType::ALL {
            assert!(
                ScalarArithmetic::new(arithmetic, value_type.clone()).is_err(),
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
    for arithmetic in ArithmeticType::ALL {
        assert!(
            ScalarArithmetic::new(arithmetic, fixtures::owner_namespaced_type()).is_err(),
            "an unregistered owner-namespaced type must not create a subject"
        );
    }

    // A width or class that merely resembles the arithmetic type is still
    // refused: `tiler::u32@1` is exactly f32's width and a different class.
    assert!(
        ScalarArithmetic::new(ArithmeticType::F32, fixtures::governed_scalar("u32")).is_err(),
        "a matching width is not evidence of a matching format"
    );
    assert!(
        ScalarArithmetic::new(ArithmeticType::F32, fixtures::governed_scalar("f16")).is_err(),
        "a matching class is not evidence of a matching width"
    );
    println!("  {refusals} recognized (type, arithmetic) pairs refused; 2 governed pairs admitted");
}

/// Builds the record the ticket's first Required-evidence bullet names.
///
/// One subject, all eleven dimensions resolved, most `NotRequired`, one required
/// dimension, and **multiple distinct loci on one `(type, dimension)`** — the
/// property a dtype-wide ceiling alone cannot express.
fn complete_subject() {
    let record = ticket_fixture();
    perturb::assert_total_coverage(&record);

    let subject = fixtures::f32_subject();
    let view = record
        .scalar_arithmetic(&subject.identity())
        .expect("the declared subject resolves");

    // Every dimension answers, and the dispositions are the honest split.
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
        let evidence = view.evidence_for(row);
        assert_eq!(
            evidence.declared(),
            row.required(),
            "each obligation's evidence speaks about its own required behaviour"
        );
    }
    println!(
        "  11 dimensions covered; {required} required, {not_required} not required; 3 distinct loci on one (type, dimension)"
    );
}

/// One record carries `f16` and `f32` subnormal evidence without collision.
fn two_dtype_evidence() {
    let f32_subject = fixtures::f32_subject();
    let f16_subject = fixtures::f16_subject();
    let profile = fixtures::profile();

    let mut builder = DeliveredRealizationBuilder::new(profile);
    builder
        .declare_scalar_arithmetic(f32_subject.identity(), fixtures::flushing_resolutions())
        .expect("the f32 contract flushes");
    builder
        .declare_scalar_arithmetic(f16_subject.identity(), fixtures::strict_resolutions())
        .expect("the f16 contract preserves");

    let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    });
    let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
    builder
        .require(
            &f32_subject.identity(),
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            flush,
            fixtures::exact_evidence(flush),
        )
        .expect("the f32 row");
    builder
        .require(
            &f16_subject.identity(),
            NumericalDimension::InputSubnormals,
            fixtures::computation(1),
            preserve,
            fixtures::exact_evidence(preserve),
        )
        .expect("the f16 row");
    let record = builder.build().expect("a two-subject record");

    // The decisive assertion: one dimension, two subjects, two different answers,
    // and neither overwrote the other.
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
    assert_ne!(
        f32_view.resolution(NumericalDimension::InputSubnormals),
        f16_view.resolution(NumericalDimension::InputSubnormals),
        "the measured divergence the dtype key exists for"
    );

    // And the two evidence rows are separate rows rather than one deduplicated
    // into the other.
    assert_eq!(record.evidence().len(), 2, "two dtypes, two evidence rows");
    let decoded = decode(&record.canonical_bytes()).expect("the two-subject record round-trips");
    assert_eq!(decoded, record, "the round trip is the identity");
    println!(
        "  f32 flushes and f16 preserves in one record; 2 subjects, 2 evidence rows, no collision"
    );
}

/// Two conditional means differing only in relaxation payload stay distinct.
///
/// This is the cited defect, exercised from both sides: the presentation label
/// collapses them (and is documented as doing so), and the canonical identity
/// does not.
fn relaxation_payload_distinct() {
    let behaviour = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
    let first = fixtures::relaxation_evidence(
        behaviour,
        NumericalDimension::Contraction,
        DimensionBehaviour::Transform(tiler_ir::schedule::NumericalPermission::Permitted),
    );
    let second = fixtures::relaxation_evidence(
        behaviour,
        NumericalDimension::Reassociation,
        DimensionBehaviour::Transform(tiler_ir::schedule::NumericalPermission::Permitted),
    );

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

    // Carried into a record, the distinction survives the codec — which the
    // staged draft's opaque key bytes could not do.
    let subject = fixtures::f32_subject();
    let build = |evidence| {
        let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
        builder
            .declare_scalar_arithmetic(subject.identity(), fixtures::strict_resolutions())
            .expect("a contract");
        builder
            .require(
                &subject.identity(),
                NumericalDimension::InputSubnormals,
                fixtures::computation(0),
                behaviour,
                evidence,
            )
            .expect("an obligation");
        builder.build().expect("a record")
    };
    let left = build(first);
    let right = build(second);
    assert_ne!(
        left.canonical_bytes(),
        right.canonical_bytes(),
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
        "a reader can say which relaxation made the requirement honourable"
    );
    println!("  labels collide as documented; canonical identities and decoded payloads do not");
}

/// The producer accepts shuffled declarations; duplicates reject.
fn builder_canonicalizes() {
    let subject = fixtures::f32_subject();
    let strict = fixtures::strict_resolutions();
    let preserve = strict[NumericalDimension::InputSubnormals.index()];
    let forbidden = strict[NumericalDimension::Contraction.index()];

    // The same declarations, offered in two different orders.
    let declare = |order: [usize; 4]| {
        let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
        builder
            .declare_scalar_arithmetic(subject.identity(), strict)
            .expect("a contract");
        let calls: [(NumericalDimension, _, DimensionBehaviour); 4] = [
            (
                NumericalDimension::InputSubnormals,
                fixtures::computation(2),
                preserve,
            ),
            (
                NumericalDimension::InputSubnormals,
                fixtures::accumulator(1),
                preserve,
            ),
            (
                NumericalDimension::Contraction,
                fixtures::computation(0),
                forbidden,
            ),
            (
                NumericalDimension::InputSubnormals,
                fixtures::computation(0),
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
                    fixtures::exact_evidence(behaviour),
                )
                .expect("a well-formed obligation");
        }
        builder.build().expect("a record")
    };

    let forward = declare([0, 1, 2, 3]);
    let shuffled = declare([3, 2, 1, 0]);
    assert_eq!(
        forward.canonical_bytes(),
        shuffled.canonical_bytes(),
        "declaration order must not reach the wire"
    );
    assert!(
        decode(&shuffled.canonical_bytes()).is_ok(),
        "the canonicalized record decodes"
    );

    // Evidence identical across four obligations deduplicates to two rows: one
    // per (subject, dimension, behaviour) triple the rows actually cite.
    assert_eq!(
        forward.evidence().len(),
        2,
        "identical evidence is carried once"
    );

    // A duplicate declaration rejects rather than being taken last-wins.
    let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
    builder
        .declare_scalar_arithmetic(subject.identity(), strict)
        .expect("a contract");
    builder
        .require(
            &subject.identity(),
            NumericalDimension::InputSubnormals,
            fixtures::computation(0),
            preserve,
            fixtures::exact_evidence(preserve),
        )
        .expect("a first obligation");
    assert_eq!(
        builder
            .require(
                &subject.identity(),
                NumericalDimension::InputSubnormals,
                fixtures::computation(0),
                preserve,
                fixtures::exact_evidence(preserve),
            )
            .expect_err("a duplicate")
            .rule(),
        "obligation-redeclared"
    );
    // The rejected insertion left the draft unchanged.
    assert!(builder.build().is_ok(), "the draft survived the rejection");
    println!(
        "  two declaration orders produce identical bytes; 4 obligations cite 2 deduplicated evidence rows; duplicates reject"
    );
}

/// A selected contract with zero obligations still produces a complete subject.
fn zero_obligation_subject() {
    let subject = fixtures::f32_subject();
    let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
    builder
        .declare_scalar_arithmetic(subject.identity(), fixtures::strict_resolutions())
        .expect("a selected contract");
    let record = builder
        .build()
        .expect("a complete subject with no obligations");

    assert_eq!(
        record.subjects().len(),
        1,
        "one selected contract, one subject"
    );
    assert!(record.obligations().is_empty(), "and no obligations");
    let view = record
        .scalar_arithmetic(&subject.identity())
        .expect("the subject resolves");
    for dimension in CANONICAL_DIMENSIONS {
        assert_eq!(
            view.assessment(dimension),
            DispositionView::NotRequired,
            "{dimension} is NotRequired, and says so"
        );
        // The resolution is still carried: subject existence follows the checked
        // request's selected contract, not the obligation count.
        assert!(dimension.admits(view.resolution(dimension)));
    }

    // And the `NotRequired` claims are written rather than recoverable from
    // silence: the encoding is longer than a record that omitted them would be.
    let bytes = record.canonical_bytes();
    assert!(
        bytes.len() > shared::DIMENSION_COUNT * 2,
        "every disposition writes its own byte"
    );
    assert_eq!(
        decode(&bytes).expect("it round-trips"),
        record,
        "a zero-obligation record is a complete record"
    );
    println!(
        "  1 subject, 0 obligations, 11 explicit NotRequired dispositions, {} canonical bytes",
        bytes.len()
    );
}

/// The identity codec distinguishes all three resolved-type families.
fn identity_families() {
    let nominal = fixtures::governed_scalar("f32");
    let parameterized =
        complex_value_type(&fixtures::governed_scalar("f32")).expect("a complex identity");
    let encoded = StrictAffineU4::resolved_type();

    let identities: Vec<Vec<u8>> = [&nominal, &parameterized, &encoded]
        .into_iter()
        .map(|value| value.canonical_encoding().as_bytes().to_vec())
        .collect();
    let mut unique = identities.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        3,
        "nominal, parameterized, and encoded identities are three subjects"
    );

    // And two nominal identities differing only in namespace stay distinct, so
    // an owner-namespaced type can never alias a governed one.
    let owner =
        ResolvedValueType::nominal(TypeKey::new("acme", "f32", 1).expect("an owner namespace"));
    assert_ne!(
        owner.canonical_encoding().as_bytes(),
        nominal.canonical_encoding().as_bytes(),
        "a namespace is part of identity"
    );

    // None of the three inhabits the scalar-arithmetic schema, which the
    // production validator decides rather than this check.
    for arithmetic in ArithmeticType::ALL {
        assert!(ScalarArithmetic::new(arithmetic, parameterized.clone()).is_err());
        assert!(ScalarArithmetic::new(arithmetic, encoded.clone()).is_err());
    }
    println!(
        "  3 distinct full resolved-type identities carried; none calibrates an arithmetic subject"
    );
}

/// Encode, decode, and cross-check against the artifact that carries the record.
fn round_trip() {
    let record = ticket_fixture();
    let bytes = record.canonical_bytes();
    let decoded = decode(&bytes).expect("the record decodes");
    assert_eq!(decoded, record, "decode is the inverse of encode");
    assert_eq!(
        decoded.canonical_bytes(),
        bytes,
        "and re-encoding reproduces the exact bytes"
    );

    let profile = fixtures::profile();
    let entries = [fixtures::strict_realization()];
    validate_against_artifact(
        &decoded,
        &ArtifactCrossCheck {
            profile: &profile,
            entries: &entries,
        },
    )
    .expect("the record agrees with the artifact that carries it");
    println!(
        "  {} canonical bytes round-trip exactly; 8 overlapping resolutions agree with the entry",
        bytes.len()
    );
}

/// The proposed `tiler-build` translation, walked end to end.
fn build_translation() {
    let subject = fixtures::f32_subject();
    let resolutions = fixtures::strict_resolutions();
    let subjects = [(subject.clone(), resolutions)];
    let profile = fixtures::profile();

    let means = HonouringMeans::SupportedWithExactEmulation;
    let source = fixtures::measured_source("1.0", "spike-build-a");
    let descriptor = profile.descriptor.as_bytes().to_vec();
    let key = profile.key.as_str().to_owned();
    let evidence = SelectedEvidence::new(
        resolutions[NumericalDimension::InputSubnormals.index()],
        &means,
        &key,
        &descriptor,
        &source,
    );
    let obligations = [SelectedObligation::new(
        &subject,
        NumericalDimension::InputSubnormals,
        fixtures::computation(0),
        resolutions[NumericalDimension::InputSubnormals.index()],
        evidence,
    )];
    let view = DeliveredRealizationView::new(&key, &descriptor, &subjects, &obligations);

    let record = translate::translate(view, &profile, &[(0, subject.clone())])
        .expect("the translation succeeds");
    let read = record
        .scalar_arithmetic(&subject.identity())
        .expect("the translated subject resolves");
    let DispositionView::Required(rows) = read.assessment(NumericalDimension::InputSubnormals)
    else {
        panic!("the translated obligation is required");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(
        read.evidence_for(&rows[0]).means(),
        &HonouringMeans::SupportedWithExactEmulation,
        "the emulated means survived the translation structurally"
    );

    // A translation whose profile disagrees with the compiler view is refused
    // rather than silently re-attributed.
    let other = fixtures::other_profile();
    assert_eq!(
        translate::translate(view, &other, &[(0, subject.clone())]).expect_err("a mismatch"),
        translate::RealizationTranslationError::MalformedProfile
    );

    // And every dimension the compiler did not offer an obligation for is
    // `NotRequired` rather than absent.
    let mut derived = 0_usize;
    for dimension in CANONICAL_DIMENSIONS {
        if read.assessment(dimension) == DispositionView::NotRequired {
            derived += 1;
        }
    }
    assert_eq!(derived, DIMENSION_COUNT - 1);
    println!(
        "  1 subject, 1 obligation translated exhaustively; {derived} dispositions derived as NotRequired; profile disagreement refused"
    );
}

/// Every proposed validation check, perturbed once and watched failing.
fn perturbations() {
    let record = ticket_fixture();
    let observed = perturb::run(&record);
    for entry in &observed {
        println!("  perturbed {:<44} -> {}", entry.name, entry.observed);
    }
    let mut rules: Vec<&str> = observed
        .iter()
        .map(|entry| entry.observed.as_str())
        .collect();
    rules.sort_unstable();
    rules.dedup();
    println!(
        "  {} perturbations tripped {} distinct rules",
        observed.len(),
        rules.len()
    );
    assert!(
        observed.len() >= 24,
        "the perturbation population must cover every check the packet proposes"
    );
}

/// The record the ticket's Required-evidence section describes.
///
/// One `f32` subject over all eleven dimensions; input subnormals required at
/// three distinct loci, two of which carry different legal requirements;
/// contraction required at one locus; every other dimension `NotRequired`.
fn ticket_fixture() -> DeliveredRealizationRecord {
    let subject = fixtures::f32_subject();
    let resolutions = fixtures::strict_resolutions();
    let preserve = resolutions[NumericalDimension::InputSubnormals.index()];
    let forbidden = resolutions[NumericalDimension::Contraction.index()];
    let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    });

    let mut builder = DeliveredRealizationBuilder::new(fixtures::profile());
    builder
        .declare_scalar_arithmetic(subject.identity(), resolutions)
        .expect("the selected contract");

    // Three distinct loci on one (type, dimension). The materialization locus
    // legitimately tolerates a flush where the computation locus does not, which
    // is the case a dtype-wide ceiling alone would have collapsed.
    for (locus, required) in [
        (fixtures::computation(0), preserve),
        (fixtures::accumulator(0), preserve),
        (fixtures::materialization(1), flush),
    ] {
        builder
            .require(
                &subject.identity(),
                NumericalDimension::InputSubnormals,
                locus,
                required,
                fixtures::exact_evidence(required),
            )
            .expect("a well-formed obligation");
    }
    builder
        .require(
            &subject.identity(),
            NumericalDimension::Contraction,
            fixtures::computation(0),
            forbidden,
            fixtures::exact_evidence(forbidden),
        )
        .expect("a contraction obligation");
    // One ordered component of a compound encoded value, carrying a nonzero
    // ordinal. It is the only locus that may, which is what makes it the row the
    // malformed-key perturbation retags.
    builder
        .require(
            &subject.identity(),
            NumericalDimension::ResultSubnormals,
            fixtures::component(2, 1),
            preserve,
            fixtures::exact_evidence(preserve),
        )
        .expect("a component obligation");
    builder
        .bind_entry(0, &subject.identity())
        .expect("the one packaged entry");
    builder.build().expect("the ticket fixture")
}

/// A compile-time witness that the dense arrays and the vocabulary agree.
const _: () = {
    assert!(CANONICAL_DIMENSIONS.len() == DIMENSION_COUNT);
    // Every dimension's dense index is its position in canonical order, which is
    // what lets one exhaustive match serve every array in the record family.
    let mut index = 0;
    while index < DIMENSION_COUNT {
        assert!(CANONICAL_DIMENSIONS[index].index() == index);
        index += 1;
    }
};

/// A compile-time witness that no two dimension tags collide.
const _: () = {
    let mut left = 0;
    while left < DIMENSION_COUNT {
        let mut right = left + 1;
        while right < DIMENSION_COUNT {
            assert!(CANONICAL_DIMENSIONS[left].tag() != CANONICAL_DIMENSIONS[right].tag());
            right += 1;
        }
        left += 1;
    }
};

/// A compile-time witness that every dimension tag decodes back to itself.
const _: () = {
    let mut index = 0;
    while index < DIMENSION_COUNT {
        let dimension = CANONICAL_DIMENSIONS[index];
        match NumericalDimension::from_tag(dimension.tag()) {
            Some(resolved) => assert!(resolved.tag() == dimension.tag()),
            None => panic!("every governed dimension tag resolves"),
        }
        index += 1;
    }
};

/// A compile-time witness that the disposition vocabulary is closed and tagged.
const _: () = {
    assert!(AssessmentDisposition::NotRequired.tag() == 0x01);
    assert!(AssessmentDisposition::Required { first: 0, len: 1 }.tag() == 0x02);
};

/// A compile-time witness that a subject is the pair, not either half.
const _: () = {
    let _: fn(ArithmeticType, ResolvedValueType) -> ScalarArithmeticSubject =
        ScalarArithmeticSubject::new;
};
