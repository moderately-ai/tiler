//! Exact reference semantics for the governed strict-affine proof profile.

use std::sync::Arc;

use tiler_ir::semantic::{
    EncodedComponentDeclaration, F32, OperationAttributes, ResolvedValueType,
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
    StrictAffineU4, StrictAffineU8, U4, U8, assemble_strict_affine_op, dequantize_strict_affine_op,
    quantize_strict_affine_op,
};

use super::error::{ReferenceOperationError, ReferenceRegistryError, ReferenceValueError};
use super::registry::{
    ReferenceCapabilityRevision, ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs,
    ReferenceRegistryRegistrar, ReferenceSignature, ReferenceValueValidator,
};
use super::tensor::{
    FloatBitOrder, ReferenceComponent, ReferenceElement, Tensor, TensorPayloadView,
};

pub(super) fn register_standard_quantization(
    registrar: &mut ReferenceRegistryRegistrar<'_>,
    revision: ReferenceCapabilityRevision,
) -> Result<(), ReferenceRegistryError> {
    for (resolved_type, maximum) in [(U4::resolved_type(), 15_u8), (U8::resolved_type(), u8::MAX)] {
        registrar.register_value_type(
            resolved_type,
            revision,
            Arc::new(UnsignedCodeValidator { maximum }),
        )?;
    }
    for profile in [StrictAffineProfile::u4(), StrictAffineProfile::u8()] {
        registrar.register_value_type(
            profile.encoded_type.clone(),
            revision,
            Arc::new(StrictAffineValueValidator {
                profile: profile.clone(),
            }),
        )?;
        registrar.register(
            assemble_strict_affine_op(),
            ReferenceSignature::new(
                [
                    profile.code_type.clone(),
                    F32::resolved_type(),
                    profile.code_type.clone(),
                ],
                [profile.encoded_type.clone()],
            )?,
            revision,
            Arc::new(StrictAffineOperation::Assemble(profile.clone())),
        )?;
        registrar.register(
            quantize_strict_affine_op(),
            ReferenceSignature::new(
                [
                    F32::resolved_type(),
                    F32::resolved_type(),
                    profile.code_type.clone(),
                ],
                [profile.encoded_type.clone()],
            )?,
            revision,
            Arc::new(StrictAffineOperation::Quantize(profile.clone())),
        )?;
        registrar.register(
            dequantize_strict_affine_op(),
            ReferenceSignature::new([profile.encoded_type.clone()], [F32::resolved_type()])?,
            revision,
            Arc::new(StrictAffineOperation::Dequantize(profile)),
        )?;
    }
    Ok(())
}

#[derive(Clone)]
struct StrictAffineProfile {
    code_type: ResolvedValueType,
    encoded_type: ResolvedValueType,
    maximum: u8,
}

impl StrictAffineProfile {
    fn u4() -> Self {
        Self {
            code_type: U4::resolved_type(),
            encoded_type: StrictAffineU4::resolved_type(),
            maximum: 15,
        }
    }

    fn u8() -> Self {
        Self {
            code_type: U8::resolved_type(),
            encoded_type: StrictAffineU8::resolved_type(),
            maximum: u8::MAX,
        }
    }
}

struct UnsignedCodeValidator {
    maximum: u8,
}

impl ReferenceValueValidator for UnsignedCodeValidator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        validate_unsigned_codes(tensor, self.maximum)
    }
}

struct StrictAffineValueValidator {
    profile: StrictAffineProfile,
}

impl ReferenceValueValidator for StrictAffineValueValidator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        validate_strict_affine(tensor, &self.profile).map(|_| ())
    }
}

enum StrictAffineOperation {
    Assemble(StrictAffineProfile),
    Quantize(StrictAffineProfile),
    Dequantize(StrictAffineProfile),
}

impl ReferenceOperation for StrictAffineOperation {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        reject_attributes(request.attributes())?;
        match self {
            Self::Assemble(profile) => assemble(profile, request.operands(), outputs),
            Self::Quantize(profile) => quantize(profile, request.operands(), outputs),
            Self::Dequantize(profile) => dequantize(profile, request.operands(), outputs),
        }
    }
}

