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
use std::sync::{Arc, OnceLock};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    AccessMode, CanonicalScalarDefinitionProjection, FrozenScalarRegistry, IndexExprView,
    MAX_INDEX_INTEGER_BYTES, ReducerBodyValueDefinitionView, ReductionTraversal,
    ScalarAttributeField, ScalarAttributes, ScalarAuthorityEvidence, ScalarOpKey,
    ScalarOperationDefinition, ScalarOperationKindRef, ScalarOperationRef, ScalarReductionRef,
    ScalarRegistryError, ScalarValueDefinitionView, SourcedExtent, TensorAccessRef, TensorRole,
    VerifiedDimensionId, VerifiedIndexExprId, VerifiedIndexHandleError, VerifiedIndexRegion,
    VerifiedReducerBodyOperationId, VerifiedReducerBodyValueId, VerifiedScalarOperationId,
    VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId, add_f32_scalar_op,
    canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, multiply_f32_scalar_op,
    strict_affine_u4_dequantize_scalar_op,
};
use tiler_ir::semantic::{
    CanonicalField, CanonicalValue, CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE,
    FrozenSemanticRegistry, ProviderIdentity, ResolvedValueType, TypeKey, U4,
};
use tiler_ir::shape::{Extent, Shape};

use crate::arithmetic::{ExactInteger, MagnitudeExceeded};
use crate::evaluate::{decode_f32, f32_element};
use crate::identity::{
    encode_provider_capability, encode_signature, encoded_bytes_len,
    reference_provider_identity_len, reference_signature_identity_len,
};
use crate::{
    EvaluationError, FloatBitOrder, FrozenReferenceRegistry, MAX_REFERENCE_CAPABILITIES,
    MAX_REFERENCE_REGISTRY_IDENTITY_BYTES, MAX_REFERENCE_TENSOR_ELEMENTS,
    ReferenceCapabilityRevision, ReferenceElement, ReferenceNumericalConformance,
    ReferenceOperationError, ReferenceRegistryError, ReferenceRegistryResource, ReferenceSignature,
    Tensor, TensorPayloadView, canonicalize_arithmetic_f32,
};

