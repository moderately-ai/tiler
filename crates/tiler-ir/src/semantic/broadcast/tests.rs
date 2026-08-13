use super::*;
use crate::semantic::{
    F32Broadcast, F32Reindex, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
    RegistryError, ReindexForm, SemanticProgramBuilder,
};
use crate::shape::{
    BindingSource, EXTENT_PHASE_CEILING, ExtentRelation, ExtentTerm, FactProvenance, RootBinding,
    Shape, ShapeEnv, ShapeEnvBuilder, ShapeSymbol, SourcedExtent, SymbolScope,
};
use std::sync::Arc;

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn from_operand(value: u32) -> BroadcastAxisSource {
    BroadcastAxisSource::FromOperand(axis(value))
}

fn stretch_unit(value: u32) -> BroadcastAxisSource {
    BroadcastAxisSource::StretchUnit(axis(value))
}

const REPLICATE: BroadcastAxisSource = BroadcastAxisSource::Replicate;

fn mapping(result: &[u64], sources: &[BroadcastAxisSource]) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        result.iter().copied().map(Extent::new),
        sources.iter().copied(),
    )
    .expect("a test mapping is admitted")
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(
        F32::resolved_type(),
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn attributes(mapping: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(
        BROADCAST_AXIS_MAPPING_ATTRIBUTE,
        mapping,
    )])
    .expect("a test attribute record is canonical")
}

fn sourced_extent_value(extent: &SourcedExtent) -> CanonicalValue {
    let mut bytes = Vec::new();
    extent.encode(&mut bytes);
    CanonicalValue::bytes_owned(bytes).expect("a test sourced extent is bounded")
}

/// Builds a mapping record from raw parts, bypassing every constructor check.
///
/// This is how a frontend that hand-assembles the canonical attribute reaches
/// the registered inference routine, so a rule decided at construction is still
/// demonstrated firing as a provider diagnostic.
fn raw_mapping(result: &[u64], sources: Vec<CanonicalValue>) -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            BROADCAST_MAPPING_RESULT_EXTENTS,
            CanonicalValue::sequence(
                result.iter().copied().map(|extent| {
                    sourced_extent_value(&SourcedExtent::Static(Extent::new(extent)))
                }),
            )
            .expect("a test extent sequence is bounded"),
        ),
        CanonicalField::new(
            BROADCAST_MAPPING_SOURCES,
            CanonicalValue::sequence(sources).expect("a test source sequence is bounded"),
        ),
    ])
    .expect("a test mapping record is canonical")
}

fn raw_mapping_extents(result: &[SourcedExtent], sources: Vec<CanonicalValue>) -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            BROADCAST_MAPPING_RESULT_EXTENTS,
            CanonicalValue::sequence(result.iter().map(sourced_extent_value))
                .expect("a test extent sequence is bounded"),
        ),
        CanonicalField::new(
            BROADCAST_MAPPING_SOURCES,
            CanonicalValue::sequence(sources).expect("a test source sequence is bounded"),
        ),
    ])
    .expect("a test mapping record is canonical")
}

fn raw_source(relation: &str, operand_axis: Option<u32>) -> CanonicalValue {
    let name = CanonicalField::new(
        BROADCAST_SOURCE_RELATION,
        CanonicalValue::utf8(relation).expect("a test relation name is bounded"),
    );
    match operand_axis {
        None => CanonicalValue::record([name]),
        Some(value) => CanonicalValue::record([
            name,
            CanonicalField::new(BROADCAST_SOURCE_AXIS, CanonicalValue::unsigned_u32(value)),
        ]),
    }
    .expect("a test source record is canonical")
}

