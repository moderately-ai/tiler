use super::*;
use crate::semantic::accuracy::AccuracyContractForm;
use crate::semantic::{
    Bf16, FrozenSemanticRegistry, OperationAttributes, RegistryError, ResolvedValueType,
    add_f32_op, constant_f32_op, silu_f32_op, strict_serial_sum_f32_op,
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

fn f32_operand(dims: &[u64]) -> ValueFact {
    operand(F32::resolved_type(), dims)
}

/// The governed attribute pair a well-formed occurrence carries.
fn attributes(axis: u32, eps_bits: u32) -> OperationAttributes {
    OperationAttributes::new([
        CanonicalField::new(
            RMS_NORM_REDUCED_AXES_ATTRIBUTE,
            rms_norm_f32_axis_attribute(Axis::new(axis)),
        ),
        CanonicalField::new(
            RMS_NORM_EPS_BITS_ATTRIBUTE,
            rms_norm_f32_eps_attribute(eps_bits),
        ),
    ])
    .expect("the governed attribute pair is canonical")
}

/// The attribute pair with an arbitrary axis sequence, so a malformed axis is statable.
fn attributes_with_axes(axes: CanonicalValue, eps_bits: u32) -> OperationAttributes {
    OperationAttributes::new([
        CanonicalField::new(RMS_NORM_REDUCED_AXES_ATTRIBUTE, axes),
        CanonicalField::new(
            RMS_NORM_EPS_BITS_ATTRIBUTE,
            rms_norm_f32_eps_attribute(eps_bits),
        ),
    ])
    .expect("a probe attribute pair is canonical")
}

fn axis_sequence(axes: &[u32]) -> CanonicalValue {
    CanonicalValue::sequence(axes.iter().map(|axis| CanonicalValue::unsigned_u32(*axis)))
        .expect("a probe axis sequence is canonical")
}

fn infer(
    operands: &[ValueFact],
    attributes: &OperationAttributes,
) -> Result<Vec<ValueFact>, RegistryError> {
    registry().infer_operation(&rms_norm_f32_op(), operands, attributes)
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operands: &[ValueFact], attributes: &OperationAttributes) -> String {
    let error = infer(operands, attributes).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("an RMS normalization refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

// ---------------------------------------------------------------------------
// Registration and identity
// ---------------------------------------------------------------------------

/// The key is registered beside the existing families rather than widening one.
#[test]
fn the_rms_normalization_key_is_registered_beside_the_existing_f32_families() {
    let registry = registry();
    assert!(registry.operation_definition(&rms_norm_f32_op()).is_some());
    assert_eq!(rms_norm_f32_op().to_string(), "tiler::rms-norm-f32@1");
    assert_ne!(rms_norm_f32_op(), silu_f32_op());
    assert_ne!(rms_norm_f32_op(), strict_serial_sum_f32_op());
    assert!(registry.operation_definition(&silu_f32_op()).is_some());
    assert!(
        registry
            .operation_definition(&strict_serial_sum_f32_op())
            .is_some()
    );
}

/// None of the four stated non-goals came with it.
///
/// The ticket's non-goals checked rather than asserted in prose: layer
/// normalization, a general `Rsqrt`, a general mean, and a bias-carrying
/// normalization. A later change that mints any of them has to move this test
/// and say why.
#[test]
fn no_layer_normalization_rsqrt_mean_or_bias_key_is_registered() {
    let registry = registry();
    for name in [
        "layer-norm-f32",
        "rsqrt-f32",
        "sqrt-f32",
        "mean-f32",
        "rms-norm-bias-f32",
    ] {
        let key = OpKey::new("tiler", name, 1).expect("a probe key is well formed");
        assert!(
            registry.operation_definition(&key).is_none(),
            "{key} is a non-goal of the RMS normalization admission and is not registered"
        );
    }
}

/// The registered reference pins the three decisions the usual spelling hides.
#[test]
fn the_registered_reference_pins_the_eps_position_the_rsqrt_and_the_weight_order() {
    let registry = registry();
    let definition = registry
        .operation_definition(&rms_norm_f32_op())
        .expect("the RMS normalization definition is registered");
    let reference = definition.normative_definition().as_str();
    assert!(
        reference.contains("eps inside the reciprocal square root's argument"),
        "the reference must place eps inside the root's argument: {reference}"
    );
    assert!(
        reference.contains("Rsqrt(t) and deliberately not 1 / Sqrt(t)"),
        "the reference must pin rsqrt against the two-rounding composition: {reference}"
    );
    assert!(
        reference.contains("weight applied after the identity conversion"),
        "the reference must order the weight after the conversion back: {reference}"
    );
    assert!(
        reference.contains("not layer normalization"),
        "the reference must exclude layer normalization by name: {reference}"
    );
    assert!(
        reference.contains("a division and never a multiplication by 1/N"),
        "the reference must pin the extent division: {reference}"
    );
}

/// Two occurrences differing only in `eps` do not share an attribute record.
///
/// This is the property the ticket's "different operations, no shared identity,
/// cache subject, or golden" sentence names, checked at the level identity is
/// actually computed at: the canonical attribute record.
#[test]
fn two_normalizations_differing_only_in_eps_carry_different_attributes() {
    let governed = attributes(1, RMS_NORM_F32_QWEN3_EPS_BITS);
    // The binary32 successor of the governed payload — the smallest possible
    // difference, so a comparison that only distinguished coarse differences
    // would pass here and must not.
    let neighbour = attributes(1, RMS_NORM_F32_QWEN3_EPS_BITS + 1);
    assert_ne!(governed.fields(), neighbour.fields());
    let operands = [f32_operand(&[2, 4]), f32_operand(&[2, 4])];
    // Both infer, so the difference is not a validity artefact: it is two legal
    // occurrences of one key that are nonetheless different operations.
    assert!(infer(&operands, &governed).is_ok());
    assert!(infer(&operands, &neighbour).is_ok());
}

/// The pinned workload's `eps` payload is the binary32 rounding of `1e-06`.
#[test]
fn the_governed_eps_payload_is_the_binary32_rounding_of_one_millionth() {
    assert_eq!(RMS_NORM_F32_QWEN3_EPS_BITS, 1e-6_f32.to_bits());
    // Not exactly representable, which is why the identity carries the payload
    // rather than the literal: the exact value is about 9.999999975e-07.
    assert!((f64::from(f32::from_bits(RMS_NORM_F32_QWEN3_EPS_BITS)) - 1e-6).abs() > 0.0);
}

/// The named squaring-overflow threshold is the largest binary32 with a finite square.
#[test]
fn the_squaring_overflow_threshold_is_the_last_argument_whose_square_is_finite() {
    let threshold = f32::from_bits(RMS_NORM_F32_SQUARING_OVERFLOW_BITS);
    assert!((threshold * threshold).is_finite());
    let successor = f32::from_bits(RMS_NORM_F32_SQUARING_OVERFLOW_BITS + 1);
    assert!(!(successor * successor).is_finite());
}

// ---------------------------------------------------------------------------
// The resolved accuracy contract
// ---------------------------------------------------------------------------

/// The reciprocal square root's contract is `Faithful` and not correctly rounded.
///
/// The form *is* the derivation's conclusion, so it is asserted directly: Table
/// 8.1 states `rsqrt` correctly rounded, §8.2 leaves the rounding mode open
/// between ties-to-even and toward-zero, and the union of the two admitted modes
/// is exactly the faithful result set. Writing `CorrectlyRounded` here would
/// claim the mode §8.2 declines to fix.
#[test]
fn the_reciprocal_square_root_contract_is_faithful_rather_than_correctly_rounded() {
    let contract = rms_norm_f32_rsqrt_accuracy_contract();
    assert!(matches!(contract.form(), AccuracyContractForm::Faithful));
    assert_eq!(contract.operation(), &rms_norm_f32_op());
    assert_eq!(contract.operand_types(), &[F32::resolved_type()]);
    assert_eq!(contract.result_type(), &F32::resolved_type());
}

/// The contract measures under no metric at all, which is the point of the form.
///
/// `tiler::silu-f32@1`'s exponential needed a registered cross-metric implication
/// because its bound is a ULP count under Apple's own definition. A faithful
/// contract names a result set instead, so this family needs none — and this
/// asserts the absence rather than leaving it to be inferred from the form.
#[test]
fn the_reciprocal_square_root_contract_states_no_ulp_metric() {
    let encoded = rms_norm_f32_rsqrt_accuracy_contract()
        .to_canonical_value()
        .expect("the governed contract is canonical");
    let rendered = format!("{encoded:?}");
    assert!(
        !rendered.contains("ulp"),
        "a faithful contract carries no ULP metric: {rendered}"
    );
}

/// The ordinary domain is open at zero and unbounded above.
#[test]
fn the_reciprocal_square_root_domain_excludes_zero_and_is_unbounded_above() {
    let domain = rms_norm_f32_rsqrt_ordinary_domain();
    assert!(matches!(domain.lower(), DomainBound::Open(value) if value.is_zero()));
    assert!(matches!(domain.upper(), DomainBound::Unbounded));
    // Open at zero rather than closed, so `1/sqrt(+0)` is carried by the
    // infinite-reference rule instead of by a finite-result obligation.
    assert!(!domain.contains(&ExactRational::zero()));
    assert!(domain.contains(&ExactRational::power_of_two(-149)));
    assert!(domain.contains(&ExactRational::power_of_two(127)));
}

/// Every reciprocal square root of a positive finite binary32 stays normal.
///
/// The derivation the contract's header states, checked at both ends of the
/// format rather than asserted: at the least positive subnormal the reference is
/// about `2^74.5` and at `f32::MAX` about `2^-63.5`, so no admitted argument
/// reaches the finite-overflow rule or produces a subnormal result. That is what
/// makes the exceptional contract's fourth rule vacuous rather than absent.
#[test]
fn the_reciprocal_square_root_of_every_positive_finite_argument_is_normal() {
    for bits in [1_u32, 0x0080_0000, 0x3f80_0000, 0x7f7f_ffff] {
        let argument = f32::from_bits(bits);
        assert!(argument > 0.0 && argument.is_finite());
        let reference = f64::from(argument).sqrt().recip();
        assert!(reference.is_finite(), "{argument:e}");
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the narrowed value is only classified, never used as a result"
        )]
        let narrowed = reference as f32;
        assert!(
            narrowed.is_normal(),
            "1/sqrt({argument:e}) narrowed to {narrowed:e}, which is not normal"
        );
    }
}

/// The exceptional contract states all four rules independently of the form.
#[test]
fn the_exceptional_contract_states_four_independent_rules() {
    let exceptional = rms_norm_f32_rsqrt_exceptional_contract();
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
        rms_norm_f32_rsqrt_accuracy_contract().exceptional(),
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
        .operation_definition(&rms_norm_f32_op())
        .expect("the RMS normalization definition is registered");
    assert_eq!(definition.canonical_facts().value(), &rms_norm_f32_facts());
}

/// The facts carry the complete resolved accuracy contract, not a summary of it.
#[test]
fn the_facts_carry_the_whole_resolved_accuracy_contract() {
    assert_eq!(
        fact_value(RMS_NORM_F32_FACT_RSQRT_ACCURACY_CONTRACT),
        rms_norm_f32_rsqrt_accuracy_contract()
            .to_canonical_value()
            .expect("the governed contract is canonical")
    );
}

/// The accumulator type is stated explicitly rather than left to the element type.
///
/// Criterion 3 of `implement-parallel-reduction-strategies` requires the width to
/// be an explicit part of the declaration; this asserts the declaration exists
/// and names `tiler::f32@1`, so a later change that widens it has to move this
/// test rather than silently inherit a new default.
#[test]
fn the_accumulator_type_is_declared_explicitly_as_binary32() {
    assert_eq!(
        fact_text(RMS_NORM_F32_FACT_ACCUMULATOR_TYPE),
        "tiler::f32@1"
    );
}

/// The extent division is declared as a division, never a reciprocal multiply.
#[test]
fn the_extent_division_is_declared_as_a_division() {
    assert_eq!(
        fact_text(RMS_NORM_F32_FACT_EXTENT_DIVISION),
        "divide-by-the-static-extent-never-multiply-by-its-reciprocal"
    );
    // The permission that would allow the substitution is withheld rather than
    // absent, and the two statements have to agree.
    assert!(!fact_boolean(
        RMS_NORM_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED
    ));
}

/// Decision D-3 is recorded as *defined*, and the record says what it produces.
#[test]
fn the_squaring_overflow_fact_defines_the_behaviour_rather_than_refusing_it() {
    let text = fact_text(RMS_NORM_F32_FACT_SQUARING_OVERFLOW);
    assert!(text.starts_with("defined-and-not-refused"), "{text}");
    assert!(text.contains("row-of-signed-zeros"), "{text}");
}

/// The approximate-intrinsic and contraction permissions are withheld.
#[test]
fn the_approximate_intrinsic_permission_is_withheld() {
    assert!(!fact_boolean(
        RMS_NORM_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED
    ));
    assert!(!fact_boolean(
        RMS_NORM_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED
    ));
}

fn fact_value(id: AttributeFieldId) -> CanonicalValue {
    let facts = rms_norm_f32_facts();
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

/// A well-formed occurrence is shape-preserving over both workload extents.
#[test]
fn the_normalization_preserves_its_operand_shape_at_both_extents() {
    for (dims, axis) in [
        (vec![3_u64, 1024], 1_u32),
        (vec![2, 16, 128], 2),
        (vec![1, 2], 1),
    ] {
        let operands = [f32_operand(&dims), f32_operand(&dims)];
        let results = infer(&operands, &attributes(axis, RMS_NORM_F32_QWEN3_EPS_BITS))
            .expect("a well-formed occurrence infers");
        let [result] = results.as_slice() else {
            panic!("the normalization has one result");
        };
        assert_eq!(result.resolved_type(), &F32::resolved_type());
        assert_eq!(result.shape(), operands[0].shape());
    }
}

/// Each malformed axis refuses by the rule it violated, and the rules are distinct.
///
/// Every row is a deliberate perturbation of the well-formed occurrence above,
/// so each code below was observed to fire rather than assumed reachable.
#[test]
fn a_malformed_reduced_axis_refuses_by_the_rule_it_violated() {
    let operands = [f32_operand(&[3, 1024]), f32_operand(&[3, 1024])];
    let eps = RMS_NORM_F32_QWEN3_EPS_BITS;
    let cases = [
        (axis_sequence(&[]), "rms-norm.f32.axis.absent"),
        (axis_sequence(&[1, 1]), "rms-norm.f32.axis.duplicated"),
        (axis_sequence(&[0, 1]), "rms-norm.f32.axis.rank"),
        (axis_sequence(&[2]), "rms-norm.f32.axis.range"),
    ];
    for (axes, expected) in cases {
        assert_eq!(
            refusal(&operands, &attributes_with_axes(axes, eps)),
            expected
        );
    }
    // A sequence element of the wrong canonical shape refuses by its own rule
    // rather than by one of the four above.
    let wrong_element =
        CanonicalValue::sequence([CanonicalValue::boolean(true)]).expect("a probe is canonical");
    assert_eq!(
        refusal(&operands, &attributes_with_axes(wrong_element, eps)),
        "rms-norm.f32.axis.type"
    );
}

/// Each inadmissible `eps` refuses at construction, by the rule it violated.
///
/// The zero row is the one that matters and it is refused rather than tolerated:
/// a zero `eps` is a different operation whose domain excludes the zero vector,
/// not a degenerate parameter of this one.
#[test]
fn an_inadmissible_eps_refuses_at_construction() {
    let operands = [f32_operand(&[3, 1024]), f32_operand(&[3, 1024])];
    let cases = [
        (0x0000_0000_u32, "rms-norm.f32.eps.zero"),
        (0x8000_0000, "rms-norm.f32.eps.zero"),
        (0xb586_37bd, "rms-norm.f32.eps.negative"),
        (0x7f80_0000, "rms-norm.f32.eps.non-finite"),
        (0xff80_0000, "rms-norm.f32.eps.non-finite"),
        (0x7fc0_0000, "rms-norm.f32.eps.nan"),
    ];
    for (bits, expected) in cases {
        assert_eq!(refusal(&operands, &attributes(1, bits)), expected);
    }
    // The governed payload is the control: the same operands with an admissible
    // eps infer, so the refusals above are about the payload and not the shape.
    assert!(infer(&operands, &attributes(1, RMS_NORM_F32_QWEN3_EPS_BITS)).is_ok());
}

/// An `eps` attribute naming another float format refuses by its own rule.
#[test]
fn an_eps_attribute_of_the_wrong_format_refuses() {
    let operands = [f32_operand(&[3, 1024]), f32_operand(&[3, 1024])];
    let wrong_format = OperationAttributes::new([
        CanonicalField::new(
            RMS_NORM_REDUCED_AXES_ATTRIBUTE,
            rms_norm_f32_axis_attribute(Axis::new(1)),
        ),
        CanonicalField::new(
            RMS_NORM_EPS_BITS_ATTRIBUTE,
            CanonicalValue::float_bits(
                crate::semantic::TypeKey::new("tiler", "bf16", 1).expect("a probe key is valid"),
                0x3586_u16.to_be_bytes(),
            )
            .expect("a probe payload is canonical"),
        ),
    ])
    .expect("a probe attribute pair is canonical");
    assert_eq!(refusal(&operands, &wrong_format), "rms-norm.f32.eps.format");
}

/// A weight the caller did not broadcast is refused, not implicitly widened.
#[test]
fn a_narrow_weight_is_refused_rather_than_implicitly_broadcast() {
    let value = f32_operand(&[3, 1024]);
    let narrow = f32_operand(&[1024]);
    assert_eq!(
        refusal(
            &[value.clone(), narrow],
            &attributes(1, RMS_NORM_F32_QWEN3_EPS_BITS)
        ),
        "rms-norm.f32.weight-shape"
    );
    // A rank-zero scalar weight is refused by the same rule: the scalar
    // admission elsewhere in the registry does not reach this operand.
    assert_eq!(
        refusal(
            &[value, f32_operand(&[])],
            &attributes(1, RMS_NORM_F32_QWEN3_EPS_BITS)
        ),
        "rms-norm.f32.weight-shape"
    );
}

/// The arity and operand-type rules each refuse by name.
///
/// The arity rows refuse under the *schema's* own code rather than the
/// inferencer's, because the registry validates the declared arity before it
/// dispatches. The inferencer's `rms-norm.f32.arity` arm stays as the fail-closed
/// answer on a direct call, which is the only path that reaches it.
#[test]
fn the_structural_rules_refuse_by_name() {
    let shaped = f32_operand(&[3, 1024]);
    let governed = attributes(1, RMS_NORM_F32_QWEN3_EPS_BITS);
    assert_eq!(
        refusal(std::slice::from_ref(&shaped), &governed),
        "tiler.schema.operand-arity"
    );
    assert_eq!(
        refusal(&[shaped.clone(), shaped.clone(), shaped.clone()], &governed),
        "tiler.schema.operand-arity"
    );
    let bf16 = operand(Bf16::resolved_type(), &[3, 1024]);
    assert_eq!(
        refusal(&[bf16.clone(), shaped.clone()], &governed),
        "rms-norm.f32.implicit-promotion"
    );
    assert_eq!(
        refusal(&[shaped.clone(), bf16], &governed),
        "rms-norm.f32.implicit-promotion"
    );
    // `add_f32_op` and `constant_f32_op` remain reachable, so the refusals above
    // are about this key rather than a broken registry.
    assert!(registry().operation_definition(&add_f32_op()).is_some());
    assert!(
        registry()
            .operation_definition(&constant_f32_op())
            .is_some()
    );
    let _ = shaped;
}
