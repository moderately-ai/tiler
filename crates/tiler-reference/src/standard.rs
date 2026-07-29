//! The standard reference provider and the `f32` implementations it registers.
//!
//! This is the one provider the crate ships. It is a *consumer* of the
//! registry rather than part of it, so a second provider would be added
//! beside it without touching the registration mechanism.

use std::sync::Arc;

use tiler_ir::semantic::{
    CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE, ProviderIdentity, TypeKey, add_f32_op,
    constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};

use super::error::{ReferenceOperationError, ReferenceRegistryError, ReferenceValueError};
use super::evaluate::{binary, reduction_axes, strict_sum};
use super::quantization::register_standard_quantization;
use super::registry::{
    ReferenceCapabilityRevision, ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs,
    ReferenceRegistryProvider, ReferenceRegistryRegistrar, ReferenceSignature,
    ReferenceValueValidator,
};
use super::tensor::{FloatBitOrder, ReferenceElement, Tensor, TensorPayloadView};

pub(crate) struct StandardReferenceProvider;

impl ReferenceRegistryProvider for StandardReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler", "standard-reference", 4)
            .expect("the governed reference provider identity is valid")
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        let revision = ReferenceCapabilityRevision::new(4)?;
        registrar.register_value_type(
            F32::resolved_type(),
            revision,
            Arc::new(F32ValueValidator),
        )?;
        registrar.register(
            constant_f32_op(),
            ReferenceSignature::new([], [F32::resolved_type()])?,
            revision,
            Arc::new(F32ConstantReference),
        )?;
        let binary_signature = ReferenceSignature::new(
            [F32::resolved_type(), F32::resolved_type()],
            [F32::resolved_type()],
        )?;
        registrar.register(
            multiply_f32_op(),
            binary_signature.clone(),
            revision,
            Arc::new(F32BinaryReference::Multiply),
        )?;
        registrar.register(
            add_f32_op(),
            binary_signature,
            revision,
            Arc::new(F32BinaryReference::Add),
        )?;
        registrar.register(
            strict_serial_sum_f32_op(),
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
            revision,
            Arc::new(StrictSerialF32SumReference),
        )?;
        register_standard_quantization(registrar, revision)
    }
}

struct F32ValueValidator;

impl ReferenceValueValidator for F32ValueValidator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        if tensor.resolved_type() != &F32::resolved_type() {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            return Err(ReferenceValueError::InvalidRepresentation);
        };
        if elements.iter().any(|element| element.as_bytes().len() != 4) {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        Ok(())
    }
}

struct F32ConstantReference;

impl ReferenceOperation for F32ConstantReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if !operands.is_empty() || attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(CanonicalValueView::FloatBits(bits)) = attributes
            .get(F32_CONSTANT_BITS_ATTRIBUTE)
            .map(tiler_ir::semantic::CanonicalValue::view)
        else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if bits.format()
            != &TypeKey::new("tiler", "f32", 1)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?
        {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let element =
            ReferenceElement::from_float_bits(bits.bits(), FloatBitOrder::MostSignificantByteFirst)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let tensor = Tensor::scalar(F32::resolved_type(), element)
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        outputs.push(tensor)
    }
}

enum F32BinaryReference {
    Multiply,
    Add,
}

impl ReferenceOperation for F32BinaryReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let attributes = request.attributes();
        let [left, right] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !attributes.fields().is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let result = match self {
            Self::Multiply => binary(left, right, |left, right| left * right)?,
            Self::Add => binary(left, right, |left, right| left + right)?,
        };
        outputs.push(result)
    }
}

struct StrictSerialF32SumReference;

impl ReferenceOperation for StrictSerialF32SumReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let [input] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let axes = reduction_axes(request.attributes())?;
        outputs.push(strict_sum(input, &axes)?)
    }
}
