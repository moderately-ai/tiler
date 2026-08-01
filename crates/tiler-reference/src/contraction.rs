//! Reference semantics for `tiler::strict-tensor-contraction-f32@1`.
//!
//! # The fold, and the part of it that is easy to get wrong
//!
//! For each output coordinate, over the ascending lexicographic contributor
//! sequence `d` of the canonically ordered contracted index space:
//!
//! ```text
//! p_d = fl(A[..., d] * B[..., d])   # one rounding each
//! acc = p_0                         # the FIRST product, never +0.0
//! acc = fl(acc + p_d)               # d = 1 .. K-1
//! ```
//!
//! `fl(+0.0 + x)` equals `x` for every binary32 `x` except `x = -0.0`, where it
//! is `+0.0`. So on a vector whose every product is `-0.0` a first-product seed
//! returns `0x80000000` and a `+0.0` seed returns `0x00000000`, and the idiomatic
//! accumulator-starts-at-zero loop silently computes the second one. A `+0.0`
//! start is not a defect on its own — it is a reduction carrying an explicit
//! initial contributor, which is a *different* operation — so [`ContractionSeed`]
//! expresses both and neither is supplied silently.
//!
//! # Why the contract is decoded rather than restated
//!
//! Every value this fold is parameterized by is declared by the operation's own
//! fourteen-field numerical signature, which
//! [`strict_tensor_contraction_f32_facts`] returns. Four are read as values — the
//! accumulator type, the result type, the canonical arithmetic-NaN payload, and
//! the seed. The other ten are *verified*: the declaration must say the one thing
//! this evaluator realizes, and a record saying anything else is refused by field
//! ID rather than evaluated under whichever reading the code happens to
//! implement.
//!
//! Refusing is what makes both directions matter. A declaration this evaluator
//! over-satisfies is as wrong to accept as one it under-satisfies: accepting a
//! boundary-only canonicalization and canonicalizing per combine anyway would
//! report a bitwise agreement the contract never promised, which is exactly how
//! an oracle comes to answer a question it was not asked.
//!
//! This reading lives here, in the one consumer, rather than in `tiler-ir` beside
//! the declaration. A second consumer needing it should promote it to the
//! declaring crate rather than write a second reading of the same record.
//!
//! # What per-combine canonicalization does and does not change here
//!
//! D-8 is answered in the declared signature as
//! `after-every-combine-and-at-the-result-boundary`, and this evaluator
//! implements exactly that: the product, every accumulation step, and the result
//! all commit the canonical payload. That is strictly stronger than canonicalizing
//! nowhere — a payload-propagating fold returns the operand's `0x7fc0dead` where
//! this returns `0x7fc00000`. It is *not* observable against a boundary-only fold
//! **in this evaluator's outputs**: binary32 addition of a NaN accumulator yields
//! a NaN whatever the payload, so no intermediate payload reaches the result by
//! any route other than the boundary, which canonicalizes it. The declared
//! per-combine rule therefore binds realizations — a matrix instruction, a split
//! fold, a device whose propagation differs — rather than the arithmetic below.
//! Saying so is the point: the check that can say no about the *site* is the
//! signature decode, not a value comparison.

use std::collections::BTreeMap;

use tiler_ir::semantic::{
    CONTRACTION_F32_FACT_ACCUMULATOR_TYPE, CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
    CONTRACTION_F32_FACT_CANONICAL_NAN_BITS, CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
    CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE, CONTRACTION_F32_FACT_CONVERSION,
    CONTRACTION_F32_FACT_DETERMINISM, CONTRACTION_F32_FACT_DISTRIBUTIVITY,
    CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN, CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
    CONTRACTION_F32_FACT_PERMUTATION_PERMITTED, CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED,
    CONTRACTION_F32_FACT_RESULT_TYPE, CONTRACTION_F32_FACT_SEED,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalValue, CanonicalValueView, ContractionIndex,
    ContractionIndexStructure, F32, OperationAttributes, ResolvedValueType, TypeKey,
    strict_tensor_contraction_f32_facts,
};
use tiler_ir::shape::{Extent, Shape};

