use super::*;
use crate::semantic::accuracy::AccuracyContractForm;
use crate::semantic::{
    Bf16, FrozenSemanticRegistry, OperationAttributes, RegistryError, ResolvedValueType,
    SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS, SILU_F32_EXPONENTIAL_ULP_TOLERANCE, add_f32_op,
    constant_f32_op, rms_norm_f32_op, silu_f32_op, strict_serial_sum_f32_op,
};
use crate::shape::{
    Extent, ExtentSourceError, Shape, ShapeSymbol, SourcedExtent, SourcedShape, SymbolScope,
};

fn registry() -> FrozenSemanticRegistry {
    FrozenSemanticRegistry::standard().expect("the standard registry builds")
}

fn operand(resolved_type: ResolvedValueType, dims: &[u64]) -> ValueFact {
    ValueFact::new(
        resolved_type,
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    operand(F32::resolved_type(), dims)
}

/// A binary32 operand whose boundary may name a declared symbol.
fn symbolic_f32_operand(extents: &[SourcedExtent]) -> ValueFact {
    ValueFact::new(
        F32::resolved_type(),
        SourcedShape::sourced(extents.to_vec()).expect("a test boundary is bounded"),
    )
}

/// The governed single-axis attribute a well-formed occurrence carries.
fn attributes(axis: u32) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(
        SOFTMAX_REDUCED_AXES_ATTRIBUTE,
        softmax_f32_axis_attribute(Axis::new(axis)),
    )])
    .expect("the governed attribute is canonical")
}

/// The attribute with an arbitrary axis sequence, so a malformed axis is statable.
fn attributes_with_axes(axes: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(SOFTMAX_REDUCED_AXES_ATTRIBUTE, axes)])
        .expect("a probe attribute is canonical")
}

fn axis_sequence(axes: &[u32]) -> CanonicalValue {
    CanonicalValue::sequence(axes.iter().map(|axis| CanonicalValue::unsigned_u32(*axis)))
        .expect("a probe axis sequence is canonical")
}

