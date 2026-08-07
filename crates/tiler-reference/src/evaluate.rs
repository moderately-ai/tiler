//! The semantic-program evaluation authority.
//!
//! [`ReferenceEvaluator`] walks a verified `SemanticProgram` and produces
//! reference tensors by dispatching each operation through the frozen
//! reference registry. It is one of the crate's two independent oracles;
//! `oracle` owns the other, over index regions, and neither reuses the
//! other's host expressions.
//!
//! # The declared contract reaches every capability, and refusing to carry it
//!
//! An evaluator carries one [`ReferenceNumericalConformance`] and hands it to
//! every capability through [`ReferenceEvaluationRequest::conformance_for`],
//! which refuses a capability whose own arithmetic type is not the one the
//! conformance was resolved for.
//! [`ReferenceEvaluator::new`] states the strict reading and
//! [`ReferenceEvaluator::under`] states another, exactly as
//! [`crate::IndexRegionEvaluator`] does — the two oracles answer the same program
//! and are told the same contract, so neither can silently answer a question the
//! other was asked.
//!
//! The declared-order reduction oracle below carries it the same way:
//! [`strict_partial_sums_under`] and [`strict_partitioned_sum_under`] take the
//! contract, and [`strict_partial_sums`] and [`strict_partitioned_sum`] are those
//! two at the strict reading rather than a second fold that ignores one.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValueView, Definition, F32, OperationId,
    REDUCTION_AXES_ATTRIBUTE, ResolvedValueType, SemanticProgram, ValueId,
};
use tiler_ir::shape::{Axis, Shape};

use super::conformance::ReferenceNumericalConformance;
use super::error::{
    EvaluationError, ReferenceOperationError, ReferenceRegistryError, ReferenceResource,
    dense_result_error,
};
use super::registry::{
    FrozenReferenceRegistry, ReferenceEvaluationRequest, ReferenceOutputs, ReferenceSignature,
};
use super::tensor::{
    FloatBitOrder, InputBinding, ReferenceComponent, ReferenceElement, Tensor, TensorPayloadView,
};
use super::value_conformance::ValueConformanceLedger;
use super::{
    MAX_REFERENCE_COMPONENT_DEPTH, MAX_REFERENCE_COMPONENTS, MAX_REFERENCE_TENSOR_BYTES,
    MAX_REFERENCE_TENSOR_ELEMENTS, canonicalize_arithmetic_f32,
};

/// Host evaluator for the bounded semantic profile.
#[derive(Clone, Debug)]
pub struct ReferenceEvaluator {
    registry: FrozenReferenceRegistry,
    iteration_step_allowance: usize,
    conformance: ReferenceNumericalConformance,
}

impl ReferenceEvaluator {
    /// Creates an evaluator with one explicit frozen capability snapshot,
    /// evaluating the strict reading.
    ///
    /// The iteration-step allowance is the crate's own per-window work bound,
    /// which is the number one operation could always walk, so this constructor
    /// changes no result and no refusal. Use
    /// [`Self::with_iteration_step_allowance`] to state a different one, and
    /// [`Self::iteration_step_allowance`] to read the one in force.
    ///
    /// The strict reading is what this evaluator computed before it could be told
    /// a contract, so naming it here changes no value either. Use [`Self::under`]
    /// to evaluate a program whose declared realization flushes subnormals;
    /// [`ReferenceNumericalConformance::from_realization`] is the checked bridge
    /// from a region's declared realization to the contract this carries.
    #[must_use]
    pub const fn new(registry: FrozenReferenceRegistry) -> Self {
        Self::under(registry, ReferenceNumericalConformance::strict())
    }

    /// Creates an evaluator bound to one stated numerical contract.
    ///
    /// Every capability the walk dispatches to receives this through
    /// [`ReferenceEvaluationRequest::conformance_for`], naming its own arithmetic
    /// type. A capability whose family can reach a subnormal operand or produce a
    /// subnormal result applies the two dimensions — over binary32 or over its own
    /// value set, which are two ways of discharging one obligation — and one that
    /// performs no arithmetic at all documents why it cannot at its own definition.
    /// Neither answers by omission, which is what makes a comparison against this
    /// evaluator a comparison against the *declared* contract rather than against
    /// the host's own subnormal behaviour.
    #[must_use]
    pub const fn under(
        registry: FrozenReferenceRegistry,
        conformance: ReferenceNumericalConformance,
    ) -> Self {
        Self {
            registry,
            iteration_step_allowance: MAX_REFERENCE_TENSOR_ELEMENTS,
            conformance,
        }
    }

    /// Returns the numerical contract every evaluation is performed under.
    #[must_use]
    pub const fn conformance(&self) -> ReferenceNumericalConformance {
        self.conformance
    }

    /// Creates an evaluator using Tiler's governed initial reference profile.
    ///
    /// # Errors
    ///
    /// Returns a typed registry construction error.
    pub fn standard() -> Result<Self, ReferenceRegistryError> {
        FrozenReferenceRegistry::standard().map(Self::new)
    }

