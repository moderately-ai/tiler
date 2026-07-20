use std::error::Error;
use std::fmt;

/// Maximum UTF-8 byte length of one canonical identity component.
pub const MAX_IDENTITY_COMPONENT_BYTES: usize = 255;
/// Maximum nesting depth of a resolved value type and its arguments.
pub const MAX_RESOLVED_TYPE_DEPTH: usize = 32;
/// Maximum total nodes in one resolved value type.
pub const MAX_RESOLVED_TYPE_NODES: usize = 4_096;
/// Maximum items in one parameter or record collection.
pub const MAX_RESOLVED_TYPE_ITEMS: usize = 1_024;
/// Maximum total byte payload in one resolved value type.
pub const MAX_RESOLVED_TYPE_BYTES: usize = 1_048_576;

/// A canonical nominal tensor-type identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeKey(Key);

impl TypeKey {
    /// Creates a validated namespaced, versioned type key.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for an invalid namespace, name, or version.
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        Key::new(namespace.into(), name.into(), semantic_version).map(Self)
    }

    /// Returns the canonical namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.0.namespace
    }

    /// Returns the canonical name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Returns the nonzero semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version
    }
}

impl fmt::Display for TypeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A canonical encoded-numeric scheme identity, distinct from [`TypeKey`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QuantSchemeKey(Key);

impl QuantSchemeKey {
    /// Creates a validated namespaced, versioned scheme key.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for an invalid namespace, name, or version.
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        Key::new(namespace.into(), name.into(), semantic_version).map(Self)
    }

    /// Returns the canonical namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.0.namespace
    }

    /// Returns the canonical name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Returns the nonzero semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version
    }
}

impl fmt::Display for QuantSchemeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Key {
    namespace: String,
    name: String,
    semantic_version: u32,
}

impl Key {
    fn new(
        namespace: String,
        name: String,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        validate_component(IdentityComponent::Namespace, &namespace)?;
        validate_component(IdentityComponent::Name, &name)?;
        if semantic_version == 0 {
            return Err(TypeIdentityError::ZeroSemanticVersion);
        }
        Ok(Self {
            namespace,
            name,
            semantic_version,
        })
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        encode_bytes(output, self.namespace.as_bytes());
        encode_bytes(output, self.name.as_bytes());
        output.extend_from_slice(&self.semantic_version.to_be_bytes());
    }
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}::{}@{}",
            self.namespace, self.name, self.semantic_version
        )
    }
}

/// Which part of a namespaced identity is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IdentityComponent {
    /// The authority namespace.
    Namespace,
    /// The name within that namespace.
    Name,
}

impl fmt::Display for IdentityComponent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Namespace => formatter.write_str("namespace"),
            Self::Name => formatter.write_str("name"),
        }
    }
}

/// Failure to construct canonical type identity data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TypeIdentityError {
    /// An identity component was empty.
    EmptyComponent {
        /// Rejected component.
        component: IdentityComponent,
    },
    /// An identity component exceeded the canonical bound.
    ComponentTooLong {
        /// Rejected component.
        component: IdentityComponent,
        /// Actual UTF-8 byte length.
        bytes: usize,
    },
    /// An identity component contained a byte outside the portable grammar.
    InvalidComponentCharacter {
        /// Rejected component.
        component: IdentityComponent,
        /// Zero-based byte position.
        byte_index: usize,
    },
    /// Semantic identity version zero is reserved.
    ZeroSemanticVersion,
    /// A parameterized type supplied no arguments.
    EmptyTypeArguments,
    /// An encoded-numeric contract supplied no static fields.
    EmptyEncodedNumericContract,
    /// Two record fields used the same stable field ID.
    DuplicateFieldId {
        /// Duplicated field identifier.
        field_id: u32,
    },
    /// A collection exceeded the per-collection item bound.
    TooManyItems {
        /// Actual item count.
        items: usize,
    },
    /// One byte or UTF-8 payload exceeded the per-type byte bound.
    PayloadTooLarge {
        /// Actual byte count.
        bytes: usize,
    },
    /// Recursive type structure exceeded the depth bound.
    NestingTooDeep,
    /// Recursive type structure exceeded the total-node bound.
    TooManyNodes,
    /// Recursive type structure exceeded the total payload-byte bound.
    TooManyPayloadBytes,
}

