//! Unit evidence for the contraction reference's signature decode and fold.
//!
//! The bit-exact corpus lives in `tests/contraction_conformance.rs`, which drives
//! the public boundary. These cover what that path cannot reach: the refusals of
//! a perturbed declaration, and the seeded fold, which no registered contraction
//! declares and which therefore has no public spelling to drive it through.

use tiler_ir::semantic::{
    CONTRACTION_F32_FACT_ACCUMULATOR_TYPE, CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
    CONTRACTION_F32_FACT_CANONICAL_NAN_BITS, CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
    CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE, CONTRACTION_F32_FACT_CONVERSION,
    CONTRACTION_F32_FACT_DETERMINISM, CONTRACTION_F32_FACT_DISTRIBUTIVITY,
    CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN, CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
    CONTRACTION_F32_FACT_PERMUTATION_PERMITTED, CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED,
    CONTRACTION_F32_FACT_RESULT_TYPE, CONTRACTION_F32_FACT_SEED, CanonicalField, CanonicalValue,
    CanonicalValueView, ContractionIndex, ContractionIndexStructure, F32, U8,
    strict_tensor_contraction_f32_facts,
};
use tiler_ir::shape::Shape;

use super::{ContractionContract, ContractionSeed, contract_operands};
use crate::MAX_REFERENCE_TENSOR_ELEMENTS;
use crate::error::{ReferenceOperationError, UnsupportedContractionDeclaration};
use crate::evaluate::{f32_element, f32_elements};
use crate::tensor::Tensor;

/// The `td,od->to` structure the admitted profile names.
fn structure() -> ContractionIndexStructure {
    let index = ContractionIndex::new;
    ContractionIndexStructure::new(
        [vec![index(0), index(1)], vec![index(2), index(1)]],
        [index(0), index(2)],
    )
    .expect("`td,od->to` is admitted")
}

fn tensor(dims: [u64; 2], bits: &[u32]) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims(dims),
        bits.iter()
            .map(|bits| f32_element(f32::from_bits(*bits)))
            .collect::<Result<_, _>>()
            .expect("a four-byte payload"),
    )
    .expect("the fixture is well formed")
}

