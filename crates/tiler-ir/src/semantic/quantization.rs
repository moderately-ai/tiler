//! Governed compound encoded-value vocabulary and strict affine proof profile.
//!
//! The types here separate four subjects which happen to meet in the first
//! executable proof: logical integer codes, a numerical interpretation,
//! component association, and conversion operations. Physical packing is not
//! part of this module and cannot change the numerical meaning recorded here.

use std::sync::Arc;

use crate::shape::Shape;

use super::{
    AttributeFieldId, CanonicalField, CanonicalValue, EncodedComponentDeclaration,
    EncodedComponentRole, EncodedComponentShape, EncodedNumericContract, F32,
    NormativeDefinitionRef, OpKey, OperationArity, OperationConformance, OperationDefinition,
    OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferenceOutputs,
    OperationInferenceRequest, OperationInferencer, OperationSchema, ParameterIndexMap,
    ProviderDiagnosticCode, QuantSchemeKey, RegistryError, ResolvedValueType,
    SemanticRegistryRegistrar, TypeDefinitionFacts, TypeInstanceError, TypeKey, ValueFact,
    ValueTypeDefinition, ValueTypeDefinitionKey, ValueTypeInstanceValidator, ValueTypeMarker,
};

/// Static-contract field naming the primitive code value type.
pub const ENCODED_NUMERIC_CODE_TYPE: AttributeFieldId = AttributeFieldId::new(1);
/// Static-contract field naming the expressed value type.
pub const ENCODED_NUMERIC_EXPRESSED_TYPE: AttributeFieldId = AttributeFieldId::new(2);
/// Static-contract field carrying the inclusive minimum code.
pub const ENCODED_NUMERIC_CODE_MIN: AttributeFieldId = AttributeFieldId::new(3);
/// Static-contract field carrying the inclusive maximum code.
pub const ENCODED_NUMERIC_CODE_MAX: AttributeFieldId = AttributeFieldId::new(4);
/// Static-contract field naming the conversion computation type.
pub const ENCODED_NUMERIC_COMPUTE_TYPE: AttributeFieldId = AttributeFieldId::new(5);
/// Static-contract field naming the encode rounding rule.
pub const ENCODED_NUMERIC_ROUNDING: AttributeFieldId = AttributeFieldId::new(6);
/// Static-contract field naming ordered overflow behavior.
pub const ENCODED_NUMERIC_SATURATION: AttributeFieldId = AttributeFieldId::new(7);
/// Static-contract field naming exceptional-input behavior.
pub const ENCODED_NUMERIC_NAN_BEHAVIOR: AttributeFieldId = AttributeFieldId::new(8);
/// Static-contract field naming the exact decode evaluation order.
pub const ENCODED_NUMERIC_DECODE_EVALUATION: AttributeFieldId = AttributeFieldId::new(9);
/// Static-contract field naming observable materialization behavior.
pub const ENCODED_NUMERIC_MATERIALIZATION: AttributeFieldId = AttributeFieldId::new(10);

/// Primary integer-code component of a strict affine value.
pub const STRICT_AFFINE_CODES_ROLE: EncodedComponentRole = EncodedComponentRole::new(1);
/// Positive finite scale component of a strict affine value.
pub const STRICT_AFFINE_SCALE_ROLE: EncodedComponentRole = EncodedComponentRole::new(2);
/// In-range integer zero-point component of a strict affine value.
pub const STRICT_AFFINE_ZERO_POINT_ROLE: EncodedComponentRole = EncodedComponentRole::new(3);

/// Governed logical unsigned four-bit integer marker.
pub enum U4 {}

impl ValueTypeMarker for U4 {}

impl U4 {
    /// Returns the governed complete U4 semantic identity.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        nominal_type("u4")
    }
}

/// Governed logical unsigned eight-bit integer marker.
pub enum U8 {}

impl ValueTypeMarker for U8 {}

