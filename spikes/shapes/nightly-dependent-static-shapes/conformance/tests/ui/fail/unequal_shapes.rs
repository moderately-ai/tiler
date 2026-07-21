use std::marker::PhantomData;

use nightly_shape_api::StaticShape;

type Left = StaticShape<2, { [2, 3] }>;
type Right = StaticShape<2, { [3, 2] }>;

fn require_same<T>(_: PhantomData<T>, _: PhantomData<T>) {}

fn main() {
    require_same(PhantomData::<Left>, PhantomData::<Right>);
}
