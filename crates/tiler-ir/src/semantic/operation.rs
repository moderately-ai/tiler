use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::shape::{Axis, Shape};

use super::handles::{GraphId, OperationId, OperationIndex, ValueId, ValueIndex};
use super::interface::InputIndex;
use super::registry::NormativeDefinitionRef;
use super::types::{CanonicalField, CanonicalValue, ResolvedValueType, TypeIdentityError, TypeKey};

/// The bounded profile's canonical quiet NaN produced by arithmetic.
pub const CANONICAL_F32_ARITHMETIC_NAN_BITS: u32 = 0x7fc0_0000;
/// Stable field ID carrying exact f32 bits on the standard constant operation.
pub const F32_CONSTANT_BITS_ATTRIBUTE: u32 = 1;
/// Stable field ID carrying canonical axes on the standard strict Sum.
pub const REDUCTION_AXES_ATTRIBUTE: u32 = 1;
/// Maximum declared fields in one operation-attribute schema.
pub const MAX_OPERATION_ATTRIBUTES: usize = 1_024;

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
        namespace: impl Into<String>,
        name: impl Into<String>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, semantic_version).map(Self)
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
        let super::types::CanonicalValueView::Record(fields) = value.view() else {
            unreachable!("record construction returns a record")
        };
        Ok(Self(fields.to_vec()))
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
    pub fn get(&self, id: u32) -> Option<&CanonicalValue> {
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
            output.extend_from_slice(&field.id().to_be_bytes());
            field.value().encode(output);
        }
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
                | (Self::Signed, super::types::CanonicalValueView::Signed(_))
                | (
                    Self::Unsigned,
                    super::types::CanonicalValueView::Unsigned(_)
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
            Self::Bytes => 5,
            Self::Utf8 => 6,
            Self::Sequence => 7,
            Self::Record => 8,
        }
    }
}

/// One field in a host-owned canonical operation-attribute schema.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationAttributeSchema {
    id: u32,
    kind: CanonicalValueKind,
    required: bool,
}

impl OperationAttributeSchema {
    /// Creates one required attribute field.
    #[must_use]
    pub const fn required(id: u32, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: true,
        }
    }

    /// Creates one optional attribute field.
    #[must_use]
    pub const fn optional(id: u32, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: false,
        }
    }

    /// Returns the stable schema-local field ID.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }

    /// Returns the required canonical value category.
    #[must_use]
    pub const fn kind(self) -> CanonicalValueKind {
        self.kind
    }

    /// Returns whether the field must occur.
    #[must_use]
    pub const fn is_required(self) -> bool {
        self.required
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
    /// Two attribute declarations used one field ID.
    DuplicateAttribute {
        /// Duplicated schema-local field ID.
        field_id: u32,
    },
    /// The schema declared too many attribute fields.
    TooManyAttributes {
        /// Actual declared field count.
        fields: usize,
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
            Self::DuplicateAttribute { field_id } => {
                write!(formatter, "duplicate operation attribute field {field_id}")
            }
            Self::TooManyAttributes { fields } => write!(
                formatter,
                "operation schema has {fields} attributes, exceeding {MAX_OPERATION_ATTRIBUTES}"
            ),
        }
    }
}

impl Error for OperationSchemaError {}

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
        let mut attributes: Vec<_> = attributes.into_iter().collect();
        if attributes.len() > MAX_OPERATION_ATTRIBUTES {
            return Err(OperationSchemaError::TooManyAttributes {
                fields: attributes.len(),
            });
        }
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

    fn validate_inputs(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<(), OperationInferenceError> {
        if !self.operands.admits(operands.len()) {
            return Err(OperationInferenceError::new(
                "tiler.schema.operand-arity",
                "operand arity is outside the registered schema",
            ));
        }
        for field in attributes.fields() {
            let Some(schema) = self
                .attributes
                .binary_search_by_key(&field.id(), |candidate| candidate.id)
                .ok()
                .map(|index| self.attributes[index])
            else {
                return Err(OperationInferenceError::new(
                    "tiler.schema.unknown-attribute",
                    "attribute field is absent from the registered schema",
                ));
            };
            if !schema.kind.accepts(field.value()) {
                return Err(OperationInferenceError::new(
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
            return Err(OperationInferenceError::new(
                "tiler.schema.missing-attribute",
                "a required attribute field is absent",
            ));
        }
        Ok(())
    }

    fn validate_results(&self, results: &[ValueFact]) -> Result<(), OperationInferenceError> {
        if !self.results.admits(results.len()) {
            return Err(OperationInferenceError::new(
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
            output.extend_from_slice(&field.id.to_be_bytes());
            output.push(field.kind.encode());
            output.push(u8::from(field.required));
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
    code: String,
    message: String,
}

impl OperationInferenceError {
    /// Creates a provider-attributed rejection.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns diagnostic detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for OperationInferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for OperationInferenceError {}

/// Immutable semantic inference for one operation family.
pub trait OperationInferencer: Send + Sync + 'static {
    /// Validates operands and canonical attributes, then exclusively derives
    /// the ordered result facts.
    ///
    /// # Errors
    ///
    /// Returns a stable provider diagnostic for an invalid application.
    fn infer(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueFact>, OperationInferenceError>;
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

    pub(super) fn infer(
        &self,
        operands: &[ValueFact],
        attributes: &OperationAttributes,
    ) -> Result<Vec<ValueFact>, OperationInferenceError> {
        self.schema.validate_inputs(operands, attributes)?;
        let results = self.inferencer.infer(operands, attributes)?;
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

pub(super) fn axes_attribute(axes: &[Axis]) -> Result<CanonicalValue, TypeIdentityError> {
    CanonicalValue::sequence(
        axes.iter()
            .map(|axis| CanonicalValue::unsigned(u64::from(axis.get()))),
    )
}
