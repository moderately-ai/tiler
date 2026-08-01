use super::*;
use crate::semantic::{
    Bf16Add, Bf16Constant, Bf16Multiply, F32, FrozenSemanticRegistry, InputKey,
    OperationAttributes, OutputKey, RegistryError, SemanticProgramBuilder, add_f32_op,
    multiply_f32_op,
};
use crate::shape::Shape;

fn registry() -> FrozenSemanticRegistry {
    FrozenSemanticRegistry::standard().expect("the standard registry builds")
}

fn bf16_operand(dims: &[u64]) -> ValueFact {
    operand(Bf16::resolved_type(), dims)
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    operand(F32::resolved_type(), dims)
}

fn operand(resolved_type: ResolvedValueType, dims: &[u64]) -> ValueFact {
    ValueFact::new(
        resolved_type,
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn constant_attributes(bits: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(BF16_CONSTANT_BITS_ATTRIBUTE, bits)])
        .expect("a test attribute record is canonical")
}

fn infer(
    key: &OpKey,
    operands: &[ValueFact],
    attributes: &OperationAttributes,
) -> Result<Vec<ValueFact>, RegistryError> {
    registry().infer_operation(key, operands, attributes)
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(key: &OpKey, operands: &[ValueFact], attributes: &OperationAttributes) -> String {
    let error = infer(key, operands, attributes).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a bf16 refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Reads one field of a registered operation's canonical fact record.
fn registered_fact(key: &OpKey, field: AttributeFieldId) -> Option<CanonicalValue> {
    let registry = registry();
    let facts = registry.operation_facts(key)?;
    let CanonicalValueView::Record(fields) = facts.value().view() else {
        panic!("a governed operation fact record is a record");
    };
    fields
        .iter()
        .find(|candidate| candidate.id() == field)
        .map(|candidate| candidate.value().clone())
}

fn fact_value(key: &OpKey, field: AttributeFieldId) -> CanonicalValue {
    registered_fact(key, field)
        .unwrap_or_else(|| panic!("{key} carries the unconditional fact field {field}"))
}

// ---------------------------------------------------------------------------
// Registration and identity
// ---------------------------------------------------------------------------

#[test]
fn the_three_bf16_keys_are_registered_and_distinct_from_their_f32_neighbours() {
    let registry = registry();
    for key in [constant_bf16_op(), multiply_bf16_op(), add_bf16_op()] {
        assert!(
            registry.operation_definition(&key).is_some(),
            "{key} is registered by standard semantics"
        );
    }
    assert_eq!(constant_bf16_op().to_string(), "tiler::constant-bf16@1");
    assert_eq!(multiply_bf16_op().to_string(), "tiler::multiply-bf16@1");
    assert_eq!(add_bf16_op().to_string(), "tiler::add-bf16@1");

    // ADR 0026: operand type is part of an operation's identity, so the bf16
    // keys sit beside the f32 keys rather than replacing them. Both remain
    // registered and no key is shared.
    assert_ne!(multiply_bf16_op(), multiply_f32_op());
    assert_ne!(add_bf16_op(), add_f32_op());
    assert!(registry.operation_definition(&multiply_f32_op()).is_some());
    assert!(registry.operation_definition(&add_f32_op()).is_some());
}

#[test]
fn the_bf16_marker_resolves_to_the_registered_catalog_identity() {
    let registry = registry();
    assert_eq!(
        registry
            .resolve_marker::<Bf16>()
            .expect("the Bf16 marker is bound by standard semantics"),
        &Bf16::resolved_type()
    );
    assert_eq!(
        Bf16::resolved_type().nominal_key().map(ToString::to_string),
        Some("tiler::bf16@1".to_owned())
    );
    // The identity is the catalog's, not this module's: nothing here re-registers it.
    registry
        .validate_type(&Bf16::resolved_type())
        .expect("tiler::bf16@1 is an admitted registered identity");
}

#[test]
fn the_normative_definitions_name_the_ratified_operand_format_and_its_source_id() {
    let registry = registry();
    for key in [constant_bf16_op(), multiply_bf16_op(), add_bf16_op()] {
        let reference = registry
            .operation_definition(&key)
            .expect("the key is registered")
            .normative_definition()
            .as_str()
            .to_owned();
        assert!(
            reference.contains("RISC-V BF16 operand format"),
            "{key} names the ratified operand format, found {reference}"
        );
        assert!(
            reference.contains("source riscv-unprivileged-isa-20260120"),
            "{key} preserves the catalog row's source id, found {reference}"
        );
        assert!(
            reference.contains("tiler::bf16@1"),
            "{key} names the identity it is defined over, found {reference}"
        );
        // The format table itself belongs to the catalog row; restating widths
        // here would give one format two authorities that could disagree.
        assert!(
            !reference.contains("trailing"),
            "{key} does not restate the format table, found {reference}"
        );
    }
}

// ---------------------------------------------------------------------------
// The four type facts, asserted individually
// ---------------------------------------------------------------------------

#[test]
fn every_bf16_definition_states_all_four_types_separately_and_each_is_bf16() {
    // Asserted one field at a time, deliberately. A grouped assertion would
    // pass if a future edit collapsed four fields into one, which is exactly
    // the change this record exists to make visible.
    for key in [constant_bf16_op(), multiply_bf16_op(), add_bf16_op()] {
        let computation = fact_value(&key, BF16_FACT_COMPUTATION_TYPE);
        let accumulator = fact_value(&key, BF16_FACT_ACCUMULATOR_TYPE);
        let intermediate = fact_value(&key, BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE);
        let result = fact_value(&key, BF16_FACT_RESULT_TYPE);

        assert_eq!(
            computation,
            bf16_value_type(),
            "{key} computes at tiler::bf16@1"
        );
        assert_eq!(
            accumulator,
            bf16_value_type(),
            "{key} accumulates at tiler::bf16@1"
        );
        assert_eq!(
            intermediate,
            bf16_value_type(),
            "{key} materializes intermediates at tiler::bf16@1"
        );
        assert_eq!(result, bf16_value_type(), "{key} results in tiler::bf16@1");

        // Four separate fields, not one value read four times.
        for (left, right) in [
            (BF16_FACT_COMPUTATION_TYPE, BF16_FACT_ACCUMULATOR_TYPE),
            (
                BF16_FACT_ACCUMULATOR_TYPE,
                BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE,
            ),
            (
                BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE,
                BF16_FACT_RESULT_TYPE,
            ),
        ] {
            assert_ne!(left, right, "{key}'s type facts have distinct field IDs");
        }
    }
}

#[test]
fn the_arithmetic_facts_state_the_rounding_rule_and_the_canonical_nan_payload() {
    for key in [multiply_bf16_op(), add_bf16_op()] {
        assert_eq!(
            fact_value(&key, BF16_FACT_ROUNDING),
            CanonicalValue::utf8(
                "bf16-round-to-nearest-ties-to-even-at-every-observable-materialization"
            )
            .expect("the fact is bounded"),
            "{key} rounds to nearest, ties to even, at every observable materialization"
        );
        assert_eq!(
            fact_value(&key, BF16_FACT_CANONICAL_NAN_BITS),
            canonical_bf16_bits(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
            "{key} installs the canonical arithmetic NaN payload"
        );
    }

    // The canonical NaN is tagged with the bf16 format, so it can never be
    // confused with the binary32 canonical NaN even though the two agree on
    // their leading sixteen bits.
    let canonical_nan = canonical_bf16_bits(CANONICAL_BF16_ARITHMETIC_NAN_BITS);
    let CanonicalValueView::FloatBits(bits) = canonical_nan.view() else {
        panic!("a canonical bf16 payload is FloatBits");
    };
    assert_eq!(bits.format(), &bf16_type_key());
    assert_eq!(bits.bits(), &0x7fc0_u16.to_be_bytes());
    assert_eq!(
        CANONICAL_BF16_ARITHMETIC_NAN_BITS,
        u16::try_from(super::super::CANONICAL_F32_ARITHMETIC_NAN_BITS >> 16)
            .expect("the leading sixteen bits fit a u16"),
        "bf16 shares binary32's sign and exponent fields, so the two canonical \
         NaN payloads agree on those bits — which is why each is written down \
         separately rather than one deriving the other"
    );
}

#[test]
fn the_exceptional_value_contract_is_stated_on_every_definition() {
    for key in [constant_bf16_op(), multiply_bf16_op(), add_bf16_op()] {
        for field in [
            BF16_FACT_ROUNDING,
            BF16_FACT_SUBNORMALS,
            BF16_FACT_SIGNED_ZERO,
            BF16_FACT_NAN_BEHAVIOUR,
            BF16_FACT_INFINITY_AND_OVERFLOW,
        ] {
            let value = fact_value(&key, field);
            let CanonicalValueView::Utf8(text) = value.view() else {
                panic!("{key} field {field} is a named behaviour");
            };
            assert!(
                !text.is_empty(),
                "{key} states field {field} rather than leaving it inferable"
            );
        }
    }

    // The constant preserves what it was given; the arithmetic rounds and
    // canonicalizes. Those are different contracts and the records say so.
    assert_ne!(
        fact_value(&constant_bf16_op(), BF16_FACT_NAN_BEHAVIOUR),
        fact_value(&multiply_bf16_op(), BF16_FACT_NAN_BEHAVIOUR),
    );
    assert_ne!(
        fact_value(&constant_bf16_op(), BF16_FACT_ROUNDING),
        fact_value(&multiply_bf16_op(), BF16_FACT_ROUNDING),
    );

    // Subnormals participate in the *semantics*. Whether a target flushes them
    // is a numerical-honourability fact of that target's profile row, and it is
    // deliberately absent from a consumer-neutral operation definition.
    let subnormal_fact = fact_value(&multiply_bf16_op(), BF16_FACT_SUBNORMALS);
    let CanonicalValueView::Utf8(subnormals) = subnormal_fact.view() else {
        panic!("the subnormal fact is a named behaviour");
    };
    assert!(subnormals.contains("not-flushed"));
}

#[test]
fn the_five_fences_are_stated_false_on_the_arithmetic_and_absent_on_the_constant() {
    const FENCES: [AttributeFieldId; 5] = [
        BF16_FACT_MIXED_PRECISION_PERMITTED,
        BF16_FACT_IMPLICIT_PROMOTION_PERMITTED,
        BF16_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
        BF16_FACT_FUSED_MULTIPLY_ADD_PERMITTED,
        BF16_FACT_REASSOCIATION_PERMITTED,
    ];
    for key in [multiply_bf16_op(), add_bf16_op()] {
        for field in FENCES {
            assert_eq!(
                fact_value(&key, field),
                CanonicalValue::boolean(false),
                "{key} withholds permission {field}"
            );
        }
    }

    // Conditional, not defaulted: a zero-operand constant has no operand pair to
    // promote, no adjacent rounding to contract, and no contributors to regroup,
    // so `false` there would claim a permission exists and is withheld.
    for field in FENCES {
        assert!(
            registered_fact(&constant_bf16_op(), field).is_none(),
            "the bf16 constant carries no {field}"
        );
    }
    assert!(
        registered_fact(&constant_bf16_op(), BF16_FACT_CANONICAL_NAN_BITS).is_none(),
        "the bf16 constant installs no arithmetic NaN"
    );
    assert_eq!(
        fact_value(&constant_bf16_op(), BF16_CONSTANT_FACT_PAYLOAD_RULE),
        CanonicalValue::utf8("exact-bf16-bits").expect("the fact is bounded"),
    );
    // And the constant's payload rule is conditional the other way.
    for key in [multiply_bf16_op(), add_bf16_op()] {
        assert!(registered_fact(&key, BF16_CONSTANT_FACT_PAYLOAD_RULE).is_none());
    }
}

#[test]
fn no_bf16_arithmetic_declares_an_algebraic_law_while_its_f32_neighbour_does() {
    let registry = registry();
    for key in [multiply_bf16_op(), add_bf16_op()] {
        assert!(
            !registry
                .operation_definition(&key)
                .expect("the key is registered")
                .algebraic_capabilities()
                .declares_ordered_associativity(),
            "{key} withholds ordered associativity; `BF16_FACT_REASSOCIATION_PERMITTED` \
             is false and a missing declaration reads as unknown, never as the inverse law"
        );
    }
    // The contrast is the point: this is a deliberate divergence from the f32
    // idiom, not an omission.
    assert!(
        registry
            .operation_definition(&add_f32_op())
            .expect("tiler::add-f32@1 is registered")
            .algebraic_capabilities()
            .declares_ordered_associativity()
    );
}

#[test]
fn the_declared_fact_constructors_return_exactly_what_registration_installed() {
    let registry = registry();
    assert_eq!(
        registry
            .operation_facts(&constant_bf16_op())
            .expect("the key is registered")
            .value(),
        &constant_bf16_facts()
    );
    for key in [multiply_bf16_op(), add_bf16_op()] {
        assert_eq!(
            registry
                .operation_facts(&key)
                .expect("the key is registered")
                .value(),
            &arithmetic_bf16_facts(),
            "{key} carries the one arithmetic record"
        );
    }
}

// ---------------------------------------------------------------------------
// Accepted applications
// ---------------------------------------------------------------------------

#[test]
fn bf16_arithmetic_over_two_bf16_operands_infers_a_bf16_result() {
    for key in [multiply_bf16_op(), add_bf16_op()] {
        let results = infer(
            &key,
            &[bf16_operand(&[4, 8]), bf16_operand(&[4, 8])],
            &OperationAttributes::empty(),
        )
        .expect("a pure bf16 application is admitted");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resolved_type(), &Bf16::resolved_type());
        assert_eq!(results[0].shape(), &Shape::try_from_dims([4, 8]).unwrap());
    }
}

#[test]
fn a_scalar_bf16_operand_broadcasts_against_a_shaped_one() {
    let results = infer(
        &multiply_bf16_op(),
        &[bf16_operand(&[]), bf16_operand(&[4, 8])],
        &OperationAttributes::empty(),
    )
    .expect("a scalar operand broadcasts");
    assert_eq!(results[0].shape(), &Shape::try_from_dims([4, 8]).unwrap());

    let results = infer(
        &multiply_bf16_op(),
        &[bf16_operand(&[4, 8]), bf16_operand(&[])],
        &OperationAttributes::empty(),
    )
    .expect("a scalar operand broadcasts");
    assert_eq!(results[0].shape(), &Shape::try_from_dims([4, 8]).unwrap());
}

#[test]
fn the_bf16_constant_admits_an_exact_bf16_payload() {
    let results = infer(
        &constant_bf16_op(),
        &[],
        &constant_attributes(canonical_bf16_bits(0x3f80)),
    )
    .expect("an exact bf16 payload is admitted");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].resolved_type(), &Bf16::resolved_type());
    assert_eq!(results[0].shape().rank(), 0);

    // Every one of the format's five special-value classes is expressible as a
    // constant, including both zeros, a subnormal, and a NaN payload.
    for bits in [
        0x0000_u16, // +0
        0x8000,     // -0
        0x0001,     // smallest positive subnormal
        0x7f80,     // +inf
        0xff80,     // -inf
        0x7fc0,     // canonical quiet NaN
        0x7fc1,     // a NaN with a payload, preserved rather than canonicalized
    ] {
        infer(
            &constant_bf16_op(),
            &[],
            &constant_attributes(canonical_bf16_bits(bits)),
        )
        .unwrap_or_else(|error| panic!("bf16 payload {bits:#06x} is a constant, got {error}"));
    }
}

