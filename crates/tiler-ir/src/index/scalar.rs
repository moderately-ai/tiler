use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::semantic::{
    AttributeFieldId, CanonicalValue, CanonicalValueKind, CanonicalValueView,
    FrozenSemanticRegistry, MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES, NormativeDefinitionRef,
    ProviderDiagnosticCode, ProviderDiagnosticError, ProviderIdentity, RegistryError,
    ResolvedValueType, SemanticAdmissionProvenanceIdentity, SemanticDefinitionProjectionIdentity,
    SemanticRegistrySnapshotIdentity, TypeIdentityError, TypeKey,
};

use super::{
    CanonicalIndexRegionIdentity, ScalarOperationKindRef, VerifiedIndexHandleError,
    VerifiedIndexRegion,
};

const MAX_SCALAR_ATTRIBUTES: usize = 256;
const MAX_SCALAR_ARITY: usize = 4_096;
const MAX_SCALAR_DEFINITIONS: usize = 65_536;
const MAX_SCALAR_REGISTRY_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
const MAX_SCALAR_DEFINITION_PROJECTION_BYTES: usize = 8 * 1024 * 1024;

/// Stable identity of one scalar operation family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarOpKey(TypeKey);

impl ScalarOpKey {
    /// Creates a portable, versioned operation identity.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when any identity component is invalid.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, version).map(Self)
    }

    /// Validates and retains already-owned operation-key components without copying them.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining invalid components.
    pub fn from_owned(
        namespace: String,
        name: String,
        version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::from_owned(namespace, name, version).map(Self)
    }
    /// Returns the namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }
    /// Returns the name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }
    /// Returns the semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version()
    }
}

/// One bounded canonical scalar attribute record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributes(CanonicalValue);

impl ScalarAttributes {
    /// Creates attributes from a canonical record.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::AttributesNotRecord`] for any other value kind.
    pub fn new(value: CanonicalValue) -> Result<Self, ScalarRegistryError> {
        if !matches!(value.view(), CanonicalValueView::Record(_)) {
            return Err(ScalarRegistryError::AttributesNotRecord);
        }
        Ok(Self(value))
    }
    /// Creates an empty record.
    ///
    /// # Panics
    ///
    /// Panics only if the semantic canonical-value implementation rejects an empty record.
    #[must_use]
    pub fn empty() -> Self {
        Self(CanonicalValue::record([]).expect("empty record is valid"))
    }
    /// Returns the canonical value.
    #[must_use]
    pub const fn value(&self) -> &CanonicalValue {
        &self.0
    }
}

/// One scalar attribute-schema field.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributeField {
    id: AttributeFieldId,
    kind: CanonicalValueKind,
    required: bool,
    default: Option<CanonicalValue>,
}

impl ScalarAttributeField {
    /// Creates one required field.
    #[must_use]
    pub const fn required(id: AttributeFieldId, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: true,
            default: None,
        }
    }
    /// Creates one optional field without a default.
    #[must_use]
    pub const fn optional(id: AttributeFieldId, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: false,
            default: None,
        }
    }
    /// Creates an optional field whose explicit default canonicalizes to omission.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::AttributeDefaultKind`] for a category mismatch.
    pub fn defaulted(
        id: AttributeFieldId,
        kind: CanonicalValueKind,
        default: CanonicalValue,
    ) -> Result<Self, ScalarRegistryError> {
        if canonical_kind(&default) != kind {
            return Err(ScalarRegistryError::AttributeDefaultKind { id });
        }
        Ok(Self {
            id,
            kind,
            required: false,
            default: Some(default),
        })
    }
    /// Returns the stable field ID.
    #[must_use]
    pub const fn id(&self) -> AttributeFieldId {
        self.id
    }
    /// Returns the required canonical value kind.
    #[must_use]
    pub const fn kind(&self) -> CanonicalValueKind {
        self.kind
    }
    /// Returns whether the field must be present.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }
    /// Returns the schema-owned default, if any.
    #[must_use]
    pub const fn default(&self) -> Option<&CanonicalValue> {
        self.default.as_ref()
    }
}

/// Bounded field-ID ordered scalar attribute schema.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributeSchema(Vec<ScalarAttributeField>);

impl ScalarAttributeSchema {
    /// Creates a checked schema.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema exceeds its bound or repeats a field ID.
    pub fn new(
        fields: impl IntoIterator<Item = ScalarAttributeField>,
    ) -> Result<Self, ScalarRegistryError> {
        let mut collected = Vec::new();
        for field in fields {
            if collected.len() == MAX_SCALAR_ATTRIBUTES {
                return Err(ScalarRegistryError::TooManyAttributeFields {
                    actual: MAX_SCALAR_ATTRIBUTES + 1,
                });
            }
            collected.push(field);
        }
        collected.sort_by_key(|field| field.id);
        if collected.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(ScalarRegistryError::DuplicateAttributeField);
        }
        Ok(Self(collected))
    }
    /// Returns an empty schema.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }
    /// Returns fields in stable field-ID order.
    #[must_use]
    pub fn fields(&self) -> &[ScalarAttributeField] {
        &self.0
    }
    fn validate(&self, attributes: &ScalarAttributes) -> Result<(), ScalarRegistryError> {
        let CanonicalValueView::Record(values) = attributes.0.view() else {
            return Err(ScalarRegistryError::AttributesNotRecord);
        };
        for value in values {
            let Some(field) = self.0.iter().find(|field| field.id == value.id()) else {
                return Err(ScalarRegistryError::UnknownAttribute { id: value.id() });
            };
            if canonical_kind(value.value()) != field.kind {
                return Err(ScalarRegistryError::AttributeKind { id: value.id() });
            }
        }
        for field in &self.0 {
            if field.required && !values.iter().any(|value| value.id() == field.id) {
                return Err(ScalarRegistryError::MissingAttribute { id: field.id });
            }
        }
        Ok(())
    }

    fn normalize(
        &self,
        attributes: &ScalarAttributes,
    ) -> Result<ScalarAttributes, ScalarRegistryError> {
        self.validate(attributes)?;
        let CanonicalValueView::Record(values) = attributes.value().view() else {
            return Err(ScalarRegistryError::AttributesNotRecord);
        };
        let fields = values.iter().filter(|field| {
            self.0
                .binary_search_by_key(&field.id(), ScalarAttributeField::id)
                .ok()
                .and_then(|index| self.0[index].default.as_ref())
                != Some(field.value())
        });
        let value = CanonicalValue::record(fields.cloned())
            .map_err(|error| ScalarRegistryError::CanonicalAttributes(Arc::new(error)))?;
        ScalarAttributes::new(value)
    }

    fn resolve_defaults(
        &self,
        canonical: &ScalarAttributes,
    ) -> Result<ScalarAttributes, ScalarRegistryError> {
        let CanonicalValueView::Record(values) = canonical.value().view() else {
            return Err(ScalarRegistryError::AttributesNotRecord);
        };
        let mut fields = values.to_vec();
        for schema in &self.0 {
            if let Some(default) = &schema.default
                && !values.iter().any(|field| field.id() == schema.id)
            {
                fields.push(crate::semantic::CanonicalField::new(
                    schema.id,
                    default.clone(),
                ));
            }
        }
        let value = CanonicalValue::record(fields)
            .map_err(|error| ScalarRegistryError::CanonicalAttributes(Arc::new(error)))?;
        ScalarAttributes::new(value)
    }
}

