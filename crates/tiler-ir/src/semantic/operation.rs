use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::push_len;
use crate::shape::{Axis, ExtentSourceError, ExtentSources, Shape, ShapeSymbol, SourcedShape};

use super::handles::{GraphId, OperationId, OperationIndex, ValueId, ValueIndex};
use super::interface::InputIndex;
use super::precondition::{
    SemanticPreconditionData, SemanticPreconditionDeclarations, SemanticPreconditionRef,
};
use super::program::ProgramData;
use super::registry::{NormativeDefinitionRef, ProviderIdentity};
use super::types::{
    AttributeFieldId, CanonicalField, CanonicalValue, ResolvedValueType, TypeIdentityError, TypeKey,
};

/// The bounded profile's canonical quiet NaN produced by arithmetic.
pub const CANONICAL_F32_ARITHMETIC_NAN_BITS: u32 = 0x7fc0_0000;
/// Stable field ID carrying exact f32 bits on the standard constant operation.
pub const F32_CONSTANT_BITS_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);
/// Stable field ID carrying canonical axes on the standard strict Sum.
pub const REDUCTION_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);
/// # The governed fact-field vocabulary
///
/// `facts()` is publicly readable on every governed definition, and these are
/// the identifiers that make what it returns interpretable. Before they were
/// published a reader could obtain a fact record and had no stated way to read
/// it — it had to hardcode a bare integer against a numbering no contract
/// stated, or ignore the record and rely on prose in the normative reference.
/// The first breaks silently the moment a field is renumbered; the second is
/// the situation these facts were declared to end.
///
/// **Field IDs are record-local.** Each constant below names a field of *one*
/// record, and equal integers in different records are unrelated: field 1 of
/// the `f32` type record names a class, field 1 of the arithmetic operation
/// record names a rounding rule, and nothing normalizes them merely because
/// both are stored as field 1. A shared vocabulary across records would have to
/// be introduced explicitly and is not what these are.
///
/// **Renumbering a published ID is a breaking identity change.** These reach
/// out-of-crate reference capabilities and index-access lowering providers,
/// which are exactly the consumers the facts exist for, so a renumbering
/// silently changes what a conforming provider reads.
///
/// Presence and absence both carry meaning, and only where stated. A field
/// documented as conditional is absent when its condition does not hold rather
/// than present with a value the operation never produces; absence of an
/// unconditional field is a malformed record rather than a default.
/// Field naming a governed value type's representation class.
///
/// Unconditional on every governed built-in dtype definition. The value is one
/// of `logical-predicate`, `signed-integer`, `unsigned-integer`, `ieee-binary`,
/// `bfloat`, `ocp-binary-element`, `ocp-exponent-scale`, `ieee-decimal`,
/// `complex`, or `ocp-microscaling-block-scheme`. The class selects which of the
/// conditional fields below the record carries; it never replaces the nominal
/// key, and two formats sharing a class are not thereby related.
pub const SCALAR_TYPE_FACT_CLASS: AttributeFieldId = AttributeFieldId::new(1);
/// Field naming a governed value type's logical width in bits.
///
/// Conditional: absent on `logical-predicate`, whose two members have no logical
/// width and whose bit-, byte-, or other ABI-sized representation is a physical
/// storage choice; absent on `complex` and `ocp-microscaling-block-scheme`,
/// whose widths follow from their constituents and physical layout rather than
/// from the logical identity.
pub const SCALAR_TYPE_FACT_WIDTH_BITS: AttributeFieldId = AttributeFieldId::new(2);
/// Field naming the alias and equivalence policy a governed identity carries.
///
/// Unconditional on every governed built-in dtype definition, and deliberately
/// one shared value: ADRs 0027 and 0034 state one rule for the whole catalog.
pub const SCALAR_TYPE_FACT_ALIAS_POLICY: AttributeFieldId = AttributeFieldId::new(3);
/// Field counting the members of a fixed-cardinality logical value set.
///
/// Conditional: present on `logical-predicate`, where it is the two-valued
/// contract itself, and on `complex`, where it counts the ordered components.
pub const SCALAR_TYPE_FACT_VALUE_CARDINALITY: AttributeFieldId = AttributeFieldId::new(4);
/// Field naming a binary floating-point format's sign-field width in bits.
///
/// Conditional on a binary floating-point class. Zero for unsigned
/// exponent-only scale data.
pub const SCALAR_TYPE_FACT_SIGN_BITS: AttributeFieldId = AttributeFieldId::new(5);
/// Field naming a binary floating-point format's exponent-field width in bits.
///
/// Conditional on a binary floating-point class.
pub const SCALAR_TYPE_FACT_EXPONENT_BITS: AttributeFieldId = AttributeFieldId::new(6);
/// Field naming a binary floating-point format's stored fraction width in bits.
///
/// Conditional on a binary floating-point class. This is the trailing
/// significand only; any implicit leading bit is not counted.
pub const SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS: AttributeFieldId = AttributeFieldId::new(7);
/// Field carrying a binary floating-point format's exponent bias.
///
/// Conditional on Tiler holding evidence that fixes the value, which is a
/// narrower condition than the format having a bias. Absence means the pinned
/// normative reference owns the bias and this repository does not re-derive it;
/// it never means the format is unbiased.
pub const SCALAR_TYPE_FACT_EXPONENT_BIAS: AttributeFieldId = AttributeFieldId::new(8);
/// Field stating whether a binary floating-point value set contains infinities.
///
/// Conditional on a binary floating-point class.
pub const SCALAR_TYPE_FACT_HAS_INFINITIES: AttributeFieldId = AttributeFieldId::new(9);
/// Field stating whether a binary floating-point value set contains NaNs.
///
/// Conditional on a binary floating-point class.
pub const SCALAR_TYPE_FACT_HAS_NAN: AttributeFieldId = AttributeFieldId::new(10);
/// Field stating whether a binary floating-point value set contains zero.
///
/// Conditional on a binary floating-point class.
pub const SCALAR_TYPE_FACT_HAS_ZERO: AttributeFieldId = AttributeFieldId::new(11);
/// Field stating whether a binary floating-point format's zero is signed.
///
/// Conditional on a binary floating-point class.
pub const SCALAR_TYPE_FACT_HAS_SIGNED_ZERO: AttributeFieldId = AttributeFieldId::new(12);
/// Field stating whether a binary floating-point value set contains subnormals.
///
/// Conditional on a binary floating-point class. This is a fact about the
/// logical value set, never a claim that a target honours it.
pub const SCALAR_TYPE_FACT_HAS_SUBNORMALS: AttributeFieldId = AttributeFieldId::new(13);
/// Field naming a decimal format's coefficient precision in decimal digits.
///
/// Conditional on the `ieee-decimal` class.
pub const SCALAR_TYPE_FACT_COEFFICIENT_DIGITS: AttributeFieldId = AttributeFieldId::new(14);
/// Field carrying the constituent value types a compound identity composes.
///
/// Conditional on a compound class. On `complex` it is the admitted component
/// set; on `ocp-microscaling-block-scheme` it is the ordered element-code and
/// block-scale pair.
pub const SCALAR_TYPE_FACT_COMPONENT_TYPES: AttributeFieldId = AttributeFieldId::new(15);
/// Field naming the semantic order of a compound identity's components.
///
/// Conditional on a compound class.
pub const SCALAR_TYPE_FACT_COMPONENT_ORDER: AttributeFieldId = AttributeFieldId::new(16);
/// Field counting the element codes that share one block scale.
///
/// Conditional on the `ocp-microscaling-block-scheme` class.
pub const SCALAR_TYPE_FACT_BLOCK_SIZE: AttributeFieldId = AttributeFieldId::new(17);
/// Field naming how a block-scaled scheme selects the scale for its codes.
///
/// Conditional on the `ocp-microscaling-block-scheme` class.
pub const SCALAR_TYPE_FACT_SCALE_SELECTION: AttributeFieldId = AttributeFieldId::new(18);

/// Field naming how the standard constant operation treats its declared payload.
pub const CONSTANT_F32_FACT_PAYLOAD_RULE: AttributeFieldId = AttributeFieldId::new(1);

/// Field naming the rounding rule the standard `f32` arithmetic applies.
pub const ARITHMETIC_F32_FACT_ROUNDING: AttributeFieldId = AttributeFieldId::new(1);
/// Field carrying the canonical arithmetic-NaN payload the arithmetic installs.
pub const ARITHMETIC_F32_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(2);
/// Field stating whether the arithmetic may be contracted with an adjacent one.
///
/// `false` on the standard multiply and add, whose normative references name
/// them "separate" — the semantic layer's counterpart to the scalar layer's own
/// contraction fact, which is a different record and numbers it differently.
pub const ARITHMETIC_F32_FACT_CONTRACTION_PERMITTED: AttributeFieldId = AttributeFieldId::new(3);

/// Field naming the strict serial Sum's contributor fold order.
pub const SERIAL_SUM_F32_FACT_FOLD_ORDER: AttributeFieldId = AttributeFieldId::new(1);
/// Field naming the width each accumulation step is performed at.
pub const SERIAL_SUM_F32_FACT_ACCUMULATION: AttributeFieldId = AttributeFieldId::new(2);
/// Field carrying the canonical arithmetic-NaN payload the reduction installs.
pub const SERIAL_SUM_F32_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(3);

