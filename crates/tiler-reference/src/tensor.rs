//! The reference implementation's tensor representation.
//!
//! An element is an opaque byte run, a component names a role within a
//! compound value, and a [`Tensor`] pairs a shape with either. Nothing here
//! evaluates anything: this module owns what a reference value *is*, and the
//! registry and evaluator own what may be done with one.

use std::sync::Arc;

use tiler_ir::semantic::{EncodedComponentRole, InputKey, ResolvedValueType};
use tiler_ir::shape::Shape;

use super::error::{EvaluationError, ReferenceResource};
use super::evaluate::{ReferenceWork, validate_compound_resources};
use super::{
    MAX_REFERENCE_ELEMENT_BYTES, MAX_REFERENCE_TENSOR_BYTES, MAX_REFERENCE_TENSOR_ELEMENTS,
};

/// Byte order supplied when constructing exact floating-point elements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FloatBitOrder {
    /// Most-significant byte first, which is Tiler's canonical representation.
    MostSignificantByteFirst,
    /// Least-significant byte first; construction normalizes it to canonical order.
    LeastSignificantByteFirst,
}

/// One exact canonical logical element in a dense reference tensor.
///
/// The enclosing tensor's resolved semantic type and registered reference
/// validator define how these bytes are interpreted. Compound values use
/// [`ReferenceComponent`] tensors instead of embedding an untyped recursive
/// scalar structure.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReferenceElement(Vec<u8>);

impl ReferenceElement {
    /// Creates one bounded canonical element representation.
    ///
    /// # Errors
    ///
    /// Returns a resource error before retaining an oversized element.
    pub fn new(bytes: impl AsRef<[u8]>) -> Result<Self, EvaluationError> {
        let bytes = bytes.as_ref();
        if bytes.len() > MAX_REFERENCE_ELEMENT_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::ElementBytes,
                limit: MAX_REFERENCE_ELEMENT_BYTES,
                actual: bytes.len(),
            });
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Creates exact floating-point bits and normalizes them to canonical
    /// most-significant-byte-first order.
    ///
    /// The byte order is part of this public constructor rather than inherited
    /// from the host. The resolved tensor type remains the authority for the
    /// floating-point format and therefore for the required payload width.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::EmptyFloatBits`] for an empty payload or a
    /// resource error for an oversized payload.
    pub fn from_float_bits(
        bits: impl AsRef<[u8]>,
        order: FloatBitOrder,
    ) -> Result<Self, EvaluationError> {
        let bits = bits.as_ref();
        if bits.is_empty() {
            return Err(EvaluationError::EmptyFloatBits);
        }
        if bits.len() > MAX_REFERENCE_ELEMENT_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::ElementBytes,
                limit: MAX_REFERENCE_ELEMENT_BYTES,
                actual: bits.len(),
            });
        }
        let mut canonical = bits.to_vec();
        if order == FloatBitOrder::LeastSignificantByteFirst {
            canonical.reverse();
        }
        Ok(Self(canonical))
    }

    /// Returns exact canonical element bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Stable schema-local role shared with the semantic encoded-value contract.
pub type ReferenceComponentRole = EncodedComponentRole;

/// One stable-role tensor component of a compound logical reference value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceComponent {
    role: ReferenceComponentRole,
    tensor: Tensor,
}

impl ReferenceComponent {
    /// Creates one compound component.
    #[must_use]
    pub const fn new(role: ReferenceComponentRole, tensor: Tensor) -> Self {
        Self { role, tensor }
    }

    /// Returns the stable component role.
    #[must_use]
    pub const fn role(&self) -> ReferenceComponentRole {
        self.role
    }

    /// Returns the exact component tensor.
    #[must_use]
    pub const fn tensor(&self) -> &Tensor {
        &self.tensor
    }
}

