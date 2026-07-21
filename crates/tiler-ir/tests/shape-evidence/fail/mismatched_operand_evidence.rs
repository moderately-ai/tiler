#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

use tiler_ir::semantic::{F32, F32Add, SemanticProgramBuilder, ShapedValue};
use tiler_ir::shape::StaticShape;

type Matrix = StaticShape<2, { [2, 3] }>;
type Transposed = StaticShape<2, { [3, 2] }>;

fn add(
    builder: &mut SemanticProgramBuilder,
    left: ShapedValue<F32, Matrix>,
    right: ShapedValue<F32, Transposed>,
) {
    let _ = F32Add::apply_shaped(builder, left, right);
}

fn main() {}
