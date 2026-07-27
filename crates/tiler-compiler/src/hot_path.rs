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
use crate::workcount::{REGION_FORMATIONS, REQUEST_SUBJECT_REBUILDS};

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
#[test]
fn hot_path_compile_time_by_shape() {
    const REPEATS: u32 = 5;
    for (rows, columns) in [(4, 3), (1024, 3), (4, 1024)] {
        let program = program(rows, columns);
        let _ = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32);
        let start = Instant::now();
        for _ in 0..REPEATS {
            let _ = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32);
        }
        println!(
            "MEASURE compile {rows}x{columns}: {:?} per compile",
            start.elapsed() / REPEATS
        );
    }
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