fn canonical_kind(value: &CanonicalValue) -> CanonicalValueKind {
    match value.view() {
        CanonicalValueView::Type(_) => CanonicalValueKind::Type,
        CanonicalValueView::Bool(_) => CanonicalValueKind::Bool,
        CanonicalValueView::Signed { .. } => CanonicalValueKind::Signed,
        CanonicalValueView::Unsigned { .. } => CanonicalValueKind::Unsigned,
        CanonicalValueView::FloatBits(_) => CanonicalValueKind::FloatBits,
        CanonicalValueView::Bytes(_) => CanonicalValueKind::Bytes,
        CanonicalValueView::Utf8(_) => CanonicalValueKind::Utf8,
        CanonicalValueView::Sequence(_) => CanonicalValueKind::Sequence,
        CanonicalValueView::Record(_) => CanonicalValueKind::Record,
    }
}

/// Inclusive operand or result arity bounds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArity {
    min: usize,
    max: usize,
}

impl ScalarArity {
    /// Creates inclusive bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::InvalidArityRange`] for reversed or oversized bounds.
    pub fn range(min: usize, max: usize) -> Result<Self, ScalarRegistryError> {
        if min > max || max > MAX_SCALAR_ARITY {
            return Err(ScalarRegistryError::InvalidArityRange);
        }
        Ok(Self { min, max })
    }
    /// Creates an exact arity.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::InvalidArityRange`] when `count` exceeds the bound.
    pub fn exact(count: usize) -> Result<Self, ScalarRegistryError> {
        Self::range(count, count)
    }
    /// Returns the inclusive minimum arity.
    #[must_use]
    pub const fn min(self) -> usize {
        self.min
    }
    /// Returns the inclusive maximum arity.
    #[must_use]
    pub const fn max(self) -> usize {
        self.max
    }
    /// Returns whether `actual` satisfies these bounds.
    #[must_use]
    pub fn accepts(self, actual: usize) -> bool {
        (self.min..=self.max).contains(&actual)
    }
}

/// Immutable input passed once to a provider inferencer during construction.
#[derive(Clone, Copy, Debug)]
pub struct ScalarInferenceRequest<'a> {
    operands: &'a [ResolvedValueType],
    attributes: &'a ScalarAttributes,
}

impl<'a> ScalarInferenceRequest<'a> {
    const fn new(operands: &'a [ResolvedValueType], attributes: &'a ScalarAttributes) -> Self {
        Self {
            operands,
            attributes,
        }
    }

    /// Returns operand types in semantic order.
    #[must_use]
    pub const fn operands(self) -> &'a [ResolvedValueType] {
        self.operands
    }

    /// Returns resolved canonical attributes.
    #[must_use]
    pub const fn attributes(self) -> &'a ScalarAttributes {
        self.attributes
    }
}

/// Stable provider rejection of one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarInferenceError {
    code: ProviderDiagnosticCode,
    message: String,
}

impl ScalarInferenceError {
    /// Creates an inference rejection.
    ///
    /// # Errors
    ///
    /// Returns a provider-diagnostic contract error for an empty or oversized message.
    pub fn new(
        code: ProviderDiagnosticCode,
        message: impl Into<String>,
    ) -> Result<Self, ProviderDiagnosticError> {
        let message = message.into();
        if message.is_empty() {
            return Err(ProviderDiagnosticError::EmptyMessage);
        }
        if message.len() > MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES {
            return Err(ProviderDiagnosticError::MessageTooLong {
                bytes: message.len(),
            });
        }
        Ok(Self { code, message })
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
}
impl fmt::Display for ScalarInferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl Error for ScalarInferenceError {}

/// Complete provider-attributed rejection of one scalar application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarApplicationRejection {
    key: ScalarOpKey,
    provider: ProviderIdentity,
    source: ScalarInferenceError,
}

impl ScalarApplicationRejection {
    /// Returns the rejected scalar operation family.
    #[must_use]
    pub const fn key(&self) -> &ScalarOpKey {
        &self.key
    }

    /// Returns the provider governing the rejected definition.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the provider's bounded diagnostic.
    #[must_use]
    pub const fn rejection(&self) -> &ScalarInferenceError {
        &self.source
    }
}

impl fmt::Display for ScalarApplicationRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "scalar operation {:?} rejected by {}: {}",
            self.key, self.provider, self.source
        )
    }
}

impl Error for ScalarApplicationRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

fn host_inference_error(code: &'static str, message: &'static str) -> ScalarInferenceError {
    ScalarInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("host diagnostic code is canonical"),
        message,
    )
    .expect("host diagnostic message is canonical")
}

