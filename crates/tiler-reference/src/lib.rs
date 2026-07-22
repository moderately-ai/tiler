//! Host reference values and evaluation for verified Tiler semantic programs.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

use tiler_ir::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalIntegerWidth, CanonicalValueView, Definition, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, FrozenSemanticRegistry, InputKey, MAX_OPERATION_OPERANDS,
    MAX_OPERATION_RESULTS, OpKey, OperationAttributes, OperationId, ProviderIdentity,
    REDUCTION_AXES_ATTRIBUTE, RegistryError, ResolvedValueType, SemanticCapabilityAuthority,
    SemanticProgram, TypeKey, ValueId, add_f32_op, constant_f32_op, multiply_f32_op,
    strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

const MAX_REFERENCE_ELEMENT_BYTES: usize = 1024 * 1024;
const MAX_REFERENCE_TENSOR_ELEMENTS: usize = 16 * 1024 * 1024;
const MAX_REFERENCE_TENSOR_BYTES: usize = 64 * 1024 * 1024;
const MAX_REFERENCE_COMPONENTS: usize = 1_024;
const MAX_REFERENCE_COMPONENT_DEPTH: usize = 32;
const MAX_REFERENCE_CAPABILITIES: usize = 4_096;
const MAX_REFERENCE_REGISTRY_IDENTITY_BYTES: usize = 16 * 1024 * 1024;

/// Byte order supplied when constructing exact floating-point elements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FloatBitOrder {
    /// Most-significant byte first, which is Tiler's canonical representation.
    MostSignificantByteFirst,
    /// Least-significant byte first; construction normalizes it to canonical order.
    LeastSignificantByteFirst,
}

/// One exact canonical logical element in a dense reference tensor.
///
/// The enclosing tensor's resolved semantic type and registered reference
/// validator define how these bytes are interpreted. Compound values use
/// [`ReferenceComponent`] tensors instead of embedding an untyped recursive
/// scalar structure.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReferenceElement(Vec<u8>);

impl ReferenceElement {
    /// Creates one bounded canonical element representation.
    ///
    /// # Errors
    ///
    /// Returns a resource error before retaining an oversized element.
    pub fn new(bytes: impl AsRef<[u8]>) -> Result<Self, EvaluationError> {
        let bytes = bytes.as_ref();
        if bytes.len() > MAX_REFERENCE_ELEMENT_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::ElementBytes,
                limit: MAX_REFERENCE_ELEMENT_BYTES,
                actual: bytes.len(),
            });
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Creates exact floating-point bits and normalizes them to canonical
    /// most-significant-byte-first order.
    ///
    /// The byte order is part of this public constructor rather than inherited
    /// from the host. The resolved tensor type remains the authority for the
    /// floating-point format and therefore for the required payload width.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::EmptyFloatBits`] for an empty payload or a
    /// resource error for an oversized payload.
    pub fn from_float_bits(
        bits: impl AsRef<[u8]>,
        order: FloatBitOrder,
    ) -> Result<Self, EvaluationError> {
        let bits = bits.as_ref();
        if bits.is_empty() {
            return Err(EvaluationError::EmptyFloatBits);
        }
        if bits.len() > MAX_REFERENCE_ELEMENT_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::ElementBytes,
                limit: MAX_REFERENCE_ELEMENT_BYTES,
                actual: bits.len(),
            });
        }
        let mut canonical = bits.to_vec();
        if order == FloatBitOrder::LeastSignificantByteFirst {
            canonical.reverse();
        }
        Ok(Self(canonical))
    }

    /// Returns exact canonical element bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Stable schema-local role of one compound reference component.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReferenceComponentRole(u32);

impl ReferenceComponentRole {
    /// Creates a stable component role ID.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the portable role ID.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// One stable-role tensor component of a compound logical reference value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceComponent {
    role: ReferenceComponentRole,
    tensor: Tensor,
}

impl ReferenceComponent {
    /// Creates one compound component.
    #[must_use]
    pub const fn new(role: ReferenceComponentRole, tensor: Tensor) -> Self {
        Self { role, tensor }
    }

    /// Returns the stable component role.
    #[must_use]
    pub const fn role(&self) -> ReferenceComponentRole {
        self.role
    }

    /// Returns the exact component tensor.
    #[must_use]
    pub const fn tensor(&self) -> &Tensor {
        &self.tensor
    }
}

/// Borrowed representation of one reference tensor payload.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum TensorPayloadView<'a> {
    /// Dense exact elements in logical row-major order.
    Dense(&'a [ReferenceElement]),
    /// Ordered stable-role component tensors for one compound logical value.
    Compound(&'a [ReferenceComponent]),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TensorPayload {
    Dense(Vec<ReferenceElement>),
    Compound(Vec<ReferenceComponent>),
}

/// An owned, exact, dense row-major tensor used by the reference evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tensor(Arc<TensorData>);

#[derive(Debug, Eq, PartialEq)]
struct TensorData {
    resolved_type: ResolvedValueType,
    shape: Shape,
    payload: TensorPayload,
}

impl Tensor {
    /// Creates a tensor after checking its resolved type, element count, and
    /// aggregate retained-byte bounds.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::ElementCount`] when the payload length does
    /// not match the shape, or [`EvaluationError::ShapeTooLarge`] when the
    /// element count cannot be represented on this host.
    pub fn dense(
        resolved_type: ResolvedValueType,
        shape: Shape,
        elements: Vec<ReferenceElement>,
    ) -> Result<Self, EvaluationError> {
        let expected = shape
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?;
        if expected > MAX_REFERENCE_TENSOR_ELEMENTS {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual: expected,
            });
        }
        if elements.len() != expected {
            return Err(EvaluationError::ElementCount {
                expected,
                actual: elements.len(),
            });
        }
        let bytes = elements.iter().try_fold(0_usize, |bytes, element| {
            bytes
                .checked_add(element.as_bytes().len())
                .ok_or(EvaluationError::ResourceExceeded {
                    resource: ReferenceResource::TensorBytes,
                    limit: MAX_REFERENCE_TENSOR_BYTES,
                    actual: usize::MAX,
                })
        })?;
        if bytes > MAX_REFERENCE_TENSOR_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorBytes,
                limit: MAX_REFERENCE_TENSOR_BYTES,
                actual: bytes,
            });
        }
        Ok(Self(Arc::new(TensorData {
            resolved_type,
            shape,
            payload: TensorPayload::Dense(elements),
        })))
    }

    /// Creates a compound tensor from ordered stable-role component tensors.
    ///
    /// Component shapes are intentionally independent: the resolved compound
    /// type's validator owns role, type, and shape relationships.
    ///
    /// # Errors
    ///
    /// Returns a typed resource error before retaining an over-limit compound value.
    pub fn compound(
        resolved_type: ResolvedValueType,
        shape: Shape,
        components: Vec<ReferenceComponent>,
    ) -> Result<Self, EvaluationError> {
        let logical_elements = shape
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?;
        if logical_elements > MAX_REFERENCE_TENSOR_ELEMENTS {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual: logical_elements,
            });
        }
        let mut resources = ReferenceWork {
            elements: logical_elements,
            ..ReferenceWork::default()
        };
        validate_compound_resources(&components, 1, &mut resources)?;
        Ok(Self(Arc::new(TensorData {
            resolved_type,
            shape,
            payload: TensorPayload::Compound(components),
        })))
    }

    /// Creates a rank-zero dense tensor with one exact element.
    ///
    /// # Errors
    ///
    /// Returns a bounded reference-value error.
    pub fn scalar(
        resolved_type: ResolvedValueType,
        value: ReferenceElement,
    ) -> Result<Self, EvaluationError> {
        Self::dense(resolved_type, Shape::new([]), vec![value])
    }

    /// Returns the exact shape-independent semantic value type.
    #[must_use]
    pub fn resolved_type(&self) -> &ResolvedValueType {
        &self.0.resolved_type
    }

    /// Returns the logical shape.
    #[must_use]
    pub fn shape(&self) -> &Shape {
        &self.0.shape
    }

    /// Returns the exact payload representation.
    #[must_use]
    pub fn payload(&self) -> TensorPayloadView<'_> {
        match &self.0.payload {
            TensorPayload::Dense(elements) => TensorPayloadView::Dense(elements),
            TensorPayload::Compound(components) => TensorPayloadView::Compound(components),
        }
    }

    fn storage_id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
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
    ///
    /// # Errors
    ///
    /// Returns a typed resource error before retaining an over-limit operand
    /// or result sequence.
    pub fn new(
        operands: impl IntoIterator<Item = ResolvedValueType>,
        results: impl IntoIterator<Item = ResolvedValueType>,
    ) -> Result<Self, ReferenceRegistryError> {
        Ok(Self {
            operands: collect_signature_types(
                operands,
                ReferenceRegistryResource::SignatureOperands,
                usize::try_from(MAX_OPERATION_OPERANDS).unwrap_or(usize::MAX),
            )?,
            results: collect_signature_types(
                results,
                ReferenceRegistryResource::SignatureResults,
                usize::try_from(MAX_OPERATION_RESULTS).unwrap_or(usize::MAX),
            )?,
        })
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
///
/// Implementations are trusted native callbacks. They must be deterministic
/// functions of the request and must not panic. Tiler does not catch panics:
/// an unwind (or process abort under the active panic profile) is outside the
/// recoverable evaluation contract. Returned failures and host-owned output
/// validation remain recoverable and retain provider attribution.
pub trait ReferenceOperation: Send + Sync + 'static {
    /// Evaluates ordered operands and canonical attributes without fusion.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when inputs violate this capability's contract.
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError>;
}

/// Borrowed inputs to one exact reference-operation callback.
#[derive(Clone, Copy)]
pub struct ReferenceEvaluationRequest<'a> {
    operands: &'a [&'a Tensor],
    attributes: &'a OperationAttributes,
}

impl fmt::Debug for ReferenceEvaluationRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReferenceEvaluationRequest")
            .field("operand_count", &self.operands.len())
            .field("attributes", &self.attributes)
            .finish()
    }
}

