use super::*;
use crate::program::abi::AvailabilityPhase;
use crate::semantic::{
    Bf16, F32Slice, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
    RegistryError, ReindexForm, SemanticProgramBuilder,
};
use crate::shape::{
    Axis, BindingSource, ExtentRelation, ExtentTerm, FactProvenance, RootBinding, Shape, ShapeEnv,
    ShapeEnvBuilder, ShapeSymbol, SourcedExtent, SymbolScope,
};
use std::sync::Arc;

const WHOLE: SliceAxisSelection = SliceAxisSelection::WholeAxis;

fn window(offset: u64, extent: u64) -> SliceAxisSelection {
    SliceAxisSelection::static_window(offset, Extent::new(extent))
}

fn static_extent(value: u64) -> SourcedExtent {
    SourcedExtent::Static(Extent::new(value))
}

fn selection(axes: &[SliceAxisSelection]) -> SliceSelection {
    SliceSelection::new(axes.iter().cloned()).expect("a test selection is admitted")
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(
        F32::resolved_type(),
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn attributes(value: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(SLICE_SELECTION_ATTRIBUTE, value)])
        .expect("a test attribute record is canonical")
}

/// Builds a selection record from raw parts, bypassing every constructor check.
///
/// This is how a frontend that hand-assembles the canonical attribute reaches
/// the registered inference routine, so a rule decided at construction is still
/// demonstrated firing as a provider diagnostic.
fn raw_selection(axes: Vec<CanonicalValue>) -> CanonicalValue {
    CanonicalValue::record([CanonicalField::new(
        SLICE_SELECTION_AXES,
        CanonicalValue::sequence(axes).expect("a test axis sequence is bounded"),
    )])
    .expect("a test selection record is canonical")
}

fn raw_axis(relation: &str, bounds: Option<(u64, u64)>) -> CanonicalValue {
    let name = CanonicalField::new(
        SLICE_AXIS_RELATION,
        CanonicalValue::utf8(relation).expect("a test relation name is bounded"),
    );
    match bounds {
        None => CanonicalValue::record([name]),
        Some((offset, extent)) => CanonicalValue::record([
            name,
            CanonicalField::new(SLICE_AXIS_OFFSET, CanonicalValue::unsigned_u64(offset)),
            CanonicalField::new(SLICE_AXIS_EXTENT, CanonicalValue::unsigned_u64(extent)),
        ]),
    }
    .expect("a test axis record is canonical")
}

