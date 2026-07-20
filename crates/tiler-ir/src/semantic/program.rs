use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use crate::shape::{Axis, Shape};

use super::error::{
    BuildError, BuilderCreateError, EntityKind, HandleError, ProgramBuildError,
    ProgramBuildFailure, ValidationDiagnostic, ValidationDiagnostics, ValueRole,
};
use super::handles::{GraphId, OperationId, OperationIndex, ValueId, ValueIndex, next_graph_id};
use super::identity::CanonicalIdentity;
use super::interface::{
    InputIndex, InputKey, OutputKey, OutputSelector, ProgramInput, ProgramInputRef, ProgramOutput,
    ProgramOutputRef,
};
use super::operation::{
    OperationData, OperationKind, OperationRef, ResultIndex, ValueData, ValueDefinition, ValueRef,
};
use super::registry::{F32, FrozenSemanticRegistry};
use super::types::ResolvedValueType;

/// A verified, immutable semantic program for the bounded f32 prototype.
#[derive(Clone, Debug)]
pub struct SemanticProgram {
    pub(super) data: Arc<ProgramData>,
}

#[derive(Debug)]
pub(super) struct ProgramData {
    pub(super) owner: GraphId,
    pub(super) origin: GraphId,
    pub(super) inputs: Vec<ProgramInput>,
    pub(super) operations: Vec<OperationData>,
    pub(super) values: Vec<ValueData>,
    pub(super) outputs: Vec<ProgramOutput>,
    pub(super) identity: OnceLock<CanonicalIdentity>,
    pub(super) semantic_registry: FrozenSemanticRegistry,
}

impl SemanticProgram {
    /// Returns ordered input interface entries.
    #[must_use]
    pub fn inputs(
        &self,
    ) -> impl ExactSizeIterator<Item = ProgramInputRef<'_>> + DoubleEndedIterator {
        self.data.inputs.iter().map(|input| ProgramInputRef {
            owner: self.data.owner,
            input,
        })
    }

    /// Returns operations in verified topological order.
    #[must_use]
    pub fn operations(
        &self,
    ) -> impl ExactSizeIterator<Item = OperationRef<'_>> + DoubleEndedIterator {
        self.data
            .operations
            .iter()
            .enumerate()
            .map(|(index, operation)| OperationRef {
                owner: self.data.owner,
                index: OperationIndex::from_verified_len(index),
                operation,
            })
    }

    /// Returns all values in graph-local ordinal order.
    #[must_use]
    pub fn values(&self) -> impl ExactSizeIterator<Item = ValueRef<'_>> + DoubleEndedIterator {
        self.data
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| ValueRef {
                owner: self.data.owner,
                index: ValueIndex::from_verified_len(index),
                value,
            })
    }

    /// Returns ordered, named program outputs.
    #[must_use]
    pub fn outputs(
        &self,
    ) -> impl ExactSizeIterator<Item = ProgramOutputRef<'_>> + DoubleEndedIterator {
        self.data.outputs.iter().map(|output| ProgramOutputRef {
            owner: self.data.owner,
            output,
        })
    }

    /// Looks up a graph-owned value.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign or invalid local handle.
    pub fn value(&self, id: ValueId) -> Result<ValueRef<'_>, HandleError> {
        if id.owner != self.data.owner {
            return Err(HandleError::ForeignGraph {
                entity: EntityKind::Value,
            });
        }
        self.data
            .values
            .get(id.index.as_usize())
            .map(|value| ValueRef {
                owner: self.data.owner,
                index: id.index,
                value,
            })
            .ok_or(HandleError::InvalidLocal {
                entity: EntityKind::Value,
            })
    }

    /// Looks up a graph-owned operation.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign or invalid local handle.
    pub fn operation(&self, id: OperationId) -> Result<OperationRef<'_>, HandleError> {
        if id.owner != self.data.owner {
            return Err(HandleError::ForeignGraph {
                entity: EntityKind::Operation,
            });
        }
        self.data
            .operations
            .get(id.index.as_usize())
            .map(|operation| OperationRef {
                owner: self.data.owner,
                index: id.index,
                operation,
            })
            .ok_or(HandleError::InvalidLocal {
                entity: EntityKind::Operation,
            })
    }

    /// Resolves one selector produced by the draft committed into this program.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a selector from another draft or for an
    /// invalid local selector.
    pub fn resolve_output(
        &self,
        selector: &OutputSelector,
    ) -> Result<ProgramOutputRef<'_>, HandleError> {
        if selector.origin != self.data.origin {
            return Err(HandleError::ForeignGraph {
                entity: EntityKind::Output,
            });
        }
        self.data
            .outputs
            .iter()
            .find(|output| output.key == selector.key)
            .map(|output| ProgramOutputRef {
                owner: self.data.owner,
                output,
            })
            .ok_or(HandleError::InvalidLocal {
                entity: EntityKind::Output,
            })
    }

    /// Returns the ordered input count.
    #[must_use]
    pub fn input_count(&self) -> usize {
        self.data.inputs.len()
    }

    /// Returns the output-reachable operation count.
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.data.operations.len()
    }

    /// Returns the output-reachable value count.
    #[must_use]
    pub fn value_count(&self) -> usize {
        self.data.values.len()
    }

    /// Returns the ordered output count.
    #[must_use]
    pub fn output_count(&self) -> usize {
        self.data.outputs.len()
    }

    /// Returns the shape of a graph-owned value.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign or invalid local handle.
    pub fn shape(&self, value: ValueId) -> Result<&Shape, HandleError> {
        self.value(value)?;
        Ok(&self.data.values[value.index.as_usize()].shape)
    }

    /// Returns the immutable semantic authority that validated this program.
    #[must_use]
    pub fn semantic_registry(&self) -> &FrozenSemanticRegistry {
        &self.data.semantic_registry
    }
}

