//! Host reference values and evaluation for verified semantic programs.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use crate::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalValueView, Definition, F32_CONSTANT_BITS_ATTRIBUTE,
    InputKey, OperationId, REDUCTION_AXES_ATTRIBUTE, SemanticProgram, ValueId, add_f32_op,
    constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use crate::shape::{Axis, Shape};

/// An owned, dense, row-major f32 tensor used by the reference evaluator.
#[derive(Clone, Debug, PartialEq)]
pub struct Tensor {
    shape: Shape,
    elements: Vec<f32>,
}

impl Tensor {
    /// Creates a tensor after checking its element count.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::ElementCount`] when the payload length does
    /// not match the shape, or [`EvaluationError::ShapeTooLarge`] when the
    /// element count cannot be represented on this host.
    pub fn new(shape: Shape, elements: Vec<f32>) -> Result<Self, EvaluationError> {
        let expected = shape
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?;
        if elements.len() != expected {
            return Err(EvaluationError::ElementCount {
                expected,
                actual: elements.len(),
            });
        }
        Ok(Self { shape, elements })
    }

    /// Creates a rank-zero tensor.
    #[must_use]
    pub fn scalar(value: f32) -> Self {
        Self {
            shape: Shape::new([]),
            elements: vec![value],
        }
    }

    /// Returns the logical shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns dense row-major elements.
    #[must_use]
    pub fn elements(&self) -> &[f32] {
        &self.elements
    }
}

/// One key-checked entry in the ordered reference-evaluation input interface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputBinding<'a> {
    key: &'a InputKey,
    tensor: &'a Tensor,
}

impl<'a> InputBinding<'a> {
    /// Creates an input binding.
    #[must_use]
    pub const fn new(key: &'a InputKey, tensor: &'a Tensor) -> Self {
        Self { key, tensor }
    }

    /// Returns the stable interface key.
    #[must_use]
    pub const fn key(&self) -> &'a InputKey {
        self.key
    }

    /// Returns the bound reference tensor.
    #[must_use]
    pub const fn tensor(&self) -> &'a Tensor {
        self.tensor
    }
}

/// Host evaluator for the bounded semantic profile.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceEvaluator;

impl ReferenceEvaluator {
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
        program: &SemanticProgram,
        inputs: &[InputBinding<'_>],
    ) -> Result<Vec<Tensor>, EvaluationError> {
        if inputs.len() != program.input_count() {
            return Err(EvaluationError::InputCount {
                expected: program.input_count(),
                actual: inputs.len(),
            });
        }

        let mut values = HashMap::with_capacity(program.value_count());
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
            values.insert(declaration.value(), binding.tensor.clone());
        }

        let reachable_operations = reachable_operations(program)?;
        for operation in program
            .operations()
            .filter(|operation| reachable_operations.contains(&operation.id()))
        {
            let operands: Vec<_> = operation.operands().collect();
            let results: Vec<_> = operation.results().collect();
            if results.len() != 1 {
                return Err(EvaluationError::MalformedProgram);
            }
            let result = if operation.key() == &constant_f32_op() && operands.is_empty() {
                let Some(CanonicalValueView::Unsigned(bits)) = operation
                    .attributes()
                    .get(F32_CONSTANT_BITS_ATTRIBUTE)
                    .map(crate::semantic::CanonicalValue::view)
                else {
                    return Err(EvaluationError::MalformedProgram);
                };
                Tensor::scalar(f32::from_bits(
                    u32::try_from(bits).map_err(|_| EvaluationError::MalformedProgram)?,
                ))
            } else if operation.key() == &multiply_f32_op() && operands.len() == 2 {
                binary(&values, operands[0], operands[1], |left, right| {
                    left * right
                })?
            } else if operation.key() == &add_f32_op() && operands.len() == 2 {
                binary(&values, operands[0], operands[1], |left, right| {
                    left + right
                })?
            } else if operation.key() == &strict_serial_sum_f32_op() && operands.len() == 1 {
                let axes = reduction_axes(operation.attributes())?;
                strict_sum(get_value(&values, operands[0])?, &axes)?
            } else {
                return Err(EvaluationError::MalformedProgram);
            };
            if program
                .shape(results[0])
                .map_err(|_| EvaluationError::MalformedProgram)?
                != result.shape()
            {
                return Err(EvaluationError::MalformedProgram);
            }
            values.insert(results[0], result);
        }

