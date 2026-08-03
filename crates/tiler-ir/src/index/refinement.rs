//! Checked semantic-occurrence to index-region refinement receipts.
//!
//! A verified index region proves structural safety, but does not by itself say
//! which semantic occurrence it realizes. This module owns the dependency-neutral
//! verifier that checks that association and mints an opaque receipt. Provider
//! selection, capability attribution, search, and explanation remain compiler
//! concerns layered above this receipt.
//!
//! The public surface is a concrete alpha draft pending Tom's review. In
//! particular, callers cannot construct a receipt or its identity from bytes:
//! [`ResolvedIndexRealization::verify`] sees the complete semantic occurrence and
//! the actual [`VerifiedIndexRegion`], and [`ResolvedIndexRealization::complete`]
//! independently discharges every retained logical-index obligation before it
//! mints a receipt.

use core::fmt;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;

use crate::identity::{push_len, push_slice};
use crate::program::SemanticOccurrence;
use crate::semantic::{
    FrozenSemanticRegistry, OpKey, OperationAttributes, OperationEffect, ProviderIdentity,
    RegistryError, ResolvedValueType, SemanticCapabilityAuthority, SemanticGraphIdentity,
    SemanticProgram,
};
use crate::shape::Shape;

use super::{
    CanonicalIndexRegionIdentity, CanonicalScalarDefinitionProjection, FrozenScalarRegistry,
    IndexDomainSoundProof, IndexDomainUnknownReason, ScalarAuthorityEvidence, ScalarOpKey,
    ScalarRegistryError, TensorRole, UnknownIndexDomainPredicate, VerifiedIndexHandleError,
    VerifiedIndexRegion, VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
};

const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-receipt.v1\0";
const SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v1\0";
const AUTHORITY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-authority.v1\0";
const RESOLUTION_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-resolution.v1\0";
const PROOF_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-domain-proof.v1\0";
const VERIFIER_REGISTRY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-verifier-registry.v1\0";
const VERIFIER_CAPABILITY_IDENTITY_TAG: &[u8] =
    b"tiler.ir.index-realization-verifier-capability.v1\0";
const MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES: usize = 256;
const MAX_REALIZATION_REFUSAL_BYTES: usize = 1_024;
const MAX_DOMAIN_EVIDENCE_BYTES: usize = 4_096;

/// One semantic value boundary derived from a verified program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementBoundary {
    value_type: ResolvedValueType,
    shape: Shape,
}

impl IndexRefinementBoundary {
    /// Returns the boundary element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the boundary shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Canonical identity of the numerical contract an occurrence is lowered under.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NumericalContractIdentity(Box<str>);

impl NumericalContractIdentity {
    /// Identifies the numerical contract by its canonical key.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when the key is empty or exceeds the governed
    /// byte bound.
    pub fn try_from_key(key: &str) -> Result<Self, IndexRefinementVerificationError> {
        if key.is_empty() || key.len() > MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES {
            return Err(
                IndexRefinementVerificationError::InvalidNumericalContractIdentity {
                    actual: key.len(),
                    limit: MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES,
                },
            );
        }
        Ok(Self(key.into()))
    }