/// Field carrying a governed conformance record's stable identity.
pub const CONFORMANCE_FACT_IDENTITY: AttributeFieldId = AttributeFieldId::new(1);
/// Field carrying a governed conformance record's version.
pub const CONFORMANCE_FACT_VERSION: AttributeFieldId = AttributeFieldId::new(2);

/// Maximum declared fields in one operation-attribute schema.
pub const MAX_OPERATION_ATTRIBUTES: usize = 1_024;
/// Maximum aggregate canonical default-value bytes in one operation schema.
pub const MAX_OPERATION_SCHEMA_BYTES: usize = 1_048_576;
/// Maximum operands admitted by one bounded semantic operation schema.
pub const MAX_OPERATION_OPERANDS: u32 = 4_096;
/// Maximum results admitted by one bounded semantic operation schema.
pub const MAX_OPERATION_RESULTS: u32 = 1_024;
/// Maximum UTF-8 bytes in one stable provider diagnostic code.
pub const MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES: usize = 255;
/// Maximum UTF-8 bytes in one provider diagnostic message.
pub const MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES: usize = 4_096;
const MAX_OPERATION_RESULT_CANONICAL_BYTES: usize = 16 * 1024 * 1024;

/// Returns the governed scalar-f32 constant operation key.
#[must_use]
pub fn constant_f32_op() -> OpKey {
    governed_op("constant-f32", 1)
}

/// Returns the governed elementwise-f32 multiplication operation key.
#[must_use]
pub fn multiply_f32_op() -> OpKey {
    governed_op("multiply-f32", 1)
}

/// Returns the governed elementwise-f32 addition operation key.
#[must_use]
pub fn add_f32_op() -> OpKey {
    governed_op("add-f32", 1)
}

/// Returns the governed strict serial f32 Sum operation key.
#[must_use]
pub fn strict_serial_sum_f32_op() -> OpKey {
    governed_op("strict-serial-sum-f32", 1)
}

fn governed_op(name: &str, version: u32) -> OpKey {
    OpKey::new("tiler", name, version).expect("governed operation key is valid")
}

/// Stable namespaced identity of one atomic semantic operation family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpKey(TypeKey);

impl OpKey {
    /// Creates a validated, versioned operation key.
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

    /// Validates and retains already-owned operation-key components without copying them.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining invalid components.
    pub fn from_owned(
        namespace: String,
        name: String,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::from_owned(namespace, name, semantic_version).map(Self)
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

    pub(super) fn encoded_len(&self) -> usize {
        self.0.encoded_len()
    }
}

impl fmt::Display for OpKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Canonical attributes attached to one operation occurrence.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationAttributes(Vec<CanonicalField>);

impl OperationAttributes {
    /// Creates a field-ID-sorted bounded attribute record.
    ///
    /// Empty attributes are valid. Duplicate fields and canonical-value bound
    /// violations are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when fields are duplicated or over bounds.
    pub fn new(
        fields: impl IntoIterator<Item = CanonicalField>,
    ) -> Result<Self, TypeIdentityError> {
        let value = CanonicalValue::record(fields)?;
        let Some(fields) = value.into_record() else {
            unreachable!("record construction returns a record")
        };
        Ok(Self(fields))
    }

    /// Returns an empty attribute record.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Returns attributes in stable field-ID order.
    #[must_use]
    pub fn fields(&self) -> &[CanonicalField] {
        &self.0
    }

    /// Looks up one stable field ID.
    #[must_use]
    pub fn get(&self, id: AttributeFieldId) -> Option<&CanonicalValue> {
        self.0
            .binary_search_by_key(&id, CanonicalField::id)
            .ok()
            .map(|index| self.0[index].value())
    }

    /// Returns the collision-free canonical encoding of this attribute record.
    ///
    /// The encoding is the same length-prefixed field-ID-ordered form the
    /// semantic identity uses, so an authority outside this crate can bind an
    /// occurrence's exact attributes into its own canonical identity without
    /// re-deriving a second encoding that could disagree with this one.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalOperationAttributes {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        self.encode(&mut bytes);
        CanonicalOperationAttributes(bytes)
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.0.len());
        for field in &self.0 {
            output.extend_from_slice(&field.id().get().to_be_bytes());
            field.value().encode(output);
        }
    }

    pub(super) fn encoded_len(&self) -> usize {
        std::mem::size_of::<u64>().saturating_add(
            self.0
                .iter()
                .map(|field| std::mem::size_of::<u32>().saturating_add(field.value().encoded_len()))
                .fold(0_usize, usize::saturating_add),
        )
    }
}

/// Collision-free canonical encoding of one operation's attribute record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalOperationAttributes(Vec<u8>);

impl CanonicalOperationAttributes {
    /// Returns the canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Host-recognized canonical value category used by an attribute schema.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CanonicalValueKind {
    /// A complete resolved value type.
    Type,
    /// A Boolean.
    Bool,
    /// A signed integer.
    Signed,
    /// An unsigned integer.
    Unsigned,
    /// Exact floating-point bits with explicit format identity.
    FloatBits,
    /// Exact bytes.
    Bytes,
    /// UTF-8 text.
    Utf8,
    /// An ordered sequence.
    Sequence,
    /// A stable-field-ID record.
    Record,
}

impl CanonicalValueKind {
    fn accepts(self, value: &CanonicalValue) -> bool {
        matches!(
            (self, value.view()),
            (Self::Type, super::types::CanonicalValueView::Type(_))
                | (Self::Bool, super::types::CanonicalValueView::Bool(_))
                | (
                    Self::Unsigned,
                    super::types::CanonicalValueView::Unsigned { .. }
                )
                | (
                    Self::Signed,
                    super::types::CanonicalValueView::Signed { .. }
                )
                | (
                    Self::FloatBits,
                    super::types::CanonicalValueView::FloatBits(_)
                )
                | (Self::Bytes, super::types::CanonicalValueView::Bytes(_))
                | (Self::Utf8, super::types::CanonicalValueView::Utf8(_))
                | (
                    Self::Sequence,
                    super::types::CanonicalValueView::Sequence(_)
                )
                | (Self::Record, super::types::CanonicalValueView::Record(_))
        )
    }

    fn encode(self) -> u8 {
        match self {
            Self::Type => 1,
            Self::Bool => 2,
            Self::Signed => 3,
            Self::Unsigned => 4,
            Self::FloatBits => 5,
            Self::Bytes => 6,
            Self::Utf8 => 7,
            Self::Sequence => 8,
            Self::Record => 9,
        }
    }
}

/// One field in a host-owned canonical operation-attribute schema.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationAttributeSchema {
    id: AttributeFieldId,
    kind: CanonicalValueKind,
    required: bool,
    default: Option<CanonicalValue>,
}

impl OperationAttributeSchema {
    /// Creates one required attribute field.
    #[must_use]
    pub const fn required(id: AttributeFieldId, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: true,
            default: None,
        }
    }

    /// Creates one optional attribute field.
    #[must_use]
    pub const fn optional(id: AttributeFieldId, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: false,
            default: None,
        }
    }

    /// Creates an optional field whose explicit default is canonicalized to omission.
    ///
    /// # Errors
    ///
    /// Returns [`OperationSchemaError::AttributeDefaultKind`] when the value category differs.
    pub fn defaulted(
        id: AttributeFieldId,
        kind: CanonicalValueKind,
        default: CanonicalValue,
    ) -> Result<Self, OperationSchemaError> {
        if !kind.accepts(&default) {
            return Err(OperationSchemaError::AttributeDefaultKind { field_id: id });
        }
        Ok(Self {
            id,
            kind,
            required: false,
            default: Some(default),
        })
    }

    /// Returns the stable schema-local field ID.
    #[must_use]
    pub const fn id(&self) -> AttributeFieldId {
        self.id
    }

    /// Returns the required canonical value category.
    #[must_use]
    pub const fn kind(&self) -> CanonicalValueKind {
        self.kind
    }

    /// Returns whether the field must occur.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }

    /// Returns the schema-owned default, if explicit-default elision is enabled.
    #[must_use]
    pub const fn default(&self) -> Option<&CanonicalValue> {
        self.default.as_ref()
    }
}

/// Inclusive fixed-width arity admitted by an operation schema.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationArity {
    minimum: u32,
    maximum: u32,
}

impl OperationArity {
    /// Creates an exact arity.
    #[must_use]
    pub const fn exact(value: u32) -> Self {
        Self {
            minimum: value,
            maximum: value,
        }
    }

