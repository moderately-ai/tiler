//! Checked semantic-occurrence to index-region refinement receipts.
//!
//! A verified index region proves structural safety, but does not by itself say
//! which semantic occurrence it realizes. This module owns the dependency-neutral
//! verifier that checks that association and mints an opaque receipt. Provider
//! selection, capability attribution, search, and explanation remain compiler
//! concerns layered above this receipt.
//!
//! A realization is an ordered [`VerifiedIndexRegionSequence`], not necessarily
//! one region: a family whose canonical form is a reduction feeding a pass over
//! the reduction's result is two regions with a value handed between them.
//! [`ResolvedIndexRealization::verify`] is the one-region spelling of
//! [`ResolvedIndexRealization::verify_sequence`], and a one-stage sequence's
//! identity is its region's identity, so nothing a single-region law ever minted
//! is changed by the sequence vocabulary's arrival.
//!
//! The public surface is a concrete alpha draft pending Tom's review. In
//! particular, callers cannot construct a receipt or its identity from bytes:
//! the verifier sees the complete semantic occurrence and the actual regions,
//! and [`ResolvedIndexRealization::complete`] independently discharges every
//! retained logical-index obligation before it mints a receipt.

use core::fmt;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};
#[cfg(test)]
use tiler_digest::DIGEST_BYTES;
use tiler_digest::DigestAlgorithm;

use crate::identity::{push_len, push_slice};
use crate::program::SemanticOccurrence;
use crate::schedule::{
    ArithmeticType, BF16_NUMERICAL_CONTRACT_KEY_DOMAIN, Bf16NumericalContractKey,
    F32NumericalContractKey, NumericalContractKeyError,
};
use crate::semantic::{
    EncodedComponentRole, FrozenSemanticRegistry, OpKey, OperationAttributes, OperationEffect,
    OperationId, ProviderIdentity, RegistryError, ResolvedValueType, SemanticCapabilityAuthority,
    SemanticGraphIdentity, SemanticProgram, SemanticRegistrySnapshotIdentity,
};
use crate::shape::Shape;

use super::{
    CanonicalIndexRegionIdentity, CanonicalIndexRegionSequenceIdentity,
    CanonicalScalarDefinitionProjection, CanonicalScalarRegistrySnapshotIdentity,
    FrozenScalarRegistry, IndexDomainPredicate, IndexDomainUnknownReason, IndexExprView,
    IndexExtentRef, IndexInteger, IndexRealizationLaw, MAX_BOUNDARY_TENSORS,
    ScalarAuthorityEvidence, ScalarOpKey, ScalarRegistryError, StagedInputSource, TensorRole,
    UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedIndexRegion, VerifiedIndexRegionSequence,
    VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
};

const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-receipt.v1\0";
const STAGED_RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-staged-receipt.v1\0";
const EXECUTABLE_COVERAGE_IDENTITY_TAG: &[u8] =
    b"tiler.ir.index-refinement-executable-coverage.v2\0";
const STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG: &[u8] =
    b"tiler.ir.index-refinement-staged-executable-coverage.v2\0";
/// Governed digest domain of the bound graph identity a coverage record folds.
///
/// [ADR 0104](../../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)
/// replaced the framed `SemanticGraphIdentity` preimage at the head of every
/// coverage record with a fixed-width digest under this domain, which is why
/// both tags above step to `v2`: a `v1` reader handed these bytes would read
/// thirty-two digest bytes as a length prefix and recover nothing.
///
/// **It is a separate domain rather than a reuse of either coverage tag** because
/// those two are *encoding* separators — the first bytes of a canonical run —
/// while this one is a *digest* separator, the first bytes of a pre-image. The
/// two kinds never have to be distinguished from each other, but two digests do:
/// this is the only subject this crate hashes, and any later one must be
/// checkably non-prefixing against it.
///
/// The no-prefix obligation `docs/artifact-abi.md` records normatively spans
/// every domain the workspace admits, because one algorithm hashes them all in
/// one process. It is discharged across crates by construction rather than by a
/// check neither crate could hold: this domain opens `tiler.ir.` and all eight
/// of `tiler-artifact`'s open `tiler.artifact-`, so no prefix relation between
/// the two sets is expressible.
const COVERAGE_GRAPH_DIGEST_DOMAIN: &[u8] = b"tiler.ir.index-refinement-coverage-graph.v1\0";
const SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v2\0";
#[cfg(test)]
const LEGACY_SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v1\0";
const AUTHORITY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-authority.v1\0";
const RESOLUTION_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-resolution.v1\0";
const PROOF_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-domain-proof.v1\0";
const LAW_REGISTRY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-law-registry.v1\0";
const MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES: usize = 256;
const MAX_DOMAIN_EVIDENCE_BYTES: usize = 4_096;
/// Maximum operands or results admitted on one refinement signature side.
pub const MAX_INDEX_REFINEMENT_SIGNATURE_VALUES: usize = 4_096;
/// Maximum operand-use bindings retained by one refinement receipt.
///
/// A binding associates one semantic operand use with one verified region input
/// boundary. The independent name is required because aliasing can produce
/// more bindings than distinct boundaries; the value deliberately matches the
/// region boundary population ceiling so an alias-expanded binding inventory
/// cannot exceed the boundary inventory the region itself may retain.
pub const MAX_INDEX_REFINEMENT_OPERAND_BINDINGS: usize = super::MAX_BOUNDARY_TENSORS;
/// Maximum raw scalar-operation declarations admitted by one authority.
pub const MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS: usize = 4_096;
/// Maximum residual obligations one canonical realization may retain, summed
/// over its stages.
///
/// The closed law vocabulary's widest single-region template emits three
/// rank-wide accesses, each with at most [`super::MAX_TENSOR_RANK`] coordinates
/// and two predicates per coordinate; rank-zero component reads retain no
/// coordinate obligations. Its widest staged template is a two-access fold
/// followed by a three-access pointwise pass, so five accesses is the widest any
/// realization of this vocabulary reaches, and the six below is a margin over
/// that rather than a tight bound. The bound is over the realization because
/// that is what one caller funds one completion budget for.
pub const MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS: usize = 6 * super::MAX_TENSOR_RANK * 2;
/// Maximum cells the closed exact-finite residual proof algorithm may evaluate.
pub const MAX_FINITE_DOMAIN_PROOF_CELLS: u64 = 16 * 1024 * 1024;
/// Maximum cumulative arbitrary-precision integer bytes the closed residual
/// proof algorithm may process.
pub const MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES: u64 = 64 * 1024 * 1024;
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
///
/// One opaque identity over the governed per-width contract key types, rather
/// than a public sum of them. The keys themselves stay siblings — distinct
/// types over mutually closed domains, because subnormal behaviour is
/// measurably per-dtype — and this type is what lets one
/// [`IndexRefinementSubject`] field hold whichever width its occurrence is
/// stated for. Keeping the sum private means every consumer discriminates
/// through [`Self::arithmetic`], whose [`ArithmeticType`] is deliberately not
/// `#[non_exhaustive]`, so a third admitted width is a build error at each such
/// site instead of falling through a match written before it existed.
///
/// **The identity bytes are the key spelling and nothing else.** No width tag is
/// written beside them: the two governed domains render mutually closed
/// preimages, so the spelling already determines the width, and a tag would move
/// every `f32` refinement receipt ever encoded to restate what the bytes say.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NumericalContractIdentity(NumericalContractKey);

/// The governed per-width contract key one refinement identity retains.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum NumericalContractKey {
    F32(F32NumericalContractKey),
    Bf16(Bf16NumericalContractKey),
}

impl NumericalContractIdentity {
    /// Validates and identifies one canonical coherent governed contract key.
    ///
    /// The governed domains are `tiler.contract.f32.v2` and
    /// `tiler.contract.bf16.v1`. They are mutually closed, so the rendered
    /// domain selects the parser and the reported refusal is that grammar's
    /// own: trying both and returning the second's error would name a grammar
    /// the caller never wrote.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when the key is not the exact IR-owned canonical
    /// spelling of a coherent contract vector in one of those domains.
    pub fn try_from_key(key: &str) -> Result<Self, IndexRefinementVerificationError> {
        if key.len() > MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES {
            return Err(
                IndexRefinementVerificationError::InvalidNumericalContractIdentity {
                    source: NumericalContractKeyError::InvalidCanonicalKey,
                },
            );
        }
        let parsed = if key.starts_with(BF16_NUMERICAL_CONTRACT_KEY_DOMAIN) {
            Bf16NumericalContractKey::try_from_str(key).map(Self::from)
        } else {
            // Every input that is not rendered under the `bf16` domain reaches
            // the `f32` parser exactly as it did before that domain existed, so
            // no previously admitted key and no previously reported refusal
            // moves.
            F32NumericalContractKey::try_from_str(key).map(Self::from)
        };
        parsed.map_err(|source| {
            IndexRefinementVerificationError::InvalidNumericalContractIdentity { source }
        })
    }

    /// Returns the canonical numerical-contract identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    /// Returns the validated UTF-8 contract key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            NumericalContractKey::F32(key) => key.as_str(),
            NumericalContractKey::Bf16(key) => key.as_str(),
        }
    }

    /// Returns the arithmetic type established by the canonical key grammar.
    #[must_use]
    pub const fn arithmetic(&self) -> ArithmeticType {
        match &self.0 {
            NumericalContractKey::F32(key) => key.arithmetic(),
            NumericalContractKey::Bf16(key) => key.arithmetic(),
        }
    }
}

impl From<F32NumericalContractKey> for NumericalContractIdentity {
    /// Retains an already-validated canonical `f32` contract key as refinement identity.
    fn from(key: F32NumericalContractKey) -> Self {
        Self(NumericalContractKey::F32(key))
    }
}

impl From<Bf16NumericalContractKey> for NumericalContractIdentity {
    /// Retains an already-validated canonical `bf16` contract key as refinement identity.
    fn from(key: Bf16NumericalContractKey) -> Self {
        Self(NumericalContractKey::Bf16(key))
    }
}

/// Exact operand/result signature admitted for one realization authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementSignature {
    operands: Vec<ResolvedValueType>,
    results: Vec<ResolvedValueType>,
}

/// Ordered side of a refinement signature.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexRefinementSignatureSide {
    /// Semantic operand boundary.
    Operands,
    /// Semantic result boundary.
    Results,
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
        let operands = operands
            .into_iter()
            .take(MAX_INDEX_REFINEMENT_SIGNATURE_VALUES + 1)
            .collect::<Vec<_>>();
        if operands.len() > MAX_INDEX_REFINEMENT_SIGNATURE_VALUES {
            return Err(IndexRefinementVerificationError::SignatureTooLarge {
                side: IndexRefinementSignatureSide::Operands,
                actual: operands.len(),
                limit: MAX_INDEX_REFINEMENT_SIGNATURE_VALUES,
            });
        }
        let results = results
            .into_iter()
            .take(MAX_INDEX_REFINEMENT_SIGNATURE_VALUES + 1)
            .collect::<Vec<_>>();
        if results.len() > MAX_INDEX_REFINEMENT_SIGNATURE_VALUES {
            return Err(IndexRefinementVerificationError::SignatureTooLarge {
                side: IndexRefinementSignatureSide::Results,
                actual: results.len(),
                limit: MAX_INDEX_REFINEMENT_SIGNATURE_VALUES,
            });
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
    realization_law_row: Option<Box<[u8]>>,
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
        if !semantic_authorities_cohere(semantic, scalars.semantic_authority()) {
            return Err(IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch);
        }
        if emitted.len() > MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS {
            return Err(
                IndexRefinementVerificationError::EmittedScalarOperationsTooLarge {
                    actual: emitted.len(),
                    limit: MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS,
                },
            );
        }
        let operation_authority = semantic
            .project_operation_authority(
                &operation,
                signature.operands.iter(),
                signature.results.iter(),
            )
            .map_err(|source| {
                IndexRefinementVerificationError::SemanticAuthority(Arc::new(source))
            })?;
        let realization_law_row = semantic.encode_index_realization_law_row_for(&operation);
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
            realization_law_row.as_deref(),
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
            realization_law_row,
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
    realization_law_row: Option<Box<[u8]>>,
    identity: Box<[u8]>,
}

