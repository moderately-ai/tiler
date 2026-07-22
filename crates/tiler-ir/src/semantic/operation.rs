use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::shape::Shape;

use super::handles::{GraphId, OperationId, OperationIndex, ValueId, ValueIndex};
use super::interface::InputIndex;
use super::registry::NormativeDefinitionRef;
use super::types::{
    AttributeFieldId, CanonicalField, CanonicalValue, ResolvedValueType, TypeIdentityError, TypeKey,
};

/// The bounded profile's canonical quiet NaN produced by arithmetic.
pub const CANONICAL_F32_ARITHMETIC_NAN_BITS: u32 = 0x7fc0_0000;
/// Stable field ID carrying exact f32 bits on the standard constant operation.
pub const F32_CONSTANT_BITS_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);
/// Stable field ID carrying canonical axes on the standard strict Sum.
pub const REDUCTION_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);
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

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(
            &u64::try_from(self.0.len())
                .expect("supported usize fits u64")
                .to_be_bytes(),
        );
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
        output.extend_from_slice(
            &u64::try_from(self.attributes.len())
                .expect("supported usize fits u64")
                .to_be_bytes(),
        );
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum OperationEffect {
    /// Deterministic and free of externally observable side effects.
    Pure,
}

/// Complete type and shape of one operand or inferred result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueFact {
    pub(super) resolved_type: ResolvedValueType,
    pub(super) shape: Shape,
}

impl ValueFact {
    /// Creates one complete semantic value fact.
    #[must_use]
    pub const fn new(resolved_type: ResolvedValueType, shape: Shape) -> Self {
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

    /// Returns the statically verified shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Stable provider diagnostic from operation inference or validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationInferenceError {
    code: ProviderDiagnosticCode,
    message: String,
    contract_failure: Option<Arc<ProviderDiagnosticError>>,
    secondary: Option<Arc<OperationInferenceError>>,
}

impl OperationInferenceError {
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
            secondary: None,
        }
    }
}

impl Error for OperationInferenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.contract_failure
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
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
}

impl<'a> OperationInferenceRequest<'a> {
    fn new(operands: &'a [ValueFact], attributes: &'a OperationAttributes) -> Self {
        Self {
            operands,
            attributes,
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
            .checked_add(std::mem::size_of::<u64>())
            .and_then(|bytes| {
                fact.shape
                    .rank()
                    .checked_mul(std::mem::size_of::<crate::shape::Extent>())
                    .and_then(|shape_bytes| bytes.checked_add(shape_bytes))
            })
            .unwrap_or(usize::MAX);
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
    conformance: OperationConformance,
    effect: OperationEffect,
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
            .field("conformance", &self.conformance)
            .field("effect", &self.effect)
            .field("inferencer", &"OperationInferencer(..)")
            .finish()
    }
}

impl OperationDefinition {
    /// Creates an immutable operation-family definition.
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
            conformance,
            effect,
            inferencer,
        }
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
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        self.schema.validate_inputs(operands, attributes)?;
        let canonical = self.schema.normalize_attributes(attributes)?;
        let resolved = self.schema.resolved_attributes(&canonical)?;
        let request = OperationInferenceRequest::new(operands, &resolved);
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
    pub(super) shape: Shape,
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

    /// Returns the statically verified shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
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
}

/// A borrowed atomic operation in a semantic program.
#[derive(Clone, Copy, Debug)]
pub struct OperationRef<'a> {
    pub(super) owner: GraphId,
    pub(super) index: OperationIndex,
    pub(super) operation: &'a OperationData,
}

impl OperationRef<'_> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
                .infer(&operands, &OperationAttributes::empty())
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
                            .infer(&operand, &OperationAttributes::empty())
                            .unwrap(),
                        operand
                    );
                });
            }
        });
    }
}
