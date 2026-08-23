//! The scheduled-region join: a staged contraction kernel the ADR 0104 witness
//! can read.
//!
//! The staging moves operand memory and threads one carried accumulator through
//! every round, so admitting the join is the removal of a refusal, not a new tree
//! shape.

use super::super::{
    AllocationOwnership, ContractionF32PlanWitness, ContractionF32PlanWitnessError,
    KernelProgramBuilder, KernelProgramDiagnostic, MaterializedOrigin, SemanticOccurrence,
    StageLaunch, ValueRole, VerifiedKernelProgram,
};
use super::support::{
    CANONICAL_NAN, checked_coverage, declare_program_contract, device, literal, program_input,
    read, strict, strict_contract, value, write_access,
};
use crate::kernel::{KernelType, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, ApproximationEnvelope, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, ExceptionalValueAssumption, ExecutionBinding,
    KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization,
    OwnershipProof, OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId,
    RegionProgram, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    VerifiedScheduledRegion,
};
use crate::semantic::{F32, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder};
use crate::shape::{Axis, Shape};

/// `activations[m, k] x weights[n, k] -> projected[m, n]`, with the same
/// arbitrary frontend labels the compiler's own witness fixture uses.
fn cooperative_contraction_semantic(m: u64, n: u64, k: u64) -> SemanticProgram {
    let structure = crate::semantic::ContractionIndexStructure::new(
        [
            [
                crate::semantic::ContractionIndex::new(19),
                crate::semantic::ContractionIndex::new(3),
            ],
            [
                crate::semantic::ContractionIndex::new(14),
                crate::semantic::ContractionIndex::new(3),
            ],
        ],
        [
            crate::semantic::ContractionIndex::new(19),
            crate::semantic::ContractionIndex::new(14),
        ],
    )
    .expect("the projection structure is admissible");
    let mut draft = SemanticProgramBuilder::try_standard().expect("registry");
    let activations = draft
        .input::<F32>(
            InputKey::new("activations").expect("key"),
            Shape::from_dims([m, k]),
        )
        .expect("activations");
    let weights = draft
        .input::<F32>(
            InputKey::new("weights").expect("key"),
            Shape::from_dims([n, k]),
        )
        .expect("weights");
    let projected =
        crate::semantic::F32TensorContraction::apply(&mut draft, &structure, activations, weights)
            .expect("contraction");
    draft
        .output(OutputKey::new("projected").expect("key"), projected)
        .expect("output");
    draft.build().expect("semantic")
}

