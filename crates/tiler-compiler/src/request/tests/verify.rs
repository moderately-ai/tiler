use super::super::{
    BudgetResource, CompilationRequest, F32, InputKey, OperationAttributes, OutputKey,
    RequestError, ResolvedValueType, Shape, TypeKey, add_f32_op, canonical_program_value_types,
    verify_planned_request,
};
use super::support::{UnusedSemantics, program};
use tiler_ir::semantic::{F32Add, F32Constant, SemanticProgramBuilder, SemanticRegistryBuilder};

#[test]
fn invalid_pointwise_arity_shape_and_dtype_fail_at_semantic_admission() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let tensor = builder
        .input::<F32>(InputKey::new("tensor").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    assert!(
        builder
            .apply(
                add_f32_op(),
                OperationAttributes::empty(),
                &[tensor.erase()],
            )
            .is_err(),
        "the semantic schema refuses invalid builtin arity before normalization",
    );

    let other_shape = builder
        .input::<F32>(InputKey::new("other").unwrap(), Shape::from_dims([3, 2]))
        .unwrap();
    assert!(
        F32Add::apply(&mut builder, tensor, other_shape).is_err(),
        "the semantic inferencer refuses incompatible shapes before normalization",
    );

    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision: 1 })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let foreign = builder
        .input_resolved(
            InputKey::new("foreign").unwrap(),
            Shape::from_dims([2, 3]),
            ResolvedValueType::nominal(TypeKey::new("tiler-test", "unused", 1).unwrap()),
        )
        .unwrap();
    let scalar = F32Constant::apply(&mut builder, 1.0_f32.to_bits())
        .unwrap()
        .erase();
    assert!(
        builder
            .apply(
                add_f32_op(),
                OperationAttributes::empty(),
                &[foreign, scalar],
            )
            .is_err(),
        "the semantic authority refuses a non-f32 builtin operand before normalization",
    );
}

#[test]
fn program_dispatch_types_are_exact_canonical_and_unique() {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision: 1 })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let f32 = builder
        .input::<F32>(InputKey::new("f32").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scalar = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let foreign_type = ResolvedValueType::nominal(TypeKey::new("tiler-test", "unused", 1).unwrap());
    let foreign = builder
        .input_resolved(
            InputKey::new("foreign").unwrap(),
            Shape::from_dims([2, 3]),
            foreign_type.clone(),
        )
        .unwrap();
    builder
        .output(OutputKey::new("f32-output").unwrap(), f32)
        .unwrap();
    builder
        .output(OutputKey::new("scalar-output").unwrap(), scalar)
        .unwrap();
    builder
        .output_resolved(OutputKey::new("foreign-output").unwrap(), foreign)
        .unwrap();
    let program = builder.build().unwrap();

    let actual = canonical_program_value_types(&program);
    assert_eq!(actual.len(), 2, "repeated F32 values are deduplicated");
    assert!(actual.contains(&F32::resolved_type()));
    assert!(actual.contains(&foreign_type));
    assert!(actual.windows(2).all(|pair| {
        pair[0].canonical_encoding().as_bytes() < pair[1].canonical_encoding().as_bytes()
    }));
}

#[test]
fn request_rejects_profile_and_budget_mismatches_stably() {
    let program = program();
    let mut request = CompilationRequest::governed(&program);
    request.budgets.semantic_operations = 4;
    assert_eq!(
        verify_planned_request(request),
        Err(RequestError::BudgetExceeded {
            resource: BudgetResource::SemanticOperations,
            limit: 4,
            reported: 5,
        })
    );

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), input)
        .unwrap();
    let invalid = builder.build().unwrap();
    assert_eq!(
        verify_planned_request(CompilationRequest::governed(&invalid)),
        Err(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "operation-set",
        })
    );
}