impl fmt::Display for TypeIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyComponent { component } => write!(formatter, "{component} is empty"),
            Self::ComponentTooLong { component, bytes } => write!(
                formatter,
                "{component} has {bytes} bytes, exceeding {MAX_IDENTITY_COMPONENT_BYTES}"
            ),
            Self::InvalidComponentCharacter {
                component,
                byte_index,
            } => write!(
                formatter,
                "{component} contains an invalid byte at position {byte_index}"
            ),
            Self::ZeroSemanticVersion => formatter.write_str("semantic version zero is reserved"),
            Self::EmptyTypeArguments => {
                formatter.write_str("a parameterized type requires at least one argument")
            }
            Self::EmptyEncodedNumericContract => {
                formatter.write_str("an encoded-numeric type requires a nonempty static contract")
            }
            Self::DuplicateFieldId { field_id } => {
                write!(formatter, "duplicate canonical field ID {field_id}")
            }
            Self::TooManyItems { items } => write!(
                formatter,
                "collection has {items} items, exceeding {MAX_RESOLVED_TYPE_ITEMS}"
            ),
            Self::PayloadTooLarge { bytes } => write!(
                formatter,
                "payload has {bytes} bytes, exceeding {MAX_RESOLVED_TYPE_BYTES}"
            ),
            Self::NestingTooDeep => write!(
                formatter,
                "resolved type exceeds nesting depth {MAX_RESOLVED_TYPE_DEPTH}"
            ),
            Self::TooManyNodes => write!(
                formatter,
                "resolved type exceeds {MAX_RESOLVED_TYPE_NODES} structural nodes"
            ),
            Self::TooManyPayloadBytes => write!(
                formatter,
                "resolved type exceeds {MAX_RESOLVED_TYPE_BYTES} payload bytes"
            ),
        }
    }
}

impl Error for TypeIdentityError {}

/// One complete shape-independent semantic tensor value type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedValueType(ResolvedValueTypeData);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ResolvedValueTypeData {
    Nominal(TypeKey),
    Parameterized {
        constructor: TypeKey,
        arguments: Vec<ResolvedValueTypeArgument>,
    },
    EncodedNumeric {
        scheme: QuantSchemeKey,
        contract: CanonicalEncodedNumericContract,
    },
}

impl ResolvedValueType {
    /// Creates a nominal value type.
    #[must_use]
    pub const fn nominal(key: TypeKey) -> Self {
        Self(ResolvedValueTypeData::Nominal(key))
    }

    /// Creates a bounded parameterized value type.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for empty or over-limit arguments.
    pub fn parameterized(
        constructor: TypeKey,
        arguments: impl IntoIterator<Item = ResolvedValueTypeArgument>,
    ) -> Result<Self, TypeIdentityError> {
        let arguments: Vec<_> = arguments.into_iter().collect();
        if arguments.is_empty() {
            return Err(TypeIdentityError::EmptyTypeArguments);
        }
        let value = Self(ResolvedValueTypeData::Parameterized {
            constructor,
            arguments,
        });
        validate_resolved_type(&value)?;
        Ok(value)
    }

    /// Creates a bounded encoded-numeric value type.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when the complete structure exceeds a
    /// canonical bound.
    pub fn encoded_numeric(
        scheme: QuantSchemeKey,
        contract: CanonicalEncodedNumericContract,
    ) -> Result<Self, TypeIdentityError> {
        let value = Self(ResolvedValueTypeData::EncodedNumeric { scheme, contract });
        validate_resolved_type(&value)?;
        Ok(value)
    }

    /// Returns the nominal identity when this is a nominal value type.
    #[must_use]
    pub const fn nominal_key(&self) -> Option<&TypeKey> {
        match &self.0 {
            ResolvedValueTypeData::Nominal(key) => Some(key),
            ResolvedValueTypeData::Parameterized { .. }
            | ResolvedValueTypeData::EncodedNumeric { .. } => None,
        }
    }

