#![feature(generic_const_exprs)]
#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;
struct ShapedValue<E>(PhantomData<E>);

const fn remove_axis<const RANK: usize>(
    extents: [u64; RANK],
    axis: usize,
) -> [u64; RANK - 1]
where
    [(); RANK - 1]:,
{
    assert!(axis < RANK);
    let mut result = [0; RANK - 1];
    let mut source = 0;
    let mut destination = 0;
    while source < RANK {
        if source != axis {
            result[destination] = extents[source];
            destination += 1;
        }
        source += 1;
    }
    result
}

fn reduce_axis<
    const RANK: usize,
    const EXTENTS: [u64; RANK],
    const AXIS: usize,
>(
    _: ShapedValue<StaticShape<RANK, EXTENTS>>,
) -> ShapedValue<StaticShape<{ RANK - 1 }, { remove_axis(EXTENTS, AXIS) }>>
where
    [(); RANK - 1]:,
{
    ShapedValue(PhantomData)
}

fn main() {
    let matrix = ShapedValue::<StaticShape<2, { [2, 3] }>>(PhantomData);
    let _: ShapedValue<StaticShape<1, { [2] }>> = reduce_axis::<2, { [2, 3] }, 1>(matrix);
}
