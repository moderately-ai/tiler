//! Resolved-value binding conformance: the contract, the scan, and the evidence.
//!
//! # What this module decides, and what it deliberately does not
//!
//! A semantic operation precondition is a property of one *occurrence* — its
//! subject is an operand of a particular apply, and its authority is the
//! operation definition. A directly bound program input has no such occurrence:
//! nothing produced it inside the graph, so no operation predicate can speak
//! about its bytes. What governs it instead is its **type**. An admitted
//! [`ResolvedValueType`] states its scheme, its ordered component declarations,
//! each component's own resolved type, each component's shape relation and
//! parameter map, and the value domains its governed contract fields name. This
//! module derives that complete obligation set from the type and checks a bound
//! value against it.
//!
//! Three separations are load-bearing and none of them may be collapsed:
//!
//! - **Logical conformance is not physical canonicality.** The scan visits
//!   logical elements only. Padding, alignment, bit order, and the unused tail
//!   bits of a packed carrier are not part of the logical value and are never
//!   read here; their owner is physical representation validation.
//! - **Semantic invalidity is not inapplicability.** A value that does not
//!   implement its declared type is invalid input, reported by exact name. It
//!   never selects another interpretation, another scheme, or another plan.
//! - **Evidence is bound to provenance, never to a pointer or a slot.** Two
//!   values with identical bytes at identical addresses are two subjects when
//!   their origin, stability, or route differ, and a proof of one does not
//!   authorize the other.
//!
//! # The three proof paths, and the one vocabulary
//!
//! A directly bound value is proved by [`conform_bound_value`], which scans its
//! authoritative logical view. A value produced inside the graph is proved by
//! [`compose_produced_conformance`], which composes the producer's verified
//! semantics — its discharged preconditions and the conformance of the operands
//! it carries through — and reads no payload at all. Both mint the same
//! [`ValueConformanceEvidence`], so every consumer asks one question regardless
//! of how the answer was reached, and a complete proof of the same subject is
//! reused rather than recomputed.
//!
//! # Where the validator identity lives
//!
//! [`ConformanceValidatorIdentity`] is static, versioned, and folded into every
//! evidence encoding, so a validator whose meaning changes invalidates the
//! proofs taken under the old one. It is deliberately *not* folded into static
//! artifact identity: no artifact selects a validator today, and adding a field
//! an artifact producer cannot fill is the producer-less placeholder this
//! repository has repeatedly had to retract. The dynamic half — value version,
//! coherence epoch, and the bytes themselves — is execution-scoped by
//! construction and belongs only here.

use std::error::Error;
use std::fmt;
use std::num::NonZeroU32;
use std::sync::Arc;

use crate::identity::{push_len, push_slice};
use crate::shape::Shape;

use super::handles::ValueId;
use super::interface::InputKey;
use super::operation::{OpKey, OperationRef};
use super::precondition::{
    SemanticInvalidInputCode, SemanticLogicalView, SemanticPreconditionObligationIdentity,
    SemanticPreconditionStatus, SemanticPredicateIdentity, positive_normal_scalar_predicate,
};
use super::quantization::{
    ENCODED_NUMERIC_CODE_MAX, ENCODED_NUMERIC_CODE_MIN, ENCODED_NUMERIC_SCALE_DOMAIN,
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, U4, U8,
    assemble_strict_affine_op, quantize_strict_affine_op, strict_affine_scheme,
};
use super::registry::F32;
use super::types::{
    AttributeFieldId, CanonicalIntegerWidth, CanonicalValueView, EncodedComponentRole,
    EncodedComponentShape, EncodedNumericContract, ParameterIndexMap, QuantSchemeKey,
    ResolvedValueType, TypeKey,
};

/// Domain separator of one canonical value-conformance evidence encoding.
const EVIDENCE_DOMAIN: &[u8] = b"tiler.value-conformance-evidence.v1\0";

/// The declared scale domain this validator has an evaluator for.
const POSITIVE_NORMAL_F32_DOMAIN: &str = "positive-normal-f32";

/// Logical elements one conformance scan is authorized to visit for one value.
///
/// A resource shortfall is refused by name before any route commits rather than
/// discovered part-way through a scan, so the bound is stated here and checked
/// against the derived obligation before the first read.
pub const MAX_CONFORMANCE_SCAN_ELEMENTS: u64 = 64 * 1024 * 1024;

/// Stable versioned identity of one resolved-value conformance validator.
///
/// Static and identity-bearing: it is folded into every evidence encoding, so
/// evidence taken under one revision cannot authorize a subject under another.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConformanceValidatorIdentity {
    key: TypeKey,
    revision: NonZeroU32,
}

impl ConformanceValidatorIdentity {
    /// Creates a validator identity from its governed key and output-affecting revision.
    #[must_use]
    pub const fn new(key: TypeKey, revision: NonZeroU32) -> Self {
        Self { key, revision }
    }

    /// Returns the governed schema key.
    #[must_use]
    pub const fn key(&self) -> &TypeKey {
        &self.key
    }

    /// Returns the output-affecting revision.
    #[must_use]
    pub const fn revision(&self) -> NonZeroU32 {
        self.revision
    }

    fn encode(&self, output: &mut Vec<u8>) {
        self.key.encode(output);
        output.extend_from_slice(&self.revision.get().to_be_bytes());
    }
}

impl fmt::Display for ConformanceValidatorIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.key, self.revision)
    }
}

/// Returns the governed resolved-value binding-conformance validator.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its grammar.
#[must_use]
pub fn standard_binding_validator() -> ConformanceValidatorIdentity {
    ConformanceValidatorIdentity::new(
        TypeKey::new("tiler", "resolved-value-binding-conformance", 1)
            .expect("the governed conformance validator key is valid"),
        NonZeroU32::new(1).expect("one is nonzero"),
    )
}

/// The governed value domain one component's logical elements must lie in.
///
/// Derived from the type's own contract fields rather than restated per
/// consumer: a scheme that narrows its code range or its scale domain narrows
/// every conformance check that reads it, with nothing else to update.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComponentValueDomain {
    /// Every logical element is an unsigned code in an inclusive range.
    UnsignedCodeRange {
        /// Inclusive minimum admitted code.
        minimum: u8,
        /// Inclusive maximum admitted code.
        maximum: u8,
    },
    /// Every logical element is a strictly positive normal binary32 value.
    PositiveNormalF32,
}

impl ComponentValueDomain {
    const fn tag(self) -> u8 {
        match self {
            Self::UnsignedCodeRange { .. } => 0x01,
            Self::PositiveNormalF32 => 0x02,
        }
    }

    fn encode(self, output: &mut Vec<u8>) {
        output.push(self.tag());
        match self {
            Self::UnsignedCodeRange { minimum, maximum } => {
                output.push(minimum);
                output.push(maximum);
            }
            Self::PositiveNormalF32 => {}
        }
    }
}

/// One component obligation derived from an admitted resolved value type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentConformanceObligation {
    role: EncodedComponentRole,
    resolved_type: ResolvedValueType,
    shape: Shape,
    map: Option<ParameterIndexMap>,
    domain: ComponentValueDomain,
}

impl ComponentConformanceObligation {
    /// Returns the stable semantic role this obligation is stated against.
    #[must_use]
    pub const fn role(&self) -> EncodedComponentRole {
        self.role
    }

    /// Returns the component's complete resolved value type.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns the exact logical component shape the declaration derives.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns the parameter index map, absent when the component carries the logical shape.
    #[must_use]
    pub const fn parameter_map(&self) -> Option<&ParameterIndexMap> {
        self.map.as_ref()
    }

