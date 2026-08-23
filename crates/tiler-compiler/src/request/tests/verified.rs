use super::super::{
    AccessOrdinal, Axis, CanonicalIntegerWidth, CanonicalValueView, CompilationRequest,
    CompilerCapabilitySnapshot, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, NormalizedProgram,
    OpKey, OutputKey, PointwiseF32Expression, PointwiseF32ExpressionBuilder, ProviderIdentity,
    REDUCTION_AXES_ATTRIBUTE, RequestError, SemanticProgram, SerialSumContributor, Shape,
    TargetProfile, TypeKey, add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
    verify_planned_request,
};
use super::support::{UnusedSemantics, program, program_with_builder};
use std::sync::Arc;
use tiler_ir::semantic::{
    CanonicalValue, CanonicalValueKind, NormativeDefinitionRef, OperationArity,
    OperationAttributeSchema, OperationConformance, OperationDefinition, OperationDefinitionFacts,
    OperationEffect, OperationInferenceError, OperationInferencer, OperationSchema,
    ProviderDiagnosticCode, RegistryError, SemanticProgramBuilder, SemanticRegistryBuilder,
    SemanticRegistryProvider, SemanticRegistryRegistrar, TypeDefinitionFacts, ValueFact,
    ValueTypeDefinition, ValueTypeDefinitionKey,
};

fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
    ProviderDiagnosticCode::new(value).unwrap()
}

/// Builds the five-node `input * scale + bias` expression a forgery swaps in.
/// Replaces one recognized fold's prologue expression, leaving its reads alone.
///
/// The mutation is the *subject's*, which is what makes the receipt check that
/// follows a perturbation rather than an assertion edit: the recomputed request
/// subject carries the forged expression whole, and no other fact moves.
fn forge_prologue(normalized: &mut NormalizedProgram, expression: PointwiseF32Expression) {
    let SerialSumContributor::PointwisePrologue {
        expression: recognized,
        ..
    } = &mut normalized.serial_sum_mut().contributor
    else {
        panic!("the fixture folds a pointwise prologue over declared inputs");
    };
    *recognized = expression;
}

fn affine_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(scale_bits).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bias_bits).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

#[derive(Clone, Copy)]
enum TestOperation {
    Constant,
    Binary,
    Sum,
}

impl OperationInferencer for TestOperation {
    fn infer(
        &self,
        request: tiler_ir::semantic::OperationInferenceRequest<'_>,
        outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let attributes = request.attributes();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = request.static_operand_shape(0)?;
                let right = request.static_operand_shape(1)?;
                let shape = if left.rank() == 0 {
                    right.clone()
                } else if right.rank() == 0 || left == right {
                    left.clone()
                } else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.binary.shape"),
                        "operands must have equal shapes or include one scalar",
                    )
                    .unwrap());
                };
                outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
            }
            Self::Sum => {
                let Some(CanonicalValueView::Sequence(values)) = attributes
                    .get(REDUCTION_AXES_ATTRIBUTE)
                    .map(CanonicalValue::view)
                else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.sum.axes"),
                        "sum axes must be a sequence",
                    )
                    .unwrap());
                };
                let axes = values
                    .iter()
                    .map(|value| match value.view() {
                        CanonicalValueView::Unsigned {
                            width: CanonicalIntegerWidth::Bits32,
                            bits,
                        } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                            OperationInferenceError::new(
                                diagnostic_code("test.sum.axis-width"),
                                "sum axis exceeds u32",
                            )
                            .unwrap()
                        }),
                        _ => Err(OperationInferenceError::new(
                            diagnostic_code("test.sum.axis-kind"),
                            "sum axes must be u32 values",
                        )
                        .unwrap()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                outputs.try_push(ValueFact::new(
                    F32::resolved_type(),
                    request.static_operand_shape(0)?.without_axes(&axes),
                ))
            }
        }
    }
}

struct GovernedTestSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for GovernedTestSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(
                    TypeKey::new("tiler", "f32", 1).expect("the test F32 key is valid"),
                ),
                NormativeDefinitionRef::new("test binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register_test_operation(
            registrar,
            constant_f32_op(),
            0,
            [OperationAttributeSchema::required(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
            TestOperation::Constant,
        )?;
        register_test_operation(registrar, multiply_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(registrar, add_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            [OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            TestOperation::Sum,
        )
    }
}

fn register_test_operation<const N: usize>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: [OperationAttributeSchema; N],
    inferencer: TestOperation,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key,
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            attributes,
        )
        .expect("the test operation schema is valid"),
        NormativeDefinitionRef::new("test governed operation semantics")?,
        OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
        OperationConformance::new(CanonicalValue::boolean(true)),
        OperationEffect::Pure,
        Arc::new(inferencer),
    ))
}

fn governed_test_program(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&GovernedTestSemantics { revision })
        .unwrap();
    program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

fn program_with_unused_provider(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision })
        .unwrap();
    program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