/// Maximum scalar, reducer-body, and index evaluations in one evaluated span.
///
/// **A running counter, not a product of extents.** Every other governed bound
/// in this file is answerable before any work happens: an output's element count
/// comes from its shape, a magnitude from the integer, a depth from the nesting.
/// This one is incremented by [`RegionEvaluation::step`] on each scalar,
/// reducer-body, and index-expression evaluation, and what a region costs per
/// iteration point is a property of its own expression graph and reducer body
/// rather than of its domain. No caller can therefore predict the cost before
/// asking — including [`IndexRegionEvaluator::evaluate`] on an arbitrary
/// verified region, which is the caller this bound exists for — and that is
/// exactly why it is a running bound rather than a preflight one.
///
/// The unit it bounds is **one span**: one call that walks a contiguous run of
/// the region's *root points* — one write root at one point of that root's own
/// iteration domain, the unit [`StagedIndexRegionEvaluation`] defines and every
/// count in this file is stated in. [`IndexRegionEvaluator::evaluate`] is one
/// span over all of them, so the whole-region walk is held to this number
/// exactly as it always was, and [`StagedIndexRegionEvaluation`] reaches a
/// larger region by spending several bounded spans rather than by weakening the
/// number. A span whose points cost more than this refuses whatever its point
/// count is: a single point over the bound is refused at a span of one.
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
    /// A floor division or modulo divided by a symbolic extent.
    ///
    /// This oracle evaluates one exact point at a time against concrete input
    /// tensors, so a divisor it cannot read is a value it would have to invent.
    /// ADR 0046 admits exactly this refusal: a pass may "conservatively decline
    /// semi-affine maps they cannot analyze", and declining is the only
    /// alternative to returning a coordinate nothing established.
    SymbolicIndexDivisor,
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
            Self::SymbolicIndexDivisor => "symbolic index floor-division or modulo divisor",
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
    conformance: ReferenceNumericalConformance,
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

    /// Returns the numerical contract this evaluation is performed under.
    ///
    /// A scalar capability that performs floating-point arithmetic must consult
    /// this. The refined region and the semantic evaluator answer the same
    /// program, so one honouring the contract and the other ignoring it would
    /// disagree on exactly the values the contract exists to decide.
    #[must_use]
    pub const fn conformance(self) -> ReferenceNumericalConformance {
        self.conformance
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
    /// Composing the governed standard scalar authority itself failed.
    ///
    /// This is a defect in Tiler's own governed profile rather than in a
    /// caller's registration, so it names no operation.
    ScalarRegistry(Arc<ScalarRegistryError>),
    /// Forming an exact resolved signature or capability revision failed.
    ReferenceRegistry(Arc<ReferenceRegistryError>),
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
            Self::ScalarRegistry(source) => {
                write!(
                    formatter,
                    "governed standard scalar authority failed: {source}"
                )
            }
            Self::ReferenceRegistry(source) => {
                write!(formatter, "reference registry failure: {source}")
            }
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
            Self::ScalarAuthority { source, .. } | Self::ScalarRegistry(source) => {
                Some(source.as_ref())
            }
            Self::ReferenceRegistry(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<ReferenceRegistryError> for ScalarReferenceRegistryError {
    fn from(value: ReferenceRegistryError) -> Self {
        Self::ReferenceRegistry(Arc::new(value))
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

    /// Creates the governed standard scalar reference profile.
    ///
    /// The builder is bound to [`FrozenScalarRegistry::standard`] and defines an
    /// executable oracle rows for `tiler.scalar::constant-f32@1`,
    /// `multiply-f32@1`, `add-f32@1`, `canonicalize-nan-f32@1`, and
    /// `strict-affine-u4-dequantize@1`. The bound standard scalar authority also
    /// defines `divide-f32@1` and `exp-f32@1`; this bounded reference profile
    /// does not install those two capabilities, so a region reaching either
    /// refuses with [`IndexRegionEvaluationError::MissingScalarCapability`]. An
    /// extension composes with this builder by registering further capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarReferenceRegistryError::ScalarRegistry`] when the
    /// governed scalar authority rejects its own standard profile, or another
    /// typed error when a governed registration violates the same public
    /// contract an extension is held to.
    pub fn standard() -> Result<Self, ScalarReferenceRegistryError> {
        let scalars = FrozenScalarRegistry::standard()
            .map_err(|source| ScalarReferenceRegistryError::ScalarRegistry(Arc::new(source)))?;
        let mut builder = Self::new(scalars);
        let provider = standard_scalar_reference_provider();
        let revision = ReferenceCapabilityRevision::new(1)?;
        let f32_type = F32::resolved_type();
        builder.register(
            provider.clone(),
            constant_f32_scalar_op(),
            ReferenceSignature::new([], [f32_type.clone()])?,
            revision,
            Arc::new(StandardScalarConstantF32),
        )?;
        let binary =
            || ReferenceSignature::new([f32_type.clone(), f32_type.clone()], [f32_type.clone()]);
        builder.register(
            provider.clone(),
            multiply_f32_scalar_op(),
            binary()?,
            revision,
            Arc::new(StandardScalarBinaryF32::Multiply),
        )?;
        builder.register(
            provider.clone(),
            add_f32_scalar_op(),
            binary()?,
            revision,
            Arc::new(StandardScalarBinaryF32::Add),
        )?;
        builder.register(
            provider.clone(),
            canonicalize_nan_f32_scalar_op(),
            ReferenceSignature::new([f32_type.clone()], [f32_type])?,
            revision,
            Arc::new(StandardScalarCanonicalizeNanF32),
        )?;
        builder.register(
            provider,
            strict_affine_u4_dequantize_scalar_op(),
            ReferenceSignature::new(
                [
                    U4::resolved_type(),
                    F32::resolved_type(),
                    U4::resolved_type(),
                ],
                [F32::resolved_type()],
            )?,
            revision,
            Arc::new(StandardScalarStrictAffineU4Dequantize),
        )?;
        Ok(builder)
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
    /// Builds the governed standard scalar reference profile.
    ///
    /// The snapshot is computed once and shared, so every consumer that executes
    /// a region emitted by the governed `f32` index-access lowerings checks it
    /// against the same oracle instead of composing an ad-hoc one.
    ///
    /// # Errors
    ///
    /// Returns a typed error when a governed registration violates the same
    /// public contract used by extensions.
    pub fn standard() -> Result<Self, ScalarReferenceRegistryError> {
        static STANDARD: OnceLock<
            Result<FrozenScalarReferenceRegistry, ScalarReferenceRegistryError>,
        > = OnceLock::new();
        STANDARD
            .get_or_init(|| ScalarReferenceRegistryBuilder::standard()?.freeze())
            .clone()
    }

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

/// Returns the admitting provider of the governed standard scalar oracle.
fn standard_scalar_reference_provider() -> ProviderIdentity {
    ProviderIdentity::new("tiler", "standard-scalar-reference", 1)
        .expect("the governed scalar reference provider identity is valid")
}

/// Returns the governed binary32 format key the scalar payloads are typed by.
fn f32_scalar_format() -> TypeKey {
    TypeKey::new("tiler", "f32", 1).expect("the governed f32 format key is valid")
}

/// Decodes one rank-zero governed `f32` operand.
fn decode_scalar_f32(value: &Tensor) -> Result<f32, ReferenceOperationError> {
    if value.resolved_type() != &F32::resolved_type() || value.shape().rank() != 0 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let TensorPayloadView::Dense([element]) = value.payload() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    decode_f32(element)
}

/// Wraps one exact `f32` payload as a rank-zero governed scalar value.
fn scalar_f32_value(value: f32) -> Result<Tensor, ReferenceOperationError> {
    Tensor::scalar(F32::resolved_type(), f32_element(value)?)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn decode_scalar_u4(value: &Tensor) -> Result<u8, ReferenceOperationError> {
    if value.resolved_type() != &U4::resolved_type() || value.shape().rank() != 0 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let TensorPayloadView::Dense([element]) = value.payload() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let [code] = element.as_bytes() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    if *code > 15 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    Ok(*code)
}

/// Evaluates `tiler.scalar::constant-f32@1` as its exact declared payload.
///
/// A constant reproduces its attribute bits unchanged, including a NaN payload
/// and the sign of a zero. It performs no arithmetic, so
/// [`canonicalize_arithmetic_f32`] deliberately does not apply: canonicalizing
/// here would make the region unable to materialize an exact binary32 pattern
/// the governed `tiler::constant-f32@1` definition promises to carry verbatim.
///
/// The request's declared conformance is not read for the same reason and at the
/// same boundary. A constant has no operands, so nothing enters an arithmetic
/// operation, and its result is a declared payload rather than a value arithmetic
/// produced — so neither subnormal dimension has a site, and a subnormal constant
/// is materialized exactly as declared under every contract.
struct StandardScalarConstantF32;

impl ScalarReferenceOperation for StandardScalarConstantF32 {
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        if !request.operands().is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let CanonicalValueView::Record(fields) = request.attributes().value().view() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if fields.len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(CanonicalValueView::FloatBits(bits)) = fields
            .iter()
            .find(|field| field.id() == F32_CONSTANT_BITS_ATTRIBUTE)
            .map(|field| field.value().view())
        else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if bits.format() != &f32_scalar_format() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let element =
            ReferenceElement::from_float_bits(bits.bits(), FloatBitOrder::MostSignificantByteFirst)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        outputs.push(
            Tensor::scalar(F32::resolved_type(), element)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?,
        )
    }
}

/// Evaluates one governed binary `f32` scalar at one iteration point.
///
/// Each application is one separately rounded binary32 operation whose NaN
/// result takes the governed canonical payload, which is exactly what the
/// tensor-level `tiler::multiply-f32@1` and `tiler::add-f32@1` oracles do. A
/// scalar that propagated the host's NaN payload instead would make a refined
/// region and the semantic evaluator disagree on the same program.
enum StandardScalarBinaryF32 {
    Multiply,
    Add,
}

impl ScalarReferenceOperation for StandardScalarBinaryF32 {
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [left, right] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let CanonicalValueView::Record(fields) = request.attributes().value().view() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !fields.is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let conformance = request.conformance();
        let left = conformance.apply_to_operand(decode_scalar_f32(left)?);
        let right = conformance.apply_to_operand(decode_scalar_f32(right)?);
        let value = match self {
            Self::Multiply => left * right,
            Self::Add => left + right,
        };
        outputs.push(scalar_f32_value(
            conformance.apply_to_result(canonicalize_arithmetic_f32(value)),
        )?)
    }
}

/// Evaluates `tiler.scalar::canonicalize-nan-f32@1` as the governed conversion.
///
/// This is a conversion, not arithmetic: it replaces a NaN with the governed
/// canonical arithmetic payload and reproduces every other binary32 pattern
/// verbatim, including the sign of a zero, subnormals, and infinities. That
/// exactness is what lets a reduction realize its result-boundary
/// canonicalization on a lone contributor without an addition, which would
/// otherwise turn an observable `-0.0` into `+0.0`.
///
/// **Neither subnormal dimension applies, and that is what makes the conversion a
/// conversion.** No operand enters an arithmetic operation and no arithmetic
/// produces a result here, so a subnormal reaches the output exactly as it
/// arrived — under a preserving contract and under a flushing one alike. This is
/// the scalar counterpart of the tensor reduction committing a lone contributor
/// without an addition: both realize a result boundary, and a flush at a boundary
/// nothing computed would model a device flushing a move.
struct StandardScalarCanonicalizeNanF32;

impl ScalarReferenceOperation for StandardScalarCanonicalizeNanF32 {
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [operand] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let CanonicalValueView::Record(fields) = request.attributes().value().view() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !fields.is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let value = decode_scalar_f32(operand)?;
        outputs.push(scalar_f32_value(canonicalize_arithmetic_f32(value))?)
    }
}

/// Executes the atomic scalar meaning used by the strict-affine logical law.
///
/// The request's declared conformance has no site with a value in range, and the
/// governed value contract is what discharges it rather than this evaluator: the
/// scale is refused unless it is a positive *normal*, the centered code is an
/// exact integer in `-15..=15`, and a nonzero product's magnitude is therefore at
/// least the scale and so at least the minimum normal. No operand and no produced
/// value can be subnormal, so both dimensions are vacuous here — which is the same
/// argument the tensor-level `dequantize-strict-affine` reference records, read at
/// the same contract.
struct StandardScalarStrictAffineU4Dequantize;

impl ScalarReferenceOperation for StandardScalarStrictAffineU4Dequantize {
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [codes, scale, zero_point] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let CanonicalValueView::Record(fields) = request.attributes().value().view() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !fields.is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let code = i32::from(decode_scalar_u4(codes)?);
        let zero_point = i32::from(decode_scalar_u4(zero_point)?);
        let scale = decode_scalar_f32(scale)?;
        if !scale.is_normal() || scale <= 0.0 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let centered = code - zero_point;
        #[allow(
            clippy::cast_precision_loss,
            reason = "the difference of two U4 codes is in -15..=15 and is represented exactly by f32"
        )]
        let centered = centered as f32;
        let value = centered * scale;
        outputs.push(scalar_f32_value(canonicalize_arithmetic_f32(value))?)
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
    push_slice(output, authority.definitions.as_bytes());
    push_slice(output, authority.provider.namespace().as_bytes());
    push_slice(output, authority.provider.name().as_bytes());
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
    push_slice(&mut bytes, scalar_registry.snapshot_identity().as_bytes());
    push_len(&mut bytes, capabilities.len());
    for (key, capability) in capabilities {
        push_slice(&mut bytes, key.operation.namespace().as_bytes());
        push_slice(&mut bytes, key.operation.name().as_bytes());
        bytes.extend_from_slice(&key.operation.semantic_version().to_be_bytes());
        encode_signature(&mut bytes, &key.signature);
        encode_scalar_authority(&mut bytes, &capability.authority);
        encode_provider_capability(&mut bytes, &capability.provider, capability.revision);
    }
    debug_assert_eq!(bytes.len(), exact_len);
    CanonicalScalarReferenceRegistryIdentity(bytes)
}

/// The exact frozen authority one verified region is evaluated under.
///
/// **One argument, because the second could disagree with it.** A scalar
/// registry is frozen *against* a semantic registry and carries it, so a caller
/// that supplied both could name a semantic authority the scalar authority was
/// never registered under — an evaluation governed by two authorities that
/// nothing compared. The semantic half is derived rather than accepted, which
/// removes the disagreement rather than checking for it.
#[derive(Clone, Copy, Debug)]
pub struct IndexRegionAuthority<'a> {
    scalar: &'a FrozenScalarRegistry,
}

