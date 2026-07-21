#![feature(generic_const_args)]
#![feature(generic_const_items)]
#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
#![feature(min_generic_const_args)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;
struct ShapedValue<E>(PhantomData<E>);

type const REDUCED_RANK<const RANK: usize>: usize = const { RANK - 1 };

type const REMOVE_AXIS<
    const RANK: usize,
    const EXTENTS: [u64; RANK],
    const AXIS: usize,
>: [u64; REDUCED_RANK::<RANK>] = const {
    assert!(AXIS < RANK);
    let mut result = [0; REDUCED_RANK::<RANK>];
    let mut source = 0;
    let mut destination = 0;
    while source < RANK {
        if source != AXIS {
            result[destination] = EXTENTS[source];
            destination += 1;
        }
        source += 1;
    }
    result
};

fn reduce_axis<
    const RANK: usize,
    const EXTENTS: [u64; RANK],
    const AXIS: usize,
>(
    _: ShapedValue<StaticShape<RANK, EXTENTS>>,
) -> ShapedValue<
    StaticShape<
        { REDUCED_RANK::<RANK> },
        { REMOVE_AXIS::<RANK, EXTENTS, AXIS> },
    >,
> {
    ShapedValue(PhantomData)
}

fn main() {
    let matrix = ShapedValue::<StaticShape<2, { [2, 3] }>>(PhantomData);
    let _: ShapedValue<StaticShape<1, { [2] }>> = reduce_axis::<2, { [2, 3] }, 1>(matrix);
}