impl<'a> ReferenceEvaluationRequest<'a> {
    /// Returns ordered operand tensors.
    #[must_use]
    pub const fn operands(self) -> &'a [&'a Tensor] {
        self.operands
    }

    /// Returns canonical operation attributes.
    #[must_use]
    pub const fn attributes(self) -> &'a OperationAttributes {
        self.attributes
    }
}

/// Host-owned bounded output writer for one reference callback.
///
/// A failed write poisons the writer. Catching or ignoring the returned error
/// cannot make a partial or over-limit result appear successful.
pub struct ReferenceOutputs {
    expected: usize,
    values: Vec<Tensor>,
    retention: EvaluationRetention,
    failure: Option<ReferenceOperationError>,
}

impl fmt::Debug for ReferenceOutputs {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReferenceOutputs")
            .field("expected", &self.expected)
            .field("written", &self.values.len())
            .field("retained_work", &self.retention.work)
            .field("failed", &self.failure.is_some())
            .finish()
    }
}

impl ReferenceOutputs {
    fn new(expected: usize, retention: EvaluationRetention) -> Self {
        Self {
            expected,
            values: Vec::with_capacity(expected),
            retention,
            failure: None,
        }
    }

    /// Writes one ordered result tensor.
    ///
    /// # Errors
    ///
    /// Returns a sticky typed failure for excess results or aggregate retained
    /// bytes. Subsequent writes return the original failure.
    pub fn push(&mut self, value: Tensor) -> Result<(), ReferenceOperationError> {
        if let Some(error) = self.failure.clone() {
            return Err(error);
        }
        let actual = self.values.len().saturating_add(1);
        if actual > self.expected {
            return Err(self.fail(ReferenceOperationError::ResultCount {
                expected: self.expected,
                actual,
            }));
        }
        if let Err(error) = reserve_output_work(&mut self.retention, &value) {
            return Err(self.fail(error));
        }
        self.values.push(value);
        Ok(())
    }

    fn fail(&mut self, error: ReferenceOperationError) -> ReferenceOperationError {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
        self.failure
            .clone()
            .expect("output failure was just recorded")
    }

    fn finish(
        mut self,
        callback: Result<(), ReferenceOperationError>,
    ) -> Result<Vec<Tensor>, ReferenceOperationError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        callback?;
        if self.values.len() != self.expected {
            return Err(ReferenceOperationError::ResultCount {
                expected: self.expected,
                actual: self.values.len(),
            });
        }
        Ok(std::mem::take(&mut self.values))
    }
}

/// Validates the exact structural representation of one resolved reference type.
///
/// Validators have the same deterministic, non-panicking native-callback
/// trust boundary as [`ReferenceOperation`]. Recoverable returned failures are
/// attributed to the selected provider.
pub trait ReferenceValueValidator: Send + Sync + 'static {
    /// Validates one complete tensor representation against the registered resolved type.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when the element does not implement that
    /// semantic value contract.
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError>;
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
    semantic_authority: SemanticCapabilityAuthority,
    implementation: Arc<dyn ReferenceOperation>,
}

#[derive(Clone)]
struct RegisteredReferenceValueValidator {
    provider: ProviderIdentity,
    revision: ReferenceCapabilityRevision,
    semantic_authority: SemanticCapabilityAuthority,
    implementation: Arc<dyn ReferenceValueValidator>,
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

struct StagedReferenceCapability {
    revision: ReferenceCapabilityRevision,
    semantic_authority: SemanticCapabilityAuthority,
    implementation: Arc<dyn ReferenceOperation>,
}

struct StagedReferenceValueValidator {
    revision: ReferenceCapabilityRevision,
    semantic_authority: SemanticCapabilityAuthority,
    implementation: Arc<dyn ReferenceValueValidator>,
}

#[derive(Default)]
struct ReferenceRegistrationBatch {
    capabilities: BTreeMap<ReferenceCapabilityKey, StagedReferenceCapability>,
    value_validators: BTreeMap<ResolvedValueType, StagedReferenceValueValidator>,
    failure: Option<ReferenceRegistryError>,
    canonical_bytes: usize,
}

/// Host-owned registration surface for one reference provider transaction.
pub struct ReferenceRegistryRegistrar<'a> {
    batch: &'a mut ReferenceRegistrationBatch,
    semantic_registry: &'a FrozenSemanticRegistry,
    provider: &'a ProviderIdentity,
    existing_capabilities: usize,
    existing_canonical_bytes: usize,
}

impl ReferenceRegistryRegistrar<'_> {
    fn prior_failure(&self) -> Option<ReferenceRegistryError> {
        self.batch.failure.clone()
    }

    fn fail(&mut self, error: &ReferenceRegistryError) -> ReferenceRegistryError {
        if self.batch.failure.is_none() {
            self.batch.failure = Some(error.clone());
        }
        self.batch
            .failure
            .clone()
            .expect("registration failure was just recorded")
    }

    /// Registers one exact resolved-type representation validator.
    ///
    /// # Errors
    ///
    /// Returns a sticky typed error for duplicate authority, missing semantic
    /// authority, or a registry resource limit.
    pub fn register_value_type(
        &mut self,
        resolved_type: ResolvedValueType,
        revision: ReferenceCapabilityRevision,
        implementation: Arc<dyn ReferenceValueValidator>,
    ) -> Result<(), ReferenceRegistryError> {
        if let Some(error) = self.prior_failure() {
            return Err(error);
        }
        if self.batch.value_validators.contains_key(&resolved_type) {
            let error = ReferenceRegistryError::DuplicateValueCapability { resolved_type };
            return Err(self.fail(&error));
        }
        self.reserve_capability()?;
        let semantic_authority = self
            .semantic_registry
            .project_value_authority(&resolved_type)
            .map_err(|source| {
                self.fail(&ReferenceRegistryError::SemanticValueAuthority {
                    resolved_type: resolved_type.clone(),
                    source: Arc::new(source),
                })
            })?;
        let added = reference_value_identity_len(
            &resolved_type,
            &semantic_authority,
            self.provider,
            revision,
        );
        self.reserve_canonical_bytes(added)?;
        self.batch.value_validators.insert(
            resolved_type,
            StagedReferenceValueValidator {
                revision,
                semantic_authority,
                implementation,
            },
        );
        Ok(())
    }

    fn reserve_capability(&mut self) -> Result<(), ReferenceRegistryError> {
        let staged = self
            .batch
            .capabilities
            .len()
            .saturating_add(self.batch.value_validators.len());
        let actual = self
            .existing_capabilities
            .saturating_add(staged)
            .saturating_add(1);
        if actual > MAX_REFERENCE_CAPABILITIES {
            let error = ReferenceRegistryError::ResourceExceeded {
                resource: ReferenceRegistryResource::Capabilities,
                limit: MAX_REFERENCE_CAPABILITIES,
                actual,
            };
            return Err(self.fail(&error));
        }
        Ok(())
    }

    fn reserve_canonical_bytes(&mut self, added: usize) -> Result<(), ReferenceRegistryError> {
        let actual = self
            .existing_canonical_bytes
            .saturating_add(self.batch.canonical_bytes)
            .saturating_add(added);
        if actual > MAX_REFERENCE_REGISTRY_IDENTITY_BYTES {
            let error = ReferenceRegistryError::ResourceExceeded {
                resource: ReferenceRegistryResource::CanonicalIdentityBytes,
                limit: MAX_REFERENCE_REGISTRY_IDENTITY_BYTES,
                actual,
            };
            return Err(self.fail(&error));
        }
        self.batch.canonical_bytes = self.batch.canonical_bytes.saturating_add(added);
        Ok(())
    }

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
        if let Some(error) = self.prior_failure() {
            return Err(error);
        }
        let key = ReferenceCapabilityKey {
            operation,
            signature,
        };
        if self.batch.capabilities.contains_key(&key) {
            let error = ReferenceRegistryError::DuplicateCapability {
                operation: key.operation,
                signature: key.signature,
            };
            return Err(self.fail(&error));
        }
        self.reserve_capability()?;
        let semantic_authority = self
            .semantic_registry
            .project_operation_authority(
                &key.operation,
                key.signature.operands(),
                key.signature.results(),
            )
            .map_err(|source| {
                self.fail(&ReferenceRegistryError::SemanticAuthority {
                    operation: key.operation.clone(),
                    source: Arc::new(source),
                })
            })?;
        let added =
            reference_capability_identity_len(&key, &semantic_authority, self.provider, revision);
        self.reserve_canonical_bytes(added)?;
        self.batch.capabilities.insert(
            key,
            StagedReferenceCapability {
                revision,
                semantic_authority,
                implementation,
            },
        );
        Ok(())
    }
}

/// Mutable single-use constructor for a frozen reference registry.
pub struct ReferenceRegistryBuilder {
    semantic_registry: FrozenSemanticRegistry,
    capabilities: BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
    value_validators: BTreeMap<ResolvedValueType, RegisteredReferenceValueValidator>,
    canonical_bytes: usize,
}

impl ReferenceRegistryBuilder {
    /// Creates an empty reference registry builder bound to one exact semantic snapshot.
    #[must_use]
    pub fn new(semantic_registry: FrozenSemanticRegistry) -> Self {
        let canonical_bytes = reference_identity_base_len(&semantic_registry);
        Self {
            semantic_registry,
            capabilities: BTreeMap::new(),
            value_validators: BTreeMap::new(),
            canonical_bytes,
        }
    }

