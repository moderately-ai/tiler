use tiler_ir::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyDomain, AccuracyDomainClause,
    AccuracyPredicate, ConformanceEvidence, ConformanceEvidenceClass, ConformanceEvidenceError,
    DomainBound, DomainInterval, ExactRational, ExactTolerance, NamedElementaryDescriptorDigest,
    NamedElementaryProfileKey, OperandOrdinal, ReferenceResultClass, ReferenceResultConstraint,
    ReferenceRoundingRule, RefinementBasis, RefinementOutcome, RefinementUnknown,
    RegisteredImplicationRegistry, UlpFormat, refines, ulp_reference_gap_metric_key,
};
use tiler_ir::semantic::{
    F32, NormativeDefinitionRef, SILU_F32_EXPONENTIAL_ULP_TOLERANCE,
    SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE, builtin_scalar_value_type_facts, rms_norm_f32_op,
    rms_norm_f32_rsqrt_accuracy_contract, rms_norm_f32_rsqrt_exceptional_contract,
    rms_norm_f32_rsqrt_reference_semantics, silu_f32_exponential_accuracy_contract, silu_f32_op,
    softmax_f32_exponential_accuracy_contract, softmax_f32_op,
};

use super::{
    APPLE_MSL_EXP_F32_ULP_BOUND, APPLE_ULP_TRANSLATION_FACTOR, ElementaryEvidenceHalf,
    ElementaryRealization, ElementaryRefusalReason, RelativeAccuracyDomain,
    RelativeAccuracyRefusalReason, apple_msl_ulp_metric_key, assess_elementary_accuracy,
    elementary_relative_accuracy, elementary_relative_accuracy_from,
    installed_elementary_realizations, installed_implication_registry,
    metal_f32_exceptional_value_evidence, metal_f32_exponential_bound_evidence,
    metal_f32_exponential_contract, metal_f32_normalization_exceptional_value_evidence,
    metal_f32_reciprocal_square_root_bound_evidence, metal_f32_reciprocal_square_root_contract,
    metal_f32_softmax_exceptional_value_evidence, metal_f32_softmax_exponential_contract,
    relative_accuracy_of_contract,
};

fn required() -> AccuracyContract {
    silu_f32_exponential_accuracy_contract()
}

fn fixture_reference(text: &str) -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(text).expect("a fixture evidence field is canonical")
}

/// A synthetic hard-discharging record. This is a test fixture, not a Metal claim.
fn discharging_fixture(scope: &str, digest: &[u8]) -> ConformanceEvidence {
    ConformanceEvidence::new(
        ConformanceEvidenceClass::NormativeGuarantee,
        fixture_reference(scope),
        fixture_reference("synthetic both-halves fixture, not a Metal specification claim"),
        fixture_reference("fixture.elementary.both-halves"),
        fixture_reference("tiler test fixture, not a toolchain row"),
        None,
        None,
        None,
        digest,
    )
    .expect("the discharging fixture is well formed")
}

fn empirical_fixture(scope: &str, digest: &[u8]) -> ConformanceEvidence {
    ConformanceEvidence::new(
        ConformanceEvidenceClass::EmpiricalQualification,
        fixture_reference(scope),
        fixture_reference("synthetic empirical fixture"),
        fixture_reference("fixture.elementary.empirical"),
        fixture_reference("tiler test fixture"),
        Some(fixture_reference("fixture device")),
        Some(fixture_reference("fixture oracle")),
        Some(fixture_reference("fixture corpus")),
        digest,
    )
    .expect("the empirical fixture is well formed")
}

fn unknown_fixture(scope: &str, digest: &[u8]) -> ConformanceEvidence {
    ConformanceEvidence::new(
        ConformanceEvidenceClass::Unknown,
        fixture_reference(scope),
        fixture_reference("synthetic unknown fixture"),
        fixture_reference("fixture.elementary.unknown"),
        fixture_reference("tiler test fixture"),
        None,
        None,
        None,
        digest,
    )
    .expect("the unknown fixture is well formed")
}

fn realization_with(
    contract: AccuracyContract,
    bound: ConformanceEvidence,
    exceptional: ConformanceEvidence,
) -> ElementaryRealization {
    ElementaryRealization::new(
        contract.operation().clone(),
        contract,
        bound,
        exceptional,
        crate::target::honourability::governed_profile_source(),
    )
}

fn discharging_activation() -> ElementaryRealization {
    realization_with(
        metal_f32_exponential_contract(),
        discharging_fixture(
            "fixture bound half for tiler::silu-f32@1",
            b"fixture:silu-bound-v1",
        ),
        discharging_fixture(
            "fixture exceptional half for tiler::silu-f32@1",
            b"fixture:silu-exceptional-v1",
        ),
    )
}

fn discharging_normalization() -> ElementaryRealization {
    realization_with(
        metal_f32_reciprocal_square_root_contract(),
        discharging_fixture(
            "fixture bound half for tiler::rms-norm-f32@1",
            b"fixture:rms-bound-v1",
        ),
        discharging_fixture(
            "fixture exceptional half for tiler::rms-norm-f32@1",
            b"fixture:rms-exceptional-v1",
        ),
    )
}

fn discharging_softmax() -> ElementaryRealization {
    realization_with(
        metal_f32_softmax_exponential_contract(),
        discharging_fixture(
            "fixture bound half for tiler::softmax-f32@1",
            b"fixture:softmax-bound-v1",
        ),
        discharging_fixture(
            "fixture exceptional half for tiler::softmax-f32@1",
            b"fixture:softmax-exceptional-v1",
        ),
    )
}

fn discharging_installation() -> Vec<ElementaryRealization> {
    vec![
        discharging_activation(),
        discharging_normalization(),
        discharging_softmax(),
    ]
}

fn assert_undischarged(
    refusal: &super::ElementaryAccuracyRefusal,
    operation: &tiler_ir::semantic::OpKey,
    half: ElementaryEvidenceHalf,
    class: ConformanceEvidenceClass,
) {
    assert_eq!(
        refusal.diagnostic_code(),
        "accuracy.elementary.undischarged-evidence"
    );
    assert_eq!(refusal.operation(), operation);
    let ElementaryRefusalReason::UndischargedEvidence {
        declaring_profile,
        half: refused_half,
        class: refused_class,
    } = refusal.reason()
    else {
        panic!("expected undischarged evidence, got {refusal:?}");
    };
    assert_eq!(*refused_half, half);
    assert_eq!(*refused_class, class);
    assert!(declaring_profile.is_valid());
}

