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
//! [`IndexRefinementVerifier::verify`] sees the complete semantic occurrence and
//! the actual [`VerifiedIndexRegion`], and [`IndexRefinementVerifier::complete`]
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
    OpKey, OperationAttributes, OperationEffect, RegistryError, ResolvedValueType,
    SemanticCapabilityAuthority, SemanticGraphIdentity, SemanticProgram,
};
use crate::shape::Shape;

use super::{
    CanonicalIndexRegionIdentity, CanonicalScalarDefinitionProjection, FrozenScalarRegistry,
    IndexDomainPredicate, IndexDomainUnknownReason, IndexExprView, IndexExtentRef, IndexInteger,
    IndexIntegerSign, ProofResource, ScalarAuthorityEvidence, ScalarOpKey, ScalarRegistryError,
    TensorRole, UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedIndexRegion, VerifiedScalarValueId, VerifiedTensorAccessId,
    VerifiedTensorId,
};

const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-receipt.v1\0";
const SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v1\0";
const AUTHORITY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-authority.v1\0";
const RESOLUTION_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-resolution.v1\0";
const PROOF_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-domain-proof.v1\0";
const EXHAUSTIVE_DERIVATION: &[u8] = b"tiler.ir.exact-index-domain-enumeration.v1\0";
/// Independent proof budget used when completing an otherwise checked receipt.
pub const MAX_REFINEMENT_PROOF_CELLS: u64 = 16 * 1024 * 1024;

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
pub struct NumericalContractIdentity(Vec<u8>);

impl NumericalContractIdentity {
    /// Identifies the numerical contract by its canonical key.
    #[must_use]
    pub fn from_key(key: &str) -> Self {
        Self(key.as_bytes().to_vec())
    }

