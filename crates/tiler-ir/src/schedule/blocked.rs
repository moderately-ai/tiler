//! Blocked-workgroup admission and coverage proof for cooperative contraction.
//!
//! Exact output blocks keep [`admit_exact_cooperative_contraction`]. Partial
//! `[M, N]` blocks use [`admit_predicated_cooperative_contraction`], which
//! ceilings the workgroup grid and never rewrites itself to Exact or direct.
//! Contracted tiles stay exact-divisible on both paths.

use crate::shape::Shape;

use super::error::{
    BlockedWorkgroupRule, CooperativeContractionAdmission, ScheduledRegionDiagnostic,
};
use super::model::ExecutionBinding;

/// Proven exact-divisible facts a cooperative-contraction schedule may carry.
///
/// Constructed only by [`admit_exact_cooperative_contraction`]. The binding is
/// required rather than defaulted: a caller that ignores this record and
/// assembles `GlobalLinearInvocation` is refused by the intrinsic verifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactCooperativeContraction {
    /// Hardware-to-logical map over the output domain.
    pub binding: ExecutionBinding,
    /// Exact tile of the contracted iteration space.
    pub contracted_tile: Shape,
    /// Times the tile must repeat to cover the contracted space.
    pub rounds: u64,
}

/// Proves the exact-divisibility equalities before a schedule is assembled.
///
/// # Errors
///
/// Returns a typed refusal naming the missing equality. Never returns a direct
/// [`super::ReductionTopology::Contraction`] schedule.
pub fn admit_exact_cooperative_contraction(
    output: &Shape,
    output_block: &Shape,
    contracted: &Shape,
    contracted_tile: &Shape,
) -> Result<ExactCooperativeContraction, CooperativeContractionAdmission> {
    let workgroups = exact_quotients(output, output_block, AxisKind::Output)?;
    let tile_counts = exact_quotients(contracted, contracted_tile, AxisKind::Contracted)?;
    let rounds = tile_counts
        .extents()
        .iter()
        .try_fold(1_u64, |total, extent| total.checked_mul(extent.get()))
        .ok_or(CooperativeContractionAdmission::ShapeProductOverflow)?;
    if rounds == 0 {
        return Err(CooperativeContractionAdmission::ShapeProductOverflow);
    }
    Ok(ExactCooperativeContraction {
        binding: ExecutionBinding::BlockedWorkgroup {
            block: output_block.clone(),
            workgroups,
        },
        contracted_tile: contracted_tile.clone(),
        rounds,
    })
}

/// Proven predicated-tail facts a cooperative-contraction schedule may carry.
///
/// Constructed only by [`admit_predicated_cooperative_contraction`]. The launch
/// may be a strict superset of the logical output; active coordinates are
/// derived from the blocked binding. The function never returns an Exact tail
/// or a direct contraction, including when `[M, N]` happens to divide.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicatedCooperativeContraction {
    /// Hardware-to-logical map over the padded workgroup grid.
    pub binding: ExecutionBinding,
    /// Exact tile of the contracted iteration space.
    pub contracted_tile: Shape,
    /// Times the tile must repeat to cover the contracted space.
    pub rounds: u64,
    /// Logical output population `M × N`.
    pub work_items: u64,
    /// Padded launch population `ceil(M/B_m) × ceil(N/B_n) × B_m × B_n`.
    pub grid_threads: u64,
}

/// Proves a padded blocked launch before a predicated schedule is assembled.
///
/// Output extents need not divide the block. Contracted extents must still
/// divide the tile. Checked ceiling and multiplication refuse overflow. A
/// divisible `[M, N]` is still Predicated: this function never normalizes to
/// Exact.
///
/// # Errors
///
/// Returns a typed refusal naming the missing equality or overflow. Never
/// returns a direct [`super::ReductionTopology::Contraction`] schedule.
pub fn admit_predicated_cooperative_contraction(
    output: &Shape,
    output_block: &Shape,
    contracted: &Shape,
    contracted_tile: &Shape,
) -> Result<PredicatedCooperativeContraction, CooperativeContractionAdmission> {
    let workgroups = ceiling_quotients(output, output_block, AxisKind::Output)?;
    let tile_counts = exact_quotients(contracted, contracted_tile, AxisKind::Contracted)?;
    let rounds = tile_counts
        .extents()
        .iter()
        .try_fold(1_u64, |total, extent| total.checked_mul(extent.get()))
        .ok_or(CooperativeContractionAdmission::ShapeProductOverflow)?;
    if rounds == 0 {
        return Err(CooperativeContractionAdmission::ShapeProductOverflow);
    }
    let work_items =
        shape_product(output).ok_or(CooperativeContractionAdmission::ShapeProductOverflow)?;
    let Some(block_product) = shape_product(output_block) else {
        return Err(CooperativeContractionAdmission::ShapeProductOverflow);
    };
    let Some(workgroup_product) = shape_product(&workgroups) else {
        return Err(CooperativeContractionAdmission::ShapeProductOverflow);
    };
    let grid_threads = workgroup_product
        .checked_mul(block_product)
        .ok_or(CooperativeContractionAdmission::ShapeProductOverflow)?;
    Ok(PredicatedCooperativeContraction {
        binding: ExecutionBinding::BlockedWorkgroup {
            block: output_block.clone(),
            workgroups,
        },
        contracted_tile: contracted_tile.clone(),
        rounds,
        work_items,
        grid_threads,
    })
}

