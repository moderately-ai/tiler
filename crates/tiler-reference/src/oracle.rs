//! Generic slow reference oracle for verified canonical index regions.
//!
//! This path is deliberately independent of any graph-specific host
//! expression. It executes exactly what a [`VerifiedIndexRegion`] says: index
//! expressions evaluate with exact bounded integer arithmetic, scalar
//! applications resolve to registered capabilities selected by operation key
//! and exact resolved signature, and reductions run the declared exact
//! lexicographic left fold over their bound dimensions. Nothing is downcast,
//! and every unsupported or unauthorized case rejects with a typed error.

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::index::{
    AccessMode, CanonicalScalarDefinitionProjection, DomainRole, FrozenScalarRegistry,
    IndexExprView, MAX_INDEX_INTEGER_BYTES, ReducerBodyValueDefinitionView, ReductionTraversal,
    ScalarAttributeField, ScalarAttributes, ScalarAuthorityEvidence, ScalarOpKey,
    ScalarOperationDefinition, ScalarOperationKindRef, ScalarOperationRef, ScalarReductionRef,
    ScalarRegistryError, ScalarValueDefinitionView, TensorAccessRef, TensorRole,
    VerifiedDimensionId, VerifiedIndexExprId, VerifiedIndexHandleError, VerifiedIndexRegion,
    VerifiedReducerBodyOperationId, VerifiedReducerBodyValueId, VerifiedScalarOperationId,
    VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
};
use tiler_ir::semantic::{
    CanonicalField, CanonicalValue, CanonicalValueView, FrozenSemanticRegistry, ProviderIdentity,
    ResolvedValueType,
};
use tiler_ir::shape::Shape;

use crate::arithmetic::{ExactInteger, MagnitudeExceeded};
use crate::{
    EvaluationError, FrozenReferenceRegistry, MAX_REFERENCE_CAPABILITIES,
    MAX_REFERENCE_REGISTRY_IDENTITY_BYTES, MAX_REFERENCE_TENSOR_ELEMENTS,
    ReferenceCapabilityRevision, ReferenceElement, ReferenceOperationError, ReferenceRegistryError,
    ReferenceRegistryResource, ReferenceSignature, Tensor, TensorPayloadView, encode_bytes,
    encode_len, encode_provider_capability, encode_signature, encoded_bytes_len,
    reference_provider_identity_len, reference_signature_identity_len,
};

/// Maximum scalar, reducer-body, and index evaluations in one region evaluation.
const MAX_EVALUATION_STEPS: u64 = 16 * 1024 * 1024;

/// Maximum combined host recursion depth of one region evaluation.
///
/// Structural verification bounds one scalar or index dependency chain, but
/// nested reductions compose several such chains. The oracle governs the
/// combined depth so a pathological region rejects instead of exhausting the
/// host stack.
const MAX_EVALUATION_DEPTH: u32 = 2_048;

/// Maximum canonical magnitude bytes admitted for one evaluated index value.
///
/// [`MAX_INDEX_INTEGER_BYTES`] governs every stored coefficient and constant.
/// One normalized linear combination whose children are dimensions or moduli
/// exceeds its largest stored operand by at most one `u64` product term plus
/// the carry of its additions, so that governed growth is admitted. A deeper
/// composition — for example scaling an already maximal floor division — is
/// rejected instead of being saturated or wrapped.
const MAX_EVALUATED_INDEX_BYTES: usize = MAX_INDEX_INTEGER_BYTES + 16;

const SCALAR_REFERENCE_IDENTITY_TAG: &[u8] = b"tiler.scalar-reference-registry.v1\0";

/// Governed resource in one index-region reference evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexReferenceResource {
    /// Scalar and index-expression evaluations across one region evaluation.
    EvaluationSteps,
    /// Combined host recursion depth of one region evaluation.
    EvaluationDepth,
    /// Canonical magnitude bytes of one evaluated index integer.
    IndexIntegerBytes,
    /// Aggregate logical elements retained by one region's outputs.
    OutputElements,
}

/// Region feature outside this bounded oracle profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedRegionFeature {
    /// A domain dimension exposed no static extent.
    SymbolicDimensionExtent,
    /// A tensor boundary exposed no static shape.
    SymbolicTensorShape,
    /// A reduction declared a traversal this oracle does not implement.
    ReductionTraversal,
    /// A boundary or scalar value used a compound representation.
    ///
    /// Per-point element access into a compound value needs a role-wise
    /// element contract that this profile does not define.
    CompoundValueRepresentation,
    /// The region used a scalar operation kind added after this oracle.
    ScalarOperationKind,
    /// The region used an index-expression form added after this oracle.
    IndexExpressionForm,
    /// The region used a scalar value definition added after this oracle.
    ScalarValueForm,
    /// The region used a reducer-body value definition added after this oracle.
    ReducerBodyValueForm,
}

impl fmt::Display for UnsupportedRegionFeature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SymbolicDimensionExtent => "symbolic domain dimension extent",
            Self::SymbolicTensorShape => "symbolic tensor boundary shape",
            Self::ReductionTraversal => "unimplemented reduction traversal",
            Self::CompoundValueRepresentation => "compound reference value representation",
            Self::ScalarOperationKind => "unimplemented scalar operation kind",
            Self::IndexExpressionForm => "unimplemented index-expression form",
            Self::ScalarValueForm => "unimplemented scalar value definition",
            Self::ReducerBodyValueForm => "unimplemented reducer-body value definition",
        })
    }
}

/// Borrowed inputs to one exact scalar reference callback.
#[derive(Clone, Copy, Debug)]
pub struct ScalarReferenceRequest<'a> {
    operands: &'a [&'a Tensor],
    attributes: &'a ScalarAttributes,
}

impl<'a> ScalarReferenceRequest<'a> {
    /// Returns ordered rank-zero operand values.
    #[must_use]
    pub const fn operands(self) -> &'a [&'a Tensor] {
        self.operands
    }

    /// Returns canonical attributes with registered schema defaults resolved.
    #[must_use]
    pub const fn attributes(self) -> &'a ScalarAttributes {
        self.attributes
    }
}

/// Host-owned bounded output writer for one scalar reference callback.
///
/// A failed write poisons the writer, so ignoring the returned error cannot
/// make a partial or over-arity result appear successful.
#[derive(Debug)]
pub struct ScalarReferenceOutputs {
    expected: usize,
    values: Vec<Tensor>,
    failure: Option<ReferenceOperationError>,
}

impl ScalarReferenceOutputs {
    fn new(expected: usize) -> Self {
        Self {
            expected,
            values: Vec::with_capacity(expected),
            failure: None,
        }
    }