    /// Returns the canonical numerical-contract identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRealizationAuthority {
    operation: OpKey,
    signature: IndexRefinementSignature,
    semantic: SemanticCapabilityAuthority,
    emitted_scalar_operations: Vec<ScalarOpKey>,
    emitted_scalar_definitions: CanonicalScalarDefinitionProjection,
    identity: Box<[u8]>,
}

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
        )
        .into_boxed_slice();
        Ok(Self {
            operation,
            signature,
            semantic: operation_authority,
            emitted_scalar_operations,
            emitted_scalar_definitions,
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
    /// Resolves this admitted authority for an exact program-derived subject.
    ///
    /// # Errors
    ///
    /// Returns a typed mismatch when the subject names another operation or
    /// signature.
    pub fn resolve(
        &self,
        subject: &IndexRefinementSubject,
    ) -> Result<ResolvedIndexRealization, IndexRefinementVerificationError> {
        if self.operation != subject.operation {
            return Err(IndexRefinementVerificationError::OperationMismatch);
        }
        if self.signature != subject.signature {
            return Err(IndexRefinementVerificationError::CapabilitySignatureMismatch);
        }
        let identity =
            encode_resolution_identity(&self.identity, &subject.identity).into_boxed_slice();
        Ok(ResolvedIndexRealization {
            authority: self.clone(),
            subject: subject.clone(),
            identity,
        })
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

/// One sealed capability resolution for an exact semantic subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedIndexRealization {
    authority: IndexRealizationAuthority,
    subject: IndexRefinementSubject,
    identity: Box<[u8]>,
}
impl ResolvedIndexRealization {
    /// Returns the exact governed subject.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        &self.subject
    }
    /// Returns the admitted authority.
    #[must_use]
    pub const fn authority(&self) -> &IndexRealizationAuthority {
        &self.authority
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

/// One independently verified residual-domain proof retained by a receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementDomainProof {
    obligation: UnknownIndexDomainPredicate,
    points: u64,
    identity: Box<[u8]>,
}

impl IndexRefinementDomainProof {
    /// Returns the exact region-owned obligation that was proved.
    #[must_use]
    pub const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }
    /// Returns the number of domain points evaluated exhaustively.
    #[must_use]
    pub const fn points(&self) -> u64 {
        self.points
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
    subject: IndexRefinementSubject,
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
        &self.subject
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
        self.subject == other.subject
            && self.resolution == other.resolution
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

/// IR-owned verifier for semantic occurrence/index-region association.
#[derive(Clone, Copy, Debug, Default)]
pub struct IndexRefinementVerifier;

impl IndexRefinementVerifier {
    /// Checks the occurrence and region together, minting no receipt while a
    /// residual index-domain obligation remains.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when scalar authority, effect, or ordered tensor
    /// interfaces disagree.
    pub fn verify(
        subject: &IndexRefinementSubject,
        resolution: &ResolvedIndexRealization,
        region: &VerifiedIndexRegion,
        scalars: &FrozenScalarRegistry,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        check_resolution_subject(subject, resolution)?;
        if subject.effect != OperationEffect::Pure {
            return Err(IndexRefinementVerificationError::EffectNotIndexable {
                effect: subject.effect,
            });
        }
        let scalar_authority = scalars.revalidate_region(region).map_err(|source| {
            IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
        })?;
        if scalar_authority.semantic_snapshot() != resolution.authority.semantic.registry_snapshot()
        {
            return Err(IndexRefinementVerificationError::SemanticAuthorityMismatch);
        }
        if scalar_authority.reached_operations().iter().any(|reached| {
            !resolution
                .authority
                .emitted_scalar_operations
                .contains(reached)
        }) {
            return Err(IndexRefinementVerificationError::ScalarAuthorityConformance);
        }
        let operand_bindings = bind_operands(subject, region)?;
        let result_bindings = bind_results(subject, region)?;
        if region.unknown_index_domain_predicates().len() != 0 {
            return Ok(IndexRefinementVerificationOutcome::Pending(Box::new(
                PendingIndexRefinementReceipt {
                    subject: subject.clone(),
                    resolution: resolution.clone(),
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
                resolution,
                region.canonical_identity(),
                scalar_authority,
                operand_bindings,
                result_bindings,
                Vec::new(),
            ),
        )))
    }

    /// Independently proves every retained obligation and mints the receipt.
    ///
    /// A disproved or unknown obligation consumes no pending state and mints no
    /// receipt. The caller retains its clone if it needs diagnostics or retry.
    ///
    /// # Errors
    ///
    /// Returns the first canonical disproved or unsupported obligation.
    pub fn complete(
        pending: PendingIndexRefinementReceipt,
    ) -> Result<IndexRefinementReceipt, IndexRefinementVerificationError> {
        let mut proofs = Vec::with_capacity(pending.obligations().len());
        for obligation in pending.obligations() {
            let points = prove_finite_domain(&pending.region, obligation)?;
            proofs.push(IndexRefinementDomainProof {
                obligation,
                points,
                identity: encode_proof_identity(&pending.region, obligation, points)
                    .into_boxed_slice(),
            });
        }
        Ok(mint_receipt(
            &pending.subject,
            &pending.resolution,
            pending.region.canonical_identity(),
            pending.scalar_authority,
            pending.operand_bindings,
            pending.result_bindings,
            proofs,
        ))
    }
}

/// Why IR-owned refinement verification refused to mint a receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRefinementVerificationError {
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
    /// Exact finite verification found a counterexample.
    IndexDomainDisproved {
        /// Exact region-local obligation that failed.
        obligation: UnknownIndexDomainPredicate,
        /// First canonical point where the predicate was false.
        point_ordinal: u64,
    },
    /// Exact finite verification could not prove an obligation.
    IndexDomainUnknown {
        /// Exact region-local obligation left open.
        obligation: Box<UnknownIndexDomainPredicate>,
        /// Why the verifier could not prove the obligation.
        reason: Box<IndexDomainUnknownReason>,
    },
}

impl fmt::Display for IndexRefinementVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            Self::IndexDomainDisproved { point_ordinal, .. } => write!(
                formatter,
                "index-domain obligation is false at point {point_ordinal}"
            ),
            Self::IndexDomainUnknown { reason, .. } => {
                write!(
                    formatter,
                    "index-domain obligation remains unknown: {reason:?}"
                )
            }
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

fn check_resolution_subject(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
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

fn prove_finite_domain(
    region: &VerifiedIndexRegion,
    obligation: UnknownIndexDomainPredicate,
) -> Result<u64, IndexRefinementVerificationError> {
    let access = region
        .access(obligation.subject())
        .expect("a verified residual names an access in its region");
    let dimensions = access
        .domain()
        .map(|dimension| {
            region
                .dimension(dimension)
                .expect("a verified access names its dimensions")
                .extent()
                .as_static()
                .map(|extent| (dimension, extent.get()))
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(IndexRefinementVerificationError::IndexDomainUnknown {
            obligation: Box::new(obligation),
            reason: Box::new(IndexDomainUnknownReason::UnsupportedFragment),
        })?;
    let points = dimensions
        .iter()
        .try_fold(1_u128, |product, (_, extent)| {
            product.checked_mul(u128::from(*extent))
        })
        .ok_or_else(|| proof_limit(obligation, u128::MAX))?;
    let expression = predicate_expression(obligation.predicate());
    let mut plan = HashSet::new();
    if !collect_expression_plan(region, expression, &mut plan) {
        return Err(IndexRefinementVerificationError::IndexDomainUnknown {
            obligation: Box::new(obligation),
            reason: Box::new(IndexDomainUnknownReason::UnsupportedFragment),
        });
    }
    let required = points.saturating_mul(plan.len() as u128);
    if required > u128::from(MAX_REFINEMENT_PROOF_CELLS) {
        return Err(proof_limit(obligation, required));
    }
    let points = u64::try_from(points).map_err(|_| proof_limit(obligation, required))?;
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
        let value = evaluate_expression(region, expression, &environment, &mut values).ok_or(
            IndexRefinementVerificationError::IndexDomainUnknown {
                obligation: Box::new(obligation),
                reason: Box::new(IndexDomainUnknownReason::UnsupportedFragment),
            },
        )?;
        if !predicate_holds(region, obligation.predicate(), &value) {
            return Err(IndexRefinementVerificationError::IndexDomainDisproved {
                obligation,
                point_ordinal,
            });
        }
        increment_coordinates(&mut coordinates, &dimensions);
    }
    Ok(points)
}

fn proof_limit(
    obligation: UnknownIndexDomainPredicate,
    required: u128,
) -> IndexRefinementVerificationError {
    IndexRefinementVerificationError::IndexDomainUnknown {
        obligation: Box::new(obligation),
        reason: Box::new(IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::Cells,
            required,
            limit: MAX_REFINEMENT_PROOF_CELLS,
        }),
    }
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
    match region
        .index_expression(expression)
        .expect("a verified predicate names its expression")
        .view()
    {
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
    let value = match region.index_expression(expression).ok()?.view() {
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
            .expect("a verified predicate names its dimension")
            .extent()
            .as_static()
            .expect("finite proof rejected symbolic dimensions")
            .get(),
        IndexExtentRef::TensorAxis { tensor, axis } => {
            let shape = region
                .tensor(tensor)
                .expect("a verified predicate names its tensor")
                .shape()
                .as_static()
                .expect("finite proof rejected symbolic boundaries");
            shape.extents()[usize::try_from(axis).expect("verified axis fits usize")].get()
        }
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
) -> Vec<u8> {
    let mut bytes = AUTHORITY_IDENTITY_TAG.to_vec();
    encode_op_key(&mut bytes, operation);
    encode_signature(&mut bytes, signature);
    push_slice(&mut bytes, semantic.reached_definitions().as_bytes());
    push_slice(&mut bytes, semantic.admission_provenance().as_bytes());
    push_slice(&mut bytes, semantic.registry_snapshot().as_bytes());
    push_slice(&mut bytes, scalar.as_bytes());
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
    points: u64,
) -> Vec<u8> {
    let mut bytes = PROOF_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region.canonical_identity().as_bytes());
    push_slice(&mut bytes, obligation.canonical_local_key().as_bytes());
    bytes.extend_from_slice(&points.to_be_bytes());
    push_slice(&mut bytes, EXHAUSTIVE_DERIVATION);
    bytes
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
