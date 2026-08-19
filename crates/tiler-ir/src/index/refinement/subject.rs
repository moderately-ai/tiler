//! The semantic side of one refinement: what is being lowered.
//!
//! Everything here is derived from a verified semantic program and a typed
//! occurrence ordinal, before any region exists. A subject fixes the exact
//! boundaries, ordered signature, host-canonical attributes, effect, and
//! numerical contract an independently verified realization must reproduce, and
//! carries the identity those are folded into. Nothing in this file reads a
//! candidate region, which is why a subject can be derived once and compared
//! against any number of them.

use std::sync::Arc;

use crate::program::SemanticOccurrence;
use crate::schedule::{
    ArithmeticType, BF16_NUMERICAL_CONTRACT_KEY_DOMAIN, Bf16NumericalContractKey,
    F32NumericalContractKey, NumericalContractKeyError,
};
use crate::semantic::{
    OpKey, OperationAttributes, OperationEffect, OperationId, ResolvedValueType,
    SemanticCapabilityAuthority, SemanticGraphIdentity, SemanticProgram,
};
use crate::shape::Shape;

use super::error::IndexRefinementVerificationError;
use super::identity::encode_subject_identity;
use super::{MAX_INDEX_REFINEMENT_SIGNATURE_VALUES, MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES};

/// One semantic value boundary derived from a verified program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementBoundary {
    pub(super) value_type: ResolvedValueType,
    pub(super) shape: Shape,
    pub(super) sourced: crate::shape::SourcedShape,
}

impl IndexRefinementBoundary {
    /// Returns the boundary element type.
    #[must_use]
    pub const fn value_type(&self) -> &ResolvedValueType {
        &self.value_type
    }

    /// Returns the static boundary shape when the boundary was authored as a
    /// literal, and an empty shape when it names a symbol.
    ///
    /// Prefer [`Self::sourced_shape`] when the occurrence may be parametric.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns the authored sourced boundary, including symbolic extents.
    #[must_use]
    pub const fn sourced_shape(&self) -> &crate::shape::SourcedShape {
        &self.sourced
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
    pub(super) operands: Vec<ResolvedValueType>,
    pub(super) results: Vec<ResolvedValueType>,
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

/// Exact semantic subject derived from a verified program and typed ordinal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRefinementSubject {
    pub(super) graph: SemanticGraphIdentity,
    pub(super) occurrence: SemanticOccurrence,
    pub(super) operation: OpKey,
    pub(super) inputs: Vec<IndexRefinementBoundary>,
    pub(super) operands: Vec<usize>,
    pub(super) results: Vec<IndexRefinementBoundary>,
    pub(super) signature: IndexRefinementSignature,
    pub(super) attributes: OperationAttributes,
    pub(super) effect: OperationEffect,
    pub(super) numerical_contract: NumericalContractIdentity,
    pub(super) semantic_authority: SemanticCapabilityAuthority,
    pub(super) realization_law_row: Option<Box<[u8]>>,
    pub(super) identity: Box<[u8]>,
    environment: SubjectEnvironment,
}

/// The program environment a subject may carry, compared by identity only.
#[derive(Clone, Debug)]
struct SubjectEnvironment(Option<std::sync::Arc<crate::shape::ShapeEnv>>);

impl PartialEq for SubjectEnvironment {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(left), Some(right)) => left.identity() == right.identity(),
            _ => false,
        }
    }
}

impl Eq for SubjectEnvironment {}

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
            let (shape, sourced) = boundary_shapes(program, value)?;
            let index = values
                .iter()
                .position(|candidate| *candidate == value)
                .unwrap_or_else(|| {
                    values.push(value);
                    inputs.push(IndexRefinementBoundary {
                        value_type: reference.resolved_type().clone(),
                        shape: shape.clone(),
                        sourced: sourced.clone(),
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
            let (shape, sourced) = boundary_shapes(program, value)?;
            result_types.push(reference.resolved_type().clone());
            results.push(IndexRefinementBoundary {
                value_type: reference.resolved_type().clone(),
                shape,
                sourced,
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
            environment: SubjectEnvironment(
                program
                    .extent_sources()
                    .map(|sources| std::sync::Arc::clone(sources.environment_arc_for_subject())),
            ),
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

    /// Returns the program environment this subject was derived under, if any.
    #[must_use]
    pub fn shape_environment(&self) -> Option<&std::sync::Arc<crate::shape::ShapeEnv>> {
        self.environment.0.as_ref()
    }
}

/// Returns one covered semantic value's static projection and authored boundary.
///
/// A wholly literal boundary keeps the static `Shape` the previous encoder
/// wrote. A boundary that names a symbol keeps an empty static projection so
/// existing subject bytes for literal occurrences do not move, and carries the
/// sourced spelling beside it.
///
/// # Errors
///
/// Returns [`IndexRefinementVerificationError::SemanticHandle`] for a handle the
/// program does not own.
fn boundary_shapes(
    program: &crate::semantic::SemanticProgram,
    value: crate::semantic::ValueId,
) -> Result<(Shape, crate::shape::SourcedShape), IndexRefinementVerificationError> {
    let sourced = program
        .shape(value)
        .map_err(IndexRefinementVerificationError::SemanticHandle)?
        .clone();
    let shape = sourced
        .as_static()
        .cloned()
        .unwrap_or_else(|| Shape::from_dims([]));
    Ok((shape, sourced))
}
