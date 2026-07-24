//! Bounded tests for the target-neutral kernel-program IR.
//!
//! Fixtures bind real verified structured kernels to real verified semantic
//! programs. Coverage assignments are structural partitions: this layer proves
//! that every operation of the bound graph is covered exactly once, never that
//! a given kernel computes the operations its stage claims.

use crate::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    ReductionTopology, RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
    TensorRole, VerifiedScheduledRegion,
};
use crate::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use crate::shape::{Axis, Shape};

use super::{
    AllocationId, AllocationOwnership, AllocationSpec, ByteWindow, KernelProgramBuildError,
    KernelProgramBuilder, KernelProgramDiagnostic, MaterializedOrigin, MaterializedValueId,
    MaterializedValueSpec, MemorySpace, ProgramEntityKind, SemanticOccurrence, StageAccess,
    StageAccessMode, StageId, ValueRole, VerifiedKernelProgram, ViewId,
};

const SCALE_BITS: u32 = 0x4000_0000; // 2.0f32
const OTHER_SCALE_BITS: u32 = 0x4040_0000; // 3.0f32
const BIAS_BITS: u32 = 0x3f80_0000; // 1.0f32
const CANONICAL_NAN: u32 = 0x7fc0_0000;

fn strict() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_NAN,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
    )
}

fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

fn elements(shape: &Shape) -> u64 {
    crate::schedule::element_count(shape).expect("test shapes do not overflow")
}

fn input_shape() -> Shape {
    Shape::from_dims([2, 3])
}

fn output_shape() -> Shape {
    Shape::from_dims([2])
}

/// Builds the canonical pointwise region: one program input to one temporary.
fn pointwise_region(region: u32, scale_bits: u32) -> VerifiedScheduledRegion {
    let shape = input_shape();
    let count = elements(&shape);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(region));
    builder.iteration_shape(shape).expect("iteration shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read access");
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            kind: BoundsProofKind::LinearRange {
                element_count: count,
            },
        })
        .expect("read proof");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Intermediate,
            kind: BoundsProofKind::LinearRange {
                element_count: count,
            },
        })
        .expect("write proof");
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: count,
            },
        })
        .expect("ownership proof");
    builder
        .scalar_program(ScalarProgram::MultiplyThenAdd {
            scale_bits,
            bias_bits: BIAS_BITS,
            canonical_nan_bits: CANONICAL_NAN,
            contraction: false,
        })
        .expect("scalar program");
    builder.numerical(strict()).expect("numerical realization");
    builder
        .schedule(linear_schedule(count, OwnershipWitnessId::new(0)))
        .expect("schedule");
    builder.build().expect("verified pointwise region")
}

/// Builds the canonical reduction region: one temporary to one program output.
fn reduction_region(region: u32) -> VerifiedScheduledRegion {
    let axes = vec![Axis::new(1)];
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(region));
    builder.iteration_shape(output_shape()).expect("shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read access");
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .expect("read proof");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            kind: BoundsProofKind::LinearRange {
                element_count: elements(&output_shape()),
            },
        })
        .expect("write proof");
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements(&output_shape()),
            },
        })
        .expect("ownership proof");
    builder
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
        })
        .expect("scalar program");
    builder.numerical(strict()).expect("numerical realization");
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(elements(&output_shape()), OwnershipWitnessId::new(0))
        })
        .expect("schedule");
    builder.build().expect("verified reduction region")
}

fn pointwise_kernel(region: u32, scale_bits: u32) -> VerifiedKernel {
    lower_scheduled_region(&pointwise_region(region, scale_bits)).expect("pointwise kernel")
}

fn reduction_kernel(region: u32) -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(region)).expect("reduction kernel")
}