    /// Returns the governed value domain every logical element must lie in.
    #[must_use]
    pub const fn value_domain(&self) -> ComponentValueDomain {
        self.domain
    }

    fn encode(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.role.get().to_be_bytes());
        self.resolved_type.encode(output);
        encode_shape(&self.shape, output);
        match &self.map {
            None => output.push(0x00),
            Some(map) => {
                output.push(0x01);
                // Only the per-tensor form reaches an obligation: `derive`
                // refuses every other map by name, so the tag below cannot
                // stand for a form this validator has not admitted.
                debug_assert_eq!(map, &ParameterIndexMap::per_tensor());
                output.push(0x01);
            }
        }
        self.domain.encode(output);
    }
}

/// The complete conformance obligation one admitted resolved value type states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedValueConformanceContract {
    resolved_type: ResolvedValueType,
    shape: Shape,
    components: Vec<ComponentConformanceObligation>,
}

impl ResolvedValueConformanceContract {
    /// Derives the complete obligation set for one admitted type and logical shape.
    ///
    /// Every structural obligation — ordered roles, component types, component
    /// shapes, and parameter maps — comes from the type's own declarations, so a
    /// scheme that declares a different role set is checked without changing
    /// this function. Only the *value domains* are scheme knowledge, and a
    /// scheme without an admitted domain authority is refused by exact name
    /// rather than checked structurally and then trusted.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedValueRepresentation`] naming the exact type,
    /// scheme, component type, or parameter map that has no admitted evaluator.
    pub fn derive(
        resolved_type: &ResolvedValueType,
        shape: &Shape,
    ) -> Result<Self, UnsupportedValueRepresentation> {
        if let Some((scheme, contract)) = resolved_type.encoded_numeric_parts() {
            return Self::derive_encoded(resolved_type, shape, scheme, contract);
        }
        // A dense logical value is one unnamed component with the whole shape.
        // The role is the reserved zero: it is not a scheme role and cannot
        // collide with one, because every governed scheme role is nonzero.
        let domain = dense_value_domain(resolved_type)?;
        Ok(Self {
            resolved_type: resolved_type.clone(),
            shape: shape.clone(),
            components: vec![ComponentConformanceObligation {
                role: DENSE_VALUE_COMPONENT_ROLE,
                resolved_type: resolved_type.clone(),
                shape: shape.clone(),
                map: None,
                domain,
            }],
        })
    }

    /// Returns whether this validator has an admitted contract for the type.
    ///
    /// A consumer that governs several value families asks this before offering
    /// a value for conformance, so a type with no obligation to discharge is
    /// distinguished from one that failed a check it was subject to.
    #[must_use]
    pub fn is_governed(resolved_type: &ResolvedValueType) -> bool {
        Self::derive(resolved_type, &Shape::new([])).is_ok()
    }

    fn derive_encoded(
        resolved_type: &ResolvedValueType,
        shape: &Shape,
        scheme: &QuantSchemeKey,
        contract: &EncodedNumericContract,
    ) -> Result<Self, UnsupportedValueRepresentation> {
        if scheme != &strict_affine_scheme() {
            return Err(UnsupportedValueRepresentation::Scheme {
                scheme: scheme.clone(),
            });
        }
        let code_maximum = unsigned_contract_field(contract, ENCODED_NUMERIC_CODE_MAX);
        let code_minimum = unsigned_contract_field(contract, ENCODED_NUMERIC_CODE_MIN);
        let scale_domain = utf8_contract_field(contract, ENCODED_NUMERIC_SCALE_DOMAIN);
        let (Some(minimum), Some(maximum), Some(POSITIVE_NORMAL_F32_DOMAIN)) =
            (code_minimum, code_maximum, scale_domain)
        else {
            return Err(UnsupportedValueRepresentation::SchemeContract {
                scheme: scheme.clone(),
                resolved_type: Arc::new(resolved_type.clone()),
            });
        };
        let mut components = Vec::with_capacity(contract.components().len());
        for declaration in contract.components() {
            let role = declaration.role();
            if declaration
                .resolved_type()
                .encoded_numeric_parts()
                .is_some()
            {
                return Err(UnsupportedValueRepresentation::NestedComponent { role });
            }
            let map = match declaration.shape_relation() {
                EncodedComponentShape::LogicalValue => None,
                EncodedComponentShape::ParameterMap(map) => {
                    if map != &ParameterIndexMap::per_tensor() {
                        return Err(UnsupportedValueRepresentation::ParameterMap { role });
                    }
                    Some(map.clone())
                }
            };
            let domain = if declaration.resolved_type() == &F32::resolved_type() {
                ComponentValueDomain::PositiveNormalF32
            } else if declaration.resolved_type() == &U4::resolved_type()
                || declaration.resolved_type() == &U8::resolved_type()
            {
                ComponentValueDomain::UnsignedCodeRange { minimum, maximum }
            } else {
                return Err(UnsupportedValueRepresentation::ComponentType {
                    role,
                    resolved_type: Arc::new(declaration.resolved_type().clone()),
                });
            };
            components.push(ComponentConformanceObligation {
                role,
                resolved_type: declaration.resolved_type().clone(),
                shape: declaration.shape_relation().component_shape(shape),
                map,
                domain,
            });
        }
        if components.is_empty() {
            return Err(UnsupportedValueRepresentation::SchemeContract {
                scheme: scheme.clone(),
                resolved_type: Arc::new(resolved_type.clone()),
            });
        }
        Ok(Self {
            resolved_type: resolved_type.clone(),
            shape: shape.clone(),
            components,
        })
    }

    /// Returns the complete logical value type the obligations were derived from.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns the logical tensor shape of the whole value.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns the ordered component obligations in declaration order.
    #[must_use]
    pub fn components(&self) -> &[ComponentConformanceObligation] {
        &self.components
    }

    fn encode(&self, output: &mut Vec<u8>) {
        self.resolved_type.encode(output);
        encode_shape(&self.shape, output);
        push_len(output, self.components.len());
        for component in &self.components {
            component.encode(output);
        }
    }
}

/// The reserved role of the single component of a dense (non-compound) value.
///
/// Zero is reserved: every governed scheme role is nonzero, so a dense
/// obligation can never be confused with a scheme's own component. A view over
/// a dense value presents this role, which is why it is public rather than an
/// internal convention a consumer would have to guess.
pub const DENSE_VALUE_COMPONENT_ROLE: EncodedComponentRole = EncodedComponentRole::new(0);

fn dense_value_domain(
    resolved_type: &ResolvedValueType,
) -> Result<ComponentValueDomain, UnsupportedValueRepresentation> {
    if resolved_type == &U4::resolved_type() {
        Ok(ComponentValueDomain::UnsignedCodeRange {
            minimum: 0,
            maximum: 15,
        })
    } else if resolved_type == &U8::resolved_type() {
        Ok(ComponentValueDomain::UnsignedCodeRange {
            minimum: 0,
            maximum: u8::MAX,
        })
    } else {
        Err(UnsupportedValueRepresentation::NoAdmittedContract {
            resolved_type: Arc::new(resolved_type.clone()),
        })
    }
}

fn unsigned_contract_field(
    contract: &EncodedNumericContract,
    field: AttributeFieldId,
) -> Option<u8> {
    match contract
        .fields()
        .iter()
        .find(|candidate| candidate.id() == field)?
        .value()
        .view()
    {
        CanonicalValueView::Unsigned {
            width: CanonicalIntegerWidth::Bits8,
            bits,
        } => u8::try_from(bits).ok(),
        _ => None,
    }
}

