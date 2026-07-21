#![feature(generic_const_parameter_types)]
#![allow(incomplete_features)]

use tiler_ir::shape::{Shape, ShapeEvidence, ShapeExpectation};

struct Forged;

impl ShapeEvidence for Forged {
    fn matches(_: &Shape) -> bool {
        true
    }

    fn expectation() -> ShapeExpectation {
        ShapeExpectation::Rank(0)
    }
}

fn main() {}