fn infer(operand: &[u64], mapping: CanonicalValue) -> Result<Vec<ValueFact>, RegistryError> {
    FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(
            &broadcast_f32_op(),
            &[f32_operand(operand)],
            &attributes(mapping),
        )
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operand: &[u64], mapping: CanonicalValue) -> String {
    let error = infer(operand, mapping).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a broadcast refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Returns the complete diagnostic message of a refused application.
fn refusal_message(operand: &[u64], mapping: CanonicalValue) -> String {
    let error = infer(operand, mapping).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a broadcast refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().message().to_owned()
}

fn result_shape(operand: &[u64], mapping: &BroadcastAxisMapping) -> Shape {
    let results =
        infer(operand, mapping.canonical_value().clone()).expect("the mapping is admitted");
    let [result] = results.as_slice() else {
        panic!("a broadcast has one result");
    };
    assert_eq!(result.resolved_type(), &F32::resolved_type());
    result
        .shape()
        .as_static()
        .expect("this family infers a literal boundary")
        .clone()
}

// --- Every occurrence class the pinned workload contains ---------------------

/// The mappings and ranks this module's positive evidence covers.
///
/// Three relation kinds over ranks one through four, at the exact extents the
/// pinned `Qwen/Qwen3-0.6B-Base` C1 profile needs: the two RMS-normalization
/// weight shapes, the rotary `cos`/`sin` tables, the causal mask, and the rotary
/// sign operand — which is the workload's only `stretch-unit` occurrence.
/// Nothing here covers a symbolic extent, a rank above four, or any dtype but
/// F32.
#[test]
fn every_workload_occurrence_class_derives_its_result_shape() {
    // The RMS-normalization weight, `[1024]` against `[T, 1024]`: one rank pad.
    assert_eq!(
        result_shape(
            &[1024],
            &mapping(&[10, 1024], &[REPLICATE, from_operand(0)])
        ),
        Shape::from_dims([10, 1024])
    );
    // The per-head normalization weight, `[128]` against `[T, 16, 128]`: two.
    assert_eq!(
        result_shape(
            &[128],
            &mapping(&[10, 16, 128], &[REPLICATE, REPLICATE, from_operand(0)])
        ),
        Shape::from_dims([10, 16, 128])
    );
    // The rotary tables, `[T, 128]` against `[T, 16, 128]`: a rank pad between
    // two one-to-one correspondences, which is exactly the reference's
    // `unsqueeze(1)`.
    assert_eq!(
        result_shape(
            &[10, 128],
            &mapping(
                &[10, 16, 128],
                &[from_operand(0), REPLICATE, from_operand(1)]
            )
        ),
        Shape::from_dims([10, 16, 128])
    );
    // The causal mask, `[T, S]` against `[8, 2, T, S]`: two leading rank pads.
    assert_eq!(
        result_shape(
            &[10, 10],
            &mapping(
                &[8, 2, 10, 10],
                &[REPLICATE, REPLICATE, from_operand(0), from_operand(1)]
            )
        ),
        Shape::from_dims([8, 2, 10, 10])
    );
    // The rotary sign operand, `[2, 1]` against `[T, 16, 2, 64]`: the workload's
    // only unit stretch, and the case that makes the two many-to-one relations
    // separately load-bearing.
    assert_eq!(
        result_shape(
            &[2, 1],
            &mapping(
                &[10, 16, 2, 64],
                &[REPLICATE, REPLICATE, from_operand(0), stretch_unit(1)]
            )
        ),
        Shape::from_dims([10, 16, 2, 64])
    );
}

#[test]
fn a_rank_pad_and_a_unit_stretch_are_different_mappings_of_different_operands() {
    // `[2]` padded to `[2, 64]` and `[2, 1]` stretched to `[2, 64]` produce the
    // same result shape from different operands under different relations. A
    // mapping vocabulary that folded the two together could not tell them apart.
    let padded = mapping(&[2, 64], &[from_operand(0), REPLICATE]);
    let stretched = mapping(&[2, 64], &[from_operand(0), stretch_unit(1)]);
    assert_eq!(result_shape(&[2], &padded), Shape::from_dims([2, 64]));
    assert_eq!(result_shape(&[2, 1], &stretched), Shape::from_dims([2, 64]));
    assert_ne!(padded.canonical_encoding(), stretched.canonical_encoding());
    // And each refuses the other's operand, so the relation is not decoration.
    assert_eq!(
        refusal(&[2, 1], padded.canonical_value().clone()),
        "broadcast.mapping.operand-axes-unconsumed"
    );
    assert_eq!(
        refusal(&[2], stretched.canonical_value().clone()),
        "broadcast.mapping.operand-axes-unconsumed"
    );
}

#[test]
fn the_authoring_facade_admits_a_mapping_through_the_governed_path() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let weight = builder
        .input::<F32>(
            InputKey::new("weight").expect("a valid key"),
            Shape::from_dims([1024]),
        )
        .expect("an F32 input");
    let axis_mapping = mapping(&[10, 1024], &[REPLICATE, from_operand(0)]);
    let widened =
        F32Broadcast::apply(&mut builder, &axis_mapping, weight).expect("the mapping is admitted");
    builder
        .output(OutputKey::new("widened").expect("a valid key"), widened)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    assert_eq!(program.operation_count(), 1);
    let occurrence = program
        .operations()
        .find(|operation| operation.key() == &broadcast_f32_op())
        .expect("the broadcast occurrence");
    assert_eq!(
        occurrence.attributes().canonical_encoding(),
        attributes(axis_mapping.canonical_value().clone()).canonical_encoding()
    );
}

