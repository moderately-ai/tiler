//! Generated const-generic dimension-tuple workload.
#![allow(clippy::too_many_lines)]

use shape_evidence_spike::{Dim, Exact, exercise_evidence};

fn main() {
    let mut matched = 0_usize;
    matched += usize::from(exercise_evidence::<Exact<(Dim<0>, Dim<2>, Dim<3>)>>(&[
        0, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<1>, Dim<2>, Dim<3>)>>(&[
        1, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<2>, Dim<2>, Dim<3>)>>(&[
        2, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<3>, Dim<2>, Dim<3>)>>(&[
        3, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<4>, Dim<2>, Dim<3>)>>(&[
        4, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<5>, Dim<2>, Dim<3>)>>(&[
        5, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<6>, Dim<2>, Dim<3>)>>(&[
        6, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<7>, Dim<2>, Dim<3>)>>(&[
        7, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<8>, Dim<2>, Dim<3>)>>(&[
        8, 2, 3,
    ]));
    matched += usize::from(exercise_evidence::<Exact<(Dim<9>, Dim<2>, Dim<3>)>>(&[
        9, 2, 3,
    ]));
    assert_eq!(matched, 10);
}
