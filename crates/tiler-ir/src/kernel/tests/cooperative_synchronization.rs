use super::super::{
    AddressSpace, BarrierSpec, BinaryOp, Builtin, CompareOp, KernelBuildError, KernelBuilder,
    KernelConstant, KernelDiagnostic, KernelType, OperationRef, OperationView, SerialLoopSpec,
    lower_scheduled_region,
};
use super::support::{
    COOPERATIVE_STAGING, cooperative_barrier, cooperative_diagnostic, cooperative_region,
    cooperative_signature, multi_round_cooperative_region, staged_accesses,
};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, PhaseId, SyncPointId};

/// One deliberate deviation from the correct cooperative body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyChange {
    /// The body the rules below are measured against.
    None,
    /// The fence moves inside the iteration guard.
    FenceInsideTheGuard,
    /// The fence is omitted.
    NoFence,
    /// The staged read is emitted ahead of the fence.
    ReadBeforeTheFence,
    /// The fence names a point the region's tile does not declare.
    UnknownPoint,
    /// The fence fences device memory rather than workgroup memory.
    DeviceFence,
    /// The staged store names the phase that declares no write.
    WrongPhase,
}

/// Hand-builds a cooperative body carrying exactly one deviation.
///
/// The body is deliberately *not* the canonical one — it folds nothing — so the
/// unchanged case fails at the reduction contract. That is the control: each
/// change below moves the diagnostic to its own synchronization rule, which is
/// what proves the rule fired rather than something upstream of it.
fn cooperative_body(change: BodyChange) -> KernelDiagnostic {
    let scheduled = cooperative_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = cooperative_signature(&mut builder, &scheduled);
    let staging = builder.declare_staging(COOPERATIVE_STAGING).unwrap();

    let fence = |builder: &mut KernelBuilder| {
        let spec = match change {
            BodyChange::UnknownPoint => BarrierSpec {
                point: SyncPointId::new(1),
                ..cooperative_barrier()
            },
            BodyChange::DeviceFence => BarrierSpec {
                fenced_spaces: vec![AddressSpace::Device],
                ..cooperative_barrier()
            },
            _ => cooperative_barrier(),
        };
        builder.barrier(spec)
    };

    let gid = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let lid = builder.builtin(Builtin::LocalInvocationIndex).unwrap();
    let participants = builder.constant(KernelConstant::Index(3)).unwrap();
    let output = builder
        .binary(BinaryOp::IndexDivide, gid, participants)
        .unwrap();
    let extent = builder.constant(KernelConstant::Index(6)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, gid, extent)
        .unwrap();
    builder
        .predicated(active, |builder| {
            let value = builder.load(read, gid, BoundsWitnessId::new(0))?;
            let phase = if change == BodyChange::WrongPhase {
                PhaseId::new(1)
            } else {
                PhaseId::FIRST
            };
            builder.staged_store(staging, lid, value, phase)?;
            if change == BodyChange::FenceInsideTheGuard {
                fence(builder)?;
            }
            Ok(())
        })
        .unwrap();
    if change == BodyChange::ReadBeforeTheFence {
        builder
            .predicated(active, |builder| {
                let zero = builder.constant(KernelConstant::Index(0))?;
                builder.staged_load(staging, zero, PhaseId::new(1))?;
                Ok(())
            })
            .unwrap();
    }
    if !matches!(
        change,
        BodyChange::NoFence | BodyChange::FenceInsideTheGuard
    ) {
        fence(&mut builder).unwrap();
    }
    builder
        .predicated(active, |builder| {
            let one = builder.constant(KernelConstant::Index(1))?;
            let commits = builder.compare(CompareOp::IndexLessThan, lid, one)?;
            builder.predicated(commits, |builder| {
                let zero = builder.constant(KernelConstant::Index(0))?;
                let staged = builder.staged_load(staging, zero, PhaseId::new(1))?;
                builder.store(
                    write,
                    output,
                    staged,
                    BoundsWitnessId::new(1),
                    OwnershipWitnessId::new(0),
                )
            })
        })
        .unwrap();
    cooperative_diagnostic(builder)
}

