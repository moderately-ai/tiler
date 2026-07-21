//! Generated direct const-generic shape-family workload.
#![allow(clippy::too_many_lines)]

use shape_evidence_spike::{Dims3, Exact, exercise_evidence};

fn main() {
    let mut matched = 0_usize;
    matched += usize::from(exercise_evidence::<Exact<Dims3<0, 2, 3>>>(&[0, 2, 3]));
    assert_eq!(matched, 1);
}
