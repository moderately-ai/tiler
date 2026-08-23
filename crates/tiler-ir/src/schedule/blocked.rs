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

/// Returns the leading batch axes of a cooperative-contraction output block.
///
/// A block is *batched* when its trailing two axes carry the participants and
/// every leading axis has extent one, so one workgroup covers exactly one
/// coordinate on each leading axis. `Some(0)` is the unbatched block, which is
/// why a rank-two block reaches the same rule rather than a parallel one.
///
/// Returns `None` for a block of rank below two, and for one whose leading
/// extents are not all one. The second is the load-bearing refusal: a leading
/// extent above one would make a workgroup span several batch coordinates with
/// no participant dimension to distinguish them, and the tile's staged operand
/// rows would then hold elements of two different batches.
fn blocked_batch_prefix(block: &Shape) -> Option<usize> {
    let prefix = block.rank().checked_sub(2)?;
    block.extents()[..prefix]
        .iter()
        .all(|extent| extent.get() == 1)
        .then_some(prefix)
}

/// Returns whether one participant space covers the binding's output block.
///
/// The participants occupy the block's **trailing two axes**, and every leading
/// axis of the block is a batch axis of extent one. At rank two the prefix is
/// empty and this is the exact equality it has always been, which is why the
/// batched rule replaces that one rather than sitting beside it: a rank-two
/// block is the batched block with no batch axes, and two predicates would be
/// two places to state one relation.
///
/// # Why this stays a bijection onto the block's positions
///
/// [`MAX_COOPERATIVE_PARTICIPANT_RANK`](super::MAX_COOPERATIVE_PARTICIPANT_RANK)
/// is three, so a rank-four block can never have a participant space of its own
/// rank; carrying the batch axes as participant dimensions is unrepresentable
/// rather than merely unimplemented. What makes the trailing-suffix rule sound
/// instead of a weakening is that a leading extent of one admits exactly one
/// block-local coordinate — zero — so the map from a participant
/// `(l_0, l_1)` to the block-local position `(0, .., 0, l_0, l_1)` is still
/// onto every position the block contains and still injective. The participant
/// count therefore still equals the block's element count, which is the
/// equality [`prove_blocked_bijection`] and [`prove_blocked_predicated_cover`]
/// compose against the launch geometry.
#[must_use]
pub fn participant_space_matches_block(
    participants: &super::ParticipantSpace,
    block: &Shape,
) -> bool {
    let Some(prefix) = blocked_batch_prefix(block) else {
        return false;
    };
    participants.rank() == 2
        && participants
            .extents()
            .iter()
            .zip(&block.extents()[prefix..])
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

#[cfg(test)]
mod tests {
    use super::{blocked_batch_prefix, participant_space_matches_block};
    use crate::schedule::ParticipantSpace;
    use crate::shape::Shape;

    fn square(block: u64) -> ParticipantSpace {
        ParticipantSpace::new(&[block, block]).expect("rank two is representable")
    }

    /// The unbatched block is the batched block with no batch axes.
    ///
    /// The control for every refusal below: without it, "the batched rule
    /// refuses a malformed block" would be consistent with a rule that refuses
    /// everything, and the rank-two regression would not be visible here.
    #[test]
    fn a_rank_two_block_is_the_empty_batch_prefix() {
        let block = Shape::from_dims([16, 16]);
        assert_eq!(blocked_batch_prefix(&block), Some(0));
        assert!(participant_space_matches_block(&square(16), &block));
    }

    /// A rank-four block whose batch extents are one carries two batch axes.
    #[test]
    fn a_batched_block_names_its_leading_axes_as_batch_axes() {
        let block = Shape::from_dims([1, 1, 16, 16]);
        assert_eq!(blocked_batch_prefix(&block), Some(2));
        assert!(participant_space_matches_block(&square(16), &block));
    }

    /// A batch axis wider than one workgroup is refused.
    ///
    /// **The load-bearing clause.** A leading extent above one would have one
    /// workgroup span two batch coordinates with no participant dimension
    /// distinguishing them, so the tile's staged operand rows would hold
    /// elements of two different batches and every participant would fold the
    /// wrong contributor set. The rank and the trailing extents are both
    /// well-formed here, so this refusal is the batch extent's alone.
    #[test]
    fn a_batch_axis_wider_than_one_workgroup_is_refused() {
        let block = Shape::from_dims([2, 1, 16, 16]);
        assert_eq!(blocked_batch_prefix(&block), None);
        assert!(!participant_space_matches_block(&square(16), &block));
    }

    /// The participants must occupy exactly the block's trailing two axes.
    ///
    /// **The extents deliberately agree on the compared prefix.** A space whose
    /// leading extents disagree is refused by the extent comparison whatever the
    /// rank clause says, so a case like `[1, 16, 16]` against this block cannot
    /// tell the two refusals apart — it was written that way first, and dropping
    /// the rank clause left it passing. `[16, 16, 4]` zips equal against the
    /// block's trailing `[16, 16]` and differs only in carrying a third
    /// dimension, so it isolates the rank clause: without it the space is
    /// accepted while contributing `16 * 16 * 4` participants to a block holding
    /// `256` positions.
    #[test]
    fn a_participant_space_of_another_rank_is_refused() {
        let block = Shape::from_dims([1, 16, 16]);
        assert_eq!(blocked_batch_prefix(&block), Some(1));
        let rank_three = ParticipantSpace::new(&[16, 16, 4]).expect("rank three is representable");
        assert_eq!(
            rank_three.participants(),
            Some(1_024),
            "the perturbation case must differ from the block population, or the rank clause \
             would be provable from the extents alone",
        );
        assert!(
            !participant_space_matches_block(&rank_three, &block),
            "the batch axis carries no participant dimension, so a space of the block's own \
             rank is not the space this tile states",
        );
        assert!(participant_space_matches_block(&square(16), &block));
    }

    /// Trailing extents that disagree with the participants are refused.
    #[test]
    fn a_trailing_extent_that_disagrees_is_refused() {
        let block = Shape::from_dims([1, 1, 16, 8]);
        assert!(!participant_space_matches_block(&square(16), &block));
    }

    /// A block of rank below two has no participant axes to carry the tile.
    #[test]
    fn a_block_below_rank_two_is_refused() {
        assert_eq!(blocked_batch_prefix(&Shape::from_dims([16])), None);
        assert!(!participant_space_matches_block(
            &square(16),
            &Shape::from_dims([16])
        ));
    }
}
