#![feature(min_adt_const_params)]
#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

pub struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;

pub type Matrix = StaticShape<2, { [2, 3] }>;