    /// Returns the canonical numerical-contract identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns the validated UTF-8 contract key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact operand/result signature admitted for one realization authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementSignature {
    operands: Vec<ResolvedValueType>,
    results: Vec<ResolvedValueType>,
}

impl IndexRefinementSignature {
    /// Creates a bounded ordered signature.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::SignatureTooLarge`] when
    /// either side exceeds the verifier's governed bound.
    pub fn new(
        operands: impl IntoIterator<Item = ResolvedValueType>,
        results: impl IntoIterator<Item = ResolvedValueType>,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let operands = operands.into_iter().collect::<Vec<_>>();
        let results = results.into_iter().collect::<Vec<_>>();
        if operands.len() > 4_096 || results.len() > 4_096 {
            return Err(IndexRefinementVerificationError::SignatureTooLarge);
        }
        Ok(Self { operands, results })
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

/// Dependency-neutral admitted authority for one lowering realization family.
#[derive(Clone)]
pub struct IndexRealizationAuthority {
    operation: OpKey,
    signature: IndexRefinementSignature,
    semantic: SemanticCapabilityAuthority,
    emitted_scalar_operations: Vec<ScalarOpKey>,
    emitted_scalar_definitions: CanonicalScalarDefinitionProjection,
    semantic_registry: FrozenSemanticRegistry,
    scalar_registry: FrozenScalarRegistry,
    identity: Box<[u8]>,
}

impl fmt::Debug for IndexRealizationAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IndexRealizationAuthority")
            .field("operation", &self.operation)
            .field("signature", &self.signature)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl PartialEq for IndexRealizationAuthority {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for IndexRealizationAuthority {}

impl IndexRealizationAuthority {
    /// Admits one exact operation/signature and scalar-emission ceiling.
    ///
    /// # Errors
    ///
    /// Returns a typed authority error when the operation/signature projection
    /// or an emitted scalar operation is absent from the supplied registries.
    pub fn admit(
        semantic: &crate::semantic::FrozenSemanticRegistry,
        scalars: &FrozenScalarRegistry,
        operation: OpKey,
        signature: IndexRefinementSignature,
        emitted: &[ScalarOpKey],
    ) -> Result<Self, IndexRefinementVerificationError> {
        let operation_authority = semantic
            .project_operation_authority(
                &operation,
                signature.operands.iter(),
                signature.results.iter(),
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        let mut emitted_scalar_operations = emitted.to_vec();
        emitted_scalar_operations.sort_unstable();
        emitted_scalar_operations.dedup();
        let emitted_scalar_definitions = scalars
            .project_reached(emitted_scalar_operations.iter())
            .map_err(|source| {
                IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
            })?;
        let identity = encode_authority_identity(
            &operation,
            &signature,
            &operation_authority,
            &emitted_scalar_definitions,
            scalars.snapshot_identity().as_bytes(),
        )
        .into_boxed_slice();
        Ok(Self {
            operation,
            signature,
            semantic: operation_authority,
            emitted_scalar_operations,
            emitted_scalar_definitions,
            semantic_registry: semantic.clone(),
            scalar_registry: scalars.clone(),
            identity,
        })
    }
    /// Returns the admitted operation.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }
    /// Returns the admitted signature.
    #[must_use]
    pub const fn signature(&self) -> &IndexRefinementSignature {
        &self.signature
    }
    /// Returns semantic authority.
    #[must_use]
    pub const fn semantic_authority(&self) -> &SemanticCapabilityAuthority {
        &self.semantic
    }
    /// Returns permitted emitted scalar operations.
    #[must_use]
    pub fn emitted_scalar_operations(&self) -> &[ScalarOpKey] {
        &self.emitted_scalar_operations
    }
    /// Returns provider-independent emitted definitions.
    #[must_use]
    pub const fn emitted_scalar_definitions(&self) -> &CanonicalScalarDefinitionProjection {
        &self.emitted_scalar_definitions
    }
    /// Returns the canonical admitted-authority identity.
    #[must_use]
    pub fn identity(&self) -> &[u8] {
        &self.identity
    }
}

/// Exact semantic subject derived from a verified program and typed ordinal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementSubject {
    graph: SemanticGraphIdentity,
    occurrence: SemanticOccurrence,
    operation: OpKey,
    inputs: Vec<IndexRefinementBoundary>,
    operands: Vec<usize>,
    results: Vec<IndexRefinementBoundary>,
    signature: IndexRefinementSignature,
    attributes: OperationAttributes,
    effect: OperationEffect,
    numerical_contract: NumericalContractIdentity,
    semantic_authority: SemanticCapabilityAuthority,
    identity: Box<[u8]>,
}

impl IndexRefinementSubject {
    /// Derives one exact subject from a verified semantic graph and graph-local ordinal.
    ///
    /// # Errors
    ///
    /// Returns a typed semantic-program error when the ordinal or one of its
    /// referenced values cannot be resolved, or its signature exceeds bounds.
    pub fn derive(
        program: &SemanticProgram,
        occurrence: SemanticOccurrence,
        numerical_contract: NumericalContractIdentity,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let operation_ref = program
            .operations()
            .nth(occurrence.get() as usize)
            .ok_or(IndexRefinementVerificationError::OccurrenceOutOfRange { occurrence })?;
        let operation = operation_ref.key().clone();
        let attributes = operation_ref.attributes().clone();
        let definition = program
            .semantic_registry()
            .operation_definition(&operation)
            .ok_or(IndexRefinementVerificationError::OperationDefinitionMissing)?;
        let effect = definition.effect();
        let mut values = Vec::new();
        let mut inputs = Vec::new();
        let mut operands = Vec::new();
        let mut operand_types = Vec::new();
        for value in operation_ref.operands() {
            let reference = program
                .value(value)
                .map_err(IndexRefinementVerificationError::SemanticHandle)?;
            let shape = program
                .shape(value)
                .map_err(IndexRefinementVerificationError::SemanticHandle)?
                .clone();
            let index = values
                .iter()
                .position(|candidate| *candidate == value)
                .unwrap_or_else(|| {
                    values.push(value);
                    inputs.push(IndexRefinementBoundary {
                        value_type: reference.resolved_type().clone(),
                        shape: shape.clone(),
                    });
                    values.len() - 1
                });
            operands.push(index);
            operand_types.push(reference.resolved_type().clone());
        }
        let mut results = Vec::new();
        let mut result_types = Vec::new();
        for value in operation_ref.results() {
            let reference = program
                .value(value)
                .map_err(IndexRefinementVerificationError::SemanticHandle)?;
            let shape = program
                .shape(value)
                .map_err(IndexRefinementVerificationError::SemanticHandle)?
                .clone();
            result_types.push(reference.resolved_type().clone());
            results.push(IndexRefinementBoundary {
                value_type: reference.resolved_type().clone(),
                shape,
            });
        }
        let signature = IndexRefinementSignature::new(operand_types, result_types)?;
        let semantic_authority = program
            .semantic_registry()
            .project_operation_occurrence_authority(
                &operation,
                signature.operands.iter(),
                signature.results.iter(),
                &attributes,
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        let graph = program.semantic_identity().graph().clone();
        let mut subject = Self {
            graph,
            occurrence,
            operation,
            inputs,
            operands,
            results,
            signature,
            attributes,
            effect,
            numerical_contract,
            semantic_authority,
            identity: Box::new([]),
        };
        subject.identity = encode_subject_identity(&subject).into_boxed_slice();
        Ok(subject)
    }

    /// Returns the semantic operation key.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns host-canonical attributes.
    #[must_use]
    pub const fn attributes(&self) -> &OperationAttributes {
        &self.attributes
    }

    /// Returns distinct ordered input boundaries.
    #[must_use]
    pub fn inputs(&self) -> &[IndexRefinementBoundary] {
        &self.inputs
    }
    /// Returns the input position for every ordered operand.
    #[must_use]
    pub fn operands(&self) -> &[usize] {
        &self.operands
    }

    /// Returns ordered results.
    #[must_use]
    pub fn results(&self) -> &[IndexRefinementBoundary] {
        &self.results
    }

    /// Returns the observable effect class.
    #[must_use]
    pub const fn effect(&self) -> OperationEffect {
        self.effect
    }

    /// Returns the bound numerical-contract identity.
    #[must_use]
    pub const fn numerical_contract(&self) -> &NumericalContractIdentity {
        &self.numerical_contract
    }

    /// Returns the bound graph identity.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraphIdentity {
        &self.graph
    }
    /// Returns the graph-local occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> SemanticOccurrence {
        self.occurrence
    }
    /// Returns the exact signature.
    #[must_use]
    pub const fn signature(&self) -> &IndexRefinementSignature {
        &self.signature
    }
}

/// Borrowed input to an independently registered semantic-realization verifier.
#[derive(Clone, Copy)]
pub struct IndexSemanticRealizationRequest<'a> {
    subject: &'a IndexRefinementSubject,
    region: &'a VerifiedIndexRegion,
    scalars: &'a FrozenScalarRegistry,
}

impl fmt::Debug for IndexSemanticRealizationRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IndexSemanticRealizationRequest")
            .field("operation", &self.subject.operation)
            .field("occurrence", &self.subject.occurrence)
            .field("region", self.region.canonical_identity())
            .finish_non_exhaustive()
    }
}

impl<'a> IndexSemanticRealizationRequest<'a> {
    /// Returns the exact program-derived semantic subject.
    #[must_use]
    pub const fn subject(self) -> &'a IndexRefinementSubject {
        self.subject
    }
    /// Returns the actual structurally verified region under review.
    #[must_use]
    pub const fn region(self) -> &'a VerifiedIndexRegion {
        self.region
    }
    /// Returns the exact admitted scalar registry.
    #[must_use]
    pub const fn scalars(self) -> &'a FrozenScalarRegistry {
        self.scalars
    }
}

/// Stable refusal from a semantic-realization verifier callback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexSemanticRealizationRefusal(Box<str>);

impl IndexSemanticRealizationRefusal {
    /// Creates one bounded nonempty refusal.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the diagnostic is empty or exceeds its bound.
    pub fn new(reason: impl Into<String>) -> Result<Self, IndexRefinementVerificationError> {
        let reason = reason.into();
        if reason.is_empty() || reason.len() > MAX_REALIZATION_REFUSAL_BYTES {
            return Err(
                IndexRefinementVerificationError::InvalidRealizationRefusal {
                    actual: reason.len(),
                    limit: MAX_REALIZATION_REFUSAL_BYTES,
                },
            );
        }
        Ok(Self(reason.into_boxed_str()))
    }
    /// Returns the stable refusal text.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.0
    }
}

/// Independent semantic authority for checking one emitted index region.
pub trait IndexSemanticRealizationVerifier: Send + Sync + 'static {
    /// Checks whether the actual region realizes the exact semantic subject.
    ///
    /// # Errors
    ///
    /// Returns a stable refusal when semantic realization is not proved.
    fn verify(
        &self,
        request: IndexSemanticRealizationRequest<'_>,
    ) -> Result<(), IndexSemanticRealizationRefusal>;
}

