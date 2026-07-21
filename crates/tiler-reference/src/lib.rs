//! Host reference values and evaluation for verified Tiler semantic programs.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

use tiler_ir::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalValueView, Definition, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey, OperationAttributes, OperationId,
    ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, ResolvedValueType, SemanticProgram, ValueId,
    add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

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

/// Exact resolved operand/result signature of one reference capability.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReferenceSignature {
    operands: Vec<ResolvedValueType>,
    results: Vec<ResolvedValueType>,
}

impl ReferenceSignature {
    /// Creates an exact ordered resolved signature.
    #[must_use]
    pub fn new(
        operands: impl IntoIterator<Item = ResolvedValueType>,
        results: impl IntoIterator<Item = ResolvedValueType>,
    ) -> Self {
        Self {
            operands: operands.into_iter().collect(),
            results: results.into_iter().collect(),
        }
    }

    /// Returns ordered operand types.
    #[must_use]
    pub fn operands(&self) -> &[ResolvedValueType] {
        &self.operands
    }

    /// Returns ordered result types.
    #[must_use]
    pub fn results(&self) -> &[ResolvedValueType] {
        &self.results
    }
}

/// Output-affecting revision of one reference implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReferenceCapabilityRevision(u32);

impl ReferenceCapabilityRevision {
    /// Creates a nonzero capability revision.
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceRegistryError::ZeroCapabilityRevision`] for zero.
    pub const fn new(value: u32) -> Result<Self, ReferenceRegistryError> {
        if value == 0 {
            return Err(ReferenceRegistryError::ZeroCapabilityRevision);
        }
        Ok(Self(value))
    }

    /// Returns the nonzero revision.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// One executable reference implementation for an exact semantic signature.
pub trait ReferenceOperation: Send + Sync + 'static {
    /// Evaluates ordered operands and canonical attributes without fusion.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when inputs violate this capability's contract.
    fn evaluate(
        &self,
        operands: &[&Tensor],
        attributes: &OperationAttributes,
    ) -> Result<Vec<Tensor>, ReferenceOperationError>;
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ReferenceCapabilityKey {
    operation: OpKey,
    signature: ReferenceSignature,
}

#[derive(Clone)]
struct RegisteredReferenceCapability {
    provider: ProviderIdentity,
    revision: ReferenceCapabilityRevision,
    implementation: Arc<dyn ReferenceOperation>,
}

/// Statically linked source of exact reference capabilities.
pub trait ReferenceRegistryProvider: Send + Sync + 'static {
    /// Returns stable provider identity and output-affecting revision.
    fn identity(&self) -> ProviderIdentity;

    /// Stages reference capabilities transactionally.
    ///
    /// # Errors
    ///
    /// Returns a typed error without mutating the destination registry.
    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError>;
}

type StagedReferenceCapability = (ReferenceCapabilityRevision, Arc<dyn ReferenceOperation>);

#[derive(Default)]
struct ReferenceRegistrationBatch {
    capabilities: BTreeMap<ReferenceCapabilityKey, StagedReferenceCapability>,
}

/// Host-owned registration surface for one reference provider transaction.
pub struct ReferenceRegistryRegistrar<'a> {
    batch: &'a mut ReferenceRegistrationBatch,
}

impl ReferenceRegistryRegistrar<'_> {
    /// Registers one exact operation/signature capability.
    ///
    /// # Errors
    ///
    /// Returns a typed collision error within the provider batch.
    pub fn register(
        &mut self,
        operation: OpKey,
        signature: ReferenceSignature,
        revision: ReferenceCapabilityRevision,
        implementation: Arc<dyn ReferenceOperation>,
    ) -> Result<(), ReferenceRegistryError> {
        let key = ReferenceCapabilityKey {
            operation,
            signature,
        };
        if self
            .batch
            .capabilities
            .insert(key.clone(), (revision, implementation))
            .is_some()
        {
            return Err(ReferenceRegistryError::DuplicateCapability {
                operation: key.operation,
                signature: key.signature,
            });
        }
        Ok(())
    }
}

/// Mutable single-use constructor for a frozen reference registry.
#[derive(Default)]
pub struct ReferenceRegistryBuilder {
    capabilities: BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
}

impl ReferenceRegistryBuilder {
    /// Creates an empty reference registry builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the governed initial F32 reference profile.
    ///
    /// # Errors
    ///
    /// Returns a typed error if governed registration violates the public contract.
    pub fn standard() -> Result<Self, ReferenceRegistryError> {
        let mut builder = Self::new();
        builder.register_provider(&StandardReferenceProvider)?;
        Ok(builder)
    }

    /// Applies one provider through an isolated transaction.
    ///
    /// # Errors
    ///
    /// Returns a typed error without changing this builder on failure.
    pub fn register_provider(
        &mut self,
        provider: &(dyn ReferenceRegistryProvider + 'static),
    ) -> Result<(), ReferenceRegistryError> {
        let identity = provider.identity();
        let mut batch = ReferenceRegistrationBatch::default();
        provider.register(&mut ReferenceRegistryRegistrar { batch: &mut batch })?;
        if batch.capabilities.is_empty() {
            return Err(ReferenceRegistryError::ProviderRegisteredNothing { provider: identity });
        }
        for key in batch.capabilities.keys() {
            if self.capabilities.contains_key(key) {
                return Err(ReferenceRegistryError::DuplicateCapability {
                    operation: key.operation.clone(),
                    signature: key.signature.clone(),
                });
            }
        }
        self.capabilities.extend(batch.capabilities.into_iter().map(
            |(key, (revision, implementation))| {
                (
                    key,
                    RegisteredReferenceCapability {
                        provider: identity.clone(),
                        revision,
                        implementation,
                    },
                )
            },
        ));
        Ok(())
    }

    /// Freezes canonical immutable reference capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceRegistryError::EmptyRegistry`] when empty.
    pub fn freeze(self) -> Result<FrozenReferenceRegistry, ReferenceRegistryError> {
        if self.capabilities.is_empty() {
            return Err(ReferenceRegistryError::EmptyRegistry);
        }
        let registry = FrozenReferenceRegistry(Arc::new(FrozenReferenceRegistryData {
            capabilities: self.capabilities,
            identity: OnceLock::new(),
        }));
        let _ = registry.canonical_identity();
        Ok(registry)
    }
}