use super::MAX_REFERENCE_TENSOR_ELEMENTS;
use super::error::{ReferenceOperationError, UnsupportedContractionDeclaration};
use super::evaluate::{
    decode_coordinate, decode_f32, f32_element, f32_elements, preflight_f32_output,
    row_major_strides,
};
use super::registry::{ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs};
use super::tensor::{ReferenceElement, Tensor};

/// Where the accumulator starts before the ascending fold.
///
/// Two states rather than an `Option<f32>`, because "unseeded" and "seeded at
/// `+0.0`" are different operations with different results, and an `Option` whose
/// `None` is implemented as zero is precisely the confusion this family's
/// signature exists to prevent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ContractionSeed {
    /// Unseeded: the accumulator starts at the first product.
    ///
    /// An empty contracted domain has no result under this seed, which is why
    /// the declared empty-domain behaviour is a refusal.
    FirstProduct,
    /// Seeded: the initial value is one logical contributor, folded first.
    ///
    /// No registered contraction declares this today, and no canonical spelling
    /// for it is invented here. It is implemented because the contract
    /// distinguishes it from the unseeded fold, and an evaluator that could not
    /// express it would have no way to say what the other operation computes —
    /// which is what the seed regression compares against.
    #[allow(
        dead_code,
        reason = "no registered contraction declares a seed, so only the seed \
                  regression constructs this; deleting it would leave the \
                  evaluator unable to state what the other operation computes, \
                  which is the whole content of that regression"
    )]
    Initial(f32),
}

/// The declared contraction contract this evaluator was parameterized by.
#[derive(Clone, Debug)]
pub(crate) struct ContractionContract {
    accumulator_type: ResolvedValueType,
    result_type: ResolvedValueType,
    canonical_nan_bits: u32,
    seed: ContractionSeed,
}

impl ContractionContract {
    /// Decodes the governed `tiler::strict-tensor-contraction-f32@1` signature.
    ///
    /// # Errors
    ///
    /// Returns the field whose declared value this reference does not realize.
    pub(crate) fn governed() -> Result<Self, UnsupportedContractionDeclaration> {
        Self::decode(&strict_tensor_contraction_f32_facts())
    }

    /// Decodes one fourteen-field contraction numerical signature.
    ///
    /// Every field is consulted, and a record carrying a different number of them
    /// is malformed rather than partially honoured: a field this reading skipped
    /// would be a declared contract term the oracle silently did not follow.
    ///
    /// # Errors
    ///
    /// Returns the field whose declared value this reference does not realize.
    pub(crate) fn decode(
        facts: &CanonicalValue,
    ) -> Result<Self, UnsupportedContractionDeclaration> {
        let CanonicalValueView::Record(fields) = facts.view() else {
            return Err(UnsupportedContractionDeclaration::MalformedRecord);
        };
        let mut declared = BTreeMap::new();
        for field in fields {
            if declared.insert(field.id(), field.value()).is_some() {
                return Err(UnsupportedContractionDeclaration::MalformedRecord);
            }
        }
        if declared.len() != CONTRACTION_SIGNATURE_FIELDS {
            return Err(UnsupportedContractionDeclaration::MalformedRecord);
        }
        let fact = |id| {
            declared
                .get(&id)
                .copied()
                .ok_or(UnsupportedContractionDeclaration::MalformedRecord)
        };

        // The seven verified strings. Each must say the one thing the arithmetic
        // below realizes; the refusal names the field, so a reader learns *which*
        // term moved rather than only that the record did.
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
            let CanonicalValueView::Utf8(declared) = fact(id)?.view() else {
                return Err(UnsupportedContractionDeclaration::unrealizable(id));
            };
            if declared != expected {
                return Err(UnsupportedContractionDeclaration::unrealizable(id));
            }
        }
        // The three permissions, each of which would turn one value into a result
        // set. `conformance` refuses the same three on a declared *realization*;
        // this is that rule reaching the semantics the realization implements.
        for id in [
            CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED,
            CONTRACTION_F32_FACT_PERMUTATION_PERMITTED,
        ] {
            if !matches!(fact(id)?.view(), CanonicalValueView::Bool(false)) {
                return Err(UnsupportedContractionDeclaration::unrealizable(id));
            }
        }

