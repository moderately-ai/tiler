use super::*;
use crate::semantic::{
    Bf16, FrozenSemanticRegistry, OperationAttributes, RegistryError, ValueFact,
};
use crate::shape::Shape;

/// The pinned workload's own source extent, `[vocab_size, hidden]`.
const VOCAB: u64 = 151_936;
const HIDDEN: u64 = 1_024;

fn shape(dims: &[u64]) -> Shape {
    Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded")
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(F32::resolved_type(), shape(dims))
}

fn index_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(gather_index_resolved_type(), shape(dims))
}

fn typed_operand(name: &str, dims: &[u64]) -> ValueFact {
    ValueFact::new(
        ResolvedValueType::nominal(TypeKey::new("tiler", name, 1).expect("a test key is valid")),
        shape(dims),
    )
}

fn attributes(axis: u32) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(
        GATHER_AXIS_ATTRIBUTE,
        CanonicalValue::unsigned_u32(axis),
    )])
    .expect("a test attribute record is canonical")
}

fn infer_with(
    operands: &[ValueFact],
    attributes: &OperationAttributes,
) -> Result<Vec<ValueFact>, RegistryError> {
    FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(&gather_f32_op(), operands, attributes)
}

fn infer(source: &[u64], index: &[u64], axis: u32) -> Result<Vec<ValueFact>, RegistryError> {
    infer_with(
        &[f32_operand(source), index_operand(index)],
        &attributes(axis),
    )
}

