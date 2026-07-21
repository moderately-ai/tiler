use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use crate::shape::{Shape, ShapeEvidence};

use super::error::{
    BuildError, BuilderCreateError, EntityKind, HandleError, ProgramBuildError,
    ProgramBuildFailure, ReifyError, ShapeRefineError, ShapeWitnessError, ShapeWitnessSubject,
    ValidationDiagnostic, ValidationDiagnostics, ValueRole,
};
use super::handles::{
    GraphId, OperationId, OperationIndex, Value, ValueId, ValueIndex, next_graph_id,
};
use super::identity::CanonicalIdentity;
use super::interface::{
    InputIndex, InputKey, Output, OutputKey, OutputSelector, ProgramInput, ProgramInputRef,
    ProgramOutput, ProgramOutputRef, TypedProgramOutputRef,
};
use super::operation::{
    OperationAttributes, OperationData, OperationRef, ResultIndex, ValueData, ValueDefinition,
    ValueFact, ValueRef,
};
use super::registry::FrozenSemanticRegistry;
use super::shape_evidence::{SameShape, ShapeWitness, ShapedValue};
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

    /// Checks and attaches Rust-side shape evidence to a typed value.
    ///
    /// Refinement does not mutate the program or alter semantic identity.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign/invalid value or when `E` disagrees
    /// with the authoritative graph shape.
    pub fn refine<T, E: ShapeEvidence>(
        &self,
        value: Value<T>,
    ) -> Result<ShapedValue<T, E>, ShapeRefineError> {
        let actual = self
            .shape(value.erase())
            .map_err(ShapeRefineError::Handle)?;
        refine_shape(value, actual)
    }

    /// Proves that two ordered values have equal authoritative shapes.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid subjects or unequal shapes.
    pub fn prove_same_shape<L, R>(
        &self,
        left: Value<L>,
        right: Value<R>,
    ) -> Result<ShapeWitness<SameShape>, ShapeWitnessError> {
        prove_same_shape(
            self.data.owner,
            left.erase(),
            right.erase(),
            |subject, value| {
                self.shape(value)
                    .map_err(|error| ShapeWitnessError::SubjectHandle { subject, error })
            },
        )
    }

    /// Validates a same-shape witness against this graph and exact subjects.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid subjects, a foreign witness, or a
    /// witness proving a different ordered pair.
    pub fn validate_same_shape_witness<L, R>(
        &self,
        witness: &ShapeWitness<SameShape>,
        left: Value<L>,
        right: Value<R>,
    ) -> Result<(), ShapeWitnessError> {
        self.shape(left.erase())
            .map_err(|error| ShapeWitnessError::SubjectHandle {
                subject: ShapeWitnessSubject::Left,
                error,
            })?;
        self.shape(right.erase())
            .map_err(|error| ShapeWitnessError::SubjectHandle {
                subject: ShapeWitnessSubject::Right,
                error,
            })?;
        validate_same_shape_witness(self.data.owner, witness, left.erase(), right.erase())
    }

    /// Returns the immutable semantic authority that validated this program.
    #[must_use]
    pub fn semantic_registry(&self) -> &FrozenSemanticRegistry {
        &self.data.semantic_registry
    }

    /// Recovers exact marker-backed type evidence for one graph-owned value.
    ///
    /// # Errors
    ///
    /// Returns [`ReifyError`] for a foreign/invalid handle, an unbound marker,
    /// or an exact resolved-type mismatch.
    pub fn reify<T: super::registry::ValueTypeMarker>(
        &self,
        value: ValueId,
    ) -> Result<Value<T>, ReifyError> {
        let actual = self
            .value(value)
            .map_err(ReifyError::Handle)?
            .resolved_type()
            .clone();
        let expected = self
            .data
            .semantic_registry
            .resolve_marker::<T>()
            .map_err(ReifyError::RegistryLookup)?;
        if &actual != expected {
            return Err(ReifyError::TypeMismatch {
                expected: Arc::new(expected.clone()),
                actual: Arc::new(actual),
            });
        }
        Ok(Value::from_verified(value))
    }

    /// Resolves a typed selector produced by the committed draft.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a selector from another draft or for an
    /// invalid local selector.
    pub fn resolve_typed_output<T: super::registry::ValueTypeMarker>(
        &self,
        selector: &Output<T>,
    ) -> Result<TypedProgramOutputRef<'_, T>, ReifyError> {
        let output = self
            .resolve_output(selector.selector())
            .map_err(ReifyError::Handle)?;
        let _ = self.reify::<T>(output.value())?;
        Ok(TypedProgramOutputRef::from_verified(output))
    }
}

