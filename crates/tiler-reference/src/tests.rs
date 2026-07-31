//! The reference implementation's test suite.
//!
//! Split out of `lib.rs` so the crate root introduces the public reference
//! boundary rather than opening with two thousand lines of fixtures. The
//! module is `#[cfg(test)]` at its declaration, so nothing here is compiled
//! into the library.

use std::sync::Arc;

use super::evaluate::{
    EvaluationRetention, ReferenceWork, decode_f32, f32_element, f32_elements,
    reserve_evaluation_work, row_major_strides, strict_sum,
};
use super::registry::ReferenceRegistrationBatch;
use super::standard::StandardReferenceProvider;
use super::*;
use tiler_ir::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, F32, F32Add, F32Constant,
    F32Multiply, InputKey, NormativeDefinitionRef, OperationArity, OperationAttributeSchema,
    OperationConformance, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferencer, OperationSchema, OutputKey,
    SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, StrictSerialF32Sum, TypeDefinitionFacts, Value, ValueTypeDefinition,
    ValueTypeDefinitionKey,
};
use tiler_ir::semantic::{
    FrozenSemanticRegistry, OpKey, OperationAttributes, ProviderIdentity, ResolvedValueType,
    SemanticProgram, TypeKey,
};
use tiler_ir::shape::{Axis, Shape};

fn constant_bits(graph: &mut SemanticProgramBuilder, bits: u32) -> Value<F32> {
    F32Constant::apply(graph, bits).unwrap()
}

fn constant(graph: &mut SemanticProgramBuilder, value: f32) -> Value<F32> {
    constant_bits(graph, value.to_bits())
}

fn multiply(graph: &mut SemanticProgramBuilder, left: Value<F32>, right: Value<F32>) -> Value<F32> {
    F32Multiply::apply(graph, left, right).unwrap()
}

fn add(graph: &mut SemanticProgramBuilder, left: Value<F32>, right: Value<F32>) -> Value<F32> {
    F32Add::apply(graph, left, right).unwrap()
}

fn sum(
    graph: &mut SemanticProgramBuilder,
    input: Value<F32>,
    axes: impl IntoIterator<Item = Axis>,
) -> Value<F32> {
    StrictSerialF32Sum::apply(graph, input, axes).unwrap()
}

fn graph(shape: Shape, axes: &[u32]) -> SemanticProgram {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let x = graph
        .input::<F32>(InputKey::new("x").unwrap(), shape)
        .unwrap();
    let scale = constant(&mut graph, 2.0);
    let bias = constant(&mut graph, 1.0);
    let product = multiply(&mut graph, x, scale);
    let mapped = add(&mut graph, product, bias);
    let sum = sum(&mut graph, mapped, axes.iter().copied().map(Axis::new));
    graph
        .output(OutputKey::new("mapped").unwrap(), mapped)
        .unwrap();
    graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    graph.build().unwrap()
}

fn evaluate_program(
    program: &SemanticProgram,
    inputs: &[InputBinding<'_>],
) -> Result<Vec<Tensor>, EvaluationError> {
    ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(program, inputs)
}

fn f32_tensor(shape: Shape, values: Vec<f32>) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        shape,
        values
            .into_iter()
            .map(f32_element)
            .collect::<Result<_, _>>()
            .unwrap(),
    )
    .unwrap()
}

fn f32_values(tensor: &Tensor) -> Vec<f32> {
    f32_elements(tensor)
        .unwrap()
        .iter()
        .map(decode_f32)
        .collect::<Result<_, _>>()
        .unwrap()
}

fn f32_bits(tensor: &Tensor) -> Vec<u32> {
    f32_values(tensor).into_iter().map(f32::to_bits).collect()
}

fn reference_builder_for(semantic_registry: FrozenSemanticRegistry) -> ReferenceRegistryBuilder {
    let mut builder = ReferenceRegistryBuilder::new(semantic_registry);
    builder
        .register_provider(&StandardReferenceProvider)
        .unwrap();
    builder
}

fn external_semantics() -> FrozenSemanticRegistry {
    let mut semantics = SemanticRegistryBuilder::standard().unwrap();
    semantics
        .register_provider(&ExternalSemanticProvider)
        .unwrap();
    semantics.freeze().unwrap()
}

fn external_identity_program(semantic_registry: FrozenSemanticRegistry) -> SemanticProgram {
    let mut graph = SemanticProgramBuilder::try_new(semantic_registry).unwrap();
    let input = graph
        .input_resolved(
            InputKey::new("x").unwrap(),
            Shape::from_dims([2]),
            F32::resolved_type(),
        )
        .unwrap();
    let result = graph
        .apply(
            external_identity_op(),
            OperationAttributes::empty(),
            &[input],
        )
        .unwrap();
    graph
        .output_resolved(OutputKey::new("result").unwrap(), result[0])
        .unwrap();
    graph.build().unwrap()
}

fn external_identity_op() -> OpKey {
    OpKey::new("test", "reference-identity", 1).unwrap()
}

struct IdentitySemantic;
impl OperationInferencer for IdentitySemantic {
    fn infer(
        &self,
        request: tiler_ir::semantic::OperationInferenceRequest<'_>,
        outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        outputs.try_push(request.operands()[0].clone())
    }
}

struct ExternalSemanticProvider;
impl SemanticRegistryProvider for ExternalSemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "reference-semantics", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            external_identity_op(),
            OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), []).unwrap(),
            NormativeDefinitionRef::new("test reference identity v1")?,
            OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            OperationConformance::new(CanonicalValue::utf8("test.reference-identity.v1").unwrap()),
            OperationEffect::Pure,
            Arc::new(IdentitySemantic),
        ))
    }
}

struct ChangedExternalSemanticProvider;
impl SemanticRegistryProvider for ChangedExternalSemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "reference-semantics", 2).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            external_identity_op(),
            OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), []).unwrap(),
            NormativeDefinitionRef::new("test reference identity changed v2")?,
            OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            OperationConformance::new(CanonicalValue::utf8("test.reference-identity.v2").unwrap()),
            OperationEffect::Pure,
            Arc::new(IdentitySemantic),
        ))
    }
}

fn external_u8_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("test", "u8", 1).unwrap())
}

struct ExternalU8SemanticProvider;
impl SemanticRegistryProvider for ExternalU8SemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "u8-semantics", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "u8", 1).unwrap()),
            NormativeDefinitionRef::new("test unsigned byte v1")?,
            TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
        ))
    }
}

