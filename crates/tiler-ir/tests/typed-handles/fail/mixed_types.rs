use tiler_ir::semantic::{F32, F32Add, SemanticProgramBuilder, Value, ValueTypeMarker};

enum External {}
impl ValueTypeMarker for External {}

fn mixed(
    builder: &mut SemanticProgramBuilder,
    left: Value<F32>,
    right: Value<External>,
) {
    let _ = F32Add::apply(builder, left, right);
}

fn main() {}
