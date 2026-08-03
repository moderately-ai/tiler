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
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::Zero;

use crate::identity::{push_len, push_slice};
use crate::program::SemanticOccurrence;
use crate::semantic::{
    FrozenSemanticRegistry, OpKey, OperationAttributes, OperationEffect, ProviderIdentity,
    RegistryError, ResolvedValueType, SemanticCapabilityAuthority, SemanticGraphIdentity,
    SemanticProgram, SemanticRegistrySnapshotIdentity,
};
use crate::shape::Shape;

use super::{
    CanonicalIndexRegionIdentity, CanonicalScalarDefinitionProjection,
    CanonicalScalarRegistrySnapshotIdentity, FrozenScalarRegistry, IndexDomainPredicate,
    IndexDomainUnknownReason, IndexExprView, IndexExtentRef, IndexInteger, IndexIntegerSign,
    ScalarAuthorityEvidence, ScalarOpKey, ScalarRegistryError, TensorRole,
    UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedIndexRegion, VerifiedScalarValueId, VerifiedTensorAccessId,
    VerifiedTensorId,
};

const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-receipt.v1\0";
const SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v1\0";
const AUTHORITY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-authority.v1\0";
const RESOLUTION_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-resolution.v1\0";
const PROOF_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-domain-proof.v1\0";
const LAW_REGISTRY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-law-registry.v1\0";
const MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES: usize = 256;
const MAX_DOMAIN_EVIDENCE_BYTES: usize = 4_096;
/// Maximum cells the closed exact-finite residual proof algorithm may evaluate.
pub const MAX_FINITE_DOMAIN_PROOF_CELLS: u64 = 16 * 1024 * 1024;
const EXHAUSTIVE_DERIVATION: &[u8] = b"tiler.ir.exact-index-domain-enumeration.v1\0";
const COUNTEREXAMPLE_TAG: &[u8] = b"tiler.ir.index-domain-counterexample.v1\0";

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

struct FrozenIndexRealizationLawRegistryData {
    semantic: FrozenSemanticRegistry,
    scalars: FrozenScalarRegistry,
    identity: Box<[u8]>,
}

/// Immutable semantic-provider-bound logical realization-law authority.
#[derive(Clone)]
pub struct FrozenIndexRealizationLawRegistry(Arc<FrozenIndexRealizationLawRegistryData>);

impl fmt::Debug for FrozenIndexRealizationLawRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenIndexRealizationLawRegistry")
            .field("identity", &self.0.identity)
            .finish_non_exhaustive()
    }
}

impl PartialEq for FrozenIndexRealizationLawRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for FrozenIndexRealizationLawRegistry {}

impl FrozenIndexRealizationLawRegistry {
    /// Derives the law snapshot inseparably retained by one semantic registry.
    #[must_use]
    pub fn from_semantic(semantic: FrozenSemanticRegistry, scalars: FrozenScalarRegistry) -> Self {
        let mut identity = Vec::new();
        identity.extend_from_slice(LAW_REGISTRY_IDENTITY_TAG);
        push_slice(&mut identity, semantic.snapshot_identity().as_bytes());
        push_slice(&mut identity, scalars.snapshot_identity().as_bytes());
        push_len(&mut identity, semantic.index_realization_laws().len());
        for (operation, registered) in semantic.index_realization_laws() {
            encode_op_key(&mut identity, operation);
            encode_provider(&mut identity, &registered.provider);
            identity.extend_from_slice(&registered.revision.to_be_bytes());
            registered.law.encode(&mut identity);
        }
        Self(Arc::new(FrozenIndexRealizationLawRegistryData {
            semantic,
            scalars,
            identity: identity.into_boxed_slice(),
        }))
    }

    /// Returns the exact canonical registry identity.
    #[must_use]
    pub fn identity(&self) -> &[u8] {
        &self.0.identity
    }

    /// Returns the semantic snapshot that owns every realization law.
    #[must_use]
    pub fn semantic_snapshot(&self) -> &SemanticRegistrySnapshotIdentity {
        self.0.semantic.snapshot_identity()
    }

