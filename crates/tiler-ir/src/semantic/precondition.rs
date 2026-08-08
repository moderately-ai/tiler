use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::{push_len, push_slice};
use crate::shape::SourcedShape;

use super::handles::{OperationId, OperationIndex, ValueId, ValueIndex};
use super::operation::OpKey;
use super::program::ProgramData;
use super::registry::F32;
use super::types::{ResolvedValueType, TypeIdentityError, TypeKey};

/// Maximum semantic preconditions declared by one operation family.
pub const MAX_OPERATION_SEMANTIC_PRECONDITIONS: usize = 1_024;
/// Maximum aggregate canonical bytes cached for one program's residual obligations.
pub const MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES: usize = 16 * 1024 * 1024;
// `v2`: a subject boundary is written through `SourcedShape::encode`, which
// tags each extent with its source kind. `v1` wrote eight untagged big-endian
// bytes per extent, which had nowhere to put a symbol, and a subject may now
// name one because an operation family may admit a symbolic operand. The tag is
// unconditional, so a wholly literal subject's bytes move even though its
// meaning does not — the same step `tiler.semantic-graph.v2` took to `v3`, for
// the same reason. `carry-a-sourced-shape-on-semantic-values` deferred this
// step to the ticket that admitted symbolic operands, and this is it.
const OBLIGATION_DOMAIN: &[u8] = b"tiler.semantic-precondition-obligation.v2\0";
const LENGTH_BYTES: usize = std::mem::size_of::<u64>();

/// Returns the governed predicate which rejects logical NaN values.
///
/// # Panics
///
/// Panics only if Tiler's hard-coded governed identity is invalid.
#[must_use]
pub fn no_nan_predicate() -> SemanticPredicateIdentity {
    SemanticPredicateIdentity::new("tiler", "no-nan", 1)
        .expect("the governed NoNaN predicate identity is valid")
}

/// Returns the governed predicate which requires one positive finite scalar.
///
/// # Panics
///
/// Panics only if Tiler's hard-coded governed identity is invalid.
#[must_use]
pub fn positive_finite_scalar_predicate() -> SemanticPredicateIdentity {
    SemanticPredicateIdentity::new("tiler", "positive-finite-scalar", 1)
        .expect("the governed PositiveFiniteScalar predicate identity is valid")
}

/// Returns the governed predicate which requires one positive *normal* scalar.
///
/// Strictly stronger than [`positive_finite_scalar_predicate`]: it admits
/// nothing that predicate rejects, and additionally rejects the subnormal
/// range. The two are declared together rather than merged, because "the value
/// is zero, negative, infinite, or NaN" and "the value is subnormal" are two
/// causes with two different fixes, and a diagnostic that shares one code
/// cannot tell a caller which one it hit.
///
/// # What the strengthening buys
///
/// A positive normal scale is what makes the strict-affine decode's numerical
/// obligation dischargeable on a target whose `f32` arithmetic flushes
/// subnormals. The derivation is exhaustive over the finite code domain rather
/// than sampled: the `i32` subtraction of two codes in `[0, 255]` is exact and
/// cannot overflow; converting a value of magnitude at most 255 to `f32` is
/// exact, so the converted operand is `+0.0` or has magnitude at least `1.0`
/// and is never subnormal; the product with the scale is `+0.0` when the codes
/// are equal, and otherwise has magnitude at least the scale, so it is
/// subnormal only if the scale is. A normal scale therefore makes the decode
/// bit-identical under a flushing and under a subnormal-preserving `f32`, and
/// the flush has nothing to act on.
///
/// **Measurement.** Finding 32 of `docs/research/apple-targets/numerical-behaviour.md`
/// ran that chain on the `apple9-f32-unified-msl4-macos26` row on 2026-07-31:
/// all 1,310,720 normal-scale cells returned bits identical to the exact
/// rational reference, `code == zero_point` returned `+0.0` in every diagonal
/// cell, and at a deliberately subnormal scale the flush acted on the operand —
/// exactly where the derivation places it. The boundary is that finding's: one
/// GPU family, one toolchain and flag row, `u8` codes, no packed extraction.
///
/// # Panics
///
/// Panics only if Tiler's hard-coded governed identity is invalid.
#[must_use]
pub fn positive_normal_scalar_predicate() -> SemanticPredicateIdentity {
    SemanticPredicateIdentity::new("tiler", "positive-normal-scalar", 1)
        .expect("the governed PositiveNormalScalar predicate identity is valid")
}

