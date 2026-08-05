//! Every fail-closed reader in this vocabulary, watched refusing.
//!
//! A `from_tag` that has never been observed returning `None` is a check whose
//! population is unknown. Each test below names the vocabulary it covers,
//! **counts** the population it walked, and asserts the count against the
//! vocabulary's own declared size — so a widened enum whose new variant nothing
//! exercises fails here rather than quietly shrinking what has been watched.

use super::{
    CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole, DIMENSION_COUNT,
    DimensionBehaviour, ExecutionEnvironmentIdentity, FactAuthority, FactEvidenceBasis,
    FactSourceProvenance, FactValidityScope, HonouringMeans, MAX_RESOLVED_TYPE_IDENTITY_BYTES,
    MeasurementContext, NumericalDimension, NumericalObligationKey, PolicyLocus,
    ProvenanceIdentity, RelaxationRequirement, ScalarArithmeticSubject,
    ScalarArithmeticSubjectError, ScalarArithmeticSubjectIdentity, approximation_envelope_from_tag,
    materialization_rounding_from_tag, materialization_rounding_tag, permission_from_tag,
    permission_tag, subnormal_from_tag, subnormal_tag, value_domain_provenance_from_tag,
    value_domain_provenance_tag,
};
use crate::program::SemanticOccurrence;
use crate::program::abi::AvailabilityPhase;
use crate::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode, ValueDomainProvenance,
};
use crate::semantic::{ResolvedValueType, StrictAffineU4, TypeKey, complex_value_type};

fn governed_scalar(name: &str) -> ResolvedValueType {
    ResolvedValueType::nominal(
        TypeKey::new("tiler", name, 1).expect("a governed catalog name is a valid identity"),
    )
}

/// Every behaviour this build can spell, in one enumerated population.
///
/// Written out rather than sampled, so a widened behaviour space is a build
/// error at the count assertion in [`every_behaviour_round_trips`] instead of a
/// silently narrower round-trip check.
fn all_behaviours() -> Vec<DimensionBehaviour> {
    let mut behaviours = vec![
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        }),
        DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        }),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        DimensionBehaviour::Transform(NumericalPermission::Permitted),
        DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden),
        DimensionBehaviour::Approximation(ApproximationEnvelope::BackendElementary),
        DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven),
    ];
    for provenance in [
        ValueDomainProvenance::CompilerProven,
        ValueDomainProvenance::RuntimeValidated,
        ValueDomainProvenance::CallerDeclaredUnvalidated,
    ] {
        behaviours.push(DimensionBehaviour::ExceptionalValue(
            ExceptionalValueAssumption::AssumeAbsent { provenance },
        ));
    }
    behaviours
}

#[test]
fn every_dimension_tag_resolves_and_an_unknown_one_refuses() {
    let mut resolved = 0_usize;
    for dimension in CANONICAL_DIMENSIONS {
        assert_eq!(
            NumericalDimension::from_tag(dimension.tag()),
            Some(dimension),
            "{dimension} must decode back to itself",
        );
        resolved += 1;
    }
    assert_eq!(
        resolved, DIMENSION_COUNT,
        "the round-trip counted a different population than the vocabulary declares",
    );

    // The fail-closed half, watched refusing over every byte the vocabulary does
    // not claim. Counting the refusals rather than spot-checking one is what
    // makes a widened tag range visible here.
    let mut refused = 0_usize;
    for tag in u8::MIN..=u8::MAX {
        if CANONICAL_DIMENSIONS
            .iter()
            .any(|dimension| dimension.tag() == tag)
        {
            continue;
        }
        assert_eq!(
            NumericalDimension::from_tag(tag),
            None,
            "tag {tag:#04x} is not a governed dimension and must refuse",
        );
        refused += 1;
    }
    assert_eq!(refused, 256 - DIMENSION_COUNT);
}

#[test]
fn every_dimension_index_is_its_canonical_position() {
    // The runtime witness of the compile-time property the dense arrays rest on.
    // It counts its population so a loop that never ran could not read as a pass.
    let mut counted = 0_usize;
    for (position, dimension) in CANONICAL_DIMENSIONS.into_iter().enumerate() {
        assert_eq!(dimension.index(), position);
        counted += 1;
    }
    assert_eq!(counted, DIMENSION_COUNT);
}

