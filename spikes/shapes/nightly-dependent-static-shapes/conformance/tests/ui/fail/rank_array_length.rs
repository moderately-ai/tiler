use nightly_shape_api::StaticShape;

type Invalid = StaticShape<2, { [2, 3, 4] }>;

fn main() {
    let _ = std::mem::size_of::<Invalid>();
}