/// Proves the blocked map is a bijection from launched invocations onto `output`.
///
/// Per-axis `workgroups[d] * block[d] == output[d]` is the theorem
/// [`super::OwnershipProofKind::OneGlobalInvocationPerOutput`] needs: a greater
/// product is two invocations claiming one coordinate, a lesser product leaves
/// a coordinate with no writer.
///
/// # Errors
///
/// Returns the named blocked-map rule the statement violated.
pub fn prove_blocked_bijection(
    output: &Shape,
    block: &Shape,
    workgroups: &Shape,
    work_items: u64,
    threads_per_workgroup: u32,
    grid_threads: u64,
) -> Result<(), ScheduledRegionDiagnostic> {
    if output.rank() != block.rank() || output.rank() != workgroups.rank() {
        return Err(blocked(BlockedWorkgroupRule::RankMismatch));
    }
    for (output_extent, block_extent, workgroup_extent) in output
        .extents()
        .iter()
        .zip(block.extents())
        .zip(workgroups.extents())
        .map(|((output, block), workgroups)| (output.get(), block.get(), workgroups.get()))
    {
        let Some(covered) = workgroup_extent.checked_mul(block_extent) else {
            return Err(blocked(BlockedWorkgroupRule::MappingOverlap));
        };
        if covered > output_extent {
            return Err(blocked(BlockedWorkgroupRule::MappingOverlap));
        }
        if covered < output_extent {
            return Err(blocked(BlockedWorkgroupRule::MappingGap));
        }
    }
    let Some(block_product) = shape_product(block) else {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    };
    let Some(workgroup_product) = shape_product(workgroups) else {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    };
    let Some(output_product) = shape_product(output) else {
        return Err(ScheduledRegionDiagnostic::ShapeProductOverflow);
    };
    let Some(launched) = workgroup_product.checked_mul(block_product) else {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    };
    if threads_per_workgroup == 0
        || u64::from(threads_per_workgroup) != block_product
        || work_items != output_product
        || grid_threads != launched
        || launched != output_product
    {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    }
    Ok(())
}

/// Proves a padded blocked launch covers the output and that `m < M ∧ n < N`
/// is a bijection onto it.
///
/// Per-axis `workgroups[d] * block[d] >= output[d]` is the cover, and equality
/// of `workgroups[d]` with `ceil(output[d] / block[d])` is the uniqueness of
/// the restriction: every logical coordinate has exactly one launched
/// preimage, and every other launched coordinate is inactive.
///
/// # Errors
///
/// Returns the named blocked-map rule the statement violated.
pub fn prove_blocked_predicated_cover(
    output: &Shape,
    block: &Shape,
    workgroups: &Shape,
    work_items: u64,
    threads_per_workgroup: u32,
    grid_threads: u64,
) -> Result<(), ScheduledRegionDiagnostic> {
    if output.rank() != block.rank() || output.rank() != workgroups.rank() {
        return Err(blocked(BlockedWorkgroupRule::RankMismatch));
    }
    for (output_extent, block_extent, workgroup_extent) in output
        .extents()
        .iter()
        .zip(block.extents())
        .zip(workgroups.extents())
        .map(|((output, block), workgroups)| (output.get(), block.get(), workgroups.get()))
    {
        if block_extent == 0 {
            return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
        }
        let Some(covered) = workgroup_extent.checked_mul(block_extent) else {
            return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
        };
        if covered < output_extent {
            return Err(blocked(BlockedWorkgroupRule::MappingGap));
        }
        let expected = output_extent.div_ceil(block_extent);
        if workgroup_extent != expected || covered < output_extent {
            return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
        }
        if covered > output_extent
            && workgroup_extent
                .saturating_sub(1)
                .saturating_mul(block_extent)
                >= output_extent
        {
            return Err(blocked(BlockedWorkgroupRule::MappingOverlap));
        }
    }
    let Some(block_product) = shape_product(block) else {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    };
    let Some(workgroup_product) = shape_product(workgroups) else {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    };
    let Some(output_product) = shape_product(output) else {
        return Err(ScheduledRegionDiagnostic::ShapeProductOverflow);
    };
    let Some(launched) = workgroup_product.checked_mul(block_product) else {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    };
    if threads_per_workgroup == 0
        || u64::from(threads_per_workgroup) != block_product
        || work_items != output_product
        || grid_threads != launched
        || launched < output_product
    {
        return Err(blocked(BlockedWorkgroupRule::LaunchGeometry));
    }
    Ok(())
}

