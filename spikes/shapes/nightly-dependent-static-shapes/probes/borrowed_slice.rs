#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![allow(incomplete_features)]

pub struct BorrowedShape<const EXTENTS: &'static [u64]>;

pub type Matrix = BorrowedShape<{ &[2, 3] }>;
