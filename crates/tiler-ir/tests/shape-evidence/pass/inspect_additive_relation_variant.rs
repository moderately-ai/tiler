use tiler_ir::shape::{ExtentRelation, ExtentTerm};

fn main() {
    let relation = ExtentRelation::additive_equality(
        ExtentTerm::Constant(3),
        ExtentTerm::Constant(2),
        ExtentTerm::Constant(1),
    );
    match relation {
        ExtentRelation::AdditiveEquality {
            sum,
            left,
            right,
            ..
        } => {
            assert_eq!(sum, ExtentTerm::Constant(3));
            assert_eq!(left, ExtentTerm::Constant(1));
            assert_eq!(right, ExtentTerm::Constant(2));
        }
        _ => panic!("the helper returns the additive relation variant"),
    }
}