    /// Creates the governed initial F32 reference profile.
    ///
    /// # Errors
    ///
    /// Returns a typed error if governed registration violates the public contract.
    pub fn standard() -> Result<Self, ReferenceRegistryError> {
        let semantic_registry = FrozenSemanticRegistry::standard()
            .map_err(|source| ReferenceRegistryError::SemanticRegistry(Arc::new(source)))?;
        let mut builder = Self::new(semantic_registry);
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
        let callback_result = provider.register(&mut ReferenceRegistryRegistrar {
            batch: &mut batch,
            semantic_registry: &self.semantic_registry,
            provider: &identity,
            existing_capabilities: self
                .capabilities
                .len()
                .saturating_add(self.value_validators.len()),
            existing_canonical_bytes: self.canonical_bytes,
        });
        if let Some(error) = batch.failure.clone() {
            return Err(error);
        }
        callback_result?;
        if batch.capabilities.is_empty() && batch.value_validators.is_empty() {
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
        for resolved_type in batch.value_validators.keys() {
            if self.value_validators.contains_key(resolved_type) {
                return Err(ReferenceRegistryError::DuplicateValueCapability {
                    resolved_type: resolved_type.clone(),
                });
            }
        }
        let batch_bytes = batch.canonical_bytes;
        self.capabilities
            .extend(batch.capabilities.into_iter().map(|(key, staged)| {
                (
                    key,
                    RegisteredReferenceCapability {
                        provider: identity.clone(),
                        revision: staged.revision,
                        semantic_authority: staged.semantic_authority,
                        implementation: staged.implementation,
                    },
                )
            }));
        self.value_validators
            .extend(
                batch
                    .value_validators
                    .into_iter()
                    .map(|(resolved_type, staged)| {
                        (
                            resolved_type,
                            RegisteredReferenceValueValidator {
                                provider: identity.clone(),
                                revision: staged.revision,
                                semantic_authority: staged.semantic_authority,
                                implementation: staged.implementation,
                            },
                        )
                    }),
            );
        self.canonical_bytes = self.canonical_bytes.saturating_add(batch_bytes);
        Ok(())
    }

    /// Freezes canonical immutable reference capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceRegistryError::EmptyRegistry`] when empty.
    pub fn freeze(self) -> Result<FrozenReferenceRegistry, ReferenceRegistryError> {
        if self.capabilities.is_empty() && self.value_validators.is_empty() {
            return Err(ReferenceRegistryError::EmptyRegistry);
        }
        let identity = compute_reference_identity(
            &self.semantic_registry,
            &self.capabilities,
            &self.value_validators,
            self.canonical_bytes,
        );
        Ok(FrozenReferenceRegistry(Arc::new(
            FrozenReferenceRegistryData {
                semantic_registry: self.semantic_registry,
                capabilities: self.capabilities,
                value_validators: self.value_validators,
                identity,
            },
        )))
    }
}

struct FrozenReferenceRegistryData {
    semantic_registry: FrozenSemanticRegistry,
    capabilities: BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
    value_validators: BTreeMap<ResolvedValueType, RegisteredReferenceValueValidator>,
    identity: CanonicalReferenceRegistryIdentity,
}

/// Immutable exact reference-capability registry.
#[derive(Clone)]
pub struct FrozenReferenceRegistry(Arc<FrozenReferenceRegistryData>);

impl fmt::Debug for FrozenReferenceRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenReferenceRegistry")
            .field("capability_count", &self.0.capabilities.len())
            .field("value_validator_count", &self.0.value_validators.len())
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
        &self.0.identity
    }

    /// Returns the exact frozen semantic registry this reference registry was
    /// built against.
    #[must_use]
    pub fn semantic_registry(&self) -> &FrozenSemanticRegistry {
        &self.0.semantic_registry
    }

    fn resolve(
        &self,
        operation: &OpKey,
        signature: &ReferenceSignature,
        attributes: &OperationAttributes,
        semantic_registry: &FrozenSemanticRegistry,
    ) -> Result<&RegisteredReferenceCapability, EvaluationError> {
        let capability = self
            .0
            .capabilities
            .get(&ReferenceCapabilityKey {
                operation: operation.clone(),
                signature: signature.clone(),
            })
            .ok_or_else(|| EvaluationError::MissingCapability {
                operation: operation.clone(),
                signature: Arc::new(signature.clone()),
            })?;
        let expected = self
            .0
            .semantic_registry
            .project_operation_occurrence_authority(
                operation,
                signature.operands(),
                signature.results(),
                attributes,
            )
            .map_err(|source| EvaluationError::SemanticAuthority {
                operation: operation.clone(),
                source: Arc::new(source),
            })?;
        let actual = semantic_registry
            .project_operation_occurrence_authority(
                operation,
                signature.operands(),
                signature.results(),
                attributes,
            )
            .map_err(|source| EvaluationError::SemanticAuthority {
                operation: operation.clone(),
                source: Arc::new(source),
            })?;
        if !compatible_authority(&expected, &actual) {
            return Err(EvaluationError::CapabilityAuthorityMismatch {
                operation: operation.clone(),
                provider: Arc::new(capability.provider.clone()),
                capability_revision: capability.revision,
            });
        }
        Ok(capability)
    }

    fn validate_value(
        &self,
        tensor: &Tensor,
        semantic_registry: &FrozenSemanticRegistry,
    ) -> Result<(), EvaluationError> {
        let validator = self
            .0
            .value_validators
            .get(tensor.resolved_type())
            .ok_or_else(|| EvaluationError::MissingValueCapability {
                resolved_type: Arc::new(tensor.resolved_type().clone()),
            })?;
        let actual = semantic_registry
            .project_value_authority(tensor.resolved_type())
            .map_err(|source| EvaluationError::SemanticValueAuthority {
                resolved_type: Arc::new(tensor.resolved_type().clone()),
                source: Arc::new(source),
            })?;
        if !compatible_authority(&validator.semantic_authority, &actual) {
            return Err(EvaluationError::ValueCapabilityAuthorityMismatch {
                resolved_type: Arc::new(tensor.resolved_type().clone()),
                provider: Arc::new(validator.provider.clone()),
                capability_revision: validator.revision,
            });
        }
        validator
            .implementation
            .validate(tensor)
            .map_err(|source| EvaluationError::Value {
                resolved_type: Arc::new(tensor.resolved_type().clone()),
                provider: Arc::new(validator.provider.clone()),
                capability_revision: validator.revision,
                source,
            })
    }
}

fn compatible_authority(
    expected: &SemanticCapabilityAuthority,
    actual: &SemanticCapabilityAuthority,
) -> bool {
    expected.reached_definitions() == actual.reached_definitions()
        && expected.admission_provenance() == actual.admission_provenance()
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

/// Governed resource in a reference-capability registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceRegistryResource {
    /// Operand types retained by one exact signature.
    SignatureOperands,
    /// Result types retained by one exact signature.
    SignatureResults,
    /// Aggregate operation and value-representation capabilities.
    Capabilities,
    /// Canonical registry identity bytes.
    CanonicalIdentityBytes,
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
    /// Two registrations claimed one exact value-representation contract.
    DuplicateValueCapability {
        /// Colliding resolved semantic type.
        resolved_type: ResolvedValueType,
    },
    /// The selected semantic registry could not be constructed.
    SemanticRegistry(Arc<RegistryError>),
    /// An operation capability lacked complete semantic authority.
    SemanticAuthority {
        /// Operation being registered.
        operation: OpKey,
        /// Semantic authority failure.
        source: Arc<RegistryError>,
    },
    /// A value validator lacked complete semantic authority.
    SemanticValueAuthority {
        /// Resolved value type being registered.
        resolved_type: ResolvedValueType,
        /// Semantic authority failure.
        source: Arc<RegistryError>,
    },
    /// A registry resource exceeded its governed bound.
    ResourceExceeded {
        /// Bounded resource.
        resource: ReferenceRegistryResource,
        /// Active limit.
        limit: usize,
        /// First rejected size.
        actual: usize,
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
            Self::DuplicateValueCapability { resolved_type } => write!(
                formatter,
                "duplicate reference value capability for {resolved_type:?}"
            ),
            Self::SemanticRegistry(source) => {
                write!(formatter, "semantic registry construction failed: {source}")
            }
            Self::SemanticAuthority { operation, source } => write!(
                formatter,
                "semantic authority for reference operation {operation} failed: {source}"
            ),
            Self::SemanticValueAuthority { source, .. } => {
                write!(
                    formatter,
                    "semantic authority for reference value failed: {source}"
                )
            }
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "reference registry resource {resource:?} has size {actual}, exceeding {limit}"
            ),
        }
    }
}

impl Error for ReferenceRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SemanticRegistry(source)
            | Self::SemanticAuthority { source, .. }
            | Self::SemanticValueAuthority { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Failure to validate a reference tensor representation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceValueError {
    /// The payload does not implement the registered resolved type.
    InvalidRepresentation,
}

impl fmt::Display for ReferenceValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRepresentation => {
                formatter.write_str("invalid reference value representation")
            }
        }
    }
}

impl Error for ReferenceValueError {}

/// Failure inside one exact reference implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceOperationError {
    /// Operands or attributes violated the registered capability contract.
    InvalidApplication,
    /// Shape arithmetic exceeded host limits.
    ShapeTooLarge,
    /// The callback produced the wrong number of ordered results.
    ResultCount {
        /// Required result count.
        expected: usize,
        /// Produced result count.
        actual: usize,
    },
    /// One callback output exceeded the governed logical element bound.
    OutputElementsExceeded {
        /// Active logical element limit.
        limit: usize,
        /// First rejected element count.
        actual: usize,
    },
    /// Callback outputs exceeded the governed aggregate component bound.
    OutputComponentsExceeded {
        /// Active recursive component limit.
        limit: usize,
        /// First rejected aggregate component count.
        actual: usize,
    },
    /// Aggregate callback output exceeded the host-owned writer budget.
    OutputResourceExceeded {
        /// Active byte limit.
        limit: usize,
        /// First rejected aggregate size.
        actual: usize,
    },
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
            Self::ResultCount { expected, actual } => {
                write!(
                    formatter,
                    "reference operation produced {actual} results, expected {expected}"
                )
            }
            Self::OutputElementsExceeded { limit, actual } => write!(
                formatter,
                "reference operation output has {actual} elements, exceeding {limit}"
            ),
            Self::OutputComponentsExceeded { limit, actual } => write!(
                formatter,
                "reference operation output has {actual} components, exceeding {limit}"
            ),
            Self::OutputResourceExceeded { limit, actual } => write!(
                formatter,
                "reference operation output retained {actual} bytes, exceeding {limit}"
            ),
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

