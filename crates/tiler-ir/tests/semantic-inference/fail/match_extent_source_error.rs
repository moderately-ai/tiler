use tiler_ir::shape::ExtentSourceError;

fn classify(error: ExtentSourceError) -> u8 {
    match error {
        ExtentSourceError::UndeclaredSymbol { .. } => 1,
        ExtentSourceError::SourceTooLate { .. } => 2,
        ExtentSourceError::DivisorNotProvedPositive { .. } => 3,
        ExtentSourceError::ExtentsNotProvedEqual(_) => 4,
    }
}

fn main() {
    let _ = classify;
}
