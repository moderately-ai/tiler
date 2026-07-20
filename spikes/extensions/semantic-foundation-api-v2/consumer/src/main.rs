use semantic_api_external::{
    ADD, ExternalProvider, F32, StandardProvider, add, register_reference,
};
use semantic_api_ir::*;
use semantic_api_reference::ReferenceRegistryBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = SemanticRegistryBuilder::default();
    registry.register_provider(&StandardProvider)?;
    registry.register_provider(&ExternalProvider)?;
    let registry = registry.freeze()?;

    let complex_f32 = ResolvedValueType::Parameterized {
        constructor: TypeKey("tiler::complex@1"),
        arguments: TypeArguments(vec![CanonicalValue::Type(ResolvedValueType::Nominal(
            TypeKey("tiler::f32@1"),
        ))]),
    };
    registry.validate_type(&complex_f32)?;

    let encoded_i8 = ResolvedValueType::EncodedNumeric {
        scheme: QuantSchemeKey("tiler::affine@1"),
        contract: EncodedNumericContract(vec![(
            1,
            CanonicalValue::Type(ResolvedValueType::Nominal(TypeKey("tiler::i8@1"))),
        )]),
    };
    registry.validate_type(&encoded_i8)?;

    let mut graph = SemanticProgramBuilder::new(registry);
    let left: Value<F32> = graph.input()?;
    let right: Value<F32> = graph.input()?;
    let _sum = add(&mut graph, left, right)?;
    let _dynamic = graph.input_resolved(complex_f32)?;

    let mut references = ReferenceRegistryBuilder::default();
    register_reference(&mut references)?;
    references
        .freeze()
        .evaluate(&ADD, &[vec![1], vec![2]], &OperationAttributes(vec![]))?;
    Ok(())
}
