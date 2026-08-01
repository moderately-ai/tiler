use super::*;
use crate::semantic::{
    F32Reindex, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey, RegistryError,
    SemanticProgramBuilder,
};
use crate::shape::Shape;

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: u64) -> Extent {
    Extent::new(value)
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(
        F32::resolved_type(),
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn attributes(form: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(REINDEX_MAPPING_ATTRIBUTE, form)])
        .expect("a test attribute record is canonical")
}

/// Builds a form record from raw parts, bypassing every constructor check.
///
/// This is how a frontend that hand-assembles the canonical attribute reaches
/// the registered inference routine. Without it, a rule decided at construction
/// would only ever be reachable through the typed constructor, and the
/// requirement that it fire as a *provider diagnostic at construction* would be
/// untested.
fn raw_form(name: &str, fields: impl IntoIterator<Item = CanonicalField>) -> CanonicalValue {
    let mut all = vec![CanonicalField::new(
        REINDEX_FORM_KIND,
        CanonicalValue::utf8(name).expect("a test name is bounded"),
    )];
    all.extend(fields);
    CanonicalValue::record(all).expect("a test form record is canonical")
}

fn raw_axes(axes: &[u32]) -> CanonicalField {
    CanonicalField::new(
        REINDEX_FORM_AXES,
        CanonicalValue::sequence(axes.iter().copied().map(CanonicalValue::unsigned_u32))
            .expect("a test axis sequence is bounded"),
    )
}

fn raw_axis(value: u32) -> CanonicalField {
    CanonicalField::new(REINDEX_FORM_AXIS, CanonicalValue::unsigned_u32(value))
}

fn raw_factors(factors: &[u64]) -> CanonicalField {
    CanonicalField::new(
        REINDEX_FORM_FACTORS,
        CanonicalValue::sequence(factors.iter().copied().map(CanonicalValue::unsigned_u64))
            .expect("a test factor sequence is bounded"),
    )
}