#[derive(Clone)]
struct RegisteredIndexSemanticRealization {
    provider: ProviderIdentity,
    revision: u32,
    semantic: SemanticCapabilityAuthority,
    verifier: Arc<dyn IndexSemanticRealizationVerifier>,
    identity: Box<[u8]>,
}

/// Mutable constructor for an independent semantic-realization registry.
pub struct IndexSemanticRealizationRegistryBuilder {
    semantic: FrozenSemanticRegistry,
    scalars: FrozenScalarRegistry,
    capabilities: BTreeMap<Vec<u8>, Arc<RegisteredIndexSemanticRealization>>,
}

impl fmt::Debug for IndexSemanticRealizationRegistryBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IndexSemanticRealizationRegistryBuilder")
            .field("capability_count", &self.capabilities.len())
            .finish_non_exhaustive()
    }
}

impl IndexSemanticRealizationRegistryBuilder {
    /// Creates an empty registry bound to exact semantic and scalar snapshots.
    #[must_use]
    pub fn new(semantic: FrozenSemanticRegistry, scalars: FrozenScalarRegistry) -> Self {
        Self {
            semantic,
            scalars,
            capabilities: BTreeMap::new(),
        }
    }

    /// Registers one independent verifier for an exact operation/signature.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when revision is zero, semantic projection fails,
    /// or an exact capability already exists. The callback itself governs the
    /// numerical-contract identities it accepts.
    pub fn register(
        &mut self,
        provider: ProviderIdentity,
        operation: &OpKey,
        signature: &IndexRefinementSignature,
        revision: u32,
        verifier: Arc<dyn IndexSemanticRealizationVerifier>,
    ) -> Result<(), IndexRefinementVerificationError> {
        if revision == 0 {
            return Err(IndexRefinementVerificationError::ZeroRealizationVerifierRevision);
        }
        let semantic = self
            .semantic
            .project_operation_authority(
                operation,
                signature.operands.iter(),
                signature.results.iter(),
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        let key = encode_verifier_key(operation, signature);
        if self.capabilities.contains_key(&key) {
            return Err(IndexRefinementVerificationError::DuplicateRealizationVerifier);
        }
        let identity = encode_verifier_capability_identity(
            &key,
            &provider,
            revision,
            &semantic,
            self.scalars.snapshot_identity().as_bytes(),
        )
        .into_boxed_slice();
        self.capabilities.insert(
            key,
            Arc::new(RegisteredIndexSemanticRealization {
                provider,
                revision,
                semantic,
                verifier,
                identity,
            }),
        );
        Ok(())
    }

    /// Freezes the exact registry snapshot.
    #[must_use]
    pub fn freeze(self) -> FrozenIndexSemanticRealizationRegistry {
        let mut identity = Vec::new();
        identity.extend_from_slice(VERIFIER_REGISTRY_IDENTITY_TAG);
        push_slice(&mut identity, self.semantic.snapshot_identity().as_bytes());
        push_slice(&mut identity, self.scalars.snapshot_identity().as_bytes());
        push_len(&mut identity, self.capabilities.len());
        for (key, capability) in &self.capabilities {
            push_slice(&mut identity, key);
            push_slice(&mut identity, &capability.identity);
        }
        FrozenIndexSemanticRealizationRegistry(Arc::new(
            FrozenIndexSemanticRealizationRegistryData {
                semantic: self.semantic,
                scalars: self.scalars,
                capabilities: self.capabilities,
                identity: identity.into_boxed_slice(),
            },
        ))
    }
}

struct FrozenIndexSemanticRealizationRegistryData {
    semantic: FrozenSemanticRegistry,
    scalars: FrozenScalarRegistry,
    capabilities: BTreeMap<Vec<u8>, Arc<RegisteredIndexSemanticRealization>>,
    identity: Box<[u8]>,
}

/// Immutable independently registered semantic-realization authority.
#[derive(Clone)]
pub struct FrozenIndexSemanticRealizationRegistry(Arc<FrozenIndexSemanticRealizationRegistryData>);

impl fmt::Debug for FrozenIndexSemanticRealizationRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenIndexSemanticRealizationRegistry")
            .field("capability_count", &self.0.capabilities.len())
            .field("identity", &self.0.identity)
            .finish_non_exhaustive()
    }
}

impl FrozenIndexSemanticRealizationRegistry {
    /// Returns the exact canonical registry identity.
    #[must_use]
    pub fn identity(&self) -> &[u8] {
        &self.0.identity
    }

    /// Resolves one verifier from an exact program-derived subject.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when no governed contract capability exists or
    /// the subject came from another semantic authority.
    pub fn resolve(
        &self,
        subject: &IndexRefinementSubject,
    ) -> Result<ResolvedIndexRealization, IndexRefinementVerificationError> {
        let key = encode_verifier_key(&subject.operation, &subject.signature);
        let capability = self
            .0
            .capabilities
            .get(&key)
            .ok_or(IndexRefinementVerificationError::MissingRealizationVerifier)?;
        let actual = self
            .0
            .semantic
            .project_operation_occurrence_authority(
                &subject.operation,
                subject.signature.operands.iter(),
                subject.signature.results.iter(),
                &subject.attributes,
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        if actual != subject.semantic_authority {
            return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
        }
        if capability.semantic.registry_snapshot() != subject.semantic_authority.registry_snapshot()
        {
            return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
        }
        let identity =
            encode_resolution_identity(&capability.identity, &subject.identity).into_boxed_slice();
        Ok(ResolvedIndexRealization {
            registry: self.clone(),
            capability: Arc::clone(capability),
            subject: subject.clone(),
            identity,
        })
    }
}

/// One sealed independent-verifier resolution for an exact semantic subject.
#[derive(Clone)]
pub struct ResolvedIndexRealization {
    registry: FrozenIndexSemanticRealizationRegistry,
    capability: Arc<RegisteredIndexSemanticRealization>,
    subject: IndexRefinementSubject,
    identity: Box<[u8]>,
}

impl fmt::Debug for ResolvedIndexRealization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedIndexRealization")
            .field("registry", &self.registry)
            .field("provider", &self.capability.provider)
            .field("revision", &self.capability.revision)
            .field("subject", &self.subject)
            .field("identity", &self.identity)
            .finish()
    }
}

impl PartialEq for ResolvedIndexRealization {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for ResolvedIndexRealization {}

impl ResolvedIndexRealization {
    /// Returns the exact governed subject.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        &self.subject
    }
    /// Returns the independent verifier provider.
    #[must_use]
    pub fn provider(&self) -> &ProviderIdentity {
        &self.capability.provider
    }
    /// Returns the independent verifier revision.
    #[must_use]
    pub fn revision(&self) -> u32 {
        self.capability.revision
    }
}

/// One ordered operand bound to its verified region input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperandBinding {
    operand: usize,
    input: usize,
    input_tensor: VerifiedTensorId,
}