#[test]
fn every_dimension_admits_exactly_its_own_space() {
    let behaviours = all_behaviours();
    let mut admitted = 0_usize;
    let mut refused = 0_usize;
    for dimension in CANONICAL_DIMENSIONS {
        for behaviour in &behaviours {
            if dimension.space() == behaviour.space() {
                assert!(
                    dimension.admits(*behaviour),
                    "{dimension} must admit its own space",
                );
                admitted += 1;
            } else {
                assert!(
                    !dimension.admits(*behaviour),
                    "{dimension} must refuse another space's behaviour",
                );
                refused += 1;
            }
        }
    }
    assert_eq!(admitted + refused, DIMENSION_COUNT * behaviours.len());
    assert!(
        admitted > 0 && refused > 0,
        "the population must cover both answers, or neither has been watched",
    );
}

#[test]
fn every_behaviour_round_trips_and_consumes_exactly_its_own_width() {
    let behaviours = all_behaviours();
    assert_eq!(
        behaviours.len(),
        12,
        "the enumerated behaviour population must be complete; widen this count with the vocabulary",
    );
    for behaviour in &behaviours {
        let bytes = behaviour.canonical_key();
        let (decoded, width) =
            DimensionBehaviour::decode(&bytes).expect("a canonical behaviour decodes");
        assert_eq!(decoded, *behaviour);
        assert_eq!(
            width,
            bytes.len(),
            "decode must consume exactly the bytes encode wrote for {behaviour:?}",
        );
    }

    // Two distinct behaviours never share canonical bytes, which is what keeps a
    // record's evidence rows from deduplicating two different claims into one.
    let mut keys: Vec<Vec<u8>> = behaviours
        .iter()
        .map(|behaviour| behaviour.canonical_key())
        .collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), behaviours.len());
}

#[test]
fn an_unknown_behaviour_tag_refuses_rather_than_approximating() {
    // A space byte no arm claims.
    assert_eq!(DimensionBehaviour::decode(&[0xee, 0x01]), None);
    // A known space carrying a value byte it does not claim.
    assert_eq!(DimensionBehaviour::decode(&[0x01, 0xee]), None);
    assert_eq!(DimensionBehaviour::decode(&[0x02, 0xee]), None);
    assert_eq!(DimensionBehaviour::decode(&[0x03, 0xee]), None);
    assert_eq!(DimensionBehaviour::decode(&[0x04, 0xee]), None);
    assert_eq!(DimensionBehaviour::decode(&[0x05, 0xee]), None);
    // An assume-absent provenance byte no arm claims.
    assert_eq!(DimensionBehaviour::decode(&[0x04, 0x02, 0xee]), None);
    // Truncated input is not a behaviour either.
    assert_eq!(DimensionBehaviour::decode(&[]), None);
    assert_eq!(DimensionBehaviour::decode(&[0x01]), None);
    assert_eq!(DimensionBehaviour::decode(&[0x04, 0x02]), None);
}

#[test]
fn every_scalar_tag_pair_is_total_in_both_directions() {
    for mode in [
        SubnormalMode::Preserve,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        },
    ] {
        assert_eq!(subnormal_from_tag(subnormal_tag(mode)), Some(mode));
    }
    assert_eq!(subnormal_from_tag(0xee), None);

    for permission in [
        NumericalPermission::Forbidden,
        NumericalPermission::Permitted,
    ] {
        assert_eq!(
            permission_from_tag(permission_tag(permission)),
            Some(permission)
        );
    }
    assert_eq!(permission_from_tag(0xee), None);

    for envelope in [
        ApproximationEnvelope::Forbidden,
        ApproximationEnvelope::BackendElementary,
    ] {
        assert_eq!(
            approximation_envelope_from_tag(envelope.tag()),
            Some(envelope)
        );
    }
    assert_eq!(approximation_envelope_from_tag(0xee), None);

    for provenance in [
        ValueDomainProvenance::CompilerProven,
        ValueDomainProvenance::RuntimeValidated,
        ValueDomainProvenance::CallerDeclaredUnvalidated,
    ] {
        assert_eq!(
            value_domain_provenance_from_tag(value_domain_provenance_tag(provenance)),
            Some(provenance)
        );
    }
    assert_eq!(value_domain_provenance_from_tag(0xee), None);

    // One admitted direction today. Asserted directly rather than looped: the
    // list would not force an update on widening, whereas the exhaustive match
    // in `materialization_rounding_tag` is a build error the day a second
    // direction is admitted.
    let rounding = MaterializationRounding::NearestTiesToEven;
    assert_eq!(
        materialization_rounding_from_tag(materialization_rounding_tag(rounding)),
        Some(rounding)
    );
    assert_eq!(materialization_rounding_from_tag(0xee), None);

    for arithmetic in ArithmeticType::ALL {
        assert_eq!(ArithmeticType::from_tag(arithmetic.tag()), Some(arithmetic));
    }
    assert_eq!(ArithmeticType::from_tag(0xee), None);
}

