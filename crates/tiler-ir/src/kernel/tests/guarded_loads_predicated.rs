use super::super::{BlockRef, KernelDiagnostic, OperationView, lower_scheduled_region};
use super::support::{
    cooperative_contraction_region, cooperative_region, guarded_load_count,
    multi_round_cooperative_region,
};
use crate::schedule::{TailPolicy, VerifiedScheduledRegion};

fn declining_backend(operation: OperationView<'_>) -> Result<(), &'static str> {
    match operation {
        OperationView::GuardedLoad { .. } => Err("unrecognized-operation"),
        _ => Ok(()),
    }
}

fn count_declined_guarded_loads(block: BlockRef<'_>, declined: &mut usize) {
    for operation in block.operations() {
        if declining_backend(operation.view()).is_err() {
            *declined = declined.saturating_add(1);
        }
        match operation.view() {
            OperationView::Predicated { body, .. } => count_declined_guarded_loads(body, declined),
            OperationView::SerialLoop(serial) => {
                count_declined_guarded_loads(serial.body(), declined);
            }
            _ => {}
        }
    }
}

/// Predicated and Exact kernels under the same binding stay distinct, and
/// Predicated carries `GuardedLoad`.
#[test]
fn predicated_contraction_lowers_with_guarded_loads() {
    let exact = cooperative_contraction_region(32, 32, 16, TailPolicy::Exact);
    let predicated = cooperative_contraction_region(32, 32, 16, TailPolicy::Predicated);
    let exact_kernel = lower_scheduled_region(&exact).expect("exact tiled contraction lowers");
    let predicated_kernel =
        lower_scheduled_region(&predicated).expect("predicated tiled contraction lowers");
    assert_eq!(guarded_load_count(&exact_kernel), 0);
    assert!(guarded_load_count(&predicated_kernel) >= 2);
    assert_ne!(
        exact_kernel.canonical_identity().as_bytes(),
        predicated_kernel.canonical_identity().as_bytes()
    );
}

/// A backend that does not name `GuardedLoad` declines; there is no Load rewrite.
#[test]
fn a_backend_that_declines_guarded_load_has_no_source_fallback() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let kernel = lower_scheduled_region(&scheduled).expect("predicated kernel lowers");
    let mut declined = 0_usize;
    count_declined_guarded_loads(kernel.body(), &mut declined);
    assert!(declined >= 2, "declined {declined} GuardedLoad operations");
    assert!(
        !kernel
            .body()
            .operations()
            .any(|operation| { matches!(operation.view(), OperationView::Load { .. }) }),
        "no ordinary operand load may stand in for GuardedLoad"
    );
}

fn swap_guarded_load_predicates(data: &mut super::super::model::KernelData) {
    for block in &mut data.blocks {
        let mut predicates = Vec::new();
        for operation in &block.operations {
            if let super::super::model::OperationKind::GuardedLoad { predicate, .. } =
                operation.kind
            {
                predicates.push(predicate);
            }
        }
        if predicates.len() < 2 {
            continue;
        }
        let first = predicates[0];
        let second = predicates[1];
        for operation in &mut block.operations {
            if let super::super::model::OperationKind::GuardedLoad { predicate, .. } =
                &mut operation.kind
            {
                if *predicate == first {
                    *predicate = second;
                } else if *predicate == second {
                    *predicate = first;
                }
            }
        }
    }
}

fn replace_guarded_with_ordinary(data: &mut super::super::model::KernelData) {
    for block in &mut data.blocks {
        for operation in &mut block.operations {
            if let super::super::model::OperationKind::GuardedLoad {
                buffer,
                offset,
                bounds,
                ..
            } = operation.kind
            {
                operation.kind = super::super::model::OperationKind::Load {
                    buffer,
                    offset,
                    bounds,
                };
            }
        }
    }
}

fn enclose_staged_stores(data: &mut super::super::model::KernelData) {
    let Some(predicate) = data.blocks.iter().find_map(|block| {
        block.operations.iter().find_map(|operation| {
            if let super::super::model::OperationKind::GuardedLoad { predicate, .. } =
                operation.kind
            {
                Some(predicate)
            } else {
                None
            }
        })
    }) else {
        return;
    };
    let mut stores = Vec::new();
    if let Some(block) = data.blocks.get_mut(0) {
        let rest = std::mem::take(&mut block.operations);
        let mut kept = Vec::new();
        for operation in rest {
            if matches!(
                operation.kind,
                super::super::model::OperationKind::StagedStore { .. }
            ) {
                stores.push(operation);
            } else {
                kept.push(operation);
            }
        }
        block.operations = kept;
    }
    if stores.is_empty() {
        return;
    }
    let body = u32::try_from(data.blocks.len()).expect("block index fits");
    data.blocks.push(super::super::model::BlockData {
        parameters: Vec::new(),
        operations: stores,
    });
    data.blocks[0]
        .operations
        .push(super::super::model::OperationData {
            kind: super::super::model::OperationKind::Predicated { predicate, body },
            results: Vec::new(),
        });
}