    /// Returns the collision-free versioned canonical identity bytes.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalResolvedValueType {
        let mut bytes = b"tiler.resolved-value-type.v1\0".to_vec();
        self.encode(&mut bytes);
        CanonicalResolvedValueType(bytes)
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        match &self.0 {
            ResolvedValueTypeData::Nominal(key) => {
                output.push(1);
                key.0.encode(output);
            }
            ResolvedValueTypeData::Parameterized {
                constructor,
                arguments,
            } => {
                output.push(2);
                constructor.0.encode(output);
                encode_len(output, arguments.len());
                for argument in arguments {
                    argument.encode(output);
                }
            }
            ResolvedValueTypeData::EncodedNumeric { scheme, contract } => {
                output.push(3);
                scheme.0.encode(output);
                contract.encode(output);
            }
        }
    }

    pub(super) fn visit_referenced_types(&self, visitor: &mut impl FnMut(&ResolvedValueType)) {
        match &self.0 {
            ResolvedValueTypeData::Nominal(_) => {}
            ResolvedValueTypeData::Parameterized { arguments, .. } => {
                for argument in arguments {
                    argument.visit_referenced_types(visitor);
                }
            }
            ResolvedValueTypeData::EncodedNumeric { contract, .. } => {
                for field in &contract.fields {
                    field.value.visit_referenced_types(visitor);
                }
            }
        }
    }
}

/// Collision-free internal canonical encoding of one resolved value type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalResolvedValueType(Vec<u8>);

impl CanonicalResolvedValueType {
    /// Returns the canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One bounded canonical parameter of a resolved value type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedValueTypeArgument(ResolvedValueTypeArgumentData);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ResolvedValueTypeArgumentData {
    Type(ResolvedValueType),
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Bytes(Vec<u8>),
    Utf8(String),
    Sequence(Vec<ResolvedValueTypeArgument>),
    Record(Vec<ResolvedValueTypeField>),
}

impl ResolvedValueTypeArgument {
    /// Creates a nested resolved-type argument.
    #[must_use]
    pub const fn value_type(value: ResolvedValueType) -> Self {
        Self(ResolvedValueTypeArgumentData::Type(value))
    }