struct FrozenReferenceRegistryData {
    capabilities: BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
    identity: OnceLock<CanonicalReferenceRegistryIdentity>,
}

/// Immutable exact reference-capability registry.
#[derive(Clone)]
pub struct FrozenReferenceRegistry(Arc<FrozenReferenceRegistryData>);

impl fmt::Debug for FrozenReferenceRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenReferenceRegistry")
            .field("capability_count", &self.0.capabilities.len())
            .finish()
    }
}

impl FrozenReferenceRegistry {
    /// Builds the governed initial F32 reference profile.
    ///
    /// # Errors
    ///
    /// Returns a typed registry construction error.
    pub fn standard() -> Result<Self, ReferenceRegistryError> {
        static STANDARD: OnceLock<Result<FrozenReferenceRegistry, ReferenceRegistryError>> =
            OnceLock::new();
        STANDARD
            .get_or_init(|| ReferenceRegistryBuilder::standard()?.freeze())
            .clone()
    }

    /// Returns deterministic complete reference-registry provenance.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalReferenceRegistryIdentity {
        self.0
            .identity
            .get_or_init(|| compute_reference_identity(&self.0.capabilities))
    }

    fn resolve(
        &self,
        operation: &OpKey,
        signature: &ReferenceSignature,
    ) -> Result<&RegisteredReferenceCapability, EvaluationError> {
        self.0
            .capabilities
            .get(&ReferenceCapabilityKey {
                operation: operation.clone(),
                signature: signature.clone(),
            })
            .ok_or_else(|| EvaluationError::MissingCapability {
                operation: operation.clone(),
                signature: signature.clone(),
            })
    }
}

/// Collision-free canonical provenance for a frozen reference registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalReferenceRegistryIdentity(Vec<u8>);