fn request_with_matching_empty_capabilities(program: &SemanticProgram) -> CompilationRequest<'_> {
    let scalars =
        tiler_ir::index::ScalarRegistryBuilder::new(program.semantic_registry().clone()).freeze();
    let lowering = crate::capability::LoweringCapabilityRegistryBuilder::new(
        program.semantic_registry().clone(),
        scalars.clone(),
    )
    .unwrap()
    .freeze();
    let mut request = CompilationRequest::governed(program);
    request.capabilities = CompilerCapabilitySnapshot::new(lowering, scalars);
    request
}

#[test]
fn request_requires_a_nonempty_unique_target_set() {
    let program = program();
    let mut empty = CompilationRequest::governed(&program);
    empty.target_profiles.clear();
    assert_eq!(
        verify_planned_request(empty),
        Err(RequestError::EmptyTargetSet)
    );

    let mut duplicate = CompilationRequest::governed(&program);
    duplicate.target_profiles.push(TargetProfile::governed());
    assert_eq!(
        verify_planned_request(duplicate),
        Err(RequestError::DuplicateTargetProfile)
    );
}

#[test]
fn verified_request_receipts_reject_post_verification_mutation() {
    let program = program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let mut forged = verified.clone();
    forged.budgets.buffers += 1;
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified.clone();
    forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified.clone();
    forged.target_slots[0].target_profile =
        TargetProfile::governed_without_numerical_declarations();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified.clone();
    forged.semantic_identity = program_with_unused_provider(7).semantic_identity().clone();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    // The recognized prologue's scale changed. It is the mutation that used
    // to be a `scale_bits` edit: the subject now carries the whole
    // expression, so a forged prologue is a forged expression.
    let mut forged = verified.clone();
    forge_prologue(
        &mut forged.normalized,
        affine_expression(3.0_f32.to_bits(), 1.0_f32.to_bits()),
    );
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );

    let mut forged = verified;
    forged.normalized.serial_sum_mut().output_key = OutputKey::new("forged").unwrap();
    assert_eq!(
        forged.for_target(0),
        Err(RequestError::UnverifiedTargetSelection)
    );
}

#[test]
fn verified_target_receipt_detects_every_governed_subject_mutation_class() {
    let program = program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let target = verified.for_target(0).unwrap();

    let mut forged = target.clone();
    forged.target_profile = TargetProfile::governed_without_numerical_declarations();
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target.clone();
    forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target.clone();
    forged.budgets.regions += 1;
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target.clone();
    forged.semantic_identity = program_with_unused_provider(11).semantic_identity().clone();
    assert!(!forged.reconstructs_its_authority());

    // One constant of the recognized prologue flipped. The expression is
    // rebuilt rather than edited in place, because it is opaque by
    // construction — which is exactly what makes the subject bind it whole.
    let mut forged = target.clone();
    forge_prologue(
        &mut forged.normalized,
        affine_expression(2.0_f32.to_bits(), 1.0_f32.to_bits() ^ 1),
    );
    assert!(!forged.reconstructs_its_authority());

    let mut forged = target;
    forged.normalized.serial_sum_mut().input_keys = vec![InputKey::new("forged").unwrap()];
    assert!(!forged.reconstructs_its_authority());
}

#[test]
fn used_provider_revision_changes_admission_and_snapshot_subjects() {
    let first = governed_test_program(1);
    let second = governed_test_program(2);
    let first = verify_planned_request(request_with_matching_empty_capabilities(&first)).unwrap();
    let second = verify_planned_request(request_with_matching_empty_capabilities(&second)).unwrap();

    assert_eq!(
        first.semantic_identity.graph(),
        second.semantic_identity.graph()
    );
    assert_eq!(
        first.semantic_identity.reached_definitions(),
        second.semantic_identity.reached_definitions()
    );
    assert_ne!(
        first.semantic_identity.admission_provenance(),
        second.semantic_identity.admission_provenance()
    );
    assert_ne!(
        first.semantic_identity.registry_snapshot(),
        second.semantic_identity.registry_snapshot()
    );
}

#[test]
fn unused_provider_revision_changes_only_the_snapshot_subject() {
    let first = program_with_unused_provider(1);
    let second = program_with_unused_provider(2);
    let first = verify_planned_request(request_with_matching_empty_capabilities(&first)).unwrap();
    let second = verify_planned_request(request_with_matching_empty_capabilities(&second)).unwrap();

    assert_eq!(
        first.semantic_identity.graph(),
        second.semantic_identity.graph()
    );
    assert_eq!(
        first.semantic_identity.reached_definitions(),
        second.semantic_identity.reached_definitions()
    );
    assert_eq!(
        first.semantic_identity.admission_provenance(),
        second.semantic_identity.admission_provenance()
    );
    assert_ne!(
        first.semantic_identity.registry_snapshot(),
        second.semantic_identity.registry_snapshot()
    );
}