impl<'a> IndexRegionAuthority<'a> {
    /// Names the scalar authority governing one region.
    ///
    /// The semantic type authority is [`FrozenScalarRegistry::semantic_authority`],
    /// the one this scalar registry was frozen against.
    #[must_use]
    pub const fn new(scalar: &'a FrozenScalarRegistry) -> Self {
        Self { scalar }
    }

    /// Returns the scalar authority.
    #[must_use]
    pub const fn scalar(self) -> &'a FrozenScalarRegistry {
        self.scalar
    }

    /// Returns the semantic type authority the scalar authority was frozen
    /// against.
    #[must_use]
    pub fn semantic(self) -> &'a FrozenSemanticRegistry {
        self.scalar.semantic_authority()
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
    /// Returns one tensor per output boundary, ordered by each boundary's first
    /// write root.
    ///
    /// An output several roots partition appears once, jointly filled, because
    /// one boundary is one tensor however many roots the region split its
    /// production across.
    #[must_use]
    pub fn outputs(&self) -> &[Tensor] {
        &self.outputs
    }

    /// Returns the scalar authority receipt bound to this exact region identity.
    #[must_use]
    pub const fn authority(&self) -> &ScalarAuthorityEvidence {
        &self.authority
    }

    /// Consumes this evaluation and returns its ordered output-boundary tensors.
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
        ///
        /// An output several roots partition has no single access at fault: the
        /// gap belongs to the root set, whose coverage obligation is joint. The
        /// output's first root in region order is named, because the alternative
        /// — naming whichever root the fill happened to reach last — would make
        /// the attribution an artifact of iteration order.
        ///
        /// Roots iterating different domains do not change that. The named root
        /// is a stable handle on the boundary whose coverage failed, not a
        /// claim that it is the short one, and the two alternatives are both
        /// worse: deciding *which* root should have covered the gap means
        /// re-deriving the partition proof this check exists to be independent
        /// of, and naming no root at all removes the only handle a caller has
        /// on which boundary failed. A root that owns nothing — the degenerate
        /// zero-extent member — can be the one named, and that is honest here
        /// in a way it was not while a sub-domain root made this fire for a
        /// *valid* region: this refusal is now reachable only for a region
        /// whose roots genuinely leave an element unwritten.
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
    /// A staged span asked for no root points.
    ///
    /// A span that walks nothing cannot advance the evaluation, so a caller
    /// looping until exhaustion on one would never finish. Refused at the call
    /// that made it rather than diagnosed later as an incomplete walk.
    EmptyStagedSpan,
    /// [`StagedIndexRegionEvaluation::finish`] ran with root points unwalked.
    IncompleteStagedWalk {
        /// Root points the spans covered before finishing.
        evaluated: u64,
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
            Self::EmptyStagedSpan => {
                formatter.write_str("a staged span must walk at least one root point")
            }
            Self::IncompleteStagedWalk { evaluated } => write!(
                formatter,
                "the staged walk finished after {evaluated} root points, leaving some write root's own domain uncovered"
            ),
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

/// Reads a floor-division or modulo divisor this oracle can actually divide by.
///
/// The one place the affine-only boundary of this evaluator is drawn. A divisor
/// the region names symbolically is refused rather than resolved through the
/// region's shape environment: this evaluator is the correctness oracle other
/// results are compared against, so a value it derived from a second authority
/// would be a value the comparison could not distinguish from the subject's own
/// derivation of it.
fn constant_divisor(divisor: &SourcedExtent) -> Result<u64, IndexRegionEvaluationError> {
    divisor
        .as_static()
        .map(Extent::get)
        .ok_or_else(|| unsupported(UnsupportedRegionFeature::SymbolicIndexDivisor))
}

/// Host evaluator for verified canonical index regions.
#[derive(Clone, Debug)]
pub struct IndexRegionEvaluator {
    values: FrozenReferenceRegistry,
    scalars: FrozenScalarReferenceRegistry,
    conformance: ReferenceNumericalConformance,
}

impl IndexRegionEvaluator {
    /// Creates an evaluator over exact value-representation and scalar
    /// snapshots, evaluating the strict reading.
    ///
    /// The strict reading is what this oracle computed before it could be told a
    /// contract, so this constructor changes no result. Use [`Self::under`] to
    /// evaluate a region whose declared realization flushes subnormals; a region
    /// carries that realization, and
    /// [`crate::ReferenceNumericalConformance::from_realization`] is the checked
    /// bridge from one to the other.
    #[must_use]
    pub const fn new(
        values: FrozenReferenceRegistry,
        scalars: FrozenScalarReferenceRegistry,
    ) -> Self {
        Self::under(values, scalars, ReferenceNumericalConformance::strict())
    }

    /// Creates an evaluator bound to one stated numerical contract.
    #[must_use]
    pub const fn under(
        values: FrozenReferenceRegistry,
        scalars: FrozenScalarReferenceRegistry,
        conformance: ReferenceNumericalConformance,
    ) -> Self {
        Self {
            values,
            scalars,
            conformance,
        }
    }

    /// Returns the numerical contract every evaluation is performed under.
    #[must_use]
    pub const fn conformance(&self) -> ReferenceNumericalConformance {
        self.conformance
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

    /// Evaluates every output boundary of one verified region, over every root
    /// that writes it.
    ///
    /// The region's scalar authority revalidates first, so a region this
    /// authority cannot admit never reaches an executable capability. Every
    /// coordinate, write coverage claim, and produced value is then checked
    /// independently of the structural verifier's own proofs.
    ///
    /// This is [`Self::stage`] walked in **one span**, which is why the whole
    /// walk is held to `MAX_EVALUATION_STEPS` as a single ask: the staged
    /// entry point below did not give this path a second, larger budget, it
    /// gave a caller a way to spend several of this one.
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
        let mut staged = self.stage(region, authority, inputs)?;
        staged.evaluate_root_points(u64::MAX)?;
        staged.finish()
    }

    /// Begins one region evaluation the caller walks in bounded spans.
    ///
    /// Revalidation, input binding, capability resolution, and output planning
    /// all happen here, so a region this authority cannot admit is refused
    /// before any point is walked and a caller learns the size of the walk
    /// ([`StagedIndexRegionEvaluation::root_point_count`]) without paying
    /// for it.
    ///
    /// # Errors
    ///
    /// Returns the same authority, binding, capability, unsupported-feature and
    /// output-planning failures [`Self::evaluate`] returns before its walk.
    pub fn stage<'a>(
        &'a self,
        region: &'a VerifiedIndexRegion,
        authority: IndexRegionAuthority<'a>,
        inputs: &[IndexRegionInput<'a>],
    ) -> Result<StagedIndexRegionEvaluation<'a>, IndexRegionEvaluationError> {
        let evidence = authority
            .scalar()
            .revalidate_region(region)
            .map_err(|source| IndexRegionEvaluationError::ScalarAuthority(Arc::new(source)))?;
        if evidence.semantic_snapshot() != authority.semantic().snapshot_identity() {
            return Err(IndexRegionEvaluationError::SemanticAuthorityMismatch);
        }
        let evaluation = RegionEvaluation::new(region, self, authority, inputs)?;
        let plans = evaluation.output_plans()?;
        // The cursor is settled at construction and after every point, so
        // `is_exhausted` reads a field rather than searching for a root with
        // work left — and a region whose every root is empty is exhausted
        // before the first span, which is what makes it finishable.
        let cursor = RootCursor::new(&plans);
        Ok(StagedIndexRegionEvaluation {
            evaluation,
            cursor,
            plans,
            authority: evidence,
            failure: None,
        })
    }
}