// --- The named admission rules, each under its own name ---------------------

#[test]
fn an_implicit_rank_pad_is_refused_rather_than_filled_in() {
    // The `[1024] -> [T, 1024]` normalization broadcast written against the
    // operand's rank rather than the result's: one source for two result axes.
    // Nothing infers the missing entry, because inferring it is exactly the
    // implicit broadcasting this family exists to remove.
    assert_eq!(
        refusal(
            &[1024],
            raw_mapping(
                &[10, 1024],
                vec![raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0))]
            )
        ),
        "broadcast.mapping.source-count"
    );
    let message = refusal_message(
        &[1024],
        raw_mapping(
            &[10, 1024],
            vec![raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0))],
        ),
    );
    assert!(
        message.contains("does not account for every result axis"),
        "the refusal names totality rather than a shape disagreement: {message}"
    );
}

#[test]
fn an_extent_one_stretch_presented_without_its_axis_mapping_is_refused() {
    // The rotary sign operand `[2, 1]` against the `[T, 16, 2, 64]` result, with
    // the innermost axis stated as an ordinary one-to-one correspondence instead
    // of a stretch. The mapping never says to stretch, so the family refuses
    // rather than stretching.
    let presented = || {
        raw_mapping(
            &[10, 16, 2, 64],
            vec![
                raw_source(BROADCAST_RELATION_REPLICATE, None),
                raw_source(BROADCAST_RELATION_REPLICATE, None),
                raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(1)),
            ],
        )
    };
    assert_eq!(
        refusal(&[2, 1], presented()),
        "broadcast.mapping.extent-disagreement"
    );
    let message = refusal_message(&[2, 1], presented());
    assert!(
        message.contains(BROADCAST_RELATION_STRETCH_UNIT),
        "and the refusal names the relation the mapping should have stated: {message}"
    );

    // Without the two rank pads the same misspelling states no many-to-one
    // relation at all, and the earlier rule fires instead. Both refusals are
    // correct; which one a caller sees depends on whether the rest of the
    // mapping still denotes a broadcast.
    assert_eq!(
        refusal(
            &[2, 1],
            raw_mapping(
                &[2, 64],
                vec![
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(1)),
                ]
            )
        ),
        "broadcast.mapping.no-many-to-one-relation"
    );
}

#[test]
fn a_stretch_of_an_axis_whose_extent_is_not_one_is_refused() {
    assert_eq!(
        refusal(
            &[2, 4],
            raw_mapping(
                &[2, 64],
                vec![
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                    raw_source(BROADCAST_RELATION_STRETCH_UNIT, Some(1)),
                ]
            )
        ),
        "broadcast.mapping.stretch-source-not-unit"
    );
}

#[test]
fn a_many_to_one_relation_that_does_not_widen_is_refused() {
    // A result axis of extent one duplicates nothing, so neither a stretch nor a
    // replication of it is a many-to-one relation. Refusing keeps one relation to
    // one spelling: an extent-one result axis with an operand behind it is
    // `from-operand`, and one without is a reindex's unit-axis insertion.
    assert!(matches!(
        BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(1)],
            [from_operand(0), REPLICATE]
        ),
        Err(BroadcastMappingError::RelationDoesNotWiden { .. })
    ));
    assert!(matches!(
        BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(1)],
            [from_operand(0), stretch_unit(1)]
        ),
        Err(BroadcastMappingError::RelationDoesNotWiden { .. })
    ));
    // An empty result axis duplicates nothing either.
    assert!(matches!(
        BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(0)],
            [from_operand(0), REPLICATE]
        ),
        Err(BroadcastMappingError::RelationDoesNotWiden { .. })
    ));
    assert_eq!(
        refusal(
            &[2],
            raw_mapping(
                &[2, 1],
                vec![
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                    raw_source(BROADCAST_RELATION_REPLICATE, None),
                ]
            )
        ),
        "broadcast.mapping.relation-does-not-widen"
    );
}

