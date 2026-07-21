use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::semantic::{
    CanonicalValue, CanonicalValueKind, CanonicalValueView, FrozenSemanticRegistry,
    NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType, TypeIdentityError,
    TypeKey,
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
        namespace: impl Into<String>,
        name: impl Into<String>,
        version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, version).map(Self)
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributeField {
    id: u32,
    kind: CanonicalValueKind,
    required: bool,
}

impl ScalarAttributeField {
    /// Creates one field.
    #[must_use]
    pub const fn new(id: u32, kind: CanonicalValueKind, required: bool) -> Self {
        Self { id, kind, required }
    }
    /// Returns the stable field ID.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }
    /// Returns the required canonical value kind.
    #[must_use]
    pub const fn kind(self) -> CanonicalValueKind {
        self.kind
    }
    /// Returns whether the field must be present.
    #[must_use]
    pub const fn is_required(self) -> bool {
        self.required
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
        let mut fields: Vec<_> = fields.into_iter().collect();
        if fields.len() > MAX_SCALAR_ATTRIBUTES {
            return Err(ScalarRegistryError::TooManyAttributeFields {
                actual: fields.len(),
            });
        }
        fields.sort_by_key(|field| field.id);
        if fields.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(ScalarRegistryError::DuplicateAttributeField);
        }
        Ok(Self(fields))
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
}

fn canonical_kind(value: &CanonicalValue) -> CanonicalValueKind {
    match value.view() {
        CanonicalValueView::Type(_) => CanonicalValueKind::Type,
        CanonicalValueView::Bool(_) => CanonicalValueKind::Bool,
        CanonicalValueView::Signed(_) => CanonicalValueKind::Signed,
        CanonicalValueView::Unsigned(_) => CanonicalValueKind::Unsigned,
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
pub struct ScalarInferenceRequest<'a> {
    /// Ordered operand types.
    pub operands: &'a [ResolvedValueType],
    /// Checked attributes.
    pub attributes: &'a ScalarAttributes,
}

/// Stable provider rejection of one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarInferenceError {
    code: String,
    message: String,
}

impl ScalarInferenceError {
    /// Creates an inference rejection.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for ScalarInferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl Error for ScalarInferenceError {}

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
    ) -> Result<Vec<ResolvedValueType>, ScalarInferenceError>;
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
    /// Canonical attribute schema.
    pub attributes: ScalarAttributeSchema,
    /// Admitted operand arity.
    pub operands: ScalarArity,
    /// Admitted result arity.
    pub results: ScalarArity,
    /// Host-enforced effect contract.
    pub effect: ScalarEffect,
    /// Canonical semantic facts.
    pub facts: CanonicalValue,
    /// Canonical conformance requirements.
    pub conformance: CanonicalValue,
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
        id: u32,
    },
    /// Attributes omitted a required field.
    MissingAttribute {
        /// Missing field ID.
        id: u32,
    },
    /// An attribute value did not match its declared kind.
    AttributeKind {
        /// Mismatched field ID.
        id: u32,
    },
    /// The semantic type authority rejected embedded or inferred type data.
    TypeAuthority(Arc<RegistryError>),
    /// The operation-specific inferencer rejected an application.
    Inference(Arc<ScalarInferenceError>),
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
}
impl fmt::Display for ScalarRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for ScalarRegistryError {}

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
        validate_canonical_types(&self.semantic, &definition.facts)?;
        validate_canonical_types(&self.semantic, &definition.conformance)?;
        let definition_bytes = encode_definition(&definition).len();
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
        FrozenScalarRegistry(Arc::new(ScalarRegistryData {
            semantic: self.semantic,
            definitions: self.definitions,
        }))
    }
}

