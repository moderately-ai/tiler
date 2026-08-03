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
use crate::semantic::{OpKey, OperationAttributes, OperationEffect, ResolvedValueType};
use crate::shape::Shape;

use super::{
    CanonicalIndexRegionIdentity, FrozenScalarRegistry, IndexDomainPredicate,
    IndexDomainUnknownReason, IndexExprView, IndexExtentRef, IndexInteger, IndexIntegerSign,
    ProofResource, ScalarAuthorityEvidence, ScalarRegistryError, TensorRole,
    UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedIndexRegion, VerifiedScalarValueId, VerifiedTensorAccessId,
    VerifiedTensorId,
};

const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-receipt.v1\0";
const PROOF_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-domain-proof.v1\0";
const EXHAUSTIVE_DERIVATION: &[u8] = b"tiler.ir.exact-index-domain-enumeration.v1\0";
/// Independent proof budget used when completing an otherwise checked receipt.
pub const MAX_REFINEMENT_PROOF_CELLS: u64 = 16 * 1024 * 1024;

/// Occurrence-local identity of one semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrenceValueId(pub u32);

/// One ordered operand of a semantic occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceOperand {
    value: OccurrenceValueId,
    value_type: ResolvedValueType,
    shape: Shape,
}

impl OccurrenceOperand {
    /// Binds one ordered operand value, element type, and boundary shape.
    #[must_use]
    pub const fn new(
        value: OccurrenceValueId,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Self {
        Self {
            value,
            value_type,
            shape,
        }
    }

    /// Returns the semantic value this operand references.
    #[must_use]
    pub const fn value(&self) -> OccurrenceValueId {
        self.value
    }

    /// Returns the operand element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the operand boundary shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// One ordered result of a semantic occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceResult {
    value_type: ResolvedValueType,
    shape: Shape,
}

impl OccurrenceResult {
    /// Binds one ordered result element type and boundary shape.
    #[must_use]
    pub const fn new(value_type: ResolvedValueType, shape: Shape) -> Self {
        Self { value_type, shape }
    }

