//! Measurements and work-count guards for the compile path.
//!
//! # What each half is for
//!
//! The measurement tests *print* and assert nothing about time. A timing
//! assertion would fail on a loaded machine and pass on a fast one, which makes
//! it a flake rather than a guard. What they are for is a reproducible number
//! to compare across a change: run them before and after and read the two.
//!
//! The guards *assert*, and they assert counts rather than durations, because
//! the cost this crate is trying to remove is duplicated work rather than slow
//! work. A count is stable across hosts and profiles, so it can be checked in
//! the ordinary gate.
//!
//! Reproduce the measurements with:
//!
//! ```text
//! cargo nextest run --release -p tiler-compiler -E 'test(hot_path)' --no-capture
//! ```
//!
//! Release matters: workspace crates build at `opt-level = 0` by default and
//! the compile path measures ~4x slower in dev.

use std::time::Instant;

use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

use crate::session::{NumericalContract, compile_governed};
use crate::workcount::{
    FRONTIER_ENUMERATIONS, REGION_FORMATIONS, REGION_GRAPH_BUILDS, REQUEST_SUBJECT_REBUILDS,
};

/// The governed scale-then-reduce program at one shape.
fn program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("a valid input key"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(OutputKey::new("result").expect("a valid output key"), sum)
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Reports compile time, and that it does not vary with the declared shape.
///
/// **The flatness is the finding, not the absolute number.** The cost is fixed
/// per compilation and independent of problem size, so it is structural rather
/// than data-driven — which is what makes it worth attacking at all.
///
/// **Report the minimum, not the mean.** Every perturbation a host applies —
/// a scheduler preemption, a frequency drop, a competing build — makes a
/// compile *slower*; none makes it faster. So the distribution has a hard floor
/// at the true cost and an unbounded tail of noise, and the minimum of enough
/// runs estimates that floor while the mean estimates the floor plus whatever
/// else the machine happened to be doing. The first measurement of this change
/// read 1.96 ms against a 1.49 ms baseline and looked like a regression; three
/// reruns read 1.16-1.30 ms. The mean of five was reporting the machine.
#[test]
fn hot_path_compile_time_by_shape() {
    /// Enough that the minimum is a stable floor rather than a lucky sample;
    /// at ~1 ms per compile the whole test still costs a fraction of a second.
    const REPEATS: u32 = 200;

    for (rows, columns) in [(4, 3), (1024, 3), (4, 1024)] {
        let program = program(rows, columns);
        // Warm the allocator and the branch predictors so the first timed run is
        // not measuring first-touch page faults.
        for _ in 0..8 {
            let _ = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32);
        }
        let mut best = std::time::Duration::MAX;
        let total = Instant::now();
        for _ in 0..REPEATS {
            let start = Instant::now();
            let _ = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32);
            best = best.min(start.elapsed());
        }
        println!(
            "MEASURE compile {rows}x{columns}: min {best:?}, mean {:?} over {REPEATS}",
            total.elapsed() / REPEATS,
        );
    }
}

/// Compiles in a loop long enough for a sampling profiler to attribute the cost.
///
/// **This is the harness that says *where* the time goes; the counters say
/// *how often* something ran.** A counter only reports on a site somebody
/// already suspected, so a programme driven by counters alone optimizes the
/// list it started with. A sampler has no such blind spot, and it is what
/// should choose which counters are worth keeping as guards.
///
/// It is `#[ignore]`d because it deliberately runs for seconds and asserts
/// nothing. Record it with `samply`, which is what found the region-graph
/// rebuild:
///
/// ```text
/// CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --tests -p tiler-compiler
/// TILER_PROFILE_SECONDS=20 samply record --save-only --unstable-presymbolicate \
///     --rate 4000 -o compile.profile.json.gz \
///     -- target/release/deps/tiler_compiler-<hash> \
///        --ignored --exact hot_path::hot_path_profile_loop --nocapture
/// ```
///
/// Three details are load-bearing. `CARGO_PROFILE_RELEASE_DEBUG=true` is
/// required: the release profile carries no debug information, and without it
/// every frame symbolicates to a bare hex address. `--unstable-presymbolicate`
/// writes the `*.syms.json` sidecar that holds the names — the profile's own
/// string table does not. And the harness must run long enough to sample; a
/// single compile is a millisecond, which is one sample.
///
/// `TILER_PROFILE_SECONDS` sets the duration and defaults to ten.
#[test]
#[ignore = "runs for seconds under a profiler; not part of the gate"]
fn hot_path_profile_loop() {
    let seconds = std::env::var("TILER_PROFILE_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);
    let program = program(4, 3);
    let deadline = Instant::now() + std::time::Duration::from_secs(seconds);
    let mut compiles = 0_u64;
    while Instant::now() < deadline {
        for _ in 0..64 {
            let _ = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32);
        }
        compiles += 64;
    }
    println!("MEASURE profile loop: {compiles} compiles in {seconds}s");
}