#[test]
fn the_out_of_order_provenance_tags_are_preserved_exactly() {
    // These tags are deliberately not in declaration order. Renumbering them
    // into tidy order would silently change every target-profile descriptor that
    // declares a measured fact, and a diff that only looked tidier would not
    // show it. Pinning the exact bytes is what makes such an edit fail here.
    assert_eq!(FactAuthority::GovernedProfile.tag(), 0x01);
    assert_eq!(FactAuthority::ArtifactEvidence.tag(), 0x02);
    assert_eq!(FactAuthority::DeviceRuntime.tag(), 0x03);
    assert_eq!(FactAuthority::PreparedKernel.tag(), 0x04);
    assert_eq!(FactAuthority::LaunchInstance.tag(), 0x05);
    assert_eq!(FactAuthority::ExternalProfile.tag(), 0x06);
    assert_eq!(FactAuthority::MeasuredProfile.tag(), 0x07);

    assert_eq!(FactValidityScope::PortableProfile.tag(), 0x01);
    assert_eq!(FactValidityScope::DeviceInstance.tag(), 0x02);
    assert_eq!(FactValidityScope::PreparedArtifact.tag(), 0x03);
    assert_eq!(FactValidityScope::LaunchInstance.tag(), 0x04);
    assert_eq!(FactValidityScope::MeasuredEnvironment.tag(), 0x05);

    assert_eq!(CompilerBuildRole::IntermediateTranslator.tag(), 0x07);
    assert_eq!(
        FactEvidenceBasis::ExternalGuarantee {
            reference: ProvenanceIdentity::new("tiler.reference.v1", 1),
        }
        .tag(),
        0x03,
    );
}

#[test]
fn every_provenance_authority_and_scope_tag_is_total_and_fails_closed() {
    let authorities = [
        FactAuthority::GovernedProfile,
        FactAuthority::ExternalProfile,
        FactAuthority::MeasuredProfile,
        FactAuthority::ArtifactEvidence,
        FactAuthority::DeviceRuntime,
        FactAuthority::PreparedKernel,
        FactAuthority::LaunchInstance,
    ];
    for authority in authorities {
        assert_eq!(FactAuthority::from_tag(authority.tag()), Some(authority));
    }
    assert_eq!(authorities.len(), 7);
    assert_eq!(FactAuthority::from_tag(0x00), None);
    assert_eq!(FactAuthority::from_tag(0x08), None);

    let scopes = [
        FactValidityScope::PortableProfile,
        FactValidityScope::MeasuredEnvironment,
        FactValidityScope::DeviceInstance,
        FactValidityScope::PreparedArtifact,
        FactValidityScope::LaunchInstance,
    ];
    for scope in scopes {
        assert_eq!(FactValidityScope::from_tag(scope.tag()), Some(scope));
    }
    assert_eq!(scopes.len(), 5);
    assert_eq!(FactValidityScope::from_tag(0x00), None);
    assert_eq!(FactValidityScope::from_tag(0x06), None);
}

#[test]
fn every_policy_locus_tag_is_total_and_fails_closed() {
    let loci = [
        PolicyLocus::Input,
        PolicyLocus::Computation,
        PolicyLocus::Accumulator,
        PolicyLocus::Result,
        PolicyLocus::Component,
        PolicyLocus::Materialization,
    ];
    let mut keys = Vec::new();
    for locus in loci {
        assert_eq!(PolicyLocus::from_tag(locus.tag()), Some(locus));
        keys.push(locus.key());
    }
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), loci.len(), "every locus key is distinct");
    assert_eq!(PolicyLocus::from_tag(0x00), None);
    assert_eq!(PolicyLocus::from_tag(0x07), None);
}

