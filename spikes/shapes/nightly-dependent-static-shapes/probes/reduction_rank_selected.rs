#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

struct Rank<const RANK: usize>;
struct ShapedValue<E>(PhantomData<E>);

fn reduce_axis<const RANK: usize, const AXIS: usize>(
    _: ShapedValue<Rank<RANK>>,
) -> ShapedValue<Rank<{ RANK - 1 }>> {
    let _ = AXIS;
    ShapedValue(PhantomData)
}

fn main() {}
