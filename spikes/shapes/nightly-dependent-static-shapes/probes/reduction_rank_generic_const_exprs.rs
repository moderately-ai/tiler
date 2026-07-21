#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

struct Rank<const RANK: usize>;
struct ShapedValue<E>(PhantomData<E>);

fn reduce_axis<const RANK: usize, const AXIS: usize>(
    _: ShapedValue<Rank<RANK>>,
) -> ShapedValue<Rank<{ RANK - 1 }>>
where
    [(); RANK - 1]:,
{
    let _ = AXIS;
    ShapedValue(PhantomData)
}

fn main() {
    let matrix = ShapedValue::<Rank<2>>(PhantomData);
    let _: ShapedValue<Rank<1>> = reduce_axis::<2, 1>(matrix);
}