fn infer(
    operands: &[ValueFact],
    attributes: &OperationAttributes,
) -> Result<Vec<ValueFact>, RegistryError> {
    registry().infer_operation(&softmax_f32_op(), operands, attributes)
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operands: &[ValueFact], attributes: &OperationAttributes) -> String {
    let error = infer(operands, attributes).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a softmax refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

// ---------------------------------------------------------------------------
// Registration and identity
// ---------------------------------------------------------------------------

/// The key is registered beside the existing families rather than widening one.
#[test]
fn the_softmax_key_is_registered_beside_the_existing_f32_families() {
    let registry = registry();
    assert!(registry.operation_definition(&softmax_f32_op()).is_some());
    assert_eq!(softmax_f32_op().to_string(), "tiler::softmax-f32@1");
    assert_ne!(softmax_f32_op(), silu_f32_op());
    assert_ne!(softmax_f32_op(), rms_norm_f32_op());
    assert_ne!(softmax_f32_op(), strict_serial_sum_f32_op());
    for existing in [silu_f32_op(), rms_norm_f32_op(), strict_serial_sum_f32_op()] {
        assert!(
            registry.operation_definition(&existing).is_some(),
            "{existing} must remain registered"
        );
    }
}

/// None of the stated non-goals came with it.
///
/// The ticket's non-goals checked rather than asserted in prose: a general `Exp`,
/// a standalone maximum reduction, log-softmax, and a `Select` for the derived
/// mask route. A later change that mints any of them has to move this test and
/// say why.
#[test]
fn no_general_exponential_maximum_reduction_or_log_softmax_key_is_registered() {
    let registry = registry();
    for name in [
        "exp-f32",
        "maximum-f32",
        "maximum-number-f32",
        "max-reduce-f32",
        "log-softmax-f32",
        "select-f32",
        "divide-f32",
        "subtract-f32",
    ] {
        let key = OpKey::new("tiler", name, 1).expect("a probe key is well formed");
        assert!(
            registry.operation_definition(&key).is_none(),
            "{key} is a non-goal of the softmax admission and is not registered"
        );
    }
}

/// The registered reference pins the subtraction, the family, and the reciprocal.
#[test]
fn the_registered_reference_pins_the_maximum_the_family_and_the_reciprocal() {
    let registry = registry();
    let definition = registry
        .operation_definition(&softmax_f32_op())
        .expect("the softmax definition is registered");
    let reference = definition.normative_definition().as_str();
    for required in [
        "NaN-propagating Maximum extrema family",
        "deliberately not MaximumNumber",
        "e_i = Exp(s_i - m)",
        "c = 1.0 / d as one division",
        "deliberately not e_i / d",
        "zero-length reduced axis yields a zero-length output",
        "do not sum to exactly 1.0",
    ] {
        assert!(
            reference.contains(required),
            "the reference must state {required:?}: {reference}"
        );
    }
}

// ---------------------------------------------------------------------------
// The resolved accuracy contract
// ---------------------------------------------------------------------------

/// The exponential's contract is a ULP bound over the non-positive reals.
#[test]
fn the_exponential_contract_is_a_ulp_bound_closed_at_zero() {
    let contract = softmax_f32_exponential_accuracy_contract();
    assert_eq!(contract.operation(), &softmax_f32_op());
    assert_eq!(contract.operand_types(), &[F32::resolved_type()]);
    assert_eq!(contract.result_type(), &F32::resolved_type());
    let AccuracyContractForm::BoundedPiecewise(domain) = contract.form() else {
        panic!("the exponential's contract is a bounded piecewise one");
    };
    let [interval] = domain.admitted() else {
        panic!("the exponential's domain is one admitted interval");
    };
    assert!(matches!(interval.lower(), DomainBound::Unbounded));
    assert!(matches!(interval.upper(), DomainBound::Closed(value) if value.is_zero()));
    // Closed at zero, so the largest reachable argument is admitted and the
    // smallest positive one is not.
    assert!(interval.contains(&ExactRational::zero()));
    assert!(!interval.contains(&ExactRational::power_of_two(-149)));
    assert!(interval.contains(&ExactRational::power_of_two(-149).negate()));
}

/// The domain is *narrower* than the activation's, and the narrowing is the point.
///
/// The maximum subtraction confines every argument to `(-inf, 0]`, so the
/// overflow band `tiler::silu-f32@1` reaches is unreachable here. Stating the
/// activation's wider ceiling would place an obligation on arguments this
/// operation cannot produce; the two ceilings are compared directly so a change
/// to either has to move this test.
#[test]
fn the_softmax_exponential_domain_is_narrower_than_the_activations() {
    // Compared as payloads rather than as values, because the ceiling *is* a bit
    // pattern: `+0.0` and `-0.0` compare equal as floats and are different
    // ceilings, so a value comparison would admit the one this domain excludes.
    assert_eq!(SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS, 0);
    assert_eq!(
        f32::from_bits(SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS).to_bits(),
        0.0_f32.to_bits()
    );
    let activation = f32::from_bits(SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS);
    assert!(
        activation > 0.0,
        "the activation's ceiling is in the overflow band, not at zero"
    );
    // Every argument this operation can reach is at or below zero, so the
    // reference never leaves `(0, 1]` and the finite-overflow rule is vacuous.
    assert_eq!(f64::from(0.0_f32).exp().to_bits(), 1.0_f64.to_bits());
}

/// The two exponential tolerances agree, because the derivation is one derivation.
///
/// Metal's Table 8.1 bounds *the exponential*, not the operation that calls one,
/// so the activation's twelve and this family's twelve are the same number for
/// the same reason. They are separate constants so that one moving for a reason
/// of its own cannot silently move the other — and this asserts the equality so
/// that a divergence has to be deliberate.
#[test]
fn the_two_exponential_tolerances_agree_because_the_derivation_is_one() {
    assert_eq!(
        SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE,
        SILU_F32_EXPONENTIAL_ULP_TOLERANCE
    );
    assert_eq!(SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE, 4 * 3);
}

/// The exceptional contract states all four rules independently of the form.
#[test]
fn the_exceptional_contract_states_four_independent_rules() {
    let exceptional = softmax_f32_exponential_exceptional_contract();
    assert_eq!(exceptional.nan_reference(), NanReferenceRule::CanonicalNan);
    assert_eq!(
        exceptional.infinite_reference(),
        InfiniteReferenceRule::SignedInfinity
    );
    assert_eq!(exceptional.outside_domain(), DomainErrorRule::CanonicalNan);
    assert_eq!(
        exceptional.finite_overflow(),
        FiniteOverflowRule::SignedInfinity
    );
    assert_eq!(
        softmax_f32_exponential_accuracy_contract().exceptional(),
        exceptional
    );
}

// ---------------------------------------------------------------------------
// Definition facts
// ---------------------------------------------------------------------------

/// The declared fact record is exactly the registered one.
#[test]
fn the_declared_facts_are_the_registered_facts() {
    let registry = registry();
    let definition = registry
        .operation_definition(&softmax_f32_op())
        .expect("the softmax definition is registered");
    assert_eq!(definition.canonical_facts().value(), &softmax_f32_facts());
}

/// The facts carry the complete resolved accuracy contract, not a summary of it.
#[test]
fn the_facts_carry_the_whole_resolved_accuracy_contract() {
    assert_eq!(
        fact_value(SOFTMAX_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT),
        softmax_f32_exponential_accuracy_contract()
            .to_canonical_value()
            .expect("the governed contract is canonical")
    );
}

/// Decision **D-2** is recorded as the NaN-propagating family, by name.
#[test]
fn the_extrema_family_fact_names_the_propagating_family_and_excludes_the_other() {
    let text = fact_text(SOFTMAX_F32_FACT_MAXIMUM_EXTREMA_FAMILY);
    assert!(text.starts_with("maximum-nan-propagating"), "{text}");
    assert!(text.contains("not-maximum-number"), "{text}");
    assert!(
        text.contains("negative-zero-below-positive-zero"),
        "the deterministic zero ordering is part of the family: {text}"
    );
}

/// Decision **D-1** is recorded as *no repair*, and the record says what follows.
#[test]
fn the_fully_masked_row_fact_states_no_repair_rather_than_an_answer() {
    let text = fact_text(SOFTMAX_F32_FACT_FULLY_MASKED_ROW);
    assert!(text.starts_with("no-special-case-and-no-repair"), "{text}");
    // Both consequences are named, because the operation returns one or the other
    // depending on the caller's mask convention rather than choosing.
    assert!(text.contains("uniform-under-a-finite-fill"), "{text}");
    assert!(text.contains("nan-under-an-infinite-one"), "{text}");
}

/// The two folds carry two facts, and they say different things.
///
/// This is the "one permission does not answer for both passes" property, checked
/// where a legality reasoner reads it. A change that collapsed the two facts into
/// one would fail here.
#[test]
fn the_two_reductions_state_their_order_obligations_separately() {
    let maximum = fact_text(SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY);
    let sum = fact_text(SOFTMAX_F32_FACT_SUM_FOLD_ORDER);
    assert_ne!(maximum, sum);
    assert!(maximum.contains("consuming-no-permission"), "{maximum}");
    assert!(maximum.contains("associative-and-commutative"), "{maximum}");
    assert!(sum.contains("strict-left-fold"), "{sum}");
    assert!(
        sum.contains("only-under-the-separate-reassociation-and-permutation-permissions"),
        "{sum}"
    );
}

/// The normalization form is a reciprocal multiplication and the permission is withheld.
///
/// The withholding runs in the *opposite* direction from the siblings': those pin
/// a division and refuse to become a reciprocal multiply, and this one pins the
/// multiply and refuses to become a division. Both statements are asserted, so a
/// row copied from a sibling would fail.
#[test]
fn the_normalization_is_a_reciprocal_multiplication_and_the_reverse_is_withheld() {
    assert_eq!(
        fact_text(SOFTMAX_F32_FACT_NORMALIZATION_FORM),
        "multiply-by-the-denominators-reciprocal-never-divide-by-the-denominator"
    );
    assert!(!fact_boolean(
        SOFTMAX_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED
    ));
}

/// The row sum is declared not to be one, in both directions.
#[test]
fn the_row_sum_fact_forbids_a_unit_sum_check() {
    let text = fact_text(SOFTMAX_F32_FACT_ROW_SUM);
    assert!(text.starts_with("not-exactly-one"), "{text}");
    assert!(text.contains("both-directions"), "{text}");
}

/// The empty reduced axis is declared outside the reduction empty-domain rules.
#[test]
fn the_empty_reduced_axis_fact_places_the_case_outside_the_reduction_rules() {
    let text = fact_text(SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS);
    assert!(text.contains("zero-length-output"), "{text}");
    assert!(text.contains("no-scalar-softmax-evaluated"), "{text}");
    assert!(
        text.contains("outside-the-reduction-empty-domain-rules"),
        "{text}"
    );
}

/// The online single-pass form names both freedoms it consumes, and refuses the
/// reassociation reading in its first clause.
///
/// The wall is that a scheduler reading this fact cannot spend a permission that
/// does not reach the rewrite. Two properties carry it and both are asserted: the
/// string refuses the reassociation reading before it says anything else, and it
/// names *both* missing freedoms — a fact naming one of two would imply that
/// granting that one admits the rewrite.
#[test]
fn the_online_single_pass_form_names_both_freedoms_it_consumes() {
    let text = fact_text(SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM);
    assert!(text.starts_with("not-a-reassociation-of-the-sum"), "{text}");
    assert!(text.contains("horner-nesting"), "{text}");
    assert!(text.contains("consuming-distributivity"), "{text}");
    assert!(text.contains("elementary-function-identity"), "{text}");
    assert!(
        text.contains("no-reassociation-or-permutation-permission-reaches-it"),
        "{text}"
    );
}

/// The accumulator type is stated explicitly rather than left to the element type.
#[test]
fn the_accumulator_type_is_declared_explicitly_as_binary32() {
    assert_eq!(fact_text(SOFTMAX_F32_FACT_ACCUMULATOR_TYPE), "tiler::f32@1");
}

/// The approximate-intrinsic and contraction permissions are withheld.
#[test]
fn the_approximate_intrinsic_and_contraction_permissions_are_withheld() {
    assert!(!fact_boolean(
        SOFTMAX_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED
    ));
    assert!(!fact_boolean(
        SOFTMAX_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED
    ));
}

fn fact_value(id: AttributeFieldId) -> CanonicalValue {
    let facts = softmax_f32_facts();
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("the fact record is a record");
    };
    fields
        .iter()
        .find(|field| field.id() == id)
        .expect("the requested fact is present")
        .value()
        .clone()
}

