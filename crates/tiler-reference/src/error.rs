//! Every typed failure the reference implementation reports.
//!
//! Collected here because the crate root is a facade over the reference
//! boundary, and because the registry, value, operation, and evaluation
//! error vocabularies were previously separated by a thousand lines of the
//! evaluator that raises them.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::semantic::{
    AttributeFieldId, InputKey, OpKey, ProviderIdentity, RegistryError, ResolvedValueType,
};
use tiler_ir::shape::Shape;

use super::{ReferenceCapabilityRevision, ReferenceComponentRole, ReferenceSignature};

/// Governed resource in a reference-capability registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceRegistryResource {
    /// Operand types retained by one exact signature.
    SignatureOperands,
    /// Result types retained by one exact signature.
    SignatureResults,
    /// Aggregate operation and value-representation capabilities.
    Capabilities,
    /// Canonical registry identity bytes.
    CanonicalIdentityBytes,
}

/// Why a declared contraction numerical signature cannot be realized here.
///
/// The reference reads `tiler::strict-tensor-contraction-f32@1`'s fourteen-field
/// signature instead of restating it, so a declaration naming a contract this
/// evaluator does not compute has to be refusable. Refusing *by field* is what
/// makes the refusal usable: a reader learns which declared term moved, and the
/// public [`tiler_ir::semantic`] field-ID constants name it.
///
/// A declaration this evaluator would over-satisfy is refused on the same
/// footing as one it would under-satisfy. Evaluating a weaker contract strictly
/// and reporting bit equality would assert a guarantee the contract never made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedContractionDeclaration {
    /// The facts were not a record of the governed field set.
    MalformedRecord,
    /// One field declared a value this reference does not realize.
    UnrealizableFact {
        /// The contraction fact field, by its stable schema-local ID.
        field: AttributeFieldId,
    },
}

impl UnsupportedContractionDeclaration {
    /// Names one field's declared value as unrealizable.
    #[must_use]
    pub const fn unrealizable(field: AttributeFieldId) -> Self {
        Self::UnrealizableFact { field }
    }

    /// The stable diagnostic rule this refusal reports under.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::MalformedRecord => "reference.contraction.malformed-signature",
            Self::UnrealizableFact { .. } => "reference.contraction.unrealizable-fact",
        }
    }
}

impl fmt::Display for UnsupportedContractionDeclaration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRecord => write!(
                formatter,
                "{}: the declared contraction signature is not the governed fact record",
                self.rule()
            ),
            Self::UnrealizableFact { field } => write!(
                formatter,
                "{}: contraction fact field {field} declares a contract this reference does not compute",
                self.rule()
            ),
        }
    }
}

impl Error for UnsupportedContractionDeclaration {}

/// Failure to construct or extend a reference registry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceRegistryError {
    /// Capability revision zero is reserved.
    ZeroCapabilityRevision,
    /// No reference capability was registered.
    EmptyRegistry,
    /// A provider transaction contributed nothing.
    ProviderRegisteredNothing {
        /// Provider which registered nothing.
        provider: ProviderIdentity,
    },
    /// Two registrations claimed one exact operation/signature pair.
    DuplicateCapability {
        /// Colliding semantic operation.
        operation: OpKey,
        /// Colliding resolved signature.
        signature: ReferenceSignature,
    },
    /// Two registrations claimed one exact value-representation contract.
    DuplicateValueCapability {
        /// Colliding resolved semantic type.
        resolved_type: ResolvedValueType,
    },
    /// The selected semantic registry could not be constructed.
    SemanticRegistry(Arc<RegistryError>),
    /// An operation capability lacked complete semantic authority.
    SemanticAuthority {
        /// Operation being registered.
        operation: OpKey,
        /// Semantic authority failure.
        source: Arc<RegistryError>,
    },
    /// A value validator lacked complete semantic authority.
    SemanticValueAuthority {
        /// Resolved value type being registered.
        resolved_type: ResolvedValueType,
        /// Semantic authority failure.
        source: Arc<RegistryError>,
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
    /// An operation's declared normative signature is one this reference cannot
    /// compute, so no capability was registered for it.
    ///
    /// The registry is the right place to fail: a reference that registered a
    /// capability anyway would answer for a contract it does not implement, and
    /// every later bit comparison would inherit that.
    UnsupportedContraction {
        /// Operation whose declared signature was refused.
        operation: OpKey,
        /// The declared term this reference does not realize.
        source: UnsupportedContractionDeclaration,
    },
}

