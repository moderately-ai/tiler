use tiler_ir::semantic::OperationInferenceError;
use tiler_ir::shape::{ExtentSourceError, ShapeSymbol, SymbolScope};

fn main() {
    let symbol = ShapeSymbol::new(SymbolScope::new("test").unwrap(), "n").unwrap();
    let _ = OperationInferenceError::from_extent_source(ExtentSourceError::UndeclaredSymbol {
        symbol,
    });
}