fn refine_shape<T, E: ShapeEvidence>(
    value: Value<T>,
    actual: &Shape,
) -> Result<ShapedValue<T, E>, ShapeRefineError> {
    if E::matches(actual) {
        Ok(ShapedValue::from_verified(value))
    } else {
        Err(ShapeRefineError::EvidenceMismatch {
            expected: E::expectation(),
            actual: actual.clone(),
        })
    }
}

fn prove_same_shape<'a>(
    owner: GraphId,
    left: ValueId,
    right: ValueId,
    mut shape: impl FnMut(ShapeWitnessSubject, ValueId) -> Result<&'a Shape, ShapeWitnessError>,
) -> Result<ShapeWitness<SameShape>, ShapeWitnessError> {
    let left_shape = shape(ShapeWitnessSubject::Left, left)?;
    let right_shape = shape(ShapeWitnessSubject::Right, right)?;
    if left_shape != right_shape {
        return Err(ShapeWitnessError::NotSameShape {
            left: left_shape.clone(),
            right: right_shape.clone(),
        });
    }
    Ok(ShapeWitness::from_verified(owner, left, right))
}

fn validate_same_shape_witness(
    owner: GraphId,
    witness: &ShapeWitness<SameShape>,
    left: ValueId,
    right: ValueId,
) -> Result<(), ShapeWitnessError> {
    if witness.owner != owner {
        return Err(ShapeWitnessError::ForeignWitness);
    }
    if witness.left != left || witness.right != right {
        return Err(ShapeWitnessError::SubjectMismatch);
    }
    Ok(())
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

    /// Adds an ordered fixed-shape input through an exact registered marker.
    ///
    /// # Errors
    ///
    /// Returns a typed error for duplicate keys, unsupported shapes, or exhausted IDs.
    pub fn input<T: super::registry::ValueTypeMarker>(
        &mut self,
        key: InputKey,
        shape: Shape,
    ) -> Result<Value<T>, BuildError> {
        let resolved_type = self
            .semantic_registry
            .resolve_marker::<T>()
            .map_err(BuildError::RegistryLookup)?
            .clone();
        self.input_resolved(key, shape, resolved_type)
            .map(Value::from_verified)
    }

    /// Adds a checked runtime-resolved input for parsed or generated frontends.
    ///
    /// This is an unknown-typed path, not an `any` escape hatch: the frozen
    /// semantic registry must admit the complete supplied type.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an unregistered type, duplicate key,
    /// unsupported shape, or exhausted IDs.
    pub fn input_resolved(
        &mut self,
        key: InputKey,
        shape: Shape,
        resolved_type: ResolvedValueType,
    ) -> Result<ValueId, BuildError> {
        validate_shape(&shape)?;
        self.semantic_registry
            .validate_type(&resolved_type)
            .map_err(BuildError::SemanticRegistry)?;
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
            resolved_type: Arc::new(resolved_type),
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

    /// Applies one registered semantic operation through the sole checked,
    /// transactional admission path.
    ///
    /// Result facts are derived exclusively by the frozen semantic authority;
    /// callers cannot declare result types or shapes.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid handles, missing authority, rejected
    /// semantics, unsupported inferred shapes, or exhausted graph IDs. The
    /// builder is unchanged on every error.
    pub fn apply(
        &mut self,
        key: super::operation::OpKey,
        attributes: OperationAttributes,
        operands: &[ValueId],
    ) -> Result<Vec<ValueId>, BuildError> {
        self.push_operation(key, attributes, operands, |_, _| Ok(()))
    }

    pub(super) fn apply_typed_single<T: super::registry::ValueTypeMarker>(
        &mut self,
        key: super::operation::OpKey,
        attributes: OperationAttributes,
        operands: &[ValueId],
    ) -> Result<Value<T>, BuildError> {
        self.apply_typed_single_checked(key, attributes, operands, |_| Ok(()))
    }

    pub(super) fn apply_shaped_single<T: super::registry::ValueTypeMarker, E: ShapeEvidence>(
        &mut self,
        key: super::operation::OpKey,
        attributes: OperationAttributes,
        operands: &[ValueId],
    ) -> Result<ShapedValue<T, E>, BuildError> {
        self.apply_typed_single_checked(key, attributes, operands, |fact| {
            if E::matches(fact.shape()) {
                Ok(())
            } else {
                Err(BuildError::ShapeRefinement(
                    ShapeRefineError::EvidenceMismatch {
                        expected: E::expectation(),
                        actual: fact.shape().clone(),
                    },
                ))
            }
        })
        .map(ShapedValue::from_verified)
    }

    fn apply_typed_single_checked<T, F>(
        &mut self,
        key: super::operation::OpKey,
        attributes: OperationAttributes,
        operands: &[ValueId],
        validate_fact: F,
    ) -> Result<Value<T>, BuildError>
    where
        T: super::registry::ValueTypeMarker,
        F: FnOnce(&ValueFact) -> Result<(), BuildError>,
    {
        let mut results = self.push_operation(key, attributes, operands, |registry, facts| {
            let expected = registry
                .resolve_marker::<T>()
                .map_err(BuildError::RegistryLookup)?;
            let [fact] = facts else {
                return Err(BuildError::TypedResultArity {
                    expected: 1,
                    actual: facts.len(),
                });
            };
            if fact.resolved_type() != expected {
                return Err(BuildError::Reify(ReifyError::TypeMismatch {
                    expected: Arc::new(expected.clone()),
                    actual: Arc::new(fact.resolved_type().clone()),
                }));
            }
            validate_fact(fact)
        })?;
        let result = results
            .pop()
            .expect("single-result facade was validated before graph mutation");
        Ok(Value::from_verified(result))
    }

    /// Adds an ordered named output with exact static type evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign value or duplicate output key.
    pub fn output<T: super::registry::ValueTypeMarker>(
        &mut self,
        key: OutputKey,
        value: Value<T>,
    ) -> Result<Output<T>, BuildError> {
        self.output_resolved(key, value.erase())
            .map(Output::from_verified)
    }

    /// Adds an ordered named output from an unknown-typed identity.
    ///
    /// The value remains authoritatively typed in the graph; this method only
    /// omits static Rust evidence for parsed frontends.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign value or duplicate output key.
    pub fn output_resolved(
        &mut self,
        key: OutputKey,
        value: ValueId,
    ) -> Result<OutputSelector, BuildError> {
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

    fn push_operation<F>(
        &mut self,
        key: super::operation::OpKey,
        attributes: OperationAttributes,
        operands: &[ValueId],
        validate_results: F,
    ) -> Result<Vec<ValueId>, BuildError>
    where
        F: FnOnce(&FrozenSemanticRegistry, &[ValueFact]) -> Result<(), BuildError>,
    {
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
        let operand_facts: Vec<_> = operand_indices
            .iter()
            .map(|index| {
                let value = &self.values[index.as_usize()];
                ValueFact::new(value.resolved_type.as_ref().clone(), value.shape.clone())
            })
            .collect();
        let inferred = self
            .semantic_registry
            .infer_operation(&key, &operand_facts, &attributes)
            .map_err(BuildError::SemanticRegistry)?;
        for fact in &inferred {
            validate_shape(fact.shape())?;
        }
        validate_results(&self.semantic_registry, &inferred)?;
        let operation_index =
            OperationIndex::from_len(self.operations.len()).ok_or(BuildError::TooManyEntities {
                entity: EntityKind::Operation,
            })?;
        for offset in 0..inferred.len() {
            let index =
                self.values
                    .len()
                    .checked_add(offset)
                    .ok_or(BuildError::TooManyEntities {
                        entity: EntityKind::Value,
                    })?;
            checked_index(index, EntityKind::Value)?;
            ResultIndex::from_len(offset).ok_or(BuildError::TooManyEntities {
                entity: EntityKind::Value,
            })?;
        }
        let mut result_indices = Vec::with_capacity(inferred.len());
        let mut result_ids = Vec::with_capacity(inferred.len());
        for (offset, fact) in inferred.into_iter().enumerate() {
            let value_index = ValueIndex::from_verified_len(self.values.len());
            let result_index = ResultIndex::from_len(offset).expect("result capacity was checked");
            self.values.push(ValueData {
                definition: ValueDefinition::OperationResult {
                    operation: operation_index,
                    result_index,
                },
                shape: fact.shape,
                resolved_type: Arc::new(fact.resolved_type),
            });
            result_indices.push(value_index);
            result_ids.push(ValueId {
                owner: self.owner,
                index: value_index,
            });
        }
        self.operations.push(OperationData {
            key,
            attributes,
            operands: operand_indices,
            results: result_indices,
        });
        Ok(result_ids)
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

    /// Recovers exact marker-backed type evidence for one draft-owned value.
    ///
    /// # Errors
    ///
    /// Returns [`ReifyError`] for a foreign/invalid handle, an unbound marker,
    /// or an exact resolved-type mismatch.
    pub fn reify<T: super::registry::ValueTypeMarker>(
        &self,
        value: ValueId,
    ) -> Result<Value<T>, ReifyError> {
        let index = self
            .value_index(value, ValueRole::OperationOperand { index: 0 })
            .map_err(|error| match error {
                BuildError::ForeignValue { .. } => ReifyError::Handle(HandleError::ForeignGraph {
                    entity: EntityKind::Value,
                }),
                BuildError::InvalidLocalValue { .. } => {
                    ReifyError::Handle(HandleError::InvalidLocal {
                        entity: EntityKind::Value,
                    })
                }
                _ => unreachable!("value lookup returns only handle failures"),
            })?;
        let actual = &self.values[index.as_usize()].resolved_type;
        let expected = self
            .semantic_registry
            .resolve_marker::<T>()
            .map_err(ReifyError::RegistryLookup)?;
        if actual.as_ref() != expected {
            return Err(ReifyError::TypeMismatch {
                expected: Arc::new(expected.clone()),
                actual: Arc::clone(actual),
            });
        }
        Ok(Value::from_verified(value))
    }

    /// Checks and attaches Rust-side shape evidence to a typed draft value.
    ///
    /// Refinement does not mutate the builder or alter semantic identity.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign/invalid value or when `E` disagrees
    /// with the authoritative graph shape.
    pub fn refine<T, E: ShapeEvidence>(
        &self,
        value: Value<T>,
    ) -> Result<ShapedValue<T, E>, ShapeRefineError> {
        let actual = self
            .shape_for_handle(value.erase())
            .map_err(ShapeRefineError::Handle)?;
        refine_shape(value, actual)
    }

    /// Proves that two ordered draft values have equal authoritative shapes.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid subjects or unequal shapes.
    pub fn prove_same_shape<L, R>(
        &self,
        left: Value<L>,
        right: Value<R>,
    ) -> Result<ShapeWitness<SameShape>, ShapeWitnessError> {
        prove_same_shape(self.owner, left.erase(), right.erase(), |subject, value| {
            self.shape_for_handle(value)
                .map_err(|error| ShapeWitnessError::SubjectHandle { subject, error })
        })
    }

    /// Validates a same-shape witness against this draft and exact subjects.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid subjects, a foreign witness, or a
    /// witness proving a different ordered pair.
    pub fn validate_same_shape_witness<L, R>(
        &self,
        witness: &ShapeWitness<SameShape>,
        left: Value<L>,
        right: Value<R>,
    ) -> Result<(), ShapeWitnessError> {
        self.shape_for_handle(left.erase())
            .map_err(|error| ShapeWitnessError::SubjectHandle {
                subject: ShapeWitnessSubject::Left,
                error,
            })?;
        self.shape_for_handle(right.erase())
            .map_err(|error| ShapeWitnessError::SubjectHandle {
                subject: ShapeWitnessSubject::Right,
                error,
            })?;
        validate_same_shape_witness(self.owner, witness, left.erase(), right.erase())
    }

    fn shape_for_handle(&self, value: ValueId) -> Result<&Shape, HandleError> {
        if value.owner != self.owner {
            return Err(HandleError::ForeignGraph {
                entity: EntityKind::Value,
            });
        }
        self.values
            .get(value.index.as_usize())
            .map(|data| &data.shape)
            .ok_or(HandleError::InvalidLocal {
                entity: EntityKind::Value,
            })
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
            if operation.results.is_empty() {
                return Some("an operation has no results");
            }
            for (position, result_value) in operation.results.iter().copied().enumerate() {
                let Some(result) = self.values.get(result_value.as_usize()) else {
                    return Some("an operation references an invalid result value");
                };
                let Some(expected_result_index) = ResultIndex::from_len(position) else {
                    return Some("an operation exceeds its fixed-width result space");
                };
                if !matches!(result.definition, ValueDefinition::OperationResult { operation, result_index } if operation == operation_index && result_index == expected_result_index)
                {
                    return Some("an operation result has the wrong definition");
                }
            }
            let first_result = operation.results[0];
            if operation.operands.iter().any(|operand| {
                operand.as_usize() >= self.values.len() || operand.get() >= first_result.get()
            }) {
                return Some("an operation operand is invalid or not topologically prior");
            }
            if !self.operation_contract_holds(operation) {
                return Some("an operation violates its arity, attributes, or shape contract");
            }
        }
        let result_count = self
            .operations
            .iter()
            .try_fold(0_usize, |count, operation| {
                count.checked_add(operation.results.len())
            });
        if result_count.and_then(|count| self.inputs.len().checked_add(count))
            != Some(self.values.len())
        {
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

    fn operation_contract_holds(&self, operation: &OperationData) -> bool {
        let operand_facts: Vec<_> = operation
            .operands
            .iter()
            .map(|operand| {
                let value = &self.values[operand.as_usize()];
                ValueFact::new(value.resolved_type.as_ref().clone(), value.shape.clone())
            })
            .collect();
        let Ok(expected) = self.semantic_registry.infer_operation(
            &operation.key,
            &operand_facts,
            &operation.attributes,
        ) else {
            return false;
        };
        expected.len() == operation.results.len()
            && operation
                .results
                .iter()
                .zip(expected)
                .all(|(actual, expected)| {
                    let actual = &self.values[actual.as_usize()];
                    actual.shape == expected.shape
                        && actual.resolved_type.as_ref() == &expected.resolved_type
                })
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
    use super::super::{
        CanonicalValue, F32, F32Add, F32Constant, F32Multiply, NormativeDefinitionRef, OpKey,
        OperationArity, OperationConformance, OperationDefinition, OperationDefinitionFacts,
        OperationEffect, OperationInferenceError, OperationInferencer, OperationSchema,
        ProviderIdentity, SemanticRegistryBuilder, SemanticRegistryProvider,
        SemanticRegistryRegistrar, StrictSerialF32Sum, add_f32_op,
    };
    use super::*;
    use crate::shape::{Axis, Shape, StaticShape};

    fn input_key(value: &str) -> InputKey {
        InputKey::new(value).unwrap()
    }
    fn output_key(value: &str) -> OutputKey {
        OutputKey::new(value).unwrap()
    }

    fn constant_bits(
        builder: &mut SemanticProgramBuilder,
        bits: u32,
    ) -> Result<Value<F32>, BuildError> {
        F32Constant::apply(builder, bits)
    }

    fn constant(
        builder: &mut SemanticProgramBuilder,
        value: f32,
    ) -> Result<Value<F32>, BuildError> {
        constant_bits(builder, value.to_bits())
    }

    fn multiply(
        builder: &mut SemanticProgramBuilder,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        F32Multiply::apply(builder, left, right)
    }

    fn add(
        builder: &mut SemanticProgramBuilder,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        F32Add::apply(builder, left, right)
    }

    fn sum(
        builder: &mut SemanticProgramBuilder,
        input: Value<F32>,
        axes: impl IntoIterator<Item = Axis>,
    ) -> Result<Value<F32>, BuildError> {
        StrictSerialF32Sum::apply(builder, input, axes)
    }

    struct Identity;
    impl OperationInferencer for Identity {
        fn infer(
            &self,
            operands: &[ValueFact],
            attributes: &OperationAttributes,
        ) -> Result<Vec<ValueFact>, OperationInferenceError> {
            if operands.len() == 1 && attributes.fields().is_empty() {
                Ok(vec![operands[0].clone()])
            } else {
                Err(OperationInferenceError::new(
                    "test.identity.signature",
                    "identity requires one operand and no attributes",
                ))
            }
        }
    }

    struct Pair;
    impl OperationInferencer for Pair {
        fn infer(
            &self,
            operands: &[ValueFact],
            _: &OperationAttributes,
        ) -> Result<Vec<ValueFact>, OperationInferenceError> {
            Ok(vec![operands[0].clone(), operands[0].clone()])
        }
    }

    struct OperationProvider;
    impl SemanticRegistryProvider for OperationProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "operations", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), super::super::RegistryError> {
            registrar.register_operation(test_operation("identity", 1, Arc::new(Identity)))?;
            registrar.register_operation(test_operation("pair", 2, Arc::new(Pair)))
        }
    }

    fn test_operation(
        name: &str,
        results: u32,
        inferencer: Arc<dyn OperationInferencer>,
    ) -> OperationDefinition {
        OperationDefinition::new(
            OpKey::new("test", name, 1).unwrap(),
            OperationSchema::new(OperationArity::exact(1), OperationArity::exact(results), [])
                .unwrap(),
            NormativeDefinitionRef::new(format!("test {name} v1")).unwrap(),
            OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            OperationConformance::new(CanonicalValue::utf8(format!("test.{name}.v1")).unwrap()),
            OperationEffect::Pure,
            inferencer,
        )
    }

    fn program(dead_first: bool, share: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input::<F32>(input_key("x"), Shape::from_dims([2, 3]))
            .unwrap();
        if dead_first {
            let _ = constant(&mut builder, f32::NAN).unwrap();
        }
        let scale = constant_bits(&mut builder, (-0.0_f32).to_bits()).unwrap();
        let first = multiply(&mut builder, x, scale).unwrap();
        let second = if share {
            first
        } else {
            multiply(&mut builder, x, scale).unwrap()
        };
        if !dead_first {
            let _ = constant(&mut builder, f32::NAN).unwrap();
        }
        builder.output(output_key("first"), first).unwrap();
        builder.output(output_key("second"), second).unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn failed_edits_are_transactional() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input::<F32>(input_key("x"), Shape::from_dims([2]))
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
            builder.input::<F32>(input_key("x"), Shape::from_dims([2])),
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
            .input::<F32>(input_key("y"), Shape::from_dims([2]))
            .unwrap();
        assert!(matches!(
            add(&mut builder, x, y),
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
    fn typed_and_resolved_inputs_require_their_distinct_registry_authority() {
        use crate::semantic::{
            NormativeDefinitionRef, ProviderIdentity, SemanticRegistryBuilder,
            SemanticRegistryProvider, SemanticRegistryRegistrar, TypeDefinitionFacts,
            ValueTypeDefinition, ValueTypeDefinitionKey,
        };

        struct ExternalOnlyProvider;
        impl SemanticRegistryProvider for ExternalOnlyProvider {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("acme", "external-only", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), crate::semantic::RegistryError> {
                registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                    ValueTypeDefinitionKey::Nominal(
                        crate::semantic::TypeKey::new("acme", "external", 1).unwrap(),
                    ),
                    NormativeDefinitionRef::new("https://example.invalid/external/v1")?,
                    TypeDefinitionFacts::new(crate::semantic::CanonicalValue::boolean(true)),
                ))
            }
        }

        let mut registry = SemanticRegistryBuilder::new();
        registry.register_provider(&ExternalOnlyProvider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();

        assert!(matches!(
            builder.input::<F32>(input_key("x"), Shape::from_dims([1])),
            Err(BuildError::RegistryLookup(
                super::super::RegistryLookupError::UnregisteredMarker { .. }
            ))
        ));
        assert!(matches!(
            builder.input_resolved(
                input_key("resolved"),
                Shape::from_dims([1]),
                F32::resolved_type()
            ),
            Err(BuildError::SemanticRegistry(
                super::super::RegistryError::UnregisteredTypeAuthority { .. }
            ))
        ));
        assert!(builder.inputs.is_empty());
        assert!(builder.values.is_empty());
    }

    #[test]
    fn failed_build_returns_builder_for_retry() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input::<F32>(input_key("x"), Shape::from_dims([1]))
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
    fn reification_requires_an_exact_marker_binding() {
        struct External;
        impl super::super::ValueTypeMarker for External {}

        struct Provider;
        impl SemanticRegistryProvider for Provider {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("test", "external-type", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), super::super::RegistryError> {
                let resolved = ResolvedValueType::nominal(
                    super::super::TypeKey::new("test", "external", 1).unwrap(),
                );
                registrar.register_marked_value_type::<External>(
                    super::super::ValueTypeDefinition::structurally_valid(
                        super::super::ValueTypeDefinitionKey::Nominal(
                            super::super::TypeKey::new("test", "external", 1).unwrap(),
                        ),
                        NormativeDefinitionRef::new("test external v1")?,
                        super::super::TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
                    ),
                    resolved,
                )
            }
        }

        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry.register_provider(&Provider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let value: Value<F32> = builder
            .input(input_key("f32"), Shape::from_dims([1]))
            .unwrap();

        assert!(matches!(
            builder.reify::<External>(value.erase()),
            Err(ReifyError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn completed_owner_exhaustion_returns_the_intact_builder() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = constant(&mut builder, 1.0).unwrap();
        builder.output(output_key("result"), value).unwrap();

        let error = builder.build_with_completed_owner(None).unwrap_err();
        assert!(matches!(
            error.failure(),
            ProgramBuildFailure::GraphIdentityExhausted
        ));
        assert!(error.diagnostics().is_none());

        let mut builder = error.into_builder();
        let increment = constant(&mut builder, 2.0).unwrap();
        let sum = add(&mut builder, value, increment).unwrap();
        builder.output(output_key("sum"), sum).unwrap();
        assert_eq!(builder.build().unwrap().output_count(), 2);
    }

    #[test]
    fn handle_admission_distinguishes_owner_locality_and_argument_role() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let local = constant(&mut builder, 1.0).unwrap();
        let invalid = ValueId {
            owner: builder.owner,
            index: ValueIndex::from_verified_len(builder.values.len() + 10),
        };
        let mut foreign_builder = SemanticProgramBuilder::try_standard().unwrap();
        let foreign = constant(&mut foreign_builder, 2.0).unwrap();

        assert_eq!(
            add(&mut builder, foreign, local),
            Err(BuildError::ForeignValue {
                role: ValueRole::OperationOperand { index: 0 }
            })
        );
        assert!(matches!(
            builder.apply(
                add_f32_op(),
                OperationAttributes::empty(),
                &[local.erase(), invalid]
            ),
            Err(BuildError::InvalidLocalValue {
                role: ValueRole::OperationOperand { index: 1 }
            })
        ));
        assert!(matches!(
            builder.output(output_key("foreign"), foreign),
            Err(BuildError::ForeignValue {
                role: ValueRole::ProgramOutput
            })
        ));
        assert_eq!(
            builder.output_resolved(output_key("invalid"), invalid),
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
            .input::<F32>(input_key("dead"), Shape::from_dims([4]))
            .unwrap();
        let _dead_result = sum(&mut builder, dead_input, [Axis::new(0)]).unwrap();
        let live_input = builder
            .input::<F32>(input_key("live"), Shape::from_dims([2]))
            .unwrap();
        let scale = constant(&mut builder, 3.0).unwrap();
        let result = multiply(&mut builder, live_input, scale).unwrap();
        let selector = builder.output(output_key("result"), result).unwrap();

        let program = builder.build().unwrap();
        assert_eq!(program.input_count(), 1);
        assert_eq!(program.operation_count(), 2);
        assert_eq!(program.value_count(), 3);
        assert_eq!(program.inputs().next().unwrap().key().as_str(), "live");

        let completed = program.resolve_typed_output(&selector).unwrap().value();
        assert_eq!(
            program.shape(completed.erase()).unwrap(),
            &Shape::from_dims([2])
        );
        assert!(matches!(
            program.value(result.erase()),
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
        fn build() -> (SemanticProgram, Output<F32>) {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let value = constant(&mut builder, 1.0).unwrap();
            let selector = builder.output(output_key("same-key"), value).unwrap();
            (builder.build().unwrap(), selector)
        }

        let (first, first_selector) = build();
        let (second, second_selector) = build();
        assert_eq!(
            first.resolve_typed_output(&first_selector).unwrap().key(),
            first_selector.key()
        );
        assert!(matches!(
            first.resolve_typed_output(&second_selector),
            Err(ReifyError::Handle(HandleError::ForeignGraph {
                entity: EntityKind::Output
            }))
        ));
        assert!(matches!(
            second.resolve_typed_output(&first_selector),
            Err(ReifyError::Handle(HandleError::ForeignGraph {
                entity: EntityKind::Output
            }))
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
                .input::<F32>(input_key("x"), Shape::from_dims([1]))
                .unwrap();
            let scalar = constant_bits(&mut builder, bits).unwrap();
            let value = add(&mut builder, x, scalar).unwrap();
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
        let mut value = constant(&mut builder, 1.0).unwrap();
        let increment = constant(&mut builder, 0.0).unwrap();
        for _ in 0..DEPTH {
            value = add(&mut builder, value, increment).unwrap();
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
        let value = builder
            .input::<F32>(input_key("huge"), shape.clone())
            .unwrap();
        let output = builder.output(output_key("huge"), value).unwrap();
        let program = builder.build().unwrap();
        let completed = program.resolve_typed_output(&output).unwrap().value();

        assert_eq!(program.shape(completed.erase()).unwrap(), &shape);
        assert!(matches!(
            program.shape(value.erase()),
            Err(HandleError::ForeignGraph {
                entity: EntityKind::Value
            })
        ));
    }

    #[test]
    fn all_rejected_operation_edits_preserve_arena_lengths() {
        fn has_code<T>(result: Result<T, BuildError>, code: &str) -> bool {
            matches!(
                result,
                Err(BuildError::SemanticRegistry(
                    super::super::registry::RegistryError::RejectedOperationApplication(
                        rejection
                    )
                )) if rejection.source_error().code() == code
            )
        }

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let x = builder
            .input::<F32>(input_key("x"), Shape::from_dims([2, 3]))
            .unwrap();
        let y = builder
            .input::<F32>(input_key("y"), Shape::from_dims([2, 4]))
            .unwrap();
        let before = (
            builder.operations.len(),
            builder.values.len(),
            builder.outputs.len(),
        );
        assert!(has_code(add(&mut builder, x, y), "binary.shape"));
        assert_eq!(
            before,
            (
                builder.operations.len(),
                builder.values.len(),
                builder.outputs.len()
            )
        );
        assert!(has_code(sum(&mut builder, x, []), "sum.axes.empty"));
        assert!(has_code(
            sum(&mut builder, x, [Axis::new(1), Axis::new(1)]),
            "sum.axes.canonical"
        ));
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

    #[test]
    fn external_operation_is_admitted_without_a_closed_operation_enum() {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry.register_provider(&OperationProvider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let input = builder
            .input::<F32>(input_key("x"), Shape::from_dims([2, 3]))
            .unwrap();
        let results = builder
            .apply(
                OpKey::new("test", "identity", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap();
        assert_eq!(results.len(), 1);
        let result = builder.reify::<F32>(results[0]).unwrap();
        builder.output(output_key("result"), result).unwrap();
        let program = builder.build().unwrap();

        assert_eq!(
            program.operations().next().unwrap().key().namespace(),
            "test"
        );
        assert_eq!(
            program
                .value(program.outputs().next().unwrap().value())
                .unwrap()
                .resolved_type(),
            &F32::resolved_type()
        );

        let registry = program.semantic_registry().clone();
        let mut shared = SemanticProgramBuilder::try_new(registry.clone()).unwrap();
        let input = shared
            .input::<F32>(input_key("x"), Shape::from_dims([2]))
            .unwrap();
        let pair = shared
            .apply(
                OpKey::new("test", "pair", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap();
        assert_eq!(pair.len(), 2);
        shared.output_resolved(output_key("left"), pair[0]).unwrap();
        shared
            .output_resolved(output_key("right"), pair[1])
            .unwrap();
        let shared = shared.build().unwrap();

        let mut separate = SemanticProgramBuilder::try_new(registry).unwrap();
        let input = separate
            .input::<F32>(input_key("x"), Shape::from_dims([2]))
            .unwrap();
        let first = separate
            .apply(
                OpKey::new("test", "pair", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap();
        let second = separate
            .apply(
                OpKey::new("test", "pair", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap();
        separate
            .output_resolved(output_key("left"), first[0])
            .unwrap();
        separate
            .output_resolved(output_key("right"), second[1])
            .unwrap();
        let separate = separate.build().unwrap();

        assert_ne!(shared.canonical_identity(), separate.canonical_identity());
    }

    #[test]
    fn typed_result_checks_are_transactional() {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry.register_provider(&OperationProvider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let input = builder
            .input::<F32>(input_key("x"), Shape::from_dims([2]))
            .unwrap();
        let operation_count = builder.operations.len();
        let value_count = builder.values.len();

        let arity_error = builder
            .apply_typed_single::<F32>(
                OpKey::new("test", "pair", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap_err();
        assert_eq!(
            arity_error,
            BuildError::TypedResultArity {
                expected: 1,
                actual: 2,
            }
        );
        assert_eq!(builder.operations.len(), operation_count);
        assert_eq!(builder.values.len(), value_count);

        let shape_error = builder
            .apply_shaped_single::<F32, StaticShape<1, { [3] }>>(
                OpKey::new("test", "identity", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap_err();
        assert!(matches!(
            shape_error,
            BuildError::ShapeRefinement(ShapeRefineError::EvidenceMismatch { .. })
        ));
        assert_eq!(builder.operations.len(), operation_count);
        assert_eq!(builder.values.len(), value_count);
    }
}