/// A five-operation graph: `result = strict_serial_sum(input * scale + 1.0, 1)`.
fn serial_sum_program(scale_bits: u32) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input::<F32>(InputKey::new("input").expect("key"), input_shape())
        .expect("input");
    let scale = F32Constant::apply(&mut draft, scale_bits).expect("scale");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("bias");
    let product = F32Multiply::apply(&mut draft, input, scale).expect("product");
    let mapped = F32Add::apply(&mut draft, product, bias).expect("mapped");
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).expect("sum");
    draft
        .output(OutputKey::new("result").expect("key"), sum)
        .expect("output");
    let program = draft.build().expect("verified semantic program");
    assert_eq!(program.operation_count(), 5);
    program
}

/// An eight-operation graph with two independent chains and two named outputs.
fn two_chain_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let first = draft
        .input::<F32>(InputKey::new("a").expect("key"), input_shape())
        .expect("first input");
    let second = draft
        .input::<F32>(InputKey::new("b").expect("key"), input_shape())
        .expect("second input");
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("scale");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("bias");
    let first_product = F32Multiply::apply(&mut draft, first, scale).expect("first product");
    let first_mapped = F32Add::apply(&mut draft, first_product, bias).expect("first mapped");
    let second_product = F32Multiply::apply(&mut draft, second, scale).expect("second product");
    let second_mapped = F32Add::apply(&mut draft, second_product, bias).expect("second mapped");
    let first_sum =
        StrictSerialF32Sum::apply(&mut draft, first_mapped, [Axis::new(1)]).expect("first sum");
    let second_sum =
        StrictSerialF32Sum::apply(&mut draft, second_mapped, [Axis::new(1)]).expect("second sum");
    draft
        .output(OutputKey::new("sum_a").expect("key"), first_sum)
        .expect("first output");
    draft
        .output(OutputKey::new("sum_b").expect("key"), second_sum)
        .expect("second output");
    let program = draft.build().expect("verified semantic program");
    assert_eq!(program.operation_count(), 8);
    program
}

fn occurrences(range: std::ops::Range<u32>) -> Vec<SemanticOccurrence> {
    range.map(SemanticOccurrence::new).collect()
}

fn device(capacity_bytes: u64, ownership: AllocationOwnership) -> AllocationSpec {
    AllocationSpec {
        capacity_bytes,
        alignment: 4,
        memory_space: MemorySpace::Device,
        ownership,
    }
}

fn value(origin: MaterializedOrigin, role: ValueRole, shape: Shape) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin,
        role,
        shape,
        element_type: KernelType::F32,
        alignment: 4,
        memory_space: MemorySpace::Device,
    }
}

fn program_input(key: &str) -> MaterializedOrigin {
    MaterializedOrigin::ProgramInput {
        key: InputKey::new(key).expect("input key"),
    }
}

fn read(view: ViewId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Read,
    }
}

fn write(view: ViewId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Write,
    }
}

fn diagnostic(builder: KernelProgramBuilder) -> KernelProgramDiagnostic {
    let error = builder.build().expect_err("verification must fail");
    *error.diagnostics().first().expect("one diagnostic")
}

/// The wired materialized two-stage serial-sum program.
struct TwoStage {
    builder: KernelProgramBuilder,
    pointwise: StageId,
    reduction: StageId,
    source: MaterializedValueId,
    temporary: MaterializedValueId,
    output: MaterializedValueId,
    temporary_allocation: AllocationId,
    output_allocation: AllocationId,
    source_view: ViewId,
    temporary_view: ViewId,
}

/// How one two-stage fixture deviates from the canonical program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TwoStageShape {
    /// The canonical complete program.
    Canonical,
    /// The temporary and the output share one program-owned allocation.
    SharedOutputStorage,
    /// The coverage partition assigns a different split to the two stages.
    ShiftedCoverage,
    /// The two stages leave one occurrence for a third stage to cover.
    ReservedCoverage,
    /// Stages are declared in reverse order and arenas are declared last-first.
    ReversedDeclaration,
}

