use super::*;
use crate::semantic::{
    F32TensorContraction, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
    RegistryError, SemanticProgramBuilder,
};
use crate::shape::Shape;

/// Frontend labels for the pinned workload's structure, deliberately not dense
/// and not ascending: canonical numbering is this module's job, not a caller's.
const T: ContractionIndex = ContractionIndex::new(19);
const D: ContractionIndex = ContractionIndex::new(3);
const O: ContractionIndex = ContractionIndex::new(14);

/// Dense ascending labels for the same structure spelled `ab,cb->ac`.
const A: ContractionIndex = ContractionIndex::new(0);
const B: ContractionIndex = ContractionIndex::new(1);
const C: ContractionIndex = ContractionIndex::new(2);

/// Frontend labels for the two attention structures, again not dense.
///
/// `G` is the key/value group, `R` the grouped-query repetition, `TQ` the query
/// position, `SK` the key position, and `DH` the head lane. Structure 2 is
/// `grtd,gsd->grts` and structure 3 is `grts,gsd->grtd`.
const G: ContractionIndex = ContractionIndex::new(70);
const R: ContractionIndex = ContractionIndex::new(71);
const TQ: ContractionIndex = ContractionIndex::new(72);
const SK: ContractionIndex = ContractionIndex::new(73);
const DH: ContractionIndex = ContractionIndex::new(74);

/// The C1 prefill row's extents, from the retained attention-block probe.
const C1_GROUPS: u64 = 8;
const C1_REPEATS: u64 = 2;
const C1_QUERY_POSITIONS: u64 = 10;
const C1_HEAD_DIM: u64 = 128;

/// The score structure, `grtd,gsd->grts`.
fn score_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new([vec![G, R, TQ, DH], vec![G, SK, DH]], [G, R, TQ, SK])
        .expect("grtd,gsd->grts is admitted")
}

/// The value structure, `grts,gsd->grtd`.
fn value_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new([vec![G, R, TQ, SK], vec![G, SK, DH]], [G, R, TQ, DH])
        .expect("grts,gsd->grtd is admitted")
}

fn index(label: u32) -> ContractionIndex {
    ContractionIndex::new(label)
}

/// The pinned workload's structure: `td,od->to`.
fn workload_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new([[T, D], [O, D]], [T, O]).expect("td,od->to is admitted")
}

/// Builds a structure attribute from raw labels, bypassing every check.
///
/// This is how a frontend that hand-assembles the canonical attribute reaches
/// the registered inference routine. Without it the five structural rules would
/// only ever be decidable through the typed constructor, and the requirement
/// that they fire as *provider diagnostics at construction* would be untested.
fn raw_structure(operands: &[&[u32]], output: &[u32], contracted: &[u32]) -> CanonicalValue {
    let tuple = |labels: &[u32]| {
        CanonicalValue::sequence(labels.iter().copied().map(CanonicalValue::unsigned_u32))
            .expect("a test tuple is bounded")
    };
    CanonicalValue::record([
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OPERAND_INDICES,
            CanonicalValue::sequence(operands.iter().copied().map(tuple))
                .expect("a test structure is bounded"),
        ),
        CanonicalField::new(CONTRACTION_STRUCTURE_OUTPUT_INDICES, tuple(output)),
        CanonicalField::new(CONTRACTION_STRUCTURE_CONTRACTED_INDICES, tuple(contracted)),
    ])
    .expect("a test structure record is canonical")
}

fn f32_operand(dims: &[u64]) -> ValueFact {
    ValueFact::new(
        F32::resolved_type(),
        Shape::try_from_dims(dims.iter().copied()).expect("a test shape is bounded"),
    )
}

fn attributes(structure: CanonicalValue) -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(
        CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
        structure,
    )])
    .expect("a test attribute record is canonical")
}

/// Runs the registered inference routine and returns its ordered result facts.
fn infer(
    operands: &[ValueFact],
    structure: CanonicalValue,
) -> Result<Vec<ValueFact>, RegistryError> {
    FrozenSemanticRegistry::standard()
        .expect("the standard registry builds")
        .infer_operation(
            &strict_tensor_contraction_f32_op(),
            operands,
            &attributes(structure),
        )
}