    /// Returns the scalar snapshot under which every law is interpreted.
    #[must_use]
    pub fn scalar_snapshot(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        self.0.scalars.snapshot_identity()
    }

    /// Resolves one semantic-provider-bound law from an exact subject.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when no governed contract capability exists or
    /// the subject came from another semantic authority.
    pub fn resolve(
        &self,
        subject: &IndexRefinementSubject,
    ) -> Result<ResolvedIndexRealization, IndexRefinementVerificationError> {
        let registered = self
            .0
            .semantic
            .index_realization_law(&subject.operation)
            .ok_or(IndexRefinementVerificationError::MissingRealizationLaw)?;
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
        if actual.registry_snapshot() != subject.semantic_authority.registry_snapshot() {
            return Err(IndexRefinementVerificationError::SubjectSemanticAuthorityMismatch);
        }
        let mut law_identity = Vec::new();
        encode_op_key(&mut law_identity, &subject.operation);
        encode_provider(&mut law_identity, &registered.provider);
        law_identity.extend_from_slice(&registered.revision.to_be_bytes());
        registered.law.encode(&mut law_identity);
        let identity =
            encode_resolution_identity(&law_identity, &subject.identity).into_boxed_slice();
        Ok(ResolvedIndexRealization {
            registry: self.clone(),
            law: registered.law.clone(),
            provider: registered.provider.clone(),
            revision: registered.revision,
            subject: subject.clone(),
            identity,
        })
    }
}

/// One sealed independent-verifier resolution for an exact semantic subject.
#[derive(Clone)]
pub struct ResolvedIndexRealization {
    registry: FrozenIndexRealizationLawRegistry,
    law: super::IndexRealizationLaw,
    provider: ProviderIdentity,
    revision: u32,
    subject: IndexRefinementSubject,
    identity: Box<[u8]>,
}

impl fmt::Debug for ResolvedIndexRealization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedIndexRealization")
            .field("registry", &self.registry)
            .field("law", &self.law)
            .field("provider", &self.provider)
            .field("revision", &self.revision)
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
        &self.provider
    }
    /// Returns the independent verifier revision.
    #[must_use]
    pub fn revision(&self) -> u32 {
        self.revision
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
    fn exact_finite() -> Self {
        Self {
            provider: ProviderIdentity::new("tiler", "ir-index-domain-proof", 1)
                .expect("the IR proof provider identity is canonical"),
            rule: ProviderIdentity::new("tiler", "exact-finite-index-domain-enumeration", 1)
                .expect("the IR proof rule identity is canonical"),
            revision: 1,
        }
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

/// Proof evidence produced by IR's closed residual-domain algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexDomainProofEvidence {
    /// Exact evaluation of every point in a bounded finite domain.
    #[non_exhaustive]
    ExhaustiveFinite {
        /// Number of evaluated domain points.
        points: u64,
        /// Authority-owned canonical derivation bytes.
        derivation: Box<[u8]>,
    },
}

/// Bounded policy input to IR's closed exact-finite proof algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexDomainProofBudget {
    max_cells: u64,
}

impl IndexDomainProofBudget {
    /// Creates a nonzero budget no larger than IR's hard proof bound.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::InvalidDomainProofBudget`]
    /// when `max_cells` is zero or exceeds
    /// [`MAX_FINITE_DOMAIN_PROOF_CELLS`].
    pub fn try_new(max_cells: u64) -> Result<Self, IndexRefinementVerificationError> {
        if max_cells == 0 || max_cells > MAX_FINITE_DOMAIN_PROOF_CELLS {
            return Err(IndexRefinementVerificationError::InvalidDomainProofBudget);
        }
        Ok(Self { max_cells })
    }