/// Incremental constructor for a verified bounded semantic program.
#[derive(Debug)]
pub struct SemanticProgramBuilder {
    owner: GraphId,
    inputs: Vec<ProgramInput>,
    operations: Vec<OperationData>,
    values: Vec<ValueData>,
    outputs: Vec<ProgramOutput>,
    input_keys: HashSet<InputKey>,
    output_keys: HashSet<OutputKey>,
    semantic_registry: FrozenSemanticRegistry,
}

impl SemanticProgramBuilder {
    /// Tries to create an empty builder with a distinct graph owner.
    ///
    /// # Errors
    ///
    /// Returns [`BuilderCreateError::GraphIdentityExhausted`] without creating
    /// a builder when the process-local owner space is exhausted.
    pub fn try_new(semantic_registry: FrozenSemanticRegistry) -> Result<Self, BuilderCreateError> {
        Ok(Self {
            owner: next_graph_id().ok_or(BuilderCreateError::GraphIdentityExhausted)?,
            inputs: Vec::new(),
            operations: Vec::new(),
            values: Vec::new(),
            outputs: Vec::new(),
            input_keys: HashSet::new(),
            output_keys: HashSet::new(),
            semantic_registry,
        })
    }

    /// Tries to create a builder using Tiler's governed standard registry.
    ///
    /// # Errors
    ///
    /// Returns a typed error if standard registry construction or graph-owner
    /// allocation fails.
    pub fn try_standard() -> Result<Self, BuilderCreateError> {
        let registry =
            FrozenSemanticRegistry::standard().map_err(BuilderCreateError::StandardRegistry)?;
        Self::try_new(registry)
    }

    /// Adds an ordered fixed-shape f32 tensor input.
    ///
    /// # Errors
    ///
    /// Returns a typed error for duplicate keys, unsupported shapes, or exhausted IDs.
    pub fn input_f32(&mut self, key: InputKey, shape: Shape) -> Result<ValueId, BuildError> {
        validate_shape(&shape)?;
        let resolved_type = self.require_f32()?;
        if self.input_keys.contains(&key) {
            return Err(BuildError::DuplicateInputKey(key));
        }
        let input_index =
            InputIndex::from_len(self.inputs.len()).ok_or(BuildError::TooManyEntities {
                entity: EntityKind::Input,
            })?;
        let value_index = checked_index(self.values.len(), EntityKind::Value)?;
        self.values.push(ValueData {
            definition: ValueDefinition::Input { input_index },
            shape,
            resolved_type,
        });
        self.inputs.push(ProgramInput {
            key: key.clone(),
            value: value_index,
        });
        let inserted = self.input_keys.insert(key);
        debug_assert!(inserted);
        Ok(ValueId {
            owner: self.owner,
            index: value_index,
        })
    }

    /// Adds a rank-zero immutable f32 constant using exact bits.
    ///
    /// # Errors
    ///
    /// Returns a typed error if an arena's fixed-width ID space is exhausted.
    pub fn scalar_f32_bits(&mut self, bits: u32) -> Result<ValueId, BuildError> {
        let resolved_type = self.require_f32()?;
        self.push_operation(
            OperationKind::ConstantF32 { bits },
            &[],
            Shape::new([]),
            resolved_type,
        )
    }

    /// Adds a rank-zero immutable f32 constant.
    ///
    /// # Errors
    ///
    /// Returns a typed error if an arena's fixed-width ID space is exhausted.
    pub fn scalar_f32(&mut self, value: f32) -> Result<ValueId, BuildError> {
        self.scalar_f32_bits(value.to_bits())
    }

    /// Adds elementwise f32 multiplication with exact-shape or scalar broadcast.
    ///
    /// # Errors
    ///
    /// Returns a typed error for foreign operands, incompatible shapes, or exhausted IDs.
    pub fn multiply_f32(&mut self, left: ValueId, right: ValueId) -> Result<ValueId, BuildError> {
        self.binary(OperationKind::MultiplyF32, left, right)
    }

    /// Adds elementwise f32 addition with exact-shape or scalar broadcast.
    ///
    /// # Errors
    ///
    /// Returns a typed error for foreign operands, incompatible shapes, or exhausted IDs.
    pub fn add_f32(&mut self, left: ValueId, right: ValueId) -> Result<ValueId, BuildError> {
        self.binary(OperationKind::AddF32, left, right)
    }

