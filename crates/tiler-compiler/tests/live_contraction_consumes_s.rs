//! Schedule-verified live contraction consumes `S` without specializing identity.
//!
//! The working construction path is [`ScheduledRegionBuilder`] plus
//! [`lower_scheduled_region`], not `compile()`. Neighbouring live extents move
//! the load-count oracle; baking either neighbour changes kernel identity.

use tiler_ir::kernel::{LoopBound, OperationView, VerifiedBufferId, lower_scheduled_region};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContractionAxisSource,
    ContributorOrder, ExceptionalValueAssumption, ExecutionBinding, InputOrdinal, KernelSchedule,
    LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId, ScalarProgram,
    ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole, VerifiedScheduledRegion,
    element_count,
};
use tiler_ir::shape::{Axis, Shape};

const NAN_BITS: u32 = 0x7fc0_0000;

fn numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        NAN_BITS,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
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

fn live_contraction_region() -> VerifiedScheduledRegion {
    let left = Shape::from_dims([2]);
    let right = Shape::from_dims([3]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([]);
    let output_elements = element_count(&output).unwrap();
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(9));
    builder.iteration_shape(output.clone()).unwrap();
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).unwrap();
        let tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(witness),
        };
        builder
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
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .unwrap();
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
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictTensorContraction {
            contracted_shape: contracted,
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
        })
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::LiveContraction {
                live_input: InputOrdinal::FIRST,
                live_axis: Axis::new(1),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, owner)
        })
        .unwrap();
    builder.build().unwrap()
}

fn baked_contraction_region(k: u64) -> VerifiedScheduledRegion {
    let left = Shape::from_dims([2, k]);
    let right = Shape::from_dims([3, k]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([k]);
    let output_elements = element_count(&output).unwrap();
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(9));
    builder.iteration_shape(output.clone()).unwrap();
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).unwrap();
        let tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(witness),
        };
        builder
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand.clone(),
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![
                        ContractionAxisSource::Output { position: free },
                        ContractionAxisSource::Contracted { position: 0 },
                    ],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: element_count(operand).unwrap(),
                },
            })
            .unwrap();
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
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictTensorContraction {
            contracted_shape: contracted.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
        })
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, owner)
        })
        .unwrap();
    builder.build().unwrap()
}

fn count_live_input_loads(
    block: tiler_ir::kernel::BlockRef<'_>,
    live: VerifiedBufferId,
    seed: &mut u64,
    body: &mut u64,
    in_loop: bool,
) {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Load { buffer, .. } if buffer == live => {
                if in_loop {
                    *body = body.saturating_add(1);
                } else {
                    *seed = seed.saturating_add(1);
                }
            }
            OperationView::Predicated { body: nested, .. } => {
                count_live_input_loads(nested, live, seed, body, in_loop);
            }
            OperationView::SerialLoop(serial) => {
                count_live_input_loads(serial.body(), live, seed, body, true);
            }
            _ => {}
        }
    }
}

fn live_input_loads(kernel: &tiler_ir::kernel::VerifiedKernel, bound: u64) -> u64 {
    let live = kernel
        .declared_buffers()
        .find_map(|(id, buffer)| {
            (buffer.tensor
                == TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                })
            .then_some(id)
        })
        .expect("the named live input is a buffer");
    let mut seed = 0;
    let mut body = 0;
    count_live_input_loads(kernel.body(), live, &mut seed, &mut body, false);
    let remaining = bound
        .checked_sub(1)
        .expect("preflight must refuse an empty strict contraction before execution");
    seed.checked_add(remaining.checked_mul(body).unwrap())
        .unwrap()
}

#[test]
fn neighbouring_s_values_move_the_load_oracle_and_leave_identity() {
    let scheduled = live_contraction_region();
    let kernel = lower_scheduled_region(&scheduled).expect("the live contraction lowers");
    let loop_end = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .and_then(|body| {
            body.operations()
                .find_map(|operation| match operation.view() {
                    OperationView::SerialLoop(reduction) => Some(reduction.end_bound()),
                    _ => None,
                })
        })
        .expect("a live contributor loop");
    assert!(
        matches!(loop_end, LoopBound::Value(_)),
        "the contributor bound must be the live operand"
    );

    let loads_1 = live_input_loads(&kernel, 1);
    let loads_14 = live_input_loads(&kernel, 14);
    let loads_15 = live_input_loads(&kernel, 15);
    assert_eq!(loads_1, 1);
    assert_eq!(loads_14, 14);
    assert_eq!(loads_15, 15);
    assert_ne!(loads_14, loads_15);

    let again = lower_scheduled_region(&live_contraction_region()).unwrap();
    assert_eq!(
        kernel.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes()
    );
    assert_eq!(
        scheduled.canonical_identity(),
        live_contraction_region().canonical_identity()
    );

    let baked_14 = lower_scheduled_region(&baked_contraction_region(14)).unwrap();
    let baked_15 = lower_scheduled_region(&baked_contraction_region(15)).unwrap();
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_14.canonical_identity().as_bytes()
    );
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes()
    );
}