/// Every synchronization rule of the structured-kernel verifier, driven once.
#[test]
fn each_kernel_synchronization_rule_refuses_its_own_defect() {
    // The control. An unchanged body reaches the reduction contract, so every
    // row below is evidence that its own rule fired first.
    assert_eq!(
        cooperative_body(BodyChange::None),
        KernelDiagnostic::ReductionContract
    );
    for (change, expected) in [
        (
            BodyChange::FenceInsideTheGuard,
            KernelDiagnostic::SynchronizationConvergence,
        ),
        (
            BodyChange::NoFence,
            KernelDiagnostic::UndischargedVisibility,
        ),
        (
            BodyChange::ReadBeforeTheFence,
            KernelDiagnostic::UnorderedStagedHandoff,
        ),
        (
            BodyChange::UnknownPoint,
            KernelDiagnostic::UnexpectedSynchronization,
        ),
        (
            BodyChange::DeviceFence,
            KernelDiagnostic::SynchronizationContract,
        ),
        (
            BodyChange::WrongPhase,
            KernelDiagnostic::StagedAccessEvidence,
        ),
    ] {
        assert_eq!(cooperative_body(change), expected, "{change:?}");
    }
}

/// A loop-carried tile lowers to a peeled round zero and a round loop.
///
/// Every structural claim is asserted rather than described, because the shape
/// *is* the correctness argument: round zero is emitted ahead of the loop because
/// the fold seeds at its first contributor; the accumulator that carries the
/// round totals is defined at the kernel's top level because a predicated region
/// produces no value that could cross the back edge; and the round boundary sits
/// at the head of the loop body because that is the only position that also
/// orders the peeled round's reads against the loop's first rewrite.
#[test]
fn a_loop_carried_tile_lowers_to_a_peeled_round_body() {
    let scheduled = multi_round_cooperative_region();
    let tile = crate::schedule::cooperative_tile(&scheduled.region().schedule.reduction)
        .expect("the region carries a tile");
    assert_eq!(tile.anti_dependency_edges().len(), 1);
    let kernel = lower_scheduled_region(&scheduled).expect("the loop-carried region lowers");

    let top: Vec<_> = kernel.body().operations().map(OperationRef::view).collect();
    // One barrier at the top level: the peeled round's phase boundary. The round
    // boundary has no top-level realization at all, because `rounds` rounds have
    // `rounds - 1` transitions between them.
    let top_barriers: Vec<_> = top
        .iter()
        .filter_map(|view| match view {
            OperationView::Barrier { spec } => Some(*spec),
            _ => None,
        })
        .collect();
    assert_eq!(top_barriers.len(), 1);
    assert_eq!(top_barriers[0].point, SyncPointId::FIRST);

    // The round loop runs `1..rounds` and carries exactly one `f32`, which is
    // the accumulator the peel seeded.
    let rounds: Vec<_> = top
        .iter()
        .filter_map(|view| match view {
            OperationView::SerialLoop(loops) if loops.end() == 2 => Some(*loops),
            _ => None,
        })
        .collect();
    let [round] = rounds.as_slice() else {
        panic!("expected exactly one round loop at the top level")
    };
    assert_eq!(round.start(), 1);
    assert_eq!(round.accumulators().len(), 1);
    assert_eq!(
        kernel
            .value_type(round.accumulators().next().expect("one accumulator"))
            .expect("the accumulator resolves"),
        KernelType::F32
    );

    // The round boundary is the round body's first operation, ahead of the
    // guarded rewrite; the phase boundary follows it.
    let body: Vec<_> = round.body().operations().map(OperationRef::view).collect();
    let barriers: Vec<(usize, SyncPointId)> = body
        .iter()
        .enumerate()
        .filter_map(|(position, view)| match view {
            OperationView::Barrier { spec } => Some((position, spec.point)),
            _ => None,
        })
        .collect();
    assert_eq!(
        barriers,
        [(0, SyncPointId::new(1)), (2, SyncPointId::FIRST)]
    );
    assert!(matches!(body[1], OperationView::Predicated { .. }));

    // Both rounds stage and consume: two writes and four reads, the reads being
    // a seed and a folded slot in each of the peel and the loop body.
    let (writes, reads) = staged_accesses(&kernel);
    assert_eq!(writes, [PhaseId::FIRST; 2]);
    assert_eq!(reads, [PhaseId::new(1); 4]);
}