/// Stable namespaced semantic meaning of one value predicate.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticPredicateIdentity(TypeKey);

impl SemanticPredicateIdentity {
    /// Creates a validated, versioned semantic predicate identity.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for an invalid component or version.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, semantic_version).map(Self)
    }

    /// Returns the canonical namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }

    /// Returns the name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the nonzero semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version()
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        self.0.encode(output);
    }

    fn encoded_len(&self) -> usize {
        self.0.encoded_len()
    }
}

impl fmt::Display for SemanticPredicateIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable ordered invalid-input class produced by one semantic precondition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticInvalidInputCode(TypeKey);

impl SemanticInvalidInputCode {
    /// Creates a validated, versioned invalid-input code.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for an invalid component or version.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, semantic_version).map(Self)
    }

    /// Returns the canonical namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }

    /// Returns the name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the nonzero semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version()
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        self.0.encode(output);
    }

    fn encoded_len(&self) -> usize {
        self.0.encoded_len()
    }
}

impl fmt::Display for SemanticInvalidInputCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Zero-based position in an operation's ordered operand list.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationOperandIndex(u32);

impl OperationOperandIndex {
    /// Creates an operand position.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the fixed-width position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Exact logical projection inspected by a semantic precondition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticLogicalView {
    /// The complete logical value, excluding physical padding and unused storage.
    WholeValue,
}

impl SemanticLogicalView {
    pub(super) fn encode(self, output: &mut Vec<u8>) {
        match self {
            Self::WholeValue => output.push(1),
        }
    }
}

/// One operation-owned semantic precondition declaration.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticPreconditionDeclaration {
    predicate: SemanticPredicateIdentity,
    operand: OperationOperandIndex,
    view: SemanticLogicalView,
    invalid_input_code: SemanticInvalidInputCode,
}

impl SemanticPreconditionDeclaration {
    /// Declares a predicate over one exact operation operand and logical view.
    #[must_use]
    pub const fn new(
        predicate: SemanticPredicateIdentity,
        operand: OperationOperandIndex,
        view: SemanticLogicalView,
        invalid_input_code: SemanticInvalidInputCode,
    ) -> Self {
        Self {
            predicate,
            operand,
            view,
            invalid_input_code,
        }
    }

    /// Returns the stable predicate meaning.
    #[must_use]
    pub fn predicate(&self) -> &SemanticPredicateIdentity {
        &self.predicate
    }

    /// Returns the exact operand selector.
    #[must_use]
    pub const fn operand(&self) -> OperationOperandIndex {
        self.operand
    }

    /// Returns the exact logical projection.
    #[must_use]
    pub fn view(&self) -> SemanticLogicalView {
        self.view
    }

    /// Returns the stable invalid-input class.
    #[must_use]
    pub fn invalid_input_code(&self) -> &SemanticInvalidInputCode {
        &self.invalid_input_code
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        self.predicate.encode(output);
        output.extend_from_slice(&self.operand.get().to_be_bytes());
        self.view.encode(output);
        self.invalid_input_code.encode(output);
    }

    fn encoded_len(&self) -> usize {
        self.predicate
            .encoded_len()
            .saturating_add(std::mem::size_of::<u32>())
            .saturating_add(std::mem::size_of::<u8>())
            .saturating_add(self.invalid_input_code.encoded_len())
    }
}

/// Bounded declaration-order-preserving semantic preconditions.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticPreconditionDeclarations(Vec<SemanticPreconditionDeclaration>);

