use tiler_ir::semantic::{F32, Value, ValueId};

fn forge(id: ValueId) -> Value<F32> {
    Value::from_verified(id)
}

fn main() {}
