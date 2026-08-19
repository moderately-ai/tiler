//! The fold admissions: serial, split across dispatches, and cooperative.
//!
//! One dispatch and three gates, each proving that a topology agrees with the
//! algebra [`super::family`] derived from the scalar program — the reduced
//! axes, the contributor order, the boundary tensors, and the empty-domain
//! obligation. They share a file because they share that premise and the one
//! accumulation-width authority, so a change admitting a narrower accumulator
//! for one strategy cannot leave another refusing it.

use crate::schedule::MAX_COOPERATIVE_ROUNDS;
use crate::schedule::cooperative::ContributorArrival;
use crate::schedule::error::{
    ContributorCoverageRule, CooperativeTileRule, ScheduledRegionDiagnostic,
};
use crate::schedule::model::{
    Access, AccessMode, ContributorCoverage, LogicalAccess, ReductionPass, ReductionTopology,
    ScalarProgram, ScheduledRegion, TensorRole, contributor_count, partial_reduction_axis,
    partial_reduction_shape, scalar_arithmetic_type,
};
use crate::schedule::numerics::ArithmeticType;
use crate::schedule::pointwise::PointwiseF32Node;

use super::coverage::verify_contributor_coverage;
use super::diagnostics::{cooperative, coverage_rule, numerical_program};
use super::family::{
    CommittedTensor, EmptyDomainContract, FamilyTopology, empty_domain_is_satisfied, split_family,
};
use super::proof::verify_proof_records;
use super::tile::verify_cooperative_tile;

pub(super) fn verify_access_and_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    if read.mode != AccessMode::Read
        || read.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    verify_proof_records(region, &[read], write)?;
    if matches!(
        region.schedule.reduction,
        ReductionTopology::MultiPass { .. }
    ) {
        return verify_multi_pass_semantics(region, read, write);
    }
    if matches!(
        region.schedule.reduction,
        ReductionTopology::CooperativeWorkgroup { .. }
    ) {
        return verify_cooperative_semantics(region, read, write);
    }
    verify_serial_semantics(region, read, write)
}