fn result_bits(tensor: &Tensor) -> Vec<u32> {
    f32_elements(tensor)
        .expect("a dense f32 result")
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// Replaces one field of the governed signature, keeping every other field.
fn perturbed(
    field: tiler_ir::semantic::AttributeFieldId,
    value: &CanonicalValue,
) -> CanonicalValue {
    let governed = strict_tensor_contraction_f32_facts();
    let CanonicalValueView::Record(fields) = governed.view() else {
        panic!("the governed signature is a record");
    };
    let mut replaced = false;
    let rebuilt: Vec<CanonicalField> = fields
        .iter()
        .map(|existing| {
            if existing.id() == field {
                replaced = true;
                CanonicalField::new(field, value.clone())
            } else {
                existing.clone()
            }
        })
        .collect();
    assert!(replaced, "the perturbed field must exist in the record");
    CanonicalValue::record(rebuilt).expect("the perturbed record is canonical")
}

fn text(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a bounded fact")
}

#[test]
fn the_governed_signature_decodes_to_the_unseeded_binary32_fold() {
    let contract = ContractionContract::governed().expect("the governed signature is realizable");
    assert_eq!(contract.seed, ContractionSeed::FirstProduct);
    assert_eq!(contract.accumulator_type, F32::resolved_type());
    assert_eq!(contract.result_type, F32::resolved_type());
    assert_eq!(contract.canonical_nan_bits, 0x7fc0_0000);
}

/// Every declared term the fold depends on can refuse, under its own field ID.
///
/// Written as a table over the fourteen fields rather than as one representative
/// case, because a decode that silently ignored a field would still pass a
/// representative case — and an ignored field is a contract term the oracle did
/// not honour.
#[test]
fn a_declaration_this_reference_does_not_compute_is_refused_by_field() {
    for (field, value) in [
        (
            CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
            text("binary32-operands-and-binary64-products"),
        ),
        (
            CONTRACTION_F32_FACT_ACCUMULATOR_TYPE,
            CanonicalValue::value_type(U8::resolved_type()),
        ),
        (
            CONTRACTION_F32_FACT_RESULT_TYPE,
            CanonicalValue::value_type(U8::resolved_type()),
        ),
        (
            CONTRACTION_F32_FACT_CONVERSION,
            text("widen-products-to-binary64-and-round-the-result"),
        ),
        (
            CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE,
            text("descending-lexicographic-over-the-canonically-ordered-contracted-index-space"),
        ),
        (
            CONTRACTION_F32_FACT_SEED,
            text("positive-zero-the-accumulator-starts-at-an-explicit-initial"),
        ),
        (
            CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN,
            text("positive-zero-the-seed-is-the-empty-result"),
        ),
        (
            CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED,
            CanonicalValue::boolean(true),
        ),
        (
            CONTRACTION_F32_FACT_PERMUTATION_PERMITTED,
            CanonicalValue::boolean(true),
        ),
        (
            CONTRACTION_F32_FACT_DISTRIBUTIVITY,
            text("permitted-over-addition"),
        ),
        (
            CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(true),
        ),
        (
            CONTRACTION_F32_FACT_CANONICAL_NAN_BITS,
            CanonicalValue::float_bits(
                tiler_ir::semantic::TypeKey::new("tiler", "f32", 1).unwrap(),
                0x3f80_0000_u32.to_be_bytes(),
            )
            .unwrap(),
        ),
        (
            // The boundary-only reading D-8 did not take. Refused rather than
            // accepted-and-over-satisfied: this fold canonicalizes per combine,
            // and reporting agreement with a weaker declaration would assert a
            // guarantee that declaration never made.
            CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
            text("at-the-result-boundary-only"),
        ),
        (
            CONTRACTION_F32_FACT_DETERMINISM,
            text("run-to-run-nondeterministic"),
        ),
    ] {
        assert_eq!(
            ContractionContract::decode(&perturbed(field, &value)).unwrap_err(),
            UnsupportedContractionDeclaration::unrealizable(field),
            "field {field} must be able to refuse"
        );
    }
}

#[test]
fn a_record_that_is_not_the_governed_field_set_is_malformed() {
    assert_eq!(
        ContractionContract::decode(&text("not a record")).unwrap_err(),
        UnsupportedContractionDeclaration::MalformedRecord
    );
    let governed = strict_tensor_contraction_f32_facts();
    let CanonicalValueView::Record(fields) = governed.view() else {
        panic!("the governed signature is a record");
    };
    let short: Vec<CanonicalField> = fields.iter().skip(1).cloned().collect();
    assert_eq!(
        ContractionContract::decode(&CanonicalValue::record(short).unwrap()).unwrap_err(),
        UnsupportedContractionDeclaration::MalformedRecord
    );
}

/// The seed is a read parameter, and the two seeds compute different values.
///
/// This is the `negative_zero_seed` vector: every product is `-0.0`, so the
/// unseeded fold returns `0x80000000` and the `+0.0`-seeded one returns
/// `0x00000000`. It is retained as the permanent form of the regression the
/// ticket required watched failing — with the seed wrong, the corpus expectation
/// is the value on the right.
#[test]
fn the_first_product_seed_and_a_positive_zero_seed_disagree_on_signed_zero() {
    let structure = structure();
    let left = tensor([1, 16], &[0xbf80_0000; 16]);
    let right = tensor([1, 16], &[0x0000_0000; 16]);
    let governed = ContractionContract::governed().expect("the governed signature is realizable");
    assert_eq!(
        result_bits(&contract_operands(&governed, &structure, &left, &right).unwrap()),
        vec![0x8000_0000]
    );
    let seeded = governed.with_seed(ContractionSeed::Initial(0.0));
    assert_eq!(
        result_bits(&contract_operands(&seeded, &structure, &left, &right).unwrap()),
        vec![0x0000_0000]
    );
}

/// The declared empty-domain refusal, reached where only a direct application can.
///
/// A built program cannot carry it: the semantic inferencer refuses a zero
/// contracted extent at construction. Driving the evaluator directly is what
/// proves this branch can say no rather than being unreachable prose.
#[test]
fn an_empty_contracted_domain_is_refused_rather_than_returning_a_seed() {
    let contract = ContractionContract::governed().expect("the governed signature is realizable");
    let left = tensor([1, 0], &[]);
    let right = tensor([1, 0], &[]);
    assert_eq!(
        contract_operands(&contract, &structure(), &left, &right),
        Err(ReferenceOperationError::InvalidApplication)
    );
    // The admitted neighbour, so the refusal above discriminates the empty domain
    // rather than the fixture.
    let left = tensor([1, 1], &[0x4000_0000]);
    let right = tensor([1, 1], &[0x4040_0000]);
    assert_eq!(
        result_bits(&contract_operands(&contract, &structure(), &left, &right).unwrap()),
        vec![0x40c0_0000]
    );
}

/// The fold's work bound refuses under its own variant, not the storage bound's.
///
/// Three cases, because one refusal proves nothing about *which* bound spoke.
/// With `d = 1` the output is the whole iteration space, so the stored-element
/// bound is what refuses — the meaning `OutputElementsExceeded` documents. With
/// `d = 2` and an output deliberately under that bound, no shape and no stored
/// result is over any limit; only the fold's step count is, and only the work
/// bound can name it. The third case is the same structure at a small extent, so
/// the two refusals discriminate the bound rather than the fixture family.
///
/// Every fixture is built from extents: both refusals happen before the fold
/// allocates its result or takes a step, so the test costs two small operands.
#[test]
fn an_iteration_space_over_the_bound_is_refused_as_iteration_work() {
    let contract = ContractionContract::governed().expect("the governed signature is realizable");
    let ones = |count: usize| vec![0x3f80_0000_u32; count];

    // `td,od->to` with `d = 1`: `t * o` steps producing `t * o` elements.
    let left = tensor([4096, 1], &ones(4096));
    let right = tensor([4097, 1], &ones(4097));
    assert_eq!(
        contract_operands(&contract, &structure(), &left, &right),
        Err(ReferenceOperationError::OutputElementsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual: 4096 * 4097,
        })
    );

    // `d = 2`: 8,389,712 output elements, under the limit, folded over two
    // contributors each — 16,779,424 multiply-accumulate steps, over it.
    let left = tensor([2896, 2], &ones(5792));
    let right = tensor([2897, 2], &ones(5794));
    // The premise the case rests on: this output is *not* over the storage bound,
    // so the refusal below can only be the work bound.
    const { assert!(2896 * 2897 <= MAX_REFERENCE_TENSOR_ELEMENTS) };
    assert_eq!(
        contract_operands(&contract, &structure(), &left, &right),
        Err(ReferenceOperationError::IterationStepsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual: 2896 * 2897 * 2,
        })
    );

    // The admitted neighbour: the same structure and the same `d = 2` fold, at an
    // extent whose work fits.
    let left = tensor([2, 2], &ones(4));
    let right = tensor([3, 2], &ones(6));
    assert_eq!(
        result_bits(&contract_operands(&contract, &structure(), &left, &right).unwrap()),
        vec![0x4000_0000; 6]
    );
}

/// A disagreeing extent on a shared index is refused, not silently truncated.
#[test]
fn a_contracted_extent_that_disagrees_between_operands_is_refused() {
    let contract = ContractionContract::governed().expect("the governed signature is realizable");
    let left = tensor([1, 3], &[0x3f80_0000; 3]);
    let right = tensor([1, 2], &[0x3f80_0000; 2]);
    assert_eq!(
        contract_operands(&contract, &structure(), &left, &right),
        Err(ReferenceOperationError::InvalidApplication)
    );
}
