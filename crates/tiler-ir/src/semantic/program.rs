use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use crate::shape::{
    ExtentSources, Shape, ShapeEnv, ShapeEvidence, SourcedExtent, SourcedShape,
    empty_environment_identity,
};

use super::error::{
    BuildError, BuilderCreateError, EntityKind, HandleError, ProgramBuildError,
    ProgramBuildFailure, ReifyError, ShapeRefineError, ShapeWitnessError, ShapeWitnessSubject,
    ValidationDiagnostic, ValidationDiagnostics, ValueRole,
};
use super::handles::{
    GraphId, OperationId, OperationIndex, Value, ValueId, ValueIndex, next_graph_id,
};
use super::identity::{
    MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES, SemanticIdentity,
    canonical_coordinates_for_verified, compute_graph_identity, empty_graph_canonical_work_bytes,
    graph_identity_encoded_len_for_verified, input_canonical_work_bytes,
    operation_canonical_work_bytes, output_canonical_work_bytes,
};
use super::interface::{
    InputIndex, InputKey, Output, OutputKey, OutputSelector, ProgramInput, ProgramInputRef,
    ProgramOutput, ProgramOutputRef, TypedProgramOutputRef,
};
use super::operation::{
    MAX_OPERATION_OPERANDS, OperationAttributes, OperationData, OperationRef, ResultIndex,
    ValueData, ValueDefinition, ValueFact, ValueRef,
};
use super::precondition::{
    MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES, SemanticPreconditionData,
    SemanticPreconditionDisproof, SemanticPreconditionOrdinal, SemanticPreconditionStatus,
    StaticAssessment, StaticValueEvidence, assess_static_precondition,
    initialize_obligation_identities, obligation_identity_total_encoded_len,
};
use super::registry::{
    FrozenSemanticRegistry, SemanticAdmissionProvenanceIdentity,
    SemanticDefinitionProjectionIdentity, SemanticRegistrySnapshotIdentity,
};
use super::shape_evidence::{SameShape, ShapeWitness, ShapedValue};
use super::types::CanonicalValueView;
use super::types::ResolvedValueType;