impl fmt::Display for ReferenceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapabilityRevision => {
                formatter.write_str("reference capability revision zero is reserved")
            }
            Self::EmptyRegistry => formatter.write_str("reference capability registry is empty"),
            Self::ProviderRegisteredNothing { provider } => {
                write!(
                    formatter,
                    "reference provider {provider} registered nothing"
                )
            }
            Self::DuplicateCapability { operation, .. } => {
                write!(formatter, "duplicate reference capability for {operation}")
            }
            Self::DuplicateValueCapability { resolved_type } => write!(
                formatter,
                "duplicate reference value capability for {resolved_type:?}"
            ),
            Self::SemanticRegistry(source) => {
                write!(formatter, "semantic registry construction failed: {source}")
            }
            Self::SemanticAuthority { operation, source } => write!(
                formatter,
                "semantic authority for reference operation {operation} failed: {source}"
            ),
            Self::SemanticValueAuthority { source, .. } => {
                write!(
                    formatter,
                    "semantic authority for reference value failed: {source}"
                )
            }
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "reference registry resource {resource:?} has size {actual}, exceeding {limit}"
            ),
            Self::UnsupportedContraction { operation, source } => write!(
                formatter,
                "reference operation {operation} declares an unrealizable signature: {source}"
            ),
        }
    }
}

impl Error for ReferenceRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SemanticRegistry(source)
            | Self::SemanticAuthority { source, .. }
            | Self::SemanticValueAuthority { source, .. } => Some(source.as_ref()),
            Self::UnsupportedContraction { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Failure to validate a reference tensor representation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceValueError {
    /// The payload does not implement the registered resolved type.
    InvalidRepresentation,
}

impl fmt::Display for ReferenceValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRepresentation => {
                formatter.write_str("invalid reference value representation")
            }
        }
    }
}

impl Error for ReferenceValueError {}

/// Failure inside one exact reference implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceOperationError {
    /// Operands or attributes violated the registered capability contract.
    InvalidApplication,
    /// Shape arithmetic exceeded host limits.
    ShapeTooLarge,
    /// The callback produced the wrong number of ordered results.
    ResultCount {
        /// Required result count.
        expected: usize,
        /// Produced result count.
        actual: usize,
    },
    /// One callback output exceeded the governed logical element bound.
    OutputElementsExceeded {
        /// Active logical element limit.
        limit: usize,
        /// First rejected element count.
        actual: usize,
    },
    /// Callback outputs exceeded the governed aggregate component bound.
    OutputComponentsExceeded {
        /// Active recursive component limit.
        limit: usize,
        /// First rejected aggregate component count.
        actual: usize,
    },
    /// Aggregate callback output exceeded the host-owned writer budget.
    OutputResourceExceeded {
        /// Active byte limit.
        limit: usize,
        /// First rejected aggregate size.
        actual: usize,
    },
    /// The work an implementation would step through exceeded its governed bound.
    ///
    /// Separate from the output bounds above because it bounds a different thing.
    /// A contraction's fold walks `output_count * contracted_count`
    /// multiply-accumulate steps — larger than either operand, and bounded by
    /// neither the stored-element nor the byte limit the operands and the output
    /// already passed. Bounding what a result *retains* does not bound the work
    /// that produces it.
    IterationStepsExceeded {
        /// Active iteration-step limit.
        limit: usize,
        /// First rejected step count, saturated at `usize::MAX` when the count
        /// exceeds host arithmetic. Saturation only under-reports, so a step
        /// count too large to name is still refused.
        actual: usize,
    },
}

impl fmt::Display for ReferenceOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidApplication => {
                formatter.write_str("invalid reference operation application")
            }
            Self::ShapeTooLarge => {
                formatter.write_str("reference operation shape exceeds host limits")
            }
            Self::ResultCount { expected, actual } => {
                write!(
                    formatter,
                    "reference operation produced {actual} results, expected {expected}"
                )
            }
            Self::OutputElementsExceeded { limit, actual } => write!(
                formatter,
                "reference operation output has {actual} elements, exceeding {limit}"
            ),
            Self::OutputComponentsExceeded { limit, actual } => write!(
                formatter,
                "reference operation output has {actual} components, exceeding {limit}"
            ),
            Self::OutputResourceExceeded { limit, actual } => write!(
                formatter,
                "reference operation output retained {actual} bytes, exceeding {limit}"
            ),
            Self::IterationStepsExceeded { limit, actual } => write!(
                formatter,
                "reference operation iteration space has {actual} steps, exceeding {limit}"
            ),
        }
    }
}

impl Error for ReferenceOperationError {}