fn fact_text(id: AttributeFieldId) -> String {
    let value = fact_value(id);
    let CanonicalValueView::Utf8(text) = value.view() else {
        panic!("the requested fact is text");
    };
    text.to_owned()
}

fn fact_boolean(id: AttributeFieldId) -> bool {
    let value = fact_value(id);
    let CanonicalValueView::Bool(flag) = value.view() else {
        panic!("the requested fact is a boolean");
    };
    flag
}

// ---------------------------------------------------------------------------
// Inference and its typed refusals
// ---------------------------------------------------------------------------

/// A well-formed occurrence is shape-preserving at every rank the workload uses.
#[test]
fn the_softmax_preserves_its_operand_shape() {
    for (dims, axis) in [
        (vec![1_u64, 4], 1_u32),
        (vec![1, 16, 10, 10], 3),
        (vec![8, 2, 10, 10], 3),
        (vec![3, 5], 0),
    ] {
        let operands = [f32_operand(&dims)];
        let results = infer(&operands, &attributes(axis)).expect("a well-formed occurrence infers");
        let [result] = results.as_slice() else {
            panic!("the softmax has one result");
        };
        assert_eq!(result.resolved_type(), &F32::resolved_type());
        assert_eq!(
            result.shape(),
            operands[0].shape(),
            "the softmax is shape-preserving, never shape-reducing"
        );
    }
}

