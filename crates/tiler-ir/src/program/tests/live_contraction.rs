//! The live-contraction ABI requirement: the guard `push_stage` derives and folds
//! into the applicability guard so a live contraction refuses a zero input extent.

use super::super::abi::{
    AbiBinaryOp, AbiFacts, AbiRoot, AbiValue, AvailabilityPhase, ExprNode, evaluate,
};
use super::super::{
    AllocationOwnership, ByteWindow, CoveredOccurrence, KernelProgramBuildError,
    KernelProgramBuilder, KernelProgramDiagnostic, MAX_PROGRAM_ABI_EXPRESSIONS, MaterializedOrigin,
    MaterializedValueId, StageAccess, StageId, StageLaunch, ValueRole, VerifiedKernelProgram,
};
use super::support::{
    AbiGrowth, CANONICAL_NAN, checked_coverage, coverage_range, declare_program_contract,
    declare_routing_commit, device, elements, grown_guard, linear_schedule, literal, program_input,
    read, strict, strict_contract, value, write_access,
};
use crate::kernel::{VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, KernelSchedule, LogicalAccess, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId, RegionProgram,
    ScalarProgram, ScheduledRegionBuilder, TensorRole,
};
use crate::semantic::{
    F32, F32Add, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use crate::shape::{Axis, Shape};

/// The schedule whose strict fold seeds contributor zero before looping over
/// contributors `1..S`.
fn live_contraction_kernel_for_program() -> VerifiedKernel {
    let left = Shape::from_dims([2]);
    let right = Shape::from_dims([3]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([]);
    let output_elements = elements(&output);
    let owner = OwnershipWitnessId::new(0);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(40));
    region.iteration_shape(output.clone()).expect("shape");
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).expect("two operands");
        let tensor = TensorRole::Input;
        region
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand.clone(),
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![ContractionAxisSource::Output { position: free }],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .expect("operand access");
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .expect("live bounds");
    }
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(owner),
        })
        .expect("output access");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .expect("output bounds");
    region
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .expect("ownership");
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
            },
            numerical: strict(),
        })
        .expect("strict contraction");
    region
        .schedule(KernelSchedule {
            reduction: ReductionTopology::LiveContraction {
                live_access: AccessOrdinal::FIRST,
                live_axis: Axis::new(1),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, owner)
        })
        .expect("schedule");
    lower_scheduled_region(&region.build().expect("verified schedule"))
        .expect("lowered contraction")
}

/// A two-input graph whose one occurrence supplies real refinement evidence.
///
/// Program coverage is structural rather than a second semantic equivalence
/// proof, so this deliberately small graph keeps the test focused on the stage
/// binding and ABI guard added by the live-contraction kernel.
fn live_contraction_semantic(shape: Shape) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("registry");
    let left = draft
        .input::<F32>(InputKey::new("left").expect("key"), shape.clone())
        .expect("left");
    let right = draft
        .input::<F32>(InputKey::new("right").expect("key"), shape)
        .expect("right");
    let result = F32Add::apply(&mut draft, left, right).expect("add");
    draft
        .output(OutputKey::new("result").expect("key"), result)
        .expect("output");
    draft.build().expect("semantic")
}

/// One live-contraction stage before its guard, coverage, and named output are
/// committed to the program builder.
///
/// Keeping these pieces separately lets the limit tests exercise the
/// transactional `push_stage` boundary without forging a kernel or reaching
/// through the builder's private state.
struct LiveContractionProgramDraft {
    builder: KernelProgramBuilder,
    kernel: VerifiedKernel,
    coverage: Vec<CoveredOccurrence>,
    accesses: [StageAccess; 3],
    launch: StageLaunch,
    output: MaterializedValueId,
}