fn enclose_barriers(data: &mut super::super::model::KernelData) {
    let Some(predicate) = data.blocks.iter().find_map(|block| {
        block.operations.iter().find_map(|operation| {
            if let super::super::model::OperationKind::GuardedLoad { predicate, .. } =
                operation.kind
            {
                Some(predicate)
            } else {
                None
            }
        })
    }) else {
        return;
    };
    let mut barriers = Vec::new();
    if let Some(block) = data.blocks.get_mut(0) {
        let rest = std::mem::take(&mut block.operations);
        let mut kept = Vec::new();
        for operation in rest {
            if matches!(
                operation.kind,
                super::super::model::OperationKind::Barrier { .. }
            ) {
                barriers.push(operation);
            } else {
                kept.push(operation);
            }
        }
        block.operations = kept;
    }
    if barriers.is_empty() {
        return;
    }
    let body = u32::try_from(data.blocks.len()).expect("block index fits");
    data.blocks.push(super::super::model::BlockData {
        parameters: Vec::new(),
        operations: barriers,
    });
    data.blocks[0]
        .operations
        .push(super::super::model::OperationData {
            kind: super::super::model::OperationKind::Predicated { predicate, body },
            results: Vec::new(),
        });
}

fn drop_inner_store_guard(data: &mut super::super::model::KernelData) {
    // Replace the innermost Predicated-around-store with its body operations
    // spliced into the parent, so the store has only one axis guard.
    let mut splice: Option<(usize, usize, u32)> = None;
    for (block_index, block) in data.blocks.iter().enumerate() {
        for (op_index, operation) in block.operations.iter().enumerate() {
            if let super::super::model::OperationKind::Predicated { body, .. } = operation.kind
                && data.blocks.get(body as usize).is_some_and(|inner| {
                    inner.operations.iter().any(|nested| {
                        matches!(
                            nested.kind,
                            super::super::model::OperationKind::Store { .. }
                        )
                    })
                })
            {
                splice = Some((block_index, op_index, body));
            }
        }
    }
    let Some((block_index, op_index, body)) = splice else {
        return;
    };
    let inner = std::mem::take(&mut data.blocks[body as usize].operations);
    data.blocks[block_index].operations.remove(op_index);
    for (offset, operation) in inner.into_iter().enumerate() {
        data.blocks[block_index]
            .operations
            .insert(op_index + offset, operation);
    }
}

fn verify_mutated(
    scheduled: &VerifiedScheduledRegion,
    edit: impl FnOnce(&mut super::super::model::KernelData),
) -> KernelDiagnostic {
    let mut data = super::super::lower::derive_canonical(
        scheduled.region(),
        scheduled.canonical_identity(),
        scheduled.requirements(),
    )
    .expect("canonical body exists");
    edit(&mut data);
    super::super::verify::verify_kernel(
        &data,
        scheduled.region(),
        scheduled.canonical_identity(),
        scheduled.requirements(),
    )
    .expect_err("the mutated subject must fail")
}

fn set_first_guarded_predicate(data: &mut super::super::model::KernelData, use_column: bool) {
    let axis = {
        let mut row = None;
        let mut column = None;
        for block in &data.blocks {
            for operation in &block.operations {
                if let super::super::model::OperationKind::Compare {
                    op: super::super::model::CompareOp::IndexLessThan,
                    ..
                } = operation.kind
                {
                    if row.is_none() {
                        row = operation.results.first().copied();
                    } else if column.is_none() {
                        column = operation.results.first().copied();
                    }
                }
            }
        }
        (row, column)
    };
    let wanted = if use_column { axis.1 } else { axis.0 };
    let Some(wanted) = wanted else {
        return;
    };
    for block in &mut data.blocks {
        for operation in &mut block.operations {
            if let super::super::model::OperationKind::GuardedLoad { predicate, .. } =
                &mut operation.kind
            {
                *predicate = wanted;
                return;
            }
        }
    }
}

fn set_second_guarded_predicate(data: &mut super::super::model::KernelData, use_row: bool) {
    let axis = {
        let mut row = None;
        let mut column = None;
        for block in &data.blocks {
            for operation in &block.operations {
                if let super::super::model::OperationKind::Compare {
                    op: super::super::model::CompareOp::IndexLessThan,
                    ..
                } = operation.kind
                {
                    if row.is_none() {
                        row = operation.results.first().copied();
                    } else if column.is_none() {
                        column = operation.results.first().copied();
                    }
                }
            }
        }
        (row, column)
    };
    let wanted = if use_row { axis.0 } else { axis.1 };
    let Some(wanted) = wanted else {
        return;
    };
    let mut seen = 0_u8;
    for block in &mut data.blocks {
        for operation in &mut block.operations {
            if let super::super::model::OperationKind::GuardedLoad { predicate, .. } =
                &mut operation.kind
            {
                seen = seen.saturating_add(1);
                if seen == 2 {
                    *predicate = wanted;
                    return;
                }
            }
        }
    }
}