        // The four read values.
        let accumulator_type = declared_type(fact(CONTRACTION_F32_FACT_ACCUMULATOR_TYPE)?)
            .ok_or_else(|| {
                UnsupportedContractionDeclaration::unrealizable(
                    CONTRACTION_F32_FACT_ACCUMULATOR_TYPE,
                )
            })?;
        let result_type =
            declared_type(fact(CONTRACTION_F32_FACT_RESULT_TYPE)?).ok_or_else(|| {
                UnsupportedContractionDeclaration::unrealizable(CONTRACTION_F32_FACT_RESULT_TYPE)
            })?;
        let canonical_nan_bits = declared_f32_bits(fact(CONTRACTION_F32_FACT_CANONICAL_NAN_BITS)?)
            .ok_or_else(|| {
                UnsupportedContractionDeclaration::unrealizable(
                    CONTRACTION_F32_FACT_CANONICAL_NAN_BITS,
                )
            })?;
        let seed = declared_seed(fact(CONTRACTION_F32_FACT_SEED)?).ok_or_else(|| {
            UnsupportedContractionDeclaration::unrealizable(CONTRACTION_F32_FACT_SEED)
        })?;

        // The two declared types are read, and then required to be the one type
        // the host arithmetic below computes in. Any other declaration is a
        // contract this evaluator cannot compute, not one it should approximate
        // by rounding through `f32` anyway.
        if accumulator_type != F32::resolved_type() {
            return Err(UnsupportedContractionDeclaration::unrealizable(
                CONTRACTION_F32_FACT_ACCUMULATOR_TYPE,
            ));
        }
        if result_type != F32::resolved_type() {
            return Err(UnsupportedContractionDeclaration::unrealizable(
                CONTRACTION_F32_FACT_RESULT_TYPE,
            ));
        }
        // A canonical payload that is not a NaN would make every canonicalization
        // site replace an arithmetic NaN with a finite value.
        if !f32::from_bits(canonical_nan_bits).is_nan() {
            return Err(UnsupportedContractionDeclaration::unrealizable(
                CONTRACTION_F32_FACT_CANONICAL_NAN_BITS,
            ));
        }
        Ok(Self {
            accumulator_type,
            result_type,
            canonical_nan_bits,
            seed,
        })
    }

    /// Returns the contract this one declares, with a different accumulator seed.
    ///
    /// The seeded fold is a different operation, so this is a constructor for a
    /// *comparison*, never a relaxation of the governed one: it exists so a test
    /// can state what the `+0.0`-seeded reduction computes and show it differs.
    #[cfg(test)]
    pub(crate) fn with_seed(&self, seed: ContractionSeed) -> Self {
        Self {
            seed,
            ..self.clone()
        }
    }

    /// Replaces an arithmetic NaN with the payload this contract declares.
    ///
    /// Applied to a *produced* value only. An operand is read, never rewritten,
    /// so a non-canonical NaN reaches the multiply exactly as it was supplied.
    fn canonicalize(&self, value: f32) -> f32 {
        if value.is_nan() {
            f32::from_bits(self.canonical_nan_bits)
        } else {
            value
        }
    }
}

/// Fields the governed contraction's numerical signature declares.
const CONTRACTION_SIGNATURE_FIELDS: usize = 14;

fn declared_type(value: &CanonicalValue) -> Option<ResolvedValueType> {
    match value.view() {
        CanonicalValueView::Type(resolved) => Some(resolved.clone()),
        _ => None,
    }
}

fn declared_f32_bits(value: &CanonicalValue) -> Option<u32> {
    let CanonicalValueView::FloatBits(payload) = value.view() else {
        return None;
    };
    if payload.format() != &TypeKey::new("tiler", "f32", 1).ok()? {
        return None;
    }
    <[u8; 4]>::try_from(payload.bits())
        .ok()
        .map(u32::from_be_bytes)
}

/// Decodes the declared accumulator seed.
///
/// Only the unseeded spelling is admitted, because it is the only one any
/// registered contraction declares. [`ContractionSeed::Initial`] deliberately has
/// no canonical spelling here: inventing one would introduce a semantics the
/// normative text has not defined, and the fold expresses the seeded operation
/// without a record needing to name it.
fn declared_seed(value: &CanonicalValue) -> Option<ContractionSeed> {
    match value.view() {
        CanonicalValueView::Utf8("none-the-accumulator-starts-at-the-first-product") => {
            Some(ContractionSeed::FirstProduct)
        }
        _ => None,
    }
}