fn utf8_contract_field(contract: &EncodedNumericContract, field: AttributeFieldId) -> Option<&str> {
    match contract
        .fields()
        .iter()
        .find(|candidate| candidate.id() == field)?
        .value()
        .view()
    {
        CanonicalValueView::Utf8(value) => Some(value),
        _ => None,
    }
}

/// A representation this validator has no admitted evaluator for.
///
/// Every variant names its exact subject. Nothing here is approximated through
/// a resembling scheme, a wider integer, or a nearest supported map: a
/// representation without an evaluator is refused, and the refusal says which
/// one it was.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedValueRepresentation {
    /// This validator has no admitted conformance contract for the logical type.
    ///
    /// Two situations reach this variant and it deliberately does not claim to
    /// separate them, because nothing here can: a type whose logical values
    /// carry no governed value domain — an ordinary binary32 tensor, whose
    /// every bit pattern is a value of its type — states no obligation to
    /// discharge, and a family with no evaluator at all — packed Boolean,
    /// complex, sparse, ragged — states one this build cannot check. Both are
    /// refused rather than approximated, both name the exact type, and the
    /// semantic registry remains the authority on which of the two a type is.
    NoAdmittedContract {
        /// The exact refused type.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// The encoded-numeric scheme has no admitted conformance evaluator.
    Scheme {
        /// The exact refused scheme family.
        scheme: QuantSchemeKey,
    },
    /// The scheme is admitted but this exact static contract is not.
    SchemeContract {
        /// The admitted scheme family.
        scheme: QuantSchemeKey,
        /// The exact refused contract instance.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// One component's resolved type has no admitted logical scan.
    ComponentType {
        /// The component's stable role.
        role: EncodedComponentRole,
        /// The exact refused component type.
        resolved_type: Arc<ResolvedValueType>,
    },
    /// One component's parameter index map has no admitted evaluator.
    ParameterMap {
        /// The component's stable role.
        role: EncodedComponentRole,
    },
    /// One component's type is itself an encoded value.
    NestedComponent {
        /// The component's stable role.
        role: EncodedComponentRole,
    },
}

impl UnsupportedValueRepresentation {
    /// Returns the stable name of the refused representation class.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::NoAdmittedContract { .. } => "tiler.value-conformance.no-admitted-contract",
            Self::Scheme { .. } => "tiler.value-conformance.unsupported-scheme",
            Self::SchemeContract { .. } => "tiler.value-conformance.unsupported-scheme-contract",
            Self::ComponentType { .. } => "tiler.value-conformance.unsupported-component-type",
            Self::ParameterMap { .. } => "tiler.value-conformance.unsupported-parameter-map",
            Self::NestedComponent { .. } => "tiler.value-conformance.unsupported-nested-component",
        }
    }
}

impl fmt::Display for UnsupportedValueRepresentation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        match self {
            Self::NoAdmittedContract { resolved_type } => write!(
                formatter,
                "{name}: {resolved_type:?} has no admitted conformance contract",
            ),
            Self::Scheme { scheme } => {
                write!(
                    formatter,
                    "{name}: encoded scheme {scheme} has no admitted evaluator"
                )
            }
            Self::SchemeContract { scheme, .. } => write!(
                formatter,
                "{name}: scheme {scheme} is admitted but this exact static contract is not",
            ),
            Self::ComponentType {
                role,
                resolved_type,
            } => write!(
                formatter,
                "{name}: component role {} carries {resolved_type:?}, which has no admitted logical scan",
                role.get(),
            ),
            Self::ParameterMap { role } => write!(
                formatter,
                "{name}: component role {} uses a parameter index map with no admitted evaluator",
                role.get(),
            ),
            Self::NestedComponent { role } => write!(
                formatter,
                "{name}: component role {} is itself an encoded value",
                role.get(),
            ),
        }
    }
}

impl Error for UnsupportedValueRepresentation {}

/// One logical scalar read from a bound value's authoritative logical view.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LogicalScalar {
    /// An unsigned integer code, already extracted from whatever carrier held it.
    UnsignedCode(u8),
    /// Exact binary32 bits.
    F32Bits(u32),
}

/// Why a bound value's authoritative logical view could not be read.
///
/// These are refusals about *reaching* the logical value, distinct from the
/// value being wrong. A route that cannot reconstruct a logical view refuses
/// before it commits; it never proceeds on a partial read.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LogicalViewFault {
    /// The component's memory is not readable by this host.
    Inaccessible,
    /// No coherence has been established for the component's memory.
    IncoherentView,
    /// The view cannot reconstruct a logical element at the requested index.
    UnreconstructableIndex,
    /// The stored element cannot be read as a logical scalar of its declared type.
    UnrepresentableScalar,
}

impl LogicalViewFault {
    /// Returns the stable name of this fault class.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Inaccessible => "tiler.value-conformance.inaccessible-memory",
            Self::IncoherentView => "tiler.value-conformance.absent-coherence",
            Self::UnreconstructableIndex => {
                "tiler.value-conformance.unreconstructable-logical-view"
            }
            Self::UnrepresentableScalar => "tiler.value-conformance.unrepresentable-scalar",
        }
    }
}

impl fmt::Display for LogicalViewFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// What one bound value presents as a component, before it is checked.
///
/// The *presented* role, type, and shape are what the binding claims. The
/// contract states what they must be, and mismatches are the missing,
/// duplicate, extra, swapped, wrong-type, wrong-shape, and cross-value cases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentedComponent<'a> {
    /// Stable semantic role the binding presents this component under.
    pub role: EncodedComponentRole,
    /// Complete resolved type the binding presents.
    pub resolved_type: &'a ResolvedValueType,
    /// Logical component shape the binding presents.
    pub shape: &'a Shape,
}

/// A consumer-supplied reader for one bound value's authoritative logical view.
///
/// The trait is payload-neutral on purpose: the compiler core owns what a value
/// *means*, and a host tensor, a byte range under a packed encoding, and a
/// device allocation made coherent by an adapter are three ways of reaching the
/// same logical elements. An implementor exposes logical elements at canonical
/// row-major logical indices and nothing else — there is no method by which
/// padding, alignment, or an unused packed tail could be observed.
pub trait EncodedLogicalView {
    /// Returns the number of components this binding presents, in order.
    fn presented_components(&self) -> usize;

    /// Returns the presented facts of one component, or `None` past the end.
    fn presented_component(&self, position: usize) -> Option<PresentedComponent<'_>>;

    /// Reads one logical scalar at a canonical row-major logical index.
    ///
    /// # Errors
    ///
    /// Returns the exact [`LogicalViewFault`] when the logical element cannot
    /// be reconstructed, is unreachable, or is not coherent.
    fn read_logical_scalar(
        &self,
        position: usize,
        index: u64,
    ) -> Result<LogicalScalar, LogicalViewFault>;
}

