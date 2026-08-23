use super::super::{
    ArithmeticType, Axis, ContractionIndex, ContractionIndexStructure, F32,
    FrozenIndexRealizationLawRegistry, InputKey, NormalizedOutput, NormalizedProgram, OutputKey,
    ProviderIdentity, RequestError, SemanticProgram, Shape, TypeKey, governed_scalars,
    recognize_program_outputs, select_supported_strategy,
};
use tiler_ir::semantic::{
    CanonicalValue, F32Add, F32Constant, F32Multiply, NormativeDefinitionRef, RegistryError,
    SemanticProgramBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
    StrictSerialF32Sum, TypeDefinitionFacts, ValueTypeDefinition, ValueTypeDefinitionKey,
};

pub(in crate::request) fn program() -> SemanticProgram {
    program_with_builder(SemanticProgramBuilder::try_standard().unwrap())
}

pub(super) fn program_with_builder(mut builder: SemanticProgramBuilder) -> SemanticProgram {
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// A normalization over `[2, 2]` reduced on axis one, optionally scaled.
///
/// `weighted` decides which of the two shapes the ticket names is built: the
/// family as the whole declared output, and the family as a program stage a
/// later elementwise pass consumes.
pub(super) fn normalization_program(weighted: bool, eps_bits: u32) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), shape)
        .unwrap();
    let normalized =
        tiler_ir::semantic::F32RmsNorm::apply(&mut builder, value, weight, Axis::new(1), eps_bits)
            .unwrap();
    let root = if weighted {
        F32Multiply::apply(&mut builder, normalized, value).unwrap()
    } else {
        normalized
    };
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// A normalization over a materialized contraction result, optionally with
/// a trailing elementwise pass and optionally normalizing that result twice.
///
/// `ab,bc->ac` over `a` and `b`, with an independent third `[2, 2]` input
/// `w` serving as the normalization weight. The contraction's two reads are
/// therefore a strict subset of the complete interface in the ordinary
/// `rms_norm(matmul(a, b), w)` spelling.
pub(super) fn contraction_fed_normalization(passed: bool, doubly_staged: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let left = builder
        .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("b").unwrap(), shape.clone())
        .unwrap();
    let independent_weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), shape)
        .unwrap();
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product =
        tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, left, right)
            .unwrap();
    let weight = if doubly_staged {
        product
    } else {
        independent_weight
    };
    let normalized = tiler_ir::semantic::F32RmsNorm::apply(
        &mut builder,
        product,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    let root = if passed {
        F32Multiply::apply(&mut builder, normalized, independent_weight).unwrap()
    } else {
        normalized
    };
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// The realization-law authority recognition consults, for one fixture.
///
/// Paired with the governed scalar profile, which is what the compile path
/// pairs. A fixture that registers its own operations has a semantic
/// authority the governed scalars were never frozen over, so it is paired
/// with the empty scalar registry built over *its* semantic authority
/// instead — recognition asks this registry one question, whether a family's
/// registered law realizes a region sequence, and that reads the semantic
/// law rows alone.
pub(super) fn laws_of(program: &SemanticProgram) -> FrozenIndexRealizationLawRegistry {
    let semantic = program.semantic_registry().clone();
    FrozenIndexRealizationLawRegistry::from_semantic(
        semantic.clone(),
        governed_scalars().expect("the governed scalar profile is coherent"),
    )
    .or_else(|_| {
        FrozenIndexRealizationLawRegistry::from_semantic(
            semantic.clone(),
            tiler_ir::index::ScalarRegistryBuilder::new(semantic).freeze(),
        )
    })
    .expect("a law authority over the fixture's own semantic authority coheres")
}

/// Recognizes one program through the whole boundary, or reports the rule.
///
/// Answers with the sole recognized output, because every fixture reaching
/// it declares one; [`recognize_outputs`] is the multi-output form.
pub(super) fn recognize(program: &SemanticProgram) -> Result<NormalizedOutput, &'static str> {
    strategy_rule(select_supported_strategy(program, &laws_of(program))).map(|recognized| {
        let [output] = recognized.outputs() else {
            panic!("the fixture declares one output");
        };
        output.clone()
    })
}

/// Recognizes one program's ordered named outputs, or reports the rule.
///
/// Drives [`recognize_program_outputs`] directly rather than through
/// [`select_supported_strategy`], so a refusal this helper returns is one
/// the walks themselves produced. The two program-wide properties the
/// boundary checks before them are asserted rather than reported, which is
/// what makes that attribution exact.
pub(super) fn recognize_outputs(
    program: &SemanticProgram,
) -> Result<NormalizedProgram, &'static str> {
    assert_ne!(program.input_count(), 0, "the fixture declares an input");
    assert!(
        program
            .values()
            .all(|value| value.resolved_type() == &F32::resolved_type()),
        "the fixture is f32 throughout",
    );
    strategy_rule(recognize_program_outputs(
        program,
        &laws_of(program),
        ArithmeticType::F32,
    ))
}

/// Reduces one recognition outcome to the strategy rule it refused under.
pub(super) fn strategy_rule(
    outcome: Result<NormalizedProgram, RequestError>,
) -> Result<NormalizedProgram, &'static str> {
    outcome.map_err(|error| match error {
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        } => rule,
        other => panic!("recognition refuses under the strategy phase, got {other:?}"),
    })
}

pub(super) struct UnusedSemantics {
    pub(super) revision: u32,
}

impl SemanticRegistryProvider for UnusedSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(
                TypeKey::new("tiler-test", "unused", 1).expect("the test key is valid"),
            ),
            NormativeDefinitionRef::new("unused test semantics")?,
            TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
        ))
    }
}

/// Builds a binary contraction, optionally with an elementwise epilogue.
pub(super) fn contraction_program(epilogue: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let left = builder
        .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([3, 4]))
        .unwrap();
    // `ab,bc->ac`: the ordinary matrix product, stated as the index
    // structure the operation's identity is.
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product =
        tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, left, right)
            .unwrap();
    let root = if epilogue {
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        F32Multiply::apply(&mut builder, product, scale).unwrap()
    } else {
        product
    };
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// The verified kernel program one compiled target packaged.
pub(super) fn packaged_program(
    compiled: &crate::pipeline::CompilationProduct,
) -> &crate::program::KernelProgram {
    &compiled.targets[0]
        .compiled()
        .expect("the governed target compiled")
        .portfolio
        .alternatives[0]
        .program
}

pub(super) fn planning_capability_rule(
    error: &crate::pipeline::CompileError,
) -> Option<(&'static str, &'static str)> {
    match error {
        crate::pipeline::CompileError::UnsupportedCapability(
            RequestError::UnsupportedCapability { phase, rule },
        ) => Some((*phase, *rule)),
        crate::pipeline::CompileError::Explained { source, .. } => planning_capability_rule(source),
        _ => None,
    }
}
