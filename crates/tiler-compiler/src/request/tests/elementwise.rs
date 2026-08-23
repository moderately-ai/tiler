use super::super::{
    Bf16, CompilationRequest, DTypeDispatchRefusalDisposition, DeclaredInputOrdinal, F32, InputKey,
    LogicalAccess, NormalizedOutput, OutputKey, PointwiseF32Node, RecognizedPointwise,
    RequestError, SemanticProgram, Shape, TargetProfile, VerifiedRequest, VerifiedTargetResolution,
    verify_request,
};
use super::support::recognize;
use tiler_ir::semantic::{
    Bf16Add, Bf16Constant, Bf16Multiply, F32Add, F32Constant, F32Multiply, SemanticProgramBuilder,
};

/// Builds one whole-program elementwise fixture and its expected nodes.
///
/// `(first * second) + third` over three declared inputs. It is deliberately
/// *not* a shape the superseded template could spell: two of its leaves are
/// distinct input tensors rather than constants, and the old recognizer
/// demanded exactly one input.
fn three_input_elementwise() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let root = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// Elementwise recognition follows the graph, not a taught depth or arity.
///
/// Each shape below was refused by the superseded template, and each was
/// refused for the *leaf count* rather than for anything about what it
/// computes: the old recognizer admitted exactly two operations over exactly
/// three leaves in one of two associations.
#[test]
fn elementwise_recognition_admits_depth_sharing_and_multiple_inputs() {
    // Three declared inputs and a mixed multiply-then-add chain.
    let three = three_input_elementwise();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&three).expect("a three-input expression is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    assert_eq!(
        recognized.input_keys,
        [
            InputKey::new("a").unwrap(),
            InputKey::new("b").unwrap(),
            InputKey::new("c").unwrap(),
        ],
    );
    assert_eq!(recognized.expression.f32().input_count(), 3);
    assert_eq!(recognized.members.len(), three.operation_count());

    // A four-deep chain: `((a * 2.0) + b) * ((a * 2.0) + b)`, whose shared
    // subexpression is one node rather than two. Depth and sharing are both
    // beyond what a three-leaf template could spell.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, first, scale).unwrap();
    let shifted = F32Add::apply(&mut builder, scaled, second).unwrap();
    let root = F32Multiply::apply(&mut builder, shifted, shifted).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let deep = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&deep).expect("a deep shared expression is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    assert_eq!(recognized.expression.f32().input_count(), 2);
    assert_eq!(recognized.members.len(), deep.operation_count());
    assert_eq!(
        recognized.expression.f32().nodes().len(),
        6,
        "the shared `(a * 2.0) + b` is one node, not two",
    );

    // One input read at two leaves, which binds one read access.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let constant = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let doubled = F32Add::apply(&mut builder, input, input).unwrap();
    let root = F32Add::apply(&mut builder, doubled, constant).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let repeated = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&repeated).expect("a repeated read is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    assert_eq!(recognized.expression.f32().input_count(), 1);
    assert_eq!(recognized.input_keys.len(), 1);
}

/// The recognizer admits a `bf16` program and mints its own vocabulary.
///
/// **The wall this replaces refused every program carrying a non-`f32`
/// value under `dtype-f32`, before a subject was normalized**, so no
/// `NormalizedProgram` for one could exist and nothing downstream could be
/// asked about it. Recognition now derives the program's one arithmetic type
/// and walks it with the same authority the `f32` walk uses — the same
/// classification, the same shape checks, the same leaf ordering — and only
/// the minting differs.
///
/// The expression is asserted whole rather than by node count alone: the
/// constant leaf carries the *sixteen* declared payload bits, which is the
/// one place a widened `f32` reading would show up as a number no `bf16`
/// program stated.
#[test]
fn a_bf16_program_is_recognized_in_its_own_expression_vocabulary() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    // `3.0` in bf16, whose sixteen bits are not the low half of any binary32
    // pattern this walk could have read instead.
    let scale = Bf16Constant::apply(&mut builder, 0x4040).unwrap();
    let scaled = Bf16Multiply::apply(&mut builder, input, scale).unwrap();
    let bias = Bf16Constant::apply(&mut builder, 0x8000).unwrap();
    let root = Bf16Add::apply(&mut builder, scaled, bias).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    let program = builder.build().unwrap();

    let NormalizedOutput::Pointwise(recognized) =
        recognize(&program).expect("a bf16 elementwise program is recognized")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    let expression = recognized.expression.bf16();
    assert_eq!(expression.input_count(), 1);
    // The population, counted: every occurrence the program declares is
    // claimed, so an assertion about the expression is an assertion about
    // the whole program rather than about a prefix of it.
    assert_eq!(recognized.members.len(), program.operation_count());
    assert_eq!(
        expression.nodes().len(),
        5,
        "one input leaf, two constants, the multiply, and the add",
    );
    let constants: Vec<u16> = expression
        .nodes()
        .iter()
        .filter_map(|node| match node {
            tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
            _ => None,
        })
        .collect();
    assert_eq!(
        constants,
        [0x4040, 0x8000],
        "the constants are the declared bf16 payloads, not a widened reading",
    );
    assert_eq!(
        recognized.reads,
        vec![(DeclaredInputOrdinal::new(0), LogicalAccess::LinearIdentity)],
        "one dense read of the one declared input",
    );
}