impl OperandBinding {
    /// Returns the ordered operand position.
    #[must_use]
    pub const fn operand(&self) -> usize {
        self.operand
    }
    /// Returns the occurrence-local semantic value.
    #[must_use]
    pub const fn input(&self) -> usize {
        self.input
    }
    /// Returns the verified input tensor carrying the value.
    #[must_use]
    pub const fn input_tensor(&self) -> VerifiedTensorId {
        self.input_tensor
    }
}

/// One ordered result bound to its verified output root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResultBinding {
    result: usize,
    output_tensor: VerifiedTensorId,
    write_access: VerifiedTensorAccessId,
    written_value: VerifiedScalarValueId,
}

impl ResultBinding {
    /// Returns the ordered result position.
    #[must_use]
    pub const fn result(&self) -> usize {
        self.result
    }
    /// Returns the verified output tensor.
    #[must_use]
    pub const fn output_tensor(&self) -> VerifiedTensorId {
        self.output_tensor
    }
    /// Returns the complete unique write.
    #[must_use]
    pub const fn write_access(&self) -> VerifiedTensorAccessId {
        self.write_access
    }
    /// Returns the scalar value written by the output root.
    #[must_use]
    pub const fn written_value(&self) -> VerifiedScalarValueId {
        self.written_value
    }
}

/// Canonical identity of one checked occurrence-to-region receipt.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementReceiptIdentity(Box<[u8]>);

impl IndexRefinementReceiptIdentity {
    /// Returns the canonical receipt bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Complete identity of one trusted residual-domain proof authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexDomainProofAuthority {
    provider: ProviderIdentity,
    rule: ProviderIdentity,
    revision: u32,
}

impl IndexDomainProofAuthority {
    /// Creates a proof authority with a nonzero output-affecting revision.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when `revision` is zero.
    pub fn new(
        provider: ProviderIdentity,
        rule: ProviderIdentity,
        revision: u32,
    ) -> Result<Self, IndexRefinementVerificationError> {
        if revision == 0 {
            return Err(IndexRefinementVerificationError::ZeroDomainProofRevision);
        }
        Ok(Self {
            provider,
            rule,
            revision,
        })
    }

    /// Returns the proof provider identity.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }
    /// Returns the versioned proof rule identity.
    #[must_use]
    pub const fn rule(&self) -> &ProviderIdentity {
        &self.rule
    }
    /// Returns the output-affecting authority revision.
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }
}

/// A proving basis a trusted domain verifier may claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexDomainProofEvidence {
    /// A sound named derivation over the complete predicate domain.
    #[non_exhaustive]
    Sound {
        /// Sound proof class.
        proof: IndexDomainSoundProof,
        /// Authority-owned canonical derivation bytes.
        derivation: Box<[u8]>,
    },
    /// Exact evaluation of every point in a bounded finite domain.
    #[non_exhaustive]
    ExhaustiveFinite {
        /// Number of evaluated domain points.
        points: u64,
        /// Authority-owned canonical derivation bytes.
        derivation: Box<[u8]>,
    },
}

impl IndexDomainProofEvidence {
    /// Creates bounded sound-proof evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when the derivation is empty or oversized.
    pub fn sound(
        proof: IndexDomainSoundProof,
        derivation: impl Into<Box<[u8]>>,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let derivation = derivation.into();
        validate_domain_evidence(&derivation)?;
        Ok(Self::Sound { proof, derivation })
    }

    /// Creates bounded exhaustive-finite evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when the derivation is empty or oversized.
    pub fn exhaustive_finite(
        points: u64,
        derivation: impl Into<Box<[u8]>>,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let derivation = derivation.into();
        validate_domain_evidence(&derivation)?;
        Ok(Self::ExhaustiveFinite { points, derivation })
    }
}

fn validate_domain_evidence(bytes: &[u8]) -> Result<(), IndexRefinementVerificationError> {
    if bytes.is_empty() || bytes.len() > MAX_DOMAIN_EVIDENCE_BYTES {
        return Err(IndexRefinementVerificationError::InvalidDomainProofEvidence);
    }
    Ok(())
}

/// A typed exact counterexample from a trusted domain verifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDomainDisproof {
    reason: Box<str>,
    point_ordinal: Option<u64>,
    counterexample: Box<[u8]>,
}

impl IndexDomainDisproof {
    /// Creates a bounded disproof payload.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal for empty or oversized evidence.
    pub fn new(
        reason: impl Into<Box<str>>,
        counterexample: impl Into<Box<[u8]>>,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let reason = reason.into();
        let counterexample = counterexample.into();
        if reason.is_empty()
            || reason.len() > MAX_DOMAIN_EVIDENCE_BYTES
            || counterexample.is_empty()
            || counterexample.len() > MAX_DOMAIN_EVIDENCE_BYTES
        {
            return Err(IndexRefinementVerificationError::InvalidDomainProofEvidence);
        }
        Ok(Self {
            reason,
            point_ordinal: None,
            counterexample,
        })
    }

    /// Attaches the exact enumerated point ordinal of the counterexample.
    #[must_use]
    pub fn with_point_ordinal(mut self, point_ordinal: u64) -> Self {
        self.point_ordinal = Some(point_ordinal);
        self
    }
    /// Returns the stable reason code.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    /// Returns the optional exact point ordinal.
    #[must_use]
    pub const fn point_ordinal(&self) -> Option<u64> {
        self.point_ordinal
    }
}

/// One trusted verifier's total claim about an exact residual obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexDomainProofClaim {
    /// The obligation is proved.
    Proved(IndexDomainProofEvidence),
    /// The obligation has an exact counterexample.
    Disproved(IndexDomainDisproof),
    /// The verifier cannot prove or disprove the obligation.
    Unknown(IndexDomainUnknownReason),
}

/// Trusted callback assessed exactly once by IR receipt completion.
pub trait IndexDomainProofVerifier: Send + Sync {
    /// Returns the authority governing every claim from this verifier.
    fn authority(&self) -> &IndexDomainProofAuthority;
    /// Assesses one exact obligation from `region`.
    fn assess(
        &self,
        region: &VerifiedIndexRegion,
        obligation: UnknownIndexDomainPredicate,
    ) -> IndexDomainProofClaim;
}

/// One exact assessment retained for success identity or refusal explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDomainProofAssessment {
    obligation: UnknownIndexDomainPredicate,
    authority: IndexDomainProofAuthority,
    claim: IndexDomainProofClaim,
}

impl IndexDomainProofAssessment {
    /// Returns the exact region-owned obligation.
    #[must_use]
    pub const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }
    /// Returns the authority that made the claim.
    #[must_use]
    pub const fn authority(&self) -> &IndexDomainProofAuthority {
        &self.authority
    }
    /// Returns the verifier's total claim.
    #[must_use]
    pub const fn claim(&self) -> &IndexDomainProofClaim {
        &self.claim
    }
}

/// One IR-sealed residual-domain proof retained by a refinement receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementDomainProof {
    obligation: UnknownIndexDomainPredicate,
    authority: IndexDomainProofAuthority,
    proof: IndexDomainProofEvidence,
    identity: Box<[u8]>,
}

