//! The semantic-program evaluation authority.
//!
//! [`ReferenceEvaluator`] walks a verified `SemanticProgram` and produces
//! reference tensors by dispatching each operation through the frozen
//! reference registry. It is one of the crate's two independent oracles;
//! `oracle` owns the other, over index regions, and neither reuses the
//! other's host expressions.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValueView, Definition, F32, OperationId,
    REDUCTION_AXES_ATTRIBUTE, ResolvedValueType, SemanticProgram, ValueId,
};
use tiler_ir::shape::{Axis, Shape};

use super::error::{
    EvaluationError, ReferenceOperationError, ReferenceRegistryError, ReferenceResource,
};
use super::registry::{
    FrozenReferenceRegistry, ReferenceEvaluationRequest, ReferenceOutputs, ReferenceSignature,
};
use super::tensor::{
    FloatBitOrder, InputBinding, ReferenceComponent, ReferenceElement, Tensor, TensorPayloadView,
};
use super::{
    MAX_REFERENCE_COMPONENT_DEPTH, MAX_REFERENCE_COMPONENTS, MAX_REFERENCE_TENSOR_BYTES,
    MAX_REFERENCE_TENSOR_ELEMENTS, canonicalize_arithmetic_f32,
};

/// Host evaluator for the bounded semantic profile.
#[derive(Clone, Debug)]
pub struct ReferenceEvaluator {
    registry: FrozenReferenceRegistry,
}

impl ReferenceEvaluator {
    /// Creates an evaluator with one explicit frozen capability snapshot.
    #[must_use]
    pub const fn new(registry: FrozenReferenceRegistry) -> Self {
        Self { registry }
    }

    /// Creates an evaluator using Tiler's governed initial reference profile.
    ///
    /// # Errors
    ///
    /// Returns a typed registry construction error.
    pub fn standard() -> Result<Self, ReferenceRegistryError> {
        FrozenReferenceRegistry::standard().map(Self::new)
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
        let (mut values, mut retained_work) = self.bind_inputs(program, inputs)?;

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
                self.registry
                    .validate_value(&evaluated, program.semantic_registry())?;
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
    ) -> Result<(HashMap<ValueId, Tensor>, EvaluationRetention), EvaluationError> {
        if inputs.len() != program.input_count() {
            return Err(EvaluationError::InputCount {
                expected: program.input_count(),
                actual: inputs.len(),
            });
        }

        let mut values = HashMap::with_capacity(program.value_count());
        let mut retained_work = EvaluationRetention::default();
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
            self.registry
                .validate_value(binding.tensor, program.semantic_registry())?;
            reserve_evaluation_work(&mut retained_work, binding.tensor)?;
            values.insert(declaration.value(), binding.tensor.clone());
        }
        Ok((values, retained_work))
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

pub(crate) fn binary(
    left_value: &Tensor,
    right_value: &Tensor,
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
            f32_element(canonicalize_arithmetic_f32(operation(left, right)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Tensor::dense(F32::resolved_type(), result_shape.clone(), elements)
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)
}

pub(crate) fn strict_sum(input: &Tensor, axes: &[Axis]) -> Result<Tensor, ReferenceOperationError> {
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
            .map_err(|_| ReferenceOperationError::ShapeTooLarge);
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
            .map_err(|_| ReferenceOperationError::ShapeTooLarge);
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
                Some(value) => canonicalize_arithmetic_f32(value + contributor),
            });
        }
        elements.push(f32_element(canonicalize_arithmetic_f32(
            accumulator.unwrap_or(0.0_f32),
        ))?);
    }
    Tensor::dense(F32::resolved_type(), output_shape, elements)
        .map_err(|_| ReferenceOperationError::ShapeTooLarge)
}

fn preflight_f32_output(output_count: usize) -> Result<(), ReferenceOperationError> {
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

fn decode_coordinate(
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