fn infer(operand: &[u64], form: CanonicalValue) -> Result<Vec<ValueFact>, RegistryError> {
    FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(
            &reindex_f32_op(),
            &[f32_operand(operand)],
            &attributes(form),
        )
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operand: &[u64], form: CanonicalValue) -> String {
    let error = infer(operand, form).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a reindex refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Returns the complete diagnostic message of a refused application.
fn refusal_message(operand: &[u64], form: CanonicalValue) -> String {
    let error = infer(operand, form).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a reindex refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().message().to_owned()
}

fn result_shape(operand: &[u64], form: &ReindexForm) -> Shape {
    let results = infer(operand, form.canonical_value().clone()).expect("the form is admitted");
    let [result] = results.as_slice() else {
        panic!("a reindex has one result");
    };
    assert_eq!(result.resolved_type(), &F32::resolved_type());
    result.shape().clone()
}

// --- Every admitted form, at the pinned workload's own extents ---------------

/// The forms and ranks this module's positive evidence covers.
///
/// Six forms at ranks one through four, over the exact extents the pinned
/// `Qwen/Qwen3-0.6B-Base` C1 profile needs: the query and key head splits, the
/// grouped-query head-layout permutation, the attention-output merge, the rotary
/// half-split and its coordinate swap, and unit-axis insertion and removal at
/// the rotary sign operand's rank. Nothing here covers a symbolic extent, a rank
/// above four, or any dtype but F32.
#[test]
fn every_admitted_form_derives_its_result_shape_at_the_workload_extents() {
    // Split: the query projection's head split, `[T, 2048] -> [T, 16, 128]`.
    let head_split = ReindexForm::split_axis(axis(1), [extent(16), extent(128)])
        .expect("2048 = 16 x 128 is admitted");
    assert_eq!(
        result_shape(&[10, 2048], &head_split),
        Shape::from_dims([10, 16, 128])
    );
    // The same form at the key projection's smaller width.
    assert_eq!(
        result_shape(
            &[10, 1024],
            &ReindexForm::split_axis(axis(1), [extent(8), extent(128)]).expect("1024 = 8 x 128")
        ),
        Shape::from_dims([10, 8, 128])
    );
    // And the grouped-query split of the 16-head axis into (8, 2), major first,
    // which is what makes the group index `h / 2` rather than `h % 8`.
    assert_eq!(
        result_shape(
            &[10, 16, 128],
            &ReindexForm::split_axis(axis(1), [extent(8), extent(2)]).expect("16 = 8 x 2")
        ),
        Shape::from_dims([10, 8, 2, 128])
    );

    // Permute: `[T, 8, 2, 128] -> [8, 2, T, 128]`, the head-layout permutation.
    let layout = ReindexForm::permute_axes([axis(1), axis(2), axis(0), axis(3)])
        .expect("a four-axis permutation is admitted");
    assert_eq!(
        result_shape(&[10, 8, 2, 128], &layout),
        Shape::from_dims([8, 2, 10, 128])
    );

    // Merge: the attention output's `[T, 16, 128] -> [T, 2048]`.
    let output_merge =
        ReindexForm::merge_axes([axis(1), axis(2)]).expect("an adjacent pair merges");
    assert_eq!(
        result_shape(&[10, 16, 128], &output_merge),
        Shape::from_dims([10, 2048])
    );

    // Reverse: the rotary coordinate swap on the size-2 axis of a
    // `[T, heads, 2, 64]` operand. The shape is unchanged; only the reading
    // order of axis 2 is.
    let swap = ReindexForm::reverse_axis(axis(2)).expect("a reversal is admitted");
    assert_eq!(
        result_shape(&[10, 16, 2, 64], &swap),
        Shape::from_dims([10, 16, 2, 64])
    );

    // Unit-axis insertion and removal, at the rotary sign operand's rank.
    let insert = ReindexForm::insert_unit_axis(axis(1)).expect("an insertion is admitted");
    assert_eq!(result_shape(&[2], &insert), Shape::from_dims([2, 1]));
    let remove = ReindexForm::remove_unit_axis(axis(1)).expect("a removal is admitted");
    assert_eq!(result_shape(&[2, 1], &remove), Shape::from_dims([2]));
    // Insertion at the end is a position, not an axis, so `rank` is legal.
    assert_eq!(
        result_shape(
            &[10, 16],
            &ReindexForm::insert_unit_axis(axis(2)).expect("insertion at rank is admitted")
        ),
        Shape::from_dims([10, 16, 1])
    );
}

#[test]
fn a_split_and_its_merge_compose_back_to_the_operand_shape() {
    // The rotary composition's outer pair: `[…, 128] -> […, 2, 64] -> […, 128]`.
    let split = ReindexForm::split_axis(axis(2), [extent(2), extent(64)]).expect("128 = 2 x 64");
    let intermediate = result_shape(&[10, 16, 128], &split);
    assert_eq!(intermediate, Shape::from_dims([10, 16, 2, 64]));
    let merge = ReindexForm::merge_axes([axis(2), axis(3)]).expect("an adjacent pair merges");
    let dims: Vec<u64> = intermediate
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    assert_eq!(result_shape(&dims, &merge), Shape::from_dims([10, 16, 128]));
}

#[test]
fn the_authoring_facade_admits_a_form_through_the_governed_path() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let projection = builder
        .input::<F32>(
            InputKey::new("projection").expect("a valid key"),
            Shape::from_dims([10, 2048]),
        )
        .expect("an F32 input");
    let form =
        ReindexForm::split_axis(axis(1), [extent(16), extent(128)]).expect("2048 = 16 x 128");
    let heads = F32Reindex::apply(&mut builder, &form, projection).expect("the form is admitted");
    builder
        .output(OutputKey::new("heads").expect("a valid key"), heads)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    assert_eq!(program.operation_count(), 1);
    let occurrence = program
        .operations()
        .find(|operation| operation.key() == &reindex_f32_op())
        .expect("the reindex occurrence");
    assert_eq!(
        occurrence.attributes().canonical_encoding(),
        attributes(form.canonical_value().clone()).canonical_encoding()
    );
}

// --- Decision D-10 ----------------------------------------------------------

