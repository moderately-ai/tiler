//! Whole-region entry gate and the dispatch to one family's verifier.
//!
//! What lives here is every obligation a region carries whatever it computes:
//! launch coverage, the tail policy, the execution binding's own arithmetic,
//! and a cooperative tile's participant space — decided before the program is
//! read, because the participant count they settle is load-bearing for the
//! proof rules that follow. The seam is exactly where an obligation stops
//! being universal: the moment the region's program decides the rule, the
//! region leaves this file for the family gate that owns it.

use crate::schedule::blocked::{prove_blocked_bijection, prove_blocked_predicated_cover};
use crate::schedule::error::{BlockedWorkgroupRule, ScheduledRegionDiagnostic, VectorLaneRule};
use crate::schedule::model::{
    ExecutionBinding, ReductionTopology, RegionProgram, ScalarProgram, ScheduledRegion, TailPolicy,
    cooperative_tile, element_count,
};

use super::contraction::verify_contraction;
use super::copy::verify_partitioned_copy;
use super::diagnostics::{blocked, vector_lane};
use super::elementwise::{
    verify_pointwise_bf16, verify_pointwise_f32, verify_strict_affine_u4_dequantize,
};
use super::reduction::verify_access_and_semantics;
use super::tile::verify_participant_space;

/// Runs the intrinsic schedule verifier over an assembled region.
pub(super) fn verify_intrinsic(region: &ScheduledRegion) -> Result<(), ScheduledRegionDiagnostic> {
    let iteration_count = element_count(&region.index.iteration_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    let schedule = &region.schedule;
    if schedule.launch.threads_per_workgroup != schedule.threads_per_workgroup
        || schedule.threads_per_workgroup == 0
        || !schedule.launch.zero_work_skips_dispatch
        || schedule.work_items != iteration_count
    {
        return Err(ScheduledRegionDiagnostic::LaunchCoverage);
    }
    match schedule.tail {
        TailPolicy::Exact => {
            // The exact grid population is the binding's invocation
            // population. The scalar and blocked bindings run one invocation
            // per iteration coordinate; the fixed-vector map runs one packet
            // per `lanes` coordinates, and its packet arithmetic is proved in
            // the binding match below rather than compared against a count it
            // is deliberately not equal to.
            if !matches!(schedule.binding, ExecutionBinding::FixedVectorMap { .. })
                && schedule.launch.grid_threads != iteration_count
            {
                return Err(ScheduledRegionDiagnostic::LaunchCoverage);
            }
        }
        TailPolicy::Predicated => {
            // The fixed-vector map admits `Exact` alone, refused under its own
            // name before the cooperative pairing rule: a producer that asked
            // for a predicated vector tail needs to be told the tail is the
            // defect, not the topology.
            if matches!(schedule.binding, ExecutionBinding::FixedVectorMap { .. }) {
                return Err(vector_lane(VectorLaneRule::ExactTailRequired));
            }
            if !matches!(
                schedule.reduction,
                ReductionTopology::CooperativeContraction { .. }
            ) || !matches!(schedule.binding, ExecutionBinding::BlockedWorkgroup { .. })
            {
                return Err(ScheduledRegionDiagnostic::LaunchCoverage);
            }
        }
    }
    match &schedule.binding {
        ExecutionBinding::GlobalLinearInvocation => {
            if matches!(
                schedule.reduction,
                ReductionTopology::CooperativeContraction { .. }
            ) {
                return Err(blocked(BlockedWorkgroupRule::BindingRequired));
            }
        }
        // The accepted fixed-vector map slice: `Exact` tail (proved above),
        // the map-parallel topologies alone, checked `N mod W == 0`, and the
        // accepted launch identity `work_items = N`, `grid_threads = N / W`.
        // Packet `p`, lane `l` owns output `p * W + l`, which stays inside the
        // domain exactly because the division is exact. Nothing here reads
        // the numerical realization: grouping independent outputs into
        // packets changes no operand, rounding site, or contributor order
        // (ADR 0093 decision 2), so no permission is consulted or consumed.
        // Nothing here consults a target either — every fact is the region's
        // own literal `N`, `W`, and launch.
        ExecutionBinding::FixedVectorMap { lanes } => {
            match schedule.reduction {
                ReductionTopology::None | ReductionTopology::Serial { .. } => {}
                // Refused by name, not by wildcard semantics: every other
                // topology awaits its own accepted vector boundary, and the
                // arms are spelled out so a topology added later is a build
                // error here rather than a silent admission.
                ReductionTopology::MultiPass { .. }
                | ReductionTopology::Contraction { .. }
                | ReductionTopology::LiveContraction { .. }
                | ReductionTopology::CooperativeWorkgroup { .. }
                | ReductionTopology::CooperativeContraction { .. } => {
                    return Err(vector_lane(VectorLaneRule::UnsupportedReduction));
                }
            }
            let lane_count = lanes.get();
            // Divisibility is a fact of `N` and `W` alone, decided before the
            // launch is read so a nondivisible domain is named as such rather
            // than as whatever launch the producer happened to state. A zero
            // domain is a multiple of every admitted width and dispatches
            // nothing under `zero_work_skips_dispatch`.
            if !iteration_count.is_multiple_of(lane_count) {
                return Err(vector_lane(VectorLaneRule::NondivisibleCoverage));
            }
            // The launch is checked as `grid_threads * W == N` with checked
            // multiplication rather than as a division, so an overflowing
            // packet product and a wrong packet count are distinct refusals.
            // With divisibility already proved, equality holds exactly for
            // `grid_threads = N / W` — and `grid_threads = N` with a
            // reinterpreted builtin is exactly what the second refusal
            // catches, for every `W >= 2` the lane type can hold.
            let Some(covered) = schedule.launch.grid_threads.checked_mul(lane_count) else {
                return Err(vector_lane(VectorLaneRule::PacketArithmeticOverflow));
            };
            if covered != iteration_count {
                return Err(vector_lane(VectorLaneRule::PacketPopulation));
            }
        }
        ExecutionBinding::BlockedWorkgroup { block, workgroups } => {
            if !matches!(
                schedule.reduction,
                ReductionTopology::CooperativeContraction { .. }
            ) {
                return Err(blocked(BlockedWorkgroupRule::BindingForbidden));
            }
            match schedule.tail {
                TailPolicy::Exact => prove_blocked_bijection(
                    &region.index.iteration_shape,
                    block,
                    workgroups,
                    schedule.work_items,
                    schedule.threads_per_workgroup,
                    schedule.launch.grid_threads,
                )?,
                TailPolicy::Predicated => prove_blocked_predicated_cover(
                    &region.index.iteration_shape,
                    block,
                    workgroups,
                    schedule.work_items,
                    schedule.threads_per_workgroup,
                    schedule.launch.grid_threads,
                )?,
            }
        }
    }
    // A cooperative tile's participant space is decided here, beside launch
    // coverage, rather than with the rest of the tile's rules — because the
    // participant count it determines is load-bearing *before* those rules run.
    // `owned_output_positions` divides the work items by that count during proof
    // verification, so a malformed space or one disagreeing with the launch
    // would otherwise surface as a proof-reference mismatch: fail-closed, but
    // naming the ownership proof for a defect entirely in the tile.
    if let Some(tile) = cooperative_tile(&schedule.reduction) {
        verify_participant_space(
            tile.coordinates.participants,
            schedule.threads_per_workgroup,
        )?;
    }
    let scalar = match &region.index.program {
        // The copy gate owns everything after the shared launch, binding, and
        // participant gates above; its rule order is the accepted
        // partitioned-copy precedence.
        RegionProgram::PartitionedCopy(program) => {
            return verify_partitioned_copy(region, program);
        }
        RegionProgram::Numerical { scalar, .. } => scalar,
    };
    match scalar {
        ScalarProgram::StrictAffineU4Dequantize { .. } => {
            let [codes, scale, zero_point, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_strict_affine_u4_dequantize(region, codes, scale, zero_point, write)
        }
        // A pointwise region reads one boundary tensor per expression leaf, so
        // its access count is a property of its scalar program rather than a
        // constant. The reduction families still read exactly one tensor: their
        // multi-input stories are separately owned, and a second read here would
        // leave the contributor domain unable to say which access it counts.
        ScalarProgram::PointwiseF32(expression) => {
            let Some((write, reads)) = region.index.accesses.split_last() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_pointwise_f32(region, expression, reads, write)
        }
        // The same access contract at a different width, plus the one obligation
        // that is `bf16`'s alone: the region's declared canonical arithmetic NaN
        // payload must be its own format's.
        ScalarProgram::PointwiseBf16(expression) => {
            let Some((write, reads)) = region.index.accesses.split_last() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_pointwise_bf16(region, expression, reads, write)
        }
        // A fold carrying an epilogue reads one tensor like every other fold: the
        // epilogue's own leaf is the folded value rather than a boundary, which
        // is why it joins this group instead of the pointwise one whose read
        // count is its expression's leaf count.
        ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::StrictSerialMaximum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            let [read, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_access_and_semantics(region, read, write)
        }
        // The registered operation's exact-two semantic signature requires two
        // reads, and this scheduled form preserves that arity. ADR 0087's fifth
        // structural rule is independent: it limits one index's participation
        // across operands, not the operation's operand count.
        ScalarProgram::StrictTensorContraction { .. } => {
            let [left, right, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_contraction(region, left, right, write)
        }
    }
}
