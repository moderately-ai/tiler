use super::super::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, Builtin, ConvertOp,
    InputExtentParameter, KernelBuildError, KernelBuilder, KernelConstant, KernelDiagnostic,
    KernelType, LoopBound, OperationView, SerialLoopRef, VerifiedBufferId, VerifiedKernel,
    lower_scheduled_region,
};
use super::support::{NAN_BITS, contraction_region, guard, linear_schedule, numerical};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, KernelSchedule, LogicalAccess, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId, RegionProgram,
    ScalarProgram, ScheduledRegionBuilder, TensorRole, VerifiedScheduledRegion, element_count,
};
use crate::shape::{Axis, Shape};

fn live_contraction_region(id: RegionId) -> VerifiedScheduledRegion {
    let left = Shape::from_dims([2]);
    let right = Shape::from_dims([3]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([]);
    let output_elements = element_count(&output).unwrap();
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output.clone()).unwrap();
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).unwrap();
        let tensor = TensorRole::Input;
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: numerical(),
        })
        .unwrap();
    builder
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
        .unwrap();
    builder.build().unwrap()
}

fn live_contraction_loop(kernel: &VerifiedKernel) -> SerialLoopRef<'_> {
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("a guarded live contraction");
    guarded
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some(reduction),
            _ => None,
        })
        .expect("a live-bound contributor loop")
}

fn count_live_input_loads(
    block: BlockRef<'_>,
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

fn live_input_load_sites(kernel: &VerifiedKernel) -> (u64, u64) {
    let live_buffer = kernel
        .declared_buffers()
        .find_map(|(id, buffer)| (buffer.tensor == TensorRole::Input).then_some(id))
        .expect("the named live input is a buffer");
    let mut seed = 0;
    let mut body = 0;
    count_live_input_loads(kernel.body(), live_buffer, &mut seed, &mut body, false);
    (seed, body)
}

fn live_contraction_loads(kernel: &VerifiedKernel, bound: u64) -> u64 {
    let (seed, body) = live_input_load_sites(kernel);
    let remaining = bound
        .checked_sub(1)
        .expect("preflight must refuse an empty strict contraction before execution");
    seed.checked_add(remaining.checked_mul(body).unwrap())
        .unwrap()
}

/// Neighbouring live extents move the load-count oracle and leave identity still.
#[test]
fn a_live_contraction_consumes_s_as_the_contributor_bound_without_baking_it() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let kernel = lower_scheduled_region(&scheduled).expect("the live contraction lowers");
    let extents: Vec<_> = kernel.input_extents().collect();
    assert_eq!(extents.len(), 1);
    assert_eq!(extents[0].access, AccessOrdinal::FIRST);
    assert_eq!(extents[0].axis, Axis::new(1));

    let reduction = live_contraction_loop(&kernel);
    assert!(
        matches!(reduction.start_bound(), LoopBound::Value(_)),
        "the fold start is the first-product constant, not a baked trip count"
    );
    assert!(
        matches!(reduction.end_bound(), LoopBound::Value(_)),
        "the contributor bound must be the live operand, not a literal S"
    );

    let loads_1 = live_contraction_loads(&kernel, 1);
    let loads_14 = live_contraction_loads(&kernel, 14);
    let loads_15 = live_contraction_loads(&kernel, 15);
    assert_eq!(
        loads_1, 1,
        "S = 1 must execute exactly the unseeded fold's first contributor"
    );
    assert_eq!(
        loads_14, 14,
        "S = 14 must perform exactly 14 loads of the live input"
    );
    assert_eq!(
        loads_15, 15,
        "S = 15 must perform exactly 15 loads of the live input"
    );
    assert_ne!(
        loads_14, loads_15,
        "neighbouring extents must move the load-count oracle"
    );

    let again = lower_scheduled_region(&live_contraction_region(RegionId::new(11))).unwrap();
    assert_eq!(
        kernel.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes(),
        "re-lowering the same live contraction must keep identity"
    );
    assert_eq!(
        scheduled.canonical_identity(),
        live_contraction_region(RegionId::new(12)).canonical_identity(),
        "the live value is excluded from schedule identity"
    );

    let baked_14 = lower_scheduled_region(&contraction_region(RegionId::new(9), 2, 3, 14)).unwrap();
    let baked_15 = lower_scheduled_region(&contraction_region(RegionId::new(9), 2, 3, 15)).unwrap();
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_14.canonical_identity().as_bytes(),
        "baking S = 14 must change identity"
    );
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking S = 15 must change identity"
    );
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking neighbouring extents must change identity"
    );
    assert!(
        kernel
            .canonical_identity()
            .as_bytes()
            .starts_with(b"tiler.kernel.v9\0"),
        "the live contraction stays on the current kernel domain"
    );
}

/// Omitting the scheduled live operand fails at kernel verification.
#[test]
fn an_omitted_live_contraction_extent_is_input_extent_contract() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let left_value = builder.load(left, invocation, BoundsWitnessId::new(0))?;
            let right_value = builder.load(right, invocation, BoundsWitnessId::new(1))?;
            let product = builder.binary(BinaryOp::F32Multiply, left_value, right_value)?;
            builder.store(
                write,
                invocation,
                product,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("omitting the live operand must not verify");
    assert_eq!(
        error.diagnostics()[0],
        KernelDiagnostic::InputExtentContract,
        "omitted live operand: {:?}",
        error.diagnostics()
    );
    assert_eq!(error.diagnostics()[0].rule(), "input-extent-contract");
}

/// Declaring the live operand and never reading it fails at verification.
#[test]
fn an_unused_live_contraction_extent_is_unused_input_extent() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let _left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let _right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let _write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(1),
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("an unread live operand must not verify");
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.rule() == "unused-input-extent"),
        "unused live operand: {:?}",
        error.diagnostics()
    );
}

/// An axis the scheduled live input does not have is refused at the builder.
#[test]
fn a_wrong_axis_live_contraction_extent_is_refused_at_declaration() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let error = builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(5),
        })
        .expect_err("axis 5 is outside the live input's rank");
    assert_eq!(error, KernelBuildError::InputExtentWrongAxis);
    assert_eq!(format!("{error:?}"), "InputExtentWrongAxis");
}

/// A live extent introduced inside the iteration predicate is too late to bind
/// the contributor loop.
#[test]
fn a_late_phase_live_contraction_extent_is_input_extent_contract() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let extent = builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(1),
        })
        .unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let bound = builder.input_extent(extent)?;
            let start = builder.constant(KernelConstant::Index(1))?;
            let left_value = builder.load(left, invocation, BoundsWitnessId::new(0))?;
            let right_value = builder.load(right, invocation, BoundsWitnessId::new(1))?;
            let product = builder.binary(BinaryOp::F32Multiply, left_value, right_value)?;
            let seed = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
            let results =
                builder.serial_loop_range(start, bound, &[seed], |_builder, parameters| {
                    let accumulator = parameters
                        .accumulator(0)
                        .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                    Ok(vec![accumulator])
                })?;
            let total = results
                .get(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            builder.store(
                write,
                invocation,
                total,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("a live operand read inside the predicate is too late");
    assert!(
        error.diagnostics().iter().any(|diagnostic| {
            diagnostic.rule() == "input-extent-contract"
                || diagnostic.rule() == "reduction-contract"
        }),
        "late-phase live operand: {:?}",
        error.diagnostics()
    );
}
