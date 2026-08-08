use super::*;
use crate::semantic::{
    Bf16, F32Concatenate, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
    RegistryError, SemanticProgramBuilder,
};

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn shape(dims: &[u64]) -> Shape {
    Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded")
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(F32::resolved_type(), shape(dims))
}

fn attributes(axis: Axis) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(
        CONCATENATE_AXIS_ATTRIBUTE,
        concatenate_f32_axis_attribute(axis),
    )])
    .expect("a test attribute record is canonical")
}

fn infer(operands: &[ValueFact], axis: Axis) -> Result<Vec<ValueFact>, RegistryError> {
    FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(&concatenate_f32_op(), operands, &attributes(axis))
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operands: &[ValueFact], axis: Axis) -> String {
    let error = infer(operands, axis).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a concatenate refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Returns the complete diagnostic message of a refused application.
fn refusal_message(operands: &[ValueFact], axis: Axis) -> String {
    let error = infer(operands, axis).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a concatenate refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().message().to_owned()
}

fn result_shape(operands: &[&[u64]], axis: Axis) -> Shape {
    let facts: Vec<ValueFact> = operands.iter().map(|dims| f32_operand(dims)).collect();
    let results = infer(&facts, axis).expect("the occurrence is admitted");
    let [result] = results.as_slice() else {
        panic!("a concatenation has one result");
    };
    assert_eq!(result.resolved_type(), &F32::resolved_type());
    result
        .shape()
        .as_static()
        .expect("this family infers a literal boundary")
        .clone()
}

// --- The result extent, at the pinned workload's own shapes ------------------

/// The KV append of one decode step, at the C1 profile's own extents.
///
/// L5's worked example runs `C` from 0 through 17 across nine executions of one
/// layer, joining a cached `[8, C, 128]` with the step's new `[8, T, 128]` on
/// axis 1. Every row of that table is checked here, which is what makes the
/// zero-extent prefill row evidence rather than an assertion about a case nobody
/// reaches.
#[test]
fn the_result_extent_is_the_sum_across_the_c1_decode_table() {
    // Prefill: an empty cache and ten prompt positions.
    assert_eq!(
        result_shape(&[&[8, 0, 128], &[8, 10, 128]], axis(1)),
        shape(&[8, 10, 128])
    );
    // The eight decode steps, `C = 10 ..= 17` at `T = 1`.
    for cached in 10..=17_u64 {
        assert_eq!(
            result_shape(&[&[8, cached, 128], &[8, 1, 128]], axis(1)),
            shape(&[8, cached + 1, 128]),
            "decode at C = {cached} extends the context by exactly T"
        );
    }
}

#[test]
fn the_concatenated_axis_may_be_any_axis_and_only_that_axis_grows() {
    assert_eq!(
        result_shape(&[&[2, 3, 4], &[5, 3, 4]], axis(0)),
        shape(&[7, 3, 4])
    );
    assert_eq!(
        result_shape(&[&[2, 3, 4], &[2, 6, 4]], axis(1)),
        shape(&[2, 9, 4])
    );
    assert_eq!(
        result_shape(&[&[2, 3, 4], &[2, 3, 1]], axis(2)),
        shape(&[2, 3, 5])
    );
}

#[test]
fn the_variadic_arity_joins_every_admitted_operand_count() {
    for arity in MIN_CONCATENATE_OPERANDS..=MAX_CONCATENATE_OPERANDS {
        let operands: Vec<ValueFact> = (0..arity).map(|_| f32_operand(&[3, 2])).collect();
        let results = infer(&operands, axis(0)).expect("an admitted arity is joined");
        let [result] = results.as_slice() else {
            panic!("a concatenation has one result");
        };
        assert_eq!(
            result.shape().as_static(),
            Some(&shape(&[3 * u64::from(arity), 2]))
        );
    }
}

// --- The zero-extent rule, stated rather than inherited ----------------------

/// Prefill's `C = 0` is admitted, and an empty operand is not otherwise special.
///
/// The pair matters together: the first half is the case L5 reaches on every
/// prefill, and the second half is what stops "contributes no coordinate" from
/// being read as "is skipped". An operand that contributes nothing to the
/// concatenated axis still has to agree on every other one.
#[test]
fn a_zero_extent_operand_is_admitted_and_still_agrees_on_every_other_axis() {
    assert_eq!(
        result_shape(&[&[8, 0, 128], &[8, 10, 128]], axis(1)),
        shape(&[8, 10, 128])
    );
    // Both operands empty on the concatenated axis: admitted, and the result is
    // empty on it too rather than being refused as a degenerate occurrence.
    assert_eq!(
        result_shape(&[&[8, 0, 128], &[8, 0, 128]], axis(1)),
        shape(&[8, 0, 128])
    );
    // The same empty operand, disagreeing on a *different* axis, is refused.
    assert_eq!(
        refusal(
            &[f32_operand(&[8, 0, 128]), f32_operand(&[4, 10, 128])],
            axis(1)
        ),
        "concatenate.operands.extent-disagreement",
        "an empty operand is not skipped: it agrees on every axis except the \
         concatenated one, exactly as any other operand does"
    );
}

#[test]
fn an_operand_empty_on_another_axis_makes_the_result_empty_rather_than_refused() {
    let joined = result_shape(&[&[0, 3, 4], &[0, 5, 4]], axis(1));
    assert_eq!(joined, shape(&[0, 8, 4]));
    assert_eq!(joined.element_count(), Some(0));
}

// --- The extent-domain refusal ----------------------------------------------

/// The exact sum leaving the extent domain is a refusal, not a saturation.
///
/// `ExtentRelation::AdditiveEquality` can state `S == C + T`, but it does not
/// widen the `u64` extent domain. When the sum is not representable there is no
/// extent this family could bind that the operands determine.
///
/// The admitted neighbour is the load-bearing half of this test. `u64::MAX - 1`
/// plus one is representable and is admitted with the exact sum; adding one more
/// is refused. A saturating implementation would pass the first assertion and
/// would answer the second with `u64::MAX` — a plausible extent unrelated to its
/// operands — so the refusal is what separates the two.
#[test]
fn the_extent_refusal_fires_when_the_exact_sum_leaves_the_extent_domain() {
    // Admitted neighbour: the exact sum is the largest representable extent.
    assert_eq!(
        result_shape(&[&[u64::MAX - 1], &[1]], axis(0)),
        shape(&[u64::MAX])
    );
    // One coordinate further, and the sum is not representable.
    assert_eq!(
        refusal(&[f32_operand(&[u64::MAX]), f32_operand(&[1])], axis(0)),
        "concatenate.axis.result-extent-unrelatable"
    );
    let message = refusal_message(&[f32_operand(&[u64::MAX]), f32_operand(&[1])], axis(0));
    assert!(
        message.contains("does not widen that domain"),
        "the refusal distinguishes stateability from representability: {message}"
    );
    assert!(
        message.contains(&u64::MAX.to_string()),
        "and names the accumulated extent it could not extend: {message}"
    );
    // The overflow is detected across the whole operand sequence rather than
    // pairwise from the front, so a third operand that tips the sum is caught.
    assert_eq!(
        refusal(
            &[
                f32_operand(&[u64::MAX / 2]),
                f32_operand(&[u64::MAX / 2]),
                f32_operand(&[2])
            ],
            axis(0)
        ),
        "concatenate.axis.result-extent-unrelatable"
    );
    // And its representable neighbour, one coordinate short, is admitted.
    assert_eq!(
        result_shape(&[&[u64::MAX / 2], &[u64::MAX / 2], &[1]], axis(0)),
        shape(&[u64::MAX])
    );
}

// --- Construction-time admission --------------------------------------------

#[test]
fn an_extent_disagreement_names_the_axis_and_both_observed_extents() {
    let operands = [f32_operand(&[8, 3, 128]), f32_operand(&[4, 5, 128])];
    assert_eq!(
        refusal(&operands, axis(1)),
        "concatenate.operands.extent-disagreement"
    );
    let message = refusal_message(&operands, axis(1));
    assert!(
        message.contains("extent 4") && message.contains("extent 8"),
        "a disproof names both observed extents rather than one of them: {message}"
    );
    assert!(
        message.contains("axis 0"),
        "and the axis they disagree on: {message}"
    );
    // The concatenated axis is exempt, which is the point of the family.
    assert_eq!(
        result_shape(&[&[8, 3, 128], &[8, 5, 128]], axis(1)),
        shape(&[8, 8, 128])
    );
}

#[test]
fn no_extent_one_axis_is_stretched_to_match() {
    // A broadcast would widen `[1, 7]` against `[3, 4]`. This family refuses, and
    // a caller that means the widening writes a `tiler::broadcast-f32@1`
    // occurrence for it.
    assert_eq!(
        refusal(&[f32_operand(&[3, 4]), f32_operand(&[1, 7])], axis(1)),
        "concatenate.operands.extent-disagreement"
    );
}

#[test]
fn a_rank_disagreement_refuses_rather_than_padding() {
    let operands = [f32_operand(&[8, 3, 128]), f32_operand(&[3, 128])];
    assert_eq!(
        refusal(&operands, axis(1)),
        "concatenate.operands.rank-disagreement"
    );
    let message = refusal_message(&operands, axis(1));
    assert!(
        message.contains("rank 2") && message.contains("rank 3"),
        "the refusal names both observed ranks: {message}"
    );
}

#[test]
fn an_axis_beyond_the_shared_rank_refuses() {
    assert_eq!(
        refusal(&[f32_operand(&[8, 3]), f32_operand(&[8, 5])], axis(2)),
        "concatenate.axis.out-of-range"
    );
    // A rank-zero operand has no axis at all, so every axis is out of range.
    assert_eq!(
        refusal(&[f32_operand(&[]), f32_operand(&[])], axis(0)),
        "concatenate.axis.out-of-range"
    );
}

#[test]
fn the_family_grants_no_promotion_and_no_weak_scalar_rule() {
    let mixed = [
        f32_operand(&[3, 4]),
        ValueFact::new(Bf16::resolved_type(), shape(&[3, 4])),
    ];
    assert_eq!(
        refusal(&mixed, axis(0)),
        "concatenate.f32.implicit-promotion"
    );
    // A rank-zero F32 operand is not a weak scalar that adopts the other's shape;
    // it is a rank disagreement.
    assert_eq!(
        refusal(&[f32_operand(&[3, 4]), f32_operand(&[])], axis(0)),
        "concatenate.operands.rank-disagreement"
    );
}

#[test]
fn the_arity_range_is_the_schemas_and_the_family_re_decides_it() {
    // Through the registry the schema refuses first, under its own name, exactly
    // as it does for a missing attribute.
    let one = [f32_operand(&[3, 4])];
    assert_eq!(refusal(&one, axis(0)), "tiler.schema.operand-arity");
    let nine: Vec<ValueFact> = (0..9).map(|_| f32_operand(&[3, 4])).collect();
    assert_eq!(refusal(&nine, axis(0)), "tiler.schema.operand-arity");
    // The family's own rule is re-decided rather than trusted, because
    // `concatenate_result_shape` is public and the reference evaluator calls it
    // with whatever operands arrive.
    let single = shape(&[3, 4]);
    assert_eq!(
        concatenate_result_shape(axis(0), &[&single]),
        Err(ConcatenateError::OperandCount {
            operands: 1,
            minimum: MIN_CONCATENATE_OPERANDS,
            maximum: MAX_CONCATENATE_OPERANDS,
        })
    );
    assert!(concatenate_result_shape(axis(0), &[]).is_err());
}

#[test]
fn a_malformed_axis_attribute_refuses_under_its_own_rule() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let malformed = OperationAttributes::new([CanonicalField::new(
        CONCATENATE_AXIS_ATTRIBUTE,
        CanonicalValue::unsigned_u64(1),
    )])
    .expect("a test attribute record is canonical");
    let error = registry
        .infer_operation(
            &concatenate_f32_op(),
            &[f32_operand(&[3, 4]), f32_operand(&[3, 4])],
            &malformed,
        )
        .expect_err("a 64-bit axis is not the declared attribute");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a concatenate refusal is a provider-attributed rejection, not {error}");
    };
    assert_eq!(
        rejection.source_error().code().as_str(),
        "concatenate.axis.malformed-attribute"
    );
    // A missing attribute never reaches this family's inference: the schema
    // declares the axis required and refuses first.
    let error = registry
        .infer_operation(
            &concatenate_f32_op(),
            &[f32_operand(&[3, 4]), f32_operand(&[3, 4])],
            &OperationAttributes::empty(),
        )
        .expect_err("a missing axis attribute is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a concatenate refusal is a provider-attributed rejection, not {error}");
    };
    assert_eq!(
        rejection.source_error().code().as_str(),
        "tiler.schema.missing-attribute"
    );
}