/// The installed Metal contract still refines, and admission still refuses.
///
/// Refinement and discharge are independent: the registered implication proves
/// the bound containment, and the empirical exceptional half still cannot
/// discharge. Collapsing those into one success would be the defect this ticket
/// closes.
#[test]
fn the_metal_realization_refines_the_resolved_contract_through_the_registered_implication() {
    let outcome = refines(
        &metal_f32_exponential_contract(),
        &required(),
        &installed_implication_registry(),
    );
    let RefinementOutcome::Refines {
        basis: RefinementBasis::RegisteredImplication { .. },
    } = outcome
    else {
        panic!("the Metal contract still refines through the registered implication: {outcome:?}");
    };
    let refusal = assess_elementary_accuracy(
        &required(),
        &installed_elementary_realizations(),
        &installed_implication_registry(),
    )
    .expect_err("empirical exceptional evidence cannot admit the refining Metal row");
    assert_undischarged(
        &refusal,
        &silu_f32_op(),
        ElementaryEvidenceHalf::ExceptionalValue,
        ConformanceEvidenceClass::EmpiricalQualification,
    );
}

/// The translation is exact arithmetic, and it lands exactly on the requirement.
///
/// Both factors are named so that changing either moves this assertion instead of
/// moving silently with it.
#[test]
fn the_translated_bound_is_exactly_the_requirement() {
    assert_eq!(APPLE_MSL_EXP_F32_ULP_BOUND, 4);
    assert_eq!(APPLE_ULP_TRANSLATION_FACTOR, 3);
    assert_eq!(
        ExactTolerance::from_integer(APPLE_MSL_EXP_F32_ULP_BOUND * APPLE_ULP_TRANSLATION_FACTOR),
        ExactTolerance::from_integer(SILU_F32_EXPONENTIAL_ULP_TOLERANCE)
    );
}

/// Without the registered cross-metric row the same declaration is infeasible.
///
/// **This is the perturbation, and it fires.** Nothing about the declaration or
/// the requirement changes; only the registry loses the derivation. The refusal
/// names both metrics, which is ADR 0042's "a distinct metric key is not a name
/// to match on" made operative: four ULPs under Apple's definition implies
/// nothing under Tiler's until someone derives the factor.
#[test]
fn without_the_registered_implication_the_declaration_is_refused() {
    let refusal = assess_elementary_accuracy(
        &required(),
        &installed_elementary_realizations(),
        &RegisteredImplicationRegistry::standard().expect("the governed registry composes"),
    )
    .expect_err("a cross-metric bound implies nothing without a registered derivation");
    assert_eq!(
        refusal.diagnostic_code(),
        "accuracy.elementary.unrefined-realization"
    );
    assert_eq!(refusal.operation(), &silu_f32_op());
    let ElementaryRefusalReason::Unrefined {
        declaring_profile,
        unknown,
    } = refusal.reason()
    else {
        panic!("a declared-but-unrefined realization is not an absent one: {refusal:?}");
    };
    let RefinementUnknown::UnregisteredMetricImplication { from, to } = unknown else {
        panic!("the refusal names the two metrics: {unknown:?}");
    };
    assert_eq!(*from, apple_msl_ulp_metric_key());
    assert_eq!(*to, ulp_reference_gap_metric_key());
    // The declaring profile's versioned identity travels with the refusal, which
    // is what ADR 0076 item 5 requires of a rejection and what a generic
    // unsupported-operation error cannot carry.
    assert!(declaring_profile.is_valid());
    assert!(!format!("{:?}", declaring_profile.authority_identity()).is_empty());
}

/// A profile that installs no realization for the key fails closed by name.
///
/// Distinct from the refusal above, and deliberately so: "nobody declared this"
/// is ADR 0043's `Unknown` — no admissible proof path — while "this declaration
/// does not refine" is a disproved predicate. Collapsing them would lose which
/// one a reader has to act on.
#[test]
fn an_uninstalled_operation_is_refused_as_undeclared_rather_than_unrefined() {
    let refusal = assess_elementary_accuracy(&required(), &[], &installed_implication_registry())
        .expect_err("an empty installation refines nothing");
    assert_eq!(
        refusal.diagnostic_code(),
        "accuracy.elementary.no-installed-realization"
    );
    assert!(matches!(
        refusal.reason(),
        ElementaryRefusalReason::NoInstalledRealization
    ));
}

/// A declaration whose exceptional-value contract differs is refused outright.
///
/// The bound is untouched and would still translate. ADR 0042 keeps the
/// exceptional-value contract independent of the error metric, and `refines`
/// refuses before it reaches the tolerance — so perturbing the *other* half is
/// what shows the independence is enforced rather than described.
#[test]
fn a_declaration_with_a_different_exceptional_contract_is_refused_before_the_bound() {
    use tiler_ir::semantic::accuracy::{
        DomainErrorRule, ExceptionalValueContract, FiniteOverflowRule, InfiniteReferenceRule,
        NanReferenceRule,
    };
    let declared = metal_f32_exponential_contract();
    let perturbed = AccuracyContract::new(
        declared.operation().clone(),
        declared.operand_types().to_vec(),
        declared.result_type().clone(),
        declared.reference_semantics().clone(),
        declared.form().clone(),
        // The one plausible alternative: a target returning the largest finite
        // value on overflow instead of an infinity. The `-88.73` band's exact
        // negative zero rests on the infinity, so this is a different operation
        // rather than a coarser one.
        ExceptionalValueContract::new(
            NanReferenceRule::CanonicalNan,
            InfiniteReferenceRule::SignedInfinity,
            DomainErrorRule::CanonicalNan,
            FiniteOverflowRule::LargestFinite,
        ),
    );
    let outcome = refines(&perturbed, &required(), &installed_implication_registry());
    let RefinementOutcome::Unknown { reason } = outcome else {
        panic!("a different exceptional contract is not a refinement: {outcome:?}");
    };
    assert_eq!(reason, RefinementUnknown::DifferentExceptionalValueContract);
}