    /// Returns the maximum expression cells the proof may evaluate.
    #[must_use]
    pub const fn max_cells(self) -> u64 {
        self.max_cells
    }
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
    fn new(
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
    fn with_point_ordinal(mut self, point_ordinal: u64) -> Self {
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

    /// Returns the authority-owned canonical counterexample bytes.
    #[must_use]
    pub fn counterexample(&self) -> &[u8] {
        &self.counterexample
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
    /// The candidate must be the exact canonical region constructed by the
    /// registered semantic law. Semantic equivalence is not approximated here:
    /// an alternate logical spelling is refused and may become a physical
    /// alternative only after this semantic association is established.
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
        if !super::IndexRealizationLaw::accepts_numerical_contract(
            subject.numerical_contract().as_str(),
        ) {
            return Err(IndexRefinementVerificationError::NumericalContractNotGoverned);
        }
        let expected = self
            .law
            .realize(subject, &self.registry.0.scalars)
            .map_err(
                |source| IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(subject.operation().clone()),
                    rule: source.rule(),
                },
            )?;
        if expected.canonical_identity() != region.canonical_identity() {
            return Err(
                IndexRefinementVerificationError::SemanticRealizationMismatch {
                    expected: expected.canonical_identity().clone(),
                    actual: region.canonical_identity().clone(),
                },
            );
        }
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
        budget: IndexDomainProofBudget,
    ) -> Result<(IndexRefinementReceipt, Vec<IndexDomainProofAssessment>), IndexDomainProofRefusal>
    {
        let authority = IndexDomainProofAuthority::exact_finite();
        let assessments = pending
            .obligations()
            .map(|obligation| IndexDomainProofAssessment {
                obligation,
                authority: authority.clone(),
                claim: assess_finite_domain(&pending.region, obligation, budget),
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

fn assess_finite_domain(
    region: &VerifiedIndexRegion,
    obligation: UnknownIndexDomainPredicate,
    budget: IndexDomainProofBudget,
) -> IndexDomainProofClaim {
    let access = region
        .access(obligation.subject())
        .expect("a verified residual names an access in its own region");
    let dimensions = access
        .domain()
        .map(|dimension| {
            region
                .dimension(dimension)
                .expect("a verified access domain names its own dimensions")
                .extent()
                .as_static()
                .map(|extent| (dimension, extent.get()))
        })
        .collect::<Option<Vec<_>>>();
    let Some(dimensions) = dimensions else {
        return IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment);
    };
    let Some(points) = dimensions.iter().try_fold(1_u128, |product, (_, extent)| {
        product.checked_mul(u128::from(*extent))
    }) else {
        return proof_resource_limit(u128::MAX, budget);
    };
    let expression = predicate_expression(obligation.predicate());
    let mut plan = HashSet::new();
    if !collect_expression_plan(region, expression, &mut plan) {
        return IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment);
    }
    let required = points.saturating_mul(plan.len() as u128);
    if required > u128::from(budget.max_cells()) {
        return proof_resource_limit(required, budget);
    }
    let Ok(points) = u64::try_from(points) else {
        return proof_resource_limit(required, budget);
    };
    let mut coordinates = vec![0_u64; dimensions.len()];
    let mut environment = HashMap::with_capacity(dimensions.len());
    let mut values = HashMap::with_capacity(plan.len());
    for point_ordinal in 0..points {
        environment.clear();
        environment.extend(
            dimensions
                .iter()
                .zip(&coordinates)
                .map(|((dimension, _), coordinate)| (*dimension, *coordinate)),
        );
        values.clear();
        let Some(value) = evaluate_expression(region, expression, &environment, &mut values) else {
            return IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment);
        };
        if !predicate_holds(region, obligation.predicate(), &value) {
            let reason = match obligation.predicate() {
                IndexDomainPredicate::NonNegative { .. } => "logical-index-negative",
                IndexDomainPredicate::LessThanExtent { .. } => "logical-index-not-less-than-extent",
            };
            return IndexDomainProofClaim::Disproved(
                IndexDomainDisproof::new(reason, encode_counterexample(&coordinates, &value))
                    .expect("IR-owned counterexamples satisfy their own evidence bound")
                    .with_point_ordinal(point_ordinal),
            );
        }
        increment_coordinates(&mut coordinates, &dimensions);
    }
    IndexDomainProofClaim::Proved(IndexDomainProofEvidence::ExhaustiveFinite {
        points,
        derivation: EXHAUSTIVE_DERIVATION.into(),
    })
}

fn proof_resource_limit(required: u128, budget: IndexDomainProofBudget) -> IndexDomainProofClaim {
    IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
        resource: super::ProofResource::Cells,
        required,
        limit: budget.max_cells(),
    })
}

