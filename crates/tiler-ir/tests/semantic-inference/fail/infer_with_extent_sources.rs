use tiler_ir::semantic::{
    FrozenSemanticRegistry, OperationAttributes, ValueFact, multiply_f32_op,
};
use tiler_ir::shape::Shape;

fn main() {
    let registry = FrozenSemanticRegistry::standard().unwrap();
    let operand = ValueFact::new(
        tiler_ir::semantic::F32::resolved_type(),
        Shape::from_dims([2]),
    );
    let _ = registry.infer_operation_with_extent_sources(
        &multiply_f32_op(),
        &[operand.clone(), operand],
        &OperationAttributes::empty(),
        None,
    );
}