// --- The authoring facade ---------------------------------------------------

#[test]
fn the_authoring_facade_admits_a_concatenation_through_the_governed_path() {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the standard builder is available");
    let cache = builder
        .input::<F32>(
            InputKey::new("k_cache").expect("a test key is bounded"),
            shape(&[8, 17, 128]),
        )
        .expect("an input is declared");
    let new_rows = builder
        .input::<F32>(
            InputKey::new("k_new").expect("a test key is bounded"),
            shape(&[8, 1, 128]),
        )
        .expect("an input is declared");
    let joined = F32Concatenate::apply(&mut builder, &[cache, new_rows], axis(1))
        .expect("the join is admitted");
    builder
        .output(
            OutputKey::new("k_rope").expect("a test key is bounded"),
            joined,
        )
        .expect("an output is declared");
    let program = builder.build().expect("the program verifies");
    assert_eq!(program.operation_count(), 1);

    // Operand order is semantic, so the reversed occurrence is a different
    // computation rather than the same one. Both are admitted and both have the
    // same result shape — which is exactly why the reference evaluator, and not a
    // shape check, is what distinguishes them.
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the standard builder is available");
    let cache = builder
        .input::<F32>(
            InputKey::new("k_cache").expect("a test key is bounded"),
            shape(&[8, 17, 128]),
        )
        .expect("an input is declared");
    let new_rows = builder
        .input::<F32>(
            InputKey::new("k_new").expect("a test key is bounded"),
            shape(&[8, 1, 128]),
        )
        .expect("an input is declared");
    let reversed = F32Concatenate::apply(&mut builder, &[new_rows, cache], axis(1))
        .expect("the reversed join is admitted");
    builder
        .output(
            OutputKey::new("k_rope").expect("a test key is bounded"),
            reversed,
        )
        .expect("an output is declared");
    assert!(builder.build().is_ok());
}

