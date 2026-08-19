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
    if contracted_shape != scheduled_contracted
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || !*permits_reassociation
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