/// The one within-axis coordinate permutation D-10 admits, and the boundary.
#[test]
fn d10_admits_the_within_axis_reversal_and_no_other_within_axis_permutation() {
    // `i -> 1 - i` on a size-2 axis is `i -> extent - 1 - i` at extent two, which
    // is exactly the map `rotate_half` performs.
    let swap = ReindexForm::reverse_axis(axis(2)).expect("the rotary swap is admitted");
    assert_eq!(swap.kind(), ReindexFormKind::ReverseAxis);
    assert_eq!(
        result_shape(&[1, 16, 2, 64], &swap),
        Shape::from_dims([1, 16, 2, 64]),
        "the reversal changes the reading order of one axis and no extent"
    );

    // A within-axis rotation is expressible in the index vocabulary and is
    // deliberately not admitted here. It is refused by name, so a frontend that
    // reaches for one learns which family it is outside rather than receiving a
    // nearest admitted form.
    assert_eq!(
        refusal(&[1, 16, 2, 64], raw_form("rotate-axis", [raw_axis(2)])),
        "reindex.form.unadmitted-kind"
    );
    let message = refusal_message(&[1, 16, 2, 64], raw_form("rotate-axis", [raw_axis(2)]));
    assert!(
        message.contains("no other within-axis coordinate permutation is admitted"),
        "the refusal states D-10's boundary rather than only naming the form: {message}"
    );
    assert!(
        message.contains(REINDEX_FORM_REVERSE_AXIS),
        "and it names the one within-axis form that is admitted: {message}"
    );

    // A general within-axis permutation table lands in the same refusal, which
    // is the case that would otherwise admit a tensor-data-derived index.
    assert_eq!(
        refusal(
            &[1, 16, 2, 64],
            raw_form("permute-coordinates-within-axis", [raw_axis(2)])
        ),
        "reindex.form.unadmitted-kind"
    );
}

#[test]
fn a_reversal_of_an_axis_shorter_than_two_coordinates_denotes_no_reindex() {
    // `i -> extent - 1 - i` is the identity at extent one and issues no access at
    // extent zero, so neither is a reindex. This is the one rule of the reversal
    // form that depends on an extent rather than on the form alone.
    for operand in [[10, 1], [10, 0]] {
        assert_eq!(
            refusal(&operand, raw_form("reverse-axis", [raw_axis(1)])),
            "reindex.form.identity-mapping"
        );
    }
}

// --- The named admission rules, each under its own name ---------------------

#[test]
fn a_split_whose_factors_exceed_the_axis_extent_is_not_total() {
    // 2 x 100 = 200 over a 128-wide axis: result coordinate (1, 99) reads operand
    // coordinate 227, past the end of the axis.
    assert_eq!(
        refusal(
            &[10, 128],
            raw_form("split-axis", [raw_axis(1), raw_factors(&[2, 100])])
        ),
        "reindex.split.not-total"
    );
    let message = refusal_message(
        &[10, 128],
        raw_form("split-axis", [raw_axis(1), raw_factors(&[2, 100])]),
    );
    assert!(
        message.contains("200") && message.contains("128"),
        "the refusal names the declared product and the extent it must equal: {message}"
    );
}

#[test]
fn a_split_whose_factors_fall_short_of_the_axis_extent_is_a_slice() {
    // 2 x 32 = 64 over a 128-wide axis: the map is total and injective and reads
    // only the first 64 coordinates. That is a slice — a different family — and
    // the refusal says so rather than admitting a narrow reindex.
    assert_eq!(
        refusal(
            &[10, 128],
            raw_form("split-axis", [raw_axis(1), raw_factors(&[2, 32])])
        ),
        "reindex.split.not-surjective"
    );
    let message = refusal_message(
        &[10, 128],
        raw_form("split-axis", [raw_axis(1), raw_factors(&[2, 32])]),
    );
    assert!(
        message.contains("slice"),
        "a non-surjective mapping is refused as the family it actually is: {message}"
    );
}

#[test]
fn an_axis_order_that_is_not_a_permutation_is_refused_two_ways() {
    // A repeated axis reads one axis twice and drops another. Decided at
    // construction, because it is a property of the order alone.
    assert!(matches!(
        ReindexForm::permute_axes([axis(0), axis(0)]),
        Err(ReindexFormError::NotAPermutation { .. })
    ));
    assert_eq!(
        refusal(&[10, 16], raw_form("permute-axes", [raw_axes(&[0, 0])])),
        "reindex.permute.not-a-permutation"
    );
    // An axis outside `0..rank` is the remaining way an order of the right length
    // fails to be a permutation, and it is reported as one rather than as an
    // out-of-range axis.
    assert_eq!(
        refusal(&[10, 16], raw_form("permute-axes", [raw_axes(&[1, 5])])),
        "reindex.permute.not-a-permutation"
    );
    // An order of the wrong length is a different failure with its own code.
    assert_eq!(
        refusal(
            &[10, 16, 128],
            raw_form("permute-axes", [raw_axes(&[1, 0])])
        ),
        "reindex.permute.rank"
    );
}