/// Why one bound value does not conform to its declared resolved type.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValueConformanceCause {
    /// The declared representation has no admitted conformance evaluator.
    Unsupported(UnsupportedValueRepresentation),
    /// The binding presents a different number of components than the type declares.
    ComponentCount {
        /// Components the type declares.
        expected: usize,
        /// Components the binding presents.
        actual: usize,
    },
    /// The component at this position carries a different role than the type declares.
    ComponentRole {
        /// Role the type declares at this position.
        expected: EncodedComponentRole,
        /// Role the binding presents.
        actual: EncodedComponentRole,
    },
    /// The component's presented type is not the declared component type.
    ComponentType {
        /// Declared component type.
        expected: Arc<ResolvedValueType>,
        /// Presented component type.
        actual: Arc<ResolvedValueType>,
    },
    /// The component's presented shape is not the one the declaration derives.
    ComponentShape {
        /// Shape the component declaration derives from the logical shape.
        expected: Arc<Shape>,
        /// Shape the binding presents.
        actual: Arc<Shape>,
    },
    /// The authoritative logical view could not be read.
    ViewFault(LogicalViewFault),
    /// A logical element is not a code in the governed inclusive range.
    CodeOutOfDomain {
        /// The exact code read.
        code: u8,
        /// Inclusive minimum admitted code.
        minimum: u8,
        /// Inclusive maximum admitted code.
        maximum: u8,
    },
    /// A logical element is not a strictly positive normal binary32 value.
    ScaleOutOfDomain {
        /// Exact bits of the refused value.
        bits: u32,
    },
    /// A logical element was not the scalar kind the declared component type requires.
    ScalarKindMismatch,
    /// The scan this obligation requires exceeds the governed element budget.
    ScanBudgetExceeded {
        /// Logical elements the obligation would visit.
        elements: u64,
        /// Governed maximum.
        limit: u64,
    },
}

impl ValueConformanceCause {
    /// Returns the stable ordered invalid-input class of this cause.
    ///
    /// # Panics
    ///
    /// Panics only if Tiler's compile-time governed code violates its grammar.
    #[must_use]
    pub fn invalid_input_code(&self) -> SemanticInvalidInputCode {
        let name = match self {
            Self::Unsupported(_) => "value-conformance-unsupported-representation",
            Self::ComponentCount { .. } => "value-conformance-component-count",
            Self::ComponentRole { .. } => "value-conformance-component-role",
            Self::ComponentType { .. } => "value-conformance-component-type",
            Self::ComponentShape { .. } => "value-conformance-component-shape",
            Self::ViewFault(_) => "value-conformance-logical-view",
            Self::CodeOutOfDomain { .. } => "value-conformance-code-out-of-domain",
            Self::ScaleOutOfDomain { .. } => "value-conformance-scale-out-of-domain",
            Self::ScalarKindMismatch => "value-conformance-scalar-kind",
            Self::ScanBudgetExceeded { .. } => "value-conformance-scan-budget",
        };
        SemanticInvalidInputCode::new("tiler", name, 1)
            .expect("the governed conformance invalid-input code is valid")
    }
}

impl fmt::Display for ValueConformanceCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(unsupported) => unsupported.fmt(formatter),
            Self::ComponentCount { expected, actual } => write!(
                formatter,
                "the type declares {expected} components and the binding presents {actual}",
            ),
            Self::ComponentRole { expected, actual } => write!(
                formatter,
                "component role {} was declared and role {} was presented",
                expected.get(),
                actual.get(),
            ),
            Self::ComponentType { expected, actual } => write!(
                formatter,
                "component type {expected:?} was declared and {actual:?} was presented",
            ),
            Self::ComponentShape { expected, actual } => write!(
                formatter,
                "component shape {expected:?} was derived and {actual:?} was presented",
            ),
            Self::ViewFault(fault) => fault.fmt(formatter),
            Self::CodeOutOfDomain {
                code,
                minimum,
                maximum,
            } => write!(
                formatter,
                "code {code} is outside the inclusive governed domain {minimum}..={maximum}",
            ),
            Self::ScaleOutOfDomain { bits } => write!(
                formatter,
                "binary32 value {bits:#010x} is not strictly positive and normal",
            ),
            Self::ScalarKindMismatch => {
                formatter.write_str("the logical element is not the scalar kind the type requires")
            }
            Self::ScanBudgetExceeded { elements, limit } => write!(
                formatter,
                "the obligation would visit {elements} logical elements, over the governed limit {limit}",
            ),
        }
    }
}

/// One deterministic typed refusal of a bound value.
///
/// The diagnostic coordinate is `(logical index, invalid-input code, component
/// ordinal)` and the reported refusal is the minimum under that order, so the
/// same invalid value reports the same refusal regardless of traversal order,
/// chunking, or which equivalent physical layout the logical view was
/// reconstructed from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueConformanceRejection {
    cause: ValueConformanceCause,
    component: Option<u32>,
    logical_index: Option<u64>,
}

impl ValueConformanceRejection {
    fn structural(cause: ValueConformanceCause, component: Option<u32>) -> Self {
        Self {
            cause,
            component,
            logical_index: None,
        }
    }

    fn at(cause: ValueConformanceCause, component: u32, logical_index: u64) -> Self {
        Self {
            cause,
            component: Some(component),
            logical_index: Some(logical_index),
        }
    }

    /// Returns why the value does not conform.
    #[must_use]
    pub const fn cause(&self) -> &ValueConformanceCause {
        &self.cause
    }

    /// Returns the stable ordered invalid-input class.
    #[must_use]
    pub fn invalid_input_code(&self) -> SemanticInvalidInputCode {
        self.cause.invalid_input_code()
    }

    /// Returns the declaration-order component ordinal, absent for a whole-value refusal.
    #[must_use]
    pub const fn component_ordinal(&self) -> Option<u32> {
        self.component
    }

    /// Returns the canonical row-major logical index, absent for a structural refusal.
    ///
    /// The index is a property of the *logical* value, never a byte offset, so
    /// two equivalent physical layouts of the same logical value report the
    /// same index.
    #[must_use]
    pub const fn logical_index(&self) -> Option<u64> {
        self.logical_index
    }

    /// Returns the deterministic diagnostic order key.
    fn order_key(&self) -> (u64, SemanticInvalidInputCode, u32) {
        (
            self.logical_index.unwrap_or(0),
            self.invalid_input_code(),
            self.component.unwrap_or(0),
        )
    }
}

impl fmt::Display for ValueConformanceRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "value.{}: ", self.invalid_input_code())?;
        match (self.component, self.logical_index) {
            (Some(component), Some(index)) => {
                write!(formatter, "component {component} logical index {index}: ")?;
            }
            (Some(component), None) => write!(formatter, "component {component}: ")?,
            (None, _) => {}
        }
        self.cause.fmt(formatter)
    }
}

impl Error for ValueConformanceRejection {}

/// A caller-stated monotone version of one bound value's contents.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValueVersion(u64);

impl ValueVersion {
    /// Creates a value version.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the fixed-width version.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The coherence epoch under which one bound value's memory was made readable.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoherenceEpoch(u64);

impl CoherenceEpoch {
    /// Creates a coherence epoch.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the fixed-width epoch.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// How a bound value's contents are held stable for the life of one proof.
///
/// The two arms are not interchangeable and neither is a default. An immutable
/// host value cannot change under a proof, so no version or epoch exists to
/// name. A mutable binding can, so its owner states both, and a proof taken at
/// one version and epoch does not authorize another.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ValueStability {
    /// The host owns an immutable value for the whole life of the proof.
    ImmutableHost,
    /// A versioned binding whose owner establishes coherence before each read.
    Versioned {
        /// Monotone version of the bound contents.
        version: ValueVersion,
        /// Epoch under which the contents were made readable.
        coherence: CoherenceEpoch,
    },
}

impl ValueStability {
    fn encode(self, output: &mut Vec<u8>) {
        match self {
            Self::ImmutableHost => output.push(0x01),
            Self::Versioned { version, coherence } => {
                output.push(0x02);
                output.extend_from_slice(&version.get().to_be_bytes());
                output.extend_from_slice(&coherence.get().to_be_bytes());
            }
        }
    }
}

/// The identity of the route one conformance proof is valid for.
///
/// Opaque bytes supplied by whichever authority owns the route — a semantic
/// graph identity for a host evaluation, an artifact program identity for a
/// loaded route. A proof does not travel between routes: evidence taken against
/// one route never authorizes a subject under another.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RouteDependency(Vec<u8>);

impl RouteDependency {
    /// Names the route this proof depends on.
    #[must_use]
    pub fn new(identity: impl AsRef<[u8]>) -> Self {
        Self(identity.as_ref().to_vec())
    }