        program
            .outputs()
            .map(|output| get_value(&values, output.value()).cloned())
            .collect()
    }
}

fn reduction_axes(
    attributes: &crate::semantic::OperationAttributes,
) -> Result<Vec<Axis>, EvaluationError> {
    let Some(CanonicalValueView::Sequence(values)) = attributes
        .get(REDUCTION_AXES_ATTRIBUTE)
        .map(crate::semantic::CanonicalValue::view)
    else {
        return Err(EvaluationError::MalformedProgram);
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned(axis) = value.view() else {
                return Err(EvaluationError::MalformedProgram);
            };
            u32::try_from(axis)
                .map(Axis::new)
                .map_err(|_| EvaluationError::MalformedProgram)
        })
        .collect()
}

fn binary(
    values: &HashMap<ValueId, Tensor>,
    left: ValueId,
    right: ValueId,
    operation: impl Fn(f32, f32) -> f32,
) -> Result<Tensor, EvaluationError> {
    let left_value = get_value(values, left)?;
    let right_value = get_value(values, right)?;
    let result_shape = if left_value.shape().rank() == 0 {
        right_value.shape()
    } else {
        left_value.shape()
    };
    let count = result_shape
        .element_count()
        .ok_or(EvaluationError::ShapeTooLarge)?;
    let elements = (0..count)
        .map(|index| {
            let left = if left_value.shape().rank() == 0 {
                left_value.elements()[0]
            } else {
                left_value.elements()[index]
            };
            let right = if right_value.shape().rank() == 0 {
                right_value.elements()[0]
            } else {
                right_value.elements()[index]
            };
            canonicalize_arithmetic_f32(operation(left, right))
        })
        .collect();
    Tensor::new(result_shape.clone(), elements)
}

fn strict_sum(input: &Tensor, axes: &[Axis]) -> Result<Tensor, EvaluationError> {
    let reduced: Vec<usize> = axes
        .iter()
        .map(|axis| usize::try_from(axis.get()).expect("verified axis fits usize"))
        .collect();
    let survivor: Vec<usize> = (0..input.shape().rank())
        .filter(|axis| !reduced.contains(axis))
        .collect();
    let output_shape = Shape::new(survivor.iter().map(|axis| input.shape().extents()[*axis]));
    let output_count = output_shape
        .element_count()
        .ok_or(EvaluationError::ShapeTooLarge)?;
    let input_strides = row_major_strides(input.shape())?;
    let output_coordinates = coordinates(&output_shape)?;
    let reduced_shape = Shape::new(reduced.iter().map(|axis| input.shape().extents()[*axis]));
    let reduced_coordinates = coordinates(&reduced_shape)?;
    let mut elements = Vec::with_capacity(output_count);

    for output_coordinate in output_coordinates {
        let mut accumulator = None;
        for reduced_coordinate in &reduced_coordinates {
            let mut input_coordinate = vec![0_usize; input.shape().rank()];
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
            let contributor = input.elements()[linear];
            accumulator = Some(match accumulator {
                None => contributor,
                Some(value) => canonicalize_arithmetic_f32(value + contributor),
            });
        }
        elements.push(canonicalize_arithmetic_f32(accumulator.unwrap_or(0.0_f32)));
    }
    Tensor::new(output_shape, elements)
}

fn coordinates(shape: &Shape) -> Result<Vec<Vec<usize>>, EvaluationError> {
    let count = shape
        .element_count()
        .ok_or(EvaluationError::ShapeTooLarge)?;
    let strides = row_major_strides(shape)?;
    let mut result = Vec::with_capacity(count);
    for linear in 0..count {
        let mut remainder = linear;
        let mut coordinate = Vec::with_capacity(shape.rank());
        for (axis, stride) in strides.iter().enumerate() {
            let extent = usize::try_from(shape.extents()[axis].get())
                .map_err(|_| EvaluationError::ShapeTooLarge)?;
            let value = if extent == 0 { 0 } else { remainder / stride };
            remainder = if extent == 0 { 0 } else { remainder % stride };
            coordinate.push(value);
        }
        result.push(coordinate);
    }
    Ok(result)
}

