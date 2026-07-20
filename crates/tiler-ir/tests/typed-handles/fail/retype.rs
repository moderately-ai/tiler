use tiler_ir::semantic::{F32, Value, ValueTypeMarker};

enum External {}
impl ValueTypeMarker for External {}

fn retype(value: Value<F32>) -> Value<External> {
    value
}

fn main() {}
