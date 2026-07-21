//! Generated direct const-generic shape-family workload.
#![allow(clippy::too_many_lines)]

use shape_evidence_spike::{Dims3, Exact, exercise_evidence};

fn main() {
    let mut matched = 0_usize;
    matched += usize::from(exercise_evidence::<Exact<Dims3<0, 2, 3>>>(&[0, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<1, 2, 3>>>(&[1, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<2, 2, 3>>>(&[2, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<3, 2, 3>>>(&[3, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<4, 2, 3>>>(&[4, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<5, 2, 3>>>(&[5, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<6, 2, 3>>>(&[6, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<7, 2, 3>>>(&[7, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<8, 2, 3>>>(&[8, 2, 3]));
    matched += usize::from(exercise_evidence::<Exact<Dims3<9, 2, 3>>>(&[9, 2, 3]));
    assert_eq!(matched, 10);
}