#[test]
fn a_mapping_of_only_one_to_one_correspondences_denotes_no_broadcast() {
    assert!(matches!(
        BroadcastAxisMapping::new(
            [Extent::new(10), Extent::new(1024)],
            [from_operand(0), from_operand(1)]
        ),
        Err(BroadcastMappingError::NoManyToOneRelation)
    ));
    assert_eq!(
        refusal(
            &[10, 1024],
            raw_mapping(
                &[10, 1024],
                vec![
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(1)),
                ]
            )
        ),
        "broadcast.mapping.no-many-to-one-relation"
    );
}

#[test]
fn a_mapping_that_reorders_or_drops_an_operand_axis_is_refused_by_name() {
    // Reordering is a reindex composed with a broadcast, and the refusal says so
    // rather than reporting an extent disagreement further down.
    assert!(matches!(
        BroadcastAxisMapping::new(
            [Extent::new(10), Extent::new(16), Extent::new(128)],
            [from_operand(1), REPLICATE, from_operand(0)]
        ),
        Err(BroadcastMappingError::OperandAxisOutOfOrder { .. })
    ));
    assert_eq!(
        refusal(
            &[10, 128],
            raw_mapping(
                &[10, 16, 128],
                vec![
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(1)),
                    raw_source(BROADCAST_RELATION_REPLICATE, None),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                ]
            )
        ),
        "broadcast.mapping.operand-axis-out-of-order"
    );
    // Dropping an operand axis is a reduction or a slice. The mapping below is
    // well formed on its own and consumes one axis of a rank-two operand.
    let dropping = || {
        raw_mapping(
            &[10, 16],
            vec![
                raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                raw_source(BROADCAST_RELATION_REPLICATE, None),
            ],
        )
    };
    assert_eq!(
        refusal(&[10, 128], dropping()),
        "broadcast.mapping.operand-axes-unconsumed"
    );
    let message = refusal_message(&[10, 128], dropping());
    assert!(
        message.contains("reduction or a slice"),
        "the refusal names the families a dropped axis would belong to: {message}"
    );
}

#[test]
fn a_one_to_one_correspondence_that_disagrees_on_its_extent_is_refused() {
    let disagreeing = || {
        raw_mapping(
            &[10, 16, 1024],
            vec![
                raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                raw_source(BROADCAST_RELATION_REPLICATE, None),
                raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(1)),
            ],
        )
    };
    assert_eq!(
        refusal(&[10, 512], disagreeing()),
        "broadcast.mapping.extent-disagreement"
    );
    let message = refusal_message(&[10, 512], disagreeing());
    assert!(
        message.contains("512") && message.contains("1024"),
        "the refusal names both the declared extent and the operand's: {message}"
    );
}

// --- Malformed attributes ---------------------------------------------------

#[test]
fn a_malformed_mapping_attribute_is_refused_under_its_own_subject() {
    assert_eq!(
        refusal(
            &[1024],
            CanonicalValue::record([CanonicalField::new(
                BROADCAST_MAPPING_SOURCES,
                CanonicalValue::boolean(true),
            )])
            .expect("a test record is canonical")
        ),
        "broadcast.mapping.malformed-attribute"
    );
    // An unadmitted relation name is not a malformed record; it has its own rule.
    assert_eq!(
        refusal(
            &[1024],
            raw_mapping(
                &[10, 1024],
                vec![
                    raw_source("tile", None),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                ]
            )
        ),
        "broadcast.mapping.unadmitted-relation"
    );
    // A field set the relation does not use: a replication carrying an operand
    // axis, and a correspondence missing one.
    assert_eq!(
        refusal(
            &[1024],
            raw_mapping(
                &[10, 1024],
                vec![
                    raw_source(BROADCAST_RELATION_REPLICATE, Some(0)),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                ]
            )
        ),
        "broadcast.mapping.malformed-attribute"
    );
    assert_eq!(
        refusal(
            &[1024],
            raw_mapping(
                &[10, 1024],
                vec![
                    raw_source(BROADCAST_RELATION_REPLICATE, None),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, None),
                ]
            )
        ),
        "broadcast.mapping.malformed-attribute"
    );
    assert_eq!(
        BroadcastAttributeSubject::SourceRecord.to_string(),
        "source record"
    );
    assert_eq!(
        BroadcastAttributeSubject::ResultExtents.to_string(),
        "result-extent sequence"
    );
}