    /// Writes one ordered rank-zero result value.
    ///
    /// # Errors
    ///
    /// Returns a sticky typed failure once the callback exceeds its declared
    /// result arity. Subsequent writes return the original failure.
    pub fn push(&mut self, value: Tensor) -> Result<(), ReferenceOperationError> {
        if let Some(error) = self.failure.clone() {
            return Err(error);
        }
        let actual = self.values.len().saturating_add(1);
        if actual > self.expected {
            let error = ReferenceOperationError::ResultCount {
                expected: self.expected,
                actual,
            };
            self.failure = Some(error.clone());
            return Err(error);
        }
        self.values.push(value);
        Ok(())
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

/// One executable reference implementation for an exact scalar signature.
///
/// Implementations are trusted native callbacks with the same contract as
/// [`crate::ReferenceOperation`]: deterministic, non-panicking functions of the
/// request. Returned failures and host-owned result validation stay
/// recoverable and retain provider attribution.
pub trait ScalarReferenceOperation: Send + Sync + 'static {
    /// Evaluates one scalar application at one iteration point.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when operands or attributes violate this
    /// capability's contract.
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError>;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ScalarCapabilityKey {
    operation: ScalarOpKey,
    signature: ReferenceSignature,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScalarCapabilityAuthority {
    definitions: CanonicalScalarDefinitionProjection,
    provider: ProviderIdentity,
}

#[derive(Clone)]
struct RegisteredScalarCapability {
    provider: ProviderIdentity,
    revision: ReferenceCapabilityRevision,
    authority: ScalarCapabilityAuthority,
    implementation: Arc<dyn ScalarReferenceOperation>,
}

/// Failure to construct a scalar reference-capability registry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ScalarReferenceRegistryError {
    /// No scalar reference capability was registered.
    EmptyRegistry,
    /// Two registrations claimed one exact operation/signature pair.
    DuplicateCapability {
        /// Colliding scalar operation.
        operation: Box<ScalarOpKey>,
        /// Colliding resolved signature.
        signature: Box<ReferenceSignature>,
    },
    /// The selected scalar authority does not define the registered operation.
    MissingScalarDefinition {
        /// Undefined scalar operation.
        operation: Box<ScalarOpKey>,
    },
    /// The scalar authority rejected the registered operation's projection.
    ScalarAuthority {
        /// Operation being registered.
        operation: Box<ScalarOpKey>,
        /// Typed scalar-registry cause.
        source: Arc<ScalarRegistryError>,
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

impl fmt::Display for ScalarReferenceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegistry => {
                formatter.write_str("scalar reference capability registry is empty")
            }
            Self::DuplicateCapability { operation, .. } => write!(
                formatter,
                "duplicate scalar reference capability for {operation:?}"
            ),
            Self::MissingScalarDefinition { operation } => {
                write!(formatter, "scalar authority does not define {operation:?}")
            }
            Self::ScalarAuthority { operation, source } => write!(
                formatter,
                "scalar authority for {operation:?} failed: {source}"
            ),
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "scalar reference registry resource {resource:?} has size {actual}, exceeding {limit}"
            ),
        }
    }
}

impl Error for ScalarReferenceRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ScalarAuthority { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Mutable single-use constructor for a frozen scalar reference registry.
///
/// Registration takes the admitting provider directly, mirroring
/// [`tiler_ir::index::ScalarRegistryBuilder`] rather than the semantic
/// reference registry's provider-transaction surface.
pub struct ScalarReferenceRegistryBuilder {
    scalar_registry: FrozenScalarRegistry,
    capabilities: BTreeMap<ScalarCapabilityKey, RegisteredScalarCapability>,
    canonical_bytes: usize,
}

impl fmt::Debug for ScalarReferenceRegistryBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScalarReferenceRegistryBuilder")
            .field("capability_count", &self.capabilities.len())
            .finish_non_exhaustive()
    }
}

impl ScalarReferenceRegistryBuilder {
    /// Creates an empty builder bound to one exact frozen scalar authority.
    #[must_use]
    pub fn new(scalar_registry: FrozenScalarRegistry) -> Self {
        let canonical_bytes = SCALAR_REFERENCE_IDENTITY_TAG
            .len()
            .saturating_add(encoded_bytes_len(
                scalar_registry.snapshot_identity().as_bytes().len(),
            ))
            .saturating_add(size_of::<u64>());
        Self {
            scalar_registry,
            capabilities: BTreeMap::new(),
            canonical_bytes,
        }
    }

    /// Registers one exact scalar operation/signature capability.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a duplicate capability, an operation the
    /// selected scalar authority does not define, or an exceeded resource.
    pub fn register(
        &mut self,
        provider: ProviderIdentity,
        operation: ScalarOpKey,
        signature: ReferenceSignature,
        revision: ReferenceCapabilityRevision,
        implementation: Arc<dyn ScalarReferenceOperation>,
    ) -> Result<(), ScalarReferenceRegistryError> {
        let key = ScalarCapabilityKey {
            operation,
            signature,
        };
        if self.capabilities.contains_key(&key) {
            return Err(ScalarReferenceRegistryError::DuplicateCapability {
                operation: Box::new(key.operation),
                signature: Box::new(key.signature),
            });
        }
        let actual = self.capabilities.len().saturating_add(1);
        if actual > MAX_REFERENCE_CAPABILITIES {
            return Err(ScalarReferenceRegistryError::ResourceExceeded {
                resource: ReferenceRegistryResource::Capabilities,
                limit: MAX_REFERENCE_CAPABILITIES,
                actual,
            });
        }
        let authority = project_capability_authority(&self.scalar_registry, &key.operation)?;
        let added = scalar_capability_identity_len(&key, &authority, &provider);
        let bytes = self.canonical_bytes.saturating_add(added);
        if bytes > MAX_REFERENCE_REGISTRY_IDENTITY_BYTES {
            return Err(ScalarReferenceRegistryError::ResourceExceeded {
                resource: ReferenceRegistryResource::CanonicalIdentityBytes,
                limit: MAX_REFERENCE_REGISTRY_IDENTITY_BYTES,
                actual: bytes,
            });
        }
        self.capabilities.insert(
            key,
            RegisteredScalarCapability {
                provider,
                revision,
                authority,
                implementation,
            },
        );
        self.canonical_bytes = bytes;
        Ok(())
    }

    /// Freezes canonical immutable scalar reference capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarReferenceRegistryError::EmptyRegistry`] when empty.
    pub fn freeze(self) -> Result<FrozenScalarReferenceRegistry, ScalarReferenceRegistryError> {
        if self.capabilities.is_empty() {
            return Err(ScalarReferenceRegistryError::EmptyRegistry);
        }
        let identity = compute_scalar_reference_identity(
            &self.scalar_registry,
            &self.capabilities,
            self.canonical_bytes,
        );
        Ok(FrozenScalarReferenceRegistry(Arc::new(
            FrozenScalarReferenceRegistryData {
                scalar_registry: self.scalar_registry,
                capabilities: self.capabilities,
                identity,
            },
        )))
    }
}

struct FrozenScalarReferenceRegistryData {
    scalar_registry: FrozenScalarRegistry,
    capabilities: BTreeMap<ScalarCapabilityKey, RegisteredScalarCapability>,
    identity: CanonicalScalarReferenceRegistryIdentity,
}

/// Immutable exact scalar reference-capability registry.
#[derive(Clone)]
pub struct FrozenScalarReferenceRegistry(Arc<FrozenScalarReferenceRegistryData>);

impl fmt::Debug for FrozenScalarReferenceRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrozenScalarReferenceRegistry")
            .field("capability_count", &self.0.capabilities.len())
            .finish()
    }
}

