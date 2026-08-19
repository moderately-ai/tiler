//! The relaxed reference route, end to end through the ordinary pipeline.
//!
//! A `tiler::tensor-contraction-f32@1` occurrence is compiled through the
//! ordinary entry point, its plan-owned combine tree is derived as a
//! [`ContractionF32PlanWitness`] from the verified kernel program, and the
//! concrete [`ContractionF32TopologyEvaluator`] evaluates exactly that tree.
//! The witnessed tree of the direct realization is the canonical left chain,
//! so its topology evaluation must agree bit for bit with the ordinary
//! strict-cell reference — which is the successor contract's central identity:
//! the strict request cell preserves the retired key's answer exactly, and the
//! reassociation-permitted cell contains it as one witnessed member.
//!
//! The refusal half drives the accepted fail-closed joins: a strict effective
//! profile never reaches the topology route; a witness binds its exact
//! semantic graph on both construction and evaluation; and every caller budget
//! bound refuses one step short of the exact preflighted amount.

use tiler_compiler::session::{CompileRequest, NumericalContract, compile};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::program::{ContractionF32PlanWitness, ContractionF32PlanWitnessError};
use tiler_ir::schedule::{
    ApproximationEnvelope, ContractionF32TopologyLimits, ExceptionalValueAssumption,
    NumericalPermission, NumericalRealization, SubnormalMode,
};
use tiler_ir::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, ContractionF32ResultClass, ContractionIndex,
    ContractionIndexStructure, F32, F32TensorContraction, FrozenSemanticRegistry, InputKey,
    OutputKey, SemanticProgram, SemanticProgramBuilder,
    tensor_contraction_f32_reduction_descriptor,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    ContractionF32ReferenceBudget, ContractionF32ReferenceResource,
    ContractionF32TopologyEvaluationError, ContractionF32TopologyEvaluationRequest,
    ExtentBindingContext, FloatBitOrder, FrozenReferenceRegistry, InputBinding, ReferenceElement,
    ReferenceEvaluator, Tensor, TensorPayloadView,
};

// Inside the governed baseline profile's four-thread launch bound, like every
// fixture on the direct path: `M * N = 4` output invocations.
const M: u64 = 2;
const N: u64 = 2;
const K: u64 = 4;

/// The profile's `td,od->to` structure with arbitrary frontend labels.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap()
}

/// `activations[m, k] x weights[n, k] -> projected[m, n]`.
fn projection(m: u64, n: u64, k: u64) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([m, k]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([n, k]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), activations, weights)
            .unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    builder.build().unwrap()
}

/// Small exact integer operands, so any disagreement is evaluation order and
/// never operand rounding.
fn operand(rows: u64, k: u64, salt: u64) -> Tensor {
    let count = rows * k;
    let elements = (0..count)
        .map(|index| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "small integers are exactly representable in binary32"
            )]
            let value = ((index * 7 + salt * 3) % 23) as f32 - 11.0;
            ReferenceElement::from_float_bits(
                value.to_bits().to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .unwrap()
        })
        .collect();
    Tensor::dense(F32::resolved_type(), Shape::from_dims([rows, k]), elements).unwrap()
}

fn result_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a contraction result is dense");
    };
    elements
        .iter()
        .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
        .collect()
}

/// Compiles the program and hands back one closure-driven witness derivation.
fn with_compiled_witness<T>(
    semantic: &SemanticProgram,
    drive: impl FnOnce(&SemanticProgram, &tiler_ir::program::VerifiedKernelProgram) -> T,
) -> T {
    let targets = TargetRequest::new([TargetProfile::governed()]).unwrap();
    let batch = compile(CompileRequest::new(
        semantic,
        NumericalContract::REASSOCIATE_F32,
        targets,
    ))
    .unwrap();
    let outcome = batch.targets().next().unwrap();
    let compilation = outcome.outcome().unwrap();
    let selected = compilation.selected().unwrap();
    drive(semantic, selected.abi().kernel_program())
}