impl CanonicalReferenceRegistryIdentity {
    /// Returns canonical provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Failure to construct or extend a reference registry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceRegistryError {
    /// Capability revision zero is reserved.
    ZeroCapabilityRevision,
    /// No reference capability was registered.
    EmptyRegistry,
    /// A provider transaction contributed nothing.
    ProviderRegisteredNothing {
        /// Provider which registered nothing.
        provider: ProviderIdentity,
    },
    /// Two registrations claimed one exact operation/signature pair.
    DuplicateCapability {
        /// Colliding semantic operation.
        operation: OpKey,
        /// Colliding resolved signature.
        signature: ReferenceSignature,
    },
}

impl fmt::Display for ReferenceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapabilityRevision => {
                formatter.write_str("reference capability revision zero is reserved")
            }
            Self::EmptyRegistry => formatter.write_str("reference capability registry is empty"),
            Self::ProviderRegisteredNothing { provider } => {
                write!(
                    formatter,
                    "reference provider {provider} registered nothing"
                )
            }
            Self::DuplicateCapability { operation, .. } => {
                write!(formatter, "duplicate reference capability for {operation}")
            }
        }
    }
}

impl Error for ReferenceRegistryError {}

/// Failure inside one exact reference implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceOperationError {
    /// Operands or attributes violated the registered capability contract.
    InvalidApplication,
    /// Shape arithmetic exceeded host limits.
    ShapeTooLarge,
}

impl fmt::Display for ReferenceOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidApplication => {
                formatter.write_str("invalid reference operation application")
            }
            Self::ShapeTooLarge => {
                formatter.write_str("reference operation shape exceeds host limits")
            }
        }
    }
}

impl Error for ReferenceOperationError {}

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
            let signature = ReferenceSignature::new(
                operands
                    .iter()
                    .map(|value| resolved_type(program, *value))
                    .collect::<Result<Vec<_>, _>>()?,
                results
                    .iter()
                    .map(|value| resolved_type(program, *value))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            let capability = self.registry.resolve(operation.key(), &signature)?;
            let operand_values = operands
                .iter()
                .map(|value| get_value(&values, *value))
                .collect::<Result<Vec<_>, _>>()?;
            let evaluated = capability
                .implementation
                .evaluate(&operand_values, operation.attributes())
                .map_err(|source| EvaluationError::Operation {
                    operation: operation.key().clone(),
                    source,
                })?;
            if evaluated.len() != results.len() {
                return Err(EvaluationError::MalformedProgram);
            }
            for (result, evaluated) in results.into_iter().zip(evaluated) {
                if program
                    .shape(result)
                    .map_err(|_| EvaluationError::MalformedProgram)?
                    != evaluated.shape()
                {
                    return Err(EvaluationError::MalformedProgram);
                }
                values.insert(result, evaluated);
            }
        }

        program
            .outputs()
            .map(|output| get_value(&values, output.value()).cloned())
            .collect()
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

fn reduction_axes(
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
            let CanonicalValueView::Unsigned(axis) = value.view() else {
                return Err(ReferenceOperationError::InvalidApplication);
            };
            u32::try_from(axis)
                .map(Axis::new)
                .map_err(|_| ReferenceOperationError::InvalidApplication)
        })
        .collect()
}

fn binary(
    left_value: &Tensor,
    right_value: &Tensor,
    operation: impl Fn(f32, f32) -> f32,
) -> Result<Tensor, ReferenceOperationError> {
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
    Ok(Tensor {
        shape: result_shape.clone(),
        elements,
    })
}

fn strict_sum(input: &Tensor, axes: &[Axis]) -> Result<Tensor, ReferenceOperationError> {
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
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
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
    Ok(Tensor {
        shape: output_shape,
        elements,
    })
}

fn coordinates(shape: &Shape) -> Result<Vec<Vec<usize>>, ReferenceOperationError> {
    let count = shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    let strides = row_major_strides(shape)?;
    let mut result = Vec::with_capacity(count);
    for linear in 0..count {
        let mut remainder = linear;
        let mut coordinate = Vec::with_capacity(shape.rank());
        for (axis, stride) in strides.iter().enumerate() {
            let extent = usize::try_from(shape.extents()[axis].get())
                .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
            let value = if extent == 0 { 0 } else { remainder / stride };
            remainder = if extent == 0 { 0 } else { remainder % stride };
            coordinate.push(value);
        }
        result.push(coordinate);
    }
    Ok(result)
}