impl FrozenScalarReferenceRegistry {
    /// Returns deterministic complete scalar reference-registry provenance.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalScalarReferenceRegistryIdentity {
        &self.0.identity
    }

    /// Returns the exact frozen scalar authority this registry was built against.
    #[must_use]
    pub fn scalar_registry(&self) -> &FrozenScalarRegistry {
        &self.0.scalar_registry
    }
}

/// Collision-free canonical provenance for a frozen scalar reference registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScalarReferenceRegistryIdentity(Vec<u8>);

impl CanonicalScalarReferenceRegistryIdentity {
    /// Returns canonical provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

fn project_capability_authority(
    registry: &FrozenScalarRegistry,
    operation: &ScalarOpKey,
) -> Result<ScalarCapabilityAuthority, ScalarReferenceRegistryError> {
    let definitions = registry.project_reached([operation]).map_err(|source| {
        ScalarReferenceRegistryError::ScalarAuthority {
            operation: Box::new(operation.clone()),
            source: Arc::new(source),
        }
    })?;
    let provider = registry.provider(operation).cloned().ok_or_else(|| {
        ScalarReferenceRegistryError::MissingScalarDefinition {
            operation: Box::new(operation.clone()),
        }
    })?;
    Ok(ScalarCapabilityAuthority {
        definitions,
        provider,
    })
}

/// Encodes the reached scalar definition and its admitting scalar provider.
fn encode_scalar_authority(output: &mut Vec<u8>, authority: &ScalarCapabilityAuthority) {
    encode_bytes(output, authority.definitions.as_bytes());
    encode_bytes(output, authority.provider.namespace().as_bytes());
    encode_bytes(output, authority.provider.name().as_bytes());
    output.extend_from_slice(&authority.provider.revision().to_be_bytes());
}

fn scalar_authority_identity_len(authority: &ScalarCapabilityAuthority) -> usize {
    encoded_bytes_len(authority.definitions.as_bytes().len())
        .saturating_add(encoded_bytes_len(authority.provider.namespace().len()))
        .saturating_add(encoded_bytes_len(authority.provider.name().len()))
        .saturating_add(size_of::<u32>())
}

fn scalar_capability_identity_len(
    key: &ScalarCapabilityKey,
    authority: &ScalarCapabilityAuthority,
    provider: &ProviderIdentity,
) -> usize {
    encoded_bytes_len(key.operation.namespace().len())
        .saturating_add(encoded_bytes_len(key.operation.name().len()))
        .saturating_add(size_of::<u32>())
        .saturating_add(reference_signature_identity_len(&key.signature))
        .saturating_add(scalar_authority_identity_len(authority))
        .saturating_add(reference_provider_identity_len(provider))
}

fn compute_scalar_reference_identity(
    scalar_registry: &FrozenScalarRegistry,
    capabilities: &BTreeMap<ScalarCapabilityKey, RegisteredScalarCapability>,
    exact_len: usize,
) -> CanonicalScalarReferenceRegistryIdentity {
    let mut bytes = Vec::with_capacity(exact_len);
    bytes.extend_from_slice(SCALAR_REFERENCE_IDENTITY_TAG);
    encode_bytes(&mut bytes, scalar_registry.snapshot_identity().as_bytes());
    encode_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        encode_bytes(&mut bytes, key.operation.namespace().as_bytes());
        encode_bytes(&mut bytes, key.operation.name().as_bytes());
        bytes.extend_from_slice(&key.operation.semantic_version().to_be_bytes());
        encode_signature(&mut bytes, &key.signature);
        encode_scalar_authority(&mut bytes, &capability.authority);
        encode_provider_capability(&mut bytes, &capability.provider, capability.revision);
    }
    debug_assert_eq!(bytes.len(), exact_len);
    CanonicalScalarReferenceRegistryIdentity(bytes)
}

/// The exact frozen authority one verified region is evaluated under.
#[derive(Clone, Copy, Debug)]
pub struct IndexRegionAuthority<'a> {
    scalar: &'a FrozenScalarRegistry,
    semantic: &'a FrozenSemanticRegistry,
}

impl<'a> IndexRegionAuthority<'a> {
    /// Names the scalar and semantic authority governing one region.
    #[must_use]
    pub const fn new(
        scalar: &'a FrozenScalarRegistry,
        semantic: &'a FrozenSemanticRegistry,
    ) -> Self {
        Self { scalar, semantic }
    }

    /// Returns the scalar authority.
    #[must_use]
    pub const fn scalar(self) -> &'a FrozenScalarRegistry {
        self.scalar
    }

    /// Returns the semantic type authority.
    #[must_use]
    pub const fn semantic(self) -> &'a FrozenSemanticRegistry {
        self.semantic
    }
}

/// One boundary-checked entry in the ordered region input interface.
#[derive(Clone, Copy, Debug)]
pub struct IndexRegionInput<'a> {
    tensor: VerifiedTensorId,
    value: &'a Tensor,
}

impl<'a> IndexRegionInput<'a> {
    /// Binds one exact input tensor boundary.
    #[must_use]
    pub const fn new(tensor: VerifiedTensorId, value: &'a Tensor) -> Self {
        Self { tensor, value }
    }

    /// Returns the bound boundary identity.
    #[must_use]
    pub const fn tensor(self) -> VerifiedTensorId {
        self.tensor
    }

    /// Returns the bound reference tensor.
    #[must_use]
    pub const fn value(self) -> &'a Tensor {
        self.value
    }
}

/// Ordered outputs and scalar authority evidence from one region evaluation.
#[derive(Clone, Debug)]
pub struct IndexRegionEvaluation {
    outputs: Vec<Tensor>,
    authority: ScalarAuthorityEvidence,
}

impl IndexRegionEvaluation {
    /// Returns ordered output-root tensors.
    #[must_use]
    pub fn outputs(&self) -> &[Tensor] {
        &self.outputs
    }

    /// Returns the scalar authority receipt bound to this exact region identity.
    #[must_use]
    pub const fn authority(&self) -> &ScalarAuthorityEvidence {
        &self.authority
    }

    /// Consumes this evaluation and returns its ordered output tensors.
    #[must_use]
    pub fn into_outputs(self) -> Vec<Tensor> {
        self.outputs
    }
}

/// Complete attribution of one resolved scalar reference capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarCapabilityAttribution {
    operation: ScalarOpKey,
    provider: ProviderIdentity,
    revision: ReferenceCapabilityRevision,
}

impl ScalarCapabilityAttribution {
    /// Returns the scalar operation family.
    #[must_use]
    pub const fn operation(&self) -> &ScalarOpKey {
        &self.operation
    }

    /// Returns the admitting reference provider.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the exact output-affecting implementation revision.
    #[must_use]
    pub const fn revision(&self) -> ReferenceCapabilityRevision {
        self.revision
    }
}

impl fmt::Display for ScalarCapabilityAttribution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} implemented by {} revision {}",
            self.operation,
            self.provider,
            self.revision.get()
        )
    }
}