/// Returns whether one participant space is exactly the binding's output block.
#[must_use]
pub fn participant_space_matches_block(
    participants: &super::ParticipantSpace,
    block: &Shape,
) -> bool {
    participants.rank() == block.rank()
        && participants
            .extents()
            .iter()
            .zip(block.extents())
            .all(|(participant, extent)| *participant == extent.get())
}

const fn blocked(rule: BlockedWorkgroupRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::BlockedWorkgroup { rule }
}

#[derive(Clone, Copy)]
enum AxisKind {
    Output,
    Contracted,
}

fn exact_quotients(
    whole: &Shape,
    tile: &Shape,
    kind: AxisKind,
) -> Result<Shape, CooperativeContractionAdmission> {
    if whole.rank() != tile.rank() {
        return Err(match kind {
            AxisKind::Output => CooperativeContractionAdmission::OutputBlockRankMismatch {
                output_rank: whole.rank(),
                block_rank: tile.rank(),
            },
            AxisKind::Contracted => CooperativeContractionAdmission::ContractedTileRankMismatch {
                contracted_rank: whole.rank(),
                tile_rank: tile.rank(),
            },
        });
    }
    let mut quotients = Vec::with_capacity(whole.rank());
    for (axis, (extent, tile_extent)) in whole.extents().iter().zip(tile.extents()).enumerate() {
        let extent = extent.get();
        let tile_extent = tile_extent.get();
        if tile_extent == 0 {
            return Err(match kind {
                AxisKind::Output => CooperativeContractionAdmission::EmptyOutputBlock { axis },
                AxisKind::Contracted => {
                    CooperativeContractionAdmission::EmptyContractedTile { axis }
                }
            });
        }
        if !extent.is_multiple_of(tile_extent) {
            return Err(match kind {
                AxisKind::Output => CooperativeContractionAdmission::OutputBlockNotDivisible {
                    axis,
                    output: extent,
                    block: tile_extent,
                },
                AxisKind::Contracted => {
                    CooperativeContractionAdmission::ContractedTileNotDivisible {
                        axis,
                        contracted: extent,
                        tile: tile_extent,
                    }
                }
            });
        }
        quotients.push(crate::shape::Extent::new(extent / tile_extent));
    }
    Shape::try_new(quotients).map_err(|_| CooperativeContractionAdmission::ShapeProductOverflow)
}

fn ceiling_quotients(
    whole: &Shape,
    tile: &Shape,
    kind: AxisKind,
) -> Result<Shape, CooperativeContractionAdmission> {
    if whole.rank() != tile.rank() {
        return Err(match kind {
            AxisKind::Output => CooperativeContractionAdmission::OutputBlockRankMismatch {
                output_rank: whole.rank(),
                block_rank: tile.rank(),
            },
            AxisKind::Contracted => CooperativeContractionAdmission::ContractedTileRankMismatch {
                contracted_rank: whole.rank(),
                tile_rank: tile.rank(),
            },
        });
    }
    let mut quotients = Vec::with_capacity(whole.rank());
    for (axis, (extent, tile_extent)) in whole.extents().iter().zip(tile.extents()).enumerate() {
        let extent = extent.get();
        let tile_extent = tile_extent.get();
        if tile_extent == 0 {
            return Err(match kind {
                AxisKind::Output => CooperativeContractionAdmission::EmptyOutputBlock { axis },
                AxisKind::Contracted => {
                    CooperativeContractionAdmission::EmptyContractedTile { axis }
                }
            });
        }
        quotients.push(crate::shape::Extent::new(extent.div_ceil(tile_extent)));
    }
    Shape::try_new(quotients).map_err(|_| CooperativeContractionAdmission::ShapeProductOverflow)
}

fn shape_product(shape: &Shape) -> Option<u64> {
    shape
        .extents()
        .iter()
        .try_fold(1_u64, |total, extent| total.checked_mul(extent.get()))
}
