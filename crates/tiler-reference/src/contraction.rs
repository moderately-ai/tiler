//! Reference semantics for `tiler::tensor-contraction-f32@1`'s strict cell.
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
//! thirteen-field numerical signature, validated and read through the **sole
//! decoder** — [`ContractionF32ReductionDescriptor`] in `tiler-ir` — rather
//! than by a second per-field reading here. The accepted successor contract
//! forbids a second compiler or reference decoder, because two readings are
//! two places for one fact to move apart; what this module used to verify
//! field by field the sole decoder now verifies once, for every consumer.
//!
//! What survives here is the *cell selection*: this evaluator computes the
//! strict left fold, which is the successor's result under every request
//! withholding reassociation — bit-identical to the retired strict key's
//! answer. A realization permitting reassociation never reaches this evaluator:
//! [`ReferenceNumericalConformance::from_realization`] refuses it first, and
//! the explicit topology evaluator is the only relaxed reference route.
//!
//! # What per-combine canonicalization does and does not change here
//!
//! D-8 is answered in the declared signature's canonicalization fact as
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
//!
//! # Where the declared numerical conformance applies in this fold
//!
//! The fold is one multiply and one add per contributor, so both subnormal
//! dimensions have sites here and both are applied: each decoded factor and the
//! running accumulator pass through
//! [`ReferenceNumericalConformance::apply_to_operand`] as they enter an
//! operation, and the product and every accumulation pass through
//! [`ReferenceNumericalConformance::apply_to_result`]. The contract's own NaN
//! canonicalization sits between them and commutes with both, because no NaN is
//! subnormal and no subnormal is a NaN.
//!
//! This is *separate* from the three permissions the signature decode verifies.
//! Those refuse a declaration whose result is a set rather than one value; the
//! subnormal modes name one function each and are realized rather than refused.
//! A contract resolving both to a flush and every permission to forbidden still
//! determines exactly one value, and it is not the value a preserving fold
//! computes.
//!
//! [`ReferenceNumericalConformance::apply_to_operand`]: crate::ReferenceNumericalConformance::apply_to_operand
//! [`ReferenceNumericalConformance::apply_to_result`]: crate::ReferenceNumericalConformance::apply_to_result
//!
//! # Two ways to walk one fold, and the two numbers that bound them
//!
//! `ContractionFold` holds everything a contraction is parameterized by once the
//! operands are validated, and two callers walk it. The registered operation
//! walks the whole result, in windows each of which passes the per-window work
//! bound, and refuses a fold larger than the iteration-step allowance its
//! evaluator carries. [`StagedStrictTensorContractionF32`] walks one output slab
//! per call, each under that same bound, which is how a caller with no evaluator
//! in the picture reaches a fold larger than one call may perform. Neither
//! carries its own arithmetic — that is what makes their agreement uninteresting
//! and their results interchangeable.
//!
//! The two numbers are deliberately not one. `MAX_REFERENCE_TENSOR_ELEMENTS`
//! bounds **one window**: the steps a single uninterrupted walk of the iteration
//! space may cost, and it does not move — a fold over it is spent as several
//! windows, never as one larger one.
//! [`ReferenceEvaluator::with_iteration_step_allowance`] states **how many steps
//! one occurrence may spend in total**, defaults to that same constant, and is
//! the caller's visible authorization for a fold that needs more than one window.
//! Collapsing them would either put the unbounded ask back or make the caller's
//! authorization silently widen what a single walk costs.
//!
//! [`ReferenceEvaluator::with_iteration_step_allowance`]: crate::ReferenceEvaluator::with_iteration_step_allowance

use std::collections::BTreeMap;
use std::fmt;

use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::{
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, ContractionF32ReductionDescriptor, ContractionF32Seed,
    ContractionIndex, ContractionIndexStructure, F32, FrozenSemanticRegistry, OperationAttributes,
    ResolvedValueType, tensor_contraction_f32_reduction_descriptor,
};
use tiler_ir::shape::{Extent, Shape};