/// Returns the stable diagnostic code of a refused application.
fn refusal_of(operands: &[ValueFact], attributes: &OperationAttributes) -> String {
    let error = infer_with(operands, attributes).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a gather refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

fn refusal(source: &[u64], index: &[u64], axis: u32) -> String {
    refusal_of(
        &[f32_operand(source), index_operand(index)],
        &attributes(axis),
    )
}

fn result_shape(source: &[u64], index: &[u64], axis: u32) -> Shape {
    let results = infer(source, index, axis).expect("the occurrence is admitted");
    let [result] = results.as_slice() else {
        panic!("a gather has one result");
    };
    assert_eq!(
        result.resolved_type(),
        &F32::resolved_type(),
        "a gather result is binary32"
    );
    result
        .shape()
        .as_static()
        .expect("this family infers a literal boundary")
        .clone()
}

/// The pinned occurrence infers the shape the workload derivation states.
///
/// `[151936, 1024]` gathered on axis 0 by `[T]` yields `[T, 1024]`, at both
/// conformance extents the L2 record fixes — `T = 10` then `T = 1`.
#[test]
fn the_tied_embedding_lookup_infers_the_derived_result_shape() {
    assert_eq!(
        result_shape(&[VOCAB, HIDDEN], &[10], 0),
        shape(&[10, HIDDEN])
    );
    assert_eq!(result_shape(&[VOCAB, HIDDEN], &[1], 0), shape(&[1, HIDDEN]));
}

/// The result composes the index shape into the gathered axis's position.
///
/// Three cases separately: the gathered axis first, in the middle, and last, so
/// the rule is demonstrated as a composition rather than as a prefix rule that
/// happens to hold for the workload's own axis 0.
#[test]
fn the_result_composes_the_index_shape_into_the_gathered_axis_position() {
    assert_eq!(result_shape(&[4, 5, 6], &[2, 3], 0), shape(&[2, 3, 5, 6]));
    assert_eq!(result_shape(&[4, 5, 6], &[2, 3], 1), shape(&[4, 2, 3, 6]));
    assert_eq!(result_shape(&[4, 5, 6], &[2, 3], 2), shape(&[4, 5, 2, 3]));
}

/// A rank-zero index operand drops the gathered axis rather than being refused.
#[test]
fn a_rank_zero_index_operand_drops_the_gathered_axis() {
    assert_eq!(result_shape(&[4, 5, 6], &[], 1), shape(&[4, 6]));
}

/// An empty index operand is admitted and yields a result with no elements.
///
/// The prefill boundary can bind `T = 0`, so this is a pinned occurrence rather
/// than a hypothetical one. It is admitted for the reason `Concatenate`'s
/// zero-extent rule states: the family decides the occurrence a caller wrote,
/// and an empty operand is a shape the program already had.
#[test]
fn an_empty_index_operand_is_admitted_and_yields_no_elements() {
    let result = result_shape(&[VOCAB, HIDDEN], &[0], 0);
    assert_eq!(result, shape(&[0, HIDDEN]));
    assert_eq!(result.element_count(), Some(0));
}

/// A rank-zero source has no axis to gather along and is refused.
#[test]
fn a_rank_zero_source_is_refused() {
    assert_eq!(refusal(&[], &[3], 0), "gather.source.rank-zero");
}

/// An axis outside the source's rank is refused, and by its own rule.
#[test]
fn an_axis_outside_the_source_rank_is_refused() {
    assert_eq!(refusal(&[4, 5], &[3], 2), "gather.axis.out-of-range");
    assert_eq!(refusal(&[4, 5], &[3], u32::MAX), "gather.axis.out-of-range");
}

/// A non-binary32 source is refused rather than promoted.
#[test]
fn a_non_binary32_source_is_refused_rather_than_promoted() {
    let operands = [
        ValueFact::new(Bf16::resolved_type(), shape(&[4, 5])),
        index_operand(&[3]),
    ];
    assert_eq!(
        refusal_of(&operands, &attributes(0)),
        "gather.source.implicit-promotion"
    );
}

/// Every governed signed integer index is refused under the reserved-convention
/// rule, not under the narrow-profile one.
///
/// The distinction is the whole point of the separate variant: a caller reaching
/// for negative indexing is told the convention is reserved rather than that the
/// type is unadmitted. All five signed identities are checked, so the refusal is
/// a property of signedness rather than of `i32` specifically.
#[test]
fn every_signed_index_identity_is_refused_under_the_reserved_convention_rule() {
    for name in ["i4", "i8", "i16", "i32", "i64"] {
        let operands = [f32_operand(&[4, 5]), typed_operand(name, &[3])];
        assert_eq!(
            refusal_of(&operands, &attributes(0)),
            "gather.index.signed-unsupported",
            "{name} must be refused as a reserved convention"
        );
    }
}

/// An unsigned index identity that is not `tiler::u32@1` is refused by name.
///
/// `u8` and `u16` are separately meaningful: the pinned vocabulary is 151,936,
/// which needs eighteen bits, so neither could carry a token ID even if the
/// family admitted them.
#[test]
fn an_unadmitted_unsigned_index_identity_is_refused_by_name() {
    for name in ["u8", "u16", "u64"] {
        let operands = [f32_operand(&[4, 5]), typed_operand(name, &[3])];
        assert_eq!(
            refusal_of(&operands, &attributes(0)),
            "gather.index.unadmitted-type",
            "{name} must be refused as an unadmitted index identity"
        );
    }
}

/// A binary32 index operand is refused, so a float coordinate cannot enter.
#[test]
fn a_binary32_index_operand_is_refused() {
    let operands = [f32_operand(&[4, 5]), f32_operand(&[3])];
    assert_eq!(
        refusal_of(&operands, &attributes(0)),
        "gather.index.unadmitted-type"
    );
}

/// The occurrence takes exactly two operands, and the *schema* decides it.
///
/// The registered `OperationArity::exact(2)` refuses before the inferencer runs,
/// so the code is the schema's rather than this family's. That is asserted here
/// rather than worked around: a family-shaped code would be evidence the arity
/// rule had moved out of the schema, which is where every other family's lives.
/// [`GatherError::OperandCount`] survives as the inferencer's own defensive
/// destructuring arm and is unreachable through this path.
#[test]
fn the_schema_refuses_an_operand_count_that_is_not_two() {
    assert_eq!(
        refusal_of(&[f32_operand(&[4, 5])], &attributes(0)),
        "tiler.schema.operand-arity"
    );
    assert_eq!(
        refusal_of(
            &[
                f32_operand(&[4, 5]),
                index_operand(&[3]),
                index_operand(&[3])
            ],
            &attributes(0)
        ),
        "tiler.schema.operand-arity"
    );
}

/// A malformed axis attribute is refused under its own rule.
///
/// The attribute's rule is decided before anything about the operands, which is
/// demonstrated by supplying a malformed attribute *and* a rank-zero source: the
/// attribute rule wins.
#[test]
fn a_malformed_axis_attribute_is_refused_before_the_operands_are_read() {
    let malformed = OperationAttributes::new([CanonicalField::new(
        GATHER_AXIS_ATTRIBUTE,
        CanonicalValue::unsigned_u64(0),
    )])
    .expect("a test attribute record is canonical");
    assert_eq!(
        refusal_of(&[f32_operand(&[]), index_operand(&[3])], &malformed),
        "gather.axis.malformed-attribute"
    );
}

/// The gathered-axis attribute is required, and the *schema* decides that too.
///
/// Declared `OperationAttributeSchema::required`, so an occurrence carrying no
/// attribute is refused before the inferencer runs. The family's own
/// `"gather.attributes"` code covers the case the schema cannot — an attribute
/// record carrying the required field *and* something else.
#[test]
fn the_schema_refuses_a_missing_gathered_axis_attribute() {
    assert_eq!(
        refusal_of(
            &[f32_operand(&[4, 5]), index_operand(&[3])],
            &OperationAttributes::empty()
        ),
        "tiler.schema.missing-attribute"
    );
}

/// The bounds rule refuses out of range and never clamps or wraps.
///
/// Three properties are perturbed separately because one assertion cannot
/// distinguish them: that the last in-range coordinate is admitted, that the
/// first out-of-range one is refused, and that a value far outside is refused
/// rather than reduced modulo the extent — which is what a wrapping convention
/// would return, and which would land back in range.
#[test]
fn an_out_of_range_index_is_refused_and_is_neither_clamped_nor_wrapped() {
    let extent = Extent::new(VOCAB);
    assert_eq!(
        decide_gather_index(0, VOCAB - 1, extent),
        Ok(usize::try_from(VOCAB - 1).expect("the host carries the extent"))
    );
    assert_eq!(
        decide_gather_index(7, VOCAB, extent),
        Err(GatherError::IndexOutOfBounds {
            position: 7,
            value: VOCAB,
            extent: VOCAB,
        })
    );
    // A wrapping convention would return `VOCAB + 3 - VOCAB == 3`, which is in
    // range; a clamping one would return `VOCAB - 1`. Both are admitted results
    // and this refusal is what distinguishes them from the stated posture.
    assert_eq!(
        decide_gather_index(0, VOCAB + 3, extent),
        Err(GatherError::IndexOutOfBounds {
            position: 0,
            value: VOCAB + 3,
            extent: VOCAB,
        })
    );
}

/// A zero-extent gathered axis admits no index at all.
#[test]
fn a_zero_extent_gathered_axis_admits_no_index() {
    assert!(decide_gather_index(0, 0, Extent::new(0)).is_err());
}

/// Every refusal carries its own diagnostic code, so no two rules share one.
///
/// Counted rather than spot-checked: two rules sharing a code would make a
/// caller unable to read which rule refused, which is the property the codes
/// exist to provide.
#[test]
fn every_refusal_rule_carries_a_distinct_diagnostic_code() {
    let errors = [
        GatherError::OperandCount { operands: 3 },
        GatherError::MalformedAxisAttribute,
        GatherError::SourceIsRankZero,
        GatherError::AxisOutOfRange {
            axis: Axis::new(9),
            rank: 2,
        },
        GatherError::SourceNotF32,
        GatherError::SignedIndexUnsupported,
        GatherError::UnadmittedIndexType,
        GatherError::IndexOutOfBounds {
            position: 0,
            value: 1,
            extent: 1,
        },
        GatherError::ResultShape(crate::shape::ShapeError::RankTooLarge { rank: 1, limit: 0 }),
    ];
    let mut codes: Vec<&str> = errors.iter().map(GatherError::diagnostic_code).collect();
    let population = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(
        codes.len(),
        population,
        "each of the {population} rules owns its own code"
    );
    for code in &codes {
        assert!(
            code.starts_with("gather."),
            "{code} is namespaced to this family"
        );
    }
}

/// Every refusal renders a message, and none is empty.
#[test]
fn every_refusal_renders_a_nonempty_message() {
    for error in [
        GatherError::SourceIsRankZero,
        GatherError::SourceNotF32,
        GatherError::SignedIndexUnsupported,
        GatherError::UnadmittedIndexType,
        GatherError::MalformedAxisAttribute,
    ] {
        assert!(!error.to_string().is_empty());
    }
}

/// The family is registered, and it registers no realization law.
///
/// Both halves are the claim. A key with no law is what makes a program stating
/// an occurrence fail closed at refinement rather than reach a plan, which is
/// the delivered maturity boundary: registered and reference-evaluated, not
/// lowered.
#[test]
fn the_family_is_registered_and_carries_no_realization_law() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let key = gather_f32_op();
    assert!(
        registry
            .operation_definitions()
            .any(|definition| definition.key() == &key),
        "the gather key is registered"
    );
    assert!(
        registry.index_realization_law(&key).is_none(),
        "no realization law is registered, so an occurrence fails closed at refinement"
    );
}

/// The normative definition names the rules a reader must be able to find.
///
/// Each clause is checked separately: a single "is nonempty" assertion would
/// pass for a definition that had lost the bounds posture.
#[test]
fn the_normative_definition_states_the_four_closure_rules() {
    let text = GATHER_F32_NORMATIVE_DEFINITION;
    for clause in [
        "never clamped",
        "never wrapped",
        "Duplicate indices are admitted",
        "duplicate-*write* rule is stated and not implemented",
        "Determinism",
        "tensor-data-derived",
    ] {
        assert!(text.contains(clause), "the definition states {clause:?}");
    }
}