fn infer(operand: &[u64], value: CanonicalValue) -> Result<Vec<ValueFact>, RegistryError> {
    FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(&slice_f32_op(), &[f32_operand(operand)], &attributes(value))
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operand: &[u64], value: CanonicalValue) -> String {
    let error = infer(operand, value).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a slice refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Returns the complete diagnostic message of a refused application.
fn refusal_message(operand: &[u64], value: CanonicalValue) -> String {
    let error = infer(operand, value).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a slice refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().message().to_owned()
}

fn result_shape(operand: &[u64], selection: &SliceSelection) -> Shape {
    let results =
        infer(operand, selection.canonical_value().clone()).expect("the selection is admitted");
    let [result] = results.as_slice() else {
        panic!("a slice has one result");
    };
    assert_eq!(result.resolved_type(), &F32::resolved_type());
    result
        .shape()
        .as_static()
        .expect("this family infers a literal boundary")
        .clone()
}

// --- What the family admits --------------------------------------------------

/// The selection classes this module's positive evidence covers.
///
/// Ranks one through four, a window on the outermost axis, on an interior axis,
/// on the innermost axis, and on several axes at once. Two of the shapes are the
/// selections the two consumer tickets need — a row range of a rotary table and
/// the final position of a logit projection — at literal offsets and stand-in
/// extents. Nothing here covers a symbolic extent, a rank above four, or any
/// dtype but F32.
#[test]
fn every_covered_selection_class_derives_its_result_shape() {
    // Rank one: a run of coordinates out of the middle.
    assert_eq!(
        result_shape(&[16], &selection(&[window(4, 6)])),
        Shape::from_dims([6])
    );
    // Rank two, outermost axis: rows `C ..= C + T` of a rotary table at `C = 4`,
    // `T = 6`. This is the selection the position-identity trigger needs, with a
    // literal cursor standing in for the bound symbol the family refuses.
    assert_eq!(
        result_shape(&[64, 128], &selection(&[window(4, 6), WHOLE])),
        Shape::from_dims([6, 128])
    );
    // Rank two, one position out of a projection: the final-position logits, whose
    // extent-one axis this family leaves in place for a reindex to remove.
    assert_eq!(
        result_shape(&[10, 2048], &selection(&[window(9, 1), WHOLE])),
        Shape::from_dims([1, 2048])
    );
    // Rank three, interior axis.
    assert_eq!(
        result_shape(&[2, 16, 128], &selection(&[WHOLE, window(0, 8), WHOLE])),
        Shape::from_dims([2, 8, 128])
    );
    // Rank three, innermost axis.
    assert_eq!(
        result_shape(&[2, 16, 128], &selection(&[WHOLE, WHOLE, window(64, 64)])),
        Shape::from_dims([2, 16, 64])
    );
    // Rank four, three axes restricted at once.
    assert_eq!(
        result_shape(
            &[4, 8, 16, 32],
            &selection(&[window(1, 2), WHOLE, window(0, 4), window(30, 2)])
        ),
        Shape::from_dims([2, 8, 4, 2])
    );
}

/// Every admitted occurrence reads a proper sub-region, and that is proved.
///
/// The family's mapping-class fact claims injectivity *and* non-surjectivity.
/// Injectivity is structural — distinct result coordinates map to distinct
/// operand coordinates because each axis's map is `c -> offset + c` — while
/// non-surjectivity is what [`SliceSelectionError::NoRestrictedAxis`] and
/// [`SliceSelectionError::WindowIsWholeAxis`] jointly enforce: an admitted
/// selection restricts at least one axis, and a restricting window leaves at
/// least one coordinate of that axis unread. The element counts are the
/// observable consequence over an operand that has elements; the sibling test
/// below covers the one operand shape where they are not, and says why that
/// occurrence is admitted anyway.
#[test]
fn every_admitted_selection_reads_strictly_fewer_elements_than_its_operand() {
    for (operand, axes) in [
        (vec![16_u64], vec![window(4, 6)]),
        (vec![64, 128], vec![window(4, 6), WHOLE]),
        (vec![10, 2048], vec![window(9, 1), WHOLE]),
        (vec![2, 16, 128], vec![WHOLE, window(0, 8), WHOLE]),
        (
            vec![4, 8, 16, 32],
            vec![window(1, 2), WHOLE, window(0, 4), window(30, 2)],
        ),
    ] {
        let selection = selection(&axes);
        let source = Shape::try_from_dims(operand.iter().copied()).expect("a test shape");
        let result = result_shape(&operand, &selection);
        assert_eq!(
            result.rank(),
            source.rank(),
            "a selection preserves rank: dropping an axis is a reindex written after it"
        );
        assert!(
            result.element_count() < source.element_count(),
            "{result:?} is not a proper sub-region of {source:?}"
        );
    }
}

/// An operand already empty on an unrestricted axis is admitted, not refused.
///
/// The empty-window rule refuses a *selection* that states emptiness. It does not
/// refuse an operand that has no elements, because that is a shape the program
/// had before the selection was written, and refusing it here would be this
/// family deciding something other than the selection — the same separation
/// `tiler::concatenate-f32@1`'s zero-extent rule makes. The strict element
/// inequality above is vacuous for this occurrence and the per-axis claim is not:
/// axis 1 is restricted to 64 of its 128 coordinates either way.
#[test]
fn an_operand_empty_on_an_unrestricted_axis_is_admitted_and_has_no_elements() {
    let selection = selection(&[WHOLE, window(0, 64)]);
    let result = result_shape(&[0, 128], &selection);
    assert_eq!(result, Shape::from_dims([0, 64]));
    assert_eq!(result.element_count(), Some(0));
    // And the empty *window* on the same operand is still refused, so the two
    // rules stay distinct rather than one swallowing the other.
    assert_eq!(
        SliceSelection::new([WHOLE, window(0, 0)]),
        Err(SliceSelectionError::EmptyWindow {
            axis: 1,
            offset: static_extent(0),
        })
    );
}

// --- The rules a selection owes on its own -----------------------------------

/// A selection that restricts nothing returns its operand and denotes no slice.
#[test]
fn a_selection_that_restricts_no_axis_is_refused() {
    assert_eq!(
        SliceSelection::new([WHOLE, WHOLE]),
        Err(SliceSelectionError::NoRestrictedAxis)
    );
    assert_eq!(
        refusal(
            &[10, 2048],
            raw_selection(vec![
                raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
                raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
            ])
        ),
        "slice.selection.no-restricted-axis"
    );
    // A rank-zero operand has no axis to restrict, so the empty selection its
    // rank would require lands in the same rule rather than in the entry count.
    assert_eq!(
        SliceSelection::new([]),
        Err(SliceSelectionError::NoRestrictedAxis)
    );
    assert_eq!(
        refusal(&[], raw_selection(Vec::new())),
        "slice.selection.no-restricted-axis"
    );
}

/// An empty window selects nothing, and is refused rather than admitted.
#[test]
fn an_empty_window_is_refused() {
    assert_eq!(
        SliceSelection::new([window(3, 0), WHOLE]),
        Err(SliceSelectionError::EmptyWindow {
            axis: 0,
            offset: static_extent(3),
        })
    );
    assert_eq!(
        refusal(
            &[10, 2048],
            raw_selection(vec![
                raw_axis(SLICE_RELATION_WINDOW, Some((3, 0))),
                raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
            ])
        ),
        "slice.selection.empty-window"
    );
    // The neighbouring admitted case, so the rule is a boundary rather than a
    // blanket refusal of small windows.
    assert_eq!(
        result_shape(&[10, 2048], &selection(&[window(3, 1), WHOLE])),
        Shape::from_dims([1, 2048])
    );
}

// --- The rules a selection owes against its operand ---------------------------

/// A window that leaves its axis is refused, never clamped and never wrapped.
#[test]
fn a_window_that_leaves_its_axis_is_refused_rather_than_clamped() {
    // One coordinate past the end.
    assert_eq!(
        selection(&[window(4, 13), WHOLE]).result_shape(&Shape::from_dims([16, 128])),
        Err(SliceSelectionError::WindowOutOfBounds {
            axis: 0,
            offset: static_extent(4),
            extent: 13,
            axis_extent: static_extent(16),
        })
    );
    assert_eq!(
        refusal(
            &[16, 128],
            selection(&[window(4, 13), WHOLE]).canonical_value().clone()
        ),
        "slice.selection.out-of-bounds"
    );
    // The clamping convention two primary authorities define would return a
    // `[12, 128]` result here. Nothing does.
    assert!(
        infer(
            &[16, 128],
            selection(&[window(4, 13), WHOLE]).canonical_value().clone()
        )
        .is_err()
    );
    // An offset alone past the end, with a window that would fit inside the axis.
    assert_eq!(
        refusal(
            &[16, 128],
            selection(&[window(20, 2), WHOLE]).canonical_value().clone()
        ),
        "slice.selection.out-of-bounds"
    );
    // An offset whose sum with the extent leaves the extent domain. The saturated
    // sum is still out of bounds, so the arithmetic cannot wrap into an admitted
    // window: `u64::MAX + 2` wrapped would be `1`, which is inside every axis of
    // extent two or more.
    assert_eq!(
        refusal(
            &[16, 128],
            selection(&[window(u64::MAX, 2), WHOLE])
                .canonical_value()
                .clone()
        ),
        "slice.selection.out-of-bounds"
    );
    // The neighbouring admitted case: the window that ends exactly at the end.
    assert_eq!(
        result_shape(&[16, 128], &selection(&[window(4, 12), WHOLE])),
        Shape::from_dims([12, 128])
    );
    // An axis of extent zero admits no window at all, because every window this
    // family admits selects at least one coordinate.
    assert_eq!(
        refusal(
            &[0, 128],
            selection(&[window(0, 1), WHOLE]).canonical_value().clone()
        ),
        "slice.selection.out-of-bounds"
    );
}

/// A window covering its axis is the whole-axis relation, so it has one spelling.
#[test]
fn a_window_covering_its_axis_is_refused_as_the_whole_axis_relation() {
    assert_eq!(
        selection(&[window(0, 16), window(0, 8)]).result_shape(&Shape::from_dims([16, 128])),
        Err(SliceSelectionError::WindowIsWholeAxis {
            axis: 0,
            extent: 16,
        })
    );
    assert_eq!(
        refusal(
            &[16, 128],
            selection(&[window(0, 16), window(0, 8)])
                .canonical_value()
                .clone()
        ),
        "slice.selection.window-is-whole-axis"
    );
    // An extent-one axis is covered by any window it admits, so it is always
    // stated as a whole axis.
    assert_eq!(
        refusal(
            &[1, 128],
            selection(&[window(0, 1), window(0, 8)])
                .canonical_value()
                .clone()
        ),
        "slice.selection.window-is-whole-axis"
    );
}

/// A selection states exactly one entry per operand axis.
///
/// This rule is what replaces a sparse form's duplicated-axis and out-of-range
/// axis refusals: neither state is representable here, and the entry count is
/// decided against the operand's own rank rather than against a caller's claim
/// about it.
#[test]
fn a_selection_that_is_not_one_entry_per_operand_axis_is_refused() {
    let two = selection(&[window(1, 2), WHOLE]);
    assert_eq!(
        two.result_shape(&Shape::from_dims([16, 128, 4])),
        Err(SliceSelectionError::SelectionCountMismatch {
            entries: 2,
            rank: 3,
        })
    );
    assert_eq!(
        refusal(&[16, 128, 4], two.canonical_value().clone()),
        "slice.selection.entry-count"
    );
    let four = selection(&[window(1, 2), WHOLE, WHOLE, WHOLE]);
    assert_eq!(
        refusal(&[16, 128, 4], four.canonical_value().clone()),
        "slice.selection.entry-count"
    );
    assert!(
        refusal_message(&[16, 128, 4], four.canonical_value().clone())
            .contains("states 4 entries and the operand has 3 axes"),
        "the refusal names both counts so a caller can fix the program"
    );
}

// --- The two reserved relations ----------------------------------------------

/// The strided and symbolic forms are refused under their own rules.
///
/// Each is a *reserved* name rather than an unrecognized one, so a frontend
/// reaching for a form this family does not have is told which form is missing
/// and why, instead of being told its name was not recognized.
#[test]
fn the_two_reserved_relations_are_refused_under_their_own_rules() {
    let strided = raw_selection(vec![
        raw_axis(SLICE_RELATION_STRIDED_WINDOW, Some((0, 8))),
        raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
    ]);
    assert_eq!(
        refusal(&[16, 128], strided.clone()),
        "slice.selection.strided-window-unsupported"
    );
    assert!(
        refusal_message(&[16, 128], strided).contains("negative stride"),
        "the refusal names the reserved half of the schema that blocks it"
    );

    let symbolic = raw_selection(vec![
        raw_axis(SLICE_RELATION_SYMBOLIC_WINDOW, Some((0, 8))),
        raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
    ]);
    assert_eq!(
        refusal(&[16, 128], symbolic.clone()),
        "slice.selection.symbolic-offset-unsupported"
    );
    let message = refusal_message(&[16, 128], symbolic);
    assert_eq!(
        message,
        "symbolic-window is reserved and not admitted: a source-bearing offset uses the admitted window relation with a sourced extent field, and this reserved name is not a second variant of that relation",
        "the refusal states that the reserved name is not a second window variant"
    );

    // Both are decided before the record's field shape, so a reserved relation
    // carrying no bounds at all still refuses under its own rule rather than as a
    // malformed record.
    assert_eq!(
        refusal(
            &[16, 128],
            raw_selection(vec![
                raw_axis(SLICE_RELATION_SYMBOLIC_WINDOW, None),
                raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
            ])
        ),
        "slice.selection.symbolic-offset-unsupported"
    );
}

/// Every other name is refused by name, and never mapped to a nearest match.
#[test]
fn an_unadmitted_relation_is_refused_by_name() {
    let value = raw_selection(vec![
        raw_axis("windows", Some((0, 8))),
        raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
    ]);
    assert_eq!(
        refusal(&[16, 128], value.clone()),
        "slice.selection.unadmitted-relation"
    );
    let message = refusal_message(&[16, 128], value);
    assert!(
        message.contains("windows") && message.contains(SLICE_RELATION_WINDOW),
        "the refusal names the rejected relation and the admitted set: {message}"
    );

    // A rejected name is truncated into the governed message bound rather than
    // turning the refusal into a provider-contract failure about its own length.
    let long = "w".repeat(4_096);
    assert_eq!(
        refusal(
            &[16, 128],
            raw_selection(vec![
                raw_axis(&long, None),
                raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
            ])
        ),
        "slice.selection.unadmitted-relation"
    );
}

// --- Malformed attributes -----------------------------------------------------

/// Every shape of malformed selection attribute is refused, by subject.
#[test]
fn every_malformed_selection_attribute_is_refused() {
    // An attribute that is not a record at all never reaches this family, because
    // the schema declares the attribute's kind and refuses first. The family's own
    // record rule is still reachable — and still fires — through the decoder a
    // reference evaluator calls.
    assert_eq!(
        refusal(&[16, 128], CanonicalValue::unsigned_u64(3)),
        "tiler.schema.attribute-kind"
    );
    assert_eq!(
        SliceSelection::from_canonical_value(&CanonicalValue::unsigned_u64(3)),
        Err(SliceSelectionError::MalformedAttribute {
            subject: SliceAttributeSubject::SelectionRecord,
        })
    );

    let malformed = [
        // The record's one field is not the axis sequence.
        CanonicalValue::record([CanonicalField::new(
            SLICE_AXIS_OFFSET,
            CanonicalValue::sequence([raw_axis(SLICE_RELATION_WHOLE_AXIS, None)])
                .expect("a test sequence"),
        )])
        .expect("a test record"),
        // The record carries a second field.
        CanonicalValue::record([
            CanonicalField::new(
                SLICE_SELECTION_AXES,
                CanonicalValue::sequence([raw_axis(SLICE_RELATION_WHOLE_AXIS, None)])
                    .expect("a test sequence"),
            ),
            CanonicalField::new(SLICE_AXIS_OFFSET, CanonicalValue::unsigned_u64(0)),
        ])
        .expect("a test record"),
        // The axis field is not a sequence.
        CanonicalValue::record([CanonicalField::new(
            SLICE_SELECTION_AXES,
            CanonicalValue::unsigned_u64(2),
        )])
        .expect("a test record"),
        // One entry is not a record.
        raw_selection(vec![CanonicalValue::unsigned_u64(0)]),
        // One relation name is not UTF-8 text.
        raw_selection(vec![
            CanonicalValue::record([CanonicalField::new(
                SLICE_AXIS_RELATION,
                CanonicalValue::unsigned_u64(0),
            )])
            .expect("a test record"),
        ]),
        // A whole axis carrying window bounds.
        raw_selection(vec![raw_axis(SLICE_RELATION_WHOLE_AXIS, Some((0, 4)))]),
        // A window carrying no bounds.
        raw_selection(vec![raw_axis(SLICE_RELATION_WINDOW, None)]),
        // A window whose offset is not a 64-bit unsigned value.
        raw_selection(vec![
            CanonicalValue::record([
                CanonicalField::new(
                    SLICE_AXIS_RELATION,
                    CanonicalValue::utf8(SLICE_RELATION_WINDOW).expect("a test name"),
                ),
                CanonicalField::new(SLICE_AXIS_OFFSET, CanonicalValue::unsigned_u32(0)),
                CanonicalField::new(SLICE_AXIS_EXTENT, CanonicalValue::unsigned_u64(4)),
            ])
            .expect("a test record"),
        ]),
        // A window whose extent is not a 64-bit unsigned value.
        raw_selection(vec![
            CanonicalValue::record([
                CanonicalField::new(
                    SLICE_AXIS_RELATION,
                    CanonicalValue::utf8(SLICE_RELATION_WINDOW).expect("a test name"),
                ),
                CanonicalField::new(SLICE_AXIS_OFFSET, CanonicalValue::unsigned_u64(0)),
                CanonicalField::new(SLICE_AXIS_EXTENT, CanonicalValue::unsigned_u32(4)),
            ])
            .expect("a test record"),
        ]),
    ];
    for value in malformed {
        assert_eq!(
            refusal(&[16, 128], value.clone()),
            "slice.selection.malformed-attribute",
            "{value:?} is malformed and must be refused as such"
        );
    }
    // The subjects a reader sees in those messages.
    assert_eq!(
        SliceAttributeSubject::SelectionRecord.to_string(),
        "selection record"
    );
    assert_eq!(SliceAttributeSubject::AxisRecord.to_string(), "axis record");
    assert_eq!(SliceAttributeSubject::Offset.to_string(), "window offset");
}

/// A selection wider than the canonical structure admits is refused.
///
/// Two different bounds are reachable here and both are refusals rather than
/// truncations. The entry count is refused above [`MAX_SLICE_SELECTION_AXES`],
/// and a narrower selection of *windows* can still exhaust the shared canonical
/// node budget first, because a window record costs four nodes where a whole axis
/// costs two.
#[test]
fn a_selection_wider_than_the_canonical_structure_admits_is_refused() {
    let too_many = MAX_SLICE_SELECTION_AXES.saturating_add(1);
    let mut axes = vec![WHOLE; too_many];
    axes[0] = window(0, 1);
    assert_eq!(
        SliceSelection::new(axes),
        Err(SliceSelectionError::TooManyAxes {
            axes: too_many,
            limit: MAX_SLICE_SELECTION_AXES,
        })
    );

    let node_bound = SliceSelection::new(vec![window(0, 1); MAX_SLICE_SELECTION_AXES])
        .expect_err("a selection of that many windows exceeds the canonical node budget");
    assert_eq!(
        node_bound.diagnostic_code(),
        "slice.selection.canonical-bound"
    );
    // The neighbouring admitted width, so the bound is a boundary rather than a
    // blanket refusal of wide selections.
    assert!(SliceSelection::new(vec![window(0, 1); 1_000]).is_ok());
}

// --- The occurrence's own shape ------------------------------------------------

#[test]
fn a_slice_admits_exactly_one_f32_operand_and_produces_one_result() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let selection = selection(&[window(4, 6), WHOLE]);
    let results = registry
        .infer_operation(
            &slice_f32_op(),
            &[f32_operand(&[64, 128])],
            &attributes(selection.canonical_value().clone()),
        )
        .expect("the selection is admitted");
    assert_eq!(results.len(), 1);

    // An empty attribute record never reaches this family's inference, because
    // the schema declares the selection attribute required and refuses first.
    let error = registry
        .infer_operation(
            &slice_f32_op(),
            &[f32_operand(&[64, 128])],
            &OperationAttributes::empty(),
        )
        .expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a slice refusal is a provider-attributed rejection, not {error}");
    };
    assert_eq!(
        rejection.source_error().code().as_str(),
        "tiler.schema.missing-attribute"
    );

    // A second dtype is refused: this family converts nothing, so it admits only
    // the value type its signature names.
    let error = registry
        .infer_operation(
            &slice_f32_op(),
            &[ValueFact::new(
                Bf16::resolved_type(),
                Shape::from_dims([64, 128]),
            )],
            &attributes(selection.canonical_value().clone()),
        )
        .expect_err("a bf16 operand is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a slice refusal is a provider-attributed rejection, not {error}");
    };
    assert_eq!(rejection.source_error().code().as_str(), "slice.type");
}