fn live_contraction_program_draft(
    semantic: &SemanticProgram,
) -> Result<LiveContractionProgramDraft, KernelProgramBuildError> {
    let kernel = live_contraction_kernel_for_program();
    let mut builder = KernelProgramBuilder::new(semantic)?;
    let shape = semantic
        .shape(semantic.inputs().next().expect("left input").value())
        .expect("input shape")
        .as_static()
        .expect("static fixture")
        .clone();
    let bytes = elements(&shape) * 4;
    let left_allocation = builder.push_allocation(device(bytes, AllocationOwnership::External))?;
    let right_allocation = builder.push_allocation(device(bytes, AllocationOwnership::External))?;
    let output_allocation = builder.push_allocation(device(bytes, AllocationOwnership::Program))?;
    let left = builder.push_value(
        value(program_input("left"), ValueRole::Input, shape.clone()),
        left_allocation,
    )?;
    let right = builder.push_value(
        value(program_input("right"), ValueRole::Input, shape.clone()),
        right_allocation,
    )?;
    let output = builder.push_value(
        value(MaterializedOrigin::Internal, ValueRole::Output, shape),
        output_allocation,
    )?;
    let left_view = builder.push_view(
        left,
        ByteWindow {
            offset: 0,
            length: 0,
        },
    )?;
    let right_view = builder.push_view(
        right,
        ByteWindow {
            offset: 0,
            length: 0,
        },
    )?;
    let output_view = builder.push_whole_view(output)?;
    let zero = literal(&mut builder, 0);
    let six = literal(&mut builder, 6);
    let one = literal(&mut builder, 1);
    let output_bytes = literal(&mut builder, bytes);
    Ok(LiveContractionProgramDraft {
        builder,
        kernel,
        coverage: checked_coverage(semantic, &strict_contract()),
        accesses: [
            read(left_view, zero),
            read(right_view, zero),
            write_access(output_view, output_bytes),
        ],
        launch: StageLaunch {
            grid_threads: six,
            threads_per_workgroup: one,
        },
        output,
    })
}

fn push_live_contraction_stage(
    draft: &mut LiveContractionProgramDraft,
) -> Result<StageId, KernelProgramBuildError> {
    draft.builder.push_stage(
        &draft.kernel,
        &draft.coverage,
        &draft.accesses,
        draft.launch,
    )
}

fn publish_live_contraction_output(
    draft: &mut LiveContractionProgramDraft,
) -> Result<(), KernelProgramBuildError> {
    draft
        .builder
        .push_output(OutputKey::new("result").expect("key"), draft.output)
}

fn live_contraction_program_builder(
    semantic: &SemanticProgram,
) -> Result<KernelProgramBuilder, KernelProgramBuildError> {
    let mut draft = live_contraction_program_draft(semantic)?;
    declare_program_contract(&mut draft.builder);
    push_live_contraction_stage(&mut draft)?;
    publish_live_contraction_output(&mut draft)?;
    Ok(draft.builder)
}

#[test]
fn a_live_contraction_derives_the_same_input_extent_guard_and_refuses_zero() {
    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let program = live_contraction_program_builder(&semantic)
        .expect("stage binding")
        .build()
        .expect("verified program");
    let key = InputKey::new("left").expect("key");
    let evaluate_guard = |extent| {
        evaluate(
            program.abi_expressions(),
            program.applicability_guard(),
            &AbiFacts::new(
                AvailabilityPhase::LiveDevicePreflight,
                vec![(key.clone(), Axis::new(1), extent)],
                Vec::new(),
            ),
        )
        .expect("guard evaluates")
    };
    assert_eq!(evaluate_guard(0), AbiValue::Boolean(false));
    assert_eq!(evaluate_guard(1), AbiValue::Boolean(true));
    assert_eq!(evaluate_guard(14), AbiValue::Boolean(true));
    assert_eq!(evaluate_guard(15), AbiValue::Boolean(true));
}

