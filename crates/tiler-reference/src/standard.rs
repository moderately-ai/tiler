//! The standard reference provider and the `f32` implementations it registers.
//!
//! This is the one provider the crate ships. It is a *consumer* of the
//! registry rather than part of it, so a second provider would be added
//! beside it without touching the registration mechanism.

use std::sync::Arc;

use tiler_ir::semantic::{
    CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE, MAX_CONCATENATE_OPERANDS,
    MIN_CONCATENATE_OPERANDS, ProviderIdentity, TypeKey, add_f32_op, broadcast_f32_op,
    concatenate_f32_op, constant_f32_op, multiply_f32_op, reindex_f32_op, rms_norm_f32_op,
    silu_f32_op, slice_f32_op, softmax_f32_op, strict_serial_sum_f32_op,
    strict_tensor_contraction_f32_op,
};

use super::bf16::register_standard_bf16;
use super::contraction::{ContractionContract, StrictTensorContractionF32Reference};
use super::error::{ReferenceOperationError, ReferenceRegistryError, ReferenceValueError};
use super::evaluate::{binary, reduction_axes, strict_sum};
use super::quantization::register_standard_quantization;
use super::registry::{
    ReferenceCapabilityRevision, ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs,
    ReferenceRegistryProvider, ReferenceRegistryRegistrar, ReferenceSignature,
    ReferenceValueValidator,
};
use super::rms_norm::rms_norm_reference;
use super::silu::silu_reference;
use super::softmax::softmax_reference;
use super::structural::{
    BroadcastF32Reference, ConcatenateF32Reference, ReindexF32Reference, SliceF32Reference,
};
use super::tensor::{FloatBitOrder, ReferenceElement, Tensor, TensorPayloadView};

pub(crate) struct StandardReferenceProvider;

impl ReferenceRegistryProvider for StandardReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler", "standard-reference", 7)
            .expect("the governed reference provider identity is valid")
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        let revision = ReferenceCapabilityRevision::new(7)?;
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
            binary_signature.clone(),
            revision,
            Arc::new(F32BinaryReference::Add),
        )?;
        // The contraction's own numerical signature parameterizes its evaluator,
        // so a declaration this reference cannot compute refuses the registration
        // rather than binding an implementation that would answer for it anyway.
        let contraction = ContractionContract::governed().map_err(|source| {
            ReferenceRegistryError::UnsupportedContraction {
                operation: strict_tensor_contraction_f32_op(),
                source,
            }
        })?;
        registrar.register(
            strict_tensor_contraction_f32_op(),
            binary_signature,
            revision,
            Arc::new(StrictTensorContractionF32Reference::new(contraction)),
        )?;
        let unary_signature =
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?;
        let normalization_signature = ReferenceSignature::new(
            [F32::resolved_type(), F32::resolved_type()],
            [F32::resolved_type()],
        )?;
        registrar.register(
            strict_serial_sum_f32_op(),
            unary_signature.clone(),
            revision,
            Arc::new(StrictSerialF32SumReference),
        )?;
        registrar.register(
            reindex_f32_op(),
            unary_signature.clone(),
            revision,
            Arc::new(ReindexF32Reference),
        )?;
        registrar.register(
            broadcast_f32_op(),
            unary_signature.clone(),
            revision,
            Arc::new(BroadcastF32Reference),
        )?;
        // The third coordinate-mapping family shares the two above's signature
        // exactly: one f32 operand, one f32 result, and everything that
        // distinguishes it carried by its attribute. What separates the three is
        // the class of map the attribute may state — bijective, many-to-one, and
        // injective-not-surjective — which is a semantic admission rather than a
        // signature, so nothing here needs to tell them apart.
        registrar.register(
            slice_f32_op(),
            unary_signature.clone(),
            revision,
            Arc::new(SliceF32Reference),
        )?;
        // One capability per admitted arity, because a capability is keyed by an
        // *exact* resolved signature and the concatenation's operand arity is a
        // bounded range. Enumerating the range here is what makes the semantic
        // schema and this provider agree: an arity the schema admitted and this
        // loop skipped would verify and then fail to evaluate as a missing
        // capability, which is a family admitting an occurrence nothing can answer
        // for. The two bounds are the semantic layer's own, so widening the family
        // widens this loop rather than leaving it behind.
        for arity in MIN_CONCATENATE_OPERANDS..=MAX_CONCATENATE_OPERANDS {
            let operands = (0..arity).map(|_| F32::resolved_type());
            registrar.register(
                concatenate_f32_op(),
                ReferenceSignature::new(operands, [F32::resolved_type()])?,
                revision,
                Arc::new(ConcatenateF32Reference),
            )?;
        }
        // The activation's exponential is the first reference in this crate whose
        // value is not a rational function of its operands, so its implementation
        // certifies the rounding it reports instead of trusting a host library.
        registrar.register(silu_f32_op(), unary_signature, revision, silu_reference())?;
        // The normalization takes its weight as a second operand of the same
        // shape, because the graph admits no implicit broadcasting: the widening
        // from `[N]` to `[T, N]` is a `tiler::broadcast-f32@1` occurrence the
        // program writes, and this signature is what refuses to absorb it.
        registrar.register(
            rms_norm_f32_op(),
            normalization_signature,
            revision,
            rms_norm_reference(),
        )?;
        // The softmax takes one operand and never two: the causal mask is added
        // upstream by a `tiler::add-f32@1` occurrence over a broadcast mask
        // input, so what reaches this signature is already the shifted score
        // tensor. A two-operand signature that absorbed the mask would make the
        // fill value part of this key's identity, which decision D-1 turns on.
        registrar.register(
            softmax_f32_op(),
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
            revision,
            softmax_reference(),
        )?;
        // The second dtype. Its value contract and its three capabilities are
        // registered together and are parameterized by the registered
        // `tiler::bf16@1` descriptor, so a catalog that stopped describing the
        // format refuses this provider rather than binding an evaluator that
        // would answer for a value set nobody declared.
        register_standard_bf16(registrar, revision)?;
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