/// A typed index-region reference-evaluation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRegionEvaluationError {
    /// The selected scalar authority rejected region revalidation.
    ScalarAuthority(Arc<ScalarRegistryError>),
    /// The supplied semantic registry is not the region's type authority.
    SemanticAuthorityMismatch,
    /// A verified handle did not resolve against its own region.
    Handle(VerifiedIndexHandleError),
    /// Forming an exact resolved signature failed.
    ReferenceRegistry(Arc<ReferenceRegistryError>),
    /// The caller supplied the wrong number of ordered input bindings.
    InputCount {
        /// Declared input-boundary count.
        expected: usize,
        /// Supplied binding count.
        actual: usize,
    },
    /// A binding named a different boundary than the ordered interface.
    InputBoundary {
        /// Position in the ordered input interface.
        input_index: usize,
    },
    /// An input shape disagreed with its verified boundary declaration.
    InputShape {
        /// Position in the ordered input interface.
        input_index: usize,
        /// Declared boundary shape.
        expected: Box<Shape>,
        /// Supplied tensor shape.
        actual: Box<Shape>,
    },
    /// An input resolved type disagreed with its verified boundary declaration.
    InputType {
        /// Position in the ordered input interface.
        input_index: usize,
    },
    /// A reference value validator rejected a bound or produced value.
    Value(Arc<EvaluationError>),
    /// No registered capability implements an exact scalar signature.
    MissingScalarCapability {
        /// Scalar operation lacking an oracle.
        operation: Box<ScalarOpKey>,
        /// Exact operand/result signature lacking an oracle.
        signature: Box<ReferenceSignature>,
    },
    /// A capability was registered against different reached scalar authority.
    ScalarCapabilityAuthorityMismatch {
        /// Complete capability attribution.
        capability: Arc<ScalarCapabilityAttribution>,
    },
    /// A resolved capability rejected execution.
    ScalarOperation {
        /// Complete capability attribution.
        capability: Arc<ScalarCapabilityAttribution>,
        /// Typed implementation failure.
        source: ReferenceOperationError,
    },
    /// A capability produced a result with the wrong rank or resolved type.
    ScalarResult {
        /// Complete capability attribution.
        capability: Arc<ScalarCapabilityAttribution>,
        /// Ordered result index.
        result_index: usize,
    },
    /// An independently evaluated coordinate left its declared tensor bounds.
    CoordinateOutOfBounds {
        /// Access whose coordinate left bounds.
        access: VerifiedTensorAccessId,
    },
    /// An independently evaluated write covered one element more than once.
    DuplicateWrite {
        /// Access that wrote one element twice.
        access: VerifiedTensorAccessId,
    },
    /// An independently evaluated write left one element uninitialized.
    IncompleteWrite {
        /// Access that left an element unwritten.
        access: VerifiedTensorAccessId,
    },
    /// A governed evaluation resource exceeded its limit.
    ResourceExceeded {
        /// Bounded resource.
        resource: IndexReferenceResource,
        /// Active limit.
        limit: u64,
        /// First rejected size.
        actual: u64,
    },
    /// The region uses a feature outside this bounded oracle profile.
    Unsupported {
        /// Rejected region feature.
        feature: UnsupportedRegionFeature,
    },
    /// An internally inconsistent verified region reached the oracle.
    MalformedRegion,
}

impl fmt::Display for IndexRegionEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScalarAuthority(source) => {
                write!(formatter, "region scalar authority failed: {source}")
            }
            Self::SemanticAuthorityMismatch => formatter
                .write_str("the supplied semantic registry is not the region's type authority"),
            Self::Handle(source) => write!(formatter, "verified region handle failed: {source}"),
            Self::ReferenceRegistry(source) => {
                write!(formatter, "reference registry failure: {source}")
            }
            Self::InputCount { expected, actual } => {
                write!(formatter, "expected {expected} inputs, received {actual}")
            }
            Self::InputBoundary { input_index } => {
                write!(formatter, "input {input_index} names another boundary")
            }
            Self::InputShape {
                input_index,
                expected,
                actual,
            } => write!(
                formatter,
                "input {input_index} has shape {actual}, expected {expected}"
            ),
            Self::InputType { input_index } => {
                write!(formatter, "input {input_index} has the wrong resolved type")
            }
            Self::Value(source) => write!(formatter, "reference value failure: {source}"),
            _ => self.fmt_evaluation_error(formatter),
        }
    }
}

impl IndexRegionEvaluationError {
    fn fmt_evaluation_error(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingScalarCapability { operation, .. } => write!(
                formatter,
                "no scalar reference capability for {operation:?} and exact resolved signature"
            ),
            Self::ScalarCapabilityAuthorityMismatch { capability } => write!(
                formatter,
                "{capability} does not implement the region's reached scalar authority"
            ),
            Self::ScalarOperation { capability, source } => {
                write!(formatter, "{capability} failed: {source}")
            }
            Self::ScalarResult {
                capability,
                result_index,
            } => write!(
                formatter,
                "{capability} produced invalid result {result_index}"
            ),
            Self::CoordinateOutOfBounds { .. } => {
                formatter.write_str("an evaluated coordinate left its declared tensor bounds")
            }
            Self::DuplicateWrite { .. } => {
                formatter.write_str("an evaluated write covered one element more than once")
            }
            Self::IncompleteWrite { .. } => {
                formatter.write_str("an evaluated write left one element uninitialized")
            }
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "index reference resource {resource:?} has size {actual}, exceeding {limit}"
            ),
            Self::Unsupported { feature } => {
                write!(formatter, "unsupported region feature: {feature}")
            }
            Self::MalformedRegion => {
                formatter.write_str("verified index region is internally malformed")
            }
            _ => unreachable!("only evaluation failures use this formatter"),
        }
    }
}