fn row_major_strides(shape: &Shape) -> Result<Vec<usize>, ReferenceOperationError> {
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

struct StandardReferenceProvider;

impl ReferenceRegistryProvider for StandardReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler", "standard-reference", 1)
            .expect("the governed reference provider identity is valid")
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        let revision = ReferenceCapabilityRevision::new(1)?;
        registrar.register(
            constant_f32_op(),
            ReferenceSignature::new([], [F32::resolved_type()]),
            revision,
            Arc::new(F32ConstantReference),
        )?;
        let binary_signature = ReferenceSignature::new(
            [F32::resolved_type(), F32::resolved_type()],
            [F32::resolved_type()],
        );
        registrar.register(
            multiply_f32_op(),
            binary_signature.clone(),
            revision,
            Arc::new(F32BinaryReference::Multiply),
        )?;
        registrar.register(
            add_f32_op(),
            binary_signature,
            revision,
            Arc::new(F32BinaryReference::Add),
        )?;
        registrar.register(
            strict_serial_sum_f32_op(),
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()]),
            revision,
            Arc::new(StrictSerialF32SumReference),
        )
    }
}

struct F32ConstantReference;

impl ReferenceOperation for F32ConstantReference {
    fn evaluate(
        &self,
        operands: &[&Tensor],
        attributes: &OperationAttributes,
    ) -> Result<Vec<Tensor>, ReferenceOperationError> {
        if !operands.is_empty() || attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(CanonicalValueView::Unsigned(bits)) = attributes
            .get(F32_CONSTANT_BITS_ATTRIBUTE)
            .map(tiler_ir::semantic::CanonicalValue::view)
        else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let bits = u32::try_from(bits).map_err(|_| ReferenceOperationError::InvalidApplication)?;
        Ok(vec![Tensor::scalar(f32::from_bits(bits))])
    }
}

enum F32BinaryReference {
    Multiply,
    Add,
}

impl ReferenceOperation for F32BinaryReference {
    fn evaluate(
        &self,
        operands: &[&Tensor],
        attributes: &OperationAttributes,
    ) -> Result<Vec<Tensor>, ReferenceOperationError> {
        let [left, right] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !attributes.fields().is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let result = match self {
            Self::Multiply => binary(left, right, |left, right| left * right)?,
            Self::Add => binary(left, right, |left, right| left + right)?,
        };
        Ok(vec![result])
    }
}

struct StrictSerialF32SumReference;

impl ReferenceOperation for StrictSerialF32SumReference {
    fn evaluate(
        &self,
        operands: &[&Tensor],
        attributes: &OperationAttributes,
    ) -> Result<Vec<Tensor>, ReferenceOperationError> {
        let [input] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let axes = reduction_axes(attributes)?;
        strict_sum(input, &axes).map(|result| vec![result])
    }
}