    /// Creates an inclusive arity range.
    ///
    /// # Errors
    ///
    /// Returns [`OperationSchemaError`] when the range is reversed.
    pub const fn inclusive(minimum: u32, maximum: u32) -> Result<Self, OperationSchemaError> {
        if minimum > maximum {
            return Err(OperationSchemaError::ReversedArity { minimum, maximum });
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns the inclusive minimum admitted arity.
    #[must_use]
    pub const fn minimum(self) -> u32 {
        self.minimum
    }

    /// Returns the inclusive maximum admitted arity.
    #[must_use]
    pub const fn maximum(self) -> u32 {
        self.maximum
    }

    /// Returns whether this contract admits exactly one arity.
    #[must_use]
    pub const fn is_exact(self) -> bool {
        self.minimum == self.maximum
    }

    fn admits(self, actual: usize) -> bool {
        u32::try_from(actual).is_ok_and(|actual| actual >= self.minimum && actual <= self.maximum)
    }

    fn encode(self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.minimum.to_be_bytes());
        output.extend_from_slice(&self.maximum.to_be_bytes());
    }
}

/// Invalid host-owned operation schema.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationSchemaError {
    /// An inclusive arity range was reversed.
    ReversedArity {
        /// Inclusive lower bound.
        minimum: u32,
        /// Inclusive upper bound.
        maximum: u32,
    },
    /// The schema could admit a zero-result operation without an effect/token model.
    ZeroResultArity,
    /// An operand or result arity exceeded the bounded semantic profile.
    ArityTooLarge {
        /// Whether the rejected arity described operands or results.
        role: OperationArityRole,
        /// Declared inclusive maximum.
        maximum: u32,
        /// Governed maximum.
        limit: u32,
    },
    /// Two attribute declarations used one field ID.
    DuplicateAttribute {
        /// Duplicated schema-local field ID.
        field_id: AttributeFieldId,
    },
    /// The schema declared too many attribute fields.
    TooManyAttributes {
        /// Actual declared field count.
        fields: usize,
    },
    /// Aggregate canonical defaults exceeded the schema byte budget.
    SchemaTooLarge {
        /// First rejected aggregate byte count.
        bytes: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// A schema default had the wrong canonical value category.
    AttributeDefaultKind {
        /// Invalid default field.
        field_id: AttributeFieldId,
    },
    /// A precondition selected an operand not present in every admitted signature.
    SemanticPreconditionOperandOutOfRange {
        /// Zero-based selected operand.
        operand: super::precondition::OperationOperandIndex,
        /// Minimum operand arity admitted by the schema.
        minimum_arity: u32,
    },
}

impl fmt::Display for OperationSchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReversedArity { minimum, maximum } => {
                write!(
                    formatter,
                    "operation arity {minimum}..={maximum} is reversed"
                )
            }
            Self::ZeroResultArity => formatter
                .write_str("zero-result operation schemas require an accepted effect/token model"),
            Self::ArityTooLarge {
                role,
                maximum,
                limit,
            } => write!(
                formatter,
                "operation {role} maximum {maximum} exceeds governed limit {limit}"
            ),
            Self::DuplicateAttribute { field_id } => {
                write!(formatter, "duplicate operation attribute field {field_id}")
            }
            Self::TooManyAttributes { fields } => write!(
                formatter,
                "operation schema has {fields} attributes, exceeding {MAX_OPERATION_ATTRIBUTES}"
            ),
            Self::SchemaTooLarge { bytes, limit } => write!(
                formatter,
                "operation schema has {bytes} canonical default-value bytes, exceeding {limit}"
            ),
            Self::AttributeDefaultKind { field_id } => {
                write!(
                    formatter,
                    "operation attribute field {field_id} has a mismatched default"
                )
            }
            Self::SemanticPreconditionOperandOutOfRange {
                operand,
                minimum_arity,
            } => write!(
                formatter,
                "semantic precondition operand {} is absent from signatures with the schema minimum arity {minimum_arity}",
                operand.get()
            ),
        }
    }
}

impl Error for OperationSchemaError {}

/// Which operation-schema arity exceeded its governed bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationArityRole {
    /// Ordered operation operands.
    Operands,
    /// Ordered operation results.
    Results,
}

impl fmt::Display for OperationArityRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operands => formatter.write_str("operand"),
            Self::Results => formatter.write_str("result"),
        }
    }
}

/// Bounded host-owned structural schema for an operation family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationSchema {
    operands: OperationArity,
    results: OperationArity,
    attributes: Vec<OperationAttributeSchema>,
}

impl OperationSchema {
    /// Creates a canonical schema, sorting fields by stable ID.
    ///
    /// # Errors
    ///
    /// Returns [`OperationSchemaError`] for duplicate attribute fields.
    pub fn new(
        operands: OperationArity,
        results: OperationArity,
        attributes: impl IntoIterator<Item = OperationAttributeSchema>,
    ) -> Result<Self, OperationSchemaError> {
        if operands.maximum > MAX_OPERATION_OPERANDS {
            return Err(OperationSchemaError::ArityTooLarge {
                role: OperationArityRole::Operands,
                maximum: operands.maximum,
                limit: MAX_OPERATION_OPERANDS,
            });
        }
        if results.minimum == 0 {
            return Err(OperationSchemaError::ZeroResultArity);
        }
        if results.maximum > MAX_OPERATION_RESULTS {
            return Err(OperationSchemaError::ArityTooLarge {
                role: OperationArityRole::Results,
                maximum: results.maximum,
                limit: MAX_OPERATION_RESULTS,
            });
        }
        let mut collected: Vec<OperationAttributeSchema> = Vec::new();
        let mut canonical_bytes = 0_usize;
        for attribute in attributes
            .into_iter()
            .take(MAX_OPERATION_ATTRIBUTES.saturating_add(1))
        {
            if collected.len() == MAX_OPERATION_ATTRIBUTES {
                return Err(OperationSchemaError::TooManyAttributes {
                    fields: MAX_OPERATION_ATTRIBUTES.saturating_add(1),
                });
            }
            if let Some(default) = &attribute.default {
                canonical_bytes = canonical_bytes.checked_add(default.encoded_len()).ok_or(
                    OperationSchemaError::SchemaTooLarge {
                        bytes: usize::MAX,
                        limit: MAX_OPERATION_SCHEMA_BYTES,
                    },
                )?;
                if canonical_bytes > MAX_OPERATION_SCHEMA_BYTES {
                    return Err(OperationSchemaError::SchemaTooLarge {
                        bytes: canonical_bytes,
                        limit: MAX_OPERATION_SCHEMA_BYTES,
                    });
                }
            }
            collected.push(attribute);
        }
        let mut attributes = collected;
        attributes.sort_unstable_by_key(|field| field.id);
        if let Some(field_id) = attributes
            .windows(2)
            .find(|pair| pair[0].id == pair[1].id)
            .map(|pair| pair[0].id)
        {
            return Err(OperationSchemaError::DuplicateAttribute { field_id });
        }
        Ok(Self {
            operands,
            results,
            attributes,
        })
    }

    /// Returns canonical attribute fields in stable field-ID order.
    #[must_use]
    pub fn attributes(&self) -> &[OperationAttributeSchema] {
        &self.attributes
    }

    /// Returns the bounded operand arity contract.
    #[must_use]
    pub const fn operands(&self) -> OperationArity {
        self.operands
    }

    /// Returns the bounded result arity contract.
    #[must_use]
    pub const fn results(&self) -> OperationArity {
        self.results
    }

    fn validate_inputs(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<(), OperationInferenceError> {
        if !self.operands.admits(operands.len()) {
            return Err(host_inference_error(
                "tiler.schema.operand-arity",
                "operand arity is outside the registered schema",
            ));
        }
        self.validate_attributes(attributes)
    }

    fn validate_attributes(
        &self,
        attributes: &OperationAttributes,
    ) -> Result<(), OperationInferenceError> {
        for field in attributes.fields() {
            let Some(schema) = self
                .attributes
                .binary_search_by_key(&field.id(), |candidate| candidate.id)
                .ok()
                .map(|index| &self.attributes[index])
            else {
                return Err(host_inference_error(
                    "tiler.schema.unknown-attribute",
                    "attribute field is absent from the registered schema",
                ));
            };
            if !schema.kind.accepts(field.value()) {
                return Err(host_inference_error(
                    "tiler.schema.attribute-kind",
                    "attribute value has the wrong canonical category",
                ));
            }
        }
        if self
            .attributes
            .iter()
            .any(|field| field.required && attributes.get(field.id).is_none())
        {
            return Err(host_inference_error(
                "tiler.schema.missing-attribute",
                "a required attribute field is absent",
            ));
        }
        Ok(())
    }

    pub(super) fn normalize_attributes(
        &self,
        attributes: &OperationAttributes,
    ) -> Result<OperationAttributes, OperationInferenceError> {
        self.validate_attributes(attributes)?;
        let fields = attributes.fields().iter().filter(|field| {
            self.attributes
                .binary_search_by_key(&field.id(), |candidate| candidate.id)
                .ok()
                .and_then(|index| self.attributes[index].default.as_ref())
                != Some(field.value())
        });
        OperationAttributes::new(fields.cloned()).map_err(|error| {
            host_inference_error("tiler.schema.attribute-normalization", error.to_string())
        })
    }

    fn resolved_attributes(
        &self,
        canonical: &OperationAttributes,
    ) -> Result<OperationAttributes, OperationInferenceError> {
        let mut fields = canonical.fields().to_vec();
        for schema in &self.attributes {
            if let Some(default) = &schema.default
                && canonical.get(schema.id).is_none()
            {
                fields.push(CanonicalField::new(schema.id, default.clone()));
            }
        }
        OperationAttributes::new(fields).map_err(|error| {
            host_inference_error("tiler.schema.attribute-resolution", error.to_string())
        })
    }

    fn validate_results(&self, results: &[ValueFact]) -> Result<(), OperationInferenceError> {
        if !self.results.admits(results.len()) {
            return Err(host_inference_error(
                "tiler.schema.result-arity",
                "inferred result arity is outside the registered schema",
            ));
        }
        Ok(())
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        self.operands.encode(output);
        self.results.encode(output);
        push_len(output, self.attributes.len());
        for field in &self.attributes {
            output.extend_from_slice(&field.id.get().to_be_bytes());
            output.push(field.kind.encode());
            output.push(match (&field.default, field.required) {
                (None, true) => 1,
                (None, false) => 2,
                (Some(_), false) => 3,
                (Some(_), true) => unreachable!("required fields cannot carry defaults"),
            });
            if let Some(default) = &field.default {
                default.encode(output);
            }
        }
    }
}

/// Bounded canonical descriptive facts owned by an operation definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationDefinitionFacts(CanonicalValue);

impl OperationDefinitionFacts {
    /// Wraps an already bounded canonical value in its definition-fact role.
    #[must_use]
    pub const fn new(value: CanonicalValue) -> Self {
        Self(value)
    }