    /// Returns the result element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the result boundary shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Opaque collision-free identity of the exact semantic source being lowered.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticOccurrenceIdentity(Vec<u8>);

impl SemanticOccurrenceIdentity {
    /// Wraps collision-free semantic-source identity bytes.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the semantic-source identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
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

/// The exact semantic occurrence one index region is checked against.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticOccurrence {
    operation: OpKey,
    operands: Vec<OccurrenceOperand>,
    results: Vec<OccurrenceResult>,
    attributes: OperationAttributes,
    effect: OperationEffect,
    numerical_contract: NumericalContractIdentity,
    identity: SemanticOccurrenceIdentity,
}

impl SemanticOccurrence {
    /// Describes one semantic occurrence for checked index refinement.
    #[must_use]
    pub fn new(
        operation: OpKey,
        operands: Vec<OccurrenceOperand>,
        results: Vec<OccurrenceResult>,
        attributes: OperationAttributes,
        effect: OperationEffect,
        numerical_contract: NumericalContractIdentity,
        identity: SemanticOccurrenceIdentity,
    ) -> Self {
        Self {
            operation,
            operands,
            results,
            attributes,
            effect,
            numerical_contract,
            identity,
        }
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

    /// Returns ordered operands, including aliases.
    #[must_use]
    pub fn operands(&self) -> &[OccurrenceOperand] {
        &self.operands
    }

    /// Returns ordered results.
    #[must_use]
    pub fn results(&self) -> &[OccurrenceResult] {
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

    /// Returns the semantic-source identity.
    #[must_use]
    pub const fn identity(&self) -> &SemanticOccurrenceIdentity {
        &self.identity
    }

    fn distinct_operands(
        &self,
    ) -> Result<Vec<&OccurrenceOperand>, IndexRefinementVerificationError> {
        let mut distinct: Vec<&OccurrenceOperand> = Vec::new();
        for (position, operand) in self.operands.iter().enumerate() {
            if let Some(seen) = distinct
                .iter()
                .find(|candidate| candidate.value == operand.value)
            {
                if seen.value_type != operand.value_type || seen.shape != operand.shape {
                    return Err(
                        IndexRefinementVerificationError::AliasedOperandInconsistent {
                            operand: position,
                        },
                    );
                }
            } else {
                distinct.push(operand);
            }
        }
        Ok(distinct)
    }
}

/// One ordered operand bound to its verified region input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperandBinding {
    operand: usize,
    value: OccurrenceValueId,
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
    pub const fn value(&self) -> OccurrenceValueId {
        self.value
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
    occurrence: SemanticOccurrenceIdentity,
    region: CanonicalIndexRegionIdentity,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
    identity: IndexRefinementReceiptIdentity,
}

impl IndexRefinementReceipt {
    /// Returns the semantic occurrence this receipt binds.
    #[must_use]
    pub const fn occurrence(&self) -> &SemanticOccurrenceIdentity {
        &self.occurrence
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
    occurrence: SemanticOccurrence,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    region: VerifiedIndexRegion,
}

impl PendingIndexRefinementReceipt {
    /// Returns the checked semantic occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> &SemanticOccurrence {
        &self.occurrence
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
        self.occurrence == other.occurrence
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
    Verified(IndexRefinementReceipt),
    /// The association is checked, but residual obligations grant no permission.
    Pending(PendingIndexRefinementReceipt),
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
        occurrence: &SemanticOccurrence,
        region: &VerifiedIndexRegion,
        scalars: &FrozenScalarRegistry,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        if occurrence.effect != OperationEffect::Pure {
            return Err(IndexRefinementVerificationError::EffectNotIndexable {
                effect: occurrence.effect,
            });
        }
        let scalar_authority = scalars.revalidate_region(region).map_err(|source| {
            IndexRefinementVerificationError::ScalarAuthority(Arc::new(source))
        })?;
        let operand_bindings = bind_operands(occurrence, region)?;
        let result_bindings = bind_results(occurrence, region)?;
        if region.unknown_index_domain_predicates().len() != 0 {
            return Ok(IndexRefinementVerificationOutcome::Pending(
                PendingIndexRefinementReceipt {
                    occurrence: occurrence.clone(),
                    scalar_authority,
                    operand_bindings,
                    result_bindings,
                    region: region.clone(),
                },
            ));
        }
        Ok(IndexRefinementVerificationOutcome::Verified(mint_receipt(
            occurrence,
            region.canonical_identity(),
            scalar_authority,
            operand_bindings,
            result_bindings,
            Vec::new(),
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
            &pending.occurrence,
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
    /// Two aliased operands disagree on type or shape.
    AliasedOperandInconsistent {
        /// Position of the inconsistent aliased operand.
        operand: usize,
    },
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
            Self::AliasedOperandInconsistent { operand } => write!(
                formatter,
                "operand {operand} aliases another value but disagrees on type or shape"
            ),
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

fn bind_operands(
    occurrence: &SemanticOccurrence,
    region: &VerifiedIndexRegion,
) -> Result<Vec<OperandBinding>, IndexRefinementVerificationError> {
    let inputs = region
        .tensors()
        .filter(|tensor| tensor.role() == TensorRole::Input)
        .collect::<Vec<_>>();
    let distinct = occurrence.distinct_operands()?;
    if inputs.len() != distinct.len() {
        return Err(IndexRefinementVerificationError::OperandArity {
            region_inputs: inputs.len(),
            distinct_operands: distinct.len(),
        });
    }
    for (position, (operand, input)) in distinct.iter().zip(&inputs).enumerate() {
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
        .map(|(position, operand)| {
            let distinct_index = distinct
                .iter()
                .position(|candidate| candidate.value == operand.value)
                .ok_or(IndexRefinementVerificationError::OperandInterface { position })?;
            Ok(OperandBinding {
                operand: position,
                value: operand.value,
                input_tensor: inputs[distinct_index].id(),
            })
        })
        .collect()
}

fn bind_results(
    occurrence: &SemanticOccurrence,
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
    occurrence: &SemanticOccurrence,
    region: &CanonicalIndexRegionIdentity,
    scalar_authority: ScalarAuthorityEvidence,
    operand_bindings: Vec<OperandBinding>,
    result_bindings: Vec<ResultBinding>,
    index_domain_proofs: Vec<IndexRefinementDomainProof>,
) -> IndexRefinementReceipt {
    let identity =
        encode_receipt_identity(occurrence, region, &scalar_authority, &index_domain_proofs);
    IndexRefinementReceipt {
        occurrence: occurrence.identity.clone(),
        region: region.clone(),
        scalar_authority,
        operand_bindings,
        result_bindings,
        index_domain_proofs,
        identity: IndexRefinementReceiptIdentity(identity.into_boxed_slice()),
    }
}

fn encode_receipt_identity(
    occurrence: &SemanticOccurrence,
    region: &CanonicalIndexRegionIdentity,
    scalar_authority: &ScalarAuthorityEvidence,
    proofs: &[IndexRefinementDomainProof],
) -> Vec<u8> {
    let mut bytes = RECEIPT_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region.as_bytes());
    push_slice(&mut bytes, occurrence.identity.as_bytes());
    encode_op_key(&mut bytes, &occurrence.operation);
    let operand_interface = canonical_operand_interface(occurrence);
    push_len(&mut bytes, operand_interface.len());
    for (local, value_type, shape) in operand_interface {
        bytes.extend_from_slice(&local.to_be_bytes());
        push_slice(&mut bytes, value_type.canonical_encoding().as_bytes());
        encode_shape(&mut bytes, &shape);
    }
    push_len(&mut bytes, occurrence.results.len());
    for result in &occurrence.results {
        push_slice(
            &mut bytes,
            result.value_type.canonical_encoding().as_bytes(),
        );
        encode_shape(&mut bytes, &result.shape);
    }
    bytes.push(match occurrence.effect {
        OperationEffect::Pure => 1,
    });
    push_slice(
        &mut bytes,
        occurrence.attributes.canonical_encoding().as_bytes(),
    );
    push_slice(&mut bytes, occurrence.numerical_contract.as_bytes());
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

fn canonical_operand_interface(
    occurrence: &SemanticOccurrence,
) -> Vec<(u32, ResolvedValueType, Shape)> {
    let mut order = Vec::new();
    occurrence
        .operands
        .iter()
        .map(|operand| {
            let local = if let Some(index) = order.iter().position(|value| *value == operand.value)
            {
                u32::try_from(index).expect("operand count is host bounded")
            } else {
                let index = u32::try_from(order.len()).expect("operand count is host bounded");
                order.push(operand.value);
                index
            };
            (local, operand.value_type.clone(), operand.shape.clone())
        })
        .collect()
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
    use crate::index::{
        DomainRole, IndexRegionBuilder, ScalarRegistryBuilder, SourcedExtent, TensorRole,
    };
    use crate::semantic::{F32, FrozenSemanticRegistry, OperationEffect, multiply_f32_op};
    use crate::shape::{Extent, Shape};

    use super::{
        IndexRefinementVerificationError, IndexRefinementVerificationOutcome,
        IndexRefinementVerifier, NumericalContractIdentity, OccurrenceOperand, OccurrenceResult,
        OccurrenceValueId, SemanticOccurrence, SemanticOccurrenceIdentity,
    };

    const LENGTH: u64 = 65_535;

    fn residual_region() -> (
        crate::index::FrozenScalarRegistry,
        crate::index::VerifiedIndexRegion,
    ) {
        let scalars =
            ScalarRegistryBuilder::new(FrozenSemanticRegistry::standard().unwrap()).freeze();
        let mut builder = IndexRegionBuilder::new(scalars.clone()).unwrap();
        let first = builder
            .dimension(DomainRole::Parallel, Extent::new(LENGTH))
            .unwrap();
        let second = builder
            .dimension(DomainRole::Parallel, Extent::new(64))
            .unwrap();
        let shape = Shape::from_dims([LENGTH, 64]);
        let input = builder
            .tensor(TensorRole::Input, F32::resolved_type(), shape.clone())
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, F32::resolved_type(), shape)
            .unwrap();
        let first_coordinate = builder.dimension_expr(first).unwrap();
        let second_coordinate = builder.dimension_expr(second).unwrap();
        let mut conservative = first_coordinate;
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
        (scalars, region)
    }

    #[test]
    fn proof_budget_gap_mints_no_receipt() {
        let (scalars, region) = residual_region();
        let shape = Shape::from_dims([LENGTH, 64]);
        let occurrence = SemanticOccurrence::new(
            multiply_f32_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                F32::resolved_type(),
                shape.clone(),
            )],
            vec![OccurrenceResult::new(F32::resolved_type(), shape)],
            crate::semantic::OperationAttributes::empty(),
            OperationEffect::Pure,
            NumericalContractIdentity::from_key("test.strict-f32"),
            SemanticOccurrenceIdentity::from_bytes(b"proof-gap".to_vec()),
        );
        let pending = match IndexRefinementVerifier::verify(&occurrence, &region, &scalars).unwrap()
        {
            IndexRefinementVerificationOutcome::Pending(pending) => pending,
            IndexRefinementVerificationOutcome::Verified(_) => {
                panic!("a residual must not mint a receipt")
            }
        };

        let error = IndexRefinementVerifier::complete(pending).unwrap_err();
        assert!(matches!(
            error,
            IndexRefinementVerificationError::IndexDomainUnknown { reason, .. }
                if matches!(
                    reason.as_ref(),
                    crate::index::IndexDomainUnknownReason::ResourceLimit { .. }
                )
        ));
    }
}