#[test]
fn a_live_contraction_conjoins_its_requirement_with_a_nontrivial_authored_guard() {
    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let mut draft = live_contraction_program_draft(&semantic).expect("draft");
    let two = literal(&mut draft.builder, 2);
    let right_axis_zero = draft
        .builder
        .push_abi_root(AbiRoot::InputExtent {
            key: InputKey::new("right").expect("key"),
            axis: Axis::new(0),
        })
        .expect("right extent root");
    let authored = draft
        .builder
        .push_abi_binary(AbiBinaryOp::LessOrEqual, two, right_axis_zero)
        .expect("2 <= right axis 0");
    draft
        .builder
        .applicability_guard(authored)
        .expect("authored guard");
    declare_routing_commit(&mut draft.builder);
    push_live_contraction_stage(&mut draft).expect("stage");
    publish_live_contraction_output(&mut draft).expect("output");
    let program = draft.builder.build().expect("verified program");

    let left = InputKey::new("left").expect("key");
    let right = InputKey::new("right").expect("key");
    let evaluate_guard = |left_axis_one, right_axis_zero| {
        evaluate(
            program.abi_expressions(),
            program.applicability_guard(),
            &AbiFacts::new(
                AvailabilityPhase::LiveDevicePreflight,
                vec![
                    (left.clone(), Axis::new(1), left_axis_one),
                    (right.clone(), Axis::new(0), right_axis_zero),
                ],
                Vec::new(),
            ),
        )
        .expect("guard evaluates")
    };
    for (left_axis_one, right_axis_zero, expected) in
        [(0, 1, false), (0, 2, false), (1, 1, false), (1, 2, true)]
    {
        assert_eq!(
            evaluate_guard(left_axis_one, right_axis_zero),
            AbiValue::Boolean(expected),
            "left axis 1 = {left_axis_one}, right axis 0 = {right_axis_zero}"
        );
    }
}

/// Grows the authored guard so the live requirement reaches the exact ABI-node
/// limit after `push_stage` reserves and `build` materializes it.
fn saturated_live_contraction_draft(
    semantic: &SemanticProgram,
    growth_levels: usize,
) -> LiveContractionProgramDraft {
    let mut draft = live_contraction_program_draft(semantic).expect("draft");
    let authored = grown_guard(&mut draft.builder, AbiGrowth::SharedDag, growth_levels);
    draft
        .builder
        .applicability_guard(authored)
        .expect("authored guard");
    declare_routing_commit(&mut draft.builder);
    draft
}

#[test]
fn an_exact_limit_failed_build_recovers_and_retries_to_the_same_identity_as_a_twin() {
    // Four authored size/launch roots, one authored boolean root, 4,088 shared
    // DAG levels, and three derived nodes (extent, predicate, final `And`) are
    // exactly 4,096. The derived guard reuses the authored literal one.
    const GROWTH_LEVELS: usize = MAX_PROGRAM_ABI_EXPRESSIONS - 8;

    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let mut interrupted = saturated_live_contraction_draft(&semantic, GROWTH_LEVELS);
    push_live_contraction_stage(&mut interrupted).expect("exact-limit stage");
    let output = interrupted.output;
    let error = interrupted
        .builder
        .build()
        .expect_err("the deliberately omitted named output fails verification");
    assert_eq!(
        error.diagnostics(),
        &[KernelProgramDiagnostic::EmptyProgram]
    );
    let (mut recovered, diagnostics) = error.into_parts();
    assert_eq!(diagnostics, vec![KernelProgramDiagnostic::EmptyProgram]);
    recovered
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("repair the recovered builder");
    let retried = recovered.build().expect("recovered exact-limit program");

    let mut twin = saturated_live_contraction_draft(&semantic, GROWTH_LEVELS);
    push_live_contraction_stage(&mut twin).expect("twin stage");
    publish_live_contraction_output(&mut twin).expect("twin output");
    let twin = twin.builder.build().expect("fresh exact-limit twin");

    assert_eq!(retried.abi_expressions().len(), MAX_PROGRAM_ABI_EXPRESSIONS);
    assert_eq!(twin.abi_expressions().len(), MAX_PROGRAM_ABI_EXPRESSIONS);
    assert_eq!(retried.abi_expressions(), twin.abi_expressions());
    assert_eq!(retried.applicability_guard(), twin.applicability_guard());
    assert_eq!(retried.canonical_identity(), twin.canonical_identity());
    assert_eq!(retried, twin);
}