    /// Returns the canonical value.
    #[must_use]
    pub const fn value(&self) -> &CanonicalValue {
        &self.0
    }
}

/// Algebraic laws one operation definition promises for every admitted signature.
///
/// A missing declaration is unknown, never evidence that the inverse law holds.
/// These capabilities describe algebraic structure only: consuming one still
/// requires the independently resolved numerical permission for the rewrite.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationAlgebraicCapabilities {
    ordered_associativity: bool,
}

impl OperationAlgebraicCapabilities {
    /// Creates a capability set with no declared algebraic laws.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            ordered_associativity: false,
        }
    }

    /// Declares that regrouping preserves operand order for every admitted signature.
    #[must_use]
    pub const fn with_ordered_associativity(mut self) -> Self {
        self.ordered_associativity = true;
        self
    }

    /// Returns whether ordered associativity is declared.
    #[must_use]
    pub const fn declares_ordered_associativity(&self) -> bool {
        self.ordered_associativity
    }

    pub(super) fn encode(self, output: &mut Vec<u8>) {
        push_len(output, usize::from(self.ordered_associativity));
        if self.ordered_associativity {
            output.push(0x01);
        }
    }
}

/// Bounded canonical identity of required operation conformance evidence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationConformance(CanonicalValue);

impl OperationConformance {
    /// Creates a conformance identity from bounded canonical data.
    #[must_use]
    pub const fn new(value: CanonicalValue) -> Self {
        Self(value)
    }

    /// Returns its canonical identity value.
    #[must_use]
    pub const fn value(&self) -> &CanonicalValue {
        &self.0
    }
}

/// Observable effect class of an atomic semantic operation.
///
/// Deliberately **not** `#[non_exhaustive]`, under ADR 0074's amended
/// convention 5b: three encoders outside this crate map this vocabulary
/// *totally* onto a canonical identity tag — `tiler_compiler::legality`'s and
/// `tiler_compiler::fusion_legality`'s `effect_tag`, alongside this crate's own
/// registry encoder — and no wildcard value is derivable from the variant it
/// would cover. Convention 3 requires those matches to be exhaustive with no
/// wildcard arm, which `#[non_exhaustive]` makes uncompilable across a crate
/// boundary, so where 3 and 5 meet, 3 wins. The failure the attribute would
/// buy back is one in-workspace source edit that `cargo check` enumerates; the
/// failure it would cost is two structurally distinct occurrences sharing
/// identity bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OperationEffect {
    /// Deterministic and free of externally observable side effects.
    Pure,
}

/// Required shape-inference participation for one operation definition.
///
/// Every definition carries exactly one of these. There is no default, no
/// optional policy, and no fallback between modes. Public construction is
/// always [`Self::LiteralOnly`]. Governed environment-aware construction is
/// crate-private until a separately accepted host-proof protocol exists.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ShapeInferenceParticipation {
    /// The family decides shapes over literal extents only.
    LiteralOnly,
    /// The family may consult the program's exact shape environment.
    GovernedEnvironmentAware,
}

impl ShapeInferenceParticipation {
    /// Identity tag. Written by a match so adding a mode is a build error at
    /// every encoder rather than a silent re-encoding (ADR 0074 convention 3).
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::LiteralOnly => 0x01,
            Self::GovernedEnvironmentAware => 0x02,
        }
    }
}

/// Host-owned refusal: this operation family does not infer over a symbolic extent.
///
/// Distinct from every [`ExtentSourceError`]: the environment was not asked and
/// nothing about it failed. The family has no answer for a boundary whose
/// extent is bound later. A caller remediates by supplying a literal shape or
/// by using a governed family that has been taught the question, not by
/// declaring or constraining a symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolicOperandUnsupported {
    key: OpKey,
    provider: ProviderIdentity,
    operand: u32,
    axis: Axis,
    symbol: ShapeSymbol,
}

impl SymbolicOperandUnsupported {
    pub(super) fn new(
        key: OpKey,
        provider: ProviderIdentity,
        operand: u32,
        axis: Axis,
        symbol: ShapeSymbol,
    ) -> Self {
        Self {
            key,
            provider,
            operand,
            axis,
            symbol,
        }
    }

    /// Returns the operation family that cannot infer over the symbol.
    #[must_use]
    pub const fn key(&self) -> &OpKey {
        &self.key
    }

    /// Returns the provider that admitted the family.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the zero-based operand that named the symbol.
    #[must_use]
    pub const fn operand(&self) -> u32 {
        self.operand
    }

    /// Returns the zero-based axis that named the symbol.
    #[must_use]
    pub const fn axis(&self) -> Axis {
        self.axis
    }

    /// Returns the symbol the family cannot resolve.
    #[must_use]
    pub const fn symbol(&self) -> &ShapeSymbol {
        &self.symbol
    }
}

impl fmt::Display for SymbolicOperandUnsupported {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "semantic.symbolic-operand-unsupported: {} admitted by {} decides shapes over literal extents only, and operand {} axis {} names {}",
            self.key,
            self.provider,
            self.operand,
            self.axis.get(),
            self.symbol
        )
    }
}

impl Error for SymbolicOperandUnsupported {}

/// Complete type and shape of one operand or inferred result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueFact {
    pub(super) resolved_type: ResolvedValueType,
    pub(super) shape: SourcedShape,
}

impl ValueFact {
    /// Creates one complete semantic value fact from a static shape.
    ///
    /// External providers construct only this form. A sourced result is
    /// crate-private so a public inferencer cannot mint a symbolic boundary.
    #[must_use]
    pub fn new(resolved_type: ResolvedValueType, shape: Shape) -> Self {
        Self {
            resolved_type,
            shape: SourcedShape::from(shape),
        }
    }

    /// Creates one value fact that may name a declared symbol.
    ///
    /// Crate-private: governed semantic inference is the only author of a
    /// sourced result. The public constructor stays a [`Shape`].
    #[must_use]
    pub(crate) fn from_sourced(resolved_type: ResolvedValueType, shape: SourcedShape) -> Self {
        Self {
            resolved_type,
            shape,
        }
    }

    /// Returns the complete shape-independent type.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// Returns the verified shape and where each extent's value comes from.
    ///
    /// Total over both source kinds rather than paired with an optional
    /// symbolic accessor, exactly as
    /// [`ValueRef::shape`](super::operation::ValueRef::shape) is. A rule that
    /// decides shapes only over literals reads
    /// [`OperationInferenceRequest::static_operand_shape`] instead of narrowing
    /// this itself, so its refusal is named and cannot be forgotten.
    #[must_use]
    pub const fn shape(&self) -> &SourcedShape {
        &self.shape
    }
}

/// Stable provider diagnostic from operation inference or validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationInferenceError {
    code: ProviderDiagnosticCode,
    message: String,
    contract_failure: Option<Arc<ProviderDiagnosticError>>,
    extent_source: Option<Arc<ExtentSourceError>>,
    secondary: Option<Arc<OperationInferenceError>>,
}

impl OperationInferenceError {
    /// Creates a host-owned refusal that preserves a typed environment failure.
    ///
    /// Crate-private: a public provider cannot stamp a host environment verdict.
    /// The builder and registry re-derive every [`BuildError::ExtentSource`] from
    /// their own validation or comparison.
    ///
    /// [`BuildError::ExtentSource`]: super::BuildError::ExtentSource
    #[must_use]
    pub(crate) fn from_extent_source(error: ExtentSourceError) -> Self {
        Self {
            code: provider_diagnostic_code("tiler.shape.extent-source"),
            message: error.to_string(),
            contract_failure: None,
            extent_source: Some(Arc::new(error)),
            secondary: None,
        }
    }

    /// Returns the typed shape-environment failure this refusal preserves.
    ///
    /// Crate-private: a public provider cannot inspect a host environment verdict.
    #[must_use]
    pub(crate) fn extent_source(&self) -> Option<&ExtentSourceError> {
        self.extent_source.as_deref()
    }
    /// Creates a provider-attributed rejection.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderDiagnosticError`] when the dynamic message is empty or
    /// oversized. Within an [`OperationInferencer`] callback, `?` converts that
    /// provider-contract failure into this role-specific error while preserving
    /// it as the causal [`Error::source`].
    pub fn new<'a>(
        code: ProviderDiagnosticCode,
        message: impl Into<Cow<'a, str>>,
    ) -> Result<Self, ProviderDiagnosticError> {
        let message = message.into();
        validate_provider_diagnostic_message(message.as_ref())?;
        Ok(Self {
            code,
            message: message.into_owned(),
            contract_failure: None,
            extent_source: None,
            secondary: None,
        })
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &ProviderDiagnosticCode {
        &self.code
    }

    /// Returns diagnostic detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns a malformed provider diagnostic that causally produced this error.
    #[must_use]
    pub fn provider_contract_failure(&self) -> Option<&ProviderDiagnosticError> {
        self.contract_failure.as_deref()
    }

    /// Returns a distinct later failure retained alongside the primary failure.
    ///
    /// This is not exposed as [`Error::source`] because the later diagnostic did
    /// not cause the primary failure.
    #[must_use]
    pub fn secondary(&self) -> Option<&Self> {
        self.secondary.as_deref()
    }

    fn retain_secondary(mut self, secondary: Self) -> Self {
        if self != secondary {
            self.secondary = Some(Arc::new(secondary));
        }
        self
    }
}

