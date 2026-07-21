//! Runtime checks for identity, ownership, and refinement authority.

use std::marker::PhantomData;
use std::mem::size_of;

use nightly_shape_aliases_a::{ConstMatrix, LiteralMatrix, ReexportedMatrix};
use nightly_shape_aliases_b::{LiteralMatrix as OtherLiteralMatrix, MacroMatrix};
use nightly_shape_api::{Builder, F32, ShapeError, StaticShape};

fn require_same<T>(_: PhantomData<T>, _: PhantomData<T>) {}

#[test]
fn equal_structural_values_unify_across_crates_and_generation_paths() {
    require_same(PhantomData::<LiteralMatrix>, PhantomData::<ConstMatrix>);
    require_same(
        PhantomData::<LiteralMatrix>,
        PhantomData::<ReexportedMatrix>,
    );
    require_same(
        PhantomData::<LiteralMatrix>,
        PhantomData::<OtherLiteralMatrix>,
    );
    require_same(PhantomData::<LiteralMatrix>, PhantomData::<MacroMatrix>);
}

#[test]
fn one_zero_sized_family_covers_representative_and_high_ranks() {
    type Rank0 = StaticShape<0, { [] }>;
    type Rank1 = StaticShape<1, { [1] }>;
    type Rank2 = StaticShape<2, { [2, 3] }>;
    type Rank4 = StaticShape<4, { [1, 2, 3, 4] }>;
    type Rank8 = StaticShape<8, { [1, 2, 3, 4, 5, 6, 7, 8] }>;
    type Rank64 = StaticShape<64, { [1; 64] }>;

    assert_eq!(size_of::<Rank0>(), 0);
    assert_eq!(size_of::<Rank1>(), 0);
    assert_eq!(size_of::<Rank2>(), 0);
    assert_eq!(size_of::<Rank4>(), 0);
    assert_eq!(size_of::<Rank8>(), 0);
    assert_eq!(size_of::<Rank64>(), 0);
}

#[test]
fn refinement_is_checked_owned_and_identity_neutral() {
    type Matrix = StaticShape<2, { [2, 3] }>;
    type Wrong = StaticShape<2, { [3, 2] }>;

    let mut graph = Builder::new();
    let value = graph.input::<F32>([2, 3]);
    let refined = graph.refine::<_, Matrix>(value).unwrap();
    assert_eq!(value.semantic_identity(), refined.semantic_identity());
    assert_eq!(
        refined.weaken().semantic_identity(),
        value.semantic_identity()
    );
    assert_eq!(
        graph.refine::<_, Wrong>(value).err(),
        Some(ShapeError::EvidenceMismatch)
    );

    let mut foreign = Builder::new();
    let foreign_value = foreign.input::<F32>([2, 3]);
    assert_eq!(
        graph.refine::<_, Matrix>(foreign_value).err(),
        Some(ShapeError::ForeignGraph)
    );
}