/// Wires the two-stage program without declaring dependencies or named outputs.
fn wire_two_stage(
    semantic: &SemanticProgram,
    pointwise_kernel: &VerifiedKernel,
    reduction_kernel: &VerifiedKernel,
    shape: TwoStageShape,
) -> TwoStage {
    let (pointwise_coverage, reduction_coverage) = match shape {
        TwoStageShape::ShiftedCoverage => (occurrences(0..3), occurrences(3..5)),
        TwoStageShape::ReservedCoverage => (occurrences(0..2), occurrences(2..4)),
        TwoStageShape::Canonical
        | TwoStageShape::SharedOutputStorage
        | TwoStageShape::ReversedDeclaration => (occurrences(0..4), occurrences(4..5)),
    };
    let mut builder = KernelProgramBuilder::new(semantic).expect("builder");
    let reversed = shape == TwoStageShape::ReversedDeclaration;
    let shared_output = shape == TwoStageShape::SharedOutputStorage;

    // Slot 0 is the externally bound input, slot 1 the temporary, slot 2 the
    // output. The shared-storage fixture declares no separate output storage.
    let mut requested = vec![(0_usize, device(24, AllocationOwnership::External))];
    requested.push((1, device(24, AllocationOwnership::Program)));
    if !shared_output {
        requested.push((2, device(8, AllocationOwnership::Program)));
    }
    if reversed {
        requested.reverse();
    }
    let mut slots: [Option<AllocationId>; 3] = [None; 3];
    for (slot, spec) in requested {
        slots[slot] = Some(builder.push_allocation(spec).expect("allocation"));
    }
    let external = slots[0].expect("external allocation");
    let temporary_allocation = slots[1].expect("temporary allocation");
    let output_allocation = slots[2].unwrap_or(temporary_allocation);

    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            external,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            temporary_allocation,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");

    let push_pointwise = |builder: &mut KernelProgramBuilder| {
        builder
            .push_stage(
                pointwise_kernel,
                &pointwise_coverage,
                &[read(source_view), write(temporary_view)],
            )
            .expect("pointwise stage")
    };
    let push_reduction = |builder: &mut KernelProgramBuilder| {
        builder
            .push_stage(
                reduction_kernel,
                &reduction_coverage,
                &[read(temporary_view), write(output_view)],
            )
            .expect("reduction stage")
    };
    let (pointwise, reduction) = if reversed {
        let reduction = push_reduction(&mut builder);
        (push_pointwise(&mut builder), reduction)
    } else {
        let pointwise = push_pointwise(&mut builder);
        (pointwise, push_reduction(&mut builder))
    };

    TwoStage {
        builder,
        pointwise,
        reduction,
        source,
        temporary,
        output,
        temporary_allocation,
        output_allocation,
        source_view,
        temporary_view,
    }
}

/// Completes the two-stage program with its data dependency and named output.
fn complete_two_stage(mut wired: TwoStage) -> KernelProgramBuilder {
    wired
        .builder
        .push_data_dependency(wired.pointwise, wired.reduction, wired.temporary)
        .expect("data dependency");
    wired
        .builder
        .push_output(OutputKey::new("result").expect("key"), wired.output)
        .expect("named output");
    wired.builder
}

fn two_stage(semantic: &SemanticProgram, shape: TwoStageShape) -> TwoStage {
    wire_two_stage(
        semantic,
        &pointwise_kernel(0, SCALE_BITS),
        &reduction_kernel(1),
        shape,
    )
}

fn canonical_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    complete_two_stage(two_stage(semantic, TwoStageShape::Canonical))
        .build()
        .expect("verified kernel program")
}