/// Returns the stable diagnostic code of a refused application.
fn refusal(operands: &[ValueFact], structure: CanonicalValue) -> String {
    let error = infer(operands, structure).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a contraction refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Returns the complete diagnostic message of a refused application.
fn refusal_message(operands: &[ValueFact], structure: CanonicalValue) -> String {
    let error = infer(operands, structure).expect_err("the application is refused");
    let RegistryError::RejectedOperationApplication(rejection) = error else {
        panic!("a contraction refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().message().to_owned()
}

#[test]
fn the_workload_structure_canonicalizes_exactly_as_the_l3_record_derives_it() {
    let structure = workload_structure();
    assert_eq!(
        structure.operand(0),
        Some([index(0), index(1)].as_slice()),
        "operand 0 of td,od->to is (0, 1)"
    );
    assert_eq!(
        structure.operand(1),
        Some([index(2), index(1)].as_slice()),
        "operand 1 of td,od->to is (2, 1): the contracted index is the LAST axis of both \
         operands, because the checkpoint stores every projection weight [out, in]"
    );
    assert_eq!(structure.output(), [index(0), index(2)].as_slice());
    assert_eq!(structure.contracted(), [index(1)].as_slice());
    assert_eq!(structure.operand_count(), 2);
}

#[test]
fn renaming_invariance_holds_and_the_ordinary_matmul_is_a_different_structure() {
    let workload = workload_structure();
    // `ab,cb->ac` spelled with dense ascending labels, and `ij,kj->ik` spelled
    // with a third set: three spellings, one structure, one identity.
    let ab_cb = ContractionIndexStructure::new([[A, B], [C, B]], [A, C]).expect("ab,cb->ac");
    let ij_kj = ContractionIndexStructure::new(
        [[index(7), index(8)], [index(9), index(8)]],
        [index(7), index(9)],
    )
    .expect("ij,kj->ik");
    assert_eq!(workload.canonical_encoding(), ab_cb.canonical_encoding());
    assert_eq!(workload.canonical_encoding(), ij_kj.canonical_encoding());

    // `td,do->to` is the ordinary `[M, K] x [K, N]` matmul. It is a *different*
    // structure, and the whole reason this profile exists is that the pinned
    // workload is not it.
    let ordinary_matmul =
        ContractionIndexStructure::new([[T, D], [D, O]], [T, O]).expect("td,do->to");
    assert_eq!(
        ordinary_matmul.operand(1),
        Some([index(1), index(2)].as_slice())
    );
    assert_ne!(
        workload.canonical_encoding(),
        ordinary_matmul.canonical_encoding()
    );

    // The encoding is domain-separated, so a structure's bytes cannot be
    // mistaken for another canonical subject's.
    assert!(
        workload
            .canonical_encoding()
            .as_bytes()
            .starts_with(&(CONTRACTION_INDEX_STRUCTURE_DOMAIN.len() as u64).to_be_bytes())
    );
}

/// The mutation proof ADR 0087 item 1 requires before the encoder is trusted.
///
/// Two perturbations, each demonstrated producing the exact defect it would
/// introduce. Neither assertion below is a property of the shipped encoder; each
/// is a property of a deliberately broken twin, which is what makes the shipped
/// encoder's own assertions load-bearing rather than incidental.
#[test]
fn the_canonical_encoder_is_mutation_proved_against_collision_and_double_encoding() {
    // `ab,cb->ac` and `abc,b->ac` are two admitted, structurally different
    // contractions. The first is the workload's projection; the second sums one
    // index of a rank-three operand against a rank-one operand.
    let ab_cb = ContractionIndexStructure::new([[A, B], [C, B]], [A, C]).expect("ab,cb->ac");
    let abc_b =
        ContractionIndexStructure::new([vec![A, B, C], vec![B]], [A, C]).expect("abc,b->ac");
    assert_ne!(
        ab_cb.canonical_encoding(),
        abc_b.canonical_encoding(),
        "the shipped encoder separates two distinct structures"
    );

    // Perturbation one: remove the per-operand framing, so the operand tuples
    // become one flat index run. The two structures then encode *equally* —
    // both flatten to 0, 1, 2, 1 with output (0, 2) and contracted {1} — which
    // is the identity collision the framing exists to prevent.
    assert_eq!(
        flattened_operand_encoding(&ab_cb),
        flattened_operand_encoding(&abc_b),
        "without operand framing two distinct structures collide"
    );

    // Perturbation two: remove the canonical renumbering, so a structure is
    // encoded under whatever labels a frontend chose. One structure then
    // encodes two ways, which is the other half of the same defect.
    assert_ne!(
        unrenumbered_encoding(
            &[&[T.get(), D.get()], &[O.get(), D.get()]],
            &[T.get(), O.get()]
        ),
        unrenumbered_encoding(
            &[&[A.get(), B.get()], &[C.get(), B.get()]],
            &[A.get(), C.get()]
        ),
        "without canonical renumbering one structure encodes two ways"
    );
    assert_eq!(
        ab_cb.canonical_encoding(),
        workload_structure().canonical_encoding(),
        "and the shipped encoder gives those two spellings one identity"
    );
}

/// The shipped encoder with its per-operand framing removed.
fn flattened_operand_encoding(structure: &ContractionIndexStructure) -> Vec<u8> {
    let flat: Vec<ContractionIndex> = structure.operands().flatten().copied().collect();
    let mut bytes = Vec::new();
    push_slice(&mut bytes, CONTRACTION_INDEX_STRUCTURE_DOMAIN);
    CanonicalValue::record([
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OPERAND_INDICES,
            index_sequence(&flat).expect("a test sequence is bounded"),
        ),
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OUTPUT_INDICES,
            index_sequence(structure.output()).expect("a test sequence is bounded"),
        ),
        CanonicalField::new(
            CONTRACTION_STRUCTURE_CONTRACTED_INDICES,
            index_sequence(structure.contracted()).expect("a test sequence is bounded"),
        ),
    ])
    .expect("a test record is canonical")
    .encode(&mut bytes);
    bytes
}

/// The shipped encoder with its canonical renumbering removed.
fn unrenumbered_encoding(operands: &[&[u32]], output: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, CONTRACTION_INDEX_STRUCTURE_DOMAIN);
    let contracted: Vec<u32> = {
        let mut summed: Vec<u32> = operands
            .iter()
            .flat_map(|tuple| tuple.iter().copied())
            .filter(|label| !output.contains(label))
            .collect();
        summed.sort_unstable();
        summed.dedup();
        summed
    };
    raw_structure(operands, output, &contracted).encode(&mut bytes);
    bytes
}

#[test]
fn the_admitted_profile_infers_its_result_shape_from_the_structure() {
    let results = infer(
        &[f32_operand(&[128, 1024]), f32_operand(&[3072, 1024])],
        workload_structure().canonical_value().clone(),
    )
    .expect("the B1-a prefill gate projection is admitted");
    let [result] = results.as_slice() else {
        panic!("a contraction has one result");
    };
    assert_eq!(result.resolved_type(), &F32::resolved_type());
    assert_eq!(
        result.shape().as_static(),
        Some(&Shape::from_dims([128, 3072]))
    );
}