fn compound_limit_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("test", "compound-limit", 1).unwrap())
}

struct CompoundLimitSemanticProvider;
impl SemanticRegistryProvider for CompoundLimitSemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "compound-limit-semantics", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "compound-limit", 1).unwrap()),
            NormativeDefinitionRef::new("test compound limit v1")?,
            TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
        ))
    }
}

fn attributed_identity_op() -> OpKey {
    OpKey::new("test", "attributed-reference-identity", 1).unwrap()
}

struct AttributeTypeProvider {
    provider_revision: u32,
    definition_revision: u32,
}
impl SemanticRegistryProvider for AttributeTypeProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "attribute-type-semantics", self.provider_revision).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "attribute-type", 1).unwrap()),
            NormativeDefinitionRef::new(format!(
                "test attribute type v{}",
                self.definition_revision
            ))?,
            TypeDefinitionFacts::new(CanonicalValue::unsigned_u32(self.definition_revision)),
        ))
    }
}

fn attribute_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("test", "attribute-type", 1).unwrap())
}

struct AttributedIdentitySemanticProvider;
impl SemanticRegistryProvider for AttributedIdentitySemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "attributed-identity-semantics", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            attributed_identity_op(),
            OperationSchema::new(
                OperationArity::exact(1),
                OperationArity::exact(1),
                [OperationAttributeSchema::required(
                    AttributeFieldId::new(1),
                    CanonicalValueKind::Type,
                )],
            )
            .unwrap(),
            NormativeDefinitionRef::new("test attributed reference identity v1")?,
            OperationDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            OperationConformance::new(
                CanonicalValue::utf8("test.attributed-reference-identity.v1").unwrap(),
            ),
            OperationEffect::Pure,
            Arc::new(IdentitySemantic),
        ))
    }
}

struct AttributedIdentityReferenceProvider;
impl ReferenceRegistryProvider for AttributedIdentityReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "attributed-reference-capability", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        registrar.register(
            attributed_identity_op(),
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
            ReferenceCapabilityRevision::new(7)?,
            Arc::new(IdentityReference),
        )
    }
}

fn attributed_semantics(
    attribute_provider_revision: u32,
    attribute_definition_revision: u32,
) -> FrozenSemanticRegistry {
    let mut builder = SemanticRegistryBuilder::standard().unwrap();
    builder
        .register_provider(&AttributeTypeProvider {
            provider_revision: attribute_provider_revision,
            definition_revision: attribute_definition_revision,
        })
        .unwrap();
    builder
        .register_provider(&AttributedIdentitySemanticProvider)
        .unwrap();
    builder.freeze().unwrap()
}

fn attributed_program(semantics: FrozenSemanticRegistry) -> SemanticProgram {
    let mut graph = SemanticProgramBuilder::try_new(semantics).unwrap();
    let input = graph
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let attributes = OperationAttributes::new([CanonicalField::new(
        AttributeFieldId::new(1),
        CanonicalValue::value_type(attribute_type()),
    )])
    .unwrap();
    let result = graph
        .apply(attributed_identity_op(), attributes, &[input.erase()])
        .unwrap();
    graph
        .output_resolved(OutputKey::new("result").unwrap(), result[0])
        .unwrap();
    graph.build().unwrap()
}

struct IdentityReference;
impl ReferenceOperation for IdentityReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        outputs.push(request.operands()[0].clone())
    }
}

#[derive(Clone, Copy)]
enum MalformedReferenceResult {
    CallbackFailure,
    WrongArity,
    WrongShape,
    WrongType,
}

struct MalformedReference {
    result: MalformedReferenceResult,
}

impl ReferenceOperation for MalformedReference {
    fn evaluate(
        &self,
        _: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        match self.result {
            MalformedReferenceResult::CallbackFailure => {
                Err(ReferenceOperationError::InvalidApplication)
            }
            MalformedReferenceResult::WrongArity => Ok(()),
            MalformedReferenceResult::WrongShape => {
                outputs.push(f32_tensor(Shape::new([]), vec![0.0]))
            }
            MalformedReferenceResult::WrongType => outputs.push(
                Tensor::dense(
                    external_u8_type(),
                    Shape::from_dims([2]),
                    vec![
                        ReferenceElement::new([1]).unwrap(),
                        ReferenceElement::new([2]).unwrap(),
                    ],
                )
                .unwrap(),
            ),
        }
    }
}

struct ExternalReferenceProvider {
    capability_revision: u32,
}

impl ReferenceRegistryProvider for ExternalReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "reference-capabilities", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        registrar.register(
            external_identity_op(),
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
            ReferenceCapabilityRevision::new(self.capability_revision)?,
            Arc::new(IdentityReference),
        )
    }
}

struct ExternalU8Validator;
impl ReferenceValueValidator for ExternalU8Validator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            return Err(ReferenceValueError::InvalidRepresentation);
        };
        if tensor.resolved_type() != &external_u8_type()
            || elements.iter().any(|element| element.as_bytes().len() != 1)
        {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        Ok(())
    }
}

struct ExternalU8ReferenceProvider;
impl ReferenceRegistryProvider for ExternalU8ReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "u8-reference", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        registrar.register_value_type(
            external_u8_type(),
            ReferenceCapabilityRevision::new(1)?,
            Arc::new(ExternalU8Validator),
        )
    }
}

struct CompoundLimitValidator;
impl ReferenceValueValidator for CompoundLimitValidator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        if tensor.resolved_type() == &compound_limit_type()
            && matches!(tensor.payload(), TensorPayloadView::Compound([]))
        {
            Ok(())
        } else {
            Err(ReferenceValueError::InvalidRepresentation)
        }
    }
}

struct CompoundLimitReferenceProvider;
impl ReferenceRegistryProvider for CompoundLimitReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "compound-limit-reference", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        registrar.register_value_type(
            compound_limit_type(),
            ReferenceCapabilityRevision::new(1)?,
            Arc::new(CompoundLimitValidator),
        )
    }
}

struct IgnoredDuplicateReferenceProvider;
impl ReferenceRegistryProvider for IgnoredDuplicateReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "ignored-reference-duplicate", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        let signature = ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?;
        registrar.register(
            external_identity_op(),
            signature.clone(),
            ReferenceCapabilityRevision::new(1)?,
            Arc::new(IdentityReference),
        )?;
        let _ = registrar.register(
            external_identity_op(),
            signature,
            ReferenceCapabilityRevision::new(1)?,
            Arc::new(IdentityReference),
        );
        Ok(())
    }
}

