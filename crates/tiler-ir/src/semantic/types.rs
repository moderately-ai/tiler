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
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        Key::new(namespace.as_ref(), name.as_ref(), semantic_version).map(Self)
    }

    /// Validates and retains already-owned type-key components without copying them.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining invalid components.
    pub fn from_owned(
        namespace: String,
        name: String,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        Key::from_owned(namespace, name, semantic_version).map(Self)
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

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        self.0.encode(output);
    }

    pub(super) fn encoded_len(&self) -> usize {
        self.0.encoded_len()
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
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        Key::new(namespace.as_ref(), name.as_ref(), semantic_version).map(Self)
    }

    /// Validates and retains already-owned scheme-key components without copying them.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining invalid components.
    pub fn from_owned(
        namespace: String,
        name: String,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        Key::from_owned(namespace, name, semantic_version).map(Self)
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
    fn new(namespace: &str, name: &str, semantic_version: u32) -> Result<Self, TypeIdentityError> {
        validate_key(namespace, name, semantic_version)?;
        Ok(Self {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            semantic_version,
        })
    }

    fn from_owned(
        namespace: String,
        name: String,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        validate_key(&namespace, &name, semantic_version)?;
        Ok(Self {
            namespace,
            name,
            semantic_version,
        })
    }
}

pub(super) fn validate_key(
    namespace: &str,
    name: &str,
    semantic_version: u32,
) -> Result<(), TypeIdentityError> {
    validate_component(IdentityComponent::Namespace, namespace)?;
    validate_component(IdentityComponent::Name, name)?;
    if semantic_version == 0 {
        return Err(TypeIdentityError::ZeroSemanticVersion);
    }
    Ok(())
}