/// Borrowed representation of one reference tensor payload.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum TensorPayloadView<'a> {
    /// Dense exact elements in logical row-major order.
    Dense(&'a [ReferenceElement]),
    /// Ordered stable-role component tensors for one compound logical value.
    Compound(&'a [ReferenceComponent]),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TensorPayload {
    Dense(Vec<ReferenceElement>),
    Compound(Vec<ReferenceComponent>),
}

/// An owned, exact, dense row-major tensor used by the reference evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tensor(Arc<TensorData>);

#[derive(Debug, Eq, PartialEq)]
struct TensorData {
    resolved_type: ResolvedValueType,
    shape: Shape,
    payload: TensorPayload,
}

impl Tensor {
    /// Creates a tensor after checking its resolved type, element count, and
    /// aggregate retained-byte bounds.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::ElementCount`] when the payload length does
    /// not match the shape, or [`EvaluationError::ShapeTooLarge`] when the
    /// element count cannot be represented on this host.
    pub fn dense(
        resolved_type: ResolvedValueType,
        shape: Shape,
        elements: Vec<ReferenceElement>,
    ) -> Result<Self, EvaluationError> {
        let expected = shape
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?;
        if expected > MAX_REFERENCE_TENSOR_ELEMENTS {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual: expected,
            });
        }
        if elements.len() != expected {
            return Err(EvaluationError::ElementCount {
                expected,
                actual: elements.len(),
            });
        }
        let bytes = elements.iter().try_fold(0_usize, |bytes, element| {
            bytes
                .checked_add(element.as_bytes().len())
                .ok_or(EvaluationError::ResourceExceeded {
                    resource: ReferenceResource::TensorBytes,
                    limit: MAX_REFERENCE_TENSOR_BYTES,
                    actual: usize::MAX,
                })
        })?;
        if bytes > MAX_REFERENCE_TENSOR_BYTES {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorBytes,
                limit: MAX_REFERENCE_TENSOR_BYTES,
                actual: bytes,
            });
        }
        Ok(Self(Arc::new(TensorData {
            resolved_type,
            shape,
            payload: TensorPayload::Dense(elements),
        })))
    }

    /// Creates a compound tensor from ordered stable-role component tensors.
    ///
    /// Component shapes are intentionally independent: the resolved compound
    /// type's validator owns role, type, and shape relationships.
    ///
    /// # Errors
    ///
    /// Returns a typed resource error before retaining an over-limit compound value.
    pub fn compound(
        resolved_type: ResolvedValueType,
        shape: Shape,
        components: Vec<ReferenceComponent>,
    ) -> Result<Self, EvaluationError> {
        let logical_elements = shape
            .element_count()
            .ok_or(EvaluationError::ShapeTooLarge)?;
        if logical_elements > MAX_REFERENCE_TENSOR_ELEMENTS {
            return Err(EvaluationError::ResourceExceeded {
                resource: ReferenceResource::TensorElements,
                limit: MAX_REFERENCE_TENSOR_ELEMENTS,
                actual: logical_elements,
            });
        }
        let mut resources = ReferenceWork {
            elements: logical_elements,
            ..ReferenceWork::default()
        };
        validate_compound_resources(&components, 1, &mut resources)?;
        Ok(Self(Arc::new(TensorData {
            resolved_type,
            shape,
            payload: TensorPayload::Compound(components),
        })))
    }

    /// Creates a rank-zero dense tensor with one exact element.
    ///
    /// # Errors
    ///
    /// Returns a bounded reference-value error.
    pub fn scalar(
        resolved_type: ResolvedValueType,
        value: ReferenceElement,
    ) -> Result<Self, EvaluationError> {
        Self::dense(resolved_type, Shape::new([]), vec![value])
    }

    /// Returns the exact shape-independent semantic value type.
    #[must_use]
    pub fn resolved_type(&self) -> &ResolvedValueType {
        &self.0.resolved_type
    }

    /// Returns the logical shape.
    #[must_use]
    pub fn shape(&self) -> &Shape {
        &self.0.shape
    }

    /// Returns the exact payload representation.
    #[must_use]
    pub fn payload(&self) -> TensorPayloadView<'_> {
        match &self.0.payload {
            TensorPayload::Dense(elements) => TensorPayloadView::Dense(elements),
            TensorPayload::Compound(components) => TensorPayloadView::Compound(components),
        }
    }

    pub(crate) fn storage_id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
}

/// One key-checked entry in the ordered reference-evaluation input interface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputBinding<'a> {
    pub(crate) key: &'a InputKey,
    pub(crate) tensor: &'a Tensor,
}

impl<'a> InputBinding<'a> {
    /// Creates an input binding.
    #[must_use]
    pub const fn new(key: &'a InputKey, tensor: &'a Tensor) -> Self {
        Self { key, tensor }
    }

    /// Returns the stable interface key.
    #[must_use]
    pub const fn key(&self) -> &'a InputKey {
        self.key
    }

    /// Returns the bound reference tensor.
    #[must_use]
    pub const fn tensor(&self) -> &'a Tensor {
        self.tensor
    }
}