#[test]
fn a_verified_program_binds_its_refinements_coverage_and_named_outputs() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);

    assert_eq!(program.stages().len(), 2);
    assert_eq!(program.values().len(), 3);
    assert_eq!(program.allocations().len(), 3);
    assert_eq!(program.views().len(), 3);
    assert_eq!(program.dependencies().len(), 1);
    assert_eq!(program.outputs().len(), 1);
    assert_eq!(
        program.semantic_graph_identity(),
        semantic.semantic_identity().graph()
    );

    // The stage DAG is ordered by its typed dependency, not by insertion.
    let order: Vec<_> = program
        .execution_order()
        .map(|stage| stage.coverage().to_vec())
        .collect();
    assert_eq!(order, vec![occurrences(0..4), occurrences(4..5)]);

    // Each stage retains the exact structured kernel it dispatches, which in
    // turn retains the exact scheduled region that kernel refines.
    let pointwise = program.stages().next().expect("pointwise stage");
    assert_eq!(
        pointwise.kernel().canonical_identity(),
        pointwise_kernel(0, SCALE_BITS).canonical_identity()
    );
    assert_eq!(
        pointwise.kernel().scheduled_region_identity(),
        pointwise_region(0, SCALE_BITS).canonical_identity()
    );
    assert_eq!(pointwise.accesses().len(), 2);

    // The temporary is defined by the pointwise stage and lives in its own
    // program-owned allocation.
    let temporary = program
        .values()
        .find(|value| value.role() == ValueRole::Temporary)
        .expect("one temporary");
    assert_eq!(temporary.required_bytes(), 24);
    assert_eq!(temporary.shape(), &input_shape());
    assert_eq!(temporary.definition(), Some(pointwise));
    assert_eq!(
        temporary.allocation().ownership(),
        AllocationOwnership::Program
    );
    assert_eq!(temporary.allocation().values().count(), 1);

    // The input is externally bound and has no defining stage.
    let source = program
        .values()
        .find(|value| value.role() == ValueRole::Input)
        .expect("one input");
    assert_eq!(source.definition(), None);
    assert_eq!(
        source.origin(),
        &MaterializedOrigin::ProgramInput {
            key: InputKey::new("input").expect("key"),
        }
    );

    let output = program.outputs().next().expect("one output");
    assert_eq!(output.key().as_str(), "result");
    assert_eq!(output.value().role(), ValueRole::Output);
    assert_eq!(output.value().required_bytes(), 8);
}

#[test]
fn identity_is_deterministic_and_independent_of_declaration_order() {
    let semantic = serial_sum_program(SCALE_BITS);
    let first = canonical_program(&semantic);
    let second = canonical_program(&semantic);
    assert_eq!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );

    let reordered = complete_two_stage(two_stage(&semantic, TwoStageShape::ReversedDeclaration))
        .build()
        .expect("verified kernel program");
    assert_eq!(
        first.canonical_identity().as_bytes(),
        reordered.canonical_identity().as_bytes()
    );
    assert_eq!(first, reordered);
}