impl fmt::Display for OperationInferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl From<ProviderDiagnosticError> for OperationInferenceError {
    fn from(source: ProviderDiagnosticError) -> Self {
        Self {
            code: provider_diagnostic_code("tiler.provider.invalid-diagnostic"),
            message: format!("provider produced an invalid diagnostic: {source}"),
            contract_failure: Some(Arc::new(source)),
            extent_source: None,
            secondary: None,
        }
    }
}

impl Error for OperationInferenceError {
    /// Returns whichever causal failure produced this refusal.
    ///
    /// At most one is ever set: a malformed provider diagnostic and a typed
    /// extent failure arrive through different constructors, and neither
    /// constructs the other.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.contract_failure
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
            .or_else(|| {
                self.extent_source
                    .as_deref()
                    .map(|source| source as &(dyn Error + 'static))
            })
    }
}

fn host_inference_error(code: &'static str, message: impl Into<String>) -> OperationInferenceError {
    OperationInferenceError::new(provider_diagnostic_code(code), message.into())
        .expect("host diagnostic is canonical")
}

/// Validated stable code identifying one provider diagnostic class.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderDiagnosticCode(Arc<str>);

impl ProviderDiagnosticCode {
    /// Validates and retains one bounded portable diagnostic code.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderDiagnosticError`] when the code is empty, oversized,
    /// or contains a byte outside the portable grammar.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ProviderDiagnosticError> {
        let value = value.as_ref();
        validate_provider_diagnostic_code(value)?;
        Ok(Self(Arc::from(value)))
    }

    /// Returns the exact validated code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderDiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn provider_diagnostic_code(value: &'static str) -> ProviderDiagnosticCode {
    ProviderDiagnosticCode::new(value).expect("host diagnostic code is canonical")
}

/// Invalid bounded provider diagnostic data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderDiagnosticError {
    /// The stable diagnostic code was empty.
    EmptyCode,
    /// The human-readable message was empty.
    EmptyMessage,
    /// The stable diagnostic code exceeded its byte bound.
    CodeTooLong {
        /// Actual UTF-8 bytes.
        bytes: usize,
    },
    /// The human-readable message exceeded its byte bound.
    MessageTooLong {
        /// Actual UTF-8 bytes.
        bytes: usize,
    },
    /// The stable code contained a byte outside its portable grammar.
    InvalidCodeCharacter {
        /// Zero-based invalid byte position.
        byte_index: usize,
    },
}

impl fmt::Display for ProviderDiagnosticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCode => formatter.write_str("provider diagnostic code is empty"),
            Self::EmptyMessage => formatter.write_str("provider diagnostic message is empty"),
            Self::CodeTooLong { bytes } => write!(
                formatter,
                "provider diagnostic code has {bytes} bytes, exceeding {MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES}"
            ),
            Self::MessageTooLong { bytes } => write!(
                formatter,
                "provider diagnostic message has {bytes} bytes, exceeding {MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES}"
            ),
            Self::InvalidCodeCharacter { byte_index } => write!(
                formatter,
                "provider diagnostic code contains an invalid byte at position {byte_index}"
            ),
        }
    }
}

impl Error for ProviderDiagnosticError {}

fn validate_provider_diagnostic_code(code: &str) -> Result<(), ProviderDiagnosticError> {
    if code.is_empty() {
        return Err(ProviderDiagnosticError::EmptyCode);
    }
    if code.len() > MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES {
        return Err(ProviderDiagnosticError::CodeTooLong { bytes: code.len() });
    }
    if let Some((byte_index, _)) = code
        .bytes()
        .enumerate()
        .find(|(_, byte)| !byte.is_ascii_alphanumeric() && !matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(ProviderDiagnosticError::InvalidCodeCharacter { byte_index });
    }
    Ok(())
}

pub(super) fn validate_provider_diagnostic_message(
    message: &str,
) -> Result<(), ProviderDiagnosticError> {
    if message.is_empty() {
        return Err(ProviderDiagnosticError::EmptyMessage);
    }
    if message.len() > MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES {
        return Err(ProviderDiagnosticError::MessageTooLong {
            bytes: message.len(),
        });
    }
    Ok(())
}

/// Immutable host-validated inputs to one semantic inference callback.
#[derive(Clone, Copy, Debug)]
pub struct OperationInferenceRequest<'a> {
    operands: &'a [ValueFact],
    attributes: &'a OperationAttributes,
    extent_sources: Option<&'a ExtentSources>,
}

impl<'a> OperationInferenceRequest<'a> {
    fn new(
        operands: &'a [ValueFact],
        attributes: &'a OperationAttributes,
        extent_sources: Option<&'a ExtentSources>,
    ) -> Self {
        Self {
            operands,
            attributes,
            extent_sources,
        }
    }

    /// Returns operands in semantic order.
    #[must_use]
    pub const fn operands(self) -> &'a [ValueFact] {
        self.operands
    }

    /// Returns resolved canonical attributes.
    #[must_use]
    pub const fn attributes(self) -> &'a OperationAttributes {
        self.attributes
    }

    /// Returns the environment every symbolic operand extent resolves in.
    ///
    /// Crate-private for the narrow release: external providers receive only
    /// static facts. Governed built-ins read this only through the builder's
    /// environment-bound path.
    ///
    /// This is the **only** authority over whether two symbolic extents are one
    /// extent. A rule that compared spellings beside it would disagree the first
    /// time a constraint forced two differently spelled symbols together, and it
    /// would disagree in the admitting direction, which is the direction that
    /// produces a wrong program rather than a refused one.
    #[must_use]
    pub(crate) const fn extent_sources(self) -> Option<&'a ExtentSources> {
        self.extent_sources
    }

    /// Returns one operand's fixed shape after the host has preflighted symbols.
    ///
    /// The single entry point for a rule that decides shapes over literal
    /// extents only. The host refuses a symbolic operand before invoking a
    /// literal-only callback, so a symbol here is a leaked host invariant
    /// rather than a family-owned environment verdict.
    ///
    /// # Errors
    ///
    /// Returns a host diagnostic for an operand position this application does
    /// not have, and a host diagnostic if a symbolic extent reached the
    /// callback despite preflight.
    pub fn static_operand_shape(
        self,
        operand: usize,
    ) -> Result<&'a Shape, OperationInferenceError> {
        let Some(fact) = self.operands.get(operand) else {
            return Err(host_inference_error(
                "tiler.schema.operand-position",
                "inference requested an operand position this application does not have",
            ));
        };
        static_shape_of(&fact.shape)
    }
}

/// Returns the fixed shape after host preflight has refused every symbol.
fn static_shape_of(shape: &SourcedShape) -> Result<&Shape, OperationInferenceError> {
    shape.as_static().ok_or_else(|| {
        host_inference_error(
            "tiler.shape.symbolic-operand-leaked",
            "a symbolic operand reached a literal-only callback after host preflight",
        )
    })
}

/// Returns the first symbolic operand as a host-owned capability refusal.
pub(super) fn first_symbolic_operand(
    key: &OpKey,
    provider: &ProviderIdentity,
    operands: &[ValueFact],
) -> Option<SymbolicOperandUnsupported> {
    for (operand, fact) in operands.iter().enumerate() {
        if let Some((axis, symbol)) = first_symbol(fact.shape()) {
            let operand = u32::try_from(operand).expect("a bounded operand count fits u32");
            return Some(SymbolicOperandUnsupported::new(
                key.clone(),
                provider.clone(),
                operand,
                axis,
                symbol,
            ));
        }
    }
    None
}

/// Returns the outermost symbolic axis and the symbol it names.
pub(super) fn first_symbol(shape: &SourcedShape) -> Option<(Axis, ShapeSymbol)> {
    shape.extents().enumerate().find_map(|(axis, extent)| {
        let axis = u32::try_from(axis).expect("a bounded rank fits the axis space");
        extent
            .symbol()
            .map(|symbol| (Axis::new(axis), symbol.clone()))
    })
}

/// Host-owned bounded writer for ordered operation-inference results.
///
/// Tiler only accepts results successfully pushed through this writer. A
/// rejected push makes the writer sticky-failing, so ignoring the error cannot
/// produce a committable partial result list. Tiler cannot police
/// arbitrary allocation or nontermination inside provider code itself.
///
/// The writer also bounds the aggregate canonical bytes used to identify accepted
/// result facts. This bounds semantic identity work; it is not heap accounting,
/// because allocator overhead and sharing are implementation-dependent.
#[derive(Debug)]
pub struct OperationInferenceOutputs<'a> {
    results: Vec<ValueFact>,
    result_arity: OperationArity,
    remaining_canonical_bytes: usize,
    failure: Option<OperationInferenceError>,
    schema: &'a OperationSchema,
}