/// The barrier-convergence rule admits exactly the nesting a tile authorizes.
///
/// Driven over the predicate directly, because a body cannot reach every row:
/// the depths a canonical body emits are a subset of the admitted ones, and a
/// rule with only its refusals driven would be half-evidenced. The refusals are
/// additionally driven end to end through real bodies by
/// `each_kernel_synchronization_rule_refuses_its_own_defect`'s
/// `FenceInsideTheGuard` row and by
/// `each_loop_carried_synchronization_rule_refuses_its_own_defect`.
#[test]
fn the_barrier_convergence_rule_admits_only_the_nesting_a_tile_authorizes() {
    for (block_depth, loop_depth, rounds, admitted) in [
        // A single-round tile authorizes the top level and nothing else.
        (0, 0, 1, true),
        (1, 0, 1, false),
        (1, 1, 1, false),
        (2, 1, 1, false),
        // A loop-carried tile authorizes the round loop *and* the top level,
        // because the fold's seed peels round zero out of the loop and its
        // barrier is realized there. What stops a stray top-level barrier from
        // riding on that is the realization count, not this predicate.
        (1, 1, 2, true),
        (0, 0, 2, true),
        (2, 2, 2, false),
        // A predicate on the path is refused whatever the round count: the
        // difference between the two depths counts the predicates, and any of
        // them admits a dynamic subset of the participants.
        (1, 0, 2, false),
        (2, 1, 2, false),
        (3, 1, 2, false),
    ] {
        assert_eq!(
            super::super::verify::barrier_is_convergent(block_depth, loop_depth, rounds),
            admitted,
            "block {block_depth}, loop {loop_depth}, rounds {rounds}"
        );
    }
}

/// The barrier realizing the loop-carried fixture's round boundary.
fn cooperative_round_barrier() -> BarrierSpec {
    BarrierSpec {
        point: SyncPointId::new(1),
        ..cooperative_barrier()
    }
}

/// One deliberate deviation from a loop-carried cooperative body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoopCarriedChange {
    /// The body the rules below are measured against.
    None,
    /// The round boundary moves to the end of the round body.
    RoundBoundaryAtTheTail,
    /// The round boundary is omitted.
    NoRoundBoundary,
    /// The round boundary is additionally realized in the peeled round.
    RoundBoundaryInThePeel,
    /// The round loop runs `0..rounds`, as a body with no peel would.
    UnpeeledRoundLoop,
    /// The peel's fence sits inside a loop that is not the round loop.
    FenceInAnotherLoop,
    /// The round body reads staging ahead of its own phase boundary.
    ReadBeforeTheFence,
}