#[test]
fn identity_excludes_the_transient_planning_region_ordinal() {
    let semantic = serial_sum_program(SCALE_BITS);
    // The same schedules planned under different `RegionId` ordinals.
    let renumbered_pointwise = pointwise_kernel(41, SCALE_BITS);
    let renumbered_reduction = reduction_kernel(97);
    assert_ne!(
        renumbered_pointwise.scheduled_region(),
        pointwise_kernel(0, SCALE_BITS).scheduled_region()
    );
    assert_eq!(
        renumbered_pointwise.canonical_identity(),
        pointwise_kernel(0, SCALE_BITS).canonical_identity()
    );

    let renumbered = complete_two_stage(wire_two_stage(
        &semantic,
        &renumbered_pointwise,
        &renumbered_reduction,
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    assert_eq!(
        canonical_program(&semantic).canonical_identity().as_bytes(),
        renumbered.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_changes_when_the_semantic_graph_layer_changes() {
    // Identical bound implementations, coverage, and structure over two graphs
    // that differ only in one constant: only the ADR 0072 semantic-graph layer
    // moves, and program identity must move with it.
    let first = serial_sum_program(SCALE_BITS);
    let second = serial_sum_program(OTHER_SCALE_BITS);
    assert_ne!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );

    let over_first = canonical_program(&first);
    let over_second = canonical_program(&second);
    assert_ne!(
        over_first.canonical_identity().as_bytes(),
        over_second.canonical_identity().as_bytes()
    );
    assert_ne!(over_first, over_second);
}

#[test]
fn identity_changes_when_a_bound_refinement_changes() {
    // One semantic graph, one coverage split, one structure: only the selected
    // pointwise refinement differs.
    let semantic = serial_sum_program(SCALE_BITS);
    let selected = pointwise_kernel(0, SCALE_BITS);
    let alternative = pointwise_kernel(0, OTHER_SCALE_BITS);
    assert_ne!(
        selected.canonical_identity(),
        alternative.canonical_identity()
    );

    let first = complete_two_stage(wire_two_stage(
        &semantic,
        &selected,
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    let second = complete_two_stage(wire_two_stage(
        &semantic,
        &alternative,
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    assert_ne!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_changes_when_complete_coverage_is_partitioned_differently() {
    // One semantic graph and one pair of bound implementations; two different
    // complete and disjoint coverage partitions.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);
    let shifted = complete_two_stage(two_stage(&semantic, TwoStageShape::ShiftedCoverage))
        .build()
        .expect("verified kernel program");
    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        shifted.canonical_identity().as_bytes()
    );
}

#[test]
fn incomplete_coverage_of_the_bound_graph_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("external allocation");
    let owned = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            external,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            owned,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let pointwise = builder
        .push_stage(
            &pointwise_kernel(0, SCALE_BITS),
            // One graph operation is left uncovered.
            &occurrences(0..3),
            &[read(source_view), write(temporary_view)],
        )
        .expect("pointwise stage");
    let reduction = builder
        .push_stage(
            &reduction_kernel(1),
            &occurrences(3..4),
            &[read(temporary_view), write(output_view)],
        )
        .expect("reduction stage");
    builder
        .push_data_dependency(pointwise, reduction, temporary)
        .expect("data dependency");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");

    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::IncompleteCoverage {
            covered: 4,
            required: 5,
        }
    );
}

#[test]
fn covering_one_occurrence_twice_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(3..5),
                &[read(wired.source_view), write(wired.temporary_view)],
            )
            .expect_err("repeated coverage is rejected"),
        KernelProgramBuildError::DuplicateCoverage {
            occurrence: SemanticOccurrence::new(3),
        }
    );
}

#[test]
fn a_read_without_its_declared_data_dependency_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_output(OutputKey::new("result").expect("key"), wired.output)
        .expect("named output");
    assert_eq!(
        diagnostic(wired.builder),
        KernelProgramDiagnostic::MissingDataDependency
    );
}

#[test]
fn a_dependency_that_states_an_unrealized_obligation_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // A handoff on an allocation holding a single value can never release
    // storage from one value to another: the edge names an obligation its two
    // stages do not realize.
    wired
        .builder
        .push_storage_handoff(wired.reduction, wired.pointwise, wired.temporary_allocation)
        .expect("the edge is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::MisattributedDependency
    );
}

#[test]
fn an_output_may_not_share_storage_with_another_value() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::SharedOutputStorage);
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::ForbiddenAlias
    );
}

#[test]
fn an_unused_view_or_allocation_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_view(
            wired.temporary,
            ByteWindow {
                offset: 0,
                length: 4,
            },
        )
        .expect("the view is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::UnusedView
    );

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_allocation(device(64, AllocationOwnership::Program))
        .expect("the allocation is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::UnusedAllocation
    );
}

#[test]
fn two_indistinguishable_entities_make_identity_ambiguous_and_are_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // Two allocations with identical content and identical (empty) bindings
    // cannot be told apart by any canonical key, so identity would be
    // ambiguous rather than merely redundant.
    for _ in 0..2 {
        wired
            .builder
            .push_allocation(device(64, AllocationOwnership::Program))
            .expect("the allocation is locally well formed");
    }
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::AmbiguousCanonicalKey {
            entity: ProgramEntityKind::Allocation,
        }
    );
}