    /// Returns this evaluator with one stated per-occurrence iteration-step
    /// allowance.
    ///
    /// # What this authorizes, and what it deliberately cannot
    ///
    /// An operation's *work* is not answerable from its result: a contraction of
    /// `[10, 2048]` from `[10, 1024]` and `[2048, 1024]` retains 20,480 elements
    /// and walks 20,971,520 multiply-accumulate steps, which neither operand bound
    /// nor result bound describes. The reference therefore holds one occurrence to
    /// a step count, and by default that count is the crate's per-window work
    /// bound — the one thing standing between a malformed program and an unbounded
    /// ask on host time. [`Self::iteration_step_allowance`] reports it rather than
    /// this sentence: it is 16,777,216 today, and the accessor is the authority.
    ///
    /// A caller that has decided to pay for a larger fold states the number here.
    /// **The stated number is the authorization**, in visible caller code, which is
    /// the same discipline [`crate::StagedStrictTensorContractionF32`] expresses as
    /// a loop the caller writes: neither reaches a larger fold by moving a constant
    /// nobody re-derives. What the allowance never does is widen what *one* walk of
    /// the iteration space may cost — an occurrence over the per-window bound is
    /// folded in windows each of which passes exactly the test a single-window fold
    /// passes, so raising this number buys more bounded windows and never a larger
    /// one.
    ///
    /// The whole program's total is still this number times the occurrence count,
    /// exactly as it was when the number was fixed; the allowance moves where the
    /// per-occurrence limit comes from and not what kind of limit it is.
    ///
    /// A value below the default narrows the evaluator rather than widening it,
    /// which is a legitimate ask — a caller that wants an expensive program
    /// declined early states a smaller number. A zero allowance declines every
    /// fold, including a one-step one.
    #[must_use]
    pub fn with_iteration_step_allowance(self, allowance: usize) -> Self {
        Self {
            registry: self.registry,
            iteration_step_allowance: allowance,
            conformance: self.conformance,
        }
    }

    /// Returns the iteration steps one operation occurrence may walk.
    #[must_use]
    pub const fn iteration_step_allowance(&self) -> usize {
        self.iteration_step_allowance
    }

    /// Returns the exact capability snapshot used for evaluation.
    #[must_use]
    pub const fn registry(&self) -> &FrozenReferenceRegistry {
        &self.registry
    }

