//! The loop-carried vertical's runs, in two halves that fail for unrelated
//! reasons.
//!
//! Everything above the measured cases is deterministic and runs on every
//! host: the operand pair and its counts, the launch geometry the scheduled
//! program publishes, the four property perturbations, and emission of both
//! kernels. The measured runs report their boundary — or its absence — rather
//! than skipping, and they never treat source emission as execution.

use tiler_reference::CooperativeCellLayout;

use super::{
    CONTRIBUTOR_SET_BITS, GROUPING_SENSITIVE_BITS, PARTICIPANTS, ROWS, dropped_round_grouping,
    emit_region, grouped_bits, identity_corruption_census, measured_execution,
    multi_round_grouping, multi_round_region, participant_major_grouping, scheduled_grouping,
    scheduled_launch, single_round_grouping, single_round_region, source_without_barriers,
    staging_slots,
};
use crate::dispatch::Launch;
use crate::measurement::require_or_report;

/// Each operand set covers what the other cannot, and the populations are
/// pinned rather than described.
///
/// The contributor-set half leaves no identity-corruption undetected under
/// either declared grouping and cannot tell those groupings apart. The
/// grouping-sensitive half tells them apart and lets some identity-corruptions
/// escape. Neither half replaces the other.
#[test]
fn the_operand_pair_covers_what_each_half_alone_cannot() {
    let multi = multi_round_grouping();
    let neighbour = participant_major_grouping();
    let single = single_round_grouping();

    let (set_population, set_escaped) = identity_corruption_census(&CONTRIBUTOR_SET_BITS, multi);
    assert_eq!(
        set_population, 12,
        "one identity replacement per contributor"
    );
    assert_eq!(
        set_escaped, 0,
        "a dropped contributor must change the exact powers-of-two sum"
    );

    let (group_population, group_escaped) =
        identity_corruption_census(&GROUPING_SENSITIVE_BITS, multi);
    assert_eq!(group_population, 12);
    assert!(
        group_escaped > 0,
        "the grouping-sensitive set is not a contributor-set claim; {group_escaped} identity \
         replacement(s) must escape so the pair stays two questions"
    );

    assert_eq!(
        grouped_bits(&CONTRIBUTOR_SET_BITS, multi),
        grouped_bits(&CONTRIBUTOR_SET_BITS, neighbour),
        "exact operands cannot tell the two layouts apart"
    );
    assert_eq!(
        grouped_bits(&CONTRIBUTOR_SET_BITS, multi),
        grouped_bits(&CONTRIBUTOR_SET_BITS, single),
        "exact operands cannot tell the multi-round grouping from its one-round neighbour"
    );
    assert_ne!(
        grouped_bits(&GROUPING_SENSITIVE_BITS, multi),
        grouped_bits(&GROUPING_SENSITIVE_BITS, neighbour),
        "the grouping-sensitive set must separate the declared layout from the participant-major \
         neighbour"
    );
    assert_ne!(
        grouped_bits(&GROUPING_SENSITIVE_BITS, multi),
        grouped_bits(&GROUPING_SENSITIVE_BITS, single),
        "the grouping-sensitive set must separate the multi-round grouping from a dropped-round \
         regrouping"
    );
}

/// Launch geometry is the scheduled program's, not the staging allocation and
/// not a fixture constant.
///
/// On this fixture staging slots and the workgroup width are both three, so
/// staging cannot distinguish the launch. The grid (6) and a serial width (1)
/// can, which is why those are the launch-width perturbations rather than the
/// staging count.
#[test]
fn launch_geometry_comes_from_the_scheduled_program_not_staging_or_a_fixture() {
    let region = multi_round_region();
    let launch = scheduled_launch(&region);
    assert_eq!(
        launch,
        Launch {
            grid_threads: ROWS * PARTICIPANTS,
            threads_per_workgroup: PARTICIPANTS,
        }
    );
    assert_eq!(
        staging_slots(&region),
        launch.threads_per_workgroup,
        "staging slots cannot distinguish launch width on this fixture"
    );
    assert_ne!(
        launch.grid_threads, launch.threads_per_workgroup,
        "reading the grid as the workgroup width must be a different geometry"
    );
    assert_ne!(
        launch.threads_per_workgroup, 1,
        "a serial workgroup width must be a different geometry"
    );
    assert_ne!(
        launch.threads_per_workgroup, launch.grid_threads,
        "launching the whole grid as one workgroup must be a different geometry"
    );
    let grouping = scheduled_grouping(&region);
    assert_eq!(grouping, multi_round_grouping());
    assert_eq!(grouping.layout, CooperativeCellLayout::RoundMajor);
    assert_ne!(
        grouping.participants, launch.grid_threads,
        "inferring participants from the grid would invent a grouping the tile did not declare"
    );
}