impl Error for IndexRegionEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ScalarAuthority(source) => Some(source.as_ref()),
            Self::Handle(source) => Some(source),
            Self::ReferenceRegistry(source) => Some(source.as_ref()),
            Self::Value(source) => Some(source.as_ref()),
            Self::ScalarOperation { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn resource(
    resource: IndexReferenceResource,
    limit: u64,
    actual: u64,
) -> IndexRegionEvaluationError {
    IndexRegionEvaluationError::ResourceExceeded {
        resource,
        limit,
        actual,
    }
}

fn unsupported(feature: UnsupportedRegionFeature) -> IndexRegionEvaluationError {
    IndexRegionEvaluationError::Unsupported { feature }
}

/// Host evaluator for verified canonical index regions.
#[derive(Clone, Debug)]
pub struct IndexRegionEvaluator {
    values: FrozenReferenceRegistry,
    scalars: FrozenScalarReferenceRegistry,
}

impl IndexRegionEvaluator {
    /// Creates an evaluator over exact value-representation and scalar snapshots.
    #[must_use]
    pub const fn new(
        values: FrozenReferenceRegistry,
        scalars: FrozenScalarReferenceRegistry,
    ) -> Self {
        Self { values, scalars }
    }

    /// Returns the value-representation capability snapshot.
    #[must_use]
    pub const fn value_registry(&self) -> &FrozenReferenceRegistry {
        &self.values
    }

    /// Returns the scalar capability snapshot.
    #[must_use]
    pub const fn scalar_registry(&self) -> &FrozenScalarReferenceRegistry {
        &self.scalars
    }

    /// Evaluates every ordered output root of one verified region.
    ///
    /// The region's scalar authority revalidates first, so a region this
    /// authority cannot admit never reaches an executable capability. Every
    /// coordinate, write coverage claim, and produced value is then checked
    /// independently of the structural verifier's own proofs.
    ///
    /// # Errors
    ///
    /// Returns an [`IndexRegionEvaluationError`] for missing authority, a
    /// missing capability, an unsupported region feature, an out-of-bounds or
    /// incomplete write, an exceeded governed resource, or a rejected value.
    pub fn evaluate(
        &self,
        region: &VerifiedIndexRegion,
        authority: IndexRegionAuthority<'_>,
        inputs: &[IndexRegionInput<'_>],
    ) -> Result<IndexRegionEvaluation, IndexRegionEvaluationError> {
        let evidence = authority
            .scalar()
            .revalidate_region(region)
            .map_err(|source| IndexRegionEvaluationError::ScalarAuthority(Arc::new(source)))?;
        if evidence.semantic_snapshot() != authority.semantic().snapshot_identity() {
            return Err(IndexRegionEvaluationError::SemanticAuthorityMismatch);
        }
        let mut evaluation = RegionEvaluation::new(region, self, authority, inputs)?;
        let outputs = evaluation.evaluate_outputs()?;
        Ok(IndexRegionEvaluation {
            outputs,
            authority: evidence,
        })
    }
}

struct Application {
    capability: Arc<ScalarCapabilityAttribution>,
    implementation: Arc<dyn ScalarReferenceOperation>,
    attributes: ScalarAttributes,
}

#[derive(Default)]
struct Frame {
    environment: BTreeMap<VerifiedDimensionId, u64>,
    values: HashMap<VerifiedScalarValueId, Tensor>,
    expressions: HashMap<VerifiedIndexExprId, ExactInteger>,
}

struct BodyContext<'a> {
    state: &'a [Tensor],
    contributors: &'a [Tensor],
    values: HashMap<VerifiedReducerBodyValueId, Tensor>,
}

struct OutputPlan<'a> {
    access: TensorAccessRef<'a>,
    value: VerifiedScalarValueId,
    value_type: ResolvedValueType,
    shape: &'a Shape,
    elements: Vec<Option<ReferenceElement>>,
}

struct RegionEvaluation<'a> {
    region: &'a VerifiedIndexRegion,
    evaluator: &'a IndexRegionEvaluator,
    authority: IndexRegionAuthority<'a>,
    inputs: HashMap<VerifiedTensorId, &'a Tensor>,
    applications: HashMap<VerifiedScalarOperationId, Application>,
    body_applications: HashMap<VerifiedReducerBodyOperationId, Application>,
    steps: u64,
    depth: u32,
}