/// Host-owned bounded writer for ordered scalar inference results.
///
/// A rejected push permanently poisons the writer. Ignoring the returned error
/// therefore cannot commit a truncated result list.
#[derive(Debug)]
pub struct ScalarInferenceOutputs {
    results: Vec<ResolvedValueType>,
    contract_maximum: usize,
    host_result_slots: usize,
    result_count_before: usize,
    result_limit: usize,
    remaining_canonical_bytes: usize,
    initial_canonical_bytes: usize,
    retained_bytes_before: usize,
    retained_byte_limit: usize,
    per_result_overhead: usize,
    byte_multiplier: usize,
    provider_failure: Option<ScalarInferenceError>,
    host_failure: Option<ScalarInferenceHostFailure>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScalarInferenceCapacity {
    pub result_slots: usize,
    pub result_count_before: usize,
    pub result_limit: usize,
    pub retained_bytes: usize,
    pub retained_bytes_before: usize,
    pub retained_byte_limit: usize,
    pub per_result_overhead: usize,
    pub byte_multiplier: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ScalarInferenceHostFailure {
    ResultSlots { actual: usize, limit: usize },
    CanonicalBytes { actual: usize, limit: usize },
}

#[derive(Debug, Eq, PartialEq)]
enum ScalarInferenceFinishError {
    Provider(ScalarInferenceError),
    Host(ScalarInferenceHostFailure),
}

#[derive(Debug)]
pub(super) enum ScalarApplyError {
    Authority(ScalarRegistryError),
    Host(ScalarInferenceHostFailure),
}

impl ScalarInferenceOutputs {
    fn new(maximum: usize, capacity: ScalarInferenceCapacity) -> Self {
        Self {
            results: Vec::new(),
            contract_maximum: maximum,
            host_result_slots: capacity.result_slots,
            result_count_before: capacity.result_count_before,
            result_limit: capacity.result_limit,
            remaining_canonical_bytes: capacity.retained_bytes,
            initial_canonical_bytes: capacity.retained_bytes,
            retained_bytes_before: capacity.retained_bytes_before,
            retained_byte_limit: capacity.retained_byte_limit,
            per_result_overhead: capacity.per_result_overhead,
            byte_multiplier: capacity.byte_multiplier,
            provider_failure: None,
            host_failure: None,
        }
    }

    /// Appends one inferred result in semantic order.
    ///
    /// # Errors
    ///
    /// Returns a sticky host diagnostic after the registered result maximum or
    /// aggregate canonical-byte budget is exceeded.
    pub fn try_push(&mut self, value_type: ResolvedValueType) -> Result<(), ScalarInferenceError> {
        if let Some(failure) = self.host_failure {
            return Err(host_failure_diagnostic(failure));
        }
        if let Some(error) = &self.provider_failure {
            return Err(error.clone());
        }
        if self.results.len() >= self.contract_maximum {
            let error = host_inference_error(
                "tiler.scalar.result-limit",
                "scalar inference produced more results than its registered contract permits",
            );
            self.provider_failure = Some(error.clone());
            return Err(error);
        }
        if self.results.len() >= self.host_result_slots {
            let failure = ScalarInferenceHostFailure::ResultSlots {
                actual: self
                    .result_count_before
                    .saturating_add(self.results.len())
                    .saturating_add(1),
                limit: self.result_limit,
            };
            self.host_failure = Some(failure);
            return Err(host_failure_diagnostic(failure));
        }
        let bytes = value_type
            .canonical_encoding()
            .as_bytes()
            .len()
            .saturating_add(self.per_result_overhead)
            .saturating_mul(self.byte_multiplier);
        let Some(remaining) = self.remaining_canonical_bytes.checked_sub(bytes) else {
            let consumed = self
                .retained_bytes_before
                .saturating_add(
                    self.initial_canonical_bytes
                        .saturating_sub(self.remaining_canonical_bytes),
                )
                .saturating_add(bytes);
            let failure = ScalarInferenceHostFailure::CanonicalBytes {
                actual: consumed,
                limit: self.retained_byte_limit,
            };
            self.host_failure = Some(failure);
            return Err(host_failure_diagnostic(failure));
        };
        self.results.push(value_type);
        self.remaining_canonical_bytes = remaining;
        Ok(())
    }

    fn finish(
        self,
        callback: Result<(), ScalarInferenceError>,
        minimum: usize,
    ) -> Result<Vec<ResolvedValueType>, ScalarInferenceFinishError> {
        if let Some(failure) = self.host_failure {
            return Err(ScalarInferenceFinishError::Host(failure));
        }
        if let Some(error) = self.provider_failure {
            return Err(ScalarInferenceFinishError::Provider(error));
        }
        callback.map_err(ScalarInferenceFinishError::Provider)?;
        if self.results.len() < minimum {
            return Err(ScalarInferenceFinishError::Provider(host_inference_error(
                "tiler.scalar.result-minimum",
                "scalar inference produced fewer results than its registered contract requires",
            )));
        }
        Ok(self.results)
    }
}

fn host_failure_diagnostic(failure: ScalarInferenceHostFailure) -> ScalarInferenceError {
    match failure {
        ScalarInferenceHostFailure::ResultSlots { .. } => host_inference_error(
            "tiler.scalar.host-result-capacity",
            "scalar inference exceeds the enclosing graph's result capacity",
        ),
        ScalarInferenceHostFailure::CanonicalBytes { .. } => host_inference_error(
            "tiler.scalar.host-byte-capacity",
            "scalar inference exceeds the enclosing graph's canonical-byte capacity",
        ),
    }
}

/// Pure construction-time result-type inference.
pub trait ScalarOperationInferencer: Send + Sync + 'static {
    /// Infers all ordered result types.
    ///
    /// # Errors
    ///
    /// Returns a stable provider error when the operand types or attributes are unsupported.
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError>;
}

/// Host-enforced effect contract for scalar operations admitted to CSE.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ScalarEffect {
    /// Deterministic, side-effect-free semantics depending only on explicit inputs.
    Pure,
}

/// Declarative, provider-independent contract of one scalar operation family.
#[derive(Clone, Debug)]
pub struct ScalarOperationContract {
    attributes: ScalarAttributeSchema,
    operands: ScalarArity,
    results: ScalarArity,
    effect: ScalarEffect,
    facts: CanonicalValue,
    conformance: CanonicalValue,
}

impl ScalarOperationContract {
    /// Creates a complete additive scalar operation contract.
    #[must_use]
    pub const fn new(
        attributes: ScalarAttributeSchema,
        operands: ScalarArity,
        results: ScalarArity,
        effect: ScalarEffect,
        facts: CanonicalValue,
        conformance: CanonicalValue,
    ) -> Self {
        Self {
            attributes,
            operands,
            results,
            effect,
            facts,
            conformance,
        }
    }

    /// Returns the canonical attribute schema.
    #[must_use]
    pub const fn attributes(&self) -> &ScalarAttributeSchema {
        &self.attributes
    }

    /// Returns admitted operand arity.
    #[must_use]
    pub const fn operands(&self) -> ScalarArity {
        self.operands
    }

    /// Returns admitted result arity.
    #[must_use]
    pub const fn results(&self) -> ScalarArity {
        self.results
    }

    /// Returns the effect contract.
    #[must_use]
    pub const fn effect(&self) -> ScalarEffect {
        self.effect
    }

    /// Returns canonical semantic facts.
    #[must_use]
    pub const fn facts(&self) -> &CanonicalValue {
        &self.facts
    }

    /// Returns canonical conformance requirements.
    #[must_use]
    pub const fn conformance(&self) -> &CanonicalValue {
        &self.conformance
    }
}

/// Provider-independent portable scalar operation definition.
#[derive(Clone)]
pub struct ScalarOperationDefinition {
    key: ScalarOpKey,
    normative_definition: NormativeDefinitionRef,
    attributes: ScalarAttributeSchema,
    operands: ScalarArity,
    results: ScalarArity,
    effect: ScalarEffect,
    facts: CanonicalValue,
    conformance: CanonicalValue,
    inferencer: Arc<dyn ScalarOperationInferencer>,
}

impl fmt::Debug for ScalarOperationDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarOperationDefinition")
            .field("key", &self.key)
            .field("normative_definition", &self.normative_definition)
            .field("attributes", &self.attributes)
            .field("operands", &self.operands)
            .field("results", &self.results)
            .field("effect", &self.effect)
            .field("facts", &self.facts)
            .field("conformance", &self.conformance)
            .finish_non_exhaustive()
    }
}