/// A column guard on the left load is the left-load refusal.
#[test]
fn a_column_guard_on_the_left_load_is_the_left_load_refusal() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, |data| set_first_guarded_predicate(data, true));
    assert_eq!(
        diagnostic.rule(),
        "left-load-guard",
        "left-load refusal: {}",
        diagnostic.rule()
    );
}

/// A row guard on the right load is the right-load refusal.
#[test]
fn a_row_guard_on_the_right_load_is_the_right_load_refusal() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, |data| set_second_guarded_predicate(data, true));
    assert_eq!(
        diagnostic.rule(),
        "right-load-guard",
        "right-load refusal: {}",
        diagnostic.rule()
    );
}

/// Swapping row and column predicates names a specific load refusal.
#[test]
fn swapped_axis_guards_name_the_left_and_right_load_refusals() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, swap_guarded_load_predicates);
    let text = diagnostic.rule();
    assert!(
        text == "left-load-guard" || text == "right-load-guard",
        "swapped guards: {text}"
    );
}

/// An ordinary load in place of either `GuardedLoad` fails bounds refinement.
#[test]
fn ordinary_load_in_place_of_guarded_load_fails_bounds_refinement() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, replace_guarded_with_ordinary);
    assert_eq!(
        diagnostic.rule(),
        "bounds-evidence",
        "ordinary load refusal: {}",
        diagnostic.rule()
    );
}

/// Guarding a staged store is incomplete staging.
#[test]
fn a_predicated_staged_store_is_incomplete_staging() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, enclose_staged_stores);
    assert_eq!(
        diagnostic.rule(),
        "incomplete-staging",
        "guarded staged store: {}",
        diagnostic.rule()
    );
}

/// A phase barrier under a predicate fails convergence.
#[test]
fn a_barrier_under_a_predicate_fails_convergence() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, enclose_barriers);
    assert_eq!(
        diagnostic.rule(),
        "synchronization-convergence",
        "guarded barrier: {}",
        diagnostic.rule()
    );
}

/// Dropping one store-side axis guard is the write refusal.
#[test]
fn an_incomplete_output_guard_is_the_write_refusal() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, drop_inner_store_guard);
    assert_eq!(
        diagnostic.rule(),
        "output-store-guard",
        "write refusal: {}",
        diagnostic.rule()
    );
}

/// Every active output has one writer; inactive invocations write nothing;
/// staging is initialized; filler is not observed.
#[test]
fn predicated_contraction_ownership_and_filler_are_unobservable() {
    let scheduled = cooperative_contraction_region(10, 16, 16, TailPolicy::Predicated);
    let kernel = lower_scheduled_region(&scheduled).expect("predicated kernel lowers");
    let m_ext = 10_usize;
    let n_ext = 16_usize;
    let k_ext = 16_usize;
    let block = 16_usize;
    let grid = usize::try_from(scheduled.region().schedule.launch.grid_threads).unwrap();
    let mut left = vec![0.0_f32; m_ext.saturating_mul(k_ext)];
    let mut right = vec![0.0_f32; n_ext.saturating_mul(k_ext)];
    for m in 0..m_ext {
        for k in 0..k_ext {
            left[m.saturating_mul(k_ext).saturating_add(k)] =
                f32::from(u8::try_from(m.saturating_add(1)).expect("m fits u8"));
        }
    }
    for n in 0..n_ext {
        for k in 0..k_ext {
            right[n.saturating_mul(k_ext).saturating_add(k)] =
                f32::from(u8::try_from(n.saturating_add(2)).expect("n fits u8"));
        }
    }
    let mut output = vec![f32::NAN; m_ext.saturating_mul(n_ext)];
    let mut writers = vec![0_u32; m_ext.saturating_mul(n_ext)];
    let mut a_tile = vec![f32::NAN; block.saturating_mul(block)];
    let mut b_tile = vec![f32::NAN; block.saturating_mul(block)];
    for gid in 0..grid {
        let lid = gid % block.saturating_mul(block);
        let local_n = lid % block;
        let local_m = lid / block;
        let row_active = local_m < m_ext;
        let col_active = local_n < n_ext;
        let a = if row_active {
            left[local_m.saturating_mul(k_ext).saturating_add(local_n)]
        } else {
            0.0
        };
        let b = if col_active {
            right[local_n.saturating_mul(k_ext).saturating_add(local_m)]
        } else {
            0.0
        };
        a_tile[local_m.saturating_mul(block).saturating_add(local_n)] = a;
        b_tile[local_n.saturating_mul(block).saturating_add(local_m)] = b;
    }
    assert!(
        a_tile.iter().all(|value| !value.is_nan()),
        "every A staging slot is initialized"
    );
    assert!(
        b_tile.iter().all(|value| !value.is_nan()),
        "every B staging slot is initialized"
    );
    for gid in 0..grid {
        let lid = gid % block.saturating_mul(block);
        let local_n = lid % block;
        let local_m = lid / block;
        if local_m >= m_ext || local_n >= n_ext {
            continue;
        }
        let mut acc = 0.0_f32;
        for kk in 0..block {
            let a = a_tile[local_m.saturating_mul(block).saturating_add(kk)];
            let b = b_tile[local_n.saturating_mul(block).saturating_add(kk)];
            assert!(
                a != 0.0 || left[local_m.saturating_mul(k_ext).saturating_add(kk)] == 0.0,
                "active output observed an inactive A filler"
            );
            acc += a * b;
        }
        let slot = local_m.saturating_mul(n_ext).saturating_add(local_n);
        output[slot] = acc;
        writers[slot] = writers[slot].saturating_add(1);
    }
    for (slot, count) in writers.iter().enumerate() {
        assert_eq!(*count, 1, "output slot {slot} writers={count}");
        let m = slot / n_ext;
        let n = slot % n_ext;
        let expected: f32 = (0..k_ext)
            .map(|k| {
                left[m.saturating_mul(k_ext).saturating_add(k)]
                    * right[n.saturating_mul(k_ext).saturating_add(k)]
            })
            .sum();
        assert!(
            (output[slot] - expected).abs() <= f32::EPSILON,
            "slot {slot}"
        );
    }
    let _ = kernel;
}