    /// Returns the exact route identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Where one value came from, and therefore which producer completed it.
///
/// This is the producer-completion field: a directly bound value has no
/// producer inside the program, and a produced value names the exact completed
/// occurrence and result position it is. Neither is a pointer and neither is a
/// slot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ValueOrigin {
    /// The value entered across the program interface under one stable key.
    DirectBinding {
        /// Interface key the value was bound under.
        input: InputKey,
    },
    /// The value is one ordered result of a completed operation occurrence.
    ProducedResult {
        /// Governed operation family of the completed occurrence.
        operation: OpKey,
        /// Canonical graph-local coordinate of the occurrence's first result.
        coordinate: u64,
        /// Ordered result position within the occurrence.
        result: u32,
    },
}

impl ValueOrigin {
    /// Names one directly bound program input.
    #[must_use]
    pub const fn direct_binding(input: InputKey) -> Self {
        Self::DirectBinding { input }
    }

    /// Names one ordered result of a completed operation occurrence.
    ///
    /// The value is taken rather than a bare position, so the origin cannot
    /// name a result of a different occurrence: `result` is located in this
    /// occurrence's own ordered results or the call is refused. The coordinate
    /// recorded is the occurrence's canonical graph-local one, so the origin is
    /// stable across construction order and is not a handle.
    ///
    /// # Errors
    ///
    /// Returns [`ValueOriginError::NotAResultOfThisOccurrence`] when the value
    /// is not one of this occurrence's ordered results.
    ///
    /// # Panics
    ///
    /// Panics only if a verified occurrence's bounded result list exceeds
    /// `u32::MAX`, which its own construction bound forbids.
    pub fn produced_result(
        operation: OperationRef<'_>,
        result: ValueId,
    ) -> Result<Self, ValueOriginError> {
        let position = operation
            .results()
            .position(|candidate| candidate == result)
            .ok_or_else(|| ValueOriginError::NotAResultOfThisOccurrence {
                operation: operation.key().clone(),
                results: operation.operation.results.len(),
            })?;
        Ok(Self::ProducedResult {
            operation: operation.key().clone(),
            coordinate: occurrence_coordinate(operation),
            result: u32::try_from(position).expect("a bounded result list fits u32"),
        })
    }

    fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::DirectBinding { input } => {
                output.push(0x01);
                push_slice(output, input.as_str().as_bytes());
            }
            Self::ProducedResult {
                operation,
                coordinate,
                result,
            } => {
                output.push(0x02);
                operation.encode(output);
                output.extend_from_slice(&coordinate.to_be_bytes());
                output.extend_from_slice(&result.to_be_bytes());
            }
        }
    }
}

fn occurrence_coordinate(operation: OperationRef<'_>) -> u64 {
    let first_result = operation.operation.results[0];
    operation.program.canonical_value_ids[first_result.as_usize()]
}

/// Why one value origin could not be named.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValueOriginError {
    /// The named value is not one of this occurrence's ordered results.
    NotAResultOfThisOccurrence {
        /// Governed operation family.
        operation: OpKey,
        /// Ordered results the occurrence has.
        results: usize,
    },
}

impl fmt::Display for ValueOriginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAResultOfThisOccurrence { operation, results } => write!(
                formatter,
                "the named value is not one of the {results} ordered results of {operation}",
            ),
        }
    }
}

impl Error for ValueOriginError {}

/// The exact subject one conformance proof is about.
///
/// Every field is identity-bearing. Perturbing any one of them yields a
/// different subject, and evidence for one subject never authorizes another —
/// which is what makes the proof unforgeable by transplant and unusable after a
/// version, coherence, or route change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueConformanceSubject {
    origin: ValueOrigin,
    stability: ValueStability,
    route: RouteDependency,
    view: SemanticLogicalView,
    resolved_type: ResolvedValueType,
    shape: Shape,
}

impl ValueConformanceSubject {
    /// Names the exact subject of one conformance proof.
    #[must_use]
    pub const fn new(
        origin: ValueOrigin,
        stability: ValueStability,
        route: RouteDependency,
        view: SemanticLogicalView,
        resolved_type: ResolvedValueType,
        shape: Shape,
    ) -> Self {
        Self {
            origin,
            stability,
            route,
            view,
            resolved_type,
            shape,
        }
    }

    /// Returns where the value came from and which producer completed it.
    #[must_use]
    pub const fn origin(&self) -> &ValueOrigin {
        &self.origin
    }

    /// Returns how the contents are held stable for the life of the proof.
    #[must_use]
    pub const fn stability(&self) -> ValueStability {
        self.stability
    }

    /// Returns the route this proof depends on.
    #[must_use]
    pub const fn route(&self) -> &RouteDependency {
        &self.route
    }

    /// Returns the exact logical projection the proof is about.
    #[must_use]
    pub const fn view(&self) -> SemanticLogicalView {
        self.view
    }

    /// Returns the complete declared logical value type.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns the logical tensor shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    fn encode(&self, output: &mut Vec<u8>) {
        self.origin.encode(output);
        self.stability.encode(output);
        push_slice(output, self.route.as_bytes());
        self.view.encode(output);
        self.resolved_type.encode(output);
        encode_shape(&self.shape, output);
    }
}

/// Proof that one exact value subject conforms to its complete resolved type.
///
/// Held rather than recomputed: a consumer that already has evidence for the
/// same subject under the same validator has a complete proof and rescanning
/// would answer a question it already holds the answer to.
///
/// The derived contract is encoded alongside the subject even though it is
/// derivable from it. That is not redundancy: it makes the discharged
/// obligations readable from the evidence itself, and it means a change to the
/// derivation invalidates every proof taken under the old one rather than
/// silently reinterpreting it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueConformanceEvidence {
    validator: ConformanceValidatorIdentity,
    subject: ValueConformanceSubject,
    contract: ResolvedValueConformanceContract,
    canonical: Vec<u8>,
}

impl ValueConformanceEvidence {
    fn new(
        validator: &ConformanceValidatorIdentity,
        subject: &ValueConformanceSubject,
        contract: &ResolvedValueConformanceContract,
    ) -> Self {
        let mut canonical = EVIDENCE_DOMAIN.to_vec();
        validator.encode(&mut canonical);
        subject.encode(&mut canonical);
        contract.encode(&mut canonical);
        Self {
            validator: validator.clone(),
            subject: subject.clone(),
            contract: contract.clone(),
            canonical,
        }
    }

    /// Returns the validator whose revision this proof was taken under.
    #[must_use]
    pub const fn validator(&self) -> &ConformanceValidatorIdentity {
        &self.validator
    }

    /// Returns the exact subject this proof is about.
    #[must_use]
    pub const fn subject(&self) -> &ValueConformanceSubject {
        &self.subject
    }

    /// Returns the obligations this proof discharged.
    #[must_use]
    pub const fn contract(&self) -> &ResolvedValueConformanceContract {
        &self.contract
    }

    /// Returns collision-free canonical evidence bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.canonical
    }

    /// Returns whether this proof authorizes that exact subject under that validator.
    ///
    /// Pointer equality and slot position are never consulted, and there is no
    /// way to reach a `true` here from a subject that differs in origin,
    /// stability, route, view, type, or shape.
    #[must_use]
    pub fn authorizes(
        &self,
        subject: &ValueConformanceSubject,
        validator: &ConformanceValidatorIdentity,
    ) -> bool {
        &self.subject == subject && &self.validator == validator
    }
}