    /// Adds strict serial f32 Sum over a nonempty, sorted, unique axis set.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign input, invalid axes, or exhausted IDs.
    pub fn strict_serial_sum_f32(
        &mut self,
        input: ValueId,
        axes: impl IntoIterator<Item = Axis>,
    ) -> Result<ValueId, BuildError> {
        let input_shape = self
            .value_data(input, ValueRole::OperationOperand { index: 0 })?
            .shape
            .clone();
        let axes: Vec<_> = axes.into_iter().collect();
        if axes.is_empty() {
            return Err(BuildError::EmptyReductionAxes);
        }
        let mut previous = None;
        for axis in &axes {
            if usize::try_from(axis.get()).map_or(true, |axis| axis >= input_shape.rank()) {
                return Err(BuildError::AxisOutOfRange {
                    axis: *axis,
                    rank: input_shape.rank(),
                });
            }
            if previous.is_some_and(|prior| axis.get() <= prior) {
                return Err(BuildError::NonCanonicalReductionAxes);
            }
            previous = Some(axis.get());
        }
        let shape = input_shape.without_axes(&axes);
        let resolved_type = Arc::clone(
            &self
                .value_data(input, ValueRole::OperationOperand { index: 0 })?
                .resolved_type,
        );
        self.push_operation(
            OperationKind::StrictSerialSumF32 { axes },
            &[input],
            shape,
            resolved_type,
        )
    }

    /// Adds an ordered named output. Multiple outputs may share a value.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign value or duplicate output key.
    pub fn output(&mut self, key: OutputKey, value: ValueId) -> Result<OutputSelector, BuildError> {
        let value_index = self.value_index(value, ValueRole::ProgramOutput)?;
        if self.output_keys.contains(&key) {
            return Err(BuildError::DuplicateOutputKey(key));
        }
        let selector = OutputSelector {
            origin: self.owner,
            key: key.clone(),
        };
        self.outputs.push(ProgramOutput {
            key: key.clone(),
            value: value_index,
        });
        let inserted = self.output_keys.insert(key);
        debug_assert!(inserted);
        Ok(selector)
    }

    /// Checks all whole-program invariants without consuming the builder.
    ///
    /// # Errors
    ///
    /// Returns all diagnostics found in deterministic validation order.
    pub fn validate(&self) -> Result<(), ValidationDiagnostics> {
        let mut diagnostics = Vec::new();
        if self.outputs.is_empty() {
            diagnostics.push(ValidationDiagnostic::NoProgramOutputs);
        }
        self.validate_internal(&mut diagnostics);
        match ValidationDiagnostics::new(diagnostics) {
            Some(errors) => Err(errors),
            None => Ok(()),
        }
    }

    /// Validates and compacts this draft into an immutable shared program.
    ///
    /// # Errors
    ///
    /// Returns the exact failure together with the intact builder when
    /// validation or completed-owner allocation fails.
    pub fn build(self) -> Result<SemanticProgram, ProgramBuildError> {
        self.build_with_completed_owner(next_graph_id())
    }

    fn build_with_completed_owner(
        mut self,
        completed_owner: Option<GraphId>,
    ) -> Result<SemanticProgram, ProgramBuildError> {
        if let Err(diagnostics) = self.validate() {
            return Err(ProgramBuildError {
                builder: Box::new(self),
                failure: ProgramBuildFailure::Validation(diagnostics),
            });
        }
        let Some(completed_owner) = completed_owner else {
            return Err(ProgramBuildError {
                builder: Box::new(self),
                failure: ProgramBuildFailure::GraphIdentityExhausted,
            });
        };
        let origin = self.owner;
        self.compact_to_outputs();
        Ok(SemanticProgram {
            data: Arc::new(ProgramData {
                owner: completed_owner,
                origin,
                inputs: self.inputs,
                operations: self.operations,
                values: self.values,
                outputs: self.outputs,
                identity: OnceLock::new(),
                semantic_registry: self.semantic_registry,
            }),
        })
    }

    fn binary(
        &mut self,
        kind: OperationKind,
        left: ValueId,
        right: ValueId,
    ) -> Result<ValueId, BuildError> {
        let left_shape = &self
            .value_data(left, ValueRole::OperationOperand { index: 0 })?
            .shape;
        let right_shape = &self
            .value_data(right, ValueRole::OperationOperand { index: 1 })?
            .shape;
        let shape = if left_shape.rank() == 0 {
            right_shape.clone()
        } else if right_shape.rank() == 0 || left_shape == right_shape {
            left_shape.clone()
        } else {
            return Err(BuildError::IncompatiblePointwiseShapes {
                left: left_shape.clone(),
                right: right_shape.clone(),
            });
        };
        let resolved_type = Arc::clone(
            &self
                .value_data(left, ValueRole::OperationOperand { index: 0 })?
                .resolved_type,
        );
        self.push_operation(kind, &[left, right], shape, resolved_type)
    }