struct MalformedReferenceProvider {
    result: MalformedReferenceResult,
}

impl ReferenceRegistryProvider for MalformedReferenceProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "malformed-reference-capability", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut ReferenceRegistryRegistrar<'_>,
    ) -> Result<(), ReferenceRegistryError> {
        registrar.register(
            external_identity_op(),
            ReferenceSignature::new([F32::resolved_type()], [F32::resolved_type()])?,
            ReferenceCapabilityRevision::new(1)?,
            Arc::new(MalformedReference {
                result: self.result,
            }),
        )
    }
}

fn evaluate_one(program: &SemanticProgram, input: &Tensor) -> Vec<Tensor> {
    let key = InputKey::new("x").unwrap();
    evaluate_program(program, &[InputBinding::new(&key, input)]).unwrap()
}

#[test]
fn evaluates_pointwise_prologue_and_multiple_outputs() {
    let program = graph(Shape::from_dims([2, 3]), &[1]);
    let input = f32_tensor(Shape::from_dims([2, 3]), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let outputs = evaluate_one(&program, &input);
    assert_eq!(f32_values(&outputs[0]), [3.0, 5.0, 7.0, 9.0, 11.0, 13.0]);
    assert_eq!(outputs[1].shape(), &Shape::from_dims([2]));
    assert_eq!(f32_values(&outputs[1]), [15.0, 33.0]);
}

#[test]
fn contributor_order_is_original_axis_lexicographic() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let x = graph
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2, 2]))
        .unwrap();
    let sum = sum(&mut graph, x, [Axis::new(0), Axis::new(2)]);
    graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    let program = graph.build().unwrap();
    let input = f32_tensor(
        Shape::from_dims([2, 2, 2]),
        vec![1.0e20, 1.0, 7.0, 8.0, -1.0e20, 3.0, 9.0, 10.0],
    );
    let outputs = evaluate_one(&program, &input);
    assert_eq!(
        f32_bits(&outputs[0]),
        [3.0_f32.to_bits(), 34.0_f32.to_bits()]
    );
}

#[test]
fn strict_sum_preserves_non_nan_singletons_and_canonicalizes_nan_results() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let x = graph
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([3, 1]))
        .unwrap();
    let sum = sum(&mut graph, x, [Axis::new(1)]);
    graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    let program = graph.build().unwrap();
    let nan = f32::from_bits(0x7fc0_1234);
    let input = f32_tensor(Shape::from_dims([3, 1]), vec![-0.0, f32::INFINITY, nan]);
    let output = evaluate_one(&program, &input);
    let bits = f32_bits(&output[0]);
    assert_eq!(bits[0], (-0.0_f32).to_bits());
    assert_eq!(bits[1], f32::INFINITY.to_bits());
    assert_eq!(bits[2], CANONICAL_F32_ARITHMETIC_NAN_BITS);
}

#[test]
fn multiply_and_add_remain_two_rounding_operations() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let x = graph
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([1]))
        .unwrap();
    let scale = constant_bits(&mut graph, 0x3f7f_ffff);
    let bias = constant(&mut graph, -1.0);
    let product = multiply(&mut graph, x, scale);
    let mapped = add(&mut graph, product, bias);
    let sum = sum(&mut graph, mapped, [Axis::new(0)]);
    graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    let program = graph.build().unwrap();
    let input = f32_tensor(Shape::from_dims([1]), vec![f32::from_bits(0x3f80_0001)]);
    let output = evaluate_one(&program, &input);
    assert_eq!(f32_bits(&output[0])[0], 0.0_f32.to_bits());
    assert_ne!(
        f32::from_bits(0x3f80_0001)
            .mul_add(f32::from_bits(0x3f7f_ffff), -1.0)
            .to_bits(),
        0.0_f32.to_bits()
    );
}

#[test]
fn empty_reduced_domain_is_positive_zero_but_empty_survivor_has_no_elements() {
    let program = graph(Shape::from_dims([2, 0]), &[1]);
    let input = f32_tensor(Shape::from_dims([2, 0]), vec![]);
    let outputs = evaluate_one(&program, &input);
    assert_eq!(f32_values(&outputs[1]).len(), 2);
    assert!(
        f32_values(&outputs[1])
            .iter()
            .all(|value| value.to_bits() == 0.0_f32.to_bits())
    );

    let program = graph(Shape::from_dims([0, 2]), &[1]);
    let input = f32_tensor(Shape::from_dims([0, 2]), vec![]);
    let outputs = evaluate_one(&program, &input);
    assert!(f32_values(&outputs[1]).is_empty());
}

#[test]
fn bindings_validate_ordered_keys_shapes_and_payloads() {
    assert_eq!(
        Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([2]),
            vec![f32_element(1.0).unwrap()],
        )
        .unwrap_err(),
        EvaluationError::ElementCount {
            expected: 2,
            actual: 1,
        }
    );
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let left_key = InputKey::new("left").unwrap();
    let right_key = InputKey::new("right").unwrap();
    let left = graph
        .input::<F32>(left_key.clone(), Shape::from_dims([2]))
        .unwrap();
    let right = graph
        .input::<F32>(right_key.clone(), Shape::from_dims([2]))
        .unwrap();
    let sum = add(&mut graph, left, right);
    graph.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    let program = graph.build().unwrap();
    let left_tensor = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
    let right_tensor = f32_tensor(Shape::from_dims([2]), vec![3.0, 4.0]);
    let swapped = [
        InputBinding::new(&right_key, &right_tensor),
        InputBinding::new(&left_key, &left_tensor),
    ];
    assert!(matches!(
        evaluate_program(&program, &swapped),
        Err(EvaluationError::InputKey { input_index: 0, .. })
    ));
    assert!(matches!(
        evaluate_program(&program, &[InputBinding::new(&left_key, &left_tensor)]),
        Err(EvaluationError::InputCount { .. })
    ));
    let wrong = f32_tensor(Shape::from_dims([1]), vec![1.0]);
    assert!(matches!(
        evaluate_program(
            &program,
            &[
                InputBinding::new(&left_key, &wrong),
                InputBinding::new(&right_key, &right_tensor)
            ]
        ),
        Err(EvaluationError::InputShape { .. })
    ));
}