#[test]
fn a_broadcast_admits_exactly_one_f32_operand_and_produces_one_result() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let axis_mapping = mapping(&[10, 1024], &[REPLICATE, from_operand(0)]);
    let results = registry
        .infer_operation(
            &broadcast_f32_op(),
            &[f32_operand(&[1024])],
            &attributes(axis_mapping.canonical_value().clone()),
        )
        .expect("the mapping is admitted");
    assert_eq!(results.len(), 1);
    // A missing mapping attribute is refused by the schema, before this family's
    // own rules are consulted.
    let error = registry
        .infer_operation(
            &broadcast_f32_op(),
            &[f32_operand(&[1024])],
            &OperationAttributes::empty(),
        )
        .expect_err("an empty attribute record is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("the refusal is provider-attributed");
    };
    assert_eq!(
        rejection.source_error().code().as_str(),
        "tiler.schema.missing-attribute"
    );
}

// --- Canonical identity -----------------------------------------------------

#[test]
fn the_canonical_encoder_separates_mappings_a_relation_free_encoding_would_collide() {
    let padded = mapping(&[2, 64], &[from_operand(0), REPLICATE]);
    let stretched = mapping(&[2, 64], &[from_operand(0), stretch_unit(1)]);
    assert_ne!(
        padded.canonical_encoding(),
        stretched.canonical_encoding(),
        "the shipped encoder separates a rank pad from a unit stretch"
    );

    // Perturbation: encode the result extents alone, dropping the source
    // sequence. The two mappings above then collide exactly — both are `[2, 64]`
    // — even though they consume different operands under different relations,
    // which is the identity collision the source sequence exists to prevent.
    // This assertion is a property of a deliberately broken twin, not of the
    // shipped encoder, which is what makes the separation above load-bearing.
    assert_eq!(
        extents_only_encoding(&padded),
        extents_only_encoding(&stretched),
        "without the source sequence two distinct mappings share one identity"
    );

    // Two mappings differing only in *where* the rank pad sits are also distinct,
    // and would collide under any encoding that recorded a count rather than an
    // ordered sequence.
    let leading = mapping(
        &[16, 10, 128],
        &[REPLICATE, from_operand(0), from_operand(1)],
    );
    let middle = mapping(
        &[10, 16, 128],
        &[from_operand(0), REPLICATE, from_operand(1)],
    );
    assert_ne!(leading.canonical_encoding(), middle.canonical_encoding());

    // The encoding is domain-separated, so a mapping's bytes cannot be mistaken
    // for another canonical subject's.
    assert!(
        padded
            .canonical_encoding()
            .as_bytes()
            .starts_with(&(BROADCAST_AXIS_MAPPING_DOMAIN.len() as u64).to_be_bytes())
    );
}

/// The shipped encoder with its source sequence removed.
fn extents_only_encoding(mapping: &BroadcastAxisMapping) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, BROADCAST_AXIS_MAPPING_DOMAIN);
    let CanonicalValueView::Record(fields) = mapping.canonical_value().view() else {
        panic!("a mapping is a record");
    };
    CanonicalValue::record([fields[0].clone()])
        .expect("a test record is canonical")
        .encode(&mut bytes);
    bytes
}

#[test]
fn a_decoded_mapping_round_trips_to_the_mapping_it_encodes() {
    for original in [
        mapping(&[10, 1024], &[REPLICATE, from_operand(0)]),
        mapping(
            &[10, 16, 128],
            &[from_operand(0), REPLICATE, from_operand(1)],
        ),
        mapping(
            &[10, 16, 2, 64],
            &[REPLICATE, REPLICATE, from_operand(0), stretch_unit(1)],
        ),
    ] {
        let decoded = BroadcastAxisMapping::from_canonical_value(original.canonical_value())
            .expect("a mapping this module encoded decodes");
        assert_eq!(decoded, original);
        assert_eq!(decoded.sources(), original.sources());
        assert_eq!(decoded.result_extents(), original.result_extents());
    }
}