#[test]
fn a_stage_whose_derived_guard_would_be_node_4097_refuses_transactionally() {
    const GROWTH_LEVELS: usize = MAX_PROGRAM_ABI_EXPRESSIONS - 7;

    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let mut draft = saturated_live_contraction_draft(&semantic, GROWTH_LEVELS);
    let expected = KernelProgramBuildError::StructuralLimit {
        resource: super::super::ProgramLimitKind::AbiExpressions,
        actual: MAX_PROGRAM_ABI_EXPRESSIONS + 1,
        limit: MAX_PROGRAM_ABI_EXPRESSIONS,
    };
    assert_eq!(
        push_live_contraction_stage(&mut draft).expect_err("node 4,097 must refuse"),
        expected
    );
    assert_eq!(
        push_live_contraction_stage(&mut draft)
            .expect_err("the failed insertion must not retain coverage or a requirement"),
        expected
    );
    draft
        .builder
        .push_abi_root(AbiRoot::BooleanLiteral(false))
        .expect("the failed stage left the authored 4,094-node arena intact");
}

/// Two independent operations over the same pair of program inputs.
///
/// Structural coverage is the program layer's authority, so the two stages may
/// bind the same verified kernel while carrying distinct refinement receipts
/// and distinct output storage. That makes this a direct probe of requirement
/// deduplication and ordering rather than a second semantic-equivalence test.
fn two_live_contraction_semantic() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("registry");
    let shape = Shape::from_dims([2, 3]);
    let left = draft
        .input::<F32>(InputKey::new("left").expect("key"), shape.clone())
        .expect("left");
    let right = draft
        .input::<F32>(InputKey::new("right").expect("key"), shape)
        .expect("right");
    let first = F32Add::apply(&mut draft, left, right).expect("first operation");
    let second = F32Multiply::apply(&mut draft, left, right).expect("second operation");
    draft
        .output(OutputKey::new("first").expect("key"), first)
        .expect("first output");
    draft
        .output(OutputKey::new("second").expect("key"), second)
        .expect("second output");
    draft.build().expect("semantic")
}