#[test]
fn only_a_component_locus_may_carry_an_ordinal() {
    let occurrence = SemanticOccurrence::new(7);
    for locus in [
        PolicyLocus::Input,
        PolicyLocus::Computation,
        PolicyLocus::Accumulator,
        PolicyLocus::Result,
        PolicyLocus::Materialization,
    ] {
        let key = NumericalObligationKey::new(occurrence, locus);
        assert_eq!(
            key.component_ordinal(),
            0,
            "{locus:?} forces a zero ordinal"
        );
        assert!(key.is_well_formed());
    }
    let component = NumericalObligationKey::component(occurrence, 3);
    assert_eq!(component.locus(), PolicyLocus::Component);
    assert_eq!(component.component_ordinal(), 3);
    assert!(component.is_well_formed());

    // A component locus may also carry zero — the ordinal is the position, not a
    // presence flag, so component 0 is a real row rather than a malformed one.
    assert!(NumericalObligationKey::component(occurrence, 0).is_well_formed());
}

#[test]
fn an_obligation_key_encodes_at_its_declared_width() {
    let key = NumericalObligationKey::component(SemanticOccurrence::new(9), 2);
    assert_eq!(
        key.canonical_key().len(),
        NumericalObligationKey::ENCODED_BYTES,
        "a reader computing an offset must be able to trust the declared width",
    );
}

#[test]
fn the_means_label_is_not_injective_and_the_identity_is() {
    let subject = ScalarArithmeticSubject::f32().identity();
    let relaxed = |dimension| HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
        relaxation: RelaxationRequirement::new(
            subject.clone(),
            dimension,
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
        ),
    };
    let first = relaxed(NumericalDimension::Contraction);
    let second = relaxed(NumericalDimension::Reassociation);

    // The documented collapse. This is the defect the record's structural carry
    // exists to correct, asserted from both sides so a future "fix" that made
    // the label injective would have to update the documentation with it.
    assert_eq!(first.label(), second.label());
    // And the identity that does not collapse.
    assert_ne!(first.canonical_key(), second.canonical_key());
    assert_ne!(first, second);
}

#[test]
fn every_means_tag_is_distinct_and_the_conditional_arm_carries_its_payload() {
    let subject = ScalarArithmeticSubject::f32().identity();
    let means = [
        HonouringMeans::SupportedExactly,
        HonouringMeans::SupportedWithExactEmulation,
        HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
            relaxation: RelaxationRequirement::new(
                subject,
                NumericalDimension::Contraction,
                DimensionBehaviour::Transform(NumericalPermission::Permitted),
            ),
        },
        HonouringMeans::Unsupported,
    ];
    let mut tags: Vec<u8> = means.iter().map(HonouringMeans::tag).collect();
    tags.sort_unstable();
    tags.dedup();
    assert_eq!(tags.len(), means.len());

    // Only the conditional arm writes more than its tag, and it writes the whole
    // relaxation: a record that carried the label instead could not say which
    // relaxation made a requirement honourable.
    for value in &means {
        let bytes = value.canonical_key();
        match value {
            HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { .. } => {
                assert!(bytes.len() > 1);
            }
            _ => assert_eq!(bytes.len(), 1),
        }
    }
}

#[test]
fn a_recognized_value_type_is_not_a_calibrated_arithmetic_subject() {
    // Recognition and arithmetic-subject calibration are separate facts. Every
    // pairing below is a recognized identity that must still be refused, and the
    // population is counted so a table that silently shrank could not pass.
    let rows: Vec<(&str, ResolvedValueType)> = vec![
        ("bool", governed_scalar("bool")),
        ("integer i32", governed_scalar("i32")),
        ("decimal64", governed_scalar("decimal64")),
        (
            "complex over f32",
            complex_value_type(&governed_scalar("f32")).expect("a governed complex identity"),
        ),
        ("strict-affine u4", StrictAffineU4::resolved_type()),
        (
            "owner-namespaced posit16",
            ResolvedValueType::nominal(
                TypeKey::new("acme", "posit16", 1).expect("an owner namespace is a valid identity"),
            ),
        ),
    ];
    assert!(
        rows.len() >= 6,
        "the refusal population must cover bool, integer, decimal, complex, encoded, and owner-namespaced",
    );
    let mut refusals = 0_usize;
    for (name, value_type) in &rows {
        for arithmetic in ArithmeticType::ALL {
            assert_eq!(
                ScalarArithmeticSubject::new(arithmetic, value_type.clone()),
                Err(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic),
                "{name} must not calibrate a {arithmetic:?} arithmetic subject",
            );
            refusals += 1;
        }
    }
    assert_eq!(refusals, rows.len() * ArithmeticType::ALL.len());
}

