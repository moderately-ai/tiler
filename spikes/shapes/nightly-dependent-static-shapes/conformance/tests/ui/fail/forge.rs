use std::marker::PhantomData;

use nightly_shape_api::{F32, ShapedValue, StaticShape, Value};

type Matrix = StaticShape<2, { [2, 3] }>;

fn value<T>() -> Value<T> {
    panic!()
}

fn main() {
    let _ = ShapedValue::<F32, Matrix> {
        value: value(),
        evidence: PhantomData,
    };
}