fn compute_reference_identity(
    capabilities: &BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
) -> CanonicalReferenceRegistryIdentity {
    let mut bytes = b"tiler.reference-registry.v1\0".to_vec();
    encode_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        encode_op_key(&mut bytes, &key.operation);
        encode_signature(&mut bytes, &key.signature);
        encode_bytes(&mut bytes, capability.provider.namespace().as_bytes());
        encode_bytes(&mut bytes, capability.provider.name().as_bytes());
        bytes.extend_from_slice(&capability.provider.revision().to_be_bytes());
        bytes.extend_from_slice(&capability.revision.get().to_be_bytes());
    }
    CanonicalReferenceRegistryIdentity(bytes)
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    encode_bytes(output, key.namespace().as_bytes());
    encode_bytes(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_signature(output: &mut Vec<u8>, signature: &ReferenceSignature) {
    for values in [signature.operands(), signature.results()] {
        encode_len(output, values.len());
        for value in values {
            let canonical = value.canonical_encoding();
            encode_bytes(output, canonical.as_bytes());
        }
    }
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
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
    /// The frozen registry has no executable oracle for an exact semantic signature.
    MissingCapability {
        /// Semantically valid operation lacking an oracle.
        operation: OpKey,
        /// Exact operand/result signature lacking an oracle.
        signature: ReferenceSignature,
    },
    /// A resolved reference capability rejected execution.
    Operation {
        /// Operation whose capability failed.
        operation: OpKey,
        /// Typed implementation failure.
        source: ReferenceOperationError,
    },
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
            Self::MissingCapability { operation, .. } => write!(
                formatter,
                "no reference capability for semantic operation {operation} and exact resolved signature"
            ),
            Self::Operation { operation, source } => {
                write!(
                    formatter,
                    "reference capability for {operation} failed: {source}"
                )
            }
            Self::MalformedProgram => formatter.write_str("verified semantic program is malformed"),
        }
    }
}

impl Error for EvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Operation { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiler_ir::semantic::{
        CanonicalValue, F32, F32Add, F32Constant, F32Multiply, InputKey, NormativeDefinitionRef,
        OperationArity, OperationConformance, OperationDefinition, OperationDefinitionFacts,
        OperationEffect, OperationInferenceError, OperationInferencer, OperationSchema, OutputKey,
        SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
        SemanticRegistryRegistrar, StrictSerialF32Sum, Value, ValueFact,
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

