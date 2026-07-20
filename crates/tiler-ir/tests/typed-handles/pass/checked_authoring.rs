use tiler_ir::semantic::{
    BuildError, F32, F32Add, InputKey, OpKey, OperationAttributes, OutputKey, ReifyError,
    SemanticProgramBuilder, Value, ValueTypeMarker,
};
use tiler_ir::shape::Shape;

enum External {}
impl ValueTypeMarker for External {}

fn explicit_conversion(
    builder: &mut SemanticProgramBuilder,
    input: Value<F32>,
) -> Result<Value<External>, BuildError> {
    let result = builder.apply(
        OpKey::new("fixture", "f32-to-external", 1).unwrap(),
        OperationAttributes::empty(),
        &[input.erase()],
    )?;
    builder.reify(result[0]).map_err(BuildError::Reify)
}

fn external_input(
    builder: &mut SemanticProgramBuilder,
) -> Result<Value<External>, BuildError> {
    builder.input(InputKey::new("external")?, Shape::from_dims([2]))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = SemanticProgramBuilder::try_standard()?;
    let left: Value<F32> = builder.input(InputKey::new("left")?, Shape::from_dims([2]))?;
    let right: Value<F32> = builder.input(InputKey::new("right")?, Shape::from_dims([2]))?;
    let sum = F32Add::apply(&mut builder, left, right)?;
    let erased = sum.erase();
    let sum: Value<F32> = builder.reify(erased)?;
    let output = builder.output(OutputKey::new("sum")?, sum)?;
    let program = builder.build()?;
    let resolved = program.resolve_typed_output(&output)?;
    let _: Value<F32> = resolved.value();

    let _external_authoring: fn(&mut SemanticProgramBuilder) -> Result<Value<External>, BuildError> =
        external_input;
    let _explicit_conversion: fn(
        &mut SemanticProgramBuilder,
        Value<F32>,
    ) -> Result<Value<External>, BuildError> = explicit_conversion;
    let _reify_error: Option<ReifyError> = None;
    Ok(())
}
