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
    OperationInferenceRequest, OperationInferencer, OperationOperandIndex, OperationSchema,
    ParameterIndexMap, ProviderDiagnosticCode, QuantSchemeKey, RegistryError, ResolvedValueType,
    SemanticInvalidInputCode, SemanticLogicalView, SemanticPreconditionDeclaration,
    SemanticPreconditionDeclarations, SemanticRegistryRegistrar, TypeDefinitionFacts,
    TypeInstanceError, TypeKey, ValueFact, ValueTypeDefinition, ValueTypeDefinitionKey,
    ValueTypeInstanceValidator, ValueTypeMarker, no_nan_predicate,
    positive_finite_scalar_predicate,
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
    // The U4 and U8 nominal identities are governed catalog rows registered by
    // `catalog::register_builtin_dtype_catalog`; this module binds the Rust
    // markers that the strict-affine authoring path resolves through, and owns
    // the strict-affine scheme itself.
    registrar.bind_marker::<U4>(U4::resolved_type())?;
    registrar.bind_marker::<U8>(U8::resolved_type())?;
    registrar.register_value_type(ValueTypeDefinition::new(
        ValueTypeDefinitionKey::EncodedNumeric(strict_affine_scheme()),
        NormativeDefinitionRef::new(
            "Tiler strict affine quantization v1; ADRs 0029-0033; tiler::strict-affine@1",
        )?,
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
        "zero-point-in-code-domain; exact-component-preservation",
        SemanticPreconditionDeclarations::empty(),
        Arc::new(AssembleStrictAffine),
    )?;
    register_operation(
        registrar,
        &quantize_strict_affine_op(),
        3,
        "f32 divide, add zero point, clamp, nearest-even round",
        "infinities-saturate",
        quantize_preconditions(),
        Arc::new(QuantizeStrictAffine),
    )?;
    register_operation(
        registrar,
        &dequantize_strict_affine_op(),
        1,
        "widened code-minus-zero-point then f32 multiply",
        "code-equals-zero-point-produces-positive-zero; preserve-subnormals",
        SemanticPreconditionDeclarations::empty(),
        Arc::new(DequantizeStrictAffine),
    )
}

fn register_operation(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: &OpKey,
    operands: u32,
    semantics: &'static str,
    exceptional_contract: &'static str,
    semantic_preconditions: SemanticPreconditionDeclarations,
    inferencer: Arc<dyn OperationInferencer>,
) -> Result<(), RegistryError> {
    registrar.register_operation(
        OperationDefinition::new(
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
                        CanonicalValue::utf8(exceptional_contract)
                            .expect("operation fact is bounded"),
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
        )
        .with_semantic_preconditions(semantic_preconditions)
        .expect("governed strict-affine precondition subjects fit the schema"),
    )
}