const fn predicate_expression(predicate: IndexDomainPredicate) -> VerifiedIndexExprId {
    match predicate {
        IndexDomainPredicate::NonNegative { expression }
        | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
    }
}

fn collect_expression_plan(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    reached: &mut HashSet<VerifiedIndexExprId>,
) -> bool {
    if !reached.insert(expression) {
        return true;
    }
    let expression = region
        .index_expression(expression)
        .expect("a verified predicate names an expression in its own region");
    match expression.view() {
        IndexExprView::Constant(_) | IndexExprView::Dimension(_) => true,
        IndexExprView::LinearCombination { terms, .. } => terms
            .map(super::LinearTermRef::value)
            .all(|child| collect_expression_plan(region, child, reached)),
        IndexExprView::FloorDiv { dividend, divisor }
        | IndexExprView::Modulo { dividend, divisor } => {
            divisor.as_static().is_some() && collect_expression_plan(region, dividend, reached)
        }
    }
}

fn evaluate_expression(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    environment: &HashMap<VerifiedDimensionId, u64>,
    values: &mut HashMap<VerifiedIndexExprId, BigInt>,
) -> Option<BigInt> {
    if let Some(value) = values.get(&expression) {
        return Some(value.clone());
    }
    let view = region
        .index_expression(expression)
        .expect("a verified predicate names an expression in its own region")
        .view();
    let value = match view {
        IndexExprView::Constant(value) => decode_integer(value),
        IndexExprView::Dimension(dimension) => BigInt::from(*environment.get(&dimension)?),
        IndexExprView::LinearCombination { constant, terms } => {
            let mut total = decode_integer(constant);
            for term in terms {
                total += decode_integer(term.coefficient())
                    * evaluate_expression(region, term.value(), environment, values)?;
            }
            total
        }
        IndexExprView::FloorDiv { dividend, divisor } => {
            evaluate_expression(region, dividend, environment, values)?
                .div_floor(&BigInt::from(divisor.as_static()?.get()))
        }
        IndexExprView::Modulo { dividend, divisor } => {
            evaluate_expression(region, dividend, environment, values)?
                .mod_floor(&BigInt::from(divisor.as_static()?.get()))
        }
    };
    values.insert(expression, value.clone());
    Some(value)
}

fn decode_integer(value: &IndexInteger) -> BigInt {
    let (sign, magnitude) = value.to_sign_magnitude();
    BigInt::from_bytes_be(
        match sign {
            IndexIntegerSign::Positive => Sign::Plus,
            IndexIntegerSign::Negative => Sign::Minus,
            IndexIntegerSign::Zero => Sign::NoSign,
        },
        &magnitude,
    )
}

fn predicate_holds(
    region: &VerifiedIndexRegion,
    predicate: IndexDomainPredicate,
    value: &BigInt,
) -> bool {
    match predicate {
        IndexDomainPredicate::NonNegative { .. } => value >= &BigInt::zero(),
        IndexDomainPredicate::LessThanExtent { extent, .. } => {
            value < &BigInt::from(resolve_extent(region, extent))
        }
    }
}

fn resolve_extent(region: &VerifiedIndexRegion, extent: IndexExtentRef) -> u64 {
    match extent {
        IndexExtentRef::Dimension(dimension) => region
            .dimension(dimension)
            .expect("a verified predicate names its own dimension")
            .extent()
            .as_static()
            .expect("finite proof rejected symbolic dimensions")
            .get(),
        IndexExtentRef::TensorAxis { tensor, axis } => region
            .tensor(tensor)
            .expect("a verified predicate names its own tensor")
            .shape()
            .as_static()
            .expect("finite proof rejected symbolic boundaries")
            .extents()[usize::try_from(axis).expect("a verified tensor axis fits usize")]
        .get(),
    }
}