impl Key {
    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        encode_bytes(output, self.namespace.as_bytes());
        encode_bytes(output, self.name.as_bytes());
        output.extend_from_slice(&self.semantic_version.to_be_bytes());
    }

    fn encoded_len(&self) -> usize {
        std::mem::size_of::<u64>()
            .saturating_add(self.namespace.len())
            .saturating_add(std::mem::size_of::<u64>())
            .saturating_add(self.name.len())
            .saturating_add(std::mem::size_of::<u32>())
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
    /// An exact floating-point payload supplied no bits.
    EmptyFloatBits,
    /// Two record fields used the same stable field ID.
    DuplicateFieldId {
        /// Duplicated field identifier.
        field_id: AttributeFieldId,
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
            Self::EmptyFloatBits => {
                formatter.write_str("an exact floating-point payload requires at least one byte")
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
        arguments: TypeArguments,
    },
    EncodedNumeric {
        scheme: QuantSchemeKey,
        contract: EncodedNumericContract,
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
        arguments: TypeArguments,
    ) -> Result<Self, TypeIdentityError> {
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
        contract: EncodedNumericContract,
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

    /// Returns a parameterized constructor and its canonical arguments.
    #[must_use]
    pub const fn parameterized_parts(&self) -> Option<(&TypeKey, &TypeArguments)> {
        match &self.0 {
            ResolvedValueTypeData::Parameterized {
                constructor,
                arguments,
            } => Some((constructor, arguments)),
            ResolvedValueTypeData::Nominal(_) | ResolvedValueTypeData::EncodedNumeric { .. } => {
                None
            }
        }
    }

    /// Returns an encoded-numeric scheme and its static contract.
    #[must_use]
    pub const fn encoded_numeric_parts(
        &self,
    ) -> Option<(&QuantSchemeKey, &EncodedNumericContract)> {
        match &self.0 {
            ResolvedValueTypeData::EncodedNumeric { scheme, contract } => Some((scheme, contract)),
            ResolvedValueTypeData::Nominal(_) | ResolvedValueTypeData::Parameterized { .. } => None,
        }
    }

    /// Returns the collision-free versioned canonical identity bytes.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalResolvedValueType {
        let mut bytes = b"tiler.resolved-value-type.v2\0".to_vec();
        self.encode(&mut bytes);
        CanonicalResolvedValueType(bytes)
    }

    pub(super) fn canonical_encoded_len(&self) -> usize {
        b"tiler.resolved-value-type.v2\0"
            .len()
            .saturating_add(self.encoded_len())
    }

    pub(super) fn encoded_len(&self) -> usize {
        match &self.0 {
            ResolvedValueTypeData::Nominal(key) => 1_usize.saturating_add(key.0.encoded_len()),
            ResolvedValueTypeData::Parameterized {
                constructor,
                arguments,
            } => 1_usize
                .saturating_add(constructor.0.encoded_len())
                .saturating_add(std::mem::size_of::<u64>())
                .saturating_add(
                    arguments
                        .values
                        .iter()
                        .map(CanonicalValue::encoded_len)
                        .fold(0_usize, usize::saturating_add),
                ),
            ResolvedValueTypeData::EncodedNumeric { scheme, contract } => 1_usize
                .saturating_add(scheme.0.encoded_len())
                .saturating_add(contract.encoded_len()),
        }
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
                encode_len(output, arguments.values.len());
                for argument in &arguments.values {
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
                for argument in &arguments.values {
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

/// Ordered canonical arguments applied to one parameterized type constructor.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeArguments {
    values: Vec<CanonicalValue>,
}

impl TypeArguments {
    /// Creates one nonempty bounded argument list.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when the argument list is empty or exceeds
    /// a canonical structural bound.
    pub fn new(
        values: impl IntoIterator<Item = CanonicalValue>,
    ) -> Result<Self, TypeIdentityError> {
        let mut budget = ValidationBudget::default();
        budget.node(1)?;
        let values = collect_bounded_canonical_values(values, 2, &mut budget)?;
        if values.is_empty() {
            return Err(TypeIdentityError::EmptyTypeArguments);
        }
        Ok(Self { values })
    }

    /// Returns arguments in semantic order.
    #[must_use]
    pub fn values(&self) -> &[CanonicalValue] {
        &self.values
    }
}

/// One bounded canonical parameter of a resolved value type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalValue(CanonicalValueData);

/// Stable schema-local field identity for canonical records.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AttributeFieldId(u32);

impl AttributeFieldId {
    /// Creates a field identity from its portable fixed-width representation.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the portable fixed-width representation.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for AttributeFieldId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Width carried by one canonical signed or unsigned integer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CanonicalIntegerWidth {
    /// Eight bits.
    Bits8,
    /// Sixteen bits.
    Bits16,
    /// Thirty-two bits.
    Bits32,
    /// Sixty-four bits.
    Bits64,
}

impl CanonicalIntegerWidth {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Bits8 => 8,
            Self::Bits16 => 16,
            Self::Bits32 => 32,
            Self::Bits64 => 64,
        }
    }

    const fn byte_count(self) -> usize {
        match self {
            Self::Bits8 => 1,
            Self::Bits16 => 2,
            Self::Bits32 => 4,
            Self::Bits64 => 8,
        }
    }
}

/// Borrowed exact floating-point attribute payload.
#[derive(Clone, Copy, Debug)]
pub struct CanonicalFloatBitsRef<'a> {
    format: &'a TypeKey,
    bits: &'a [u8],
}

impl<'a> CanonicalFloatBitsRef<'a> {
    /// Returns the nominal floating-point format identity.
    #[must_use]
    pub const fn format(self) -> &'a TypeKey {
        self.format
    }

    /// Returns the exact big-endian payload bytes.
    #[must_use]
    pub const fn bits(self) -> &'a [u8] {
        self.bits
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum CanonicalValueData {
    Type(ResolvedValueType),
    Bool(bool),
    Signed {
        width: CanonicalIntegerWidth,
        bits: u64,
    },
    Unsigned {
        width: CanonicalIntegerWidth,
        bits: u64,
    },
    FloatBits {
        format: TypeKey,
        bits: Vec<u8>,
    },
    Bytes(Vec<u8>),
    Utf8(String),
    Sequence(Vec<CanonicalValue>),
    Record(Vec<CanonicalField>),
}

/// Borrowed inspection of one host-canonical value.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CanonicalValueView<'a> {
    /// A complete resolved semantic value type.
    Type(&'a ResolvedValueType),
    /// A Boolean value.
    Bool(bool),
    /// A signed fixed-width host value.
    Signed {
        /// Declared integer width.
        width: CanonicalIntegerWidth,
        /// Exact two's-complement bits in the low part of this word.
        bits: u64,
    },
    /// An unsigned fixed-width host value.
    Unsigned {
        /// Declared integer width.
        width: CanonicalIntegerWidth,
        /// Exact unsigned bits in the low part of this word.
        bits: u64,
    },
    /// Exact floating-point bits with an explicit nominal format.
    FloatBits(CanonicalFloatBitsRef<'a>),
    /// Exact bytes.
    Bytes(&'a [u8]),
    /// Exact UTF-8.
    Utf8(&'a str),
    /// An ordered sequence.
    Sequence(&'a [CanonicalValue]),
    /// A stable-field-ID record.
    Record(&'a [CanonicalField]),
}

impl CanonicalValue {
    /// Returns a borrowed, exhaustively tagged view of this canonical value.
    #[must_use]
    pub fn view(&self) -> CanonicalValueView<'_> {
        match &self.0 {
            CanonicalValueData::Type(value) => CanonicalValueView::Type(value),
            CanonicalValueData::Bool(value) => CanonicalValueView::Bool(*value),
            CanonicalValueData::Signed { width, bits } => CanonicalValueView::Signed {
                width: *width,
                bits: *bits,
            },
            CanonicalValueData::Unsigned { width, bits } => CanonicalValueView::Unsigned {
                width: *width,
                bits: *bits,
            },
            CanonicalValueData::FloatBits { format, bits } => {
                CanonicalValueView::FloatBits(CanonicalFloatBitsRef { format, bits })
            }
            CanonicalValueData::Bytes(value) => CanonicalValueView::Bytes(value),
            CanonicalValueData::Utf8(value) => CanonicalValueView::Utf8(value),
            CanonicalValueData::Sequence(value) => CanonicalValueView::Sequence(value),
            CanonicalValueData::Record(value) => CanonicalValueView::Record(value),
        }
    }
    /// Creates a nested resolved-type argument.
    #[must_use]
    pub const fn value_type(value: ResolvedValueType) -> Self {
        Self(CanonicalValueData::Type(value))
    }

    /// Creates a Boolean argument.
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self(CanonicalValueData::Bool(value))
    }

    /// Creates a canonical signed `i8` argument.
    #[must_use]
    pub const fn signed_i8(value: i8) -> Self {
        Self::signed_bits(CanonicalIntegerWidth::Bits8, value.cast_unsigned() as u64)
    }

    /// Creates a canonical signed `i16` argument.
    #[must_use]
    pub const fn signed_i16(value: i16) -> Self {
        Self::signed_bits(CanonicalIntegerWidth::Bits16, value.cast_unsigned() as u64)
    }

    /// Creates a canonical signed `i32` argument.
    #[must_use]
    pub const fn signed_i32(value: i32) -> Self {
        Self::signed_bits(CanonicalIntegerWidth::Bits32, value.cast_unsigned() as u64)
    }

    /// Creates a canonical signed `i64` argument.
    #[must_use]
    pub const fn signed_i64(value: i64) -> Self {
        Self::signed_bits(CanonicalIntegerWidth::Bits64, value.cast_unsigned())
    }

    const fn signed_bits(width: CanonicalIntegerWidth, bits: u64) -> Self {
        Self(CanonicalValueData::Signed { width, bits })
    }

    /// Creates a canonical unsigned `u8` argument.
    #[must_use]
    pub const fn unsigned_u8(value: u8) -> Self {
        Self::unsigned_bits(CanonicalIntegerWidth::Bits8, value as u64)
    }

    /// Creates a canonical unsigned `u16` argument.
    #[must_use]
    pub const fn unsigned_u16(value: u16) -> Self {
        Self::unsigned_bits(CanonicalIntegerWidth::Bits16, value as u64)
    }

    /// Creates a canonical unsigned `u32` argument.
    #[must_use]
    pub const fn unsigned_u32(value: u32) -> Self {
        Self::unsigned_bits(CanonicalIntegerWidth::Bits32, value as u64)
    }

    /// Creates a canonical unsigned `u64` argument.
    #[must_use]
    pub const fn unsigned_u64(value: u64) -> Self {
        Self::unsigned_bits(CanonicalIntegerWidth::Bits64, value)
    }

    const fn unsigned_bits(width: CanonicalIntegerWidth, bits: u64) -> Self {
        Self(CanonicalValueData::Unsigned { width, bits })
    }

    /// Creates an exact floating-point payload with explicit format identity.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when the payload is empty or exceeds canonical bounds.
    pub fn float_bits(format: TypeKey, bits: impl AsRef<[u8]>) -> Result<Self, TypeIdentityError> {
        let bits = bits.as_ref();
        if bits.is_empty() {
            return Err(TypeIdentityError::EmptyFloatBits);
        }
        validate_payload_len(bits.len())?;
        Ok(Self(CanonicalValueData::FloatBits {
            format,
            bits: bits.to_vec(),
        }))
    }

    /// Validates and retains already-owned floating-point bits without copying them.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining an empty or oversized payload.
    pub fn float_bits_owned(format: TypeKey, bits: Vec<u8>) -> Result<Self, TypeIdentityError> {
        if bits.is_empty() {
            return Err(TypeIdentityError::EmptyFloatBits);
        }
        validate_payload_len(bits.len())?;
        Ok(Self(CanonicalValueData::FloatBits { format, bits }))
    }

    /// Creates a bounded exact byte argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError::PayloadTooLarge`] when over the byte bound.
    pub fn bytes(value: impl AsRef<[u8]>) -> Result<Self, TypeIdentityError> {
        let value = value.as_ref();
        validate_payload_len(value.len())?;
        Ok(Self(CanonicalValueData::Bytes(value.to_vec())))
    }

    /// Validates and retains an already-owned byte payload without copying it.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining an oversized payload.
    pub fn bytes_owned(value: Vec<u8>) -> Result<Self, TypeIdentityError> {
        validate_payload_len(value.len())?;
        Ok(Self(CanonicalValueData::Bytes(value)))
    }

    /// Creates a bounded exact UTF-8 argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError::PayloadTooLarge`] when over the byte bound.
    pub fn utf8(value: impl AsRef<str>) -> Result<Self, TypeIdentityError> {
        let value = value.as_ref();
        validate_payload_len(value.len())?;
        Ok(Self(CanonicalValueData::Utf8(value.to_owned())))
    }

    /// Validates and retains an already-owned UTF-8 payload without copying it.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining an oversized payload.
    pub fn utf8_owned(value: String) -> Result<Self, TypeIdentityError> {
        validate_payload_len(value.len())?;
        Ok(Self(CanonicalValueData::Utf8(value)))
    }

    /// Creates an ordered bounded sequence argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when a canonical bound is exceeded.
    pub fn sequence(values: impl IntoIterator<Item = Self>) -> Result<Self, TypeIdentityError> {
        let mut budget = ValidationBudget::default();
        budget.node(1)?;
        let values = collect_bounded_canonical_values(values, 2, &mut budget)?;
        Ok(Self(CanonicalValueData::Sequence(values)))
    }

    /// Creates a field-ID-sorted bounded record argument.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for duplicate fields or exceeded bounds.
    pub fn record(
        fields: impl IntoIterator<Item = CanonicalField>,
    ) -> Result<Self, TypeIdentityError> {
        let fields = canonical_fields(fields)?;
        Ok(Self(CanonicalValueData::Record(fields)))
    }

    pub(super) fn into_record(self) -> Option<Vec<CanonicalField>> {
        match self.0 {
            CanonicalValueData::Record(fields) => Some(fields),
            _ => None,
        }
    }

    pub(crate) fn encode(&self, output: &mut Vec<u8>) {
        match &self.0 {
            CanonicalValueData::Type(value) => {
                output.push(1);
                value.encode(output);
            }
            CanonicalValueData::Bool(value) => {
                output.push(2);
                output.push(u8::from(*value));
            }
            CanonicalValueData::Signed { width, bits } => {
                output.push(3);
                output.push(width.tag());
                let bytes = bits.to_be_bytes();
                output.extend_from_slice(&bytes[bytes.len() - width.byte_count()..]);
            }
            CanonicalValueData::Unsigned { width, bits } => {
                output.push(4);
                output.push(width.tag());
                let bytes = bits.to_be_bytes();
                output.extend_from_slice(&bytes[bytes.len() - width.byte_count()..]);
            }
            CanonicalValueData::FloatBits { format, bits } => {
                output.push(5);
                format.encode(output);
                encode_bytes(output, bits);
            }
            CanonicalValueData::Bytes(value) => {
                output.push(6);
                encode_bytes(output, value);
            }
            CanonicalValueData::Utf8(value) => {
                output.push(7);
                encode_bytes(output, value.as_bytes());
            }
            CanonicalValueData::Sequence(values) => {
                output.push(8);
                encode_len(output, values.len());
                for value in values {
                    value.encode(output);
                }
            }
            CanonicalValueData::Record(fields) => {
                output.push(9);
                encode_len(output, fields.len());
                for field in fields {
                    output.extend_from_slice(&field.id.get().to_be_bytes());
                    field.value.encode(output);
                }
            }
        }
    }

    pub(crate) fn encoded_len(&self) -> usize {
        match &self.0 {
            CanonicalValueData::Type(value) => 1_usize.saturating_add(value.encoded_len()),
            CanonicalValueData::Bool(_) => 2,
            CanonicalValueData::Signed { width, .. }
            | CanonicalValueData::Unsigned { width, .. } => {
                2_usize.saturating_add(width.byte_count())
            }
            CanonicalValueData::FloatBits { format, bits } => 1_usize
                .saturating_add(format.0.encoded_len())
                .saturating_add(std::mem::size_of::<u64>())
                .saturating_add(bits.len()),
            CanonicalValueData::Bytes(value) => 1_usize
                .saturating_add(std::mem::size_of::<u64>())
                .saturating_add(value.len()),
            CanonicalValueData::Utf8(value) => 1_usize
                .saturating_add(std::mem::size_of::<u64>())
                .saturating_add(value.len()),
            CanonicalValueData::Sequence(values) => 1_usize
                .saturating_add(std::mem::size_of::<u64>())
                .saturating_add(
                    values
                        .iter()
                        .map(Self::encoded_len)
                        .fold(0_usize, usize::saturating_add),
                ),
            CanonicalValueData::Record(fields) => 1_usize
                .saturating_add(std::mem::size_of::<u64>())
                .saturating_add(
                    fields
                        .iter()
                        .map(|field| {
                            std::mem::size_of::<u32>().saturating_add(field.value.encoded_len())
                        })
                        .fold(0_usize, usize::saturating_add),
                ),
        }
    }

    pub(super) fn visit_referenced_types(&self, visitor: &mut impl FnMut(&ResolvedValueType)) {
        match &self.0 {
            CanonicalValueData::Type(value) => {
                visitor(value);
                value.visit_referenced_types(visitor);
            }
            CanonicalValueData::FloatBits { format, .. } => {
                visitor(&ResolvedValueType::nominal(format.clone()));
            }
            CanonicalValueData::Sequence(values) => {
                for value in values {
                    value.visit_referenced_types(visitor);
                }
            }
            CanonicalValueData::Record(fields) => {
                for field in fields {
                    field.value.visit_referenced_types(visitor);
                }
            }
            CanonicalValueData::Bool(_)
            | CanonicalValueData::Signed { .. }
            | CanonicalValueData::Unsigned { .. }
            | CanonicalValueData::Bytes(_)
            | CanonicalValueData::Utf8(_) => {}
        }
    }
}

/// One stable field in a canonical type record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalField {
    id: AttributeFieldId,
    value: CanonicalValue,
}

impl CanonicalField {
    /// Creates a field with a stable schema-local ID.
    #[must_use]
    pub const fn new(id: AttributeFieldId, value: CanonicalValue) -> Self {
        Self { id, value }
    }

    /// Returns the stable field ID.
    #[must_use]
    pub const fn id(&self) -> AttributeFieldId {
        self.id
    }

    /// Returns the canonical field value.
    #[must_use]
    pub const fn value(&self) -> &CanonicalValue {
        &self.value
    }
}

/// A host-canonical static encoded-numeric contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EncodedNumericContract {
    fields: Vec<CanonicalField>,
}

impl EncodedNumericContract {
    /// Creates a nonempty field-ID-sorted static contract.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for empty, duplicate, or over-limit fields.
    pub fn new(
        fields: impl IntoIterator<Item = CanonicalField>,
    ) -> Result<Self, TypeIdentityError> {
        let fields = canonical_fields(fields)?;
        if fields.is_empty() {
            return Err(TypeIdentityError::EmptyEncodedNumericContract);
        }
        Ok(Self { fields })
    }

    /// Returns fields in canonical ascending ID order.
    #[must_use]
    pub fn fields(&self) -> &[CanonicalField] {
        &self.fields
    }

    fn encode(&self, output: &mut Vec<u8>) {
        encode_len(output, self.fields.len());
        for field in &self.fields {
            output.extend_from_slice(&field.id.get().to_be_bytes());
            field.value.encode(output);
        }
    }

    fn encoded_len(&self) -> usize {
        std::mem::size_of::<u64>().saturating_add(
            self.fields
                .iter()
                .map(|field| std::mem::size_of::<u32>().saturating_add(field.value.encoded_len()))
                .fold(0_usize, usize::saturating_add),
        )
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
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<CanonicalField>, TypeIdentityError> {
    let mut fields = fields.into_iter();
    let mut collected = Vec::new();
    let mut budget = ValidationBudget::default();
    budget.node(1)?;
    for field in fields
        .by_ref()
        .take(MAX_RESOLVED_TYPE_ITEMS.saturating_add(1))
    {
        if collected.len() == MAX_RESOLVED_TYPE_ITEMS {
            return Err(TypeIdentityError::TooManyItems {
                items: MAX_RESOLVED_TYPE_ITEMS.saturating_add(1),
            });
        }
        validate_argument_at(&field.value, 2, &mut budget)?;
        collected.push(field);
    }
    let mut fields = collected;
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

fn collect_bounded_canonical_values(
    values: impl IntoIterator<Item = CanonicalValue>,
    depth: usize,
    budget: &mut ValidationBudget,
) -> Result<Vec<CanonicalValue>, TypeIdentityError> {
    let mut values = values.into_iter();
    let mut collected = Vec::new();
    for item in values
        .by_ref()
        .take(MAX_RESOLVED_TYPE_ITEMS.saturating_add(1))
    {
        if collected.len() == MAX_RESOLVED_TYPE_ITEMS {
            return Err(TypeIdentityError::TooManyItems {
                items: MAX_RESOLVED_TYPE_ITEMS.saturating_add(1),
            });
        }
        validate_argument_at(&item, depth, budget)?;
        collected.push(item);
    }
    Ok(collected)
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

/// Validates one standalone canonical value against the complete structural bounds.
///
/// The collection constructors validate their own items, but
/// [`CanonicalValue::value_type`] wraps an already-admitted resolved type
/// without re-measuring it, which adds one structural level. Any boundary that
/// retains a caller-supplied canonical value must therefore re-measure the
/// complete value before it reaches durable identity.
pub(super) fn validate_canonical_value(value: &CanonicalValue) -> Result<(), TypeIdentityError> {
    let mut budget = ValidationBudget::default();
    validate_argument_at(value, 1, &mut budget)
}

/// Returns a resolved type whose deepest node sits exactly on
/// [`MAX_RESOLVED_TYPE_DEPTH`], so any further wrapping costs one level too many.
#[cfg(test)]
pub(super) fn maximal_depth_resolved_type() -> ResolvedValueType {
    let wrap = |argument| {
        ResolvedValueType::parameterized(
            TypeKey::new("tiler", "wrap", 1).expect("the wrapper key is canonical"),
            TypeArguments::new([argument]).expect("each wrapper stays within its bound"),
        )
        .expect("each wrapper stays within its bound")
    };
    // One parameterized wrapper occupies two structural levels: its own type
    // node and its argument node.
    let mut value = wrap(CanonicalValue::boolean(true));
    for _ in 1..MAX_RESOLVED_TYPE_DEPTH / 2 {
        value = wrap(CanonicalValue::value_type(value));
    }
    value
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
            validate_items(arguments.values.len())?;
            for argument in &arguments.values {
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
    value: &CanonicalValue,
    depth: usize,
    budget: &mut ValidationBudget,
) -> Result<(), TypeIdentityError> {
    budget.node(depth)?;
    match &value.0 {
        CanonicalValueData::Type(value) => validate_type_at(value, depth + 1, budget),
        CanonicalValueData::FloatBits { bits, .. } | CanonicalValueData::Bytes(bits) => {
            budget.payload(bits.len())
        }
        CanonicalValueData::Utf8(value) => budget.payload(value.len()),
        CanonicalValueData::Sequence(values) => {
            validate_items(values.len())?;
            for value in values {
                validate_argument_at(value, depth + 1, budget)?;
            }
            Ok(())
        }
        CanonicalValueData::Record(fields) => {
            validate_items(fields.len())?;
            for field in fields {
                validate_argument_at(&field.value, depth + 1, budget)?;
            }
            Ok(())
        }
        CanonicalValueData::Bool(_)
        | CanonicalValueData::Signed { .. }
        | CanonicalValueData::Unsigned { .. } => Ok(()),
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
    fn structural_encoded_lengths_match_canonical_encoding() {
        let nominal = ResolvedValueType::nominal(key("scalar"));
        let values = [
            CanonicalValue::value_type(nominal.clone()),
            CanonicalValue::boolean(true),
            CanonicalValue::signed_i8(-1),
            CanonicalValue::signed_i16(-2),
            CanonicalValue::signed_i32(-3),
            CanonicalValue::signed_i64(-4),
            CanonicalValue::unsigned_u8(1),
            CanonicalValue::unsigned_u16(2),
            CanonicalValue::unsigned_u32(3),
            CanonicalValue::unsigned_u64(4),
            CanonicalValue::float_bits(key("float"), [0x3f, 0x80, 0, 0]).unwrap(),
            CanonicalValue::bytes([1, 2, 3]).unwrap(),
            CanonicalValue::utf8("value").unwrap(),
            CanonicalValue::sequence([CanonicalValue::boolean(false)]).unwrap(),
            CanonicalValue::record([CanonicalField::new(
                AttributeFieldId::new(7),
                CanonicalValue::unsigned_u32(9),
            )])
            .unwrap(),
        ];
        for value in &values {
            let mut encoded = Vec::new();
            value.encode(&mut encoded);
            assert_eq!(value.encoded_len(), encoded.len());
        }

        let parameterized = ResolvedValueType::parameterized(
            key("container"),
            TypeArguments::new(values.clone()).unwrap(),
        )
        .unwrap();
        let encoded_numeric = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("tiler", "quant", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::value_type(nominal.clone()),
            )])
            .unwrap(),
        )
        .unwrap();
        for value_type in [nominal, parameterized, encoded_numeric] {
            assert_eq!(
                value_type.canonical_encoded_len(),
                value_type.canonical_encoding().as_bytes().len()
            );
        }
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
    fn bounded_borrowed_values_validate_before_owned_retention() {
        struct BorrowedText<'a>(&'a str);
        impl AsRef<str> for BorrowedText<'_> {
            fn as_ref(&self) -> &str {
                self.0
            }
        }
        struct BorrowedBytes<'a>(&'a [u8]);
        impl AsRef<[u8]> for BorrowedBytes<'_> {
            fn as_ref(&self) -> &[u8] {
                self.0
            }
        }

        let oversized_component = "x".repeat(MAX_IDENTITY_COMPONENT_BYTES + 1);
        assert!(matches!(
            TypeKey::new(BorrowedText(&oversized_component), "name", 1),
            Err(TypeIdentityError::ComponentTooLong { .. })
        ));
        assert!(matches!(
            QuantSchemeKey::new("namespace", BorrowedText(&oversized_component), 1),
            Err(TypeIdentityError::ComponentTooLong { .. })
        ));

        let namespace = String::from("owned-namespace");
        let name = String::from("owned-name");
        let namespace_pointer = namespace.as_ptr();
        let name_pointer = name.as_ptr();
        let retained = TypeKey::from_owned(namespace, name, 1).unwrap();
        assert_eq!(retained.namespace().as_ptr(), namespace_pointer);
        assert_eq!(retained.name().as_ptr(), name_pointer);

        let oversized_payload = vec![0_u8; MAX_RESOLVED_TYPE_BYTES + 1];
        assert_eq!(
            CanonicalValue::bytes(BorrowedBytes(&oversized_payload)),
            Err(TypeIdentityError::PayloadTooLarge {
                bytes: MAX_RESOLVED_TYPE_BYTES + 1,
            })
        );
        assert_eq!(
            CanonicalValue::utf8(BorrowedText(
                std::str::from_utf8(&vec![b'x'; MAX_RESOLVED_TYPE_BYTES + 1]).unwrap()
            )),
            Err(TypeIdentityError::PayloadTooLarge {
                bytes: MAX_RESOLVED_TYPE_BYTES + 1,
            })
        );
        assert_eq!(
            CanonicalValue::float_bits(key("f32"), BorrowedBytes(&oversized_payload)),
            Err(TypeIdentityError::PayloadTooLarge {
                bytes: MAX_RESOLVED_TYPE_BYTES + 1,
            })
        );

        let owned_bytes = vec![1_u8, 2, 3];
        let byte_pointer = owned_bytes.as_ptr();
        let retained = CanonicalValue::bytes_owned(owned_bytes).unwrap();
        let CanonicalValueView::Bytes(retained) = retained.view() else {
            panic!("byte constructor must preserve its canonical kind")
        };
        assert_eq!(retained.as_ptr(), byte_pointer);

        let owned_bits = vec![0x3f_u8, 0x80, 0, 0];
        let bits_pointer = owned_bits.as_ptr();
        let retained = CanonicalValue::float_bits_owned(key("f32"), owned_bits).unwrap();
        let CanonicalValueView::FloatBits(retained) = retained.view() else {
            panic!("float-bits constructor must preserve its canonical kind")
        };
        assert_eq!(retained.bits().as_ptr(), bits_pointer);

        let owned_text = String::from("owned-text");
        let text_pointer = owned_text.as_ptr();
        let retained = CanonicalValue::utf8_owned(owned_text).unwrap();
        let CanonicalValueView::Utf8(retained) = retained.view() else {
            panic!("UTF-8 constructor must preserve its canonical kind")
        };
        assert_eq!(retained.as_ptr(), text_pointer);
    }

    #[test]
    fn records_sort_fields_and_reject_duplicates() {
        let record = CanonicalValue::record([
            CanonicalField::new(AttributeFieldId::new(9), CanonicalValue::unsigned_u32(1)),
            CanonicalField::new(AttributeFieldId::new(2), CanonicalValue::boolean(true)),
        ])
        .unwrap();
        let duplicate = CanonicalValue::record([
            CanonicalField::new(AttributeFieldId::new(2), CanonicalValue::unsigned_u32(1)),
            CanonicalField::new(AttributeFieldId::new(2), CanonicalValue::unsigned_u32(2)),
        ]);

        let CanonicalValueData::Record(fields) = record.0 else {
            panic!("record constructor produced another argument family")
        };
        assert_eq!(fields[0].id(), AttributeFieldId::new(2));
        assert_eq!(fields[1].id(), AttributeFieldId::new(9));
        assert_eq!(
            duplicate,
            Err(TypeIdentityError::DuplicateFieldId {
                field_id: AttributeFieldId::new(2)
            })
        );
    }

    #[test]
    fn canonical_numeric_values_preserve_width_format_and_exact_bits() {
        fn encoding(value: &CanonicalValue) -> Vec<u8> {
            let mut output = Vec::new();
            value.encode(&mut output);
            output
        }

        assert_ne!(
            encoding(&CanonicalValue::unsigned_u8(1)),
            encoding(&CanonicalValue::unsigned_u32(1))
        );
        assert_eq!(encoding(&CanonicalValue::unsigned_u8(1)), [4, 8, 1]);
        assert_eq!(
            encoding(&CanonicalValue::unsigned_u32(1)),
            [4, 32, 0, 0, 0, 1]
        );
        assert_ne!(
            encoding(&CanonicalValue::signed_i8(-1)),
            encoding(&CanonicalValue::unsigned_u8(u8::MAX))
        );
        assert_eq!(encoding(&CanonicalValue::signed_i8(-1)), [3, 8, 255]);
        let f32_bits = CanonicalValue::float_bits(key("f32"), 1_u32.to_be_bytes()).unwrap();
        let bf16_bits = CanonicalValue::float_bits(key("bf16"), 1_u16.to_be_bytes()).unwrap();
        assert_ne!(encoding(&f32_bits), encoding(&bf16_bits));
        assert_eq!(
            CanonicalValue::float_bits(key("f32"), []),
            Err(TypeIdentityError::EmptyFloatBits)
        );
    }

    #[test]
    fn canonical_encoding_distinguishes_all_outer_identity_domains() {
        let nominal = ResolvedValueType::nominal(key("f32"));
        let parameterized = ResolvedValueType::parameterized(
            key("complex"),
            TypeArguments::new([CanonicalValue::value_type(nominal.clone())]).unwrap(),
        )
        .unwrap();
        let contract = EncodedNumericContract::new([
            CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::value_type(nominal.clone()),
            ),
            CanonicalField::new(AttributeFieldId::new(2), CanonicalValue::unsigned_u32(32)),
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
        let mut argument = CanonicalValue::unsigned_u32(1);
        for _ in 0..(MAX_RESOLVED_TYPE_DEPTH - 1) {
            argument = CanonicalValue::sequence([argument]).unwrap();
        }
        assert!(matches!(
            CanonicalValue::sequence([argument]),
            Err(TypeIdentityError::NestingTooDeep)
        ));
    }

    #[test]
    fn wrapping_a_maximal_resolved_type_is_rejected_by_standalone_validation() {
        let maximal = maximal_depth_resolved_type();
        assert_eq!(
            validate_canonical_value(&CanonicalValue::value_type(maximal.clone())),
            Err(TypeIdentityError::NestingTooDeep)
        );
        assert_eq!(
            TypeArguments::new([CanonicalValue::value_type(maximal.clone())]),
            Err(TypeIdentityError::NestingTooDeep)
        );
        // The wrapped type is itself admitted, so only the added level is at fault.
        assert!(validate_resolved_type(&maximal).is_ok());
    }

    #[test]
    fn arbitrary_iterators_stop_at_the_first_over_limit_item() {
        assert_eq!(
            TypeArguments::new(std::iter::repeat(CanonicalValue::boolean(true))),
            Err(TypeIdentityError::TooManyItems {
                items: MAX_RESOLVED_TYPE_ITEMS + 1,
            })
        );
        assert_eq!(
            CanonicalValue::sequence(std::iter::repeat(CanonicalValue::boolean(true))),
            Err(TypeIdentityError::TooManyItems {
                items: MAX_RESOLVED_TYPE_ITEMS + 1,
            })
        );
        assert_eq!(
            CanonicalValue::record(std::iter::repeat_with(|| CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::boolean(true),
            ))),
            Err(TypeIdentityError::TooManyItems {
                items: MAX_RESOLVED_TYPE_ITEMS + 1,
            })
        );
    }

    #[test]
    fn canonical_collections_enforce_aggregate_payload_before_item_count() {
        fn payload() -> CanonicalValue {
            CanonicalValue::bytes_owned(vec![0_u8; MAX_RESOLVED_TYPE_BYTES / 2 + 1]).unwrap()
        }

        assert_eq!(
            CanonicalValue::sequence([payload(), payload()]),
            Err(TypeIdentityError::TooManyPayloadBytes)
        );
        assert_eq!(
            TypeArguments::new([payload(), payload()]),
            Err(TypeIdentityError::TooManyPayloadBytes)
        );
        assert_eq!(
            CanonicalValue::record([
                CanonicalField::new(AttributeFieldId::new(1), payload()),
                CanonicalField::new(AttributeFieldId::new(2), payload()),
            ]),
            Err(TypeIdentityError::TooManyPayloadBytes)
        );
    }
}