// --- The semantic signature -------------------------------------------------

#[test]
fn the_semantic_signature_states_the_alias_policy_and_the_admitted_relations() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let facts = registry
        .operation_facts(&broadcast_f32_op())
        .expect("the broadcast is registered")
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
            BROADCAST_FACT_VALUE_BEHAVIOUR,
            "none-every-result-element-is-an-operand-element-unchanged",
        ),
        (
            BROADCAST_FACT_MAPPING_CLASS,
            "total-over-the-result-domain-and-many-to-one-onto-the-operand-domain",
        ),
        (
            BROADCAST_FACT_STORAGE_CLAIM,
            "none-no-replication-or-materialization-is-claimed-and-reads-may-alias",
        ),
        (
            BROADCAST_FACT_ADMITTED_RELATIONS,
            "from-operand,stretch-unit,replicate",
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
fn the_normative_reference_names_every_relation_and_the_two_neighbouring_families() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let definition = registry
        .operation_definition(&broadcast_f32_op())
        .expect("the broadcast is registered");
    let reference = definition.normative_definition().as_str();
    for relation in [
        BROADCAST_RELATION_FROM_OPERAND,
        BROADCAST_RELATION_STRETCH_UNIT,
        BROADCAST_RELATION_REPLICATE,
    ] {
        assert!(
            reference.contains(relation),
            "the normative reference names every admitted relation, and omits {relation}"
        );
    }
    assert!(
        reference.contains("reindex") && reference.contains("reduction or a slice"),
        "and it names the families a malformed mapping would belong to: {reference}"
    );
    assert!(
        reference.contains("reads may alias"),
        "the alias property a broadcast introduces is part of its definition: {reference}"
    );
}

#[test]
fn the_broadcast_declares_no_algebraic_capability() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    assert!(
        !registry
            .operation_definition(&broadcast_f32_op())
            .expect("the broadcast is registered")
            .algebraic_capabilities()
            .declares_ordered_associativity(),
        "a family that performs no arithmetic has no associativity to declare, and \
         a missing declaration is unknown rather than the inverse law"
    );
}

// --- Sourced v2 application -------------------------------------------------

fn scope() -> SymbolScope {
    SymbolScope::new("broadcast/0").unwrap()
}

fn sym(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(scope(), name).unwrap()
}

fn axis_binding(input: &str, axis: u32) -> RootBinding {
    RootBinding::new(
        BindingSource::InputDimension {
            input: InputKey::new(input).expect("a valid key"),
            axis: Axis::new(axis),
        },
        EXTENT_PHASE_CEILING,
        FactProvenance::RuntimeValidated,
    )
    .unwrap()
}