impl IndexRefinementSubject {
    /// Derives one exact subject from a verified semantic graph and operation handle.
    ///
    /// The handle is a transient selector owned by `program`; the retained
    /// [`SemanticOccurrence`] is the selected operation's canonical traversal
    /// ordinal and is the only operation coordinate included in durable identity.
    ///
    /// # Errors
    ///
    /// Returns a typed semantic-program error when the handle or one of its
    /// referenced values cannot be resolved, or its signature exceeds bounds.
    pub fn derive(
        program: &SemanticProgram,
        operation: OperationId,
        numerical_contract: NumericalContractIdentity,
    ) -> Result<Self, IndexRefinementVerificationError> {
        let operation_ref = program
            .operation(operation)
            .map_err(IndexRefinementVerificationError::SemanticHandle)?;
        let occurrence =
            SemanticOccurrence::new(program.canonical_operation_ordinal(operation_ref));
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
        let realization_law_row = program
            .semantic_registry()
            .encode_index_realization_law_row_for(&operation);
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
            realization_law_row,
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

/// Canonical identity of one frozen semantic realization-law registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRealizationLawRegistryIdentity(Box<[u8]>);

impl IndexRealizationLawRegistryIdentity {
    /// Returns the canonical registry identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

struct FrozenIndexRealizationLawRegistryData {
    semantic: FrozenSemanticRegistry,
    scalars: FrozenScalarRegistry,
    identity: IndexRealizationLawRegistryIdentity,
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

fn semantic_authorities_cohere(
    semantic: &FrozenSemanticRegistry,
    scalar_semantic: &FrozenSemanticRegistry,
) -> bool {
    semantic.snapshot_identity() == scalar_semantic.snapshot_identity()
        && semantic.encode_index_realization_law_sidecar()
            == scalar_semantic.encode_index_realization_law_sidecar()
}

impl FrozenIndexRealizationLawRegistry {
    /// Derives the law snapshot inseparably retained by one semantic registry.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch`]
    /// when `scalars` was built over a different semantic authority, including
    /// one with equal semantic snapshot bytes but different realization laws.
    pub fn from_semantic(
        semantic: FrozenSemanticRegistry,
        scalars: FrozenScalarRegistry,
    ) -> Result<Self, IndexRefinementVerificationError> {
        if !semantic_authorities_cohere(&semantic, scalars.semantic_authority()) {
            return Err(IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch);
        }
        let mut identity = Vec::new();
        identity.extend_from_slice(LAW_REGISTRY_IDENTITY_TAG);
        push_slice(&mut identity, semantic.snapshot_identity().as_bytes());
        push_slice(&mut identity, scalars.snapshot_identity().as_bytes());
        identity.extend_from_slice(&semantic.encode_index_realization_law_sidecar());
        Ok(Self(Arc::new(FrozenIndexRealizationLawRegistryData {
            semantic,
            scalars,
            identity: IndexRealizationLawRegistryIdentity(identity.into_boxed_slice()),
        })))
    }

    /// Returns the exact canonical registry identity.
    #[must_use]
    pub fn identity(&self) -> &IndexRealizationLawRegistryIdentity {
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

    /// Returns whether the law registered for one operation family realizes a
    /// region *sequence* rather than a single region.
    ///
    /// **Accepted public surface** — by Tom on 2026-08-06 at the live session's
    /// decision round, as-is with no exclusion;
    /// [`accept-the-registered-family-region-sequence-query`](../../../../tickets/accept-the-registered-family-region-sequence-query.md)
    /// records the provenance.
    ///
    /// **The question is about the registered law and nothing else, which is why
    /// it takes an operation key rather than a subject.** A caller that already
    /// holds an [`IndexRefinementSubject`] asks
    /// [`ResolvedIndexRealization::realizes_region_sequence`] through
    /// [`Self::resolve`] and gets the same answer off the same registry row; a
    /// caller classifying a *program* — deciding whether an occurrence is one
    /// region's worth of work or a staged one — has no subject to derive,
    /// because deriving one requires a numerical contract and the classification
    /// does not depend on one. [`super::IndexRealizationLaw`]'s own predicate
    /// reads the variant alone, so answering it here from the operation key is
    /// the same fact read from the same place rather than a second derivation.
    ///
    /// Answers `false` for an operation the registry carries no law for. That is
    /// the fail-closed direction and not an approximation: an occurrence with no
    /// registered law realizes no region sequence this authority can describe,
    /// and refinement reports the absent law by name when the occurrence is
    /// lowered.
    #[must_use]
    pub fn family_realizes_region_sequence(&self, operation: &OpKey) -> bool {
        self.0
            .semantic
            .index_realization_law(operation)
            .is_some_and(|registered| registered.law.realizes_region_sequence())
    }

    /// Returns the law registered for one operation family, if it carries one.
    ///
    /// **Accepted public surface** — by Tom on 2026-08-06 at the live session's
    /// decision round, as-is with no exclusion;
    /// [`accept-the-registered-family-realization-law-query`](../../../../tickets/accept-the-registered-family-realization-law-query.md)
    /// records the provenance.
    ///
    /// **Why a caller needs the law itself and not a predicate over it.** A
    /// physical planner spelling a staged family's stage has to know *what that
    /// stage computes* — which axes it folds, which payload its epilogue carries —
    /// and that is the law's content. Deriving it from the operation key instead
    /// would key the planner to a family, so a second family registering this law
    /// would need a second arm for one template; deriving it from the shapes is
    /// not possible at all, because a `[2, 2]` input reduced to `[2]` names two
    /// different reductions. Answering with the closed typed law is what lets a
    /// consumer be written against the *vocabulary* — one arm per law, a
    /// fail-closed wildcard for the rest — which is the same discipline this
    /// module's own interpretation follows.
    ///
    /// It takes an operation key for the reason
    /// [`Self::family_realizes_region_sequence`] does, reads the same registry row
    /// that method and [`Self::resolve`] read, and is deliberately *not* a
    /// resolution: it performs no contract check, no authority projection, and no
    /// realization, so it answers what is registered rather than what a subject
    /// may have. A caller acting on the answer still resolves.
    ///
    /// `None` for an operation the registry carries no law for, which is the
    /// fail-closed direction: an occurrence with no registered law has no
    /// realization this authority describes.
    #[must_use]
    pub fn family_realization_law(&self, operation: &OpKey) -> Option<&IndexRealizationLaw> {
        self.0
            .semantic
            .index_realization_law(operation)
            .map(|registered| &registered.law)
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
        let registry_row = self
            .0
            .semantic
            .encode_index_realization_law_row_for(&subject.operation);
        if registry_row != subject.realization_law_row {
            return Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch);
        }
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
    /// Returns whether this law realizes a region *sequence*.
    ///
    /// The cheap half of [`Self::realize_sequence`]'s answer: a consumer that
    /// only wants single-region occurrences filtered out asks this before
    /// paying for a realization it would discard.
    #[must_use]
    pub const fn realizes_region_sequence(&self) -> bool {
        self.law.realizes_region_sequence()
    }

    /// Realizes the resolved law's canonical region sequence for its subject.
    ///
    /// The same realization refinement performs internally when it compares a
    /// provider's emission against the law, over the same law, subject, and
    /// frozen scalar authority this resolution already binds — exposed so a
    /// consumer that needs the realization's *shape* (its stage count, each
    /// stage's reads, and the handed values) reads it from the one authority
    /// that owns it instead of deriving a second account of the law.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::SemanticRealizationLawRefused`]
    /// carrying the law's own refusal rule when the subject does not realize —
    /// the identical refusal refinement reports for the same subject.
    pub fn realize_sequence(
        &self,
    ) -> Result<super::VerifiedIndexRegionSequence, IndexRefinementVerificationError> {
        self.law
            .realize_sequence(&self.subject, &self.registry.0.scalars)
            .map_err(
                |source| IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(self.subject.operation().clone()),
                    rule: source.rule(),
                },
            )
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

/// One ordered operand projection bound to its verified region input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperandBinding {
    stage: usize,
    operand: usize,
    input: usize,
    input_tensor: VerifiedTensorId,
    component_role: Option<EncodedComponentRole>,
}

impl OperandBinding {
    /// Returns the ordered realization stage that reads this operand.
    ///
    /// A tensor handle is region-local, so this is what says which stage's
    /// region [`Self::input_tensor`] resolves against. One occurrence input read
    /// by two stages produces one binding per reading stage.
    #[must_use]
    pub const fn stage(&self) -> usize {
        self.stage
    }
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
    /// Returns the encoded logical component carried by this input tensor.
    ///
    /// `None` names an ordinary whole-value input. An encoded operand produces
    /// one binding per component in its contract's semantic order.
    #[must_use]
    pub const fn component_role(&self) -> Option<EncodedComponentRole> {
        self.component_role
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

/// Reached-only executable provenance minted with one completed refinement receipt.
///
/// ```compile_fail
/// use tiler_ir::index::IndexRefinementExecutableCoverageIdentity;
///
/// // Executable coverage is proof-derived; opaque bytes are not a constructor.
/// let _ = IndexRefinementExecutableCoverageIdentity(Box::new([]));
/// ```
///
/// ```compile_fail
/// use tiler_ir::index::IndexRefinementExecutableCoverageIdentity;
///
/// // Nor is there a byte-level conversion: a caller holding one receipt's
/// // coverage bytes cannot re-mint them onto another receipt's occurrence.
/// fn cross(bytes: &[u8]) -> IndexRefinementExecutableCoverageIdentity {
///     IndexRefinementExecutableCoverageIdentity::from(bytes)
/// }
/// ```
///
/// This identity deliberately excludes the complete semantic, scalar, and
/// realization-law registry snapshots retained by [`IndexRefinementReceiptIdentity`].
/// It retains the selected graph occurrence, numerical contract, realization
/// law and provider, the realization's regions, reached semantic/scalar/type
/// definition and admission projections, exact operand/result bindings, and
/// every residual proof identity. Callers may read these bytes but cannot
/// construct this type from bytes or independently supplied fields.
///
/// The operation key, ordered signature, host-canonical attributes, and operand
/// and result boundary shapes are not re-encoded: `tiler.semantic-graph.v2`
/// already writes each of them for every operation in canonical traversal
/// order, and [`IndexRefinementSubject::derive`] fixes the retained occurrence
/// to that same canonical ordinal. Encoding them a second time would restate
/// what the graph and occurrence pair already determines rather than close a
/// substitution the pair leaves open.
///
/// **The graph half of that pair is named by digest rather than restated**, as
/// of `v2` and ADR 0104: the record opens with a fixed-width governed digest of
/// the bound graph's identity instead of the identity itself. What the pair
/// determines is unchanged — two receipts for one occurrence ordinal of two
/// different graphs still mint different bytes — and what changes is that the
/// graph identity is no longer recoverable from these bytes, which nothing in
/// the workspace attempted. The restatement was the whole of kernel-program
/// identity's quadratic term — one graph identity per record, one record per
/// semantic operation — and folding it makes that curve linear; `encode_executable_coverage_identity`
/// carries the derivation and `docs/artifact-abi.md` the measured constants.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementExecutableCoverageIdentity(Box<[u8]>);

impl IndexRefinementExecutableCoverageIdentity {
    /// Returns the canonical reached-only executable-coverage bytes.
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
    max_integer_bytes: u64,
}

impl IndexDomainProofBudget {
    /// Creates a nonzero budget no larger than IR's hard proof bound.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRefinementVerificationError::InvalidDomainProofBudget`]
    /// with the exact resource, supplied value, and hard limit when either
    /// limit is zero or exceeds its corresponding
    /// [`MAX_FINITE_DOMAIN_PROOF_CELLS`] or
    /// [`MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES`] hard bound.
    pub fn try_new(
        max_cells: u64,
        max_integer_bytes: u64,
    ) -> Result<Self, IndexRefinementVerificationError> {
        if max_cells == 0 || max_cells > MAX_FINITE_DOMAIN_PROOF_CELLS {
            return Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: super::ProofResource::Cells,
                actual: max_cells,
                limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
            });
        }
        if max_integer_bytes == 0 || max_integer_bytes > MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES {
            return Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: super::ProofResource::IntegerBytes,
                actual: max_integer_bytes,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            });
        }
        Ok(Self {
            max_cells,
            max_integer_bytes,
        })
    }

    /// Returns the maximum cumulative structural and evaluation work cells.
    ///
    /// This includes domain and extent resolution, expression planning,
    /// coordinate initialization/advance, DAG traversal, memo clearing, and
    /// predicate evaluation—not merely expression nodes.
    #[must_use]
    pub const fn max_cells(self) -> u64 {
        self.max_cells
    }

    /// Returns the maximum cumulative integer-byte work the proof may perform.
    #[must_use]
    pub const fn max_integer_bytes(self) -> u64 {
        self.max_integer_bytes
    }
}

/// A typed exact counterexample from IR's closed domain evaluator.
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

/// IR's total claim about one exact residual obligation.
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
    authority: Arc<IndexDomainProofAuthority>,
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
    pub fn authority(&self) -> &IndexDomainProofAuthority {
        &self.authority
    }
    /// Returns the verifier's total claim.
    #[must_use]
    pub const fn claim(&self) -> &IndexDomainProofClaim {
        &self.claim
    }
}

/// One IR-sealed residual-domain proof retained by a refinement receipt.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexRefinementDomainProofIdentity(Box<[u8]>);

impl IndexRefinementDomainProofIdentity {
    /// Returns the canonical proof identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One IR-sealed residual-domain proof retained by a refinement receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementDomainProof {
    stage: usize,
    obligation: UnknownIndexDomainPredicate,
    authority: Arc<IndexDomainProofAuthority>,
    proof: IndexDomainProofEvidence,
    identity: IndexRefinementDomainProofIdentity,
}

impl IndexRefinementDomainProof {
    /// Returns the ordered realization stage that retained the obligation.
    ///
    /// An obligation is region-local, so this is what says which region its
    /// handles resolve against.
    #[must_use]
    pub const fn stage(&self) -> usize {
        self.stage
    }
    /// Returns the exact region-owned obligation that was proved.
    #[must_use]
    pub const fn obligation(&self) -> UnknownIndexDomainPredicate {
        self.obligation
    }
    /// Returns the authority that proved the obligation.
    #[must_use]
    pub fn authority(&self) -> &IndexDomainProofAuthority {
        &self.authority
    }
    /// Returns the retained proof basis.
    #[must_use]
    pub const fn proof(&self) -> &IndexDomainProofEvidence {
        &self.proof
    }
    /// Returns the canonical proof identity.
    #[must_use]
    pub const fn identity(&self) -> &IndexRefinementDomainProofIdentity {
        &self.identity
    }
}

/// Opaque checked binding of one semantic occurrence to one verified
/// realization.
///
/// A realization is an *ordered sequence* of verified regions, and every field
/// below that names a region names all of them. [`Self::final_stage`] and
/// [`Self::final_scalar_authority`] answer the last stage alone and never the
/// realization; a consumer that must see the whole chain reads
/// [`Self::regions`], [`Self::scalar_authorities`], or [`Self::realization`].
/// The accessors are named for what they return because the one-stage
/// realization every registered law produces makes stage and realization
/// indistinguishable, and a reader who learned the accessor there would
/// otherwise carry that reading into the first chain met.
#[derive(Clone, Debug)]
pub struct IndexRefinementReceipt {
    graph: SemanticGraphIdentity,
    occurrence: SemanticOccurrence,
    /// Every region identity except the final stage's, in stage order.
    ///
    /// Split the way [`VerifiedIndexRegionSequence`] splits its stages, so the
    /// stage whose writes are the occurrence's results is a field rather than a
    /// lookup a reader has to establish cannot fail.
    leading_regions: Vec<CanonicalIndexRegionIdentity>,
    region: CanonicalIndexRegionIdentity,
    realization: CanonicalIndexRegionSequenceIdentity,
    leading_scalar_authorities: Vec<ScalarAuthorityEvidence>,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
    identity: IndexRefinementReceiptIdentity,
    executable_coverage_identity: IndexRefinementExecutableCoverageIdentity,
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
    /// Returns the final stage's verified-region identity.
    ///
    /// The final stage is the one whose writes are the occurrence's results. For
    /// a one-stage realization it is the only region, and its identity is the
    /// realization identity byte for byte. For a chain it identifies one stage
    /// of several and is **not** the realization: the leading stages leave no
    /// trace in it, so two chains that merely end alike agree here. Compare
    /// [`Self::realization`] to compare realizations, and read [`Self::regions`]
    /// for every stage in order.
    #[must_use]
    pub const fn final_stage(&self) -> &CanonicalIndexRegionIdentity {
        &self.region
    }
    /// Returns every verified-region identity in stage order.
    #[must_use]
    pub fn regions(&self) -> Vec<CanonicalIndexRegionIdentity> {
        let mut regions = self.leading_regions.clone();
        regions.push(self.region.clone());
        regions
    }
    /// Returns the exact canonical identity of the whole ordered realization.
    #[must_use]
    pub const fn realization(&self) -> &CanonicalIndexRegionSequenceIdentity {
        &self.realization
    }
    /// Returns the checked scalar authority bound to the final stage alone.
    ///
    /// A chain's stages reach their own scalar operations and need not overlap:
    /// the governed staged template's fold reaches the add and its pass reaches
    /// the multiply, and neither reaches the other's. So this is one stage's
    /// reached vocabulary, not the realization's;
    /// [`Self::scalar_authorities`] answers that, in stage order.
    #[must_use]
    pub const fn final_scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }
    /// Returns the checked scalar authority of every stage, in stage order.
    #[must_use]
    pub fn scalar_authorities(&self) -> Vec<ScalarAuthorityEvidence> {
        let mut authorities = self.leading_scalar_authorities.clone();
        authorities.push(self.scalar_authority.clone());
        authorities
    }
    /// Returns ordered operand-to-input bindings.
    ///
    /// An encoded logical operand contributes one binding for every component
    /// in its semantic contract order; an ordinary operand contributes one.
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
    /// Returns reached-only provenance suitable for executable coverage.
    ///
    /// Unlike [`Self::identity`], this subject excludes unused registry rows.
    /// Its only minting path is successful receipt completion.
    #[must_use]
    pub const fn executable_coverage_identity(&self) -> &IndexRefinementExecutableCoverageIdentity {
        &self.executable_coverage_identity
    }
}

/// Checked association awaiting proof of retained index-domain obligations.
///
/// A pending association has no executable-coverage spelling. Only
/// [`ResolvedIndexRealization::complete`] discharges the retained obligations,
/// and only its success value carries an
/// [`IndexRefinementExecutableCoverageIdentity`]:
///
/// ```compile_fail
/// fn coverage(
///     pending: &tiler_ir::index::PendingIndexRefinementReceipt,
/// ) -> &tiler_ir::index::IndexRefinementExecutableCoverageIdentity {
///     pending.executable_coverage_identity()
/// }
/// ```
#[derive(Clone, Debug)]
pub struct PendingIndexRefinementReceipt {
    resolution: ResolvedIndexRealization,
    /// Every stage's evidence except the final one's, in stage order.
    ///
    /// Split the same way [`VerifiedIndexRegionSequence`] splits its stages: a
    /// realization always has a final stage, so its evidence is a field rather
    /// than a lookup that could fail.
    leading_scalar_authorities: Vec<ScalarAuthorityEvidence>,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    realization: VerifiedIndexRegionSequence,
}

impl PendingIndexRefinementReceipt {
    /// Returns the checked semantic occurrence.
    #[must_use]
    pub const fn subject(&self) -> &IndexRefinementSubject {
        self.resolution.subject()
    }
    /// Returns the exact retained final-stage verified region.
    ///
    /// For the one-stage realization every registered law produces this is the
    /// only region, and evaluating it evaluates the occurrence. For a chain it
    /// is one stage of several and evaluating it does not: at least one of its
    /// input boundaries reads the value the preceding stage handed on, which no
    /// operand named by [`Self::operand_bindings`] carries. A consumer that can
    /// run exactly one region must therefore establish that
    /// [`Self::realization`] has exactly one stage before it runs this one —
    /// otherwise it runs part of a realization and reports the result as the
    /// occurrence's.
    #[must_use]
    pub const fn final_stage(&self) -> &VerifiedIndexRegion {
        self.realization.final_stage()
    }
    /// Returns the exact retained ordered realization.
    #[must_use]
    pub const fn realization(&self) -> &VerifiedIndexRegionSequence {
        &self.realization
    }
    /// Returns the final stage's checked scalar authority evidence alone.
    ///
    /// Each stage carries its own reached vocabulary, so for a chain this omits
    /// every scalar operation only a leading stage reaches;
    /// [`Self::scalar_authorities`] answers the whole realization, in stage
    /// order.
    #[must_use]
    pub const fn final_scalar_authority(&self) -> &ScalarAuthorityEvidence {
        &self.scalar_authority
    }
    /// Returns every stage's checked scalar authority evidence, in stage order.
    #[must_use]
    pub fn scalar_authorities(&self) -> Vec<ScalarAuthorityEvidence> {
        let mut authorities = self.leading_scalar_authorities.clone();
        authorities.push(self.scalar_authority.clone());
        authorities
    }
    /// Returns ordered operand bindings, expanding encoded components in their
    /// semantic contract order.
    #[must_use]
    pub fn operand_bindings(&self) -> &[OperandBinding] {
        &self.operand_bindings
    }
    /// Returns ordered result bindings.
    #[must_use]
    pub fn result_bindings(&self) -> &[ResultBinding] {
        &self.result_bindings
    }
    /// Returns every exact residual obligation, in stage order and within a
    /// stage in canonical region order.
    ///
    /// An obligation is region-local, so a caller reading this flat sequence
    /// needs [`Self::staged_obligations`] to know which stage each belongs to;
    /// the flat order is what a completed receipt's proofs are aligned against.
    #[must_use]
    pub fn obligations(&self) -> impl ExactSizeIterator<Item = UnknownIndexDomainPredicate> + '_ {
        self.staged_obligations()
            .into_iter()
            .map(|(_, obligation)| obligation)
    }

    /// Returns every residual obligation paired with the stage that retains it.
    #[must_use]
    pub fn staged_obligations(&self) -> Vec<(usize, UnknownIndexDomainPredicate)> {
        self.realization
            .stages()
            .enumerate()
            .flat_map(|(stage, region)| {
                region
                    .unknown_index_domain_predicates()
                    .map(move |obligation| (stage, obligation))
            })
            .collect()
    }