impl SemanticPreconditionDeclarations {
    /// Validates and retains semantic preconditions in declaration order.
    ///
    /// The declaration ordinal is derived from this order and cannot be
    /// supplied separately. Two declarations cannot repeat the same predicate,
    /// operand, and logical view merely by changing their error code.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticPreconditionDeclarationError`] for an excessive or
    /// duplicate declaration set.
    pub fn new(
        declarations: impl IntoIterator<Item = SemanticPreconditionDeclaration>,
    ) -> Result<Self, SemanticPreconditionDeclarationError> {
        let mut retained = Vec::new();
        for declaration in declarations
            .into_iter()
            .take(MAX_OPERATION_SEMANTIC_PRECONDITIONS.saturating_add(1))
        {
            if retained.len() == MAX_OPERATION_SEMANTIC_PRECONDITIONS {
                return Err(SemanticPreconditionDeclarationError::TooMany {
                    actual: MAX_OPERATION_SEMANTIC_PRECONDITIONS.saturating_add(1),
                    limit: MAX_OPERATION_SEMANTIC_PRECONDITIONS,
                });
            }
            if retained
                .iter()
                .any(|prior: &SemanticPreconditionDeclaration| {
                    prior.predicate == declaration.predicate
                        && prior.operand == declaration.operand
                        && prior.view == declaration.view
                })
            {
                return Err(SemanticPreconditionDeclarationError::Duplicate {
                    predicate: declaration.predicate,
                    operand: declaration.operand,
                    view: declaration.view,
                });
            }
            retained.push(declaration);
        }
        Ok(Self(retained))
    }

    /// Returns no declarations.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Returns declarations in semantic ordinal order.
    #[must_use]
    pub fn as_slice(&self) -> &[SemanticPreconditionDeclaration] {
        &self.0
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.0.len());
        for (ordinal, declaration) in self.0.iter().enumerate() {
            output.extend_from_slice(
                &u32::try_from(ordinal)
                    .expect("the bounded declaration count fits u32")
                    .to_be_bytes(),
            );
            declaration.encode(output);
        }
    }
}

/// Invalid operation-owned semantic precondition declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SemanticPreconditionDeclarationError {
    /// One operation declared more predicates than the governed bound.
    TooMany {
        /// First rejected count.
        actual: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// The same predicate, operand, and view were declared twice.
    Duplicate {
        /// Duplicated predicate.
        predicate: SemanticPredicateIdentity,
        /// Duplicated operand selector.
        operand: OperationOperandIndex,
        /// Duplicated logical view.
        view: SemanticLogicalView,
    },
}

impl fmt::Display for SemanticPreconditionDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooMany { actual, limit } => write!(
                formatter,
                "operation declares {actual} semantic preconditions, exceeding governed limit {limit}"
            ),
            Self::Duplicate {
                predicate,
                operand,
                view,
            } => write!(
                formatter,
                "duplicate semantic precondition {predicate} on operand {} view {view:?}",
                operand.get()
            ),
        }
    }
}

impl Error for SemanticPreconditionDeclarationError {}

/// Stable declaration position assigned by the host.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticPreconditionOrdinal(u32);

impl SemanticPreconditionOrdinal {
    pub(super) fn from_verified_position(value: usize) -> Self {
        Self(u32::try_from(value).expect("bounded declaration position fits u32"))
    }

    /// Returns the fixed-width declaration position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Static assessment retained for one valid semantic operation occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticPreconditionStatus {
    /// Authoritative compile-time evidence proved the predicate.
    Proven,
    /// The predicate remains an exact runtime validation obligation.
    Residual,
}