// --- The semantic signature -------------------------------------------------

#[test]
fn the_semantic_signature_states_the_storage_claim_and_the_promotion_rule() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let facts = registry
        .operation_facts(&concatenate_f32_op())
        .expect("the concatenation is registered")
        .value();
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("the semantic signature is a record");
    };
    let read = |id: AttributeFieldId| {
        fields
            .iter()
            .find(|field| field.id() == id)
            .unwrap_or_else(|| panic!("field {id} is unconditional on this definition"))
            .value()
            .clone()
    };
    for (id, expected) in [
        (
            CONCATENATE_FACT_VALUE_BEHAVIOUR,
            "none-every-result-element-is-an-operand-element-unchanged",
        ),
        (
            CONCATENATE_FACT_OPERAND_ORDER,
            "semantic-result-coordinates-run-through-the-operands-in-order",
        ),
        (
            CONCATENATE_FACT_RESULT_EXTENT,
            "exact-sum-of-the-operand-extents-on-the-concatenated-axis-or-refusal",
        ),
        (
            CONCATENATE_FACT_EMPTY_OPERAND,
            "admitted-contributes-no-coordinate-and-still-agrees-on-every-other-axis",
        ),
        (
            CONCATENATE_FACT_TYPE_PROMOTION,
            "none-every-operand-is-already-tiler-f32-1",
        ),
        (
            CONCATENATE_FACT_STORAGE_CLAIM,
            "none-no-copy-move-or-materialization-is-claimed",
        ),
    ] {
        assert_eq!(
            read(id),
            CanonicalValue::utf8(expected).expect("a test fact is bounded")
        );
    }
    assert_eq!(
        fields.len(),
        6,
        "the signature has exactly the six published fields, so a new one cannot \
         be added without moving this count and the identity behind it"
    );
}

