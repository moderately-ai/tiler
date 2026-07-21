#![feature(min_adt_const_params)]
#![allow(incomplete_features)]

pub struct StaticShape<const RANK: usize, const EXTENTS: [u64; RANK]>;