/// The two evidence halves carry different classes, and only one discharges.
///
/// The honest state of the Metal realization, made checkable: the ordinary-domain
/// bound rests on a quoted normative guarantee and the exceptional-value
/// behaviour on a bounded corpus. Reporting one summary boolean would have to
/// pick which half to believe.
#[test]
fn the_bound_is_normative_and_the_exceptional_behaviour_is_only_empirical() {
    let bound = metal_f32_exponential_bound_evidence().expect("well formed");
    let exceptional = metal_f32_exceptional_value_evidence().expect("well formed");
    assert_eq!(bound.class(), ConformanceEvidenceClass::NormativeGuarantee);
    assert_eq!(
        exceptional.class(),
        ConformanceEvidenceClass::EmpiricalQualification
    );
    assert!(bound.discharge().is_ok());
    assert_eq!(
        exceptional
            .discharge()
            .expect_err("empirical evidence cannot discharge a hard requirement"),
        ConformanceEvidenceError::ClassCannotDischarge {
            class: ConformanceEvidenceClass::EmpiricalQualification
        }
    );

    let installed = installed_elementary_realizations();
    let silu = installed
        .iter()
        .find(|row| row.operation() == &silu_f32_op())
        .expect("the SiLU row is installed");
    let discharge = silu.discharge();
    assert!(discharge.bound_is_discharged());
    assert!(
        !discharge.exceptional_is_discharged(),
        "an empirical qualification does not become a guarantee by sitting beside one"
    );
    assert_eq!(
        discharge.exceptional_class(),
        ConformanceEvidenceClass::EmpiricalQualification
    );
    let refusal =
        assess_elementary_accuracy(&required(), &installed, &installed_implication_registry())
            .expect_err("a row that discharges only one half is not admitted");
    assert_undischarged(
        &refusal,
        &silu_f32_op(),
        ElementaryEvidenceHalf::ExceptionalValue,
        ConformanceEvidenceClass::EmpiricalQualification,
    );
}

/// The empirical record names its oracle, corpus, device, and toolchain.
///
/// ADR 0042 requires them of exactly the classes that measure something, and the
/// constructor refuses a record missing either — an irreproducible measurement is
/// not evidence. Asserting they are present keeps the record from decaying into a
/// class label.
#[test]
fn the_empirical_record_is_reproducible_from_its_own_fields() {
    let record = metal_f32_exceptional_value_evidence().expect("well formed");
    assert!(record.device().is_some());
    let oracle = record.reference_oracle().expect("an oracle is named");
    let corpus = record.corpus().expect("a corpus is named");
    assert!(oracle.as_str().contains("silu_f32"));
    assert!(corpus.as_str().contains("boundary corpus"));
    assert!(record.toolchain().as_str().contains("metal version"));
    assert!(!record.digest().is_empty());
}

/// The normative record cites the retained specification by digest.
#[test]
fn the_normative_record_cites_the_retained_specification_digest() {
    let record = metal_f32_exponential_bound_evidence().expect("well formed");
    assert_eq!(
        record.digest(),
        b"sha256:41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5"
    );
    assert!(
        record.scope().as_str().contains("1.6.3"),
        "the applicability inference is inside the record's own scope rather than omitted from it"
    );
    // A normative guarantee has no device and no corpus, and the constructor does
    // not require them: a promise about an implementation is not an observation
    // of one.
    assert!(record.device().is_none());
    assert!(record.corpus().is_none());
}

// ---------------------------------------------------------------------------
// The normalization's reciprocal square root, whose contract needs no metric
// ---------------------------------------------------------------------------

/// The installed Metal normalization contract still refines by identity.
///
/// Refinement is not admission: the exceptional half remains empirical, so
/// assessment refuses the same row the identity proof would otherwise accept.
#[test]
fn the_metal_normalization_realization_refines_by_identity_rather_than_implication() {
    let outcome = refines(
        &metal_f32_reciprocal_square_root_contract(),
        &rms_norm_f32_rsqrt_accuracy_contract(),
        &installed_implication_registry(),
    );
    assert_eq!(
        outcome,
        RefinementOutcome::Refines {
            basis: RefinementBasis::IdenticalNormalizedContract
        }
    );
    let refusal = assess_elementary_accuracy(
        &rms_norm_f32_rsqrt_accuracy_contract(),
        &installed_elementary_realizations(),
        &installed_implication_registry(),
    )
    .expect_err("empirical exceptional evidence cannot admit the normalizing Metal row");
    assert_undischarged(
        &refusal,
        &rms_norm_f32_op(),
        ElementaryEvidenceHalf::ExceptionalValue,
        ConformanceEvidenceClass::EmpiricalQualification,
    );
}

/// It refines even under an empty implication registry, which the exponential does not.
///
/// **The perturbation that separates the two families.** Stripping every
/// registered implication leaves the reciprocal square root's admission
/// untouched, because a faithful contract states its result set outright, while
/// the exponential's becomes an unregistered-metric refusal. A change that
/// quietly restated the normalization's contract as a ULP bound would fail here.
#[test]
fn the_normalization_needs_no_registered_implication_at_all() {
    let outcome = refines(
        &metal_f32_reciprocal_square_root_contract(),
        &rms_norm_f32_rsqrt_accuracy_contract(),
        &RegisteredImplicationRegistry::empty(),
    );
    assert_eq!(
        outcome,
        RefinementOutcome::Refines {
            basis: RefinementBasis::IdenticalNormalizedContract
        }
    );

    let refusal = assess_elementary_accuracy(
        &required(),
        &installed_elementary_realizations(),
        &RegisteredImplicationRegistry::empty(),
    )
    .expect_err("the exponential's requirement does need one");
    assert!(matches!(
        refusal.reason(),
        ElementaryRefusalReason::Unrefined {
            unknown: RefinementUnknown::UnregisteredMetricImplication { .. },
            ..
        }
    ));
}

/// The declaration is not stronger than the specification supports.
///
/// **This is the check that catches the over-claim, and the over-claim is the one
/// that would otherwise pass.** Declaring `CorrectlyRounded { NearestTiesToEven }`
/// would still refine the faithful requirement — the vocabulary registers
/// correctly-rounded-satisfies-faithful — so a wrong declaration would be
/// *admitted* rather than rejected. Nothing but this assertion stands between the
/// build and a claim §8.2 explicitly declines to make.
#[test]
fn the_metal_normalization_declaration_is_not_stronger_than_the_specification() {
    let declared = metal_f32_reciprocal_square_root_contract();
    assert!(matches!(declared.form(), AccuracyContractForm::Faithful));
    assert_eq!(declared, rms_norm_f32_rsqrt_accuracy_contract());

    // The over-claim, constructed and shown to be admitted, which is why the
    // assertion above is the control rather than a restatement of it.
    let overclaimed = AccuracyContract::new(
        rms_norm_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::CorrectlyRounded {
            rounding: ReferenceRoundingRule::NearestTiesToEven,
        },
        rms_norm_f32_rsqrt_exceptional_contract(),
    );
    assert!(
        refines(
            &overclaimed,
            &rms_norm_f32_rsqrt_accuracy_contract(),
            &installed_implication_registry(),
        )
        .is_physically_feasible(),
        "the over-claim refines, which is exactly why the declaration must not make it"
    );
}

