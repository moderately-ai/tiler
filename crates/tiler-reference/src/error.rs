//! Every typed failure the reference implementation reports.
//!
//! Collected here because the crate root is a facade over the reference
//! boundary, and because the registry, value, operation, and evaluation
//! error vocabularies were previously separated by a thousand lines of the
//! evaluator that raises them.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::{
    AttributeFieldId, InputKey, OpKey, ProviderIdentity, RegistryError, ResolvedValueType,
    ValueConformanceRejection,
};
use tiler_ir::shape::{Shape, ShapeSymbol};

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

/// Why a staged contraction could not be planned.
///
/// Two failures with genuinely different causes, kept apart rather than merged.
/// [`Self::UnsupportedDeclaration`] says the governed signature declares a
/// contract this reference does not compute, which is a statement about the
/// *declaration* and is repaired by changing what is declared or what is
/// realized. [`Self::Operation`] says the operands, the structure, or the
/// requested slab were refused, which is a statement about this application.
/// Collapsing them would leave a caller unable to tell a moved contract from a
/// mismatched pair of tensors.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StagedContractionError {
    /// The governed contraction signature declares an unrealizable contract.
    UnsupportedDeclaration(UnsupportedContractionDeclaration),
    /// The operands, the structure, or the requested slab were refused.
    Operation(ReferenceOperationError),
}

impl fmt::Display for StagedContractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDeclaration(source) => write!(
                formatter,
                "a staged contraction cannot be planned against this declaration: {source}"
            ),
            Self::Operation(source) => {
                write!(formatter, "a staged contraction was refused: {source}")
            }
        }
    }
}

impl Error for StagedContractionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedDeclaration(source) => Some(source),
            Self::Operation(source) => Some(source),
        }
    }
}

/// Why the governed BF16 declarations cannot parameterize this reference.
///
/// The BF16 evaluators restate no format parameter. Every number the decode, the
/// rounding, and the encode need — the encoded width, the precision, the exponent
/// range and its bias, whether subnormals are members, and the canonical
/// arithmetic NaN payload — is read from the registered `tiler::bf16@1` descriptor
/// and from the family's own declared fact record. So a declaration this evaluator
/// cannot realize has to be refusable, and refusing *by what was read* is what
/// makes the refusal usable: a reader learns which declared term this reference
/// does not implement rather than which line of it panicked.
///
/// The refusal lands at registration. A reference that registered a capability
/// against a descriptor it could not decode would answer with a value set nobody
/// declared, and every later bit comparison would inherit that.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedBf16Declaration {
    /// The governed catalog describes no `tiler::bf16@1` scalar at all.
    MissingDescriptor,
    /// A required declared field was absent or carried the wrong canonical kind.
    MalformedFact {
        /// What was being read, named for a reader rather than by field ID.
        field: &'static str,
    },
    /// The descriptor does not expose an ordered finite value set to round onto.
    ///
    /// Carries `tiler::ulp-reference-gap@1`'s own refusal, because the rounding
    /// this reference performs is that metric's format rule applied to BF16's
    /// parameters, and a descriptor the metric rejects is one this cannot round to.
    IncompatibleFormat(tiler_ir::semantic::accuracy::UlpFormatError),
    /// The declared encoded width is not one this reference's element carrier holds.
    UnsupportedWidth {
        /// The declared encoded width in bits.
        width_bits: u32,
    },
    /// The declared exponent range and the encoded width do not describe each other.
    InconsistentExponentRange {
        /// Exponent width derived from the encoded width and the precision.
        exponent_bits: u32,
        /// Greatest finite exponent the descriptor's own fields fix.
        max_exponent: i32,
    },
    /// The descriptor overrides the exponent bias its exponent range implies.
    ///
    /// The decode below biases by the greatest finite exponent, which is the
    /// parameterization the `bfloat` format rule names. A descriptor stating a
    /// different bias describes a different encoding of the same value set, and
    /// decoding it with the derived bias would misread every normal operand.
    OverriddenExponentBias {
        /// The bias the descriptor declares.
        declared: i32,
        /// The bias the declared exponent range implies.
        derived: i32,
    },
    /// The descriptor declares no subnormals, which this family's semantics preserve.
    SubnormalsAbsent,
    /// The declared canonical arithmetic NaN payload is not a NaN encoding.
    ArithmeticNanPayloadIsNotNan {
        /// The declared payload.
        bits: u16,
    },
}

