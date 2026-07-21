#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

pub struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;