    fn evaluate_program(
        program: &SemanticProgram,
        inputs: &[InputBinding<'_>],
    ) -> Result<Vec<Tensor>, EvaluationError> {
        ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(program, inputs)
    }

    fn external_identity_op() -> OpKey {
        OpKey::new("test", "reference-identity", 1).unwrap()
    }

    struct IdentitySemantic;
    impl OperationInferencer for IdentitySemantic {
        fn infer(
            &self,
            operands: &[ValueFact],
            _: &OperationAttributes,
        ) -> Result<Vec<ValueFact>, OperationInferenceError> {
            Ok(vec![operands[0].clone()])
        }
    }

    struct ExternalSemanticProvider;
    impl SemanticRegistryProvider for ExternalSemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "reference-semantics", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), tiler_ir::semantic::RegistryError> {
            registrar.register_operation(OperationDefinition::new(
                external_identity_op(),
                OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), [])
                    .unwrap(),
                NormativeDefinitionRef::new("test reference identity v1")?,
                OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
                OperationConformance::new(
                    CanonicalValue::utf8("test.reference-identity.v1").unwrap(),
                ),
                OperationEffect::Pure,
                Arc::new(IdentitySemantic),
            ))
        }
    }

    struct IdentityReference;
    impl ReferenceOperation for IdentityReference {
        fn evaluate(
            &self,
            operands: &[&Tensor],
            _: &OperationAttributes,
        ) -> Result<Vec<Tensor>, ReferenceOperationError> {
            Ok(vec![operands[0].clone()])
        }
    }

    #[derive(Clone, Copy)]
    enum MalformedReferenceResult {
        WrongArity,
        WrongShape,
    }

    struct MalformedReference {
        result: MalformedReferenceResult,
    }

    impl ReferenceOperation for MalformedReference {
        fn evaluate(
            &self,
            _: &[&Tensor],
            _: &OperationAttributes,
        ) -> Result<Vec<Tensor>, ReferenceOperationError> {
            Ok(match self.result {
                MalformedReferenceResult::WrongArity => Vec::new(),
                MalformedReferenceResult::WrongShape => vec![Tensor::scalar(0.0)],
            })
        }
    }

    struct ExternalReferenceProvider {
        capability_revision: u32,
    }

    impl ReferenceRegistryProvider for ExternalReferenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "reference-capabilities", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut ReferenceRegistryRegistrar<'_>,
        ) -> Result<(), ReferenceRegistryError> {
            registrar.register(
                external_identity_op(),
                ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()]),
                ReferenceCapabilityRevision::new(self.capability_revision)?,
                Arc::new(IdentityReference),
            )
        }
    }

    struct MalformedReferenceProvider {
        result: MalformedReferenceResult,
    }

    impl ReferenceRegistryProvider for MalformedReferenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "malformed-reference-capability", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut ReferenceRegistryRegistrar<'_>,
        ) -> Result<(), ReferenceRegistryError> {
            registrar.register(
                external_identity_op(),
                ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()]),
                ReferenceCapabilityRevision::new(1)?,
                Arc::new(MalformedReference {
                    result: self.result,
                }),
            )
        }
    }

    fn evaluate_one(program: &SemanticProgram, input: &Tensor) -> Vec<Tensor> {
        let key = InputKey::new("x").unwrap();
        evaluate_program(program, &[InputBinding::new(&key, input)]).unwrap()
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
            evaluate_program(&program, &swapped),
            Err(EvaluationError::InputKey { input_index: 0, .. })
        ));
        assert!(matches!(
            evaluate_program(&program, &[InputBinding::new(&left_key, &left_tensor)]),
            Err(EvaluationError::InputCount { .. })
        ));
        let wrong = Tensor::new(Shape::from_dims([1]), vec![1.0]).unwrap();
        assert!(matches!(
            evaluate_program(
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

        let output = evaluate_program(&program, &[]).unwrap();
        assert_eq!(output[0].elements()[0].to_bits(), payload);
        assert_eq!(
            output[1].elements()[0].to_bits(),
            CANONICAL_F32_ARITHMETIC_NAN_BITS
        );
    }

    #[test]
    fn f32_arithmetic_preserves_subnormals_and_signed_zero_and_overflows_to_infinity() {
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let one = constant(&mut graph, 1.0);
        let two = constant(&mut graph, 2.0);
        let half = constant(&mut graph, 0.5);
        let minimum_subnormal = constant_bits(&mut graph, 0x0000_0001);
        let minimum_normal = constant_bits(&mut graph, 0x0080_0000);
        let maximum_finite = constant_bits(&mut graph, 0x7f7f_ffff);
        let negative_zero = constant_bits(&mut graph, 0x8000_0000);
        let positive_infinity = constant_bits(&mut graph, f32::INFINITY.to_bits());
        let negative_infinity = constant_bits(&mut graph, f32::NEG_INFINITY.to_bits());

        let preserved_subnormal = multiply(&mut graph, minimum_subnormal, one);
        let produced_subnormal = multiply(&mut graph, minimum_normal, half);
        let overflow = multiply(&mut graph, maximum_finite, two);
        let signed_zero = multiply(&mut graph, negative_zero, two);
        let invalid_infinities = add(&mut graph, positive_infinity, negative_infinity);

        for (key, value) in [
            ("preserved-subnormal", preserved_subnormal),
            ("produced-subnormal", produced_subnormal),
            ("overflow", overflow),
            ("signed-zero", signed_zero),
            ("invalid-infinities", invalid_infinities),
        ] {
            graph.output(OutputKey::new(key).unwrap(), value).unwrap();
        }
        let outputs = evaluate_program(&graph.build().unwrap(), &[]).unwrap();

        assert_eq!(outputs[0].elements()[0].to_bits(), 0x0000_0001);
        assert_eq!(outputs[1].elements()[0].to_bits(), 0x0040_0000);
        assert_eq!(outputs[2].elements()[0].to_bits(), f32::INFINITY.to_bits());
        assert_eq!(outputs[3].elements()[0].to_bits(), 0x8000_0000);
        assert_eq!(
            outputs[4].elements()[0].to_bits(),
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
            Err(tiler_ir::semantic::HandleError::ForeignGraph { .. })
        ));
        assert_eq!(program.input_count(), 0);
        assert_eq!(program.operation_count(), 1);
        let outputs = evaluate_program(&program, &[]).unwrap();
        assert_eq!(outputs[0].elements(), &[7.0]);
    }

    #[test]
    fn missing_and_external_reference_capabilities_are_explicit() {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics
            .register_provider(&ExternalSemanticProvider)
            .unwrap();
        let mut graph = SemanticProgramBuilder::try_new(semantics.freeze().unwrap()).unwrap();
        let input: Value<F32> = graph
            .input(InputKey::new("x").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let result = graph
            .apply(
                external_identity_op(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap();
        graph
            .output_resolved(OutputKey::new("result").unwrap(), result[0])
            .unwrap();
        let program = graph.build().unwrap();
        let key = InputKey::new("x").unwrap();
        let tensor = Tensor::new(Shape::from_dims([2]), vec![1.0, 2.0]).unwrap();
        let bindings = [InputBinding::new(&key, &tensor)];

        let error = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(&program, &bindings)
            .unwrap_err();
        assert!(matches!(
            error,
            EvaluationError::MissingCapability { operation, .. }
                if operation == external_identity_op()
        ));

        let mut references = ReferenceRegistryBuilder::standard().unwrap();
        references
            .register_provider(&ExternalReferenceProvider {
                capability_revision: 1,
            })
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        assert_eq!(
            evaluator.evaluate(&program, &bindings).unwrap(),
            vec![tensor]
        );
    }

    #[test]
    fn malformed_reference_results_fail_closed() {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics
            .register_provider(&ExternalSemanticProvider)
            .unwrap();
        let mut graph = SemanticProgramBuilder::try_new(semantics.freeze().unwrap()).unwrap();
        let input: Value<F32> = graph
            .input(InputKey::new("x").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let result = graph
            .apply(
                external_identity_op(),
                OperationAttributes::empty(),
                &[input.erase()],
            )
            .unwrap();
        graph
            .output_resolved(OutputKey::new("result").unwrap(), result[0])
            .unwrap();
        let program = graph.build().unwrap();
        let key = InputKey::new("x").unwrap();
        let tensor = Tensor::new(Shape::from_dims([2]), vec![1.0, 2.0]).unwrap();
        let bindings = [InputBinding::new(&key, &tensor)];

        for result in [
            MalformedReferenceResult::WrongArity,
            MalformedReferenceResult::WrongShape,
        ] {
            let mut references = ReferenceRegistryBuilder::new();
            references
                .register_provider(&MalformedReferenceProvider { result })
                .unwrap();
            let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
            assert_eq!(
                evaluator.evaluate(&program, &bindings),
                Err(EvaluationError::MalformedProgram)
            );
        }
    }

    #[test]
    fn registry_identity_is_deterministic_and_revision_complete() {
        let standard_a = ReferenceRegistryBuilder::standard()
            .unwrap()
            .freeze()
            .unwrap();
        let standard_b = ReferenceRegistryBuilder::standard()
            .unwrap()
            .freeze()
            .unwrap();
        assert_eq!(
            standard_a.canonical_identity(),
            standard_b.canonical_identity()
        );

        let with_revision = |capability_revision| {
            let mut builder = ReferenceRegistryBuilder::standard().unwrap();
            builder
                .register_provider(&ExternalReferenceProvider {
                    capability_revision,
                })
                .unwrap();
            builder.freeze().unwrap()
        };
        let revision_one = with_revision(1);
        let revision_two = with_revision(2);
        assert_ne!(
            revision_one.canonical_identity(),
            standard_a.canonical_identity()
        );
        assert_ne!(
            revision_one.canonical_identity(),
            revision_two.canonical_identity()
        );
    }

    #[test]
    fn duplicate_provider_registration_is_transactional() {
        let provider = ExternalReferenceProvider {
            capability_revision: 1,
        };
        let mut builder = ReferenceRegistryBuilder::standard().unwrap();
        builder.register_provider(&provider).unwrap();
        assert!(matches!(
            builder.register_provider(&provider),
            Err(ReferenceRegistryError::DuplicateCapability { operation, .. })
                if operation == external_identity_op()
        ));
        let after_rejection = builder.freeze().unwrap();

        let mut expected = ReferenceRegistryBuilder::standard().unwrap();
        expected.register_provider(&provider).unwrap();
        let expected = expected.freeze().unwrap();
        assert_eq!(
            after_rejection.canonical_identity(),
            expected.canonical_identity()
        );
    }
}