fn ceiling(reassociation: NumericalPermission) -> NumericalRealization {
    NumericalRealization::new(
        "test.topology.ceiling",
        CANONICAL_F32_ARITHMETIC_NAN_BITS,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        reassociation,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

fn limits() -> ContractionF32TopologyLimits {
    ContractionF32TopologyLimits::new(1024, 1024).unwrap()
}

fn budget() -> ContractionF32ReferenceBudget {
    ContractionF32ReferenceBudget::new(1 << 20, 1 << 10, 1 << 20, 1 << 10).unwrap()
}

fn occurrence() -> tiler_ir::program::SemanticOccurrence {
    tiler_ir::program::SemanticOccurrence::new(0)
}

/// The witnessed left chain of the compiled direct realization agrees with the
/// ordinary strict reference bit for bit, and the result carries its exact
/// subjects.
#[test]
fn the_witnessed_left_chain_agrees_with_the_strict_reference_bit_for_bit() {
    let semantic = projection(M, N, K);
    let left = operand(M, K, 1);
    let right = operand(N, K, 2);

    let registry = FrozenReferenceRegistry::standard().unwrap();
    let strict_bits = {
        let evaluator = ReferenceEvaluator::new(registry.clone());
        let outputs = evaluator
            .evaluate(
                &semantic,
                &[
                    InputBinding::new(&InputKey::new("activations").unwrap(), &left),
                    InputBinding::new(&InputKey::new("weights").unwrap(), &right),
                ],
            )
            .unwrap();
        result_bits(&outputs[0])
    };

    with_compiled_witness(&semantic, |semantic, program| {
        let witness =
            ContractionF32PlanWitness::from_program(semantic, program, occurrence(), limits())
                .unwrap();
        assert_eq!(witness.occurrence(), occurrence());
        assert_eq!(witness.tree().contributor_count(), K);
        assert_eq!(
            witness.tree().depth(),
            usize::try_from(K).unwrap(),
            "the direct realization witnesses the canonical left chain"
        );
        assert_eq!(
            witness.kernel_program_identity().as_bytes(),
            program.canonical_identity().as_bytes()
        );

        let descriptor = tensor_contraction_f32_reduction_descriptor(
            &FrozenSemanticRegistry::standard().unwrap(),
        )
        .unwrap();
        let profile = descriptor
            .resolve(ceiling(NumericalPermission::Permitted))
            .unwrap();
        assert_eq!(
            profile.result_class(),
            ContractionF32ResultClass::OrderedFullBinaryTrees
        );

        let evaluator = registry.contraction_f32_topology_evaluator().unwrap();
        let bindings = ExtentBindingContext::empty();
        let evaluation = evaluator
            .evaluate(ContractionF32TopologyEvaluationRequest::new(
                semantic,
                occurrence(),
                [&left, &right],
                &bindings,
                profile,
                &witness,
                budget(),
            ))
            .unwrap();
        assert_eq!(
            result_bits(evaluation.tensor()),
            strict_bits,
            "the witnessed left chain is the strict cell's answer bit for bit"
        );
        assert_eq!(evaluation.occurrence(), occurrence());
        assert_eq!(
            evaluation.reference_registry_identity().as_bytes(),
            registry.canonical_identity().as_bytes()
        );
        assert_eq!(
            evaluation.kernel_program_identity().as_bytes(),
            witness.kernel_program_identity().as_bytes()
        );
    });
}

/// A strict effective profile never reaches the topology route, and every
/// budget bound refuses one step short of the exact preflighted amount.
#[test]
fn the_topology_route_refuses_the_strict_cell_and_exhausted_budgets() {
    let semantic = projection(M, N, K);
    let left = operand(M, K, 1);
    let right = operand(N, K, 2);
    let registry = FrozenReferenceRegistry::standard().unwrap();
    let descriptor =
        tensor_contraction_f32_reduction_descriptor(&FrozenSemanticRegistry::standard().unwrap())
            .unwrap();

    with_compiled_witness(&semantic, |semantic, program| {
        let witness =
            ContractionF32PlanWitness::from_program(semantic, program, occurrence(), limits())
                .unwrap();
        let evaluator = registry.contraction_f32_topology_evaluator().unwrap();
        let bindings = ExtentBindingContext::empty();

        // The strict cell belongs to the ordinary evaluator; no error defaults
        // to strict evaluation in either direction.
        let strict_profile = descriptor
            .resolve(ceiling(NumericalPermission::Forbidden))
            .unwrap();
        assert_eq!(
            evaluator
                .evaluate(ContractionF32TopologyEvaluationRequest::new(
                    semantic,
                    occurrence(),
                    [&left, &right],
                    &bindings,
                    strict_profile,
                    &witness,
                    budget(),
                ))
                .expect_err("the strict cell must not reach the topology route"),
            ContractionF32TopologyEvaluationError::ResultClass {
                expected: ContractionF32ResultClass::OrderedFullBinaryTrees,
                actual: ContractionF32ResultClass::StrictLeftFold,
            }
        );

        // Exact budget preflight: `output_count = 4`, `nodes = 2K - 1 = 7`,
        // `visits = steps = 4 * 7 = 28`, `depth = 4`. Each bound refuses one
        // short.
        let profile = descriptor
            .resolve(ceiling(NumericalPermission::Permitted))
            .unwrap();
        for (budget, resource, limit, actual) in [
            (
                ContractionF32ReferenceBudget::new(27, 1 << 10, 1 << 20, 1 << 10).unwrap(),
                ContractionF32ReferenceResource::ArithmeticSteps,
                27,
                28,
            ),
            (
                ContractionF32ReferenceBudget::new(1 << 20, 6, 1 << 20, 1 << 10).unwrap(),
                ContractionF32ReferenceResource::TopologyNodes,
                6,
                7,
            ),
            (
                ContractionF32ReferenceBudget::new(1 << 20, 1 << 10, 27, 1 << 10).unwrap(),
                ContractionF32ReferenceResource::TopologyNodeVisits,
                27,
                28,
            ),
            (
                ContractionF32ReferenceBudget::new(1 << 20, 1 << 10, 1 << 20, 3).unwrap(),
                ContractionF32ReferenceResource::TopologyDepth,
                3,
                4,
            ),
        ] {
            assert_eq!(
                evaluator
                    .evaluate(ContractionF32TopologyEvaluationRequest::new(
                        semantic,
                        occurrence(),
                        [&left, &right],
                        &bindings,
                        profile,
                        &witness,
                        budget,
                    ))
                    .expect_err("an exhausted budget must refuse before arithmetic"),
                ContractionF32TopologyEvaluationError::BudgetExceeded {
                    resource,
                    limit,
                    actual,
                }
            );
        }
    });
}

/// A witness binds its exact semantic graph: construction against a foreign
/// program refuses, and evaluation with a foreign semantic program refuses.
#[test]
fn a_witness_binds_its_exact_semantic_graph_on_both_sides() {
    let semantic = projection(M, N, K);
    let foreign = projection(M, N, K + 1);
    let left = operand(M, K + 1, 1);
    let right = operand(N, K + 1, 2);
    let registry = FrozenReferenceRegistry::standard().unwrap();
    let descriptor =
        tensor_contraction_f32_reduction_descriptor(&FrozenSemanticRegistry::standard().unwrap())
            .unwrap();

    with_compiled_witness(&semantic, |semantic, program| {
        assert_eq!(
            ContractionF32PlanWitness::from_program(&foreign, program, occurrence(), limits())
                .expect_err("a foreign semantic program must refuse at witness construction"),
            ContractionF32PlanWitnessError::SemanticGraphMismatch
        );

        let witness =
            ContractionF32PlanWitness::from_program(semantic, program, occurrence(), limits())
                .unwrap();
        let evaluator = registry.contraction_f32_topology_evaluator().unwrap();
        let bindings = ExtentBindingContext::empty();
        let profile = descriptor
            .resolve(ceiling(NumericalPermission::Permitted))
            .unwrap();
        assert_eq!(
            evaluator
                .evaluate(ContractionF32TopologyEvaluationRequest::new(
                    &foreign,
                    occurrence(),
                    [&left, &right],
                    &bindings,
                    profile,
                    &witness,
                    budget(),
                ))
                .expect_err("a foreign semantic program must refuse at evaluation"),
            ContractionF32TopologyEvaluationError::SemanticSubjectMismatch
        );
    });
}
