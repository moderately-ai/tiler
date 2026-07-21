use tiler_ir::semantic::{SameShape, ShapeWitness};

fn duplicate(witness: ShapeWitness<SameShape>) -> ShapeWitness<SameShape> {
    ShapeWitness { ..witness }
}

fn main() {}
