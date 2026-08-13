use std::sync::Arc;
use tiler_ir::semantic::{
    CanonicalValue, F32, FrozenSemanticRegistry, NormativeDefinitionRef, OpKey,
    OperationAttributes, OperationConformance, OperationDefinition, OperationDefinitionFacts,
    OperationEffect, OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, OperationArity, ProviderDiagnosticCode, ProviderIdentity,
    SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar, ValueFact,
};
use tiler_ir::shape::Shape;

struct Echo;

impl OperationInferencer for Echo {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        outputs.try_push(request.operands()[0].clone())
    }
}

struct EchoProvider;

impl SemanticRegistryProvider for EchoProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "echo", 1).unwrap()
    }

    fn register(
        &self,
        registrar: &mut SemanticRegistryRegistrar<'_>,
    ) -> Result<(), tiler_ir::semantic::RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            OpKey::new("test", "echo", 1).unwrap(),
            OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), []).unwrap(),
            NormativeDefinitionRef::new("test echo")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(Echo),
        ))
    }
}

fn main() {
    let mut builder = SemanticRegistryBuilder::standard().unwrap();
    builder.register_provider(&EchoProvider).unwrap();
    let registry = builder.freeze().unwrap();
    let operand = ValueFact::new(F32::resolved_type(), Shape::from_dims([2]));
    let results = registry
        .infer_operation(
            &OpKey::new("test", "echo", 1).unwrap(),
            &[operand],
            &OperationAttributes::empty(),
        )
        .unwrap();
    assert_eq!(results.len(), 1);
    let _ = FrozenSemanticRegistry::standard();
    let _ = ProviderDiagnosticCode::new("test.ok");
}