    fn push_operation(
        &mut self,
        kind: OperationKind,
        operands: &[ValueId],
        shape: Shape,
        resolved_type: Arc<ResolvedValueType>,
    ) -> Result<ValueId, BuildError> {
        let operand_indices: Vec<_> = operands
            .iter()
            .enumerate()
            .map(|(index, operand)| {
                self.value_index(
                    *operand,
                    ValueRole::OperationOperand {
                        index: u32::try_from(index).expect("operation arity was admitted"),
                    },
                )
            })
            .collect::<Result<_, _>>()?;
        let operation_index =
            OperationIndex::from_len(self.operations.len()).ok_or(BuildError::TooManyEntities {
                entity: EntityKind::Operation,
            })?;
        let value_index = checked_index(self.values.len(), EntityKind::Value)?;
        self.values.push(ValueData {
            definition: ValueDefinition::OperationResult {
                operation: operation_index,
                result_index: ResultIndex::ZERO,
            },
            shape,
            resolved_type,
        });
        self.operations.push(OperationData {
            kind,
            operands: operand_indices,
            results: vec![value_index],
        });
        Ok(ValueId {
            owner: self.owner,
            index: value_index,
        })
    }

    fn value_index(&self, id: ValueId, role: ValueRole) -> Result<ValueIndex, BuildError> {
        if id.owner != self.owner {
            return Err(BuildError::ForeignValue { role });
        }
        if self.values.get(id.index.as_usize()).is_none() {
            return Err(BuildError::InvalidLocalValue { role });
        }
        Ok(id.index)
    }

    fn value_data(&self, id: ValueId, role: ValueRole) -> Result<&ValueData, BuildError> {
        let index = self.value_index(id, role)?;
        Ok(&self.values[index.as_usize()])
    }

    fn require_f32(&self) -> Result<Arc<ResolvedValueType>, BuildError> {
        let resolved_type = F32::resolved_type();
        if !self.semantic_registry.contains(&resolved_type) {
            return Err(BuildError::UnregisteredValueType { resolved_type });
        }
        Ok(Arc::new(resolved_type))
    }

    fn compact_to_outputs(&mut self) {
        let mut reachable_values = vec![false; self.values.len()];
        let mut reachable_operations = vec![false; self.operations.len()];
        let mut pending: Vec<_> = self.outputs.iter().map(|output| output.value).collect();

        while let Some(value_index) = pending.pop() {
            if std::mem::replace(&mut reachable_values[value_index.as_usize()], true) {
                continue;
            }
            let ValueDefinition::OperationResult { operation, .. } =
                self.values[value_index.as_usize()].definition
            else {
                continue;
            };
            if std::mem::replace(&mut reachable_operations[operation.as_usize()], true) {
                continue;
            }
            let operation = &self.operations[operation.as_usize()];
            pending.extend(operation.operands.iter().copied());
            pending.extend(operation.results.iter().copied());
        }

        let mut value_map = vec![None; self.values.len()];
        let mut next_value = 0_usize;
        for (old_index, reachable) in reachable_values.iter().copied().enumerate() {
            if reachable {
                value_map[old_index] = Some(ValueIndex::from_verified_len(next_value));
                next_value += 1;
            }
        }

        let mut operation_map = vec![None; self.operations.len()];
        let mut next_operation = 0_usize;
        for (old_index, reachable) in reachable_operations.iter().copied().enumerate() {
            if reachable {
                operation_map[old_index] = Some(OperationIndex::from_verified_len(next_operation));
                next_operation += 1;
            }
        }

        let old_inputs = std::mem::take(&mut self.inputs);
        let mut input_map = vec![None; old_inputs.len()];
        for (old_position, mut input) in old_inputs.into_iter().enumerate() {
            let Some(value) = value_map[input.value.as_usize()] else {
                continue;
            };
            let new_position = InputIndex::from_len(self.inputs.len())
                .expect("validated live input count fits its fixed-width space");
            input_map[old_position] = Some(new_position);
            input.value = value;
            self.inputs.push(input);
        }
        self.input_keys = self.inputs.iter().map(|input| input.key.clone()).collect();

        let old_values = std::mem::take(&mut self.values);
        for (old_index, mut value) in old_values.into_iter().enumerate() {
            if value_map[old_index].is_none() {
                continue;
            }
            value.definition = match value.definition {
                ValueDefinition::Input { input_index } => ValueDefinition::Input {
                    input_index: input_map[usize::try_from(input_index.get())
                        .expect("u32 fits every supported host usize")]
                    .expect("a reachable input value retains its declaration"),
                },
                ValueDefinition::OperationResult {
                    operation,
                    result_index,
                } => ValueDefinition::OperationResult {
                    operation: operation_map[operation.as_usize()]
                        .expect("a reachable result retains its defining operation"),
                    result_index,
                },
            };
            self.values.push(value);
        }

        let old_operations = std::mem::take(&mut self.operations);
        for (old_index, mut operation) in old_operations.into_iter().enumerate() {
            if operation_map[old_index].is_none() {
                continue;
            }
            for operand in &mut operation.operands {
                *operand = value_map[operand.as_usize()]
                    .expect("a reachable operation retains every operand");
            }
            for result in &mut operation.results {
                *result = value_map[result.as_usize()]
                    .expect("a reachable operation retains every result");
            }
            self.operations.push(operation);
        }

        for output in &mut self.outputs {
            output.value = value_map[output.value.as_usize()]
                .expect("every declared output is a reachable value");
        }
    }