/// The two halves of the normalization's evidence answer differently, on purpose.
#[test]
fn the_normalization_evidence_discharges_only_its_normative_half() {
    let bound = metal_f32_reciprocal_square_root_bound_evidence()
        .expect("the normative record is well formed");
    assert_eq!(bound.class(), ConformanceEvidenceClass::NormativeGuarantee);
    assert!(bound.discharge().is_ok());

    let exceptional = metal_f32_normalization_exceptional_value_evidence()
        .expect("the empirical record is well formed");
    assert_eq!(
        exceptional.class(),
        ConformanceEvidenceClass::EmpiricalQualification
    );
    assert!(exceptional.discharge().is_err());
}

/// Every registered family has an installed realization, and only those three.
///
/// The list is asserted in order rather than as a set, so an added row moves this
/// test whichever position it takes — which is what makes "and only those" a
/// checked claim rather than a lower bound.
#[test]
fn the_installed_realizations_are_exactly_the_registered_families() {
    let installed = installed_elementary_realizations();
    let operations: Vec<String> = installed
        .iter()
        .map(|realization| realization.operation().to_string())
        .collect();
    assert_eq!(
        operations,
        vec![
            "tiler::silu-f32@1",
            "tiler::rms-norm-f32@1",
            "tiler::softmax-f32@1"
        ]
    );

    // A family with no installed row fails closed rather than borrowing another
    // row's evidence.
    let absent = AccuracyContract::new(
        tiler_ir::semantic::add_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::Faithful,
        rms_norm_f32_rsqrt_exceptional_contract(),
    );
    let refusal =
        assess_elementary_accuracy(&absent, &installed, &installed_implication_registry())
            .expect_err("no realization speaks about the addition");
    assert!(matches!(
        refusal.reason(),
        ElementaryRefusalReason::NoInstalledRealization
    ));
}

/// The softmax's exponential admits through the *same* registered implication.
///
/// **The reuse is the assertion.** The bound, the metric, and the derivation are
/// the activation's; only the operation key and the admitted domain differ. If
/// this vertical had installed a second cross-metric row, stripping the registry
/// to the one row the activation registered would still admit — so the test
/// below, which uses the installed registry unchanged and then perturbs it, is
/// what shows one row serves both.
#[test]
fn the_softmax_exponential_refines_through_the_registered_implication() {
    let outcome = refines(
        &metal_f32_softmax_exponential_contract(),
        &softmax_f32_exponential_accuracy_contract(),
        &installed_implication_registry(),
    );
    let RefinementOutcome::Refines {
        basis: RefinementBasis::RegisteredImplication { .. },
    } = outcome
    else {
        panic!(
            "the softmax Metal contract still refines through the registered implication: {outcome:?}"
        );
    };
    let installed = installed_elementary_realizations();
    let softmax = installed
        .iter()
        .find(|row| row.operation() == &softmax_f32_op())
        .expect("the softmax row is installed");
    assert!(softmax.discharge().bound_is_discharged());
    assert!(!softmax.discharge().exceptional_is_discharged());
    assert_eq!(
        softmax.discharge().exceptional_class(),
        ConformanceEvidenceClass::EmpiricalQualification
    );
    let refusal = assess_elementary_accuracy(
        &softmax_f32_exponential_accuracy_contract(),
        &installed,
        &installed_implication_registry(),
    )
    .expect_err("empirical exceptional evidence cannot admit the softmax Metal row");
    assert_undischarged(
        &refusal,
        &softmax_f32_op(),
        ElementaryEvidenceHalf::ExceptionalValue,
        ConformanceEvidenceClass::EmpiricalQualification,
    );
}

/// This vertical registers no second cross-metric row, and one row serves both.
///
/// The registry is compared against the activation's own by count, so an added
/// row would fail here whether or not it happened to be equivalent. The
/// perturbation beside it is the same one the activation's test runs: stripped to
/// the vocabulary's standard rows, the softmax's declaration stops refining, so
/// the admission above genuinely rests on the one registered derivation rather
/// than on an identical contract.
#[test]
fn the_softmax_needs_no_second_registered_implication() {
    let refusal = assess_elementary_accuracy(
        &softmax_f32_exponential_accuracy_contract(),
        &installed_elementary_realizations(),
        &RegisteredImplicationRegistry::standard().expect("the governed registry composes"),
    )
    .expect_err("a cross-metric bound implies nothing without a registered derivation");
    assert_eq!(refusal.operation(), &softmax_f32_op());
    let ElementaryRefusalReason::Unrefined { unknown, .. } = refusal.reason() else {
        panic!("a declared-but-unrefined realization is not an absent one: {refusal:?}");
    };
    let RefinementUnknown::UnregisteredMetricImplication { from, to } = unknown else {
        panic!("the refusal names the two metrics: {unknown:?}");
    };
    assert_eq!(*from, apple_msl_ulp_metric_key());
    assert_eq!(*to, ulp_reference_gap_metric_key());
}

/// The two exponential requirements differ only in operation and domain.
///
/// The tolerance and the metric are identical because Table 8.1 bounds the
/// *function*, not the operation calling it; the domain differs because the
/// maximum subtraction confines the softmax's argument to the non-positive reals
/// while the activation reaches the overflow band. Asserting both halves is what
/// keeps the sharing honest in one direction and the narrowing honest in the
/// other.
#[test]
fn the_two_exponential_requirements_share_a_bound_and_differ_in_domain() {
    assert_eq!(
        SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE,
        SILU_F32_EXPONENTIAL_ULP_TOLERANCE
    );
    let softmax = softmax_f32_exponential_accuracy_contract();
    let activation = silu_f32_exponential_accuracy_contract();
    assert_ne!(softmax.operation(), activation.operation());
    assert_ne!(softmax, activation);
    // And the two *declarations* differ the same way, so neither could be
    // consulted for the other's requirement.
    assert_ne!(
        metal_f32_softmax_exponential_contract(),
        metal_f32_exponential_contract()
    );
    assert_eq!(
        metal_f32_softmax_exponential_contract().operation(),
        &softmax_f32_op()
    );
}

