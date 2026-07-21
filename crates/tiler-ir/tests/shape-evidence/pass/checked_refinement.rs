#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

use tiler_ir::semantic::{F32, F32Add, InputKey, SemanticProgramBuilder, ShapedValue};
use tiler_ir::shape::{Shape, StaticShape};

fn main() {
    type Matrix = StaticShape<2, { [2, 3] }>;

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let left = builder
        .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let left: ShapedValue<F32, Matrix> = builder.refine(left).unwrap();
    let right: ShapedValue<F32, Matrix> = builder.refine(right).unwrap();
    let result: ShapedValue<F32, Matrix> =
        F32Add::apply_shaped(&mut builder, left, right).unwrap();
    let _ = result.weaken();
}