/// The registered reference implementation of the governed contraction.
pub(crate) struct StrictTensorContractionF32Reference {
    contract: ContractionContract,
}

impl StrictTensorContractionF32Reference {
    pub(crate) const fn new(contract: ContractionContract) -> Self {
        Self { contract }
    }
}

impl ReferenceOperation for StrictTensorContractionF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [left, right] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let structure = contraction_structure(request.attributes())?;
        contract_operands(&self.contract, &structure, left, right)
            .and_then(|tensor| outputs.push(tensor))
    }
}

fn contraction_structure(
    attributes: &OperationAttributes,
) -> Result<ContractionIndexStructure, ReferenceOperationError> {
    if attributes.fields().len() != 1 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let Some(value) = attributes.get(CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE) else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    ContractionIndexStructure::from_canonical_value(value)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

/// Evaluates one contraction over an arbitrary admitted binary index structure.
///
/// The extent of every index, the result shape, and the contracted iteration
/// space are recomputed from the structure and the operands rather than taken
/// from the graph, following the structural families' rule: the semantic registry
/// already refused a malformed occurrence at construction, so a disagreement here
/// is invalid state, reported rather than resolved in favour of either side.
pub(crate) fn contract_operands(
    contract: &ContractionContract,
    structure: &ContractionIndexStructure,
    left: &Tensor,
    right: &Tensor,
) -> Result<Tensor, ReferenceOperationError> {
    let operands = [left, right];
    if structure.operand_count() != operands.len() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let mut elements: Vec<&[ReferenceElement]> = Vec::with_capacity(operands.len());
    for operand in operands {
        // The declared computation precision is binary32 operands, so an operand
        // outside the declared accumulator type is refused against the *contract*
        // rather than against a type this function chose.
        if operand.resolved_type() != &contract.accumulator_type {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        elements.push(f32_elements(operand)?);
    }

    // One extent per index, bound by the first operand axis naming it and
    // required to agree everywhere after. Zipped rather than indexed, so the
    // operand-count refusal above is what keeps every access in range.
    let mut extents: BTreeMap<ContractionIndex, Extent> = BTreeMap::new();
    for (tuple, operand) in structure.operands().zip(operands) {
        if operand.shape().rank() != tuple.len() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        for (axis, index) in tuple.iter().enumerate() {
            let extent = operand.shape().extents()[axis];
            if *extents.entry(*index).or_insert(extent) != extent {
                return Err(ReferenceOperationError::InvalidApplication);
            }
        }
    }
    let extent_of = |index: &ContractionIndex| -> Result<Extent, ReferenceOperationError> {
        extents
            .get(index)
            .copied()
            .ok_or(ReferenceOperationError::InvalidApplication)
    };

    let output_shape = shape_of(structure.output(), extent_of)?;
    let output_count = output_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    preflight_f32_output(output_count)?;

    // The contracted iteration space, ascending lexicographic: `contracted()` is
    // ascending by canonical index, and row-major strides over it make the
    // lowest-numbered contracted index the most significant coordinate. That
    // ordering *is* the declared contributor sequence; reversing it would be the
    // permutation this family declares forbidden.
    let contracted_shape = shape_of(structure.contracted(), extent_of)?;
    let contracted_count = contracted_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    if contracted_count == 0 {
        // The declared empty-domain behaviour, verified at decode: an unseeded
        // fold has no empty result. The semantic inferencer refuses a zero
        // contracted extent at construction, so this is unreachable through a
        // built program and reachable through a direct reference application.
        return Err(ReferenceOperationError::InvalidApplication);
    }
    // The fold performs `output_count * contracted_count` multiply-accumulate
    // steps, which is larger than either operand and is bounded by neither of the
    // tensor limits the operands already passed. Refused before the loop rather
    // than discovered inside it.
    if output_count
        .checked_mul(contracted_count)
        .is_none_or(|steps| steps > MAX_REFERENCE_TENSOR_ELEMENTS)
    {
        return Err(ReferenceOperationError::ShapeTooLarge);
    }
    if output_count == 0 {
        return Tensor::dense(contract.result_type.clone(), output_shape, Vec::new())
            .map_err(|_| ReferenceOperationError::ShapeTooLarge);
    }

    // Each operand axis reads either an output coordinate or a contracted one.
    // Resolved once per operand axis rather than per element, so the inner loop
    // is two strided reads and two roundings.
    let mut readers: Vec<Vec<AxisReader>> = Vec::with_capacity(operands.len());
    for tuple in structure.operands() {
        let mut operand_readers = Vec::with_capacity(tuple.len());
        for index in tuple {
            let reader =
                if let Some(position) = structure.output().iter().position(|free| free == index) {
                    AxisReader::Output(position)
                } else if let Some(position) = structure
                    .contracted()
                    .iter()
                    .position(|summed| summed == index)
                {
                    AxisReader::Contracted(position)
                } else {
                    // Every operand index is free or contracted by the structure's own
                    // derivation, so reaching here is invalid state rather than a
                    // caller error. Refused rather than assumed away.
                    return Err(ReferenceOperationError::InvalidApplication);
                };
            operand_readers.push(reader);
        }
        readers.push(operand_readers);
    }

    let output_strides = row_major_strides(&output_shape)?;
    let contracted_strides = row_major_strides(&contracted_shape)?;
    let operand_strides = operands
        .iter()
        .map(|operand| row_major_strides(operand.shape()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output_coordinate = vec![0_usize; output_shape.rank()];
    let mut contracted_coordinate = vec![0_usize; contracted_shape.rank()];
    let mut results = Vec::with_capacity(output_count);
    for output_linear in 0..output_count {
        decode_coordinate(
            output_linear,
            &output_shape,
            &output_strides,
            &mut output_coordinate,
        )?;
        let mut accumulator = match contract.seed {
            ContractionSeed::FirstProduct => None,
            ContractionSeed::Initial(initial) => Some(initial),
        };
        for contracted_linear in 0..contracted_count {
            decode_coordinate(
                contracted_linear,
                &contracted_shape,
                &contracted_strides,
                &mut contracted_coordinate,
            )?;
            let mut factors = [0.0_f32; 2];
            for (position, factor) in factors.iter_mut().enumerate() {
                let offset = readers[position]
                    .iter()
                    .zip(&operand_strides[position])
                    .try_fold(0_usize, |offset, (reader, stride)| {
                        let coordinate = match reader {
                            AxisReader::Output(axis) => output_coordinate[*axis],
                            AxisReader::Contracted(axis) => contracted_coordinate[*axis],
                        };
                        coordinate
                            .checked_mul(*stride)
                            .and_then(|scaled| offset.checked_add(scaled))
                            .ok_or(ReferenceOperationError::ShapeTooLarge)
                    })?;
                let element = elements[position]
                    .get(offset)
                    .ok_or(ReferenceOperationError::InvalidApplication)?;
                *factor = decode_f32(element)?;
            }
            // One rounding for the product and one for the accumulation, each
            // canonicalized: the fused single-rounding form is the permission
            // this family declares forbidden.
            let product = contract.canonicalize(factors[0] * factors[1]);
            accumulator = Some(match accumulator {
                None => product,
                Some(value) => contract.canonicalize(value + product),
            });
        }
        let value = accumulator.ok_or(ReferenceOperationError::InvalidApplication)?;
        results.push(f32_element(contract.canonicalize(value))?);
    }
    Tensor::dense(contract.result_type.clone(), output_shape, results)
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)
}

fn shape_of(
    indices: &[ContractionIndex],
    extent_of: impl Fn(&ContractionIndex) -> Result<Extent, ReferenceOperationError>,
) -> Result<Shape, ReferenceOperationError> {
    Shape::try_new(
        indices
            .iter()
            .map(extent_of)
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|_| ReferenceOperationError::ShapeTooLarge)
}

/// Which coordinate one operand axis reads.
#[derive(Clone, Copy, Debug)]
enum AxisReader {
    /// The output coordinate at this position of the output tuple.
    Output(usize),
    /// The contracted coordinate at this position of the contracted set.
    Contracted(usize),
}

#[cfg(test)]
mod tests;