fn binary(
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

fn strict_sum(input: &Tensor, axes: &[Axis]) -> Result<Tensor, ReferenceOperationError> {
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

fn row_major_strides(shape: &Shape) -> Result<Vec<usize>, ReferenceOperationError> {
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

fn f32_elements(tensor: &Tensor) -> Result<&[ReferenceElement], ReferenceOperationError> {
    if tensor.resolved_type() != &F32::resolved_type() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    match tensor.payload() {
        TensorPayloadView::Dense(elements) => Ok(elements),
        TensorPayloadView::Compound(_) => Err(ReferenceOperationError::InvalidApplication),
    }
}

fn f32_element(value: f32) -> Result<ReferenceElement, ReferenceOperationError> {
    ReferenceElement::from_float_bits(
        value.to_bits().to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn decode_f32(element: &ReferenceElement) -> Result<f32, ReferenceOperationError> {
    let bits = <[u8; 4]>::try_from(element.as_bytes())
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    Ok(f32::from_bits(u32::from_be_bytes(bits)))
}

fn validate_compound_resources(
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
struct ReferenceWork {
    bytes: usize,
    elements: usize,
    components: usize,
}

#[derive(Clone, Debug, Default)]
struct EvaluationRetention {
    work: ReferenceWork,
    storage_ids: HashSet<usize>,
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

fn reserve_evaluation_work(
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

fn reserve_output_work(
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

struct StandardReferenceProvider;

impl ReferenceRegistryProvider for StandardReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler", "standard-reference", 3)
            .expect("the governed reference provider identity is valid")
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        let revision = ReferenceCapabilityRevision::new(3)?;
        registrar.register_value_type(
            F32::resolved_type(),
            revision,
            Arc::new(F32ValueValidator),
        )?;
        registrar.register(
            constant_f32_op(),
            ReferenceSignature::new([], [F32::resolved_type()])?,
            revision,
            Arc::new(F32ConstantReference),
        )?;
        let binary_signature = ReferenceSignature::new(
            [F32::resolved_type(), F32::resolved_type()],
            [F32::resolved_type()],
        )?;
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
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
            revision,
            Arc::new(StrictSerialF32SumReference),
        )
    }
}

struct F32ValueValidator;

impl ReferenceValueValidator for F32ValueValidator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        if tensor.resolved_type() != &F32::resolved_type() {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            return Err(ReferenceValueError::InvalidRepresentation);
        };
        if elements.iter().any(|element| element.as_bytes().len() != 4) {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        Ok(())
    }
}

struct F32ConstantReference;

impl ReferenceOperation for F32ConstantReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if !operands.is_empty() || attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(CanonicalValueView::FloatBits(bits)) = attributes
            .get(F32_CONSTANT_BITS_ATTRIBUTE)
            .map(tiler_ir::semantic::CanonicalValue::view)
        else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if bits.format()
            != &TypeKey::new("tiler", "f32", 1)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?
        {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let element =
            ReferenceElement::from_float_bits(bits.bits(), FloatBitOrder::MostSignificantByteFirst)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let tensor = Tensor::scalar(F32::resolved_type(), element)
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        outputs.push(tensor)
    }
}

enum F32BinaryReference {
    Multiply,
    Add,
}

impl ReferenceOperation for F32BinaryReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let attributes = request.attributes();
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
        outputs.push(result)
    }
}

struct StrictSerialF32SumReference;

impl ReferenceOperation for StrictSerialF32SumReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let [input] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let axes = reduction_axes(request.attributes())?;
        outputs.push(strict_sum(input, &axes)?)
    }
}

fn compute_reference_identity(
    semantic_registry: &FrozenSemanticRegistry,
    capabilities: &BTreeMap<ReferenceCapabilityKey, RegisteredReferenceCapability>,
    value_validators: &BTreeMap<ResolvedValueType, RegisteredReferenceValueValidator>,
    exact_len: usize,
) -> CanonicalReferenceRegistryIdentity {
    let mut bytes = Vec::with_capacity(exact_len);
    bytes.extend_from_slice(b"tiler.reference-registry.v2\0");
    encode_bytes(&mut bytes, semantic_registry.snapshot_identity().as_bytes());
    encode_len(&mut bytes, value_validators.len());
    for (resolved_type, validator) in value_validators {
        encode_bytes(&mut bytes, resolved_type.canonical_encoding().as_bytes());
        encode_reference_authority(&mut bytes, &validator.semantic_authority);
        encode_provider_capability(&mut bytes, &validator.provider, validator.revision);
    }
    encode_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        encode_op_key(&mut bytes, &key.operation);
        encode_signature(&mut bytes, &key.signature);
        encode_reference_authority(&mut bytes, &capability.semantic_authority);
        encode_provider_capability(&mut bytes, &capability.provider, capability.revision);
    }
    debug_assert_eq!(bytes.len(), exact_len);
    CanonicalReferenceRegistryIdentity(bytes)
}

fn encode_reference_authority(output: &mut Vec<u8>, authority: &SemanticCapabilityAuthority) {
    encode_bytes(output, authority.reached_definitions().as_bytes());
    encode_bytes(output, authority.admission_provenance().as_bytes());
    encode_bytes(output, authority.registry_snapshot().as_bytes());
}