// ---------------------------------------------------------------------------
// Refusals, each watched failing
// ---------------------------------------------------------------------------

#[test]
fn a_bf16_arithmetic_refuses_an_f32_operand_pair_as_an_implicit_promotion() {
    for key in [multiply_bf16_op(), add_bf16_op()] {
        assert_eq!(
            refusal(
                &key,
                &[f32_operand(&[4]), f32_operand(&[4])],
                &OperationAttributes::empty()
            ),
            "bf16.binary.implicit-promotion",
            "{key} does not promote an f32 pair into bf16"
        );
    }
}

#[test]
fn a_bf16_arithmetic_refuses_a_mixed_operand_pair_by_a_different_name() {
    for key in [multiply_bf16_op(), add_bf16_op()] {
        assert_eq!(
            refusal(
                &key,
                &[bf16_operand(&[4]), f32_operand(&[4])],
                &OperationAttributes::empty()
            ),
            "bf16.binary.mixed-precision",
            "{key} refuses a bf16/f32 pair"
        );
        assert_eq!(
            refusal(
                &key,
                &[f32_operand(&[4]), bf16_operand(&[4])],
                &OperationAttributes::empty()
            ),
            "bf16.binary.mixed-precision",
            "{key} refuses the mirrored pair identically"
        );
    }
    // The two refusals are genuinely different names, so a caller can tell a
    // mixed application from a wholly-promoted one.
    assert_ne!(
        refusal(
            &multiply_bf16_op(),
            &[bf16_operand(&[4]), f32_operand(&[4])],
            &OperationAttributes::empty()
        ),
        refusal(
            &multiply_bf16_op(),
            &[f32_operand(&[4]), f32_operand(&[4])],
            &OperationAttributes::empty()
        ),
    );
}