/// Governed resource in one host reference value or evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceResource {
    /// Bytes in one exact dense element.
    ElementBytes,
    /// Logical elements in one dense tensor.
    TensorElements,
    /// Aggregate exact payload bytes in one tensor or output set.
    TensorBytes,
    /// Direct components in one compound tensor.
    Components,
    /// Recursive compound-tensor depth.
    ComponentDepth,
    /// Aggregate retained payload bytes across one evaluation.
    EvaluationBytes,
    /// Aggregate logical and component tensor elements across one evaluation.
    EvaluationElements,
    /// Aggregate recursive compound components across one evaluation.
    EvaluationComponents,
}

/// A typed reference-evaluation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EvaluationError {
    /// The caller supplied the wrong number of ordered input bindings.
    InputCount {
        /// Declared program input count.
        expected: usize,
        /// Supplied binding count.
        actual: usize,
    },
    /// A binding key disagreed with the ordered semantic interface.
    InputKey {
        /// Position in the ordered input interface.
        input_index: usize,
        /// Declared key at that position.
        expected: InputKey,
        /// Supplied key at that position.
        actual: InputKey,
    },
    /// An input shape disagreed with its verified declaration.
    InputShape {
        /// Stable key identifying the input.
        key: InputKey,
        /// Statically declared shape.
        expected: Shape,
        /// Supplied tensor shape.
        actual: Shape,
    },
    /// An input resolved type disagreed with its verified declaration.
    InputType {
        /// Stable input key.
        key: InputKey,
        /// Declared resolved type.
        expected: Arc<ResolvedValueType>,
        /// Supplied resolved type.
        actual: Arc<ResolvedValueType>,
    },
    /// A tensor payload length disagreed with its shape.
    ElementCount {
        /// Element count implied by the shape.
        expected: usize,
        /// Supplied payload element count.
        actual: usize,
    },
    /// Shape arithmetic exceeded host limits.
    ShapeTooLarge,
    /// Exact floating-point construction supplied no bits.
    EmptyFloatBits,
    /// A bounded reference resource exceeded its active limit.
    ResourceExceeded {
        /// Bounded resource.
        resource: ReferenceResource,
        /// Active limit.
        limit: usize,
        /// First rejected size.
        actual: usize,
    },
    /// A compound tensor repeated one schema-local role.
    DuplicateComponentRole {
        /// Repeated role.
        role: ReferenceComponentRole,
    },
    /// Reference registry construction failed while forming an exact signature.
    ReferenceRegistry(Arc<ReferenceRegistryError>),
    /// The frozen registry has no executable oracle for an exact semantic signature.
    MissingCapability {
        /// Semantically valid operation lacking an oracle.
        operation: OpKey,
        /// Exact operand/result signature lacking an oracle.
        signature: Arc<ReferenceSignature>,
    },
    /// No validator exists for one exact resolved reference value type.
    MissingValueCapability {
        /// Unsupported resolved type.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// Program semantic authority could not be projected for an operation.
    SemanticAuthority {
        /// Operation whose authority projection failed.
        operation: OpKey,
        /// Typed semantic registry cause.
        source: Arc<RegistryError>,
    },
    /// Program semantic authority could not be projected for a value type.
    SemanticValueAuthority {
        /// Resolved value type whose projection failed.
        resolved_type: Arc<ResolvedValueType>,
        /// Typed semantic registry cause.
        source: Arc<RegistryError>,
    },
    /// An operation capability was built for different reached semantic authority.
    CapabilityAuthorityMismatch {
        /// Operation being resolved.
        operation: OpKey,
        /// Reference provider whose claim did not match.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting reference implementation revision.
        capability_revision: ReferenceCapabilityRevision,
    },
    /// A value validator was built for different reached semantic authority.
    ValueCapabilityAuthorityMismatch {
        /// Value type being validated.
        resolved_type: Arc<ResolvedValueType>,
        /// Reference provider whose claim did not match.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting validator revision.
        capability_revision: ReferenceCapabilityRevision,
    },
    /// A selected value validator rejected a tensor representation.
    Value {
        /// Exact resolved type.
        resolved_type: Arc<ResolvedValueType>,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting validator revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Typed validation cause.
        source: ReferenceValueError,
    },
    /// A resolved reference capability rejected execution.
    Operation {
        /// Operation whose capability failed.
        operation: OpKey,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting implementation revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Typed implementation failure.
        source: ReferenceOperationError,
    },
    /// A provider produced a result with the wrong shape.
    ResultShape {
        /// Operation whose result failed validation.
        operation: OpKey,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting implementation revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Ordered result index.
        result_index: usize,
        /// Declared shape.
        expected: Arc<Shape>,
        /// Produced shape.
        actual: Arc<Shape>,
    },
    /// A provider produced a result with the wrong resolved type.
    ResultType {
        /// Operation whose result failed validation.
        operation: OpKey,
        /// Selected reference provider.
        provider: Arc<ProviderIdentity>,
        /// Exact output-affecting implementation revision.
        capability_revision: ReferenceCapabilityRevision,
        /// Ordered result index.
        result_index: usize,
        /// Declared type.
        expected: Arc<ResolvedValueType>,
        /// Produced type.
        actual: Arc<ResolvedValueType>,
    },
    /// An internally malformed verified program reached the evaluator.
    MalformedProgram,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputCount { expected, actual } => {
                write!(formatter, "expected {expected} inputs, received {actual}")
            }
            Self::InputKey {
                input_index,
                expected,
                actual,
            } => write!(
                formatter,
                "input {input_index} has key {:?}, expected {:?}",
                actual.as_str(),
                expected.as_str()
            ),
            Self::InputShape {
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "input {:?} has shape {actual:?}, expected {expected:?}",
                key.as_str()
            ),
            Self::InputType { key, .. } => write!(
                formatter,
                "input {:?} has the wrong resolved reference type",
                key.as_str()
            ),
            Self::ElementCount { expected, actual } => {
                write!(
                    formatter,
                    "tensor has {actual} elements, expected {expected}"
                )
            }
            Self::ShapeTooLarge => formatter.write_str("tensor shape exceeds host limits"),
            Self::EmptyFloatBits => {
                formatter.write_str("exact floating-point reference bits are empty")
            }
            Self::ResourceExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "reference resource {resource:?} has size {actual}, exceeding {limit}"
            ),
            Self::DuplicateComponentRole { role } => {
                write!(
                    formatter,
                    "duplicate reference component role {}",
                    role.get()
                )
            }
            Self::ReferenceRegistry(source) => {
                write!(formatter, "reference registry failure: {source}")
            }
            Self::MalformedProgram => formatter.write_str("verified semantic program is malformed"),
            _ => self.fmt_capability_error(formatter),
        }
    }
}