/// A zero-length reduced axis infers a zero-length result rather than refusing.
#[test]
fn a_zero_length_reduced_axis_infers_a_zero_length_result() {
    let operands = [f32_operand(&[1, 0])];
    let results = infer(&operands, &attributes(1)).expect("an empty reduced axis is admitted");
    let [result] = results.as_slice() else {
        panic!("the softmax has one result");
    };
    assert_eq!(result.shape(), operands[0].shape());
    assert_eq!(
        result.shape().as_static().and_then(Shape::element_count),
        Some(0)
    );
}

/// Each malformed axis refuses by the rule it violated, and the rules are distinct.
///
/// Every row is a deliberate perturbation of the well-formed occurrence above, so
/// each code below was observed to fire rather than assumed reachable.
#[test]
fn a_malformed_reduced_axis_refuses_by_the_rule_it_violated() {
    let operands = [f32_operand(&[8, 2, 10, 10])];
    let cases = [
        (axis_sequence(&[]), "softmax.f32.axis.absent"),
        (axis_sequence(&[3, 3]), "softmax.f32.axis.duplicated"),
        (axis_sequence(&[2, 3]), "softmax.f32.axis.rank"),
        (axis_sequence(&[4]), "softmax.f32.axis.range"),
    ];
    for (axes, expected) in cases {
        assert_eq!(refusal(&operands, &attributes_with_axes(axes)), expected);
    }
    let wrong_element =
        CanonicalValue::sequence([CanonicalValue::boolean(true)]).expect("a probe is canonical");
    assert_eq!(
        refusal(&operands, &attributes_with_axes(wrong_element)),
        "softmax.f32.axis.type"
    );
    // The control: the same operand with an admissible axis infers, so the
    // refusals above are about the attribute and not about the shape.
    assert!(infer(&operands, &attributes(3)).is_ok());
}