impl U8 {
    /// Returns the governed complete U8 semantic identity.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        nominal_type("u8")
    }
}

/// Governed per-tensor strict-affine U4-to-F32 encoded value marker.
pub enum StrictAffineU4 {}

impl ValueTypeMarker for StrictAffineU4 {}

impl StrictAffineU4 {
    /// Returns the complete strict-affine U4-to-F32 semantic identity.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        strict_affine_type(U4::resolved_type(), 15)
    }
}

/// Governed per-tensor strict-affine U8-to-F32 encoded value marker.
pub enum StrictAffineU8 {}

impl ValueTypeMarker for StrictAffineU8 {}

impl StrictAffineU8 {
    /// Returns the complete strict-affine U8-to-F32 semantic identity.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        strict_affine_type(U8::resolved_type(), u8::MAX)
    }
}

/// Returns the governed strict-affine scheme-family identity.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its canonical grammar.
#[must_use]
pub fn strict_affine_scheme() -> QuantSchemeKey {
    QuantSchemeKey::new("tiler", "strict-affine", 1)
        .expect("the governed strict-affine scheme key is valid")
}

/// Returns the pure association operation for strict-affine components.
#[must_use]
pub fn assemble_strict_affine_op() -> OpKey {
    governed_op("assemble-strict-affine")
}

/// Returns the strict F32-to-affine conversion operation.
#[must_use]
pub fn quantize_strict_affine_op() -> OpKey {
    governed_op("quantize-strict-affine")
}

/// Returns the strict affine-to-F32 conversion operation.
#[must_use]
pub fn dequantize_strict_affine_op() -> OpKey {
    governed_op("dequantize-strict-affine")
}

pub(super) fn register_standard_quantization(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    register_integer::<U4>(registrar, "u4", 4, 15, U4::resolved_type())?;
    register_integer::<U8>(registrar, "u8", 8, u8::MAX, U8::resolved_type())?;
    registrar.register_value_type(ValueTypeDefinition::new(
        ValueTypeDefinitionKey::EncodedNumeric(strict_affine_scheme()),
        NormativeDefinitionRef::new("Tiler strict affine quantization v1; ADRs 0029-0033")?,
        TypeDefinitionFacts::new(strict_affine_family_facts()),
        Arc::new(StrictAffineTypeValidator),
    ))?;
    registrar.bind_marker::<StrictAffineU4>(StrictAffineU4::resolved_type())?;
    registrar.bind_marker::<StrictAffineU8>(StrictAffineU8::resolved_type())?;
    register_operation(
        registrar,
        &assemble_strict_affine_op(),
        3,
        "component association without numeric conversion",
        "scale-positive-finite; zero-point-in-code-domain; exact component preservation",
        Arc::new(AssembleStrictAffine),
    )?;
    register_operation(
        registrar,
        &quantize_strict_affine_op(),
        3,
        "f32 divide, add zero point, clamp, nearest-even round",
        "nan-reject; infinities-saturate; scale-positive-finite",
        Arc::new(QuantizeStrictAffine),
    )?;
    register_operation(
        registrar,
        &dequantize_strict_affine_op(),
        1,
        "widened code-minus-zero-point then f32 multiply",
        "code-equals-zero-point-produces-positive-zero; preserve-subnormals",
        Arc::new(DequantizeStrictAffine),
    )
}

fn register_integer<T: ValueTypeMarker>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    name: &str,
    width: u32,
    maximum: u8,
    resolved_type: ResolvedValueType,
) -> Result<(), RegistryError> {
    registrar.register_marked_value_type::<T>(
        ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(
                TypeKey::new("tiler", name, 1).expect("governed integer key is valid"),
            ),
            NormativeDefinitionRef::new(format!(
                "Tiler governed unsigned {width}-bit logical integer; ADR 0028"
            ))?,
            TypeDefinitionFacts::new(
                CanonicalValue::record([
                    CanonicalField::new(
                        AttributeFieldId::new(1),
                        CanonicalValue::utf8("unsigned-integer")
                            .expect("governed integer class is bounded"),
                    ),
                    CanonicalField::new(
                        AttributeFieldId::new(2),
                        CanonicalValue::unsigned_u32(width),
                    ),
                    CanonicalField::new(
                        AttributeFieldId::new(3),
                        CanonicalValue::unsigned_u8(maximum),
                    ),
                ])
                .expect("governed integer facts are canonical"),
            ),
        ),
        resolved_type,
    )
}

