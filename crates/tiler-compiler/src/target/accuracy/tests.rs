use tiler_ir::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, ConformanceEvidenceClass, ConformanceEvidenceError,
    ExactTolerance, ReferenceRoundingRule, RefinementBasis, RefinementOutcome, RefinementUnknown,
    RegisteredImplicationRegistry, refines, ulp_reference_gap_metric_key,
};
use tiler_ir::semantic::{
    F32, SILU_F32_EXPONENTIAL_ULP_TOLERANCE, rms_norm_f32_op, rms_norm_f32_rsqrt_accuracy_contract,
    rms_norm_f32_rsqrt_exceptional_contract, rms_norm_f32_rsqrt_reference_semantics,
    silu_f32_exponential_accuracy_contract, silu_f32_op,
};

use super::{
    APPLE_MSL_EXP_F32_ULP_BOUND, APPLE_ULP_TRANSLATION_FACTOR, ElementaryRefusalReason,
    apple_msl_ulp_metric_key, assess_elementary_accuracy, installed_elementary_realizations,
    installed_implication_registry, metal_f32_exceptional_value_evidence,
    metal_f32_exponential_bound_evidence, metal_f32_exponential_contract,
    metal_f32_normalization_exceptional_value_evidence,
    metal_f32_reciprocal_square_root_bound_evidence, metal_f32_reciprocal_square_root_contract,
};

fn required() -> AccuracyContract {
    silu_f32_exponential_accuracy_contract()
}

/// The installed Metal realization provably refines the resolved contract.
///
/// The admission is by the *registered* implication rather than by an identical
/// contract or a tighter bound of the same shape, which is what makes the
/// derivation load-bearing rather than decorative.
#[test]
fn the_metal_realization_refines_the_resolved_contract_through_the_registered_implication() {
    let admission = assess_elementary_accuracy(
        &required(),
        &installed_elementary_realizations(),
        &installed_implication_registry(),
    )
    .expect("the installed realization refines the requirement");
    let RefinementBasis::RegisteredImplication { .. } = admission.basis() else {
        panic!("the admission rests on a registered implication: {admission:?}");
    };
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

    let admission = assess_elementary_accuracy(
        &required(),
        &installed_elementary_realizations(),
        &installed_implication_registry(),
    )
    .expect("refines");
    let discharge = admission.discharge();
    assert!(discharge.bound_is_discharged());
    assert!(
        !discharge.exceptional_is_discharged(),
        "an empirical qualification does not become a guarantee by sitting beside one"
    );
    assert_eq!(
        discharge.exceptional_class(),
        ConformanceEvidenceClass::EmpiricalQualification
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

/// The installed Metal realization refines the normalization's requirement.
///
/// The admission rests on an *identical normalized contract* rather than on a
/// registered implication, and that difference is the finding: what Metal
/// promises for `rsqrt` and what `tiler::rms-norm-f32@1` requires are the same
/// result set, so there is no translation to perform.
#[test]
fn the_metal_normalization_realization_refines_by_identity_rather_than_implication() {
    let admission = assess_elementary_accuracy(
        &rms_norm_f32_rsqrt_accuracy_contract(),
        &installed_elementary_realizations(),
        &installed_implication_registry(),
    )
    .expect("the installed realization refines the requirement");
    assert_eq!(
        admission.basis(),
        &RefinementBasis::IdenticalNormalizedContract
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
    let admission = assess_elementary_accuracy(
        &rms_norm_f32_rsqrt_accuracy_contract(),
        &installed_elementary_realizations(),
        &RegisteredImplicationRegistry::empty(),
    )
    .expect("a faithful requirement needs no implication");
    assert_eq!(
        admission.basis(),
        &RefinementBasis::IdenticalNormalizedContract
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

/// Both registered families have an installed realization, and only those two.
#[test]
fn the_installed_realizations_are_exactly_the_two_registered_families() {
    let installed = installed_elementary_realizations();
    let operations: Vec<String> = installed
        .iter()
        .map(|realization| realization.operation().to_string())
        .collect();
    assert_eq!(
        operations,
        vec!["tiler::silu-f32@1", "tiler::rms-norm-f32@1"]
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
