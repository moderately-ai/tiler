use tiler_ir::shape::SourcedExtent;

fn classify(extent: SourcedExtent) -> u8 {
    match extent {
        SourcedExtent::Static(_) => 1,
        SourcedExtent::Symbol(_) => 2,
    }
}

fn main() {
    let _ = classify;
}