#[test]
fn the_authoring_facade_admits_a_selection_through_the_governed_path() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let table = builder
        .input::<F32>(
            InputKey::new("table").expect("a valid key"),
            Shape::from_dims([64, 128]),
        )
        .expect("an F32 input");
    let selection = selection(&[window(4, 6), WHOLE]);
    let rows = F32Slice::apply(&mut builder, &selection, table).expect("the selection is admitted");
    builder
        .output(OutputKey::new("rows").expect("a valid key"), rows)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    assert_eq!(program.operation_count(), 1);
    let occurrence = program
        .operations()
        .find(|operation| operation.key() == &slice_f32_op())
        .expect("the slice occurrence");
    assert_eq!(
        occurrence.attributes().canonical_encoding(),
        attributes(selection.canonical_value().clone()).canonical_encoding()
    );
}

// --- Canonical identity --------------------------------------------------------

#[test]
fn the_canonical_encoder_separates_selections_a_bounds_only_encoding_would_collide() {
    let leading = selection(&[window(0, 4), WHOLE]);
    let trailing = selection(&[WHOLE, window(0, 4)]);
    assert_ne!(
        leading.canonical_encoding(),
        trailing.canonical_encoding(),
        "two selections differing only in which axis is restricted are distinct"
    );

    // Perturbation: encode the window bounds alone, dropping the per-axis
    // structure. The two selections above then collide exactly — both restrict
    // one axis to four coordinates from zero — even though they read different
    // sub-regions, which is the identity collision the ordered per-axis sequence
    // exists to prevent. This assertion is a property of a deliberately broken
    // twin, not of the shipped encoder.
    assert_eq!(
        bounds_only_encoding(&leading),
        bounds_only_encoding(&trailing),
        "without the per-axis structure two distinct selections share one identity"
    );

    // Offset and extent are separate fields rather than one pair, so a window
    // moved by one and a window widened by one stay distinct.
    assert_ne!(
        selection(&[window(1, 4), WHOLE]).canonical_encoding(),
        selection(&[window(0, 5), WHOLE]).canonical_encoding()
    );
    // A whole axis and a window that would cover it are different entries, which
    // is what makes the whole-axis refusal a canonicality rule rather than a
    // redundancy.
    assert_ne!(
        selection(&[window(0, 4), window(0, 4)]).canonical_encoding(),
        selection(&[WHOLE, window(0, 4)]).canonical_encoding()
    );

    // The encoding is domain-separated, so a selection's bytes cannot be mistaken
    // for another canonical subject's.
    assert!(
        leading
            .canonical_encoding()
            .as_bytes()
            .starts_with(&(SLICE_SELECTION_DOMAIN.len() as u64).to_be_bytes())
    );
}