fn env_with(relations: &[ExtentRelation], names: &[&str]) -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    for (index, name) in names.iter().enumerate() {
        let declared = sym(name);
        draft.declare(declared.clone()).unwrap();
        draft
            .bind(
                &declared,
                axis_binding("operand", u32::try_from(index).unwrap()),
            )
            .unwrap();
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

fn equal_to(name: &str, value: u64) -> ExtentRelation {
    ExtentRelation::equal(ExtentTerm::Symbol(sym(name)), ExtentTerm::Constant(value))
}

fn apply_sourced(
    environment: Arc<ShapeEnv>,
    operand: Vec<SourcedExtent>,
    result: Vec<SourcedExtent>,
    sources: &[BroadcastAxisSource],
) -> Result<Vec<SourcedExtent>, String> {
    let mut builder = SemanticProgramBuilder::try_standard_with_shape_environment(environment)
        .expect("the standard builder opens");
    let input = builder
        .input_sourced::<crate::semantic::F32>(
            InputKey::new("operand").expect("a valid key"),
            operand,
        )
        .map_err(|error| format!("input refused: {error}"))?;
    let mapping = BroadcastAxisMapping::new(result, sources.iter().copied())
        .map_err(|error| format!("mapping refused: {error}"))?;
    let widened =
        F32Broadcast::apply(&mut builder, &mapping, input).map_err(|error| match error {
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
        .output(OutputKey::new("widened").expect("a valid key"), widened)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    let value = program.outputs().next().unwrap().value();
    Ok(program.shape(value).unwrap().extents().collect())
}

#[test]
fn a_literal_extent_one_still_fails_while_its_reindex_neighbour_succeeds() {
    assert!(matches!(
        BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(1)],
            [from_operand(0), REPLICATE]
        ),
        Err(BroadcastMappingError::RelationDoesNotWiden { .. })
    ));

    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let operand = builder
        .input::<crate::semantic::F32>(
            InputKey::new("operand").expect("a valid key"),
            Shape::from_dims([2]),
        )
        .expect("an F32 input");
    let inserted = F32Reindex::apply(
        &mut builder,
        &ReindexForm::insert_unit_axis(axis(1)).expect("a unit-axis insertion is admitted"),
        operand,
    )
    .expect("the reindex neighbour of a literal unit pad succeeds");
    builder
        .output(OutputKey::new("inserted").expect("a valid key"), inserted)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    assert_eq!(program.operation_count(), 1);
    assert_eq!(
        program
            .shape(program.outputs().next().unwrap().value())
            .unwrap()
            .as_static()
            .expect("the reindex result is literal"),
        &Shape::from_dims([2, 1])
    );
}

#[test]
fn symbolic_many_to_one_extents_are_admitted_when_proved_positive() {
    let t = SourcedExtent::Symbol(sym("t"));
    let n = SourcedExtent::Symbol(sym("n"));
    for relation in [interval("t", 1, 64), interval("t", 2, 64), equal_to("t", 1)] {
        let environment = env_with(&[relation, interval("n", 2, 64)], &["n", "t"]);
        let result = apply_sourced(
            environment,
            vec![n.clone()],
            vec![t.clone(), n.clone()],
            &[REPLICATE, from_operand(0)],
        )
        .expect("a positive symbolic widening is admitted");
        assert_eq!(result, vec![t.clone(), n.clone()]);
    }
}

#[test]
fn a_symbol_whose_lower_bound_is_zero_is_refused_before_graph_mutation() {
    let t = SourcedExtent::Symbol(sym("t"));
    let n = SourcedExtent::Symbol(sym("n"));
    let environment = env_with(&[interval("t", 0, 64), interval("n", 2, 64)], &["n", "t"]);
    let mut builder = SemanticProgramBuilder::try_standard_with_shape_environment(environment)
        .expect("the standard builder opens");
    let input = builder
        .input_sourced::<crate::semantic::F32>(
            InputKey::new("operand").expect("a valid key"),
            vec![n.clone()],
        )
        .expect("the operand is admitted");
    let mapping = BroadcastAxisMapping::new(vec![t, n], [REPLICATE, from_operand(0)])
        .expect("a symbolic mapping is context-free");
    let error = F32Broadcast::apply(&mut builder, &mapping, input)
        .expect_err("a possibly-zero widening is refused");
    let crate::semantic::BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication(
        rejection,
    )) = error
    else {
        panic!("the refusal is provider-attributed, not {error}");
    };
    assert_eq!(
        rejection.source_error().code().as_str(),
        "broadcast.mapping.extent-not-proved-positive"
    );
    builder
        .output(OutputKey::new("operand").expect("a valid key"), input)
        .expect("the refused apply left the input in place");
    let program = builder.build().expect("the draft still commits");
    assert_eq!(
        program.operation_count(),
        0,
        "a zero-lower-bound symbol is refused before an occurrence exists"
    );
}