#[test]
fn constants_preserve_nan_payloads_but_arithmetic_results_are_canonical() {
    let payload = 0x7fc0_1234;
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let literal = constant_bits(&mut graph, payload);
    let zero = constant(&mut graph, 0.0);
    let arithmetic = add(&mut graph, literal, zero);
    graph
        .output(OutputKey::new("constant").unwrap(), literal)
        .unwrap();
    graph
        .output(OutputKey::new("arithmetic").unwrap(), arithmetic)
        .unwrap();
    let program = graph.build().unwrap();

    let output = evaluate_program(&program, &[]).unwrap();
    assert_eq!(f32_bits(&output[0])[0], payload);
    assert_eq!(f32_bits(&output[1])[0], CANONICAL_F32_ARITHMETIC_NAN_BITS);
}

#[test]
fn f32_arithmetic_preserves_subnormals_and_signed_zero_and_overflows_to_infinity() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let one = constant(&mut graph, 1.0);
    let two = constant(&mut graph, 2.0);
    let half = constant(&mut graph, 0.5);
    let minimum_subnormal = constant_bits(&mut graph, 0x0000_0001);
    let minimum_normal = constant_bits(&mut graph, 0x0080_0000);
    let maximum_finite = constant_bits(&mut graph, 0x7f7f_ffff);
    let negative_zero = constant_bits(&mut graph, 0x8000_0000);
    let positive_infinity = constant_bits(&mut graph, f32::INFINITY.to_bits());
    let negative_infinity = constant_bits(&mut graph, f32::NEG_INFINITY.to_bits());

    let preserved_subnormal = multiply(&mut graph, minimum_subnormal, one);
    let produced_subnormal = multiply(&mut graph, minimum_normal, half);
    let overflow = multiply(&mut graph, maximum_finite, two);
    let signed_zero = multiply(&mut graph, negative_zero, two);
    let invalid_infinities = add(&mut graph, positive_infinity, negative_infinity);

    for (key, value) in [
        ("preserved-subnormal", preserved_subnormal),
        ("produced-subnormal", produced_subnormal),
        ("overflow", overflow),
        ("signed-zero", signed_zero),
        ("invalid-infinities", invalid_infinities),
    ] {
        graph.output(OutputKey::new(key).unwrap(), value).unwrap();
    }
    let outputs = evaluate_program(&graph.build().unwrap(), &[]).unwrap();

    assert_eq!(f32_bits(&outputs[0])[0], 0x0000_0001);
    assert_eq!(f32_bits(&outputs[1])[0], 0x0040_0000);
    assert_eq!(f32_bits(&outputs[2])[0], f32::INFINITY.to_bits());
    assert_eq!(f32_bits(&outputs[3])[0], 0x8000_0000);
    assert_eq!(f32_bits(&outputs[4])[0], CANONICAL_F32_ARITHMETIC_NAN_BITS);
}