/// The square-blocked cooperative contraction region over one `[16, 16]` output.
///
/// `contracted` fixes the round count: one round per `16`-wide contracted tile.
/// The realization is the fixture's strict one, which the topology admits — its
/// fold is the declared contributor sequence and consumes no reassociation.
fn cooperative_contraction_region(region_id: u32, contracted: u64) -> VerifiedScheduledRegion {
    const BLOCK: u64 = 16;
    let output = Shape::from_dims([BLOCK, BLOCK]);
    let contracted_shape = Shape::from_dims([contracted]);
    let admitted = crate::schedule::admit_exact_cooperative_contraction(
        &output,
        &Shape::from_dims([BLOCK, BLOCK]),
        &contracted_shape,
        &Shape::from_dims([BLOCK]),
    )
    .expect("exact admission");
    let work_items = BLOCK * BLOCK;
    let owner = OwnershipWitnessId::new(0);
    let operand_map = |free_position: u32, operand: Shape| LogicalAccess::ContractionOperand {
        operand_shape: operand,
        output_shape: output.clone(),
        contracted_shape: contracted_shape.clone(),
        sources: vec![
            ContractionAxisSource::Output {
                position: free_position,
            },
            ContractionAxisSource::Contracted { position: 0 },
        ],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(region_id));
    builder.iteration_shape(output.clone()).expect("shape");
    for (witness, free) in [(0_u32, 0_u32), (1, 1)] {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map: operand_map(free, Shape::from_dims([BLOCK, contracted])),
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .expect("operand access");
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor: TensorRole::Input,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: BLOCK * contracted,
                },
            })
            .expect("operand bounds");
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(owner),
        })
        .expect("output access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: work_items,
            },
        })
        .expect("output bounds");
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: work_items,
            },
        })
        .expect("ownership");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
            },
            numerical: strict(),
        })
        .expect("strict contraction");
    let threads = u32::try_from(work_items).expect("256 fits a u32");
    builder
        .schedule(KernelSchedule {
            binding: admitted.binding,
            work_items,
            threads_per_workgroup: threads,
            tail: TailPolicy::Exact,
            output_owner: owner,
            reduction: ReductionTopology::CooperativeContraction {
                tile: crate::schedule::blocked_operand_tile(BLOCK, admitted.rounds)
                    .expect("a 16-wide operand tile is statable"),
                contracted_shape,
                contracted_tile: admitted.contracted_tile,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: crate::schedule::ArithmeticType::F32,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("schedule");
    builder
        .build()
        .expect("the cooperative contraction verifies")
}

/// The one-stage program whose covering kernel is that staged contraction.
fn cooperative_contraction_program(
    semantic: &SemanticProgram,
    region: &VerifiedScheduledRegion,
    contracted: u64,
) -> VerifiedKernelProgram {
    const BLOCK: u64 = 16;
    let kernel = lower_scheduled_region(region).expect("the cooperative contraction lowers");
    let mut builder = KernelProgramBuilder::new(semantic).expect("program builder");
    declare_program_contract(&mut builder);
    let operand_bytes = BLOCK * contracted * 4;
    let output_bytes = BLOCK * BLOCK * 4;
    let left_allocation = builder
        .push_allocation(device(operand_bytes, AllocationOwnership::External))
        .expect("left allocation");
    let right_allocation = builder
        .push_allocation(device(operand_bytes, AllocationOwnership::External))
        .expect("right allocation");
    let output_allocation = builder
        .push_allocation(device(output_bytes, AllocationOwnership::Program))
        .expect("output allocation");
    let left = builder
        .push_value(
            value(
                program_input("activations"),
                ValueRole::Input,
                Shape::from_dims([BLOCK, contracted]),
            ),
            left_allocation,
        )
        .expect("left value");
    let right = builder
        .push_value(
            value(
                program_input("weights"),
                ValueRole::Input,
                Shape::from_dims([BLOCK, contracted]),
            ),
            right_allocation,
        )
        .expect("right value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([BLOCK, BLOCK]),
            ),
            output_allocation,
        )
        .expect("output value");
    let left_view = builder.push_whole_view(left).expect("left view");
    let right_view = builder.push_whole_view(right).expect("right view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let operand_extent = literal(&mut builder, operand_bytes);
    let output_extent = literal(&mut builder, output_bytes);
    let threads = literal(&mut builder, BLOCK * BLOCK);
    builder
        .push_stage(
            &kernel,
            &checked_coverage(semantic, &strict_contract()),
            &[
                read(left_view, operand_extent),
                read(right_view, operand_extent),
                write_access(output_view, output_extent),
            ],
            StageLaunch {
                grid_threads: threads,
                threads_per_workgroup: threads,
            },
        )
        .expect("the staged contraction stage");
    builder
        .push_output(OutputKey::new("projected").expect("key"), output)
        .expect("published output");
    builder.build().expect("the staged contraction program")
}

fn topology_limits() -> crate::schedule::ContractionF32TopologyLimits {
    crate::schedule::ContractionF32TopologyLimits::new(1024, 1024).expect("limits")
}

/// The join admits the staged contraction, and the tree it yields is the same
/// canonical left chain the unstaged realization witnesses.
///
/// The staging moves operand *memory* and threads one carried accumulator
/// through every round, so it regroups nothing — which is why admitting it is
/// not a new tree shape but the removal of a refusal.
#[test]
fn a_joined_staged_contraction_witnesses_the_canonical_left_chain() {
    const K: u64 = 16;
    let semantic = cooperative_contraction_semantic(16, 16, K);
    let region = cooperative_contraction_region(21, K);
    let program = cooperative_contraction_program(&semantic, &region, K);
    let witness = ContractionF32PlanWitness::from_program_and_regions(
        &semantic,
        &program,
        SemanticOccurrence::new(0),
        std::slice::from_ref(&region),
        topology_limits(),
    )
    .expect("the joined region states this kernel's combine tree");
    assert_eq!(witness.tree().contributor_count(), K);
    assert_eq!(
        witness.tree().depth(),
        usize::try_from(K).expect("K fits a usize"),
        "a left chain over K contributors has depth K"
    );
    assert_eq!(
        witness.kernel_program_identity().as_bytes(),
        program.canonical_identity().as_bytes()
    );
}

/// A multi-round tile changes which memory a round reads and nothing about the
/// association, so it witnesses the same chain over its own contributor count.
#[test]
fn a_multi_round_staged_contraction_witnesses_the_same_chain() {
    const K: u64 = 32;
    let semantic = cooperative_contraction_semantic(16, 16, K);
    let region = cooperative_contraction_region(21, K);
    let program = cooperative_contraction_program(&semantic, &region, K);
    let witness = ContractionF32PlanWitness::from_program_and_regions(
        &semantic,
        &program,
        SemanticOccurrence::new(0),
        std::slice::from_ref(&region),
        topology_limits(),
    )
    .expect("a two-round tile states its tree just as a one-round tile does");
    assert_eq!(witness.tree().contributor_count(), K);
    assert_eq!(
        witness.tree().depth(),
        usize::try_from(K).expect("K fits a usize")
    );
}

/// Without the region, the very same program refuses — unchanged from before
/// the join existed, because program scope still cannot read the staging.
#[test]
fn the_same_staged_program_refuses_when_no_region_is_supplied() {
    const K: u64 = 16;
    let semantic = cooperative_contraction_semantic(16, 16, K);
    let region = cooperative_contraction_region(21, K);
    let program = cooperative_contraction_program(&semantic, &region, K);
    assert_eq!(
        ContractionF32PlanWitness::from_program(
            &semantic,
            &program,
            SemanticOccurrence::new(0),
            topology_limits(),
        )
        .expect_err("program scope cannot read workgroup staging"),
        ContractionF32PlanWitnessError::TopologyUnsupported
    );
    assert_eq!(
        ContractionF32PlanWitness::from_program_and_regions(
            &semantic,
            &program,
            SemanticOccurrence::new(0),
            &[],
            topology_limits(),
        )
        .expect_err("an empty region set joins nothing"),
        ContractionF32PlanWitnessError::ScheduledRegionUnjoined
    );
}

/// The crossed-region control: a region that is not the one this kernel refines
/// is refused, so the join answers from the right record or from none.
#[test]
fn a_crossed_region_is_refused_by_the_identity_join() {
    const K: u64 = 16;
    let semantic = cooperative_contraction_semantic(16, 16, K);
    let region = cooperative_contraction_region(21, K);
    let crossed = cooperative_contraction_region(21, 32);
    let program = cooperative_contraction_program(&semantic, &region, K);
    assert_ne!(
        region.canonical_identity().as_bytes(),
        crossed.canonical_identity().as_bytes(),
        "the two regions differ, so the join has something to reject"
    );
    assert_eq!(
        ContractionF32PlanWitness::from_program_and_regions(
            &semantic,
            &program,
            SemanticOccurrence::new(0),
            std::slice::from_ref(&crossed),
            topology_limits(),
        )
        .expect_err("the crossed region is not the one this kernel refines"),
        ContractionF32PlanWitnessError::ScheduledRegionUnjoined
    );
    // And the true region still answers when it is present beside the crossed
    // one, so the rejection above is selection and not a blanket refusal.
    ContractionF32PlanWitness::from_program_and_regions(
        &semantic,
        &program,
        SemanticOccurrence::new(0),
        &[crossed, region],
        topology_limits(),
    )
    .expect("the matching region is found among several");
}

/// A cooperative-workgroup reduction over a program input, `[16,16,16] -> [16,16]`.
fn cooperative_workgroup_region(region_id: u32) -> VerifiedScheduledRegion {
    const PARTICIPANTS: u64 = 4;
    let input = Shape::from_dims([16, 16, 16]);
    let output = Shape::from_dims([16, 16]);
    let axes = vec![Axis::new(2)];
    let work_items = 256 * PARTICIPANTS;
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(region_id));
    builder
        .iteration_shape(Shape::from_dims([16, 16, PARTICIPANTS]))
        .expect("shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("contributor access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output.clone(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .expect("contributor bounds");
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(owner),
        })
        .expect("output access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 256 },
        })
        .expect("output bounds");
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 256 },
        })
        .expect("ownership");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: NumericalRealization::new(
                "tiler.test.reassociate-f32",
                CANONICAL_NAN,
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Permitted,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ),
        })
        .expect("strict serial sum");
    let threads = u32::try_from(PARTICIPANTS).expect("4 fits");
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items,
            threads_per_workgroup: threads,
            tail: TailPolicy::Exact,
            output_owner: owner,
            reduction: ReductionTopology::CooperativeWorkgroup {
                coverage: crate::schedule::ContributorCoverage::Exact(
                    crate::schedule::ContributorPartition {
                        partitions: PARTICIPANTS,
                        contributors_per_partition: 4,
                    },
                ),
                tile: crate::schedule::workgroup_tree_tile(PARTICIPANTS)
                    .expect("the canonical tree tile"),
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: crate::schedule::ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
                arrival: crate::schedule::ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("schedule");
    builder
        .build()
        .expect("the cooperative workgroup region verifies")
}