/// One region evaluation walked in caller-sized spans of its **root points**.
///
/// # The unit this type is stated in
///
/// A **root point** is one write root at one point of *that root's own*
/// iteration domain. It is the unit of [`Self::root_point_count`],
/// [`Self::evaluate_root_points`], [`Self::evaluated_root_points`], and of the
/// step budget's span, and by the bijection below there is exactly one root
/// point per element the region's output boundaries retain.
///
/// The unit is not the parallel point, and the difference is not cosmetic. A
/// write's iteration domain is any subset of the region's parallel dimensions,
/// which is what makes a partition with unequally sized members expressible: two
/// roots over one boundary declare dimensions of different extents rather than
/// sharing one, and a root's point count is the product of the extents *it*
/// iterates. The product of all the parallel extents is therefore a number no
/// root walks — for three-and-five into eight it is fifteen where eight elements
/// exist, and a zero-extent member zeroes it entirely while the region's other
/// roots still have every one of their points to walk. A caller sizing spans
/// against it would be dividing by a number this evaluation never counts.
///
/// # What this is for
///
/// `MAX_EVALUATION_STEPS` bounds one span, and a region's cost per iteration
/// point is not computable from its extents, so a region large enough is refused
/// by the whole-region path however well formed it is. This type reaches such a
/// region without moving that bound: each [`Self::evaluate_root_points`] call is
/// one bounded ask that passes exactly the test
/// [`IndexRegionEvaluator::evaluate`] applies, and the total is the caller's own
/// loop.
///
/// There is deliberately **no convenience that walks every remaining point in
/// one call**. The loop is the authorization: a single call that walked an
/// arbitrary total would put back the unbounded ask the bound exists to prevent,
/// and it is already available under its real name as
/// [`IndexRegionEvaluator::evaluate`].
///
/// # Why a span boundary cannot change a value
///
/// The argument is about what a [`VerifiedIndexRegion`] *proves*, not about the
/// shape of the loop below; the loop is what was checked against it.
///
/// - **A write root's stored value is a function of its own domain and nothing
///   else.** Two refusals hold this, one on each side of the root. A coordinate
///   may not name a dimension outside its access domain
///   (`IndexBuildError::CoordinateOutsideAccessDomain`), and — since a write
///   domain became a subset — verification refuses a root whose stored value
///   varies along a parallel dimension the write does not iterate
///   (`IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain`), because both
///   readings of such a region are wrong: firing the root once per point of the
///   omitted dimension stores several values to one element, and picking one
///   point of it stores a value nothing in the region selected. So seeding a
///   root's frame from its own domain alone leaves nothing undefined, and the
///   dimensions the root omits are not merely unvisited but unmentionable.
/// - **An output's writes are jointly total and injective over their own
///   domains.** Every write access carries a `WriteOwnershipProofView` in one of
///   three forms. `CoordinatePermutation`: each coordinate *is* a distinct
///   domain dimension whose extent the environment proves equal to the axis it
///   indexes. `Exhaustive`: a finite enumeration set one bit per covered element
///   and refused both a repeat and a gap. `PartitionMember`: this root is total
///   and injective over its own partition of an output several roots share, and
///   the joint obligation across that root set — pairwise disjoint partitions
///   whose union is the boundary exactly — was discharged over all of them
///   together, by interval reasoning over their rectangles or by one shared
///   enumeration bitset. Each of the three quantifies over the root's *own*
///   domain; none of them ever quantified over the region's parallel set, which
///   is why relaxing the write-domain rule cost the argument nothing.
///
///   The entity in bijection with an output's elements is therefore the
///   **(root, point of that root's own domain) pair** — the root point. Under
///   the first two forms the output has one root and the pair collapses to that
///   root's point, which is why the pair was invisible until partitions existed.
///   Under the third the map is still a bijection: each root is injective over
///   its own domain by its own proof, and the joint obligation makes the roots'
///   images pairwise disjoint and exactly covering, so distinct pairs reach
///   distinct elements and every element is reached. A zero-extent member
///   contributes no pair and owns no element, which is the degenerate case of
///   the same statement rather than an exception to it. Splitting the root
///   points therefore splits the pairs, so **a partition of the root points is a
///   partition of each output's elements**, taken over that output's roots
///   together, and no span can land on an element another span produced.
/// - **No value in a region can read a boundary the region writes.** The same
///   access preparation refuses a read of an output tensor with
///   `IndexBuildError::ReadFromOutput`. The written elements are the only state
///   spans share, and this makes them unobservable to the computation, so a root
///   point's value cannot depend on which other root points have already run.
///   This fact does the extra work the per-root walk needs: the walk visits the
///   pairs root-major rather than point-major, and an order change is only sound
///   because no root point can observe another's write.
///
/// What was checked against those three facts:
/// `RegionEvaluation::evaluate_root_point` builds a fresh `Frame` per root
/// point, seeded from that root's own domain; a read resolves only against the
/// immutable bound inputs; a reduction's state is built from its `init` values
/// inside the root point's own frame and folded through a `BodyContext` that
/// does not outlive it. How many reads a contributor performs never enters this
/// — a fold taking two operand reads per contributor composes exactly as one
/// taking a single read does.
///
/// # What staging does not weaken
///
/// The output element buffers live here, across spans, and there is one per
/// output *boundary* rather than one per root, so `DuplicateWrite` and
/// `IncompleteWrite` see precisely the elements an unstaged walk would have
/// seen: a duplicate across two spans or across two roots is the same slot
/// written twice, and a gap is the same `None` at [`Self::finish`]. Both checks
/// are the oracle's own, independent of the ownership proof cited above — for a
/// partitioned output they are this evaluator's own joint obligation, computed
/// over the same buffer the verifier's shared bitset covered — and neither
/// staging nor the per-root walk moves them. Their populations stay honest under
/// unequal roots for the reason the bijection gives: each root fires once per
/// point of its own domain, so a strict-subset root writes its rectangle exactly
/// once and `DuplicateWrite` is left reporting only a genuine collision between
/// two roots' images, while a root with no points writes nothing and the gap
/// `IncompleteWrite` reports is a gap in the boundary rather than an artifact of
/// which product the walk happened to take.
///
/// The step budget is not caller-held state. Each span is a self-contained
/// evaluation of a *stated* run of root points whose result is the whole walk's
/// result restricted to them, and the budget bounds one such ask unchanged; a
/// caller who wants more work must write more calls, which was already true of
/// [`IndexRegionEvaluator::evaluate`] itself. What is refused is resuming
/// *inside* a root point with a fresh budget, which would make the unit a caller
/// pays for something no one can state.
///
/// A span width is the caller's because nothing else can supply it: unlike a
/// fold whose widest admissible slab divides out of a step product, a region's
/// per-point cost is discovered by walking. The budget is what tells a caller
/// the width was too large.
pub struct StagedIndexRegionEvaluation<'a> {
    evaluation: RegionEvaluation<'a>,
    cursor: RootCursor,
    plans: Vec<OutputPlan<'a>>,
    authority: ScalarAuthorityEvidence,
    failure: Option<IndexRegionEvaluationError>,
}