/// The shipped encoder with the per-axis structure flattened away.
fn bounds_only_encoding(selection: &SliceSelection) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, SLICE_SELECTION_DOMAIN);
    let bounds: Vec<CanonicalValue> = selection
        .axes()
        .iter()
        .filter(|entry| entry.is_restricting())
        .map(|entry| match entry {
            SliceAxisSelection::WholeAxis => unreachable!("filtered above"),
            SliceAxisSelection::Window { offset, extent } => CanonicalValue::sequence([
                match offset {
                    SourcedExtent::Static(value) => CanonicalValue::unsigned_u64(value.get()),
                    SourcedExtent::Symbol(_) => {
                        panic!("the bounds-only twin is only defined for literal offsets")
                    }
                },
                CanonicalValue::unsigned_u64(extent.get()),
            ])
            .expect("a test bound pair"),
        })
        .collect();
    CanonicalValue::sequence(bounds)
        .expect("a test bound sequence")
        .encode(&mut bytes);
    bytes
}

#[test]
fn a_decoded_selection_round_trips_to_the_selection_it_encodes() {
    for original in [
        selection(&[window(4, 6)]),
        selection(&[window(4, 6), WHOLE]),
        selection(&[WHOLE, window(0, 8), WHOLE]),
        selection(&[window(1, 2), WHOLE, window(0, 4), window(30, 2)]),
    ] {
        let decoded = SliceSelection::from_canonical_value(original.canonical_value())
            .expect("a selection this module encoded decodes");
        assert_eq!(decoded, original);
        assert_eq!(decoded.axes(), original.axes());
    }
}

