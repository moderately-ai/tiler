#!/usr/bin/env bash
set -euo pipefail

experiment_dir="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "${experiment_dir}/src/bin"

for count in 1 10 100 1000; do
    output="${experiment_dir}/src/bin/shapes_${count}.rs"
    {
        echo '//! Generated distinct-static-shape compile-time workload.'
        echo '#![allow(clippy::too_many_lines)]'
        echo
        echo 'use shape_evidence_spike::{Exact, StaticShapeSpec, exercise_evidence};'
        echo
        for ((index = 0; index < count; index++)); do
            echo "struct Shape${index};"
            echo "impl StaticShapeSpec for Shape${index} { const EXTENTS: &'static [u64] = &[${index}, 2, 3]; }"
        done
        echo
        echo 'fn main() {'
        echo '    let mut matched = 0_usize;'
        for ((index = 0; index < count; index++)); do
            echo "    matched += usize::from(exercise_evidence::<Exact<Shape${index}>>(&[${index}, 2, 3]));"
        done
        echo "    assert_eq!(matched, ${count});"
        echo '}'
    } > "${output}"
done

cargo +1.89.0 fmt --manifest-path "${experiment_dir}/Cargo.toml" --all