fn assemble(
    profile: &StrictAffineProfile,
    operands: &[&Tensor],
    outputs: &mut ReferenceOutputs,
) -> Result<(), ReferenceOperationError> {
    let [codes, scale, zero_point] = operands else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    validate_unsigned_codes(codes, profile.maximum)
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    read_scale(scale)?;
    read_zero_point(zero_point, profile)?;
    let result = compound_value(profile, codes, scale, zero_point)?;
    outputs.push(result)
}

fn quantize(
    profile: &StrictAffineProfile,
    operands: &[&Tensor],
    outputs: &mut ReferenceOutputs,
) -> Result<(), ReferenceOperationError> {
    let [expressed, scale, zero_point] = operands else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let scale_value = read_scale(scale)?;
    let zero = read_zero_point(zero_point, profile)?;
    let values = dense_f32(expressed)?;
    let mut codes = Vec::with_capacity(values.len());
    for value in values {
        codes.push(
            ReferenceElement::new([quantize_one(value, scale_value, zero, profile.maximum)?])
                .map_err(|_| ReferenceOperationError::InvalidApplication)?,
        );
    }
    let codes = Tensor::dense(profile.code_type.clone(), expressed.shape().clone(), codes)
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    outputs.push(compound_value(profile, &codes, scale, zero_point)?)
}

fn dequantize(
    profile: &StrictAffineProfile,
    operands: &[&Tensor],
    outputs: &mut ReferenceOutputs,
) -> Result<(), ReferenceOperationError> {
    let [encoded] = operands else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let components = validate_strict_affine(encoded, profile)
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    let scale = read_scale(components[1].tensor())?;
    let zero = read_zero_point(components[2].tensor(), profile)?;
    let codes = dense_code_bytes(components[0].tensor(), profile.maximum)
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    let elements = codes
        .iter()
        .map(|code| {
            let value = dequantize_one(*code, scale, zero);
            ReferenceElement::from_float_bits(
                value.to_bits().to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .map_err(|_| ReferenceOperationError::InvalidApplication)
        })
        .collect::<Result<Vec<_>, _>>()?;
    outputs.push(
        Tensor::dense(F32::resolved_type(), encoded.shape().clone(), elements)
            .map_err(|_| ReferenceOperationError::InvalidApplication)?,
    )
}

fn compound_value(
    profile: &StrictAffineProfile,
    codes: &Tensor,
    scale: &Tensor,
    zero_point: &Tensor,
) -> Result<Tensor, ReferenceOperationError> {
    Tensor::compound(
        profile.encoded_type.clone(),
        codes.shape().clone(),
        vec![
            ReferenceComponent::new(STRICT_AFFINE_CODES_ROLE, codes.clone()),
            ReferenceComponent::new(STRICT_AFFINE_SCALE_ROLE, scale.clone()),
            ReferenceComponent::new(STRICT_AFFINE_ZERO_POINT_ROLE, zero_point.clone()),
        ],
    )
    .map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn validate_strict_affine<'a>(
    tensor: &'a Tensor,
    profile: &StrictAffineProfile,
) -> Result<&'a [ReferenceComponent], ReferenceValueError> {
    if tensor.resolved_type() != &profile.encoded_type {
        return Err(ReferenceValueError::InvalidRepresentation);
    }
    let TensorPayloadView::Compound(components) = tensor.payload() else {
        return Err(ReferenceValueError::InvalidRepresentation);
    };
    let (_, contract) = tensor
        .resolved_type()
        .encoded_numeric_parts()
        .ok_or(ReferenceValueError::InvalidRepresentation)?;
    validate_component_structure(tensor, components, contract.components())?;
    validate_unsigned_codes(components[0].tensor(), profile.maximum)?;
    read_scale_value(components[1].tensor())?;
    read_zero_point_value(components[2].tensor(), profile)?;
    Ok(components)
}