/// Hand-builds a loop-carried cooperative body carrying exactly one deviation.
///
/// The body is deliberately *not* the canonical one — it folds nothing, so both
/// its per-round contributor fold and its staged folds are missing — which makes
/// the unchanged case fail at the reduction contract. That is the control: each
/// change below moves the diagnostic to its own rule, which is what proves the
/// rule fired rather than something upstream of it.
fn loop_carried_body(change: LoopCarriedChange) -> KernelDiagnostic {
    let scheduled = multi_round_cooperative_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = cooperative_signature(&mut builder, &scheduled);
    let staging = builder.declare_staging(COOPERATIVE_STAGING).unwrap();

    let gid = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let lid = builder.builtin(Builtin::LocalInvocationIndex).unwrap();
    let participants = builder.constant(KernelConstant::Index(3)).unwrap();
    let output = builder
        .binary(BinaryOp::IndexDivide, gid, participants)
        .unwrap();
    let extent = builder.constant(KernelConstant::Index(6)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, gid, extent)
        .unwrap();
    let zero = builder.constant(KernelConstant::Index(0)).unwrap();

    // The producing phase, emitted identically in the peel and in the loop.
    let produce = move |builder: &mut KernelBuilder| {
        builder.predicated(active, move |builder| {
            let value = builder.load(read, gid, BoundsWitnessId::new(0))?;
            builder.staged_store(staging, lid, value, PhaseId::FIRST)
        })
    };

    produce(&mut builder).unwrap();
    if change == LoopCarriedChange::FenceInAnotherLoop {
        // A loop at the kernel's top level that is not the round loop, standing
        // in for a contributor fold. Its accumulator is a constant rather than a
        // staged read, so the only rule this row can trip is the round-loop one.
        let accumulator = builder.constant(KernelConstant::F32Bits(0)).unwrap();
        builder
            .serial_loop(
                SerialLoopSpec { start: 1, end: 3 },
                &[accumulator],
                |builder, parameters| {
                    builder.barrier(cooperative_barrier())?;
                    Ok(vec![
                        parameters
                            .accumulator(0)
                            .ok_or(KernelBuildError::EmptyLoopAccumulators)?,
                    ])
                },
            )
            .unwrap();
    } else {
        builder.barrier(cooperative_barrier()).unwrap();
    }
    if change == LoopCarriedChange::RoundBoundaryInThePeel {
        builder.barrier(cooperative_round_barrier()).unwrap();
    }
    let seed = builder.staged_load(staging, zero, PhaseId::new(1)).unwrap();
    let start = u64::from(change != LoopCarriedChange::UnpeeledRoundLoop);
    let results = builder
        .serial_loop(
            SerialLoopSpec { start, end: 2 },
            &[seed],
            move |builder, parameters| {
                let accumulator = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                if !matches!(
                    change,
                    LoopCarriedChange::NoRoundBoundary | LoopCarriedChange::RoundBoundaryAtTheTail
                ) {
                    builder.barrier(cooperative_round_barrier())?;
                }
                produce(builder)?;
                if change == LoopCarriedChange::ReadBeforeTheFence {
                    builder.staged_load(staging, zero, PhaseId::new(1))?;
                }
                builder.barrier(cooperative_barrier())?;
                let staged = builder.staged_load(staging, zero, PhaseId::new(1))?;
                let sum = builder.binary(BinaryOp::F32Add, accumulator, staged)?;
                if change == LoopCarriedChange::RoundBoundaryAtTheTail {
                    builder.barrier(cooperative_round_barrier())?;
                }
                Ok(vec![sum])
            },
        )
        .unwrap();
    let total = results.get(0).unwrap();
    builder
        .predicated(active, |builder| {
            let one = builder.constant(KernelConstant::Index(1))?;
            let commits = builder.compare(CompareOp::IndexLessThan, lid, one)?;
            builder.predicated(commits, |builder| {
                builder.store(
                    write,
                    output,
                    total,
                    BoundsWitnessId::new(1),
                    OwnershipWitnessId::new(0),
                )
            })
        })
        .unwrap();
    cooperative_diagnostic(builder)
}

/// Every rule the round structure adds, driven once against its own defect.
#[test]
fn each_loop_carried_synchronization_rule_refuses_its_own_defect() {
    // The control. An unchanged body reaches the reduction contract, so every
    // row below is evidence that its own rule fired first.
    assert_eq!(
        loop_carried_body(LoopCarriedChange::None),
        KernelDiagnostic::ReductionContract
    );
    for (change, expected) in [
        // The cyclic rule's `b < w` arm is the only one that also orders the
        // peeled round's reads against the loop's first rewrite, so a boundary
        // at the tail satisfies the back edge and still leaves a race.
        (
            LoopCarriedChange::RoundBoundaryAtTheTail,
            KernelDiagnostic::UnorderedStagedRewrite,
        ),
        (
            LoopCarriedChange::NoRoundBoundary,
            KernelDiagnostic::UndischargedAntiDependency,
        ),
        // `rounds` rounds have `rounds - 1` transitions, so a round boundary
        // realized in the peel as well is realized once too often.
        (
            LoopCarriedChange::RoundBoundaryInThePeel,
            KernelDiagnostic::SynchronizationRealization,
        ),
        // The trip-count obligation: the enclosing loop must be the round loop,
        // and a `0..rounds` loop is the shape a body with no peel would emit.
        (
            LoopCarriedChange::UnpeeledRoundLoop,
            KernelDiagnostic::SynchronizationConvergence,
        ),
        (
            LoopCarriedChange::FenceInAnotherLoop,
            KernelDiagnostic::SynchronizationConvergence,
        ),
        (
            LoopCarriedChange::ReadBeforeTheFence,
            KernelDiagnostic::UnorderedStagedHandoff,
        ),
    ] {
        assert_eq!(loop_carried_body(change), expected, "{change:?}");
    }
}