/// Establishes conformance for one bound value, reusing a complete proof.
///
/// When `existing` already authorizes this exact subject under this validator,
/// it is returned unchanged and `view` is never read: a complete
/// same-provenance proof is the answer, and rescanning would only recompute it.
/// Any difference in origin, stability, route, view, type, or shape is a
/// different subject and is scanned.
///
/// # Errors
///
/// Returns the deterministic [`ValueConformanceRejection`] naming the first
/// refusal under the diagnostic order.
pub fn conform_bound_value(
    validator: &ConformanceValidatorIdentity,
    subject: &ValueConformanceSubject,
    existing: Option<&ValueConformanceEvidence>,
    view: &dyn EncodedLogicalView,
) -> Result<ValueConformanceEvidence, ValueConformanceRejection> {
    if let Some(evidence) = existing
        && evidence.authorizes(subject, validator)
    {
        return Ok(evidence.clone());
    }
    scan_bound_value(validator, subject, view)
}

/// Establishes conformance for one bound value by scanning its logical view.
///
/// # Errors
///
/// Returns the deterministic [`ValueConformanceRejection`] naming the first
/// refusal under the diagnostic order.
pub fn scan_bound_value(
    validator: &ConformanceValidatorIdentity,
    subject: &ValueConformanceSubject,
    view: &dyn EncodedLogicalView,
) -> Result<ValueConformanceEvidence, ValueConformanceRejection> {
    let contract = check_bound_value(subject.resolved_type(), subject.shape(), view)?;
    Ok(ValueConformanceEvidence::new(validator, subject, &contract))
}

/// Checks one bound value against its type's obligations and mints no evidence.
///
/// This is the single authority for what a bound value of an admitted type must
/// satisfy, and [`scan_bound_value`] is it plus a provenance binding. A
/// consumer whose own validation boundary has no provenance to bind — a
/// representation validator registered against a type rather than against a
/// value occurrence — checks here rather than restating a domain, so a scheme
/// that narrows its contract narrows both paths at once.
///
/// # Errors
///
/// Returns the deterministic [`ValueConformanceRejection`] naming the first
/// refusal under the diagnostic order.
pub fn check_bound_value(
    resolved_type: &ResolvedValueType,
    shape: &Shape,
    view: &dyn EncodedLogicalView,
) -> Result<ResolvedValueConformanceContract, ValueConformanceRejection> {
    let contract = ResolvedValueConformanceContract::derive(resolved_type, shape).map_err(
        |unsupported| {
            ValueConformanceRejection::structural(
                ValueConformanceCause::Unsupported(unsupported),
                None,
            )
        },
    )?;
    check_presented_structure(&contract, view)?;
    check_scan_budget(&contract)?;
    if let Some(rejection) = scan_logical_elements(&contract, view) {
        return Err(rejection);
    }
    Ok(contract)
}

fn check_presented_structure(
    contract: &ResolvedValueConformanceContract,
    view: &dyn EncodedLogicalView,
) -> Result<(), ValueConformanceRejection> {
    let expected = contract.components().len();
    let actual = view.presented_components();
    if expected != actual {
        return Err(ValueConformanceRejection::structural(
            ValueConformanceCause::ComponentCount { expected, actual },
            None,
        ));
    }
    for (position, obligation) in contract.components().iter().enumerate() {
        let ordinal = component_ordinal(position);
        let Some(presented) = view.presented_component(position) else {
            return Err(ValueConformanceRejection::structural(
                ValueConformanceCause::ComponentCount {
                    expected,
                    actual: position,
                },
                Some(ordinal),
            ));
        };
        if presented.role != obligation.role() {
            return Err(ValueConformanceRejection::structural(
                ValueConformanceCause::ComponentRole {
                    expected: obligation.role(),
                    actual: presented.role,
                },
                Some(ordinal),
            ));
        }
        if presented.resolved_type != obligation.resolved_type() {
            return Err(ValueConformanceRejection::structural(
                ValueConformanceCause::ComponentType {
                    expected: Arc::new(obligation.resolved_type().clone()),
                    actual: Arc::new(presented.resolved_type.clone()),
                },
                Some(ordinal),
            ));
        }
        if presented.shape != obligation.shape() {
            return Err(ValueConformanceRejection::structural(
                ValueConformanceCause::ComponentShape {
                    expected: Arc::new(obligation.shape().clone()),
                    actual: Arc::new(presented.shape.clone()),
                },
                Some(ordinal),
            ));
        }
    }
    Ok(())
}

fn check_scan_budget(
    contract: &ResolvedValueConformanceContract,
) -> Result<(), ValueConformanceRejection> {
    let mut total = 0_u64;
    for (position, obligation) in contract.components().iter().enumerate() {
        let elements = logical_element_count(obligation.shape());
        total = total.saturating_add(elements);
        if total > MAX_CONFORMANCE_SCAN_ELEMENTS {
            return Err(ValueConformanceRejection::structural(
                ValueConformanceCause::ScanBudgetExceeded {
                    elements: total,
                    limit: MAX_CONFORMANCE_SCAN_ELEMENTS,
                },
                Some(component_ordinal(position)),
            ));
        }
    }
    Ok(())
}

/// Returns the minimum refusal across every component, or `None` when all conform.
///
/// Each component is scanned in ascending logical index and stops at its own
/// first refusal, which is therefore that component's minimum index. The
/// reported refusal is the minimum of those under the full diagnostic order, so
/// the answer does not depend on the order components happen to be visited in.
fn scan_logical_elements(
    contract: &ResolvedValueConformanceContract,
    view: &dyn EncodedLogicalView,
) -> Option<ValueConformanceRejection> {
    let mut minimum: Option<ValueConformanceRejection> = None;
    for (position, obligation) in contract.components().iter().enumerate() {
        let Some(rejection) = scan_component(position, obligation, view) else {
            continue;
        };
        let replace = minimum
            .as_ref()
            .is_none_or(|prior| rejection.order_key() < prior.order_key());
        if replace {
            minimum = Some(rejection);
        }
    }
    minimum
}

fn scan_component(
    position: usize,
    obligation: &ComponentConformanceObligation,
    view: &dyn EncodedLogicalView,
) -> Option<ValueConformanceRejection> {
    let ordinal = component_ordinal(position);
    let elements = logical_element_count(obligation.shape());
    for index in 0..elements {
        let scalar = match view.read_logical_scalar(position, index) {
            Ok(scalar) => scalar,
            Err(fault) => {
                return Some(ValueConformanceRejection::at(
                    ValueConformanceCause::ViewFault(fault),
                    ordinal,
                    index,
                ));
            }
        };
        if let Some(cause) = check_scalar(obligation.value_domain(), scalar) {
            return Some(ValueConformanceRejection::at(cause, ordinal, index));
        }
    }
    None
}