#[test]
fn a_value_with_no_writer_or_two_writers_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);

    // No writer: the reduction stage alone reads a temporary nobody defines.
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let owned = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            owned,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    builder
        .push_stage(
            &reduction_kernel(1),
            &occurrences(0..5),
            &[read(temporary_view), write(output_view)],
        )
        .expect("reduction stage");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    assert_eq!(diagnostic(builder), KernelProgramDiagnostic::MissingWriter);

    // Two writers: a third stage redefines the temporary the pointwise stage
    // already fully initializes.
    let mut wired = two_stage(&semantic, TwoStageShape::ReservedCoverage);
    wired
        .builder
        .push_stage(
            &pointwise_kernel(2, OTHER_SCALE_BITS),
            &occurrences(4..5),
            &[read(wired.source_view), write(wired.temporary_view)],
        )
        .expect("second writing stage");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::MultipleWriters
    );
}

#[test]
fn a_handle_minted_by_another_program_builder_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let foreign = two_stage(&semantic, TwoStageShape::Canonical);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);

    assert_eq!(
        wired
            .builder
            .push_data_dependency(wired.pointwise, wired.reduction, foreign.temporary)
            .expect_err("a foreign value handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Value,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_data_dependency(foreign.pointwise, wired.reduction, wired.temporary)
            .expect_err("a foreign stage handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Stage,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_whole_view(foreign.source)
            .expect_err("a foreign value handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Value,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_storage_handoff(wired.pointwise, wired.reduction, foreign.output_allocation)
            .expect_err("a foreign allocation handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Allocation,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(3, SCALE_BITS),
                &occurrences(4..5),
                &[read(foreign.source_view), write(foreign.temporary_view)],
            )
            .expect_err("a foreign view handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::View,
        }
    );
}

#[test]
fn a_stage_access_must_realize_its_bound_kernel_signature() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let kernel = pointwise_kernel(2, OTHER_SCALE_BITS);

    assert_eq!(
        wired
            .builder
            .push_stage(&kernel, &occurrences(3..4), &[read(wired.source_view)])
            .expect_err("access arity is checked"),
        KernelProgramBuildError::StageAccessArity {
            expected: 2,
            actual: 1,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(3..4),
                &[read(wired.temporary_view), write(wired.temporary_view)],
            )
            .expect_err("tensor roles are checked"),
        KernelProgramBuildError::StageTensorRole {
            position: 0,
            expected: TensorRole::Input,
            actual: TensorRole::Intermediate,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(3..4),
                &[write(wired.source_view), write(wired.temporary_view)],
            )
            .expect_err("access modes are checked"),
        KernelProgramBuildError::StageAccessMode {
            position: 0,
            expected: StageAccessMode::Read,
            actual: StageAccessMode::Write,
        }
    );

    let partial = wired
        .builder
        .push_view(
            wired.source,
            ByteWindow {
                offset: 0,
                length: 8,
            },
        )
        .expect("partial view");
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(3..4),
                &[read(partial), write(wired.temporary_view)],
            )
            .expect_err("addressed extents are checked"),
        KernelProgramBuildError::StageElementCount {
            position: 0,
            expected: 6,
            actual: 2,
        }
    );
    assert!(matches!(
        wired.builder.push_view(
            wired.source,
            ByteWindow {
                offset: 16,
                length: 16,
            }
        ),
        Err(KernelProgramBuildError::ViewOutOfRange { .. })
    ));
}

/// The wired four-stage two-chain program with a shared temporary allocation.
struct TwoChain {
    builder: KernelProgramBuilder,
    first_map: StageId,
    second_reduce: StageId,
    first_output: MaterializedValueId,
    second_output: MaterializedValueId,
    shared: AllocationId,
}

