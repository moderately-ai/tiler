#!/usr/bin/env bash
set -euo pipefail

experiment_dir="$(cd "$(dirname "$0")" && pwd)"
raw_dir="${experiment_dir}/measurements/raw"
mkdir -p "${raw_dir}"

"${experiment_dir}/generate-workloads.sh"

rustc +1.89.0 --version --verbose > "${raw_dir}/rustc.txt"
cargo +1.89.0 --version --verbose > "${raw_dir}/cargo.txt"
uname -a > "${raw_dir}/uname.txt"
system_profiler SPHardwareDataType > "${raw_dir}/hardware.txt"
cargo +1.89.0 fetch --manifest-path "${experiment_dir}/Cargo.toml"

for count in 1 10 100 1000; do
    cargo +1.89.0 clean --manifest-path "${experiment_dir}/Cargo.toml" \
        -p shape-evidence-spike
    /usr/bin/time -lp cargo +1.89.0 check \
        --manifest-path "${experiment_dir}/Cargo.toml" \
        --bin "shapes_${count}" \
        > "${raw_dir}/check_${count}_cold.stdout" \
        2> "${raw_dir}/check_${count}_cold.time"
    touch "${experiment_dir}/src/bin/shapes_${count}.rs"
    /usr/bin/time -lp cargo +1.89.0 check \
        --manifest-path "${experiment_dir}/Cargo.toml" \
        --bin "shapes_${count}" \
        > "${raw_dir}/check_${count}_incremental.stdout" \
        2> "${raw_dir}/check_${count}_incremental.time"
    /usr/bin/time -lp cargo +1.89.0 build --release \
        --manifest-path "${experiment_dir}/Cargo.toml" \
        --bin "shapes_${count}" \
        > "${raw_dir}/release_${count}.stdout" \
        2> "${raw_dir}/release_${count}.time"
    stat -f '%z' "${experiment_dir}/target/release/shapes_${count}" \
        > "${raw_dir}/release_${count}.bytes"
done