// --- The semantic signature -----------------------------------------------------

#[test]
fn the_semantic_signature_states_the_mapping_class_and_the_out_of_bounds_rule() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let facts = registry
        .operation_facts(&slice_f32_op())
        .expect("the slice is registered")
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
            SLICE_FACT_VALUE_BEHAVIOUR,
            "none-every-result-element-is-an-operand-element-unchanged",
        ),
        (
            SLICE_FACT_MAPPING_CLASS,
            "total-over-the-result-domain-and-injective-not-surjective-into-the-operand-domain",
        ),
        (
            SLICE_FACT_STORAGE_CLAIM,
            "none-no-copy-view-or-materialization-is-claimed",
        ),
        (SLICE_FACT_ADMITTED_RELATIONS, "whole-axis,window"),
        (
            SLICE_FACT_OUT_OF_BOUNDS,
            "refused-at-construction-never-clamped-and-never-wrapped",
        ),
    ] {
        assert_eq!(
            read(id),
            CanonicalValue::utf8(expected).expect("a test fact is bounded")
        );
    }
    assert_eq!(
        fields.len(),
        5,
        "the signature has exactly the five published fields, so a new one cannot \
         be added without moving this count and the identity behind it"
    );
}

#[test]
fn the_normative_reference_states_both_relations_both_reservations_and_the_bound_rule() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let definition = registry
        .operation_definition(&slice_f32_op())
        .expect("the slice is registered");
    let reference = definition.normative_definition().as_str();
    for relation in [
        SLICE_RELATION_WHOLE_AXIS,
        SLICE_RELATION_WINDOW,
        SLICE_RELATION_STRIDED_WINDOW,
        SLICE_RELATION_SYMBOLIC_WINDOW,
    ] {
        assert!(
            reference.contains(relation),
            "the normative reference names every admitted and reserved relation, and omits {relation}"
        );
    }
    assert!(
        reference.contains("never clamped")
            && reference.contains("never")
            && reference.contains("wrapped"),
        "the out-of-bounds posture is part of the definition rather than an implementation detail: {reference}"
    );
    assert!(
        reference.contains("injective and ") && reference.contains("not surjective"),
        "the mapping class is stated: {reference}"
    );
    assert!(
        reference.contains("an operand that is already empty"),
        "and so is the one occurrence where that class does not become an element count: {reference}"
    );
    assert!(
        reference.contains("tiler::reindex-f32@1"),
        "and it names the family that removes the axis a selection leaves behind: {reference}"
    );
    for clause in [
        "offset is a SourcedExtent",
        "ShapeEnv is the only binding authority",
        "offset + extent <= available_axis",
        "after its name is decoded and before parsing any relation-specific fields",
        "Literal window bytes stay an unsigned 64-bit offset",
        "this reserved name is not a second variant of that relation",
    ] {
        assert!(
            reference.contains(clause),
            "the normative reference omits the source-bearing window clause {clause:?}: {reference}"
        );
    }
    for retired_clause in [
        "no index-expression variant carries",
        "a semantic value fact carries static extents",
        "window grammar carries only a literal offset and a literal extent",
        "no source-bearing selection field",
        "no source-bearing selection reaches that inference",
    ] {
        assert!(
            !reference.contains(retired_clause),
            "the normative reference retains the false clause {retired_clause:?}: {reference}"
        );
    }
}