#[test]
fn commitment_removes_dead_operations_and_inputs_before_evaluation() {
    let mut graph = SemanticProgramBuilder::try_standard().unwrap();
    let live = constant(&mut graph, 7.0);
    let dead_input = graph
        .input::<F32>(InputKey::new("dead").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let dead = sum(&mut graph, dead_input, [Axis::new(0)]);
    graph.output(OutputKey::new("live").unwrap(), live).unwrap();
    let program = graph.build().unwrap();

    assert!(matches!(
        program.value(dead.erase()),
        Err(tiler_ir::semantic::HandleError::ForeignGraph { .. })
    ));
    assert_eq!(program.input_count(), 0);
    assert_eq!(program.operation_count(), 1);
    let outputs = evaluate_program(&program, &[]).unwrap();
    assert_eq!(f32_values(&outputs[0]), [7.0]);
}

#[test]
fn missing_and_external_reference_capabilities_are_explicit() {
    let mut semantics = SemanticRegistryBuilder::standard().unwrap();
    semantics
        .register_provider(&ExternalSemanticProvider)
        .unwrap();
    let mut graph = SemanticProgramBuilder::try_new(semantics.freeze().unwrap()).unwrap();
    let input: Value<F32> = graph
        .input(InputKey::new("x").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let result = graph
        .apply(
            external_identity_op(),
            OperationAttributes::empty(),
            &[input.erase()],
        )
        .unwrap();
    graph
        .output_resolved(OutputKey::new("result").unwrap(), result[0])
        .unwrap();
    let program = graph.build().unwrap();
    let key = InputKey::new("x").unwrap();
    let tensor = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
    let bindings = [InputBinding::new(&key, &tensor)];

    let error = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&program, &bindings)
        .unwrap_err();
    assert!(matches!(
        error,
        EvaluationError::MissingCapability { operation, .. }
            if operation == external_identity_op()
    ));

    let mut references = reference_builder_for(program.semantic_registry().clone());
    references
        .register_provider(&ExternalReferenceProvider {
            capability_revision: 1,
        })
        .unwrap();
    let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
    assert_eq!(
        evaluator.evaluate(&program, &bindings).unwrap(),
        vec![tensor]
    );
}

#[test]
fn malformed_reference_results_fail_closed() {
    let mut semantics = SemanticRegistryBuilder::standard().unwrap();
    semantics
        .register_provider(&ExternalSemanticProvider)
        .unwrap();
    let mut graph = SemanticProgramBuilder::try_new(semantics.freeze().unwrap()).unwrap();
    let input: Value<F32> = graph
        .input(InputKey::new("x").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let result = graph
        .apply(
            external_identity_op(),
            OperationAttributes::empty(),
            &[input.erase()],
        )
        .unwrap();
    graph
        .output_resolved(OutputKey::new("result").unwrap(), result[0])
        .unwrap();
    let program = graph.build().unwrap();
    let key = InputKey::new("x").unwrap();
    let tensor = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
    let bindings = [InputBinding::new(&key, &tensor)];

    for result in [
        MalformedReferenceResult::CallbackFailure,
        MalformedReferenceResult::WrongArity,
        MalformedReferenceResult::WrongShape,
        MalformedReferenceResult::WrongType,
    ] {
        let mut references = reference_builder_for(program.semantic_registry().clone());
        references
            .register_provider(&MalformedReferenceProvider { result })
            .unwrap();
        let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
        let error = evaluator.evaluate(&program, &bindings).unwrap_err();
        match result {
            MalformedReferenceResult::CallbackFailure => assert!(matches!(
                error,
                EvaluationError::Operation {
                    provider,
                    capability_revision,
                    source: ReferenceOperationError::InvalidApplication,
                    ..
                } if provider.name() == "malformed-reference-capability"
                    && capability_revision.get() == 1
            )),
            MalformedReferenceResult::WrongArity => assert!(matches!(
                error,
                EvaluationError::Operation {
                    provider,
                    capability_revision,
                    source: ReferenceOperationError::ResultCount { .. },
                    ..
                } if provider.name() == "malformed-reference-capability"
                    && capability_revision.get() == 1
            )),
            MalformedReferenceResult::WrongShape => assert!(matches!(
                error,
                EvaluationError::ResultShape {
                    provider,
                    capability_revision,
                    ..
                } if provider.name() == "malformed-reference-capability"
                    && capability_revision.get() == 1
            )),
            MalformedReferenceResult::WrongType => assert!(matches!(
                error,
                EvaluationError::ResultType {
                    provider,
                    capability_revision,
                    ..
                } if provider.name() == "malformed-reference-capability"
                    && capability_revision.get() == 1
            )),
        }
    }
}

#[test]
fn registry_identity_is_deterministic_and_revision_complete() {
    let standard_a = ReferenceRegistryBuilder::standard()
        .unwrap()
        .freeze()
        .unwrap();
    let standard_b = ReferenceRegistryBuilder::standard()
        .unwrap()
        .freeze()
        .unwrap();
    assert_eq!(
        standard_a.canonical_identity(),
        standard_b.canonical_identity()
    );

    let semantic_registry = external_semantics();
    let baseline = reference_builder_for(semantic_registry.clone())
        .freeze()
        .unwrap();
    let with_revision = |capability_revision| {
        let mut builder = reference_builder_for(semantic_registry.clone());
        builder
            .register_provider(&ExternalReferenceProvider {
                capability_revision,
            })
            .unwrap();
        builder.freeze().unwrap()
    };
    let revision_one = with_revision(1);
    let revision_two = with_revision(2);
    assert_ne!(
        revision_one.canonical_identity(),
        baseline.canonical_identity()
    );
    assert_ne!(
        revision_one.canonical_identity(),
        revision_two.canonical_identity()
    );
}

#[test]
fn duplicate_provider_registration_is_transactional() {
    let provider = ExternalReferenceProvider {
        capability_revision: 1,
    };
    let semantic_registry = external_semantics();
    let mut builder = reference_builder_for(semantic_registry.clone());
    builder.register_provider(&provider).unwrap();
    assert!(matches!(
        builder.register_provider(&provider),
        Err(ReferenceRegistryError::DuplicateCapability { operation, .. })
            if operation == external_identity_op()
    ));
    let after_rejection = builder.freeze().unwrap();

    let mut expected = reference_builder_for(semantic_registry);
    expected.register_provider(&provider).unwrap();
    let expected = expected.freeze().unwrap();
    assert_eq!(
        after_rejection.canonical_identity(),
        expected.canonical_identity()
    );
}

#[test]
fn non_f32_nominal_values_use_the_same_exact_tensor_boundary() {
    let mut semantics = SemanticRegistryBuilder::standard().unwrap();
    semantics
        .register_provider(&ExternalU8SemanticProvider)
        .unwrap();
    let semantics = semantics.freeze().unwrap();
    let mut graph = SemanticProgramBuilder::try_new(semantics.clone()).unwrap();
    let input = graph
        .input_resolved(
            InputKey::new("bytes").unwrap(),
            Shape::from_dims([3]),
            external_u8_type(),
        )
        .unwrap();
    graph
        .output_resolved(OutputKey::new("bytes").unwrap(), input)
        .unwrap();
    let program = graph.build().unwrap();
    let tensor = Tensor::dense(
        external_u8_type(),
        Shape::from_dims([3]),
        [1_u8, 2, 255]
            .map(|value| ReferenceElement::new([value]).unwrap())
            .into(),
    )
    .unwrap();
    let mut references = reference_builder_for(semantics);
    references
        .register_provider(&ExternalU8ReferenceProvider)
        .unwrap();
    let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
    let key = InputKey::new("bytes").unwrap();
    assert_eq!(
        evaluator
            .evaluate(&program, &[InputBinding::new(&key, &tensor)])
            .unwrap(),
        [tensor]
    );
}

#[test]
fn value_validator_failure_retains_exact_implementation_attribution() {
    let mut semantics = SemanticRegistryBuilder::standard().unwrap();
    semantics
        .register_provider(&ExternalU8SemanticProvider)
        .unwrap();
    let semantics = semantics.freeze().unwrap();
    let mut graph = SemanticProgramBuilder::try_new(semantics.clone()).unwrap();
    let input = graph
        .input_resolved(
            InputKey::new("bytes").unwrap(),
            Shape::from_dims([1]),
            external_u8_type(),
        )
        .unwrap();
    graph
        .output_resolved(OutputKey::new("bytes").unwrap(), input)
        .unwrap();
    let program = graph.build().unwrap();
    let invalid = Tensor::dense(
        external_u8_type(),
        Shape::from_dims([1]),
        vec![ReferenceElement::new([1, 2]).unwrap()],
    )
    .unwrap();
    let mut references = reference_builder_for(semantics);
    references
        .register_provider(&ExternalU8ReferenceProvider)
        .unwrap();
    let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
    let key = InputKey::new("bytes").unwrap();
    assert!(matches!(
        evaluator.evaluate(&program, &[InputBinding::new(&key, &invalid)]),
        Err(EvaluationError::Value {
            provider,
            capability_revision,
            source: ReferenceValueError::InvalidRepresentation,
            ..
        }) if provider.name() == "u8-reference" && capability_revision.get() == 1
    ));
}

#[test]
fn capability_authority_rejects_changed_meaning_but_not_unrelated_snapshot_entries() {
    let baseline_semantics = external_semantics();
    let mut references = reference_builder_for(baseline_semantics.clone());
    references
        .register_provider(&ExternalReferenceProvider {
            capability_revision: 1,
        })
        .unwrap();
    let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
    let input = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
    let key = InputKey::new("x").unwrap();

    let mut changed = SemanticRegistryBuilder::standard().unwrap();
    changed
        .register_provider(&ChangedExternalSemanticProvider)
        .unwrap();
    let changed_program = external_identity_program(changed.freeze().unwrap());
    assert!(matches!(
        evaluator.evaluate(&changed_program, &[InputBinding::new(&key, &input)]),
        Err(EvaluationError::CapabilityAuthorityMismatch { operation, .. })
            if operation == external_identity_op()
    ));

    let mut extended = SemanticRegistryBuilder::standard().unwrap();
    extended
        .register_provider(&ExternalSemanticProvider)
        .unwrap();
    extended
        .register_provider(&ExternalU8SemanticProvider)
        .unwrap();
    let extended_program = external_identity_program(extended.freeze().unwrap());
    assert_eq!(
        evaluator
            .evaluate(&extended_program, &[InputBinding::new(&key, &input)])
            .unwrap(),
        [input]
    );
}

#[test]
fn occurrence_authority_follows_attribute_types_and_admission_providers() {
    let baseline_semantics = attributed_semantics(1, 1);
    let mut references = reference_builder_for(baseline_semantics.clone());
    references
        .register_provider(&AttributedIdentityReferenceProvider)
        .unwrap();
    let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
    let input = f32_tensor(Shape::from_dims([2]), vec![1.0, 2.0]);
    let key = InputKey::new("x").unwrap();

    for changed in [attributed_semantics(1, 2), attributed_semantics(2, 1)] {
        let changed = attributed_program(changed);
        assert!(matches!(
            evaluator.evaluate(&changed, &[InputBinding::new(&key, &input)]),
            Err(EvaluationError::CapabilityAuthorityMismatch {
                operation,
                provider,
                capability_revision,
            }) if operation == attributed_identity_op()
                && provider.name() == "attributed-reference-capability"
                && capability_revision.get() == 7
        ));
    }

    let mut extended = SemanticRegistryBuilder::standard().unwrap();
    extended
        .register_provider(&AttributeTypeProvider {
            provider_revision: 1,
            definition_revision: 1,
        })
        .unwrap();
    extended
        .register_provider(&AttributedIdentitySemanticProvider)
        .unwrap();
    extended
        .register_provider(&ExternalU8SemanticProvider)
        .unwrap();
    let extended = attributed_program(extended.freeze().unwrap());
    assert_eq!(
        evaluator
            .evaluate(&extended, &[InputBinding::new(&key, &input)])
            .unwrap(),
        [input]
    );
}

#[test]
fn ignored_registration_failure_poisoned_the_provider_batch() {
    let mut builder = reference_builder_for(external_semantics());
    assert!(matches!(
        builder.register_provider(&IgnoredDuplicateReferenceProvider),
        Err(ReferenceRegistryError::DuplicateCapability { operation, .. })
            if operation == external_identity_op()
    ));
    builder
        .register_provider(&ExternalReferenceProvider {
            capability_revision: 1,
        })
        .unwrap();
}

#[test]
fn exact_tensor_equality_distinguishes_nan_payloads_and_signed_zero() {
    let tensor = |bits| {
        Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([1]),
            vec![
                ReferenceElement::from_float_bits(
                    u32::to_be_bytes(bits),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    };
    assert_eq!(tensor(0x7fc0_1234), tensor(0x7fc0_1234));
    assert_ne!(tensor(0x7fc0_1234), tensor(0x7fc0_5678));
    assert_ne!(tensor(0x0000_0000), tensor(0x8000_0000));
}

#[test]
fn float_bit_order_is_explicit_and_normalizes_to_canonical_bytes() {
    let canonical = ReferenceElement::from_float_bits(
        [0x3f, 0x80, 0x00, 0x00],
        FloatBitOrder::MostSignificantByteFirst,
    )
    .unwrap();
    let little = ReferenceElement::from_float_bits(
        [0x00, 0x00, 0x80, 0x3f],
        FloatBitOrder::LeastSignificantByteFirst,
    )
    .unwrap();
    assert_eq!(canonical, little);
    assert_eq!(canonical.as_bytes(), [0x3f, 0x80, 0x00, 0x00]);
    assert_eq!(
        ReferenceElement::from_float_bits([], FloatBitOrder::MostSignificantByteFirst),
        Err(EvaluationError::EmptyFloatBits)
    );
    let oversized = vec![0_u8; MAX_REFERENCE_ELEMENT_BYTES + 1];
    assert!(matches!(
        ReferenceElement::from_float_bits(
            &oversized,
            FloatBitOrder::MostSignificantByteFirst,
        ),
        Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::ElementBytes,
            limit: MAX_REFERENCE_ELEMENT_BYTES,
            actual,
        }) if actual == MAX_REFERENCE_ELEMENT_BYTES + 1
    ));
}

#[test]
fn compound_values_preserve_stable_role_tensors_without_any_downcasts() {
    let codes = Tensor::dense(
        external_u8_type(),
        Shape::from_dims([2]),
        vec![
            ReferenceElement::new([1]).unwrap(),
            ReferenceElement::new([2]).unwrap(),
        ],
    )
    .unwrap();
    let scale = f32_tensor(Shape::new([]), vec![0.5]);
    let compound_type =
        ResolvedValueType::nominal(TypeKey::new("test", "compound-quantized", 1).unwrap());
    let value = Tensor::compound(
        compound_type,
        Shape::from_dims([2]),
        vec![
            ReferenceComponent::new(ReferenceComponentRole::new(1), codes),
            ReferenceComponent::new(ReferenceComponentRole::new(2), scale),
        ],
    )
    .unwrap();
    let TensorPayloadView::Compound(components) = value.payload() else {
        panic!("expected compound payload")
    };
    assert_eq!(components[0].role(), ReferenceComponentRole::new(1));
    assert_eq!(components[1].tensor().shape(), &Shape::new([]));

    let duplicate = ReferenceComponent::new(
        ReferenceComponentRole::new(1),
        f32_tensor(Shape::new([]), vec![1.0]),
    );
    assert!(matches!(
        Tensor::compound(
            ResolvedValueType::nominal(
                TypeKey::new("test", "compound-quantized", 1).unwrap()
            ),
            Shape::from_dims([2]),
            vec![components[0].clone(), duplicate],
        ),
        Err(EvaluationError::DuplicateComponentRole { role })
            if role == ReferenceComponentRole::new(1)
    ));
}

#[test]
fn compound_and_evaluation_resources_are_bounded_in_aggregate() {
    let compound_type = compound_limit_type();
    let components = |start: u32| {
        (0..(MAX_REFERENCE_COMPONENTS / 2))
            .map(|offset| {
                ReferenceComponent::new(
                    ReferenceComponentRole::new(start + u32::try_from(offset).unwrap()),
                    Tensor::dense(external_u8_type(), Shape::from_dims([0]), Vec::new()).unwrap(),
                )
            })
            .collect()
    };
    let left = Tensor::compound(compound_type.clone(), Shape::new([]), components(0)).unwrap();
    let right = Tensor::compound(
        compound_type.clone(),
        Shape::new([]),
        components(u32::try_from(MAX_REFERENCE_COMPONENTS / 2).unwrap()),
    )
    .unwrap();
    assert!(matches!(
        Tensor::compound(
            compound_type.clone(),
            Shape::new([]),
            vec![
                ReferenceComponent::new(ReferenceComponentRole::new(1), left),
                ReferenceComponent::new(ReferenceComponentRole::new(2), right),
            ],
        ),
        Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::Components,
            limit: MAX_REFERENCE_COMPONENTS,
            actual,
        }) if actual == MAX_REFERENCE_COMPONENTS + 2
    ));

    let at_limit = Tensor::compound(
        compound_type.clone(),
        Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
        Vec::new(),
    )
    .unwrap();
    let one_more = Tensor::compound(compound_type, Shape::from_dims([1]), Vec::new()).unwrap();
    let mut retained = EvaluationRetention::default();
    reserve_evaluation_work(&mut retained, &at_limit).unwrap();
    assert!(matches!(
        reserve_evaluation_work(&mut retained, &one_more),
        Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::EvaluationElements,
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual,
        }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
    ));

    let mut outputs = ReferenceOutputs::new(
        1,
        EvaluationRetention {
            work: ReferenceWork {
                elements: MAX_REFERENCE_TENSOR_ELEMENTS,
                ..ReferenceWork::default()
            },
            ..EvaluationRetention::default()
        },
    );
    assert!(matches!(
        outputs.push(one_more),
        Err(ReferenceOperationError::OutputElementsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual,
        }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
    ));
    assert!(outputs.values.is_empty());
}