impl<'a> RegionEvaluation<'a> {
    fn new(
        region: &'a VerifiedIndexRegion,
        evaluator: &'a IndexRegionEvaluator,
        authority: IndexRegionAuthority<'a>,
        inputs: &[IndexRegionInput<'a>],
    ) -> Result<Self, IndexRegionEvaluationError> {
        let mut evaluation = Self {
            region,
            evaluator,
            authority,
            inputs: HashMap::new(),
            applications: HashMap::new(),
            body_applications: HashMap::new(),
            steps: 0,
            depth: 0,
        };
        evaluation.bind_inputs(inputs)?;
        evaluation.resolve_applications()?;
        Ok(evaluation)
    }

    fn bind_inputs(
        &mut self,
        inputs: &[IndexRegionInput<'a>],
    ) -> Result<(), IndexRegionEvaluationError> {
        let declarations: Vec<_> = self
            .region
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .collect();
        if declarations.len() != inputs.len() {
            return Err(IndexRegionEvaluationError::InputCount {
                expected: declarations.len(),
                actual: inputs.len(),
            });
        }
        for (input_index, (declaration, binding)) in declarations.iter().zip(inputs).enumerate() {
            if declaration.id() != binding.tensor() {
                return Err(IndexRegionEvaluationError::InputBoundary { input_index });
            }
            let shape = declaration
                .static_shape()
                .ok_or_else(|| unsupported(UnsupportedRegionFeature::SymbolicTensorShape))?;
            if binding.value().shape() != shape {
                return Err(IndexRegionEvaluationError::InputShape {
                    input_index,
                    expected: Box::new(shape.clone()),
                    actual: Box::new(binding.value().shape().clone()),
                });
            }
            if binding.value().resolved_type() != declaration.value_type() {
                return Err(IndexRegionEvaluationError::InputType { input_index });
            }
            self.evaluator
                .values
                .validate_value(binding.value(), self.authority.semantic())
                .map_err(|source| IndexRegionEvaluationError::Value(Arc::new(source)))?;
            self.inputs.insert(declaration.id(), binding.value());
        }
        Ok(())
    }

    fn resolve_applications(&mut self) -> Result<(), IndexRegionEvaluationError> {
        for operation in self.region.scalar_operations() {
            match operation.kind() {
                ScalarOperationKindRef::Apply { key, attributes } => {
                    let operands = self.value_types(operation.operands())?;
                    let results = self.value_types(operation.results())?;
                    let application = self.resolve(key, attributes, operands, results)?;
                    self.applications.insert(operation.id(), application);
                }
                ScalarOperationKindRef::Reduce(reduction) => {
                    if !matches!(
                        reduction.traversal(),
                        ReductionTraversal::ExactLexicographicLeftFold
                    ) {
                        return Err(unsupported(UnsupportedRegionFeature::ReductionTraversal));
                    }
                    for application in reduction.body().operations() {
                        let operands = self.body_value_types(application.operands())?;
                        let results = self.body_value_types(application.results())?;
                        let resolved = self.resolve(
                            application.key(),
                            application.attributes(),
                            operands,
                            results,
                        )?;
                        self.body_applications.insert(application.id(), resolved);
                    }
                }
                _ => return Err(unsupported(UnsupportedRegionFeature::ScalarOperationKind)),
            }
        }
        Ok(())
    }

    fn value_types(
        &self,
        values: impl Iterator<Item = VerifiedScalarValueId>,
    ) -> Result<Vec<ResolvedValueType>, IndexRegionEvaluationError> {
        values
            .map(|id| {
                self.region
                    .scalar_value(id)
                    .map(|value| value.value_type().clone())
                    .map_err(IndexRegionEvaluationError::Handle)
            })
            .collect()
    }

    fn body_value_types(
        &self,
        values: impl Iterator<Item = VerifiedReducerBodyValueId>,
    ) -> Result<Vec<ResolvedValueType>, IndexRegionEvaluationError> {
        values
            .map(|id| {
                self.region
                    .reducer_body_value(id)
                    .map(|value| value.value_type().clone())
                    .map_err(IndexRegionEvaluationError::Handle)
            })
            .collect()
    }

    fn resolve(
        &self,
        key: &ScalarOpKey,
        attributes: &ScalarAttributes,
        operands: Vec<ResolvedValueType>,
        results: Vec<ResolvedValueType>,
    ) -> Result<Application, IndexRegionEvaluationError> {
        let signature = ReferenceSignature::new(operands, results)
            .map_err(|source| IndexRegionEvaluationError::ReferenceRegistry(Arc::new(source)))?;
        let lookup = ScalarCapabilityKey {
            operation: key.clone(),
            signature,
        };
        let registered = self.evaluator.scalars.0.capabilities.get(&lookup).ok_or(
            IndexRegionEvaluationError::MissingScalarCapability {
                operation: Box::new(lookup.operation.clone()),
                signature: Box::new(lookup.signature.clone()),
            },
        )?;
        let capability = Arc::new(ScalarCapabilityAttribution {
            operation: key.clone(),
            provider: registered.provider.clone(),
            revision: registered.revision,
        });
        let actual = project_capability_authority(self.authority.scalar(), key).map_err(|_| {
            IndexRegionEvaluationError::ScalarCapabilityAuthorityMismatch {
                capability: Arc::clone(&capability),
            }
        })?;
        if actual != registered.authority {
            return Err(
                IndexRegionEvaluationError::ScalarCapabilityAuthorityMismatch { capability },
            );
        }
        let definition = self
            .authority
            .scalar()
            .definition(key)
            .ok_or(IndexRegionEvaluationError::MalformedRegion)?;
        Ok(Application {
            capability,
            implementation: Arc::clone(&registered.implementation),
            attributes: resolved_attributes(definition, attributes)
                .ok_or(IndexRegionEvaluationError::MalformedRegion)?,
        })
    }

    fn step(&mut self) -> Result<(), IndexRegionEvaluationError> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > MAX_EVALUATION_STEPS {
            return Err(resource(
                IndexReferenceResource::EvaluationSteps,
                MAX_EVALUATION_STEPS,
                self.steps,
            ));
        }
        Ok(())
    }

    fn enter(&mut self) -> Result<(), IndexRegionEvaluationError> {
        self.depth = self.depth.saturating_add(1);
        if self.depth > MAX_EVALUATION_DEPTH {
            return Err(resource(
                IndexReferenceResource::EvaluationDepth,
                u64::from(MAX_EVALUATION_DEPTH),
                u64::from(self.depth),
            ));
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

fn resolved_attributes(
    definition: &ScalarOperationDefinition,
    stored: &ScalarAttributes,
) -> Option<ScalarAttributes> {
    let CanonicalValueView::Record(fields) = stored.value().view() else {
        return None;
    };
    let missing = definition
        .attributes()
        .fields()
        .iter()
        .filter(|field| !fields.iter().any(|stored| stored.id() == field.id()))
        .filter_map(|field| {
            ScalarAttributeField::default(field)
                .map(|value| CanonicalField::new(field.id(), value.clone()))
        });
    let resolved: Vec<_> = fields.iter().cloned().chain(missing).collect();
    CanonicalValue::record(resolved)
        .ok()
        .and_then(|value| ScalarAttributes::new(value).ok())
}

fn advance_point(point: &mut [u64], extents: &[u64]) -> bool {
    for axis in (0..point.len()).rev() {
        point[axis] += 1;
        if point[axis] < extents[axis] {
            return true;
        }
        point[axis] = 0;
    }
    false
}

fn dense_element(value: &Tensor) -> Result<ReferenceElement, IndexRegionEvaluationError> {
    match value.payload() {
        TensorPayloadView::Dense([element]) if value.shape().rank() == 0 => Ok(element.clone()),
        TensorPayloadView::Dense(_) => Err(IndexRegionEvaluationError::MalformedRegion),
        TensorPayloadView::Compound(_) => Err(unsupported(
            UnsupportedRegionFeature::CompoundValueRepresentation,
        )),
    }
}

fn magnitude_error(error: MagnitudeExceeded) -> IndexRegionEvaluationError {
    resource(
        IndexReferenceResource::IndexIntegerBytes,
        MAX_EVALUATED_INDEX_BYTES as u64,
        u64::try_from(error.required_bytes).unwrap_or(u64::MAX),
    )
}

fn admit_index(value: ExactInteger) -> Result<ExactInteger, IndexRegionEvaluationError> {
    let bytes = value.magnitude_bytes();
    if bytes > MAX_EVALUATED_INDEX_BYTES {
        return Err(magnitude_error(MagnitudeExceeded {
            required_bytes: bytes,
        }));
    }
    Ok(value)
}

fn finish_output(plan: OutputPlan<'_>) -> Result<Tensor, IndexRegionEvaluationError> {
    let access = plan.access.id();
    let elements = plan
        .elements
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(IndexRegionEvaluationError::IncompleteWrite { access })?;
    Tensor::dense(plan.value_type, plan.shape.clone(), elements)
        .map_err(|source| IndexRegionEvaluationError::Value(Arc::new(source)))
}

impl<'a> RegionEvaluation<'a> {
    fn evaluate_outputs(&mut self) -> Result<Vec<Tensor>, IndexRegionEvaluationError> {
        let parallel = self.domain(
            self.region
                .dimensions()
                .filter(|dimension| dimension.role() == DomainRole::Parallel)
                .map(tiler_ir::index::DomainDimensionRef::id),
        )?;
        let mut plans = self.output_plans()?;
        let extents: Vec<u64> = parallel.iter().map(|(_, extent)| *extent).collect();
        if !extents.contains(&0) {
            let mut point = vec![0_u64; extents.len()];
            loop {
                self.evaluate_point(&parallel, &point, &mut plans)?;
                if !advance_point(&mut point, &extents) {
                    break;
                }
            }
        }
        plans.into_iter().map(finish_output).collect()
    }

    fn domain(
        &self,
        dimensions: impl Iterator<Item = VerifiedDimensionId>,
    ) -> Result<Vec<(VerifiedDimensionId, u64)>, IndexRegionEvaluationError> {
        dimensions
            .map(|id| {
                let dimension = self
                    .region
                    .dimension(id)
                    .map_err(IndexRegionEvaluationError::Handle)?;
                let extent = dimension.static_extent().ok_or_else(|| {
                    unsupported(UnsupportedRegionFeature::SymbolicDimensionExtent)
                })?;
                Ok((id, extent.get()))
            })
            .collect()
    }

    fn output_plans(&self) -> Result<Vec<OutputPlan<'a>>, IndexRegionEvaluationError> {
        let region = self.region;
        let mut retained = 0_usize;
        let mut plans = Vec::with_capacity(region.outputs().len());
        for output in region.outputs() {
            let access = region
                .access(output.access())
                .map_err(IndexRegionEvaluationError::Handle)?;
            if access.mode() != AccessMode::Write {
                return Err(IndexRegionEvaluationError::MalformedRegion);
            }
            let tensor = region
                .tensor(access.tensor())
                .map_err(IndexRegionEvaluationError::Handle)?;
            let shape = tensor
                .static_shape()
                .ok_or_else(|| unsupported(UnsupportedRegionFeature::SymbolicTensorShape))?;
            let count = shape.element_count().ok_or_else(|| {
                resource(
                    IndexReferenceResource::OutputElements,
                    MAX_REFERENCE_TENSOR_ELEMENTS as u64,
                    u64::MAX,
                )
            })?;
            retained = retained.saturating_add(count);
            if retained > MAX_REFERENCE_TENSOR_ELEMENTS {
                return Err(resource(
                    IndexReferenceResource::OutputElements,
                    MAX_REFERENCE_TENSOR_ELEMENTS as u64,
                    u64::try_from(retained).unwrap_or(u64::MAX),
                ));
            }
            plans.push(OutputPlan {
                access,
                value: output.value(),
                value_type: tensor.value_type().clone(),
                shape,
                elements: vec![None; count],
            });
        }
        Ok(plans)
    }

    fn evaluate_point(
        &mut self,
        parallel: &[(VerifiedDimensionId, u64)],
        point: &[u64],
        plans: &mut [OutputPlan<'a>],
    ) -> Result<(), IndexRegionEvaluationError> {
        let mut frame = Frame::default();
        for ((dimension, _), coordinate) in parallel.iter().zip(point) {
            frame.environment.insert(*dimension, *coordinate);
        }
        for plan in plans {
            let value = self.value(&mut frame, plan.value)?;
            if value.resolved_type() != &plan.value_type {
                return Err(IndexRegionEvaluationError::MalformedRegion);
            }
            let element = dense_element(&value)?;
            let offset = self.access_offset(&mut frame, plan.access, plan.shape)?;
            let access = plan.access.id();
            let slot = plan
                .elements
                .get_mut(offset)
                .ok_or(IndexRegionEvaluationError::CoordinateOutOfBounds { access })?;
            if slot.is_some() {
                return Err(IndexRegionEvaluationError::DuplicateWrite { access });
            }
            *slot = Some(element);
        }
        Ok(())
    }

    fn access_offset(
        &mut self,
        frame: &mut Frame,
        access: TensorAccessRef<'a>,
        shape: &Shape,
    ) -> Result<usize, IndexRegionEvaluationError> {
        let coordinates: Vec<_> = access.coordinates().collect();
        let extents = shape.extents();
        if coordinates.len() != extents.len() {
            return Err(IndexRegionEvaluationError::MalformedRegion);
        }
        let outside = IndexRegionEvaluationError::CoordinateOutOfBounds {
            access: access.id(),
        };
        let mut linear = 0_usize;
        for (expression, extent) in coordinates.into_iter().zip(extents) {
            let evaluated = self.expression(frame, expression)?;
            let index = evaluated
                .to_u64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| outside.clone())?;
            let bound = usize::try_from(extent.get()).map_err(|_| outside.clone())?;
            if index >= bound {
                return Err(outside);
            }
            linear = linear
                .checked_mul(bound)
                .and_then(|base| base.checked_add(index))
                .ok_or_else(|| outside.clone())?;
        }
        Ok(linear)
    }

    fn expression(
        &mut self,
        frame: &mut Frame,
        id: VerifiedIndexExprId,
    ) -> Result<ExactInteger, IndexRegionEvaluationError> {
        if let Some(value) = frame.expressions.get(&id) {
            return Ok(value.clone());
        }
        self.step()?;
        self.enter()?;
        let region = self.region;
        let expression = region
            .index_expression(id)
            .map_err(IndexRegionEvaluationError::Handle)?;
        let value = match expression.view() {
            IndexExprView::Constant(constant) => {
                admit_index(ExactInteger::from_index_integer(constant))?
            }
            IndexExprView::Dimension(dimension) => ExactInteger::from_u64(
                frame
                    .environment
                    .get(&dimension)
                    .copied()
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?,
            ),
            IndexExprView::LinearCombination { constant, terms } => {
                let mut total = admit_index(ExactInteger::from_index_integer(constant))?;
                for term in terms {
                    let coefficient =
                        admit_index(ExactInteger::from_index_integer(term.coefficient()))?;
                    let child = self.expression(frame, term.value())?;
                    let product = coefficient
                        .checked_mul(&child, MAX_EVALUATED_INDEX_BYTES)
                        .map_err(magnitude_error)?;
                    total = total
                        .checked_add(&product, MAX_EVALUATED_INDEX_BYTES)
                        .map_err(magnitude_error)?;
                }
                total
            }
            IndexExprView::FloorDiv { dividend, divisor } => {
                self.expression(frame, dividend)?
                    .div_mod_floor(divisor)
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?
                    .0
            }
            IndexExprView::Modulo { dividend, divisor } => {
                self.expression(frame, dividend)?
                    .div_mod_floor(divisor)
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?
                    .1
            }
            _ => return Err(unsupported(UnsupportedRegionFeature::IndexExpressionForm)),
        };
        self.leave();
        frame.expressions.insert(id, value.clone());
        Ok(value)
    }

    fn value(
        &mut self,
        frame: &mut Frame,
        id: VerifiedScalarValueId,
    ) -> Result<Tensor, IndexRegionEvaluationError> {
        if let Some(value) = frame.values.get(&id) {
            return Ok(value.clone());
        }
        self.step()?;
        self.enter()?;
        let region = self.region;
        let scalar = region
            .scalar_value(id)
            .map_err(IndexRegionEvaluationError::Handle)?;
        let value = match scalar.definition() {
            ScalarValueDefinitionView::AccessRead(access) => {
                let value = self.read(frame, access, scalar.value_type())?;
                frame.values.insert(id, value.clone());
                value
            }
            ScalarValueDefinitionView::OperationResult { operation, .. } => {
                let results = self.operation(frame, operation)?;
                let ids: Vec<_> = region
                    .scalar_operation(operation)
                    .map_err(IndexRegionEvaluationError::Handle)?
                    .results()
                    .collect();
                if ids.len() != results.len() {
                    return Err(IndexRegionEvaluationError::MalformedRegion);
                }
                for (result, value) in ids.into_iter().zip(results) {
                    frame.values.insert(result, value);
                }
                frame
                    .values
                    .get(&id)
                    .cloned()
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?
            }
            _ => return Err(unsupported(UnsupportedRegionFeature::ScalarValueForm)),
        };
        self.leave();
        Ok(value)
    }

    fn read(
        &mut self,
        frame: &mut Frame,
        access: VerifiedTensorAccessId,
        value_type: &ResolvedValueType,
    ) -> Result<Tensor, IndexRegionEvaluationError> {
        let region = self.region;
        let access = region
            .access(access)
            .map_err(IndexRegionEvaluationError::Handle)?;
        let tensor = region
            .tensor(access.tensor())
            .map_err(IndexRegionEvaluationError::Handle)?;
        if access.mode() != AccessMode::Read || tensor.value_type() != value_type {
            return Err(IndexRegionEvaluationError::MalformedRegion);
        }
        let shape = tensor
            .static_shape()
            .ok_or_else(|| unsupported(UnsupportedRegionFeature::SymbolicTensorShape))?;
        let offset = self.access_offset(frame, access, shape)?;
        let bound = *self
            .inputs
            .get(&access.tensor())
            .ok_or(IndexRegionEvaluationError::MalformedRegion)?;
        let TensorPayloadView::Dense(elements) = bound.payload() else {
            return Err(unsupported(
                UnsupportedRegionFeature::CompoundValueRepresentation,
            ));
        };
        let element = elements
            .get(offset)
            .ok_or(IndexRegionEvaluationError::CoordinateOutOfBounds {
                access: access.id(),
            })?
            .clone();
        Tensor::scalar(value_type.clone(), element)
            .map_err(|source| IndexRegionEvaluationError::Value(Arc::new(source)))
    }

    fn operation(
        &mut self,
        frame: &mut Frame,
        id: VerifiedScalarOperationId,
    ) -> Result<Vec<Tensor>, IndexRegionEvaluationError> {
        let region = self.region;
        let operation = region
            .scalar_operation(id)
            .map_err(IndexRegionEvaluationError::Handle)?;
        match operation.kind() {
            ScalarOperationKindRef::Apply { .. } => {
                let operand_ids: Vec<_> = operation.operands().collect();
                let mut operands = Vec::with_capacity(operand_ids.len());
                for operand in operand_ids {
                    operands.push(self.value(frame, operand)?);
                }
                let results = self.value_types(operation.results())?;
                let application = self
                    .applications
                    .get(&id)
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?;
                self.run(application, &operands, &results)
            }
            ScalarOperationKindRef::Reduce(reduction) => self.reduce(frame, operation, reduction),
            _ => Err(unsupported(UnsupportedRegionFeature::ScalarOperationKind)),
        }
    }

    fn run(
        &self,
        application: &Application,
        operands: &[Tensor],
        results: &[ResolvedValueType],
    ) -> Result<Vec<Tensor>, IndexRegionEvaluationError> {
        let borrowed: Vec<&Tensor> = operands.iter().collect();
        let mut outputs = ScalarReferenceOutputs::new(results.len());
        let callback = application.implementation.evaluate(
            ScalarReferenceRequest {
                operands: &borrowed,
                attributes: &application.attributes,
            },
            &mut outputs,
        );
        let values = outputs.finish(callback).map_err(|source| {
            IndexRegionEvaluationError::ScalarOperation {
                capability: Arc::clone(&application.capability),
                source,
            }
        })?;
        for (result_index, (value, expected)) in values.iter().zip(results).enumerate() {
            if value.shape().rank() != 0 || value.resolved_type() != expected {
                return Err(IndexRegionEvaluationError::ScalarResult {
                    capability: Arc::clone(&application.capability),
                    result_index,
                });
            }
            self.evaluator
                .values
                .validate_value(value, self.authority.semantic())
                .map_err(|source| IndexRegionEvaluationError::Value(Arc::new(source)))?;
        }
        Ok(values)
    }

    fn reduce(
        &mut self,
        frame: &mut Frame,
        operation: ScalarOperationRef<'a>,
        reduction: ScalarReductionRef<'a>,
    ) -> Result<Vec<Tensor>, IndexRegionEvaluationError> {
        let init: Vec<_> = reduction.init().collect();
        let mut state = Vec::with_capacity(init.len());
        for id in init {
            state.push(self.value(frame, id)?);
        }
        let bound = self.domain(reduction.dimensions())?;
        let contributors: Vec<_> = reduction.contributors().collect();
        let yields: Vec<_> = reduction.body().yields().collect();
        let extents: Vec<u64> = bound.iter().map(|(_, extent)| *extent).collect();
        if !extents.contains(&0) {
            let mut point = vec![0_u64; extents.len()];
            loop {
                state = self.reduce_step(frame, &bound, &point, &contributors, &yields, &state)?;
                if !advance_point(&mut point, &extents) {
                    break;
                }
            }
        }
        let results = self.value_types(operation.results())?;
        if results.len() != state.len()
            || state
                .iter()
                .zip(&results)
                .any(|(value, expected)| value.resolved_type() != expected)
        {
            return Err(IndexRegionEvaluationError::MalformedRegion);
        }
        Ok(state)
    }

    fn reduce_step(
        &mut self,
        outer: &Frame,
        bound: &[(VerifiedDimensionId, u64)],
        point: &[u64],
        contributors: &[VerifiedScalarValueId],
        yields: &[VerifiedReducerBodyValueId],
        state: &[Tensor],
    ) -> Result<Vec<Tensor>, IndexRegionEvaluationError> {
        let mut inner = Frame {
            environment: outer.environment.clone(),
            ..Frame::default()
        };
        for ((dimension, _), coordinate) in bound.iter().zip(point) {
            inner.environment.insert(*dimension, *coordinate);
        }
        let mut contributor_values = Vec::with_capacity(contributors.len());
        for id in contributors {
            contributor_values.push(self.value(&mut inner, *id)?);
        }
        let mut context = BodyContext {
            state,
            contributors: &contributor_values,
            values: HashMap::new(),
        };
        let mut next = Vec::with_capacity(yields.len());
        for id in yields {
            next.push(self.body_value(&mut context, *id)?);
        }
        Ok(next)
    }

    fn body_value(
        &mut self,
        context: &mut BodyContext<'_>,
        id: VerifiedReducerBodyValueId,
    ) -> Result<Tensor, IndexRegionEvaluationError> {
        if let Some(value) = context.values.get(&id) {
            return Ok(value.clone());
        }
        self.step()?;
        self.enter()?;
        let region = self.region;
        let declaration = region
            .reducer_body_value(id)
            .map_err(IndexRegionEvaluationError::Handle)?;
        let value = match declaration.definition() {
            ReducerBodyValueDefinitionView::StateParameter(index) => {
                parameter(context.state, index)?
            }
            ReducerBodyValueDefinitionView::ContributorParameter(index) => {
                parameter(context.contributors, index)?
            }
            ReducerBodyValueDefinitionView::OperationResult { operation, .. } => {
                self.body_operation(context, operation)?;
                context
                    .values
                    .get(&id)
                    .cloned()
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?
            }
            _ => return Err(unsupported(UnsupportedRegionFeature::ReducerBodyValueForm)),
        };
        if value.resolved_type() != declaration.value_type() {
            return Err(IndexRegionEvaluationError::MalformedRegion);
        }
        self.leave();
        context.values.insert(id, value.clone());
        Ok(value)
    }

    fn body_operation(
        &mut self,
        context: &mut BodyContext<'_>,
        id: VerifiedReducerBodyOperationId,
    ) -> Result<(), IndexRegionEvaluationError> {
        let region = self.region;
        let operation = region
            .reducer_body_operation(id)
            .map_err(IndexRegionEvaluationError::Handle)?;
        let operand_ids: Vec<_> = operation.operands().collect();
        let mut operands = Vec::with_capacity(operand_ids.len());
        for operand in operand_ids {
            operands.push(self.body_value(context, operand)?);
        }
        let result_ids: Vec<_> = operation.results().collect();
        let results = self.body_value_types(operation.results())?;
        let application = self
            .body_applications
            .get(&id)
            .ok_or(IndexRegionEvaluationError::MalformedRegion)?;
        let values = self.run(application, &operands, &results)?;
        if values.len() != result_ids.len() {
            return Err(IndexRegionEvaluationError::MalformedRegion);
        }
        for (result, value) in result_ids.into_iter().zip(values) {
            context.values.insert(result, value);
        }
        Ok(())
    }
}

fn parameter(values: &[Tensor], index: u32) -> Result<Tensor, IndexRegionEvaluationError> {
    usize::try_from(index)
        .ok()
        .and_then(|index| values.get(index))
        .cloned()
        .ok_or(IndexRegionEvaluationError::MalformedRegion)
}