/// Wires two independent chains whose temporaries share one allocation.
///
/// The forward handoff orders the first chain's final reader before the second
/// chain's writer, which is what makes reusing the shared allocation legal.
fn two_chain(semantic: &SemanticProgram, handoff: bool) -> TwoChain {
    let pointwise = pointwise_kernel(0, SCALE_BITS);
    let reduction = reduction_kernel(1);
    let mut builder = KernelProgramBuilder::new(semantic).expect("builder");
    let storage = wire_chain_storage(&mut builder);

    let first_map = builder
        .push_stage(
            &pointwise,
            &occurrences(0..4),
            &[
                read(storage.first_source_view),
                write(storage.first_temporary_view),
            ],
        )
        .expect("first map stage");
    let first_reduce = builder
        .push_stage(
            &reduction,
            &occurrences(4..5),
            &[
                read(storage.first_temporary_view),
                write(storage.first_output_view),
            ],
        )
        .expect("first reduce stage");
    let second_map = builder
        .push_stage(
            &pointwise,
            &occurrences(5..7),
            &[
                read(storage.second_source_view),
                write(storage.second_temporary_view),
            ],
        )
        .expect("second map stage");
    let second_reduce = builder
        .push_stage(
            &reduction,
            &occurrences(7..8),
            &[
                read(storage.second_temporary_view),
                write(storage.second_output_view),
            ],
        )
        .expect("second reduce stage");

    builder
        .push_data_dependency(first_map, first_reduce, storage.first_temporary)
        .expect("first data dependency");
    builder
        .push_data_dependency(second_map, second_reduce, storage.second_temporary)
        .expect("second data dependency");
    if handoff {
        builder
            .push_storage_handoff(first_reduce, second_map, storage.shared)
            .expect("storage handoff");
    }
    TwoChain {
        builder,
        first_map,
        second_reduce,
        first_output: storage.first_output,
        second_output: storage.second_output,
        shared: storage.shared,
    }
}

/// The allocations, values, and views of the two-chain fixture.
struct ChainStorage {
    shared: AllocationId,
    first_temporary: MaterializedValueId,
    second_temporary: MaterializedValueId,
    first_output: MaterializedValueId,
    second_output: MaterializedValueId,
    first_source_view: ViewId,
    second_source_view: ViewId,
    first_temporary_view: ViewId,
    second_temporary_view: ViewId,
    first_output_view: ViewId,
    second_output_view: ViewId,
}

/// Declares two externally bound inputs, two temporaries sharing one
/// program-owned allocation, and two separately allocated program outputs.
fn wire_chain_storage(builder: &mut KernelProgramBuilder) -> ChainStorage {
    let first_external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("first external allocation");
    let second_external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("second external allocation");
    let shared = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("shared temporary allocation");
    let first_output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("first output allocation");
    let second_output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("second output allocation");

    let internal_temporary = || {
        value(
            MaterializedOrigin::Internal,
            ValueRole::Temporary,
            input_shape(),
        )
    };
    let internal_output = || {
        value(
            MaterializedOrigin::Internal,
            ValueRole::Output,
            output_shape(),
        )
    };
    let first_source = builder
        .push_value(
            value(program_input("a"), ValueRole::Input, input_shape()),
            first_external,
        )
        .expect("first input value");
    let second_source = builder
        .push_value(
            value(program_input("b"), ValueRole::Input, input_shape()),
            second_external,
        )
        .expect("second input value");
    let first_temporary = builder
        .push_value(internal_temporary(), shared)
        .expect("first temporary");
    let second_temporary = builder
        .push_value(internal_temporary(), shared)
        .expect("second temporary");
    let first_output = builder
        .push_value(internal_output(), first_output_allocation)
        .expect("first output value");
    let second_output = builder
        .push_value(internal_output(), second_output_allocation)
        .expect("second output value");

    ChainStorage {
        shared,
        first_temporary,
        second_temporary,
        first_output,
        second_output,
        first_source_view: builder.push_whole_view(first_source).expect("view"),
        second_source_view: builder.push_whole_view(second_source).expect("view"),
        first_temporary_view: builder.push_whole_view(first_temporary).expect("view"),
        second_temporary_view: builder.push_whole_view(second_temporary).expect("view"),
        first_output_view: builder.push_whole_view(first_output).expect("view"),
        second_output_view: builder.push_whole_view(second_output).expect("view"),
    }
}