#[test]
fn registering_bf16_did_not_weaken_the_existing_f32_signatures() {
    for key in [multiply_f32_op(), add_f32_op()] {
        assert_eq!(
            refusal(
                &key,
                &[bf16_operand(&[4]), bf16_operand(&[4])],
                &OperationAttributes::empty()
            ),
            "binary.type",
            "{key} still refuses a bf16 operand pair"
        );
        assert_eq!(
            refusal(
                &key,
                &[f32_operand(&[4]), bf16_operand(&[4])],
                &OperationAttributes::empty()
            ),
            "binary.type",
            "{key} still refuses a mixed pair"
        );
        // And the f32 signature still accepts what it always accepted, so the
        // check above is about bf16 rather than about a dead operation.
        infer(
            &key,
            &[f32_operand(&[4]), f32_operand(&[4])],
            &OperationAttributes::empty(),
        )
        .expect("the f32 signature still admits an f32 pair");
    }
}

#[test]
fn the_bf16_constant_refuses_a_binary32_payload_and_a_wrong_width_payload_separately() {
    // A binary32 payload: wrong format *and* wrong width. It fails on the
    // format, which is the check that names what went wrong.
    assert_eq!(
        refusal(
            &constant_bf16_op(),
            &[],
            &constant_attributes(super::super::canonical_f32_bits(0x3f80_0000)),
        ),
        "bf16.constant.bits.format",
    );

    // A payload wearing the bf16 format at the binary32 *width*. Without this
    // case the width check could never be observed failing, because the format
    // check would always fire first.
    let wide = CanonicalValue::float_bits(bf16_type_key(), 0x3f80_0000_u32.to_be_bytes())
        .expect("a four-byte payload is bounded");
    assert_eq!(
        refusal(&constant_bf16_op(), &[], &constant_attributes(wide)),
        "bf16.constant.bits.width",
    );

    // A one-byte payload fails the same way, so the check is a width equality
    // rather than an upper bound.
    let narrow =
        CanonicalValue::float_bits(bf16_type_key(), [0x3f]).expect("a one-byte payload is bounded");
    assert_eq!(
        refusal(&constant_bf16_op(), &[], &constant_attributes(narrow)),
        "bf16.constant.bits.width",
    );
}