fn increment_coordinates(coordinates: &mut [u64], dimensions: &[(VerifiedDimensionId, u64)]) {
    for (coordinate, (_, extent)) in coordinates.iter_mut().zip(dimensions).rev() {
        *coordinate += 1;
        if *coordinate < *extent {
            return;
        }
        *coordinate = 0;
    }
}

fn encode_counterexample(coordinates: &[u64], value: &BigInt) -> Box<[u8]> {
    let mut output = COUNTEREXAMPLE_TAG.to_vec();
    push_len(&mut output, coordinates.len());
    for coordinate in coordinates {
        output.extend_from_slice(&coordinate.to_be_bytes());
    }
    let (sign, magnitude) = value.to_bytes_be();
    output.push(u8::from(matches!(sign, Sign::Minus)));
    push_slice(&mut output, &magnitude);
    output.into_boxed_slice()
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
    /// A residual-domain reason, derivation, or counterexample was invalid.
    InvalidDomainProofEvidence,
    /// Exact-finite proof budget is zero or exceeds IR's hard bound.
    InvalidDomainProofBudget,
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
    MissingRealizationLaw,
    /// The semantic law does not govern this numerical-contract domain.
    NumericalContractNotGoverned,
    /// The semantic law could not construct its expected canonical region.
    SemanticRealizationLawRefused {
        /// Operation whose registered law refused.
        operation: Box<OpKey>,
        /// Stable failing law rule.
        rule: &'static str,
    },
    /// The candidate's exact canonical identity differs from the semantic law's
    /// expected canonical region; semantic equivalence alone is insufficient.
    SemanticRealizationMismatch {
        /// Canonical region required by semantic authority.
        expected: CanonicalIndexRegionIdentity,
        /// Canonical region emitted by the selected lowering.
        actual: CanonicalIndexRegionIdentity,
    },
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
            Self::InvalidDomainProofEvidence => {
                formatter.write_str("domain-proof evidence is empty or exceeds its byte bound")
            }
            Self::InvalidDomainProofBudget => {
                formatter.write_str("domain-proof budget is zero or exceeds the IR hard bound")
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
            Self::MissingRealizationLaw => {
                formatter.write_str("no semantic-realization law is registered")
            }
            Self::NumericalContractNotGoverned => formatter
                .write_str("the semantic realization law does not govern this numerical contract"),
            Self::SemanticRealizationLawRefused { operation, rule } => {
                write!(
                    formatter,
                    "semantic realization law for {operation} refused at {rule}"
                )
            }
            Self::SemanticRealizationMismatch { expected, actual } => write!(
                formatter,
                "candidate region {:?} differs from semantic law region {:?}",
                actual.as_bytes(),
                expected.as_bytes()
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::index::{DomainRole, EXTENT_PHASE_CEILING, IndexRegionBuilder, SourcedExtent};
    use crate::program::abi::AvailabilityPhase;
    use crate::semantic::TypeKey;
    use crate::shape::{
        BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, InterfaceParameterKey,
        RootBinding, SemanticInputConstraint, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
    };

    const LENGTH: u64 = 65_535;

    fn residual_region(second_extent: u64, rounds: usize, offset: i128) -> VerifiedIndexRegion {
        let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
            .expect("the fixture receives a fresh builder identity");
        let first = builder
            .dimension(DomainRole::Parallel, Extent::new(LENGTH))
            .unwrap();
        let second = builder
            .dimension(DomainRole::Parallel, Extent::new(second_extent))
            .unwrap();
        let shape = Shape::from_dims([LENGTH, second_extent]);
        let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
        let input = builder
            .tensor(TensorRole::Input, value_type.clone(), shape.clone())
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type, shape)
            .unwrap();
        let first_coordinate = builder.dimension_expr(first).unwrap();
        let second_coordinate = builder.dimension_expr(second).unwrap();
        let mut conservative = first_coordinate;
        for _ in 0..rounds {
            let two = SourcedExtent::Static(Extent::new(2));
            let modulo = builder.modulo(conservative, two.clone()).unwrap();
            let quotient = builder.floor_div(conservative, two).unwrap();
            conservative = builder
                .linear_combination(
                    0_i128.into(),
                    &[(2_i128.into(), quotient), (1_i128.into(), modulo)],
                )
                .unwrap();
        }
        if offset != 0 {
            conservative = builder
                .linear_combination(offset.into(), &[(1_i128.into(), conservative)])
                .unwrap();
        }
        let value = builder
            .read(input, &[first, second], &[conservative, second_coordinate])
            .unwrap();
        let write = builder
            .write(
                output,
                &[first, second],
                &[first_coordinate, second_coordinate],
            )
            .unwrap();
        builder.output(write, value).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(region.unknown_index_domain_predicates().len(), 1);
        region
    }

    fn assess(region: &VerifiedIndexRegion, budget: u64) -> IndexDomainProofClaim {
        let obligation = region
            .unknown_index_domain_predicates()
            .next()
            .expect("the fixture retains one residual");
        assess_finite_domain(
            region,
            obligation,
            IndexDomainProofBudget::try_new(budget).unwrap(),
        )
    }

    #[test]
    fn exact_finite_evaluation_proves_every_point() {
        let claim = assess(&residual_region(1, 5, 0), MAX_FINITE_DOMAIN_PROOF_CELLS);
        assert!(matches!(
            claim,
            IndexDomainProofClaim::Proved(IndexDomainProofEvidence::ExhaustiveFinite {
                points: LENGTH,
                ..
            })
        ));
    }

    #[test]
    fn exact_finite_evaluation_returns_the_first_counterexample() {
        let region = residual_region(1, 5, 1);
        let first = assess(&region, MAX_FINITE_DOMAIN_PROOF_CELLS);
        let second = assess(&region, MAX_FINITE_DOMAIN_PROOF_CELLS);
        assert_eq!(first, second);
        assert!(matches!(
            first,
            IndexDomainProofClaim::Disproved(disproof)
                if disproof.reason() == "logical-index-not-less-than-extent"
                    && disproof.point_ordinal() == Some(LENGTH - 1)
                    && !disproof.counterexample().is_empty()
        ));
    }

    #[test]
    fn exact_finite_evaluation_fails_closed_at_the_callers_budget() {
        let claim = assess(&residual_region(1, 5, 0), 1);
        assert!(matches!(
            claim,
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::Cells,
                required,
                limit: 1,
            }) if required > 1
        ));
    }

    #[test]
    fn invalid_budgets_are_rejected_before_evaluation() {
        assert_eq!(
            IndexDomainProofBudget::try_new(0),
            Err(IndexRefinementVerificationError::InvalidDomainProofBudget)
        );
        assert_eq!(
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS + 1),
            Err(IndexRefinementVerificationError::InvalidDomainProofBudget)
        );
    }

    #[test]
    fn a_symbolic_domain_is_unsupported_and_mints_no_proof() {
        let symbol = ShapeSymbol::new(SymbolScope::new("proof/0").unwrap(), "n").unwrap();
        let mut environment = ShapeEnvBuilder::new();
        environment.declare(symbol.clone()).unwrap();
        environment
            .bind(
                &symbol,
                RootBinding::new(
                    BindingSource::InterfaceParameter {
                        key: InterfaceParameterKey::new("n").unwrap(),
                    },
                    AvailabilityPhase::LiveDevicePreflight,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        environment
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(symbol.clone()), 1, 16).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        let environment = Arc::new(environment.build().unwrap());
        let mut builder = IndexRegionBuilder::new_with_shape_environment(
            FrozenScalarRegistry::standard().unwrap(),
            environment,
        )
        .unwrap();
        let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
        let input = builder
            .tensor(TensorRole::Input, value_type.clone(), Shape::from_dims([8]))
            .unwrap();
        let output = builder
            .sourced_tensor(
                TensorRole::Output,
                value_type,
                vec![SourcedExtent::Symbol(symbol.clone())],
            )
            .unwrap();
        let dimension = builder
            .symbolic_dimension(DomainRole::Parallel, symbol)
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(EXTENT_PHASE_CEILING, AvailabilityPhase::LiveDevicePreflight);
        assert!(matches!(
            assess(&region, MAX_FINITE_DOMAIN_PROOF_CELLS),
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
        ));
    }
}