fn encode_provider_capability(
    output: &mut Vec<u8>,
    provider: &ProviderIdentity,
    revision: ReferenceCapabilityRevision,
) {
    encode_bytes(output, provider.namespace().as_bytes());
    encode_bytes(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
    output.extend_from_slice(&revision.get().to_be_bytes());
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

const fn encoded_bytes_len(payload_len: usize) -> usize {
    std::mem::size_of::<u64>().saturating_add(payload_len)
}

fn reference_identity_base_len(semantic_registry: &FrozenSemanticRegistry) -> usize {
    b"tiler.reference-registry.v2\0"
        .len()
        .saturating_add(encoded_bytes_len(
            semantic_registry.snapshot_identity().as_bytes().len(),
        ))
        .saturating_add(2 * std::mem::size_of::<u64>())
}

fn reference_authority_identity_len(authority: &SemanticCapabilityAuthority) -> usize {
    [
        authority.reached_definitions().as_bytes().len(),
        authority.admission_provenance().as_bytes().len(),
        authority.registry_snapshot().as_bytes().len(),
    ]
    .into_iter()
    .map(encoded_bytes_len)
    .fold(0_usize, usize::saturating_add)
}

fn reference_provider_identity_len(provider: &ProviderIdentity) -> usize {
    encoded_bytes_len(provider.namespace().len())
        .saturating_add(encoded_bytes_len(provider.name().len()))
        .saturating_add(2 * std::mem::size_of::<u32>())
}

fn reference_value_identity_len(
    resolved_type: &ResolvedValueType,
    authority: &SemanticCapabilityAuthority,
    provider: &ProviderIdentity,
    _revision: ReferenceCapabilityRevision,
) -> usize {
    encoded_bytes_len(resolved_type.canonical_encoding().as_bytes().len())
        .saturating_add(reference_authority_identity_len(authority))
        .saturating_add(reference_provider_identity_len(provider))
}

fn reference_signature_identity_len(signature: &ReferenceSignature) -> usize {
    [signature.operands(), signature.results()]
        .into_iter()
        .map(|values| {
            values
                .iter()
                .map(|value| encoded_bytes_len(value.canonical_encoding().as_bytes().len()))
                .fold(std::mem::size_of::<u64>(), usize::saturating_add)
        })
        .fold(0_usize, usize::saturating_add)
}

fn reference_capability_identity_len(
    key: &ReferenceCapabilityKey,
    authority: &SemanticCapabilityAuthority,
    provider: &ProviderIdentity,
    _revision: ReferenceCapabilityRevision,
) -> usize {
    encoded_bytes_len(key.operation.namespace().len())
        .saturating_add(encoded_bytes_len(key.operation.name().len()))
        .saturating_add(std::mem::size_of::<u32>())
        .saturating_add(reference_signature_identity_len(&key.signature))
        .saturating_add(reference_authority_identity_len(authority))
        .saturating_add(reference_provider_identity_len(provider))
}

fn collect_signature_types(
    values: impl IntoIterator<Item = ResolvedValueType>,
    resource: ReferenceRegistryResource,
    limit: usize,
) -> Result<Vec<ResolvedValueType>, ReferenceRegistryError> {
    let mut retained = Vec::new();
    let mut retained_bytes = 0_usize;
    for value in values.into_iter().take(limit.saturating_add(1)) {
        if retained.len() == limit {
            return Err(ReferenceRegistryError::ResourceExceeded {
                resource,
                limit,
                actual: limit.saturating_add(1),
            });
        }
        retained_bytes = retained_bytes.saturating_add(value.canonical_encoding().as_bytes().len());
        if retained_bytes > MAX_REFERENCE_REGISTRY_IDENTITY_BYTES {
            return Err(ReferenceRegistryError::ResourceExceeded {
                resource: ReferenceRegistryResource::CanonicalIdentityBytes,
                limit: MAX_REFERENCE_REGISTRY_IDENTITY_BYTES,
                actual: retained_bytes,
            });
        }
        retained.push(value);
    }
    Ok(retained)
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

/// Governed resource in one host reference value or evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceResource {
    /// Bytes in one exact dense element.
    ElementBytes,
    /// Logical elements in one dense tensor.
    TensorElements,
    /// Aggregate exact payload bytes in one tensor or output set.
    TensorBytes,
    /// Direct components in one compound tensor.
    Components,
    /// Recursive compound-tensor depth.
    ComponentDepth,
    /// Aggregate retained payload bytes across one evaluation.
    EvaluationBytes,
    /// Aggregate logical and component tensor elements across one evaluation.
    EvaluationElements,
    /// Aggregate recursive compound components across one evaluation.
    EvaluationComponents,
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
    /// An input resolved type disagreed with its verified declaration.
    InputType {
        /// Stable input key.
        key: InputKey,
        /// Declared resolved type.
        expected: Arc<ResolvedValueType>,
        /// Supplied resolved type.
        actual: Arc<ResolvedValueType>,
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
    /// Exact floating-point construction supplied no bits.
    EmptyFloatBits,
    /// A bounded reference resource exceeded its active limit.
    ResourceExceeded {
        /// Bounded resource.
        resource: ReferenceResource,
        /// Active limit.
        limit: usize,
        /// First rejected size.
        actual: usize,
    },
    /// A compound tensor repeated one schema-local role.
    DuplicateComponentRole {
        /// Repeated role.
        role: ReferenceComponentRole,
    },
    /// Reference registry construction failed while forming an exact signature.
    ReferenceRegistry(Arc<ReferenceRegistryError>),
    /// The frozen registry has no executable oracle for an exact semantic signature.
    MissingCapability {
        /// Semantically valid operation lacking an oracle.
        operation: OpKey,
        /// Exact operand/result signature lacking an oracle.
        signature: Arc<ReferenceSignature>,
    },
    /// No validator exists for one exact resolved reference value type.
    MissingValueCapability {
        /// Unsupported resolved type.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// Program semantic authority could not be projected for an operation.
    SemanticAuthority {
        /// Operation whose authority projection failed.
        operation: OpKey,
        /// Typed semantic registry cause.
        source: Arc<RegistryError>,
    },
    /// Program semantic authority could not be projected for a value type.
    SemanticValueAuthority {
        /// Resolved value type whose projection failed.
        resolved_type: Arc<ResolvedValueType>,
        /// Typed semantic registry cause.
        source: Arc<RegistryError>,
    },
    /// An operation capability was built for different reached semantic authority.
    CapabilityAuthorityMismatch {
        /// Operation being resolved.
        operation: OpKey,
        /// Reference provider whose claim did not match.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting reference implementation revision.
        capability_revision: ReferenceCapabilityRevision,
    },
    /// A value validator was built for different reached semantic authority.
    ValueCapabilityAuthorityMismatch {
        /// Value type being validated.
        resolved_type: Arc<ResolvedValueType>,
        /// Reference provider whose claim did not match.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting validator revision.
        capability_revision: ReferenceCapabilityRevision,
    },
    /// A selected value validator rejected a tensor representation.
    Value {
        /// Exact resolved type.
        resolved_type: Arc<ResolvedValueType>,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting validator revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Typed validation cause.
        source: ReferenceValueError,
    },
    /// A resolved reference capability rejected execution.
    Operation {
        /// Operation whose capability failed.
        operation: OpKey,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting implementation revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Typed implementation failure.
        source: ReferenceOperationError,
    },
    /// A provider produced a result with the wrong shape.
    ResultShape {
        /// Operation whose result failed validation.
        operation: OpKey,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting implementation revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Ordered result index.
        result_index: usize,
        /// Declared shape.
        expected: Arc<Shape>,
        /// Produced shape.
        actual: Arc<Shape>,
    },
    /// A provider produced a result with the wrong resolved type.
    ResultType {
        /// Operation whose result failed validation.
        operation: OpKey,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting implementation revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Ordered result index.
        result_index: usize,
        /// Declared type.
        expected: Arc<ResolvedValueType>,
        /// Produced type.
        actual: Arc<ResolvedValueType>,
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
            Self::InputType { key, .. } => write!(
                formatter,
                "input {:?} has the wrong resolved reference type",
                key.as_str()
            ),
            Self::ElementCount { expected, actual } => {
                write!(
                    formatter,
                    "tensor has {actual} elements, expected {expected}"
                )
            }
            Self::ShapeTooLarge => formatter.write_str("tensor shape exceeds host limits"),
            Self::EmptyFloatBits => {
                formatter.write_str("exact floating-point reference bits are empty")
            }
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "reference resource {resource:?} has size {actual}, exceeding {limit}"
            ),
            Self::DuplicateComponentRole { role } => {
                write!(
                    formatter,
                    "duplicate reference component role {}",
                    role.get()
                )
            }
            Self::ReferenceRegistry(source) => {
                write!(formatter, "reference registry failure: {source}")
            }
            Self::MalformedProgram => formatter.write_str("verified semantic program is malformed"),
            _ => self.fmt_capability_error(formatter),
        }
    }
}

impl EvaluationError {
    fn fmt_capability_error(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCapability { operation, .. } => write!(
                formatter,
                "no reference capability for semantic operation {operation} and exact resolved signature"
            ),
            Self::MissingValueCapability { .. } => {
                formatter.write_str("no reference value validator for exact resolved type")
            }
            Self::SemanticAuthority { operation, source } => write!(
                formatter,
                "semantic authority for {operation} could not be projected: {source}"
            ),
            Self::SemanticValueAuthority { source, .. } => write!(
                formatter,
                "semantic value authority could not be projected: {source}"
            ),
            Self::CapabilityAuthorityMismatch {
                operation,
                provider,
                capability_revision,
            } => write!(
                formatter,
                "reference provider {provider} capability revision {} does not implement reached authority for {operation}",
                capability_revision.get()
            ),
            Self::ValueCapabilityAuthorityMismatch {
                provider,
                capability_revision,
                ..
            } => write!(
                formatter,
                "reference provider {provider} validator revision {} does not implement reached value authority",
                capability_revision.get()
            ),
            Self::Value {
                provider,
                capability_revision,
                source,
                ..
            } => write!(
                formatter,
                "reference value validator revision {} from {provider} failed: {source}",
                capability_revision.get()
            ),
            Self::Operation {
                operation,
                provider,
                capability_revision,
                source,
            } => write!(
                formatter,
                "reference capability revision {} from {provider} for {operation} failed: {source}",
                capability_revision.get()
            ),
            Self::ResultShape {
                operation,
                provider,
                result_index,
                ..
            } => write!(
                formatter,
                "reference provider {provider} produced invalid shape for result {result_index} of {operation}"
            ),
            Self::ResultType {
                operation,
                provider,
                result_index,
                ..
            } => write!(
                formatter,
                "reference provider {provider} produced invalid type for result {result_index} of {operation}"
            ),
            _ => unreachable!("only capability errors use this formatter"),
        }
    }
}

impl Error for EvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReferenceRegistry(source) => Some(source),
            Self::SemanticAuthority { source, .. }
            | Self::SemanticValueAuthority { source, .. } => Some(source.as_ref()),
            Self::Value { source, .. } => Some(source),
            Self::Operation { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiler_ir::semantic::{
        AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, F32, F32Add,
        F32Constant, F32Multiply, InputKey, NormativeDefinitionRef, OperationArity,
        OperationAttributeSchema, OperationConformance, OperationDefinition,
        OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferencer,
        OperationSchema, OutputKey, SemanticProgramBuilder, SemanticRegistryBuilder,
        SemanticRegistryProvider, SemanticRegistryRegistrar, StrictSerialF32Sum,
        TypeDefinitionFacts, Value, ValueTypeDefinition, ValueTypeDefinitionKey,
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

    fn f32_tensor(shape: Shape, values: Vec<f32>) -> Tensor {
        Tensor::dense(
            F32::resolved_type(),
            shape,
            values
                .into_iter()
                .map(f32_element)
                .collect::<Result<_, _>>()
                .unwrap(),
        )
        .unwrap()
    }

    fn f32_values(tensor: &Tensor) -> Vec<f32> {
        f32_elements(tensor)
            .unwrap()
            .iter()
            .map(decode_f32)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn f32_bits(tensor: &Tensor) -> Vec<u32> {
        f32_values(tensor).into_iter().map(f32::to_bits).collect()
    }

    fn reference_builder_for(
        semantic_registry: FrozenSemanticRegistry,
    ) -> ReferenceRegistryBuilder {
        let mut builder = ReferenceRegistryBuilder::new(semantic_registry);
        builder
            .register_provider(&StandardReferenceProvider)
            .unwrap();
        builder
    }

    fn external_semantics() -> FrozenSemanticRegistry {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics
            .register_provider(&ExternalSemanticProvider)
            .unwrap();
        semantics.freeze().unwrap()
    }

    fn external_identity_program(semantic_registry: FrozenSemanticRegistry) -> SemanticProgram {
        let mut graph = SemanticProgramBuilder::try_new(semantic_registry).unwrap();
        let input = graph
            .input_resolved(
                InputKey::new("x").unwrap(),
                Shape::from_dims([2]),
                F32::resolved_type(),
            )
            .unwrap();
        let result = graph
            .apply(
                external_identity_op(),
                OperationAttributes::empty(),
                &[input],
            )
            .unwrap();
        graph
            .output_resolved(OutputKey::new("result").unwrap(), result[0])
            .unwrap();
        graph.build().unwrap()
    }

    fn external_identity_op() -> OpKey {
        OpKey::new("test", "reference-identity", 1).unwrap()
    }

    struct IdentitySemantic;
    impl OperationInferencer for IdentitySemantic {
        fn infer(
            &self,
            request: tiler_ir::semantic::OperationInferenceRequest<'_>,
            outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            outputs.try_push(request.operands()[0].clone())
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

    struct ChangedExternalSemanticProvider;
    impl SemanticRegistryProvider for ChangedExternalSemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "reference-semantics", 2).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), tiler_ir::semantic::RegistryError> {
            registrar.register_operation(OperationDefinition::new(
                external_identity_op(),
                OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), [])
                    .unwrap(),
                NormativeDefinitionRef::new("test reference identity changed v2")?,
                OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
                OperationConformance::new(
                    CanonicalValue::utf8("test.reference-identity.v2").unwrap(),
                ),
                OperationEffect::Pure,
                Arc::new(IdentitySemantic),
            ))
        }
    }

    fn external_u8_type() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("test", "u8", 1).unwrap())
    }

    struct ExternalU8SemanticProvider;
    impl SemanticRegistryProvider for ExternalU8SemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "u8-semantics", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), tiler_ir::semantic::RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "u8", 1).unwrap()),
                NormativeDefinitionRef::new("test unsigned byte v1")?,
                TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            ))
        }
    }

    fn compound_limit_type() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("test", "compound-limit", 1).unwrap())
    }

    struct CompoundLimitSemanticProvider;
    impl SemanticRegistryProvider for CompoundLimitSemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "compound-limit-semantics", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), tiler_ir::semantic::RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "compound-limit", 1).unwrap()),
                NormativeDefinitionRef::new("test compound limit v1")?,
                TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            ))
        }
    }

    fn attributed_identity_op() -> OpKey {
        OpKey::new("test", "attributed-reference-identity", 1).unwrap()
    }

    struct AttributeTypeProvider {
        provider_revision: u32,
        definition_revision: u32,
    }
    impl SemanticRegistryProvider for AttributeTypeProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "attribute-type-semantics", self.provider_revision)
                .unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), tiler_ir::semantic::RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "attribute-type", 1).unwrap()),
                NormativeDefinitionRef::new(format!(
                    "test attribute type v{}",
                    self.definition_revision
                ))?,
                TypeDefinitionFacts::new(CanonicalValue::unsigned_u32(self.definition_revision)),
            ))
        }
    }

    fn attribute_type() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("test", "attribute-type", 1).unwrap())
    }

    struct AttributedIdentitySemanticProvider;
    impl SemanticRegistryProvider for AttributedIdentitySemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "attributed-identity-semantics", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), tiler_ir::semantic::RegistryError> {
            registrar.register_operation(OperationDefinition::new(
                attributed_identity_op(),
                OperationSchema::new(
                    OperationArity::exact(1),
                    OperationArity::exact(1),
                    [OperationAttributeSchema::required(
                        AttributeFieldId::new(1),
                        CanonicalValueKind::Type,
                    )],
                )
                .unwrap(),
                NormativeDefinitionRef::new("test attributed reference identity v1")?,
                OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
                OperationConformance::new(
                    CanonicalValue::utf8("test.attributed-reference-identity.v1").unwrap(),
                ),
                OperationEffect::Pure,
                Arc::new(IdentitySemantic),
            ))
        }
    }

    struct AttributedIdentityReferenceProvider;
    impl ReferenceRegistryProvider for AttributedIdentityReferenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "attributed-reference-capability", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut ReferenceRegistryRegistrar<'_>,
        ) -> Result<(), ReferenceRegistryError> {
            registrar.register(
                attributed_identity_op(),
                ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
                ReferenceCapabilityRevision::new(7)?,
                Arc::new(IdentityReference),
            )
        }
    }

    fn attributed_semantics(
        attribute_provider_revision: u32,
        attribute_definition_revision: u32,
    ) -> FrozenSemanticRegistry {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder
            .register_provider(&AttributeTypeProvider {
                provider_revision: attribute_provider_revision,
                definition_revision: attribute_definition_revision,
            })
            .unwrap();
        builder
            .register_provider(&AttributedIdentitySemanticProvider)
            .unwrap();
        builder.freeze().unwrap()
    }

    fn attributed_program(semantics: FrozenSemanticRegistry) -> SemanticProgram {
        let mut graph = SemanticProgramBuilder::try_new(semantics).unwrap();
        let input = graph
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let attributes = OperationAttributes::new([CanonicalField::new(
            AttributeFieldId::new(1),
            CanonicalValue::value_type(attribute_type()),
        )])
        .unwrap();
        let result = graph
            .apply(attributed_identity_op(), attributes, &[input.erase()])
            .unwrap();
        graph
            .output_resolved(OutputKey::new("result").unwrap(), result[0])
            .unwrap();
        graph.build().unwrap()
    }

    struct IdentityReference;
    impl ReferenceOperation for IdentityReference {
        fn evaluate(
            &self,
            request: ReferenceEvaluationRequest<'_>,
            outputs: &mut ReferenceOutputs,
        ) -> Result<(), ReferenceOperationError> {
            outputs.push(request.operands()[0].clone())
        }
    }

    #[derive(Clone, Copy)]
    enum MalformedReferenceResult {
        CallbackFailure,
        WrongArity,
        WrongShape,
        WrongType,
    }

    struct MalformedReference {
        result: MalformedReferenceResult,
    }

    impl ReferenceOperation for MalformedReference {
        fn evaluate(
            &self,
            _: ReferenceEvaluationRequest<'_>,
            outputs: &mut ReferenceOutputs,
        ) -> Result<(), ReferenceOperationError> {
            match self.result {
                MalformedReferenceResult::CallbackFailure => {
                    Err(ReferenceOperationError::InvalidApplication)
                }
                MalformedReferenceResult::WrongArity => Ok(()),
                MalformedReferenceResult::WrongShape => {
                    outputs.push(f32_tensor(Shape::new([]), vec![0.0]))
                }
                MalformedReferenceResult::WrongType => outputs.push(
                    Tensor::dense(
                        external_u8_type(),
                        Shape::from_dims([2]),
                        vec![
                            ReferenceElement::new([1]).unwrap(),
                            ReferenceElement::new([2]).unwrap(),
                        ],
                    )
                    .unwrap(),
                ),
            }
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
                ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
                ReferenceCapabilityRevision::new(self.capability_revision)?,
                Arc::new(IdentityReference),
            )
        }
    }

    struct ExternalU8Validator;
    impl ReferenceValueValidator for ExternalU8Validator {
        fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
            let TensorPayloadView::Dense(elements) = tensor.payload() else {
                return Err(ReferenceValueError::InvalidRepresentation);
            };
            if tensor.resolved_type() != &external_u8_type()
                || elements.iter().any(|element| element.as_bytes().len() != 1)
            {
                return Err(ReferenceValueError::InvalidRepresentation);
            }
            Ok(())
        }
    }

    struct ExternalU8ReferenceProvider;
    impl ReferenceRegistryProvider for ExternalU8ReferenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "u8-reference", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut ReferenceRegistryRegistrar<'_>,
        ) -> Result<(), ReferenceRegistryError> {
            registrar.register_value_type(
                external_u8_type(),
                ReferenceCapabilityRevision::new(1)?,
                Arc::new(ExternalU8Validator),
            )
        }
    }

    struct CompoundLimitValidator;
    impl ReferenceValueValidator for CompoundLimitValidator {
        fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
            if tensor.resolved_type() == &compound_limit_type()
                && matches!(tensor.payload(), TensorPayloadView::Compound([]))
            {
                Ok(())
            } else {
                Err(ReferenceValueError::InvalidRepresentation)
            }
        }
    }

    struct CompoundLimitReferenceProvider;
    impl ReferenceRegistryProvider for CompoundLimitReferenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "compound-limit-reference", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut ReferenceRegistryRegistrar<'_>,
        ) -> Result<(), ReferenceRegistryError> {
            registrar.register_value_type(
                compound_limit_type(),
                ReferenceCapabilityRevision::new(1)?,
                Arc::new(CompoundLimitValidator),
            )
        }
    }

    struct IgnoredDuplicateReferenceProvider;
    impl ReferenceRegistryProvider for IgnoredDuplicateReferenceProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "ignored-reference-duplicate", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut ReferenceRegistryRegistrar<'_>,
        ) -> Result<(), ReferenceRegistryError> {
            let signature =
                ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?;
            registrar.register(
                external_identity_op(),
                signature.clone(),
                ReferenceCapabilityRevision::new(1)?,
                Arc::new(IdentityReference),
            )?;
            let _ = registrar.register(
                external_identity_op(),
                signature,
                ReferenceCapabilityRevision::new(1)?,
                Arc::new(IdentityReference),
            );
            Ok(())
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
                ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
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
        let input = f32_tensor(Shape::from_dims([2, 3]), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let outputs = evaluate_one(&program, &input);
        assert_eq!(f32_values(&outputs[0]), [3.0, 5.0, 7.0, 9.0, 11.0, 13.0]);
        assert_eq!(outputs[1].shape(), &Shape::from_dims([2]));
        assert_eq!(f32_values(&outputs[1]), [15.0, 33.0]);
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
        let input = f32_tensor(
            Shape::from_dims([2, 2, 2]),
            vec![1.0e20, 1.0, 7.0, 8.0, -1.0e20, 3.0, 9.0, 10.0],
        );
        let outputs = evaluate_one(&program, &input);
        assert_eq!(
            f32_bits(&outputs[0]),
            [3.0_f32.to_bits(), 34.0_f32.to_bits()]
        );
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
        let input = f32_tensor(Shape::from_dims([3, 1]), vec![-0.0, f32::INFINITY, nan]);
        let output = evaluate_one(&program, &input);
        let bits = f32_bits(&output[0]);
        assert_eq!(bits[0], (-0.0_f32).to_bits());
        assert_eq!(bits[1], f32::INFINITY.to_bits());
        assert_eq!(bits[2], CANONICAL_F32_ARITHMETIC_NAN_BITS);
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
        let input = f32_tensor(Shape::from_dims([1]), vec![f32::from_bits(0x3f80_0001)]);
        let output = evaluate_one(&program, &input);
        assert_eq!(f32_bits(&output[0])[0], 0.0_f32.to_bits());
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
        let input = f32_tensor(Shape::from_dims([2, 0]), vec![]);
        let outputs = evaluate_one(&program, &input);
        assert_eq!(f32_values(&outputs[1]).len(), 2);
        assert!(
            f32_values(&outputs[1])
                .iter()
                .all(|value| value.to_bits() == 0.0_f32.to_bits())
        );

        let program = graph(Shape::from_dims([0, 2]), &[1]);
        let input = f32_tensor(Shape::from_dims([0, 2]), vec![]);
        let outputs = evaluate_one(&program, &input);
        assert!(f32_values(&outputs[1]).is_empty());
    }

    #[test]
    fn bindings_validate_ordered_keys_shapes_and_payloads() {
        assert_eq!(
            Tensor::dense(
                F32::resolved_type(),
                Shape::from_dims([2]),
                vec![f32_element(1.0).unwrap()],
            )
            .unwrap_err(),
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
        let left_tensor = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
        let right_tensor = f32_tensor(Shape::from_dims([2]), vec![3.0, 4.0]);
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
        let wrong = f32_tensor(Shape::from_dims([1]), vec![1.0]);
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
        assert_eq!(f32_bits(&output[0])[0], payload);
        assert_eq!(f32_bits(&output[1])[0], CANONICAL_F32_ARITHMETIC_NAN_BITS);
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

        assert_eq!(f32_bits(&outputs[0])[0], 0x0000_0001);
        assert_eq!(f32_bits(&outputs[1])[0], 0x0040_0000);
        assert_eq!(f32_bits(&outputs[2])[0], f32::INFINITY.to_bits());
        assert_eq!(f32_bits(&outputs[3])[0], 0x8000_0000);
        assert_eq!(f32_bits(&outputs[4])[0], CANONICAL_F32_ARITHMETIC_NAN_BITS);
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
        assert_eq!(f32_values(&outputs[0]), [7.0]);
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
        let tensor = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
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

        let mut references = reference_builder_for(program.semantic_registry().clone());
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
        let tensor = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
        let bindings = [InputBinding::new(&key, &tensor)];

        for result in [
            MalformedReferenceResult::CallbackFailure,
            MalformedReferenceResult::WrongArity,
            MalformedReferenceResult::WrongShape,
            MalformedReferenceResult::WrongType,
        ] {
            let mut references = reference_builder_for(program.semantic_registry().clone());
            references
                .register_provider(&MalformedReferenceProvider { result })
                .unwrap();
            let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
            let error = evaluator.evaluate(&program, &bindings).unwrap_err();
            match result {
                MalformedReferenceResult::CallbackFailure => assert!(matches!(
                    error,
                    EvaluationError::Operation {
                        provider,
                        capability_revision,
                        source: ReferenceOperationError::InvalidApplication,
                        ..
                    } if provider.name() == "malformed-reference-capability"
                        && capability_revision.get() == 1
                )),
                MalformedReferenceResult::WrongArity => assert!(matches!(
                    error,
                    EvaluationError::Operation {
                        provider,
                        capability_revision,
                        source: ReferenceOperationError::ResultCount { .. },
                        ..
                    } if provider.name() == "malformed-reference-capability"
                        && capability_revision.get() == 1
                )),
                MalformedReferenceResult::WrongShape => assert!(matches!(
                    error,
                    EvaluationError::ResultShape {
                        provider,
                        capability_revision,
                        ..
                    } if provider.name() == "malformed-reference-capability"
                        && capability_revision.get() == 1
                )),
                MalformedReferenceResult::WrongType => assert!(matches!(
                    error,
                    EvaluationError::ResultType {
                        provider,
                        capability_revision,
                        ..
                    } if provider.name() == "malformed-reference-capability"
                        && capability_revision.get() == 1
                )),
            }
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

        let semantic_registry = external_semantics();
        let baseline = reference_builder_for(semantic_registry.clone())
            .freeze()
            .unwrap();
        let with_revision = |capability_revision| {
            let mut builder = reference_builder_for(semantic_registry.clone());
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
            baseline.canonical_identity()
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
        let semantic_registry = external_semantics();
        let mut builder = reference_builder_for(semantic_registry.clone());
        builder.register_provider(&provider).unwrap();
        assert!(matches!(
            builder.register_provider(&provider),
            Err(ReferenceRegistryError::DuplicateCapability { operation, .. })
                if operation == external_identity_op()
        ));
        let after_rejection = builder.freeze().unwrap();

        let mut expected = reference_builder_for(semantic_registry);
        expected.register_provider(&provider).unwrap();
        let expected = expected.freeze().unwrap();
        assert_eq!(
            after_rejection.canonical_identity(),
            expected.canonical_identity()
        );
    }

    #[test]
    fn non_f32_nominal_values_use_the_same_exact_tensor_boundary() {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics
            .register_provider(&ExternalU8SemanticProvider)
            .unwrap();
        let semantics = semantics.freeze().unwrap();
        let mut graph = SemanticProgramBuilder::try_new(semantics.clone()).unwrap();
        let input = graph
            .input_resolved(
                InputKey::new("bytes").unwrap(),
                Shape::from_dims([3]),
                external_u8_type(),
            )
            .unwrap();
        graph
            .output_resolved(OutputKey::new("bytes").unwrap(), input)
            .unwrap();
        let program = graph.build().unwrap();
        let tensor = Tensor::dense(
            external_u8_type(),
            Shape::from_dims([3]),
            [1_u8, 2, 255]
                .map(|value| ReferenceElement::new([value]).unwrap())
                .into(),
        )
        .unwrap();
        let mut references = reference_builder_for(semantics);
        references
            .register_provider(&ExternalU8ReferenceProvider)
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        let key = InputKey::new("bytes").unwrap();
        assert_eq!(
            evaluator
                .evaluate(&program, &[InputBinding::new(&key, &tensor)])
                .unwrap(),
            [tensor]
        );
    }

    #[test]
    fn value_validator_failure_retains_exact_implementation_attribution() {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics
            .register_provider(&ExternalU8SemanticProvider)
            .unwrap();
        let semantics = semantics.freeze().unwrap();
        let mut graph = SemanticProgramBuilder::try_new(semantics.clone()).unwrap();
        let input = graph
            .input_resolved(
                InputKey::new("bytes").unwrap(),
                Shape::from_dims([1]),
                external_u8_type(),
            )
            .unwrap();
        graph
            .output_resolved(OutputKey::new("bytes").unwrap(), input)
            .unwrap();
        let program = graph.build().unwrap();
        let invalid = Tensor::dense(
            external_u8_type(),
            Shape::from_dims([1]),
            vec![ReferenceElement::new([1, 2]).unwrap()],
        )
        .unwrap();
        let mut references = reference_builder_for(semantics);
        references
            .register_provider(&ExternalU8ReferenceProvider)
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        let key = InputKey::new("bytes").unwrap();
        assert!(matches!(
            evaluator.evaluate(&program, &[InputBinding::new(&key, &invalid)]),
            Err(EvaluationError::Value {
                provider,
                capability_revision,
                source: ReferenceValueError::InvalidRepresentation,
                ..
            }) if provider.name() == "u8-reference" && capability_revision.get() == 1
        ));
    }

    #[test]
    fn capability_authority_rejects_changed_meaning_but_not_unrelated_snapshot_entries() {
        let baseline_semantics = external_semantics();
        let mut references = reference_builder_for(baseline_semantics.clone());
        references
            .register_provider(&ExternalReferenceProvider {
                capability_revision: 1,
            })
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        let input = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
        let key = InputKey::new("x").unwrap();

        let mut changed = SemanticRegistryBuilder::standard().unwrap();
        changed
            .register_provider(&ChangedExternalSemanticProvider)
            .unwrap();
        let changed_program = external_identity_program(changed.freeze().unwrap());
        assert!(matches!(
            evaluator.evaluate(&changed_program, &[InputBinding::new(&key, &input)]),
            Err(EvaluationError::CapabilityAuthorityMismatch { operation, .. })
                if operation == external_identity_op()
        ));

        let mut extended = SemanticRegistryBuilder::standard().unwrap();
        extended
            .register_provider(&ExternalSemanticProvider)
            .unwrap();
        extended
            .register_provider(&ExternalU8SemanticProvider)
            .unwrap();
        let extended_program = external_identity_program(extended.freeze().unwrap());
        assert_eq!(
            evaluator
                .evaluate(&extended_program, &[InputBinding::new(&key, &input)])
                .unwrap(),
            [input]
        );
    }

    #[test]
    fn occurrence_authority_follows_attribute_types_and_admission_providers() {
        let baseline_semantics = attributed_semantics(1, 1);
        let mut references = reference_builder_for(baseline_semantics.clone());
        references
            .register_provider(&AttributedIdentityReferenceProvider)
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        let input = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
        let key = InputKey::new("x").unwrap();

        for changed in [attributed_semantics(1, 2), attributed_semantics(2, 1)] {
            let changed = attributed_program(changed);
            assert!(matches!(
                evaluator.evaluate(&changed, &[InputBinding::new(&key, &input)]),
                Err(EvaluationError::CapabilityAuthorityMismatch {
                    operation,
                    provider,
                    capability_revision,
                }) if operation == attributed_identity_op()
                    && provider.name() == "attributed-reference-capability"
                    && capability_revision.get() == 7
            ));
        }

        let mut extended = SemanticRegistryBuilder::standard().unwrap();
        extended
            .register_provider(&AttributeTypeProvider {
                provider_revision: 1,
                definition_revision: 1,
            })
            .unwrap();
        extended
            .register_provider(&AttributedIdentitySemanticProvider)
            .unwrap();
        extended
            .register_provider(&ExternalU8SemanticProvider)
            .unwrap();
        let extended = attributed_program(extended.freeze().unwrap());
        assert_eq!(
            evaluator
                .evaluate(&extended, &[InputBinding::new(&key, &input)])
                .unwrap(),
            [input]
        );
    }

    #[test]
    fn ignored_registration_failure_poisoned_the_provider_batch() {
        let mut builder = reference_builder_for(external_semantics());
        assert!(matches!(
            builder.register_provider(&IgnoredDuplicateReferenceProvider),
            Err(ReferenceRegistryError::DuplicateCapability { operation, .. })
                if operation == external_identity_op()
        ));
        builder
            .register_provider(&ExternalReferenceProvider {
                capability_revision: 1,
            })
            .unwrap();
    }

    #[test]
    fn exact_tensor_equality_distinguishes_nan_payloads_and_signed_zero() {
        let tensor = |bits| {
            Tensor::dense(
                F32::resolved_type(),
                Shape::from_dims([1]),
                vec![
                    ReferenceElement::from_float_bits(
                        u32::to_be_bytes(bits),
                        FloatBitOrder::MostSignificantByteFirst,
                    )
                    .unwrap(),
                ],
            )
            .unwrap()
        };
        assert_eq!(tensor(0x7fc0_1234), tensor(0x7fc0_1234));
        assert_ne!(tensor(0x7fc0_1234), tensor(0x7fc0_5678));
        assert_ne!(tensor(0x0000_0000), tensor(0x8000_0000));
    }

    #[test]
    fn float_bit_order_is_explicit_and_normalizes_to_canonical_bytes() {
        let canonical = ReferenceElement::from_float_bits(
            [0x3f, 0x80, 0x00, 0x00],
            FloatBitOrder::MostSignificantByteFirst,
        )
        .unwrap();
        let little = ReferenceElement::from_float_bits(
            [0x00, 0x00, 0x80, 0x3f],
            FloatBitOrder::LeastSignificantByteFirst,
        )
        .unwrap();
        assert_eq!(canonical, little);
        assert_eq!(canonical.as_bytes(), [0x3f, 0x80, 0x00, 0x00]);
        assert_eq!(
            ReferenceElement::from_float_bits([], FloatBitOrder::MostSignificantByteFirst),
            Err(EvaluationError::EmptyFloatBits)
        );
        let oversized = vec![0_u8; MAX_REFERENCE_ELEMENT_BYTES + 1];
        assert!(matches!(
            ReferenceElement::from_float_bits(
                &oversized,
                FloatBitOrder::MostSignificantByteFirst,
            ),
            Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::ElementBytes,
                limit: MAX_REFERENCE_ELEMENT_BYTES,
                actual,
            }) if actual == MAX_REFERENCE_ELEMENT_BYTES + 1
        ));
    }

    #[test]
    fn compound_values_preserve_stable_role_tensors_without_any_downcasts() {
        let codes = Tensor::dense(
            external_u8_type(),
            Shape::from_dims([2]),
            vec![
                ReferenceElement::new([1]).unwrap(),
                ReferenceElement::new([2]).unwrap(),
            ],
        )
        .unwrap();
        let scale = f32_tensor(Shape::new([]), vec![0.5]);
        let compound_type =
            ResolvedValueType::nominal(TypeKey::new("test", "compound-quantized", 1).unwrap());
        let value = Tensor::compound(
            compound_type,
            Shape::from_dims([2]),
            vec![
                ReferenceComponent::new(ReferenceComponentRole::new(1), codes),
                ReferenceComponent::new(ReferenceComponentRole::new(2), scale),
            ],
        )
        .unwrap();
        let TensorPayloadView::Compound(components) = value.payload() else {
            panic!("expected compound payload")
        };
        assert_eq!(components[0].role(), ReferenceComponentRole::new(1));
        assert_eq!(components[1].tensor().shape(), &Shape::new([]));

        let duplicate = ReferenceComponent::new(
            ReferenceComponentRole::new(1),
            f32_tensor(Shape::new([]), vec![1.0]),
        );
        assert!(matches!(
            Tensor::compound(
                ResolvedValueType::nominal(
                    TypeKey::new("test", "compound-quantized", 1).unwrap()
                ),
                Shape::from_dims([2]),
                vec![components[0].clone(), duplicate],
            ),
            Err(EvaluationError::DuplicateComponentRole { role })
                if role == ReferenceComponentRole::new(1)
        ));
    }

    #[test]
    fn compound_and_evaluation_resources_are_bounded_in_aggregate() {
        let compound_type = compound_limit_type();
        let components = |start: u32| {
            (0..(MAX_REFERENCE_COMPONENTS / 2))
                .map(|offset| {
                    ReferenceComponent::new(
                        ReferenceComponentRole::new(start + u32::try_from(offset).unwrap()),
                        Tensor::dense(external_u8_type(), Shape::from_dims([0]), Vec::new())
                            .unwrap(),
                    )
                })
                .collect()
        };
        let left = Tensor::compound(compound_type.clone(), Shape::new([]), components(0)).unwrap();
        let right = Tensor::compound(
            compound_type.clone(),
            Shape::new([]),
            components(u32::try_from(MAX_REFERENCE_COMPONENTS / 2).unwrap()),
        )
        .unwrap();
        assert!(matches!(
            Tensor::compound(
                compound_type.clone(),
                Shape::new([]),
                vec![
                    ReferenceComponent::new(ReferenceComponentRole::new(1), left),
                    ReferenceComponent::new(ReferenceComponentRole::new(2), right),
                ],
            ),
            Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::Components,
                limit: MAX_REFERENCE_COMPONENTS,
                actual,
            }) if actual == MAX_REFERENCE_COMPONENTS + 2
        ));

        let at_limit = Tensor::compound(
            compound_type.clone(),
            Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
            Vec::new(),
        )
        .unwrap();
        let one_more = Tensor::compound(compound_type, Shape::from_dims([1]), Vec::new()).unwrap();
        let mut retained = EvaluationRetention::default();
        reserve_evaluation_work(&mut retained, &at_limit).unwrap();
        assert!(matches!(
            reserve_evaluation_work(&mut retained, &one_more),
            Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::EvaluationElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual,
            }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
        ));

        let mut outputs = ReferenceOutputs::new(
            1,
            EvaluationRetention {
                work: ReferenceWork {
                    elements: MAX_REFERENCE_TENSOR_ELEMENTS,
                    ..ReferenceWork::default()
                },
                ..EvaluationRetention::default()
            },
        );
        assert!(matches!(
            outputs.push(one_more),
            Err(ReferenceOperationError::OutputElementsExceeded {
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual,
            }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
        ));
        assert!(outputs.values.is_empty());
    }

    #[test]
    fn compound_root_elements_participate_in_the_aggregate_bound() {
        let compound_type = compound_limit_type();
        assert!(matches!(
            Tensor::compound(
                compound_type.clone(),
                Shape::from_dims([
                    u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap() + 1
                ]),
                Vec::new(),
            ),
            Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual,
            }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
        ));
        let scalar_child = Tensor::dense(
            external_u8_type(),
            Shape::new([]),
            vec![ReferenceElement::new([1]).unwrap()],
        )
        .unwrap();
        assert!(matches!(
            Tensor::compound(
                compound_type.clone(),
                Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
                vec![ReferenceComponent::new(
                    ReferenceComponentRole::new(1),
                    scalar_child.clone(),
                )],
            ),
            Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual,
            }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
        ));
        Tensor::compound(
            compound_type,
            Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS - 1).unwrap()]),
            vec![ReferenceComponent::new(
                ReferenceComponentRole::new(1),
                scalar_child,
            )],
        )
        .unwrap();
    }

    #[test]
    fn repeated_outputs_share_one_governed_tensor_allocation() {
        let mut semantics = SemanticRegistryBuilder::standard().unwrap();
        semantics
            .register_provider(&CompoundLimitSemanticProvider)
            .unwrap();
        let semantics = semantics.freeze().unwrap();
        let mut graph = SemanticProgramBuilder::try_new(semantics.clone()).unwrap();
        let input = graph
            .input_resolved(
                InputKey::new("value").unwrap(),
                Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
                compound_limit_type(),
            )
            .unwrap();
        graph
            .output_resolved(OutputKey::new("first").unwrap(), input)
            .unwrap();
        graph
            .output_resolved(OutputKey::new("second").unwrap(), input)
            .unwrap();
        let program = graph.build().unwrap();
        let input = Tensor::compound(
            compound_limit_type(),
            Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
            Vec::new(),
        )
        .unwrap();
        let mut references = reference_builder_for(semantics);
        references
            .register_provider(&CompoundLimitReferenceProvider)
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        let key = InputKey::new("value").unwrap();
        let outputs = evaluator
            .evaluate(&program, &[InputBinding::new(&key, &input)])
            .unwrap();
        assert_eq!(outputs, [input.clone(), input.clone()]);
        assert_eq!(outputs[0].storage_id(), outputs[1].storage_id());
        assert_eq!(outputs[0].storage_id(), input.storage_id());
    }

    #[test]
    fn registry_identity_budget_is_exact_at_boundary() {
        let builder = ReferenceRegistryBuilder::standard().unwrap();
        let exact_len = builder.canonical_bytes;
        let frozen = builder.freeze().unwrap();
        assert_eq!(frozen.canonical_identity().as_bytes().len(), exact_len);

        let semantics = FrozenSemanticRegistry::standard().unwrap();
        let provider = ProviderIdentity::new("test", "budget-boundary", 1).unwrap();
        for (existing, added, succeeds) in [
            (MAX_REFERENCE_REGISTRY_IDENTITY_BYTES, 0, true),
            (MAX_REFERENCE_REGISTRY_IDENTITY_BYTES, 1, false),
        ] {
            let mut batch = ReferenceRegistrationBatch::default();
            let mut registrar = ReferenceRegistryRegistrar {
                batch: &mut batch,
                semantic_registry: &semantics,
                provider: &provider,
                existing_capabilities: 0,
                existing_canonical_bytes: existing,
            };
            assert_eq!(registrar.reserve_canonical_bytes(added).is_ok(), succeeds);
        }
    }

    #[test]
    fn late_zero_shapes_are_accepted_before_overflow_prone_work() {
        let tensor = Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([0, u64::MAX, 2]),
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(tensor.payload(), TensorPayloadView::Dense([])));
        assert_eq!(row_major_strides(tensor.shape()).unwrap(), [0, 0, 0]);

        let output = strict_sum(&tensor, &[Axis::new(1), Axis::new(2)]).unwrap();
        assert_eq!(output.shape(), &Shape::from_dims([0]));
        assert!(matches!(output.payload(), TensorPayloadView::Dense([])));
    }

    #[test]
    fn empty_contributor_reduction_preflights_oversized_survivor() {
        let input = Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap() + 1, 0]),
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            strict_sum(&input, &[Axis::new(1)]),
            Err(ReferenceOperationError::OutputElementsExceeded {
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual,
            }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
        ));
    }

    #[test]
    fn large_empty_contributor_domains_are_iterated_without_coordinate_materialization() {
        let input = f32_tensor(Shape::from_dims([100_000, 0]), Vec::new());
        let output = strict_sum(&input, &[Axis::new(1)]).unwrap();
        assert_eq!(output.shape(), &Shape::from_dims([100_000]));
        assert!(
            f32_bits(&output)
                .into_iter()
                .all(|bits| bits == 0.0_f32.to_bits())
        );
    }

    #[test]
    fn maximum_rank_reduction_classifies_many_axes_linearly() {
        let rank = 4_096_usize;
        let input = f32_tensor(
            Shape::try_from_dims(std::iter::repeat_n(1, rank)).unwrap(),
            vec![1.0],
        );
        let axes: Vec<_> = (0..rank)
            .step_by(2)
            .map(|axis| Axis::new(u32::try_from(axis).unwrap()))
            .collect();
        let output = strict_sum(&input, &axes).unwrap();
        assert_eq!(output.shape().rank(), rank / 2);
        assert_eq!(f32_values(&output), [1.0]);
        assert_eq!(
            strict_sum(&input, &[Axis::new(0), Axis::new(0)]),
            Err(ReferenceOperationError::InvalidApplication)
        );
        assert_eq!(
            strict_sum(&input, &[Axis::new(u32::try_from(rank).unwrap())]),
            Err(ReferenceOperationError::InvalidApplication)
        );
    }
}
