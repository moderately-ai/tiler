//! Generated const-generic dimension-tuple workload.
#![allow(clippy::too_many_lines)]

use shape_evidence_spike::{Dim, Exact, exercise_evidence};

fn main() {
    let mut matched = 0_usize;
    matched += usize::from(exercise_evidence::<Exact<(Dim<0>, Dim<2>, Dim<3>)>>(&[
        0, 2, 3,
    ]));
    assert_eq!(matched, 1);
}