impl fmt::Debug for StagedIndexRegionEvaluation<'_> {
    /// Renders the walk, never the borrowed input tensors.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StagedIndexRegionEvaluation")
            .field("root_point_count", &self.root_point_count())
            .field("evaluated_root_points", &self.cursor.evaluated)
            .field("exhausted", &self.cursor.exhausted)
            .field("output_count", &self.plans.len())
            .field("failed", &self.failure.is_some())
            .finish_non_exhaustive()
    }
}

impl StagedIndexRegionEvaluation<'_> {
    /// Returns the number of root points the whole walk covers.
    ///
    /// The sum over every root of the product of the extents *that root*
    /// iterates, which by the bijection above is the number of elements this
    /// region's output boundaries retain between them. A root over an empty
    /// domain contributes one — it stores one element at constant coordinates —
    /// and a root one of whose extents is zero contributes none.
    ///
    /// `None` when a root's own product, or the sum across roots, exceeds `u64`.
    /// That is not "very many": it is the case where no point count exists to
    /// report, and a saturated one would be a number a caller could divide by.
    /// Such a region is still walkable — and still refused by the step budget or
    /// by the write-coverage checks long before the count would have mattered.
    #[must_use]
    pub fn root_point_count(&self) -> Option<u64> {
        self.plans
            .iter()
            .flat_map(|plan| plan.roots.iter())
            .try_fold(0_u64, |total, root| {
                total.checked_add(root.walk.point_count()?)
            })
    }

    /// Returns the root points walked so far.
    #[must_use]
    pub const fn evaluated_root_points(&self) -> u64 {
        self.cursor.evaluated
    }

    /// Returns whether every root has been walked over its whole domain.
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.cursor.exhausted
    }

    /// Walks up to `points` further root points under one step budget.
    ///
    /// Returns how many were walked, which is fewer than asked exactly when the
    /// walk reached its end, and zero only when it had already reached it. A
    /// span may cross from one root to the next, and from one output boundary to
    /// the next, which is sound for the same reason it may cross between two
    /// points of one root: no root point can observe another's write.
    ///
    /// A failed span poisons this evaluation: the outputs it had written are a
    /// partial result, and a later call returning the original failure is what
    /// stops that partial result from being finished as if it were whole.
    ///
    /// # Errors
    ///
    /// Returns [`IndexRegionEvaluationError::EmptyStagedSpan`] for a span of no
    /// points, the retained failure of an earlier span, or any evaluation
    /// failure this span reaches — including
    /// [`IndexReferenceResource::EvaluationSteps`] when the span's points cost
    /// more than `MAX_EVALUATION_STEPS` between them.
    pub fn evaluate_root_points(&mut self, points: u64) -> Result<u64, IndexRegionEvaluationError> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        if points == 0 {
            return Err(IndexRegionEvaluationError::EmptyStagedSpan);
        }
        self.evaluation
            .evaluate_span(&mut self.cursor, &mut self.plans, points)
            .inspect_err(|failure| self.failure = Some(failure.clone()))
    }

    /// Finishes the walked outputs and their scalar authority receipt.
    ///
    /// # Errors
    ///
    /// Returns the retained failure of a poisoned evaluation,
    /// [`IndexRegionEvaluationError::IncompleteStagedWalk`] when root points
    /// remain unwalked, or the same write-coverage and value failures the
    /// whole-region path reports when it finishes its outputs.
    pub fn finish(self) -> Result<IndexRegionEvaluation, IndexRegionEvaluationError> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        if !self.cursor.exhausted {
            return Err(IndexRegionEvaluationError::IncompleteStagedWalk {
                evaluated: self.cursor.evaluated,
            });
        }
        let outputs = self
            .plans
            .into_iter()
            .map(finish_output)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(IndexRegionEvaluation {
            outputs,
            authority: self.authority,
        })
    }
}