/// A verified, immutable semantic tensor program.
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
    pub(super) semantic_identity: OnceLock<SemanticIdentity>,
    pub(super) graph_identity_encoded_len: usize,
    pub(super) canonical_work_bytes: usize,
    pub(super) canonical_value_ids: Vec<u64>,
    pub(super) canonical_operation_ordinals: Vec<u32>,
    pub(super) reached_definitions: SemanticDefinitionProjectionIdentity,
    pub(super) admission_provenance: SemanticAdmissionProvenanceIdentity,
    pub(super) registry_snapshot: SemanticRegistrySnapshotIdentity,
    pub(super) semantic_registry: FrozenSemanticRegistry,
    pub(super) extent_sources: Option<ExtentSources>,
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
                program: &self.data,
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
                program: &self.data,
                operation,
            })
            .ok_or(HandleError::InvalidLocal {
                entity: EntityKind::Operation,
            })
    }

    pub(crate) fn canonical_operation_ordinal(&self, operation: OperationRef<'_>) -> u32 {
        debug_assert_eq!(operation.owner, self.data.owner);
        self.data.canonical_operation_ordinals[operation.index.as_usize()]
    }

    #[cfg(test)]
    pub(crate) fn canonical_operation_ordinal_count(&self) -> usize {
        self.data.canonical_operation_ordinals.len()
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

    /// Returns the shape of a graph-owned value and where each extent comes from.
    ///
    /// The one total view rather than a fixed accessor paired with an optional
    /// symbolic one: a paired accessor makes the caller choose which question to
    /// ask, and fails silently for the caller that only ever asks the first when
    /// a third source kind arrives. A pass that handles only literals reads
    /// [`SourcedShape::as_static`] once and refuses the rest with its own typed
    /// reason.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign or invalid local handle.
    pub fn shape(&self, value: ValueId) -> Result<&SourcedShape, HandleError> {
        self.value(value)?;
        Ok(&self.data.values[value.index.as_usize()].shape)
    }

    /// Returns the environment this program's symbolic extents resolve against.
    ///
    /// `None` for a program built without one, which is every program whose
    /// extents are all literals. A symbol means nothing without the environment
    /// that declares and binds it, so this is what an inspector needs to
    /// interpret one found on a boundary — matching
    /// [`VerifiedIndexRegion::extent_sources`](crate::index::VerifiedIndexRegion::extent_sources).
    ///
    /// Absence here is not absence from identity: the environment's identity is
    /// a *total* subject of [`SemanticIdentity`], and a program with no
    /// environment reports the empty environment's identity.
    #[must_use]
    pub fn extent_sources(&self) -> Option<&ExtentSources> {
        self.data.extent_sources.as_ref()
    }

    /// Returns the fixed shape of a graph-owned value, for a literal one only.
    fn static_shape(&self, value: ValueId) -> Result<Option<&Shape>, HandleError> {
        self.shape(value).map(SourcedShape::as_static)
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
            .static_shape(value.erase())
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
                self.static_shape(value)
                    .map_err(|error| ShapeWitnessError::SubjectHandle { subject, error })?
                    .ok_or(ShapeWitnessError::SymbolicShape { subject })
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

    /// Returns the complete, internally consistent semantic identity bundle.
    ///
    /// Graph meaning, reached definitions, admission provenance, the full
    /// registry snapshot, and the shape environment remain available through
    /// named borrowed accessors on [`SemanticIdentity`].
    #[must_use]
    pub fn semantic_identity(&self) -> &SemanticIdentity {
        self.data.semantic_identity.get_or_init(|| {
            SemanticIdentity::new(
                compute_graph_identity(&self.data),
                self.data.reached_definitions.clone(),
                self.data.admission_provenance.clone(),
                self.data.registry_snapshot.clone(),
                shape_environment_identity(self.data.extent_sources.as_ref()),
            )
        })
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
    actual: Option<&Shape>,
) -> Result<ShapedValue<T, E>, ShapeRefineError> {
    let Some(actual) = actual else {
        return Err(ShapeRefineError::SymbolicShape {
            expected: E::expectation(),
        });
    };
    if E::matches(actual) {
        Ok(ShapedValue::from_verified(value))
    } else {
        Err(ShapeRefineError::EvidenceMismatch {
            expected: E::expectation(),
            actual: actual.clone(),
        })
    }
}

/// Returns the identity of the environment a program's extents resolve in.
///
/// Total: a program that named no symbol and was never given an environment
/// reports the empty environment's identity, so the fifth semantic subject is
/// a value for every program rather than a presence to be framed.
fn shape_environment_identity(sources: Option<&ExtentSources>) -> crate::shape::ShapeEnvIdentity {
    sources.map_or_else(
        || empty_environment_identity().clone(),
        |sources| sources.environment_identity().clone(),
    )
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
    canonical_work_bytes: usize,
    extent_sources: Option<ExtentSources>,
}

impl SemanticProgramBuilder {
    #[cfg(test)]
    pub(super) const fn retained_canonical_work_bytes(&self) -> usize {
        self.canonical_work_bytes
    }

    fn clone_for_identity_preflight(&self) -> Self {
        Self {
            owner: self.owner,
            inputs: self.inputs.clone(),
            operations: self.operations.clone(),
            values: self.values.clone(),
            outputs: self.outputs.clone(),
            input_keys: self.input_keys.clone(),
            output_keys: self.output_keys.clone(),
            semantic_registry: self.semantic_registry.clone(),
            canonical_work_bytes: self.canonical_work_bytes,
            extent_sources: self.extent_sources.clone(),
        }
    }

    /// Tries to create an empty builder with a distinct graph owner.
    ///
    /// # Errors
    ///
    /// Returns [`BuilderCreateError::GraphIdentityExhausted`] without creating
    /// a builder when the process-local owner space is exhausted.
    pub fn try_new(semantic_registry: FrozenSemanticRegistry) -> Result<Self, BuilderCreateError> {
        Self::open(semantic_registry, None)
    }

    fn open(
        semantic_registry: FrozenSemanticRegistry,
        extent_sources: Option<ExtentSources>,
    ) -> Result<Self, BuilderCreateError> {
        Ok(Self {
            owner: next_graph_id().ok_or(BuilderCreateError::GraphIdentityExhausted)?,
            inputs: Vec::new(),
            operations: Vec::new(),
            values: Vec::new(),
            outputs: Vec::new(),
            input_keys: HashSet::new(),
            output_keys: HashSet::new(),
            semantic_registry,
            canonical_work_bytes: empty_graph_canonical_work_bytes(),
            extent_sources,
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

    /// Tries to create a standard-registry builder whose symbolic input extents
    /// resolve in one verified environment.
    ///
    /// **A constructor rather than a setter, and there is no setter.** A program
    /// has exactly one environment for the whole of its life: a second one would
    /// silently reinterpret every extent already authored against the first, and
    /// the program's identity — which folds the environment's identity as its
    /// fifth subject — would name whichever happened to be installed last.
    /// Fixing the environment before any input exists makes that replacement
    /// unrepresentable rather than merely discouraged, which is the same reason
    /// [`IndexRegionBuilder::new_with_shape_environment`](crate::index::IndexRegionBuilder::new_with_shape_environment)
    /// takes one and offers no setter.
    ///
    /// # Errors
    ///
    /// Returns a typed error if standard registry construction or graph-owner
    /// allocation fails.
    pub fn try_standard_with_shape_environment(
        environment: Arc<ShapeEnv>,
    ) -> Result<Self, BuilderCreateError> {
        let registry =
            FrozenSemanticRegistry::standard().map_err(BuilderCreateError::StandardRegistry)?;
        Self::open(registry, Some(ExtentSources::new(environment)))
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
        let resolved_type = self.marker_type::<T>()?;
        self.input_resolved(key, shape, resolved_type)
            .map(Value::from_verified)
    }

    /// Adds an ordered input whose extents may name declared `ShapeEnv` symbols.
    ///
    /// Beside [`Self::input`] rather than replacing it: a wholly static caller
    /// needs no environment and is not asked for an absent one, exactly as
    /// [`IndexRegionBuilder::tensor`](crate::index::IndexRegionBuilder::tensor)
    /// stands beside
    /// [`IndexRegionBuilder::sourced_tensor`](crate::index::IndexRegionBuilder::sourced_tensor).
    ///
    /// # Errors
    ///
    /// Returns the errors [`Self::input_resolved_sourced`] returns, and
    /// additionally a typed error when the Rust marker is unbound in the frozen
    /// registry.
    pub fn input_sourced<T: super::registry::ValueTypeMarker>(
        &mut self,
        key: InputKey,
        extents: Vec<SourcedExtent>,
    ) -> Result<Value<T>, BuildError> {
        let resolved_type = self.marker_type::<T>()?;
        self.input_resolved_sourced(key, extents, resolved_type)
            .map(Value::from_verified)
    }

    fn marker_type<T: super::registry::ValueTypeMarker>(
        &self,
    ) -> Result<ResolvedValueType, BuildError> {
        self.semantic_registry
            .resolve_marker::<T>()
            .map_err(BuildError::RegistryLookup)
            .cloned()
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
        self.push_input(key, SourcedShape::from_shape(shape), resolved_type)
    }

    /// Adds a checked runtime-resolved input whose extents may name symbols.
    ///
    /// Every symbolic extent is admitted against this program's one environment
    /// before the input exists, so a refused source leaves the draft exactly as
    /// it was rather than half-applied. An extent whose binding arrives after
    /// [`EXTENT_PHASE_CEILING`](crate::shape::EXTENT_PHASE_CEILING) is refused
    /// here, at the constructor, and not deferred to
    /// [`Self::build`]: a shape that is not evaluable before device work begins
    /// is not a program this layer can hold.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::ExtentSource`] when no environment is bound, when
    /// the environment does not declare a named symbol, or when a symbol's root
    /// binding arrives too late; [`BuildError::ShapeVocabulary`] when the shape
    /// vocabulary cannot represent the normalized boundary; and the errors
    /// [`Self::input_resolved`] returns for an unregistered type, a duplicate
    /// key, an unsupported rank, or exhausted IDs.
    pub fn input_resolved_sourced(
        &mut self,
        key: InputKey,
        extents: Vec<SourcedExtent>,
        resolved_type: ResolvedValueType,
    ) -> Result<ValueId, BuildError> {
        for extent in &extents {
            let Some(symbol) = extent.symbol() else {
                continue;
            };
            let Some(sources) = self.extent_sources.as_ref() else {
                // No environment can declare the symbol, so it is undeclared
                // here for exactly the reason the variant names.
                return Err(BuildError::ExtentSource(
                    crate::shape::ExtentSourceError::UndeclaredSymbol {
                        symbol: symbol.clone(),
                    },
                ));
            };
            sources.admit(extent).map_err(BuildError::ExtentSource)?;
        }
        let shape = SourcedShape::sourced(extents).map_err(BuildError::ShapeVocabulary)?;
        self.push_input(key, shape, resolved_type)
    }

    fn push_input(
        &mut self,
        key: InputKey,
        shape: SourcedShape,
        resolved_type: ResolvedValueType,
    ) -> Result<ValueId, BuildError> {
        validate_rank(shape.rank())?;
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
        let added_canonical_work = input_canonical_work_bytes(&key, &resolved_type, &shape);
        let canonical_work_bytes = self.reserve_canonical_work(added_canonical_work)?;
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
        self.canonical_work_bytes = canonical_work_bytes;
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
        let canonical_work_bytes =
            self.reserve_canonical_work(output_canonical_work_bytes(&key))?;
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
        self.canonical_work_bytes = canonical_work_bytes;
        Ok(selector)
    }

    /// Checks all whole-program invariants without consuming the builder.
    ///
    /// # Errors
    ///
    /// Returns all diagnostics found in deterministic validation order.
    pub fn validate(&self) -> Result<(), ValidationDiagnostics> {
        self.validate_and_project_authority().map(|_| ())
    }

    fn validate_and_project_authority(
        &self,
    ) -> Result<
        (
            SemanticDefinitionProjectionIdentity,
            SemanticAdmissionProvenanceIdentity,
        ),
        ValidationDiagnostics,
    > {
        let mut diagnostics = Vec::new();
        if self.outputs.is_empty() {
            diagnostics.push(ValidationDiagnostic::NoProgramOutputs);
        }
        self.validate_internal(&mut diagnostics);
        if let Some(errors) = ValidationDiagnostics::new(diagnostics) {
            return Err(errors);
        }
        self.project_reachable_semantic_authority()
            .map_err(|error| {
                ValidationDiagnostics::new(vec![ValidationDiagnostic::SemanticAuthority(error)])
                    .expect("semantic authority failure creates one diagnostic")
            })
    }

    /// Validates and compacts this draft into an immutable shared program.
    ///
    /// # Errors
    ///
    /// Returns the exact failure together with the intact builder when
    /// validation or completed-owner allocation fails.
    pub fn build(self) -> Result<SemanticProgram, ProgramBuildError> {
        self.build_with_owner_allocator(next_graph_id)
    }

    fn build_with_owner_allocator(
        mut self,
        allocate_owner: impl FnOnce() -> Option<GraphId>,
    ) -> Result<SemanticProgram, ProgramBuildError> {
        let (reached_definitions, admission_provenance) =
            match self.validate_and_project_authority() {
                Ok(projections) => projections,
                Err(diagnostics) => {
                    return Err(ProgramBuildError {
                        builder: Box::new(self),
                        failure: ProgramBuildFailure::Validation(diagnostics),
                    });
                }
            };
        let registry_snapshot = self.semantic_registry.snapshot_identity().clone();
        let has_residual_preconditions = self.operations.iter().any(|operation| {
            operation
                .semantic_preconditions
                .iter()
                .any(|precondition| precondition.status == SemanticPreconditionStatus::Residual)
        });
        let obligation_identity_bytes = if has_residual_preconditions {
            let mut preview = self.clone_for_identity_preflight();
            preview.compact_to_outputs();
            let mut preview_data = ProgramData {
                owner: preview.owner,
                origin: preview.owner,
                inputs: preview.inputs,
                operations: preview.operations,
                values: preview.values,
                outputs: preview.outputs,
                semantic_identity: OnceLock::new(),
                graph_identity_encoded_len: 0,
                canonical_work_bytes: preview.canonical_work_bytes,
                canonical_value_ids: Vec::new(),
                canonical_operation_ordinals: Vec::new(),
                reached_definitions: reached_definitions.clone(),
                admission_provenance: admission_provenance.clone(),
                registry_snapshot: registry_snapshot.clone(),
                semantic_registry: preview.semantic_registry,
                extent_sources: preview.extent_sources,
            };
            let (canonical_value_ids, canonical_operation_ordinals) =
                canonical_coordinates_for_verified(&preview_data);
            preview_data.canonical_value_ids = canonical_value_ids;
            preview_data.canonical_operation_ordinals = canonical_operation_ordinals;
            preview_data.graph_identity_encoded_len =
                graph_identity_encoded_len_for_verified(&preview_data);
            obligation_identity_total_encoded_len(&preview_data)
        } else {
            0
        };
        if let Err(failure) = validate_obligation_identity_bytes(obligation_identity_bytes) {
            return Err(ProgramBuildError {
                builder: Box::new(self),
                failure,
            });
        }
        let Some(completed_owner) = allocate_owner() else {
            return Err(ProgramBuildError {
                builder: Box::new(self),
                failure: ProgramBuildFailure::GraphIdentityExhausted,
            });
        };
        let origin = self.owner;
        self.compact_to_outputs();
        let mut data = ProgramData {
            owner: completed_owner,
            origin,
            inputs: self.inputs,
            operations: self.operations,
            values: self.values,
            outputs: self.outputs,
            semantic_identity: OnceLock::new(),
            graph_identity_encoded_len: 0,
            canonical_work_bytes: self.canonical_work_bytes,
            canonical_value_ids: Vec::new(),
            canonical_operation_ordinals: Vec::new(),
            reached_definitions,
            admission_provenance,
            registry_snapshot,
            semantic_registry: self.semantic_registry,
            extent_sources: self.extent_sources,
        };
        let (canonical_value_ids, canonical_operation_ordinals) =
            canonical_coordinates_for_verified(&data);
        data.canonical_value_ids = canonical_value_ids;
        data.canonical_operation_ordinals = canonical_operation_ordinals;
        data.graph_identity_encoded_len = graph_identity_encoded_len_for_verified(&data);
        assert!(
            data.graph_identity_encoded_len <= data.canonical_work_bytes
                && data.graph_identity_encoded_len <= MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES,
            "verified graph identity exceeds its admitted canonical-work budget"
        );
        if obligation_identity_bytes != 0 {
            debug_assert_eq!(
                obligation_identity_total_encoded_len(&data),
                obligation_identity_bytes
            );
            let graph = compute_graph_identity(&data);
            initialize_obligation_identities(&mut data, &graph);
            data.semantic_identity
                .set(SemanticIdentity::new(
                    graph,
                    data.reached_definitions.clone(),
                    data.admission_provenance.clone(),
                    data.registry_snapshot.clone(),
                    shape_environment_identity(data.extent_sources.as_ref()),
                ))
                .expect("completed program semantic identity initializes exactly once");
        }
        Ok(SemanticProgram {
            data: Arc::new(data),
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
        if operands.len() > MAX_OPERATION_OPERANDS as usize {
            return Err(BuildError::TooManyOperationOperands {
                actual: operands.len(),
                limit: MAX_OPERATION_OPERANDS,
            });
        }
        let operand_indices: Vec<_> = operands
            .iter()
            .enumerate()
            .map(|(index, operand)| {
                self.value_index(
                    *operand,
                    ValueRole::OperationOperand {
                        index: u32::try_from(index).map_err(|_| {
                            BuildError::TooManyOperationOperands {
                                actual: operands.len(),
                                limit: MAX_OPERATION_OPERANDS,
                            }
                        })?,
                    },
                )
            })
            .collect::<Result<_, _>>()?;
        let operand_facts: Vec<_> = operand_indices
            .iter()
            .enumerate()
            .map(|(position, index)| {
                let value = &self.values[index.as_usize()];
                // Refused here rather than at `build`, and refused rather than
                // specialized: an operand naming a symbol has no fixed shape to
                // hand the frozen authority, and substituting one the
                // environment merely determines would encode a program nobody
                // wrote.
                let shape = value.shape.as_static().ok_or_else(|| {
                    BuildError::SymbolicOperandUnsupported {
                        role: ValueRole::OperationOperand {
                            index: u32::try_from(position)
                                .expect("operand count was bounded above"),
                        },
                    }
                })?;
                Ok(ValueFact::new(
                    value.resolved_type.as_ref().clone(),
                    shape.clone(),
                ))
            })
            .collect::<Result<_, BuildError>>()?;
        let attributes = self
            .semantic_registry
            .normalize_operation_attributes(&key, attributes)
            .map_err(BuildError::SemanticRegistry)?;
        let inferred = self
            .semantic_registry
            .infer_operation(&key, &operand_facts, &attributes)
            .map_err(BuildError::SemanticRegistry)?;
        for fact in &inferred {
            validate_rank(fact.shape().rank())?;
        }
        validate_results(&self.semantic_registry, &inferred)?;
        let semantic_preconditions =
            self.assess_semantic_preconditions(&key, &operand_indices, &operand_facts)?;
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
        let added_canonical_work = operation_canonical_work_bytes(
            &key,
            &attributes,
            operand_indices.len(),
            &inferred,
            semantic_preconditions.len(),
        );
        let canonical_work_bytes = self.reserve_canonical_work(added_canonical_work)?;
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
                shape: SourcedShape::from_shape(fact.shape),
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
            semantic_preconditions,
        });
        self.canonical_work_bytes = canonical_work_bytes;
        Ok(result_ids)
    }

    fn assess_semantic_preconditions(
        &self,
        key: &super::operation::OpKey,
        operands: &[ValueIndex],
        operand_facts: &[ValueFact],
    ) -> Result<Vec<SemanticPreconditionData>, BuildError> {
        let definition = self
            .semantic_registry
            .operation_definition(key)
            .expect("successful inference resolved the operation definition");
        let mut assessed = Vec::with_capacity(definition.semantic_preconditions().as_slice().len());
        let mut disproof: Option<SemanticPreconditionDisproof> = None;
        for (position, declaration) in definition
            .semantic_preconditions()
            .as_slice()
            .iter()
            .enumerate()
        {
            let ordinal = SemanticPreconditionOrdinal::from_verified_position(position);
            let operand_position = usize::try_from(declaration.operand().get())
                .expect("u32 fits every supported host usize");
            let subject = operands[operand_position];
            let fact = &operand_facts[operand_position];
            let evidence = self.static_value_evidence(subject);
            let (status, proof_basis) = match assess_static_precondition(
                declaration,
                fact.resolved_type(),
                fact.shape(),
                evidence,
            ) {
                StaticAssessment::Proven(proof_basis) => {
                    (SemanticPreconditionStatus::Proven, Some(proof_basis))
                }
                StaticAssessment::Residual => (SemanticPreconditionStatus::Residual, None),
                StaticAssessment::Disproved => {
                    let candidate = SemanticPreconditionDisproof::new(
                        key.clone(),
                        declaration,
                        ordinal,
                        ValueId {
                            owner: self.owner,
                            index: subject,
                        },
                        Arc::clone(&self.values[subject.as_usize()].resolved_type),
                        fact.shape().clone(),
                    );
                    let replace = disproof.as_ref().is_none_or(|prior| {
                        super::precondition::semantic_disproof_precedes(&candidate, prior)
                    });
                    if replace {
                        disproof = Some(candidate);
                    }
                    continue;
                }
            };
            assessed.push(SemanticPreconditionData {
                ordinal,
                subject,
                status,
                proof_basis,
                obligation_identity: None,
            });
        }
        if let Some(disproof) = disproof {
            return Err(BuildError::SemanticPreconditionDisproved(Arc::new(
                disproof,
            )));
        }
        Ok(assessed)
    }

    fn static_value_evidence(&self, subject: ValueIndex) -> Option<StaticValueEvidence> {
        let value = self.values.get(subject.as_usize())?;
        let f32_type = super::registry::F32::resolved_type();
        if value.resolved_type.as_ref() != &f32_type || value.shape.rank() != 0 {
            return None;
        }
        let ValueDefinition::OperationResult {
            operation,
            result_index,
        } = value.definition
        else {
            return None;
        };
        if result_index.get() != 0 {
            return None;
        }
        let producer = self.operations.get(operation.as_usize())?;
        if producer.key != super::operation::constant_f32_op() {
            return None;
        }
        if !self
            .semantic_registry
            .is_standard_static_evidence_authority(&producer.key)
        {
            return None;
        }
        let CanonicalValueView::FloatBits(bits) = producer
            .attributes
            .get(super::operation::F32_CONSTANT_BITS_ATTRIBUTE)?
            .view()
        else {
            return None;
        };
        if bits.format() != f32_type.nominal_key()? || bits.bits().len() != 4 {
            return None;
        }
        let bytes: [u8; 4] = bits.bits().try_into().ok()?;
        Some(StaticValueEvidence::F32ScalarBits(u32::from_be_bytes(
            bytes,
        )))
    }

    fn reserve_canonical_work(&self, added: usize) -> Result<usize, BuildError> {
        let actual = self.canonical_work_bytes.saturating_add(added);
        if actual > MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES {
            return Err(BuildError::CanonicalWorkExceeded {
                actual,
                limit: MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES,
            });
        }
        Ok(actual)
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
            .map(SourcedShape::as_static)
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
                .map(SourcedShape::as_static)
                .map_err(|error| ShapeWitnessError::SubjectHandle { subject, error })?
                .ok_or(ShapeWitnessError::SymbolicShape { subject })
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

    fn shape_for_handle(&self, value: ValueId) -> Result<&SourcedShape, HandleError> {
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
            for precondition in &mut operation.semantic_preconditions {
                precondition.subject = value_map[precondition.subject.as_usize()]
                    .expect("a reachable precondition retains its exact subject");
            }
            self.operations.push(operation);
        }

        for output in &mut self.outputs {
            output.value = value_map[output.value.as_usize()]
                .expect("every declared output is a reachable value");
        }
    }

    fn project_reachable_semantic_authority(
        &self,
    ) -> Result<
        (
            SemanticDefinitionProjectionIdentity,
            SemanticAdmissionProvenanceIdentity,
        ),
        super::registry::RegistryError,
    > {
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
        let value_types = self
            .values
            .iter()
            .zip(&reachable_values)
            .filter_map(|(value, reached)| reached.then_some(value.resolved_type.as_ref()));
        let operations: Vec<_> = self
            .operations
            .iter()
            .zip(&reachable_operations)
            .filter_map(|(operation, reached)| reached.then_some(operation))
            .collect();
        let operation_keys = operations.iter().map(|operation| &operation.key);
        let occurrence_attributes = operations
            .iter()
            .flat_map(|operation| operation.attributes.fields())
            .map(super::types::CanonicalField::value);
        self.semantic_registry.project_program_authority(
            value_types,
            operation_keys,
            occurrence_attributes,
        )
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
            .any(|value| validate_rank(value.shape.rank()).is_err())
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
        // A retained operand is literal by construction: `push_operation`
        // refuses a symbolic one before the operation exists. A sourced shape
        // here is therefore a broken invariant, and reporting the contract
        // unheld is the conservative answer this predicate already gives every
        // other way an operation can fail to reproduce.
        let Some(operand_facts) = operation
            .operands
            .iter()
            .map(|operand| {
                let value = &self.values[operand.as_usize()];
                value.shape.as_static().map(|shape| {
                    ValueFact::new(value.resolved_type.as_ref().clone(), shape.clone())
                })
            })
            .collect::<Option<Vec<_>>>()
        else {
            return false;
        };
        let Ok(expected) = self.semantic_registry.infer_operation(
            &operation.key,
            &operand_facts,
            &operation.attributes,
        ) else {
            return false;
        };
        let Ok(expected_preconditions) =
            self.assess_semantic_preconditions(&operation.key, &operation.operands, &operand_facts)
        else {
            return false;
        };
        expected_preconditions == operation.semantic_preconditions
            && expected.len() == operation.results.len()
            && operation
                .results
                .iter()
                .zip(expected)
                .all(|(actual, expected)| {
                    let actual = &self.values[actual.as_usize()];
                    actual.shape.as_static() == Some(&expected.shape)
                        && actual.resolved_type.as_ref() == &expected.resolved_type
                })
    }
}

fn validate_rank(rank: usize) -> Result<(), BuildError> {
    if u32::try_from(rank).is_err() {
        return Err(BuildError::RankTooLarge { rank });
    }
    Ok(())
}

fn checked_index(index: usize, entity: EntityKind) -> Result<ValueIndex, BuildError> {
    ValueIndex::from_len(index).ok_or(BuildError::TooManyEntities { entity })
}

fn validate_obligation_identity_bytes(actual: usize) -> Result<(), ProgramBuildFailure> {
    if actual > MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES {
        return Err(
            ProgramBuildFailure::SemanticPreconditionObligationIdentityBytesExceeded {
                actual,
                limit: MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES,
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::EncodedNumericContract;
    use super::super::{
        AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, F32, F32Add,
        F32Constant, F32Multiply, NormativeDefinitionRef, OpKey, OperationArity,
        OperationAttributeSchema, OperationConformance, OperationDefinition,
        OperationDefinitionFacts, OperationEffect, OperationInferenceError,
        OperationInferenceOutputs, OperationInferenceRequest, OperationInferencer, OperationSchema,
        ProviderDiagnosticCode, ProviderIdentity, QuantSchemeKey, RegistryError,
        SemanticGraphIdentity, SemanticRegistryBuilder, SemanticRegistryProvider,
        SemanticRegistryRegistrar, StrictSerialF32Sum, TypeArguments, TypeDefinitionFacts, TypeKey,
        U4, ValueTypeDefinition, ValueTypeDefinitionKey, add_f32_op,
    };
    use super::*;
    use crate::program::abi::AvailabilityPhase;
    use crate::shape::{
        Axis, BindingSource, EXTENT_PHASE_CEILING, Extent, ExtentRelation, ExtentSourceError,
        ExtentTerm, FactProvenance, RootBinding, SemanticInputConstraint, Shape, ShapeEnvBuilder,
        ShapeSymbol, StaticShape, SymbolScope,
    };

    #[test]
    fn residual_obligation_identity_cache_bound_accepts_boundary_and_rejects_one_over() {
        assert!(
            validate_obligation_identity_bytes(MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES)
                .is_ok()
        );
        assert!(matches!(
            validate_obligation_identity_bytes(
                MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES + 1
            ),
            Err(ProgramBuildFailure::SemanticPreconditionObligationIdentityBytesExceeded {
                actual,
                limit,
            }) if actual == MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES + 1
                && limit == MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES
        ));
    }

    // --- The sourced-shape surface -----------------------------------------
    //
    // Every fixture below is a *pair*: the refused program beside the accepted
    // neighbour it differs from in exactly the refused fact, so a refusal that
    // started firing for an unrelated reason would take its neighbour with it.

    fn scope() -> SymbolScope {
        SymbolScope::new("program/0").unwrap()
    }

    fn sym(name: &str) -> ShapeSymbol {
        ShapeSymbol::new(scope(), name).unwrap()
    }

    /// A symbol bound to one axis of a named input's shape metadata.
    fn axis_binding(input: &str, axis: u32, phase: AvailabilityPhase) -> RootBinding {
        RootBinding::new(
            BindingSource::InputDimension {
                input: input_key(input),
                axis: Axis::new(axis),
            },
            phase,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    /// An environment declaring `n` over one input axis, with given relations.
    fn env_over(
        input: &str,
        axis: u32,
        phase: AvailabilityPhase,
        relations: &[ExtentRelation],
    ) -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        let declared = sym("n");
        draft.declare(declared.clone()).unwrap();
        draft
            .bind(&declared, axis_binding(input, axis, phase))
            .unwrap();
        for relation in relations {
            draft
                .require(SemanticInputConstraint::new(
                    relation.clone(),
                    FactProvenance::FrontendRequired,
                ))
                .unwrap();
        }
        Arc::new(draft.build().unwrap())
    }

    fn env() -> Arc<ShapeEnv> {
        env_over("rows", 0, EXTENT_PHASE_CEILING, &[])
    }

    /// An environment declaring `n` and `m` over two axes of one input.
    ///
    /// Both symbols in *one* environment, so a program naming `n` and a program
    /// naming `m` share every other subject — the environment identity included
    /// — and the only thing that can separate them is the graph bytes.
    fn two_symbol_env() -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        for (name, axis) in [("n", 0), ("m", 1)] {
            let declared = sym(name);
            draft.declare(declared.clone()).unwrap();
            draft
                .bind(&declared, axis_binding("rows", axis, EXTENT_PHASE_CEILING))
                .unwrap();
        }
        Arc::new(draft.build().unwrap())
    }

    /// A one-input, one-output program whose input has the given extents.
    ///
    /// The output *is* the input, so nothing downstream of a symbolic value is
    /// constructed: at this boundary a symbolic extent reaches an input and
    /// travels no further, which is what `apply` refuses below.
    fn sourced_program(
        environment: Option<Arc<ShapeEnv>>,
        extents: Vec<SourcedExtent>,
    ) -> Result<SemanticProgram, BuildError> {
        let mut builder = match environment {
            Some(environment) => {
                SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap()
            }
            None => SemanticProgramBuilder::try_standard().unwrap(),
        };
        let value = builder.input_sourced::<F32>(input_key("rows"), extents)?;
        builder.output(output_key("out"), value)?;
        Ok(builder.build().expect("the one-input program verifies"))
    }

    #[test]
    fn a_symbolic_input_builds_and_is_visible_through_the_total_shape_view() {
        let environment = env();
        let program = sourced_program(
            Some(Arc::clone(&environment)),
            vec![SourcedExtent::Symbol(sym("n"))],
        )
        .expect("a declared, in-time symbol is admitted");

        let value = program.inputs().next().unwrap().value();
        let shape = program.shape(value).unwrap();
        assert_eq!(shape.rank(), 1);
        assert_eq!(
            shape.extents().collect::<Vec<_>>(),
            vec![SourcedExtent::Symbol(sym("n"))],
            "the boundary exposes the symbol it was sourced from, not a resolved value",
        );
        assert_eq!(
            shape.as_static(),
            None,
            "a symbolic boundary answers no fixed-shape borrow",
        );
        assert_eq!(
            program
                .extent_sources()
                .expect("a symbolic program retains its environment")
                .environment_identity(),
            environment.identity(),
            "the environment a symbol resolves in is reachable from the program",
        );
    }

    #[test]
    fn a_wholly_literal_program_stays_static_through_every_construction_path() {
        let written = Shape::from_dims([2, 3]);
        for (label, program) in [
            ("input", {
                let mut builder = SemanticProgramBuilder::try_standard().unwrap();
                let value = builder
                    .input::<F32>(input_key("rows"), written.clone())
                    .unwrap();
                builder.output(output_key("out"), value).unwrap();
                builder.build().unwrap()
            }),
            (
                "input_sourced normalization",
                sourced_program(
                    None,
                    vec![
                        SourcedExtent::Static(Extent::new(2)),
                        SourcedExtent::Static(Extent::new(3)),
                    ],
                )
                .expect("an all-literal sourced boundary needs no environment"),
            ),
        ] {
            let value = program.inputs().next().unwrap().value();
            let shape = program.shape(value).unwrap();
            assert!(
                matches!(shape, SourcedShape::Static(_)),
                "{label}: an all-literal boundary normalizes to one spelling",
            );
            assert_eq!(
                shape.as_static(),
                Some(&written),
                "{label}: and answers the borrow of the fixed shape it was written with",
            );
            assert_eq!(
                program
                    .extent_sources()
                    .map(ExtentSources::environment_identity),
                None,
                "{label}: a program resolving nothing acquires no environment",
            );
        }
    }

    #[test]
    fn a_foreign_symbol_is_refused_as_undeclared_beside_its_accepted_neighbour() {
        let environment = env();
        let foreign = ShapeSymbol::new(SymbolScope::new("other/0").unwrap(), "n").unwrap();

        assert!(
            matches!(
                sourced_program(
                    Some(Arc::clone(&environment)),
                    vec![SourcedExtent::Symbol(foreign.clone())],
                ),
                Err(BuildError::ExtentSource(ExtentSourceError::UndeclaredSymbol {
                    symbol,
                })) if symbol == foreign,
            ),
            "a symbol from another environment names a binding this program cannot resolve",
        );
        assert!(
            sourced_program(Some(environment), vec![SourcedExtent::Symbol(sym("n"))]).is_ok(),
            "the neighbour differing only in whose environment declares the symbol is admitted",
        );
    }

    #[test]
    fn a_symbol_with_no_environment_at_all_is_refused_as_undeclared() {
        assert!(
            matches!(
                sourced_program(None, vec![SourcedExtent::Symbol(sym("n"))]),
                Err(BuildError::ExtentSource(
                    ExtentSourceError::UndeclaredSymbol { .. }
                )),
            ),
            "no environment can declare a symbol, so it is undeclared for that reason",
        );
    }

    #[test]
    fn a_post_ceiling_binding_is_refused_as_too_late_beside_its_accepted_neighbour() {
        let late = env_over("rows", 0, AvailabilityPhase::PreparedKernelPreflight, &[]);
        assert!(
            matches!(
                sourced_program(Some(late), vec![SourcedExtent::Symbol(sym("n"))]),
                Err(BuildError::ExtentSource(ExtentSourceError::SourceTooLate {
                    available,
                    ceiling,
                    ..
                })) if available == AvailabilityPhase::PreparedKernelPreflight
                    && ceiling == EXTENT_PHASE_CEILING,
            ),
            "an extent readable only once a pipeline is prepared cannot size an initial output",
        );
        assert!(
            sourced_program(
                Some(env_over("rows", 0, EXTENT_PHASE_CEILING, &[])),
                vec![SourcedExtent::Symbol(sym("n"))],
            )
            .is_ok(),
            "the neighbour differing only in the binding's phase is admitted",
        );
    }

    #[test]
    fn a_symbolic_value_is_refused_as_an_operation_operand() {
        let mut builder =
            SemanticProgramBuilder::try_standard_with_shape_environment(env()).unwrap();
        let symbolic = builder
            .input_sourced::<F32>(input_key("rows"), vec![SourcedExtent::Symbol(sym("n"))])
            .expect("the symbolic input is admitted");
        let scalar = constant(&mut builder, 2.0).unwrap();
        assert!(
            matches!(
                multiply(&mut builder, symbolic, scalar),
                Err(BuildError::SymbolicOperandUnsupported {
                    role: ValueRole::OperationOperand { index: 0 },
                }),
            ),
            "shape inference over symbolic operands is a separate delivery, so this refuses",
        );

        let mut neighbour =
            SemanticProgramBuilder::try_standard_with_shape_environment(env()).unwrap();
        let literal = neighbour
            .input_sourced::<F32>(
                input_key("rows"),
                vec![SourcedExtent::Static(Extent::new(4))],
            )
            .expect("the literal input is admitted");
        let scalar = constant(&mut neighbour, 2.0).unwrap();
        assert!(
            multiply(&mut neighbour, literal, scalar).is_ok(),
            "the neighbour differing only in the operand's source kind applies",
        );
    }

    #[test]
    fn rust_side_evidence_and_shape_witnesses_refuse_a_symbolic_value() {
        let program = sourced_program(Some(env()), vec![SourcedExtent::Symbol(sym("n"))]).unwrap();
        let value: Value<F32> = program
            .reify(program.inputs().next().unwrap().value())
            .expect("the input's type is exact");
        assert!(
            matches!(
                program.refine::<F32, StaticShape<1, { [4] }>>(value),
                Err(ShapeRefineError::SymbolicShape { .. }),
            ),
            "evidence fixed when the consumer compiled cannot match an extent bound later",
        );
        assert!(
            matches!(
                program.prove_same_shape(value, value),
                Err(ShapeWitnessError::SymbolicShape {
                    subject: ShapeWitnessSubject::Left,
                }),
            ),
            "structural equality decides nothing about a boundary naming a symbol",
        );
    }

    // --- Collision probes ---------------------------------------------------

    #[test]
    fn a_symbol_and_the_value_its_environment_pins_are_two_programs() {
        // The environment fixes `n` at exactly four, so a consumer that resolved
        // the symbol would produce the literal program's bytes.
        let pinned = env_over(
            "rows",
            0,
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(ExtentTerm::Symbol(sym("n")), 4, 4).unwrap()],
        );
        let symbolic = sourced_program(
            Some(Arc::clone(&pinned)),
            vec![SourcedExtent::Symbol(sym("n"))],
        )
        .unwrap();
        assert_eq!(
            symbolic
                .extent_sources()
                .unwrap()
                .determined(&SourcedExtent::Symbol(sym("n"))),
            Some(Extent::new(4)),
            "the environment really does pin the symbol, so the probe is not vacuous",
        );

        let literal = sourced_program(None, vec![SourcedExtent::Static(Extent::new(4))]).unwrap();
        assert_ne!(
            symbolic.semantic_identity().graph(),
            literal.semantic_identity().graph(),
            "a boundary sized by a symbol is a different program from one sized by its value",
        );
    }

    #[test]
    fn two_symbols_in_one_environment_are_two_programs() {
        // The probe the value-versus-symbol pair above cannot make. That pair is
        // separated by `SourcedExtent`'s source tag alone, so an encoder that
        // wrote a *resolved value* after the symbol tag would still pass it.
        // Here both programs carry the symbol tag and one environment, so the
        // only thing that can tell them apart is the symbol's own bytes.
        let environment = two_symbol_env();
        let by_n = sourced_program(
            Some(Arc::clone(&environment)),
            vec![SourcedExtent::Symbol(sym("n"))],
        )
        .unwrap();
        let by_m =
            sourced_program(Some(environment), vec![SourcedExtent::Symbol(sym("m"))]).unwrap();

        assert_eq!(
            by_n.semantic_identity().shape_environment(),
            by_m.semantic_identity().shape_environment(),
            "one environment declares both, so the environment subject cannot separate them",
        );
        assert_ne!(
            by_n.semantic_identity().graph(),
            by_m.semantic_identity().graph(),
            "a boundary sized by `n` is a different program from one sized by `m`",
        );
    }

    #[test]
    fn a_symbolic_axis_and_a_literal_one_do_not_collide_across_ranks() {
        // Framing, not content: the rank prefix and the per-extent tag must keep
        // a rank-one symbolic boundary from reading as the head of a rank-two
        // one. Both programs live in one environment for the same reason as
        // above.
        let environment = two_symbol_env();
        let rank_one = sourced_program(
            Some(Arc::clone(&environment)),
            vec![SourcedExtent::Symbol(sym("n"))],
        )
        .unwrap();
        let rank_two = sourced_program(
            Some(environment),
            vec![
                SourcedExtent::Symbol(sym("n")),
                SourcedExtent::Static(Extent::new(1)),
            ],
        )
        .unwrap();
        assert_ne!(
            rank_one.semantic_identity().graph(),
            rank_two.semantic_identity().graph(),
            "an appended literal axis is a different boundary, not a suffix of the same one",
        );
    }

    #[test]
    fn two_environments_over_one_spelling_are_two_programs_and_one_graph() {
        let by_first_axis = env_over("rows", 0, EXTENT_PHASE_CEILING, &[]);
        let by_second_axis = env_over("rows", 1, EXTENT_PHASE_CEILING, &[]);
        let constrained = env_over(
            "rows",
            0,
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(ExtentTerm::Symbol(sym("n")), 1, 8).unwrap()],
        );
        let extents = vec![SourcedExtent::Symbol(sym("n"))];

        let first = sourced_program(Some(Arc::clone(&by_first_axis)), extents.clone()).unwrap();
        let repeated = sourced_program(Some(by_first_axis), extents.clone()).unwrap();
        let other_axis = sourced_program(Some(by_second_axis), extents.clone()).unwrap();
        let narrowed = sourced_program(Some(constrained), extents).unwrap();

        assert_eq!(
            first.semantic_identity(),
            repeated.semantic_identity(),
            "one environment and one structure name one program",
        );
        for (label, other) in [("binding axis", &other_axis), ("constraint", &narrowed)] {
            assert_ne!(
                first.semantic_identity(),
                other.semantic_identity(),
                "{label}: a differently identified environment is a different program",
            );
            assert_eq!(
                first.semantic_identity().graph(),
                other.semantic_identity().graph(),
                "{label}: and the difference lands on the environment subject, not on graph meaning",
            );
            assert_ne!(
                first.semantic_identity().shape_environment(),
                other.semantic_identity().shape_environment(),
                "{label}: which is the subject that separates them",
            );
        }
    }

    #[test]
    fn a_program_that_declares_no_symbol_carries_the_empty_environment_subject() {
        let literal = sourced_program(None, vec![SourcedExtent::Static(Extent::new(4))]).unwrap();
        assert_eq!(
            literal.semantic_identity().shape_environment(),
            crate::shape::empty_environment_identity(),
            "the fifth subject is total: no environment reports the empty one",
        );

        // And the empty environment is not a value some other environment can
        // reach, so the totality does not collide two programs into one.
        let symbolic = sourced_program(Some(env()), vec![SourcedExtent::Symbol(sym("n"))]).unwrap();
        assert_ne!(
            symbolic.semantic_identity().shape_environment(),
            crate::shape::empty_environment_identity(),
        );
    }

    #[test]
    fn no_symbolic_program_reaches_a_verified_kernel_program() {
        // The coupling the artifact's three carried subjects rest on, asserted
        // rather than assumed: `project_semantic` does not travel the
        // shape-environment subject, and that is only sound while no two
        // packaged artifacts can differ by it. Every artifact is built over a
        // kernel program, so this refusal is what makes that true. If this test
        // ever fails, an artifact can ship an unkeyed symbolic program.
        let symbolic = sourced_program(Some(env()), vec![SourcedExtent::Symbol(sym("n"))]).unwrap();
        assert!(
            matches!(
                crate::program::KernelProgramBuilder::new(&symbolic),
                Err(crate::program::KernelProgramBuildError::SymbolicInterfaceExtent { interface })
                    if interface == "rows",
            ),
            "a symbolic interface extent has no fixed boundary a physical plan could cover",
        );

        let literal = sourced_program(None, vec![SourcedExtent::Static(Extent::new(4))]).unwrap();
        assert!(
            crate::program::KernelProgramBuilder::new(&literal).is_ok(),
            "the neighbour differing only in the extent's source kind opens",
        );
    }

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
            request: OperationInferenceRequest<'_>,
            outputs: &mut OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            let operands = request.operands();
            let attributes = request.attributes();
            if operands.len() == 1 && attributes.fields().is_empty() {
                outputs.try_push(operands[0].clone())
            } else {
                Err(OperationInferenceError::new(
                    ProviderDiagnosticCode::new("test.identity.signature").unwrap(),
                    "identity requires one operand and no attributes",
                )
                .unwrap())
            }
        }
    }

    struct PreconditionPrecedenceProvider;

    impl SemanticRegistryProvider for PreconditionPrecedenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "precondition-precedence", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            let declarations = super::super::SemanticPreconditionDeclarations::new([
                super::super::SemanticPreconditionDeclaration::new(
                    super::super::no_nan_predicate(),
                    super::super::OperationOperandIndex::new(0),
                    super::super::SemanticLogicalView::WholeValue,
                    super::super::SemanticInvalidInputCode::new("test", "invalid-nan", 1).unwrap(),
                ),
            ])
            .unwrap();
            registrar.register_operation(
                OperationDefinition::new(
                    OpKey::new("test", "precondition-precedence", 1).unwrap(),
                    OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), [])
                        .unwrap(),
                    NormativeDefinitionRef::new("test precondition precedence")?,
                    OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
                    OperationConformance::new(CanonicalValue::boolean(true)),
                    OperationEffect::Pure,
                    Arc::new(Identity),
                )
                .with_semantic_preconditions(declarations)
                .unwrap(),
            )
        }
    }

    #[test]
    fn typed_result_contract_failure_precedes_static_input_disproof() {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry
            .register_provider(&PreconditionPrecedenceProvider)
            .unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let nan = F32Constant::apply(&mut builder, f32::NAN.to_bits()).unwrap();
        let error = builder
            .apply_typed_single::<U4>(
                OpKey::new("test", "precondition-precedence", 1).unwrap(),
                OperationAttributes::empty(),
                &[nan.erase()],
            )
            .unwrap_err();
        assert!(matches!(error, BuildError::Reify(_)));
    }

    struct Pair;
    impl OperationInferencer for Pair {
        fn infer(
            &self,
            request: OperationInferenceRequest<'_>,
            outputs: &mut OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            let operands = request.operands();
            outputs.try_push(operands[0].clone())?;
            outputs.try_push(operands[0].clone())
        }
    }

    struct DefaultedIdentity;
    impl OperationInferencer for DefaultedIdentity {
        fn infer(
            &self,
            request: OperationInferenceRequest<'_>,
            outputs: &mut OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            let operands = request.operands();
            let attributes = request.attributes();
            let field = AttributeFieldId::new(7);
            if operands.len() == 1
                && attributes.get(field) == Some(&CanonicalValue::unsigned_u32(4))
            {
                outputs.try_push(operands[0].clone())
            } else {
                Err(OperationInferenceError::new(
                    ProviderDiagnosticCode::new("test.defaulted-identity.signature").unwrap(),
                    "defaulted identity requires one operand and the resolved default",
                )
                .unwrap())
            }
        }
    }

    struct OperationProvider(u32);
    impl SemanticRegistryProvider for OperationProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "operations", self.0).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), super::super::RegistryError> {
            registrar.register_operation(test_operation("identity", 1, Arc::new(Identity)))?;
            registrar.register_operation(test_operation("pair", 2, Arc::new(Pair)))?;
            registrar.register_operation(OperationDefinition::new(
                OpKey::new("test", "defaulted-identity", 1).unwrap(),
                OperationSchema::new(
                    OperationArity::exact(1),
                    OperationArity::exact(1),
                    [OperationAttributeSchema::defaulted(
                        AttributeFieldId::new(7),
                        CanonicalValueKind::Unsigned,
                        CanonicalValue::unsigned_u32(4),
                    )
                    .unwrap()],
                )
                .unwrap(),
                NormativeDefinitionRef::new("test defaulted-identity v1").unwrap(),
                OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
                OperationConformance::new(
                    CanonicalValue::utf8("test.defaulted-identity.v1").unwrap(),
                ),
                OperationEffect::Pure,
                Arc::new(DefaultedIdentity),
            ))
        }
    }

    struct CompositeTypeProvider;

    impl SemanticRegistryProvider for CompositeTypeProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "composite-types", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), super::super::RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Parameterized(
                    TypeKey::new("test", "container", 1).unwrap(),
                ),
                NormativeDefinitionRef::new("test container v1")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))?;
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::EncodedNumeric(
                    QuantSchemeKey::new("test", "encoded", 1).unwrap(),
                ),
                NormativeDefinitionRef::new("test encoded v1")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))
        }
    }

    struct NominalTypeProvider {
        name: &'static str,
        revision: u32,
    }

    impl SemanticRegistryProvider for NominalTypeProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", self.name, self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), super::super::RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("test", self.name, 1).unwrap()),
                NormativeDefinitionRef::from_owned(format!("test {} v1", self.name))?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))
        }
    }

    struct AttributedIdentity;

    impl OperationInferencer for AttributedIdentity {
        fn infer(
            &self,
            request: OperationInferenceRequest<'_>,
            outputs: &mut OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            outputs.try_push(request.operands()[0].clone())
        }
    }

    struct AttributedOperationProvider;

    impl SemanticRegistryProvider for AttributedOperationProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "attributed-operation", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), super::super::RegistryError> {
            registrar.register_operation(OperationDefinition::new(
                OpKey::new("test", "attributed-identity", 1).unwrap(),
                OperationSchema::new(
                    OperationArity::exact(1),
                    OperationArity::exact(1),
                    [
                        OperationAttributeSchema::required(
                            AttributeFieldId::new(1),
                            CanonicalValueKind::Type,
                        ),
                        OperationAttributeSchema::required(
                            AttributeFieldId::new(2),
                            CanonicalValueKind::FloatBits,
                        ),
                    ],
                )
                .unwrap(),
                NormativeDefinitionRef::new("test attributed identity v1")?,
                OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
                OperationConformance::new(CanonicalValue::boolean(true)),
                OperationEffect::Pure,
                Arc::new(AttributedIdentity),
            ))
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
            NormativeDefinitionRef::from_owned(format!("test {name} v1")).unwrap(),
            OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            OperationConformance::new(
                CanonicalValue::utf8_owned(format!("test.{name}.v1")).unwrap(),
            ),
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
    fn borrowed_validation_reports_reachable_authority_exhaustion_with_typed_source() {
        let mut registry = SemanticRegistryBuilder::new();
        registry.register_provider(&CompositeTypeProvider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        for index in 0..super::super::registry::TEST_MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS {
            let resolved_type = ResolvedValueType::parameterized(
                TypeKey::new("test", "container", 1).unwrap(),
                TypeArguments::new([CanonicalValue::unsigned_u64(
                    u64::try_from(index).expect("the governed authority limit fits u64"),
                )])
                .unwrap(),
            )
            .unwrap();
            let value = builder
                .input_resolved(
                    input_key(&format!("input-{index}")),
                    Shape::new([]),
                    resolved_type,
                )
                .unwrap();
            builder
                .output_resolved(output_key(&format!("output-{index}")), value)
                .unwrap();
        }

        let diagnostics = builder.validate().unwrap_err();
        assert!(matches!(
            diagnostics.as_slice(),
            [ValidationDiagnostic::SemanticAuthority(
                super::super::RegistryError::SemanticAuthorityResourceExceeded {
                    resource: super::super::SemanticAuthorityResource::ClosureItems,
                    limit: super::super::registry::TEST_MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS,
                    actual,
                }
            )] if *actual == super::super::registry::TEST_MAX_SEMANTIC_AUTHORITY_CLOSURE_ITEMS + 1
        ));
        let diagnostic_source = std::error::Error::source(&diagnostics).unwrap();
        assert!(std::error::Error::source(diagnostic_source).is_some());

        let error = builder.build().unwrap_err();
        assert_eq!(error.diagnostics(), Some(&diagnostics));
        assert!(matches!(
            error.failure(),
            ProgramBuildFailure::Validation(_)
        ));
    }

    #[test]
    fn commitment_rejects_corrupted_internal_graph_state() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = constant(&mut builder, 1.0).unwrap();
        builder.output(output_key("result"), value).unwrap();
        builder.operations[0].results.clear();

        let diagnostics = builder.validate().unwrap_err();
        assert_eq!(
            diagnostics.as_slice(),
            &[ValidationDiagnostic::InvalidInternalGraph {
                reason: "an operation has no results",
            }]
        );
        let error = builder.build().unwrap_err();
        assert_eq!(error.diagnostics(), Some(&diagnostics));
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

        let error = builder.build_with_owner_allocator(|| None).unwrap_err();
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
    fn failed_validation_does_not_advance_completed_owner_allocator() {
        let builder = SemanticProgramBuilder::try_standard().unwrap();
        let calls = std::cell::Cell::new(0_usize);
        let error = builder
            .build_with_owner_allocator(|| {
                calls.set(calls.get() + 1);
                None
            })
            .unwrap_err();
        assert_eq!(calls.get(), 0);
        assert!(matches!(
            error.failure(),
            ProgramBuildFailure::Validation(_)
        ));
    }

    #[test]
    fn oversized_operand_lists_fail_before_handle_conversion_or_mutation() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = constant(&mut builder, 1.0).unwrap().erase();
        let operations = builder.operations.len();
        let values = builder.values.len();
        let operands = vec![value; MAX_OPERATION_OPERANDS as usize + 1];
        assert_eq!(
            builder.apply(add_f32_op(), OperationAttributes::empty(), &operands),
            Err(BuildError::TooManyOperationOperands {
                actual: MAX_OPERATION_OPERANDS as usize + 1,
                limit: MAX_OPERATION_OPERANDS,
            })
        );
        assert_eq!(builder.operations.len(), operations);
        assert_eq!(builder.values.len(), values);
    }

    #[test]
    fn aggregate_canonical_work_is_transactional_and_bounds_final_identity() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let large_shape = Shape::try_new(vec![crate::shape::Extent::new(1); 4_096]).unwrap();
        let first = builder
            .input::<F32>(input_key("reachable"), large_shape.clone())
            .unwrap();
        builder.output(output_key("result"), first).unwrap();

        let rejected = loop {
            let before = (
                builder.inputs.len(),
                builder.values.len(),
                builder.operations.len(),
                builder.outputs.len(),
                builder.canonical_work_bytes,
            );
            let index = builder.inputs.len();
            match builder.input::<F32>(
                InputKey::from_owned(format!("dead-{index}")).unwrap(),
                large_shape.clone(),
            ) {
                Ok(_) => {}
                Err(error) => {
                    assert_eq!(
                        before,
                        (
                            builder.inputs.len(),
                            builder.values.len(),
                            builder.operations.len(),
                            builder.outputs.len(),
                            builder.canonical_work_bytes,
                        )
                    );
                    break error;
                }
            }
        };
        assert!(matches!(
            rejected,
            BuildError::CanonicalWorkExceeded { actual, limit }
                if actual > limit && limit == MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES
        ));
        builder.validate().unwrap();

        let program = builder.build().unwrap();
        let identity = program.semantic_identity().graph().as_bytes();
        assert_eq!(identity.len(), program.data.graph_identity_encoded_len);
        assert!(identity.len() <= program.data.canonical_work_bytes);
        assert!(identity.len() <= MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES);
        assert_eq!(program.input_count(), 1);
    }

    #[test]
    fn operation_and_output_budget_failures_precede_arena_mutation() {
        let mut operation_builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = operation_builder
            .input::<F32>(input_key("input"), Shape::new([]))
            .unwrap();
        operation_builder.canonical_work_bytes = MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES;
        let before = (
            operation_builder.operations.len(),
            operation_builder.values.len(),
            operation_builder.canonical_work_bytes,
        );
        assert!(matches!(
            multiply(&mut operation_builder, input, input),
            Err(BuildError::CanonicalWorkExceeded { .. })
        ));
        assert_eq!(
            before,
            (
                operation_builder.operations.len(),
                operation_builder.values.len(),
                operation_builder.canonical_work_bytes,
            )
        );

        let mut output_builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = output_builder
            .input::<F32>(input_key("input"), Shape::new([]))
            .unwrap();
        output_builder.canonical_work_bytes = MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES;
        let before = (
            output_builder.outputs.len(),
            output_builder.output_keys.len(),
            output_builder.canonical_work_bytes,
        );
        assert!(matches!(
            output_builder.output(output_key("result"), input),
            Err(BuildError::CanonicalWorkExceeded { .. })
        ));
        assert_eq!(
            before,
            (
                output_builder.outputs.len(),
                output_builder.output_keys.len(),
                output_builder.canonical_work_bytes,
            )
        );
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
            program.shape(completed.erase()).unwrap().as_static(),
            Some(&Shape::from_dims([2]))
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
            first.semantic_identity(),
            second.semantic_identity()
        ));
        assert_eq!(
            first.data.graph_identity_encoded_len,
            first.semantic_identity().graph().as_bytes().len()
        );
    }

    #[test]
    fn semantic_program_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SemanticProgram>();
    }

    #[test]
    fn identity_ignores_dead_insertion_order_but_preserves_sharing() {
        assert_eq!(
            program(true, true).semantic_identity().graph(),
            program(false, true).semantic_identity().graph()
        );
        assert_ne!(
            program(false, true).semantic_identity().graph(),
            program(false, false).semantic_identity().graph()
        );
    }

    #[test]
    fn identity_ignores_live_topological_insertion_order() {
        /// Builds one live DAG in two valid topological insertion orders. Only
        /// the order of the two independent multiplications changes; the
        /// ordered input interface and every operand edge stay identical.
        fn build(swap_independent_operations: bool) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let x = builder
                .input::<F32>(input_key("x"), Shape::from_dims([2, 3]))
                .unwrap();
            let y = builder
                .input::<F32>(input_key("y"), Shape::from_dims([2, 3]))
                .unwrap();
            let scale = constant(&mut builder, 3.0).unwrap();
            let (left, right) = if swap_independent_operations {
                let right = multiply(&mut builder, y, scale).unwrap();
                (multiply(&mut builder, x, scale).unwrap(), right)
            } else {
                let left = multiply(&mut builder, x, scale).unwrap();
                (left, multiply(&mut builder, y, scale).unwrap())
            };
            let result = add(&mut builder, left, right).unwrap();
            builder.output(output_key("result"), result).unwrap();
            builder.build().unwrap()
        }

        fn arena_edges(program: &SemanticProgram) -> Vec<Vec<usize>> {
            program
                .operations()
                .map(|operation| {
                    operation
                        .operands()
                        .map(|operand| operand.index.as_usize())
                        .collect()
                })
                .collect()
        }

        let ordered = build(false);
        let swapped = build(true);

        // Without genuinely different arenas the identity comparison below
        // would hold vacuously.
        assert_ne!(arena_edges(&ordered), arena_edges(&swapped));
        assert_eq!(
            ordered.semantic_identity().graph(),
            swapped.semantic_identity().graph()
        );
    }

    #[test]
    fn graph_meaning_excludes_provider_revision_but_provenance_retains_it() {
        fn build(revision: u32) -> SemanticProgram {
            let mut registry = SemanticRegistryBuilder::standard().unwrap();
            registry
                .register_provider(&OperationProvider(revision))
                .unwrap();
            let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
            let input = builder
                .input::<F32>(input_key("x"), Shape::from_dims([2]))
                .unwrap();
            let result = builder
                .apply_typed_single::<F32>(
                    OpKey::new("test", "identity", 1).unwrap(),
                    OperationAttributes::empty(),
                    &[input.erase()],
                )
                .unwrap();
            builder.output(output_key("result"), result).unwrap();
            builder.build().unwrap()
        }

        let first = build(1);
        let second = build(2);

        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );
        assert_eq!(
            first.semantic_identity().reached_definitions(),
            second.semantic_identity().reached_definitions()
        );
        assert_ne!(
            first.semantic_identity().admission_provenance(),
            second.semantic_identity().admission_provenance()
        );
        assert_ne!(
            first.semantic_identity().registry_snapshot(),
            second.semantic_identity().registry_snapshot()
        );
    }

    #[test]
    fn program_authority_projection_follows_nested_and_encoded_value_types() {
        fn build(leaf_revision: u32) -> SemanticProgram {
            let leaf = ResolvedValueType::nominal(TypeKey::new("test", "leaf", 1).unwrap());
            let encoded = ResolvedValueType::encoded_numeric(
                QuantSchemeKey::new("test", "encoded", 1).unwrap(),
                EncodedNumericContract::new([CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::value_type(leaf),
                )])
                .unwrap(),
            )
            .unwrap();
            let container = ResolvedValueType::parameterized(
                TypeKey::new("test", "container", 1).unwrap(),
                TypeArguments::new([CanonicalValue::value_type(encoded)]).unwrap(),
            )
            .unwrap();
            let mut registry = SemanticRegistryBuilder::new();
            registry.register_provider(&CompositeTypeProvider).unwrap();
            registry
                .register_provider(&NominalTypeProvider {
                    name: "leaf",
                    revision: leaf_revision,
                })
                .unwrap();
            let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
            let input = builder
                .input_resolved(input_key("x"), Shape::from_dims([2]), container)
                .unwrap();
            builder
                .output_resolved(output_key("result"), input)
                .unwrap();
            builder.build().unwrap()
        }

        let first = build(1);
        let second = build(2);
        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );
        assert_eq!(
            first.semantic_identity().reached_definitions(),
            second.semantic_identity().reached_definitions()
        );
        assert_ne!(
            first.semantic_identity().admission_provenance(),
            second.semantic_identity().admission_provenance()
        );
    }

    #[test]
    fn program_authority_projection_follows_type_and_float_bits_occurrence_attributes() {
        fn build(type_revision: u32, float_revision: u32) -> SemanticProgram {
            let mut registry = SemanticRegistryBuilder::standard().unwrap();
            registry
                .register_provider(&NominalTypeProvider {
                    name: "type-attribute",
                    revision: type_revision,
                })
                .unwrap();
            registry
                .register_provider(&NominalTypeProvider {
                    name: "float-format",
                    revision: float_revision,
                })
                .unwrap();
            registry
                .register_provider(&AttributedOperationProvider)
                .unwrap();
            let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
            let input = builder
                .input::<F32>(input_key("x"), Shape::from_dims([2]))
                .unwrap();
            let attributes = OperationAttributes::new([
                CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::value_type(ResolvedValueType::nominal(
                        TypeKey::new("test", "type-attribute", 1).unwrap(),
                    )),
                ),
                CanonicalField::new(
                    AttributeFieldId::new(2),
                    CanonicalValue::float_bits(
                        TypeKey::new("test", "float-format", 1).unwrap(),
                        [0_u8],
                    )
                    .unwrap(),
                ),
            ])
            .unwrap();
            let result = builder
                .apply_typed_single::<F32>(
                    OpKey::new("test", "attributed-identity", 1).unwrap(),
                    attributes,
                    &[input.erase()],
                )
                .unwrap();
            builder.output(output_key("result"), result).unwrap();
            builder.build().unwrap()
        }

        let baseline = build(1, 1);
        let type_changed = build(2, 1);
        let float_changed = build(1, 2);
        for changed in [&type_changed, &float_changed] {
            assert_eq!(
                baseline.semantic_identity().graph(),
                changed.semantic_identity().graph()
            );
            assert_eq!(
                baseline.semantic_identity().reached_definitions(),
                changed.semantic_identity().reached_definitions()
            );
            assert_ne!(
                baseline.semantic_identity().admission_provenance(),
                changed.semantic_identity().admission_provenance()
            );
        }
    }

    #[test]
    fn unused_provider_revision_changes_only_registry_snapshot_provenance() {
        fn build(revision: u32) -> SemanticProgram {
            let mut registry = SemanticRegistryBuilder::standard().unwrap();
            registry
                .register_provider(&OperationProvider(revision))
                .unwrap();
            let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
            let input = builder
                .input::<F32>(input_key("x"), Shape::from_dims([2]))
                .unwrap();
            builder.output(output_key("result"), input).unwrap();
            builder.build().unwrap()
        }

        let first = build(1);
        let second = build(2);
        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );
        assert_eq!(
            first.semantic_identity().reached_definitions(),
            second.semantic_identity().reached_definitions()
        );
        assert_eq!(
            first.semantic_identity().admission_provenance(),
            second.semantic_identity().admission_provenance()
        );
        assert_ne!(
            first.semantic_identity().registry_snapshot(),
            second.semantic_identity().registry_snapshot()
        );
    }

    #[test]
    fn identity_preserves_exact_float_bits_and_output_order() {
        fn identity(bits: u32, reverse: bool) -> SemanticGraphIdentity {
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
            builder.build().unwrap().semantic_identity().graph().clone()
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
    fn semantic_graph_identity_handles_a_deep_chain_iteratively() {
        const DEPTH: usize = 50_000;

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let mut value = constant(&mut builder, 1.0).unwrap();
        let increment = constant(&mut builder, 0.0).unwrap();
        for _ in 0..DEPTH {
            value = add(&mut builder, value, increment).unwrap();
        }
        builder.output(output_key("result"), value).unwrap();
        let program = builder.build().unwrap();

        assert!(!program.semantic_identity().graph().as_bytes().is_empty());
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

        assert_eq!(
            program.shape(completed.erase()).unwrap().as_static(),
            Some(&shape)
        );
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
                )) if rejection.source_error().code().as_str() == code
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
        // The operand has rank two, so axis two is the first position at or
        // beyond its rank.
        assert!(has_code(
            sum(&mut builder, x, [Axis::new(2)]),
            "sum.axes.range"
        ));
        assert!(has_code(
            sum(&mut builder, x, [Axis::new(0), Axis::new(u32::MAX)]),
            "sum.axes.range"
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
        registry.register_provider(&OperationProvider(1)).unwrap();
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

        assert_ne!(
            shared.semantic_identity().graph(),
            separate.semantic_identity().graph()
        );
    }

    #[test]
    fn semantic_program_identity_normalizes_explicit_schema_defaults() {
        fn build(
            registry: FrozenSemanticRegistry,
            attributes: OperationAttributes,
        ) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_new(registry).unwrap();
            let input = builder
                .input::<F32>(input_key("x"), Shape::from_dims([2]))
                .unwrap();
            let result = builder
                .apply(
                    OpKey::new("test", "defaulted-identity", 1).unwrap(),
                    attributes,
                    &[input.erase()],
                )
                .unwrap()[0];
            builder
                .output_resolved(output_key("result"), result)
                .unwrap();
            builder.build().unwrap()
        }

        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry.register_provider(&OperationProvider(1)).unwrap();
        let registry = registry.freeze().unwrap();
        let omitted = build(registry.clone(), OperationAttributes::empty());
        let explicit = build(
            registry,
            OperationAttributes::new([CanonicalField::new(
                AttributeFieldId::new(7),
                CanonicalValue::unsigned_u32(4),
            )])
            .unwrap(),
        );

        assert_eq!(
            omitted.semantic_identity().graph(),
            explicit.semantic_identity().graph()
        );
        assert!(
            explicit
                .operations()
                .next()
                .unwrap()
                .attributes()
                .fields()
                .is_empty()
        );
    }

    #[test]
    fn typed_result_checks_are_transactional() {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry.register_provider(&OperationProvider(1)).unwrap();
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

    #[test]
    fn panicking_inferencer_cannot_commit_graph_state() {
        struct PanicInferencer;
        impl OperationInferencer for PanicInferencer {
            fn infer(
                &self,
                _request: OperationInferenceRequest<'_>,
                _outputs: &mut OperationInferenceOutputs<'_>,
            ) -> Result<(), OperationInferenceError> {
                panic!("provider panic")
            }
        }

        struct PanicProvider;
        impl SemanticRegistryProvider for PanicProvider {
            fn identity(&self) -> ProviderIdentity {
                ProviderIdentity::new("test", "panic", 1).unwrap()
            }

            fn register(
                &self,
                registrar: &mut SemanticRegistryRegistrar<'_>,
            ) -> Result<(), super::super::RegistryError> {
                registrar.register_operation(test_operation("panic", 1, Arc::new(PanicInferencer)))
            }
        }

        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry.register_provider(&PanicProvider).unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let input = builder
            .input::<F32>(input_key("x"), Shape::from_dims([2]))
            .unwrap();
        let operation_count = builder.operations.len();
        let value_count = builder.values.len();

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = builder.apply(
                OpKey::new("test", "panic", 1).unwrap(),
                OperationAttributes::empty(),
                &[input.erase()],
            );
        }));
        assert!(panic.is_err());
        assert_eq!(builder.operations.len(), operation_count);
        assert_eq!(builder.values.len(), value_count);

        let result = add(&mut builder, input, input).unwrap();
        assert_eq!(result.erase().index.as_usize(), value_count);
        assert_eq!(builder.operations.len(), operation_count + 1);
        assert_eq!(builder.values.len(), value_count + 1);
    }
}