struct ScalarRegistryData {
    semantic: FrozenSemanticRegistry,
    definitions: BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
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
    pub(super) fn validate_type(
        &self,
        value: &ResolvedValueType,
    ) -> Result<(), ScalarRegistryError> {
        self.0
            .semantic
            .validate_type(value)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))
    }

    pub(super) fn infer(
        &self,
        key: &ScalarOpKey,
        operands: &[ResolvedValueType],
        attributes: &ScalarAttributes,
    ) -> Result<Vec<ResolvedValueType>, ScalarRegistryError> {
        let registered = self
            .0
            .definitions
            .get(key)
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
        let definition = &registered.definition;
        if !definition.operands.accepts(operands.len()) {
            return Err(ScalarRegistryError::OperandArity {
                key: key.clone(),
                actual: operands.len(),
            });
        }
        definition.attributes.validate(attributes)?;
        validate_canonical_types(&self.0.semantic, attributes.value())?;
        for operand in operands {
            self.validate_type(operand)?;
        }
        let results = definition
            .inferencer
            .infer(ScalarInferenceRequest {
                operands,
                attributes,
            })
            .map_err(|error| ScalarRegistryError::Inference(Arc::new(error)))?;
        if !definition.results.accepts(results.len()) {
            return Err(ScalarRegistryError::ResultArity {
                key: key.clone(),
                actual: results.len(),
            });
        }
        for result in &results {
            self.validate_type(result)?;
        }
        Ok(results)
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
        let mut output = b"tiler.scalar-definition-projection.v1\0".to_vec();
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
}

fn encode_definition(definition: &ScalarOperationDefinition) -> Vec<u8> {
    let mut encoded = Vec::new();
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
        encoded.extend_from_slice(&field.id.to_be_bytes());
        encoded.push(canonical_kind_tag(field.kind));
        encoded.push(u8::from(field.required));
    }
    encode_canonical(&mut encoded, &definition.facts);
    encode_canonical(&mut encoded, &definition.conformance);
    encoded
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

fn validate_canonical_types(
    registry: &FrozenSemanticRegistry,
    value: &CanonicalValue,
) -> Result<(), ScalarRegistryError> {
    match value.view() {
        CanonicalValueView::Type(value_type) => registry
            .validate_type(value_type)
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
        | CanonicalValueView::Signed(_)
        | CanonicalValueView::Unsigned(_)
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
        CanonicalValueKind::Bytes => 5,
        CanonicalValueKind::Utf8 => 6,
        CanonicalValueKind::Sequence => 7,
        CanonicalValueKind::Record => 8,
    }
}

pub(super) fn encode_key(output: &mut Vec<u8>, key: &ScalarOpKey) {
    encode_bytes(output, key.namespace().as_bytes());
    encode_bytes(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}
pub(super) fn encode_canonical(output: &mut Vec<u8>, value: &CanonicalValue) {
    match value.view() {
        CanonicalValueView::Type(value) => {
            output.push(1);
            encode_bytes(output, value.canonical_encoding().as_bytes());
        }
        CanonicalValueView::Bool(value) => {
            output.push(2);
            output.push(u8::from(value));
        }
        CanonicalValueView::Signed(value) => {
            output.push(3);
            output.extend_from_slice(&value.to_be_bytes());
        }
        CanonicalValueView::Unsigned(value) => {
            output.push(4);
            output.extend_from_slice(&value.to_be_bytes());
        }
        CanonicalValueView::Bytes(value) => {
            output.push(5);
            encode_bytes(output, value);
        }
        CanonicalValueView::Utf8(value) => {
            output.push(6);
            encode_bytes(output, value.as_bytes());
        }
        CanonicalValueView::Sequence(values) => {
            output.push(7);
            encode_len(output, values.len());
            for value in values {
                encode_canonical(output, value);
            }
        }
        CanonicalValueView::Record(fields) => {
            output.push(8);
            encode_len(output, fields.len());
            for field in fields {
                output.extend_from_slice(&field.id().to_be_bytes());
                encode_canonical(output, field.value());
            }
        }
    }
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