/// Lexicographic cursor over **one write root's own** iteration domain.
///
/// Holds the position a span left off at, which is what makes a span boundary a
/// resumption between root points rather than inside one. One per root rather
/// than one per region, because a root's iteration space is the product of the
/// extents it declares and two roots partitioning one boundary into unequally
/// sized pieces declare different ones.
struct DomainWalk {
    dimensions: Vec<(VerifiedDimensionId, u64)>,
    extents: Vec<u64>,
    point: Vec<u64>,
    exhausted: bool,
}

impl DomainWalk {
    fn new(dimensions: Vec<(VerifiedDimensionId, u64)>) -> Self {
        let extents: Vec<u64> = dimensions.iter().map(|(_, extent)| *extent).collect();
        // One empty extent makes *this root's* space empty, which is the
        // degenerate partition member: it owns no element and must contribute
        // none, without emptying any sibling's walk. A rank-zero domain is the
        // one point of the empty tuple, which is what `advance_point`'s
        // immediate `false` on an empty slice already produced.
        let exhausted = extents.contains(&0);
        let point = vec![0_u64; extents.len()];
        Self {
            dimensions,
            extents,
            point,
            exhausted,
        }
    }

    fn point_count(&self) -> Option<u64> {
        if self.extents.contains(&0) {
            return Some(0);
        }
        self.extents
            .iter()
            .try_fold(1_u64, |total, extent| total.checked_mul(*extent))
    }