#[test]
fn the_constant_payload_width_is_read_from_the_registered_descriptor() {
    // The spike's one seam probe whose negative answer is a defect: the width
    // the constant validates against comes from the catalog row, not a literal.
    assert_eq!(registered_bf16_payload_bytes(), 2);

    let facts = builtin_scalar_value_type_facts(&Bf16::resolved_type())
        .expect("the governed catalog describes tiler::bf16@1");
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("a governed scalar descriptor is a record");
    };
    let width = fields
        .iter()
        .find(|field| field.id() == SCALAR_TYPE_FACT_WIDTH_BITS)
        .expect("the descriptor states a width");
    assert_eq!(width.value(), &CanonicalValue::unsigned_u32(16));
}

#[test]
fn every_structural_refusal_on_the_bf16_family_can_say_no() {
    // Arity, on both the constant and the arithmetic.
    assert_eq!(
        refusal(
            &constant_bf16_op(),
            &[bf16_operand(&[])],
            &constant_attributes(canonical_bf16_bits(0x3f80))
        ),
        "tiler.schema.operand-arity",
        "the schema rejects a constant operand before inference sees it"
    );
    assert_eq!(
        refusal(
            &multiply_bf16_op(),
            &[bf16_operand(&[4])],
            &OperationAttributes::empty()
        ),
        "tiler.schema.operand-arity",
    );

    // A missing required attribute, and an attribute of the wrong category.
    assert_eq!(
        refusal(&constant_bf16_op(), &[], &OperationAttributes::empty()),
        "tiler.schema.missing-attribute",
    );
    assert_eq!(
        refusal(
            &constant_bf16_op(),
            &[],
            &constant_attributes(CanonicalValue::unsigned_u32(1))
        ),
        "tiler.schema.attribute-kind",
    );

    // An attribute on an operation that declares none.
    assert_eq!(
        refusal(
            &multiply_bf16_op(),
            &[bf16_operand(&[4]), bf16_operand(&[4])],
            &constant_attributes(canonical_bf16_bits(0x3f80))
        ),
        "tiler.schema.unknown-attribute",
    );

    // Incompatible shapes on an otherwise pure-bf16 pair.
    assert_eq!(
        refusal(
            &add_bf16_op(),
            &[bf16_operand(&[4, 8]), bf16_operand(&[8, 4])],
            &OperationAttributes::empty()
        ),
        "bf16.binary.shape",
    );
}