#[test]
fn an_identity_form_denotes_no_reindex() {
    // Each spelling of "this operation returns its operand".
    assert!(matches!(
        ReindexForm::permute_axes([axis(0), axis(1)]),
        Err(ReindexFormError::IdentityMapping {
            kind: ReindexFormKind::PermuteAxes
        })
    ));
    assert!(matches!(
        ReindexForm::split_axis(axis(0), [extent(128)]),
        Err(ReindexFormError::IdentityMapping {
            kind: ReindexFormKind::SplitAxis
        })
    ));
    assert!(matches!(
        ReindexForm::merge_axes([axis(0)]),
        Err(ReindexFormError::IdentityMapping {
            kind: ReindexFormKind::MergeAxes
        })
    ));
    // And through a hand-assembled attribute, so the rule is a provider
    // diagnostic and not only a constructor error.
    assert_eq!(
        refusal(&[10, 16], raw_form("permute-axes", [raw_axes(&[0, 1])])),
        "reindex.form.identity-mapping"
    );
}

#[test]
fn a_merge_of_non_adjacent_axes_is_refused_under_its_own_rule() {
    // `[8, 2, T, 128]` merging axes 0 and 2 is a permutation composed with a
    // merge, and this family spells a composition as a chain of occurrences.
    assert!(matches!(
        ReindexForm::merge_axes([axis(0), axis(2)]),
        Err(ReindexFormError::MergeAxesNotAdjacent { .. })
    ));
    assert_eq!(
        refusal(
            &[8, 2, 10, 128],
            raw_form("merge-axes", [raw_axes(&[0, 2])])
        ),
        "reindex.merge.non-adjacent-axes"
    );
    // Descending axes are equally non-adjacent, and are reported the same way.
    assert_eq!(
        refusal(
            &[8, 2, 10, 128],
            raw_form("merge-axes", [raw_axes(&[2, 1])])
        ),
        "reindex.merge.non-adjacent-axes"
    );
}

#[test]
fn only_an_extent_one_axis_may_be_removed_and_only_a_real_position_inserted_at() {
    assert_eq!(
        refusal(&[10, 16], raw_form("remove-unit-axis", [raw_axis(1)])),
        "reindex.remove-unit-axis.not-unit"
    );
    let message = refusal_message(&[10, 16], raw_form("remove-unit-axis", [raw_axis(1)]));
    assert!(message.contains("extent 16"), "{message}");
    assert_eq!(
        refusal(&[10, 1], raw_form("remove-unit-axis", [raw_axis(4)])),
        "reindex.form.axis-out-of-range"
    );
    // A unit axis may be inserted at any position of the result, so `rank` is
    // legal and `rank + 1` is not.
    assert_eq!(
        refusal(&[10, 16], raw_form("insert-unit-axis", [raw_axis(3)])),
        "reindex.insert-unit-axis.out-of-range"
    );
}

#[test]
fn a_form_naming_an_axis_the_operand_does_not_have_is_refused() {
    assert_eq!(
        refusal(
            &[10, 128],
            raw_form("split-axis", [raw_axis(7), raw_factors(&[2, 64])])
        ),
        "reindex.form.axis-out-of-range"
    );
    assert_eq!(
        refusal(&[10, 128], raw_form("reverse-axis", [raw_axis(7)])),
        "reindex.form.axis-out-of-range"
    );
    assert_eq!(
        refusal(&[10, 128], raw_form("merge-axes", [raw_axes(&[5, 6])])),
        "reindex.form.axis-out-of-range"
    );
}

// --- Malformed attributes ---------------------------------------------------