    fn advance(&mut self) {
        if !advance_point(&mut self.point, &self.extents) {
            self.exhausted = true;
        }
    }
}

/// Cursor over one region's root points, in plan then root then domain order.
///
/// Kept **settled** by construction and after every point walked: it stands
/// either on a root with a point left or past the last plan, never on an
/// exhausted root. That is what lets [`StagedIndexRegionEvaluation::is_exhausted`]
/// read a field instead of searching, and it is why a region whose every root is
/// empty — legal, and produced by a partition all of whose members are
/// degenerate — reports exhaustion before the first span rather than deadlocking
/// a caller looping until [`StagedIndexRegionEvaluation::evaluate_root_points`]
/// returns zero.
struct RootCursor {
    plan: usize,
    root: usize,
    evaluated: u64,
    exhausted: bool,
}

impl RootCursor {
    fn new(plans: &[OutputPlan<'_>]) -> Self {
        let mut cursor = Self {
            plan: 0,
            root: 0,
            evaluated: 0,
            exhausted: false,
        };
        cursor.settle(plans);
        cursor
    }

    /// Advances past exhausted roots and finished plans to the next root point.
    fn settle(&mut self, plans: &[OutputPlan<'_>]) {
        loop {
            let Some(plan) = plans.get(self.plan) else {
                self.exhausted = true;
                return;
            };
            match plan.roots.get(self.root) {
                Some(root) if !root.walk.exhausted => return,
                Some(_) => self.root = self.root.saturating_add(1),
                None => {
                    self.plan = self.plan.saturating_add(1);
                    self.root = 0;
                }
            }
        }
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

/// One write root of an output boundary, with its own iteration cursor.
///
/// The cursor is per root because the domain is: a write iterates any subset of
/// the region's parallel dimensions, so a root's point count is the product of
/// the extents *it* declares and two roots of one boundary need not agree on it.
struct OutputRoot<'a> {
    access: TensorAccessRef<'a>,
    value: VerifiedScalarValueId,
    walk: DomainWalk,
}

/// One output **boundary** and every root that writes it.
///
/// The buffer is per boundary rather than per root because an output several
/// roots partition is one tensor they fill between them. One buffer per root
/// would leave each root's copy filled only where that root wrote, which is a
/// tensor no part of the program ever produces.
struct OutputPlan<'a> {
    roots: Vec<OutputRoot<'a>>,
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
                .shape()
                .as_static()
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
    // Every plan is created from a root and only ever gains more, so an empty
    // root list is a fail-closed floor rather than a reachable state. The first
    // root in region order is the boundary's stable handle for a gap, not a
    // fault attribution; `IncompleteWrite` says why that survives roots of
    // unequal domains.
    let access = plan
        .roots
        .first()
        .ok_or(IndexRegionEvaluationError::MalformedRegion)?
        .access
        .id();
    let elements = plan
        .elements
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(IndexRegionEvaluationError::IncompleteWrite { access })?;
    Tensor::dense(plan.value_type, plan.shape.clone(), elements)
        .map_err(|source| IndexRegionEvaluationError::Value(Arc::new(source)))
}

impl<'a> RegionEvaluation<'a> {
    /// Walks up to `points` root points from where the cursor stands.
    ///
    /// The step budget starts here and nowhere else, so one span is one ask: the
    /// whole-region path calls this once with an unbounded point count and is
    /// held to exactly the number it always was.
    ///
    /// No parallel dimension goes unread by skipping the region's own dimension
    /// list: a parallel dimension a reachable read or value names is in that
    /// value's free dimensions, and verification requires an output root's free
    /// dimensions to lie inside its write domain, so every parallel dimension a
    /// verified region declares is in some root's domain and its extent is
    /// resolved — and refused when symbolic — where that root's walk is built.
    fn evaluate_span(
        &mut self,
        cursor: &mut RootCursor,
        plans: &mut [OutputPlan<'a>],
        points: u64,
    ) -> Result<u64, IndexRegionEvaluationError> {
        self.steps = 0;
        let mut walked = 0_u64;
        while walked < points && !cursor.exhausted {
            self.evaluate_root_point(cursor, plans)?;
            cursor.evaluated = cursor.evaluated.saturating_add(1);
            cursor.settle(plans);
            walked = walked.saturating_add(1);
        }
        Ok(walked)
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
                let extent = dimension.extent().as_static().ok_or_else(|| {
                    unsupported(UnsupportedRegionFeature::SymbolicDimensionExtent)
                })?;
                Ok((id, extent.get()))
            })
            .collect()
    }

    /// Plans one buffer per output boundary, carrying every root that writes it,
    /// each with a cursor over its own iteration domain.
    ///
    /// # Which partitioned regions this admits
    ///
    /// All of them, and the admitting boundary is a decision rather than a
    /// fallthrough. A write's domain is any subset of the region's parallel
    /// dimensions (`IndexBuildError::InvalidWriteDomain` now refuses the
    /// reduction half alone), so a root's point count is the product of the
    /// extents *it* iterates and two roots of one boundary need not agree on it
    /// — which is the whole of what a partition with unequally sized members
    /// needs. Giving each root its own [`DomainWalk`] reproduces every such
    /// partition exactly: the (root, point of that root's own domain) pairs are
    /// the elements, and evaluating all of them is evaluating the whole tensor.
    /// There is no partition shape this evaluator can see but not reproduce, so
    /// there is no [`UnsupportedRegionFeature`] to raise here — one would be a
    /// refusal nothing could ever trigger, which reads as a guarantee while
    /// checking nothing.
    ///
    /// What is left checked rather than assumed is the coverage. Disjointness
    /// and totality are the oracle's own per-element obligation against this
    /// shared buffer, independent of the verifier's joint proof:
    /// [`IndexRegionEvaluationError::DuplicateWrite`] fires when two roots' rectangles
    /// collide on one slot, and [`IndexRegionEvaluationError::IncompleteWrite`]
    /// when the roots together leave one unfilled. Neither can now fire for a
    /// valid region — a strict-subset root fires once per point of its own
    /// domain, so it writes its rectangle once, and a root with a zero extent
    /// writes nothing without emptying a sibling's walk.
    ///
    /// The two things counted here are counted over different populations, each
    /// over the one it is about. The retained-element budget counts each
    /// *boundary* once, because one boundary is what is retained; charging a
    /// partitioned output once per root would refuse a program on memory it
    /// never allocates. The walk is built per *root*, over that root's own
    /// extents, because a root's points are what it visits; giving it the
    /// region's parallel product would size it by points nothing walks, and the
    /// step budget — a running count of the work a span actually does — would
    /// then be spent on them.
    fn output_plans(&self) -> Result<Vec<OutputPlan<'a>>, IndexRegionEvaluationError> {
        let region = self.region;
        let mut retained = 0_usize;
        let mut plans: Vec<OutputPlan<'a>> = Vec::with_capacity(region.outputs().len());
        let mut planned: HashMap<VerifiedTensorId, usize> = HashMap::new();
        for output in region.outputs() {
            let access = region
                .access(output.access())
                .map_err(IndexRegionEvaluationError::Handle)?;
            if access.mode() != AccessMode::Write {
                return Err(IndexRegionEvaluationError::MalformedRegion);
            }
            // Built before the boundary's own budget so a symbolic extent is
            // reported as one wherever the root sits in region order.
            let root = OutputRoot {
                access,
                value: output.value(),
                walk: DomainWalk::new(self.domain(access.domain())?),
            };
            // Ordered by each boundary's first root, so a region no partition
            // touches produces exactly the plans, in exactly the order, that one
            // plan per root produced.
            if let Some(position) = planned.get(&access.tensor()) {
                plans
                    .get_mut(*position)
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?
                    .roots
                    .push(root);
                continue;
            }
            let tensor = region
                .tensor(access.tensor())
                .map_err(IndexRegionEvaluationError::Handle)?;
            let shape = tensor
                .shape()
                .as_static()
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
            planned.insert(access.tensor(), plans.len());
            plans.push(OutputPlan {
                roots: vec![root],
                value_type: tensor.value_type().clone(),
                shape,
                elements: vec![None; count],
            });
        }
        Ok(plans)
    }