impl IndexRefinementDomainProof {
    /// Returns the exact region-owned obligation that was proved.
    #[must_use]
    pub const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }
    /// Returns the authority that proved the obligation.
    #[must_use]
    pub const fn authority(&self) -> &IndexDomainProofAuthority {
        &self.authority
    }
    /// Returns the retained proof basis.
    #[must_use]
    pub const fn proof(&self) -> &IndexDomainProofEvidence {
        &self.proof
    }
    /// Returns the canonical proof identity.
    #[must_use]
    pub fn identity(&self) -> &[u8] {
        &self.identity
    }
}

/// Opaque checked binding of one semantic occurrence to one verified region.
#[derive(Clone, Debug)]
pub struct IndexRefinementReceipt {
    graph: SemanticGraphIdentity,
    occurrence: SemanticOccurrence,
    region: CanonicalIndexRegionIdentity,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
    identity: IndexRefinementReceiptIdentity,
}

impl IndexRefinementReceipt {
    /// Returns the semantic graph this receipt binds.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraphIdentity {
        &self.graph
    }
    /// Returns the graph-local semantic occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> SemanticOccurrence {
        self.occurrence
    }
    /// Returns the exact verified-region identity this receipt binds.
    #[must_use]
    pub const fn region(&self) -> &CanonicalIndexRegionIdentity {
        &self.region
    }
    /// Returns the checked scalar authority bound to the region.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }
    /// Returns ordered operand-to-input bindings.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }
    /// Returns ordered result-to-output bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }
    /// Returns independently verified residual-domain proofs.
    #[must_use]
    pub fn index_domain_proofs(&self) -> &[IndexRefinementDomainProof] {
        &self.index_domain_proofs
    }
    /// Returns the canonical receipt identity.
    #[must_use]
    pub const fn identity(&self) -> &IndexRefinementReceiptIdentity {
        &self.identity
    }
}

/// Checked association awaiting proof of retained index-domain obligations.
#[derive(Clone, Debug)]
pub struct PendingIndexRefinementReceipt {
    resolution: ResolvedIndexRealization,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    region: VerifiedIndexRegion,
}

impl PendingIndexRefinementReceipt {
    /// Returns the checked semantic occurrence.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        self.resolution.subject()
    }
    /// Returns the exact retained verified region.
    #[must_use]
    pub const fn region(&self) -> &VerifiedIndexRegion {
        &self.region
    }
    /// Returns checked scalar authority evidence.
    #[must_use]
    pub const fn scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }
    /// Returns ordered operand bindings.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }
    /// Returns ordered result bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }
    /// Returns every exact residual obligation in canonical region order.
    #[must_use]
    pub fn obligations(&self) -> impl ExactSizeIterator<Item = UnknownIndexDomainPredicate> + '_ {
        self.region.unknown_index_domain_predicates()
    }
}

impl PartialEq for PendingIndexRefinementReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.resolution == other.resolution
            && self.scalar_authority == other.scalar_authority
            && self.operand_bindings == other.operand_bindings
            && self.result_bindings == other.result_bindings
            && self.region.canonical_identity() == other.region.canonical_identity()
    }
}

impl Eq for PendingIndexRefinementReceipt {}

/// Result of checking the dependency-neutral refinement association.
#[derive(Clone, Debug)]
#[must_use]
pub enum IndexRefinementVerificationOutcome {
    /// All obligations are discharged and a receipt was minted.
    Verified(Box<IndexRefinementReceipt>),
    /// The association is checked, but residual obligations grant no permission.
    Pending(Box<PendingIndexRefinementReceipt>),
}

impl ResolvedIndexRealization {
    /// Checks the occurrence and region together, minting no receipt while a
    /// residual index-domain obligation remains.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when scalar authority, effect, or ordered tensor
    /// interfaces disagree.
    pub fn verify(
        &self,
        lowering: &IndexRealizationAuthority,
        region: &VerifiedIndexRegion,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        let subject = &self.subject;
        check_lowering_authority(subject, self, lowering)?;
        if subject.effect != OperationEffect::Pure {
            return Err(IndexRefinementVerificationError::EffectNotIndexable {
                effect: subject.effect,
            });
        }
        let scalar_authority =
            self.registry
                .0
                .scalars
                .revalidate_region(region)
                .map_err(|source| {
                    IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
                })?;
        if scalar_authority.scalar_snapshot() != self.registry.0.scalars.snapshot_identity() {
            return Err(IndexRefinementVerificationError::ScalarSnapshotMismatch);
        }
        if scalar_authority
            .reached_operations()
            .iter()
            .any(|reached| !lowering.emitted_scalar_operations.contains(reached))
        {
            return Err(IndexRefinementVerificationError::ScalarAuthorityConformance);
        }
        let operand_bindings = bind_operands(subject, region)?;
        let result_bindings = bind_results(subject, region)?;
        self.capability
            .verifier
            .verify(IndexSemanticRealizationRequest {
                subject,
                region,
                scalars: &self.registry.0.scalars,
            })
            .map_err(IndexRefinementVerificationError::SemanticRealizationRefused)?;
        if region.unknown_index_domain_predicates().len() != 0 {
            return Ok(IndexRefinementVerificationOutcome::Pending(Box::new(
                PendingIndexRefinementReceipt {
                    resolution: self.clone(),
                    scalar_authority,
                    operand_bindings,
                    result_bindings,
                    region: region.clone(),
                },
            )));
        }
        Ok(IndexRefinementVerificationOutcome::Verified(Box::new(
            mint_receipt(
                subject,
                self,
                region.canonical_identity(),
                scalar_authority,
                operand_bindings,
                result_bindings,
                Vec::new(),
            ),
        )))
    }

    /// Assesses every retained obligation exactly once and mints the receipt.
    ///
    /// A disproved or unknown obligation consumes no pending state and mints no
    /// receipt. The caller retains its clone if it needs diagnostics or retry.
    ///
    /// # Errors
    ///
    /// Returns the first canonical disproved or unsupported obligation.
    pub fn complete(
        pending: &PendingIndexRefinementReceipt,
        verifier: &dyn IndexDomainProofVerifier,
    ) -> Result<(IndexRefinementReceipt, Vec<IndexDomainProofAssessment>), IndexDomainProofRefusal>
    {
        let assessments = pending
            .obligations()
            .map(|obligation| IndexDomainProofAssessment {
                obligation,
                authority: verifier.authority().clone(),
                claim: verifier.assess(&pending.region, obligation),
            })
            .collect::<Vec<_>>();
        if assessments
            .iter()
            .any(|assessment| matches!(assessment.claim, IndexDomainProofClaim::Disproved(_)))
        {
            return Err(IndexDomainProofRefusal {
                assessments,
                kind: IndexDomainProofRefusalKind::Disproved,
            });
        }
        if assessments
            .iter()
            .any(|assessment| matches!(assessment.claim, IndexDomainProofClaim::Unknown(_)))
        {
            return Err(IndexDomainProofRefusal {
                assessments,
                kind: IndexDomainProofRefusalKind::Unknown,
            });
        }
        let mut proofs = Vec::with_capacity(pending.obligations().len());
        for assessment in &assessments {
            let IndexDomainProofClaim::Proved(proof) = &assessment.claim else {
                unreachable!("the refusal scans removed every non-proof claim")
            };
            proofs.push(IndexRefinementDomainProof {
                obligation: assessment.obligation,
                authority: assessment.authority.clone(),
                proof: proof.clone(),
                identity: encode_proof_identity(
                    &pending.region,
                    assessment.obligation,
                    &assessment.authority,
                    proof,
                )
                .into_boxed_slice(),
            });
        }
        Ok((
            mint_receipt(
                pending.resolution.subject(),
                &pending.resolution,
                pending.region.canonical_identity(),
                pending.scalar_authority.clone(),
                pending.operand_bindings.clone(),
                pending.result_bindings.clone(),
                proofs,
            ),
            assessments,
        ))
    }
}

