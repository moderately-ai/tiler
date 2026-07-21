#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;
struct ShapedValue<E>(PhantomData<E>);

fn add_scalar_right<E>(
    _: ShapedValue<E>,
    _: ShapedValue<StaticShape<0, { [] }>>,
) -> ShapedValue<E> {
    ShapedValue(PhantomData)
}

fn main() {
    let matrix = ShapedValue::<StaticShape<2, { [2, 3] }>>(PhantomData);
    let scalar = ShapedValue::<StaticShape<0, { [] }>>(PhantomData);
    let _: ShapedValue<StaticShape<2, { [2, 3] }>> = add_scalar_right(matrix, scalar);
}