fn register_operation(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: &OpKey,
    operands: u32,
    semantics: &'static str,
    exceptional_contract: &'static str,
    inferencer: Arc<dyn OperationInferencer>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key.clone(),
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            [],
        )
        .expect("governed strict-affine schema is valid"),
        NormativeDefinitionRef::new(format!("{key}; {semantics}"))?,
        OperationDefinitionFacts::new(
            CanonicalValue::record([
                CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::utf8(semantics).expect("operation fact is bounded"),
                ),
                CanonicalField::new(
                    AttributeFieldId::new(2),
                    CanonicalValue::utf8(exceptional_contract).expect("operation fact is bounded"),
                ),
            ])
            .expect("operation facts are canonical"),
        ),
        OperationConformance::new(
            CanonicalValue::utf8_owned(format!("tiler.conformance.{}", key.name()))
                .expect("conformance identity is bounded"),
        ),
        OperationEffect::Pure,
        inferencer,
    ))
}

fn strict_affine_family_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            AttributeFieldId::new(1),
            CanonicalValue::utf8("compound-encoded-numeric").expect("family fact is bounded"),
        ),
        CanonicalField::new(
            AttributeFieldId::new(2),
            CanonicalValue::utf8("ordered-typed-component-declarations")
                .expect("family fact is bounded"),
        ),
        CanonicalField::new(
            AttributeFieldId::new(3),
            CanonicalValue::utf8("per-tensor-u4-and-u8-proof-profile")
                .expect("family fact is bounded"),
        ),
    ])
    .expect("strict-affine family facts are canonical")
}

fn strict_affine_type(code_type: ResolvedValueType, maximum: u8) -> ResolvedValueType {
    ResolvedValueType::encoded_numeric(
        strict_affine_scheme(),
        EncodedNumericContract::with_components(
            [
                CanonicalField::new(
                    ENCODED_NUMERIC_CODE_TYPE,
                    CanonicalValue::value_type(code_type.clone()),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_EXPRESSED_TYPE,
                    CanonicalValue::value_type(F32::resolved_type()),
                ),
                CanonicalField::new(ENCODED_NUMERIC_CODE_MIN, CanonicalValue::unsigned_u8(0)),
                CanonicalField::new(
                    ENCODED_NUMERIC_CODE_MAX,
                    CanonicalValue::unsigned_u8(maximum),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_COMPUTE_TYPE,
                    CanonicalValue::value_type(F32::resolved_type()),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_ROUNDING,
                    CanonicalValue::utf8("round-to-nearest-ties-even")
                        .expect("rounding contract is bounded"),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_SATURATION,
                    CanonicalValue::utf8("clamp-inclusive-code-domain-before-integer-conversion")
                        .expect("saturation contract is bounded"),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_NAN_BEHAVIOR,
                    CanonicalValue::utf8("reject").expect("NaN contract is bounded"),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_DECODE_EVALUATION,
                    CanonicalValue::utf8(
                        "widen-code-and-zero-point-to-i32; subtract; convert-f32; multiply-scale",
                    )
                    .expect("decode contract is bounded"),
                ),
                CanonicalField::new(
                    ENCODED_NUMERIC_MATERIALIZATION,
                    CanonicalValue::utf8("preserve-exact-codes-and-associated-parameters")
                        .expect("materialization contract is bounded"),
                ),
            ],
            [
                EncodedComponentDeclaration::new(
                    STRICT_AFFINE_CODES_ROLE,
                    code_type.clone(),
                    EncodedComponentShape::LogicalValue,
                ),
                EncodedComponentDeclaration::new(
                    STRICT_AFFINE_SCALE_ROLE,
                    F32::resolved_type(),
                    EncodedComponentShape::ParameterMap(ParameterIndexMap::per_tensor()),
                ),
                EncodedComponentDeclaration::new(
                    STRICT_AFFINE_ZERO_POINT_ROLE,
                    code_type,
                    EncodedComponentShape::ParameterMap(ParameterIndexMap::per_tensor()),
                ),
            ],
        )
        .expect("the governed strict-affine contract is canonical"),
    )
    .expect("the governed strict-affine resolved type is valid")
}

