#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

use tiler_ir::shape::StaticShape;

type Invalid = StaticShape<2, { [1, 2, 3] }>;

fn main() {
    let _ = std::mem::size_of::<Invalid>();
}
