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
use crate::workcount::REQUEST_SUBJECT_REBUILDS;

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

/// The request-subject rebuild count does not regress.
///
/// **A ratchet, not the target.** The subject is a pure function of an
/// immutable request, and `VerifiedTargetRequest` already stores it as its
/// `authority`, so every reconstruction beyond the first is duplicated work by
/// definition. The bound below is the *measured current* count, not a number
/// anybody chose: it exists so the figure cannot quietly grow while the ticket
/// that reduces it is still open.
///
/// `store-the-verified-request-subject-instead-of-rebuilding-it` lowers this to
/// a small constant and tightens the bound in the same change. Until then, a
/// failure here means a new call site started reconstructing the subject —
/// which is worth knowing immediately, because the sites that already do it are
/// inside per-region and per-candidate loops.
#[test]
fn the_request_subject_rebuild_count_does_not_regress() {
    /// Measured on 2026-07-27, before any reduction. See the note above.
    const MEASURED_BASELINE: usize = 57;

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
         baseline of {MEASURED_BASELINE}; a new call site is rebuilding a value it could borrow",
    );
}