/// The reserved arm's reconsideration trigger.
///
/// [`ContractionF32PlanWitness::from_program_and_regions`] refuses a staged
/// kernel whose joined region states a combine tree it cannot express — a
/// cooperative *workgroup* tile, whose partitioned round-structured chain has no
/// representation here. That arm is unreachable at this base, and this test is
/// what says so out loud rather than leaving an untested branch looking
/// exercised.
///
/// A cooperative reduction region is admitted only through the fold gate that
/// hands it exactly one read and one write, so its kernel has one read buffer;
/// and a contraction occurrence has two operands, so a program pairing them
/// either leaves the second operand unread — `UnusedValue` — or omits it, which
/// is [`KernelProgramDiagnostic::IncompleteComponentSet`] below.
///
/// **If this test starts failing, the reserved arm has become reachable** and
/// the refusal needs a real subject perturbation rather than this note.
#[test]
fn a_cooperative_workgroup_kernel_cannot_cover_a_contraction_occurrence() {
    let semantic = cooperative_contraction_semantic(16, 16, 256);
    let region = cooperative_workgroup_region(9);
    let kernel = lower_scheduled_region(&region).expect("the cooperative region lowers");
    assert_ne!(
        kernel.staging().len(),
        0,
        "the tile stages partials, so this kernel is one the join would have to read"
    );
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");
    declare_program_contract(&mut builder);
    let input_allocation = builder
        .push_allocation(device(4096 * 4, AllocationOwnership::External))
        .expect("input allocation");
    let output_allocation = builder
        .push_allocation(device(256 * 4, AllocationOwnership::Program))
        .expect("output allocation");
    let input = builder
        .push_value(
            value(
                program_input("activations"),
                ValueRole::Input,
                Shape::from_dims([16, 256]),
            ),
            input_allocation,
        )
        .expect("input value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([16, 16]),
            ),
            output_allocation,
        )
        .expect("output value");
    let input_view = builder.push_whole_view(input).expect("input view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let input_bytes = literal(&mut builder, 4096 * 4);
    let output_bytes = literal(&mut builder, 256 * 4);
    let grid = literal(&mut builder, 1024);
    let group = literal(&mut builder, 4);
    builder
        .push_stage(
            &kernel,
            &checked_coverage(&semantic, &strict_contract()),
            &[
                read(input_view, input_bytes),
                write_access(output_view, output_bytes),
            ],
            StageLaunch {
                grid_threads: grid,
                threads_per_workgroup: group,
            },
        )
        .expect("the stage itself is structurally admissible");
    builder
        .push_output(OutputKey::new("projected").expect("key"), output)
        .expect("published output");
    let error = builder
        .build()
        .expect_err("the second contraction operand has no value");
    assert_eq!(
        error.diagnostics(),
        [KernelProgramDiagnostic::IncompleteComponentSet]
    );
}