#[test]
fn compound_root_elements_participate_in_the_aggregate_bound() {
    let compound_type = compound_limit_type();
    assert!(matches!(
        Tensor::compound(
            compound_type.clone(),
            Shape::from_dims([
                u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap() + 1
            ]),
            Vec::new(),
        ),
        Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::TensorElements,
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual,
        }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
    ));
    let scalar_child = Tensor::dense(
        external_u8_type(),
        Shape::new([]),
        vec![ReferenceElement::new([1]).unwrap()],
    )
    .unwrap();
    assert!(matches!(
        Tensor::compound(
            compound_type.clone(),
            Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
            vec![ReferenceComponent::new(
                ReferenceComponentRole::new(1),
                scalar_child.clone(),
            )],
        ),
        Err(EvaluationError::ResourceExceeded {
            resource: ReferenceResource::TensorElements,
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual,
        }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
    ));
    Tensor::compound(
        compound_type,
        Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS - 1).unwrap()]),
        vec![ReferenceComponent::new(
            ReferenceComponentRole::new(1),
            scalar_child,
        )],
    )
    .unwrap();
}

#[test]
fn repeated_outputs_share_one_governed_tensor_allocation() {
    let mut semantics = SemanticRegistryBuilder::standard().unwrap();
    semantics
        .register_provider(&CompoundLimitSemanticProvider)
        .unwrap();
    let semantics = semantics.freeze().unwrap();
    let mut graph = SemanticProgramBuilder::try_new(semantics.clone()).unwrap();
    let input = graph
        .input_resolved(
            InputKey::new("value").unwrap(),
            Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
            compound_limit_type(),
        )
        .unwrap();
    graph
        .output_resolved(OutputKey::new("first").unwrap(), input)
        .unwrap();
    graph
        .output_resolved(OutputKey::new("second").unwrap(), input)
        .unwrap();
    let program = graph.build().unwrap();
    let input = Tensor::compound(
        compound_limit_type(),
        Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap()]),
        Vec::new(),
    )
    .unwrap();
    let mut references = reference_builder_for(semantics);
    references
        .register_provider(&CompoundLimitReferenceProvider)
        .unwrap();
    let evaluator = ReferenceEvaluator::new(references.freeze().unwrap());
    let key = InputKey::new("value").unwrap();
    let outputs = evaluator
        .evaluate(&program, &[InputBinding::new(&key, &input)])
        .unwrap();
    assert_eq!(outputs, [input.clone(), input.clone()]);
    assert_eq!(outputs[0].storage_id(), outputs[1].storage_id());
    assert_eq!(outputs[0].storage_id(), input.storage_id());
}