/// Round arithmetic, barrier placement, launch width, and grouping each fail
/// for a reason the others do not produce.
#[test]
fn each_property_perturbation_fails_for_its_own_reason() {
    let region = multi_round_region();
    let emitted = emit_region(&region).expect("the multi-round subject emits");
    let declared = grouped_bits(&GROUPING_SENSITIVE_BITS, emitted.grouping);

    // Round contribution arithmetic: the one-round regrouping a dropped round
    // term computes on the same six cells.
    let dropped = grouped_bits(&GROUPING_SENSITIVE_BITS, dropped_round_grouping());
    assert_ne!(
        declared, dropped,
        "dropping the round term must change the grouping-sensitive bits"
    );

    // Grouping: the participant-major neighbour is a different order of the
    // same cells, not a different fold.
    let neighbour = grouped_bits(&GROUPING_SENSITIVE_BITS, participant_major_grouping());
    assert_ne!(
        declared, neighbour,
        "the participant-major neighbour must change the grouping-sensitive bits"
    );
    assert_ne!(
        dropped, neighbour,
        "round-arithmetic and grouping perturbations must not collapse into one refusal"
    );

    // Launch width: the scheduled 6×3 is not 6×1 and not 6×6.
    let scheduled = Launch {
        grid_threads: emitted.grid_threads,
        threads_per_workgroup: emitted.threads_per_workgroup,
    };
    assert_ne!(
        scheduled.threads_per_workgroup, 1,
        "a serial launch width is not the scheduled workgroup"
    );
    assert_ne!(
        scheduled.threads_per_workgroup, scheduled.grid_threads,
        "a single-workgroup launch of the whole grid is not the scheduled workgroup"
    );

    // Barrier placement: the execution subject is the emitted peeled body. A
    // source with its fences deleted is a different program and is not
    // dispatched.
    let source = emitted.unit.source();
    let stripped = source_without_barriers(source);
    assert!(
        source.contains("threadgroup_barrier"),
        "the execution subject must carry the fences the tile declared: {source}"
    );
    assert!(
        !stripped.contains("threadgroup_barrier"),
        "the barrier-stripped source must contain no fence"
    );
    assert_ne!(
        source, stripped,
        "a barrier-stripped source must not be the execution subject"
    );
}

/// The accepted single-round neighbour emits, and its launch and grouping are
/// the scheduled program's.
#[test]
fn the_single_round_neighbour_emits_its_declared_launch_and_grouping() {
    let region = single_round_region();
    let emitted = emit_region(&region).expect("the single-round neighbour emits");
    assert_eq!(emitted.grouping, single_round_grouping());
    assert_eq!(emitted.grid_threads, ROWS * PARTICIPANTS);
    assert_eq!(emitted.threads_per_workgroup, PARTICIPANTS);
    let source = emitted.unit.source();
    assert_eq!(
        source.matches("threadgroup_barrier").count(),
        1,
        "the single-round neighbour fences its one handoff once: {source}"
    );
    assert_eq!(
        source.matches("  // tile phase 0").count(),
        1,
        "the single-round neighbour stages once, not once per round: {source}"
    );
}

/// The multi-round subject emits the peeled body, and its launch and grouping
/// are the scheduled program's.
#[test]
fn the_multi_round_subject_emits_its_declared_launch_and_grouping() {
    let region = multi_round_region();
    let emitted = emit_region(&region).expect("the multi-round subject emits");
    assert_eq!(emitted.grouping, multi_round_grouping());
    assert_eq!(emitted.grid_threads, ROWS * PARTICIPANTS);
    assert_eq!(emitted.threads_per_workgroup, PARTICIPANTS);
    let source = emitted.unit.source();
    assert!(
        source.contains("// serial loop over [1, 2)"),
        "the multi-round subject must emit the 1..rounds loop: {source}"
    );
    assert_eq!(
        source.matches("  // tile phase 0").count(),
        2,
        "the peel and the round body must each stage: {source}"
    );
    assert!(
        source.matches("threadgroup_barrier").count() >= 3,
        "the peeled phase fence plus the two in-loop fences must be present: {source}"
    );
}

fn assert_device_matches(label: &str, emitted: &super::EmittedCooperative, bits: &[u32]) {
    let expected = grouped_bits(bits, emitted.grouping);
    let Some(observed) = require_or_report(label, measured_execution(emitted, bits)) else {
        return;
    };
    assert_eq!(
        observed, expected,
        "{label} returned {observed:08x?} and the declared grouping requires {expected:08x?}"
    );
    eprintln!(
        "{label}: bit-for-bit agreement on {} element(s) under grouping {:?}; backend Metal, \
         profile {}, launch {}x{}, not a threaded CPU realization",
        expected.len(),
        emitted.grouping,
        emitted.declaration.profile().profile_key(),
        emitted.grid_threads,
        emitted.threads_per_workgroup,
    );
}

/// The accepted single-round neighbour agrees with its declared grouping on
/// this host's Metal backend.
#[test]
fn the_single_round_neighbour_agrees_on_the_measured_row() {
    let emitted = emit_region(&single_round_region()).expect("the single-round neighbour emits");
    assert_device_matches(
        "loop-carried single-round neighbour",
        &emitted,
        &CONTRIBUTOR_SET_BITS,
    );
}

/// The multi-round subject agrees with its declared grouping on the
/// contributor-set input.
#[test]
fn the_multi_round_subject_agrees_on_the_contributor_set_on_the_measured_row() {
    let emitted = emit_region(&multi_round_region()).expect("the multi-round subject emits");
    assert_device_matches(
        "loop-carried multi-round contributor-set",
        &emitted,
        &CONTRIBUTOR_SET_BITS,
    );
}

/// The multi-round subject agrees with its declared grouping on the
/// grouping-sensitive input, where the neighbour groupings disagree.
#[test]
fn the_multi_round_subject_agrees_on_the_grouping_sensitive_input_on_the_measured_row() {
    let emitted = emit_region(&multi_round_region()).expect("the multi-round subject emits");
    let expected = grouped_bits(&GROUPING_SENSITIVE_BITS, emitted.grouping);
    let neighbour = grouped_bits(&GROUPING_SENSITIVE_BITS, participant_major_grouping());
    let dropped = grouped_bits(&GROUPING_SENSITIVE_BITS, dropped_round_grouping());
    assert_ne!(expected, neighbour);
    assert_ne!(expected, dropped);
    assert_device_matches(
        "loop-carried multi-round grouping-sensitive",
        &emitted,
        &GROUPING_SENSITIVE_BITS,
    );
}