use super::MAX_REFERENCE_TENSOR_ELEMENTS;
use super::conformance::ReferenceNumericalConformance;
use super::error::{
    ReferenceOperationError, StagedContractionError, UnsupportedContractionDeclaration,
    dense_result_error,
};
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
    /// Derives the strict-cell contract from the standard registry's governed
    /// `tiler::tensor-contraction-f32@1` definition, through the sole decoder.
    ///
    /// # Errors
    ///
    /// Returns the sole decoder's typed refusal. Unreachable through any
    /// frozen semantic registry, because the semantic registrar refuses to
    /// register an untyped governed contraction definition.
    pub(crate) fn governed() -> Result<Self, UnsupportedContractionDeclaration> {
        // A standard registry that cannot build leaves the governed definition
        // unreachable, which is the same refusal as an absent operation.
        let registry = FrozenSemanticRegistry::standard().map_err(|_| {
            UnsupportedContractionDeclaration::Descriptor(Box::new(
                tiler_ir::semantic::ContractionF32DescriptorError::OperationMissing {
                    operation: tiler_ir::semantic::tensor_contraction_f32_op(),
                },
            ))
        })?;
        Self::from_registry(&registry)
    }

    /// Derives the strict-cell contract from one semantic registry's reached
    /// governed definition, through the sole decoder.
    ///
    /// # Errors
    ///
    /// Returns the sole decoder's typed refusal for an absent or invalid
    /// governed definition.
    pub(crate) fn from_registry(
        registry: &FrozenSemanticRegistry,
    ) -> Result<Self, UnsupportedContractionDeclaration> {
        let descriptor = tensor_contraction_f32_reduction_descriptor(registry)
            .map_err(|source| UnsupportedContractionDeclaration::Descriptor(Box::new(source)))?;
        Ok(Self::from_descriptor(descriptor))
    }

    /// Projects the strict-cell fold parameters out of a decoded descriptor.
    ///
    /// Infallible by construction: holding a descriptor is evidence the sole
    /// decoder validated every fact, and the four values this fold reads — the
    /// accumulator type, the result type, the canonical arithmetic-NaN
    /// payload, and the seed — are all fixed by that validation.
    fn from_descriptor(descriptor: ContractionF32ReductionDescriptor) -> Self {
        let seed = match descriptor.seed() {
            ContractionF32Seed::FirstProduct => ContractionSeed::FirstProduct,
        };
        Self {
            accumulator_type: F32::resolved_type(),
            result_type: F32::resolved_type(),
            canonical_nan_bits: descriptor.canonical_nan_bits(),
            seed,
        }
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

/// The registered reference implementation of the governed contraction.
pub(crate) struct TensorContractionF32Reference {
    contract: ContractionContract,
}

impl TensorContractionF32Reference {
    pub(crate) const fn new(contract: ContractionContract) -> Self {
        Self { contract }
    }
}

impl ReferenceOperation for TensorContractionF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [left, right] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let structure = contraction_structure(request.attributes())?;
        contract_operands(
            &self.contract,
            &structure,
            left,
            right,
            request.iteration_step_allowance(),
            request.conformance_for(ArithmeticType::F32)?,
        )
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
    iteration_step_allowance: usize,
    conformance: ReferenceNumericalConformance,
) -> Result<Tensor, ReferenceOperationError> {
    let fold = ContractionFold::plan(contract, structure, left, right)?;
    // The fold performs `output_count * contracted_count` multiply-accumulate
    // steps, which is larger than either operand and is bounded by neither of the
    // tensor limits the operands already passed. Refused before the loop rather
    // than discovered inside it, and under the work bound's own variant: nothing
    // about the shapes is too large here, so `ShapeTooLarge` would send a reader
    // looking at the wrong quantity.
    //
    // The allowance is what this occurrence's *caller* authorized, and defaults to
    // `MAX_REFERENCE_TENSOR_ELEMENTS` — so an evaluator nobody told otherwise
    // refuses exactly the folds it always refused. What the allowance never
    // widens is one window: `evaluate_every_output` spends several windows each
    // passing the same per-window test [`StagedStrictTensorContractionF32`]
    // applies, never one larger walk.
    let steps = fold.steps();
    if steps > iteration_step_allowance {
        return Err(ReferenceOperationError::IterationStepsExceeded {
            limit: iteration_step_allowance,
            actual: steps,
        });
    }
    let results = fold.evaluate_every_output(contract, conformance)?;
    Tensor::dense(
        contract.result_type.clone(),
        fold.output_shape.clone(),
        results,
    )
    .map_err(|source| dense_result_error(&source))
}