    fn validate_internal(&self, diagnostics: &mut Vec<ValidationDiagnostic>) {
        if let Some(reason) = self.internal_graph_error() {
            diagnostics.push(ValidationDiagnostic::InvalidInternalGraph { reason });
        }
    }

    fn internal_graph_error(&self) -> Option<&'static str> {
        let actual_input_keys: HashSet<_> =
            self.inputs.iter().map(|input| input.key.clone()).collect();
        if actual_input_keys.len() != self.inputs.len() || actual_input_keys != self.input_keys {
            return Some("input key index does not match the ordered interface");
        }
        let actual_output_keys: HashSet<_> = self
            .outputs
            .iter()
            .map(|output| output.key.clone())
            .collect();
        if actual_output_keys.len() != self.outputs.len() || actual_output_keys != self.output_keys
        {
            return Some("output key index does not match the ordered interface");
        }
        if self
            .values
            .iter()
            .any(|value| validate_shape(&value.shape).is_err())
        {
            return Some("a value has an unsupported shape");
        }
        if self
            .values
            .iter()
            .any(|value| !self.semantic_registry.contains(&value.resolved_type))
        {
            return Some("a value type is absent from the frozen semantic registry");
        }
        for (position, input) in self.inputs.iter().enumerate() {
            let Some(value) = self.values.get(input.value.as_usize()) else {
                return Some("an input references an invalid value");
            };
            let Some(input_index) = InputIndex::from_len(position) else {
                return Some("the input interface exceeds its fixed-width index space");
            };
            if !matches!(value.definition, ValueDefinition::Input { input_index: actual } if actual == input_index)
            {
                return Some("an input value has the wrong definition");
            }
        }
        for (position, operation) in self.operations.iter().enumerate() {
            let Some(operation_index) = OperationIndex::from_len(position) else {
                return Some("the operation arena exceeds its fixed-width index space");
            };
            if operation.results.len() != 1 {
                return Some("a bounded-profile operation does not have exactly one result");
            }
            let result_index = operation.results[0];
            let Some(result) = self.values.get(result_index.as_usize()) else {
                return Some("an operation references an invalid result value");
            };
            if !matches!(result.definition, ValueDefinition::OperationResult { operation, result_index } if operation == operation_index && result_index == ResultIndex::ZERO)
            {
                return Some("an operation result has the wrong definition");
            }
            if operation.operands.iter().any(|operand| {
                operand.as_usize() >= self.values.len() || operand.get() >= result_index.get()
            }) {
                return Some("an operation operand is invalid or not topologically prior");
            }
            if !self.operation_contract_holds(operation, result) {
                return Some("an operation violates its arity, attributes, or shape contract");
            }
        }
        if self.inputs.len().checked_add(self.operations.len()) != Some(self.values.len()) {
            return Some("the value arena contains an unowned value");
        }
        if self
            .outputs
            .iter()
            .any(|output| output.value.as_usize() >= self.values.len())
        {
            return Some("an output references an invalid value");
        }
        None
    }

    fn operation_contract_holds(&self, operation: &OperationData, result: &ValueData) -> bool {
        let f32_type = F32::resolved_type();
        match &operation.kind {
            OperationKind::ConstantF32 { .. } => {
                operation.operands.is_empty()
                    && result.shape.rank() == 0
                    && result.resolved_type.as_ref() == &f32_type
            }
            OperationKind::MultiplyF32 | OperationKind::AddF32 => {
                if operation.operands.len() != 2 {
                    return false;
                }
                let left = &self.values[operation.operands[0].as_usize()];
                let right = &self.values[operation.operands[1].as_usize()];
                if left.resolved_type.as_ref() != &f32_type
                    || right.resolved_type.as_ref() != &f32_type
                    || result.resolved_type.as_ref() != &f32_type
                {
                    return false;
                }
                let expected = if left.shape.rank() == 0 {
                    &right.shape
                } else if right.shape.rank() == 0 || left.shape == right.shape {
                    &left.shape
                } else {
                    return false;
                };
                &result.shape == expected
            }
            OperationKind::StrictSerialSumF32 { axes } => {
                if operation.operands.len() != 1 || axes.is_empty() {
                    return false;
                }
                let input_value = &self.values[operation.operands[0].as_usize()];
                if input_value.resolved_type.as_ref() != &f32_type
                    || result.resolved_type.as_ref() != &f32_type
                {
                    return false;
                }
                let input = &input_value.shape;
                let canonical = axes.windows(2).all(|pair| pair[0] < pair[1])
                    && axes.iter().all(|axis| {
                        usize::try_from(axis.get()).is_ok_and(|axis| axis < input.rank())
                    });
                canonical && result.shape == input.without_axes(axes)
            }
        }
    }
}

fn validate_shape(shape: &Shape) -> Result<(), BuildError> {
    if u32::try_from(shape.rank()).is_err() {
        return Err(BuildError::RankTooLarge { rank: shape.rank() });
    }
    Ok(())
}