fn nominal_type(name: &str) -> ResolvedValueType {
    ResolvedValueType::nominal(
        TypeKey::new("tiler", name, 1).expect("the governed integer key is valid"),
    )
}

fn governed_op(name: &str) -> OpKey {
    OpKey::new("tiler", name, 1).expect("the governed strict-affine operation key is valid")
}

struct StrictAffineTypeValidator;

impl ValueTypeInstanceValidator for StrictAffineTypeValidator {
    fn validate(&self, value: &ResolvedValueType) -> Result<(), TypeInstanceError> {
        if value == &StrictAffineU4::resolved_type() || value == &StrictAffineU8::resolved_type() {
            return Ok(());
        }
        Err(type_error(
            "strict-affine.unsupported-contract",
            "strict-affine@1 admits only the exact per-tensor u4/f32 and u8/f32 proof contracts",
        ))
    }
}

struct AssembleStrictAffine;

impl OperationInferencer for AssembleStrictAffine {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        reject_attributes(request)?;
        let [codes, scale, zero_point] = request.operands() else {
            return Err(op_error(
                "strict-affine.assemble.arity",
                "AssembleStrictAffine requires codes, scale, and zero point",
            ));
        };
        let result_type = encoded_type_for_code(codes.resolved_type()).ok_or_else(|| {
            op_error(
                "strict-affine.assemble.code-type",
                "codes must use the governed u4 or u8 logical type",
            )
        })?;
        require_scalar_f32(scale, "strict-affine.assemble.scale")?;
        require_scalar_type(
            zero_point,
            codes.resolved_type(),
            "strict-affine.assemble.zero-point",
        )?;
        outputs.try_push(ValueFact::new(result_type, codes.shape().clone()))
    }
}

struct QuantizeStrictAffine;

impl OperationInferencer for QuantizeStrictAffine {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        reject_attributes(request)?;
        let [expressed, scale, zero_point] = request.operands() else {
            return Err(op_error(
                "strict-affine.quantize.arity",
                "QuantizeStrictAffine requires expressed data, scale, and zero point",
            ));
        };
        if expressed.resolved_type() != &F32::resolved_type() {
            return Err(op_error(
                "strict-affine.quantize.expressed-type",
                "strict affine quantization requires f32 expressed data",
            ));
        }
        require_scalar_f32(scale, "strict-affine.quantize.scale")?;
        let result_type = encoded_type_for_code(zero_point.resolved_type()).ok_or_else(|| {
            op_error(
                "strict-affine.quantize.zero-point-type",
                "zero point must use the governed u4 or u8 logical type",
            )
        })?;
        require_scalar_type(
            zero_point,
            zero_point.resolved_type(),
            "strict-affine.quantize.zero-point",
        )?;
        outputs.try_push(ValueFact::new(result_type, expressed.shape().clone()))
    }
}

struct DequantizeStrictAffine;

impl OperationInferencer for DequantizeStrictAffine {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        reject_attributes(request)?;
        let [encoded] = request.operands() else {
            return Err(op_error(
                "strict-affine.dequantize.arity",
                "DequantizeStrictAffine requires one encoded value",
            ));
        };
        if encoded.resolved_type() != &StrictAffineU4::resolved_type()
            && encoded.resolved_type() != &StrictAffineU8::resolved_type()
        {
            return Err(op_error(
                "strict-affine.dequantize.type",
                "value must use an admitted strict-affine proof contract",
            ));
        }
        outputs.try_push(ValueFact::new(
            F32::resolved_type(),
            encoded.shape().clone(),
        ))
    }
}