/// The arity and operand-type rules each refuse by name.
#[test]
fn the_structural_rules_refuse_by_name() {
    let shaped = f32_operand(&[1, 4]);
    let governed = attributes(1);
    assert_eq!(
        refusal(&[shaped.clone(), shaped.clone()], &governed),
        "tiler.schema.operand-arity"
    );
    assert_eq!(refusal(&[], &governed), "tiler.schema.operand-arity");
    let bf16 = operand(Bf16::resolved_type(), &[1, 4]);
    assert_eq!(
        refusal(&[bf16], &governed),
        "softmax.f32.implicit-promotion"
    );
    // `add_f32_op` and `constant_f32_op` remain reachable, so the refusals above
    // are about this key rather than a broken registry.
    assert!(registry().operation_definition(&add_f32_op()).is_some());
    assert!(
        registry()
            .operation_definition(&constant_f32_op())
            .is_some()
    );
}

// ---------------------------------------------------------------------------
// The symbolic-extent boundary
// ---------------------------------------------------------------------------

/// A symbolic reduced extent is refused by name, and every literal one infers.
///
/// **This replaces a test whose premise the tree has since falsified.** It used
/// to assert that no symbolic refusal *could* fire, on the ground that a
/// [`ValueFact`] carried a fixed `Shape` and so had nothing symbolic to refuse.
/// A fact now carries a [`SourcedShape`](crate::shape::SourcedShape), so the
/// case is reachable. This direct registry fixture supplies no shape
/// environment, so the host refuses the symbol as undeclared before asking the
/// family. Program construction with an environment separately proves that this
/// family decides shapes over literal extents only and reports
/// `SymbolicExtentUnsupported` there.
///
/// The refusal is a *typed extent* failure rather than a family shape
/// diagnostic, which is the discrimination that matters: the operand is not the
/// wrong size, it names a source this direct registry call has no authority to
/// interpret.
#[test]
fn a_symbolic_reduced_extent_is_refused_and_every_literal_one_infers() {
    // The reduced extent this workload grows is exercised at the static values a
    // program can actually state, and each is an ordinary literal extent.
    for extent in [1_u64, 10, 128, 8192] {
        let operands = [f32_operand(&[2, extent])];
        let results = infer(&operands, &attributes(1)).expect("a literal extent infers");
        assert_eq!(
            results[0]
                .shape()
                .as_static()
                .expect("a literal occurrence infers a literal boundary")
                .extents()[1]
                .get(),
            extent
        );
    }

    // The neighbour differing only in the reduced extent's source kind. Nothing
    // else about the occurrence moves, so the refusal is attributable to the
    // symbol and to nothing else.
    let symbol = ShapeSymbol::new(SymbolScope::new("tiler.test/0").unwrap(), "s").unwrap();
    let symbolic = symbolic_f32_operand(&[
        SourcedExtent::Static(Extent::new(2)),
        SourcedExtent::Symbol(symbol.clone()),
    ]);
    let error = infer(&[symbolic], &attributes(1)).expect_err("a symbolic extent is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a softmax refusal is a provider-attributed rejection");
    };
    assert_eq!(
        rejection.source_error().extent_source(),
        Some(&ExtentSourceError::UndeclaredSymbol { symbol }),
        "the refusal names the symbol this call supplied no environment for",
    );
}