    /// Revalidates that a completed receipt was minted from this exact pending
    /// association and its canonical residual obligations.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when any occurrence, region, authority,
    /// interface, proof, or identity field was crossed with another pending
    /// association.
    pub fn verify_completion(
        &self,
        receipt: &IndexRefinementReceipt,
    ) -> Result<(), IndexRefinementVerificationError> {
        let subject = self.subject();
        if receipt.graph != subject.graph || receipt.occurrence != subject.occurrence {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.realization != *self.realization.identity() {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        let bound_regions = receipt.regions();
        if bound_regions.len() != self.realization.stage_count()
            || bound_regions
                .iter()
                .zip(self.realization.stages())
                .any(|(bound, stage)| bound != stage.canonical_identity())
        {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.scalar_authorities() != self.scalar_authorities() {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.operand_bindings != self.operand_bindings {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        if receipt.result_bindings != self.result_bindings {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        let obligations = self.staged_obligations();
        if receipt.index_domain_proofs.len() != obligations.len()
            || receipt.index_domain_proofs.iter().zip(obligations).any(
                |(proof, (stage, obligation))| {
                    proof.obligation != obligation || proof.stage != stage
                },
            )
        {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        let expected = encode_receipt_identity(
            subject,
            &self.resolution,
            &self.realization,
            &self.scalar_authorities(),
            &receipt.index_domain_proofs,
        );
        if receipt.identity.as_bytes() != expected {
            return Err(IndexRefinementVerificationError::CompletionReceiptMismatch);
        }
        Ok(())
    }
}

impl PartialEq for PendingIndexRefinementReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.resolution == other.resolution
            && self.leading_scalar_authorities == other.leading_scalar_authorities
            && self.scalar_authority == other.scalar_authority
            && self.operand_bindings == other.operand_bindings
            && self.result_bindings == other.result_bindings
            && self.realization == other.realization
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
    /// Returns
    /// [`IndexRefinementVerificationError::SemanticRealizationLawRefused`] under
    /// `staged-law-requires-region-sequence` when the registered law realizes a
    /// region *sequence*, which one region cannot satisfy — [`Self::verify_sequence`]
    /// is the method for those. Otherwise returns a typed refusal when scalar
    /// authority, effect, or ordered tensor interfaces disagree.
    pub fn verify(
        &self,
        lowering: &IndexRealizationAuthority,
        region: &VerifiedIndexRegion,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        if self.law.realizes_region_sequence() {
            // Named before anything else looks at the region. A staged law's
            // final stage reads a value no occurrence input carries, so the
            // ordinary interface check would refuse a lone region by naming that
            // boundary — a true statement that sends a reader to the provider's
            // tensor list instead of to the arity of what it was asked for.
            return Err(
                IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(self.subject.operation().clone()),
                    rule: "staged-law-requires-region-sequence",
                },
            );
        }
        self.verify_sequence(
            lowering,
            &VerifiedIndexRegionSequence::single(region.clone()),
        )
    }

    /// Checks the occurrence against an ordered multi-region realization.
    ///
    /// The candidate must be the exact canonical region *sequence* the
    /// registered law constructs: the stages in order, each stage's own
    /// canonical region identity, and the source every stage input is bound to.
    /// A truncated chain, a reordered one, and one whose stages are individually
    /// correct but wired differently each render a different sequence identity
    /// and are refused for that reason alone.
    ///
    /// [`Self::verify`] is this method over a one-stage sequence, and a one-stage
    /// sequence's identity is its region's identity, so the two paths agree
    /// byte for byte on everything single-region verification ever accepted.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal when scalar authority, effect, ordered tensor
    /// interfaces, or the realized sequence disagree.
    pub fn verify_sequence(
        &self,
        lowering: &IndexRealizationAuthority,
        realization: &VerifiedIndexRegionSequence,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        let subject = &self.subject;
        check_lowering_authority(subject, self, lowering)?;
        if subject.effect != OperationEffect::Pure {
            return Err(IndexRefinementVerificationError::EffectNotIndexable {
                effect: subject.effect,
            });
        }
        // Per stage, so the containment check covers the union of everything the
        // realization reaches. A stage reaching an unadmitted scalar operation
        // refuses the whole realization: the admission is what the emitted
        // program is checked against, and a chain is one program.
        let stage_authority = |stage: &VerifiedIndexRegion| {
            let evidence = self
                .registry
                .0
                .scalars
                .revalidate_region(stage)
                .map_err(|source| {
                    IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
                })?;
            if evidence.scalar_snapshot() != self.registry.0.scalars.snapshot_identity() {
                return Err(IndexRefinementVerificationError::ScalarSnapshotMismatch);
            }
            if evidence
                .reached_operations()
                .iter()
                .any(|reached| !lowering.emitted_scalar_operations.contains(reached))
            {
                return Err(IndexRefinementVerificationError::ScalarAuthorityConformance);
            }
            Ok(evidence)
        };
        let mut leading_scalar_authorities = Vec::with_capacity(realization.leading_stages().len());
        for stage in realization.leading_stages() {
            leading_scalar_authorities.push(stage_authority(stage)?);
        }
        let scalar_authority = stage_authority(realization.final_stage())?;
        let mut scalar_authorities = leading_scalar_authorities.clone();
        scalar_authorities.push(scalar_authority.clone());
        let operand_bindings = bind_operands(subject, realization)?;
        let result_bindings = bind_results(subject, realization.final_stage())?;
        if !self.law.accepts_numerical_contract(subject) {
            return Err(IndexRefinementVerificationError::NumericalContractNotGoverned);
        }
        let expected = self
            .law
            .realize_sequence(subject, &self.registry.0.scalars)
            .map_err(
                |source| IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    operation: Box::new(subject.operation().clone()),
                    rule: source.rule(),
                },
            )?;
        if expected.identity() != realization.identity() {
            // A one-stage expectation against a one-stage candidate reports the
            // region identities the single-region refusal has always reported;
            // anything else reports the whole chain, because naming one stage of
            // a mismatched chain would hide which part disagreed.
            return Err(
                if expected.is_single_stage() && realization.is_single_stage() {
                    IndexRefinementVerificationError::SemanticRealizationMismatch {
                        expected: expected.final_stage().canonical_identity().clone(),
                        actual: realization.final_stage().canonical_identity().clone(),
                    }
                } else {
                    IndexRefinementVerificationError::SemanticRealizationSequenceMismatch {
                        expected: expected.identity().clone(),
                        actual: realization.identity().clone(),
                    }
                },
            );
        }
        let residual_obligations = realization
            .stages()
            .map(|stage| stage.unknown_index_domain_predicates().len())
            .try_fold(0_usize, usize::checked_add)
            .unwrap_or(usize::MAX);
        check_residual_obligation_count(residual_obligations)?;
        if residual_obligations != 0 {
            return Ok(IndexRefinementVerificationOutcome::Pending(Box::new(
                PendingIndexRefinementReceipt {
                    resolution: self.clone(),
                    leading_scalar_authorities,
                    scalar_authority,
                    operand_bindings,
                    result_bindings,
                    realization: realization.clone(),
                },
            )));
        }
        Ok(IndexRefinementVerificationOutcome::Verified(Box::new(
            mint_receipt(
                subject,
                self,
                realization,
                scalar_authorities,
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
    /// Returns an atomic refusal retaining every canonical assessment when any
    /// obligation is disproved or unsupported.
    pub fn complete(
        pending: &PendingIndexRefinementReceipt,
        budget: IndexDomainProofBudget,
    ) -> Result<(IndexRefinementReceipt, Vec<IndexDomainProofAssessment>), IndexDomainProofRefusal>
    {
        let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
        // One ledger for the whole realization: the caller funded one budget,
        // and a per-stage ledger would silently multiply it by the stage count.
        let mut ledger = IndexDomainProofLedger::new(budget);
        // Each retained obligation stays paired with the region its handles
        // resolve against, so nothing downstream re-derives that association.
        let mut owners: Vec<(usize, &VerifiedIndexRegion)> = Vec::new();
        let mut assessments = Vec::new();
        for (stage, region) in pending.realization.stages().enumerate() {
            let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
            let claims = assess_finite_domains_with(region, &obligations, &mut ledger);
            for (obligation, claim) in obligations.into_iter().zip(claims) {
                owners.push((stage, region));
                assessments.push(IndexDomainProofAssessment {
                    obligation,
                    authority: authority.clone(),
                    claim,
                });
            }
        }
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
        let mut proofs = Vec::with_capacity(assessments.len());
        for (assessment, (stage, region)) in assessments.iter().zip(&owners) {
            let IndexDomainProofClaim::Proved(proof) = &assessment.claim else {
                unreachable!("the refusal scans removed every non-proof claim")
            };
            proofs.push(IndexRefinementDomainProof {
                stage: *stage,
                obligation: assessment.obligation,
                authority: assessment.authority.clone(),
                proof: proof.clone(),
                identity: IndexRefinementDomainProofIdentity(
                    encode_proof_identity(
                        region,
                        assessment.obligation,
                        &assessment.authority,
                        proof,
                    )
                    .into_boxed_slice(),
                ),
            });
        }
        Ok((
            mint_receipt(
                pending.resolution.subject(),
                &pending.resolution,
                &pending.realization,
                pending.scalar_authorities(),
                pending.operand_bindings.clone(),
                pending.result_bindings.clone(),
                proofs,
            ),
            assessments,
        ))
    }
}

fn check_residual_obligation_count(actual: usize) -> Result<(), IndexRefinementVerificationError> {
    if actual > MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS {
        return Err(
            IndexRefinementVerificationError::ResidualObligationsTooLarge {
                actual,
                limit: MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS,
            },
        );
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct IndexDomainProofExhaustion {
    resource: super::ProofResource,
    required: u128,
    limit: u64,
}

struct IndexDomainProofLedger {
    cell_limit: u64,
    integer_byte_limit: u64,
    used_cells: u128,
    used_integer_bytes: u128,
    exhaustion: Option<IndexDomainProofExhaustion>,
}

impl IndexDomainProofLedger {
    const fn new(budget: IndexDomainProofBudget) -> Self {
        Self {
            cell_limit: budget.max_cells(),
            integer_byte_limit: budget.max_integer_bytes(),
            used_cells: 0,
            used_integer_bytes: 0,
            exhaustion: None,
        }
    }

    fn debit(
        &mut self,
        resource: super::ProofResource,
        amount: u128,
    ) -> Result<(), ProofPlanningFailure> {
        if let Some(exhaustion) = self.exhaustion {
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        let (used, limit) = match resource {
            super::ProofResource::Cells => (&mut self.used_cells, self.cell_limit),
            super::ProofResource::IntegerBytes => {
                (&mut self.used_integer_bytes, self.integer_byte_limit)
            }
        };
        let Some(required) = used.checked_add(amount) else {
            return Err(ProofPlanningFailure::Unsupported);
        };
        if required > u128::from(limit) {
            let exhaustion = IndexDomainProofExhaustion {
                resource,
                required,
                limit,
            };
            self.exhaustion = Some(exhaustion);
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        *used = required;
        Ok(())
    }

    fn reserve_evaluation(
        &mut self,
        cells: u128,
        integer_bytes: u128,
    ) -> Result<(), ProofPlanningFailure> {
        if let Some(exhaustion) = self.exhaustion {
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        let Some(required_cells) = self.used_cells.checked_add(cells) else {
            return Err(ProofPlanningFailure::Unsupported);
        };
        let Some(required_integer_bytes) = self.used_integer_bytes.checked_add(integer_bytes)
        else {
            return Err(ProofPlanningFailure::Unsupported);
        };
        let exhaustion = if required_cells > u128::from(self.cell_limit) {
            Some(IndexDomainProofExhaustion {
                resource: super::ProofResource::Cells,
                required: required_cells,
                limit: self.cell_limit,
            })
        } else if required_integer_bytes > u128::from(self.integer_byte_limit) {
            Some(IndexDomainProofExhaustion {
                resource: super::ProofResource::IntegerBytes,
                required: required_integer_bytes,
                limit: self.integer_byte_limit,
            })
        } else {
            None
        };
        if let Some(exhaustion) = exhaustion {
            self.exhaustion = Some(exhaustion);
            return Err(ProofPlanningFailure::Exhausted(exhaustion));
        }
        self.used_cells = required_cells;
        self.used_integer_bytes = required_integer_bytes;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum ProofPlanningFailure {
    Unsupported,
    Exhausted(IndexDomainProofExhaustion),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct IndexDomainKey(Vec<(VerifiedDimensionId, u64)>);

#[derive(Clone)]
struct ResolvedIndexDomain {
    key: IndexDomainKey,
    points: u64,
}

struct PlannedDomainObligation {
    slot: usize,
    obligation: UnknownIndexDomainPredicate,
    upper_bound: Option<u64>,
}

struct IndexDomainGroup {
    domain: IndexDomainKey,
    points: u64,
    obligations: Vec<PlannedDomainObligation>,
}

/// Assesses one region's obligations against a budget of its own.
///
/// The single-region spelling the exact-finite proof tests are written against.
/// Completion itself always goes through [`assess_finite_domains_with`], because
/// a realization's stages share one ledger.
#[cfg(test)]
fn assess_finite_domains(
    region: &VerifiedIndexRegion,
    obligations: &[UnknownIndexDomainPredicate],
    budget: IndexDomainProofBudget,
) -> Vec<IndexDomainProofClaim> {
    let mut ledger = IndexDomainProofLedger::new(budget);
    assess_finite_domains_with(region, obligations, &mut ledger)
}

/// Assesses one stage's obligations against a ledger the whole realization
/// shares.
///
/// The ledger is a parameter rather than a fresh budget per stage because a
/// caller states one bound for the work it is willing to fund. Re-funding it per
/// stage would let an n-stage realization spend n times the limit its caller
/// named, which is the same fail-closed bound quietly weakened by the arrival of
/// a second stage.
fn assess_finite_domains_with(
    region: &VerifiedIndexRegion,
    obligations: &[UnknownIndexDomainPredicate],
    ledger: &mut IndexDomainProofLedger,
) -> Vec<IndexDomainProofClaim> {
    let mut claims = vec![None; obligations.len()];
    let mut access_domains = HashMap::<VerifiedTensorAccessId, Option<ResolvedIndexDomain>>::new();
    let mut extents = HashMap::<IndexExtentRef, Option<u64>>::new();
    let mut groups = Vec::<IndexDomainGroup>::new();
    let mut group_indices = HashMap::<IndexDomainKey, usize>::new();

    for (slot, obligation) in obligations.iter().copied().enumerate() {
        if ledger.exhaustion.is_some() {
            break;
        }
        if let Err(failure) = ledger.debit(super::ProofResource::Cells, 1) {
            if matches!(failure, ProofPlanningFailure::Unsupported) {
                claims[slot] = Some(unsupported_proof_claim());
            }
            break;
        }
        let domain = if let Some(cached) = access_domains.get(&obligation.subject()) {
            cached.clone()
        } else {
            let resolved = match resolve_domain(region, obligation.subject(), &mut *ledger) {
                Ok(domain) => Some(domain),
                Err(ProofPlanningFailure::Unsupported) => None,
                Err(ProofPlanningFailure::Exhausted(_)) => break,
            };
            access_domains.insert(obligation.subject(), resolved.clone());
            resolved
        };
        let Some(domain) = domain else {
            claims[slot] = Some(unsupported_proof_claim());
            continue;
        };
        let upper_bound =
            if let IndexDomainPredicate::LessThanExtent { extent, .. } = obligation.predicate() {
                let resolved = if let Some(cached) = extents.get(&extent) {
                    *cached
                } else {
                    if ledger.debit(super::ProofResource::Cells, 1).is_err() {
                        break;
                    }
                    let resolved = resolve_extent(region, extent);
                    extents.insert(extent, resolved);
                    resolved
                };
                if resolved.is_none() {
                    claims[slot] = Some(unsupported_proof_claim());
                    continue;
                }
                resolved
            } else {
                None
            };
        let group_index = if let Some(group_index) = group_indices.get(&domain.key) {
            *group_index
        } else {
            let group_index = groups.len();
            groups.push(IndexDomainGroup {
                domain: domain.key.clone(),
                points: domain.points,
                obligations: Vec::new(),
            });
            group_indices.insert(domain.key.clone(), group_index);
            group_index
        };
        groups[group_index]
            .obligations
            .push(PlannedDomainObligation {
                slot,
                obligation,
                upper_bound,
            });
    }

    if let Some(exhaustion) = ledger.exhaustion {
        fill_unassessed(&mut claims, exhaustion);
        return claims.into_iter().map(Option::unwrap).collect();
    }

    for group in groups {
        if let Some(exhaustion) = ledger.exhaustion {
            fill_unassessed(&mut claims, exhaustion);
            break;
        }
        if group.points == 0 {
            for planned in group.obligations {
                claims[planned.slot] = Some(exhaustive_proof_claim(0));
            }
            continue;
        }
        match assess_domain_group(region, &group, &mut *ledger) {
            Ok(group_claims) => {
                for (planned, claim) in group.obligations.iter().zip(group_claims) {
                    claims[planned.slot] = Some(claim);
                }
            }
            Err(ProofPlanningFailure::Unsupported) => {
                for planned in group.obligations {
                    claims[planned.slot] = Some(unsupported_proof_claim());
                }
            }
            Err(ProofPlanningFailure::Exhausted(exhaustion)) => {
                fill_unassessed(&mut claims, exhaustion);
                break;
            }
        }
    }
    claims
        .into_iter()
        .map(|claim| claim.unwrap_or_else(unsupported_proof_claim))
        .collect()
}

fn resolve_domain(
    region: &VerifiedIndexRegion,
    subject: VerifiedTensorAccessId,
    ledger: &mut IndexDomainProofLedger,
) -> Result<ResolvedIndexDomain, ProofPlanningFailure> {
    let access = region
        .access(subject)
        .map_err(|_| ProofPlanningFailure::Unsupported)?;
    let mut dimensions = Vec::with_capacity(access.domain().len());
    for dimension in access.domain() {
        ledger.debit(super::ProofResource::Cells, 1)?;
        let extent = region
            .dimension(dimension)
            .ok()
            .and_then(|dimension| dimension.extent().as_static())
            .ok_or(ProofPlanningFailure::Unsupported)?;
        dimensions.push((dimension, extent.get()));
    }
    let points = finite_point_count(
        &dimensions
            .iter()
            .map(|(_, extent)| *extent)
            .collect::<Vec<_>>(),
    )
    .and_then(|points| u64::try_from(points).ok())
    .ok_or(ProofPlanningFailure::Unsupported)?;
    Ok(ResolvedIndexDomain {
        key: IndexDomainKey(dimensions),
        points,
    })
}

fn assess_domain_group(
    region: &VerifiedIndexRegion,
    group: &IndexDomainGroup,
    ledger: &mut IndexDomainProofLedger,
) -> Result<Vec<IndexDomainProofClaim>, ProofPlanningFailure> {
    let mut reached = HashSet::new();
    let mut postorder = Vec::new();
    let mut widths = HashMap::new();
    let mut node_bytes = 0_u128;
    let mut edge_count = 0_u128;
    let mut supported = vec![true; group.obligations.len()];
    for (index, planned) in group.obligations.iter().enumerate() {
        let postorder_start = postorder.len();
        let node_bytes_start = node_bytes;
        let edge_count_start = edge_count;
        let result = plan_expression(
            region,
            predicate_expression(planned.obligation.predicate()),
            &group.domain,
            &mut reached,
            &mut postorder,
            &mut widths,
            &mut node_bytes,
            &mut edge_count,
            ledger,
        );
        match result {
            Ok(()) => {}
            Err(ProofPlanningFailure::Unsupported) => {
                for expression in postorder.drain(postorder_start..) {
                    reached.remove(&expression);
                    widths.remove(&expression);
                }
                node_bytes = node_bytes_start;
                edge_count = edge_count_start;
                supported[index] = false;
            }
            Err(exhausted @ ProofPlanningFailure::Exhausted(_)) => return Err(exhausted),
        }
    }
    if supported.iter().all(|supported| !supported) {
        return Ok(group
            .obligations
            .iter()
            .map(|_| unsupported_proof_claim())
            .collect());
    }
    let predicate_bytes = group
        .obligations
        .iter()
        .zip(&supported)
        .try_fold(0_u128, |bytes, (planned, supported)| {
            if !supported {
                return Some(bytes);
            }
            let width = *widths.get(&predicate_expression(planned.obligation.predicate()))?;
            bytes.checked_add(match planned.obligation.predicate() {
                IndexDomainPredicate::NonNegative { .. } => 8,
                IndexDomainPredicate::LessThanExtent { .. } => {
                    checked_mul(8, checked_add(byte_limbs(width).ok()?, 1).ok()?).ok()?
                }
            })
        })
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let dimension_cells = (group.domain.0.len() as u128)
        .checked_mul(2)
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let node_cells = (postorder.len() as u128)
        .checked_mul(2)
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let cells_per_point = dimension_cells
        .checked_add(node_cells)
        .and_then(|cells| cells.checked_add(edge_count))
        .and_then(|cells| {
            cells.checked_add(supported.iter().filter(|value| **value).count() as u128)
        })
        .ok_or(ProofPlanningFailure::Unsupported)?;
    let integer_bytes_per_point = node_bytes
        .checked_add(predicate_bytes)
        .ok_or(ProofPlanningFailure::Unsupported)?;
    ledger.reserve_evaluation(
        u128::from(group.points)
            .checked_mul(cells_per_point)
            .ok_or(ProofPlanningFailure::Unsupported)?,
        u128::from(group.points)
            .checked_mul(integer_bytes_per_point)
            .ok_or(ProofPlanningFailure::Unsupported)?,
    )?;

    let mut coordinates = vec![0_u64; group.domain.0.len()];
    let mut environment = HashMap::with_capacity(group.domain.0.len());
    let mut values = HashMap::with_capacity(postorder.len());
    let mut first_counterexamples = vec![None; group.obligations.len()];
    for point_ordinal in 0..group.points {
        environment.clear();
        environment.extend(
            group
                .domain
                .0
                .iter()
                .zip(&coordinates)
                .map(|((dimension, _), coordinate)| (*dimension, *coordinate)),
        );
        values.clear();
        for expression in &postorder {
            evaluate_planned_node(region, *expression, &environment, &mut values)
                .ok_or(ProofPlanningFailure::Unsupported)?;
        }
        for (index, planned) in group.obligations.iter().enumerate() {
            if !supported[index] || first_counterexamples[index].is_some() {
                continue;
            }
            let expression = predicate_expression(planned.obligation.predicate());
            let value = values
                .get(&expression)
                .ok_or(ProofPlanningFailure::Unsupported)?;
            if !predicate_holds(planned.obligation.predicate(), planned.upper_bound, value) {
                first_counterexamples[index] = Some(point_ordinal);
            }
        }
        increment_coordinates(&mut coordinates, &group.domain.0);
    }
    group
        .obligations
        .iter()
        .zip(first_counterexamples)
        .enumerate()
        .map(|(index, (planned, counterexample))| {
            if !supported[index] {
                return Ok(unsupported_proof_claim());
            }
            if let Some(point_ordinal) = counterexample {
                let reason = match planned.obligation.predicate() {
                    IndexDomainPredicate::NonNegative { .. } => "logical-index-negative",
                    IndexDomainPredicate::LessThanExtent { .. } => {
                        "logical-index-not-less-than-extent"
                    }
                };
                let disproof =
                    IndexDomainDisproof::new(reason, encode_counterexample(point_ordinal))
                        .map_err(|_| ProofPlanningFailure::Unsupported)?;
                Ok(IndexDomainProofClaim::Disproved(
                    disproof.with_point_ordinal(point_ordinal),
                ))
            } else {
                Ok(exhaustive_proof_claim(group.points))
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn plan_expression(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    domain: &IndexDomainKey,
    reached: &mut HashSet<VerifiedIndexExprId>,
    postorder: &mut Vec<VerifiedIndexExprId>,
    widths: &mut HashMap<VerifiedIndexExprId, u128>,
    node_bytes: &mut u128,
    edge_count: &mut u128,
    ledger: &mut IndexDomainProofLedger,
) -> Result<(), ProofPlanningFailure> {
    if reached.contains(&expression) {
        return Ok(());
    }
    ledger.debit(super::ProofResource::Cells, 1)?;
    let expression_view = region
        .index_expression(expression)
        .map_err(|_| ProofPlanningFailure::Unsupported)?;
    let width = match expression_view.view() {
        IndexExprView::Constant(value) => {
            let width = integer_width(value);
            *node_bytes = checked_add(*node_bytes, copy_cost(width)?)?;
            width
        }
        IndexExprView::Dimension(dimension) => {
            if !domain
                .0
                .iter()
                .any(|(candidate, _)| *candidate == dimension)
            {
                return Err(ProofPlanningFailure::Unsupported);
            }
            *node_bytes = checked_add(*node_bytes, copy_cost(8)?)?;
            8
        }
        IndexExprView::LinearCombination { constant, terms } => {
            let mut accumulator = integer_width(constant);
            *node_bytes = checked_add(*node_bytes, copy_cost(accumulator)?)?;
            for term in terms {
                ledger.debit(super::ProofResource::Cells, 1)?;
                *edge_count = checked_add(*edge_count, 1)?;
                plan_expression(
                    region,
                    term.value(),
                    domain,
                    reached,
                    postorder,
                    widths,
                    node_bytes,
                    edge_count,
                    ledger,
                )?;
                let coefficient = integer_width(term.coefficient());
                let child = *widths
                    .get(&term.value())
                    .ok_or(ProofPlanningFailure::Unsupported)?;
                let product = checked_add(coefficient, child)?;
                let next_accumulator = checked_add(accumulator.max(product), 1)?;
                *node_bytes = checked_add(
                    *node_bytes,
                    multiplication_cost(coefficient, child, product)?,
                )?;
                *node_bytes = checked_add(
                    *node_bytes,
                    addition_cost(accumulator, product, next_accumulator)?,
                )?;
                accumulator = next_accumulator;
            }
            accumulator
        }
        IndexExprView::FloorDiv { dividend, divisor } => {
            let divisor = divisor
                .as_static()
                .ok_or(ProofPlanningFailure::Unsupported)?;
            ledger.debit(super::ProofResource::Cells, 1)?;
            *edge_count = checked_add(*edge_count, 1)?;
            plan_expression(
                region, dividend, domain, reached, postorder, widths, node_bytes, edge_count,
                ledger,
            )?;
            let width = *widths
                .get(&dividend)
                .ok_or(ProofPlanningFailure::Unsupported)?;
            let _ = divisor;
            let result_width = checked_add(width, 1)?;
            *node_bytes = checked_add(*node_bytes, division_cost(width, result_width)?)?;
            result_width
        }
        IndexExprView::Modulo { dividend, divisor } => {
            divisor
                .as_static()
                .ok_or(ProofPlanningFailure::Unsupported)?;
            ledger.debit(super::ProofResource::Cells, 1)?;
            *edge_count = checked_add(*edge_count, 1)?;
            plan_expression(
                region, dividend, domain, reached, postorder, widths, node_bytes, edge_count,
                ledger,
            )?;
            let width = *widths
                .get(&dividend)
                .ok_or(ProofPlanningFailure::Unsupported)?;
            *node_bytes = checked_add(*node_bytes, division_cost(width, 8)?)?;
            8
        }
    };
    reached.insert(expression);
    widths.insert(expression, width);
    postorder.push(expression);
    Ok(())
}

fn checked_add(left: u128, right: u128) -> Result<u128, ProofPlanningFailure> {
    left.checked_add(right)
        .ok_or(ProofPlanningFailure::Unsupported)
}

fn checked_mul(left: u128, right: u128) -> Result<u128, ProofPlanningFailure> {
    left.checked_mul(right)
        .ok_or(ProofPlanningFailure::Unsupported)
}

// These byte-work bounds follow the locked num-bigint 0.4.8 multiplication
// and division implementations. A dependency revision requires a formula
// audit. They conservatively count 8-byte limb touches, including operands,
// results, and transient work; they do not describe proof identity.
fn byte_limbs(bytes: u128) -> Result<u128, ProofPlanningFailure> {
    Ok(checked_add(bytes.max(1), 7)? / 8)
}

fn copy_cost(width: u128) -> Result<u128, ProofPlanningFailure> {
    checked_mul(16, byte_limbs(width)?)
}

fn addition_cost(left: u128, right: u128, result: u128) -> Result<u128, ProofPlanningFailure> {
    let limbs = checked_add(byte_limbs(left)?, byte_limbs(right)?)?;
    checked_mul(32, checked_add(limbs, byte_limbs(result)?)?)
}

fn multiplication_cost(
    left: u128,
    right: u128,
    result: u128,
) -> Result<u128, ProofPlanningFailure> {
    let left = byte_limbs(left)?;
    let right = byte_limbs(right)?;
    let result = byte_limbs(result)?;
    let nonlinear = checked_mul(
        256,
        checked_mul(checked_add(left, 1)?, checked_add(right, 1)?)?,
    )?;
    checked_add(
        nonlinear,
        checked_mul(32, checked_add(checked_add(left, right)?, result)?)?,
    )
}

fn division_cost(dividend: u128, result: u128) -> Result<u128, ProofPlanningFailure> {
    let dividend = byte_limbs(dividend)?;
    let result = byte_limbs(result)?;
    let division = checked_mul(8, checked_mul(6, checked_add(dividend, 1)?)?)?;
    checked_add(division, checked_mul(16, checked_add(result, 1)?)?)
}

fn evaluate_planned_node(
    region: &VerifiedIndexRegion,
    expression: VerifiedIndexExprId,
    environment: &HashMap<VerifiedDimensionId, u64>,
    values: &mut HashMap<VerifiedIndexExprId, BigInt>,
) -> Option<()> {
    let value = match region.index_expression(expression).ok()?.view() {
        IndexExprView::Constant(value) => decode_integer(value),
        IndexExprView::Dimension(dimension) => BigInt::from(*environment.get(&dimension)?),
        IndexExprView::LinearCombination { constant, terms } => {
            let mut total = decode_integer(constant);
            for term in terms {
                total += decode_integer(term.coefficient()) * values.get(&term.value())?;
            }
            total
        }
        IndexExprView::FloorDiv { dividend, divisor } => values
            .get(&dividend)?
            .div_floor(&BigInt::from(divisor.as_static()?.get())),
        IndexExprView::Modulo { dividend, divisor } => values
            .get(&dividend)?
            .mod_floor(&BigInt::from(divisor.as_static()?.get())),
    };
    values.insert(expression, value);
    Some(())
}

fn exhaustive_proof_claim(points: u64) -> IndexDomainProofClaim {
    IndexDomainProofClaim::Proved(IndexDomainProofEvidence::ExhaustiveFinite {
        points,
        derivation: EXHAUSTIVE_DERIVATION.into(),
    })
}

fn unsupported_proof_claim() -> IndexDomainProofClaim {
    IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
}

fn fill_unassessed(
    claims: &mut [Option<IndexDomainProofClaim>],
    exhaustion: IndexDomainProofExhaustion,
) {
    for claim in claims.iter_mut().filter(|claim| claim.is_none()) {
        *claim = Some(proof_resource_limit(exhaustion));
    }
}

fn proof_resource_limit(exhaustion: IndexDomainProofExhaustion) -> IndexDomainProofClaim {
    IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
        resource: exhaustion.resource,
        required: exhaustion.required,
        limit: exhaustion.limit,
    })
}

fn finite_point_count(extents: &[u64]) -> Option<u128> {
    if extents.contains(&0) {
        return Some(0);
    }
    extents.iter().try_fold(1_u128, |product, extent| {
        product.checked_mul(u128::from(*extent))
    })
}

fn integer_width(value: &IndexInteger) -> u128 {
    (value.magnitude_byte_len() as u128).max(1)
}

const fn predicate_expression(predicate: IndexDomainPredicate) -> VerifiedIndexExprId {
    match predicate {
        IndexDomainPredicate::NonNegative { expression }
        | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
    }
}

fn decode_integer(value: &IndexInteger) -> BigInt {
    value.to_bigint()
}

fn predicate_holds(
    predicate: IndexDomainPredicate,
    upper_bound: Option<u64>,
    value: &BigInt,
) -> bool {
    match predicate {
        IndexDomainPredicate::NonNegative { .. } => value >= &BigInt::zero(),
        IndexDomainPredicate::LessThanExtent { .. } => upper_bound.is_some_and(|extent| {
            value.sign() == Sign::Minus || value.to_u64().is_some_and(|value| value < extent)
        }),
    }
}

fn resolve_extent(region: &VerifiedIndexRegion, extent: IndexExtentRef) -> Option<u64> {
    match extent {
        IndexExtentRef::Dimension(dimension) => region
            .dimension(dimension)
            .ok()?
            .extent()
            .as_static()
            .map(crate::shape::Extent::get),
        IndexExtentRef::TensorAxis { tensor, axis } => {
            let axis = usize::try_from(axis).ok()?;
            region
                .tensor(tensor)
                .ok()?
                .shape()
                .as_static()?
                .extents()
                .get(axis)
                .copied()
                .map(crate::shape::Extent::get)
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

fn encode_counterexample(point_ordinal: u64) -> Box<[u8]> {
    let mut output = COUNTEREXAMPLE_TAG.to_vec();
    output.extend_from_slice(&point_ordinal.to_be_bytes());
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

impl fmt::Display for IndexDomainProofRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} index-domain obligation(s) reached IR proof completion as {:?}",
            self.assessments.len(),
            self.kind
        )
    }
}

impl Error for IndexDomainProofRefusal {}

/// Why IR-owned refinement verification refused to mint a receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRefinementVerificationError {
    /// A numerical-contract key failed the IR-owned canonical grammar.
    InvalidNumericalContractIdentity {
        /// Exact canonical-key validation failure.
        source: NumericalContractKeyError,
    },
    /// A residual-domain reason, derivation, or counterexample was invalid.
    InvalidDomainProofEvidence,
    /// Exact-finite proof budget is zero or exceeds IR's hard bound.
    InvalidDomainProofBudget {
        /// Completion-stage resource whose limit was invalid.
        resource: super::ProofResource,
        /// Supplied limit.
        actual: u64,
        /// Maximum admitted limit.
        limit: u64,
    },
    /// A canonical region retained more obligations than the closed law
    /// vocabulary can produce.
    ResidualObligationsTooLarge {
        /// Exact retained count.
        actual: usize,
        /// IR-owned structural maximum.
        limit: usize,
    },
    /// An admitted signature exceeded the governed bound.
    SignatureTooLarge {
        /// Ordered signature side that exceeded the bound.
        side: IndexRefinementSignatureSide,
        /// Bounded count observed before refusing the iterator.
        actual: usize,
        /// Maximum admitted count.
        limit: usize,
    },
    /// A raw emitted-scalar declaration exceeded the governed bound.
    EmittedScalarOperationsTooLarge {
        /// Supplied raw declaration count before deduplication.
        actual: usize,
        /// Maximum admitted raw declaration count.
        limit: usize,
    },
    /// A completed receipt did not belong to the pending association supplied
    /// to the consumer.
    CompletionReceiptMismatch,
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
    /// The scalar registry was built over another semantic authority.
    ScalarSemanticAuthorityMismatch,
    /// The program-derived subject came from another semantic authority.
    SubjectSemanticAuthorityMismatch,
    /// The operation-specific semantic realization-law row (including its
    /// absence) differs from the row bound into the program-derived subject.
    SubjectRealizationLawMismatch,
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
    /// The candidate's ordered realization differs from the semantic law's
    /// expected region sequence.
    ///
    /// Distinct from [`Self::SemanticRealizationMismatch`], which names two
    /// regions: a chain can disagree in its stage count, in a stage's own
    /// region, or only in how a stage's inputs are sourced, and the sequence
    /// identity is what covers all three.
    SemanticRealizationSequenceMismatch {
        /// Canonical realization required by semantic authority.
        expected: CanonicalIndexRegionSequenceIdentity,
        /// Canonical realization emitted by the selected lowering.
        actual: CanonicalIndexRegionSequenceIdentity,
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
    /// An encoded semantic input declared no component boundary to bind.
    EmptyEncodedOperandComponents {
        /// Position in the distinct semantic input population.
        input: usize,
    },
    /// Region input count disagrees with the expanded semantic input boundary.
    OperandArity {
        /// Number of verified input boundaries.
        region_inputs: usize,
        /// Expected ordinary inputs plus ordered encoded components, saturated
        /// at `usize::MAX` if count arithmetic overflowed.
        expanded_inputs: usize,
    },
    /// One region input disagrees with its expanded semantic input boundary.
    OperandInterface {
        /// Position in the ordered expanded semantic input boundaries.
        position: usize,
    },
    /// Alias and component expansion exceeded the receipt binding population.
    OperandBindingsTooLarge {
        /// Binding count, saturated at `usize::MAX` on arithmetic overflow.
        actual: usize,
        /// Maximum operand bindings retained by one receipt.
        limit: usize,
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
            Self::InvalidNumericalContractIdentity { source } => {
                write!(
                    formatter,
                    "numerical-contract identity is invalid: {source}"
                )
            }
            Self::InvalidDomainProofEvidence => {
                formatter.write_str("domain-proof evidence is empty or exceeds its byte bound")
            }
            Self::InvalidDomainProofBudget {
                resource,
                actual,
                limit,
            } => write!(
                formatter,
                "domain-proof {resource:?} budget is {actual}; expected 1..={limit}"
            ),
            Self::ResidualObligationsTooLarge { actual, limit } => write!(
                formatter,
                "canonical realization retained {actual} residual obligations; limit is {limit}"
            ),
            Self::SignatureTooLarge {
                side,
                actual,
                limit,
            } => write!(
                formatter,
                "refinement signature {side:?} side reached {actual} values; limit is {limit}"
            ),
            Self::EmittedScalarOperationsTooLarge { actual, limit } => write!(
                formatter,
                "emitted scalar-operation declaration has {actual} entries; limit is {limit}"
            ),
            Self::CompletionReceiptMismatch => formatter
                .write_str("completed receipt does not match its pending refinement association"),
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
            Self::ScalarSemanticAuthorityMismatch => {
                formatter.write_str("scalar registry was built over another semantic authority")
            }
            Self::SubjectSemanticAuthorityMismatch => {
                formatter.write_str("program subject came from another semantic authority")
            }
            Self::SubjectRealizationLawMismatch => formatter
                .write_str("program subject came from another operation-specific realization law"),
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
            Self::SemanticRealizationSequenceMismatch { expected, actual } => write!(
                formatter,
                "candidate realization {:?} differs from semantic law realization {:?}",
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
            Self::EmptyEncodedOperandComponents { input } => write!(
                formatter,
                "encoded semantic input {input} declares no component boundaries"
            ),
            Self::OperandArity {
                region_inputs,
                expanded_inputs,
            } => write!(
                formatter,
                "region declares {region_inputs} inputs for {expanded_inputs} expanded semantic input boundaries"
            ),
            Self::OperandInterface { position } => {
                write!(
                    formatter,
                    "region input {position} does not match its expanded semantic input boundary"
                )
            }
            Self::OperandBindingsTooLarge { actual, limit } => write!(
                formatter,
                "expanded operand bindings {actual} exceed receipt limit {limit}"
            ),
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
            Self::InvalidNumericalContractIdentity { source } => Some(source),
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
    if lowering.realization_law_row != subject.realization_law_row {
        return Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch);
    }
    if lowering.scalar_registry.snapshot_identity()
        != resolution.registry.0.scalars.snapshot_identity()
    {
        return Err(IndexRefinementVerificationError::ScalarSnapshotMismatch);
    }
    Ok(())
}

/// One expanded semantic input boundary and the type and shape it demands.
///
/// An ordinary input expands to one entry; an encoded compound input expands to
/// one entry per component in its contract order, each carrying that component's
/// own resolved type and derived shape.
struct ExpandedInput {
    input: usize,
    component_role: Option<EncodedComponentRole>,
    value_type: ResolvedValueType,
    shape: Shape,
}

fn bind_operands(
    occurrence: &IndexRefinementSubject,
    realization: &VerifiedIndexRegionSequence,
) -> Result<Vec<OperandBinding>, IndexRefinementVerificationError> {
    let region_inputs = realization
        .stages()
        .map(|stage| {
            stage
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .count()
        })
        .try_fold(0_usize, usize::checked_add)
        .unwrap_or(usize::MAX);
    let expanded = expand_inputs(&occurrence.inputs, region_inputs)?;

    // Every stage input sourced from the occurrence is checked against the
    // boundary it claims, and the tensor is recorded against that boundary. One
    // boundary can be claimed by several stages — a value a fold reads and the
    // pass consuming the fold reads again is the motivating case — so this is a
    // list per boundary rather than a single tensor.
    let mut physical_by_expanded = vec![Vec::new(); expanded.len()];
    for (stage, region) in realization.stages().enumerate() {
        let inputs = region
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .collect::<Vec<_>>();
        let sources = realization.stage_sources(stage).ok_or(
            IndexRefinementVerificationError::OperandArity {
                region_inputs,
                expanded_inputs: expanded.len(),
            },
        )?;
        for (slot, source) in sources.iter().enumerate() {
            let StagedInputSource::Occurrence(position) = source else {
                // An intermediate is the sequence's own value: it binds to no
                // semantic operand, and the chain check already proved it agrees
                // with the boundary that produced it.
                continue;
            };
            let boundary =
                expanded
                    .get(*position)
                    .ok_or(IndexRefinementVerificationError::OperandArity {
                        region_inputs,
                        expanded_inputs: expanded.len(),
                    })?;
            let input = inputs[slot];
            let shape = input
                .shape()
                .as_static()
                .ok_or(IndexRefinementVerificationError::SymbolicBoundary)?;
            if input.value_type() != &boundary.value_type || shape != &boundary.shape {
                return Err(IndexRefinementVerificationError::OperandInterface {
                    position: *position,
                });
            }
            physical_by_expanded[*position].push((stage, input.id()));
        }
    }
    // A declared boundary no stage reads is an arity disagreement, not a silent
    // omission: the occurrence states an input the realization never consumes.
    // Reported as arity rather than interface because nothing disagreed about
    // the boundary — there was no tensor to disagree with it.
    if physical_by_expanded.iter().any(Vec::is_empty) {
        return Err(IndexRefinementVerificationError::OperandArity {
            region_inputs,
            expanded_inputs: expanded.len(),
        });
    }

    let component_counts = occurrence
        .inputs
        .iter()
        .enumerate()
        .map(|(input, _)| {
            expanded
                .iter()
                .enumerate()
                .filter(|(_, boundary)| boundary.input == input)
                .map(|(position, _)| physical_by_expanded[position].len())
                .try_fold(0_usize, usize::checked_add)
                .unwrap_or(usize::MAX)
        })
        .collect::<Vec<_>>();
    let binding_count = count_operand_bindings(&occurrence.operands, &component_counts)?;
    let mut bindings = Vec::with_capacity(binding_count);
    for (position, input) in occurrence.operands.iter().copied().enumerate() {
        for (expanded_position, boundary) in expanded.iter().enumerate() {
            if boundary.input != input {
                continue;
            }
            for (stage, input_tensor) in &physical_by_expanded[expanded_position] {
                bindings.push(OperandBinding {
                    stage: *stage,
                    operand: position,
                    input,
                    input_tensor: *input_tensor,
                    component_role: boundary.component_role,
                });
            }
        }
    }
    debug_assert_eq!(bindings.len(), binding_count);
    Ok(bindings)
}

/// Expands semantic inputs to the ordered boundary list a realization sources.
fn expand_inputs(
    inputs: &[IndexRefinementBoundary],
    region_inputs: usize,
) -> Result<Vec<ExpandedInput>, IndexRefinementVerificationError> {
    let expanded_inputs = count_expanded_inputs(inputs, region_inputs)?;
    let mut expanded = Vec::with_capacity(expanded_inputs);
    for (input, boundary) in inputs.iter().enumerate() {
        if let Some((_, contract)) = boundary.value_type.encoded_numeric_parts() {
            for component in contract.components() {
                expanded.push(ExpandedInput {
                    input,
                    component_role: Some(component.role()),
                    value_type: component.resolved_type().clone(),
                    shape: component.shape_relation().component_shape(&boundary.shape),
                });
            }
        } else {
            expanded.push(ExpandedInput {
                input,
                component_role: None,
                value_type: boundary.value_type.clone(),
                shape: boundary.shape.clone(),
            });
        }
    }
    debug_assert_eq!(expanded.len(), expanded_inputs);
    Ok(expanded)
}

/// Counts component-expanded semantic inputs without deriving component shapes.
///
/// The verified-region boundary ceiling is the authoritative retained
/// population bound. Counting first prevents a wide signature of maximum-size
/// encoded contracts from multiplying component-shape allocations before the
/// arity mismatch is known.
fn count_expanded_inputs(
    inputs: &[IndexRefinementBoundary],
    region_inputs: usize,
) -> Result<usize, IndexRefinementVerificationError> {
    let mut expanded_inputs = 0_usize;
    for (input, boundary) in inputs.iter().enumerate() {
        let contribution = if let Some((_, contract)) = boundary.value_type.encoded_numeric_parts()
        {
            if contract.components().is_empty() {
                return Err(
                    IndexRefinementVerificationError::EmptyEncodedOperandComponents { input },
                );
            }
            contract.components().len()
        } else {
            1
        };
        expanded_inputs = expanded_inputs.saturating_add(contribution);
    }
    if expanded_inputs > MAX_BOUNDARY_TENSORS {
        return Err(IndexRefinementVerificationError::OperandArity {
            region_inputs,
            expanded_inputs,
        });
    }
    Ok(expanded_inputs)
}

/// Counts final operand-use bindings before allocating the retained receipt
/// population.
fn count_operand_bindings(
    operands: &[usize],
    component_counts: &[usize],
) -> Result<usize, IndexRefinementVerificationError> {
    let mut bindings = 0_usize;
    for input in operands {
        let contribution = component_counts.get(*input).copied().unwrap_or(usize::MAX);
        bindings = bindings.saturating_add(contribution);
    }
    if bindings > MAX_INDEX_REFINEMENT_OPERAND_BINDINGS {
        return Err(IndexRefinementVerificationError::OperandBindingsTooLarge {
            actual: bindings,
            limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
        });
    }
    Ok(bindings)
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
    realization: &VerifiedIndexRegionSequence,
    scalar_authorities: Vec<ScalarAuthorityEvidence>,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
) -> IndexRefinementReceipt {
    let identity = encode_receipt_identity(
        subject,
        resolution,
        realization,
        &scalar_authorities,
        &index_domain_proofs,
    );
    let executable_coverage_identity = encode_executable_coverage_identity(
        subject,
        resolution,
        realization,
        &scalar_authorities,
        &operand_bindings,
        &result_bindings,
        &index_domain_proofs,
    );
    let mut leading_scalar_authorities = scalar_authorities;
    let Some(scalar_authority) = leading_scalar_authorities.pop() else {
        unreachable!("a realization has a final stage and therefore its evidence")
    };
    IndexRefinementReceipt {
        graph: subject.graph.clone(),
        occurrence: subject.occurrence,
        leading_regions: realization
            .leading_stages()
            .iter()
            .map(|stage| stage.canonical_identity().clone())
            .collect(),
        region: realization.final_stage().canonical_identity().clone(),
        realization: realization.identity().clone(),
        leading_scalar_authorities,
        scalar_authority,
        operand_bindings,
        result_bindings,
        index_domain_proofs,
        identity: IndexRefinementReceiptIdentity(identity.into_boxed_slice()),
        executable_coverage_identity: IndexRefinementExecutableCoverageIdentity(
            executable_coverage_identity.into_boxed_slice(),
        ),
    }
}

/// Encodes reached-only executable provenance.
///
/// **Why a one-stage realization writes the bytes it always wrote.** A
/// realization spanning several stages carries more than one region identity,
/// more than one scalar authority, and a stage ordinal on every binding — none of
/// which a one-stage realization has anything to say about. Rather than write
/// empty or constant fields into every receipt ever minted, the one-stage form
/// keeps its established encoding and the staged form is written under its own
/// domain tag. The two tags are distinct byte strings in the first position, so
/// the preimages are disjoint and no staged coverage can spell a single-region
/// one.
///
/// **Why the graph is a digest and not the identity itself.** One whole
/// `SemanticGraphIdentity` used to open every record, and there is one record per
/// semantic operation, so the product of a linear encoding with a linear count
/// made kernel-program identity quadratic in operation count — measured at
/// `134n² + 3650n + 727` bytes, whose quadratic coefficient *is* the graph
/// encoding's per-operation slope. [ADR 0104] folds it to
/// [`DIGEST_BYTES`] under [`COVERAGE_GRAPH_DIGEST_DOMAIN`], which makes the
/// curve linear.
///
/// It is written unframed because it is fixed width: a length prefix exists to
/// make a variable-length run self-delimiting, and thirty-two bytes that are
/// always thirty-two bytes are already that. The record therefore says exactly
/// what it said before — "this occurrence of *this* graph" — and still refuses
/// two records naming one occurrence ordinal in different graphs, which is the
/// injectivity the pair carries and the reason the graph could not simply be
/// dropped. What it stops doing is carrying bytes the graph identity could be
/// reconstructed from, which nothing in the workspace does: the type has no
/// decoder, no field accessors, and two `compile_fail` doctests holding that it
/// has no byte constructor.
///
/// [ADR 0104]: ../../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md
/// [`DIGEST_BYTES`]: tiler_digest::DIGEST_BYTES
fn encode_executable_coverage_identity(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    realization: &VerifiedIndexRegionSequence,
    scalar_authorities: &[ScalarAuthorityEvidence],
    operand_bindings: &[OperandBinding],
    result_bindings: &[ResultBinding],
    proofs: &[IndexRefinementDomainProof],
) -> Vec<u8> {
    let staged = !realization.is_single_stage();
    let mut bytes = if staged {
        STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG.to_vec()
    } else {
        EXECUTABLE_COVERAGE_IDENTITY_TAG.to_vec()
    };
    bytes.extend_from_slice(
        DigestAlgorithm::GOVERNED
            .digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())
            .as_bytes(),
    );
    bytes.extend_from_slice(&subject.occurrence.get().to_be_bytes());
    push_slice(&mut bytes, subject.numerical_contract.as_bytes());
    if staged {
        push_slice(&mut bytes, realization.identity().as_bytes());
    } else {
        push_slice(
            &mut bytes,
            realization.final_stage().canonical_identity().as_bytes(),
        );
    }
    push_slice(
        &mut bytes,
        subject.semantic_authority.reached_definitions().as_bytes(),
    );
    push_slice(
        &mut bytes,
        subject.semantic_authority.admission_provenance().as_bytes(),
    );
    encode_optional_law_row(&mut bytes, subject.realization_law_row.as_deref());
    encode_provider(&mut bytes, resolution.provider());
    bytes.extend_from_slice(&resolution.revision().to_be_bytes());
    if staged {
        push_len(&mut bytes, scalar_authorities.len());
    }
    for authority in scalar_authorities {
        push_slice(&mut bytes, authority.definitions().as_bytes());
        push_slice(&mut bytes, authority.admission().as_bytes());
        push_slice(&mut bytes, authority.type_definitions().as_bytes());
        push_slice(&mut bytes, authority.type_admission().as_bytes());
    }
    push_len(&mut bytes, operand_bindings.len());
    for binding in operand_bindings {
        if staged {
            push_len(&mut bytes, binding.stage);
        }
        push_len(&mut bytes, binding.operand);
        push_len(&mut bytes, binding.input);
        bytes.extend_from_slice(&binding.input_tensor.index.to_be_bytes());
        match binding.component_role {
            None => bytes.push(0),
            Some(role) => {
                bytes.push(1);
                bytes.extend_from_slice(&role.get().to_be_bytes());
            }
        }
    }
    push_len(&mut bytes, result_bindings.len());
    for binding in result_bindings {
        push_len(&mut bytes, binding.result);
        bytes.extend_from_slice(&binding.output_tensor.index.to_be_bytes());
        bytes.extend_from_slice(&binding.write_access.index.to_be_bytes());
        bytes.extend_from_slice(&binding.written_value.index.to_be_bytes());
    }
    push_len(&mut bytes, proofs.len());
    for proof in proofs {
        if staged {
            push_len(&mut bytes, proof.stage);
        }
        push_slice(&mut bytes, proof.identity().as_bytes());
    }
    bytes
}

/// Encodes the canonical receipt identity.
///
/// Domain-separated the same way [`encode_executable_coverage_identity`] is, and
/// for the same reason: a one-stage realization keeps
/// [`RECEIPT_IDENTITY_TAG`] and the exact field order it has always written, so
/// every receipt a single-region law ever minted is byte-identical; a staged
/// realization writes its whole ordered chain under
/// [`STAGED_RECEIPT_IDENTITY_TAG`].
fn encode_receipt_identity(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    realization: &VerifiedIndexRegionSequence,
    scalar_authorities: &[ScalarAuthorityEvidence],
    proofs: &[IndexRefinementDomainProof],
) -> Vec<u8> {
    let staged = !realization.is_single_stage();
    let mut bytes = if staged {
        STAGED_RECEIPT_IDENTITY_TAG.to_vec()
    } else {
        RECEIPT_IDENTITY_TAG.to_vec()
    };
    if staged {
        push_slice(&mut bytes, realization.identity().as_bytes());
    } else {
        push_slice(
            &mut bytes,
            realization.final_stage().canonical_identity().as_bytes(),
        );
    }
    push_slice(&mut bytes, &subject.identity);
    push_slice(&mut bytes, &resolution.identity);
    if staged {
        push_len(&mut bytes, scalar_authorities.len());
    }
    for authority in scalar_authorities {
        push_slice(&mut bytes, authority.definitions().as_bytes());
        push_slice(&mut bytes, authority.type_definitions().as_bytes());
        push_slice(&mut bytes, authority.semantic_snapshot().as_bytes());
        push_slice(&mut bytes, authority.scalar_snapshot().as_bytes());
    }
    push_len(&mut bytes, proofs.len());
    for proof in proofs {
        if staged {
            push_len(&mut bytes, proof.stage);
        }
        push_slice(&mut bytes, proof.identity().as_bytes());
    }
    bytes
}

fn encode_subject_identity(subject: &IndexRefinementSubject) -> Vec<u8> {
    encode_subject_identity_with(subject, SUBJECT_IDENTITY_TAG, subject.occurrence)
}

fn encode_subject_identity_with(
    subject: &IndexRefinementSubject,
    domain: &[u8],
    occurrence: SemanticOccurrence,
) -> Vec<u8> {
    let mut bytes = domain.to_vec();
    push_slice(&mut bytes, subject.graph.as_bytes());
    bytes.extend_from_slice(&occurrence.get().to_be_bytes());
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
    encode_optional_law_row(&mut bytes, subject.realization_law_row.as_deref());
    bytes
}

fn encode_authority_identity(
    operation: &OpKey,
    signature: &IndexRefinementSignature,
    semantic: &SemanticCapabilityAuthority,
    scalar: &CanonicalScalarDefinitionProjection,
    scalar_snapshot: &[u8],
    realization_law_row: Option<&[u8]>,
) -> Vec<u8> {
    let mut bytes = AUTHORITY_IDENTITY_TAG.to_vec();
    encode_op_key(&mut bytes, operation);
    encode_signature(&mut bytes, signature);
    push_slice(&mut bytes, semantic.reached_definitions().as_bytes());
    push_slice(&mut bytes, semantic.admission_provenance().as_bytes());
    push_slice(&mut bytes, semantic.registry_snapshot().as_bytes());
    push_slice(&mut bytes, scalar.as_bytes());
    push_slice(&mut bytes, scalar_snapshot);
    encode_optional_law_row(&mut bytes, realization_law_row);
    bytes
}

fn encode_optional_law_row(output: &mut Vec<u8>, row: Option<&[u8]>) {
    match row {
        None => output.push(0),
        Some(row) => {
            output.push(1);
            push_slice(output, row);
        }
    }
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
    use crate::index::{
        DomainRole, EXTENT_PHASE_CEILING, IndexRegionBuilder, ScalarArity, ScalarAttributeField,
        ScalarAttributeSchema, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
        ScalarInferenceRequest, ScalarOperationContract, ScalarOperationDefinition,
        ScalarOperationInferencer, ScalarRegistryBuilder, SourcedExtent,
    };
    use crate::program::abi::AvailabilityPhase;
    use crate::semantic::{
        AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind,
        EncodedComponentDeclaration, EncodedComponentRole, EncodedComponentShape,
        EncodedNumericContract, F32, F32Constant, F32Multiply, InputKey, NormativeDefinitionRef,
        OpKey, OperationArity, OperationConformance, OperationDefinition, OperationDefinitionFacts,
        OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
        OperationInferencer, OperationSchema, OutputKey, ProviderDiagnosticCode, QuantSchemeKey,
        SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
        SemanticRegistryRegistrar, TypeKey,
    };
    use crate::shape::{
        BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, InterfaceParameterKey,
        RootBinding, SemanticInputConstraint, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
    };

    const LENGTH: u64 = 65_535;

    struct PanicAfterBound {
        yielded: usize,
        value: ResolvedValueType,
    }

    impl Iterator for PanicAfterBound {
        type Item = ResolvedValueType;

        fn next(&mut self) -> Option<Self::Item> {
            assert!(
                self.yielded <= MAX_INDEX_REFINEMENT_SIGNATURE_VALUES,
                "the bounded signature constructor over-consumed its caller iterator"
            );
            self.yielded += 1;
            Some(self.value.clone())
        }
    }

    fn f32_type() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap())
    }

    fn encoded_boundary(components: usize) -> IndexRefinementBoundary {
        let field = CanonicalField::new(AttributeFieldId::new(1), CanonicalValue::boolean(true));
        let contract = if components == 0 {
            EncodedNumericContract::new([field]).unwrap()
        } else {
            EncodedNumericContract::with_components(
                [field],
                (1..=components).map(|role| {
                    EncodedComponentDeclaration::new(
                        EncodedComponentRole::new(u32::try_from(role).unwrap()),
                        f32_type(),
                        EncodedComponentShape::LogicalValue,
                    )
                }),
            )
            .unwrap()
        };
        IndexRefinementBoundary {
            value_type: ResolvedValueType::encoded_numeric(
                QuantSchemeKey::new("test", "resource-bound", 1).unwrap(),
                contract,
            )
            .unwrap(),
            shape: Shape::from_dims([1]),
        }
    }

    fn test_contract() -> NumericalContractIdentity {
        let key = F32NumericalContractKey::new(
            crate::schedule::SubnormalMode::Preserve,
            crate::schedule::SubnormalMode::Preserve,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::ApproximationEnvelope::Forbidden,
            crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
            crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
            crate::schedule::MaterializationRounding::NearestTiesToEven,
        )
        .unwrap();
        key.into()
    }

    #[test]
    fn a_validated_contract_key_converts_without_reparsing() {
        let key = F32NumericalContractKey::new(
            crate::schedule::SubnormalMode::Preserve,
            crate::schedule::SubnormalMode::Preserve,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::ApproximationEnvelope::Forbidden,
            crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
            crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
            crate::schedule::MaterializationRounding::NearestTiesToEven,
        )
        .unwrap();
        let spelling = key.as_str().to_owned();
        let identity = NumericalContractIdentity::from(key);
        assert_eq!(identity.as_str(), spelling);
    }

    #[test]
    fn signature_ingestion_stops_after_the_first_over_limit_value_on_each_side() {
        for side in [
            IndexRefinementSignatureSide::Operands,
            IndexRefinementSignatureSide::Results,
        ] {
            let unbounded = PanicAfterBound {
                yielded: 0,
                value: f32_type(),
            };
            let result = match side {
                IndexRefinementSignatureSide::Operands => {
                    IndexRefinementSignature::new(unbounded, [])
                }
                IndexRefinementSignatureSide::Results => {
                    IndexRefinementSignature::new([], unbounded)
                }
            };
            assert_eq!(
                result,
                Err(IndexRefinementVerificationError::SignatureTooLarge {
                    side,
                    actual: MAX_INDEX_REFINEMENT_SIGNATURE_VALUES + 1,
                    limit: MAX_INDEX_REFINEMENT_SIGNATURE_VALUES,
                })
            );
        }
    }

    #[test]
    fn raw_emitted_scalar_declarations_are_bounded_before_deduplication() {
        let semantic = FrozenSemanticRegistry::standard().unwrap();
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let signature =
            IndexRefinementSignature::new([f32_type(), f32_type()], [f32_type()]).unwrap();
        let emitted = vec![
            super::super::multiply_f32_scalar_op();
            MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS + 1
        ];
        assert!(matches!(
            IndexRealizationAuthority::admit(
                &semantic,
                &scalars,
                crate::semantic::multiply_f32_op(),
                signature,
                &emitted,
            ),
            Err(IndexRefinementVerificationError::EmittedScalarOperationsTooLarge {
                actual,
                limit: MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS,
            }) if actual == MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS + 1
        ));
    }

    struct BinaryIdentity;

    impl OperationInferencer for BinaryIdentity {
        fn infer(
            &self,
            request: OperationInferenceRequest<'_>,
            outputs: &mut OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            if request.operands().len() == 2 && request.attributes().fields().is_empty() {
                outputs.try_push(request.operands()[0].clone())
            } else {
                Err(OperationInferenceError::new(
                    ProviderDiagnosticCode::new("test.refinement-law.signature").unwrap(),
                    "test operation requires two operands and no attributes",
                )
                .unwrap())
            }
        }
    }

    struct RefinementLawProvider(Option<super::super::IndexRealizationLaw>);

    struct UnusedSemanticProvider(u32);

    struct ReachedSemanticProvider(u32);

    impl SemanticRegistryProvider for UnusedSemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "unused-refinement-semantics", self.0).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_operation(OperationDefinition::new(
                OpKey::new("test", "unused-refinement-operation", 1).unwrap(),
                OperationSchema::new(OperationArity::exact(2), OperationArity::exact(1), [])
                    .unwrap(),
                NormativeDefinitionRef::new("unused refinement operation")?,
                OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
                OperationConformance::new(CanonicalValue::boolean(true)),
                OperationEffect::Pure,
                Arc::new(BinaryIdentity),
            ))
        }
    }

    fn reached_semantic_operation() -> OpKey {
        OpKey::new("test", "reached-refinement-operation", 1).unwrap()
    }

    impl SemanticRegistryProvider for ReachedSemanticProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "reached-refinement-semantics", self.0).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            let operation = reached_semantic_operation();
            registrar.register_operation(OperationDefinition::new(
                operation.clone(),
                OperationSchema::new(OperationArity::exact(2), OperationArity::exact(1), [])
                    .unwrap(),
                NormativeDefinitionRef::new("test reached refinement operation")?,
                OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
                OperationConformance::new(CanonicalValue::boolean(true)),
                OperationEffect::Pure,
                Arc::new(BinaryIdentity),
            ))?;
            registrar.register_index_realization_law(
                operation,
                1,
                super::super::IndexRealizationLaw::multiply_f32(),
            )
        }
    }

    struct TestScalarConstant;

    impl ScalarOperationInferencer for TestScalarConstant {
        fn infer(
            &self,
            _request: ScalarInferenceRequest<'_>,
            outputs: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            outputs.try_push(f32_type())
        }
    }

    fn test_scalar_definition(key: ScalarOpKey, normative: &str) -> ScalarOperationDefinition {
        ScalarOperationDefinition::new(
            key,
            NormativeDefinitionRef::new(normative).unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::new([ScalarAttributeField::required(
                    crate::semantic::F32_CONSTANT_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                )])
                .unwrap(),
                ScalarArity::exact(0).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::boolean(true),
                CanonicalValue::boolean(true),
            ),
            Arc::new(TestScalarConstant),
        )
    }

    fn test_binary_scalar_definition(
        key: ScalarOpKey,
        normative: &str,
    ) -> ScalarOperationDefinition {
        ScalarOperationDefinition::new(
            key,
            NormativeDefinitionRef::new(normative).unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::boolean(true),
                CanonicalValue::boolean(true),
            ),
            Arc::new(TestScalarConstant),
        )
    }

    fn reached_semantic_fixture(
        revision: u32,
    ) -> (
        IndexRefinementSubject,
        ResolvedIndexRealization,
        VerifiedIndexRegionSequence,
        IndexRefinementReceipt,
    ) {
        let mut semantic = SemanticRegistryBuilder::standard().unwrap();
        semantic
            .register_provider(&ReachedSemanticProvider(revision))
            .unwrap();
        let semantic = semantic.freeze().unwrap();
        let mut scalars = ScalarRegistryBuilder::new(semantic.clone());
        scalars
            .register(
                ProviderIdentity::new("test", "selected-binary-scalar", 1).unwrap(),
                test_binary_scalar_definition(
                    super::super::multiply_f32_scalar_op(),
                    "test selected multiply scalar",
                ),
            )
            .unwrap();
        let scalars = scalars.freeze();
        let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
        let input = program
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([1]))
            .unwrap();
        let result = program
            .apply(
                reached_semantic_operation(),
                OperationAttributes::empty(),
                &[input.erase(), input.erase()],
            )
            .unwrap()
            .pop()
            .unwrap();
        program
            .output_resolved(OutputKey::new("output").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
                .unwrap();
        let resolution = laws.resolve(&subject).unwrap();
        let authority = IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            subject.operation().clone(),
            subject.signature().clone(),
            &[super::super::multiply_f32_scalar_op()],
        )
        .unwrap();
        let region = super::super::IndexRealizationLaw::multiply_f32()
            .realize(&subject, &scalars)
            .unwrap();
        let IndexRefinementVerificationOutcome::Verified(receipt) =
            resolution.verify(&authority, &region).unwrap()
        else {
            panic!("the reached fixture retains no residual proof")
        };
        (
            subject,
            resolution,
            VerifiedIndexRegionSequence::single(region),
            *receipt,
        )
    }

    fn constant_receipt_with_unused_authority(
        unused_semantic_revision: Option<u32>,
        constant_scalar_revision: u32,
        unused_scalar_revision: Option<u32>,
    ) -> IndexRefinementReceipt {
        let mut semantic = SemanticRegistryBuilder::standard().unwrap();
        if let Some(revision) = unused_semantic_revision {
            semantic
                .register_provider(&UnusedSemanticProvider(revision))
                .unwrap();
        }
        let semantic = semantic.freeze().unwrap();
        let mut scalars = ScalarRegistryBuilder::new(semantic.clone());
        scalars
            .register(
                ProviderIdentity::new("test", "selected-scalar", constant_scalar_revision).unwrap(),
                test_scalar_definition(
                    super::super::constant_f32_scalar_op(),
                    "test selected constant scalar",
                ),
            )
            .unwrap();
        if let Some(revision) = unused_scalar_revision {
            scalars
                .register(
                    ProviderIdentity::new("test", "unused-scalar", revision).unwrap(),
                    test_scalar_definition(
                        ScalarOpKey::new("test", "unused-scalar", 1).unwrap(),
                        "test unused scalar",
                    ),
                )
                .unwrap();
        }
        let scalars = scalars.freeze();
        let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
        let value = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
        program
            .output(OutputKey::new("value").unwrap(), value)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
                .unwrap();
        let resolution = laws.resolve(&subject).unwrap();
        let authority = IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            subject.operation().clone(),
            subject.signature().clone(),
            &[super::super::constant_f32_scalar_op()],
        )
        .unwrap();
        let region = super::super::IndexRealizationLaw::constant_f32()
            .realize(&subject, &scalars)
            .unwrap();
        let IndexRefinementVerificationOutcome::Verified(receipt) =
            resolution.verify(&authority, &region).unwrap()
        else {
            panic!("a constant realization retains no residual proof")
        };
        *receipt
    }

    #[test]
    fn executable_coverage_excludes_unused_authority_but_retains_reached_scalar_provenance() {
        let baseline = constant_receipt_with_unused_authority(None, 1, None);
        let unused_semantic = constant_receipt_with_unused_authority(Some(1), 1, None);
        let unused_semantic_revision = constant_receipt_with_unused_authority(Some(2), 1, None);
        let unused_scalar = constant_receipt_with_unused_authority(None, 1, Some(1));
        let unused_scalar_revision = constant_receipt_with_unused_authority(None, 1, Some(2));
        let reached_scalar_revision = constant_receipt_with_unused_authority(None, 2, None);

        for receipt in [
            &unused_semantic,
            &unused_semantic_revision,
            &unused_scalar,
            &unused_scalar_revision,
        ] {
            assert_eq!(
                baseline.executable_coverage_identity(),
                receipt.executable_coverage_identity()
            );
            assert_ne!(baseline.identity(), receipt.identity());
        }
        assert_ne!(
            baseline.executable_coverage_identity(),
            reached_scalar_revision.executable_coverage_identity()
        );
        let (_, _, _, reached_semantic) = reached_semantic_fixture(1);
        let (_, _, _, reached_semantic_revision) = reached_semantic_fixture(2);
        assert_eq!(reached_semantic.graph(), reached_semantic_revision.graph());
        assert_eq!(
            reached_semantic.final_stage(),
            reached_semantic_revision.final_stage()
        );
        assert_ne!(
            reached_semantic.executable_coverage_identity(),
            reached_semantic_revision.executable_coverage_identity()
        );
    }

    fn alternate_test_contract() -> NumericalContractIdentity {
        F32NumericalContractKey::new(
            crate::schedule::SubnormalMode::Preserve,
            crate::schedule::SubnormalMode::Preserve,
            crate::schedule::NumericalPermission::Permitted,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::NumericalPermission::Forbidden,
            crate::schedule::ApproximationEnvelope::Forbidden,
            crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
            crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
            crate::schedule::MaterializationRounding::NearestTiesToEven,
        )
        .unwrap()
        .into()
    }

    #[test]
    fn executable_coverage_retains_each_replay_and_substitution_boundary() {
        let (subject, resolution, realization, receipt) = reached_semantic_fixture(1);
        let encode = |subject: &IndexRefinementSubject,
                      resolution: &ResolvedIndexRealization,
                      realization: &VerifiedIndexRegionSequence,
                      operands: &[OperandBinding],
                      results: &[ResultBinding],
                      proofs: &[IndexRefinementDomainProof]| {
            encode_executable_coverage_identity(
                subject,
                resolution,
                realization,
                &receipt.scalar_authorities(),
                operands,
                results,
                proofs,
            )
        };
        let baseline = encode(
            &subject,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            receipt.index_domain_proofs(),
        );
        assert_eq!(baseline, receipt.executable_coverage_identity().as_bytes());

        // A provider revision is excluded from graph meaning, so the graph
        // perturbation needs a program with a genuinely different selected
        // operation rather than another revision of the same one.
        let foreign = constant_receipt_with_unused_authority(None, 1, None);
        assert_ne!(subject.graph(), foreign.graph());
        let mut changed = subject.clone();
        changed.graph = foreign.graph().clone();
        assert_ne!(
            baseline,
            encode(
                &changed,
                &resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let mut changed = subject.clone();
        changed.occurrence = SemanticOccurrence::new(subject.occurrence().get() + 1);
        assert_ne!(
            baseline,
            encode(
                &changed,
                &resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let mut changed = subject.clone();
        changed.numerical_contract = alternate_test_contract();
        assert_ne!(
            baseline,
            encode(
                &changed,
                &resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let changed_region = VerifiedIndexRegionSequence::single(residual_region(1, 5, 0));
        assert_ne!(
            baseline,
            encode(
                &subject,
                &resolution,
                &changed_region,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let mut changed = subject.clone();
        let law_row = changed
            .realization_law_row
            .as_mut()
            .expect("the reached operation carries a law row");
        let mut changed_law_row = law_row.to_vec();
        changed_law_row.push(0xff);
        *law_row = changed_law_row.into_boxed_slice();
        assert_ne!(
            baseline,
            encode(
                &changed,
                &resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let mut changed_resolution = resolution.clone();
        changed_resolution.provider =
            ProviderIdentity::new("test", "different-reached-law-provider", 1).unwrap();
        assert_ne!(
            baseline,
            encode(
                &subject,
                &changed_resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let mut changed_resolution = resolution.clone();
        changed_resolution.revision += 1;
        assert_ne!(
            baseline,
            encode(
                &subject,
                &changed_resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[]
            )
        );

        let mut operands = receipt.operand_bindings().to_vec();
        operands[0].operand += 1;
        assert_ne!(
            baseline,
            encode(
                &subject,
                &resolution,
                &realization,
                &operands,
                receipt.result_bindings(),
                &[]
            )
        );

        let mut results = receipt.result_bindings().to_vec();
        results[0].result += 1;
        assert_ne!(
            baseline,
            encode(
                &subject,
                &resolution,
                &realization,
                receipt.operand_bindings(),
                &results,
                &[]
            )
        );

        let proof_region = residual_region(1, 5, 0);
        let obligation = proof_region
            .unknown_index_domain_predicates()
            .next()
            .expect("the proof fixture retains one obligation");
        let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
        let proof = IndexDomainProofEvidence::ExhaustiveFinite {
            points: 2,
            derivation: EXHAUSTIVE_DERIVATION.into(),
        };
        let proof = IndexRefinementDomainProof {
            stage: 0,
            obligation,
            authority: authority.clone(),
            identity: IndexRefinementDomainProofIdentity(
                encode_proof_identity(&proof_region, obligation, &authority, &proof)
                    .into_boxed_slice(),
            ),
            proof,
        };
        assert_ne!(
            baseline,
            encode(
                &subject,
                &resolution,
                &realization,
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[proof]
            )
        );
    }

    /// The digest is what separates two graphs at one occurrence ordinal.
    ///
    /// ADR 0104 replaced the framed `SemanticGraphIdentity` at the head of every
    /// coverage record with a fixed-width digest of it. The record's documented
    /// claim — that it names "this occurrence of *this* graph" — then rests on
    /// the digest rather than on a restatement, so this pins all three halves of
    /// that: the preimage is gone from the bytes, the digest is present at the
    /// exact position it left, and two graphs sharing one occurrence ordinal
    /// still mint different coverage identities.
    ///
    /// The neighbouring replay-and-substitution test already perturbs the graph
    /// and watches the bytes move, and it would keep passing if the encoder had
    /// written the graph identity whole. It is the *position* assertion here
    /// that says which encoding produced the difference, which is the fact the
    /// linear identity curve depends on.
    #[test]
    fn one_occurrence_of_two_graphs_is_separated_by_the_folded_graph_digest() {
        let (subject, resolution, realization, receipt) = reached_semantic_fixture(1);
        let encode = |subject: &IndexRefinementSubject| {
            encode_executable_coverage_identity(
                subject,
                &resolution,
                &realization,
                &receipt.scalar_authorities(),
                receipt.operand_bindings(),
                receipt.result_bindings(),
                &[],
            )
        };

        let baseline = encode(&subject);
        let head = EXECUTABLE_COVERAGE_IDENTITY_TAG.len();
        assert_eq!(
            &baseline[head..head + DIGEST_BYTES],
            DigestAlgorithm::GOVERNED
                .digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())
                .as_bytes(),
            "the record opens with the governed digest of its bound graph",
        );

        let graph_preimage = subject.graph.as_bytes();
        assert!(
            !baseline
                .windows(graph_preimage.len())
                .any(|window| window == graph_preimage),
            "the graph identity preimage still occurs in the coverage record",
        );

        // A second graph at the same occurrence ordinal. The constant fixture
        // selects a different operation, which is what makes its graph identity
        // genuinely different rather than another revision of one.
        let foreign = constant_receipt_with_unused_authority(None, 1, None);
        assert_ne!(subject.graph(), foreign.graph());
        let mut other = subject.clone();
        other.graph = foreign.graph().clone();
        let separated = encode(&other);
        assert_eq!(
            other.occurrence, subject.occurrence,
            "the ordinal is held fixed so the graph is the only thing that moved",
        );
        assert_ne!(
            baseline, separated,
            "two graphs at one occurrence ordinal minted equal coverage bytes",
        );
        assert_eq!(
            baseline[head + DIGEST_BYTES..],
            separated[head + DIGEST_BYTES..],
            "the graph digest is the only field that moved",
        );
    }

    fn test_law_operation() -> OpKey {
        OpKey::new("test", "refinement-law-row", 1).unwrap()
    }

    impl SemanticRegistryProvider for RefinementLawProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "refinement-law-provider", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            let operation = test_law_operation();
            registrar.register_operation(OperationDefinition::new(
                operation.clone(),
                OperationSchema::new(OperationArity::exact(2), OperationArity::exact(1), [])
                    .unwrap(),
                NormativeDefinitionRef::new("test refinement-law-row v1")?,
                OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
                OperationConformance::new(CanonicalValue::boolean(true)),
                OperationEffect::Pure,
                Arc::new(BinaryIdentity),
            ))?;
            if let Some(law) = &self.0 {
                registrar.register_index_realization_law(operation, 1, law.clone())?;
            }
            Ok(())
        }
    }

    fn semantic_with_test_law(
        law: Option<super::super::IndexRealizationLaw>,
    ) -> FrozenSemanticRegistry {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder
            .register_provider(&RefinementLawProvider(law))
            .unwrap();
        builder.freeze().unwrap()
    }

    // ---- The staged realization vocabulary -------------------------------
    //
    // These exercise a law form no standard operation carries: registering the
    // normalization that will carry it belongs to that family's own ticket, and
    // needs a governed reciprocal square root that does not yet exist. What is
    // tested here is the vocabulary the family will be stated in — the ordered
    // chain, its identity, the receipt that binds every stage, and the refusals.

    /// Ordered axes attribute for the staged test operation's fold.
    const STAGED_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

    /// Length of the folded axis in every staged fixture.
    const STAGED_LENGTH: u64 = 4;

    fn staged_test_operation() -> OpKey {
        OpKey::new("test", "staged-fold-then-pass", 1).unwrap()
    }

    /// Result type and shape follow the *second* operand, the elementwise one.
    struct StagedFoldThenPass;

    impl OperationInferencer for StagedFoldThenPass {
        fn infer(
            &self,
            request: OperationInferenceRequest<'_>,
            outputs: &mut OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            if request.operands().len() == 2 {
                outputs.try_push(request.operands()[1].clone())
            } else {
                Err(OperationInferenceError::new(
                    ProviderDiagnosticCode::new("test.staged.signature").unwrap(),
                    "the staged test operation requires two operands",
                )
                .unwrap())
            }
        }
    }

    struct StagedLawProvider(Option<super::super::IndexRealizationLaw>);

    impl SemanticRegistryProvider for StagedLawProvider {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "staged-law-provider", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            let operation = staged_test_operation();
            registrar.register_operation(OperationDefinition::new(
                operation.clone(),
                OperationSchema::new(
                    OperationArity::exact(2),
                    OperationArity::exact(1),
                    [crate::semantic::OperationAttributeSchema::required(
                        STAGED_AXES_ATTRIBUTE,
                        CanonicalValueKind::Sequence,
                    )],
                )
                .unwrap(),
                NormativeDefinitionRef::new("test staged-fold-then-pass v1")?,
                OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
                OperationConformance::new(CanonicalValue::boolean(true)),
                OperationEffect::Pure,
                Arc::new(StagedFoldThenPass),
            ))?;
            if let Some(law) = &self.0 {
                registrar.register_index_realization_law(operation, 1, law.clone())?;
            }
            Ok(())
        }
    }

    fn staged_law() -> super::super::IndexRealizationLaw {
        super::super::IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: STAGED_AXES_ATTRIBUTE,
            scalar: super::super::multiply_f32_scalar_op(),
        }
    }

    /// Complete authorities and subject for one staged-law occurrence.
    struct StagedFixture {
        scalars: FrozenScalarRegistry,
        subject: IndexRefinementSubject,
        resolution: ResolvedIndexRealization,
        authority: IndexRealizationAuthority,
    }

    /// The two scalar operations the staged vocabulary reaches.
    ///
    /// The fold's tail combine is the governed add and the pass applies the
    /// governed multiply; neither the empty-domain constant nor the
    /// single-contributor canonicalization is reachable at a folded extent above
    /// one, so registering them would admit authority nothing here uses.
    fn staged_scalars(semantic: &FrozenSemanticRegistry) -> FrozenScalarRegistry {
        let mut scalars = ScalarRegistryBuilder::new(semantic.clone());
        let provider = ProviderIdentity::new("test", "staged-scalars", 1).unwrap();
        for (key, normative) in [
            (
                super::super::multiply_f32_scalar_op(),
                "test staged multiply",
            ),
            (super::super::add_f32_scalar_op(), "test staged add"),
        ] {
            scalars
                .register(
                    provider.clone(),
                    test_binary_scalar_definition(key, normative),
                )
                .unwrap();
        }
        scalars.freeze()
    }

    fn staged_fixture(law: super::super::IndexRealizationLaw) -> StagedFixture {
        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder
            .register_provider(&StagedLawProvider(Some(law)))
            .unwrap();
        let semantic = builder.freeze().unwrap();
        let scalars = staged_scalars(&semantic);

        let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
        let folded = program
            .input::<F32>(
                InputKey::new("folded").unwrap(),
                Shape::from_dims([STAGED_LENGTH]),
            )
            .unwrap();
        let elementwise = program
            .input::<F32>(
                InputKey::new("elementwise").unwrap(),
                Shape::from_dims([STAGED_LENGTH]),
            )
            .unwrap();
        let axes = CanonicalValue::sequence([CanonicalValue::unsigned_u32(0)]).unwrap();
        let value = program
            .apply(
                staged_test_operation(),
                OperationAttributes::new([CanonicalField::new(STAGED_AXES_ATTRIBUTE, axes)])
                    .unwrap(),
                &[folded.erase(), elementwise.erase()],
            )
            .unwrap()
            .pop()
            .unwrap();
        program
            .output_resolved(OutputKey::new("scaled").unwrap(), value)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
                .unwrap();
        let resolution = laws.resolve(&subject).unwrap();
        let authority = IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            subject.operation().clone(),
            subject.signature().clone(),
            &[
                super::super::multiply_f32_scalar_op(),
                super::super::add_f32_scalar_op(),
            ],
        )
        .unwrap();
        StagedFixture {
            scalars,
            subject,
            resolution,
            authority,
        }
    }

    impl StagedFixture {
        fn realized(&self) -> VerifiedIndexRegionSequence {
            self.resolution
                .law
                .realize_sequence(&self.subject, &self.scalars)
                .expect("the staged law realizes its occurrence")
        }

        fn verify(
            &self,
            realization: &VerifiedIndexRegionSequence,
        ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
            self.resolution
                .verify_sequence(&self.authority, realization)
        }
    }

    /// The whole point: an occurrence whose realization is two regions gets a
    /// receipt that binds both of them.
    #[test]
    fn a_staged_occurrence_verifies_and_binds_every_region() {
        let fixture = staged_fixture(staged_law());
        let realization = fixture.realized();
        assert_eq!(realization.stage_count(), 2);
        assert_eq!(realization.intermediates().len(), 1);
        // The fold removed the only axis, so what it hands on is rank zero and
        // the pass reads it once per point.
        assert_eq!(realization.intermediates()[0].shape().rank(), 0);

        let IndexRefinementVerificationOutcome::Verified(receipt) = fixture
            .verify(&realization)
            .expect("the law's own realization verifies")
        else {
            panic!("the staged fixture retains no residual obligation")
        };
        assert_eq!(receipt.regions().len(), 2);
        assert_eq!(
            receipt.regions(),
            realization
                .stages()
                .map(|stage| stage.canonical_identity().clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(receipt.realization(), realization.identity());
        assert_eq!(
            receipt.final_stage(),
            realization.final_stage().canonical_identity()
        );
        // Both stages' scalar authorities are retained, and they genuinely
        // differ: at this folded extent the fold's tail combine reaches the
        // governed add and nothing else, and the pass reaches the multiply and
        // nothing else.
        assert_eq!(receipt.scalar_authorities().len(), 2);
        assert_ne!(
            receipt.scalar_authorities()[0],
            receipt.scalar_authorities()[1]
        );

        // The folded operand is read by the fold and the elementwise operand by
        // the pass, so the bindings name two different stages.
        let stages = receipt
            .operand_bindings()
            .iter()
            .map(|binding| (binding.operand(), binding.stage()))
            .collect::<Vec<_>>();
        assert_eq!(stages, vec![(0, 0), (1, 1)]);
        assert_eq!(receipt.result_bindings().len(), 1);

        // Domain separation, checked rather than only argued: a staged receipt
        // and its coverage are written under their own tags, so no staged
        // encoding can spell a single-region one — which is what lets the
        // one-stage encoding stay exactly the bytes it has always been.
        assert!(
            receipt
                .identity()
                .as_bytes()
                .starts_with(STAGED_RECEIPT_IDENTITY_TAG)
        );
        assert!(
            receipt
                .executable_coverage_identity()
                .as_bytes()
                .starts_with(STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG)
        );
        let (_, _, _, single_stage) = reached_semantic_fixture(1);
        assert!(
            single_stage
                .identity()
                .as_bytes()
                .starts_with(RECEIPT_IDENTITY_TAG)
        );
        assert!(
            single_stage
                .executable_coverage_identity()
                .as_bytes()
                .starts_with(EXECUTABLE_COVERAGE_IDENTITY_TAG)
        );
        assert!(!RECEIPT_IDENTITY_TAG.starts_with(STAGED_RECEIPT_IDENTITY_TAG));
        assert!(!STAGED_RECEIPT_IDENTITY_TAG.starts_with(RECEIPT_IDENTITY_TAG));
        assert!(
            !EXECUTABLE_COVERAGE_IDENTITY_TAG.starts_with(STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG)
        );
        assert!(
            !STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG.starts_with(EXECUTABLE_COVERAGE_IDENTITY_TAG)
        );
    }

    /// A staged realization's containment check covers every stage.
    ///
    /// Admitting only the multiply the *pass* reaches leaves the fold's own
    /// governed additions unadmitted, and the realization is refused as a whole.
    #[test]
    fn an_unadmitted_scalar_in_an_earlier_stage_refuses_the_realization() {
        let fixture = staged_fixture(staged_law());
        let realization = fixture.realized();
        let narrow = IndexRealizationAuthority::admit(
            &FrozenSemanticRegistry::standard().unwrap(),
            &fixture.scalars,
            fixture.subject.operation().clone(),
            fixture.subject.signature().clone(),
            &[super::super::multiply_f32_scalar_op()],
        );
        // The narrow authority is built over the standard registry, which does
        // not define the staged test operation at all, so admission itself is
        // what refuses first; rebuild it over the fixture's own authority.
        assert!(narrow.is_err());

        let mut builder = SemanticRegistryBuilder::standard().unwrap();
        builder
            .register_provider(&StagedLawProvider(Some(staged_law())))
            .unwrap();
        let semantic = builder.freeze().unwrap();
        let narrow = IndexRealizationAuthority::admit(
            &semantic,
            &fixture.scalars,
            fixture.subject.operation().clone(),
            fixture.subject.signature().clone(),
            &[super::super::multiply_f32_scalar_op()],
        )
        .unwrap();
        assert_eq!(
            fixture
                .resolution
                .verify_sequence(&narrow, &realization)
                .unwrap_err(),
            IndexRefinementVerificationError::ScalarAuthorityConformance
        );
    }

    /// The rubber-stamp perturbation: a well-formed chain that realizes
    /// something else is refused.
    ///
    /// Both candidates below are structurally valid region sequences —
    /// [`VerifiedIndexRegionSequence::try_new`] accepted them — so nothing about
    /// their own construction says no. What refuses is the comparison against
    /// the law's own realization, which is the only thing standing between a
    /// receipt and a provider that emitted a plausible chain for the wrong
    /// operation.
    #[test]
    fn a_chain_that_does_not_realize_the_occurrence_is_refused() {
        let fixture = staged_fixture(staged_law());
        let realization = fixture.realized();

        // Cross-family: the chain built for the *other* scalar. Every stage is
        // well formed and the wiring is identical; only the pass's arithmetic
        // differs, and that is enough.
        let other = staged_fixture(
            super::super::IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute: STAGED_AXES_ATTRIBUTE,
                scalar: super::super::add_f32_scalar_op(),
            },
        );
        let foreign = other.realized();
        assert_ne!(realization.identity(), foreign.identity());
        let refusal = fixture.verify(&foreign).unwrap_err();
        assert!(
            matches!(
                refusal,
                IndexRefinementVerificationError::SemanticRealizationSequenceMismatch { .. }
            ),
            "observed {refusal:?}"
        );

        // Wrong order: the same two regions, chained the other way round. The
        // reversal composes — `try_new` accepts it — but running the pass first
        // means the occurrence's folded operand would have to be read through a
        // boundary shaped like the fold's *result*, and the ordered interface
        // check reaches that one statement before the identity comparison does.
        //
        // Asserted at the exact position rather than "some refusal": the
        // boundary that disagrees is the evidence the order was wrong, and a
        // test satisfied by any refusal would pass for a fixture that had
        // stopped building chains at all.
        let stages = realization.stages().cloned().collect::<Vec<_>>();
        let reversed = VerifiedIndexRegionSequence::try_new(
            vec![stages[1].clone(), stages[0].clone()],
            vec![
                vec![
                    StagedInputSource::Occurrence(1),
                    StagedInputSource::Occurrence(0),
                ],
                vec![StagedInputSource::Intermediate(0)],
            ],
        )
        .expect("the reversed chain is structurally well formed");
        assert_ne!(realization.identity(), reversed.identity());
        assert_eq!(
            fixture.verify(&reversed).unwrap_err(),
            IndexRefinementVerificationError::OperandInterface { position: 0 }
        );
    }

    /// One region for a two-region occurrence, and two for a one-region one.
    ///
    /// The ticket's own perturbation, in both directions: a chain cannot be
    /// presented for a law that declares one region, and one region cannot
    /// certify a law whose realization is a chain.
    ///
    /// **Where each direction is caught differs, and that is worth recording.**
    /// A truncated chain drops the fold, so the pass's handed input boundary now
    /// claims to be an occurrence input and disagrees with it — the ordered
    /// interface check names that boundary before the realization comparison
    /// runs. A chain presented for a one-region law binds cleanly and is caught
    /// by the comparison itself. Both are typed refusals and neither mints a
    /// receipt; what would be wrong is a candidate that reached one of these
    /// paths and was approved by the other.
    #[test]
    fn region_count_disagreements_refuse_in_both_directions() {
        let fixture = staged_fixture(staged_law());
        let realization = fixture.realized();
        let stages = realization.stages().cloned().collect::<Vec<_>>();

        // A staged law against the pass alone. Its second boundary is the fold's
        // rank-zero result, which the occurrence's second input is not.
        let truncated = VerifiedIndexRegionSequence::single(stages[1].clone());
        assert_eq!(
            fixture.verify(&truncated).unwrap_err(),
            IndexRefinementVerificationError::OperandInterface { position: 1 }
        );
        // And through the single-region entry point the compiler drives, where
        // the law refuses before any comparison is reached.
        let refusal = fixture
            .resolution
            .verify(&fixture.authority, &stages[1])
            .unwrap_err();
        assert!(
            matches!(
                refusal,
                IndexRefinementVerificationError::SemanticRealizationLawRefused {
                    rule: "staged-law-requires-region-sequence",
                    ..
                }
            ),
            "observed {refusal:?}"
        );

        // A single-region law against a two-region candidate.
        let semantic =
            semantic_with_test_law(Some(super::super::IndexRealizationLaw::multiply_f32()));
        let scalars = staged_scalars(&semantic);
        let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
        let input = program
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([1]))
            .unwrap();
        let value = program
            .apply(
                test_law_operation(),
                OperationAttributes::empty(),
                &[input.erase(), input.erase()],
            )
            .unwrap()
            .pop()
            .unwrap();
        program
            .output_resolved(OutputKey::new("output").unwrap(), value)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
                .unwrap();
        let resolution = laws.resolve(&subject).unwrap();
        let authority = IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            subject.operation().clone(),
            subject.signature().clone(),
            &[super::super::multiply_f32_scalar_op()],
        )
        .unwrap();
        let single = super::super::IndexRealizationLaw::multiply_f32()
            .realize(&subject, &scalars)
            .unwrap();
        // The one-stage candidate the law does expect still verifies, which is
        // what makes the two-stage refusal below attributable to stage count
        // rather than to a broken fixture.
        assert!(matches!(
            resolution
                .verify_sequence(
                    &authority,
                    &VerifiedIndexRegionSequence::single(single.clone())
                )
                .unwrap(),
            IndexRefinementVerificationOutcome::Verified(_)
        ));
        // The operation aliases one input into both operands, so its region has
        // one input boundary; running the region twice, the second copy reading
        // the first's result, is a chain whose every interface agrees with the
        // occurrence. Nothing but the whole-realization comparison can refuse
        // it, which is exactly what makes it the case worth stating.
        let doubled = VerifiedIndexRegionSequence::try_new(
            vec![single.clone(), single],
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![StagedInputSource::Intermediate(0)],
            ],
        )
        .expect("squaring twice, the second reading the first, is a well-formed chain");
        let refusal = resolution
            .verify_sequence(&authority, &doubled)
            .unwrap_err();
        assert!(
            matches!(
                refusal,
                IndexRefinementVerificationError::SemanticRealizationSequenceMismatch { .. }
            ),
            "observed {refusal:?}"
        );
    }

    #[test]
    fn operation_specific_law_rows_are_checked_across_public_registry_boundaries() {
        let semantic_a =
            semantic_with_test_law(Some(super::super::IndexRealizationLaw::multiply_f32()));
        let semantic_b = semantic_with_test_law(Some(super::super::IndexRealizationLaw::add_f32()));
        let semantic_absent = semantic_with_test_law(None);
        assert_eq!(
            semantic_a.snapshot_identity(),
            semantic_b.snapshot_identity()
        );
        assert_eq!(
            semantic_a.snapshot_identity(),
            semantic_absent.snapshot_identity()
        );
        let scalars_a = ScalarRegistryBuilder::new(semantic_a.clone()).freeze();
        let scalars_b = ScalarRegistryBuilder::new(semantic_b.clone()).freeze();
        let scalars_absent = ScalarRegistryBuilder::new(semantic_absent.clone()).freeze();
        assert_eq!(scalars_a.snapshot_identity(), scalars_b.snapshot_identity());
        assert_eq!(
            scalars_a.snapshot_identity(),
            scalars_absent.snapshot_identity()
        );
        let mut program = SemanticProgramBuilder::try_new(semantic_a.clone()).unwrap();
        let input = program
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([1]))
            .unwrap();
        let value = program
            .apply(
                test_law_operation(),
                OperationAttributes::empty(),
                &[input.erase(), input.erase()],
            )
            .unwrap()
            .pop()
            .unwrap();
        program
            .output_resolved(OutputKey::new("output").unwrap(), value)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let laws_a =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic_a, scalars_a).unwrap();
        for (semantic, scalars) in [
            (semantic_b.clone(), scalars_b.clone()),
            (semantic_absent, scalars_absent),
        ] {
            let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars).unwrap();
            assert!(matches!(
                laws.resolve(&subject),
                Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch)
            ));
        }
        let resolution = laws_a.resolve(&subject).unwrap();
        let signature = subject.signature().clone();
        let lowering = IndexRealizationAuthority::admit(
            &semantic_b,
            &scalars_b,
            test_law_operation(),
            signature,
            &[],
        )
        .unwrap();
        assert_eq!(
            check_lowering_authority(&subject, &resolution, &lowering),
            Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch)
        );
    }

    fn residual_region(second_extent: u64, rounds: usize, offset: i128) -> VerifiedIndexRegion {
        residual_region_with_extents(
            &[LENGTH, second_extent],
            0,
            rounds,
            1_i128.into(),
            offset.into(),
        )
    }

    fn residual_region_with_extents(
        extents: &[u64],
        target_axis: usize,
        rounds: usize,
        multiplier: IndexInteger,
        offset: IndexInteger,
    ) -> VerifiedIndexRegion {
        let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
            .expect("the fixture receives a fresh builder identity");
        let dimensions = extents
            .iter()
            .map(|extent| {
                builder
                    .dimension(DomainRole::Parallel, Extent::new(*extent))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let shape = Shape::try_from_dims(extents.iter().copied()).unwrap();
        let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
        let input = builder
            .tensor(TensorRole::Input, value_type.clone(), shape.clone())
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type, shape)
            .unwrap();
        let coordinates = dimensions
            .iter()
            .map(|dimension| builder.dimension_expr(*dimension).unwrap())
            .collect::<Vec<_>>();
        let mut conservative = coordinates[target_axis];
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
        if multiplier != 1_i128.into() {
            conservative = builder
                .linear_combination(0_i128.into(), &[(multiplier, conservative)])
                .unwrap();
        }
        if offset != 0_i128.into() {
            conservative = builder
                .linear_combination(offset, &[(1_i128.into(), conservative)])
                .unwrap();
        }
        let mut read_coordinates = coordinates.clone();
        read_coordinates[target_axis] = conservative;
        let value = builder.read(input, &dimensions, &read_coordinates).unwrap();
        let write = builder.write(output, &dimensions, &coordinates).unwrap();
        builder.output(write, value).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(region.unknown_index_domain_predicates().len(), 1);
        region
    }

    fn two_domain_residual_region() -> VerifiedIndexRegion {
        let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
            .expect("the fixture receives a fresh builder identity");
        let value_type = f32_type();
        let mut dimensions = Vec::new();
        let mut coordinates = Vec::new();
        let mut values = Vec::new();
        for _ in 0..2 {
            let dimension = builder
                .dimension(DomainRole::Parallel, Extent::new(LENGTH))
                .unwrap();
            let shape = Shape::from_dims([LENGTH]);
            let input = builder
                .tensor(TensorRole::Input, value_type.clone(), shape.clone())
                .unwrap();
            let coordinate = builder.dimension_expr(dimension).unwrap();
            let mut conservative = coordinate;
            for _ in 0..5 {
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
            let value = builder.read(input, &[dimension], &[conservative]).unwrap();
            dimensions.push(dimension);
            coordinates.push(coordinate);
            values.push(value);
        }
        let sum = builder
            .apply(
                super::super::add_f32_scalar_op(),
                super::super::ScalarAttributes::empty(),
                &values,
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                value_type,
                Shape::from_dims([LENGTH, LENGTH]),
            )
            .unwrap();
        let write = builder.write(output, &dimensions, &coordinates).unwrap();
        builder.output(write, sum.get(0).unwrap()).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(region.unknown_index_domain_predicates().len(), 2);
        region
    }

    fn assess(
        region: &VerifiedIndexRegion,
        cells: u64,
        integer_bytes: u64,
    ) -> IndexDomainProofClaim {
        let obligation = region
            .unknown_index_domain_predicates()
            .next()
            .expect("the fixture retains one residual");
        assess_finite_domains(
            region,
            &[obligation],
            IndexDomainProofBudget::try_new(cells, integer_bytes).unwrap(),
        )
        .pop()
        .unwrap()
    }

    #[test]
    fn exact_finite_evaluation_refuses_when_conservative_work_exceeds_hard_limit() {
        let claim = assess(
            &residual_region(1, 5, 0),
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        );
        assert!(matches!(
            claim,
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
        ));
    }

    #[test]
    fn exact_finite_evaluation_returns_the_first_counterexample() {
        let region = residual_region(1, 5, 1);
        let first = assess(
            &region,
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        );
        let second = assess(
            &region,
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        );
        assert_eq!(first, second);
        assert!(matches!(
            first,
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
        ));
    }

    #[test]
    fn wide_counterexample_is_encoded_by_exact_point_ordinal() {
        let encoded = encode_counterexample(0);
        assert!(encoded.len() <= MAX_DOMAIN_EVIDENCE_BYTES);
        assert_eq!(&encoded[..COUNTEREXAMPLE_TAG.len()], COUNTEREXAMPLE_TAG);
    }

    #[test]
    fn an_empty_domain_is_vacuously_proved_before_overflowing_prefixes() {
        assert_eq!(finite_point_count(&[u64::MAX, u64::MAX, 0]), Some(0));
    }

    #[test]
    fn exact_finite_evaluation_fails_closed_at_the_callers_budget() {
        let claim = assess(
            &residual_region(1, 5, 0),
            1,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        );
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
    fn exact_finite_evaluation_charges_integer_byte_work() {
        let claim = assess(&residual_region(1, 5, 0), MAX_FINITE_DOMAIN_PROOF_CELLS, 1);
        assert!(matches!(
            claim,
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit: 1,
            }) if required > 1
        ));
    }

    #[test]
    fn linear_integer_work_reports_one_exact_preflight_charge() {
        let region = residual_region(1, 5, 0);
        let obligation = region.unknown_index_domain_predicates().next().unwrap();
        let required = match assess_finite_domains(
            &region,
            &[obligation],
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1).unwrap(),
        )[0]
        {
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                ..
            }) => u64::try_from(required).unwrap(),
            ref claim => panic!("one-byte perturbation did not expose charge: {claim:?}"),
        };
        assert!(required > MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES);
    }

    #[test]
    fn whole_call_ledger_preserves_earlier_group_and_stops_later_groups_atomically() {
        let region = two_domain_residual_region();
        let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
        let claims = assess_finite_domains(
            &region,
            &obligations,
            IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            )
            .unwrap(),
        );
        assert!(claims.iter().all(|claim| matches!(
            claim,
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit,
            }) if *required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
                && *limit == MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES
        )));
    }

    #[test]
    fn unsupported_root_does_not_poison_same_group_siblings() {
        let region = two_domain_residual_region();
        let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
        let second_dimension = region
            .access(obligations[1].subject())
            .unwrap()
            .domain()
            .next()
            .unwrap();
        let second_bound = match obligations[1].predicate() {
            IndexDomainPredicate::LessThanExtent { extent, .. } => {
                resolve_extent(&region, extent).unwrap()
            }
            IndexDomainPredicate::NonNegative { .. } => panic!("fixture must retain upper bound"),
        };
        let group = IndexDomainGroup {
            domain: IndexDomainKey(vec![(second_dimension, 1)]),
            points: 1,
            obligations: vec![
                PlannedDomainObligation {
                    slot: 0,
                    obligation: obligations[0],
                    upper_bound: Some(second_bound),
                },
                PlannedDomainObligation {
                    slot: 1,
                    obligation: obligations[1],
                    upper_bound: Some(second_bound),
                },
                PlannedDomainObligation {
                    slot: 2,
                    obligation: obligations[1],
                    upper_bound: Some(0),
                },
            ],
        };
        let mut ledger = IndexDomainProofLedger::new(
            IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            )
            .unwrap(),
        );
        let claims = assess_domain_group(&region, &group, &mut ledger).unwrap();
        assert!(matches!(
            claims.as_slice(),
            [
                IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment),
                IndexDomainProofClaim::Proved(_),
                IndexDomainProofClaim::Disproved(_),
            ]
        ));
    }

    #[test]
    fn all_unsupported_group_skips_large_domain_evaluation_reservation() {
        let region = two_domain_residual_region();
        let obligation = region.unknown_index_domain_predicates().next().unwrap();
        let group = IndexDomainGroup {
            domain: IndexDomainKey(Vec::new()),
            points: u64::MAX,
            obligations: vec![PlannedDomainObligation {
                slot: 0,
                obligation,
                upper_bound: Some(LENGTH),
            }],
        };
        let mut ledger =
            IndexDomainProofLedger::new(IndexDomainProofBudget::try_new(128, 1).unwrap());
        let claims = assess_domain_group(&region, &group, &mut ledger).unwrap();
        assert!(matches!(
            claims.as_slice(),
            [IndexDomainProofClaim::Unknown(
                IndexDomainUnknownReason::UnsupportedFragment
            )]
        ));
        assert_eq!(ledger.used_integer_bytes, 0);
        assert!(ledger.exhaustion.is_none());
    }

    #[test]
    fn manageable_shared_dag_has_exact_grouped_and_minus_one_charges() {
        let region = two_domain_residual_region();
        let obligation = region.unknown_index_domain_predicates().next().unwrap();
        let access = region.access(obligation.subject()).unwrap();
        let dimension = access.domain().next().unwrap();
        let upper_bound = match obligation.predicate() {
            IndexDomainPredicate::LessThanExtent { extent, .. } => {
                resolve_extent(&region, extent).unwrap()
            }
            IndexDomainPredicate::NonNegative { .. } => panic!("fixture must retain upper bound"),
        };
        let make_group = |copies: usize| IndexDomainGroup {
            domain: IndexDomainKey(vec![(dimension, LENGTH)]),
            points: 1,
            obligations: (0..copies)
                .map(|slot| PlannedDomainObligation {
                    slot,
                    obligation,
                    upper_bound: Some(upper_bound),
                })
                .collect(),
        };
        let required = |copies| {
            let mut ledger = IndexDomainProofLedger::new(
                IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1).unwrap(),
            );
            let Err(ProofPlanningFailure::Exhausted(exhaustion)) =
                assess_domain_group(&region, &make_group(copies), &mut ledger)
            else {
                panic!("one-byte budget must expose exact grouped charge")
            };
            u64::try_from(exhaustion.required).unwrap()
        };
        let grouped = required(2);
        let separate = required(1).checked_mul(2).unwrap();
        assert!(grouped < separate);
        let mut exact = IndexDomainProofLedger::new(
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, grouped).unwrap(),
        );
        assert!(assess_domain_group(&region, &make_group(2), &mut exact).is_ok());
        let mut short = IndexDomainProofLedger::new(
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, grouped - 1).unwrap(),
        );
        assert!(matches!(
            assess_domain_group(&region, &make_group(2), &mut short),
            Err(ProofPlanningFailure::Exhausted(IndexDomainProofExhaustion {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit,
            })) if required == u128::from(grouped) && limit == grouped - 1
        ));
    }

    #[test]
    fn equivalent_authoring_orders_retain_directional_canonical_occurrences() {
        let build = |reverse: bool| {
            let mut program = SemanticProgramBuilder::try_standard().unwrap();
            let (one, two) = if reverse {
                let two = F32Constant::apply(&mut program, 2.0_f32.to_bits()).unwrap();
                let one = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
                (one, two)
            } else {
                let one = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
                let two = F32Constant::apply(&mut program, 2.0_f32.to_bits()).unwrap();
                (one, two)
            };
            program.output(OutputKey::new("one").unwrap(), one).unwrap();
            program.output(OutputKey::new("two").unwrap(), two).unwrap();
            program.build().unwrap()
        };
        let receipt = |program: &SemanticProgram, storage: usize| {
            let semantic = FrozenSemanticRegistry::standard().unwrap();
            let scalars = FrozenScalarRegistry::standard().unwrap();
            let laws =
                FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
                    .unwrap();
            let operation = program.operations().nth(storage).unwrap().id();
            let subject =
                IndexRefinementSubject::derive(program, operation, test_contract()).unwrap();
            let authority = IndexRealizationAuthority::admit(
                &semantic,
                &scalars,
                subject.operation().clone(),
                subject.signature().clone(),
                &[super::super::constant_f32_scalar_op()],
            )
            .unwrap();
            let resolution = laws.resolve(&subject).unwrap();
            let region = super::super::IndexRealizationLaw::constant_f32()
                .realize(&subject, &scalars)
                .unwrap();
            let IndexRefinementVerificationOutcome::Verified(receipt) =
                resolution.verify(&authority, &region).unwrap()
            else {
                panic!("a rank-zero constant retains no residual obligation")
            };
            receipt
        };

        let forward = build(false);
        let reversed = build(true);
        assert_eq!(
            forward.semantic_identity().graph(),
            reversed.semantic_identity().graph()
        );
        assert_ne!(
            forward.operations().next().unwrap().id(),
            reversed.operations().nth(1).unwrap().id(),
            "the same named operation is selected by graph-owned handles, not a shared ordinal"
        );

        // `one` is storage operation 0 in the forward graph and 1 in the
        // reversed graph; `two` moves in the opposite direction. Compare each
        // direction explicitly so a crossed mapping cannot be sorted away.
        let forward_one = receipt(&forward, 0);
        let forward_two = receipt(&forward, 1);
        let reversed_two = receipt(&reversed, 0);
        let reversed_one = receipt(&reversed, 1);
        assert_eq!(forward_one.occurrence(), reversed_one.occurrence());
        assert_eq!(forward_one.identity(), reversed_one.identity());
        assert_eq!(
            forward_one.executable_coverage_identity(),
            reversed_one.executable_coverage_identity()
        );
        assert_eq!(forward_two.occurrence(), reversed_two.occurrence());
        assert_eq!(forward_two.identity(), reversed_two.identity());
        assert_eq!(
            forward_two.executable_coverage_identity(),
            reversed_two.executable_coverage_identity()
        );
        assert_ne!(forward_one.occurrence(), forward_two.occurrence());
        assert_ne!(forward_one.identity(), forward_two.identity());
        assert_ne!(
            forward_one.executable_coverage_identity(),
            forward_two.executable_coverage_identity()
        );

        let other = IndexRefinementSubject::derive(
            &forward,
            forward.operations().nth(1).unwrap().id(),
            test_contract(),
        )
        .unwrap();
        assert_eq!(other.occurrence(), forward_two.occurrence());
        assert_ne!(other.occurrence(), forward_one.occurrence());

        let foreign = reversed.operations().next().unwrap().id();
        assert!(matches!(
            IndexRefinementSubject::derive(&forward, foreign, test_contract()),
            Err(IndexRefinementVerificationError::SemanticHandle(
                crate::semantic::HandleError::ForeignGraph {
                    entity: crate::semantic::EntityKind::Operation
                }
            ))
        ));
    }

    #[test]
    fn v2_subject_domain_separates_the_v1_storage_ordinal_collision() {
        let build = |reverse: bool| {
            let mut program = SemanticProgramBuilder::try_standard().unwrap();
            let first = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
            let second = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
            let (alpha, beta) = if reverse {
                (second, first)
            } else {
                (first, second)
            };
            program
                .output(OutputKey::new("alpha").unwrap(), alpha)
                .unwrap();
            program
                .output(OutputKey::new("beta").unwrap(), beta)
                .unwrap();
            program.build().unwrap()
        };
        let forward = build(false);
        let reversed = build(true);
        let forward_subject = IndexRefinementSubject::derive(
            &forward,
            forward.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let reversed_subject = IndexRefinementSubject::derive(
            &reversed,
            reversed.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        assert_eq!(
            forward.semantic_identity().graph(),
            reversed.semantic_identity().graph()
        );
        assert_ne!(
            forward_subject.occurrence(),
            reversed_subject.occurrence(),
            "fixed output names distinguish two otherwise identical occurrences canonically"
        );

        let storage_zero = SemanticOccurrence::new(0);
        let old_forward = encode_subject_identity_with(
            &forward_subject,
            LEGACY_SUBJECT_IDENTITY_TAG,
            storage_zero,
        );
        let old_reversed = encode_subject_identity_with(
            &reversed_subject,
            LEGACY_SUBJECT_IDENTITY_TAG,
            storage_zero,
        );
        assert_eq!(
            old_forward, old_reversed,
            "v1 gave storage occurrence zero one byte spelling for two canonical occurrences"
        );
        assert_ne!(forward_subject.identity, reversed_subject.identity);
        assert!(forward_subject.identity.starts_with(SUBJECT_IDENTITY_TAG));
        assert!(
            !forward_subject
                .identity
                .starts_with(LEGACY_SUBJECT_IDENTITY_TAG)
        );
    }

    #[test]
    fn wide_program_derives_all_occurrences_from_one_linear_cache() {
        const OPERATIONS: usize = 1_024;
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        for ordinal in 0..OPERATIONS {
            let value = F32Constant::apply(&mut builder, u32::try_from(ordinal).unwrap()).unwrap();
            builder
                .output(
                    OutputKey::new(format!("value-{ordinal:04}")).unwrap(),
                    value,
                )
                .unwrap();
        }
        let program = builder.build().unwrap();
        assert_eq!(program.canonical_operation_ordinal_count(), OPERATIONS);
        let subjects = program
            .operations()
            .map(|operation| {
                IndexRefinementSubject::derive(&program, operation.id(), test_contract()).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(subjects.len(), OPERATIONS);
        let mut occurrences = subjects
            .iter()
            .map(IndexRefinementSubject::occurrence)
            .collect::<Vec<_>>();
        occurrences.sort_unstable();
        occurrences.dedup();
        assert_eq!(occurrences.len(), OPERATIONS);
    }

    #[test]
    fn completion_receipts_cannot_be_cross_wired_between_real_occurrences() {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([LENGTH]))
            .unwrap();
        let first_value = F32Multiply::apply(&mut program, input, input).unwrap();
        let second_value = F32Multiply::apply(&mut program, input, input).unwrap();
        program
            .output(OutputKey::new("first").unwrap(), first_value)
            .unwrap();
        program
            .output(OutputKey::new("second").unwrap(), second_value)
            .unwrap();
        let program = program.build().unwrap();
        let first_subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let second_subject = IndexRefinementSubject::derive(
            &program,
            program.operations().nth(1).unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let semantic = FrozenSemanticRegistry::standard().unwrap();
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();
        let region = two_domain_residual_region();
        let scalar_authority = scalars.revalidate_region(&region).unwrap();
        let realization = VerifiedIndexRegionSequence::single(region);
        let pending = |resolution| PendingIndexRefinementReceipt {
            resolution,
            leading_scalar_authorities: Vec::new(),
            scalar_authority: scalar_authority.clone(),
            operand_bindings: Vec::new(),
            result_bindings: Vec::new(),
            realization: realization.clone(),
        };
        let first = pending(laws.resolve(&first_subject).unwrap());
        let second = pending(laws.resolve(&second_subject).unwrap());
        let mint = |pending: &PendingIndexRefinementReceipt| {
            mint_receipt(
                pending.subject(),
                &pending.resolution,
                &pending.realization,
                pending.scalar_authorities(),
                pending.operand_bindings.clone(),
                pending.result_bindings.clone(),
                Vec::new(),
            )
        };
        let first_receipt = mint(&first);
        let second_receipt = mint(&second);
        assert_eq!(
            second.verify_completion(&first_receipt),
            Err(IndexRefinementVerificationError::CompletionReceiptMismatch)
        );

        // The two occurrences agree on every other subject the executable
        // projection reads, so a coverage identity that failed to separate them
        // would be crossable between real, equally-shaped occurrences.
        assert_eq!(first_receipt.graph(), second_receipt.graph());
        assert_eq!(first_receipt.final_stage(), second_receipt.final_stage());
        assert_eq!(
            first_receipt.final_scalar_authority(),
            second_receipt.final_scalar_authority()
        );
        assert_eq!(
            first_receipt.operand_bindings(),
            second_receipt.operand_bindings()
        );
        assert_eq!(
            first_receipt.result_bindings(),
            second_receipt.result_bindings()
        );
        assert_ne!(first_receipt.occurrence(), second_receipt.occurrence());
        assert_ne!(
            first_receipt.executable_coverage_identity(),
            second_receipt.executable_coverage_identity()
        );
    }

    /// The contract-free family query answers off the registered law row.
    ///
    /// **Three rows and one agreement, and the agreement is the load-bearing
    /// half.** `tiler::rms-norm-f32@1` carries `StagedRootMeanSquareScaleF32`
    /// and answers true; `tiler::multiply-f32@1` carries a single-region law and
    /// answers false; `tiler::softmax-f32@1` is a registered *operation* the
    /// standard authority carries no law for and answers false rather than
    /// panicking. The agreement then shows the query is the same fact read from
    /// the same row rather than a second account of it: for a derived subject,
    /// [`ResolvedIndexRealization::realizes_region_sequence`] answers
    /// identically for both families.
    #[test]
    fn the_family_region_sequence_query_agrees_with_the_resolved_law() {
        use crate::semantic::{F32Multiply, F32RmsNorm};
        use crate::shape::Axis;

        let semantic = FrozenSemanticRegistry::standard().unwrap();
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();

        assert!(laws.family_realizes_region_sequence(&crate::semantic::rms_norm_f32_op()));
        assert!(!laws.family_realizes_region_sequence(&crate::semantic::multiply_f32_op()));
        assert!(
            !laws.family_realizes_region_sequence(&crate::semantic::softmax_f32_op()),
            "a registered operation the authority carries no law for realizes no sequence"
        );

        // One program holding both families, so the resolved answers come from
        // occurrences the same authority actually admits.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let shape = Shape::from_dims([4]);
        let value = builder
            .input::<F32>(InputKey::new("x").unwrap(), shape.clone())
            .unwrap();
        let weight = builder
            .input::<F32>(InputKey::new("w").unwrap(), shape)
            .unwrap();
        let normalized = F32RmsNorm::apply(
            &mut builder,
            value,
            weight,
            Axis::new(0),
            1.0e-6_f32.to_bits(),
        )
        .unwrap();
        let scaled = F32Multiply::apply(&mut builder, normalized, value).unwrap();
        builder
            .output(OutputKey::new("y").unwrap(), scaled)
            .unwrap();
        let program = builder.build().unwrap();

        for (position, expected) in [(0_usize, true), (1, false)] {
            let operation = program.operations().nth(position).unwrap();
            let key = operation.key().clone();
            let subject =
                IndexRefinementSubject::derive(&program, operation.id(), test_contract()).unwrap();
            let resolved = laws.resolve(&subject).unwrap();
            assert_eq!(resolved.realizes_region_sequence(), expected);
            assert_eq!(
                laws.family_realizes_region_sequence(&key),
                resolved.realizes_region_sequence(),
                "the contract-free query must answer what the resolved law answers for {key}"
            );
        }
    }

    /// A residual association reaches executable coverage only through proof.
    ///
    /// The compile-fail doctest on [`PendingIndexRefinementReceipt`] carries the
    /// structural half — a pending value exposes no coverage accessor. This
    /// carries the behavioural half: an undischarged residual leaves `complete`
    /// with no receipt to project, so no coverage identity exists to name.
    ///
    /// Only the `Unknown` refusal is reachable from a verified region here.
    /// `IndexRegionBuilder` runs its own exhaustive fallback under
    /// [`MAX_EXHAUSTIVE_PROOF_CELLS`], and an access it can walk it either
    /// discharges or refuses as `CoordinateOutOfBounds` at build time, so a
    /// small disprovable region never becomes a `VerifiedIndexRegion` at all.
    /// A `Disproved` completion needs a region inside the cell window between
    /// that bound and [`MAX_FINITE_DOMAIN_PROOF_CELLS`] whose per-point integer
    /// work still fits [`MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES`]; both refusal
    /// arms leave `complete` through the same `Err` return, so the coverage
    /// claim does not depend on exhibiting the second one.
    #[test]
    fn pending_and_refused_proofs_have_no_executable_coverage_spelling() {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([LENGTH]))
            .unwrap();
        let value = F32Multiply::apply(&mut program, input, input).unwrap();
        program
            .output(OutputKey::new("output").unwrap(), value)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            test_contract(),
        )
        .unwrap();
        let semantic = FrozenSemanticRegistry::standard().unwrap();
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();
        let resolution = laws.resolve(&subject).unwrap();
        let pending = |region: VerifiedIndexRegion| PendingIndexRefinementReceipt {
            resolution: resolution.clone(),
            leading_scalar_authorities: Vec::new(),
            scalar_authority: scalars.revalidate_region(&region).unwrap(),
            operand_bindings: Vec::new(),
            result_bindings: Vec::new(),
            realization: VerifiedIndexRegionSequence::single(region),
        };
        let budget = IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        )
        .unwrap();

        let unprovable = pending(residual_region(1, 5, 0));
        assert_eq!(unprovable.obligations().len(), 1);
        let refusal = ResolvedIndexRealization::complete(&unprovable, budget)
            .expect_err("a residual beyond the hard integer budget cannot be discharged");
        assert_eq!(refusal.kind(), IndexDomainProofRefusalKind::Unknown);
        assert_eq!(refusal.assessments().len(), 1);
        // The same association mints coverage only once its obligations are
        // discharged, so the refusal above is the difference between a spelling
        // and none — not a difference in the association itself.
        let discharged = mint_receipt(
            unprovable.subject(),
            &unprovable.resolution,
            &unprovable.realization,
            unprovable.scalar_authorities(),
            unprovable.operand_bindings.clone(),
            unprovable.result_bindings.clone(),
            proofs_for(unprovable.final_stage()),
        );
        assert_eq!(discharged.index_domain_proofs().len(), 1);
        let unproved = mint_receipt(
            unprovable.subject(),
            &unprovable.resolution,
            &unprovable.realization,
            unprovable.scalar_authorities(),
            unprovable.operand_bindings.clone(),
            unprovable.result_bindings.clone(),
            Vec::new(),
        );
        assert_ne!(
            discharged.executable_coverage_identity(),
            unproved.executable_coverage_identity()
        );
    }

    /// Seals one exact-finite proof per retained obligation of `region`.
    ///
    /// The completion algorithm's own budget is not the subject here; this
    /// supplies the proof records a discharged association would carry so the
    /// minted coverage can be compared against the same association's
    /// undischarged encoding.
    fn proofs_for(region: &VerifiedIndexRegion) -> Vec<IndexRefinementDomainProof> {
        let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
        region
            .unknown_index_domain_predicates()
            .map(|obligation| {
                let proof = IndexDomainProofEvidence::ExhaustiveFinite {
                    points: 2,
                    derivation: EXHAUSTIVE_DERIVATION.into(),
                };
                IndexRefinementDomainProof {
                    stage: 0,
                    obligation,
                    authority: authority.clone(),
                    identity: IndexRefinementDomainProofIdentity(
                        encode_proof_identity(region, obligation, &authority, &proof)
                            .into_boxed_slice(),
                    ),
                    proof,
                }
            })
            .collect()
    }

    #[test]
    fn wide_domain_environment_work_reaches_the_cell_hard_limit() {
        let mut extents = vec![1; 256];
        extents[0] = 65_535;
        let region = residual_region_with_extents(&extents, 0, 5, 1_i128.into(), 0_i128.into());
        assert!(matches!(
            assess(
                &region,
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            ),
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::Cells,
                required,
                limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
            }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_CELLS)
        ));
    }

    #[test]
    fn large_exact_counterexample_stays_bounded_and_disproved() {
        const POINTS: u64 = 257;
        let mut magnitude = vec![0; MAX_DOMAIN_EVIDENCE_BYTES + 1];
        magnitude[0] = 1;
        let large =
            IndexInteger::from_sign_magnitude(super::super::IndexIntegerSign::Positive, &magnitude)
                .unwrap();
        let negative_large =
            IndexInteger::from_sign_magnitude(super::super::IndexIntegerSign::Negative, &magnitude)
                .unwrap();
        assert!(large.magnitude_byte_len() > MAX_DOMAIN_EVIDENCE_BYTES);
        let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
            .expect("the fixture receives a fresh builder identity");
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(POINTS))
            .unwrap();
        let shape = Shape::from_dims([POINTS]);
        let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
        let input = builder
            .tensor(TensorRole::Input, value_type.clone(), shape.clone())
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type, shape)
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let mut equivalents = Vec::with_capacity(2_048);
        for index in 0_u64..2_048 {
            let equivalent = builder
                .modulo(
                    coordinate,
                    SourcedExtent::Static(Extent::new(POINTS + index + 1)),
                )
                .unwrap();
            equivalents.push(equivalent);
        }
        let mut cancellations = equivalents
            .as_chunks::<2>()
            .0
            .iter()
            .enumerate()
            .map(|(index, pair)| {
                let coefficients = if index == 0 {
                    (large.clone(), negative_large.clone())
                } else {
                    (1_i128.into(), (-1_i128).into())
                };
                let cancellation = builder
                    .linear_combination(
                        0_i128.into(),
                        &[(coefficients.0, pair[0]), (coefficients.1, pair[1])],
                    )
                    .unwrap();
                builder
                    .modulo(
                        cancellation,
                        SourcedExtent::Static(Extent::new(POINTS + 2_049 + index as u64)),
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();
        while cancellations.len() > 1 {
            cancellations = cancellations
                .chunks(2)
                .map(|pair| {
                    if pair.len() == 1 {
                        pair[0]
                    } else {
                        let sum = builder
                            .linear_combination(
                                0_i128.into(),
                                &[(1_i128.into(), pair[0]), (1_i128.into(), pair[1])],
                            )
                            .unwrap();
                        builder
                            .modulo(sum, SourcedExtent::Static(Extent::new(POINTS + 4_096)))
                            .unwrap()
                    }
                })
                .collect();
        }
        let second_zero = builder
            .modulo(
                cancellations[0],
                SourcedExtent::Static(Extent::new(POINTS + 4_097)),
            )
            .unwrap();
        let exact_large = builder
            .linear_combination(
                1_i128.into(),
                &[
                    (1_i128.into(), cancellations[0]),
                    ((-1_i128).into(), second_zero),
                    (1_i128.into(), coordinate),
                ],
            )
            .unwrap();
        let value = builder.read(input, &[dimension], &[exact_large]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        let region = builder.build().unwrap();
        let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
        assert_eq!(obligations.len(), 2);
        let obligation = obligations
            .iter()
            .copied()
            .find(|obligation| {
                matches!(
                    obligation.predicate(),
                    IndexDomainPredicate::LessThanExtent { .. }
                )
            })
            .expect("the exact upper-bound residual is retained");
        let claim = assess_finite_domains(
            &region,
            &[obligation],
            IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            )
            .unwrap(),
        )
        .pop()
        .unwrap();
        assert!(
            matches!(
                &claim,
                IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                    resource: super::super::ProofResource::IntegerBytes,
                    required,
                    limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
                }) if *required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
            ),
            "{claim:?}"
        );

        let required_bytes = |obligations: &[UnknownIndexDomainPredicate]| {
            let claims = assess_finite_domains(
                &region,
                obligations,
                IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1).unwrap(),
            );
            let IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit: 1,
            }) = claims[0]
            else {
                panic!("the one-byte perturbation must stop at group reservation")
            };
            u64::try_from(required).unwrap()
        };
        let grouped_bytes = required_bytes(&obligations);
        assert!(grouped_bytes > MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES);
    }

    #[test]
    fn nonlinear_integer_work_refuses_a_one_mebibyte_product_preflight() {
        let mebibyte = 1024_u128 * 1024;
        let product = checked_add(mebibyte, mebibyte).unwrap();
        let required = multiplication_cost(mebibyte, mebibyte, product).unwrap();
        assert!(required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES));
    }

    #[test]
    fn negative_floor_division_and_modulo_have_exact_charge_boundaries() {
        let dividend = BigInt::from(-17_i32);
        assert_eq!(
            dividend.div_floor(&BigInt::from(5_u64)),
            BigInt::from(-4_i32)
        );
        assert_eq!(
            dividend.mod_floor(&BigInt::from(5_u64)),
            BigInt::from(3_u32)
        );
        for result_width in [2_u128, 8] {
            let required = division_cost(2, result_width).unwrap();
            let budget = IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                u64::try_from(required).unwrap(),
            )
            .unwrap();
            let mut exact = IndexDomainProofLedger::new(budget);
            assert!(exact.reserve_evaluation(0, required).is_ok());
            let mut short = IndexDomainProofLedger::new(
                IndexDomainProofBudget::try_new(
                    MAX_FINITE_DOMAIN_PROOF_CELLS,
                    u64::try_from(required - 1).unwrap(),
                )
                .unwrap(),
            );
            assert!(matches!(
                short.reserve_evaluation(0, required),
                Err(ProofPlanningFailure::Exhausted(IndexDomainProofExhaustion {
                    resource: super::super::ProofResource::IntegerBytes,
                    required: actual,
                    limit,
                })) if actual == required && u128::from(limit) == required - 1
            ));
        }
    }

    #[test]
    fn integer_work_overflow_refuses_before_evaluation() {
        assert!(matches!(
            multiplication_cost(u128::MAX, u128::MAX, u128::MAX),
            Err(ProofPlanningFailure::Unsupported)
        ));
    }

    #[test]
    fn invalid_budgets_are_rejected_before_evaluation() {
        assert_eq!(
            IndexDomainProofBudget::try_new(0, MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES),
            Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: super::super::ProofResource::Cells,
                actual: 0,
                limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
            })
        );
        assert_eq!(
            IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS + 1,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            ),
            Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: super::super::ProofResource::Cells,
                actual: MAX_FINITE_DOMAIN_PROOF_CELLS + 1,
                limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
            })
        );
        assert_eq!(
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 0),
            Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: super::super::ProofResource::IntegerBytes,
                actual: 0,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            })
        );
        assert_eq!(
            IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES + 1,
            ),
            Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
                resource: super::super::ProofResource::IntegerBytes,
                actual: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES + 1,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            })
        );
    }

    #[test]
    fn residual_obligation_limit_refuses_before_pending_allocation() {
        assert_eq!(
            check_residual_obligation_count(MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS + 1),
            Err(
                IndexRefinementVerificationError::ResidualObligationsTooLarge {
                    actual: MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS + 1,
                    limit: MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS,
                }
            )
        );
        assert_eq!(
            check_residual_obligation_count(MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS),
            Ok(())
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
            assess(
                &region,
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            ),
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
        ));
    }

    #[test]
    fn a_static_domain_with_symbolic_tensor_extent_is_unsupported() {
        let symbol = ShapeSymbol::new(SymbolScope::new("proof/axis").unwrap(), "n").unwrap();
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
        let mut builder = IndexRegionBuilder::new_with_shape_environment(
            FrozenScalarRegistry::standard().unwrap(),
            Arc::new(environment.build().unwrap()),
        )
        .unwrap();
        let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
        let input = builder
            .sourced_tensor(
                TensorRole::Input,
                value_type.clone(),
                vec![SourcedExtent::Symbol(symbol)],
            )
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type, Shape::from_dims([8]))
            .unwrap();
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(8))
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        let region = builder.build().unwrap();
        assert_eq!(region.unknown_index_domain_predicates().len(), 1);
        assert!(matches!(
            assess(
                &region,
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            ),
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
        ));
    }

    #[test]
    fn operand_errors_name_the_expanded_semantic_boundary() {
        assert_eq!(
            IndexRefinementVerificationError::OperandArity {
                region_inputs: 1,
                expanded_inputs: 3,
            }
            .to_string(),
            "region declares 1 inputs for 3 expanded semantic input boundaries"
        );
        assert_eq!(
            IndexRefinementVerificationError::OperandInterface { position: 2 }.to_string(),
            "region input 2 does not match its expanded semantic input boundary"
        );
        assert_eq!(
            count_expanded_inputs(&[encoded_boundary(0)], 0),
            Err(IndexRefinementVerificationError::EmptyEncodedOperandComponents { input: 0 })
        );
        assert_eq!(
            IndexRefinementVerificationError::EmptyEncodedOperandComponents { input: 2 }
                .to_string(),
            "encoded semantic input 2 declares no component boundaries"
        );
        assert_eq!(
            IndexRefinementVerificationError::OperandBindingsTooLarge {
                actual: 17_408,
                limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
            }
            .to_string(),
            "expanded operand bindings 17408 exceed receipt limit 16384"
        );
    }

    #[test]
    fn expanded_input_count_is_bounded_before_component_shapes_are_materialized() {
        // Sixteen maximum-size encoded contracts exactly fill the verified
        // region boundary population. A seventeenth crosses the public region
        // limit while this pass is still counting component declarations; no
        // component shape has been derived or retained yet.
        let boundary = encoded_boundary(1_024);
        let maximal = vec![boundary.clone(); 16];
        assert_eq!(
            count_expanded_inputs(&maximal, MAX_BOUNDARY_TENSORS),
            Ok(MAX_BOUNDARY_TENSORS)
        );
        let oversized = vec![boundary; 17];
        assert_eq!(
            count_expanded_inputs(&oversized, MAX_BOUNDARY_TENSORS),
            Err(IndexRefinementVerificationError::OperandArity {
                region_inputs: MAX_BOUNDARY_TENSORS,
                expanded_inputs: 17 * 1_024,
            })
        );
    }

    #[test]
    fn operand_binding_population_is_bounded_before_collection() {
        // One maximum-size encoded semantic input may be aliased sixteen times
        // and exactly fill the receipt binding population. A seventeenth use
        // crosses the independent receipt limit even though the distinct
        // expanded input population remains only 1,024. This count-only pass
        // runs before the final binding Vec is allocated.
        let component_counts = [1_024];
        assert_eq!(
            count_operand_bindings(&[0; 16], &component_counts),
            Ok(MAX_INDEX_REFINEMENT_OPERAND_BINDINGS)
        );
        assert_eq!(
            count_operand_bindings(&[0; 17], &component_counts),
            Err(IndexRefinementVerificationError::OperandBindingsTooLarge {
                actual: 17 * 1_024,
                limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
            })
        );
        assert_eq!(
            count_operand_bindings(&[0, 0], &[usize::MAX]),
            Err(IndexRefinementVerificationError::OperandBindingsTooLarge {
                actual: usize::MAX,
                limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
            })
        );
    }
}