/// Reports how much of a compilation is planning rather than the request
/// boundary.
#[test]
fn hot_path_planning_share() {
    let program = program(4, 3);
    let compilations =
        compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32).expect("compiles");
    let rendered = compilations[0].explain().render();
    println!(
        "MEASURE alternatives: {}, explain records: {}, explain bytes: {}",
        compilations[0].alternatives().len(),
        rendered.lines().count(),
        rendered.len(),
    );
}

/// The request subject is reconstructed only where it is being *verified*.
///
/// **What the remaining count is, and why it is not zero.** `subject()` is now
/// a borrow of the stored authority, so no reader rebuilds. What still rebuilds
/// is `reconstructs_its_authority`, which exists to re-derive and compare —
/// the tamper check — plus `for_target`, which computes the authority in the
/// first place. Those are the verification, not waste, and removing them would
/// remove a check rather than a cost.
///
/// The bound is the measured count after that split. It fell from 57, and the
/// 45 that went were readers paying a verifier's price because one method
/// served both roles.
#[test]
fn the_request_subject_rebuild_count_does_not_regress() {
    /// Measured after separating the accessor from the tamper check.
    const MEASURED_BASELINE: usize = 12;

    let program = program(4, 3);
    let (compiled, rebuilds) = REQUEST_SUBJECT_REBUILDS
        .observe(|| compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32));
    compiled.expect("the governed program compiles");
    println!(
        "MEASURE {}s per compile: {rebuilds}",
        REQUEST_SUBJECT_REBUILDS.name()
    );
    assert!(
        rebuilds <= MEASURED_BASELINE,
        "one compile reconstructed the request subject {rebuilds} times, above the measured \
         baseline of {MEASURED_BASELINE}; a new call site is calling \
         `reconstructs_its_authority` where `subject()` would do",
    );
}

/// One compilation builds the whole-program region graph exactly once.
///
/// **The profiler found this one, not a reading of the code.** See
/// [`REGION_GRAPH_BUILDS`] for what the graph construction costs and why a
/// second one is duplicated work.
#[test]
fn one_compile_builds_the_region_graph_once() {
    let program = program(4, 3);
    let (compiled, builds) = REGION_GRAPH_BUILDS
        .observe(|| compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32));
    compiled.expect("the governed program compiles");
    println!(
        "MEASURE {}s per compile: {builds}",
        REGION_GRAPH_BUILDS.name()
    );
    assert_eq!(
        builds, 1,
        "one compile built the whole-program region graph {builds} times; \
         `RegionFormationOutcome` owns one and every planner entry point is handed it, so \
         anything above one is a call site deriving a value it already has",
    );
}

/// One compilation enumerates each distinct region subject's frontier once.
///
/// **The bound is the number of distinct subjects, which is the point.** The
/// enumeration is a pure function of the request, the subject, and the
/// providers, and only the subject varies within a target compile — so the
/// count used to be the number of (cover, region) pairs, 48 of them, over 17
/// distinct subjects. The reduction region alone was enumerated 8 times because
/// eight covers place it.
///
/// An equality rather than a bound, because both directions are interesting: a
/// rise means a call site stopped consulting the memo, and a *fall* means the
/// cover enumeration changed shape and the two numbers below no longer describe
/// this program.
#[test]
fn one_compile_enumerates_each_distinct_region_subject_once() {
    /// The distinct region subjects the governed five-operation program covers.
    const DISTINCT_SUBJECTS: usize = 17;

    let program = program(4, 3);
    let (compiled, enumerations) = FRONTIER_ENUMERATIONS
        .observe(|| compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32));
    compiled.expect("the governed program compiles");
    println!(
        "MEASURE {}s per compile: {enumerations}",
        FRONTIER_ENUMERATIONS.name()
    );
    assert_eq!(
        enumerations, DISTINCT_SUBJECTS,
        "one compile enumerated {enumerations} implementation frontiers for {DISTINCT_SUBJECTS} \
         distinct region subjects; every cover that places a region asks for that region's \
         frontier, so anything above the distinct count is a re-derivation the memo should \
         have served",
    );
}

/// One compilation derives the region formation exactly once.
///
/// **This is the structural claim the whole optimization rests on, and it is
/// why the entry points take the formation instead of deriving it.** The
/// outcome is a pure function of the program, budgets, and contract, and all
/// three are fixed for the duration of a target compile — so a second
/// derivation reproduces a value that already exists, at the cost of a search
/// bounded by `region_expansions`.
///
/// It used to run once in the pipeline, again in `enumerate_covers`, once per
/// cover in `verify_cover`, once per retained plan, and once per alternative.
/// After the change every one of those takes `&RegionFormationOutcome`, so a
/// new call site that wanted to derive its own would have to say so explicitly
/// rather than doing it by reaching for the old signature.
#[test]
fn one_compile_derives_the_region_formation_once() {
    let program = program(4, 3);
    let (compiled, formations) = REGION_FORMATIONS
        .observe(|| compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32));
    compiled.expect("the governed program compiles");
    println!(
        "MEASURE {}s per compile: {formations}",
        REGION_FORMATIONS.name()
    );
    assert_eq!(
        formations, 1,
        "one compile derived the region formation {formations} times; it is a pure function of \
         inputs fixed for the target, so anything above one is a call site re-deriving a value \
         it was handed",
    );
}