/// Each family's exceptional record names its own corpus, not a shared one.
///
/// A shared record would qualify a population neither family measured. The three
/// corpus tags are asserted distinct rather than described, so a later change
/// that pointed two families at one corpus has to move this test.
#[test]
fn each_family_carries_its_own_exceptional_value_corpus() {
    let softmax =
        metal_f32_softmax_exceptional_value_evidence().expect("the softmax record is well formed");
    let activation =
        metal_f32_exceptional_value_evidence().expect("the activation record is well formed");
    let normalization = metal_f32_normalization_exceptional_value_evidence()
        .expect("the normalization record is well formed");
    assert_ne!(softmax, activation);
    assert_ne!(softmax, normalization);
    for record in [&softmax, &activation, &normalization] {
        assert_eq!(
            record.class(),
            ConformanceEvidenceClass::EmpiricalQualification,
            "the specification states no edge-case table for any of these functions"
        );
        assert!(record.discharge().is_err());
    }
}

/// A fixture whose two halves both discharge is admitted.
///
/// This is the positive path the Metal rows cannot take: the contracts are the
/// Metal ones, the evidence is a labelled test fixture, and both halves are
/// `NormativeGuarantee`. Fabricating a Metal claim is not what this does.
#[test]
fn a_realization_whose_two_halves_discharge_is_admitted() {
    let admission = assess_elementary_accuracy(
        &required(),
        &discharging_installation(),
        &installed_implication_registry(),
    )
    .expect("both halves discharge");
    let RefinementBasis::RegisteredImplication { .. } = admission.basis() else {
        panic!("the fixture still uses the Metal contract and its implication: {admission:?}");
    };
    assert!(admission.discharge().bound_is_discharged());
    assert!(admission.discharge().exceptional_is_discharged());

    let declared = ElementaryRealization::declare(
        metal_f32_exponential_contract().operation().clone(),
        metal_f32_exponential_contract(),
        discharging_fixture(
            "declaration-bound fixture for tiler::silu-f32@1",
            b"fixture:declare-silu-bound-v1",
        ),
        discharging_fixture(
            "declaration-exceptional fixture for tiler::silu-f32@1",
            b"fixture:declare-silu-exceptional-v1",
        ),
        crate::target::honourability::governed_profile_source(),
    )
    .expect("declaration of a both-halves row succeeds");
    assert!(declared.require_discharged_halves().is_ok());
}

/// Declaration refuses a row whose exceptional half cannot discharge.
///
/// Same invariant as assessment, asked at the other boundary, so a future
/// internal caller cannot assemble an admission-eligible row from empirical
/// exceptional evidence.
#[test]
fn declaration_refuses_an_empirical_exceptional_half() {
    let refusal = ElementaryRealization::declare(
        metal_f32_exponential_contract().operation().clone(),
        metal_f32_exponential_contract(),
        metal_f32_exponential_bound_evidence().expect("well formed"),
        metal_f32_exceptional_value_evidence().expect("well formed"),
        crate::target::honourability::governed_profile_source(),
    )
    .expect_err("declaration cannot mint an admission-eligible Metal row");
    assert_undischarged(
        &refusal,
        &silu_f32_op(),
        ElementaryEvidenceHalf::ExceptionalValue,
        ConformanceEvidenceClass::EmpiricalQualification,
    );
}

/// Perturbing only the bound half refuses that half, assertion unchanged.
///
/// The fixture starts admitted. Replacing the bound record with empirical
/// qualification, and nothing else, is the subject. The expected refusal is
/// written before the perturbation so a later edit that also changes the
/// assertion cannot manufacture a pass.
#[test]
fn perturbing_only_the_bound_half_refuses_that_half() {
    let expected_half = ElementaryEvidenceHalf::Bound;
    let expected_class = ConformanceEvidenceClass::EmpiricalQualification;
    let admitted = assess_elementary_accuracy(
        &required(),
        &[discharging_activation()],
        &installed_implication_registry(),
    );
    assert!(
        admitted.is_ok(),
        "the unperturbed fixture must admit: {admitted:?}"
    );

    let perturbed = realization_with(
        metal_f32_exponential_contract(),
        empirical_fixture(
            "perturbed bound half for tiler::silu-f32@1",
            b"fixture:silu-bound-empirical-v1",
        ),
        discharging_fixture(
            "unperturbed exceptional half for tiler::silu-f32@1",
            b"fixture:silu-exceptional-v1",
        ),
    );
    let refusal =
        assess_elementary_accuracy(&required(), &[perturbed], &installed_implication_registry())
            .expect_err("an empirical bound half cannot discharge");
    assert_undischarged(&refusal, &silu_f32_op(), expected_half, expected_class);
}

/// Perturbing only the exceptional half refuses that half, assertion unchanged.
#[test]
fn perturbing_only_the_exceptional_half_refuses_that_half() {
    let expected_half = ElementaryEvidenceHalf::ExceptionalValue;
    let expected_class = ConformanceEvidenceClass::EmpiricalQualification;
    let admitted = assess_elementary_accuracy(
        &required(),
        &[discharging_activation()],
        &installed_implication_registry(),
    );
    assert!(
        admitted.is_ok(),
        "the unperturbed fixture must admit: {admitted:?}"
    );

    let perturbed = realization_with(
        metal_f32_exponential_contract(),
        discharging_fixture(
            "unperturbed bound half for tiler::silu-f32@1",
            b"fixture:silu-bound-v1",
        ),
        empirical_fixture(
            "perturbed exceptional half for tiler::silu-f32@1",
            b"fixture:silu-exceptional-empirical-v1",
        ),
    );
    let refusal =
        assess_elementary_accuracy(&required(), &[perturbed], &installed_implication_registry())
            .expect_err("an empirical exceptional half cannot discharge");
    assert_undischarged(&refusal, &silu_f32_op(), expected_half, expected_class);
}

/// An `Unknown` bound half is a distinct class from empirical qualification.
#[test]
fn an_unknown_bound_half_is_refused_as_unknown_not_empirical() {
    let expected_half = ElementaryEvidenceHalf::Bound;
    let expected_class = ConformanceEvidenceClass::Unknown;
    let perturbed = realization_with(
        metal_f32_exponential_contract(),
        unknown_fixture(
            "unknown bound half for tiler::silu-f32@1",
            b"fixture:silu-bound-unknown-v1",
        ),
        discharging_fixture(
            "unperturbed exceptional half for tiler::silu-f32@1",
            b"fixture:silu-exceptional-v1",
        ),
    );
    let refusal =
        assess_elementary_accuracy(&required(), &[perturbed], &installed_implication_registry())
            .expect_err("unknown bound evidence cannot discharge");
    assert_undischarged(&refusal, &silu_f32_op(), expected_half, expected_class);
}

