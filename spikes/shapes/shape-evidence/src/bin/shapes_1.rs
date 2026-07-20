//! Generated distinct-static-shape compile-time workload.
#![allow(clippy::too_many_lines)]

use shape_evidence_spike::{Exact, StaticShapeSpec, exercise_evidence};

struct Shape0;
impl StaticShapeSpec for Shape0 {
    const EXTENTS: &'static [u64] = &[0, 2, 3];
}

fn main() {
    let mut matched = 0_usize;
    matched += usize::from(exercise_evidence::<Exact<Shape0>>(&[0, 2, 3]));
    assert_eq!(matched, 1);
}