fn validate_component_structure(
    outer: &Tensor,
    actual: &[ReferenceComponent],
    expected: &[EncodedComponentDeclaration],
) -> Result<(), ReferenceValueError> {
    if actual.len() != expected.len() {
        return Err(ReferenceValueError::InvalidRepresentation);
    }
    for (actual, expected) in actual.iter().zip(expected) {
        if actual.role() != expected.role()
            || actual.tensor().resolved_type() != expected.resolved_type()
            || actual.tensor().shape() != &expected.shape_relation().component_shape(outer.shape())
        {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
    }
    Ok(())
}

fn validate_unsigned_codes(tensor: &Tensor, maximum: u8) -> Result<(), ReferenceValueError> {
    let expected = if maximum == 15 {
        U4::resolved_type()
    } else {
        U8::resolved_type()
    };
    if tensor.resolved_type() != &expected {
        return Err(ReferenceValueError::InvalidRepresentation);
    }
    let _ = dense_code_bytes(tensor, maximum)?;
    Ok(())
}

fn dense_code_bytes(tensor: &Tensor, maximum: u8) -> Result<Vec<u8>, ReferenceValueError> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        return Err(ReferenceValueError::InvalidRepresentation);
    };
    elements
        .iter()
        .map(|element| {
            let [value] = element.as_bytes() else {
                return Err(ReferenceValueError::InvalidRepresentation);
            };
            if *value > maximum {
                return Err(ReferenceValueError::InvalidRepresentation);
            }
            Ok(*value)
        })
        .collect()
}

fn dense_f32(tensor: &Tensor) -> Result<Vec<f32>, ReferenceOperationError> {
    if tensor.resolved_type() != &F32::resolved_type() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    elements
        .iter()
        .map(|element| {
            let bytes: [u8; 4] = element
                .as_bytes()
                .try_into()
                .map_err(|_| ReferenceOperationError::InvalidApplication)?;
            Ok(f32::from_bits(u32::from_be_bytes(bytes)))
        })
        .collect()
}

fn read_scale(tensor: &Tensor) -> Result<f32, ReferenceOperationError> {
    read_scale_value(tensor).map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn read_scale_value(tensor: &Tensor) -> Result<f32, ReferenceValueError> {
    if tensor.resolved_type() != &F32::resolved_type() || tensor.shape().rank() != 0 {
        return Err(ReferenceValueError::InvalidRepresentation);
    }
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        return Err(ReferenceValueError::InvalidRepresentation);
    };
    let [element] = elements else {
        return Err(ReferenceValueError::InvalidRepresentation);
    };
    let bytes: [u8; 4] = element
        .as_bytes()
        .try_into()
        .map_err(|_| ReferenceValueError::InvalidRepresentation)?;
    let value = f32::from_bits(u32::from_be_bytes(bytes));
    if !value.is_finite() || value <= 0.0 {
        return Err(ReferenceValueError::InvalidRepresentation);
    }
    Ok(value)
}

fn read_zero_point(
    tensor: &Tensor,
    profile: &StrictAffineProfile,
) -> Result<u8, ReferenceOperationError> {
    read_zero_point_value(tensor, profile).map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn read_zero_point_value(
    tensor: &Tensor,
    profile: &StrictAffineProfile,
) -> Result<u8, ReferenceValueError> {
    if tensor.resolved_type() != &profile.code_type || tensor.shape().rank() != 0 {
        return Err(ReferenceValueError::InvalidRepresentation);
    }
    let values = dense_code_bytes(tensor, profile.maximum)?;
    let [value] = values.as_slice() else {
        return Err(ReferenceValueError::InvalidRepresentation);
    };
    Ok(*value)
}

fn quantize_one(
    value: f32,
    scale: f32,
    zero_point: u8,
    maximum: u8,
) -> Result<u8, ReferenceOperationError> {
    if value.is_nan() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let scaled = value / scale;
    let shifted = scaled + f32::from(zero_point);
    let clamped = shifted.clamp(0.0, f32::from(maximum));
    let rounded = clamped.round_ties_even();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamp and nearest-even establish an integral value in the inclusive u8 code domain"
    )]
    let code = rounded as u8;
    Ok(code)
}

fn dequantize_one(code: u8, scale: f32, zero_point: u8) -> f32 {
    if code == zero_point {
        return 0.0;
    }
    let difference = i32::from(code) - i32::from(zero_point);
    #[allow(
        clippy::cast_precision_loss,
        reason = "the difference of two u8 codes is in -255..=255 and is represented exactly by f32"
    )]
    let difference = difference as f32;
    difference * scale
}