#[test]
fn a_malformed_form_attribute_is_refused_under_its_own_subject() {
    // A record whose first field is not the kind.
    assert_eq!(
        refusal(
            &[10, 128],
            CanonicalValue::record([CanonicalField::new(
                REINDEX_FORM_AXIS,
                CanonicalValue::unsigned_u32(0),
            )])
            .expect("a test record is canonical")
        ),
        "reindex.form.malformed-attribute"
    );
    // A field set the form does not use: a split with no factors, and a reversal
    // carrying a factor sequence. Admitting either would let two attribute
    // records denote one form.
    assert_eq!(
        refusal(&[10, 128], raw_form("split-axis", [raw_axis(1)])),
        "reindex.form.malformed-attribute"
    );
    assert_eq!(
        refusal(
            &[10, 128],
            raw_form("reverse-axis", [raw_axis(1), raw_factors(&[2, 64])])
        ),
        "reindex.form.malformed-attribute"
    );
    // A kind field that is not UTF-8 at all.
    assert_eq!(
        refusal(
            &[10, 128],
            CanonicalValue::record([CanonicalField::new(
                REINDEX_FORM_KIND,
                CanonicalValue::boolean(true),
            )])
            .expect("a test record is canonical")
        ),
        "reindex.form.malformed-attribute"
    );
    assert_eq!(
        ReindexAttributeSubject::FormFields.to_string(),
        "form field set"
    );
    assert_eq!(ReindexAttributeSubject::FormKind.to_string(), "form kind");
}

#[test]
fn a_reindex_admits_exactly_one_f32_operand_and_produces_one_result() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let form =
        ReindexForm::split_axis(axis(1), [extent(16), extent(128)]).expect("2048 = 16 x 128");
    let results = registry
        .infer_operation(
            &reindex_f32_op(),
            &[f32_operand(&[10, 2048])],
            &attributes(form.canonical_value().clone()),
        )
        .expect("the form is admitted");
    assert_eq!(results.len(), 1);
    // An empty attribute record never reaches this family's inference, because
    // the schema declares the mapping attribute required and refuses first.
    assert_eq!(
        refusal_code(&OperationAttributes::empty()),
        "tiler.schema.missing-attribute",
        "a missing mapping attribute is refused by the schema, before this \
         family's own rules are consulted"
    );
}

/// Returns the diagnostic code of a refusal driven by a raw attribute record.
fn refusal_code(attributes: &OperationAttributes) -> String {
    let error = FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(&reindex_f32_op(), &[f32_operand(&[10, 2048])], attributes)
        .expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a reindex refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

// --- Canonical identity -----------------------------------------------------

#[test]
fn the_canonical_encoder_separates_forms_a_payload_only_encoding_would_collide() {
    let split = ReindexForm::split_axis(axis(0), [extent(2), extent(64)]).expect("128 = 2 x 64");
    let merge = ReindexForm::merge_axes([axis(0), axis(1)]).expect("an adjacent pair merges");
    let permute = ReindexForm::permute_axes([axis(1), axis(0)]).expect("a transpose");
    let reverse = ReindexForm::reverse_axis(axis(0)).expect("a reversal");
    let insert = ReindexForm::insert_unit_axis(axis(0)).expect("an insertion");
    let remove = ReindexForm::remove_unit_axis(axis(0)).expect("a removal");

    // Six forms, six encodings.
    let encodings = [
        split.canonical_encoding(),
        merge.canonical_encoding(),
        permute.canonical_encoding(),
        reverse.canonical_encoding(),
        insert.canonical_encoding(),
        remove.canonical_encoding(),
    ];
    for (position, left) in encodings.iter().enumerate() {
        for right in &encodings[position + 1..] {
            assert_ne!(left, right, "two distinct forms must not share an encoding");
        }
    }

    // Perturbation: drop the kind field, so a form is encoded under its payload
    // alone. `insert-unit-axis` and `remove-unit-axis` over axis 0 — two
    // operations with *inverse* meanings — then collide exactly, and so does a
    // reversal of the same axis. Neither assertion below is a property of the
    // shipped encoder; each is a property of a deliberately broken twin, which is
    // what makes the shipped encoder's own separations load-bearing.
    assert_eq!(
        payload_only_encoding(&insert),
        payload_only_encoding(&remove),
        "without the kind field an insertion and a removal share one identity"
    );
    assert_eq!(
        payload_only_encoding(&reverse),
        payload_only_encoding(&insert),
        "and so do a reversal and an insertion of the same axis"
    );
    assert_ne!(
        insert.canonical_encoding(),
        remove.canonical_encoding(),
        "the shipped encoder separates them"
    );

    // The encoding is domain-separated, so a form's bytes cannot be mistaken for
    // another canonical subject's.
    assert!(
        split
            .canonical_encoding()
            .as_bytes()
            .starts_with(&(REINDEX_FORM_DOMAIN.len() as u64).to_be_bytes())
    );
}

/// The shipped encoder with its kind field removed.
fn payload_only_encoding(form: &ReindexForm) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, REINDEX_FORM_DOMAIN);
    let CanonicalValueView::Record(fields) = form.canonical_value().view() else {
        panic!("a form is a record");
    };
    CanonicalValue::record(fields[1..].iter().cloned())
        .expect("a test record is canonical")
        .encode(&mut bytes);
    bytes
}