    /// Evaluates the root the settled cursor stands on, at that root's point.
    ///
    /// The frame is seeded from the root's **own** domain and from nothing else,
    /// and that is exactly the environment its stored value needs: a coordinate
    /// may not name a dimension outside its access domain
    /// (`IndexBuildError::CoordinateOutsideAccessDomain`) and verification
    /// refuses a root whose value varies along a parallel dimension the write
    /// does not iterate (`IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain`),
    /// so no expression this reaches can ask for a dimension the frame lacks. A
    /// reduction binds its own dimensions in an inner frame, as it always did.
    ///
    /// One root point contributes exactly one element to the boundary's shared
    /// buffer, which is what makes a partitioned output whole at the end of the
    /// walk and what makes the two coverage checks below joint over the roots
    /// rather than per root.
    fn evaluate_root_point(
        &mut self,
        cursor: &RootCursor,
        plans: &mut [OutputPlan<'a>],
    ) -> Result<(), IndexRegionEvaluationError> {
        let plan = plans
            .get_mut(cursor.plan)
            .ok_or(IndexRegionEvaluationError::MalformedRegion)?;
        // Destructured so the root's own cursor stays mutable while the
        // boundary's shared buffer is written.
        let OutputPlan {
            roots,
            value_type,
            shape,
            elements,
        } = plan;
        let root = roots
            .get_mut(cursor.root)
            .ok_or(IndexRegionEvaluationError::MalformedRegion)?;
        let mut frame = Frame::default();
        for ((dimension, _), coordinate) in root.walk.dimensions.iter().zip(&root.walk.point) {
            frame.environment.insert(*dimension, *coordinate);
        }
        let value = self.value(&mut frame, root.value)?;
        if value.resolved_type() != &*value_type {
            return Err(IndexRegionEvaluationError::MalformedRegion);
        }
        let element = dense_element(&value)?;
        let offset = self.access_offset(&mut frame, root.access, shape)?;
        let access = root.access.id();
        let slot = elements
            .get_mut(offset)
            .ok_or(IndexRegionEvaluationError::CoordinateOutOfBounds { access })?;
        if slot.is_some() {
            return Err(IndexRegionEvaluationError::DuplicateWrite { access });
        }
        *slot = Some(element);
        // Advanced only on a written point, so a failed span leaves the cursor
        // on the point that failed rather than past it.
        root.walk.advance();
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
                    .div_mod_floor(constant_divisor(divisor)?)
                    .ok_or(IndexRegionEvaluationError::MalformedRegion)?
                    .0
            }
            IndexExprView::Modulo { dividend, divisor } => {
                self.expression(frame, dividend)?
                    .div_mod_floor(constant_divisor(divisor)?)
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
            .shape()
            .as_static()
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
                conformance: self.evaluator.conformance,
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
