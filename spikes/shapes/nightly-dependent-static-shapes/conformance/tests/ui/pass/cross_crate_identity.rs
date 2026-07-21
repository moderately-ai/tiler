use std::marker::PhantomData;

use nightly_shape_aliases_a::{ConstMatrix, LiteralMatrix, ReexportedMatrix};
use nightly_shape_aliases_b::{LiteralMatrix as OtherLiteralMatrix, MacroMatrix};

fn require_same<T>(_: PhantomData<T>, _: PhantomData<T>) {}

fn main() {
    require_same(PhantomData::<LiteralMatrix>, PhantomData::<ConstMatrix>);
    require_same(PhantomData::<LiteralMatrix>, PhantomData::<ReexportedMatrix>);
    require_same(PhantomData::<LiteralMatrix>, PhantomData::<OtherLiteralMatrix>);
    require_same(PhantomData::<LiteralMatrix>, PhantomData::<MacroMatrix>);
}