impl<'a> OperationInferenceOutputs<'a> {
    fn new(schema: &'a OperationSchema) -> Self {
        Self {
            results: Vec::new(),
            result_arity: schema.results,
            remaining_canonical_bytes: MAX_OPERATION_RESULT_CANONICAL_BYTES,
            failure: None,
            schema,
        }
    }

    /// Appends one inferred result in semantic order.
    ///
    /// # Errors
    ///
    /// Returns a sticky host diagnostic once the schema's result maximum or
    /// the aggregate canonical result-fact byte budget is exceeded. Later
    /// pushes return that same error without mutation.
    pub fn try_push(&mut self, fact: ValueFact) -> Result<(), OperationInferenceError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let schema_maximum = self.result_arity.maximum as usize;
        let global_maximum = MAX_OPERATION_RESULTS as usize;
        if self.results.len() >= schema_maximum || self.results.len() >= global_maximum {
            let error = host_inference_error(
                "tiler.schema.result-limit",
                "inference produced more results than the registered schema permits",
            );
            self.failure = Some(error.clone());
            return Err(error);
        }
        let fact_bytes = fact
            .resolved_type
            .canonical_encoded_len()
            .saturating_add(fact.shape.encoded_len());
        let Some(remaining_canonical_bytes) =
            self.remaining_canonical_bytes.checked_sub(fact_bytes)
        else {
            let error = host_inference_error(
                "tiler.schema.result-bytes",
                "inference results exceed the governed aggregate canonical-byte budget",
            );
            self.failure = Some(error.clone());
            return Err(error);
        };
        self.results.push(fact);
        self.remaining_canonical_bytes = remaining_canonical_bytes;
        Ok(())
    }

    fn finish(
        self,
        callback: Result<(), OperationInferenceError>,
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        if let Some(primary) = self.failure {
            return Err(match callback {
                Ok(()) => primary,
                Err(secondary) => primary.retain_secondary(secondary),
            });
        }
        callback?;
        if self.results.len() < self.result_arity.minimum as usize {
            return Err(host_inference_error(
                "tiler.schema.result-minimum",
                "inference produced fewer results than the registered schema requires",
            ));
        }
        debug_assert!(self.schema.results.admits(self.results.len()));
        Ok(self.results)
    }
}

/// Immutable synchronous semantic inference for one operation family.
///
/// Providers are trusted in-process code: the host bounds data admitted through
/// [`OperationInferenceOutputs`], but cannot bound arbitrary provider allocation,
/// execution time, or side effects. A future asynchronous or isolated provider
/// boundary would therefore be a separate contract rather than an implementation
/// detail of this trait.
pub trait OperationInferencer: Send + Sync + 'static {
    /// Validates operands and canonical attributes, then exclusively derives
    /// the ordered result facts.
    ///
    /// # Errors
    ///
    /// Returns a stable provider diagnostic for an invalid application.
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError>;
}

/// Portable semantic definition of one operation family.
#[derive(Clone)]
pub struct OperationDefinition {
    key: OpKey,
    schema: OperationSchema,
    normative_definition: NormativeDefinitionRef,
    canonical_facts: OperationDefinitionFacts,
    algebraic_capabilities: OperationAlgebraicCapabilities,
    conformance: OperationConformance,
    effect: OperationEffect,
    semantic_preconditions: SemanticPreconditionDeclarations,
    participation: ShapeInferenceParticipation,
    inferencer: Arc<dyn OperationInferencer>,
}

impl fmt::Debug for OperationDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OperationDefinition")
            .field("key", &self.key)
            .field("schema", &self.schema)
            .field("normative_definition", &self.normative_definition)
            .field("canonical_facts", &self.canonical_facts)
            .field("algebraic_capabilities", &self.algebraic_capabilities)
            .field("conformance", &self.conformance)
            .field("effect", &self.effect)
            .field("semantic_preconditions", &self.semantic_preconditions)
            .field("participation", &self.participation)
            .field("inferencer", &"OperationInferencer(..)")
            .finish()
    }
}

impl OperationDefinition {
    /// Creates a literal-only operation-family definition.
    ///
    /// Public construction always participates as literal-only.
    /// A symbolic operand is a host-owned capability refusal before the
    /// inferencer runs. Governed environment-aware construction is crate-private.
    #[must_use]
    pub fn new(
        key: OpKey,
        schema: OperationSchema,
        normative_definition: NormativeDefinitionRef,
        canonical_facts: OperationDefinitionFacts,
        conformance: OperationConformance,
        effect: OperationEffect,
        inferencer: Arc<dyn OperationInferencer>,
    ) -> Self {
        Self {
            key,
            schema,
            normative_definition,
            canonical_facts,
            algebraic_capabilities: OperationAlgebraicCapabilities::none(),
            conformance,
            effect,
            semantic_preconditions: SemanticPreconditionDeclarations::empty(),
            participation: ShapeInferenceParticipation::LiteralOnly,
            inferencer,
        }
    }

    /// Creates a governed environment-aware operation-family definition.
    ///
    /// Crate-private: only governed built-ins taught the question may consult
    /// the program's shape environment. There is no public symbolic-provider surface.
    #[must_use]
    pub(crate) fn new_governed_environment_aware(
        key: OpKey,
        schema: OperationSchema,
        normative_definition: NormativeDefinitionRef,
        canonical_facts: OperationDefinitionFacts,
        conformance: OperationConformance,
        effect: OperationEffect,
        inferencer: Arc<dyn OperationInferencer>,
    ) -> Self {
        let mut definition = Self::new(
            key,
            schema,
            normative_definition,
            canonical_facts,
            conformance,
            effect,
            inferencer,
        );
        definition.participation = ShapeInferenceParticipation::GovernedEnvironmentAware;
        definition
    }

    /// Adds the algebraic laws this definition promises for all admitted signatures.
    #[must_use]
    pub fn with_algebraic_capabilities(
        mut self,
        algebraic_capabilities: OperationAlgebraicCapabilities,
    ) -> Self {
        self.algebraic_capabilities = algebraic_capabilities;
        self
    }

    /// Adds the bounded semantic predicates required by every admitted application.
    ///
    /// # Errors
    ///
    /// Returns [`OperationSchemaError`] when a declaration selects an operand
    /// absent from a signature admitted by this definition's arity range.
    pub fn with_semantic_preconditions(
        mut self,
        semantic_preconditions: SemanticPreconditionDeclarations,
    ) -> Result<Self, OperationSchemaError> {
        if let Some(operand) = semantic_preconditions
            .as_slice()
            .iter()
            .map(super::precondition::SemanticPreconditionDeclaration::operand)
            .find(|operand| operand.get() >= self.schema.operands.minimum())
        {
            return Err(
                OperationSchemaError::SemanticPreconditionOperandOutOfRange {
                    operand,
                    minimum_arity: self.schema.operands.minimum(),
                },
            );
        }
        self.semantic_preconditions = semantic_preconditions;
        Ok(self)
    }

    /// Returns the stable operation-family key.
    #[must_use]
    pub const fn key(&self) -> &OpKey {
        &self.key
    }

    /// Returns the host-owned structural schema.
    #[must_use]
    pub const fn schema(&self) -> &OperationSchema {
        &self.schema
    }

    /// Returns its immutable normative definition reference.
    #[must_use]
    pub const fn normative_definition(&self) -> &NormativeDefinitionRef {
        &self.normative_definition
    }

    /// Returns bounded canonical semantic facts.
    #[must_use]
    pub const fn canonical_facts(&self) -> &OperationDefinitionFacts {
        &self.canonical_facts
    }

    /// Returns the operation-owned algebraic declarations.
    #[must_use]
    pub const fn algebraic_capabilities(&self) -> &OperationAlgebraicCapabilities {
        &self.algebraic_capabilities
    }

    /// Returns required conformance-evidence identity.
    #[must_use]
    pub const fn conformance(&self) -> &OperationConformance {
        &self.conformance
    }

    /// Returns the operation's semantic effect class.
    #[must_use]
    pub const fn effect(&self) -> OperationEffect {
        self.effect
    }

    /// Returns typed semantic predicates in declaration-ordinal order.
    #[must_use]
    pub const fn semantic_preconditions(&self) -> &SemanticPreconditionDeclarations {
        &self.semantic_preconditions
    }

    pub(crate) const fn participation(&self) -> ShapeInferenceParticipation {
        self.participation
    }

    pub(super) fn preflight(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<(), OperationInferenceError> {
        self.schema.validate_inputs(operands, attributes)
    }

    pub(super) fn infer(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
        extent_sources: Option<&ExtentSources>,
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        self.schema.validate_inputs(operands, attributes)?;
        let canonical = self.schema.normalize_attributes(attributes)?;
        let resolved = self.schema.resolved_attributes(&canonical)?;
        let request = OperationInferenceRequest::new(operands, &resolved, extent_sources);
        let mut outputs = OperationInferenceOutputs::new(&self.schema);
        let callback = self.inferencer.infer(request, &mut outputs);
        let results = outputs.finish(callback)?;
        self.schema.validate_results(&results)?;
        Ok(results)
    }
}

/// A zero-based result position on a semantic operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResultIndex(u32);

impl ResultIndex {
    pub(super) fn from_len(value: usize) -> Option<Self> {
        u32::try_from(value).ok().map(Self)
    }