/// Verifies the conjunction every serial fold shares.
///
/// The family derivation owns the axes, contributor order, contributor tensor,
/// and empty-domain contract. This gate owns the serial topology's agreement
/// with those facts and deliberately does not read
/// [`SplitFamily::consumes_reassociation`]: a serial fold preserves the
/// declared contributor grouping and spends no reassociation permission.
///
/// [`SplitFamily::consumes_reassociation`]: super::family::SplitFamily::consumes_reassociation
fn verify_serial_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let numerical = numerical_program(region)?.1;
    let ReductionTopology::Serial {
        axes: scheduled_axes,
        order: scheduled_order,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let family = split_family(numerical_program(region)?.0)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let read_tensor = family
        .read_tensor(FamilyTopology::Serial)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let axes = family.axes;
    if axes != scheduled_axes.as_slice()
        || axes != access_axes.as_slice()
        || family.order != scheduled_order
        || family.order != access_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || output_shape != &region.index.iteration_shape
        || input_shape.without_axes(axes) != *output_shape
        || !read_tensor.admits(read.tensor)
        || !CommittedTensor::CoverAssigned.admits(write.tensor)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // Preserve the serial admission boundary this refactor inherited. An
    // identity-seeded fold validates the identity it carries and does not need
    // to count contributors; only an identity-less fold owes a proven non-empty
    // domain. Counting every family here would turn contributor-count
    // canonicality and overflow into new serial-sum refusals unrelated to the
    // empty-domain value this check owns.
    let contributors = match family.empty_domain {
        EmptyDomainContract::Identity { .. } => None,
        EmptyDomainContract::NoIdentity => Some(
            contributor_count(axes, &read.map)
                .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?,
        ),
    };
    if !empty_domain_is_satisfied(family.empty_domain, contributors) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    // The sole serial-only family residual. A fold carrying an epilogue applies
    // it to the complete folded value, which is why `split_family` refuses every
    // parallel topology for it. Serial admission still owes three facts the fold
    // family does not: the expression is valid, its one leaf is the accumulator,
    // and it transforms that leaf instead of respelling `SquaredSerialSum`.
    if let ScalarProgram::SquaredSerialSumThenEpilogue { epilogue, .. } =
        numerical_program(region)?.0
        && (!epilogue.is_valid()
            || epilogue.input_count() != 1
            || matches!(
                epilogue
                    .nodes()
                    .get(usize::try_from(epilogue.root().index()).unwrap_or(usize::MAX)),
                Some(PointwiseF32Node::Input { .. })
            ))
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Verifies that a parallel topology combines at the width its region computes
/// in.
///
/// **The single accumulation authority for every topology that declares one.**
/// [`ReductionTopology::MultiPass`] and
/// [`ReductionTopology::CooperativeWorkgroup`] are the two variants carrying an
/// `accumulation` field, and both reach this function rather than repeating the
/// comparison, so a change admitting a narrower accumulator for one strategy
/// cannot leave the other refusing it.
///
/// The required width is *derived* from the region's own scalar program rather
/// than compared against a literal `F32`. Every family that reaches either gate
/// is `f32` today — `split_family` refuses the pointwise programs for every
/// topology — so the derivation changes no outcome now; what it
/// changes is that a `bf16` reduction admitted later must state its accumulator
/// instead of inheriting an `f32` one nobody re-checked.
///
/// # Errors
///
/// Returns [`ScheduledRegionDiagnostic::AccumulationWidth`] carrying both
/// widths. Refusing by its own name rather than as a shared refinement failure
/// is criterion 3 of `implement-parallel-reduction-strategies`: the accumulation
/// dtype is an explicit part of the strategy, and a strategy accumulating at a
/// width the contract does not admit is rejected with a typed reason.
pub(super) fn verify_accumulation_width(
    declared: ArithmeticType,
    program: &ScalarProgram,
) -> Result<(), ScheduledRegionDiagnostic> {
    let required = scalar_arithmetic_type(program);
    if declared != required {
        return Err(ScheduledRegionDiagnostic::AccumulationWidth { declared, required });
    }
    Ok(())
}

/// Verifies one pass of a split, multi-dispatch reduction.
///
/// The two passes are checked together here rather than as two more arms of the
/// serial match because every obligation they carry is stated relative to the
/// same [`crate::schedule::model::ContributorPartition`]: the partial pass proves the split
/// covers its contributor sequence exactly, and the final pass proves it combines
/// exactly one contributor per partition of that same split. Splitting them across
/// unrelated arms would let one pass be verified against a partition the other
/// never agreed to.
fn verify_multi_pass_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::MultiPass {
        pass,
        coverage,
        axes: scheduled_axes,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let family = split_family(numerical_program(region)?.0)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let read_tensor = family
        .read_tensor(FamilyTopology::MultiPass(*pass))
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let numerical = numerical_program(region)?.1;
    // Reassociation is what a split of an order-*sensitive* fold consumes, and it
    // is checked on its own: contributor order is preserved by construction, so a
    // permitted permutation neither grants nor substitutes for this permission.
    // The extrema family consumes nothing, for the reason
    // [`SplitFamily::consumes_reassociation`] records — but it still declares the
    // permissions, and a declaration disagreeing with its own realization would be
    // incoherent whatever the fold's legality.
    if *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || (family.consumes_reassociation && !*permits_reassociation)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The accumulator, refused under its own name. Checked after the permissions
    // so a region wrong on both reports the permission, which is the ordering
    // the cooperative gate already follows for `ArrivalPermission`.
    verify_accumulation_width(*accumulation, numerical_program(region)?.0)?;

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let axes = family.axes;
    if axes != scheduled_axes.as_slice()
        || axes != access_axes.as_slice()
        || family.order != scheduled_order
        || family.order != access_order
        || input_shape.without_axes(axes) != *output_shape
        || !read_tensor.admits(read.tensor)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let contributors = contributor_count(axes, &read.map)
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    // The family's empty-domain obligation, decided against the sequence this
    // pass's split covers. It sits after the contributor count rather than in the
    // agreement block above because the identity-less arm is a statement *about*
    // that count, and the identity-carrying arm's constant is checked here for the
    // same reason it is checked in the serial arms: one empty-domain answer.
    if !empty_domain_is_satisfied(family.empty_domain, Some(contributors)) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let partition = coverage.partition();
    let partial_shape = partial_reduction_shape(output_shape, partition)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;

    let admitted = match pass {
        // The partial pass proves the split covers its own contributor
        // sequence — exactly, or as a suffix-padded extension whose identity
        // the verifier derives — and stages one partial per partition.
        //
        // Its write is the one fold write in this module the cover does not
        // choose, for the reason [`CommittedTensor::Exactly`] states: a partial
        // is an unfolded fragment of the reduction, so committing one to a
        // declared program output would publish a value that is not the fold's
        // result under any cover.
        ReductionPass::Partial => {
            verify_contributor_coverage(
                *coverage,
                contributors,
                1,
                numerical_program(region)?.0,
                numerical_program(region)?.1,
            )?;
            region.index.iteration_shape == partial_shape
                && CommittedTensor::Exactly(TensorRole::Intermediate).admits(write.tensor)
        }
        // The final pass proves it combines exactly one contributor per
        // partition of that same split, reading the staged partial tensor.
        //
        // Padding is a fact of the first-level split. The final pass's
        // contributors *are* the staged partials, including any identity-valued
        // ones the partial pass already wrote, so a padded final pass would
        // invent extra partials the tensor does not hold.
        ReductionPass::Final => {
            if matches!(coverage, ContributorCoverage::IdentityPadded { .. }) {
                return Err(coverage_rule(ContributorCoverageRule::PaddedCoverage));
            }
            partial_reduction_axis(output_shape).is_some_and(|axis| axes == [axis].as_slice())
                && *input_shape == partial_shape
                && contributors == partition.partitions
                && region.index.iteration_shape == *output_shape
                && CommittedTensor::CoverAssigned.admits(write.tensor)
        }
    };
    if admitted {
        Ok(())
    } else {
        Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)
    }
}

