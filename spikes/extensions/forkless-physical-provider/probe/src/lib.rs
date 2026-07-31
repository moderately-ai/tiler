//! Shared fixtures for the composition probe.
//!
//! The probe's claims live in `tests/`; this library holds only what more than
//! one of them needs. It is a library rather than a binary because the evidence
//! is a set of independently reportable assertions, and a binary would collapse
//! them into one exit code.

use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Rows of the probe's semantic program.
pub const ROWS: u64 = 4;

/// Reduced extent of the probe's semantic program.
pub const COLUMNS: u64 = 8;

/// Builds the bounded profile's scale-then-reduce program.
///
/// The same program shape `prototypes/serial-sum-compile` compiles, built here
/// from the public `tiler_ir::semantic` surface so the probe's compile run is a
/// genuine out-of-tree caller rather than an in-workspace shortcut.
///
/// # Panics
///
/// Panics if the governed semantic profile stops composing this program, which
/// is a defect in Tiler rather than a reachable input.
#[must_use]
pub fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
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
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}