#[test]
fn from_operand_and_stretch_unit_proofs_fail_independently() {
    let n = SourcedExtent::Symbol(sym("n"));
    let m = SourcedExtent::Symbol(sym("m"));
    let unit = SourcedExtent::Symbol(sym("unit"));

    let unequal = env_with(
        &[
            interval("n", 2, 64),
            interval("m", 2, 64),
            interval("t", 2, 64),
        ],
        &["n", "m", "t"],
    );
    let error = apply_sourced(
        unequal,
        vec![n.clone()],
        vec![m.clone(), SourcedExtent::Static(Extent::new(8))],
        &[from_operand(0), REPLICATE],
    )
    .expect_err("unrelated symbols are not proved equal");
    assert!(
        error.starts_with("broadcast.mapping.extent-not-proved-equal"),
        "FromOperand names unproved equality: {error}"
    );

    let not_unit = env_with(&[interval("n", 2, 64), interval("t", 2, 64)], &["n", "t"]);
    let error = apply_sourced(
        not_unit,
        vec![n.clone()],
        vec![SourcedExtent::Symbol(sym("t"))],
        &[stretch_unit(0)],
    )
    .expect_err("a non-unit symbol may not be stretched");
    assert!(
        error.starts_with("broadcast.mapping.stretch-source-not-proved-unit"),
        "StretchUnit names unproved unit: {error}"
    );

    let proved_equal = env_with(
        &[
            interval("n", 2, 64),
            ExtentRelation::equal(ExtentTerm::Symbol(sym("n")), ExtentTerm::Symbol(sym("m"))),
        ],
        &["n", "m"],
    );
    let result = apply_sourced(
        proved_equal,
        vec![n.clone()],
        vec![m.clone(), SourcedExtent::Static(Extent::new(8))],
        &[from_operand(0), REPLICATE],
    )
    .expect("an equality constraint makes FromOperand hold");
    assert_eq!(result, vec![m, SourcedExtent::Static(Extent::new(8))]);

    let proved_unit = env_with(&[equal_to("unit", 1), interval("t", 2, 64)], &["unit", "t"]);
    let result = apply_sourced(
        proved_unit,
        vec![unit],
        vec![SourcedExtent::Symbol(sym("t"))],
        &[stretch_unit(0)],
    )
    .expect("a determined unit may be stretched");
    assert_eq!(result, vec![SourcedExtent::Symbol(sym("t"))]);
}

#[test]
fn a_high_rank_mapping_reuses_one_semantic_solve() {
    let t = SourcedExtent::Symbol(sym("t"));
    let sources = vec![REPLICATE; MAX_BROADCAST_MAPPING_AXES];
    let result_extents = vec![t.clone(); MAX_BROADCAST_MAPPING_AXES];
    let (applied, census) = crate::shape::env::census::observe_all(|| {
        apply_sourced(
            env_with(&[interval("t", 1, 64)], &["t"]),
            Vec::new(),
            result_extents.clone(),
            &sources,
        )
        .expect("a rank-zero operand with positive symbolic pads is admitted")
    });
    assert_eq!(applied, result_extents);
    assert_eq!(
        census.semantic_closure, 1,
        "environment construction solves once; mapping application must not"
    );
    assert_eq!(census.guard_hypothesis, 0);
    assert!(
        census.positivity >= MAX_BROADCAST_MAPPING_AXES,
        "every symbolic many-to-one axis asks the retained summary: {}",
        census.positivity
    );
}

#[test]
fn undeclared_and_foreign_symbols_are_named_refusals() {
    let ghost = ShapeSymbol::new(SymbolScope::new("other").unwrap(), "ghost").unwrap();
    let environment = env_with(&[interval("n", 2, 64)], &["n"]);
    let error = apply_sourced(
        environment,
        vec![SourcedExtent::Symbol(sym("n"))],
        vec![
            SourcedExtent::Symbol(ghost),
            SourcedExtent::Symbol(sym("n")),
        ],
        &[REPLICATE, from_operand(0)],
    )
    .expect_err("a foreign symbol is refused");
    assert!(
        error.starts_with("broadcast.mapping.undeclared-symbol"),
        "foreign symbols are undeclared, not a shape mismatch: {error}"
    );

    assert_eq!(
        refusal(
            &[1024],
            raw_mapping_extents(
                &[
                    SourcedExtent::Symbol(sym("t")),
                    SourcedExtent::Static(Extent::new(1024))
                ],
                vec![
                    raw_source(BROADCAST_RELATION_REPLICATE, None),
                    raw_source(BROADCAST_RELATION_FROM_OPERAND, Some(0)),
                ]
            )
        ),
        "broadcast.mapping.undeclared-symbol"
    );
}

#[test]
fn the_old_governed_key_is_absent_from_the_standard_registry() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let old = OpKey::new("tiler", "broadcast-f32", 1).expect("the retired key is well formed");
    assert!(
        registry.operation_definition(&old).is_none(),
        "v1 is not retained as a compatibility registration"
    );
    assert!(registry.operation_definition(&broadcast_f32_op()).is_some());
    assert_eq!(broadcast_f32_op().semantic_version(), 2);
}