/// Host-owned authority which proved one semantic precondition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticPreconditionProofBasis {
    /// Exact scalar bits produced by Tiler's sealed standard `constant-f32` definition.
    StandardConstantF32BitsV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StaticValueEvidence {
    F32ScalarBits(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SemanticPreconditionData {
    pub(super) ordinal: SemanticPreconditionOrdinal,
    pub(super) subject: ValueIndex,
    pub(super) status: SemanticPreconditionStatus,
    pub(super) proof_basis: Option<SemanticPreconditionProofBasis>,
    pub(super) obligation_identity: Option<SemanticPreconditionObligationIdentity>,
}

pub(super) enum StaticAssessment {
    Proven(SemanticPreconditionProofBasis),
    Residual,
    Disproved,
}

pub(super) fn assess_static_precondition(
    declaration: &SemanticPreconditionDeclaration,
    subject_type: &ResolvedValueType,
    subject_shape: &SourcedShape,
    evidence: Option<StaticValueEvidence>,
) -> StaticAssessment {
    if declaration.view != SemanticLogicalView::WholeValue {
        return StaticAssessment::Residual;
    }
    let Some(StaticValueEvidence::F32ScalarBits(bits)) = evidence else {
        return StaticAssessment::Residual;
    };
    if subject_type != &F32::resolved_type() || subject_shape.rank() != 0 {
        return StaticAssessment::Residual;
    }
    let value = f32::from_bits(bits);
    if declaration.predicate == no_nan_predicate() {
        if value.is_nan() {
            StaticAssessment::Disproved
        } else {
            StaticAssessment::Proven(SemanticPreconditionProofBasis::StandardConstantF32BitsV1)
        }
    } else if declaration.predicate == positive_finite_scalar_predicate() {
        if value.is_finite() && value > 0.0 {
            StaticAssessment::Proven(SemanticPreconditionProofBasis::StandardConstantF32BitsV1)
        } else {
            StaticAssessment::Disproved
        }
    } else if declaration.predicate == positive_normal_scalar_predicate() {
        // `f32::is_normal` is already false for zero, subnormal, infinite, and
        // NaN values, so the sign test is the only thing it does not cover.
        if value.is_normal() && value > 0.0 {
            StaticAssessment::Proven(SemanticPreconditionProofBasis::StandardConstantF32BitsV1)
        } else {
            StaticAssessment::Disproved
        }
    } else {
        StaticAssessment::Residual
    }
}

/// Owned, typed rejection of a statically disproved semantic precondition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticPreconditionDisproof {
    operation: OpKey,
    predicate: SemanticPredicateIdentity,
    invalid_input_code: SemanticInvalidInputCode,
    declaration_ordinal: SemanticPreconditionOrdinal,
    subject: ValueId,
    view: SemanticLogicalView,
    resolved_type: Arc<ResolvedValueType>,
    shape: SourcedShape,
}

impl SemanticPreconditionDisproof {
    pub(super) fn new(
        operation: OpKey,
        declaration: &SemanticPreconditionDeclaration,
        declaration_ordinal: SemanticPreconditionOrdinal,
        subject: ValueId,
        resolved_type: Arc<ResolvedValueType>,
        shape: SourcedShape,
    ) -> Self {
        Self {
            operation,
            predicate: declaration.predicate.clone(),
            invalid_input_code: declaration.invalid_input_code.clone(),
            declaration_ordinal,
            subject,
            view: declaration.view,
            resolved_type,
            shape,
        }
    }

    /// Returns the rejected operation family.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the disproved predicate.
    #[must_use]
    pub fn predicate(&self) -> &SemanticPredicateIdentity {
        &self.predicate
    }

    /// Returns the stable invalid-input class.
    #[must_use]
    pub const fn invalid_input_code(&self) -> &SemanticInvalidInputCode {
        &self.invalid_input_code
    }

    /// Returns the operation-owned declaration ordinal.
    #[must_use]
    pub const fn declaration_ordinal(&self) -> SemanticPreconditionOrdinal {
        self.declaration_ordinal
    }

    /// Returns the exact draft-owned subject.
    #[must_use]
    pub const fn subject(&self) -> ValueId {
        self.subject
    }

    /// Returns the exact logical view.
    #[must_use]
    pub fn view(&self) -> SemanticLogicalView {
        self.view
    }

    /// Returns the complete subject type.
    #[must_use]
    pub fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns the exact logical subject shape.
    #[must_use]
    pub const fn shape(&self) -> &SourcedShape {
        &self.shape
    }
}

impl fmt::Display for SemanticPreconditionDisproof {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "operation {} has invalid input {}: predicate {} is disproved at declaration {}",
            self.operation,
            self.invalid_input_code,
            self.predicate,
            self.declaration_ordinal.get()
        )
    }
}

