#!/usr/bin/env bash
set -euo pipefail

experiment_dir="$(cd "$(dirname "$0")" && pwd)"
raw_dir="${experiment_dir}/measurements/raw/spellings"
mkdir -p "${raw_dir}"

"${experiment_dir}/generate-workloads.sh"
rustc +1.89.0 --version --verbose > "${raw_dir}/rustc.txt"
cargo +1.89.0 --version --verbose > "${raw_dir}/cargo.txt"
cargo +1.89.0 fetch --manifest-path "${experiment_dir}/Cargo.toml"

for spelling in shapes family tuple; do
    for count in 1 10 100 1000; do
        bin="${spelling}_${count}"
        wc -c < "${experiment_dir}/src/bin/${bin}.rs" \
            > "${raw_dir}/${bin}.source_bytes"
        for sample in 1 2 3 4 5; do
            check_target="${raw_dir}/target-check"
            release_target="${raw_dir}/target-release"
            cargo +1.89.0 clean --manifest-path "${experiment_dir}/Cargo.toml" \
                --target-dir "${check_target}"
            cargo +1.89.0 clean --manifest-path "${experiment_dir}/Cargo.toml" \
                --target-dir "${release_target}"
            /usr/bin/time -lp env CARGO_TARGET_DIR="${check_target}" cargo +1.89.0 check \
                --manifest-path "${experiment_dir}/Cargo.toml" \
                --bin "${bin}" \
                > "${raw_dir}/${bin}.${sample}.check.stdout" \
                2> "${raw_dir}/${bin}.${sample}.check.time"
            /usr/bin/time -lp env CARGO_TARGET_DIR="${release_target}" cargo +1.89.0 build --release \
                --manifest-path "${experiment_dir}/Cargo.toml" \
                --bin "${bin}" \
                > "${raw_dir}/${bin}.${sample}.release.stdout" \
                2> "${raw_dir}/${bin}.${sample}.release.time"
            stat -f '%z' "${release_target}/release/${bin}" \
                > "${raw_dir}/${bin}.${sample}.binary_bytes"
        done
    done
done