fn encoded_type_for_code(code_type: &ResolvedValueType) -> Option<ResolvedValueType> {
    if code_type == &U4::resolved_type() {
        Some(StrictAffineU4::resolved_type())
    } else if code_type == &U8::resolved_type() {
        Some(StrictAffineU8::resolved_type())
    } else {
        None
    }
}

fn reject_attributes(
    request: OperationInferenceRequest<'_>,
) -> Result<(), OperationInferenceError> {
    if request.attributes().fields().is_empty() {
        Ok(())
    } else {
        Err(op_error(
            "strict-affine.attributes",
            "strict-affine operations accept no attributes",
        ))
    }
}

fn require_scalar_f32(
    value: &ValueFact,
    code: &'static str,
) -> Result<(), OperationInferenceError> {
    require_scalar_type(value, &F32::resolved_type(), code)
}

fn require_scalar_type(
    value: &ValueFact,
    expected: &ResolvedValueType,
    code: &'static str,
) -> Result<(), OperationInferenceError> {
    if value.resolved_type() != expected {
        return Err(op_error(code, "operand has the wrong resolved value type"));
    }
    if value.shape() != &Shape::new([]) {
        return Err(op_error(code, "parameter operand must be rank-zero"));
    }
    Ok(())
}

fn type_error(code: &'static str, message: &'static str) -> TypeInstanceError {
    TypeInstanceError::new(
        ProviderDiagnosticCode::new(code).expect("diagnostic code is canonical"),
        message,
    )
    .expect("diagnostic message is canonical")
}

