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
// The declared subnormal behaviour
// ---------------------------------------------------------------------------

/// Returns the subnormal policy exactly as the *registered* definition carries it.
///
/// Read through the registry rather than from [`silu_f32_facts`], so a
/// registration carrying some other record than the declared one fails here
/// instead of agreeing with itself.
fn registered_subnormal_fact() -> String {
    let registry = registry();
    let facts = registry
        .operation_facts(&silu_f32_op())
        .expect("the SiLU definition carries facts");
    let CanonicalValueView::Record(fields) = facts.value().view() else {
        panic!("the governed facts are a record");
    };
    let carried = fields
        .iter()
        .find(|field| field.id() == SILU_F32_FACT_SUBNORMALS)
        .map(CanonicalField::value)
        .expect("the facts carry the subnormal policy");
    let CanonicalValueView::Utf8(text) = carried.view() else {
        panic!("the subnormal policy is a fact string");
    };
    text.to_owned()
}

/// The declared subnormal policy is true over the whole domain it quantifies.
///
/// Four assertions, each refusing a different defect, ordered so the specific
/// diagnosis fires before the opaque one. The `unreachable` assertion refuses the
/// claim this fact carried before —
/// `preserved-and-unreachable-no-binary32-silu-result-or-intermediate-is-subnormal`,
/// which generalized from the large-negative tail alone. The two region assertions
/// refuse a replacement that repairs one region and then generalizes from *it* in
/// the same way: near zero it is the *result* that is subnormal, and for large
/// positive arguments it is the *subordinate exponential*, so a spelling naming
/// one of the two is as untrue over the domain as one naming neither. Only then
/// does the exact value pin the spelling.
#[test]
fn the_declared_subnormal_policy_covers_every_region_of_the_domain() {
    let value = registered_subnormal_fact();
    assert!(
        !value.contains("unreachable"),
        "the band is reached in two regions, so no spelling may declare it unreachable: {value}"
    );
    assert!(
        value.contains("near-zero"),
        "the near-zero region, where the reference is x / 2, produces subnormal results: {value}"
    );
    assert!(
        value.contains("subordinate-exponential"),
        "the subordinate exponential is itself subnormal for large positive arguments: {value}"
    );
    assert_eq!(
        value,
        "preserved-by-this-contract-and-reached-as-a-result-near-zero-where-the-reference-is-x-\
         over-two-and-as-the-subordinate-exponential-for-large-positive-arguments-and-flushed-\
         on-a-declared-flushing-realization-a-recorded-divergence"
    );
}

/// The near-zero region the fact names, evaluated at the exact bits that bound it.
///
/// The divisor is *derived* rather than taken from a host exponential, so this
/// case pins the composition and not a library. For `0 < |x| <= 2^-25` the
/// reference `e^-x` lies strictly between `1 - 2^-25` and `1`, because
/// `e^-t > 1 - t` at every `t > 0`. The predecessor of `1.0` is `1 - 2^-24`, so
/// `1 - 2^-25` is exactly the rounding midpoint below `1.0` and the correctly
/// rounded exponential over the whole region is `1.0`. Everything after that is
/// binary32 division by exactly `2.0`, which the host performs exactly.
#[test]
fn the_near_zero_region_produces_subnormal_results_at_the_pinned_bits() {
    let region_ceiling = f32::from_bits(0x3300_0000);
    let predecessor_of_one = f32::from_bits(0x3f7f_ffff);
    let midpoint_below_one = f64::midpoint(f64::from(predecessor_of_one), 1.0);
    assert_eq!(
        midpoint_below_one.to_bits(),
        (1.0 - f64::from(region_ceiling)).to_bits(),
        "the region ceiling is exactly the distance from 1.0 down to its rounding midpoint, \
         which is what makes the correctly rounded exponential 1.0 over the whole region"
    );
    let divisor = 1.0_f32 + 1.0_f32;
    assert_eq!(
        divisor.to_bits(),
        2.0_f32.to_bits(),
        "1.0 + 1.0 is exact, so the divisor is exactly 2.0"
    );

    for (argument, result, subnormal) in [
        (0x0000_0001_u32, 0x0000_0000_u32, false),
        (0x0000_0002, 0x0000_0001, true),
        (0x007f_fffe, 0x003f_ffff, true),
        (0x007f_ffff, 0x0040_0000, true),
        (0x0080_0000, 0x0040_0000, true),
        (0x00ff_fffe, 0x007f_ffff, true),
        (0x00ff_ffff, 0x0080_0000, false),
        (0x807f_ffff, 0x8040_0000, true),
        (0x8080_0000, 0x8040_0000, true),
    ] {
        let value = f32::from_bits(argument) / divisor;
        assert_eq!(
            value.to_bits(),
            result,
            "silu(0x{argument:08x}) is 0x{result:08x}"
        );
        assert_eq!(
            value.is_subnormal(),
            subnormal,
            "silu(0x{argument:08x}) subnormality"
        );
    }

    assert!(
        !f32::from_bits(0x0080_0000).is_subnormal() && !f32::from_bits(0x00ff_fffe).is_subnormal(),
        "the arguments from 0x00800000 to 0x00fffffe are normal and their images are subnormal, \
         so a subnormal result does not require a subnormal operand"
    );
    assert!(
        f32::from_bits(0x0000_0001) != 0.0,
        "0x00000001 is a nonzero argument whose image is exactly zero by ties-to-even, which is \
         a rounding and not a flush"
    );
}

/// No argument at or beyond the exponential's argument ceiling has a subnormal image.
///
/// The old spelling of the subnormal fact generalized over this whole region from
/// two samples in it. Two samples cannot establish a region, so this states the
/// bound instead: a finite divisor is `fl(1 + e)` for a finite non-negative `e`
/// and is therefore at most `f32::MAX`, so an argument of magnitude at least the
/// ceiling has an image of magnitude at least `ceiling / f32::MAX`. An infinite
/// divisor gives an exact signed zero, which is not subnormal either.
#[test]
fn no_argument_beyond_the_exponential_ceiling_has_a_subnormal_image() {
    assert_eq!(
        (1.0_f32 + f32::MAX).to_bits(),
        f32::MAX.to_bits(),
        "a finite divisor cannot exceed f32::MAX, which is what bounds the image below"
    );
    let ceiling = f32::from_bits(SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS);
    let smallest_magnitude = f64::from(ceiling) / f64::from(f32::MAX);
    assert!(
        smallest_magnitude > 20.0 * f64::from(f32::MIN_POSITIVE),
        "the smallest magnitude the tail can produce is {smallest_magnitude:e}, more than twenty \
         times the minimum normal {:e}, so the region holds no subnormal at all",
        f64::from(f32::MIN_POSITIVE)
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
