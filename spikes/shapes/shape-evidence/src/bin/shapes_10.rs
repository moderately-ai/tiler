//! Generated distinct-static-shape compile-time workload.
#![allow(clippy::too_many_lines)]

use shape_evidence_spike::{Exact, StaticShapeSpec, exercise_evidence};

struct Shape0;
impl StaticShapeSpec for Shape0 {
    const EXTENTS: &'static [u64] = &[0, 2, 3];
}
struct Shape1;
impl StaticShapeSpec for Shape1 {
    const EXTENTS: &'static [u64] = &[1, 2, 3];
}
struct Shape2;
impl StaticShapeSpec for Shape2 {
    const EXTENTS: &'static [u64] = &[2, 2, 3];
}
struct Shape3;
impl StaticShapeSpec for Shape3 {
    const EXTENTS: &'static [u64] = &[3, 2, 3];
}
struct Shape4;
impl StaticShapeSpec for Shape4 {
    const EXTENTS: &'static [u64] = &[4, 2, 3];
}
struct Shape5;
impl StaticShapeSpec for Shape5 {
    const EXTENTS: &'static [u64] = &[5, 2, 3];
}
struct Shape6;
impl StaticShapeSpec for Shape6 {
    const EXTENTS: &'static [u64] = &[6, 2, 3];
}
struct Shape7;
impl StaticShapeSpec for Shape7 {
    const EXTENTS: &'static [u64] = &[7, 2, 3];
}
struct Shape8;
impl StaticShapeSpec for Shape8 {
    const EXTENTS: &'static [u64] = &[8, 2, 3];
}
struct Shape9;
impl StaticShapeSpec for Shape9 {
    const EXTENTS: &'static [u64] = &[9, 2, 3];
}

fn main() {
    let mut matched = 0_usize;
    matched += usize::from(exercise_evidence::<Exact<Shape0>>(&[0, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape1>>(&[1, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape2>>(&[2, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape3>>(&[3, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape4>>(&[4, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape5>>(&[5, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape6>>(&[6, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape7>>(&[7, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape8>>(&[8, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Shape9>>(&[9, 2, 3]));
    assert_eq!(matched, 10);
}