/// Whether one atomic completion pass found a disproof or an unknown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexDomainProofRefusalKind {
    /// At least one exact obligation was disproved.
    Disproved,
    /// No obligation was disproved and at least one remained unknown.
    Unknown,
}

/// Atomic refusal retaining every canonical assessment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexDomainProofRefusal {
    assessments: Vec<IndexDomainProofAssessment>,
    kind: IndexDomainProofRefusalKind,
}

impl IndexDomainProofRefusal {
    /// Returns all assessments in canonical obligation order.
    #[must_use]
    pub fn assessments(&self) -> &[IndexDomainProofAssessment] {
        &self.assessments
    }
    /// Returns the fail-closed refusal class.
    #[must_use]
    pub const fn kind(&self) -> IndexDomainProofRefusalKind {
        self.kind
    }
}

/// Why IR-owned refinement verification refused to mint a receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRefinementVerificationError {
    /// A numerical-contract identity was empty or exceeded its byte bound.
    InvalidNumericalContractIdentity {
        /// Supplied byte length.
        actual: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
    /// A realization-verifier refusal was empty or exceeded its byte bound.
    InvalidRealizationRefusal {
        /// Supplied byte length.
        actual: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
    /// A residual-domain proof authority used revision zero.
    ZeroDomainProofRevision,
    /// A residual-domain reason, derivation, or counterexample was invalid.
    InvalidDomainProofEvidence,
    /// An admitted signature exceeded the governed bound.
    SignatureTooLarge,
    /// The typed graph-local occurrence is outside the verified program.
    OccurrenceOutOfRange {
        /// Graph-local occurrence that did not resolve.
        occurrence: SemanticOccurrence,
    },
    /// A verified semantic handle failed to resolve.
    SemanticHandle(crate::semantic::HandleError),
    /// The semantic operation definition disappeared from its frozen authority.
    OperationDefinitionMissing,
    /// Semantic authority projection failed.
    SemanticAuthority(Arc<RegistryError>),
    /// Resolution and subject name different operations.
    OperationMismatch,
    /// Resolution and subject name different attributes.
    AttributeMismatch,
    /// Resolution and subject name different numerical contracts.
    NumericalContractMismatch,
    /// Resolution and subject name different semantic graphs or ordinals.
    OccurrenceMismatch,
    /// Resolution and subject signatures disagree.
    CapabilitySignatureMismatch,
    /// Semantic registry authority disagrees.
    SemanticAuthorityMismatch,
    /// The program-derived subject came from another semantic authority.
    SubjectSemanticAuthorityMismatch,
    /// The lowering or region came from another scalar-registry snapshot.
    ScalarSnapshotMismatch,
    /// No independent verifier governs the operation/signature/contract subject.
    MissingRealizationVerifier,
    /// Two independent verifiers contended for one exact subject key.
    DuplicateRealizationVerifier,
    /// A realization-verifier revision must be nonzero.
    ZeroRealizationVerifierRevision,
    /// Independent semantic realization was refused.
    SemanticRealizationRefused(IndexSemanticRealizationRefusal),
    /// The region reached scalar operations outside admission.
    ScalarAuthorityConformance,
    /// The occurrence effect is not realizable by this pure index profile.
    EffectNotIndexable {
        /// The rejected effect class.
        effect: OperationEffect,
    },
    /// Scalar authority rejected the region.
    ScalarAuthority(Arc<ScalarRegistryError>),
    /// A verified region handle could not be resolved.
    Handle(VerifiedIndexHandleError),
    /// A boundary exposes no static shape in this bounded verifier.
    SymbolicBoundary,
    /// Region input count disagrees with distinct semantic operands.
    OperandArity {
        /// Number of verified input boundaries.
        region_inputs: usize,
        /// Number of distinct semantic operands.
        distinct_operands: usize,
    },
    /// One region input disagrees with its semantic operand.
    OperandInterface {
        /// Position of the disagreeing input.
        position: usize,
    },
    /// Region output count disagrees with semantic results.
    ResultArity {
        /// Number of verified output roots.
        region_outputs: usize,
        /// Number of semantic results.
        results: usize,
    },
    /// One region output disagrees with its semantic result.
    ResultInterface {
        /// Position of the disagreeing output.
        position: usize,
    },
    /// One output writes a scalar value of the wrong type.
    ResultValueType {
        /// Position of the mistyped output.
        position: usize,
    },
    /// One output lacks complete unique-write evidence.
    IncompleteWrite {
        /// Position of the output without complete ownership.
        position: usize,
    },
}

impl fmt::Display for IndexRefinementVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumericalContractIdentity { actual, limit } => write!(
                formatter,
                "numerical-contract identity has {actual} bytes; expected 1..={limit}"
            ),
            Self::InvalidRealizationRefusal { actual, limit } => write!(
                formatter,
                "realization refusal has {actual} bytes; expected 1..={limit}"
            ),
            Self::ZeroDomainProofRevision => {
                formatter.write_str("domain-proof authority revision must be nonzero")
            }
            Self::InvalidDomainProofEvidence => {
                formatter.write_str("domain-proof evidence is empty or exceeds its byte bound")
            }
            Self::SignatureTooLarge => {
                formatter.write_str("refinement signature exceeds its bound")
            }
            Self::OccurrenceOutOfRange { occurrence } => write!(
                formatter,
                "semantic occurrence {} is outside the verified graph",
                occurrence.get()
            ),
            Self::SemanticHandle(source) => write!(formatter, "semantic handle failed: {source}"),
            Self::OperationDefinitionMissing => {
                formatter.write_str("semantic operation definition is absent")
            }
            Self::SemanticAuthority(source) => {
                write!(formatter, "semantic authority failed: {source}")
            }
            Self::OperationMismatch => {
                formatter.write_str("resolved authority names another operation")
            }
            Self::AttributeMismatch => {
                formatter.write_str("resolved authority names other attributes")
            }
            Self::NumericalContractMismatch => {
                formatter.write_str("resolved authority names another numerical contract")
            }
            Self::OccurrenceMismatch => {
                formatter.write_str("resolved authority names another graph occurrence")
            }
            Self::CapabilitySignatureMismatch => {
                formatter.write_str("resolved authority names another signature")
            }
            Self::SemanticAuthorityMismatch => {
                formatter.write_str("semantic registry authority disagrees")
            }
            Self::SubjectSemanticAuthorityMismatch => {
                formatter.write_str("program subject came from another semantic authority")
            }
            Self::ScalarSnapshotMismatch => {
                formatter.write_str("scalar-registry snapshot disagrees with admission")
            }
            Self::MissingRealizationVerifier => {
                formatter.write_str("no independent semantic-realization verifier is registered")
            }
            Self::DuplicateRealizationVerifier => {
                formatter.write_str("two semantic-realization verifiers govern one exact subject")
            }
            Self::ZeroRealizationVerifierRevision => {
                formatter.write_str("semantic-realization verifier revision is zero")
            }
            Self::SemanticRealizationRefused(source) => write!(
                formatter,
                "independent semantic-realization verifier refused: {}",
                source.reason()
            ),
            Self::ScalarAuthorityConformance => {
                formatter.write_str("region reached scalar authority outside admission")
            }
            Self::EffectNotIndexable { effect } => {
                write!(formatter, "occurrence effect {effect:?} is not indexable")
            }
            Self::ScalarAuthority(source) => write!(formatter, "scalar authority failed: {source}"),
            Self::Handle(source) => write!(formatter, "verified handle failed: {source}"),
            Self::SymbolicBoundary => formatter.write_str("a boundary exposed no static shape"),
            Self::OperandArity {
                region_inputs,
                distinct_operands,
            } => write!(
                formatter,
                "region declares {region_inputs} inputs for {distinct_operands} distinct operands"
            ),
            Self::OperandInterface { position } => {
                write!(
                    formatter,
                    "region input {position} does not match its operand"
                )
            }
            Self::ResultArity {
                region_outputs,
                results,
            } => write!(
                formatter,
                "region produces {region_outputs} outputs for {results} results"
            ),
            Self::ResultInterface { position } => {
                write!(
                    formatter,
                    "region output {position} does not match its result"
                )
            }
            Self::ResultValueType { position } => {
                write!(
                    formatter,
                    "region output {position} writes the wrong result type"
                )
            }
            Self::IncompleteWrite { position } => write!(
                formatter,
                "region output {position} lacks complete unique-write evidence"
            ),
        }
    }
}