fn row_major_strides(shape: &Shape) -> Result<Vec<usize>, EvaluationError> {
    let mut strides = vec![1_usize; shape.rank()];
    let mut running = 1_usize;
    for axis in (0..shape.rank()).rev() {
        strides[axis] = running;
        let extent = usize::try_from(shape.extents()[axis].get())
            .map_err(|_| EvaluationError::ShapeTooLarge)?;
        running = running
            .checked_mul(extent)
            .ok_or(EvaluationError::ShapeTooLarge)?;
    }
    Ok(strides)
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

fn canonicalize_arithmetic_f32(value: f32) -> f32 {
    if value.is_nan() {
        f32::from_bits(CANONICAL_F32_ARITHMETIC_NAN_BITS)
    } else {
        value
    }
}

/// A typed reference-evaluation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EvaluationError {
    /// The caller supplied the wrong number of ordered input bindings.
    InputCount {
        /// Declared program input count.
        expected: usize,
        /// Supplied binding count.
        actual: usize,
    },
    /// A binding key disagreed with the ordered semantic interface.
    InputKey {
        /// Position in the ordered input interface.
        input_index: usize,
        /// Declared key at that position.
        expected: InputKey,
        /// Supplied key at that position.
        actual: InputKey,
    },
    /// An input shape disagreed with its verified declaration.
    InputShape {
        /// Stable key identifying the input.
        key: InputKey,
        /// Statically declared shape.
        expected: Shape,
        /// Supplied tensor shape.
        actual: Shape,
    },
    /// A tensor payload length disagreed with its shape.
    ElementCount {
        /// Element count implied by the shape.
        expected: usize,
        /// Supplied payload element count.
        actual: usize,
    },
    /// Shape arithmetic exceeded host limits.
    ShapeTooLarge,
    /// An internally malformed verified program reached the evaluator.
    MalformedProgram,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputCount { expected, actual } => {
                write!(formatter, "expected {expected} inputs, received {actual}")
            }
            Self::InputKey {
                input_index,
                expected,
                actual,
            } => write!(
                formatter,
                "input {input_index} has key {:?}, expected {:?}",
                actual.as_str(),
                expected.as_str()
            ),
            Self::InputShape {
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "input {:?} has shape {actual:?}, expected {expected:?}",
                key.as_str()
            ),
            Self::ElementCount { expected, actual } => {
                write!(
                    formatter,
                    "tensor has {actual} elements, expected {expected}"
                )
            }
            Self::ShapeTooLarge => formatter.write_str("tensor shape exceeds host limits"),
            Self::MalformedProgram => formatter.write_str("verified semantic program is malformed"),
        }
    }
}

