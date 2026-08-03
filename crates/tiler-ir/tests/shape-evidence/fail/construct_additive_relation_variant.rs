use tiler_ir::shape::{ExtentRelation, ExtentTerm};

fn main() {
    let _ = ExtentRelation::AdditiveEquality {
        sum: ExtentTerm::Constant(3),
        left: ExtentTerm::Constant(1),
        right: ExtentTerm::Constant(2),
    };
}