#[test]
fn registry_identity_budget_is_exact_at_boundary() {
    let builder = ReferenceRegistryBuilder::standard().unwrap();
    let exact_len = builder.canonical_bytes;
    let frozen = builder.freeze().unwrap();
    assert_eq!(frozen.canonical_identity().as_bytes().len(), exact_len);

    let semantics = FrozenSemanticRegistry::standard().unwrap();
    let provider = ProviderIdentity::new("test", "budget-boundary", 1).unwrap();
    for (existing, added, succeeds) in [
        (MAX_REFERENCE_REGISTRY_IDENTITY_BYTES, 0, true),
        (MAX_REFERENCE_REGISTRY_IDENTITY_BYTES, 1, false),
    ] {
        let mut batch = ReferenceRegistrationBatch::default();
        let mut registrar = ReferenceRegistryRegistrar {
            batch: &mut batch,
            semantic_registry: &semantics,
            provider: &provider,
            existing_capabilities: 0,
            existing_canonical_bytes: existing,
        };
        assert_eq!(registrar.reserve_canonical_bytes(added).is_ok(), succeeds);
    }
}

#[test]
fn late_zero_shapes_are_accepted_before_overflow_prone_work() {
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([0, u64::MAX, 2]),
        Vec::new(),
    )
    .unwrap();
    assert!(matches!(tensor.payload(), TensorPayloadView::Dense([])));
    assert_eq!(row_major_strides(tensor.shape()).unwrap(), [0, 0, 0]);

    let output = strict_sum(&tensor, &[Axis::new(1), Axis::new(2)]).unwrap();
    assert_eq!(output.shape(), &Shape::from_dims([0]));
    assert!(matches!(output.payload(), TensorPayloadView::Dense([])));
}

#[test]
fn empty_contributor_reduction_preflights_oversized_survivor() {
    let input = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([u64::try_from(MAX_REFERENCE_TENSOR_ELEMENTS).unwrap() + 1, 0]),
        Vec::new(),
    )
    .unwrap();
    assert!(matches!(
        strict_sum(&input, &[Axis::new(1)]),
        Err(ReferenceOperationError::OutputElementsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual,
        }) if actual == MAX_REFERENCE_TENSOR_ELEMENTS + 1
    ));
}