    /// Returns the fixed-width operation-result position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ValueDefinition {
    Input {
        input_index: InputIndex,
    },
    OperationResult {
        operation: OperationIndex,
        result_index: ResultIndex,
    },
}

/// The unique definition of a semantic value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Definition {
    /// An ordered program input.
    Input {
        /// Zero-based position in the program input interface.
        input_index: InputIndex,
    },
    /// One ordered result of an operation.
    OperationResult {
        /// Defining graph-owned operation.
        operation: OperationId,
        /// Zero-based result position on that operation.
        result_index: ResultIndex,
    },
}

#[derive(Clone, Debug)]
pub(super) struct ValueData {
    pub(super) definition: ValueDefinition,
    pub(super) shape: SourcedShape,
    pub(super) resolved_type: Arc<ResolvedValueType>,
}

/// A borrowed typed value in a semantic program.
#[derive(Clone, Copy, Debug)]
pub struct ValueRef<'a> {
    pub(super) owner: GraphId,
    pub(super) index: ValueIndex,
    pub(super) value: &'a ValueData,
}

impl ValueRef<'_> {
    /// Returns the graph-owned value handle.
    #[must_use]
    pub const fn id(&self) -> ValueId {
        ValueId {
            owner: self.owner,
            index: self.index,
        }
    }

    /// Returns the value's unique definition.
    #[must_use]
    pub const fn definition(&self) -> Definition {
        match self.value.definition {
            ValueDefinition::Input { input_index } => Definition::Input { input_index },
            ValueDefinition::OperationResult {
                operation,
                result_index,
            } => Definition::OperationResult {
                operation: OperationId {
                    owner: self.owner,
                    index: operation,
                },
                result_index,
            },
        }
    }

    /// Returns the verified shape and where each extent's value comes from.
    ///
    /// Total over both source kinds rather than paired with an optional
    /// symbolic accessor: a caller that only handles literals reads
    /// [`SourcedShape::as_static`] once and refuses everything else with its own
    /// typed reason, and a third source kind is a build error here instead of a
    /// silently unhandled case.
    #[must_use]
    pub const fn shape(&self) -> &SourcedShape {
        &self.value.shape
    }

    /// Returns the complete shape-independent semantic value type.
    #[must_use]
    pub fn resolved_type(&self) -> &ResolvedValueType {
        &self.value.resolved_type
    }
}

#[derive(Clone, Debug)]
pub(super) struct OperationData {
    pub(super) key: OpKey,
    pub(super) attributes: OperationAttributes,
    pub(super) operands: Vec<ValueIndex>,
    pub(super) results: Vec<ValueIndex>,
    pub(super) semantic_preconditions: Vec<SemanticPreconditionData>,
}

/// A borrowed atomic operation in a semantic program.
#[derive(Clone, Copy, Debug)]
pub struct OperationRef<'a> {
    pub(super) owner: GraphId,
    pub(super) index: OperationIndex,
    pub(super) program: &'a ProgramData,
    pub(super) operation: &'a OperationData,
}

impl<'a> OperationRef<'a> {
    /// Returns the graph-owned operation handle.
    #[must_use]
    pub const fn id(&self) -> OperationId {
        OperationId {
            owner: self.owner,
            index: self.index,
        }
    }

    /// Returns the governed semantic operation-family key.
    #[must_use]
    pub const fn key(&self) -> &OpKey {
        &self.operation.key
    }

    /// Returns canonical attributes for this occurrence.
    #[must_use]
    pub const fn attributes(&self) -> &OperationAttributes {
        &self.operation.attributes
    }