fn publish_two_chain(mut chains: TwoChain) -> KernelProgramBuilder {
    chains
        .builder
        .push_output(OutputKey::new("sum_a").expect("key"), chains.first_output)
        .expect("first named output");
    chains
        .builder
        .push_output(OutputKey::new("sum_b").expect("key"), chains.second_output)
        .expect("second named output");
    chains.builder
}

#[test]
fn storage_reuse_is_admitted_only_with_an_explicit_handoff() {
    let semantic = two_chain_program();
    let program = publish_two_chain(two_chain(&semantic, true))
        .build()
        .expect("reuse with an explicit handoff verifies");
    assert_eq!(program.stages().len(), 4);
    assert_eq!(program.allocations().len(), 5);
    assert_eq!(program.outputs().len(), 2);

    // The shared allocation carries exactly the two internal temporaries.
    let shared = program
        .allocations()
        .find(|allocation| allocation.values().count() == 2)
        .expect("one shared allocation");
    assert!(
        shared
            .values()
            .all(|value| value.role() == ValueRole::Temporary)
    );

    // Without the handoff the reuse is unproven and the program fails closed.
    let rejected = diagnostic(publish_two_chain(two_chain(&semantic, false)));
    assert!(
        matches!(
            rejected,
            KernelProgramDiagnostic::ReuseMissingHandoff
                | KernelProgramDiagnostic::ReuseLifetimeOverlap
        ),
        "unexpected diagnostic: {rejected:?}"
    );
}

#[test]
fn a_dependency_cycle_is_rejected() {
    let semantic = two_chain_program();
    let mut chains = two_chain(&semantic, true);
    // The opposite handoff is locally well formed and realized — the second
    // chain's reader precedes the first chain's writer — but together with the
    // forward handoff it closes a cycle.
    chains
        .builder
        .push_storage_handoff(chains.second_reduce, chains.first_map, chains.shared)
        .expect("the edge is locally well formed");
    assert_eq!(
        diagnostic(publish_two_chain(chains)),
        KernelProgramDiagnostic::DependencyCycle
    );
}

#[test]
fn a_missing_named_output_is_rejected() {
    let semantic = two_chain_program();
    let mut chains = two_chain(&semantic, true);
    chains
        .builder
        .push_output(OutputKey::new("sum_a").expect("key"), chains.first_output)
        .expect("first named output");
    // The second declared semantic output is never published.
    assert_eq!(
        diagnostic(chains.builder),
        KernelProgramDiagnostic::MissingNamedOutput
    );
}

#[test]
fn an_output_key_outside_the_bound_interface_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    assert!(matches!(
        wired
            .builder
            .push_output(OutputKey::new("other").expect("key"), wired.output),
        Err(KernelProgramBuildError::UnknownOutputKey { .. })
    ));
    assert!(matches!(
        wired.builder.push_value(
            value(program_input("other"), ValueRole::Input, input_shape()),
            wired.output_allocation,
        ),
        Err(KernelProgramBuildError::UnknownProgramInput { .. })
    ));
    // The one declared input is already claimed by another materialized value.
    assert!(matches!(
        wired.builder.push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            wired.output_allocation,
        ),
        Err(KernelProgramBuildError::DuplicateProgramInput { .. })
    ));
    // A temporary claiming a program input is a role/origin contradiction.
    assert_eq!(
        wired
            .builder
            .push_value(
                value(program_input("input"), ValueRole::Temporary, input_shape()),
                wired.temporary_allocation,
            )
            .expect_err("role and origin must agree"),
        KernelProgramBuildError::ValueRoleOrigin {
            role: ValueRole::Temporary,
        }
    );
}