/// One contraction's validated fold, with every per-axis decision resolved.
///
/// Extracted from [`contract_operands`] because two callers walk it: the
/// registered operation, which walks every output element in bounded windows the
/// fold itself sizes, and [`StagedStrictTensorContractionF32`], which walks one
/// output slab per call at a width its caller may choose. **One fold and not
/// two** is the point — a staged evaluator carrying its own arithmetic would
/// agree with the registered one for reasons that say nothing about either being
/// right, which is the independence rule this crate is built on stated in the
/// small. Both widths come from [`Self::window_output_count`] by default, so the
/// two callers are held to one number rather than to two that happen to agree.
struct ContractionFold<'operands> {
    /// Dense elements of each operand, in structure-operand order.
    elements: Vec<&'operands [ReferenceElement]>,
    /// Which coordinate each operand axis reads, in structure-operand order.
    readers: Vec<Vec<AxisReader>>,
    /// Row-major strides of each operand, in structure-operand order.
    operand_strides: Vec<Vec<usize>>,
    output_shape: Shape,
    output_strides: Vec<usize>,
    output_count: usize,
    contracted_shape: Shape,
    contracted_strides: Vec<usize>,
    contracted_count: usize,
}

impl<'operands> ContractionFold<'operands> {
    /// Validates the operands against the structure and resolves the fold.
    ///
    /// Everything here is decided before a single multiply-accumulate step: the
    /// operand types, the per-index extents, the output and contracted shapes,
    /// the axis readers, and the strides. Nothing in this function is
    /// proportional to the fold's step count, which is what lets a caller learn
    /// the cost of a fold without paying it.
    fn plan(
        contract: &ContractionContract,
        structure: &ContractionIndexStructure,
        left: &'operands Tensor,
        right: &'operands Tensor,
    ) -> Result<Self, ReferenceOperationError> {
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

        // Each operand axis reads either an output coordinate or a contracted one.
        // Resolved once per operand axis rather than per element, so the inner loop
        // is two strided reads and two roundings.
        let mut readers: Vec<Vec<AxisReader>> = Vec::with_capacity(operands.len());
        for tuple in structure.operands() {
            let mut operand_readers = Vec::with_capacity(tuple.len());
            for index in tuple {
                let reader = if let Some(position) =
                    structure.output().iter().position(|free| free == index)
                {
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

        Ok(Self {
            elements,
            readers,
            operand_strides,
            output_shape,
            output_strides,
            output_count,
            contracted_shape,
            contracted_strides,
            contracted_count,
        })
    }

    /// Multiply-accumulate steps the whole fold would walk.
    ///
    /// Saturating rather than checked, because a product too large for `usize`
    /// still has to refuse: `usize::MAX` exceeds every governed limit, so the
    /// saturated count reports a floor of the work rather than turning an
    /// unnameable one into a wrapped small number a loop would then walk.
    const fn steps(&self) -> usize {
        self.output_count.saturating_mul(self.contracted_count)
    }

    /// Output elements the widest window the per-window work bound admits covers.
    ///
    /// The division is defined because a zero contracted domain was refused during
    /// planning. It reaches zero only when a *single* output element's fold is
    /// already over the bound — the one case no windowing can reach, refused by
    /// [`Self::evaluate_every_output`] rather than divided out of existence.
    const fn window_output_count(&self) -> usize {
        MAX_REFERENCE_TENSOR_ELEMENTS / self.contracted_count
    }

    /// Folds every output element, in windows each under the per-window bound.
    ///
    /// The whole result is what a caller of the registered operation asked for, and
    /// how many windows it costs is this fold's own arithmetic rather than the
    /// caller's; what the caller authorized is the *total*, which
    /// [`contract_operands`] checked before calling here.
    ///
    /// A fold that fits one window takes the single-call path, so nothing that
    /// evaluated before this windowing existed pays an extra traversal or an extra
    /// copy of its result.
    ///
    /// Why concatenating the windows is the unstaged result and not an
    /// approximation of it is [`StagedStrictTensorContractionF32`]'s "why a slab
    /// boundary cannot change a folded value" argument, read out of the registered
    /// signature; the windows below are the same partition of the same
    /// [`Self::evaluate_outputs`] that argument was checked against.
    fn evaluate_every_output(
        &self,
        contract: &ContractionContract,
        conformance: ReferenceNumericalConformance,
    ) -> Result<Vec<ReferenceElement>, ReferenceOperationError> {
        let window = self.window_output_count();
        if window == 0 {
            // One output element's own fold is over the bound, so no partition of
            // the result is admissible and there is nothing to narrow toward.
            // Unreachable through well-formed tensors — ADR 0087's rule two puts
            // every contracted index in both operands, so `contracted_count`
            // divides each operand's element count, which `Tensor::dense` already
            // bounded by this same constant. Refused rather than assumed away, and
            // a reservation rather than a tested guarantee.
            return Err(ReferenceOperationError::IterationStepsExceeded {
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual: self.contracted_count,
            });
        }
        if window >= self.output_count {
            return self.evaluate_outputs(contract, conformance, 0, self.output_count);
        }
        let mut results = Vec::with_capacity(self.output_count);
        let mut first_output = 0_usize;
        while first_output < self.output_count {
            let outputs = window.min(self.output_count - first_output);
            results.extend(self.evaluate_outputs(contract, conformance, first_output, outputs)?);
            first_output += outputs;
        }
        Ok(results)
    }

    /// Folds `outputs` consecutive output elements, starting at `first_output`.
    ///
    /// The window is a window on the *result*, never on any fold: every element
    /// this produces walks its own complete contracted sequence, in the declared
    /// order, seeded by its own first product. See
    /// [`StagedStrictTensorContractionF32`] for why that makes the window
    /// unobservable in the values.
    fn evaluate_outputs(
        &self,
        contract: &ContractionContract,
        conformance: ReferenceNumericalConformance,
        first_output: usize,
        outputs: usize,
    ) -> Result<Vec<ReferenceElement>, ReferenceOperationError> {
        let end = first_output
            .checked_add(outputs)
            .ok_or(ReferenceOperationError::InvalidApplication)?;
        if end > self.output_count {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let mut output_coordinate = vec![0_usize; self.output_shape.rank()];
        let mut contracted_coordinate = vec![0_usize; self.contracted_shape.rank()];
        let mut results = Vec::with_capacity(outputs);
        for output_linear in first_output..end {
            decode_coordinate(
                output_linear,
                &self.output_shape,
                &self.output_strides,
                &mut output_coordinate,
            )?;
            let mut accumulator = match contract.seed {
                ContractionSeed::FirstProduct => None,
                ContractionSeed::Initial(initial) => Some(initial),
            };
            for contracted_linear in 0..self.contracted_count {
                decode_coordinate(
                    contracted_linear,
                    &self.contracted_shape,
                    &self.contracted_strides,
                    &mut contracted_coordinate,
                )?;
                let mut factors = [0.0_f32; 2];
                for (position, factor) in factors.iter_mut().enumerate() {
                    let offset = self.readers[position]
                        .iter()
                        .zip(&self.operand_strides[position])
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
                    let element = self.elements[position]
                        .get(offset)
                        .ok_or(ReferenceOperationError::InvalidApplication)?;
                    *factor = conformance.apply_to_operand(decode_f32(element)?);
                }
                // One rounding for the product and one for the accumulation, each
                // canonicalized and each committed through the declared result
                // dimension: the fused single-rounding form is the permission this
                // family declares forbidden, and the accumulator re-enters the add
                // as an operand, so the input dimension applies to it too.
                let product =
                    conformance.apply_to_result(contract.canonicalize(factors[0] * factors[1]));
                accumulator = Some(match accumulator {
                    None => product,
                    Some(value) => conformance.apply_to_result(contract.canonicalize(
                        conformance.apply_to_operand(value) + conformance.apply_to_operand(product),
                    )),
                });
            }
            let value = accumulator.ok_or(ReferenceOperationError::InvalidApplication)?;
            results.push(f32_element(contract.canonicalize(value))?);
        }
        Ok(results)
    }
}

/// The governed contraction folded one admitted output slab at a time.
///
/// # What this is for, and what it deliberately does not change
///
/// The fold behind [`ReferenceEvaluator`] refuses an occurrence of more than the
/// evaluator's iteration-step allowance, which is `MAX_REFERENCE_TENSOR_ELEMENTS`
/// multiply-accumulate steps unless a caller states otherwise. That bound is not
/// about storage — the fold's memory is `output_count` elements whatever the step
/// count — it is the one thing standing between a malformed program and an
/// unbounded ask on host *time*. Four of the six correctness cells of the L3
/// language-model contraction profile exceed it: `w_prefill_q` at 20,971,520
/// steps, `w_prefill_o` at 268,435,456, and `w_prefill_mlp_in` and
/// `w_prefill_mlp_out` at 402,653,184 each.
///
/// This type reaches those cells **without moving that bound by one step**, and
/// with no evaluator, program, or registry in the picture. Each
/// [`Self::evaluate_slab`] call folds a slab of output elements whose work passes
/// exactly the per-window test, and the total is a loop the caller writes. So a
/// program handed to a default [`ReferenceEvaluator`] still refuses at the same
/// limit it always did, and the extra work here is authorized by visible caller
/// code rather than by a constant nobody re-derives — which is the same
/// authorization
/// [`ReferenceEvaluator::with_iteration_step_allowance`] expresses as a stated
/// number for a caller who does have a whole program in hand.
///
/// [`ReferenceEvaluator`]: crate::ReferenceEvaluator
/// [`ReferenceEvaluator::with_iteration_step_allowance`]: crate::ReferenceEvaluator::with_iteration_step_allowance
///
/// # Why a slab boundary cannot change a folded value
///
/// This is the load-bearing claim, and it is read out of the registered
/// signature — through the sole decoder,
/// [`ContractionF32ReductionDescriptor`] — rather than assumed from the shape
/// of the loop.
///
/// - The declared contributor sequence is
///   `ascending-lexicographic-over-the-canonically-ordered-contracted-index-space`.
///   The sequence one output element folds therefore ranges over the *contracted*
///   index space alone. By [`ContractionIndexStructure`]'s own derivation the
///   contracted set is exactly the operand indices absent from the output tuple,
///   so no output coordinate is a member of any contributor sequence and no
///   output element appears in another's.
/// - The declared seed is
///   `none-the-accumulator-starts-at-the-first-product`. Each fold's initial
///   value is its own first product, so no accumulator state is carried from one
///   output element to the next. (The declarable explicit-initial alternative
///   would be a declared constant, not a carried value, so the conclusion does
///   not depend on which seed is declared.)
/// - This staged fold computes the strict cell, whose effective reassociation
///   and permutation are both false, and slabbing exercises neither: it
///   regroups nothing and reorders nothing *within* a contributor sequence.
///   What it reorders is the traversal of whole output elements, and the
///   signature's thirteen fields declare no output traversal order — there is
///   no term for one to violate.
/// - The declared stability record binds ADR 0013 plan determinism: the value
///   is a function of the declared plan, and a slab width is not a plan term.
///
/// The mechanical statement matching that reading: `evaluate_outputs` reads only
/// the immutable operand elements and writes only the result it returns, and
/// re-seeds its accumulator inside the output loop. The signature is the
/// authority; the code is what was checked against it.
///
/// `slab_boundaries_do_not_change_any_folded_value` in
/// `tests/contraction_profile_cells.rs` is the executable half: several slab
/// widths, and the unstaged [`ReferenceEvaluator`], all produce the identical bit
/// patterns.
///
/// # What one slab costs
///
/// One slab is bounded at `MAX_REFERENCE_TENSOR_ELEMENTS` steps, and the whole
/// result at the `output_count` elements `preflight_f32_output` already admitted,
/// so peak host memory is the operands plus the assembled result and does not
/// grow with the number of slabs.
///
/// Time does. This is an exact scalar oracle with a coordinate decode per step,
/// measured at 9 ns per step in the dev profile and 4 ns in the release profile
/// (Apple M4 Max, 2026-08-01, `tests/contraction_profile_cells.rs`), so the
/// largest L3 cell's 402,653,184 steps cost 3.8 seconds and 1.9 seconds
/// respectively, and the whole six-cell profile 10.8 seconds and 5.5 seconds at a
/// 484 MB peak resident set. That is a measurement of this host and this profile,
/// not a rate any other fold inherits. A caller putting a large cell in a default
/// test run is choosing to pay it on every run, which is why
/// `tests/contraction_profile_cells.rs` runs the two cheapest cells by default and
/// marks the whole-profile comparison `#[ignore]` with its invocation recorded.
pub struct StagedStrictTensorContractionF32<'operands> {
    contract: ContractionContract,
    fold: ContractionFold<'operands>,
    slab_output_count: usize,
    conformance: ReferenceNumericalConformance,
}

impl<'operands> StagedStrictTensorContractionF32<'operands> {
    /// Plans a staged fold with the widest slab the work bound admits.
    ///
    /// The contract is the governed `tiler::tensor-contraction-f32@1`
    /// signature's strict cell, decoded by the same sole-decoder reading the
    /// registered operation is parameterized by — not a second reading of it.
    ///
    /// # Errors
    ///
    /// Returns [`StagedContractionError::UnsupportedDeclaration`] when the
    /// governed signature declares a contract this reference does not realize,
    /// and [`StagedContractionError::Operation`] when the operands do not match
    /// the structure or no slab of the fold is admissible.
    pub fn governed(
        structure: &ContractionIndexStructure,
        left: &'operands Tensor,
        right: &'operands Tensor,
    ) -> Result<Self, StagedContractionError> {
        let contract = ContractionContract::governed()
            .map_err(StagedContractionError::UnsupportedDeclaration)?;
        let fold = ContractionFold::plan(&contract, structure, left, right)
            .map_err(StagedContractionError::Operation)?;
        let slab_output_count = fold.window_output_count();
        Self::admitted(contract, fold, slab_output_count)
    }

    /// Plans a staged fold with an explicit slab width.
    ///
    /// For a caller that wants finer slabs than the widest admitted one —
    /// smaller peaks, more frequent progress, or a comparison across widths. A
    /// width *wider* than the bound admits is refused rather than narrowed,
    /// because silently narrowing it would make the bound unobservable to the
    /// caller who asked to exceed it.
    ///
    /// # Errors
    ///
    /// As [`Self::governed`], and additionally
    /// [`ReferenceOperationError::IterationStepsExceeded`] when one slab of
    /// `slab_output_count` output elements would walk more steps than the work
    /// bound admits, or [`ReferenceOperationError::InvalidApplication`] for a
    /// zero width.
    pub fn governed_with_slab_output_count(
        structure: &ContractionIndexStructure,
        left: &'operands Tensor,
        right: &'operands Tensor,
        slab_output_count: usize,
    ) -> Result<Self, StagedContractionError> {
        let contract = ContractionContract::governed()
            .map_err(StagedContractionError::UnsupportedDeclaration)?;
        let fold = ContractionFold::plan(&contract, structure, left, right)
            .map_err(StagedContractionError::Operation)?;
        Self::admitted(contract, fold, slab_output_count)
    }

    /// Admits a slab width against the same bound the unstaged fold is held to.
    fn admitted(
        contract: ContractionContract,
        fold: ContractionFold<'operands>,
        slab_output_count: usize,
    ) -> Result<Self, StagedContractionError> {
        if slab_output_count == 0 {
            // Reached two ways, and refused rather than resolved either way. A
            // caller may ask for a zero-wide slab, which partitions nothing. Or
            // `governed` may divide to zero, which needs a contracted space
            // larger than the work bound — the fold of a *single* output
            // element already over the limit, so no staging is admissible. The
            // second is unreachable through well-formed tensors: ADR 0087's rule
            // two puts every contracted index in both operands, so
            // `contracted_count` divides each operand's element count, which
            // `Tensor::dense` already bounded by this same constant. It is
            // refused rather than assumed away, and is a reservation rather than
            // a tested guarantee.
            return Err(StagedContractionError::Operation(
                ReferenceOperationError::InvalidApplication,
            ));
        }
        let steps = slab_output_count.saturating_mul(fold.contracted_count);
        if steps > MAX_REFERENCE_TENSOR_ELEMENTS {
            return Err(StagedContractionError::Operation(
                ReferenceOperationError::IterationStepsExceeded {
                    limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                    actual: steps,
                },
            ));
        }
        Ok(Self {
            contract,
            fold,
            slab_output_count,
            conformance: ReferenceNumericalConformance::strict(),
        })
    }

    /// Returns this staged fold performed under one stated numerical contract.
    ///
    /// The planned fold is unchanged — the split, the slab width, the contributor
    /// sequence, and the work bounds are all decisions the contract does not
    /// touch — and what moves is the arithmetic each step performs. A caller
    /// qualifying a candidate compiled under a flushing realization states it
    /// here; a caller that states nothing gets the strict reading, which is what
    /// [`Self::governed`] computed before it could be told anything.
    #[must_use]
    pub fn under(self, conformance: ReferenceNumericalConformance) -> Self {
        Self {
            conformance,
            ..self
        }
    }

    /// Returns the numerical contract every slab is folded under.
    #[must_use]
    pub const fn conformance(&self) -> ReferenceNumericalConformance {
        self.conformance
    }

    /// Returns the shape the assembled result carries.
    #[must_use]
    pub const fn output_shape(&self) -> &Shape {
        &self.fold.output_shape
    }

    /// Returns how many elements the assembled result holds.
    #[must_use]
    pub const fn output_count(&self) -> usize {
        self.fold.output_count
    }

    /// Returns how many contributors each output element's fold walks.
    #[must_use]
    pub const fn contracted_count(&self) -> usize {
        self.fold.contracted_count
    }

    /// Returns the output elements one slab folds, except a shorter final slab.
    #[must_use]
    pub const fn slab_output_count(&self) -> usize {
        self.slab_output_count
    }

    /// Returns how many slabs cover the result.
    #[must_use]
    pub const fn slab_count(&self) -> usize {
        self.fold.output_count.div_ceil(self.slab_output_count)
    }

    /// Folds one slab, returning its output elements in row-major order.
    ///
    /// Concatenating the slabs in index order yields exactly the dense payload
    /// the unstaged fold would have produced for the same operands.
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceOperationError::InvalidApplication`] for a slab index
    /// at or past [`Self::slab_count`], and otherwise whatever the fold reports.
    pub fn evaluate_slab(
        &self,
        slab: usize,
    ) -> Result<Vec<ReferenceElement>, ReferenceOperationError> {
        if slab >= self.slab_count() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let first_output = slab
            .checked_mul(self.slab_output_count)
            .ok_or(ReferenceOperationError::InvalidApplication)?;
        let outputs = self
            .slab_output_count
            .min(self.fold.output_count - first_output);
        self.fold
            .evaluate_outputs(&self.contract, self.conformance, first_output, outputs)
    }
}

/// The plan, never the operands.
///
/// Hand-written rather than derived because a derived form would render every
/// borrowed operand element — hundreds of megabytes at the profile's larger
/// cells, for a value a reader wants four numbers from.
impl fmt::Debug for StagedStrictTensorContractionF32<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StagedStrictTensorContractionF32")
            .field("output_shape", self.output_shape())
            .field("output_count", &self.output_count())
            .field("contracted_count", &self.contracted_count())
            .field("slab_output_count", &self.slab_output_count())
            .field("slab_count", &self.slab_count())
            .field("conformance", &self.conformance)
            .finish()
    }
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

pub(crate) mod topology;

#[cfg(test)]
mod tests;