impl Error for SemanticPreconditionDisproof {}

pub(super) fn semantic_disproof_precedes(
    candidate: &SemanticPreconditionDisproof,
    prior: &SemanticPreconditionDisproof,
) -> bool {
    (
        candidate.invalid_input_code(),
        candidate.declaration_ordinal(),
    ) < (prior.invalid_input_code(), prior.declaration_ordinal())
}

/// Canonical identity of one residual semantic validation obligation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticPreconditionObligationIdentity(Vec<u8>);

impl SemanticPreconditionObligationIdentity {
    /// Returns collision-free canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Borrowed assessment of one operation-owned semantic precondition.
#[derive(Clone, Copy, Debug)]
pub struct SemanticPreconditionRef<'a> {
    pub(super) program: &'a ProgramData,
    pub(super) operation_index: OperationIndex,
    pub(super) data: &'a SemanticPreconditionData,
}

impl<'a> SemanticPreconditionRef<'a> {
    /// Returns the exact completed-program operation occurrence.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        OperationId {
            owner: self.program.owner,
            index: self.operation_index,
        }
    }

    /// Returns the host-derived declaration ordinal.
    #[must_use]
    pub const fn declaration_ordinal(&self) -> SemanticPreconditionOrdinal {
        self.data.ordinal
    }

    /// Returns the stable predicate meaning.
    #[must_use]
    pub fn predicate(&self) -> &SemanticPredicateIdentity {
        &self.declaration().predicate
    }

    /// Returns the exact completed-program subject.
    #[must_use]
    pub const fn subject(&self) -> ValueId {
        ValueId {
            owner: self.program.owner,
            index: self.data.subject,
        }
    }

    /// Returns the exact logical projection.
    #[must_use]
    pub fn view(&self) -> SemanticLogicalView {
        self.declaration().view
    }

    /// Returns the stable invalid-input class.
    #[must_use]
    pub fn invalid_input_code(&self) -> &SemanticInvalidInputCode {
        &self.declaration().invalid_input_code
    }

    /// Returns whether the occurrence was statically proved or remains residual.
    #[must_use]
    pub const fn status(&self) -> SemanticPreconditionStatus {
        self.data.status
    }

    /// Returns the exact host-owned authority for a proved predicate.
    #[must_use]
    pub const fn proof_basis(&self) -> Option<SemanticPreconditionProofBasis> {
        self.data.proof_basis
    }

    /// Returns canonical obligation identity only for a residual predicate.
    #[must_use]
    pub const fn obligation_identity(self) -> Option<&'a SemanticPreconditionObligationIdentity> {
        self.data.obligation_identity.as_ref()
    }

    fn declaration(&self) -> &SemanticPreconditionDeclaration {
        let operation = &self.program.operations[self.operation_index.as_usize()];
        &self
            .program
            .semantic_registry
            .operation_definition(&operation.key)
            .expect("verified operation retains its semantic definition")
            .semantic_preconditions()
            .as_slice()
            [usize::try_from(self.data.ordinal.get()).expect("u32 fits every supported host usize")]
    }
}

pub(super) fn obligation_identity_total_encoded_len(program: &ProgramData) -> usize {
    let graph_len = program.graph_identity_encoded_len;
    program
        .operations
        .iter()
        .flat_map(|operation| {
            operation
                .semantic_preconditions
                .iter()
                .filter(|data| data.status == SemanticPreconditionStatus::Residual)
                .map(move |data| {
                    obligation_identity_encoded_len(program, operation, data, graph_len)
                })
        })
        .fold(0_usize, usize::saturating_add)
}