/// The governed profile's installed Metal rows fail closed, and that is not
/// an absent row.
#[test]
fn the_governed_profile_refuses_each_metal_row_as_undischarged_exceptional_evidence() {
    for (operation, required_contract) in [
        (silu_f32_op(), silu_f32_exponential_accuracy_contract()),
        (rms_norm_f32_op(), rms_norm_f32_rsqrt_accuracy_contract()),
        (
            softmax_f32_op(),
            softmax_f32_exponential_accuracy_contract(),
        ),
    ] {
        let refusal = assess_elementary_accuracy(
            &required_contract,
            &installed_elementary_realizations(),
            &installed_implication_registry(),
        )
        .expect_err("every current Metal row fails closed");
        assert_undischarged(
            &refusal,
            &operation,
            ElementaryEvidenceHalf::ExceptionalValue,
            ConformanceEvidenceClass::EmpiricalQualification,
        );
        let number =
            elementary_relative_accuracy(&operation, &crate::target::TargetProfile::governed())
                .expect_err("an undischarged row yields no relative accuracy");
        assert_eq!(
            number.diagnostic_code(),
            "accuracy.elementary.undischarged-evidence"
        );
    }
}

// ---------------------------------------------------------------------------
// The numeric relative accuracy a parametric rewrite bound instantiates from
// ---------------------------------------------------------------------------

/// binary32's unit roundoff, `2^-24`, written once so no test restates it.
fn unit_roundoff() -> ExactRational {
    ExactRational::power_of_two(-24)
}

/// binary32's least normal magnitude, `2^-126`.
fn least_normal() -> ExactRational {
    ExactRational::power_of_two(-126)
}

fn f32_format() -> UlpFormat {
    UlpFormat::from_value_type_facts(
        &builtin_scalar_value_type_facts(&F32::resolved_type()).expect("f32 is a governed scalar"),
    )
    .expect("binary32 carries tiler::ulp-reference-gap@1")
}

fn justification() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new("a synthetic proof standing in for an operation-specific one")
        .expect("the synthetic justification is canonical")
}

/// Builds a `BoundedPiecewise` contract over the supplied clauses.
///
/// The operation, dtypes, reference semantics, and exceptional contract are the
/// normalization's, because none of them reaches the conversion under test; the
/// clauses are the subject.
fn piecewise(clauses: Vec<AccuracyDomainClause>) -> AccuracyContract {
    AccuracyContract::new(
        rms_norm_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([DomainInterval::unbounded()], clauses)
                .expect("the synthetic domain is canonical"),
        ),
        rms_norm_f32_rsqrt_exceptional_contract(),
    )
}

/// Builds one clause over the whole domain, proving nothing about the reference.
fn whole_domain_clause(predicate: AccuracyPredicate) -> AccuracyDomainClause {
    AccuracyDomainClause::new(
        [(OperandOrdinal::new(0), DomainInterval::unbounded())],
        ReferenceResultConstraint::unconstrained(),
        predicate,
    )
    .expect("the synthetic clause is canonical")
}

/// The registered softmax requirement converts to twenty-four unit roundoffs.
///
/// **The number, and the ratio it implies, both asserted exactly.** Twelve ULPs
/// under `tiler::ulp-reference-gap@1` is `12 * 2^-23`, which is `24u` — and both
/// published bound records instantiate their prices at `eps_exp = u`, whose
/// first-order price `(u + u)` is `12.5` times smaller than `(u + 24u)`. Asserting
/// the ratio rather than describing it is what makes a query that quietly returned
/// `u` fail here instead of looking right.
#[test]
fn the_registered_softmax_accuracy_is_twenty_four_unit_roundoffs() {
    let accuracy = elementary_relative_accuracy_from(
        &softmax_f32_op(),
        &discharging_installation(),
        &installed_implication_registry(),
    )
    .expect("a both-halves fixture realizes the softmax's exponential");
    assert_eq!(accuracy.operation(), &softmax_f32_op());
    assert_eq!(
        accuracy.bound().value(),
        &ExactRational::from_integer(i128::from(SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE))
            .multiply(&ExactRational::power_of_two(-23))
    );
    assert_eq!(
        accuracy.bound().value(),
        &ExactRational::from_integer(24).multiply(&unit_roundoff())
    );
    // The first-order price ratio the two records' `eps_exp = u` instantiation
    // costs, in exact rational arithmetic.
    let published = unit_roundoff().add(&unit_roundoff());
    let requirement_side = unit_roundoff().add(accuracy.bound().value());
    assert_eq!(
        requirement_side
            .divide(&published)
            .expect("twice the unit roundoff is nonzero"),
        ExactRational::from_ratio(25, 2).expect("a nonzero denominator")
    );
    // The metric conversion's own obligation travels with the number.
    assert_eq!(
        accuracy.domain(),
        &RelativeAccuracyDomain::ReferenceMagnitudeAtOrAbove(least_normal())
    );
    let RefinementBasis::RegisteredImplication { .. } = accuracy.admission_basis() else {
        panic!("the number rests on the cross-metric admission: {accuracy:?}");
    };
}

/// The activation's exponential gives the same number, because the bound is one.
///
/// The two requirements share a tolerance and a metric and differ in domain, and
/// the domain does not reach this conversion — so agreement here is the expected
/// answer rather than a coincidence, and a divergence would mean one family's
/// tolerance moved without the other's.
#[test]
fn the_two_exponentials_yield_one_relative_accuracy() {
    let activation = elementary_relative_accuracy_from(
        &silu_f32_op(),
        &discharging_installation(),
        &installed_implication_registry(),
    )
    .expect("a both-halves fixture realizes the activation's exponential");
    let softmax = elementary_relative_accuracy_from(
        &softmax_f32_op(),
        &discharging_installation(),
        &installed_implication_registry(),
    )
    .expect("a both-halves fixture realizes the softmax's exponential");
    assert_eq!(SILU_F32_EXPONENTIAL_ULP_TOLERANCE, 12);
    assert_eq!(activation.bound(), softmax.bound());
    assert_eq!(activation.domain(), softmax.domain());
    assert_ne!(activation.operation(), softmax.operation());
}