#[test]
fn a_near_miss_width_or_class_is_still_refused() {
    // `tiler::u32@1` is exactly f32's width and a different class; `tiler::f16@1`
    // is exactly f32's class and a different width. Neither resemblance is
    // evidence that the arithmetic type's semantics were defined over it.
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F32, governed_scalar("u32")).is_err());
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F32, governed_scalar("f16")).is_err());
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F64, governed_scalar("f32")).is_err());

    // And the governed pairs the catalog does admit.
    let subject = ScalarArithmeticSubject::new(ArithmeticType::F32, governed_scalar("f32"))
        .expect("f32 arithmetic over tiler::f32@1 is a governed subject");
    assert_eq!(subject, ScalarArithmeticSubject::f32());
    assert!(ScalarArithmeticSubject::new(ArithmeticType::F16, governed_scalar("f16")).is_ok());
}

#[test]
fn three_resolved_type_families_are_three_distinct_subject_identities() {
    let nominal = governed_scalar("f32");
    let parameterized =
        complex_value_type(&governed_scalar("f32")).expect("a governed complex identity");
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
        "nominal, parameterized, and encoded identities are three subjects",
    );

    // A namespace is part of identity, so an owner-namespaced type can never
    // alias a governed one.
    let owner =
        ResolvedValueType::nominal(TypeKey::new("acme", "f32", 1).expect("an owner namespace"));
    assert_ne!(
        owner.canonical_encoding().as_bytes(),
        nominal.canonical_encoding().as_bytes(),
    );
}

#[test]
fn a_subject_identity_round_trips_through_its_serialized_parts() {
    let subject = ScalarArithmeticSubject::f32();
    let identity = subject.identity();
    assert_eq!(identity.arithmetic(), ArithmeticType::F32);
    assert_eq!(
        identity.resolved_type_identity(),
        subject.resolved_type().canonical_encoding().as_bytes(),
    );
    let rebuilt = ScalarArithmeticSubjectIdentity::from_parts(
        identity.arithmetic(),
        identity.resolved_type_identity(),
    )
    .expect("the identity's own parts rebuild it");
    assert_eq!(rebuilt, identity);
    assert_eq!(rebuilt.canonical_key(), identity.canonical_key());
}

#[test]
fn a_malformed_subject_identity_is_refused_at_its_bounds() {
    assert_eq!(
        ScalarArithmeticSubjectIdentity::from_parts(ArithmeticType::F32, &[]),
        None,
        "an empty resolved-type identity names nothing",
    );
    let oversized = vec![0x01; MAX_RESOLVED_TYPE_IDENTITY_BYTES + 1];
    assert_eq!(
        ScalarArithmeticSubjectIdentity::from_parts(ArithmeticType::F32, &oversized),
        None,
    );
    assert!(
        ScalarArithmeticSubjectIdentity::from_parts(
            ArithmeticType::F32,
            &vec![0x01; MAX_RESOLVED_TYPE_IDENTITY_BYTES],
        )
        .is_some(),
        "the bound itself is admitted, so the refusal is off-by-one free",
    );
}

fn measurement(compiler_version: &str) -> MeasurementContext {
    MeasurementContext::new(
        vec![CompilerBuildIdentity::new(
            CompilerBuildRole::CodeGenerator,
            "test-offline-compiler",
            compiler_version,
            None,
        )],
        ExecutionEnvironmentIdentity::new(
            "test-platform",
            "1.0",
            "test-build",
            "test-architecture",
            "test-hardware",
        ),
    )
}