fn reject_attributes(attributes: &OperationAttributes) -> Result<(), ReferenceOperationError> {
    if attributes.fields().is_empty() {
        Ok(())
    } else {
        Err(ReferenceOperationError::InvalidApplication)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvaluationError, FrozenReferenceRegistry, InputBinding, ReferenceEvaluator};
    use tiler_ir::semantic::{
        CanonicalField, CanonicalValue, EncodedNumericContract, InputKey, OperationAttributes,
        OutputKey, QuantSchemeKey, SemanticProgramBuilder,
    };
    use tiler_ir::shape::Shape;

    fn element(bytes: impl AsRef<[u8]>) -> ReferenceElement {
        ReferenceElement::new(bytes).unwrap()
    }

    fn f32_scalar(value: f32) -> Tensor {
        Tensor::scalar(F32::resolved_type(), element(value.to_bits().to_be_bytes())).unwrap()
    }

    fn code_tensor(values: &[u8]) -> Tensor {
        Tensor::dense(
            U4::resolved_type(),
            Shape::from_dims([u64::try_from(values.len()).unwrap()]),
            values.iter().map(|value| element([*value])).collect(),
        )
        .unwrap()
    }

    fn zero(value: u8) -> Tensor {
        Tensor::scalar(U4::resolved_type(), element([value])).unwrap()
    }

    #[test]
    fn component_declarations_derive_all_shapes_without_scheme_hard_coding() {
        let logical = Shape::from_dims([2, 3]);
        let resolved_type = StrictAffineU4::resolved_type();
        let (_, contract) = resolved_type.encoded_numeric_parts().unwrap();
        let shapes: Vec<_> = contract
            .components()
            .iter()
            .map(|component| component.shape_relation().component_shape(&logical))
            .collect();
        assert_eq!(shapes, vec![logical, Shape::new([]), Shape::new([])]);
    }

    fn compound_fixture() -> (StrictAffineProfile, Tensor, Tensor, Tensor) {
        let profile = StrictAffineProfile::u4();
        let codes = code_tensor(&[7, 8]);
        let scale = f32_scalar(0.5);
        let zero = zero(8);
        (profile, codes, scale, zero)
    }

    #[test]
    fn compound_validator_rejects_wrong_role_order() {
        let (profile, codes, scale, zero) = compound_fixture();
        let wrong_role = Tensor::compound(
            profile.encoded_type.clone(),
            codes.shape().clone(),
            vec![
                ReferenceComponent::new(
                    tiler_ir::semantic::EncodedComponentRole::new(99),
                    codes.clone(),
                ),
                ReferenceComponent::new(STRICT_AFFINE_SCALE_ROLE, scale.clone()),
                ReferenceComponent::new(STRICT_AFFINE_ZERO_POINT_ROLE, zero.clone()),
            ],
        )
        .unwrap();
        assert_eq!(
            validate_strict_affine(&wrong_role, &profile),
            Err(ReferenceValueError::InvalidRepresentation)
        );
    }

    #[test]
    fn compound_validator_rejects_missing_component() {
        let (profile, codes, scale, _) = compound_fixture();
        let missing = Tensor::compound(
            profile.encoded_type.clone(),
            codes.shape().clone(),
            vec![
                ReferenceComponent::new(STRICT_AFFINE_CODES_ROLE, codes.clone()),
                ReferenceComponent::new(STRICT_AFFINE_SCALE_ROLE, scale.clone()),
            ],
        )
        .unwrap();
        assert_eq!(
            validate_strict_affine(&missing, &profile),
            Err(ReferenceValueError::InvalidRepresentation)
        );
    }

    #[test]
    fn compound_validator_rejects_wrong_logical_component_shape() {
        let (profile, codes, scale, zero) = compound_fixture();
        let wrong_logical_shape = Tensor::compound(
            profile.encoded_type.clone(),
            Shape::from_dims([3]),
            vec![
                ReferenceComponent::new(STRICT_AFFINE_CODES_ROLE, codes.clone()),
                ReferenceComponent::new(STRICT_AFFINE_SCALE_ROLE, scale.clone()),
                ReferenceComponent::new(STRICT_AFFINE_ZERO_POINT_ROLE, zero.clone()),
            ],
        )
        .unwrap();
        assert_eq!(
            validate_strict_affine(&wrong_logical_shape, &profile),
            Err(ReferenceValueError::InvalidRepresentation)
        );
    }

    #[test]
    fn unsigned_code_validator_rejects_out_of_domain_element() {
        let invalid_code = code_tensor(&[16]);
        assert_eq!(
            validate_unsigned_codes(&invalid_code, 15),
            Err(ReferenceValueError::InvalidRepresentation)
        );
    }

    #[test]
    fn compound_validator_rejects_every_invalid_scale_class() {
        let (profile, codes, _, zero) = compound_fixture();
        for invalid_scale in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            let value =
                compound_value(&profile, &codes, &f32_scalar(invalid_scale), &zero).unwrap();
            assert_eq!(
                validate_strict_affine(&value, &profile),
                Err(ReferenceValueError::InvalidRepresentation)
            );
        }
    }

    #[test]
    fn strict_quantize_pins_nearest_even_and_infinity_saturation() {
        assert_eq!(quantize_one(f32::NEG_INFINITY, 0.5, 8, 15).unwrap(), 0);
        assert_eq!(quantize_one(0.25, 0.5, 8, 15).unwrap(), 8);
        assert_eq!(quantize_one(0.75, 0.5, 8, 15).unwrap(), 10);
        assert_eq!(quantize_one(f32::INFINITY, 0.5, 8, 15).unwrap(), 15);
    }

    #[test]
    fn strict_quantize_rejects_nan() {
        assert_eq!(
            quantize_one(f32::NAN, 0.5, 8, 15),
            Err(ReferenceOperationError::InvalidApplication)
        );
    }

    #[test]
    fn strict_dequantize_pins_widened_subtraction_and_positive_zero() {
        assert_eq!(dequantize_one(7, 0.5, 8).to_bits(), (-0.5_f32).to_bits());
        assert_eq!(dequantize_one(8, 0.5, 8).to_bits(), 0.0_f32.to_bits());
        assert_eq!(dequantize_one(0, 0.5, 15).to_bits(), (-7.5_f32).to_bits());
        assert_eq!(
            dequantize_one(1, f32::from_bits(1), 0).to_bits(),
            1,
            "the exact f32 evaluation preserves a representable subnormal"
        );
    }

    #[test]
    fn strict_u8_profile_uses_the_full_byte_code_domain() {
        let profile = StrictAffineProfile::u8();
        let codes = Tensor::dense(
            U8::resolved_type(),
            Shape::from_dims([3]),
            [0_u8, 128, u8::MAX]
                .into_iter()
                .map(|value| element([value]))
                .collect(),
        )
        .unwrap();
        let scale = f32_scalar(0.5);
        let zero = Tensor::scalar(U8::resolved_type(), element([128])).unwrap();
        let encoded = compound_value(&profile, &codes, &scale, &zero).unwrap();
        assert!(validate_strict_affine(&encoded, &profile).is_ok());
        assert_eq!(quantize_one(f32::INFINITY, 0.5, 128, u8::MAX), Ok(u8::MAX));
        assert_eq!(
            dequantize_one(u8::MAX, 0.5, 128).to_bits(),
            63.5_f32.to_bits()
        );
    }

    #[test]
    fn reference_evaluator_runs_quantize_and_dequantize_over_one_compound_value() {
        let x_key = InputKey::new("x").unwrap();
        let scale_key = InputKey::new("scale").unwrap();
        let zero_key = InputKey::new("zero").unwrap();
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let x = graph
            .input_resolved(x_key.clone(), Shape::from_dims([5]), F32::resolved_type())
            .unwrap();
        let scale = graph
            .input_resolved(scale_key.clone(), Shape::new([]), F32::resolved_type())
            .unwrap();
        let zero_value = graph
            .input_resolved(zero_key.clone(), Shape::new([]), U4::resolved_type())
            .unwrap();
        let quantized = graph
            .apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[x, scale, zero_value],
            )
            .unwrap()[0];
        let dequantized = graph
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[quantized],
            )
            .unwrap()[0];
        graph
            .output_resolved(OutputKey::new("quantized").unwrap(), quantized)
            .unwrap();
        graph
            .output_resolved(OutputKey::new("dequantized").unwrap(), dequantized)
            .unwrap();
        let program = graph.build().unwrap();

        let x = Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([5]),
            [
                f32::NEG_INFINITY,
                -0.0_f32,
                0.25_f32,
                0.75_f32,
                f32::INFINITY,
            ]
            .into_iter()
            .map(|value| element(value.to_bits().to_be_bytes()))
            .collect(),
        )
        .unwrap();
        let scale = f32_scalar(0.5);
        let zero = zero(8);
        let outputs = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(
                &program,
                &[
                    InputBinding::new(&x_key, &x),
                    InputBinding::new(&scale_key, &scale),
                    InputBinding::new(&zero_key, &zero),
                ],
            )
            .unwrap();

        let TensorPayloadView::Compound(components) = outputs[0].payload() else {
            panic!("quantized output must remain one compound value")
        };
        assert_eq!(
            dense_code_bytes(components[0].tensor(), 15).unwrap(),
            [0, 8, 8, 10, 15]
        );
        assert_eq!(components[0].role(), STRICT_AFFINE_CODES_ROLE);
        assert_eq!(components[1].role(), STRICT_AFFINE_SCALE_ROLE);
        assert_eq!(components[2].role(), STRICT_AFFINE_ZERO_POINT_ROLE);
        assert_eq!(
            dense_f32(components[1].tensor())
                .unwrap()
                .into_iter()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            [0.5_f32.to_bits()]
        );
        assert_eq!(dense_code_bytes(components[2].tensor(), 15).unwrap(), [8]);
        assert_eq!(
            dense_f32(&outputs[1])
                .unwrap()
                .into_iter()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            [-4.0_f32, 0.0_f32, 0.0_f32, 1.0_f32, 3.5_f32].map(f32::to_bits)
        );
    }

    #[test]
    fn runtime_scale_payload_changes_results_without_changing_program_identity() {
        let x_key = InputKey::new("x").unwrap();
        let scale_key = InputKey::new("scale").unwrap();
        let zero_key = InputKey::new("zero").unwrap();
        let mut graph = SemanticProgramBuilder::try_standard().unwrap();
        let x = graph
            .input_resolved(x_key.clone(), Shape::from_dims([1]), F32::resolved_type())
            .unwrap();
        let scale = graph
            .input_resolved(scale_key.clone(), Shape::new([]), F32::resolved_type())
            .unwrap();
        let zero_value = graph
            .input_resolved(zero_key.clone(), Shape::new([]), U4::resolved_type())
            .unwrap();
        let quantized = graph
            .apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[x, scale, zero_value],
            )
            .unwrap()[0];
        let dequantized = graph
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[quantized],
            )
            .unwrap()[0];
        graph
            .output_resolved(OutputKey::new("result").unwrap(), dequantized)
            .unwrap();
        let program = graph.build().unwrap();
        let identity = program.semantic_identity().graph().clone();
        let x = Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([1]),
            vec![element(0.75_f32.to_bits().to_be_bytes())],
        )
        .unwrap();
        let zero = zero(8);
        let evaluator = ReferenceEvaluator::standard().unwrap();
        let evaluate = |scale: Tensor| {
            evaluator
                .evaluate(
                    &program,
                    &[
                        InputBinding::new(&x_key, &x),
                        InputBinding::new(&scale_key, &scale),
                        InputBinding::new(&zero_key, &zero),
                    ],
                )
                .unwrap()
        };
        let half = evaluate(f32_scalar(0.5));
        let quarter = evaluate(f32_scalar(0.25));

        assert_eq!(program.semantic_identity().graph(), &identity);
        assert_eq!(dense_f32(&half[0]).unwrap()[0].to_bits(), 1.0_f32.to_bits());
        assert_eq!(
            dense_f32(&quarter[0]).unwrap()[0].to_bits(),
            0.75_f32.to_bits()
        );
    }

    #[test]
    fn unsupported_scheme_is_named_by_the_typed_missing_capability() {
        let registry = FrozenReferenceRegistry::standard().unwrap();
        let unsupported_type = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("acme", "codebook", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                tiler_ir::semantic::AttributeFieldId::new(1),
                CanonicalValue::boolean(true),
            )])
            .unwrap(),
        )
        .unwrap();
        let tensor = Tensor::compound(unsupported_type.clone(), Shape::new([]), vec![]).unwrap();
        let error = registry
            .validate_value(&tensor, registry.semantic_registry())
            .unwrap_err();
        let EvaluationError::MissingValueCapability { resolved_type } = error else {
            panic!("unsupported encoded scheme must fail as a missing exact capability")
        };
        assert_eq!(*resolved_type, unsupported_type);
    }
}