#[test]
fn large_empty_contributor_domains_are_iterated_without_coordinate_materialization() {
    let input = f32_tensor(Shape::from_dims([100_000, 0]), Vec::new());
    let output = strict_sum(&input, &[Axis::new(1)]).unwrap();
    assert_eq!(output.shape(), &Shape::from_dims([100_000]));
    assert!(
        f32_bits(&output)
            .into_iter()
            .all(|bits| bits == 0.0_f32.to_bits())
    );
}

#[test]
fn maximum_rank_reduction_classifies_many_axes_linearly() {
    let rank = 4_096_usize;
    let input = f32_tensor(
        Shape::try_from_dims(std::iter::repeat_n(1, rank)).unwrap(),
        vec![1.0],
    );
    let axes: Vec<_> = (0..rank)
        .step_by(2)
        .map(|axis| Axis::new(u32::try_from(axis).unwrap()))
        .collect();
    let output = strict_sum(&input, &axes).unwrap();
    assert_eq!(output.shape().rank(), rank / 2);
    assert_eq!(f32_values(&output), [1.0]);
    assert_eq!(
        strict_sum(&input, &[Axis::new(0), Axis::new(0)]),
        Err(ReferenceOperationError::InvalidApplication)
    );
    assert_eq!(
        strict_sum(&input, &[Axis::new(u32::try_from(rank).unwrap())]),
        Err(ReferenceOperationError::InvalidApplication)
    );
}

/// A one-partition split is bit-identical to the serial fold it replaces.
///
/// The split's whole freedom is *where* the sequence is cut, so a cut that
/// takes the whole sequence must reproduce the strict reading exactly. If it
/// did not, every later comparison would be measuring the harness rather than
/// the split.
#[test]
fn a_single_partition_split_reproduces_the_serial_reduction_exactly() {
    for (shape, values, contributors) in [
        // A domain with nothing to reduce.
        (Shape::from_dims([2, 0]), Vec::new(), 0),
        // A single contributor per output.
        (Shape::from_dims([3, 1]), vec![-0.0, 1.5, f32::INFINITY], 1),
        // A genuine fold, including a negative zero the identity must not eat.
        (
            Shape::from_dims([2, 3]),
            vec![-0.0, -0.0, -0.0, 1.0, 2.0, 3.0],
            3,
        ),
    ] {
        let input = f32_tensor(shape, values);
        let axes = [Axis::new(1)];
        let serial = strict_sum(&input, &axes).unwrap();
        let split = strict_partitioned_sum(&input, &axes, 1, contributors).unwrap();
        assert_eq!(
            f32_bits(&split),
            f32_bits(&serial),
            "a one-partition split must reproduce the strict reading bit for bit"
        );
    }
}

/// An exact split folds every contributor exactly once, in its declared order.
///
/// The expected values are written out rather than derived, so this states what
/// the split *means* instead of re-running the implementation: partition `p`
/// takes contributors `2p` and `2p + 1`, and the final pass adds the three
/// partial sums left to right.
#[test]
fn an_exact_split_folds_each_contributor_exactly_once() {
    let input = f32_tensor(
        Shape::from_dims([2, 6]),
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, //
            10.0, 20.0, 30.0, 40.0, 50.0, 60.0,
        ],
    );
    let axes = [Axis::new(1)];
    let partials = strict_partial_sums(&input, &axes, 3, 2).unwrap();
    assert_eq!(*partials.shape(), Shape::from_dims([2, 3]));
    assert_eq!(
        f32_values(&partials),
        vec![3.0, 7.0, 11.0, 30.0, 70.0, 110.0]
    );
    let split = strict_partitioned_sum(&input, &axes, 3, 2).unwrap();
    assert_eq!(f32_values(&split), vec![21.0, 210.0]);
    // Exact arithmetic here, so the split and the serial fold agree; the point
    // is coverage, and a dropped or repeated contributor would change the sum.
    assert_eq!(
        f32_bits(&split),
        f32_bits(&strict_sum(&input, &axes).unwrap())
    );
}

/// The split is a reassociation, and the oracle answers for the chosen order.
///
/// These magnitudes make the strict left fold and the split disagree in `f32`.
/// A single oracle answering "the" value for a reassociation-permitting
/// contract would therefore have to be wrong for one of them, which is why the
/// split has its own evaluator rather than a relaxed comparison against the
/// serial one.
#[test]
fn a_split_selects_one_legal_order_the_serial_fold_does_not_produce() {
    let large = 1.0e8_f32;
    let input = f32_tensor(Shape::from_dims([1, 4]), vec![large, 1.0, -large, 1.0]);
    let axes = [Axis::new(1)];
    // (((1e8 + 1) - 1e8) + 1): the middle term is lost to rounding.
    let serial = f32_values(&strict_sum(&input, &axes).unwrap());
    // ((1e8 + 1) + (-1e8 + 1)): both ones are lost instead.
    let split = f32_values(&strict_partitioned_sum(&input, &axes, 2, 2).unwrap());
    assert_eq!(serial, vec![1.0]);
    assert_eq!(split, vec![0.0]);
}

/// A split that does not cover the contributor sequence exactly is refused.
#[test]
fn an_inexact_split_is_refused_by_the_reference() {
    let input = f32_tensor(Shape::from_dims([2, 6]), vec![1.0; 12]);
    let axes = [Axis::new(1)];
    for (partitions, per_partition) in [(4, 2), (5, 1), (0, 6), (3, 3)] {
        assert_eq!(
            strict_partial_sums(&input, &axes, partitions, per_partition),
            Err(ReferenceOperationError::InvalidApplication),
            "{partitions} x {per_partition} does not cover six contributors"
        );
    }
    // The exact split of the same tensor is admitted, so the refusals above are
    // about coverage rather than about the fixture.
    assert!(strict_partial_sums(&input, &axes, 3, 2).is_ok());
}

/// An empty reduction commits the identity once per partition, and sums to it.
#[test]
fn an_empty_split_commits_the_identity_in_every_partition() {
    let input = f32_tensor(Shape::from_dims([2, 0]), Vec::new());
    let axes = [Axis::new(1)];
    let partials = strict_partial_sums(&input, &axes, 4, 0).unwrap();
    assert_eq!(*partials.shape(), Shape::from_dims([2, 4]));
    assert_eq!(f32_bits(&partials), vec![0.0_f32.to_bits(); 8]);
    assert_eq!(
        f32_bits(&strict_partitioned_sum(&input, &axes, 4, 0).unwrap()),
        vec![0.0_f32.to_bits(); 2]
    );
}