impl ScalarOperationDefinition {
    /// Creates a complete definition. The host validates every application.
    #[must_use]
    pub fn new(
        key: ScalarOpKey,
        normative_definition: NormativeDefinitionRef,
        contract: ScalarOperationContract,
        inferencer: Arc<dyn ScalarOperationInferencer>,
    ) -> Self {
        Self {
            key,
            normative_definition,
            attributes: contract.attributes,
            operands: contract.operands,
            results: contract.results,
            effect: contract.effect,
            facts: contract.facts,
            conformance: contract.conformance,
            inferencer,
        }
    }
    /// Returns the operation key.
    #[must_use]
    pub const fn key(&self) -> &ScalarOpKey {
        &self.key
    }
    /// Returns the host-enforced effect contract.
    #[must_use]
    pub const fn effect(&self) -> ScalarEffect {
        self.effect
    }
    /// Returns the stable normative definition identity.
    #[must_use]
    pub const fn normative_definition(&self) -> &NormativeDefinitionRef {
        &self.normative_definition
    }
    /// Returns the canonical attribute schema.
    #[must_use]
    pub const fn attributes(&self) -> &ScalarAttributeSchema {
        &self.attributes
    }
    /// Returns admitted operand arity.
    #[must_use]
    pub const fn operands(&self) -> ScalarArity {
        self.operands
    }
    /// Returns admitted result arity.
    #[must_use]
    pub const fn results(&self) -> ScalarArity {
        self.results
    }
    /// Returns canonical semantic facts.
    #[must_use]
    pub const fn facts(&self) -> &CanonicalValue {
        &self.facts
    }
    /// Returns canonical conformance requirements.
    #[must_use]
    pub const fn conformance(&self) -> &CanonicalValue {
        &self.conformance
    }
}

#[derive(Clone)]
struct RegisteredScalarOperation {
    definition: ScalarOperationDefinition,
    provider: ProviderIdentity,
}

/// Failure while defining or applying scalar authority.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ScalarRegistryError {
    /// Scalar attributes were not encoded as a canonical record.
    AttributesNotRecord,
    /// An attribute schema exceeded its governed field count.
    TooManyAttributeFields {
        /// Supplied field count.
        actual: usize,
    },
    /// An attribute schema repeated a field ID.
    DuplicateAttributeField,
    /// An arity range was reversed or exceeded its governed maximum.
    InvalidArityRange,
    /// A registered definition admitted zero results.
    ZeroResultDefinition,
    /// The operation key was already registered.
    DuplicateDefinition {
        /// Duplicated key.
        key: ScalarOpKey,
    },
    /// No operation definition exists for the requested key.
    UnknownOperation {
        /// Unknown key.
        key: ScalarOpKey,
    },
    /// Operand count violates the registered arity.
    OperandArity {
        /// Applied operation key.
        key: ScalarOpKey,
        /// Supplied operand count.
        actual: usize,
    },
    /// Inferred result count violates the registered arity.
    ResultArity {
        /// Applied operation key.
        key: ScalarOpKey,
        /// Inferred result count.
        actual: usize,
    },
    /// Attributes contained an undeclared field.
    UnknownAttribute {
        /// Undeclared field ID.
        id: AttributeFieldId,
    },
    /// Attributes omitted a required field.
    MissingAttribute {
        /// Missing field ID.
        id: AttributeFieldId,
    },
    /// An attribute value did not match its declared kind.
    AttributeKind {
        /// Mismatched field ID.
        id: AttributeFieldId,
    },
    /// A schema default had the wrong canonical value category.
    AttributeDefaultKind {
        /// Invalid default field.
        id: AttributeFieldId,
    },
    /// Canonical attribute normalization failed.
    CanonicalAttributes(Arc<TypeIdentityError>),
    /// A stored application retained an explicit schema default or otherwise noncanonical record.
    NonCanonicalAttributes {
        /// Scalar operation carrying the record.
        key: ScalarOpKey,
    },
    /// An opaque verified region exposed an internally inconsistent handle.
    InvalidVerifiedRegionHandle(VerifiedIndexHandleError),
    /// The semantic type authority rejected embedded or inferred type data.
    TypeAuthority(Arc<RegistryError>),
    /// The operation-specific inferencer rejected an application.
    Inference(Arc<ScalarApplicationRejection>),
    /// The registry exceeded its governed definition count.
    DefinitionCountLimit {
        /// Attempted definition count.
        actual: usize,
        /// Maximum definition count.
        limit: usize,
    },
    /// A reached-definition projection exceeded its governed byte count.
    ProjectionByteLimit {
        /// Attempted projection bytes.
        actual: usize,
        /// Maximum projection bytes.
        limit: usize,
    },
    /// A reached-definition projection exceeded its governed definition count.
    ProjectionDefinitionCountLimit {
        /// Attempted distinct reached-definition count.
        actual: usize,
        /// Maximum distinct reached-definition count.
        limit: usize,
    },
    /// The registry exceeded its aggregate canonical definition-byte limit.
    RegistryByteLimit {
        /// Attempted aggregate canonical bytes.
        actual: usize,
        /// Maximum aggregate canonical bytes.
        limit: usize,
    },
    /// Revalidation inferred a result type different from the stored structural result.
    RevalidatedResultTypeMismatch {
        /// Scalar operation being revalidated.
        key: ScalarOpKey,
        /// Ordered result position.
        position: usize,
        /// Type stored by the region.
        stored: Arc<ResolvedValueType>,
        /// Type inferred by the selected authority.
        inferred: Arc<ResolvedValueType>,
    },
    /// Revalidation inferred a different number of ordered results.
    RevalidatedResultArity {
        /// Scalar operation being revalidated.
        key: ScalarOpKey,
        /// Result count stored by the region.
        stored: usize,
        /// Result count inferred by the selected authority.
        inferred: usize,
    },
}
impl fmt::Display for ScalarRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for ScalarRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalAttributes(source) => Some(source.as_ref()),
            Self::InvalidVerifiedRegionHandle(source) => Some(source),
            Self::TypeAuthority(source) => Some(source.as_ref()),
            Self::Inference(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Mutable scalar authority composed with an exact semantic type authority.
pub struct ScalarRegistryBuilder {
    semantic: FrozenSemanticRegistry,
    definitions: BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
    canonical_bytes: usize,
}

impl ScalarRegistryBuilder {
    /// Creates an empty builder. Empty snapshots support load/copy-only regions.
    #[must_use]
    pub fn new(semantic: FrozenSemanticRegistry) -> Self {
        Self {
            semantic,
            definitions: BTreeMap::new(),
            canonical_bytes: 0,
        }
    }
    /// Registers one definition with separate admission provenance.
    ///
    /// # Errors
    ///
    /// Returns an error for duplicate or invalid definitions and unknown embedded types.
    pub fn register(
        &mut self,
        provider: ProviderIdentity,
        definition: ScalarOperationDefinition,
    ) -> Result<(), ScalarRegistryError> {
        let key = definition.key.clone();
        if self.definitions.contains_key(&key) {
            return Err(ScalarRegistryError::DuplicateDefinition { key });
        }
        if self.definitions.len() >= MAX_SCALAR_DEFINITIONS {
            return Err(ScalarRegistryError::DefinitionCountLimit {
                actual: self.definitions.len().saturating_add(1),
                limit: MAX_SCALAR_DEFINITIONS,
            });
        }
        if definition.results.min == 0 {
            return Err(ScalarRegistryError::ZeroResultDefinition);
        }
        for field in definition.attributes.fields() {
            if let Some(default) = field.default() {
                validate_canonical_types(&self.semantic, default)?;
            }
        }
        validate_canonical_types(&self.semantic, &definition.facts)?;
        validate_canonical_types(&self.semantic, &definition.conformance)?;
        let definition_bytes = encoded_definition_len(&definition);
        let actual = self.canonical_bytes.saturating_add(definition_bytes);
        if actual > MAX_SCALAR_REGISTRY_CANONICAL_BYTES {
            return Err(ScalarRegistryError::RegistryByteLimit {
                actual,
                limit: MAX_SCALAR_REGISTRY_CANONICAL_BYTES,
            });
        }
        self.definitions.insert(
            key,
            RegisteredScalarOperation {
                definition,
                provider,
            },
        );
        self.canonical_bytes = actual;
        Ok(())
    }
    /// Freezes this exact snapshot.
    #[must_use]
    pub fn freeze(self) -> FrozenScalarRegistry {
        let snapshot = compute_scalar_snapshot_identity(&self.definitions);
        FrozenScalarRegistry(Arc::new(ScalarRegistryData {
            semantic: self.semantic,
            definitions: self.definitions,
            snapshot,
        }))
    }
}

struct ScalarRegistryData {
    semantic: FrozenSemanticRegistry,
    definitions: BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
    snapshot: CanonicalScalarRegistrySnapshotIdentity,
}

/// Immutable scalar authority. Dynamic callbacks run only while constructing SSA.
#[derive(Clone)]
pub struct FrozenScalarRegistry(Arc<ScalarRegistryData>);

impl fmt::Debug for FrozenScalarRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrozenScalarRegistry")
            .field("definition_count", &self.0.definitions.len())
            .finish()
    }
}