#[test]
fn the_normative_reference_states_the_zero_extent_rule_and_the_extent_refusal() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let definition = registry
        .operation_definition(&concatenate_f32_op())
        .expect("the concatenation is registered");
    let reference = definition.normative_definition().as_str();
    assert!(
        reference.contains("A zero-extent operand is admitted and contributes no coordinate"),
        "the zero-extent rule is stated in the registered definition rather than \
         inherited from whatever the empty case happens to do: {reference}"
    );
    assert!(
        reference.contains("yields that other operand's extent on the concatenated axis"),
        "and it is stated over the operands rather than at one workload's extents: the \
         illustration at the pinned prefill shape lives in `concatenate_result_shape`'s doc \
         comment, which is not encoded, because this text is: {reference}"
    );
    assert!(
        reference.contains("refused rather than saturated or wrapped"),
        "the extent-domain refusal is part of the normative definition: {reference}"
    );
    assert!(
        reference.contains("does not widen that domain"),
        "and states why an additive relation does not remove the refusal: {reference}"
    );
    assert!(
        reference.contains("makes no claim that storage was copied"),
        "the family's storage disclaimer is part of its normative definition: {reference}"
    );
    assert!(
        reference.contains("Operand order is semantic"),
        "and so is the order rule: {reference}"
    );
}