#[test]
fn a_complete_provenance_statement_validates_and_an_incoherent_triple_does_not() {
    let measured = FactSourceProvenance::measured(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.test-authority.v1", 1),
        vec![measurement("1.0")],
    );
    assert!(measured.is_valid());

    // The triple is one claim, not three independent fields: a compile-profile
    // fact vouched for by a launch-instance authority names no readable moment.
    let incoherent = FactSourceProvenance::measured(
        AvailabilityPhase::CompileProfile,
        FactAuthority::LaunchInstance,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.test-authority.v1", 1),
        vec![measurement("1.0")],
    );
    assert!(!incoherent.is_valid());

    // A measurement basis with no context measured nothing.
    let empty = FactSourceProvenance::measured(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.test-authority.v1", 1),
        Vec::new(),
    );
    assert!(!empty.is_valid());

    // A governed guarantee vouched for by a measuring authority is the same
    // class of incoherence from the other side.
    let mismatched = FactSourceProvenance::new(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::PortableProfile,
        ProvenanceIdentity::new("tiler.test-authority.v1", 1),
        FactEvidenceBasis::GovernedGuarantee {
            guarantee: ProvenanceIdentity::new("tiler.test-guarantee.v1", 1),
        },
    );
    assert!(!mismatched.is_valid());
}

#[test]
fn a_malformed_provenance_identity_is_refused() {
    assert!(!ProvenanceIdentity::new("", 1).is_valid(), "an empty key");
    assert!(
        !ProvenanceIdentity::new("tiler.test.v1", 0).is_valid(),
        "revision zero is reserved for unset",
    );
    assert!(
        !ProvenanceIdentity::new("Tiler.Test.V1", 1).is_valid(),
        "the governed key grammar is lowercase",
    );
    assert!(ProvenanceIdentity::new("tiler.test-key_1.v1", 1).is_valid());
}

#[test]
fn measurement_contexts_are_canonically_ordered_by_construction() {
    // Two builds offered in either order produce one context, so a producer that
    // happened to enumerate its toolchain differently cannot mint a second
    // identity for the same measurement.
    let environment = || {
        ExecutionEnvironmentIdentity::new(
            "test-platform",
            "1.0",
            "test-build",
            "test-architecture",
            "test-hardware",
        )
    };
    let build = |role, version| CompilerBuildIdentity::new(role, "test-compiler", version, None);
    let forward = MeasurementContext::new(
        vec![
            build(CompilerBuildRole::Frontend, "1.0"),
            build(CompilerBuildRole::Linker, "2.0"),
        ],
        environment(),
    );
    let reversed = MeasurementContext::new(
        vec![
            build(CompilerBuildRole::Linker, "2.0"),
            build(CompilerBuildRole::Frontend, "1.0"),
        ],
        environment(),
    );
    assert_eq!(forward.canonical_bytes(), reversed.canonical_bytes());
    assert!(forward.is_valid());

    // A context with no build measured nothing, and a duplicated build is not a
    // strictly increasing table.
    assert!(!MeasurementContext::new(Vec::new(), environment()).is_valid());
    let duplicated = MeasurementContext::new(
        vec![
            build(CompilerBuildRole::Frontend, "1.0"),
            build(CompilerBuildRole::Frontend, "1.0"),
        ],
        environment(),
    );
    assert!(!duplicated.is_valid());
}

#[test]
fn a_provenance_rendering_names_every_field_its_encoding_covers() {
    let source = FactSourceProvenance::measured(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new("tiler.test-authority.v1", 1),
        vec![measurement("1.0")],
    );
    let mut rendered = String::new();
    source.render(&mut rendered);
    for expected in [
        "source-schema=3",
        "phase=compile-profile",
        "authority=measured-profile",
        "validity=measured-environment",
        "tiler.test-authority.v1@1",
        "basis=measurement",
        "test-offline-compiler@1.0",
        "test-platform/1.0/test-build/test-architecture/test-hardware",
    ] {
        assert!(
            rendered.contains(expected),
            "the rendering must show {expected}, or it is a summary of evidence rather than the evidence: {rendered}",
        );
    }
}

#[test]
fn two_provenance_statements_differing_in_one_field_differ_in_canonical_bytes() {
    let base = |version| {
        FactSourceProvenance::measured(
            AvailabilityPhase::CompileProfile,
            FactAuthority::MeasuredProfile,
            FactValidityScope::MeasuredEnvironment,
            ProvenanceIdentity::new("tiler.test-authority.v1", 1),
            vec![measurement(version)],
        )
    };
    assert_ne!(
        base("1.0").canonical_bytes(),
        base("2.0").canonical_bytes(),
        "the exact compiler build a measurement rests on is part of its identity",
    );
}
