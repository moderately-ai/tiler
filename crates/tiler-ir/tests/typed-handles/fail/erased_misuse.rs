use tiler_ir::semantic::{F32, F32Add, SemanticProgramBuilder, Value, ValueId};

fn erased(builder: &mut SemanticProgramBuilder, left: Value<F32>, right: ValueId) {
    let _ = F32Add::apply(builder, left, right);
}

fn main() {}
