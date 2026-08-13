use tiler_ir::shape::{Shape, SourcedShape};

fn main() {
    let _ = SourcedShape::Static(Shape::from_dims([2, 3]));
}