#[test]
fn the_concatenation_declares_no_algebraic_capability() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    assert!(
        !registry
            .operation_definition(&concatenate_f32_op())
            .expect("the concatenation is registered")
            .algebraic_capabilities()
            .declares_ordered_associativity(),
        "a family that performs no arithmetic has no rounding associativity to \
         declare, and a missing declaration is unknown rather than the inverse law"
    );
}

#[test]
fn every_refusal_carries_its_own_diagnostic_code() {
    let codes = [
        ConcatenateError::OperandCount {
            operands: 1,
            minimum: 2,
            maximum: 8,
        },
        ConcatenateError::RankDisagreement {
            operand: 1,
            rank: 2,
            first: 3,
        },
        ConcatenateError::AxisOutOfRange {
            axis: axis(4),
            rank: 2,
        },
        ConcatenateError::ExtentDisagreement {
            axis: axis(0),
            operand: 1,
            extent: 4,
            first: 8,
        },
        ConcatenateError::ResultExtentUnrelatable {
            axis: axis(1),
            accumulated: u64::MAX,
            operand: 1,
            extent: 1,
        },
        ConcatenateError::MalformedAxisAttribute,
        ConcatenateError::ResultShape(crate::shape::ShapeError::RankTooLarge { rank: 5, limit: 4 }),
    ]
    .map(|error| error.diagnostic_code());
    for (position, left) in codes.iter().enumerate() {
        for right in &codes[position + 1..] {
            assert_ne!(left, right, "each rule reports under its own code");
        }
        assert!(left.starts_with("concatenate."));
    }
}