impl EvaluationError {
    fn fmt_capability_error(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCapability { operation, .. } => write!(
                formatter,
                "no reference capability for semantic operation {operation} and exact resolved signature"
            ),
            Self::MissingValueCapability { .. } => {
                formatter.write_str("no reference value validator for exact resolved type")
            }
            Self::SemanticAuthority { operation, source } => write!(
                formatter,
                "semantic authority for {operation} could not be projected: {source}"
            ),
            Self::SemanticValueAuthority { source, .. } => write!(
                formatter,
                "semantic value authority could not be projected: {source}"
            ),
            Self::CapabilityAuthorityMismatch {
                operation,
                provider,
                capability_revision,
            } => write!(
                formatter,
                "reference provider {provider} capability revision {} does not implement reached authority for {operation}",
                capability_revision.get()
            ),
            Self::ValueCapabilityAuthorityMismatch {
                provider,
                capability_revision,
                ..
            } => write!(
                formatter,
                "reference provider {provider} validator revision {} does not implement reached value authority",
                capability_revision.get()
            ),
            Self::Value {
                provider,
                capability_revision,
                source,
                ..
            } => write!(
                formatter,
                "reference value validator revision {} from {provider} failed: {source}",
                capability_revision.get()
            ),
            Self::Operation {
                operation,
                provider,
                capability_revision,
                source,
            } => write!(
                formatter,
                "reference capability revision {} from {provider} for {operation} failed: {source}",
                capability_revision.get()
            ),
            Self::ResultShape {
                operation,
                provider,
                result_index,
                ..
            } => write!(
                formatter,
                "reference provider {provider} produced invalid shape for result {result_index} of {operation}"
            ),
            Self::ResultType {
                operation,
                provider,
                result_index,
                ..
            } => write!(
                formatter,
                "reference provider {provider} produced invalid type for result {result_index} of {operation}"
            ),
            _ => unreachable!("only capability errors use this formatter"),
        }
    }
}

impl Error for EvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReferenceRegistry(source) => Some(source),
            Self::SemanticAuthority { source, .. }
            | Self::SemanticValueAuthority { source, .. } => Some(source.as_ref()),
            Self::Value { source, .. } => Some(source),
            Self::Operation { source, .. } => Some(source),
            _ => None,
        }
    }
}
