#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

use std::marker::PhantomData;
use tiler_ir::semantic::{F32, ShapedValue, Value};
use tiler_ir::shape::StaticShape;

fn forge(value: Value<F32>) -> ShapedValue<F32, StaticShape<1, { [4] }>> {
    ShapedValue {
        value,
        evidence: PhantomData,
    }
}

fn main() {}