fn check_scalar(
    domain: ComponentValueDomain,
    scalar: LogicalScalar,
) -> Option<ValueConformanceCause> {
    match (domain, scalar) {
        (
            ComponentValueDomain::UnsignedCodeRange { minimum, maximum },
            LogicalScalar::UnsignedCode(code),
        ) => (code < minimum || code > maximum).then_some(ValueConformanceCause::CodeOutOfDomain {
            code,
            minimum,
            maximum,
        }),
        (ComponentValueDomain::PositiveNormalF32, LogicalScalar::F32Bits(bits)) => {
            // The governed scale domain is `positive-normal-f32`. `is_normal`
            // is already false for zero, subnormal, infinite, and NaN values,
            // so the sign test is the only thing it does not cover.
            let value = f32::from_bits(bits);
            (!value.is_normal() || value <= 0.0)
                .then_some(ValueConformanceCause::ScaleOutOfDomain { bits })
        }
        (ComponentValueDomain::UnsignedCodeRange { .. }, LogicalScalar::F32Bits(_))
        | (ComponentValueDomain::PositiveNormalF32, LogicalScalar::UnsignedCode(_)) => {
            Some(ValueConformanceCause::ScalarKindMismatch)
        }
    }
}

fn component_ordinal(position: usize) -> u32 {
    u32::try_from(position).expect("a bounded component list fits u32")
}

fn logical_element_count(shape: &Shape) -> u64 {
    let mut count = 1_u64;
    for extent in shape.extents() {
        count = count.saturating_mul(extent.get());
    }
    count
}

fn encode_shape(shape: &Shape, output: &mut Vec<u8>) {
    push_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

/// Evidence that one operation occurrence's declared preconditions all hold.
///
/// A statically proved precondition needs nothing; a residual one is an exact
/// obligation, and this type cannot be minted while one of them is
/// undischarged. That is what makes a composed proof a proof rather than an
/// assumption about the producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticPreconditionsDischarged {
    operation: OpKey,
    coordinate: u64,
    predicates: Vec<SemanticPredicateIdentity>,
}

impl SemanticPreconditionsDischarged {
    /// Discharges every declared precondition of one completed occurrence.
    ///
    /// Each residual obligation must be named exactly once in `discharged`. An
    /// obligation belonging to another occurrence is refused rather than
    /// counted: an obligation identity is occurrence-exact, so accepting a
    /// foreign one would let one discharge satisfy a different occurrence.
    ///
    /// # Errors
    ///
    /// Returns [`PreconditionDischargeError`] for an undischarged residual or a
    /// discharge that belongs to no residual of this occurrence.
    ///
    /// # Panics
    ///
    /// Panics only if a completed program retains a residual precondition
    /// without the obligation identity its own construction always assigns.
    pub fn for_occurrence(
        operation: OperationRef<'_>,
        discharged: &[&SemanticPreconditionObligationIdentity],
    ) -> Result<Self, PreconditionDischargeError> {
        let mut predicates = Vec::new();
        let mut consumed = vec![false; discharged.len()];
        for precondition in operation.semantic_preconditions() {
            predicates.push(precondition.predicate().clone());
            if precondition.status() == SemanticPreconditionStatus::Proven {
                continue;
            }
            let obligation = precondition
                .obligation_identity()
                .expect("a residual precondition carries an obligation identity");
            let position = discharged
                .iter()
                .position(|candidate| candidate.as_bytes() == obligation.as_bytes())
                .ok_or_else(|| PreconditionDischargeError::UndischargedObligation {
                    operation: Arc::new(operation.key().clone()),
                    ordinal: precondition.declaration_ordinal().get(),
                    predicate: Arc::new(precondition.predicate().clone()),
                })?;
            consumed[position] = true;
        }
        if let Some(position) = consumed.iter().position(|used| !used) {
            return Err(PreconditionDischargeError::ForeignObligation {
                operation: Arc::new(operation.key().clone()),
                position,
            });
        }
        Ok(Self {
            operation: operation.key().clone(),
            coordinate: occurrence_coordinate(operation),
            predicates,
        })
    }

    /// Returns the governed operation family whose preconditions were discharged.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the canonical graph-local coordinate of the discharged occurrence.
    #[must_use]
    pub const fn coordinate(&self) -> u64 {
        self.coordinate
    }

    fn discharges(&self, predicate: &SemanticPredicateIdentity) -> bool {
        self.predicates.contains(predicate)
    }
}

/// Why one occurrence's preconditions could not be discharged.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PreconditionDischargeError {
    /// A residual obligation of this occurrence was not discharged.
    UndischargedObligation {
        /// Governed operation family.
        operation: Arc<OpKey>,
        /// Declaration ordinal of the undischarged precondition.
        ordinal: u32,
        /// The undischarged predicate.
        predicate: Arc<SemanticPredicateIdentity>,
    },
    /// A supplied discharge names no residual obligation of this occurrence.
    ForeignObligation {
        /// Governed operation family.
        operation: Arc<OpKey>,
        /// Position of the foreign discharge in the supplied list.
        position: usize,
    },
}

impl fmt::Display for PreconditionDischargeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndischargedObligation {
                operation,
                ordinal,
                predicate,
            } => write!(
                formatter,
                "operation {operation} declaration {ordinal} leaves {predicate} undischarged",
            ),
            Self::ForeignObligation {
                operation,
                position,
            } => write!(
                formatter,
                "discharge {position} names no residual obligation of operation {operation}",
            ),
        }
    }
}

impl Error for PreconditionDischargeError {}

/// One operand whose own conformance the producer carries into its result.
#[derive(Clone, Copy, Debug)]
pub struct ComposedOperand<'a> {
    /// Result component role this operand's conformance establishes.
    pub role: EncodedComponentRole,
    /// The operand's own complete conformance proof.
    pub evidence: &'a ValueConformanceEvidence,
}

/// What establishes one component of a produced value's conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ComponentEstablishment {
    /// The conformance of one operand the producer carries through unchanged.
    OperandConformance,
    /// One of the producing operation's own declared semantic preconditions.
    OperationPrecondition(SemanticPredicateIdentity),
    /// The producing operation's declared semantics, which cannot produce an
    /// out-of-domain element.
    OperationSemantics,
}

/// Returns the admitted production rule for one governed producer, in result
/// component-declaration order.
///
/// The two admitted producers differ exactly where the ticket's derivation says
/// they do. `Assemble` associates existing components, so its codes and zero
/// point are carried through and only their conformance establishes the
/// result's. `Quantize` *computes* its codes under a declared clamp to the
/// inclusive code domain followed by nearest-even rounding, so the codes are
/// established by the operation's own semantics; its zero point is still
/// carried through. Both take the scale as a typed operand under the same
/// declared normal-scale precondition, so the scale component of either result
/// is established by that discharge and never by rescanning.
fn production_rule(
    operation: &OpKey,
) -> Option<Vec<(EncodedComponentRole, ComponentEstablishment)>> {
    let scale = (
        STRICT_AFFINE_SCALE_ROLE,
        ComponentEstablishment::OperationPrecondition(positive_normal_scalar_predicate()),
    );
    let zero_point = (
        STRICT_AFFINE_ZERO_POINT_ROLE,
        ComponentEstablishment::OperandConformance,
    );
    if operation == &assemble_strict_affine_op() {
        Some(vec![
            (
                STRICT_AFFINE_CODES_ROLE,
                ComponentEstablishment::OperandConformance,
            ),
            scale,
            zero_point,
        ])
    } else if operation == &quantize_strict_affine_op() {
        Some(vec![
            (
                STRICT_AFFINE_CODES_ROLE,
                ComponentEstablishment::OperationSemantics,
            ),
            scale,
            zero_point,
        ])
    } else {
        None
    }
}

