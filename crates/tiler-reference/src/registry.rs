//! The reference capability registry and the vocabulary it dispatches on.
//!
//! A signature names an operation's operand and result types, a capability
//! binds one to an implementation, and freezing the registry fixes both plus
//! the value validators into a canonical identity.
//!
//! **This is the semantic registry, and it is deliberately not the scalar
//! one.** `oracle` owns scalar dispatch. Both resolve behaviour by key, which
//! is the resemblance; they govern different identities and different
//! extension obligations, so merging them would lose a distinction the
//! contracts rely on.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, OnceLock};

use tiler_ir::semantic::{
    FrozenSemanticRegistry, MAX_OPERATION_OPERANDS, MAX_OPERATION_RESULTS, OpKey,
    OperationAttributes, ProviderIdentity, ResolvedValueType, SemanticCapabilityAuthority,
};

use super::conformance::ReferenceNumericalConformance;
use super::error::{
    EvaluationError, ReferenceOperationError, ReferenceRegistryError, ReferenceRegistryResource,
    ReferenceValueError,
};
use super::evaluate::{EvaluationRetention, reserve_output_work};
use super::identity::{
    collect_signature_types, compute_reference_identity, reference_capability_identity_len,
    reference_identity_base_len, reference_value_identity_len,
};
use super::standard::StandardReferenceProvider;
use super::tensor::Tensor;
use super::{MAX_REFERENCE_CAPABILITIES, MAX_REFERENCE_REGISTRY_IDENTITY_BYTES};

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
    pub(crate) operands: &'a [&'a Tensor],
    pub(crate) attributes: &'a OperationAttributes,
    pub(crate) iteration_step_allowance: usize,
    pub(crate) conformance: ReferenceNumericalConformance,
}

impl fmt::Debug for ReferenceEvaluationRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReferenceEvaluationRequest")
            .field("operand_count", &self.operands.len())
            .field("attributes", &self.attributes)
            .field("iteration_step_allowance", &self.iteration_step_allowance)
            .field("conformance", &self.conformance)
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

    /// Returns the iteration steps this occurrence is authorized to walk.
    ///
    /// An implementation whose work is not answerable from its operand and result
    /// bounds must consult this and refuse above it under
    /// [`ReferenceOperationError::IterationStepsExceeded`]; the governed
    /// contraction is the one such family today, because its fold walks
    /// `output_count * contracted_count` steps. An implementation whose cost is
    /// linear in a bound the operands already passed has nothing to read here.
    ///
    /// This is the caller's stated authorization and never a per-walk budget: the
    /// window an implementation may walk in one pass is still bounded by the
    /// crate's own limit, and an occurrence over it is folded in several such
    /// windows.
    ///
    /// [`ReferenceOperationError::IterationStepsExceeded`]: crate::ReferenceOperationError::IterationStepsExceeded
    #[must_use]
    pub const fn iteration_step_allowance(self) -> usize {
        self.iteration_step_allowance
    }

    /// Returns the numerical contract this evaluation is performed under.
    ///
    /// **A capability that performs host binary32 arithmetic must consult this**,
    /// applying [`ReferenceNumericalConformance::apply_to_operand`] to each value
    /// entering an arithmetic operation and
    /// [`ReferenceNumericalConformance::apply_to_result`] to each value one
    /// produces. The semantic evaluator and the index-region oracle answer the
    /// same program, so one honouring the contract and the other ignoring it would
    /// disagree on exactly the values the contract exists to decide — and a
    /// capability that read nothing here would answer the strict reading whatever
    /// its caller declared, which is the silent single-value oracle
    /// [`ReferenceNumericalConformance::from_realization`] refuses to be.
    ///
    /// A capability that performs no host arithmetic has nothing to read here, and
    /// says so at its own definition rather than by omission: the two dimensions
    /// are functions on an arithmetic operand and on a newly produced arithmetic
    /// result, so a family that only transports, selects, or reproduces a bit
    /// pattern reaches neither site. That is the boundary this crate's arithmetic
    /// NaN canonicalization is already drawn at.
    #[must_use]
    pub const fn conformance(self) -> ReferenceNumericalConformance {
        self.conformance
    }
}

/// Host-owned bounded output writer for one reference callback.
///
/// A failed write poisons the writer. Catching or ignoring the returned error
/// cannot make a partial or over-limit result appear successful.
pub struct ReferenceOutputs {
    expected: usize,
    pub(crate) values: Vec<Tensor>,
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
    pub(crate) fn new(expected: usize, retention: EvaluationRetention) -> Self {
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

    pub(crate) fn finish(
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
pub(crate) struct ReferenceCapabilityKey {
    pub(crate) operation: OpKey,
    pub(crate) signature: ReferenceSignature,
}

#[derive(Clone)]
pub(crate) struct RegisteredReferenceCapability {
    pub(crate) provider: ProviderIdentity,
    pub(crate) revision: ReferenceCapabilityRevision,
    pub(crate) semantic_authority: SemanticCapabilityAuthority,
    pub(crate) implementation: Arc<dyn ReferenceOperation>,
}

#[derive(Clone)]
pub(crate) struct RegisteredReferenceValueValidator {
    pub(crate) provider: ProviderIdentity,
    pub(crate) revision: ReferenceCapabilityRevision,
    pub(crate) semantic_authority: SemanticCapabilityAuthority,
    pub(crate) implementation: Arc<dyn ReferenceValueValidator>,
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
pub(crate) struct ReferenceRegistrationBatch {
    capabilities: BTreeMap<ReferenceCapabilityKey, StagedReferenceCapability>,
    value_validators: BTreeMap<ResolvedValueType, StagedReferenceValueValidator>,
    failure: Option<ReferenceRegistryError>,
    canonical_bytes: usize,
}

/// Host-owned registration surface for one reference provider transaction.
pub struct ReferenceRegistryRegistrar<'a> {
    pub(crate) batch: &'a mut ReferenceRegistrationBatch,
    pub(crate) semantic_registry: &'a FrozenSemanticRegistry,
    pub(crate) provider: &'a ProviderIdentity,
    pub(crate) existing_capabilities: usize,
    pub(crate) existing_canonical_bytes: usize,
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

    pub(crate) fn reserve_canonical_bytes(
        &mut self,
        added: usize,
    ) -> Result<(), ReferenceRegistryError> {
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
    pub(crate) canonical_bytes: usize,
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

    pub(crate) fn resolve(
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

    pub(crate) fn validate_value(
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
pub struct CanonicalReferenceRegistryIdentity(pub(crate) Vec<u8>);

impl CanonicalReferenceRegistryIdentity {
    /// Returns canonical provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