#[test]
fn the_authoring_facade_admits_the_profile_through_the_governed_path() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").expect("a valid key"),
            Shape::from_dims([128, 1024]),
        )
        .expect("an F32 input");
    let weights = builder
        .input::<F32>(
            InputKey::new("weights").expect("a valid key"),
            Shape::from_dims([3072, 1024]),
        )
        .expect("an F32 input");
    let projected =
        F32TensorContraction::apply(&mut builder, &workload_structure(), activations, weights)
            .expect("the profile is admitted");
    builder
        .output(OutputKey::new("projected").expect("a valid key"), projected)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    assert_eq!(program.operation_count(), 1);
    let occurrence = program
        .operations()
        .find(|operation| operation.key() == &strict_tensor_contraction_f32_op())
        .expect("the contraction occurrence");
    // The occurrence carries the canonical structure, so two spellings of one
    // structure produce byte-identical attribute identity.
    assert_eq!(
        occurrence.attributes().canonical_encoding(),
        attributes(
            ContractionIndexStructure::new([[A, B], [C, B]], [A, C])
                .expect("ab,cb->ac")
                .canonical_value()
                .clone()
        )
        .canonical_encoding()
    );
}

// --- The five structural admission rules, each under its own name -----------

#[test]
fn rule_one_refuses_an_output_index_present_in_no_operand() {
    // `ab,cb->ad`: `d` is index 3 and appears in neither operand.
    let structure = raw_structure(&[&[0, 1], &[2, 1]], &[0, 3], &[1]);
    assert_eq!(
        refusal(&[f32_operand(&[4, 8]), f32_operand(&[6, 8])], structure),
        "contraction.rule.output-index-in-no-operand"
    );
}

#[test]
fn rule_two_refuses_a_summed_index_present_in_only_one_operand() {
    // `abc,eb->ae`: `c` is summed but only operand 0 names it, which is a
    // reduction of that operand rather than a contraction.
    let structure = raw_structure(&[&[0, 1, 2], &[3, 1]], &[0, 3], &[1, 2]);
    assert_eq!(
        refusal(&[f32_operand(&[4, 8, 5]), f32_operand(&[6, 8])], structure),
        "contraction.rule.summed-index-in-one-operand"
    );
}

#[test]
fn rule_three_refuses_an_index_repeated_within_one_operand() {
    // `aa,ba->ab` repeats `a` inside operand 0, which is a diagonal rather than
    // a contraction operand.
    let structure = raw_structure(&[&[0, 0], &[1, 0]], &[0, 1], &[]);
    assert_eq!(
        refusal(&[f32_operand(&[8, 8]), f32_operand(&[6, 8])], structure),
        "contraction.rule.index-repeated-within-operand"
    );
}

#[test]
fn rule_four_refuses_a_duplicated_output_index() {
    // `ab,cb->aa` names `a` twice in the output, so the output is not a
    // permutation of the free indices.
    let structure = raw_structure(&[&[0, 1], &[2, 1]], &[0, 0], &[1]);
    assert_eq!(
        refusal(&[f32_operand(&[4, 8]), f32_operand(&[6, 8])], structure),
        "contraction.rule.duplicate-output-index"
    );
}

#[test]
fn rule_five_refuses_an_index_present_in_more_than_two_operands() {
    // `ab,cb,db->acd` shares `b` across three operands. This is where the
    // reserved multi-operand answer lands; until it is decided the structure is
    // refused rather than approximated.
    let structure = raw_structure(&[&[0, 1], &[2, 1], &[3, 1]], &[0, 2, 3], &[1]);
    assert_eq!(
        refusal(&[f32_operand(&[4, 8]), f32_operand(&[6, 8])], structure),
        "contraction.rule.index-in-more-than-two-operands",
        "rule five is decided on the structure's own operand count, before the \
         occurrence's operand count, or it could never fire under an exact-arity schema"
    );
}

