use nightly_shape_api::StaticShape;

type Scalar = StaticShape<0, { [] }>;
type Vector = StaticShape<1, { [8] }>;
type Matrix = StaticShape<2, { [2, 3] }>;
type Rank64 = StaticShape<64, { [1; 64] }>;

fn main() {
    assert_eq!(std::mem::size_of::<Scalar>(), 0);
    assert_eq!(std::mem::size_of::<Vector>(), 0);
    assert_eq!(std::mem::size_of::<Matrix>(), 0);
    assert_eq!(std::mem::size_of::<Rank64>(), 0);
}