pub(super) fn initialize_obligation_identities(
    program: &mut ProgramData,
    graph: &super::identity::SemanticGraphIdentity,
) {
    let mut identities = Vec::new();
    for (operation_position, operation) in program.operations.iter().enumerate() {
        for (precondition_position, data) in operation.semantic_preconditions.iter().enumerate() {
            if data.status == SemanticPreconditionStatus::Residual {
                identities.push((
                    operation_position,
                    precondition_position,
                    compute_obligation_identity(
                        program,
                        graph,
                        OperationIndex::from_verified_len(operation_position),
                        data,
                    ),
                ));
            }
        }
    }
    for (operation_position, precondition_position, identity) in identities {
        program.operations[operation_position].semantic_preconditions[precondition_position]
            .obligation_identity = Some(identity);
    }
}

fn compute_obligation_identity(
    program: &ProgramData,
    graph: &super::identity::SemanticGraphIdentity,
    operation_index: OperationIndex,
    data: &SemanticPreconditionData,
) -> SemanticPreconditionObligationIdentity {
    let operation = &program.operations[operation_index.as_usize()];
    let first_result = operation.results[0];
    let declaration = &program
        .semantic_registry
        .operation_definition(&operation.key)
        .expect("verified operation retains its semantic definition")
        .semantic_preconditions()
        .as_slice()
        [usize::try_from(data.ordinal.get()).expect("u32 fits every supported host usize")];
    let subject = &program.values[data.subject.as_usize()];
    encode_obligation_identity(ObligationIdentityParts {
        graph: graph.as_bytes(),
        reached_definitions: program.reached_definitions.as_bytes(),
        operation_coordinate: program.canonical_value_ids[first_result.as_usize()],
        declaration_ordinal: data.ordinal,
        declaration,
        subject_coordinate: program.canonical_value_ids[data.subject.as_usize()],
        resolved_type: &subject.resolved_type,
        shape: &subject.shape,
    })
}

#[derive(Clone, Copy)]
struct ObligationIdentityParts<'a> {
    graph: &'a [u8],
    reached_definitions: &'a [u8],
    operation_coordinate: u64,
    declaration_ordinal: SemanticPreconditionOrdinal,
    declaration: &'a SemanticPreconditionDeclaration,
    subject_coordinate: u64,
    resolved_type: &'a ResolvedValueType,
    shape: &'a SourcedShape,
}

fn encode_obligation_identity(
    parts: ObligationIdentityParts<'_>,
) -> SemanticPreconditionObligationIdentity {
    let encoded_len = obligation_identity_parts_encoded_len(
        parts.graph.len(),
        parts.reached_definitions.len(),
        parts.declaration,
        parts.resolved_type,
        parts.shape,
    );
    let mut bytes = Vec::with_capacity(encoded_len);
    bytes.extend_from_slice(OBLIGATION_DOMAIN);
    push_slice(&mut bytes, parts.graph);
    push_slice(&mut bytes, parts.reached_definitions);
    bytes.extend_from_slice(&parts.operation_coordinate.to_be_bytes());
    bytes.extend_from_slice(&parts.declaration_ordinal.get().to_be_bytes());
    parts.declaration.encode(&mut bytes);
    bytes.extend_from_slice(&parts.subject_coordinate.to_be_bytes());
    parts.resolved_type.encode(&mut bytes);
    parts.shape.encode(&mut bytes);
    debug_assert_eq!(bytes.len(), encoded_len);
    SemanticPreconditionObligationIdentity(bytes)
}

fn obligation_identity_encoded_len(
    program: &ProgramData,
    operation: &super::operation::OperationData,
    data: &SemanticPreconditionData,
    graph_len: usize,
) -> usize {
    let declaration = &program
        .semantic_registry
        .operation_definition(&operation.key)
        .expect("verified operation retains its semantic definition")
        .semantic_preconditions()
        .as_slice()
        [usize::try_from(data.ordinal.get()).expect("u32 fits every supported host usize")];
    let subject = &program.values[data.subject.as_usize()];
    obligation_identity_parts_encoded_len(
        graph_len,
        program.reached_definitions.as_bytes().len(),
        declaration,
        &subject.resolved_type,
        &subject.shape,
    )
}