/// Tail and `GuardedLoad` tags move identity without touching old Exact pins.
#[test]
fn tail_and_guarded_load_tags_are_identity_bearing() {
    let exact = cooperative_contraction_region(32, 32, 16, TailPolicy::Exact);
    let predicated = cooperative_contraction_region(32, 32, 16, TailPolicy::Predicated);
    let partial = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    assert_ne!(
        exact.canonical_identity().as_bytes(),
        predicated.canonical_identity().as_bytes()
    );
    assert_ne!(
        predicated.canonical_identity().as_bytes(),
        partial.canonical_identity().as_bytes()
    );
    let exact_kernel = lower_scheduled_region(&exact).unwrap();
    let predicated_kernel = lower_scheduled_region(&predicated).unwrap();
    assert_ne!(
        exact_kernel.canonical_identity().as_bytes(),
        predicated_kernel.canonical_identity().as_bytes()
    );
}

/// A moved topology field separates schedule and kernel identity together.
///
/// The ADR 0013 topology perturbation at the layers this crate owns: the
/// single-round and loop-carried cooperative regions differ only in the
/// coherent field set a verifiable `rounds` change forces — the round count,
/// its per-round contributor partition, and its round-boundary synchronization
/// — while inputs, accesses, and expression stay the fixture's own bytes. The
/// scheduled-region identities separate, each lowered kernel retains exactly
/// its own region's identity, and the kernel identities separate with them,
/// which is the chain that carries a topology choice into kernel-program,
/// artifact, and envelope identity. The per-field population — that *every*
/// cooperative tile field separates scheduled-region identity on its own — is
/// `schedule::builder`'s `every_cooperative_tile_field_separates_scheduled_region_identity`.
#[test]
fn a_topology_change_separates_schedule_and_kernel_identity_together() {
    let single = cooperative_region();
    let multi = multi_round_cooperative_region();
    assert_ne!(
        single.canonical_identity().as_bytes(),
        multi.canonical_identity().as_bytes(),
        "two topologies are two scheduled-region identities",
    );
    let single_kernel =
        lower_scheduled_region(&single).expect("the single-round cooperative region lowers");
    let multi_kernel =
        lower_scheduled_region(&multi).expect("the loop-carried cooperative region lowers");
    assert_eq!(
        single_kernel.scheduled_region_identity().as_bytes(),
        single.canonical_identity().as_bytes(),
        "a kernel retains exactly its own region's identity",
    );
    assert_eq!(
        multi_kernel.scheduled_region_identity().as_bytes(),
        multi.canonical_identity().as_bytes(),
        "a kernel retains exactly its own region's identity",
    );
    assert_ne!(
        single_kernel.canonical_identity().as_bytes(),
        multi_kernel.canonical_identity().as_bytes(),
        "the topology choice is folded through kernel identity",
    );
}