fn checked_index(index: usize, entity: EntityKind) -> Result<ValueIndex, BuildError> {
    ValueIndex::from_len(index).ok_or(BuildError::TooManyEntities { entity })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Shape;

    fn input_key(value: &str) -> InputKey {
        InputKey::new(value).unwrap()
    }
    fn output_key(value: &str) -> OutputKey {
        OutputKey::new(value).unwrap()
    }

    fn program(dead_first: bool, share: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input_f32(input_key("x"), Shape::from_dims([2, 3]))
            .unwrap();
        if dead_first {
            let _ = builder.scalar_f32(f32::NAN).unwrap();
        }
        let scale = builder.scalar_f32_bits((-0.0_f32).to_bits()).unwrap();
        let first = builder.multiply_f32(x, scale).unwrap();
        let second = if share {
            first
        } else {
            builder.multiply_f32(x, scale).unwrap()
        };
        if !dead_first {
            let _ = builder.scalar_f32(f32::NAN).unwrap();
        }
        builder.output(output_key("first"), first).unwrap();
        builder.output(output_key("second"), second).unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn failed_edits_are_transactional() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input_f32(input_key("x"), Shape::from_dims([2]))
            .unwrap();
        let before = (
            builder.inputs.len(),
            builder.values.len(),
            builder.operations.len(),
            builder.outputs.len(),
            builder.input_keys.len(),
            builder.output_keys.len(),
        );
        assert!(matches!(
            builder.input_f32(input_key("x"), Shape::from_dims([2])),
            Err(BuildError::DuplicateInputKey(_))
        ));
        assert_eq!(
            before,
            (
                builder.inputs.len(),
                builder.values.len(),
                builder.operations.len(),
                builder.outputs.len(),
                builder.input_keys.len(),
                builder.output_keys.len()
            )
        );
        let mut foreign = SemanticProgramBuilder::try_standard().unwrap();
        let y = foreign
            .input_f32(input_key("y"), Shape::from_dims([2]))
            .unwrap();
        assert!(matches!(
            builder.add_f32(x, y),
            Err(BuildError::ForeignValue {
                role: ValueRole::OperationOperand { index: 1 }
            })
        ));
        assert_eq!(
            before,
            (
                builder.inputs.len(),
                builder.values.len(),
                builder.operations.len(),
                builder.outputs.len(),
                builder.input_keys.len(),
                builder.output_keys.len()
            )
        );
    }

    #[test]
    fn f32_admission_requires_registered_semantic_authority() {
        use crate::semantic::{
            ProviderIdentity, SemanticRegistryBuilder, SemanticRegistryProvider,
            SemanticRegistryRegistrar, ValueTypeDefinition, ValueTypeMarker,
        };

        enum ExternalOnly {}
        impl ValueTypeMarker for ExternalOnly {}
        struct ExternalOnlyProvider;
        impl SemanticRegistryProvider for ExternalOnlyProvider {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("acme", "external-only", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), crate::semantic::RegistryError> {
                registrar.register_value_type::<ExternalOnly>(ValueTypeDefinition::new(
                    ResolvedValueType::nominal(
                        crate::semantic::TypeKey::new("acme", "external", 1).unwrap(),
                    ),
                    "https://example.invalid/external/v1",
                    crate::semantic::ResolvedValueTypeArgument::boolean(true),
                )?)
            }
        }

        let mut registry = SemanticRegistryBuilder::new();
        registry.register_provider(&ExternalOnlyProvider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();

        assert!(matches!(
            builder.input_f32(input_key("x"), Shape::from_dims([1])),
            Err(BuildError::UnregisteredValueType { .. })
        ));
        assert!(builder.inputs.is_empty());
        assert!(builder.values.is_empty());
    }

    #[test]
    fn failed_build_returns_builder_for_retry() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input_f32(input_key("x"), Shape::from_dims([1]))
            .unwrap();
        let error = builder.build().unwrap_err();
        assert_eq!(
            error.diagnostics().unwrap().as_slice(),
            &[ValidationDiagnostic::NoProgramOutputs]
        );
        let mut builder = error.into_builder();
        builder.output(output_key("x"), x).unwrap();
        assert_eq!(builder.build().unwrap().output_count(), 1);
    }

    #[test]
    fn completed_owner_exhaustion_returns_the_intact_builder() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = builder.scalar_f32(1.0).unwrap();
        builder.output(output_key("result"), value).unwrap();

        let error = builder.build_with_completed_owner(None).unwrap_err();
        assert!(matches!(
            error.failure(),
            ProgramBuildFailure::GraphIdentityExhausted
        ));
        assert!(error.diagnostics().is_none());

        let mut builder = error.into_builder();
        let increment = builder.scalar_f32(2.0).unwrap();
        let sum = builder.add_f32(value, increment).unwrap();
        builder.output(output_key("sum"), sum).unwrap();
        assert_eq!(builder.build().unwrap().output_count(), 2);
    }

    #[test]
    fn handle_admission_distinguishes_owner_locality_and_argument_role() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let local = builder.scalar_f32(1.0).unwrap();
        let invalid = ValueId {
            owner: builder.owner,
            index: ValueIndex::from_verified_len(builder.values.len() + 10),
        };
        let mut foreign_builder = SemanticProgramBuilder::try_standard().unwrap();
        let foreign = foreign_builder.scalar_f32(2.0).unwrap();

        assert_eq!(
            builder.add_f32(foreign, local),
            Err(BuildError::ForeignValue {
                role: ValueRole::OperationOperand { index: 0 }
            })
        );
        assert_eq!(
            builder.add_f32(local, invalid),
            Err(BuildError::InvalidLocalValue {
                role: ValueRole::OperationOperand { index: 1 }
            })
        );
        assert_eq!(
            builder.output(output_key("foreign"), foreign),
            Err(BuildError::ForeignValue {
                role: ValueRole::ProgramOutput
            })
        );
        assert_eq!(
            builder.output(output_key("invalid"), invalid),
            Err(BuildError::InvalidLocalValue {
                role: ValueRole::ProgramOutput
            })
        );
        assert_eq!(builder.operations.len(), 1);
        assert_eq!(builder.values.len(), 1);
        assert_eq!(builder.outputs.len(), 0);
    }

    #[test]
    fn handles_fail_closed_across_graphs() {
        let first = program(false, true);
        let second = program(false, true);
        let foreign = first.values().next().unwrap().id();
        assert_eq!(
            second.value(foreign).unwrap_err(),
            HandleError::ForeignGraph {
                entity: EntityKind::Value
            }
        );
        let foreign_operation = first.operations().next().unwrap().id();
        assert_eq!(
            second.operation(foreign_operation).unwrap_err(),
            HandleError::ForeignGraph {
                entity: EntityKind::Operation
            }
        );
        let invalid_value = ValueId {
            owner: second.data.owner,
            index: ValueIndex::from_verified_len(second.value_count() + 10),
        };
        assert_eq!(
            second.value(invalid_value).unwrap_err(),
            HandleError::InvalidLocal {
                entity: EntityKind::Value
            }
        );
        let invalid_operation = OperationId {
            owner: second.data.owner,
            index: OperationIndex::from_verified_len(second.operation_count() + 10),
        };
        assert_eq!(
            second.operation(invalid_operation).unwrap_err(),
            HandleError::InvalidLocal {
                entity: EntityKind::Operation
            }
        );
        let invalid_output = OutputSelector {
            origin: second.data.origin,
            key: output_key("missing"),
        };
        assert_eq!(
            second.resolve_output(&invalid_output).unwrap_err(),
            HandleError::InvalidLocal {
                entity: EntityKind::Output
            }
        );
    }

    #[test]
    fn commitment_compacts_the_live_closure_and_invalidates_draft_handles() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let dead_input = builder
            .input_f32(input_key("dead"), Shape::from_dims([4]))
            .unwrap();
        let _dead_result = builder
            .strict_serial_sum_f32(dead_input, [Axis::new(0)])
            .unwrap();
        let live_input = builder
            .input_f32(input_key("live"), Shape::from_dims([2]))
            .unwrap();
        let scale = builder.scalar_f32(3.0).unwrap();
        let result = builder.multiply_f32(live_input, scale).unwrap();
        let selector = builder.output(output_key("result"), result).unwrap();

        let program = builder.build().unwrap();
        assert_eq!(program.input_count(), 1);
        assert_eq!(program.operation_count(), 2);
        assert_eq!(program.value_count(), 3);
        assert_eq!(program.inputs().next().unwrap().key().as_str(), "live");

        let completed = program.resolve_output(&selector).unwrap().value();
        assert_eq!(program.shape(completed).unwrap(), &Shape::from_dims([2]));
        assert!(matches!(
            program.value(result),
            Err(HandleError::ForeignGraph {
                entity: EntityKind::Value
            })
        ));
        for (expected, value) in program.values().enumerate() {
            assert_eq!(value.id().index.as_usize(), expected);
        }
        for (expected, operation) in program.operations().enumerate() {
            assert_eq!(operation.id().index.as_usize(), expected);
            for operand in operation.operands() {
                program.value(operand).unwrap();
            }
            for result in operation.results() {
                program.value(result).unwrap();
            }
        }
    }

    #[test]
    fn output_selectors_are_bound_to_the_originating_draft() {
        fn build() -> (SemanticProgram, OutputSelector) {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let value = builder.scalar_f32(1.0).unwrap();
            let selector = builder.output(output_key("same-key"), value).unwrap();
            (builder.build().unwrap(), selector)
        }

        let (first, first_selector) = build();
        let (second, second_selector) = build();
        assert_eq!(
            first.resolve_output(&first_selector).unwrap().key(),
            first_selector.key()
        );
        assert!(matches!(
            first.resolve_output(&second_selector),
            Err(HandleError::ForeignGraph {
                entity: EntityKind::Output
            })
        ));
        assert!(matches!(
            second.resolve_output(&first_selector),
            Err(HandleError::ForeignGraph {
                entity: EntityKind::Output
            })
        ));
    }

    #[test]
    fn clones_share_storage_and_identity_cache() {
        let first = program(false, true);
        let second = first.clone();
        assert!(Arc::ptr_eq(&first.data, &second.data));
        assert!(std::ptr::eq(
            first.canonical_identity(),
            second.canonical_identity()
        ));
    }

    #[test]
    fn semantic_program_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SemanticProgram>();
    }

    #[test]
    fn identity_ignores_dead_insertion_order_but_preserves_sharing() {
        assert_eq!(
            program(true, true).canonical_identity(),
            program(false, true).canonical_identity()
        );
        assert_ne!(
            program(false, true).canonical_identity(),
            program(false, false).canonical_identity()
        );
    }

    #[test]
    fn identity_preserves_exact_float_bits_and_output_order() {
        fn identity(bits: u32, reverse: bool) -> CanonicalIdentity {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let x = builder
                .input_f32(input_key("x"), Shape::from_dims([1]))
                .unwrap();
            let scalar = builder.scalar_f32_bits(bits).unwrap();
            let value = builder.add_f32(x, scalar).unwrap();
            if reverse {
                builder.output(output_key("copy"), value).unwrap();
                builder.output(output_key("result"), value).unwrap();
            } else {
                builder.output(output_key("result"), value).unwrap();
                builder.output(output_key("copy"), value).unwrap();
            }
            builder.build().unwrap().canonical_identity().clone()
        }

        assert_ne!(
            identity(0.0_f32.to_bits(), false),
            identity((-0.0_f32).to_bits(), false)
        );
        assert_ne!(
            identity(0.0_f32.to_bits(), false),
            identity(0.0_f32.to_bits(), true)
        );
    }

    #[test]
    fn canonical_identity_handles_a_deep_chain_iteratively() {
        const DEPTH: usize = 50_000;

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let mut value = builder.scalar_f32(1.0).unwrap();
        let increment = builder.scalar_f32(0.0).unwrap();
        for _ in 0..DEPTH {
            value = builder.add_f32(value, increment).unwrap();
        }
        builder.output(output_key("result"), value).unwrap();
        let program = builder.build().unwrap();

        assert!(!program.canonical_identity().as_bytes().is_empty());
    }

    #[test]
    fn semantic_admission_does_not_depend_on_host_dense_element_count() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let shape = Shape::from_dims([u64::MAX, 2]);
        assert_eq!(shape.element_count(), None);
        let value = builder.input_f32(input_key("huge"), shape.clone()).unwrap();
        let output = builder.output(output_key("huge"), value).unwrap();
        let program = builder.build().unwrap();
        let completed = program.resolve_output(&output).unwrap().value();

        assert_eq!(program.shape(completed).unwrap(), &shape);
        assert!(matches!(
            program.shape(value),
            Err(HandleError::ForeignGraph {
                entity: EntityKind::Value
            })
        ));
    }

    #[test]
    fn all_rejected_operation_edits_preserve_arena_lengths() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input_f32(input_key("x"), Shape::from_dims([2, 3]))
            .unwrap();
        let y = builder
            .input_f32(input_key("y"), Shape::from_dims([2, 4]))
            .unwrap();
        let before = (
            builder.operations.len(),
            builder.values.len(),
            builder.outputs.len(),
        );
        assert!(matches!(
            builder.add_f32(x, y),
            Err(BuildError::IncompatiblePointwiseShapes { .. })
        ));
        assert_eq!(
            before,
            (
                builder.operations.len(),
                builder.values.len(),
                builder.outputs.len()
            )
        );
        assert_eq!(
            builder.strict_serial_sum_f32(x, []),
            Err(BuildError::EmptyReductionAxes)
        );
        assert_eq!(
            builder.strict_serial_sum_f32(x, [Axis::new(1), Axis::new(1)]),
            Err(BuildError::NonCanonicalReductionAxes)
        );
        assert_eq!(
            before,
            (
                builder.operations.len(),
                builder.values.len(),
                builder.outputs.len()
            )
        );
        builder.output(output_key("x"), x).unwrap();
        let after_output = (
            builder.operations.len(),
            builder.values.len(),
            builder.outputs.len(),
            builder.output_keys.len(),
        );
        assert!(matches!(
            builder.output(output_key("x"), x),
            Err(BuildError::DuplicateOutputKey(_))
        ));
        assert_eq!(
            after_output,
            (
                builder.operations.len(),
                builder.values.len(),
                builder.outputs.len(),
                builder.output_keys.len()
            )
        );
    }

    #[test]
    fn direct_views_preserve_order_and_definitions() {
        let program = program(false, true);
        let inputs: Vec<_> = program.inputs().collect();
        assert_eq!(inputs[0].key().as_str(), "x");
        let values: Vec<_> = program.values().collect();
        assert!(matches!(
            values[0].definition(),
            super::super::operation::Definition::Input { input_index }
                if input_index.get() == 0
        ));
        assert_eq!(values[0].resolved_type(), &F32::resolved_type());
        assert_eq!(
            program.semantic_registry().resolve_marker::<F32>().unwrap(),
            &F32::resolved_type()
        );
        assert_eq!(program.outputs().count(), 2);
    }
}
