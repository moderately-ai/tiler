#![feature(generic_const_parameter_types)]
#![feature(min_adt_const_params)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;
struct ShapedValue<E>(PhantomData<E>);

fn reduce_axis_as<
    const INPUT_RANK: usize,
    const INPUT_EXTENTS: [u64; INPUT_RANK],
    const OUTPUT_RANK: usize,
    const OUTPUT_EXTENTS: [u64; OUTPUT_RANK],
>(
    _: ShapedValue<StaticShape<INPUT_RANK, INPUT_EXTENTS>>,
    _runtime_axis: usize,
) -> Result<ShapedValue<StaticShape<OUTPUT_RANK, OUTPUT_EXTENTS>>, ()> {
    // A real graph checks the inferred runtime-selected output before this
    // capability is constructed.
    Ok(ShapedValue(PhantomData))
}

fn main() {
    let matrix = ShapedValue::<StaticShape<2, { [2, 3] }>>(PhantomData);
    let _: ShapedValue<StaticShape<1, { [2] }>> =
        reduce_axis_as::<2, { [2, 3] }, 1, { [2] }>(matrix, 1).unwrap();
}
