//! Governed compound encoded-value vocabulary and strict affine proof profile.
//!
//! The types here separate four subjects which happen to meet in the first
//! executable proof: logical integer codes, a numerical interpretation,
//! component association, and conversion operations. Physical packing is not
//! part of this module and cannot change the numerical meaning recorded here.

use std::sync::Arc;

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
    positive_finite_scalar_predicate, positive_normal_scalar_predicate,
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
/// **Draft surface, not yet accepted.**
///
/// Static-contract field naming the admissible domain of the scale component.
///
/// This is where the normal-scale guarantee lives, and the choice of carrier is
/// the point rather than an implementation detail: the guarantee is a property
/// of the *type*, while the obligations that establish it are properties of the
/// producers, [`assemble_strict_affine_op`] and [`quantize_strict_affine_op`].
/// The derivation, and the elimination of the two other candidate carriers, are
/// recorded on this module's private `strict_affine_type`, beside the contract
/// it builds.
pub const ENCODED_NUMERIC_SCALE_DOMAIN: AttributeFieldId = AttributeFieldId::new(11);

/// Primary integer-code component of a strict affine value.
pub const STRICT_AFFINE_CODES_ROLE: EncodedComponentRole = EncodedComponentRole::new(1);
/// Positive normal scale component of a strict affine value.
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
        scale_domain_preconditions("strict-affine-assemble"),
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
    // `preserve-subnormals` stays declared and unweakened: it is what the decode
    // *means*, and substituting a flushing realization for it would be the
    // authority substitution ADR 0076 forbids. What changed is that it became
    // dischargeable — the scale domain the type now declares makes every operand
    // and result of this evaluation either a zero or at least the scale in
    // magnitude, so a flushing and a preserving `f32` return identical bits. The
    // derivation is on `strict_affine_type` and
    // [`positive_normal_scalar_predicate`]; a target-side consumer of the
    // discharge is `crate::schedule::SubnormalFreedom`. This operation declares
    // no precondition of its own, and that doc explains why.
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
    SemanticPreconditionDeclarations::new(
        [SemanticPreconditionDeclaration::new(
            no_nan_predicate(),
            OperationOperandIndex::new(0),
            SemanticLogicalView::WholeValue,
            SemanticInvalidInputCode::new("tiler", "strict-affine-quantize-nan", 1)
                .expect("governed invalid-input code is valid"),
        )]
        .into_iter()
        .chain(scale_domain_declarations("strict-affine-quantize")),
    )
    .expect("governed strict-affine preconditions are bounded and distinct")
}

/// Declares the scale-operand value domain of one strict-affine producer.
///
/// **Both predicates are declared, and neither subsumes the other's
/// diagnostic.** [`positive_normal_scalar_predicate`] is logically strictly
/// stronger, so a single declaration of it would reject every value this pair
/// rejects — and would report one code for two unrelated causes. "The scale is
/// zero, negative, infinite, or NaN" is a caller supplying a value that is not
/// a scale at all; "the scale is subnormal" is a caller supplying a real scale
/// that is too small for the decode to remain target-honourable. The fixes
/// differ, so the codes differ.
///
/// Static disproof priority is `(invalid-input code, declaration ordinal)`, and
/// the codes are named so the *general* cause wins when both fail:
/// `…-scale-not-positive-finite` orders before `…-scale-subnormal`, so a zero
/// or negative scale — which is subnormal-free only by being invalid — reports
/// the invalidity rather than the narrower magnitude complaint.
fn scale_domain_declarations(operation: &str) -> [SemanticPreconditionDeclaration; 2] {
    let code = |name: &str| {
        SemanticInvalidInputCode::new("tiler", format!("{operation}-{name}"), 1)
            .expect("governed invalid-input code is valid")
    };
    [
        SemanticPreconditionDeclaration::new(
            positive_finite_scalar_predicate(),
            OperationOperandIndex::new(1),
            SemanticLogicalView::WholeValue,
            code("scale-not-positive-finite"),
        ),
        SemanticPreconditionDeclaration::new(
            positive_normal_scalar_predicate(),
            OperationOperandIndex::new(1),
            SemanticLogicalView::WholeValue,
            code("scale-subnormal"),
        ),
    ]
}