impl Error for EvaluationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum, Value,
    };

    fn constant_bits(graph: &mut SemanticProgramBuilder, bits: u32) -> Value<F32> {
        F32Constant::apply(graph, bits).unwrap()
    }

    fn constant(graph: &mut SemanticProgramBuilder, value: f32) -> Value<F32> {
        constant_bits(graph, value.to_bits())
    }

    fn multiply(
        graph: &mut SemanticProgramBuilder,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Value<F32> {
        F32Multiply::apply(graph, left, right).unwrap()
    }

    fn add(graph: &mut SemanticProgramBuilder, left: Value<F32>, right: Value<F32>) -> Value<F32> {
        F32Add::apply(graph, left, right).unwrap()
    }

    fn sum(
        graph: &mut SemanticProgramBuilder,
        input: Value<F32>,
        axes: impl IntoIterator<Item = Axis>,
    ) -> Value<F32> {
        StrictSerialF32Sum::apply(graph, input, axes).unwrap()
    }

    fn graph(shape: Shape, axes: &[u32]) -> SemanticProgram {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let x = graph
            .input::<F32>(InputKey::new("x").unwrap(), shape)
            .unwrap();
        let scale = constant(&mut graph, 2.0);
        let bias = constant(&mut graph, 1.0);
        let product = multiply(&mut graph, x, scale);
        let mapped = add(&mut graph, product, bias);
        let sum = sum(&mut graph, mapped, axes.iter().copied().map(Axis::new));
        graph
            .output(OutputKey::new("mapped").unwrap(), mapped)
            .unwrap();
        graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        graph.build().unwrap()
    }

    fn evaluate_one(program: &SemanticProgram, input: &Tensor) -> Vec<Tensor> {
        let key = InputKey::new("x").unwrap();
        ReferenceEvaluator::evaluate(program, &[InputBinding::new(&key, input)]).unwrap()
    }

    #[test]
    fn evaluates_pointwise_prologue_and_multiple_outputs() {
        let program = graph(Shape::from_dims([2, 3]), &[1]);
        let input =
            Tensor::new(Shape::from_dims([2, 3]), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let outputs = evaluate_one(&program, &input);
        assert_eq!(outputs[0].elements(), &[3.0, 5.0, 7.0, 9.0, 11.0, 13.0]);
        assert_eq!(outputs[1].shape(), &Shape::from_dims([2]));
        assert_eq!(outputs[1].elements(), &[15.0, 33.0]);
    }

    #[test]
    fn contributor_order_is_original_axis_lexicographic() {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let x = graph
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2, 2]))
            .unwrap();
        let sum = sum(&mut graph, x, [Axis::new(0), Axis::new(2)]);
        graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        let program = graph.build().unwrap();
        let input = Tensor::new(
            Shape::from_dims([2, 2, 2]),
            vec![1.0e20, 1.0, 7.0, 8.0, -1.0e20, 3.0, 9.0, 10.0],
        )
        .unwrap();
        let outputs = evaluate_one(&program, &input);
        assert_eq!(outputs[0].elements()[0].to_bits(), 3.0_f32.to_bits());
        assert_eq!(outputs[0].elements()[1].to_bits(), 34.0_f32.to_bits());
    }

    #[test]
    fn strict_sum_preserves_non_nan_singletons_and_canonicalizes_nan_results() {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let x = graph
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([3, 1]))
            .unwrap();
        let sum = sum(&mut graph, x, [Axis::new(1)]);
        graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        let program = graph.build().unwrap();
        let nan = f32::from_bits(0x7fc0_1234);
        let input = Tensor::new(Shape::from_dims([3, 1]), vec![-0.0, f32::INFINITY, nan]).unwrap();
        let output = evaluate_one(&program, &input);
        assert_eq!(output[0].elements()[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(output[0].elements()[1].to_bits(), f32::INFINITY.to_bits());
        assert_eq!(
            output[0].elements()[2].to_bits(),
            CANONICAL_F32_ARITHMETIC_NAN_BITS
        );
    }

    #[test]
    fn multiply_and_add_remain_two_rounding_operations() {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let x = graph
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([1]))
            .unwrap();
        let scale = constant_bits(&mut graph, 0x3f7f_ffff);
        let bias = constant(&mut graph, -1.0);
        let product = multiply(&mut graph, x, scale);
        let mapped = add(&mut graph, product, bias);
        let sum = sum(&mut graph, mapped, [Axis::new(0)]);
        graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        let program = graph.build().unwrap();
        let input = Tensor::new(Shape::from_dims([1]), vec![f32::from_bits(0x3f80_0001)]).unwrap();
        let output = evaluate_one(&program, &input);
        assert_eq!(output[0].elements()[0].to_bits(), 0.0_f32.to_bits());
        assert_ne!(
            f32::from_bits(0x3f80_0001)
                .mul_add(f32::from_bits(0x3f7f_ffff), -1.0)
                .to_bits(),
            0.0_f32.to_bits()
        );
    }

    #[test]
    fn empty_reduced_domain_is_positive_zero_but_empty_survivor_has_no_elements() {
        let program = graph(Shape::from_dims([2, 0]), &[1]);
        let input = Tensor::new(Shape::from_dims([2, 0]), vec![]).unwrap();
        let outputs = evaluate_one(&program, &input);
        assert_eq!(outputs[1].elements().len(), 2);
        assert!(
            outputs[1]
                .elements()
                .iter()
                .all(|value| value.to_bits() == 0.0_f32.to_bits())
        );

        let program = graph(Shape::from_dims([0, 2]), &[1]);
        let input = Tensor::new(Shape::from_dims([0, 2]), vec![]).unwrap();
        let outputs = evaluate_one(&program, &input);
        assert!(outputs[1].elements().is_empty());
    }

    #[test]
    fn bindings_validate_ordered_keys_shapes_and_payloads() {
        assert_eq!(
            Tensor::new(Shape::from_dims([2]), vec![1.0]).unwrap_err(),
            EvaluationError::ElementCount {
                expected: 2,
                actual: 1,
            }
        );
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let left_key = InputKey::new("left").unwrap();
        let right_key = InputKey::new("right").unwrap();
        let left = graph
            .input::<F32>(left_key.clone(), Shape::from_dims([2]))
            .unwrap();
        let right = graph
            .input::<F32>(right_key.clone(), Shape::from_dims([2]))
            .unwrap();
        let sum = add(&mut graph, left, right);
        graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        let program = graph.build().unwrap();
        let left_tensor = Tensor::new(Shape::from_dims([2]), vec![1.0, 2.0]).unwrap();
        let right_tensor = Tensor::new(Shape::from_dims([2]), vec![3.0, 4.0]).unwrap();
        let swapped = [
            InputBinding::new(&right_key, &right_tensor),
            InputBinding::new(&left_key, &left_tensor),
        ];
        assert!(matches!(
            ReferenceEvaluator::evaluate(&program, &swapped),
            Err(EvaluationError::InputKey { input_index: 0, .. })
        ));
        assert!(matches!(
            ReferenceEvaluator::evaluate(&program, &[InputBinding::new(&left_key, &left_tensor)]),
            Err(EvaluationError::InputCount { .. })
        ));
        let wrong = Tensor::new(Shape::from_dims([1]), vec![1.0]).unwrap();
        assert!(matches!(
            ReferenceEvaluator::evaluate(
                &program,
                &[
                    InputBinding::new(&left_key, &wrong),
                    InputBinding::new(&right_key, &right_tensor)
                ]
            ),
            Err(EvaluationError::InputShape { .. })
        ));
    }

    #[test]
    fn constants_preserve_nan_payloads_but_arithmetic_results_are_canonical() {
        let payload = 0x7fc0_1234;
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let literal = constant_bits(&mut graph, payload);
        let zero = constant(&mut graph, 0.0);
        let arithmetic = add(&mut graph, literal, zero);
        graph
            .output(OutputKey::new("constant").unwrap(), literal)
            .unwrap();
        graph
            .output(OutputKey::new("arithmetic").unwrap(), arithmetic)
            .unwrap();
        let program = graph.build().unwrap();

        let output = ReferenceEvaluator::evaluate(&program, &[]).unwrap();
        assert_eq!(output[0].elements()[0].to_bits(), payload);
        assert_eq!(
            output[1].elements()[0].to_bits(),
            CANONICAL_F32_ARITHMETIC_NAN_BITS
        );
    }

    #[test]
    fn commitment_removes_dead_operations_and_inputs_before_evaluation() {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let live = constant(&mut graph, 7.0);
        let dead_input = graph
            .input::<F32>(InputKey::new("dead").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let dead = sum(&mut graph, dead_input, [Axis::new(0)]);
        graph.output(OutputKey::new("live").unwrap(), live).unwrap();
        let program = graph.build().unwrap();

        assert!(matches!(
            program.value(dead.erase()),
            Err(crate::semantic::HandleError::ForeignGraph { .. })
        ));
        assert_eq!(program.input_count(), 0);
        assert_eq!(program.operation_count(), 1);
        let outputs = ReferenceEvaluator::evaluate(&program, &[]).unwrap();
        assert_eq!(outputs[0].elements(), &[7.0]);
    }
}