/// A faithful requirement converts to two unit roundoffs, with no metric crossed.
///
/// The obligation is still conditional on the subnormal band: a faithful result in
/// that band is a neighbour of the reference under a *constant* spacing, so the
/// relative error is unbounded there exactly as the ULP conversion's is.
#[test]
fn the_faithful_normalization_requirement_gives_two_unit_roundoffs() {
    let accuracy = elementary_relative_accuracy_from(
        &rms_norm_f32_op(),
        &discharging_installation(),
        &installed_implication_registry(),
    )
    .expect("a both-halves fixture realizes the reciprocal square root");
    assert_eq!(
        accuracy.bound().value(),
        &ExactRational::from_integer(2).multiply(&unit_roundoff())
    );
    assert_eq!(
        accuracy.domain(),
        &RelativeAccuracyDomain::ReferenceMagnitudeAtOrAbove(least_normal())
    );
    assert_eq!(
        accuracy.admission_basis(),
        &RefinementBasis::IdenticalNormalizedContract
    );
}

/// The number is gated on the admission, not on the requirement's existence.
///
/// **This is the perturbation that separates a requirement-side query from a
/// requirement *lookup*.** The registered requirement is untouched and would still
/// convert; only the registry loses the cross-metric derivation, so no installed
/// realization refines and the target has declared nothing about the operation. A
/// query that returned the requirement's tolerance regardless would pass every
/// other test here and fail only this one.
#[test]
fn an_unrefined_realization_yields_no_number() {
    let refusal = elementary_relative_accuracy_from(
        &softmax_f32_op(),
        &installed_elementary_realizations(),
        &RegisteredImplicationRegistry::standard().expect("the governed registry composes"),
    )
    .expect_err("an unrefined declaration bounds nothing");
    assert_eq!(
        refusal.diagnostic_code(),
        "accuracy.elementary.unrefined-realization"
    );
    assert_eq!(refusal.operation(), &softmax_f32_op());
    let RelativeAccuracyRefusalReason::Unrealized(inner) = refusal.reason() else {
        panic!("the refusal is the refinement authority's own: {refusal:?}");
    };
    assert!(matches!(
        inner.reason(),
        ElementaryRefusalReason::Unrefined { .. }
    ));
}

/// A profile installing no realization at all is refused as undeclared.
#[test]
fn an_uninstalled_operation_yields_no_number() {
    let refusal = elementary_relative_accuracy_from(
        &softmax_f32_op(),
        &[],
        &installed_implication_registry(),
    )
    .expect_err("an empty installation declares no accuracy");
    assert_eq!(
        refusal.diagnostic_code(),
        "accuracy.elementary.no-installed-realization"
    );
}

/// An operation with no registered elementary obligation is refused by name.
///
/// Distinct from an unrealized one and deliberately so: an addition has no
/// elementary evaluation, which is not the same claim as having a perfect one, and
/// returning zero would let a bound charge nothing for a function nobody bounded.
#[test]
fn an_operation_with_no_registered_requirement_yields_no_number() {
    let refusal = elementary_relative_accuracy(
        &tiler_ir::semantic::add_f32_op(),
        &crate::target::TargetProfile::governed(),
    )
    .expect_err("the addition carries no elementary accuracy obligation");
    assert_eq!(
        refusal.diagnostic_code(),
        "accuracy.elementary.no-registered-requirement"
    );
    assert!(matches!(
        refusal.reason(),
        RelativeAccuracyRefusalReason::NoRegisteredRequirement
    ));
}

/// A correctly rounded contract is the one that yields exactly `u`.
///
/// The instantiation both bound records label a choice about the target. Pinning it
/// to the form that actually states correct rounding is what makes quoting `u` for
/// a twelve-ULP requirement a detectable substitution.
#[test]
fn a_correctly_rounded_requirement_gives_exactly_one_unit_roundoff() {
    let contract = AccuracyContract::new(
        rms_norm_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::CorrectlyRounded {
            rounding: ReferenceRoundingRule::NearestTiesToEven,
        },
        rms_norm_f32_rsqrt_exceptional_contract(),
    );
    let (bound, domain) = relative_accuracy_of_contract(&contract, &f32_format())
        .expect("a correctly rounded contract converts");
    assert_eq!(bound.value(), &unit_roundoff());
    assert_eq!(
        domain,
        RelativeAccuracyDomain::ReferenceMagnitudeAtOrAbove(least_normal())
    );
}

/// A relative predicate needs no metric step, so it carries no subnormal condition.
#[test]
fn a_relative_predicate_needs_no_metric_step() {
    let tolerance = ExactTolerance::from_ratio(1, 1_000).expect("a nonnegative ratio");
    let contract = piecewise(vec![whole_domain_clause(AccuracyPredicate::relative(
        tolerance.clone(),
    ))]);
    let (bound, domain) = relative_accuracy_of_contract(&contract, &f32_format())
        .expect("a relative predicate is already relative");
    assert_eq!(bound, tolerance);
    assert_eq!(domain, RelativeAccuracyDomain::EveryAdmittedReference);
}

/// A clause proving its reference is normal discharges the conversion's condition.
///
/// **The path no registered contract takes today, exercised so the discharge is a
/// mechanism rather than a claim.** The proof is read from the clause's own
/// reference-result constraint — ADR 0042 admits one only through an
/// operation-specific proof — so this cannot be reached by inferring normality from
/// an input domain.
#[test]
fn a_clause_proving_a_normal_reference_discharges_the_subnormal_condition() {
    let proved = AccuracyDomainClause::new(
        [(OperandOrdinal::new(0), DomainInterval::unbounded())],
        ReferenceResultConstraint::new(
            [ReferenceResultClass::Positive],
            Some(
                DomainInterval::new(
                    OperandOrdinal::new(0),
                    DomainBound::Closed(least_normal()),
                    DomainBound::Unbounded,
                )
                .expect("the synthetic magnitude interval is nonempty"),
            ),
            Some(justification()),
        )
        .expect("the synthetic reference constraint is canonical"),
        AccuracyPredicate::ulp(
            ulp_reference_gap_metric_key(),
            ExactTolerance::from_integer(4),
        ),
    )
    .expect("the synthetic clause is canonical");
    let (bound, domain) = relative_accuracy_of_contract(&piecewise(vec![proved]), &f32_format())
        .expect("a governed ULP predicate converts");
    assert_eq!(
        bound.value(),
        &ExactRational::from_integer(4).multiply(&ExactRational::power_of_two(-23))
    );
    assert_eq!(domain, RelativeAccuracyDomain::EveryAdmittedReference);

    // The same predicate with nothing proved about the reference stays conditional,
    // which is the control that keeps the assertion above about the proof rather
    // than about the predicate.
    let (_, unproved) = relative_accuracy_of_contract(
        &piecewise(vec![whole_domain_clause(AccuracyPredicate::ulp(
            ulp_reference_gap_metric_key(),
            ExactTolerance::from_integer(4),
        ))]),
        &f32_format(),
    )
    .expect("a governed ULP predicate converts");
    assert_eq!(
        unproved,
        RelativeAccuracyDomain::ReferenceMagnitudeAtOrAbove(least_normal())
    );
}