#[test]
fn the_slice_declares_no_algebraic_capability() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    assert!(
        !registry
            .operation_definition(&slice_f32_op())
            .expect("the slice is registered")
            .algebraic_capabilities()
            .declares_ordered_associativity(),
        "a family that performs no arithmetic has no associativity to declare, and \
         a missing declaration is unknown rather than the inverse law"
    );
}

/// The neighbouring families still refuse a selection, now by referring to one.
///
/// Registering this key does not widen the reindex: a split whose factors fall
/// short of its axis is still refused under its own rule rather than admitted as
/// a narrow selection. Nothing here is a claim about the slice's own admission;
/// it is the check that the family boundary the two definitions describe stayed
/// where both of them say it is.
#[test]
fn the_reindex_still_refuses_a_non_surjective_split_as_a_slice() {
    let short = ReindexForm::split_axis(Axis::new(1), [Extent::new(2), Extent::new(60)])
        .expect("the form itself is well shaped");
    let error = short
        .result_shape(&Shape::from_dims([10, 128]))
        .expect_err("2 x 60 is short of 128");
    assert_eq!(error.diagnostic_code(), "reindex.split.not-surjective");
}

// --- Source-bearing window offsets -------------------------------------------

fn scope() -> SymbolScope {
    SymbolScope::new("slice/0").unwrap()
}

fn sym(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(scope(), name).unwrap()
}