// ---------------------------------------------------------------------------
// End to end through the typed facades
// ---------------------------------------------------------------------------

#[test]
fn a_pure_bf16_program_verifies_through_the_typed_facades() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("a builder is available");
    let left = builder
        .input::<Bf16>(
            InputKey::new("left").expect("a key is valid"),
            Shape::try_from_dims([4]).expect("a shape is bounded"),
        )
        .expect("a bf16 input is admitted");
    let right = builder
        .input::<Bf16>(
            InputKey::new("right").expect("a key is valid"),
            Shape::try_from_dims([4]).expect("a shape is bounded"),
        )
        .expect("a bf16 input is admitted");
    // 1.0 in bf16.
    let scale = Bf16Constant::apply(&mut builder, 0x3f80).expect("a bf16 constant is admitted");
    let scaled = Bf16Multiply::apply(&mut builder, left, scale).expect("a bf16 multiply verifies");
    let summed = Bf16Add::apply(&mut builder, scaled, right).expect("a bf16 add verifies");
    builder
        .output(OutputKey::new("out").expect("a key is valid"), summed)
        .expect("a bf16 output is admitted");
    let program = builder.build().expect("a pure bf16 program verifies");
    assert_eq!(program.operations().count(), 3);
}

#[test]
fn the_typed_bf16_facades_cannot_be_handed_an_f32_value() {
    // The refusal is at the registered signature, reached here through the
    // erased path a parsed frontend would use: the typed facades make the same
    // application a compile error, which the `typed_handles` UI suite covers.
    let mut builder = SemanticProgramBuilder::try_standard().expect("a builder is available");
    let f32_input = builder
        .input::<F32>(
            InputKey::new("f32").expect("a key is valid"),
            Shape::try_from_dims([4]).expect("a shape is bounded"),
        )
        .expect("an f32 input is admitted");
    let error = builder
        .apply(
            multiply_bf16_op(),
            OperationAttributes::empty(),
            &[f32_input.erase(), f32_input.erase()],
        )
        .expect_err("a bf16 multiply refuses an f32 pair");
    assert!(
        format!("{error}").contains("bf16.binary.implicit-promotion"),
        "the graph-level refusal names the promotion it declined, found {error}"
    );
}