/// The weakest clause decides, and one clause's proof does not cover another's.
///
/// A tighter clause binds only its own region, so a maximum is the sound fold and a
/// minimum would price a rewrite against a region it does not stay inside. The
/// second half is the sharper claim: a proved-normal clause beside an unproved one
/// leaves the whole answer conditional.
#[test]
fn the_weakest_clause_decides_and_an_unproved_one_keeps_the_condition() {
    let tight = whole_domain_clause(AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(1),
    ));
    let loose = AccuracyDomainClause::new(
        [(
            OperandOrdinal::new(0),
            DomainInterval::new(
                OperandOrdinal::new(0),
                DomainBound::Closed(ExactRational::zero()),
                DomainBound::Unbounded,
            )
            .expect("the synthetic operand interval is nonempty"),
        )],
        ReferenceResultConstraint::new(
            [ReferenceResultClass::Positive],
            Some(
                DomainInterval::new(
                    OperandOrdinal::new(0),
                    DomainBound::Closed(least_normal()),
                    DomainBound::Unbounded,
                )
                .expect("the synthetic magnitude interval is nonempty"),
            ),
            Some(justification()),
        )
        .expect("the synthetic reference constraint is canonical"),
        AccuracyPredicate::ulp(
            ulp_reference_gap_metric_key(),
            ExactTolerance::from_integer(9),
        ),
    )
    .expect("the synthetic clause is canonical");
    let (bound, domain) =
        relative_accuracy_of_contract(&piecewise(vec![tight, loose]), &f32_format())
            .expect("both clauses convert");
    assert_eq!(
        bound.value(),
        &ExactRational::from_integer(9).multiply(&ExactRational::power_of_two(-23)),
        "the weakest obligation is the one every evaluation is held to"
    );
    assert_eq!(
        domain,
        RelativeAccuracyDomain::ReferenceMagnitudeAtOrAbove(least_normal()),
        "one clause's proof does not discharge another clause's conversion"
    );
}

/// A bound under another metric is refused rather than crossed.
///
/// The implication registry crosses one ULP definition to another; this conversion
/// leaves the metric algebra entirely for a ratio against `|r|`, and a registered
/// scaling factor says nothing about that ratio. Reusing the factor here would be
/// the same name-matching ADR 0042 forbids, one level up.
#[test]
fn a_bound_under_another_metric_is_refused_rather_than_crossed() {
    let contract = piecewise(vec![whole_domain_clause(AccuracyPredicate::ulp(
        apple_msl_ulp_metric_key(),
        ExactTolerance::from_integer(4),
    ))]);
    let reason = relative_accuracy_of_contract(&contract, &f32_format())
        .expect_err("a vendor metric is not this conversion's");
    let RelativeAccuracyRefusalReason::UnconvertibleMetric { metric } = reason else {
        panic!("the refusal names the metric: {reason:?}");
    };
    assert_eq!(metric, apple_msl_ulp_metric_key());
}

/// Every predicate shape with no sound conversion refuses by its own name.
///
/// The population is named and counted rather than sampled: the four shapes below
/// plus `Ulp` and `Relative` are the whole closed vocabulary, so a new predicate
/// kind is a build error at the conversion's exhaustive match rather than a silent
/// pass through this test.
#[test]
fn a_predicate_with_no_sound_relative_conversion_refuses_by_name() {
    let half = ExactTolerance::from_ratio(1, 2).expect("a nonnegative ratio");
    let cases: Vec<(AccuracyPredicate, &'static str)> = vec![
        (AccuracyPredicate::absolute(half.clone()), "absolute"),
        (
            AccuracyPredicate::absolute_relative(half.clone(), half.clone()),
            "absolute-relative",
        ),
        (
            AccuracyPredicate::all_of([
                AccuracyPredicate::absolute(half.clone()),
                AccuracyPredicate::relative(half.clone()),
            ])
            .expect("a two-member conjunction is canonical"),
            "all-of",
        ),
        (
            AccuracyPredicate::any_of([
                AccuracyPredicate::absolute(half.clone()),
                AccuracyPredicate::relative(half),
            ])
            .expect("a two-member disjunction is canonical"),
            "any-of",
        ),
    ];
    assert_eq!(
        cases.len(),
        4,
        "the unconvertible population is four shapes"
    );
    for (predicate, expected) in cases {
        let reason = relative_accuracy_of_contract(
            &piecewise(vec![whole_domain_clause(predicate)]),
            &f32_format(),
        )
        .expect_err("this shape has no sound relative conversion");
        let RelativeAccuracyRefusalReason::UnconvertiblePredicate { predicate } = reason else {
            panic!("the refusal names the predicate kind: {reason:?}");
        };
        assert_eq!(predicate, expected);
    }
}

/// A named-elementary profile refuses rather than guessing a tolerance.
///
/// The result set lives in a descriptor this build holds only a digest of, which is
/// the same boundary `decide_contract` reports as `NamedProfileNotInterpretable`. A
/// number invented here would be exactly the plausible constant the query exists to
/// replace.
#[test]
fn a_named_elementary_requirement_refuses_rather_than_guessing() {
    let contract = AccuracyContract::new(
        rms_norm_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::NamedElementary {
            profile: NamedElementaryProfileKey::new("vendor", "rsqrt-profile", 1)
                .expect("the synthetic profile key is valid"),
            descriptor_digest: NamedElementaryDescriptorDigest::new(b"sha256:synthetic")
                .expect("a nonempty digest"),
            descriptor_basis: justification(),
        },
        rms_norm_f32_rsqrt_exceptional_contract(),
    );
    let reason = relative_accuracy_of_contract(&contract, &f32_format())
        .expect_err("a descriptor held by digest states no tolerance");
    assert!(matches!(
        reason,
        RelativeAccuracyRefusalReason::NamedProfileNotInterpretable
    ));
}