impl Error for IndexRefinementVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ScalarAuthority(source) => Some(source.as_ref()),
            Self::SemanticAuthority(source) => Some(source.as_ref()),
            Self::SemanticHandle(source) => Some(source),
            Self::Handle(source) => Some(source),
            _ => None,
        }
    }
}

impl From<VerifiedIndexHandleError> for IndexRefinementVerificationError {
    fn from(source: VerifiedIndexHandleError) -> Self {
        Self::Handle(source)
    }
}

fn check_lowering_authority(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    lowering: &IndexRealizationAuthority,
) -> Result<(), IndexRefinementVerificationError> {
    let resolved = &resolution.subject;
    if subject.operation != resolved.operation {
        return Err(IndexRefinementVerificationError::OperationMismatch);
    }
    if subject.attributes != resolved.attributes {
        return Err(IndexRefinementVerificationError::AttributeMismatch);
    }
    if subject.numerical_contract != resolved.numerical_contract {
        return Err(IndexRefinementVerificationError::NumericalContractMismatch);
    }
    if subject.graph != resolved.graph || subject.occurrence != resolved.occurrence {
        return Err(IndexRefinementVerificationError::OccurrenceMismatch);
    }
    if subject.signature != resolved.signature
        || subject.effect != resolved.effect
        || subject.identity != resolved.identity
    {
        return Err(IndexRefinementVerificationError::CapabilitySignatureMismatch);
    }
    if lowering.operation != subject.operation {
        return Err(IndexRefinementVerificationError::OperationMismatch);
    }
    if lowering.signature != subject.signature {
        return Err(IndexRefinementVerificationError::CapabilitySignatureMismatch);
    }
    let lowering_occurrence = lowering
        .semantic_registry
        .project_operation_occurrence_authority(
            &subject.operation,
            subject.signature.operands.iter(),
            subject.signature.results.iter(),
            &subject.attributes,
        )
        .map_err(|source| IndexRefinementVerificationError::SemanticAuthority(Arc::new(source)))?;
    if lowering_occurrence != subject.semantic_authority
        || lowering.semantic_registry.snapshot_identity()
            != resolution.registry.0.semantic.snapshot_identity()
    {
        return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
    }
    if lowering.scalar_registry.snapshot_identity()
        != resolution.registry.0.scalars.snapshot_identity()
    {
        return Err(IndexRefinementVerificationError::ScalarSnapshotMismatch);
    }
    Ok(())
}

fn bind_operands(
    occurrence: &IndexRefinementSubject,
    region: &VerifiedIndexRegion,
) -> Result<Vec<OperandBinding>, IndexRefinementVerificationError> {
    let inputs = region
        .tensors()
        .filter(|tensor| tensor.role() == TensorRole::Input)
        .collect::<Vec<_>>();
    if inputs.len() != occurrence.inputs.len() {
        return Err(IndexRefinementVerificationError::OperandArity {
            region_inputs: inputs.len(),
            distinct_operands: occurrence.inputs.len(),
        });
    }
    for (position, (operand, input)) in occurrence.inputs.iter().zip(&inputs).enumerate() {
        let shape = input
            .shape()
            .as_static()
            .ok_or(IndexRefinementVerificationError::SymbolicBoundary)?;
        if input.value_type() != &operand.value_type || shape != &operand.shape {
            return Err(IndexRefinementVerificationError::OperandInterface { position });
        }
    }
    occurrence
        .operands
        .iter()
        .enumerate()
        .map(|(position, input)| {
            Ok(OperandBinding {
                operand: position,
                input: *input,
                input_tensor: inputs[*input].id(),
            })
        })
        .collect()
}

fn bind_results(
    occurrence: &IndexRefinementSubject,
    region: &VerifiedIndexRegion,
) -> Result<Vec<ResultBinding>, IndexRefinementVerificationError> {
    let roots = region.outputs().collect::<Vec<_>>();
    if roots.len() != occurrence.results.len() {
        return Err(IndexRefinementVerificationError::ResultArity {
            region_outputs: roots.len(),
            results: occurrence.results.len(),
        });
    }
    roots
        .iter()
        .zip(&occurrence.results)
        .enumerate()
        .map(|(position, (root, result))| {
            let access = region.access(root.access())?;
            if access.write_ownership_proof().is_none() {
                return Err(IndexRefinementVerificationError::IncompleteWrite { position });
            }
            let output = region.tensor(access.tensor())?;
            let shape = output
                .shape()
                .as_static()
                .ok_or(IndexRefinementVerificationError::SymbolicBoundary)?;
            if output.role() != TensorRole::Output
                || output.value_type() != &result.value_type
                || shape != &result.shape
            {
                return Err(IndexRefinementVerificationError::ResultInterface { position });
            }
            let written = region.scalar_value(root.value())?;
            if written.value_type() != &result.value_type {
                return Err(IndexRefinementVerificationError::ResultValueType { position });
            }
            Ok(ResultBinding {
                result: position,
                output_tensor: output.id(),
                write_access: root.access(),
                written_value: root.value(),
            })
        })
        .collect()
}