/// The unjoined refusal's breadth is forced by the encoding, not chosen.
///
/// [`ContractionF32PlanWitness::from_program`] refuses on the presence of
/// staging alone. Narrowing that would need program scope to say *which*
/// cooperative topology a staged kernel realizes, because only
/// `CooperativeContraction` leaves the canonical left chain intact. The
/// execution binding is the field that looks like it says so — that topology
/// requires [`ExecutionBinding::BlockedWorkgroup`] and never defaults it — and
/// it does not reach the signature: `verify_signature` derives one
/// `GlobalInvocationIndex` for the blocked and global-linear bindings out of a
/// single shared match arm, then appends `LocalInvocationIndex` for either
/// cooperative tile.
///
/// So a staged contraction and a staged reduction present the same builtins,
/// and every staging row of either is a workgroup F32 allocation. What differs
/// between these two subjects is capacity — the contraction stages one tile per
/// operand at `256` slots, the reduction one partial array at `4` — and a
/// capacity is not a discriminant: `CooperativeTile::staging` is a plain
/// `Vec<WorkgroupStaging>`, so the tile vocabulary ties neither the row count
/// nor the slot count to a topology. Reading either as one would be exactly the
/// inference `staged_role` declines to make.
///
/// **If this test starts failing, program scope has gained a discriminant** and
/// `staged_role`'s reconsideration trigger fires — with the caveat recorded
/// there, that a newly discriminating *derived* field is not on its own the
/// declarative record the narrowing needs.
#[test]
fn a_staged_contraction_and_a_staged_reduction_agree_on_program_scope_builtins() {
    let contraction = lower_scheduled_region(&cooperative_contraction_region(21, 16))
        .expect("the cooperative contraction lowers");
    let reduction = lower_scheduled_region(&cooperative_workgroup_region(9))
        .expect("the cooperative workgroup region lowers");
    assert_ne!(
        contraction.staging().len(),
        0,
        "the operand tile stages, so this kernel reaches the unjoined refusal"
    );
    assert_ne!(
        reduction.staging().len(),
        0,
        "the partial tile stages, so this kernel reaches it too"
    );
    assert_eq!(
        contraction.admitted_builtins(),
        reduction.admitted_builtins(),
        "the execution binding does not reach program scope, so the builtins \
         cannot separate operand staging from partial staging",
    );
    for parameter in contraction.staging().chain(reduction.staging()) {
        assert_eq!(
            parameter.address_space,
            crate::kernel::AddressSpace::Workgroup,
            "every staging row of either topology is workgroup storage",
        );
        assert_eq!(
            parameter.element_type,
            KernelType::F32,
            "every staging row of either topology holds the one staged element type",
        );
    }
}