impl UnsupportedBf16Declaration {
    /// The stable diagnostic rule this refusal reports under.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::MissingDescriptor => "reference.bf16.missing-descriptor",
            Self::MalformedFact { .. } => "reference.bf16.malformed-fact",
            Self::IncompatibleFormat(_) => "reference.bf16.incompatible-format",
            Self::UnsupportedWidth { .. } => "reference.bf16.unsupported-width",
            Self::InconsistentExponentRange { .. } => "reference.bf16.inconsistent-exponent-range",
            Self::OverriddenExponentBias { .. } => "reference.bf16.overridden-exponent-bias",
            Self::SubnormalsAbsent => "reference.bf16.subnormals-absent",
            Self::ArithmeticNanPayloadIsNotNan { .. } => "reference.bf16.arithmetic-nan-not-nan",
        }
    }
}

impl fmt::Display for UnsupportedBf16Declaration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rule = self.rule();
        match self {
            Self::MissingDescriptor => write!(
                formatter,
                "{rule}: the governed catalog describes no tiler::bf16@1 scalar"
            ),
            Self::MalformedFact { field } => {
                write!(formatter, "{rule}: {field} is absent or malformed")
            }
            Self::IncompatibleFormat(source) => write!(
                formatter,
                "{rule}: the bf16 descriptor has no roundable finite value set: {source}"
            ),
            Self::UnsupportedWidth { width_bits } => write!(
                formatter,
                "{rule}: this reference carries a bf16 element in a 16-bit word, not {width_bits} bits"
            ),
            Self::InconsistentExponentRange {
                exponent_bits,
                max_exponent,
            } => write!(
                formatter,
                "{rule}: an exponent width of {exponent_bits} does not fix a greatest finite exponent of {max_exponent}"
            ),
            Self::OverriddenExponentBias { declared, derived } => write!(
                formatter,
                "{rule}: the descriptor declares exponent bias {declared} where its exponent range implies {derived}"
            ),
            Self::SubnormalsAbsent => write!(
                formatter,
                "{rule}: the bf16 arithmetic preserves subnormals, and the descriptor declares none"
            ),
            Self::ArithmeticNanPayloadIsNotNan { bits } => write!(
                formatter,
                "{rule}: the declared canonical arithmetic NaN payload {bits:#06x} is not a bf16 NaN encoding"
            ),
        }
    }
}

impl Error for UnsupportedBf16Declaration {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IncompatibleFormat(source) => Some(source),
            _ => None,
        }
    }
}

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
    /// The governed `tiler::bf16@1` declarations could not parameterize the BF16
    /// evaluators, so no BF16 capability was registered.
    ///
    /// Refused for the reason [`Self::UnsupportedContraction`] states: the whole
    /// BF16 value set this reference computes over is read from the registered
    /// descriptor, and one that cannot be read leaves nothing to be exact against.
    /// The resolved type is not carried because the variant names it.
    UnsupportedBf16 {
        /// The declared term this reference does not realize.
        source: UnsupportedBf16Declaration,
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
            Self::UnsupportedBf16 { source } => write!(
                formatter,
                "the governed tiler::bf16@1 declarations are unrealizable here: {source}"
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
            Self::UnsupportedBf16 { source, .. } => Some(source),
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
    /// The value does not conform to the obligations its resolved type states.
    ///
    /// Carried whole rather than flattened into
    /// [`Self::InvalidRepresentation`]: the rejection names the exact component
    /// ordinal, the canonical row-major logical index, and the stable
    /// invalid-input class, and a validator that reported only "invalid" would
    /// discard the three things a caller needs to fix it.
    Conformance(Box<ValueConformanceRejection>),
}

impl fmt::Display for ReferenceValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRepresentation => {
                formatter.write_str("invalid reference value representation")
            }
            Self::Conformance(rejection) => rejection.fmt(formatter),
        }
    }
}