    /// Creates a Boolean argument.
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self(ResolvedValueTypeArgumentData::Bool(value))
    }

    /// Creates a fixed-width signed integer argument.
    #[must_use]
    pub const fn signed(value: i64) -> Self {
        Self(ResolvedValueTypeArgumentData::Signed(value))
    }

    /// Creates a fixed-width unsigned integer argument.
    #[must_use]
    pub const fn unsigned(value: u64) -> Self {
        Self(ResolvedValueTypeArgumentData::Unsigned(value))
    }

    /// Creates a bounded exact byte argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError::PayloadTooLarge`] when over the byte bound.
    pub fn bytes(value: impl Into<Vec<u8>>) -> Result<Self, TypeIdentityError> {
        let value = value.into();
        validate_payload_len(value.len())?;
        Ok(Self(ResolvedValueTypeArgumentData::Bytes(value)))
    }

    /// Creates a bounded exact UTF-8 argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError::PayloadTooLarge`] when over the byte bound.
    pub fn utf8(value: impl Into<String>) -> Result<Self, TypeIdentityError> {
        let value = value.into();
        validate_payload_len(value.len())?;
        Ok(Self(ResolvedValueTypeArgumentData::Utf8(value)))
    }

    /// Creates an ordered bounded sequence argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when a canonical bound is exceeded.
    pub fn sequence(values: impl IntoIterator<Item = Self>) -> Result<Self, TypeIdentityError> {
        let values: Vec<_> = values.into_iter().collect();
        validate_items(values.len())?;
        let value = Self(ResolvedValueTypeArgumentData::Sequence(values));
        validate_argument_root(&value)?;
        Ok(value)
    }

    /// Creates a field-ID-sorted bounded record argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for duplicate fields or exceeded bounds.
    pub fn record(
        fields: impl IntoIterator<Item = ResolvedValueTypeField>,
    ) -> Result<Self, TypeIdentityError> {
        let fields = canonical_fields(fields)?;
        let value = Self(ResolvedValueTypeArgumentData::Record(fields));
        validate_argument_root(&value)?;
        Ok(value)
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        match &self.0 {
            ResolvedValueTypeArgumentData::Type(value) => {
                output.push(1);
                value.encode(output);
            }
            ResolvedValueTypeArgumentData::Bool(value) => {
                output.push(2);
                output.push(u8::from(*value));
            }
            ResolvedValueTypeArgumentData::Signed(value) => {
                output.push(3);
                output.extend_from_slice(&value.to_be_bytes());
            }
            ResolvedValueTypeArgumentData::Unsigned(value) => {
                output.push(4);
                output.extend_from_slice(&value.to_be_bytes());
            }
            ResolvedValueTypeArgumentData::Bytes(value) => {
                output.push(5);
                encode_bytes(output, value);
            }
            ResolvedValueTypeArgumentData::Utf8(value) => {
                output.push(6);
                encode_bytes(output, value.as_bytes());
            }
            ResolvedValueTypeArgumentData::Sequence(values) => {
                output.push(7);
                encode_len(output, values.len());
                for value in values {
                    value.encode(output);
                }
            }
            ResolvedValueTypeArgumentData::Record(fields) => {
                output.push(8);
                encode_len(output, fields.len());
                for field in fields {
                    output.extend_from_slice(&field.id.to_be_bytes());
                    field.value.encode(output);
                }
            }
        }
    }

    pub(super) fn visit_referenced_types(&self, visitor: &mut impl FnMut(&ResolvedValueType)) {
        match &self.0 {
            ResolvedValueTypeArgumentData::Type(value) => {
                visitor(value);
                value.visit_referenced_types(visitor);
            }
            ResolvedValueTypeArgumentData::Sequence(values) => {
                for value in values {
                    value.visit_referenced_types(visitor);
                }
            }
            ResolvedValueTypeArgumentData::Record(fields) => {
                for field in fields {
                    field.value.visit_referenced_types(visitor);
                }
            }
            ResolvedValueTypeArgumentData::Bool(_)
            | ResolvedValueTypeArgumentData::Signed(_)
            | ResolvedValueTypeArgumentData::Unsigned(_)
            | ResolvedValueTypeArgumentData::Bytes(_)
            | ResolvedValueTypeArgumentData::Utf8(_) => {}
        }
    }
}

/// One stable field in a canonical type record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedValueTypeField {
    id: u32,
    value: ResolvedValueTypeArgument,
}

impl ResolvedValueTypeField {
    /// Creates a field with a stable schema-local ID.
    #[must_use]
    pub const fn new(id: u32, value: ResolvedValueTypeArgument) -> Self {
        Self { id, value }
    }

    /// Returns the stable field ID.
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    /// Returns the canonical field value.
    #[must_use]
    pub const fn value(&self) -> &ResolvedValueTypeArgument {
        &self.value
    }
}

/// A host-canonical static encoded-numeric contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalEncodedNumericContract {
    fields: Vec<ResolvedValueTypeField>,
}

impl CanonicalEncodedNumericContract {
    /// Creates a nonempty field-ID-sorted static contract.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for empty, duplicate, or over-limit fields.
    pub fn new(
        fields: impl IntoIterator<Item = ResolvedValueTypeField>,
    ) -> Result<Self, TypeIdentityError> {
        let fields = canonical_fields(fields)?;
        if fields.is_empty() {
            return Err(TypeIdentityError::EmptyEncodedNumericContract);
        }
        let value = Self { fields };
        let argument =
            ResolvedValueTypeArgument(ResolvedValueTypeArgumentData::Record(value.fields.clone()));
        validate_argument_root(&argument)?;
        Ok(value)
    }

    /// Returns fields in canonical ascending ID order.
    #[must_use]
    pub fn fields(&self) -> &[ResolvedValueTypeField] {
        &self.fields
    }

    fn encode(&self, output: &mut Vec<u8>) {
        encode_len(output, self.fields.len());
        for field in &self.fields {
            output.extend_from_slice(&field.id.to_be_bytes());
            field.value.encode(output);
        }
    }
}