fn scale_domain_preconditions(operation: &str) -> SemanticPreconditionDeclarations {
    SemanticPreconditionDeclarations::new(scale_domain_declarations(operation))
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

/// Builds one complete strict-affine contract.
///
/// # Where the normal-scale obligation attaches, and why it is here
///
/// The obligation has three candidate homes, and they are not interchangeable.
///
/// **`Dequantize` cannot carry it.** Its single operand is the already-assembled
/// compound value, and a decode that receives one cannot re-derive where its
/// scale came from: the scalar producer is no longer an operand, the only
/// logical view is [`SemanticLogicalView::WholeValue`], and the sole static
/// proof basis reads the exact bits of a governed `f32` *scalar* constant. A
/// declaration there could never be proven at compile time, so every decode —
/// including one whose scale is a governed constant — would carry a residual
/// obligation, and the target-honourability question would still have no
/// answer at the point it is asked.
///
/// **The producers carry the obligations.** [`assemble_strict_affine_op`] and
/// [`quantize_strict_affine_op`] each take the scale as a typed rank-zero `f32`
/// operand whose producer *is* visible, so a governed constant proves the
/// predicate statically and a runtime value becomes an exact residual
/// obligation. Both declare it; assembly is not the weaker route.
///
/// **The type carries the guarantee, which is this field.** A value's
/// admissible component domain is part of what its encoded-numeric type *is*,
/// not a fact about one operation that produced it, and the decode consumes the
/// type. Recording it here is what lets a consumer — including a backend
/// deciding whether it can honour the declared subnormal behaviour — rely on
/// the narrowed domain without inspecting provenance it cannot reach. A value
/// bound at the program boundary rather than produced by an operation is
/// subject to the same declared domain; enforcing it against a real payload is
/// runtime validation and is owned by the semantic-precondition enforcement
/// work, not by this contract.
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
                CanonicalField::new(
                    ENCODED_NUMERIC_SCALE_DOMAIN,
                    CanonicalValue::utf8("positive-normal-f32")
                        .expect("scale-domain contract is bounded"),
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
        outputs.try_push(ValueFact::new(
            result_type,
            request.static_operand_shape(0)?.clone(),
        ))
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
        outputs.try_push(ValueFact::new(
            result_type,
            request.static_operand_shape(0)?.clone(),
        ))
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
            request.static_operand_shape(0)?.clone(),
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
    // Rank, not shape equality. Rank is fixed whatever an extent's source is,
    // and a rank-zero boundary has no extent to be symbolic — the normalization
    // invariant makes every rank-zero `SourcedShape` the static empty one — so
    // this refuses exactly what it refused before and cannot admit a symbol.
    if value.shape().rank() != 0 {
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
    use std::collections::BTreeSet;

    use super::*;
    use crate::semantic::{
        BuildError, F32Constant, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey,
        RegistryError, SemanticPreconditionStatus, SemanticPredicateIdentity, SemanticProgram,
        SemanticProgramBuilder,
    };
    use crate::shape::Shape;

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
        assert_eq!(assembled[0].shape().as_static(), Some(&tensor));

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

    fn declared_preconditions(
        operation: &OpKey,
    ) -> Vec<(SemanticPredicateIdentity, u32, SemanticInvalidInputCode)> {
        FrozenSemanticRegistry::standard()
            .unwrap()
            .operation_definition(operation)
            .unwrap()
            .semantic_preconditions()
            .as_slice()
            .iter()
            .map(|declaration| {
                assert_eq!(declaration.view(), SemanticLogicalView::WholeValue);
                (
                    declaration.predicate().clone(),
                    declaration.operand().get(),
                    declaration.invalid_input_code().clone(),
                )
            })
            .collect()
    }

    fn code(name: &str) -> SemanticInvalidInputCode {
        SemanticInvalidInputCode::new("tiler", name, 1).unwrap()
    }

    #[test]
    fn strict_quantize_declares_its_three_semantic_value_preconditions_exactly() {
        assert_eq!(
            declared_preconditions(&quantize_strict_affine_op()),
            vec![
                (no_nan_predicate(), 0, code("strict-affine-quantize-nan")),
                (
                    positive_finite_scalar_predicate(),
                    1,
                    code("strict-affine-quantize-scale-not-positive-finite"),
                ),
                (
                    positive_normal_scalar_predicate(),
                    1,
                    code("strict-affine-quantize-scale-subnormal"),
                ),
            ]
        );
    }

    /// Assembly is not the weaker route into an encoded value.
    ///
    /// A quantized language-model weight reaches a decode by being *assembled*
    /// from stored codes and parameters, never by being quantized on device, so
    /// an obligation declared only by `Quantize` would leave the profile's own
    /// path unconstrained.
    #[test]
    fn strict_assemble_declares_the_same_scale_domain_under_its_own_codes() {
        assert_eq!(
            declared_preconditions(&assemble_strict_affine_op()),
            vec![
                (
                    positive_finite_scalar_predicate(),
                    1,
                    code("strict-affine-assemble-scale-not-positive-finite"),
                ),
                (
                    positive_normal_scalar_predicate(),
                    1,
                    code("strict-affine-assemble-scale-subnormal"),
                ),
            ]
        );
    }

    /// The decode declares none, and the reason is structural rather than an
    /// omission: its only operand is the assembled compound value, whose scale
    /// is no longer a scalar a static proof basis or a runtime obligation could
    /// name. The guarantee it consumes is the type's.
    #[test]
    fn strict_dequantize_declares_no_precondition_and_the_type_carries_the_domain() {
        assert_eq!(declared_preconditions(&dequantize_strict_affine_op()), []);
        for resolved in [
            StrictAffineU4::resolved_type(),
            StrictAffineU8::resolved_type(),
        ] {
            let (_, contract) = resolved.encoded_numeric_parts().unwrap();
            let domain = contract
                .fields()
                .iter()
                .find(|field| field.id() == ENCODED_NUMERIC_SCALE_DOMAIN)
                .unwrap();
            assert_eq!(
                domain.value(),
                &CanonicalValue::utf8("positive-normal-f32").unwrap()
            );
        }
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

    /// A runtime scale leaves the normal-scale obligation residual, in order.
    ///
    /// This is the second of the two paths the strengthened predicate has to
    /// support: a governed constant proves it outright, and a value only known
    /// at run time becomes an exact obligation with its own canonical identity,
    /// for runtime validation to discharge. Nothing here enforces a payload.
    #[test]
    fn runtime_unknown_u4_and_u8_quantize_inputs_retain_three_ordered_residuals() {
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
            assert_eq!(
                preconditions
                    .iter()
                    .map(|precondition| (
                        precondition.declaration_ordinal().get(),
                        precondition.predicate().clone(),
                        precondition.status(),
                    ))
                    .collect::<Vec<_>>(),
                vec![
                    (0, no_nan_predicate(), SemanticPreconditionStatus::Residual),
                    (
                        1,
                        positive_finite_scalar_predicate(),
                        SemanticPreconditionStatus::Residual,
                    ),
                    (
                        2,
                        positive_normal_scalar_predicate(),
                        SemanticPreconditionStatus::Residual,
                    ),
                ]
            );
            // Three obligations, three distinct identities: the scale bears two
            // and they must not collapse into one runtime check.
            let identities: BTreeSet<_> = preconditions
                .iter()
                .map(|precondition| {
                    precondition
                        .obligation_identity()
                        .expect("a residual precondition carries an obligation identity")
                        .as_bytes()
                        .to_vec()
                })
                .collect();
            assert_eq!(identities.len(), 3);
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

    /// A governed constant scale proves every predicate statically.
    ///
    /// The scale corpus is the boundary rather than a sample: `MIN_POSITIVE` is
    /// the smallest value the strengthened predicate admits, and the largest
    /// subnormal one bit below it is disproved by
    /// `exact_nan_and_invalid_scale_constants_disprove_transactionally`.
    #[test]
    fn exact_governed_constants_prove_each_predicate_without_an_obligation() {
        for expressed in [0.0_f32, -0.0_f32, 1.0_f32, f32::NEG_INFINITY, f32::INFINITY] {
            let program =
                constant_quantize_program(expressed.to_bits(), f32::MIN_POSITIVE.to_bits())
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
        for scale in [f32::MIN_POSITIVE, 0.5_f32, f32::MAX] {
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
                (
                    positive_normal_scalar_predicate(),
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
                (
                    positive_normal_scalar_predicate(),
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

    /// The two scale-domain causes report two different codes.
    ///
    /// A caller that supplied `0.0` has to be told to supply a scale at all; a
    /// caller that supplied `1e-40` supplied a real scale and has to be told it
    /// is too small for the decode to stay target-honourable. One shared code
    /// would send both to the wrong fix, which is why the stronger predicate is
    /// a second declaration rather than a tightening of the first.
    ///
    /// The corpus straddles the boundary exactly: the largest subnormal is
    /// rejected and the smallest normal one bit above it is accepted, so this
    /// pins where the domain ends rather than that it ends somewhere.
    #[test]
    fn a_subnormal_scale_is_disproved_under_its_own_code_not_the_finiteness_one() {
        for scale_bits in [
            0x0000_0001,                     // smallest positive subnormal
            0x0000_ffff,                     // an interior subnormal
            f32::MIN_POSITIVE.to_bits() - 1, // largest positive subnormal
        ] {
            let error = constant_quantize_program(1.0_f32.to_bits(), scale_bits).unwrap_err();
            let BuildError::SemanticPreconditionDisproved(disproof) = error else {
                panic!("a subnormal scale must be a typed semantic precondition disproof")
            };
            assert_eq!(disproof.predicate(), &positive_normal_scalar_predicate());
            assert_eq!(
                disproof.invalid_input_code(),
                &code("strict-affine-quantize-scale-subnormal"),
                "{scale_bits:#010x} must report the subnormal cause, not the finiteness one",
            );
            assert_eq!(disproof.declaration_ordinal().get(), 2);
        }

        // One bit higher is the smallest admitted scale, and it builds.
        constant_quantize_program(1.0_f32.to_bits(), f32::MIN_POSITIVE.to_bits()).unwrap();

        // A negative subnormal fails both predicates. The general cause wins,
        // by the code ordering the declarations were named for.
        let error = constant_quantize_program(1.0_f32.to_bits(), 0x8000_0001).unwrap_err();
        let BuildError::SemanticPreconditionDisproved(disproof) = error else {
            panic!("a negative subnormal scale must be a typed disproof")
        };
        assert_eq!(
            disproof.invalid_input_code(),
            &code("strict-affine-quantize-scale-not-positive-finite"),
        );
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
        // Two occurrences, three declarations each. The two scale predicates
        // share an operand and differ only in predicate and code, so this also
        // proves the encoding separates declarations that agree on everything
        // else.
        assert_eq!(identities.len(), 6);
        assert_ne!(identities[0], identities[1]);
        assert_eq!(identities.iter().collect::<BTreeSet<_>>().len(), 6);
    }

    /// One row of the Assemble scale-class table: the class name, the exact
    /// scale bits, and the refusal the class must take — predicate,
    /// invalid-input code name, and declaration ordinal — or `None` when the
    /// class proves both declarations statically.
    type ScaleClass = (
        &'static str,
        u32,
        Option<(SemanticPredicateIdentity, &'static str, u32)>,
    );

    /// Every `f32` class the scale operand can take, and its Assemble outcome.
    ///
    /// The table is exhaustive over the classes `f32` distinguishes — both
    /// signed zeros, negative and positive finite normals, the subnormal range
    /// at both ends and in its interior, both signed subnormals, both
    /// infinities, and a quiet and a signalling NaN — rather than a sample of
    /// them, because a domain check that admits one unlisted class admits a
    /// value the decode's derivation does not cover.
    ///
    /// The ordinals are the reason this table is
    /// checked on Assemble rather than shared with Quantize: Assemble declares
    /// no `NoNaN`, so its scale predicates sit at ordinals 0 and 1 where
    /// Quantize's sit at 1 and 2, and an assertion reused across the two would
    /// pass against a subject bound to the wrong occurrence.
    fn assemble_scale_classes() -> Vec<ScaleClass> {
        let not_positive_finite = || {
            Some((
                positive_finite_scalar_predicate(),
                "strict-affine-assemble-scale-not-positive-finite",
                0,
            ))
        };
        let subnormal = || {
            Some((
                positive_normal_scalar_predicate(),
                "strict-affine-assemble-scale-subnormal",
                1,
            ))
        };
        vec![
            ("positive zero", 0.0_f32.to_bits(), not_positive_finite()),
            ("negative zero", (-0.0_f32).to_bits(), not_positive_finite()),
            (
                "negative finite normal",
                (-1.0_f32).to_bits(),
                not_positive_finite(),
            ),
            (
                "largest negative finite",
                f32::MIN.to_bits(),
                not_positive_finite(),
            ),
            ("negative subnormal", 0x8000_0001, not_positive_finite()),
            (
                "positive infinity",
                f32::INFINITY.to_bits(),
                not_positive_finite(),
            ),
            (
                "negative infinity",
                f32::NEG_INFINITY.to_bits(),
                not_positive_finite(),
            ),
            ("quiet NaN", 0x7fc0_0000, not_positive_finite()),
            ("signalling NaN", 0x7f80_0001, not_positive_finite()),
            ("smallest positive subnormal", 0x0000_0001, subnormal()),
            ("interior positive subnormal", 0x0000_ffff, subnormal()),
            (
                "largest positive subnormal",
                f32::MIN_POSITIVE.to_bits() - 1,
                subnormal(),
            ),
            (
                "smallest positive normal",
                f32::MIN_POSITIVE.to_bits(),
                None,
            ),
            ("interior positive normal", 0.5_f32.to_bits(), None),
            ("largest positive normal", f32::MAX.to_bits(), None),
        ]
    }

    /// Each scale class takes its exact Assemble outcome, transactionally.
    ///
    /// Transactionality is asserted per class rather than once: a disproved
    /// apply must leave the builder's retained canonical work exactly where it
    /// was, so no partial operation or result survives a refusal, and the
    /// builder stays usable for the classes that follow.
    #[test]
    fn every_exact_constant_scale_class_takes_its_assemble_outcome_transactionally() {
        for (code_type, encoded_type) in [
            (U4::resolved_type(), StrictAffineU4::resolved_type()),
            (U8::resolved_type(), StrictAffineU8::resolved_type()),
        ] {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let codes = builder
                .input_resolved(
                    InputKey::new("codes").unwrap(),
                    Shape::from_dims([2]),
                    code_type.clone(),
                )
                .unwrap();
            let zero = builder
                .input_resolved(
                    InputKey::new("zero").unwrap(),
                    Shape::new([]),
                    code_type.clone(),
                )
                .unwrap();
            let mut proved = Vec::new();
            for (class, scale_bits, refusal) in assemble_scale_classes() {
                let scale = F32Constant::apply(&mut builder, scale_bits)
                    .unwrap()
                    .erase();
                let committed = builder.retained_canonical_work_bytes();
                let applied = builder.apply(
                    assemble_strict_affine_op(),
                    OperationAttributes::empty(),
                    &[codes, scale, zero],
                );
                let Some((predicate, invalid_input_code, ordinal)) = refusal else {
                    proved.push(
                        applied.unwrap_or_else(|error| panic!("{class} must assemble: {error:?}"))
                            [0],
                    );
                    continue;
                };
                let Err(BuildError::SemanticPreconditionDisproved(disproof)) = applied else {
                    panic!("{class} must be a typed semantic precondition disproof")
                };
                assert_eq!(disproof.predicate(), &predicate, "{class}");
                assert_eq!(
                    disproof.invalid_input_code(),
                    &code(invalid_input_code),
                    "{class}",
                );
                assert_eq!(disproof.declaration_ordinal().get(), ordinal, "{class}");
                assert_eq!(
                    builder.retained_canonical_work_bytes(),
                    committed,
                    "{class} must commit no canonical work",
                );
            }

            assert_eq!(proved.len(), 3);
            for (ordinal, result) in proved.iter().enumerate() {
                builder
                    .output_resolved(
                        OutputKey::new(format!("assembled-{ordinal}")).unwrap(),
                        *result,
                    )
                    .unwrap();
            }
            let program = builder.build().unwrap();
            let assembled: Vec<_> = program
                .operations()
                .filter(|operation| operation.key() == &assemble_strict_affine_op())
                .collect();
            assert_eq!(assembled.len(), 3);
            for operation in assembled {
                let result = program.value(operation.results().next().unwrap()).unwrap();
                assert_eq!(result.resolved_type(), &encoded_type);
                let assessments: Vec<_> = operation.semantic_preconditions().collect();
                assert_eq!(assessments.len(), 2);
                for assessment in assessments {
                    assert_eq!(assessment.status(), SemanticPreconditionStatus::Proven);
                    assert_eq!(
                        assessment.proof_basis(),
                        Some(
                            super::super::SemanticPreconditionProofBasis::StandardConstantF32BitsV1
                        )
                    );
                    assert!(assessment.obligation_identity().is_none());
                }
            }
        }
    }

    /// A runtime scale leaves exactly two ordered residuals on Assemble.
    ///
    /// Two, not one: the scale bears both predicates, and collapsing them into
    /// a single runtime check would lose the ability to report which of the two
    /// causes a bound payload hit.
    #[test]
    fn runtime_unknown_u4_and_u8_assemble_scales_retain_two_ordered_residuals() {
        for (code_type, encoded_type) in [
            (U4::resolved_type(), StrictAffineU4::resolved_type()),
            (U8::resolved_type(), StrictAffineU8::resolved_type()),
        ] {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let codes = builder
                .input_resolved(
                    InputKey::new("codes").unwrap(),
                    Shape::from_dims([2]),
                    code_type.clone(),
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
                .input_resolved(InputKey::new("zero").unwrap(), Shape::new([]), code_type)
                .unwrap();
            let result = builder
                .apply(
                    assemble_strict_affine_op(),
                    OperationAttributes::empty(),
                    &[codes, scale, zero],
                )
                .unwrap()[0];
            builder
                .output_resolved(OutputKey::new("assembled").unwrap(), result)
                .unwrap();
            let program = builder.build().unwrap();
            let operation = program
                .operations()
                .find(|operation| operation.key() == &assemble_strict_affine_op())
                .unwrap();
            assert_eq!(
                program
                    .value(operation.results().next().unwrap())
                    .unwrap()
                    .resolved_type(),
                &encoded_type
            );
            let preconditions: Vec<_> = operation.semantic_preconditions().collect();
            assert_eq!(
                preconditions
                    .iter()
                    .map(|precondition| (
                        precondition.declaration_ordinal().get(),
                        precondition.predicate().clone(),
                        precondition.status(),
                    ))
                    .collect::<Vec<_>>(),
                vec![
                    (
                        0,
                        positive_finite_scalar_predicate(),
                        SemanticPreconditionStatus::Residual,
                    ),
                    (
                        1,
                        positive_normal_scalar_predicate(),
                        SemanticPreconditionStatus::Residual,
                    ),
                ]
            );
            let identities: BTreeSet<_> = preconditions
                .iter()
                .map(|precondition| {
                    precondition
                        .obligation_identity()
                        .expect("a residual precondition carries an obligation identity")
                        .as_bytes()
                        .to_vec()
                })
                .collect();
            assert_eq!(identities.len(), 2);
        }
    }

    #[test]
    fn dead_assemble_assessments_are_removed_by_output_compaction() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let codes = builder
            .input_resolved(
                InputKey::new("codes").unwrap(),
                Shape::from_dims([2]),
                U4::resolved_type(),
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
                assemble_strict_affine_op(),
                OperationAttributes::empty(),
                &[codes, scale, zero],
            )
            .unwrap();
        builder
            .output_resolved(OutputKey::new("codes").unwrap(), codes)
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

    /// An Assemble obligation belongs to its occurrence, not to its subject.
    ///
    /// Both occurrences take the *same* scale value, so their four scale
    /// declarations agree on predicate, operand index, logical view, subject
    /// value, resolved type, shape, and declaration ordinal — the occurrence
    /// coordinate is the only thing separating the pairs, and an encoding that
    /// folded it out would let one runtime discharge satisfy the other
    /// occurrence's obligation. This is the Assemble counterpart of
    /// [`obligation_identity_is_occurrence_exact_and_topological_order_independent`],
    /// which pins the same property over two Quantize occurrences.
    ///
    /// The Quantize occurrence sharing the same scale is included to check the
    /// cross-operation claim end to end. That claim is over-determined — the
    /// two operations' declarations already differ in ordinal *and*
    /// invalid-input code, so no single dropped field collapses them — and the
    /// declaration-level guarantee it rests on is the one pinned by
    /// [`strict_assemble_declares_the_same_scale_domain_under_its_own_codes`].
    #[test]
    fn assemble_obligations_are_occurrence_exact_over_one_shared_scale() {
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
        let mut assembled = Vec::new();
        for name in ["left", "right"] {
            let codes = builder
                .input_resolved(
                    InputKey::new(format!("codes-{name}")).unwrap(),
                    Shape::from_dims([2]),
                    U4::resolved_type(),
                )
                .unwrap();
            assembled.push(
                builder
                    .apply(
                        assemble_strict_affine_op(),
                        OperationAttributes::empty(),
                        &[codes, scale, zero],
                    )
                    .unwrap()[0],
            );
        }
        let quantized = builder
            .apply(
                quantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[expressed, scale, zero],
            )
            .unwrap()[0];
        for (ordinal, value) in assembled.iter().chain([&quantized]).enumerate() {
            builder
                .output_resolved(OutputKey::new(format!("out-{ordinal}")).unwrap(), *value)
                .unwrap();
        }
        let program = builder.build().unwrap();

        let obligations = |key: &OpKey| -> Vec<Vec<u8>> {
            program
                .operations()
                .filter(|operation| operation.key() == key)
                .flat_map(super::super::operation::OperationRef::semantic_preconditions)
                .filter_map(super::super::SemanticPreconditionRef::obligation_identity)
                .map(|identity| identity.as_bytes().to_vec())
                .collect()
        };
        let assemble_obligations = obligations(&assemble_strict_affine_op());
        assert_eq!(assemble_obligations.len(), 4);
        assert_eq!(
            assemble_obligations.iter().collect::<BTreeSet<_>>().len(),
            4,
            "two Assemble occurrences over one scale must not share an obligation",
        );

        let quantize_obligations = obligations(&quantize_strict_affine_op());
        assert_eq!(quantize_obligations.len(), 3);
        assert_eq!(
            assemble_obligations
                .iter()
                .chain(&quantize_obligations)
                .collect::<BTreeSet<_>>()
                .len(),
            7,
            "an Assemble obligation is never a Quantize obligation",
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
