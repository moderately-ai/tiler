//! Exact-divisible blocked-workgroup admission and bijection proof.
//!
//! The 2026-08-11 first pass admits only exact output blocks and exact
//! contracted tiles. A caller selecting this tiled approach receives a typed
//! [`CooperativeContractionAdmission`] when any equality is absent or false. The
//! function never substitutes the direct contraction.

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

fn shape_product(shape: &Shape) -> Option<u64> {
    shape
        .extents()
        .iter()
        .try_fold(1_u64, |total, extent| total.checked_mul(extent.get()))
}