impl Error for ReferenceValueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRepresentation => None,
            Self::Conformance(rejection) => Some(rejection.as_ref()),
        }
    }
}

impl From<ValueConformanceRejection> for ReferenceValueError {
    fn from(value: ValueConformanceRejection) -> Self {
        Self::Conformance(Box::new(value))
    }
}

/// Failure inside one exact reference implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceOperationError {
    /// Operands or attributes violated the registered capability contract.
    InvalidApplication,
    /// Shape arithmetic exceeded host limits.
    ShapeTooLarge,
    /// A certified enclosure could not decide a transcendental reference value.
    ///
    /// The fail-closed path of the exact transcendental references: the bracket
    /// straddles a rounding boundary, so which binary32 value the reference rounds
    /// to is *not established*. Refusing is the only honest answer — resolving it
    /// toward the nearer side would make a reference that cannot be wrong, and
    /// resolving it toward failure would reject a correct implementation.
    UndecidedTranscendentalReference,
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
    /// The conformance was resolved for a format this capability does not compute in.
    ///
    /// A [`crate::ReferenceNumericalConformance`]'s two subnormal dimensions name
    /// no format on their own, and the same resolution is a different behaviour
    /// over different value sets. A capability that applied a conformance stated
    /// about another arithmetic type would answer for a contract nothing declared
    /// about its own values, so it refuses instead.
    ConformanceSubject {
        /// The arithmetic type this capability computes in.
        capability: ArithmeticType,
        /// The arithmetic type the conformance was resolved for.
        stated: ArithmeticType,
    },
    /// A gather's index element named no coordinate of its gathered axis.
    ///
    /// A named variant rather than an [`Self::InvalidApplication`], and the
    /// distinction is the whole of what the gather family's bounds rule is worth.
    /// Every other refusal in this enum reports that an occurrence was malformed
    /// — a property of the *graph*, decidable before any element is read. This
    /// one reports that a well-formed occurrence was handed data outside the
    /// range its own semantics admit, which is the one obligation the semantic
    /// layer cannot discharge and which this evaluator is the named enforcement
    /// boundary for. Collapsing it into `InvalidApplication` would make an
    /// out-of-range token ID indistinguishable from a mis-declared operand, and
    /// the caller could not tell which of the two it had.
    ///
    /// It reports and never repairs: no coordinate is clamped to the axis and
    /// none is wrapped modulo its extent.
    GatherIndexOutOfBounds {
        /// Position of the offending element in the index operand, row-major.
        position: usize,
        /// The value it holds.
        value: u64,
        /// The gathered axis's extent, which it must stay below.
        extent: u64,
    },
    /// A window offset named a symbol this evaluation cannot authenticate.
    UndeclaredExtentSymbol {
        /// The symbol the offset named.
        symbol: Box<ShapeSymbol>,
    },
    /// A window offset named a binding kind with no authenticated value source.
    UnsupportedExtentBinding {
        /// The symbol the offset named.
        symbol: Box<ShapeSymbol>,
        /// The unsupported binding kind.
        kind: &'static str,
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
            Self::UndecidedTranscendentalReference => formatter.write_str(
                "the certified enclosure does not establish which value the transcendental \
                 reference rounds to",
            ),
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
            Self::ConformanceSubject { capability, stated } => write!(
                formatter,
                "reference capability computes in {} and was handed a conformance resolved for {}",
                capability.canonical_type_key(),
                stated.canonical_type_key()
            ),
            Self::GatherIndexOutOfBounds {
                position,
                value,
                extent,
            } => write!(
                formatter,
                "gather index element {position} holds {value} and the gathered axis has extent \
                 {extent}, so it names no coordinate; an out-of-range index is refused rather than \
                 clamped to the axis or wrapped modulo its extent"
            ),
            Self::UndeclaredExtentSymbol { symbol } => write!(
                formatter,
                "reference.extent.undeclared-symbol: {symbol} is not bound by this evaluation"
            ),
            Self::UnsupportedExtentBinding { symbol, kind } => write!(
                formatter,
                "reference.extent.unsupported-binding: {symbol} is a {kind} and this evaluator has no authenticated value source for that kind"
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
    /// A bound value does not conform to the obligations its type states.
    ///
    /// Distinct from [`Self::Value`], which attributes a registered
    /// representation validator's refusal to its provider. This one is the
    /// provenance-bound binding check: it names the interface key for a direct
    /// binding and carries the deterministic diagnostic coordinate.
    ValueConformance {
        /// Interface key of the refused binding, absent for a produced value.
        key: Option<InputKey>,
        /// The deterministic typed refusal.
        rejection: Box<ValueConformanceRejection>,
    },
    /// One produced value's conformance proof could not be composed.
    ///
    /// An invariant failure rather than invalid input: the producer's own
    /// verified semantics did not compose into a proof of its result, which
    /// means this build's composition rule and its operation definitions
    /// disagree.
    ValueConformanceComposition {
        /// Governed operation family whose result could not be proved.
        operation: OpKey,
        /// The composition failure, rendered by its own type.
        detail: String,
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
    /// A value's shape names a declared `ShapeEnv` symbol.
    ///
    /// Reference evaluation compares a produced tensor against the shape the
    /// program declares, and a symbolic extent declares no single one. Resolving
    /// it through the environment would make the oracle answer for a program
    /// with concrete extents nobody wrote, so the evaluator refuses instead —
    /// the same line the sourced vocabulary draws between graph identity and
    /// specialized identity.
    SymbolicShape,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputCount { expected, actual } => {
                write!(formatter, "expected {expected} inputs, received {actual}")
            }
            Self::ValueConformance { key, rejection } => match key {
                Some(key) => write!(formatter, "input {}: {rejection}", key.as_str()),
                None => rejection.fmt(formatter),
            },
            Self::ValueConformanceComposition { operation, detail } => write!(
                formatter,
                "operation {operation} produced no conformance proof for its result: {detail}",
            ),
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
            Self::SymbolicShape => {
                formatter.write_str("a value's shape names a declared shape-environment symbol")
            }
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
            Self::ValueConformance { rejection, .. } => Some(rejection.as_ref()),
            _ => None,
        }
    }
}

