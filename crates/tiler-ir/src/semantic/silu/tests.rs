use super::*;
use crate::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyPredicateView, ResultSetEstablishment,
    ulp_reference_gap_metric_key,
};
use crate::semantic::{
    Bf16, CanonicalValueView, FrozenSemanticRegistry, OperationAttributes, RegistryError,
    ResolvedValueType, add_f32_op, builtin_scalar_value_type_facts, constant_f32_op,
};
use crate::shape::Shape;

fn registry() -> FrozenSemanticRegistry {
    FrozenSemanticRegistry::standard().expect("the standard registry builds")
}

fn operand(resolved_type: ResolvedValueType, dims: &[u64]) -> ValueFact {
    ValueFact::new(
        resolved_type,
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn infer(operands: &[ValueFact]) -> Result<Vec<ValueFact>, RegistryError> {
    registry().infer_operation(&silu_f32_op(), operands, &OperationAttributes::empty())
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operands: &[ValueFact], attributes: &OperationAttributes) -> String {
    let error = registry()
        .infer_operation(&silu_f32_op(), operands, attributes)
        .expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a SiLU refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

// ---------------------------------------------------------------------------
// Registration and identity
// ---------------------------------------------------------------------------

/// The key is registered, beside the existing families rather than widening one.
#[test]
fn the_silu_key_is_registered_beside_the_existing_f32_families() {
    let registry = registry();
    assert!(registry.operation_definition(&silu_f32_op()).is_some());
    assert_eq!(silu_f32_op().to_string(), "tiler::silu-f32@1");
    assert_ne!(silu_f32_op(), add_f32_op());
    assert_ne!(silu_f32_op(), constant_f32_op());
    assert!(registry.operation_definition(&add_f32_op()).is_some());
    assert!(registry.operation_definition(&constant_f32_op()).is_some());
}

/// No general exponential or sigmoid key came with it.
///
/// The ticket's stated non-goals, checked rather than asserted in prose: a later
/// change that mints either key has to move this test and say why.
#[test]
fn no_general_exponential_or_sigmoid_key_is_registered() {
    let registry = registry();
    for name in ["exp-f32", "sigmoid-f32", "gelu-f32", "divide-f32"] {
        let key = OpKey::new("tiler", name, 1).expect("a probe key is well formed");
        assert!(
            registry.operation_definition(&key).is_none(),
            "{key} is a non-goal of the SiLU admission and is not registered"
        );
    }
}

/// The registered reference pins the division form, not the activation's name.
#[test]
fn the_registered_reference_pins_the_division_form() {
    let registry = registry();
    let definition = registry
        .operation_definition(&silu_f32_op())
        .expect("the SiLU definition is registered");
    let reference = definition.normative_definition().as_str();
    assert!(reference.contains("x / (1 + Exp(-x))"));
    assert!(reference.contains("ADR 0024"));
    assert!(
        reference.contains("not x * Sigmoid(x)"),
        "the reference rules out the spelling that differs from it"
    );
}

// ---------------------------------------------------------------------------
// The accuracy contract
// ---------------------------------------------------------------------------

/// The definition's facts carry the complete accuracy contract, canonically.
///
/// This is ADR 0016's requirement made checkable. The contract is inside the
/// registered definition, so the registry's own definition projection folds it,
/// and two builds resolving different accuracy for this key cannot reach the same
/// identity.
#[test]
fn the_registered_facts_carry_the_complete_accuracy_contract() {
    let facts = silu_f32_facts();
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("the governed facts are a record");
    };
    let carried = fields
        .iter()
        .find(|field| field.id() == SILU_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT)
        .map(CanonicalField::value)
        .expect("the facts carry the accuracy contract");
    let decoded = AccuracyContract::from_canonical_value(carried)
        .expect("the carried contract decodes exactly as it was written");
    assert_eq!(
        decoded.canonical_encoding(),
        silu_f32_exponential_accuracy_contract().canonical_encoding(),
        "the contract survives canonical serialization without loss"
    );

    let registry = registry();
    let registered = registry
        .operation_facts(&silu_f32_op())
        .expect("the SiLU definition carries facts");
    assert_eq!(
        registered.value(),
        &facts,
        "the declared record and the registered definition are one authority"
    );
}

/// The contract verifies, and the establishment names how it was decided.
#[test]
fn the_accuracy_contract_verifies_over_its_admitted_domain() {
    let contract = silu_f32_exponential_accuracy_contract();
    let facts =
        builtin_scalar_value_type_facts(&F32::resolved_type()).expect("f32 is a governed built-in");
    let verified = contract
        .verify(&facts)
        .expect("the governed contract verifies");
    assert!(matches!(
        verified.establishment(),
        ResultSetEstablishment::RoundToNearestWitness { .. }
    ));
    assert_eq!(
        verified.contract().canonical_encoding(),
        contract.canonical_encoding()
    );
}

/// The contract states the derived twelve-ULP bound under Tiler's own metric.
///
/// Spelled out rather than compared against a re-derivation, so that changing the
/// derivation moves this assertion instead of moving silently with it.
#[test]
fn the_contract_states_twelve_ulps_under_the_governed_metric() {
    let contract = silu_f32_exponential_accuracy_contract();
    let AccuracyContractForm::BoundedPiecewise(domain) = contract.form() else {
        panic!("the exponential's contract is bounded piecewise");
    };
    let [clause] = domain.clauses() else {
        panic!("the contract states exactly one clause");
    };
    let AccuracyPredicateView::Ulp { metric, tolerance } = clause.predicate().view() else {
        panic!("the clause states a ULP bound");
    };
    assert_eq!(*metric, ulp_reference_gap_metric_key());
    assert_eq!(
        *tolerance.value(),
        ExactRational::from_integer(i128::from(SILU_F32_EXPONENTIAL_ULP_TOLERANCE))
    );
    assert_eq!(SILU_F32_EXPONENTIAL_ULP_TOLERANCE, 12);
}

/// The accuracy domain stops exactly where the metric stops being defined.
///
/// Both halves are checked, because a bound merely *somewhere* below the real
/// threshold would either leave admissible arguments uncovered or claim ones the
/// metric cannot measure.
#[test]
fn the_accuracy_domain_excludes_the_finite_overflow_region() {
    let ceiling = f32::from_bits(SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS);
    let successor = f32::from_bits(SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS + 1);
    assert!(f64::from(ceiling).exp() <= f64::from(f32::MAX));
    assert!(f64::from(successor).exp() > f64::from(f32::MAX));

    let contract = silu_f32_exponential_accuracy_contract();
    let AccuracyContractForm::BoundedPiecewise(domain) = contract.form() else {
        panic!("the exponential's contract is bounded piecewise");
    };
    let [admitted] = domain.admitted() else {
        panic!("the exponential takes one argument");
    };
    assert!(admitted.contains(&ExactRational::from_f32(ceiling).expect("finite")));
    assert!(!admitted.contains(&ExactRational::from_f32(successor).expect("finite")));
}

/// The exceptional-value contract is stated in full and independently.
///
/// Every one of the four rules is named. ADR 0042 makes them independent of the
/// error metric and `refines` refuses outright when two contracts state different
/// ones, so an implementation must reproduce this record rather than approximate
/// it.
#[test]
fn the_exceptional_value_contract_states_all_four_rules() {
    let exceptional = silu_f32_exponential_exceptional_contract();
    assert_eq!(exceptional.nan_reference(), NanReferenceRule::CanonicalNan);
    assert_eq!(
        exceptional.infinite_reference(),
        InfiniteReferenceRule::SignedInfinity
    );
    assert_eq!(exceptional.outside_domain(), DomainErrorRule::CanonicalNan);
    assert_eq!(
        exceptional.finite_overflow(),
        FiniteOverflowRule::SignedInfinity,
        "the -88.73 band's exact negative zero rests on the exponential overflowing to +inf"
    );
    assert_eq!(
        silu_f32_exponential_accuracy_contract().exceptional(),
        exceptional
    );
}

/// The contract names the registered key rather than a minted exponential.
#[test]
fn the_contract_names_the_registered_key() {
    let contract = silu_f32_exponential_accuracy_contract();
    assert_eq!(contract.operation(), &silu_f32_op());
    assert_eq!(contract.operand_types(), &[F32::resolved_type()]);
    assert_eq!(contract.result_type(), &F32::resolved_type());
    assert!(
        contract
            .reference_semantics()
            .as_str()
            .contains("mints no key of its own")
    );
}

// ---------------------------------------------------------------------------
// Application refusals
// ---------------------------------------------------------------------------

/// A well-typed application infers the operand's own shape.
#[test]
fn the_activation_preserves_its_operand_shape() {
    let results = infer(&[operand(F32::resolved_type(), &[4, 3])]).expect("silu applies");
    let [result] = results.as_slice() else {
        panic!("SiLU produces one result");
    };
    assert_eq!(result.resolved_type(), &F32::resolved_type());
    assert_eq!(
        result.shape(),
        &Shape::try_from_dims([4, 3]).expect("bounded")
    );
}

/// Each refusal is a separate named defect, from the authority that owns it.
///
/// The arity refusals come from the registered *schema* and the type refusal from
/// the operation's own inferencer, and the two codes differ because the two
/// authorities are different: a schema states how many operands the key takes and
/// an inferencer states what they may be. The inferencer keeps its own arity guard
/// as defence in depth for a direct call that never passed the schema.
#[test]
fn every_refusal_names_its_own_rule() {
    assert_eq!(
        refusal(&[], &OperationAttributes::empty()),
        "tiler.schema.operand-arity"
    );
    assert_eq!(
        refusal(
            &[
                operand(F32::resolved_type(), &[2]),
                operand(F32::resolved_type(), &[2])
            ],
            &OperationAttributes::empty()
        ),
        "tiler.schema.operand-arity"
    );
    assert_eq!(
        refusal(
            &[operand(Bf16::resolved_type(), &[2])],
            &OperationAttributes::empty()
        ),
        "silu.f32.implicit-promotion",
        "a bf16 operand is refused by name rather than converted"
    );
}