fn input_binding(input: &str, axis: u32) -> RootBinding {
    RootBinding::new(
        BindingSource::InputDimension {
            input: InputKey::new(input).expect("a valid key"),
            axis: Axis::new(axis),
        },
        crate::shape::EXTENT_PHASE_CEILING,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

fn static_binding(value: u64) -> RootBinding {
    RootBinding::new(
        BindingSource::Static(Extent::new(value)),
        AvailabilityPhase::CompileProfile,
        FactProvenance::StaticallyProven,
    )
    .unwrap()
}

fn late_binding() -> RootBinding {
    RootBinding::new(
        BindingSource::TargetProperty {
            key: crate::program::abi::TargetPropertyKey::new("tiler.target.test@1").unwrap(),
        },
        AvailabilityPhase::PreparedKernelPreflight,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

fn env_with(bindings: &[(&str, RootBinding)], relations: &[ExtentRelation]) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    for (name, binding) in bindings {
        let declared = sym(name);
        draft.declare(declared.clone()).unwrap();
        draft.bind(&declared, binding.clone()).unwrap();
    }
    for relation in relations {
        draft
            .require(crate::shape::SemanticInputConstraint::new(
                relation.clone(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
    }
    Arc::new(draft.build().unwrap())
}

fn interval(name: &str, lower: u64, upper: u64) -> ExtentRelation {
    ExtentRelation::interval(ExtentTerm::Symbol(sym(name)), lower, upper).unwrap()
}

fn symbolic_window(name: &str, extent: u64) -> SliceAxisSelection {
    SliceAxisSelection::Window {
        offset: SourcedExtent::Symbol(sym(name)),
        extent: Extent::new(extent),
    }
}

fn apply_sourced(
    environment: Arc<ShapeEnv>,
    operand: Vec<SourcedExtent>,
    axes: &[SliceAxisSelection],
) -> Result<Vec<SourcedExtent>, String> {
    let mut builder = SemanticProgramBuilder::try_standard_with_shape_environment(environment)
        .expect("the standard builder opens");
    let input = builder
        .input_sourced::<crate::semantic::F32>(
            InputKey::new("table").expect("a valid key"),
            operand,
        )
        .map_err(|error| format!("input refused: {error}"))?;
    let selection =
        SliceSelection::new(axes.iter().cloned()).map_err(|error| format!("selection: {error}"))?;
    let selected =
        F32Slice::apply(&mut builder, &selection, input).map_err(|error| match error {
            crate::semantic::BuildError::SemanticRegistry(
                RegistryError::RejectedOperationApplication(rejection),
            ) => format!(
                "{}: {}",
                rejection.source_error().code().as_str(),
                rejection.source_error().message()
            ),
            other => format!("{other}"),
        })?;
    builder
        .output(OutputKey::new("rows").expect("a valid key"), selected)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    let value = program.outputs().next().unwrap().value();
    Ok(program.shape(value).unwrap().extents().collect())
}

fn apply_error(environment: Arc<ShapeEnv>, axes: &[SliceAxisSelection]) -> String {
    apply_sourced(
        environment,
        vec![static_extent(64), static_extent(128)],
        axes,
    )
    .expect_err("the sourced selection is refused")
}

#[test]
fn a_literal_window_encoding_is_byte_identical_to_the_unsigned_field() {
    let encoded = selection(&[window(4, 6), WHOLE]).canonical_value().clone();
    let expected = raw_selection(vec![
        raw_axis(SLICE_RELATION_WINDOW, Some((4, 6))),
        raw_axis(SLICE_RELATION_WHOLE_AXIS, None),
    ]);
    assert_eq!(
        encoded, expected,
        "a literal window must keep the unsigned 64-bit offset spelling"
    );
}

#[test]
fn a_symbolic_offset_is_injective_and_distinct_from_its_literal_neighbour() {
    let literal = selection(&[window(4, 6), WHOLE]);
    let symbolic = SliceSelection::new([symbolic_window("c", 6), WHOLE])
        .expect("a symbolic window is context-free");
    assert_ne!(
        literal.canonical_encoding(),
        symbolic.canonical_encoding(),
        "a symbol and the number it may resolve to are different selections"
    );

    let other = SliceSelection::new([symbolic_window("d", 6), WHOLE])
        .expect("a second symbol is context-free");
    assert_ne!(
        symbolic.canonical_encoding(),
        other.canonical_encoding(),
        "two symbols that may resolve to the same number stay identity-distinct"
    );

    let decoded = SliceSelection::from_canonical_value(symbolic.canonical_value())
        .expect("a selection this module encoded decodes");
    assert_eq!(decoded, symbolic);
}

#[test]
fn a_static_source_offset_is_admitted_when_the_environment_proves_its_bounds() {
    let environment = env_with(&[("c", static_binding(4))], &[interval("c", 4, 4)]);
    let result = apply_sourced(
        environment,
        vec![static_extent(64), static_extent(128)],
        &[symbolic_window("c", 6), WHOLE],
    )
    .expect("a proved static cursor is admitted");
    assert_eq!(result, vec![static_extent(6), static_extent(128)]);
}

#[test]
fn an_input_dimension_offset_is_admitted_when_the_environment_proves_its_bounds() {
    let environment = env_with(
        &[("c", input_binding("tokens", 0))],
        &[interval("c", 0, 58)],
    );
    let result = apply_sourced(
        environment,
        vec![static_extent(64), static_extent(128)],
        &[symbolic_window("c", 6), WHOLE],
    )
    .expect("a proved input-dimension cursor is admitted");
    assert_eq!(result, vec![static_extent(6), static_extent(128)]);
}

#[test]
fn a_sourced_operand_axis_keeps_its_extent_when_left_whole() {
    let environment = env_with(
        &[
            ("c", input_binding("tokens", 0)),
            ("n", input_binding("table", 1)),
        ],
        &[interval("c", 0, 58), interval("n", 128, 128)],
    );
    let result = apply_sourced(
        environment,
        vec![static_extent(64), SourcedExtent::Symbol(sym("n"))],
        &[symbolic_window("c", 6), WHOLE],
    )
    .expect("an untouched sourced axis is retained");
    assert_eq!(
        result,
        vec![static_extent(6), SourcedExtent::Symbol(sym("n"))]
    );
}

#[test]
fn a_foreign_symbol_is_refused_as_undeclared() {
    let environment = env_with(
        &[("c", input_binding("tokens", 0))],
        &[interval("c", 0, 58)],
    );
    let error = apply_error(environment, &[symbolic_window("ghost", 6), WHOLE]);
    assert!(
        error.starts_with("slice.selection.undeclared-symbol"),
        "a foreign symbol is undeclared in this environment: {error}"
    );
}

#[test]
fn an_undeclared_symbol_is_refused_without_an_environment() {
    let selection = SliceSelection::new([symbolic_window("c", 6)]).expect("construction is free");
    assert_eq!(
        selection.result_shape(&Shape::from_dims([64])),
        Err(SliceSelectionError::UndeclaredSymbol { symbol: sym("c") })
    );
}

#[test]
fn a_late_source_is_refused_by_phase() {
    let environment = env_with(&[("c", late_binding())], &[interval("c", 0, 58)]);
    let error = apply_error(environment, &[symbolic_window("c", 6), WHOLE]);
    assert!(
        error.starts_with("slice.selection.source-too-late"),
        "a prepared-kernel source is after the extent ceiling: {error}"
    );
}

#[test]
fn an_interval_that_cannot_prove_the_bound_is_refused() {
    let environment = env_with(
        &[("c", input_binding("tokens", 0))],
        &[interval("c", 0, 64)],
    );
    let error = apply_error(environment, &[symbolic_window("c", 6), WHOLE]);
    assert!(
        error.starts_with("slice.selection.bound-unproved"),
        "C in [0, 64] does not prove C + 6 <= 64: {error}"
    );
}

#[test]
fn a_proved_overflow_is_out_of_bounds_not_unproved() {
    let environment = env_with(
        &[("c", input_binding("tokens", 0))],
        &[interval("c", 60, 64)],
    );
    let error = apply_error(environment, &[symbolic_window("c", 6), WHOLE]);
    assert!(
        error.starts_with("slice.selection.out-of-bounds"),
        "C in [60, 64] always overflows a 64-extent axis: {error}"
    );
}

#[test]
fn changing_a_symbol_binding_source_moves_the_fifth_semantic_subject() {
    let axes = [symbolic_window("c", 6), WHOLE];
    let as_input = env_with(&[("c", input_binding("tokens", 0))], &[interval("c", 4, 4)]);
    let as_static = env_with(&[("c", static_binding(4))], &[interval("c", 4, 4)]);

    let mut first = SemanticProgramBuilder::try_standard_with_shape_environment(as_input)
        .expect("the standard builder opens");
    let table = first
        .input::<crate::semantic::F32>(
            InputKey::new("table").expect("a valid key"),
            Shape::from_dims([64, 128]),
        )
        .unwrap();
    let selected = F32Slice::apply(
        &mut first,
        &SliceSelection::new(axes.iter().cloned()).unwrap(),
        table,
    )
    .unwrap();
    first
        .output(OutputKey::new("rows").expect("a valid key"), selected)
        .unwrap();
    let input_program = first.build().unwrap();

    let mut second = SemanticProgramBuilder::try_standard_with_shape_environment(as_static)
        .expect("the standard builder opens");
    let table = second
        .input::<crate::semantic::F32>(
            InputKey::new("table").expect("a valid key"),
            Shape::from_dims([64, 128]),
        )
        .unwrap();
    let selected = F32Slice::apply(
        &mut second,
        &SliceSelection::new(axes.iter().cloned()).unwrap(),
        table,
    )
    .unwrap();
    second
        .output(OutputKey::new("rows").expect("a valid key"), selected)
        .unwrap();
    let static_program = second.build().unwrap();

    assert_eq!(
        input_program.semantic_identity().graph(),
        static_program.semantic_identity().graph(),
        "the graph names the same symbol spelling"
    );
    assert_ne!(
        input_program.semantic_identity().shape_environment(),
        static_program.semantic_identity().shape_environment(),
        "changing the root binding moves the fifth semantic subject"
    );
}

#[test]
fn a_static_bytes_offset_is_not_a_second_literal_spelling() {
    let mut bytes = Vec::new();
    static_extent(4).encode(&mut bytes);
    let disguised = CanonicalValue::record([CanonicalField::new(
        SLICE_SELECTION_AXES,
        CanonicalValue::sequence([CanonicalValue::record([
            CanonicalField::new(
                SLICE_AXIS_RELATION,
                CanonicalValue::utf8(SLICE_RELATION_WINDOW).expect("a test name"),
            ),
            CanonicalField::new(
                SLICE_AXIS_OFFSET,
                CanonicalValue::bytes_owned(bytes).expect("a test payload"),
            ),
            CanonicalField::new(SLICE_AXIS_EXTENT, CanonicalValue::unsigned_u64(6)),
        ])
        .expect("a test axis")])
        .expect("a test sequence"),
    )])
    .expect("a test record");
    assert_eq!(
        SliceSelection::from_canonical_value(&disguised),
        Err(SliceSelectionError::MalformedAttribute {
            subject: SliceAttributeSubject::Offset,
        })
    );
}