/// Reports a dense result-tensor construction failure in the operation vocabulary.
///
/// [`Tensor::dense`] answers in [`EvaluationError`] because `tensor` owns what a
/// reference value *is*; a registered implementation answers in
/// [`ReferenceOperationError`]. Every family that builds its result densely
/// crosses that boundary, and each crossed it by discarding the cause and naming
/// [`ReferenceOperationError::ShapeTooLarge`] — a name that is wrong wherever the
/// call site already established the element count, which is most of them, and
/// that drops the limit and the size that bound an exceeded budget.
///
/// The mapping is by the quantity the constructor rejected rather than by the
/// constructor the failure came from:
///
/// | cause | reported as |
/// | --- | --- |
/// | [`EvaluationError::ShapeTooLarge`] | [`ReferenceOperationError::ShapeTooLarge`] |
/// | [`ReferenceResource::TensorElements`] over its limit | [`ReferenceOperationError::OutputElementsExceeded`] |
/// | [`ReferenceResource::TensorBytes`] over its limit | [`ReferenceOperationError::OutputResourceExceeded`] |
/// | [`EvaluationError::ElementCount`] | [`ReferenceOperationError::InvalidApplication`] |
///
/// The two resource rows are the variants `preflight_f32_output` already raises
/// for those same two quantities against those same two constants, so a family
/// that preflights its output and one that does not now refuse an over-budget
/// result under one name and carrying one pair of numbers.
///
/// The domain is [`Tensor::dense`] alone, which is why the resources it cannot
/// bound are named rather than mapped. [`Tensor::compound`] additionally bounds
/// component count and recursive depth, and this vocabulary has no name for a
/// depth, so a compound construction is reported at its own call sites.
///
/// **What any of it can actually report today.** Fourteen call sites apply this
/// mapping, and every one of them is defensive. Every one takes the result
/// shape's element count successfully before constructing, so
/// [`EvaluationError::ShapeTooLarge`] — the name all fourteen used to report — is
/// the one cause that cannot arrive at any of them. `ElementCount` needs an
/// implementation whose payload disagrees with the result shape it declared for
/// itself. The retained-byte bound is unreachable by arithmetic rather than by
/// argument: every site builds elements of at most four bytes, so a payload can
/// weigh at most `4 * MAX_REFERENCE_TENSOR_ELEMENTS` bytes, which is exactly
/// `MAX_REFERENCE_TENSOR_BYTES` and therefore never over it, and the element
/// bound is tested first regardless. That leaves the element bound, which
/// `preflight_f32_output` refuses ahead of the constructor at every site whose
/// result could reach it — the two reductions, the two split-reduction passes,
/// the contraction fold, and `structural::gather` — and which the remaining
/// families cannot exceed because their result shape is an operand's, already
/// bounded when that operand was built. `gather` is the site where that bound is
/// genuinely reachable, because its broadcast caller replicates and so produces a
/// result larger than the operand it reads; it preflights the count rather than
/// materializing more than `MAX_REFERENCE_TENSOR_ELEMENTS` elements to be told
/// no. Nothing therefore drives an over-budget dense construction through a
/// family any more, which is why the mapping is tested against the constructor
/// directly.
///
/// [`Tensor::dense`]: crate::Tensor::dense
/// [`Tensor::compound`]: crate::Tensor::compound
pub(crate) fn dense_result_error(source: &EvaluationError) -> ReferenceOperationError {
    match *source {
        EvaluationError::ShapeTooLarge => ReferenceOperationError::ShapeTooLarge,
        EvaluationError::ResourceExceeded {
            resource,
            limit,
            actual,
        } => match resource {
            ReferenceResource::TensorElements => {
                ReferenceOperationError::OutputElementsExceeded { limit, actual }
            }
            ReferenceResource::TensorBytes => {
                ReferenceOperationError::OutputResourceExceeded { limit, actual }
            }
            // A dense construction bounds exactly the two resources above. The
            // rest of this vocabulary bounds one element, a compound value, or a
            // whole evaluation, and no dense construction reaches any of them.
            ReferenceResource::ElementBytes
            | ReferenceResource::Components
            | ReferenceResource::ComponentDepth
            | ReferenceResource::EvaluationBytes
            | ReferenceResource::EvaluationElements
            | ReferenceResource::EvaluationComponents => {
                ReferenceOperationError::InvalidApplication
            }
        },
        // Everything below is invalid state rather than an exceeded bound, which
        // is why one name answers for all of it.
        //
        // `ElementCount` says the implementation produced a payload whose length
        // disagrees with the result shape it declared for itself. Both facts are
        // its own, so the disagreement is reported — as the structural and
        // contraction families report every other recompute disagreement —
        // instead of being resolved in favour of either side.
        //
        // A dense construction produces none of the rest. They are named one by
        // one rather than caught by a wildcard, so that a new evaluation cause is
        // a build error here instead of a silent return to the collapse this
        // replaced.
        EvaluationError::ElementCount { .. }
        | EvaluationError::InputCount { .. }
        | EvaluationError::InputKey { .. }
        | EvaluationError::InputShape { .. }
        | EvaluationError::InputType { .. }
        | EvaluationError::EmptyFloatBits
        | EvaluationError::DuplicateComponentRole { .. }
        | EvaluationError::ReferenceRegistry(_)
        | EvaluationError::MissingCapability { .. }
        | EvaluationError::MissingValueCapability { .. }
        | EvaluationError::SemanticAuthority { .. }
        | EvaluationError::SemanticValueAuthority { .. }
        | EvaluationError::CapabilityAuthorityMismatch { .. }
        | EvaluationError::ValueCapabilityAuthorityMismatch { .. }
        | EvaluationError::Value { .. }
        | EvaluationError::Operation { .. }
        | EvaluationError::ResultShape { .. }
        | EvaluationError::ResultType { .. }
        | EvaluationError::ValueConformance { .. }
        | EvaluationError::ValueConformanceComposition { .. }
        | EvaluationError::MalformedProgram
        | EvaluationError::SymbolicShape => ReferenceOperationError::InvalidApplication,
    }
}