fn obligation_identity_parts_encoded_len(
    graph_len: usize,
    reached_definitions_len: usize,
    declaration: &SemanticPreconditionDeclaration,
    resolved_type: &ResolvedValueType,
    shape: &SourcedShape,
) -> usize {
    OBLIGATION_DOMAIN
        .len()
        .saturating_add(LENGTH_BYTES)
        .saturating_add(graph_len)
        .saturating_add(LENGTH_BYTES)
        .saturating_add(reached_definitions_len)
        .saturating_add(std::mem::size_of::<u64>())
        .saturating_add(std::mem::size_of::<u32>())
        .saturating_add(declaration.encoded_len())
        .saturating_add(std::mem::size_of::<u64>())
        .saturating_add(resolved_type.encoded_len())
        .saturating_add(shape.encoded_len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{OperationArity, OperationSchema, OperationSchemaError};
    use crate::shape::{Shape, ShapeSymbol, SourcedExtent, SymbolScope};

    fn declaration(code: &str) -> SemanticPreconditionDeclaration {
        SemanticPreconditionDeclaration::new(
            no_nan_predicate(),
            OperationOperandIndex::new(0),
            SemanticLogicalView::WholeValue,
            SemanticInvalidInputCode::new("test", code, 1).unwrap(),
        )
    }

    #[test]
    fn declaration_collection_is_bounded_and_rejects_semantic_duplicates() {
        let duplicate =
            SemanticPreconditionDeclarations::new([declaration("first"), declaration("second")]);
        assert!(matches!(
            duplicate,
            Err(SemanticPreconditionDeclarationError::Duplicate {
                operand,
                view: SemanticLogicalView::WholeValue,
                ..
            }) if operand == OperationOperandIndex::new(0)
        ));

        let excessive = SemanticPreconditionDeclarations::new(
            (0..=MAX_OPERATION_SEMANTIC_PRECONDITIONS).map(|index| {
                SemanticPreconditionDeclaration::new(
                    SemanticPredicateIdentity::new("test", format!("predicate-{index}"), 1)
                        .unwrap(),
                    OperationOperandIndex::new(0),
                    SemanticLogicalView::WholeValue,
                    SemanticInvalidInputCode::new("test", format!("code-{index}"), 1).unwrap(),
                )
            }),
        );
        assert_eq!(
            excessive,
            Err(SemanticPreconditionDeclarationError::TooMany {
                actual: MAX_OPERATION_SEMANTIC_PRECONDITIONS + 1,
                limit: MAX_OPERATION_SEMANTIC_PRECONDITIONS,
            })
        );
    }

    #[test]
    fn operation_schema_rejects_a_selector_not_present_in_every_signature() {
        let schema =
            OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), []).unwrap();
        let definition = super::super::operation::OperationDefinition::new(
            super::super::operation::OpKey::new("test", "selector", 1).unwrap(),
            schema,
            super::super::registry::NormativeDefinitionRef::new("test selector").unwrap(),
            super::super::operation::OperationDefinitionFacts::new(
                super::super::types::CanonicalValue::boolean(true),
            ),
            super::super::operation::OperationConformance::new(
                super::super::types::CanonicalValue::boolean(true),
            ),
            super::super::operation::OperationEffect::Pure,
            Arc::new(IdentityInferencer),
        );
        let declarations =
            SemanticPreconditionDeclarations::new([SemanticPreconditionDeclaration::new(
                no_nan_predicate(),
                OperationOperandIndex::new(1),
                SemanticLogicalView::WholeValue,
                SemanticInvalidInputCode::new("test", "invalid", 1).unwrap(),
            )])
            .unwrap();
        assert!(matches!(
            definition.with_semantic_preconditions(declarations),
            Err(OperationSchemaError::SemanticPreconditionOperandOutOfRange {
                operand,
                minimum_arity: 1,
            }) if operand == OperationOperandIndex::new(1)
        ));
    }

    #[test]
    fn static_disproof_priority_is_stable_code_then_declaration_ordinal() {
        fn disproof(code: &str, ordinal: usize) -> SemanticPreconditionDisproof {
            let declaration = SemanticPreconditionDeclaration::new(
                no_nan_predicate(),
                OperationOperandIndex::new(0),
                SemanticLogicalView::WholeValue,
                SemanticInvalidInputCode::new("test", code, 1).unwrap(),
            );
            SemanticPreconditionDisproof::new(
                OpKey::new("test", "priority", 1).unwrap(),
                &declaration,
                SemanticPreconditionOrdinal::from_verified_position(ordinal),
                ValueId {
                    owner: super::super::handles::next_graph_id().unwrap(),
                    index: ValueIndex::from_verified_len(0),
                },
                Arc::new(F32::resolved_type()),
                SourcedShape::from(Shape::new([])),
            )
        }

        let lower_code_later_ordinal = disproof("a", 1);
        let higher_code_earlier_ordinal = disproof("z", 0);
        assert!(semantic_disproof_precedes(
            &lower_code_later_ordinal,
            &higher_code_earlier_ordinal,
        ));
        let same_code_later = disproof("same", 1);
        let same_code_earlier = disproof("same", 0);
        assert!(semantic_disproof_precedes(
            &same_code_earlier,
            &same_code_later,
        ));
    }

    #[test]
    fn obligation_encoder_is_independently_sensitive_to_every_v2_occurrence_field() {
        fn encoded(parts: ObligationIdentityParts<'_>) -> Vec<u8> {
            encode_obligation_identity(parts).as_bytes().to_vec()
        }

        let base_declaration = declaration("invalid");
        let changed_selector = SemanticPreconditionDeclaration::new(
            no_nan_predicate(),
            OperationOperandIndex::new(1),
            SemanticLogicalView::WholeValue,
            SemanticInvalidInputCode::new("test", "invalid", 1).unwrap(),
        );
        let base_type = F32::resolved_type();
        let changed_type = super::super::quantization::U4::resolved_type();
        let scalar = SourcedShape::from(Shape::new([]));
        let vector = SourcedShape::from(Shape::from_dims([1]));
        // The probe the two literal boundaries above cannot make. They differ in
        // rank, so an encoder that dropped the source tag would still separate
        // them; this pair differs only in whether one extent is a literal or a
        // symbol, which is exactly what `tiler.semantic-precondition-obligation.v2`
        // exists to keep apart.
        let symbolic = SourcedShape::sourced(vec![SourcedExtent::Symbol(
            ShapeSymbol::new(SymbolScope::new("tiler.test/0").unwrap(), "n").unwrap(),
        )])
        .expect("a one-symbol boundary is bounded");
        let base_parts = ObligationIdentityParts {
            graph: b"graph-a",
            reached_definitions: b"definitions-a",
            operation_coordinate: 7,
            declaration_ordinal: SemanticPreconditionOrdinal::from_verified_position(0),
            declaration: &base_declaration,
            subject_coordinate: 3,
            resolved_type: &base_type,
            shape: &scalar,
        };
        let base = encoded(base_parts);
        for changed in [
            encoded(ObligationIdentityParts {
                graph: b"graph-b",
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                reached_definitions: b"definitions-b",
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                operation_coordinate: 8,
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                declaration_ordinal: SemanticPreconditionOrdinal::from_verified_position(1),
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                declaration: &changed_selector,
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                subject_coordinate: 4,
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                resolved_type: &changed_type,
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                shape: &vector,
                ..base_parts
            }),
            encoded(ObligationIdentityParts {
                shape: &symbolic,
                ..base_parts
            }),
        ] {
            assert_ne!(base, changed);
        }
    }

    #[test]
    fn whole_value_view_has_an_explicit_identity_tag() {
        let mut encoded = Vec::new();
        SemanticLogicalView::WholeValue.encode(&mut encoded);
        assert_eq!(encoded, [1]);
    }

    struct IdentityInferencer;

    impl super::super::operation::OperationInferencer for IdentityInferencer {
        fn infer(
            &self,
            request: super::super::operation::OperationInferenceRequest<'_>,
            outputs: &mut super::super::operation::OperationInferenceOutputs<'_>,
        ) -> Result<(), super::super::operation::OperationInferenceError> {
            outputs.try_push(request.operands()[0].clone())
        }
    }
}