    /// Evaluates every ordered program output without fusing semantic nodes.
    ///
    /// Bindings must match the program's ordered keys exactly. Separate
    /// multiply and add nodes produce separate f32 operations. Sum is a strict
    /// left fold over canonical contributor order and starts with the first
    /// contributor; an empty contributor sequence produces positive zero.
    ///
    /// # Errors
    ///
    /// Returns an [`EvaluationError`] for mismatched input arity, key, shape,
    /// or payload, or if private verified-program invariants are violated.
    pub fn evaluate(
        &self,
        program: &SemanticProgram,
        inputs: &[InputBinding<'_>],
    ) -> Result<Vec<Tensor>, EvaluationError> {
        let (mut values, mut retained_work, mut conformance) = self.bind_inputs(program, inputs)?;

        let reachable_operations = reachable_operations(program)?;
        for operation in program
            .operations()
            .filter(|operation| reachable_operations.contains(&operation.id()))
        {
            let operands: Vec<_> = operation.operands().collect();
            let results: Vec<_> = operation.results().collect();
            let signature = ReferenceSignature::new(
                operands
                    .iter()
                    .map(|value| resolved_type(program, *value))
                    .collect::<Result<Vec<_>, _>>()?,
                results
                    .iter()
                    .map(|value| resolved_type(program, *value))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|source| EvaluationError::ReferenceRegistry(Arc::new(source)))?;
            let capability = self.registry.resolve(
                operation.key(),
                &signature,
                operation.attributes(),
                program.semantic_registry(),
            )?;
            let operand_values = operands
                .iter()
                .map(|value| get_value(&values, *value))
                .collect::<Result<Vec<_>, _>>()?;
            let mut output_writer = ReferenceOutputs::new(results.len(), retained_work.clone());
            let callback = capability.implementation.evaluate(
                ReferenceEvaluationRequest {
                    operands: &operand_values,
                    attributes: operation.attributes(),
                    iteration_step_allowance: self.iteration_step_allowance,
                    conformance: self.conformance,
                },
                &mut output_writer,
            );
            let evaluated =
                output_writer
                    .finish(callback)
                    .map_err(|source| EvaluationError::Operation {
                        operation: operation.key().clone(),
                        provider: Arc::new(capability.provider.clone()),
                        capability_revision: capability.revision,
                        source,
                    })?;
            for (result_index, (result, evaluated)) in
                results.into_iter().zip(evaluated).enumerate()
            {
                let expected_shape = program
                    .shape(result)
                    .map_err(|_| EvaluationError::MalformedProgram)?;
                if expected_shape != evaluated.shape() {
                    return Err(EvaluationError::ResultShape {
                        operation: operation.key().clone(),
                        provider: Arc::new(capability.provider.clone()),
                        capability_revision: capability.revision,
                        result_index,
                        expected: Arc::new(expected_shape.clone()),
                        actual: Arc::new(evaluated.shape().clone()),
                    });
                }
                let expected_type = resolved_type(program, result)?;
                if evaluated.resolved_type() != &expected_type {
                    return Err(EvaluationError::ResultType {
                        operation: operation.key().clone(),
                        provider: Arc::new(capability.provider.clone()),
                        capability_revision: capability.revision,
                        result_index,
                        expected: Arc::new(expected_type),
                        actual: Arc::new(evaluated.resolved_type().clone()),
                    });
                }
                // A value this evaluator produced is proved by composing the
                // producer's verified semantics, so it is not rescanned. Only a
                // result no admitted composition rule covers falls through to
                // the registered representation validator, which is the
                // authority for exactly those.
                let composed = conformance.produce_result(
                    program,
                    operation,
                    result,
                    &expected_type,
                    expected_shape,
                )?;
                if !composed {
                    self.registry
                        .validate_value(&evaluated, program.semantic_registry())?;
                }
                reserve_evaluation_work(&mut retained_work, &evaluated)?;
                values.insert(result, evaluated);
            }
        }

        program
            .outputs()
            .map(|output| get_value(&values, output.value()).cloned())
            .collect()
    }

    fn bind_inputs(
        &self,
        program: &SemanticProgram,
        inputs: &[InputBinding<'_>],
    ) -> Result<
        (
            HashMap<ValueId, Tensor>,
            EvaluationRetention,
            ValueConformanceLedger,
        ),
        EvaluationError,
    > {
        if inputs.len() != program.input_count() {
            return Err(EvaluationError::InputCount {
                expected: program.input_count(),
                actual: inputs.len(),
            });
        }

        let mut values = HashMap::with_capacity(program.value_count());
        let mut retained_work = EvaluationRetention::default();
        let mut conformance = ValueConformanceLedger::default();
        for (index, (declaration, binding)) in program.inputs().zip(inputs).enumerate() {
            if declaration.key() != binding.key {
                return Err(EvaluationError::InputKey {
                    input_index: index,
                    expected: declaration.key().clone(),
                    actual: binding.key.clone(),
                });
            }
            let expected = program
                .shape(declaration.value())
                .map_err(|_| EvaluationError::MalformedProgram)?;
            if binding.tensor.shape() != expected {
                return Err(EvaluationError::InputShape {
                    key: declaration.key().clone(),
                    expected: expected.clone(),
                    actual: binding.tensor.shape().clone(),
                });
            }
            let expected_type = resolved_type(program, declaration.value())?;
            if binding.tensor.resolved_type() != &expected_type {
                return Err(EvaluationError::InputType {
                    key: declaration.key().clone(),
                    expected: Arc::new(expected_type),
                    actual: Arc::new(binding.tensor.resolved_type().clone()),
                });
            }
            // A directly bound value has no producing occurrence, so its type is
            // what governs it: the binding validator scans its authoritative
            // logical view and mints the proof every later composition reads.
            // A type it does not govern falls through to the registered
            // representation validator, which is the authority for exactly
            // those; a governed one is not scanned twice, because both paths
            // reach the same obligation set.
            if !conformance.bind_input(
                program,
                declaration.key(),
                declaration.value(),
                binding.tensor,
            )? {
                self.registry
                    .validate_value(binding.tensor, program.semantic_registry())?;
            }
            reserve_evaluation_work(&mut retained_work, binding.tensor)?;
            values.insert(declaration.value(), binding.tensor.clone());
        }
        Ok((values, retained_work, conformance))
    }
}

fn resolved_type(
    program: &SemanticProgram,
    value: ValueId,
) -> Result<ResolvedValueType, EvaluationError> {
    program
        .value(value)
        .map(|value| value.resolved_type().clone())
        .map_err(|_| EvaluationError::MalformedProgram)
}

pub(crate) fn reduction_axes(
    attributes: &tiler_ir::semantic::OperationAttributes,
) -> Result<Vec<Axis>, ReferenceOperationError> {
    let Some(CanonicalValueView::Sequence(values)) = attributes
        .get(REDUCTION_AXES_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned { width, bits } = value.view() else {
                return Err(ReferenceOperationError::InvalidApplication);
            };
            if width != CanonicalIntegerWidth::Bits32 {
                return Err(ReferenceOperationError::InvalidApplication);
            }
            u32::try_from(bits)
                .map(Axis::new)
                .map_err(|_| ReferenceOperationError::InvalidApplication)
        })
        .collect()
}

/// Evaluates one governed elementwise binary32 operation under a stated contract.
///
/// Each decoded operand passes through
/// [`ReferenceNumericalConformance::apply_to_operand`] before the host operation
/// and the produced value through
/// [`ReferenceNumericalConformance::apply_to_result`] after it, which is the
/// scalar oracle's own composition over a tensor rather than a second reading of
/// it. The NaN canonicalization sits between them and commutes with both: no NaN
/// is subnormal and no subnormal is a NaN, so neither ordering is a choice.
pub(crate) fn binary(
    left_value: &Tensor,
    right_value: &Tensor,
    conformance: ReferenceNumericalConformance,
    operation: impl Fn(f32, f32) -> f32,
) -> Result<Tensor, ReferenceOperationError> {
    let left_elements = f32_elements(left_value)?;
    let right_elements = f32_elements(right_value)?;
    let result_shape = if left_value.shape().rank() == 0 {
        right_value.shape()
    } else {
        left_value.shape()
    };
    let count = result_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    let elements = (0..count)
        .map(|index| {
            let left = if left_value.shape().rank() == 0 {
                decode_f32(&left_elements[0])?
            } else {
                decode_f32(&left_elements[index])?
            };
            let right = if right_value.shape().rank() == 0 {
                decode_f32(&right_elements[0])?
            } else {
                decode_f32(&right_elements[index])?
            };
            let left = conformance.apply_to_operand(left);
            let right = conformance.apply_to_operand(right);
            f32_element(
                conformance.apply_to_result(canonicalize_arithmetic_f32(operation(left, right))),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Tensor::dense(F32::resolved_type(), result_shape.clone(), elements)
        .map_err(|source| dense_result_error(&source))
}

/// Evaluates the strict serial fold under a stated contract.
///
/// **The contract reaches the fold's arithmetic and nothing else.** Each
/// contributor and the running accumulator pass through
/// [`ReferenceNumericalConformance::apply_to_operand`] as they enter an addition
/// and each sum through [`ReferenceNumericalConformance::apply_to_result`]; a
/// partition of one contributor performs no addition, so it reaches neither and
/// its value is committed as it was read. That boundary is not this function's
/// invention — it is where this crate's arithmetic NaN canonicalization is
/// already drawn, and applying a flush to a value nothing computed would model a
/// device flushing a load rather than an arithmetic unit.
pub(crate) fn strict_sum(
    input: &Tensor,
    axes: &[Axis],
    conformance: ReferenceNumericalConformance,
) -> Result<Tensor, ReferenceOperationError> {
    let mut reduced_mask = vec![false; input.shape().rank()];
    let mut reduced = Vec::with_capacity(axes.len());
    for requested_axis in axes {
        let dimension = usize::try_from(requested_axis.get())
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let Some(is_reduced) = reduced_mask.get_mut(dimension) else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if std::mem::replace(is_reduced, true) {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        reduced.push(dimension);
    }
    let survivor: Vec<usize> = (0..input.shape().rank())
        .filter(|axis| !reduced_mask[*axis])
        .collect();
    let output_shape = Shape::try_new(survivor.iter().map(|axis| input.shape().extents()[*axis]))
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
    let output_count = output_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    preflight_f32_output(output_count)?;
    if output_count == 0 {
        return Tensor::dense(F32::resolved_type(), output_shape, Vec::new())
            .map_err(|source| dense_result_error(&source));
    }
    let input_elements = f32_elements(input)?;
    let reduced_shape = Shape::try_new(reduced.iter().map(|axis| input.shape().extents()[*axis]))
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
    let reduced_count = reduced_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    if reduced_count == 0 {
        let zero = f32_element(0.0_f32)?;
        return Tensor::dense(F32::resolved_type(), output_shape, vec![zero; output_count])
            .map_err(|source| dense_result_error(&source));
    }
    let input_strides = row_major_strides(input.shape())?;
    let output_strides = row_major_strides(&output_shape)?;
    let reduced_strides = row_major_strides(&reduced_shape)?;
    let mut elements = Vec::with_capacity(output_count);
    let mut output_coordinate = vec![0_usize; output_shape.rank()];
    let mut reduced_coordinate = vec![0_usize; reduced_shape.rank()];
    let mut input_coordinate = vec![0_usize; input.shape().rank()];

    for output_linear in 0..output_count {
        decode_coordinate(
            output_linear,
            &output_shape,
            &output_strides,
            &mut output_coordinate,
        )?;
        let mut accumulator = None;
        for reduced_linear in 0..reduced_count {
            decode_coordinate(
                reduced_linear,
                &reduced_shape,
                &reduced_strides,
                &mut reduced_coordinate,
            )?;
            input_coordinate.fill(0);
            for (coordinate, axis) in output_coordinate.iter().zip(&survivor) {
                input_coordinate[*axis] = *coordinate;
            }
            for (coordinate, axis) in reduced_coordinate.iter().zip(&reduced) {
                input_coordinate[*axis] = *coordinate;
            }
            let linear = input_coordinate
                .iter()
                .zip(&input_strides)
                .map(|(coordinate, stride)| coordinate * stride)
                .sum::<usize>();
            let contributor = decode_f32(&input_elements[linear])?;
            accumulator = Some(match accumulator {
                None => contributor,
                Some(value) => conformance.apply_to_result(canonicalize_arithmetic_f32(
                    conformance.apply_to_operand(value) + conformance.apply_to_operand(contributor),
                )),
            });
        }
        elements.push(f32_element(canonicalize_arithmetic_f32(
            accumulator.unwrap_or(0.0_f32),
        ))?);
    }
    Tensor::dense(F32::resolved_type(), output_shape, elements)
        .map_err(|source| dense_result_error(&source))
}

/// Evaluates the partial values one pass of a split reduction must produce.
///
/// The split is the one a multi-pass schedule declares: partition `p` combines
/// the contiguous contributor range
/// `p * contributors_per_partition .. (p + 1) * contributors_per_partition` of
/// the same original-axis lexicographic sequence the strict serial sum folds,
/// and the
/// result carries one value per `(output position, partition)` pair with the
/// partition as the innermost axis.
///
/// This is a *second* exact oracle rather than a relaxation of the first. A
/// contract that permits reassociation admits a set of results, so no oracle
/// can answer "the" value for it; what a plan can be checked against is the one
/// order it selected, and this evaluates exactly that order.
///
/// # Errors
///
/// Returns [`ReferenceOperationError::InvalidApplication`] when the axes are
/// not a canonical in-range set or the split does not cover the contributor
/// sequence exactly once each, and a resource error when the staged partial
/// tensor exceeds the reference bounds.
pub fn strict_partial_sums(
    input: &Tensor,
    axes: &[Axis],
    partitions: u64,
    contributors_per_partition: u64,
) -> Result<Tensor, ReferenceOperationError> {
    strict_partial_sums_under(
        input,
        axes,
        partitions,
        contributors_per_partition,
        ReferenceNumericalConformance::strict(),
    )
}

/// Evaluates [`strict_partial_sums`] under one stated numerical contract.
///
/// **The declared order and the declared subnormal modes are independent
/// obligations, and a plan under a permissive contract has both.**
/// `FLUSH_AND_REASSOCIATE_F32` resolves `reassociation` to permitted *and* both
/// subnormal dimensions to a sign-preserving flush; the split argument discharges
/// the first and this argument discharges the second. An oracle given only the
/// split answers the preserving reading of the flushing device's own grouping,
/// which is a disagreement a reader attributes to the grouping because that is
/// the only thing the call names.
///
/// The two are separately observable at the same shape. Over the contributors
/// `0x00800001, 0x80800000, 0x00800001, 0x80800000` — every one of them
/// *normal* — a two-by-two split's partials are `0x00000001` twice under a
/// preserving contract and `0x00000000` twice under a result-flushing one, while
/// the same contributors under a four-by-one split have no addition to flush and
/// report the four operands back unchanged under either. So a partial sum is
/// subnormal under one declared split and not under the other, and only the
/// contract decides what happens to it.
///
/// # Errors
///
/// As [`strict_partial_sums`].
pub fn strict_partial_sums_under(
    input: &Tensor,
    axes: &[Axis],
    partitions: u64,
    contributors_per_partition: u64,
    conformance: ReferenceNumericalConformance,
) -> Result<Tensor, ReferenceOperationError> {
    let mut reduced_mask = vec![false; input.shape().rank()];
    let mut reduced = Vec::with_capacity(axes.len());
    for requested_axis in axes {
        let dimension = usize::try_from(requested_axis.get())
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let Some(is_reduced) = reduced_mask.get_mut(dimension) else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if std::mem::replace(is_reduced, true) {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        reduced.push(dimension);
    }
    let survivor: Vec<usize> = (0..input.shape().rank())
        .filter(|axis| !reduced_mask[*axis])
        .collect();
    let reduction_shape =
        Shape::try_new(survivor.iter().map(|axis| input.shape().extents()[*axis]))
            .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
    let reduced_shape = Shape::try_new(reduced.iter().map(|axis| input.shape().extents()[*axis]))
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
    let reduced_count = reduced_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    // The split must cover the contributor sequence exactly once each. An
    // inexact split is refused rather than truncated or padded: either would
    // make this oracle answer for a plan no schedule could legally declare.
    let covered = partitions
        .checked_mul(contributors_per_partition)
        .and_then(|total| usize::try_from(total).ok())
        .ok_or(ReferenceOperationError::InvalidApplication)?;
    let partition_count =
        usize::try_from(partitions).map_err(|_| ReferenceOperationError::InvalidApplication)?;
    let chunk = usize::try_from(contributors_per_partition)
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    if partition_count == 0 || covered != reduced_count {
        return Err(ReferenceOperationError::InvalidApplication);
    }

    let partial_shape = Shape::try_new(
        reduction_shape
            .extents()
            .iter()
            .copied()
            .chain(std::iter::once(tiler_ir::shape::Extent::new(partitions))),
    )
    .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
    let partial_count = partial_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    preflight_f32_output(partial_count)?;
    if partial_count == 0 {
        return Tensor::dense(F32::resolved_type(), partial_shape, Vec::new())
            .map_err(|source| dense_result_error(&source));
    }
    let reduction_count = reduction_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;

    let input_elements = f32_elements(input)?;
    let input_strides = row_major_strides(input.shape())?;
    let reduction_strides = row_major_strides(&reduction_shape)?;
    let reduced_strides = row_major_strides(&reduced_shape)?;
    let mut elements = Vec::with_capacity(partial_count);
    let mut output_coordinate = vec![0_usize; reduction_shape.rank()];
    let mut reduced_coordinate = vec![0_usize; reduced_shape.rank()];
    let mut input_coordinate = vec![0_usize; input.shape().rank()];

    for output_linear in 0..reduction_count {
        decode_coordinate(
            output_linear,
            &reduction_shape,
            &reduction_strides,
            &mut output_coordinate,
        )?;
        for partition in 0..partition_count {
            let mut accumulator = None;
            for within in 0..chunk {
                let reduced_linear = partition * chunk + within;
                decode_coordinate(
                    reduced_linear,
                    &reduced_shape,
                    &reduced_strides,
                    &mut reduced_coordinate,
                )?;
                input_coordinate.fill(0);
                for (coordinate, axis) in output_coordinate.iter().zip(&survivor) {
                    input_coordinate[*axis] = *coordinate;
                }
                for (coordinate, axis) in reduced_coordinate.iter().zip(&reduced) {
                    input_coordinate[*axis] = *coordinate;
                }
                let linear = input_coordinate
                    .iter()
                    .zip(&input_strides)
                    .map(|(coordinate, stride)| coordinate * stride)
                    .sum::<usize>();
                let contributor = decode_f32(&input_elements[linear])?;
                accumulator = Some(match accumulator {
                    None => contributor,
                    Some(value) => conformance.apply_to_result(canonicalize_arithmetic_f32(
                        conformance.apply_to_operand(value)
                            + conformance.apply_to_operand(contributor),
                    )),
                });
            }
            // An empty partition commits the reduction identity, exactly as an
            // empty whole reduction does; a partition of one commits its single
            // contributor through the same canonicalizing result boundary. That
            // boundary canonicalizes a NaN payload and applies no subnormal
            // mode — neither dimension has a site here, because no operand
            // entered an operation and no arithmetic produced a result.
            elements.push(f32_element(canonicalize_arithmetic_f32(
                accumulator.unwrap_or(0.0_f32),
            ))?);
        }
    }
    Tensor::dense(F32::resolved_type(), partial_shape, elements)
        .map_err(|source| dense_result_error(&source))
}

/// Evaluates the whole reduction a declared split computes.
///
/// This is [`strict_partial_sums`] followed by a strict serial sum of the
/// trailing partition axis — the same two folds the two passes perform, in the
/// same order, so it is the value a correct split must produce rather than an
/// independent re-derivation of it.
///
/// # Errors
///
/// Returns whatever either fold rejects.
pub fn strict_partitioned_sum(
    input: &Tensor,
    axes: &[Axis],
    partitions: u64,
    contributors_per_partition: u64,
) -> Result<Tensor, ReferenceOperationError> {
    strict_partitioned_sum_under(
        input,
        axes,
        partitions,
        contributors_per_partition,
        ReferenceNumericalConformance::strict(),
    )
}

/// Evaluates [`strict_partitioned_sum`] under one stated numerical contract.
///
/// Both passes are performed under the same contract, because both are passes of
/// the same declared realization: a device that flushes its first pass flushes
/// its second, and an oracle that flushed one of them would answer for a plan
/// nothing declared.
///
/// # Errors
///
/// Returns whatever either fold rejects.
pub fn strict_partitioned_sum_under(
    input: &Tensor,
    axes: &[Axis],
    partitions: u64,
    contributors_per_partition: u64,
    conformance: ReferenceNumericalConformance,
) -> Result<Tensor, ReferenceOperationError> {
    let partials = strict_partial_sums_under(
        input,
        axes,
        partitions,
        contributors_per_partition,
        conformance,
    )?;
    let partition_axis = u32::try_from(partials.shape().rank())
        .ok()
        .and_then(|rank| rank.checked_sub(1))
        .ok_or(ReferenceOperationError::InvalidApplication)?;
    strict_sum(&partials, &[Axis::new(partition_axis)], conformance)
}

pub(crate) fn preflight_f32_output(output_count: usize) -> Result<(), ReferenceOperationError> {
    if output_count > MAX_REFERENCE_TENSOR_ELEMENTS {
        return Err(ReferenceOperationError::OutputElementsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual: output_count,
        });
    }
    let bytes = output_count.checked_mul(std::mem::size_of::<u32>()).ok_or(
        ReferenceOperationError::OutputResourceExceeded {
            limit: MAX_REFERENCE_TENSOR_BYTES,
            actual: usize::MAX,
        },
    )?;
    if bytes > MAX_REFERENCE_TENSOR_BYTES {
        return Err(ReferenceOperationError::OutputResourceExceeded {
            limit: MAX_REFERENCE_TENSOR_BYTES,
            actual: bytes,
        });
    }
    Ok(())
}

pub(crate) fn decode_coordinate(
    linear: usize,
    shape: &Shape,
    strides: &[usize],
    output: &mut [usize],
) -> Result<(), ReferenceOperationError> {
    let mut remainder = linear;
    for (axis, (coordinate, stride)) in output.iter_mut().zip(strides).enumerate() {
        let extent = usize::try_from(shape.extents()[axis].get())
            .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
        *coordinate = if extent == 0 { 0 } else { remainder / stride };
        remainder = if extent == 0 { 0 } else { remainder % stride };
    }
    Ok(())
}

pub(crate) fn row_major_strides(shape: &Shape) -> Result<Vec<usize>, ReferenceOperationError> {
    if shape.element_count() == Some(0) {
        return Ok(vec![0_usize; shape.rank()]);
    }
    let mut strides = vec![1_usize; shape.rank()];
    let mut running = 1_usize;
    for axis in (0..shape.rank()).rev() {
        strides[axis] = running;
        let extent = usize::try_from(shape.extents()[axis].get())
            .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
        running = running
            .checked_mul(extent)
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    }
    Ok(strides)
}

/// The row decomposition one reduced axis induces over a dense tensor.
///
/// Three counts rather than a coordinate walk, because the reduced axis is a
/// single axis of a row-major layout: everything before it varies the row's outer
/// position, everything after it varies the row's inner position, and the axis
/// itself is the contributor sequence. Deriving them once per occurrence keeps
/// the per-element index arithmetic exact and total.
///
/// Shared by every family whose identity carries *one* reduced axis and whose
/// result is shape-preserving or row-scoped — `tiler::rms-norm-f32@1` and
/// `tiler::softmax-f32@1` today. It lives here beside the other dense-payload
/// helpers rather than in either family's module, because a copy in each would
/// let one family's corpus pass against index arithmetic the other never runs.
/// The *arithmetic* stays with each family; only the decomposition is shared.
pub(crate) struct RowGeometry {
    /// Contributors of one row, the extent of the reduced axis.
    pub(crate) extent: usize,
    /// Product of the extents after the reduced axis.
    pub(crate) inner: usize,
    /// Number of rows in the tensor.
    pub(crate) rows: usize,
}

impl RowGeometry {
    pub(crate) fn derive(shape: &Shape, axis: Axis) -> Result<Self, ReferenceOperationError> {
        let position =
            usize::try_from(axis.get()).map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let extents = shape.extents();
        let Some(extent) = extents.get(position) else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let extent =
            usize::try_from(extent.get()).map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
        let mut inner = 1_usize;
        for later in &extents[position.saturating_add(1)..] {
            let later =
                usize::try_from(later.get()).map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
            inner = inner
                .checked_mul(later)
                .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        }
        let mut outer = 1_usize;
        for earlier in &extents[..position] {
            let earlier = usize::try_from(earlier.get())
                .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
            outer = outer
                .checked_mul(earlier)
                .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        }
        let rows = outer
            .checked_mul(inner)
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        Ok(Self {
            extent,
            inner,
            rows,
        })
    }

    /// Returns the dense element index of one contributor of one row.
    pub(crate) const fn element_index(&self, row: usize, position: usize) -> usize {
        let outer = row / self.inner;
        let inner = row % self.inner;
        (outer * self.extent + position) * self.inner + inner
    }
}

pub(crate) fn f32_elements(
    tensor: &Tensor,
) -> Result<&[ReferenceElement], ReferenceOperationError> {
    if tensor.resolved_type() != &F32::resolved_type() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    match tensor.payload() {
        TensorPayloadView::Dense(elements) => Ok(elements),
        TensorPayloadView::Compound(_) => Err(ReferenceOperationError::InvalidApplication),
    }
}

pub(crate) fn f32_element(value: f32) -> Result<ReferenceElement, ReferenceOperationError> {
    ReferenceElement::from_float_bits(
        value.to_bits().to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .map_err(|_| ReferenceOperationError::InvalidApplication)
}

pub(crate) fn decode_f32(element: &ReferenceElement) -> Result<f32, ReferenceOperationError> {
    let bits = <[u8; 4]>::try_from(element.as_bytes())
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    Ok(f32::from_bits(u32::from_be_bytes(bits)))
}

pub(crate) fn validate_compound_resources(
    components: &[ReferenceComponent],
    depth: usize,
    resources: &mut ReferenceWork,
) -> Result<(), EvaluationError> {
    if depth > MAX_REFERENCE_COMPONENT_DEPTH {
        return Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::ComponentDepth,
            limit: MAX_REFERENCE_COMPONENT_DEPTH,
            actual: depth,
        });
    }
    let aggregate_components = resources.components.saturating_add(components.len());
    if aggregate_components > MAX_REFERENCE_COMPONENTS {
        return Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::Components,
            limit: MAX_REFERENCE_COMPONENTS,
            actual: aggregate_components,
        });
    }
    resources.components = aggregate_components;
    let mut roles = HashSet::with_capacity(components.len());
    for component in components {
        if !roles.insert(component.role()) {
            return Err(EvaluationError::DuplicateComponentRole {
                role: component.role(),
            });
        }
        let component_elements = component
            .tensor()
            .shape()
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?;
        resources.elements = resources.elements.saturating_add(component_elements);
        if resources.elements > MAX_REFERENCE_TENSOR_ELEMENTS {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual: resources.elements,
            });
        }
        match component.tensor().payload() {
            TensorPayloadView::Dense(elements) => {
                resources.bytes = resources.bytes.saturating_add(
                    elements
                        .iter()
                        .map(|element| element.as_bytes().len())
                        .fold(0_usize, usize::saturating_add),
                );
            }
            TensorPayloadView::Compound(children) => {
                validate_compound_resources(children, depth.saturating_add(1), resources)?;
            }
        }
        if resources.bytes > MAX_REFERENCE_TENSOR_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorBytes,
                limit: MAX_REFERENCE_TENSOR_BYTES,
                actual: resources.bytes,
            });
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ReferenceWork {
    pub(crate) bytes: usize,
    pub(crate) elements: usize,
    pub(crate) components: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EvaluationRetention {
    pub(crate) work: ReferenceWork,
    pub(crate) storage_ids: HashSet<usize>,
}

fn collect_unseen_tensor_work(
    tensor: &Tensor,
    retained: &HashSet<usize>,
    pending: &mut HashSet<usize>,
    work: &mut ReferenceWork,
) -> Result<(), EvaluationError> {
    let storage_id = tensor.storage_id();
    if retained.contains(&storage_id) || !pending.insert(storage_id) {
        return Ok(());
    }
    work.elements = work.elements.saturating_add(
        tensor
            .shape()
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?,
    );
    match tensor.payload() {
        TensorPayloadView::Dense(elements) => {
            work.bytes = work.bytes.saturating_add(
                elements
                    .iter()
                    .map(|element| element.as_bytes().len())
                    .fold(0_usize, usize::saturating_add),
            );
        }
        TensorPayloadView::Compound(components) => {
            work.components = work.components.saturating_add(components.len());
            for component in components {
                collect_unseen_tensor_work(component.tensor(), retained, pending, work)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn reserve_evaluation_work(
    retention: &mut EvaluationRetention,
    tensor: &Tensor,
) -> Result<(), EvaluationError> {
    let mut added = ReferenceWork::default();
    let mut pending = HashSet::new();
    collect_unseen_tensor_work(tensor, &retention.storage_ids, &mut pending, &mut added)?;
    let next = ReferenceWork {
        bytes: retention.work.bytes.saturating_add(added.bytes),
        elements: retention.work.elements.saturating_add(added.elements),
        components: retention.work.components.saturating_add(added.components),
    };
    for (resource, limit, actual) in [
        (
            ReferenceResource::EvaluationBytes,
            MAX_REFERENCE_TENSOR_BYTES,
            next.bytes,
        ),
        (
            ReferenceResource::EvaluationElements,
            MAX_REFERENCE_TENSOR_ELEMENTS,
            next.elements,
        ),
        (
            ReferenceResource::EvaluationComponents,
            MAX_REFERENCE_COMPONENTS,
            next.components,
        ),
    ] {
        if actual > limit {
            return Err(EvaluationError::ResourceExceeded {
                resource,
                limit,
                actual,
            });
        }
    }
    retention.work = next;
    retention.storage_ids.extend(pending);
    Ok(())
}

pub(crate) fn reserve_output_work(
    retention: &mut EvaluationRetention,
    tensor: &Tensor,
) -> Result<(), ReferenceOperationError> {
    let mut added = ReferenceWork::default();
    let mut pending = HashSet::new();
    collect_unseen_tensor_work(tensor, &retention.storage_ids, &mut pending, &mut added)
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
    let next = ReferenceWork {
        bytes: retention.work.bytes.saturating_add(added.bytes),
        elements: retention.work.elements.saturating_add(added.elements),
        components: retention.work.components.saturating_add(added.components),
    };
    if next.bytes > MAX_REFERENCE_TENSOR_BYTES {
        return Err(ReferenceOperationError::OutputResourceExceeded {
            limit: MAX_REFERENCE_TENSOR_BYTES,
            actual: next.bytes,
        });
    }
    if next.elements > MAX_REFERENCE_TENSOR_ELEMENTS {
        return Err(ReferenceOperationError::OutputElementsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual: next.elements,
        });
    }
    if next.components > MAX_REFERENCE_COMPONENTS {
        return Err(ReferenceOperationError::OutputComponentsExceeded {
            limit: MAX_REFERENCE_COMPONENTS,
            actual: next.components,
        });
    }
    retention.work = next;
    retention.storage_ids.extend(pending);
    Ok(())
}

fn get_value(
    values: &HashMap<ValueId, Tensor>,
    value: ValueId,
) -> Result<&Tensor, EvaluationError> {
    values.get(&value).ok_or(EvaluationError::MalformedProgram)
}

fn reachable_operations(
    program: &SemanticProgram,
) -> Result<HashSet<OperationId>, EvaluationError> {
    let mut reachable = HashSet::with_capacity(program.operation_count());
    let mut pending: Vec<_> = program.outputs().map(|output| output.value()).collect();
    while let Some(value) = pending.pop() {
        let value = program
            .value(value)
            .map_err(|_| EvaluationError::MalformedProgram)?;
        if let Definition::OperationResult { operation, .. } = value.definition()
            && reachable.insert(operation)
        {
            let operation = program
                .operation(operation)
                .map_err(|_| EvaluationError::MalformedProgram)?;
            pending.extend(operation.operands());
        }
    }
    Ok(reachable)
}