fn op_error(code: &'static str, message: &'static str) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("diagnostic code is canonical"),
        message,
    )
    .expect("diagnostic message is canonical")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        F32Constant, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
        RegistryError, SemanticProgramBuilder,
    };

    #[test]
    fn standard_registry_admits_only_the_two_complete_strict_affine_contracts() {
        let registry = FrozenSemanticRegistry::standard().unwrap();
        assert!(registry.contains(&StrictAffineU4::resolved_type()));
        assert!(registry.contains(&StrictAffineU8::resolved_type()));

        let unsupported = ResolvedValueType::encoded_numeric(
            strict_affine_scheme(),
            EncodedNumericContract::new([CanonicalField::new(
                ENCODED_NUMERIC_CODE_TYPE,
                CanonicalValue::value_type(U4::resolved_type()),
            )])
            .unwrap(),
        )
        .unwrap();
        let Err(RegistryError::RejectedTypeInstance(rejection)) =
            registry.validate_type(&unsupported)
        else {
            panic!("incomplete strict-affine contract must be rejected")
        };
        assert_eq!(
            rejection.source_error().code().as_str(),
            "strict-affine.unsupported-contract"
        );

        let unknown_scheme = ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("acme", "codebook", 1).unwrap(),
            EncodedNumericContract::new([CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::boolean(true),
            )])
            .unwrap(),
        )
        .unwrap();
        let Err(RegistryError::UnregisteredTypeAuthority { key }) =
            registry.validate_type(&unknown_scheme)
        else {
            panic!("unknown encoded scheme must be rejected by its exact family key")
        };
        assert_eq!(
            *key,
            ValueTypeDefinitionKey::EncodedNumeric(
                QuantSchemeKey::new("acme", "codebook", 1).unwrap()
            )
        );
    }

    #[test]
    fn component_declarations_reject_duplicate_roles() {
        let duplicate = EncodedNumericContract::with_components(
            [CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::boolean(true),
            )],
            [
                EncodedComponentDeclaration::new(
                    STRICT_AFFINE_CODES_ROLE,
                    U4::resolved_type(),
                    EncodedComponentShape::LogicalValue,
                ),
                EncodedComponentDeclaration::new(
                    STRICT_AFFINE_CODES_ROLE,
                    F32::resolved_type(),
                    EncodedComponentShape::ParameterMap(ParameterIndexMap::per_tensor()),
                ),
            ],
        );
        assert_eq!(
            duplicate,
            Err(
                super::super::TypeIdentityError::DuplicateEncodedComponentRole {
                    role: STRICT_AFFINE_CODES_ROLE
                }
            )
        );
    }

    #[test]
    fn operations_keep_association_conversion_and_materialization_distinct() {
        let registry = FrozenSemanticRegistry::standard().unwrap();
        let tensor = Shape::from_dims([2, 3]);
        let scalar = Shape::new([]);
        let code = ValueFact::new(U4::resolved_type(), tensor.clone());
        let scale = ValueFact::new(F32::resolved_type(), scalar.clone());
        let zero = ValueFact::new(U4::resolved_type(), scalar);
        let expressed = ValueFact::new(F32::resolved_type(), tensor.clone());

        let assembled = registry
            .infer_operation(
                &assemble_strict_affine_op(),
                &[code, scale.clone(), zero.clone()],
                &OperationAttributes::empty(),
            )
            .unwrap();
        let quantized = registry
            .infer_operation(
                &quantize_strict_affine_op(),
                &[expressed, scale, zero],
                &OperationAttributes::empty(),
            )
            .unwrap();
        assert_eq!(assembled, quantized);
        assert_eq!(
            assembled[0].resolved_type(),
            &StrictAffineU4::resolved_type()
        );
        assert_eq!(assembled[0].shape(), &tensor);

        let dequantized = registry
            .infer_operation(
                &dequantize_strict_affine_op(),
                &assembled,
                &OperationAttributes::empty(),
            )
            .unwrap();
        assert_eq!(
            dequantized,
            vec![ValueFact::new(F32::resolved_type(), tensor)]
        );
    }

    #[test]
    fn parameter_shape_check_reaches_its_failure_path() {
        let registry = FrozenSemanticRegistry::standard().unwrap();
        let result = registry.infer_operation(
            &quantize_strict_affine_op(),
            &[
                ValueFact::new(F32::resolved_type(), Shape::from_dims([2])),
                ValueFact::new(F32::resolved_type(), Shape::from_dims([1])),
                ValueFact::new(U4::resolved_type(), Shape::new([])),
            ],
            &OperationAttributes::empty(),
        );
        let Err(RegistryError::RejectedOperationApplication(rejection)) = result else {
            panic!("non-scalar scale must fail")
        };
        assert_eq!(
            rejection.source_error().code().as_str(),
            "strict-affine.quantize.scale"
        );
    }

    #[test]
    fn embedded_scale_bits_reach_quantized_graph_identity() {
        fn graph(scale_bits: u32) -> super::super::SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let expressed = builder
                .input_resolved(
                    InputKey::new("expressed").unwrap(),
                    Shape::from_dims([2]),
                    F32::resolved_type(),
                )
                .unwrap();
            let zero_point = builder
                .input_resolved(
                    InputKey::new("zero-point").unwrap(),
                    Shape::new([]),
                    U4::resolved_type(),
                )
                .unwrap();
            let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
            let encoded = builder
                .apply(
                    quantize_strict_affine_op(),
                    OperationAttributes::empty(),
                    &[expressed, scale.erase(), zero_point],
                )
                .unwrap()[0];
            builder
                .output_resolved(OutputKey::new("encoded").unwrap(), encoded)
                .unwrap();
            builder.build().unwrap()
        }

        let half = graph(0.5_f32.to_bits());
        let quarter = graph(0.25_f32.to_bits());
        assert_ne!(
            half.semantic_identity().graph(),
            quarter.semantic_identity().graph()
        );
    }
}