/// Constant occurrence identity reaches the initial recognizer and mint.
///
/// Each pair computes `x * 2 + 2` in its own arithmetic. The only authored
/// difference is whether the add reuses the exact constant value consumed by
/// the multiply or consumes a second constant occurrence with the same
/// payload. Semantic construction, elementwise planning, and minting all
/// preserve that difference for both arithmetic widths the compiler
/// currently recognizes. This drives `recognize` directly: ordinary
/// compilation normalizes equal pure constants before candidate readmission,
/// as the normalization and pipeline regressions assert separately.
///
#[test]
fn equal_constant_occurrences_remain_distinct_through_initial_recognition() {
    fn f32_program(repeat_occurrence: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let scaled = F32Multiply::apply(&mut builder, input, two).unwrap();
        let addend = if repeat_occurrence {
            F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap()
        } else {
            two
        };
        let root = F32Add::apply(&mut builder, scaled, addend).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    fn bf16_program(repeat_occurrence: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
        let scaled = Bf16Multiply::apply(&mut builder, input, two).unwrap();
        let addend = if repeat_occurrence {
            Bf16Constant::apply(&mut builder, 0x4000).unwrap()
        } else {
            two
        };
        let root = Bf16Add::apply(&mut builder, scaled, addend).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    fn recognized_pointwise(program: &SemanticProgram) -> RecognizedPointwise {
        let NormalizedOutput::Pointwise(recognized) =
            recognize(program).expect("the compiler recognizes the elementwise program")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        assert_eq!(
            recognized.members.len(),
            program.operation_count(),
            "the expression must cover every semantic occurrence",
        );
        recognized.expression
    }

    let shared_f32 = f32_program(false);
    let repeated_f32 = f32_program(true);
    assert_eq!(shared_f32.operation_count(), 3);
    assert_eq!(repeated_f32.operation_count(), 4);
    let RecognizedPointwise::F32(shared_f32_expression) = recognized_pointwise(&shared_f32) else {
        panic!("an f32 program must mint the f32 pointwise vocabulary");
    };
    let RecognizedPointwise::F32(repeated_f32_expression) = recognized_pointwise(&repeated_f32)
    else {
        panic!("an f32 program must mint the f32 pointwise vocabulary");
    };
    assert_eq!(shared_f32_expression.nodes().len(), 4);
    assert_eq!(repeated_f32_expression.nodes().len(), 5);
    assert_eq!(
        shared_f32_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                PointwiseF32Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [2.0_f32.to_bits()],
    );
    assert_eq!(
        repeated_f32_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                PointwiseF32Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [2.0_f32.to_bits(), 2.0_f32.to_bits()],
        "the extra node is a second equal-payload constant occurrence",
    );
    assert_ne!(shared_f32_expression, repeated_f32_expression);

    let shared_bf16 = bf16_program(false);
    let repeated_bf16 = bf16_program(true);
    assert_eq!(shared_bf16.operation_count(), 3);
    assert_eq!(repeated_bf16.operation_count(), 4);
    let RecognizedPointwise::Bf16(shared_bf16_expression) = recognized_pointwise(&shared_bf16)
    else {
        panic!("a bf16 program must mint the bf16 pointwise vocabulary");
    };
    let RecognizedPointwise::Bf16(repeated_bf16_expression) = recognized_pointwise(&repeated_bf16)
    else {
        panic!("a bf16 program must mint the bf16 pointwise vocabulary");
    };
    assert_eq!(shared_bf16_expression.nodes().len(), 4);
    assert_eq!(repeated_bf16_expression.nodes().len(), 5);
    assert_eq!(
        shared_bf16_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0x4000],
    );
    assert_eq!(
        repeated_bf16_expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0x4000, 0x4000],
        "the extra node is a second equal-payload constant occurrence",
    );
    assert_ne!(shared_bf16_expression, repeated_bf16_expression);

    let VerifiedRequest::Refused(refusals) = verify_request(CompilationRequest::governed_under(
        &repeated_bf16,
        crate::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16.resolve(),
    ))
    .expect("the governed target refusal is a target-local outcome") else {
        panic!("the governed target declares no bf16 dispatch row");
    };
    let [refusal] = refusals.as_slice() else {
        panic!("the governed request carries one target and one refusal");
    };
    let VerifiedTargetResolution::Rejected(refusal) = &refusal.resolution else {
        panic!("the governed target slot is refused");
    };
    assert_eq!(
        *refusal,
        RequestError::DTypeNotDispatchable {
            target_profile: TargetProfile::governed().profile_key().clone(),
            resolved_type: Box::new(Bf16::resolved_type()),
            disposition: DTypeDispatchRefusalDisposition::Unknown,
        },
        "the governed request stops at dtype dispatch before target-specific recognition",
    );
}
