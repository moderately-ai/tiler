#!/bin/sh
set -eu

if [ "${DEVELOPER_DIR:-}" != "/Applications/Xcode.app/Contents/Developer" ]; then
    echo "set DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer" >&2
    exit 2
fi

here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
out=${1:-"$here/results/local"}
work=$(mktemp -d "${TMPDIR:-/tmp}/tiler-kv-layout.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
mkdir -p "$out"

xcrun -sdk macosx metal -std=metal4.0 -target air64-apple-macos26.0 \
    -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise \
    -ffp-contract=off -c "$here/kernels.metal" -o "$work/kernels.air"
xcrun -sdk macosx metallib "$work/kernels.air" -o "$work/kernels.metallib"
xcrun -sdk macosx clang -fobjc-arc -framework Foundation -framework Metal \
    -framework QuartzCore "$here/host.m" -o "$work/host"

"$work/host" "$work/kernels.metallib" > "$out/measurements.tsv"

# Each candidate's independently wrong address spelling must reach the oracle.
for candidate in exact-head capacity-head sequence-major; do
    if "$work/host" "$work/kernels.metallib" "$candidate" > "$work/negative-$candidate.out" 2>&1; then
        echo "negative $candidate unexpectedly passed" >&2
        exit 1
    fi
    printf '%s\t%s\n' "$candidate" "$(grep 'oracle mismatch' "$work/negative-$candidate.out")" \
        >> "$out/negative-address-oracles.tsv"
done

{
    printf 'field\tvalue\n'
    printf 'sw_vers_product_version\t'; sw_vers -productVersion
    printf 'sw_vers_build_version\t'; sw_vers -buildVersion
    printf 'hardware_model\t'; sysctl -n hw.model
    printf 'hardware_chip\t'; system_profiler SPHardwareDataType | awk -F ': ' '/Chip:/ {print $2; exit}'
    printf 'xcode_version\t'; xcodebuild -version | paste -sd ' ' -
    printf 'sdk_version\t'; xcrun -sdk macosx --show-sdk-version
    printf 'metal_version\t'; xcrun -sdk macosx metal --version 2>&1 | head -1
    printf 'developer_dir\t%s\n' "$DEVELOPER_DIR"
    printf 'compile_flags\t%s\n' '-std=metal4.0 -target air64-apple-macos26.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off'
    printf 'access_schedule\t5 rotated-order rounds; 3 warmups and 7 recorded dispatches per round\n'
    printf 'allocation_schedule\t20 warmups; 101 repetitions\n'
    printf 'lifecycle_schedule\tC1: 10 warmups/51 repetitions; B1: 3 warmups/11 repetitions\n'
} > "$out/environment.tsv"

(cd "$here" && shasum -a 256 kernels.metal host.m run.sh) > "$out/source-sha256.txt"
echo "retained results in $out"