fn quantize_preconditions() -> SemanticPreconditionDeclarations {
    SemanticPreconditionDeclarations::new([
        SemanticPreconditionDeclaration::new(
            no_nan_predicate(),
            OperationOperandIndex::new(0),
            SemanticLogicalView::WholeValue,
            SemanticInvalidInputCode::new("tiler", "strict-affine-quantize-nan", 1)
                .expect("governed invalid-input code is valid"),
        ),
        SemanticPreconditionDeclaration::new(
            positive_finite_scalar_predicate(),
            OperationOperandIndex::new(1),
            SemanticLogicalView::WholeValue,
            SemanticInvalidInputCode::new(
                "tiler",
                "strict-affine-quantize-scale-not-positive-finite",
                1,
            )
            .expect("governed invalid-input code is valid"),
        ),
    ])
    .expect("governed strict-affine preconditions are bounded and distinct")
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
        BuildError, F32Constant, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
        RegistryError, SemanticPreconditionStatus, SemanticProgram, SemanticProgramBuilder,
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
    fn strict_quantize_declares_both_semantic_value_preconditions_exactly() {
        let registry = FrozenSemanticRegistry::standard().unwrap();
        let definition = registry
            .operation_definition(&quantize_strict_affine_op())
            .unwrap();
        let declarations = definition.semantic_preconditions().as_slice();
        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].predicate(), &no_nan_predicate());
        assert_eq!(declarations[0].operand(), OperationOperandIndex::new(0));
        assert_eq!(declarations[0].view(), SemanticLogicalView::WholeValue);
        assert_eq!(
            declarations[0].invalid_input_code(),
            &SemanticInvalidInputCode::new("tiler", "strict-affine-quantize-nan", 1).unwrap()
        );
        assert_eq!(
            declarations[1].predicate(),
            &positive_finite_scalar_predicate()
        );
        assert_eq!(declarations[1].operand(), OperationOperandIndex::new(1));
        assert_eq!(declarations[1].view(), SemanticLogicalView::WholeValue);
        assert_eq!(
            declarations[1].invalid_input_code(),
            &SemanticInvalidInputCode::new(
                "tiler",
                "strict-affine-quantize-scale-not-positive-finite",
                1,
            )
            .unwrap()
        );
    }

    fn runtime_quantize_program_with_zero_type(zero_type: ResolvedValueType) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let expressed = builder
            .input_resolved(
                InputKey::new("expressed").unwrap(),
                Shape::from_dims([2]),
                F32::resolved_type(),
            )
            .unwrap();
        let scale = builder
            .input_resolved(
                InputKey::new("scale").unwrap(),
                Shape::new([]),
                F32::resolved_type(),
            )
            .unwrap();
        let zero = builder
            .input_resolved(InputKey::new("zero").unwrap(), Shape::new([]), zero_type)
            .unwrap();
        let result = builder
            .apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[expressed, scale, zero],
            )
            .unwrap()[0];
        builder
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn runtime_unknown_u4_and_u8_quantize_inputs_retain_two_ordered_residuals() {
        for (zero_type, expected_result_type) in [
            (U4::resolved_type(), StrictAffineU4::resolved_type()),
            (U8::resolved_type(), StrictAffineU8::resolved_type()),
        ] {
            let program = runtime_quantize_program_with_zero_type(zero_type);
            let operation = program
                .operations()
                .find(|operation| operation.key() == &quantize_strict_affine_op())
                .unwrap();
            let result = program.value(operation.results().next().unwrap()).unwrap();
            assert_eq!(result.resolved_type(), &expected_result_type);
            let preconditions: Vec<_> = operation.semantic_preconditions().collect();
            assert_eq!(preconditions.len(), 2);
            assert_eq!(preconditions[0].declaration_ordinal().get(), 0);
            assert_eq!(preconditions[0].predicate(), &no_nan_predicate());
            assert_eq!(
                preconditions[0].status(),
                SemanticPreconditionStatus::Residual
            );
            assert!(preconditions[0].obligation_identity().is_some());
            assert_eq!(preconditions[1].declaration_ordinal().get(), 1);
            assert_eq!(
                preconditions[1].predicate(),
                &positive_finite_scalar_predicate()
            );
            assert_eq!(
                preconditions[1].status(),
                SemanticPreconditionStatus::Residual
            );
            assert!(preconditions[1].obligation_identity().is_some());
            assert!(std::ptr::eq(
                preconditions[0].obligation_identity().unwrap(),
                preconditions[0].obligation_identity().unwrap(),
            ));
        }
    }

    fn constant_quantize_program(
        expressed_bits: u32,
        scale_bits: u32,
    ) -> Result<SemanticProgram, BuildError> {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let expressed = F32Constant::apply(&mut builder, expressed_bits)?;
        let scale = F32Constant::apply(&mut builder, scale_bits)?;
        let zero = builder.input_resolved(
            InputKey::new("zero").unwrap(),
            Shape::new([]),
            U4::resolved_type(),
        )?;
        let result = builder.apply(
            quantize_strict_affine_op(),
            OperationAttributes::empty(),
            &[expressed.erase(), scale.erase(), zero],
        )?[0];
        builder.output_resolved(OutputKey::new("result").unwrap(), result)?;
        Ok(builder.build().unwrap())
    }

    #[test]
    fn exact_governed_constants_prove_each_predicate_without_an_obligation() {
        for expressed in [0.0_f32, -0.0_f32, 1.0_f32, f32::NEG_INFINITY, f32::INFINITY] {
            let program =
                constant_quantize_program(expressed.to_bits(), f32::from_bits(1).to_bits())
                    .unwrap();
            let quantize = program
                .operations()
                .find(|operation| operation.key() == &quantize_strict_affine_op())
                .unwrap();
            for precondition in quantize.semantic_preconditions() {
                assert_eq!(precondition.status(), SemanticPreconditionStatus::Proven);
                assert_eq!(
                    precondition.proof_basis(),
                    Some(super::super::SemanticPreconditionProofBasis::StandardConstantF32BitsV1)
                );
                assert!(precondition.obligation_identity().is_none());
            }
        }
        for scale in [f32::from_bits(1), 0.5_f32, f32::MAX] {
            let program = constant_quantize_program(1.0_f32.to_bits(), scale.to_bits()).unwrap();
            let quantize = program
                .operations()
                .find(|operation| operation.key() == &quantize_strict_affine_op())
                .unwrap();
            assert!(quantize.semantic_preconditions().all(|precondition| {
                precondition.status() == SemanticPreconditionStatus::Proven
                    && precondition.obligation_identity().is_none()
            }));
        }
    }

    #[test]
    fn each_exact_constant_removes_only_its_own_residual() {
        fn build(constant_expressed: bool) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let expressed = if constant_expressed {
                F32Constant::apply(&mut builder, 1.0_f32.to_bits())
                    .unwrap()
                    .erase()
            } else {
                builder
                    .input_resolved(
                        InputKey::new("expressed").unwrap(),
                        Shape::new([]),
                        F32::resolved_type(),
                    )
                    .unwrap()
            };
            let scale = if constant_expressed {
                builder
                    .input_resolved(
                        InputKey::new("scale").unwrap(),
                        Shape::new([]),
                        F32::resolved_type(),
                    )
                    .unwrap()
            } else {
                F32Constant::apply(&mut builder, 0.5_f32.to_bits())
                    .unwrap()
                    .erase()
            };
            let zero = builder
                .input_resolved(
                    InputKey::new("zero").unwrap(),
                    Shape::new([]),
                    U4::resolved_type(),
                )
                .unwrap();
            let result = builder
                .apply(
                    quantize_strict_affine_op(),
                    OperationAttributes::empty(),
                    &[expressed, scale, zero],
                )
                .unwrap()[0];
            builder
                .output_resolved(OutputKey::new("result").unwrap(), result)
                .unwrap();
            builder.build().unwrap()
        }

        let expressed_proven: Vec<_> = build(true)
            .operations()
            .find(|operation| operation.key() == &quantize_strict_affine_op())
            .unwrap()
            .semantic_preconditions()
            .map(|precondition| (precondition.predicate().clone(), precondition.status()))
            .collect();
        assert_eq!(
            expressed_proven,
            vec![
                (no_nan_predicate(), SemanticPreconditionStatus::Proven),
                (
                    positive_finite_scalar_predicate(),
                    SemanticPreconditionStatus::Residual,
                ),
            ]
        );

        let scale_proven: Vec<_> = build(false)
            .operations()
            .find(|operation| operation.key() == &quantize_strict_affine_op())
            .unwrap()
            .semantic_preconditions()
            .map(|precondition| (precondition.predicate().clone(), precondition.status()))
            .collect();
        assert_eq!(
            scale_proven,
            vec![
                (no_nan_predicate(), SemanticPreconditionStatus::Residual),
                (
                    positive_finite_scalar_predicate(),
                    SemanticPreconditionStatus::Proven,
                ),
            ]
        );
    }

    #[test]
    fn exact_nan_and_invalid_scale_constants_disprove_transactionally() {
        for nan_bits in [0x7fc0_0000_u32, 0x7f80_0001_u32] {
            let error = constant_quantize_program(nan_bits, 0.5_f32.to_bits()).unwrap_err();
            let BuildError::SemanticPreconditionDisproved(disproof) = error else {
                panic!("NaN must be a typed semantic precondition disproof")
            };
            assert_eq!(disproof.predicate(), &no_nan_predicate());
            assert_eq!(
                disproof.invalid_input_code(),
                &SemanticInvalidInputCode::new("tiler", "strict-affine-quantize-nan", 1).unwrap()
            );
            assert_eq!(disproof.declaration_ordinal().get(), 0);
        }

        for scale_bits in [
            0.0_f32.to_bits(),
            (-0.0_f32).to_bits(),
            (-1.0_f32).to_bits(),
            0x8000_0001,
            f32::INFINITY.to_bits(),
            f32::NEG_INFINITY.to_bits(),
            0x7fc0_0000,
            0x7f80_0001,
        ] {
            let error = constant_quantize_program(1.0_f32.to_bits(), scale_bits).unwrap_err();
            let BuildError::SemanticPreconditionDisproved(disproof) = error else {
                panic!("invalid scale must be a typed semantic precondition disproof")
            };
            assert_eq!(disproof.predicate(), &positive_finite_scalar_predicate());
            assert_eq!(
                disproof.invalid_input_code(),
                &SemanticInvalidInputCode::new(
                    "tiler",
                    "strict-affine-quantize-scale-not-positive-finite",
                    1,
                )
                .unwrap()
            );
            assert_eq!(disproof.declaration_ordinal().get(), 1);
        }
    }

    #[test]
    fn simultaneous_static_failures_use_stable_code_then_ordinal_priority() {
        let error = constant_quantize_program(0x7fc0_0000, 0.0_f32.to_bits()).unwrap_err();
        let BuildError::SemanticPreconditionDisproved(disproof) = error else {
            panic!("simultaneous invalid inputs must retain typed disproof")
        };
        assert_eq!(disproof.predicate(), &no_nan_predicate());
        assert_eq!(disproof.declaration_ordinal().get(), 0);
    }

    #[test]
    fn static_disproof_commits_no_partial_operation_or_result() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let invalid = F32Constant::apply(&mut builder, 0x7fc0_0000).unwrap();
        let valid = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let scale = F32Constant::apply(&mut builder, 0.5_f32.to_bits()).unwrap();
        let zero = builder
            .input_resolved(
                InputKey::new("zero").unwrap(),
                Shape::new([]),
                U4::resolved_type(),
            )
            .unwrap();
        let canonical_work_before = builder.retained_canonical_work_bytes();
        assert!(matches!(
            builder.apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[invalid.erase(), scale.erase(), zero],
            ),
            Err(BuildError::SemanticPreconditionDisproved(_))
        ));
        assert_eq!(
            builder.retained_canonical_work_bytes(),
            canonical_work_before
        );
        let result = builder
            .apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[valid.erase(), scale.erase(), zero],
            )
            .unwrap()[0];
        builder
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = builder.build().unwrap();
        assert_eq!(
            program
                .operations()
                .filter(|operation| operation.key() == &quantize_strict_affine_op())
                .count(),
            1
        );
    }

    #[test]
    fn dead_quantize_assessments_are_removed_by_output_compaction() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let expressed = builder
            .input_resolved(
                InputKey::new("expressed").unwrap(),
                Shape::from_dims([2]),
                F32::resolved_type(),
            )
            .unwrap();
        let scale = builder
            .input_resolved(
                InputKey::new("scale").unwrap(),
                Shape::new([]),
                F32::resolved_type(),
            )
            .unwrap();
        let zero = builder
            .input_resolved(
                InputKey::new("zero").unwrap(),
                Shape::new([]),
                U4::resolved_type(),
            )
            .unwrap();
        builder
            .apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[expressed, scale, zero],
            )
            .unwrap();
        builder
            .output_resolved(OutputKey::new("expressed").unwrap(), expressed)
            .unwrap();
        let program = builder.build().unwrap();
        assert_eq!(program.operation_count(), 0);
        assert_eq!(
            program
                .operations()
                .flat_map(super::super::operation::OperationRef::semantic_preconditions)
                .count(),
            0
        );
    }

    #[test]
    fn obligation_identity_is_occurrence_exact_and_topological_order_independent() {
        fn build(reverse: bool) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let left = builder
                .input_resolved(
                    InputKey::new("left").unwrap(),
                    Shape::from_dims([2]),
                    F32::resolved_type(),
                )
                .unwrap();
            let right = builder
                .input_resolved(
                    InputKey::new("right").unwrap(),
                    Shape::from_dims([2]),
                    F32::resolved_type(),
                )
                .unwrap();
            let scale = builder
                .input_resolved(
                    InputKey::new("scale").unwrap(),
                    Shape::new([]),
                    F32::resolved_type(),
                )
                .unwrap();
            let zero = builder
                .input_resolved(
                    InputKey::new("zero").unwrap(),
                    Shape::new([]),
                    U4::resolved_type(),
                )
                .unwrap();
            let apply = |builder: &mut SemanticProgramBuilder, value| {
                builder
                    .apply(
                        quantize_strict_affine_op(),
                        OperationAttributes::empty(),
                        &[value, scale, zero],
                    )
                    .unwrap()[0]
            };
            let (left_result, right_result) = if reverse {
                let right_result = apply(&mut builder, right);
                (apply(&mut builder, left), right_result)
            } else {
                let left_result = apply(&mut builder, left);
                (left_result, apply(&mut builder, right))
            };
            builder
                .output_resolved(OutputKey::new("left").unwrap(), left_result)
                .unwrap();
            builder
                .output_resolved(OutputKey::new("right").unwrap(), right_result)
                .unwrap();
            builder.build().unwrap()
        }

        fn obligations(program: &SemanticProgram) -> Vec<Vec<u8>> {
            let mut identities: Vec<_> = program
                .operations()
                .flat_map(super::super::operation::OperationRef::semantic_preconditions)
                .filter_map(super::super::SemanticPreconditionRef::obligation_identity)
                .map(|identity| identity.as_bytes().to_vec())
                .collect();
            identities.sort_unstable();
            identities
        }

        let ordered = build(false);
        let reversed = build(true);
        assert_eq!(
            ordered.semantic_identity().graph(),
            reversed.semantic_identity().graph()
        );
        assert_eq!(obligations(&ordered), obligations(&reversed));
        let identities = obligations(&ordered);
        assert_eq!(identities.len(), 4);
        assert_ne!(identities[0], identities[1]);
        assert_eq!(
            identities
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
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