/// Establishes conformance of a produced value from its producer's verified semantics.
///
/// Reads no payload. The result's components are established by the operands'
/// own conformance, by the occurrence's discharged preconditions, and by the
/// operation's declared semantics — which is precisely why a produced value is
/// not rescanned: scanning it would re-derive a fact the producer already
/// proved, and would answer a different question if the two ever disagreed.
///
/// # Errors
///
/// Returns [`ProofCompositionError`] when the subject is not a produced result,
/// when the discharge belongs to another occurrence, when the producer has no
/// admitted composition rule, or when an operand proof does not cover the
/// component it is offered for.
pub fn compose_produced_conformance(
    validator: &ConformanceValidatorIdentity,
    subject: &ValueConformanceSubject,
    preconditions: &SemanticPreconditionsDischarged,
    operands: &[ComposedOperand<'_>],
) -> Result<ValueConformanceEvidence, ProofCompositionError> {
    let ValueOrigin::ProducedResult {
        operation,
        coordinate,
        ..
    } = subject.origin()
    else {
        return Err(ProofCompositionError::NotAProducedResult);
    };
    if operation != preconditions.operation() || *coordinate != preconditions.coordinate() {
        return Err(ProofCompositionError::DischargeOccurrenceMismatch {
            subject: Arc::new(operation.clone()),
            discharged: Arc::new(preconditions.operation().clone()),
        });
    }
    let rule =
        production_rule(operation).ok_or_else(|| ProofCompositionError::UnsupportedProducer {
            operation: Arc::new(operation.clone()),
        })?;
    let contract =
        ResolvedValueConformanceContract::derive(subject.resolved_type(), subject.shape())
            .map_err(ProofCompositionError::Unsupported)?;
    if rule.len() != contract.components().len() {
        return Err(ProofCompositionError::RuleDisagreesWithContract {
            operation: Arc::new(operation.clone()),
        });
    }
    let mut consumed = vec![false; operands.len()];
    for (obligation, (role, establishment)) in contract.components().iter().zip(&rule) {
        if obligation.role() != *role {
            return Err(ProofCompositionError::RuleDisagreesWithContract {
                operation: Arc::new(operation.clone()),
            });
        }
        match establishment {
            ComponentEstablishment::OperationSemantics => {}
            ComponentEstablishment::OperationPrecondition(predicate) => {
                if !preconditions.discharges(predicate) {
                    return Err(ProofCompositionError::UndischargedComponentPredicate {
                        role: *role,
                        predicate: Arc::new(predicate.clone()),
                    });
                }
            }
            ComponentEstablishment::OperandConformance => {
                let position = operands
                    .iter()
                    .position(|operand| operand.role == *role)
                    .ok_or(ProofCompositionError::MissingOperandProof { role: *role })?;
                consumed[position] = true;
                check_operand_proof(validator, subject, obligation, operands[position].evidence)?;
            }
        }
    }
    if let Some(position) = consumed.iter().position(|used| !used) {
        return Err(ProofCompositionError::ForeignOperandProof {
            role: operands[position].role,
        });
    }
    Ok(ValueConformanceEvidence::new(validator, subject, &contract))
}

fn check_operand_proof(
    validator: &ConformanceValidatorIdentity,
    subject: &ValueConformanceSubject,
    obligation: &ComponentConformanceObligation,
    evidence: &ValueConformanceEvidence,
) -> Result<(), ProofCompositionError> {
    if evidence.validator() != validator {
        return Err(ProofCompositionError::OperandValidatorMismatch {
            role: obligation.role(),
            expected: Arc::new(validator.clone()),
            actual: Arc::new(evidence.validator().clone()),
        });
    }
    if evidence.subject().route() != subject.route() {
        return Err(ProofCompositionError::OperandRouteMismatch {
            role: obligation.role(),
        });
    }
    if evidence.subject().resolved_type() != obligation.resolved_type()
        || evidence.subject().shape() != obligation.shape()
    {
        return Err(ProofCompositionError::OperandDoesNotCoverComponent {
            role: obligation.role(),
            expected: Arc::new(obligation.resolved_type().clone()),
            actual: Arc::new(evidence.subject().resolved_type().clone()),
        });
    }
    Ok(())
}

/// Why one produced value's proof could not be composed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProofCompositionError {
    /// The subject is a direct binding, which is proved by scanning instead.
    NotAProducedResult,
    /// The discharge names a different occurrence than the subject does.
    DischargeOccurrenceMismatch {
        /// Operation the subject names.
        subject: Arc<OpKey>,
        /// Operation the discharge names.
        discharged: Arc<OpKey>,
    },
    /// The producing operation has no admitted composition rule.
    UnsupportedProducer {
        /// The exact refused producer.
        operation: Arc<OpKey>,
    },
    /// The result type has no admitted conformance evaluator.
    Unsupported(UnsupportedValueRepresentation),
    /// The admitted rule and the derived contract name different components.
    RuleDisagreesWithContract {
        /// The producer whose rule disagrees.
        operation: Arc<OpKey>,
    },
    /// A component established by a precondition has no matching discharge.
    UndischargedComponentPredicate {
        /// The component role.
        role: EncodedComponentRole,
        /// The predicate that would have established it.
        predicate: Arc<SemanticPredicateIdentity>,
    },
    /// A component established by an operand has no supplied proof.
    MissingOperandProof {
        /// The component role.
        role: EncodedComponentRole,
    },
    /// A supplied operand proof establishes no component of this result.
    ForeignOperandProof {
        /// The role the foreign proof was offered for.
        role: EncodedComponentRole,
    },
    /// An operand proof was taken under a different validator.
    OperandValidatorMismatch {
        /// The component role.
        role: EncodedComponentRole,
        /// Validator this composition uses.
        expected: Arc<ConformanceValidatorIdentity>,
        /// Validator the operand proof was taken under.
        actual: Arc<ConformanceValidatorIdentity>,
    },
    /// An operand proof depends on a different route.
    OperandRouteMismatch {
        /// The component role.
        role: EncodedComponentRole,
    },
    /// An operand proof is about a value that is not this component.
    OperandDoesNotCoverComponent {
        /// The component role.
        role: EncodedComponentRole,
        /// Component type the contract derives.
        expected: Arc<ResolvedValueType>,
        /// Type the offered proof is about.
        actual: Arc<ResolvedValueType>,
    },
}

impl fmt::Display for ProofCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAProducedResult => formatter
                .write_str("a directly bound value is proved by scanning, not by composition"),
            Self::DischargeOccurrenceMismatch {
                subject,
                discharged,
            } => write!(
                formatter,
                "the subject names occurrence of {subject} and the discharge names {discharged}",
            ),
            Self::UnsupportedProducer { operation } => write!(
                formatter,
                "operation {operation} has no admitted conformance composition rule",
            ),
            Self::Unsupported(unsupported) => unsupported.fmt(formatter),
            Self::RuleDisagreesWithContract { operation } => write!(
                formatter,
                "the admitted rule for {operation} names different components than its result type",
            ),
            Self::UndischargedComponentPredicate { role, predicate } => write!(
                formatter,
                "component role {} needs {predicate} discharged and it is not",
                role.get(),
            ),
            Self::MissingOperandProof { role } => write!(
                formatter,
                "component role {} needs an operand conformance proof and none was supplied",
                role.get(),
            ),
            Self::ForeignOperandProof { role } => write!(
                formatter,
                "the proof offered for role {} establishes no component of this result",
                role.get(),
            ),
            Self::OperandValidatorMismatch {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "component role {} was proved under {actual} and this composition uses {expected}",
                role.get(),
            ),
            Self::OperandRouteMismatch { role } => write!(
                formatter,
                "the proof offered for role {} depends on another route",
                role.get(),
            ),
            Self::OperandDoesNotCoverComponent {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "component role {} declares {expected:?} and the offered proof is about {actual:?}",
                role.get(),
            ),
        }
    }
}

impl Error for ProofCompositionError {}

#[cfg(test)]
mod tests;
