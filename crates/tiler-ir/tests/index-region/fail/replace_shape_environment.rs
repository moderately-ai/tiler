use tiler_ir::index::IndexRegionBuilder;

// A region resolves every symbolic extent against exactly one environment, and
// that environment is fixed by the constructor before any dimension, boundary,
// or divisor exists. There is no setter to replace it with, so a second
// environment is unrepresentable rather than discouraged: the name below does
// not resolve at all.
fn main() {
    let _ = IndexRegionBuilder::with_shape_environment;
}