    /// Returns operands in semantic order.
    #[must_use]
    pub fn operands(&self) -> impl ExactSizeIterator<Item = ValueId> + DoubleEndedIterator + '_ {
        self.operation
            .operands
            .iter()
            .copied()
            .map(|index| ValueId {
                owner: self.owner,
                index,
            })
    }

    /// Returns results in semantic order.
    #[must_use]
    pub fn results(&self) -> impl ExactSizeIterator<Item = ValueId> + DoubleEndedIterator + '_ {
        self.operation.results.iter().copied().map(|index| ValueId {
            owner: self.owner,
            index,
        })
    }

    /// Returns every proved or residual semantic precondition in declaration order.
    #[must_use]
    pub fn semantic_preconditions(
        self,
    ) -> impl ExactSizeIterator<Item = SemanticPreconditionRef<'a>> + DoubleEndedIterator + 'a {
        self.operation
            .semantic_preconditions
            .iter()
            .map(move |data| SemanticPreconditionRef {
                program: self.program,
                operation_index: self.index,
                data,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_capabilities_have_a_canonical_positive_tag() {
        let mut none = Vec::new();
        OperationAlgebraicCapabilities::none().encode(&mut none);
        assert_eq!(none, 0_u64.to_be_bytes());

        let mut ordered = Vec::new();
        OperationAlgebraicCapabilities::none()
            .with_ordered_associativity()
            .encode(&mut ordered);
        assert_eq!(ordered, [1_u64.to_be_bytes().as_slice(), &[0x01]].concat());
    }

    #[test]
    fn schema_defaults_have_one_canonical_identity_and_resolve_for_inference() {
        let field = AttributeFieldId::new(7);
        let default = CanonicalValue::unsigned_u32(4);
        let schema = OperationSchema::new(
            OperationArity::exact(0),
            OperationArity::exact(1),
            [OperationAttributeSchema::defaulted(
                field,
                CanonicalValueKind::Unsigned,
                default.clone(),
            )
            .unwrap()],
        )
        .unwrap();
        let omitted = OperationAttributes::empty();
        let explicit =
            OperationAttributes::new([CanonicalField::new(field, default.clone())]).unwrap();

        assert_eq!(
            schema.normalize_attributes(&omitted).unwrap(),
            schema.normalize_attributes(&explicit).unwrap()
        );
        assert_eq!(
            schema.resolved_attributes(&omitted).unwrap().get(field),
            Some(&default)
        );
    }

    #[test]
    fn schema_rejects_zero_unbounded_and_infinite_structure() {
        assert_eq!(
            OperationSchema::new(OperationArity::exact(0), OperationArity::exact(0), []),
            Err(OperationSchemaError::ZeroResultArity)
        );
        assert_eq!(
            OperationSchema::new(
                OperationArity::exact(u32::MAX),
                OperationArity::exact(1),
                [],
            ),
            Err(OperationSchemaError::ArityTooLarge {
                role: OperationArityRole::Operands,
                maximum: u32::MAX,
                limit: MAX_OPERATION_OPERANDS,
            })
        );
        assert_eq!(
            OperationSchema::new(
                OperationArity::exact(0),
                OperationArity::exact(u32::MAX),
                [],
            ),
            Err(OperationSchemaError::ArityTooLarge {
                role: OperationArityRole::Results,
                maximum: u32::MAX,
                limit: MAX_OPERATION_RESULTS,
            })
        );
        let error = OperationSchema::new(
            OperationArity::exact(0),
            OperationArity::exact(1),
            std::iter::repeat(OperationAttributeSchema::optional(
                AttributeFieldId::new(1),
                CanonicalValueKind::Bool,
            )),
        )
        .unwrap_err();
        assert_eq!(
            error,
            OperationSchemaError::TooManyAttributes {
                fields: MAX_OPERATION_ATTRIBUTES + 1,
            }
        );
    }

    #[test]
    fn schema_arity_and_diagnostics_have_forward_compatible_inspection() {
        let schema = OperationSchema::new(
            OperationArity::inclusive(1, 3).unwrap(),
            OperationArity::exact(2),
            [],
        )
        .unwrap();
        assert_eq!(schema.operands().minimum(), 1);
        assert_eq!(schema.operands().maximum(), 3);
        assert!(!schema.operands().is_exact());
        assert_eq!(schema.results().minimum(), 2);
        assert_eq!(schema.results().maximum(), 2);
        assert!(schema.results().is_exact());

        assert_eq!(
            ProviderDiagnosticCode::new(""),
            Err(ProviderDiagnosticError::EmptyCode)
        );
        assert_eq!(
            ProviderDiagnosticCode::new("provider code"),
            Err(ProviderDiagnosticError::InvalidCodeCharacter { byte_index: 8 })
        );
    }

    #[test]
    fn provider_diagnostics_are_bounded_typed_and_causally_wrapped() {
        let maximum = "a".repeat(MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES);
        let code = ProviderDiagnosticCode::new(&maximum).unwrap();
        let clone = code.clone();
        assert!(Arc::ptr_eq(&code.0, &clone.0));
        assert_eq!(code.as_str(), maximum);
        assert_eq!(code.to_string(), maximum);
        assert_eq!(
            ProviderDiagnosticCode::new("a".repeat(MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES + 1)),
            Err(ProviderDiagnosticError::CodeTooLong {
                bytes: MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES + 1,
            })
        );

        let valid = provider_diagnostic_code("test.rejection");
        assert_eq!(
            OperationInferenceError::new(valid.clone(), ""),
            Err(ProviderDiagnosticError::EmptyMessage)
        );
        assert!(
            OperationInferenceError::new(
                valid.clone(),
                "m".repeat(MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES),
            )
            .is_ok()
        );
        assert_eq!(
            OperationInferenceError::new(
                valid,
                "m".repeat(MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES + 1),
            ),
            Err(ProviderDiagnosticError::MessageTooLong {
                bytes: MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES + 1,
            })
        );

        let cause = ProviderDiagnosticError::EmptyMessage;
        let wrapped = OperationInferenceError::from(cause.clone());
        assert_eq!(wrapped.code().as_str(), "tiler.provider.invalid-diagnostic");
        assert_eq!(wrapped.provider_contract_failure(), Some(&cause));
        let source = std::error::Error::source(&wrapped).unwrap();
        assert_eq!(
            source.downcast_ref::<ProviderDiagnosticError>(),
            Some(&cause)
        );

        let provider_callback = || -> Result<(), OperationInferenceError> {
            Err(OperationInferenceError::new(
                provider_diagnostic_code("test.callback"),
                "",
            )?)
        };
        assert_eq!(
            provider_callback().unwrap_err().provider_contract_failure(),
            Some(&ProviderDiagnosticError::EmptyMessage)
        );
    }

    #[test]
    fn schema_defaults_share_one_aggregate_canonical_byte_budget() {
        let payload = vec![0_u8; MAX_OPERATION_SCHEMA_BYTES / 2 + 1];
        let error = OperationSchema::new(
            OperationArity::exact(0),
            OperationArity::exact(1),
            [
                OperationAttributeSchema::defaulted(
                    AttributeFieldId::new(1),
                    CanonicalValueKind::Bytes,
                    CanonicalValue::bytes(payload.clone()).unwrap(),
                )
                .unwrap(),
                OperationAttributeSchema::defaulted(
                    AttributeFieldId::new(2),
                    CanonicalValueKind::Bytes,
                    CanonicalValue::bytes(payload).unwrap(),
                )
                .unwrap(),
            ],
        )
        .unwrap_err();
        assert!(matches!(error, OperationSchemaError::SchemaTooLarge { .. }));
    }

    fn test_fact() -> ValueFact {
        ValueFact::new(
            ResolvedValueType::nominal(TypeKey::new("test", "scalar", 1).unwrap()),
            Shape::new([]),
        )
    }

    #[test]
    fn inference_outputs_enforce_minimum_exact_maximum_and_sticky_overflow() {
        let schema = OperationSchema::new(
            OperationArity::exact(0),
            OperationArity::inclusive(1, 3).unwrap(),
            [],
        )
        .unwrap();

        let outputs = OperationInferenceOutputs::new(&schema);
        let minimum = outputs.finish(Ok(())).unwrap_err();
        assert_eq!(minimum.code().as_str(), "tiler.schema.result-minimum");

        let mut exact = OperationInferenceOutputs::new(&schema);
        exact.try_push(test_fact()).unwrap();
        assert_eq!(exact.finish(Ok(())).unwrap().len(), 1);

        let mut maximum = OperationInferenceOutputs::new(&schema);
        for _ in 0..3 {
            maximum.try_push(test_fact()).unwrap();
        }
        assert_eq!(maximum.finish(Ok(())).unwrap().len(), 3);

        let mut overflow = OperationInferenceOutputs::new(&schema);
        for _ in 0..3 {
            overflow.try_push(test_fact()).unwrap();
        }
        let first = overflow.try_push(test_fact()).unwrap_err();
        let second = overflow.try_push(test_fact()).unwrap_err();
        assert_eq!(first, second);
        assert_eq!(overflow.finish(Ok(())).unwrap_err(), first);
    }

    #[test]
    fn writer_poison_is_primary_and_retains_a_distinct_provider_error() {
        let schema =
            OperationSchema::new(OperationArity::exact(0), OperationArity::exact(1), []).unwrap();
        let mut outputs = OperationInferenceOutputs::new(&schema);
        outputs.try_push(test_fact()).unwrap();
        let writer = outputs.try_push(test_fact()).unwrap_err();
        let provider = OperationInferenceError::new(
            provider_diagnostic_code("test.provider"),
            "provider failed later",
        )
        .unwrap();
        let combined = outputs.finish(Err(provider.clone())).unwrap_err();
        assert_eq!(combined.code(), writer.code());
        assert_eq!(combined.secondary(), Some(&provider));
        assert!(std::error::Error::source(&combined).is_none());

        let mut duplicate = OperationInferenceOutputs::new(&schema);
        duplicate.try_push(test_fact()).unwrap();
        let writer = duplicate.try_push(test_fact()).unwrap_err();
        let combined = duplicate.finish(Err(writer.clone())).unwrap_err();
        assert_eq!(combined, writer);
        assert!(combined.secondary().is_none());
    }

    #[test]
    fn provider_error_discards_successfully_staged_outputs() {
        let schema =
            OperationSchema::new(OperationArity::exact(0), OperationArity::exact(1), []).unwrap();
        let mut outputs = OperationInferenceOutputs::new(&schema);
        outputs.try_push(test_fact()).unwrap();
        let provider = OperationInferenceError::new(
            provider_diagnostic_code("test.provider"),
            "provider rejected input",
        )
        .unwrap();
        assert_eq!(outputs.finish(Err(provider.clone())), Err(provider));
    }

    #[test]
    fn inference_outputs_check_aggregate_bytes_before_schema_count() {
        let large_type = ResolvedValueType::parameterized(
            TypeKey::new("test", "large", 1).unwrap(),
            crate::semantic::TypeArguments::new([CanonicalValue::bytes_owned(vec![
                0_u8;
                1_000_000
            ])
            .unwrap()])
            .unwrap(),
        )
        .unwrap();
        let schema = OperationSchema::new(
            OperationArity::exact(0),
            OperationArity::exact(MAX_OPERATION_RESULTS),
            [],
        )
        .unwrap();
        let mut outputs = OperationInferenceOutputs::new(&schema);
        let error = loop {
            match outputs.try_push(ValueFact::new(large_type.clone(), Shape::new([]))) {
                Ok(()) => {}
                Err(error) => break error,
            }
        };
        assert_eq!(error.code().as_str(), "tiler.schema.result-bytes");
        assert!(outputs.results.len() < MAX_OPERATION_RESULTS as usize);
        assert_eq!(outputs.finish(Ok(())).unwrap_err(), error);
    }

    #[test]
    fn inference_outputs_charge_the_exact_symbolic_shape_encoding() {
        let symbol = crate::shape::ShapeSymbol::new(
            crate::shape::SymbolScope::new("s".repeat(128)).unwrap(),
            "n".repeat(128),
        )
        .unwrap();
        let shape =
            crate::shape::SourcedShape::sourced(vec![
                crate::shape::SourcedExtent::Symbol(symbol);
                4_096
            ])
            .unwrap();
        let fact = ValueFact::from_sourced(
            ResolvedValueType::nominal(TypeKey::new("test", "symbolic", 1).unwrap()),
            shape,
        );
        let fact_bytes = fact
            .resolved_type
            .canonical_encoded_len()
            .checked_add(fact.shape.encoded_len())
            .unwrap();
        let fitting = MAX_OPERATION_RESULT_CANONICAL_BYTES / fact_bytes;
        assert!(fitting < MAX_OPERATION_RESULTS as usize);

        let schema = OperationSchema::new(
            OperationArity::exact(0),
            OperationArity::exact(MAX_OPERATION_RESULTS),
            [],
        )
        .unwrap();
        let mut outputs = OperationInferenceOutputs::new(&schema);
        for _ in 0..fitting {
            outputs.try_push(fact.clone()).unwrap();
        }
        let error = outputs.try_push(fact).unwrap_err();
        assert_eq!(error.code().as_str(), "tiler.schema.result-bytes");
        assert_eq!(outputs.results.len(), fitting);
    }

    #[test]
    fn inference_trait_is_object_safe_natural_and_concurrently_callable() {
        struct Echo;
        impl OperationInferencer for Echo {
            fn infer(
                &self,
                request: OperationInferenceRequest<'_>,
                outputs: &mut OperationInferenceOutputs<'_>,
            ) -> Result<(), OperationInferenceError> {
                assert!(request.attributes().fields().is_empty());
                for operand in request.operands() {
                    outputs.try_push(operand.clone())?;
                }
                Ok(())
            }
        }

        fn assert_object_safe(_: &Arc<dyn OperationInferencer>) {}

        let inferencer: Arc<dyn OperationInferencer> = Arc::new(Echo);
        assert_object_safe(&inferencer);
        let definition = Arc::new(OperationDefinition::new(
            OpKey::new("test", "echo", 1).unwrap(),
            OperationSchema::new(
                OperationArity::inclusive(1, 3).unwrap(),
                OperationArity::inclusive(1, 3).unwrap(),
                [],
            )
            .unwrap(),
            NormativeDefinitionRef::new("test echo v1").unwrap(),
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            inferencer,
        ));
        let operands = vec![test_fact(), test_fact()];
        assert_eq!(
            definition
                .infer(&operands, &OperationAttributes::empty(), None)
                .unwrap(),
            operands
        );

        std::thread::scope(|scope| {
            for _ in 0..8 {
                let definition = Arc::clone(&definition);
                scope.spawn(move || {
                    let operand = [test_fact()];
                    assert_eq!(
                        definition
                            .infer(&operand, &OperationAttributes::empty(), None)
                            .unwrap(),
                        operand
                    );
                });
            }
        });
    }
}