/// Verifies one cooperative workgroup reduction tile and its split.
///
/// Two obligations that stay apart. The *semantic* half proves the tile realizes
/// the region's declared reduction: the split covers the contributor sequence
/// exactly once each, the participants are the partitions, the iteration domain
/// runs one invocation per (output, participant) pair, and the reassociation the
/// split performs is one the contract permits — or one the family's own algebra
/// makes free, which is the extrema fold alone. The *dataflow* half, in
/// [`verify_cooperative_tile`], proves the staging itself is well formed
/// independently of what is being reduced.
///
/// Neither half authorizes the handoff against a *machine*. The tile's derived
/// edges are discharged here by the points the tile declares, and the
/// structured-kernel verifier separately proves the emitted body puts the
/// realizing barrier between the two effects — but whether any target can
/// perform the realization those points require is a feasibility question this
/// module never answers.
///
/// [`verify_cooperative_tile`]: super::tile::verify_cooperative_tile
pub(super) fn verify_cooperative_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::CooperativeWorkgroup {
        coverage,
        tile,
        axes: scheduled_axes,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
        arrival,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let family = split_family(numerical_program(region)?.0)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let read_tensor = family
        .read_tensor(FamilyTopology::Cooperative)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let numerical = numerical_program(region)?.1;
    // Reassociation is what a split of an order-sensitive fold consumes, exactly
    // as it is for a multi-pass one, and for those families it is required rather
    // than merely recorded: a contract that forbids reassociation forbids the
    // strategy outright. The extrema family spends nothing, so a strict contract
    // admits a tile over it — the whole asymmetry this vocabulary owes the
    // softmax's two passes.
    if *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || (family.consumes_reassociation && !*permits_reassociation)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The region's own element width, refused under its own name, for the reason
    // the multi-pass gate above states.
    verify_accumulation_width(*accumulation, numerical_program(region)?.0)?;
    // The second permission, checked on its own and only where the arrival
    // actually consumes it. The order matters: a permitted-but-unrealizable
    // arrival must reach the admission rule below rather than be reported as a
    // numerical refusal, and an unpermitted one must name the permission rather
    // than the construct.
    if arrival.requires_permutation() && !numerical.permits_permutation() {
        return Err(cooperative(CooperativeTileRule::ArrivalPermission));
    }
    // The combining level folds the staged slots in ascending order through one
    // serial loop, which is the only arrival this vocabulary can order: the
    // constructs the others need are unadmitted synchronization kinds.
    if *arrival != ContributorArrival::AscendingParticipant {
        return Err(cooperative(CooperativeTileRule::UnadmittedArrival));
    }

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let axes = family.axes;
    if axes != scheduled_axes.as_slice()
        || axes != access_axes.as_slice()
        || family.order != scheduled_order
        || family.order != access_order
        || input_shape.without_axes(axes) != *output_shape
        || !read_tensor.admits(read.tensor)
        // A tile is both halves of a split in one dispatch, so its single write
        // is the fold's committing write and the cover decides where it lands —
        // there is no staging pass here whose target the split structure fixes,
        // because a tile stages in workgroup memory rather than a boundary
        // tensor.
        || !CommittedTensor::CoverAssigned.admits(write.tensor)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let contributors = contributor_count(axes, &read.map)
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    // An empty contributor domain commits the declared identity from one
    // invocation with no fold, which is what the serial topology already does
    // and what needs no staging at all. A tile over it would declare a
    // visibility edge for values no participant produces. This refusal is also
    // where the identity-less family's own precondition is discharged, which is
    // why `empty_domain_is_satisfied` below can only decide the other arm here.
    if contributors == 0 {
        return Err(cooperative(CooperativeTileRule::EmptyContributorDomain));
    }
    // The strict sum's empty result is `+0.0`, and every identity-carrying family
    // here shares it. Required at the same place the serial and multi-pass
    // admissions require it, so a tile cannot introduce a second empty-domain
    // answer.
    if !empty_domain_is_satisfied(family.empty_domain, Some(contributors)) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The iteration domain is the output shape with one trailing participant
    // axis — the same layout a partial pass uses, and for the same reason: it
    // makes the participant ordinal the innermost coordinate of the invocation
    // index, so a participant's local coordinate is derivable without a second
    // layout rule.
    //
    // The trailing axis is one coordinate per *participant*, so it takes the
    // extent product rather than the space's shape: the invocation index is
    // linear whatever shape the tile arranges its participants in.
    //
    // `verify_participant_space` already refused a space whose product does not
    // exist, so this propagates rather than decides — written as a refusal and
    // not an `expect` because the two authorities are ordered by `verify_intrinsic`
    // rather than by a type, and a reordering should cost a diagnostic instead
    // of a panic.
    let Some(participants) = tile.coordinates.participants.participants() else {
        return Err(cooperative(CooperativeTileRule::LocalCoordinates));
    };
    // The round count is bounded before anything multiplies by it. A tile whose
    // phases never run stages nothing yet would still derive staged accesses,
    // visibility edges, and a synchronization requirement — a complete
    // obligation over a program that executes nothing — and one beyond the bound
    // would overflow the product below. Both would otherwise surface as a split
    // mismatch, which names the wrong field.
    if tile.rounds == 0 || tile.rounds > MAX_COOPERATIVE_ROUNDS {
        return Err(cooperative(CooperativeTileRule::RoundStructure));
    }
    // The split covers the sequence once across *every* round — exactly, or as
    // a suffix-padded extension. Participant `p` on round `r` folds the
    // contiguous range at index `r * participants + p`. `covers` is
    // deliberately not extended to know about rounds: it is the multi-pass
    // split's rule, where the partitions are the whole story, and teaching it
    // a second dimension would give one method two meanings.
    //
    // Coverage arithmetic is named separately from the participant/shape
    // agreement [`CooperativeTileRule::ContributorSplit`] still owns: a tile
    // whose split does not match its workgroup is a different defect from a
    // tile whose coverage statement is wrong.
    let partition = coverage.partition();
    verify_contributor_coverage(
        *coverage,
        contributors,
        tile.rounds,
        numerical_program(region)?.0,
        numerical_program(region)?.1,
    )?;
    // The iteration domain appends one axis per *participant*, not one per
    // partition of the whole fold: the launch runs one invocation per (output,
    // participant) pair whatever the round count, because rounds are a loop
    // inside each invocation rather than more invocations.
    if partition.partitions != participants
        || partial_reduction_shape(output_shape, partition)
            .is_none_or(|shape| region.index.iteration_shape != shape)
    {
        return Err(cooperative(CooperativeTileRule::ContributorSplit));
    }
    verify_cooperative_tile(tile)
}