impl FrozenScalarRegistry {
    /// Returns complete scalar-registry snapshot provenance.
    #[must_use]
    pub fn snapshot_identity(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        &self.0.snapshot
    }
    pub(super) fn validate_type(
        &self,
        value: &ResolvedValueType,
    ) -> Result<(), ScalarRegistryError> {
        self.0
            .semantic
            .validate_type(value)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))
    }

    pub(super) fn minimum_results(&self, key: &ScalarOpKey) -> Result<usize, ScalarRegistryError> {
        self.0
            .definitions
            .get(key)
            .map(|registered| registered.definition.results.min())
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })
    }

    pub(super) fn infer(
        &self,
        key: &ScalarOpKey,
        operands: &[ResolvedValueType],
        attributes: &ScalarAttributes,
        capacity: ScalarInferenceCapacity,
    ) -> Result<Vec<ResolvedValueType>, ScalarApplyError> {
        let registered = self
            .0
            .definitions
            .get(key)
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })
            .map_err(ScalarApplyError::Authority)?;
        let definition = &registered.definition;
        if !definition.operands.accepts(operands.len()) {
            return Err(ScalarApplyError::Authority(
                ScalarRegistryError::OperandArity {
                    key: key.clone(),
                    actual: operands.len(),
                },
            ));
        }
        let canonical = definition
            .attributes
            .normalize(attributes)
            .map_err(ScalarApplyError::Authority)?;
        let resolved = definition
            .attributes
            .resolve_defaults(&canonical)
            .map_err(ScalarApplyError::Authority)?;
        validate_canonical_types(&self.0.semantic, attributes.value())
            .map_err(ScalarApplyError::Authority)?;
        for operand in operands {
            self.validate_type(operand)
                .map_err(ScalarApplyError::Authority)?;
        }
        let request = ScalarInferenceRequest::new(operands, &resolved);
        let mut outputs = ScalarInferenceOutputs::new(definition.results.max(), capacity);
        let callback = definition.inferencer.infer(request, &mut outputs);
        let results = outputs
            .finish(callback, definition.results.min())
            .map_err(|error| match error {
                ScalarInferenceFinishError::Provider(source) => ScalarApplyError::Authority(
                    ScalarRegistryError::Inference(Arc::new(ScalarApplicationRejection {
                        key: key.clone(),
                        provider: registered.provider.clone(),
                        source,
                    })),
                ),
                ScalarInferenceFinishError::Host(failure) => ScalarApplyError::Host(failure),
            })?;
        for result in &results {
            self.validate_type(result)
                .map_err(ScalarApplyError::Authority)?;
        }
        Ok(results)
    }

    pub(super) fn normalize_attributes(
        &self,
        key: &ScalarOpKey,
        attributes: ScalarAttributes,
    ) -> Result<ScalarAttributes, ScalarRegistryError> {
        let registered = self
            .0
            .definitions
            .get(key)
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
        let canonical = registered.definition.attributes.normalize(&attributes);
        drop(attributes);
        canonical
    }

    /// Returns admission provenance for diagnostics; it is not structural IR identity.
    #[must_use]
    pub fn provider(&self, key: &ScalarOpKey) -> Option<&ProviderIdentity> {
        self.0.definitions.get(key).map(|entry| &entry.provider)
    }

    /// Returns one provider-independent scalar definition.
    #[must_use]
    pub fn definition(&self, key: &ScalarOpKey) -> Option<&ScalarOperationDefinition> {
        self.0.definitions.get(key).map(|entry| &entry.definition)
    }

    /// Projects only provider-independent definitions reached by a region.
    ///
    /// # Errors
    ///
    /// Returns an error when any reached operation key is absent from this snapshot.
    ///
    /// # Panics
    ///
    /// Panics only if an internally validated arity cannot be represented as `u64`.
    pub fn project_reached<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a ScalarOpKey>,
    ) -> Result<CanonicalScalarDefinitionProjection, ScalarRegistryError> {
        let mut reached_keys = std::collections::BTreeSet::new();
        for key in keys {
            reached_keys.insert(key.clone());
            if reached_keys.len() > MAX_SCALAR_DEFINITIONS {
                return Err(ScalarRegistryError::ProjectionDefinitionCountLimit {
                    actual: reached_keys.len(),
                    limit: MAX_SCALAR_DEFINITIONS,
                });
            }
        }
        let mut output = b"tiler.scalar-definition-projection.v2\0".to_vec();
        encode_len(&mut output, reached_keys.len());
        for key in reached_keys {
            let definition = self
                .definition(&key)
                .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
            let encoded = encode_definition(definition);
            let actual = output.len().saturating_add(encoded.len());
            if actual > MAX_SCALAR_DEFINITION_PROJECTION_BYTES {
                return Err(ScalarRegistryError::ProjectionByteLimit {
                    actual,
                    limit: MAX_SCALAR_DEFINITION_PROJECTION_BYTES,
                });
            }
            output.extend_from_slice(&encoded);
        }
        Ok(CanonicalScalarDefinitionProjection(output))
    }

    /// Revalidates every reached scalar application and binds exact authority evidence to it.
    ///
    /// The returned receipt is separate from structural region identity. It records the selected
    /// definitions and admission providers without changing structural reuse equality.
    ///
    /// # Errors
    ///
    /// Returns an error for missing authority, rejected inference, or any stored/inferred type
    /// disagreement.
    pub fn revalidate_region(
        &self,
        region: &VerifiedIndexRegion,
    ) -> Result<ScalarAuthorityEvidence, ScalarRegistryError> {
        for tensor in region.tensors() {
            self.validate_type(tensor.value_type())?;
        }
        let reached = self.revalidate_region_operations(region)?;
        let definitions = self.project_reached(reached.iter())?;
        let mut value_types = Vec::new();
        value_types.extend(region.tensors().map(super::model::TensorRef::value_type));
        value_types.extend(
            region
                .scalar_values()
                .map(super::model::ScalarValueRef::value_type),
        );
        for operation in region.scalar_operations() {
            if let ScalarOperationKindRef::Reduce(reduction) = operation.kind() {
                value_types.extend(
                    reduction
                        .body()
                        .values()
                        .map(super::model::ReducerBodyValueRef::value_type),
                );
            }
        }
        let canonical_values = self.authority_canonical_values(region, &reached)?;
        let type_authority = self
            .0
            .semantic
            .project_value_set_authority(value_types, canonical_values)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))?;
        let mut admission = b"tiler.scalar-admission-provenance.v1\0".to_vec();
        encode_len(&mut admission, reached.len());
        for key in reached {
            encode_key(&mut admission, &key);
            let provider = self
                .provider(&key)
                .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
            encode_bytes(&mut admission, provider.namespace().as_bytes());
            encode_bytes(&mut admission, provider.name().as_bytes());
            admission.extend_from_slice(&provider.revision().to_be_bytes());
        }
        Ok(ScalarAuthorityEvidence {
            region: region.canonical_identity().clone(),
            definitions,
            admission: ScalarAdmissionProvenanceIdentity(admission),
            type_definitions: type_authority.reached_definitions().clone(),
            type_admission: type_authority.admission_provenance().clone(),
            semantic_snapshot: type_authority.registry_snapshot().clone(),
            scalar_snapshot: self.snapshot_identity().clone(),
        })
    }

    fn revalidate_region_operations(
        &self,
        region: &VerifiedIndexRegion,
    ) -> Result<std::collections::BTreeSet<ScalarOpKey>, ScalarRegistryError> {
        let mut reached = std::collections::BTreeSet::new();
        for operation in region.scalar_operations() {
            match operation.kind() {
                ScalarOperationKindRef::Apply { key, attributes } => {
                    reached.insert(key.clone());
                    let operands = operation
                        .operands()
                        .map(|id| {
                            region
                                .scalar_value(id)
                                .map(|value| value.value_type().clone())
                                .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let stored = operation
                        .results()
                        .map(|id| {
                            region
                                .scalar_value(id)
                                .map(|value| value.value_type().clone())
                                .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    self.revalidate_application(key, attributes, &operands, &stored)?;
                }
                ScalarOperationKindRef::Reduce(reduction) => {
                    let body = reduction.body();
                    for application in body.operations() {
                        reached.insert(application.key().clone());
                        let operands = application
                            .operands()
                            .map(|id| {
                                region
                                    .reducer_body_value(id)
                                    .map(|value| value.value_type().clone())
                                    .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let stored = application
                            .results()
                            .map(|id| {
                                region
                                    .reducer_body_value(id)
                                    .map(|value| value.value_type().clone())
                                    .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        self.revalidate_application(
                            application.key(),
                            application.attributes(),
                            &operands,
                            &stored,
                        )?;
                    }
                }
            }
        }
        Ok(reached)
    }

    fn authority_canonical_values<'a>(
        &'a self,
        region: &'a VerifiedIndexRegion,
        reached: &'a std::collections::BTreeSet<ScalarOpKey>,
    ) -> Result<Vec<&'a CanonicalValue>, ScalarRegistryError> {
        let mut canonical_values = Vec::new();
        for key in reached {
            let definition = self
                .definition(key)
                .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
            canonical_values.extend(
                definition
                    .attributes()
                    .fields()
                    .iter()
                    .filter_map(ScalarAttributeField::default),
            );
            canonical_values.push(definition.facts());
            canonical_values.push(definition.conformance());
        }
        for operation in region.scalar_operations() {
            match operation.kind() {
                ScalarOperationKindRef::Apply { attributes, .. } => {
                    canonical_values.push(attributes.value());
                }
                ScalarOperationKindRef::Reduce(reduction) => {
                    canonical_values.extend(
                        reduction
                            .body()
                            .operations()
                            .map(|operation| operation.attributes().value()),
                    );
                }
            }
        }
        Ok(canonical_values)
    }

    fn revalidate_application(
        &self,
        key: &ScalarOpKey,
        attributes: &ScalarAttributes,
        operands: &[ResolvedValueType],
        stored: &[ResolvedValueType],
    ) -> Result<(), ScalarRegistryError> {
        let canonical = self.normalize_attributes(key, attributes.clone())?;
        if &canonical != attributes {
            return Err(ScalarRegistryError::NonCanonicalAttributes { key: key.clone() });
        }
        let inferred = self
            .infer(
                key,
                operands,
                attributes,
                ScalarInferenceCapacity {
                    result_slots: MAX_SCALAR_ARITY,
                    result_count_before: 0,
                    result_limit: MAX_SCALAR_ARITY,
                    retained_bytes: usize::MAX,
                    retained_bytes_before: 0,
                    retained_byte_limit: usize::MAX,
                    per_result_overhead: 0,
                    byte_multiplier: 1,
                },
            )
            .map_err(|error| match error {
                ScalarApplyError::Authority(error) => error,
                ScalarApplyError::Host(_) => {
                    unreachable!("unbounded revalidation capacity cannot be exhausted")
                }
            })?;
        if stored.len() != inferred.len() {
            return Err(ScalarRegistryError::RevalidatedResultArity {
                key: key.clone(),
                stored: stored.len(),
                inferred: inferred.len(),
            });
        }
        for (position, (stored, inferred)) in stored.iter().zip(&inferred).enumerate() {
            if stored != inferred {
                return Err(ScalarRegistryError::RevalidatedResultTypeMismatch {
                    key: key.clone(),
                    position,
                    stored: Arc::new(stored.clone()),
                    inferred: Arc::new(inferred.clone()),
                });
            }
        }
        Ok(())
    }
}

fn encode_definition(definition: &ScalarOperationDefinition) -> Vec<u8> {
    let exact_capacity = encoded_definition_len(definition);
    let mut encoded = Vec::with_capacity(exact_capacity);
    encode_key(&mut encoded, &definition.key);
    encode_bytes(
        &mut encoded,
        definition.normative_definition.as_str().as_bytes(),
    );
    encoded.push(match definition.effect {
        ScalarEffect::Pure => 1,
    });
    encode_len(&mut encoded, definition.operands.min);
    encode_len(&mut encoded, definition.operands.max);
    encode_len(&mut encoded, definition.results.min);
    encode_len(&mut encoded, definition.results.max);
    encode_len(&mut encoded, definition.attributes.0.len());
    for field in &definition.attributes.0 {
        encoded.extend_from_slice(&field.id.get().to_be_bytes());
        encoded.push(canonical_kind_tag(field.kind));
        encoded.push(match (&field.default, field.required) {
            (None, true) => 1,
            (None, false) => 2,
            (Some(_), false) => 3,
            (Some(_), true) => unreachable!("required fields cannot carry defaults"),
        });
        if let Some(default) = &field.default {
            encode_canonical(&mut encoded, default);
        }
    }
    encode_canonical(&mut encoded, &definition.facts);
    encode_canonical(&mut encoded, &definition.conformance);
    debug_assert_eq!(encoded.len(), exact_capacity);
    encoded
}

fn encoded_definition_len(definition: &ScalarOperationDefinition) -> usize {
    let mut bytes = encoded_key_len(&definition.key)
        .saturating_add(encoded_bytes_len(
            definition.normative_definition.as_str().len(),
        ))
        .saturating_add(1)
        .saturating_add(4 * std::mem::size_of::<u64>())
        .saturating_add(std::mem::size_of::<u64>());
    for field in &definition.attributes.0 {
        bytes = bytes.saturating_add(std::mem::size_of::<u32>() + 2);
        if let Some(default) = &field.default {
            bytes = bytes.saturating_add(default.encoded_len());
        }
    }
    bytes
        .saturating_add(definition.facts.encoded_len())
        .saturating_add(definition.conformance.encoded_len())
}

fn encoded_key_len(key: &ScalarOpKey) -> usize {
    encoded_bytes_len(key.namespace().len())
        .saturating_add(encoded_bytes_len(key.name().len()))
        .saturating_add(std::mem::size_of::<u32>())
}

const fn encoded_bytes_len(bytes: usize) -> usize {
    std::mem::size_of::<u64>().saturating_add(bytes)
}

fn compute_scalar_snapshot_identity(
    definitions: &BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
) -> CanonicalScalarRegistrySnapshotIdentity {
    let mut bytes = b"tiler.scalar-registry-snapshot.v1\0".to_vec();
    encode_len(&mut bytes, definitions.len());
    for (key, registered) in definitions {
        encode_key(&mut bytes, key);
        encode_bytes(&mut bytes, &encode_definition(&registered.definition));
        encode_bytes(&mut bytes, registered.provider.namespace().as_bytes());
        encode_bytes(&mut bytes, registered.provider.name().as_bytes());
        bytes.extend_from_slice(&registered.provider.revision().to_be_bytes());
    }
    CanonicalScalarRegistrySnapshotIdentity(bytes)
}

/// Canonical provider-independent projection of reached scalar definitions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScalarDefinitionProjection(Vec<u8>);
impl CanonicalScalarDefinitionProjection {
    /// Returns collision-free projection bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Provider-attributed scalar admission provenance for one reached operation set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScalarAdmissionProvenanceIdentity(Vec<u8>);
impl ScalarAdmissionProvenanceIdentity {
    /// Returns collision-free provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Complete provider-attributed frozen scalar-registry snapshot identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScalarRegistrySnapshotIdentity(Vec<u8>);
impl CanonicalScalarRegistrySnapshotIdentity {
    /// Returns collision-free snapshot bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Checked scalar authority evidence bound to one exact structural region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarAuthorityEvidence {
    region: CanonicalIndexRegionIdentity,
    definitions: CanonicalScalarDefinitionProjection,
    admission: ScalarAdmissionProvenanceIdentity,
    type_definitions: SemanticDefinitionProjectionIdentity,
    type_admission: SemanticAdmissionProvenanceIdentity,
    semantic_snapshot: SemanticRegistrySnapshotIdentity,
    scalar_snapshot: CanonicalScalarRegistrySnapshotIdentity,
}
impl ScalarAuthorityEvidence {
    /// Returns the structural region identity this evidence revalidated.
    #[must_use]
    pub const fn region(&self) -> &CanonicalIndexRegionIdentity {
        &self.region
    }
    /// Returns reached provider-independent definitions.
    #[must_use]
    pub const fn definitions(&self) -> &CanonicalScalarDefinitionProjection {
        &self.definitions
    }
    /// Returns reached provider-attributed admission provenance.
    #[must_use]
    pub const fn admission(&self) -> &ScalarAdmissionProvenanceIdentity {
        &self.admission
    }
    /// Returns reached provider-independent semantic type definitions.
    #[must_use]
    pub const fn type_definitions(&self) -> &SemanticDefinitionProjectionIdentity {
        &self.type_definitions
    }
    /// Returns provider-attributed semantic type-admission provenance.
    #[must_use]
    pub const fn type_admission(&self) -> &SemanticAdmissionProvenanceIdentity {
        &self.type_admission
    }
    /// Returns the complete semantic registry snapshot provenance.
    #[must_use]
    pub const fn semantic_snapshot(&self) -> &SemanticRegistrySnapshotIdentity {
        &self.semantic_snapshot
    }
    /// Returns the complete scalar registry snapshot provenance.
    #[must_use]
    pub const fn scalar_snapshot(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        &self.scalar_snapshot
    }
}

fn validate_canonical_types(
    registry: &FrozenSemanticRegistry,
    value: &CanonicalValue,
) -> Result<(), ScalarRegistryError> {
    match value.view() {
        CanonicalValueView::Type(value_type) => registry
            .validate_type(value_type)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))?,
        CanonicalValueView::FloatBits(value) => registry
            .validate_type(&ResolvedValueType::nominal(value.format().clone()))
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))?,
        CanonicalValueView::Sequence(values) => {
            for value in values {
                validate_canonical_types(registry, value)?;
            }
        }
        CanonicalValueView::Record(fields) => {
            for field in fields {
                validate_canonical_types(registry, field.value())?;
            }
        }
        CanonicalValueView::Bool(_)
        | CanonicalValueView::Signed { .. }
        | CanonicalValueView::Unsigned { .. }
        | CanonicalValueView::Bytes(_)
        | CanonicalValueView::Utf8(_) => {}
    }
    Ok(())
}

fn canonical_kind_tag(kind: CanonicalValueKind) -> u8 {
    match kind {
        CanonicalValueKind::Type => 1,
        CanonicalValueKind::Bool => 2,
        CanonicalValueKind::Signed => 3,
        CanonicalValueKind::Unsigned => 4,
        CanonicalValueKind::FloatBits => 5,
        CanonicalValueKind::Bytes => 6,
        CanonicalValueKind::Utf8 => 7,
        CanonicalValueKind::Sequence => 8,
        CanonicalValueKind::Record => 9,
    }
}

pub(super) fn encode_key(output: &mut Vec<u8>, key: &ScalarOpKey) {
    encode_bytes(output, key.namespace().as_bytes());
    encode_bytes(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}
pub(super) fn encode_canonical(output: &mut Vec<u8>, value: &CanonicalValue) {
    value.encode(output);
}
pub(super) fn encode_len(output: &mut Vec<u8>, len: usize) {
    output.extend_from_slice(
        &u64::try_from(len)
            .expect("bounded usize fits u64")
            .to_be_bytes(),
    );
}
pub(super) fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod resource_order_tests {
    use std::cell::Cell;
    use std::sync::Arc;

    use super::{
        ScalarArity, ScalarAttributeSchema, ScalarEffect, ScalarInferenceCapacity,
        ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey,
        ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
        encode_definition, encoded_definition_len,
    };
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderDiagnosticCode, ResolvedValueType, TypeKey,
    };

    struct NoResults;
    impl ScalarOperationInferencer for NoResults {
        fn infer(
            &self,
            _: ScalarInferenceRequest<'_>,
            _: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            Ok(())
        }
    }

    fn record() -> CanonicalValue {
        CanonicalValue::record([]).unwrap()
    }

    #[test]
    fn definition_size_prepass_matches_the_encoder_path_exactly() {
        let definition = ScalarOperationDefinition::new(
            ScalarOpKey::new("test", "sized", 1).unwrap(),
            NormativeDefinitionRef::new("urn:test:sized:v1").unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(0).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::bytes(vec![7_u8; 4_096]).unwrap(),
                record(),
            ),
            Arc::new(NoResults),
        );
        assert_eq!(
            encoded_definition_len(&definition),
            encode_definition(&definition).len()
        );
    }

    #[test]
    fn inference_writer_uses_exact_enclosing_slots_without_rejecting_schema_maximum() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let mut outputs = ScalarInferenceOutputs::new(
            4_096,
            ScalarInferenceCapacity {
                result_slots: 1,
                result_count_before: 12,
                result_limit: 13,
                retained_bytes: value_type.canonical_encoding().as_bytes().len() + 7,
                retained_bytes_before: 29,
                retained_byte_limit: 4_096,
                per_result_overhead: 7,
                byte_multiplier: 1,
            },
        );
        outputs.try_push(value_type.clone()).unwrap();
        assert_eq!(outputs.finish(Ok(()), 1).unwrap(), vec![value_type]);

        let mut outputs = ScalarInferenceOutputs::new(
            4_096,
            ScalarInferenceCapacity {
                result_slots: 0,
                result_count_before: 13,
                result_limit: 13,
                retained_bytes: usize::MAX,
                retained_bytes_before: 0,
                retained_byte_limit: usize::MAX,
                per_result_overhead: 0,
                byte_multiplier: 1,
            },
        );
        let error = outputs.try_push(ResolvedValueType::nominal(
            TypeKey::new("test", "value", 1).unwrap(),
        ));
        assert_eq!(
            error.unwrap_err().code(),
            &ProviderDiagnosticCode::new("tiler.scalar.host-result-capacity").unwrap()
        );
        assert_eq!(
            outputs.finish(Ok(()), 0),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::ResultSlots {
                    actual: 14,
                    limit: 13,
                }
            ))
        );
    }

    #[test]
    fn host_slot_failure_wins_over_the_callback_result_without_provider_attribution() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let calls = Cell::new(0_u32);
        let mut outputs = ScalarInferenceOutputs::new(
            2,
            ScalarInferenceCapacity {
                result_slots: 1,
                result_count_before: 65_535,
                result_limit: 65_536,
                retained_bytes: usize::MAX,
                retained_bytes_before: 0,
                retained_byte_limit: usize::MAX,
                per_result_overhead: 0,
                byte_multiplier: 1,
            },
        );
        calls.set(1);
        outputs.try_push(value_type.clone()).unwrap();
        calls.set(2);
        let ignored = outputs.try_push(value_type);
        calls.set(3);
        assert_eq!(
            ignored.unwrap_err().code(),
            &ProviderDiagnosticCode::new("tiler.scalar.host-result-capacity").unwrap()
        );
        let callback = Err(ScalarInferenceError::new(
            ProviderDiagnosticCode::new("provider.after-host-failure").unwrap(),
            "provider returned an error after ignoring the host writer failure",
        )
        .unwrap());
        assert_eq!(
            outputs.finish(callback, 1),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::ResultSlots {
                    actual: 65_537,
                    limit: 65_536,
                }
            ))
        );
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn host_byte_failure_reports_exact_enclosing_usage_and_wins_callback_error() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let result_bytes = value_type.canonical_encoding().as_bytes().len() + 7;
        let before = 1_000;
        let limit = before + result_bytes - 1;
        let mut outputs = ScalarInferenceOutputs::new(
            1,
            ScalarInferenceCapacity {
                result_slots: 1,
                result_count_before: 0,
                result_limit: 65_536,
                retained_bytes: result_bytes - 1,
                retained_bytes_before: before,
                retained_byte_limit: limit,
                per_result_overhead: 7,
                byte_multiplier: 1,
            },
        );
        let ignored = outputs.try_push(value_type);
        assert_eq!(
            ignored.unwrap_err().code(),
            &ProviderDiagnosticCode::new("tiler.scalar.host-byte-capacity").unwrap()
        );
        let callback = Err(ScalarInferenceError::new(
            ProviderDiagnosticCode::new("provider.after-host-failure").unwrap(),
            "provider returned an error after ignoring the host writer failure",
        )
        .unwrap());
        assert_eq!(
            outputs.finish(callback, 0),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::CanonicalBytes {
                    actual: before + result_bytes,
                    limit,
                }
            ))
        );
    }

    #[test]
    fn multiplied_host_byte_capacity_is_exact_at_both_sides_of_the_boundary() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let multiplier = 3;
        let unit_bytes = value_type.canonical_encoding().as_bytes().len() + 7;
        let required = unit_bytes * multiplier;
        let before = 2_000;
        let capacity = |retained_bytes, retained_byte_limit| ScalarInferenceCapacity {
            result_slots: 1,
            result_count_before: 0,
            result_limit: 65_536,
            retained_bytes,
            retained_bytes_before: before,
            retained_byte_limit,
            per_result_overhead: 7,
            byte_multiplier: multiplier,
        };

        let mut exact = ScalarInferenceOutputs::new(1, capacity(required, before + required));
        exact.try_push(value_type.clone()).unwrap();
        assert_eq!(exact.finish(Ok(()), 1).unwrap(), vec![value_type.clone()]);

        let mut short =
            ScalarInferenceOutputs::new(1, capacity(required - 1, before + required - 1));
        short.try_push(value_type).unwrap_err();
        assert_eq!(
            short.finish(Ok(()), 0),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::CanonicalBytes {
                    actual: before + required,
                    limit: before + required - 1,
                }
            ))
        );
    }
}