fn mint_receipt(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    region: &CanonicalIndexRegionIdentity,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
) -> IndexRefinementReceipt {
    let identity = encode_receipt_identity(
        subject,
        resolution,
        region,
        &scalar_authority,
        &index_domain_proofs,
    );
    IndexRefinementReceipt {
        graph: subject.graph.clone(),
        occurrence: subject.occurrence,
        region: region.clone(),
        scalar_authority,
        operand_bindings,
        result_bindings,
        index_domain_proofs,
        identity: IndexRefinementReceiptIdentity(identity.into_boxed_slice()),
    }
}

fn encode_receipt_identity(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    region: &CanonicalIndexRegionIdentity,
    scalar_authority: &ScalarAuthorityEvidence,
    proofs: &[IndexRefinementDomainProof],
) -> Vec<u8> {
    let mut bytes = RECEIPT_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region.as_bytes());
    push_slice(&mut bytes, &subject.identity);
    push_slice(&mut bytes, &resolution.identity);
    push_slice(&mut bytes, scalar_authority.definitions().as_bytes());
    push_slice(&mut bytes, scalar_authority.type_definitions().as_bytes());
    push_slice(&mut bytes, scalar_authority.semantic_snapshot().as_bytes());
    push_slice(&mut bytes, scalar_authority.scalar_snapshot().as_bytes());
    push_len(&mut bytes, proofs.len());
    for proof in proofs {
        push_slice(&mut bytes, proof.identity());
    }
    bytes
}

fn encode_subject_identity(subject: &IndexRefinementSubject) -> Vec<u8> {
    let mut bytes = SUBJECT_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, subject.graph.as_bytes());
    bytes.extend_from_slice(&subject.occurrence.get().to_be_bytes());
    encode_op_key(&mut bytes, &subject.operation);
    encode_signature(&mut bytes, &subject.signature);
    push_len(&mut bytes, subject.inputs.len());
    for input in &subject.inputs {
        push_slice(&mut bytes, input.value_type.canonical_encoding().as_bytes());
        encode_shape(&mut bytes, &input.shape);
    }
    push_len(&mut bytes, subject.operands.len());
    for input in &subject.operands {
        bytes.extend_from_slice(&(*input as u64).to_be_bytes());
    }
    push_len(&mut bytes, subject.results.len());
    for result in &subject.results {
        push_slice(
            &mut bytes,
            result.value_type.canonical_encoding().as_bytes(),
        );
        encode_shape(&mut bytes, &result.shape);
    }
    bytes.push(match subject.effect {
        OperationEffect::Pure => 1,
    });
    push_slice(
        &mut bytes,
        subject.attributes.canonical_encoding().as_bytes(),
    );
    push_slice(&mut bytes, subject.numerical_contract.as_bytes());
    push_slice(
        &mut bytes,
        subject.semantic_authority.reached_definitions().as_bytes(),
    );
    push_slice(
        &mut bytes,
        subject.semantic_authority.admission_provenance().as_bytes(),
    );
    push_slice(
        &mut bytes,
        subject.semantic_authority.registry_snapshot().as_bytes(),
    );
    bytes
}

fn encode_authority_identity(
    operation: &OpKey,
    signature: &IndexRefinementSignature,
    semantic: &SemanticCapabilityAuthority,
    scalar: &CanonicalScalarDefinitionProjection,
    scalar_snapshot: &[u8],
) -> Vec<u8> {
    let mut bytes = AUTHORITY_IDENTITY_TAG.to_vec();
    encode_op_key(&mut bytes, operation);
    encode_signature(&mut bytes, signature);
    push_slice(&mut bytes, semantic.reached_definitions().as_bytes());
    push_slice(&mut bytes, semantic.admission_provenance().as_bytes());
    push_slice(&mut bytes, semantic.registry_snapshot().as_bytes());
    push_slice(&mut bytes, scalar.as_bytes());
    push_slice(&mut bytes, scalar_snapshot);
    bytes
}

fn encode_verifier_key(operation: &OpKey, signature: &IndexRefinementSignature) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_op_key(&mut bytes, operation);
    encode_signature(&mut bytes, signature);
    bytes
}

fn encode_verifier_capability_identity(
    key: &[u8],
    provider: &ProviderIdentity,
    revision: u32,
    semantic: &SemanticCapabilityAuthority,
    scalar_snapshot: &[u8],
) -> Vec<u8> {
    let mut bytes = VERIFIER_CAPABILITY_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, key);
    push_slice(&mut bytes, provider.namespace().as_bytes());
    push_slice(&mut bytes, provider.name().as_bytes());
    bytes.extend_from_slice(&provider.revision().to_be_bytes());
    bytes.extend_from_slice(&revision.to_be_bytes());
    push_slice(&mut bytes, semantic.reached_definitions().as_bytes());
    push_slice(&mut bytes, semantic.admission_provenance().as_bytes());
    push_slice(&mut bytes, semantic.registry_snapshot().as_bytes());
    push_slice(&mut bytes, scalar_snapshot);
    bytes
}

fn encode_resolution_identity(authority: &[u8], subject: &[u8]) -> Vec<u8> {
    let mut bytes = RESOLUTION_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, authority);
    push_slice(&mut bytes, subject);
    bytes
}

fn encode_signature(output: &mut Vec<u8>, signature: &IndexRefinementSignature) {
    push_len(output, signature.operands.len());
    for ty in &signature.operands {
        push_slice(output, ty.canonical_encoding().as_bytes());
    }
    push_len(output, signature.results.len());
    for ty in &signature.results {
        push_slice(output, ty.canonical_encoding().as_bytes());
    }
}

fn encode_proof_identity(
    region: &VerifiedIndexRegion,
    obligation: UnknownIndexDomainPredicate,
    authority: &IndexDomainProofAuthority,
    proof: &IndexDomainProofEvidence,
) -> Vec<u8> {
    let mut bytes = PROOF_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region.canonical_identity().as_bytes());
    push_slice(&mut bytes, obligation.canonical_local_key().as_bytes());
    encode_provider(&mut bytes, authority.provider());
    encode_provider(&mut bytes, authority.rule());
    bytes.extend_from_slice(&authority.revision().to_be_bytes());
    match proof {
        IndexDomainProofEvidence::Sound { proof, derivation } => {
            bytes.push(1);
            bytes.push(match proof {
                IndexDomainSoundProof::VacuousEmptyDomain => 1,
                IndexDomainSoundProof::Interval => 2,
                IndexDomainSoundProof::ProvedExtentEquality => 3,
            });
            push_slice(&mut bytes, derivation);
        }
        IndexDomainProofEvidence::ExhaustiveFinite { points, derivation } => {
            bytes.push(2);
            bytes.extend_from_slice(&points.to_be_bytes());
            push_slice(&mut bytes, derivation);
        }
    }
    bytes
}

fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_shape(output: &mut Vec<u8>, shape: &Shape) {
    push_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}