/// Builds two independent live-contraction stages.
///
/// The first always binds kernel input zero to `left`. When
/// `distinct_requirements` is true, the second swaps its two input bindings so
/// its live requirement resolves to `right`. `reverse_insertion` changes only
/// builder insertion order; canonical stage keys still order identity.
fn two_live_contraction_stage_program(
    semantic: &SemanticProgram,
    distinct_requirements: bool,
    reverse_insertion: bool,
) -> VerifiedKernelProgram {
    let kernel = live_contraction_kernel_for_program();
    let coverage = checked_coverage(semantic, &strict_contract());
    let mut builder = KernelProgramBuilder::new(semantic).expect("builder");
    let bytes = 24;
    let left_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::External))
        .expect("left allocation");
    let right_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::External))
        .expect("right allocation");
    let first_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::Program))
        .expect("first output allocation");
    let second_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::Program))
        .expect("second output allocation");
    let shape = Shape::from_dims([2, 3]);
    let left = builder
        .push_value(
            value(program_input("left"), ValueRole::Input, shape.clone()),
            left_allocation,
        )
        .expect("left value");
    let right = builder
        .push_value(
            value(program_input("right"), ValueRole::Input, shape.clone()),
            right_allocation,
        )
        .expect("right value");
    let first = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                shape.clone(),
            ),
            first_allocation,
        )
        .expect("first output value");
    let second = builder
        .push_value(
            value(MaterializedOrigin::Internal, ValueRole::Output, shape),
            second_allocation,
        )
        .expect("second output value");
    let left_view = builder
        .push_view(
            left,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("left live view");
    let right_view = builder
        .push_view(
            right,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("right live view");
    let first_view = builder.push_whole_view(first).expect("first output view");
    let second_view = builder.push_whole_view(second).expect("second output view");
    let zero = literal(&mut builder, 0);
    let six = literal(&mut builder, 6);
    let one = literal(&mut builder, 1);
    let output_bytes = literal(&mut builder, bytes);
    declare_program_contract(&mut builder);
    let first_accesses = [
        read(left_view, zero),
        read(right_view, zero),
        write_access(first_view, output_bytes),
    ];
    let second_accesses = if distinct_requirements {
        [
            read(right_view, zero),
            read(left_view, zero),
            write_access(second_view, output_bytes),
        ]
    } else {
        [
            read(left_view, zero),
            read(right_view, zero),
            write_access(second_view, output_bytes),
        ]
    };
    let launch = StageLaunch {
        grid_threads: six,
        threads_per_workgroup: one,
    };
    let push = |builder: &mut KernelProgramBuilder,
                covered: &[CoveredOccurrence],
                accesses: &[StageAccess]| {
        builder
            .push_stage(&kernel, covered, accesses, launch)
            .expect("independent live-contraction stage");
    };
    if reverse_insertion {
        push(
            &mut builder,
            &coverage_range(&coverage, 1..2),
            &second_accesses,
        );
        push(
            &mut builder,
            &coverage_range(&coverage, 0..1),
            &first_accesses,
        );
    } else {
        push(
            &mut builder,
            &coverage_range(&coverage, 0..1),
            &first_accesses,
        );
        push(
            &mut builder,
            &coverage_range(&coverage, 1..2),
            &second_accesses,
        );
    }
    builder
        .push_output(OutputKey::new("first").expect("key"), first)
        .expect("first output");
    builder
        .push_output(OutputKey::new("second").expect("key"), second)
        .expect("second output");
    builder.build().expect("verified two-stage program")
}

#[test]
fn two_live_stages_requiring_the_same_input_extent_share_one_guard_fact() {
    let semantic = two_live_contraction_semantic();
    let program = two_live_contraction_stage_program(&semantic, false, false);
    let left = InputKey::new("left").expect("key");
    let roots: Vec<_> = program
        .abi_expressions()
        .iter()
        .filter_map(|node| match node {
            ExprNode::Root(AbiRoot::InputExtent { key, axis }) => Some((key, *axis)),
            _ => None,
        })
        .collect();
    assert_eq!(roots, vec![(&left, Axis::new(1))]);
    assert_eq!(program.abi_expressions().len(), 8);
    for (extent, expected) in [(0, false), (1, true)] {
        assert_eq!(
            evaluate(
                program.abi_expressions(),
                program.applicability_guard(),
                &AbiFacts::new(
                    AvailabilityPhase::LiveDevicePreflight,
                    vec![(left.clone(), Axis::new(1), extent)],
                    Vec::new(),
                ),
            ),
            Ok(AbiValue::Boolean(expected))
        );
    }
}

#[test]
fn two_distinct_required_extents_have_one_guard_and_identity_in_either_stage_order() {
    let semantic = two_live_contraction_semantic();
    let forward = two_live_contraction_stage_program(&semantic, true, false);
    let reversed = two_live_contraction_stage_program(&semantic, true, true);

    assert_eq!(forward.abi_expressions(), reversed.abi_expressions());
    assert_eq!(
        forward.applicability_guard(),
        reversed.applicability_guard()
    );
    assert_eq!(forward.canonical_identity(), reversed.canonical_identity());
    assert_eq!(forward, reversed);

    let left = InputKey::new("left").expect("key");
    let right = InputKey::new("right").expect("key");
    for (left_extent, right_extent, expected) in
        [(0, 0, false), (0, 1, false), (1, 0, false), (1, 1, true)]
    {
        let facts = AbiFacts::new(
            AvailabilityPhase::LiveDevicePreflight,
            vec![
                (left.clone(), Axis::new(1), left_extent),
                (right.clone(), Axis::new(1), right_extent),
            ],
            Vec::new(),
        );
        assert_eq!(
            evaluate(
                forward.abi_expressions(),
                forward.applicability_guard(),
                &facts,
            ),
            Ok(AbiValue::Boolean(expected))
        );
        assert_eq!(
            evaluate(
                reversed.abi_expressions(),
                reversed.applicability_guard(),
                &facts,
            ),
            Ok(AbiValue::Boolean(expected))
        );
    }
}

#[test]
fn a_live_contraction_requirement_without_one_logical_owner_fails_typed() {
    let semantic = live_contraction_semantic(Shape::from_dims([6]));
    let error = live_contraction_program_builder(&semantic)
        .expect_err("axis 1 cannot resolve against rank-one program inputs");
    assert_eq!(
        error,
        KernelProgramBuildError::RequiredInputExtentBinding {
            tensor: TensorRole::Input,
            axis: Axis::new(1),
            matches: 0,
        }
    );
}
