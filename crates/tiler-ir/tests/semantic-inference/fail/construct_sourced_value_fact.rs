use tiler_ir::semantic::{F32, ValueFact};
use tiler_ir::shape::SourcedShape;

fn construct(shape: SourcedShape) {
    let _ = ValueFact::new(F32::resolved_type(), shape);
}

fn main() {
    let _ = construct;
}
