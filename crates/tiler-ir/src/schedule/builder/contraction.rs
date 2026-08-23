//! The two-operand contraction families and their scheduled realizations.
//!
//! Three gates over one scalar program: the static contraction, the one whose
//! contracted extent is a live input axis, and the cooperative form whose
//! invocations share staged operand tiles. They sit together because each
//! states its obligations against the same three declarations of the
//! contracted space — the scalar program's, the schedule topology's, and each
//! operand access's — and a producer that spelled one of them differently
//! would fold a different number of contributors than it addressed.

use crate::schedule::MAX_COOPERATIVE_ROUNDS;
use crate::schedule::blocked::participant_space_matches_block;
use crate::schedule::error::{
    BlockedWorkgroupRule, CooperativeTileRule, ScheduledRegionDiagnostic,
};
use crate::schedule::model::{
    Access, AccessMode, ContractionAxisSource, ExecutionBinding, LogicalAccess, ReductionTopology,
    ScalarProgram, ScheduledRegion, TensorRole, element_count,
};

use super::diagnostics::{blocked, cooperative, numerical_program};
use super::proof::verify_proof_records;
use super::reduction::verify_accumulation_width;
use super::tile::verify_operand_tile;

/// Verifies a two-operand strict tensor contraction region.
///
/// Every obligation is stated against the region's *own* three declarations of
/// the contracted space — the scalar program's, the schedule topology's, and
/// each operand access's — and they are required to agree. A producer that
/// stated one of them differently would otherwise fold a different number of
/// contributors than it addressed.
pub(super) fn verify_contraction(
    region: &ScheduledRegion,
    left: &Access,
    right: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ScalarProgram::StrictTensorContraction {
        contracted_shape,
        order,
        ..
    } = numerical_program(region)?.0
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    if matches!(
        region.schedule.reduction,
        ReductionTopology::CooperativeContraction { .. }
    ) {
        return verify_cooperative_contraction(region, left, right, write);
    }
    if matches!(
        region.schedule.reduction,
        ReductionTopology::LiveContraction { .. }
    ) {
        return verify_live_contraction(region, left, right, write);
    }
    let ReductionTopology::Contraction {
        contracted_shape: scheduled_contracted,
        order: scheduled_order,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = numerical_program(region)?.1;
    if contracted_shape != scheduled_contracted
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The one precondition this realization has. The registered family declares
    // an empty contracted domain refused rather than identity-valued, so a
    // contracted space with no points has no result to commit — and a rank-zero
    // contracted shape has one point, not none, so the check is on the element
    // count rather than on the rank.
    let contracted_points = element_count(contracted_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    if contracted_points == 0 {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.mode != AccessMode::Read
        || right.mode != AccessMode::Read
        || left.ownership.is_some()
        || right.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || write.component_role.is_some()
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    if !matches!(left.tensor, TensorRole::Input) || !matches!(right.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.component_role.is_some() || right.component_role.is_some() {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[left, right], write)?;

    let mut contracted_covered = vec![false; contracted_shape.rank()];
    let mut output_covered = vec![false; region.index.iteration_shape.rank()];
    for access in [left, right] {
        let LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape: access_contracted,
            sources,
            order: access_order,
        } = &access.map
        else {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        };
        if output_shape != &region.index.iteration_shape
            || access_contracted != contracted_shape
            || access_order != order
            || sources.len() != operand_shape.rank()
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        // Every operand axis names one in-range coordinate whose extent it
        // agrees with, and no two axes of one operand name the same coordinate.
        // Extent agreement is what makes the row-major linearization stay inside
        // the operand, which is the whole content of its bounds proof.
        let mut seen_output = vec![false; output_shape.rank()];
        let mut seen_contracted = vec![false; contracted_shape.rank()];
        for (axis, source) in sources.iter().enumerate() {
            let (shape, seen, covered) = match source {
                ContractionAxisSource::Output { .. } => {
                    (output_shape, &mut seen_output, &mut output_covered)
                }
                ContractionAxisSource::Contracted { .. } => (
                    contracted_shape,
                    &mut seen_contracted,
                    &mut contracted_covered,
                ),
            };
            let position = match source {
                ContractionAxisSource::Output { position }
                | ContractionAxisSource::Contracted { position } => usize::try_from(*position)
                    .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?,
            };
            let (Some(extent), Some(slot)) =
                (shape.extents().get(position), seen.get_mut(position))
            else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            if std::mem::replace(slot, true) || operand_shape.extents()[axis] != *extent {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            }
            covered[position] = true;
        }
        // A contracted coordinate this operand does not read would make the
        // operand invariant in it — an outer product summed over a free index,
        // not a contraction. ADR 0087's second rule refuses exactly that
        // structure, and this is where a region claiming one is caught.
        if seen_contracted.iter().any(|read| !read) {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
    }
    // Every output coordinate must be read by at least one operand: one that no
    // operand reads would make every output position along it hold the same
    // value, which is a broadcast the structure never declared.
    if output_covered.iter().any(|read| !read) || contracted_covered.iter().any(|read| !read) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Verifies a contraction whose contracted extent is a live input-axis operand.
///
/// The accepted `LiveContraction` / `ContractionOperand` spelling: free indices
/// and the output stay static, the scalar program's contracted shape is empty
/// rather than a specialized `S`, and the named input axis is the inner trip
/// count. Baking `S` into the operand shapes, the scalar program, or the
/// topology is a different region — `ReductionTopology::Contraction` — and a
/// different identity.
fn verify_live_contraction(
    region: &ScheduledRegion,
    left: &Access,
    right: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::LiveContraction {
        live_access,
        live_axis,
        order: scheduled_order,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let ScalarProgram::StrictTensorContraction {
        contracted_shape,
        order,
        ..
    } = numerical_program(region)?.0
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = numerical_program(region)?.1;
    if contracted_shape.rank() != 0
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.mode != AccessMode::Read
        || right.mode != AccessMode::Read
        || left.ownership.is_some()
        || right.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || write.component_role.is_some()
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    if !matches!(left.tensor, TensorRole::Input) || !matches!(right.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.component_role.is_some() || right.component_role.is_some() {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[left, right], write)?;
    element_count(&region.index.iteration_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;

    let mut output_covered = vec![false; region.index.iteration_shape.rank()];
    for access in [left, right] {
        let LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape: access_contracted,
            sources,
            order: access_order,
        } = &access.map
        else {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        };
        if output_shape != &region.index.iteration_shape
            || access_contracted.rank() != 0
            || access_order != order
            || sources.len() != operand_shape.rank()
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        element_count(operand_shape)
            .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
        if operand_shape
            .extents()
            .iter()
            .any(|extent| extent.get() == 0)
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        let mut seen_output = vec![false; output_shape.rank()];
        for (axis, source) in sources.iter().enumerate() {
            let ContractionAxisSource::Output { position } = source else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            let position = usize::try_from(*position)
                .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
            let (Some(extent), Some(slot)) = (
                output_shape.extents().get(position),
                seen_output.get_mut(position),
            ) else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            if std::mem::replace(slot, true) || operand_shape.extents()[axis] != *extent {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            }
            output_covered[position] = true;
        }
    }
    if output_covered.iter().any(|read| !read) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let Some(named) = usize::try_from(live_access.get())
        .ok()
        .and_then(|position| region.index.accesses.get(position))
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    if named.mode != AccessMode::Read || !matches!(named.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let LogicalAccess::ContractionOperand { operand_shape, .. } = &named.map else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let expected_axis = u32::try_from(operand_shape.rank())
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    if live_axis.get() != expected_axis {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Verifies the operand-sharing cooperative contraction and its blocked map.
///
/// The sibling of [`verify_cooperative_semantics`]. That gate proves a
/// one-committer reduction tile; this one proves a contraction whose
/// invocations each own an output position and cooperate only by staging
/// operand tiles. The two share the dataflow half of
/// [`verify_cooperative_tile`] and nothing of the ownership theorem.
///
/// [`verify_cooperative_semantics`]: super::reduction::verify_cooperative_semantics
/// [`verify_cooperative_tile`]: super::tile::verify_cooperative_tile
fn verify_cooperative_contraction(
    region: &ScheduledRegion,
    left: &Access,
    right: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::CooperativeContraction {
        tile,
        contracted_shape: scheduled_contracted,
        contracted_tile,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let ScalarProgram::StrictTensorContraction {
        contracted_shape,
        order,
        ..
    } = numerical_program(region)?.0
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = numerical_program(region)?.1;
    // Both permissions are recorded and cross-checked against the region's
    // declared realization, and neither is *consulted* to admit the topology —
    // the relation [`ReductionTopology::Contraction`] already states, for the
    // same reason. Tiling the contracted space changes which memory a
    // contributor is read from and nothing about the order contributors are
    // combined in: one invocation owns one output position and folds that
    // output's contributors in ascending contracted order across the whole round
    // loop, so the fold *is* the declared contributor sequence. Requiring
    // reassociation would refuse the strict realization the first-contraction
    // record attributes uniquely to `strict_fold+ftz`. A topology that genuinely
    // regroups the sequence into per-round subtotals is a different realization
    // with its own reserved vocabulary — `CooperativeContractionSplit`, which
    // holds reduction-topology tag `0x36` — and the carried accumulator in
    // `emit_cooperative_contraction` is what keeps this one out of that class.
    if contracted_shape != scheduled_contracted
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_accumulation_width(*accumulation, numerical_program(region)?.0)?;
    let contracted_points = element_count(contracted_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    if contracted_points == 0 {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.mode != AccessMode::Read
        || right.mode != AccessMode::Read
        || left.ownership.is_some()
        || right.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || write.component_role.is_some()
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    if !matches!(left.tensor, TensorRole::Input) || !matches!(right.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.component_role.is_some() || right.component_role.is_some() {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[left, right], write)?;

    let mut contracted_covered = vec![false; contracted_shape.rank()];
    let mut output_covered = vec![false; region.index.iteration_shape.rank()];
    let mut operand_reads_output: Vec<Vec<bool>> = Vec::with_capacity(2);
    for access in [left, right] {
        let LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape: access_contracted,
            sources,
            order: access_order,
        } = &access.map
        else {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        };
        if output_shape != &region.index.iteration_shape
            || access_contracted != contracted_shape
            || access_order != order
            || sources.len() != operand_shape.rank()
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        let mut seen_output = vec![false; output_shape.rank()];
        let mut seen_contracted = vec![false; contracted_shape.rank()];
        for (axis, source) in sources.iter().enumerate() {
            let (shape, seen, covered) = match source {
                ContractionAxisSource::Output { .. } => {
                    (output_shape, &mut seen_output, &mut output_covered)
                }
                ContractionAxisSource::Contracted { .. } => (
                    contracted_shape,
                    &mut seen_contracted,
                    &mut contracted_covered,
                ),
            };
            let position = match source {
                ContractionAxisSource::Output { position }
                | ContractionAxisSource::Contracted { position } => usize::try_from(*position)
                    .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?,
            };
            let (Some(extent), Some(slot)) =
                (shape.extents().get(position), seen.get_mut(position))
            else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            if std::mem::replace(slot, true) || operand_shape.extents()[axis] != *extent {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            }
            covered[position] = true;
        }
        if seen_contracted.iter().any(|read| !read) {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        operand_reads_output.push(seen_output);
    }
    if output_covered.iter().any(|read| !read) || contracted_covered.iter().any(|read| !read) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let ExecutionBinding::BlockedWorkgroup { block, .. } = &region.schedule.binding else {
        return Err(blocked(BlockedWorkgroupRule::BindingRequired));
    };
    if !participant_space_matches_block(&tile.coordinates.participants, block) {
        return Err(blocked(BlockedWorkgroupRule::ParticipantBlockMismatch));
    }
    verify_blocked_operand_roles(&operand_reads_output, &region.index.iteration_shape)?;
    if contracted_tile.rank() != contracted_shape.rank() {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let mut tile_count = 1_u64;
    for (extent, tile_extent) in contracted_shape
        .extents()
        .iter()
        .zip(contracted_tile.extents())
    {
        let extent = extent.get();
        let tile_extent = tile_extent.get();
        if tile_extent == 0 || !extent.is_multiple_of(tile_extent) {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        tile_count = tile_count
            .checked_mul(extent / tile_extent)
            .ok_or(ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    }
    if tile.rounds == 0 || tile.rounds > MAX_COOPERATIVE_ROUNDS {
        return Err(cooperative(CooperativeTileRule::RoundStructure));
    }
    if tile.rounds != tile_count {
        return Err(cooperative(CooperativeTileRule::ContributorSplit));
    }
    verify_operand_tile(tile)
}

/// Requires each operand to read exactly one of the block's two participant axes.
///
/// The participants occupy the output's **trailing two axes**: participant
/// `(m, n)` owns the block-local position `(m, n)` on axes `rank - 2` and
/// `rank - 1`. The staged tile is what forces this rule. Participant `(m, n)`
/// writes one element of each operand and then every participant of row `m`
/// reads the same staged left run, so the left operand's address must not vary
/// with `n` — and symmetrically the right's must not vary with `m`.
///
/// **This is a statement about the operand's declared axis sources, and it was
/// unstated before batching.** The rank-two emission hardcoded `[M, K]` and
/// `[N, K]` addressing and discarded the declared sources entirely, so a region
/// whose left operand read output axis `1` verified and lowered to a kernel
/// addressing it by axis `0`. Deriving the address from the sources is what the
/// batched form requires — the value structure `grts,gsd->grtd` has a right
/// operand `[g, s, d]` whose contracted axis sits in the *middle* — and this
/// rule is the precondition that derivation needs.
///
/// Batch axes are deliberately unconstrained: an axis outside the trailing pair
/// may be read by either operand or by both. That is what makes the grouped
/// query structure expressible — the score structure's key operand `[g, s, d]`
/// reads the group and never the repetition, so the key is shared across `r`
/// rather than broadcast into it — and both operands reading one batch axis is
/// the ordinary batched matmul rather than a defect.
fn verify_blocked_operand_roles(
    operand_reads_output: &[Vec<bool>],
    iteration_shape: &crate::shape::Shape,
) -> Result<(), ScheduledRegionDiagnostic> {
    let Some(row) = iteration_shape.rank().checked_sub(2) else {
        return Err(blocked(BlockedWorkgroupRule::ParticipantBlockMismatch));
    };
    let column = row + 1;
    let [left_reads, right_reads] = operand_reads_output else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    // Read positionally because the roles are positional: the left operand
    // carries the block's row axis and the right its column axis, which is the
    // orientation `blocked_operand_tile`'s transposed staged write states.
    if !left_reads[row] || left_reads[column] || right_reads[row] || !right_reads[column] {
        return Err(blocked(BlockedWorkgroupRule::ParticipantBlockMismatch));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify_blocked_operand_roles;
    use crate::schedule::error::{BlockedWorkgroupRule, ScheduledRegionDiagnostic};
    use crate::shape::Shape;

    /// Builds the per-operand output-axis read sets from the axes each reads.
    fn reads(rank: usize, left: &[usize], right: &[usize]) -> Vec<Vec<bool>> {
        [left, right]
            .iter()
            .map(|axes| {
                let mut seen = vec![false; rank];
                for axis in *axes {
                    seen[*axis] = true;
                }
                seen
            })
            .collect()
    }

    fn verdict(
        rank: usize,
        left: &[usize],
        right: &[usize],
    ) -> Result<(), ScheduledRegionDiagnostic> {
        let shape = Shape::try_new(vec![crate::shape::Extent::new(4); rank])
            .expect("a uniform shape of this rank is representable");
        verify_blocked_operand_roles(&reads(rank, left, right), &shape)
    }

    const MISMATCH: ScheduledRegionDiagnostic = ScheduledRegionDiagnostic::BlockedWorkgroup {
        rule: BlockedWorkgroupRule::ParticipantBlockMismatch,
    };

    /// The rank-two matmul: left carries the row axis, right the column axis.
    #[test]
    fn the_unbatched_matmul_orientation_is_admitted() {
        assert_eq!(verdict(2, &[0], &[1]), Ok(()));
    }

    /// Both attention structures are admitted, and they differ where they should.
    ///
    /// Score `grtd,gsd->grts` has output `[g, r, t, s]`: the query reads
    /// `{g, r, t}` and the key `{g, s}`. Value `grts,gsd->grtd` has output
    /// `[g, r, t, d]`: the score reads `{g, r, t}` and the value `{g, s}` — the
    /// same *sets*, because in both cases the right operand shares the group and
    /// owns the column axis while never reading the repetition.
    #[test]
    fn both_attention_structures_are_admitted() {
        assert_eq!(verdict(4, &[0, 1, 2], &[0, 3]), Ok(()), "score and value");
    }

    /// A batch axis read by both operands is the ordinary batched matmul.
    ///
    /// The control against over-constraining: the rule governs the trailing pair
    /// alone, so a shared group axis must stay admitted. Without this, "the roles
    /// are checked" would be consistent with a rule that also forbade batching.
    #[test]
    fn a_batch_axis_read_by_both_operands_is_admitted() {
        assert_eq!(verdict(3, &[0, 1], &[0, 2]), Ok(()));
    }

    /// A left operand that also reads the column axis is refused.
    ///
    /// The staged left run is shared by every participant of a row, so a left
    /// address varying with `n` would have each of them fold a different
    /// operand element than the one staged for it.
    #[test]
    fn a_left_operand_reading_the_column_axis_is_refused() {
        assert_eq!(verdict(4, &[0, 1, 2, 3], &[0, 3]), Err(MISMATCH));
    }

    /// A right operand that reads the row axis is refused.
    ///
    /// This is the *correct-but-materialized* twin at the schedule layer: a key
    /// operand carrying the repetition computes the same numbers while reading
    /// per repetition rather than sharing across it, which is precisely what the
    /// grouped-query structure exists to avoid.
    #[test]
    fn a_right_operand_reading_the_row_axis_is_refused() {
        assert_eq!(verdict(4, &[0, 1, 2], &[0, 2, 3]), Err(MISMATCH));
    }

    /// An operand that reads neither of its own block axis is refused.
    #[test]
    fn an_operand_missing_its_own_block_axis_is_refused() {
        assert_eq!(verdict(4, &[0, 1], &[0, 3]), Err(MISMATCH), "left lost row");
        assert_eq!(
            verdict(4, &[0, 1, 2], &[0]),
            Err(MISMATCH),
            "right lost col"
        );
    }

    /// A rank-below-two output has no trailing pair to carry the participants.
    #[test]
    fn an_output_below_rank_two_is_refused() {
        assert_eq!(verdict(1, &[0], &[0]), Err(MISMATCH));
    }
}