#[test]
fn a_decoded_form_round_trips_to_the_form_it_encodes() {
    for form in [
        ReindexForm::split_axis(axis(1), [extent(16), extent(128)]).expect("2048 = 16 x 128"),
        ReindexForm::merge_axes([axis(1), axis(2)]).expect("an adjacent pair merges"),
        ReindexForm::permute_axes([axis(1), axis(2), axis(0), axis(3)]).expect("a permutation"),
        ReindexForm::insert_unit_axis(axis(1)).expect("an insertion"),
        ReindexForm::remove_unit_axis(axis(1)).expect("a removal"),
        ReindexForm::reverse_axis(axis(2)).expect("a reversal"),
    ] {
        let decoded = ReindexForm::from_canonical_value(form.canonical_value())
            .expect("a form this module encoded decodes");
        assert_eq!(decoded, form);
        assert_eq!(decoded.kind(), form.kind());
        assert_eq!(decoded.axes(), form.axes());
        assert_eq!(decoded.factors(), form.factors());
    }
}

// --- The semantic signature -------------------------------------------------

#[test]
fn the_semantic_signature_states_the_storage_claim_and_the_admitted_forms() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let facts = registry
        .operation_facts(&reindex_f32_op())
        .expect("the reindex is registered")
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
            REINDEX_FACT_VALUE_BEHAVIOUR,
            "none-every-result-element-is-an-operand-element-unchanged",
        ),
        (
            REINDEX_FACT_MAPPING_CLASS,
            "total-over-the-result-domain-and-bijective-onto-the-operand-domain",
        ),
        (
            REINDEX_FACT_STORAGE_CLAIM,
            "none-no-transpose-copy-or-materialization-is-claimed",
        ),
        (
            REINDEX_FACT_ADMITTED_FORMS,
            "permute-axes,split-axis,merge-axes,insert-unit-axis,remove-unit-axis,reverse-axis",
        ),
    ] {
        assert_eq!(
            read(id),
            CanonicalValue::utf8(expected).expect("a test fact is bounded")
        );
    }
    assert_eq!(
        fields.len(),
        4,
        "the signature has exactly the four published fields, so a new one cannot \
         be added without moving this count and the identity behind it"
    );
}

#[test]
fn the_normative_reference_answers_d10_and_disclaims_the_storage_change() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let definition = registry
        .operation_definition(&reindex_f32_op())
        .expect("the reindex is registered");
    let reference = definition.normative_definition().as_str();
    assert!(
        reference.contains("D-10"),
        "the decision is answered in the registered definition rather than in a \
         research record: {reference}"
    );
    assert!(
        reference.contains("i -> extent - 1 - i"),
        "and the answer states the exact admitted coordinate map: {reference}"
    );
    assert!(
        reference.contains("makes no claim that storage was transposed"),
        "the family's storage disclaimer is part of its normative definition: {reference}"
    );
    for form in [
        REINDEX_FORM_PERMUTE_AXES,
        REINDEX_FORM_SPLIT_AXIS,
        REINDEX_FORM_MERGE_AXES,
        REINDEX_FORM_INSERT_UNIT_AXIS,
        REINDEX_FORM_REMOVE_UNIT_AXIS,
        REINDEX_FORM_REVERSE_AXIS,
    ] {
        assert!(
            reference.contains(form),
            "the normative reference names every admitted form, and omits {form}"
        );
    }
}

#[test]
fn the_reindex_declares_no_algebraic_capability() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    assert!(
        !registry
            .operation_definition(&reindex_f32_op())
            .expect("the reindex is registered")
            .algebraic_capabilities()
            .declares_ordered_associativity(),
        "a family that performs no arithmetic has no associativity to declare, and \
         a missing declaration is unknown rather than the inverse law"
    );
}