fn validate_component(component: IdentityComponent, value: &str) -> Result<(), TypeIdentityError> {
    if value.is_empty() {
        return Err(TypeIdentityError::EmptyComponent { component });
    }
    if value.len() > MAX_IDENTITY_COMPONENT_BYTES {
        return Err(TypeIdentityError::ComponentTooLong {
            component,
            bytes: value.len(),
        });
    }
    for (index, byte) in value.bytes().enumerate() {
        let valid =
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'.' | b'_' | b'-'));
        if !valid {
            return Err(TypeIdentityError::InvalidComponentCharacter {
                component,
                byte_index: index,
            });
        }
    }
    Ok(())
}

fn canonical_fields(
    fields: impl IntoIterator<Item = ResolvedValueTypeField>,
) -> Result<Vec<ResolvedValueTypeField>, TypeIdentityError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    validate_items(fields.len())?;
    fields.sort_unstable_by_key(|field| field.id);
    if let Some(field_id) = fields
        .windows(2)
        .find(|pair| pair[0].id == pair[1].id)
        .map(|pair| pair[0].id)
    {
        return Err(TypeIdentityError::DuplicateFieldId { field_id });
    }
    Ok(fields)
}

fn validate_items(items: usize) -> Result<(), TypeIdentityError> {
    if items > MAX_RESOLVED_TYPE_ITEMS {
        return Err(TypeIdentityError::TooManyItems { items });
    }
    Ok(())
}

fn validate_payload_len(bytes: usize) -> Result<(), TypeIdentityError> {
    if bytes > MAX_RESOLVED_TYPE_BYTES {
        return Err(TypeIdentityError::PayloadTooLarge { bytes });
    }
    Ok(())
}

#[derive(Default)]
struct ValidationBudget {
    nodes: usize,
    bytes: usize,
}

impl ValidationBudget {
    fn node(&mut self, depth: usize) -> Result<(), TypeIdentityError> {
        if depth > MAX_RESOLVED_TYPE_DEPTH {
            return Err(TypeIdentityError::NestingTooDeep);
        }
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(TypeIdentityError::TooManyNodes)?;
        if self.nodes > MAX_RESOLVED_TYPE_NODES {
            return Err(TypeIdentityError::TooManyNodes);
        }
        Ok(())
    }

    fn payload(&mut self, bytes: usize) -> Result<(), TypeIdentityError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(TypeIdentityError::TooManyPayloadBytes)?;
        if self.bytes > MAX_RESOLVED_TYPE_BYTES {
            return Err(TypeIdentityError::TooManyPayloadBytes);
        }
        Ok(())
    }
}

fn validate_resolved_type(value: &ResolvedValueType) -> Result<(), TypeIdentityError> {
    let mut budget = ValidationBudget::default();
    validate_type_at(value, 1, &mut budget)
}

fn validate_argument_root(value: &ResolvedValueTypeArgument) -> Result<(), TypeIdentityError> {
    let mut budget = ValidationBudget::default();
    validate_argument_at(value, 1, &mut budget)
}

fn validate_type_at(
    value: &ResolvedValueType,
    depth: usize,
    budget: &mut ValidationBudget,
) -> Result<(), TypeIdentityError> {
    budget.node(depth)?;
    match &value.0 {
        ResolvedValueTypeData::Nominal(_) => Ok(()),
        ResolvedValueTypeData::Parameterized { arguments, .. } => {
            validate_items(arguments.len())?;
            for argument in arguments {
                validate_argument_at(argument, depth + 1, budget)?;
            }
            Ok(())
        }
        ResolvedValueTypeData::EncodedNumeric { contract, .. } => {
            validate_items(contract.fields.len())?;
            for field in &contract.fields {
                validate_argument_at(&field.value, depth + 1, budget)?;
            }
            Ok(())
        }
    }
}