#[test]
fn every_structural_rule_is_reachable_through_the_typed_constructor_too() {
    // The same five refusals, from the constructor a frontend actually calls.
    // A rule reachable only through a hand-assembled attribute would leave the
    // ordinary authoring path unvalidated.
    assert!(matches!(
        ContractionIndexStructure::new([[A, B], [C, B]], [A, index(3)]),
        Err(ContractionStructureError::OutputIndexInNoOperand { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([vec![A, B, C], vec![index(3), B]], [A, index(3)]),
        Err(ContractionStructureError::SummedIndexInOneOperand { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([[A, A], [B, A]], [A, B]),
        Err(ContractionStructureError::IndexRepeatedWithinOperand { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([[A, B], [C, B]], [A, A]),
        Err(ContractionStructureError::DuplicateOutputIndex { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([[A, B], [C, B], [index(3), B]], [A, C, index(3)]),
        Err(ContractionStructureError::IndexInMoreThanTwoOperands { .. })
    ));
}

// --- Refusals that are not one of the five rules ---------------------------

#[test]
fn a_structure_that_sums_over_nothing_is_not_a_contraction() {
    // `a,b->ab` is an outer product: a different family with a different access
    // relation, refused rather than admitted under this key.
    assert!(matches!(
        ContractionIndexStructure::new([[A], [B]], [A, B]),
        Err(ContractionStructureError::NoContractedIndex)
    ));
    assert_eq!(
        refusal(
            &[f32_operand(&[4]), f32_operand(&[6])],
            raw_structure(&[&[0], &[1]], &[0, 1], &[])
        ),
        "contraction.structure.no-contracted-index"
    );
}

#[test]
fn a_declared_contracted_set_that_disagrees_with_the_derivation_is_refused() {
    assert_eq!(
        refusal(
            &[f32_operand(&[4, 8]), f32_operand(&[6, 8])],
            raw_structure(&[&[0, 1], &[2, 1]], &[0, 2], &[0, 1])
        ),
        "contraction.structure.contracted-set-mismatch"
    );
}

#[test]
fn a_second_numbering_of_one_structure_is_refused_rather_than_renumbered() {
    // The structure below is `td,od->to` spelled with the operand-1 index
    // numbered before the shared one. Admitting it would give one structure two
    // identities, which is exactly the collision the canonicalization prevents.
    assert_eq!(
        refusal(
            &[f32_operand(&[4, 8]), f32_operand(&[6, 8])],
            raw_structure(&[&[0, 2], &[1, 2]], &[0, 1], &[2])
        ),
        "contraction.structure.non-canonical-numbering"
    );
}

#[test]
fn a_structure_whose_operand_count_is_not_the_signature_is_refused() {
    // Two operand tuples are required because the schema admits exactly two
    // operands. A one-tuple structure passes every rule and still cannot
    // describe this occurrence.
    assert_eq!(
        refusal(
            &[f32_operand(&[4, 8]), f32_operand(&[6, 8])],
            raw_structure(&[&[0, 1], &[2, 1], &[3, 1]], &[0, 2, 3], &[1])
        ),
        "contraction.rule.index-in-more-than-two-operands"
    );
    // Four operands, no index shared by more than two: rule five passes and the
    // operand-count refusal is what is left.
    assert_eq!(
        refusal(
            &[f32_operand(&[4, 8]), f32_operand(&[6, 8])],
            raw_structure(
                &[&[0, 1], &[2, 1], &[3, 4], &[5, 4]],
                &[0, 2, 3, 5],
                &[1, 4]
            )
        ),
        "contraction.structure.operand-count"
    );
}

#[test]
fn a_malformed_structure_attribute_is_refused_under_its_own_subject() {
    let not_a_record = CanonicalValue::record([CanonicalField::new(
        CONTRACTION_STRUCTURE_OPERAND_INDICES,
        CanonicalValue::boolean(true),
    )])
    .expect("a test record is canonical");
    assert_eq!(
        refusal(&[f32_operand(&[4, 8]), f32_operand(&[6, 8])], not_a_record),
        "contraction.structure.malformed-attribute"
    );
    assert_eq!(
        ContractionAttributeSubject::OperandTuples.to_string(),
        "operand-tuple sequence"
    );
}

// --- Extent agreement, through the accepted three-outcome path -------------

#[test]
fn a_disproved_extent_equality_names_both_observed_sources() {
    let message = refusal_message(
        &[f32_operand(&[128, 1024]), f32_operand(&[3072, 512])],
        workload_structure().canonical_value().clone(),
    );
    assert!(
        message.contains("operand 0 axis 1") && message.contains("operand 1 axis 1"),
        "equality does not erase source identity, so a disproof names both \
         observed sources: {message}"
    );
    assert!(
        message.contains("1024") && message.contains("512"),
        "{message}"
    );
    assert_eq!(
        refusal(
            &[f32_operand(&[128, 1024]), f32_operand(&[3072, 512])],
            workload_structure().canonical_value().clone()
        ),
        "contraction.extent.disproved"
    );
}

#[test]
fn a_free_index_binds_its_extent_and_a_rank_disagreement_is_refused() {
    // A proved agreement admits the occurrence; the free indices are bound by
    // exactly one operand each and carry through to the result.
    assert!(
        infer(
            &[f32_operand(&[1, 1024]), f32_operand(&[151_936, 1024])],
            workload_structure().canonical_value().clone(),
        )
        .is_ok(),
        "the decode vocabulary projection is admitted"
    );
    assert_eq!(
        refusal(
            &[f32_operand(&[128, 1024, 2]), f32_operand(&[3072, 1024])],
            workload_structure().canonical_value().clone()
        ),
        "contraction.rank"
    );
}

#[test]
fn an_empty_contracted_domain_is_refused_because_the_fold_is_unseeded() {
    // `fl(+0.0 + x)` is not `x` at `x = -0.0`, so an unseeded strict fold and a
    // `+0.0`-seeded one are different operations. This family declares no seed,
    // so it has no empty result to return and refuses instead of inventing one.
    assert_eq!(
        refusal(
            &[f32_operand(&[128, 0]), f32_operand(&[3072, 0])],
            workload_structure().canonical_value().clone()
        ),
        "contraction.extent.empty-contracted-domain"
    );
    // A zero *free* extent is not the same case: the result is an empty tensor
    // and no reduction is performed.
    let results = infer(
        &[f32_operand(&[0, 1024]), f32_operand(&[3072, 1024])],
        workload_structure().canonical_value().clone(),
    )
    .expect("an empty free extent produces an empty result");
    assert_eq!(
        results[0].shape().as_static(),
        Some(&Shape::from_dims([0, 3072]))
    );
}

// --- The numerical signature ------------------------------------------------

#[test]
fn the_numerical_signature_is_complete_against_the_realization_record() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let facts = registry
        .operation_facts(&strict_tensor_contraction_f32_op())
        .expect("the contraction is registered")
        .value();
    let CanonicalValueView::Record(fields) = facts.view() else {
        panic!("the numerical signature is a record");
    };
    let read = |id: AttributeFieldId| {
        fields
            .iter()
            .find(|field| field.id() == id)
            .unwrap_or_else(|| panic!("field {id} is unconditional on this definition"))
            .value()
            .clone()
    };

    // Every row of the L3 realization record's reduction-contract table that is
    // a property of the operation rather than of a target.
    assert_eq!(
        read(CONTRACTION_F32_FACT_ACCUMULATOR_TYPE),
        CanonicalValue::value_type(F32::resolved_type())
    );
    assert_eq!(
        read(CONTRACTION_F32_FACT_RESULT_TYPE),
        CanonicalValue::value_type(F32::resolved_type())
    );
    assert_eq!(
        read(CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED),
        CanonicalValue::boolean(false)
    );
    assert_eq!(
        read(CONTRACTION_F32_FACT_PERMUTATION_PERMITTED),
        CanonicalValue::boolean(false)
    );
    assert_eq!(
        read(CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED),
        CanonicalValue::boolean(false),
        "ADR 0015's contraction is a separate, independently resolved permission, \
         and a *tensor* contraction does not grant it by virtue of its name"
    );
    assert_eq!(
        read(CONTRACTION_F32_FACT_CANONICAL_NAN_BITS),
        canonical_f32_bits(CANONICAL_F32_ARITHMETIC_NAN_BITS)
    );
    for (id, expected) in [
        (
            CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
            "binary32-operands-and-binary32-products",
        ),
        (
            CONTRACTION_F32_FACT_CONVERSION,
            "none-operands-products-accumulator-and-result-are-binary32",
        ),
        (
            CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE,
            "ascending-lexicographic-over-the-canonically-ordered-contracted-index-space",
        ),
        (
            CONTRACTION_F32_FACT_SEED,
            "none-the-accumulator-starts-at-the-first-product",
        ),
        (
            CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN,
            "refused-an-unseeded-fold-has-no-empty-result",
        ),
        (
            CONTRACTION_F32_FACT_DISTRIBUTIVITY,
            "absent-no-expressible-numerical-permission-grants-it",
        ),
        (
            CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
            "after-every-combine-and-at-the-result-boundary",
        ),
        (CONTRACTION_F32_FACT_DETERMINISM, "plan-deterministic"),
    ] {
        assert_eq!(
            read(id),
            CanonicalValue::utf8(expected).expect("a test fact is bounded")
        );
    }
    assert_eq!(
        fields.len(),
        14,
        "the signature has exactly the fourteen published fields, so a new one \
         cannot be added without moving this count and the identity behind it"
    );
}

// --- The two attention structures, at four-wide tuples and five indices ------

/// Both attention structures canonicalize exactly as first-appearance numbering
/// derives them, and they differ in one operand tuple's *order*.
///
/// The derivation is worth stating, because the pair is the sharpest one in the
/// workload: `grtd,gsd->grts` and `grts,gsd->grtd` agree on operand 0, on the
/// output, and on the contracted set, and disagree only on whether operand 1
/// reads `(g, s, d)` as `(0, 4, 3)` or `(0, 3, 4)`. An encoder insensitive to
/// position within an operand tuple would give the workload's two attention
/// contractions one identity.
#[test]
fn both_attention_structures_canonicalize_by_first_appearance() {
    let score = score_structure();
    assert_eq!(
        score.operand(0),
        Some([index(0), index(1), index(2), index(3)].as_slice()),
        "operand 0 of grtd,gsd->grts is (g, r, t, d)"
    );
    assert_eq!(
        score.operand(1),
        Some([index(0), index(4), index(3)].as_slice()),
        "operand 1 is (g, s, d): `s` is new at its first appearance and `d` is not"
    );
    assert_eq!(
        score.output(),
        [index(0), index(1), index(2), index(4)].as_slice()
    );
    assert_eq!(score.contracted(), [index(3)].as_slice(), "`d` is summed");

    let value = value_structure();
    assert_eq!(
        value.operand(0),
        Some([index(0), index(1), index(2), index(3)].as_slice()),
        "operand 0 of grts,gsd->grtd is (g, r, t, s)"
    );
    assert_eq!(
        value.operand(1),
        Some([index(0), index(3), index(4)].as_slice()),
        "operand 1 is (g, s, d): here `s` is the already-numbered one"
    );
    assert_eq!(
        value.output(),
        [index(0), index(1), index(2), index(4)].as_slice()
    );
    assert_eq!(value.contracted(), [index(3)].as_slice(), "`s` is summed");

    // Five distinct indices each, with a four-wide operand tuple and a four-wide
    // output tuple — against structure 1's three indices and two-wide tuples.
    assert_eq!(score.operand(0).expect("operand 0").len(), 4);
    assert_eq!(score.output().len(), 4);
    assert_eq!(value.operand(0).expect("operand 0").len(), 4);
    assert_eq!(value.output().len(), 4);
    for structure in [&score, &value] {
        let distinct: BTreeSet<ContractionIndex> = structure
            .operands()
            .flatten()
            .chain(structure.output())
            .copied()
            .collect();
        assert_eq!(distinct.len(), 5, "five distinct indices");
    }

    // Everything but operand 1 agrees, which is what makes the collision
    // perturbation below a real hazard rather than a contrived one.
    assert_eq!(score.operand(0), value.operand(0));
    assert_eq!(score.output(), value.output());
    assert_eq!(score.contracted(), value.contracted());
    assert_ne!(
        score.canonical_encoding(),
        value.canonical_encoding(),
        "the shipped encoder separates the score and value structures"
    );
}

#[test]
fn renaming_invariance_holds_at_five_indices() {
    // The score structure spelled with dense ascending labels, and with a third
    // disjoint set: three spellings, one structure, one identity.
    let dense = ContractionIndexStructure::new(
        [
            vec![index(0), index(1), index(2), index(3)],
            vec![index(0), index(4), index(3)],
        ],
        [index(0), index(1), index(2), index(4)],
    )
    .expect("the dense spelling of grtd,gsd->grts");
    let shifted = ContractionIndexStructure::new(
        [
            vec![index(900), index(901), index(902), index(903)],
            vec![index(900), index(904), index(903)],
        ],
        [index(900), index(901), index(902), index(904)],
    )
    .expect("a shifted spelling of grtd,gsd->grts");
    assert_eq!(
        score_structure().canonical_encoding(),
        dense.canonical_encoding()
    );
    assert_eq!(
        score_structure().canonical_encoding(),
        shifted.canonical_encoding()
    );
}

/// The mutation proof at four-wide tuples and five indices.
///
/// Three perturbations, each demonstrated producing the exact defect it would
/// introduce. As in the width-two proof above, no assertion here is a property
/// of the shipped encoder: each is a property of a deliberately broken twin.
#[test]
fn the_canonical_encoder_is_mutation_proved_at_four_and_five_indices() {
    let score = score_structure();
    let value = value_structure();

    // Perturbation one, and the one this width introduces: sort each operand
    // tuple before encoding. The score and value structures differ *only* in the
    // order of operand 1's last two indices, so an order-insensitive encoder
    // gives the workload's two attention contractions one identity — while the
    // shipped encoder separates them.
    assert_eq!(
        sorted_operand_encoding(&score),
        sorted_operand_encoding(&value),
        "with operand tuples sorted, grtd,gsd->grts and grts,gsd->grtd collide"
    );
    assert_ne!(score.canonical_encoding(), value.canonical_encoding());

    // Perturbation two: remove the per-operand framing. The width-two proof
    // above shows this collides `ab,cb->ac` with `abc,b->ac`; it still collides
    // at this width, over two admitted five-index structures whose flattened
    // index runs, outputs, and contracted sets are all identical and whose
    // operand framing is not.
    let split_after_three = ContractionIndexStructure::new(
        [
            vec![index(0), index(1), index(2)],
            vec![index(3), index(4), index(0), index(2)],
        ],
        [index(0), index(1), index(3), index(4)],
    )
    .expect("abc,dea c->abde is admitted");
    let split_after_four = ContractionIndexStructure::new(
        [
            vec![index(0), index(1), index(2), index(3)],
            vec![index(4), index(0), index(2)],
        ],
        [index(0), index(1), index(3), index(4)],
    )
    .expect("abcd,eac->abde is admitted");
    let flat: Vec<ContractionIndex> = split_after_three.operands().flatten().copied().collect();
    assert_eq!(
        flat,
        split_after_four
            .operands()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
        "the two structures flatten to one index run, so only the framing separates them"
    );
    assert_eq!(
        flattened_operand_encoding(&split_after_three),
        flattened_operand_encoding(&split_after_four),
        "without operand framing two distinct five-index structures collide"
    );
    assert_ne!(
        split_after_three.canonical_encoding(),
        split_after_four.canonical_encoding(),
        "the shipped encoder separates them"
    );

    // Perturbation three: remove the canonical renumbering, so a structure is
    // encoded under whatever labels a frontend chose. The score structure then
    // encodes two ways.
    assert_ne!(
        unrenumbered_encoding(
            &[
                &[G.get(), R.get(), TQ.get(), DH.get()],
                &[G.get(), SK.get(), DH.get()]
            ],
            &[G.get(), R.get(), TQ.get(), SK.get()]
        ),
        unrenumbered_encoding(&[&[0, 1, 2, 3], &[0, 4, 3]], &[0, 1, 2, 4]),
        "without canonical renumbering one five-index structure encodes two ways"
    );
}

/// The shipped encoder with each operand tuple sorted before encoding.
///
/// The perturbation this width introduces: at two-wide tuples the workload's
/// structures do not collide under it, and at four-wide tuples they do.
fn sorted_operand_encoding(structure: &ContractionIndexStructure) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, CONTRACTION_INDEX_STRUCTURE_DOMAIN);
    let tuples: Vec<CanonicalValue> = structure
        .operands()
        .map(|tuple| {
            let mut sorted = tuple.to_vec();
            sorted.sort_unstable();
            index_sequence(&sorted).expect("a test sequence is bounded")
        })
        .collect();
    CanonicalValue::record([
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OPERAND_INDICES,
            CanonicalValue::sequence(tuples).expect("a test sequence is bounded"),
        ),
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OUTPUT_INDICES,
            index_sequence(structure.output()).expect("a test sequence is bounded"),
        ),
        CanonicalField::new(
            CONTRACTION_STRUCTURE_CONTRACTED_INDICES,
            index_sequence(structure.contracted()).expect("a test sequence is bounded"),
        ),
    ])
    .expect("a test record is canonical")
    .encode(&mut bytes);
    bytes
}

/// The repetition index is free, in one operand and the output and nowhere else.
///
/// This is the first index in the workload whose access map drops it from an
/// operand, and it is the whole of what makes the eight-to-sixteen grouped-query
/// repetition free: the key operand has no `r` axis, so nothing materializes a
/// `[16, S, 128]` repeated key. Asserted rather than left to a reader of the
/// tuples, because the alternative spelling produces a correctly shaped result.
#[test]
fn the_grouped_query_repetition_index_is_free_in_one_operand_and_the_output() {
    let score = score_structure();
    let repetition = index(1);
    assert!(
        score.operand(0).expect("operand 0").contains(&repetition),
        "`r` is a query-operand index"
    );
    assert!(
        !score.operand(1).expect("operand 1").contains(&repetition),
        "`r` is absent from the key operand, which is what makes the repetition free"
    );
    assert!(
        score.output().contains(&repetition),
        "`r` is an output index"
    );
    assert!(
        !score.contracted().contains(&repetition),
        "`r` is free, not summed"
    );
    // The same index in the value structure, which drops it from the same operand.
    assert!(
        !value_structure()
            .operand(1)
            .expect("operand 1")
            .contains(&repetition)
    );

    // The result carries the repetition and the key operand does not, so the
    // occurrence verifies against a key that was never repeated.
    let results = infer(
        &[
            f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
            f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        ],
        score.canonical_value().clone(),
    )
    .expect("the C1 prefill score contraction is admitted");
    assert_eq!(
        results[0].shape().as_static(),
        Some(&Shape::from_dims([
            C1_GROUPS,
            C1_REPEATS,
            C1_QUERY_POSITIONS,
            C1_QUERY_POSITIONS
        ]))
    );

    // The broadcast spelling is what `r` avoids: a key operand carrying the
    // repetition holds twice the elements, and that materialization is exactly
    // the cost the free index removes.
    let repeated = Shape::from_dims([C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]);
    let unrepeated = Shape::from_dims([C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]);
    assert_eq!(unrepeated.element_count(), Some(10_240));
    assert_eq!(repeated.element_count(), Some(20_480));
}

#[test]
fn both_attention_structures_infer_their_c1_result_shapes() {
    // Structure 3 at the C1 prefill row, where `S` equals `T`.
    let results = infer(
        &[
            f32_operand(&[
                C1_GROUPS,
                C1_REPEATS,
                C1_QUERY_POSITIONS,
                C1_QUERY_POSITIONS,
            ]),
            f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        ],
        value_structure().canonical_value().clone(),
    )
    .expect("the C1 prefill value contraction is admitted");
    assert_eq!(
        results[0].shape().as_static(),
        Some(&Shape::from_dims([
            C1_GROUPS,
            C1_REPEATS,
            C1_QUERY_POSITIONS,
            C1_HEAD_DIM
        ]))
    );

    // Both structures reach the same admission through the authoring facade a
    // frontend actually calls, rather than only through a raw attribute.
    for (structure, left, right) in [
        (
            score_structure(),
            Shape::from_dims([C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
            Shape::from_dims([C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        ),
        (
            value_structure(),
            Shape::from_dims([
                C1_GROUPS,
                C1_REPEATS,
                C1_QUERY_POSITIONS,
                C1_QUERY_POSITIONS,
            ]),
            Shape::from_dims([C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        ),
    ] {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the standard builder opens");
        let left = builder
            .input::<F32>(InputKey::new("left").expect("a valid key"), left)
            .expect("an F32 input");
        let right = builder
            .input::<F32>(InputKey::new("right").expect("a valid key"), right)
            .expect("an F32 input");
        let result = F32TensorContraction::apply(&mut builder, &structure, left, right)
            .expect("an attention structure is admitted through the facade");
        builder
            .output(OutputKey::new("result").expect("a valid key"), result)
            .expect("an output");
        assert_eq!(
            builder
                .build()
                .expect("the program is complete")
                .operation_count(),
            1
        );
    }
}

// --- The five structural refusals at this width -----------------------------

#[test]
fn every_structural_rule_refuses_at_four_wide_tuples_too() {
    let score_operands = [
        f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
    ];

    // Rule one: `grtd,gsd->grtx`, whose output names an index no operand does.
    assert_eq!(
        refusal(
            &score_operands,
            raw_structure(&[&[0, 1, 2, 3], &[0, 4, 3]], &[0, 1, 2, 5], &[3])
        ),
        "contraction.rule.output-index-in-no-operand"
    );

    // Rule two: `grtd,gs->grts` drops the head lane from the key operand, so `d`
    // is summed in one operand — a reduction of the query rather than a
    // contraction against the key.
    assert_eq!(
        refusal(
            &[
                f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
                f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS]),
            ],
            raw_structure(&[&[0, 1, 2, 3], &[0, 4]], &[0, 1, 2, 4], &[3])
        ),
        "contraction.rule.summed-index-in-one-operand"
    );

    // Rule three: `grtt,gsd->grts` repeats the query position inside operand 0,
    // which is a diagonal rather than a contraction operand.
    assert_eq!(
        refusal(
            &[
                f32_operand(&[
                    C1_GROUPS,
                    C1_REPEATS,
                    C1_QUERY_POSITIONS,
                    C1_QUERY_POSITIONS
                ]),
                f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
            ],
            raw_structure(&[&[0, 1, 2, 2], &[0, 4, 3]], &[0, 1, 2, 4], &[3])
        ),
        "contraction.rule.index-repeated-within-operand"
    );

    // Rule four: `grtd,gsd->grtt` names the query position twice in the output.
    assert_eq!(
        refusal(
            &score_operands,
            raw_structure(&[&[0, 1, 2, 3], &[0, 4, 3]], &[0, 1, 2, 2], &[3])
        ),
        "contraction.rule.duplicate-output-index"
    );

    // Rule five: a third `gsd` operand shares the group and the head lane across
    // three operands. This is where the reserved multi-operand answer lands, and
    // it is decided on the structure's own operand count — before the exact-arity
    // schema would otherwise refuse it first.
    assert_eq!(
        refusal(
            &score_operands,
            raw_structure(
                &[&[0, 1, 2, 3], &[0, 4, 3], &[0, 4, 3]],
                &[0, 1, 2, 4],
                &[3]
            )
        ),
        "contraction.rule.index-in-more-than-two-operands"
    );

    // The same five, from the typed constructor a frontend actually calls.
    assert!(matches!(
        ContractionIndexStructure::new(
            [vec![G, R, TQ, DH], vec![G, SK, DH]],
            [G, R, TQ, ContractionIndex::new(99)]
        ),
        Err(ContractionStructureError::OutputIndexInNoOperand { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([vec![G, R, TQ, DH], vec![G, SK]], [G, R, TQ, SK]),
        Err(ContractionStructureError::SummedIndexInOneOperand { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([vec![G, R, TQ, TQ], vec![G, SK, DH]], [G, R, TQ, SK]),
        Err(ContractionStructureError::IndexRepeatedWithinOperand { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new([vec![G, R, TQ, DH], vec![G, SK, DH]], [G, R, TQ, TQ]),
        Err(ContractionStructureError::DuplicateOutputIndex { .. })
    ));
    assert!(matches!(
        ContractionIndexStructure::new(
            [vec![G, R, TQ, DH], vec![G, SK, DH], vec![G, SK, DH]],
            [G, R, TQ, SK]
        ),
        Err(ContractionStructureError::IndexInMoreThanTwoOperands { .. })
    ));
}

// --- Extent agreement at this width ----------------------------------------

/// Both attention structures resolve their shared extents through the accepted
/// three-outcome path, and a disproof names both observed operand axes.
///
/// **The unresolved outcome remains unreachable, exactly as the projection
/// profile's landing recorded.** `StrictTensorContractionF32::infer` narrows
/// each operand through `OperationInferenceRequest::static_operand_shape` before
/// that operand's extents reach binding or equality. A sourced `S` therefore
/// receives that named host refusal rather than reaching equality; this family
/// has no symbolic equality or unresolved-requirement rule yet. Structure 3's
/// contracted extent is therefore exercised at the static `S` values the C1 row
/// actually takes — ten at prefill and up to eighteen across its decode — rather
/// than as a symbol.
#[test]
fn both_attention_structures_agree_their_shared_extents_or_name_both_sources() {
    // Structure 2 shares the static head dimension 128 between operand 0 axis 3
    // and operand 1 axis 2.
    let message = refusal_message(
        &[
            f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
            f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, 64]),
        ],
        score_structure().canonical_value().clone(),
    );
    assert!(
        message.contains("operand 0 axis 3") && message.contains("operand 1 axis 2"),
        "a disproved head-dimension equality names both observed sources: {message}"
    );
    assert!(
        message.contains("128") && message.contains("64"),
        "the pinned checkpoint declares head dimension 128, and the divided \
         hidden_size / num_attention_heads reading is 64: {message}"
    );

    // The group extent is shared too, and disagreeing on it is a different pair
    // of sources than disagreeing on the head lane.
    let message = refusal_message(
        &[
            f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
            f32_operand(&[16, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        ],
        score_structure().canonical_value().clone(),
    );
    assert!(
        message.contains("operand 0 axis 0") && message.contains("operand 1 axis 0"),
        "{message}"
    );

    // Structure 3 shares the key position between operand 0 axis 3 and operand 1
    // axis 1, at every static `S` the C1 row takes.
    for context in [10_u64, 16, 18] {
        let results = infer(
            &[
                f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, context]),
                f32_operand(&[C1_GROUPS, context, C1_HEAD_DIM]),
            ],
            value_structure().canonical_value().clone(),
        )
        .unwrap_or_else(|error| panic!("S = {context} is admitted: {error}"));
        assert_eq!(
            results[0].shape().as_static(),
            Some(&Shape::from_dims([
                C1_GROUPS,
                C1_REPEATS,
                C1_QUERY_POSITIONS,
                C1_HEAD_DIM
            ])),
            "the contracted extent leaves the result shape, whatever S is"
        );
    }
    let message = refusal_message(
        &[
            f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, 18]),
            f32_operand(&[C1_GROUPS, 16, C1_HEAD_DIM]),
        ],
        value_structure().canonical_value().clone(),
    );
    assert!(
        message.contains("operand 0 axis 3") && message.contains("operand 1 axis 1"),
        "a stale context length is disproved against both sources: {message}"
    );
    assert!(
        message.contains("18") && message.contains("16"),
        "{message}"
    );
}

/// The declared empty-contracted-domain behaviour, for the growing extent.
///
/// `S = 0` is statically unreachable in this workload — the C1 row's shortest
/// context is ten positions — and the family still owes a declared behaviour,
/// because the extent is an attribute and not a proof. An unseeded strict fold
/// has no empty result, so the occurrence is refused rather than given one.
#[test]
fn the_value_structure_refuses_a_zero_context_length() {
    assert_eq!(
        refusal(
            &[
                f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, 0]),
                f32_operand(&[C1_GROUPS, 0, C1_HEAD_DIM]),
            ],
            value_structure().canonical_value().clone()
        ),
        "contraction.extent.empty-contracted-domain"
    );
    // A zero *free* extent is the other case, and it is admitted: no reduction is
    // performed and the result is an empty tensor. Decoding zero new positions is
    // the occurrence that reaches it.
    let results = infer(
        &[
            f32_operand(&[C1_GROUPS, C1_REPEATS, 0, C1_QUERY_POSITIONS]),
            f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, C1_HEAD_DIM]),
        ],
        value_structure().canonical_value().clone(),
    )
    .expect("an empty free extent produces an empty result");
    assert_eq!(
        results[0].shape().as_static(),
        Some(&Shape::from_dims([C1_GROUPS, C1_REPEATS, 0, C1_HEAD_DIM]))
    );
    // And the score structure refuses a zero head dimension for the same reason,
    // so the refusal is the family's rather than one structure's.
    assert_eq!(
        refusal(
            &[
                f32_operand(&[C1_GROUPS, C1_REPEATS, C1_QUERY_POSITIONS, 0]),
                f32_operand(&[C1_GROUPS, C1_QUERY_POSITIONS, 0]),
            ],
            score_structure().canonical_value().clone()
        ),
        "contraction.extent.empty-contracted-domain"
    );
}

/// Neither attention structure moves the family's identity.
///
/// They are structure *values* under the one key ADR 0087 accepts, so the
/// registered definition, its schema, and its fourteen-field signature are
/// unchanged by admitting them. A second key would be the collision that ADR
/// rejects, and this is the check that would notice one appearing.
#[test]
fn admitting_the_attention_structures_registers_no_second_key() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let contraction_keys = registry
        .operation_definitions()
        .filter(|definition| definition.key().name().contains("contraction"))
        .count();
    assert_eq!(
        contraction_keys, 1,
        "a frontend never chooses among contraction keys, because there is only one"
    );
    for structure in [score_structure(), value_structure(), workload_structure()] {
        assert!(
            registry
                .infer_operation(
                    &strict_tensor_contraction_f32_op(),
                    &operands_for(&structure),
                    &attributes(structure.canonical_value().clone()),
                )
                .is_ok(),
            "all three workload structures are admitted under the one key"
        );
    }
}

/// Operands whose extents satisfy one structure, for the identity check above.
fn operands_for(structure: &ContractionIndexStructure) -> Vec<ValueFact> {
    // Every index gets extent four, so agreement holds by construction and the
    // check above is about the key rather than about a shape.
    structure
        .operands()
        .map(|tuple| f32_operand(&vec![4_u64; tuple.len()]))
        .collect()
}

#[test]
fn the_contraction_declares_no_algebraic_capability() {
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    assert!(
        !registry
            .operation_definition(&strict_tensor_contraction_f32_op())
            .expect("the contraction is registered")
            .algebraic_capabilities()
            .declares_ordered_associativity(),
        "a strict fold whose contributors may not be regrouped must not declare \
         ordered associativity: a missing declaration is unknown, never the inverse law"
    );
}