fn validate_argument_at(
    value: &ResolvedValueTypeArgument,
    depth: usize,
    budget: &mut ValidationBudget,
) -> Result<(), TypeIdentityError> {
    budget.node(depth)?;
    match &value.0 {
        ResolvedValueTypeArgumentData::Type(value) => validate_type_at(value, depth + 1, budget),
        ResolvedValueTypeArgumentData::Bytes(value) => budget.payload(value.len()),
        ResolvedValueTypeArgumentData::Utf8(value) => budget.payload(value.len()),
        ResolvedValueTypeArgumentData::Sequence(values) => {
            validate_items(values.len())?;
            for value in values {
                validate_argument_at(value, depth + 1, budget)?;
            }
            Ok(())
        }
        ResolvedValueTypeArgumentData::Record(fields) => {
            validate_items(fields.len())?;
            for field in fields {
                validate_argument_at(&field.value, depth + 1, budget)?;
            }
            Ok(())
        }
        ResolvedValueTypeArgumentData::Bool(_)
        | ResolvedValueTypeArgumentData::Signed(_)
        | ResolvedValueTypeArgumentData::Unsigned(_) => Ok(()),
    }
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("bounded canonical collection length fits u64")
            .to_be_bytes(),
    );
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str) -> TypeKey {
        TypeKey::new("tiler", name, 1).unwrap()
    }

    #[test]
    fn identity_components_use_a_portable_unambiguous_grammar() {
        assert!(TypeKey::new("com.acme", "fp8-special", 1).is_ok());
        assert!(matches!(
            TypeKey::new("tiler", "bad/name", 1),
            Err(TypeIdentityError::InvalidComponentCharacter { .. })
        ));
        assert_eq!(
            TypeKey::new("tiler", "f32", 0),
            Err(TypeIdentityError::ZeroSemanticVersion)
        );
    }

    #[test]
    fn records_sort_fields_and_reject_duplicates() {
        let record = ResolvedValueTypeArgument::record([
            ResolvedValueTypeField::new(9, ResolvedValueTypeArgument::unsigned(1)),
            ResolvedValueTypeField::new(2, ResolvedValueTypeArgument::boolean(true)),
        ])
        .unwrap();
        let duplicate = ResolvedValueTypeArgument::record([
            ResolvedValueTypeField::new(2, ResolvedValueTypeArgument::unsigned(1)),
            ResolvedValueTypeField::new(2, ResolvedValueTypeArgument::unsigned(2)),
        ]);

        let ResolvedValueTypeArgumentData::Record(fields) = record.0 else {
            panic!("record constructor produced another argument family")
        };
        assert_eq!(fields[0].id(), 2);
        assert_eq!(fields[1].id(), 9);
        assert_eq!(
            duplicate,
            Err(TypeIdentityError::DuplicateFieldId { field_id: 2 })
        );
    }

    #[test]
    fn canonical_encoding_distinguishes_all_outer_identity_domains() {
        let nominal = ResolvedValueType::nominal(key("f32"));
        let parameterized = ResolvedValueType::parameterized(
            key("complex"),
            [ResolvedValueTypeArgument::value_type(nominal.clone())],
        )
        .unwrap();
        let contract = CanonicalEncodedNumericContract::new([
            ResolvedValueTypeField::new(1, ResolvedValueTypeArgument::value_type(nominal.clone())),
            ResolvedValueTypeField::new(2, ResolvedValueTypeArgument::unsigned(32)),
        ])
        .unwrap();
        let encoded = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("ocp", "mxfp8", 1).unwrap(),
            contract,
        )
        .unwrap();

        assert_ne!(
            nominal.canonical_encoding(),
            parameterized.canonical_encoding()
        );
        assert_ne!(nominal.canonical_encoding(), encoded.canonical_encoding());
        assert_ne!(
            parameterized.canonical_encoding(),
            encoded.canonical_encoding()
        );
    }

    #[test]
    fn complete_structure_is_depth_bounded() {
        let mut argument = ResolvedValueTypeArgument::unsigned(1);
        for _ in 0..(MAX_RESOLVED_TYPE_DEPTH - 1) {
            argument = ResolvedValueTypeArgument::sequence([argument]).unwrap();
        }
        assert!(matches!(
            ResolvedValueTypeArgument::sequence([argument]),
            Err(TypeIdentityError::NestingTooDeep)
        ));
    }
}
