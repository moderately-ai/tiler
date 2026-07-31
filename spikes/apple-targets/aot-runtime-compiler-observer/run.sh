#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: run.sh <result-directory>" >&2
    exit 2
fi

result_dir=$1
case "$result_dir" in
    /*) ;;
    *) result_dir=$(pwd)/$result_dir ;;
esac

spike_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/tiler-aot-observer.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

mkdir -p "$result_dir"

xcrun --sdk macosx metal -std=metal4.0 -target air64-apple-macos26.0 \
    -c "$spike_dir/kernel.metal" -o "$scratch/kernel.air"
xcrun --sdk macosx metallib "$scratch/kernel.air" -o "$scratch/kernel.metallib"
xcrun --sdk macosx clang -fobjc-arc -Wall -Wextra -Werror \
    -framework Foundation -framework Metal "$spike_dir/probe.m" -o "$scratch/probe"
xcrun --sdk macosx clang -fobjc-arc -Wall -Wextra -Werror \
    -framework Foundation -framework Metal "$spike_dir/source_jit_sentinel.m" \
    -o "$scratch/source-jit-sentinel"
xcrun --sdk macosx clang -dynamiclib -Wall -Wextra -Werror \
    "$spike_dir/unrelated_gpu_compiler.c" -o "$scratch/UnrelatedGPUCompiler.dylib"

{
    printf 'environment.os_version='
    sw_vers -productVersion
    printf 'environment.os_build='
    sw_vers -buildVersion
    printf 'environment.architecture='
    uname -m
    printf 'environment.xcode='
    xcodebuild -version | tr '\n' ';'
    printf '\n'
    printf 'environment.macos_sdk_version='
    xcrun --sdk macosx --show-sdk-version
    printf 'environment.macos_sdk_build='
    xcrun --sdk macosx --show-sdk-build-version
    printf 'environment.offline_metal='
    xcrun --sdk macosx metal --version | tr '\n' ';'
    printf '\n'
    printf 'input.kernel_sha256='
    shasum -a 256 "$spike_dir/kernel.metal" | awk '{ print $1 }'
    printf 'input.probe_sha256='
    shasum -a 256 "$spike_dir/probe.m" | awk '{ print $1 }'
    printf 'input.run_sha256='
    shasum -a 256 "$spike_dir/run.sh" | awk '{ print $1 }'
    printf 'input.validate_sha256='
    shasum -a 256 "$spike_dir/validate.sh" | awk '{ print $1 }'
    printf 'input.classify_sha256='
    shasum -a 256 "$spike_dir/classify.sh" | awk '{ print $1 }'
    printf 'input.source_jit_sentinel_sha256='
    shasum -a 256 "$spike_dir/source_jit_sentinel.m" | awk '{ print $1 }'
    printf 'input.unrelated_gpu_compiler_sha256='
    shasum -a 256 "$spike_dir/unrelated_gpu_compiler.c" | awk '{ print $1 }'
    printf 'input.metallib_sha256='
    shasum -a 256 "$scratch/kernel.metallib" | awk '{ print $1 }'
    if [ -d /System/Library/PrivateFrameworks/MTLCompiler.framework ]; then
        printf 'environment.mtlcompiler_framework_on_disk=present\n'
    else
        printf 'environment.mtlcompiler_framework_on_disk=absent\n'
    fi
    printf 'probe.run=clean-1\n'
    "$scratch/probe" "$scratch/kernel.metallib"
} > "$result_dir/clean-1.tsv"

"$scratch/probe" "$scratch/kernel.metallib" > "$result_dir/clean-2.tsv"
"$scratch/probe" "$scratch/kernel.metallib" > "$result_dir/clean-3.tsv"

"$spike_dir/validate.sh" "$scratch/probe" "$result_dir/clean-1.tsv"
"$spike_dir/validate.sh" "$scratch/probe" "$result_dir/clean-2.tsv"
"$spike_dir/validate.sh" "$scratch/probe" "$result_dir/clean-3.tsv"

if ! cmp -s "$result_dir/clean-2.tsv" "$result_dir/clean-3.tsv"; then
    echo "clean observations are not stable across repeated processes" >&2
    exit 1
fi

cp "$result_dir/clean-3.tsv" "$scratch/unstable.tsv"
printf 'perturbation.byte_stability=changed\n' >> "$scratch/unstable.tsv"
if cmp -s "$result_dir/clean-2.tsv" "$scratch/unstable.tsv"; then
    echo "byte-stability perturbation unexpectedly compared equal" >&2
    exit 1
fi

"$scratch/probe" "$scratch/kernel.metallib" --preload \
    "$scratch/UnrelatedGPUCompiler.dylib" > "$result_dir/preloaded.tsv"
"$spike_dir/validate.sh" "$scratch/probe" "$result_dir/preloaded.tsv"

"$spike_dir/classify.sh" "$result_dir/clean-1.tsv" > "$result_dir/summary.tsv"
printf 'observation.clean_repetitions=byte-identical\n' >> "$result_dir/summary.tsv"

assert_expected_measurement() {
    checked_result=$1
    checked_start=$(sed -n 's/^stage.process_start.compiler_image_count=//p' "$checked_result")
    checked_pipeline=$(sed -n 's/^stage.after_pipeline.compiler_image_count=//p' "$checked_result")
    checked_builds=$(sed -n 's/^stage.after_pipeline.image.[0-9][0-9]*.build_count=//p' \
        "$checked_result" | awk '{ total += $1 } END { print total + 0 }')
    [ "$checked_pipeline" -eq "$checked_start" ] && [ "$checked_builds" -eq 0 ]
}

if ! assert_expected_measurement "$result_dir/clean-1.tsv"; then
    echo "observations changed; review and update the bounded conclusion" >&2
    exit 1
fi

cp "$result_dir/clean-1.tsv" "$scratch/observation-drift.tsv"
sed -i '' 's/^stage.after_pipeline.compiler_image_count=.*/stage.after_pipeline.compiler_image_count=99/' \
    "$scratch/observation-drift.tsv"
if assert_expected_measurement "$scratch/observation-drift.tsv"; then
    echo "observation-drift perturbation unexpectedly passed" >&2
    exit 1
fi

cp "$result_dir/clean-1.tsv" "$scratch/mutated-result.tsv"
sed -i '' 's/probe.status=ok/probe.status=mutated/' "$scratch/mutated-result.tsv"
if "$spike_dir/validate.sh" "$scratch/probe" "$scratch/mutated-result.tsv" \
    >/dev/null 2>&1
then
    echo "result mutation unexpectedly passed validation" >&2
    exit 1
fi

cp "$result_dir/clean-1.tsv" "$scratch/absent-metadata.tsv"
sed -i '' '/^stage.after_pipeline.compiler_image_count=/d' "$scratch/absent-metadata.tsv"
if "$spike_dir/validate.sh" "$scratch/probe" "$scratch/absent-metadata.tsv" \
    >/dev/null 2>&1
then
    echo "absent-metadata fixture unexpectedly passed validation" >&2
    exit 1
fi
if "$spike_dir/classify.sh" "$scratch/absent-metadata.tsv" >/dev/null 2>&1
then
    echo "absent-metadata fixture unexpectedly passed classification" >&2
    exit 1
fi

cp "$result_dir/clean-1.tsv" "$scratch/absent-scan-status.tsv"
sed -i '' '/^stage.after_pipeline.image.[0-9][0-9]*.scan_status=/d' \
    "$scratch/absent-scan-status.tsv"
if "$spike_dir/validate.sh" "$scratch/probe" "$scratch/absent-scan-status.tsv" \
    >/dev/null 2>&1
then
    echo "absent-scan-status fixture unexpectedly passed validation" >&2
    exit 1
fi

cp "$result_dir/clean-1.tsv" "$scratch/multiple-plausible-builds.tsv"
sed -i '' 's/^stage.after_pipeline.image.0.scan_status=.*/stage.after_pipeline.image.0.scan_status=readable/' \
    "$scratch/multiple-plausible-builds.tsv"
sed -i '' 's/^stage.after_pipeline.image.1.scan_status=.*/stage.after_pipeline.image.1.scan_status=readable/' \
    "$scratch/multiple-plausible-builds.tsv"
sed -i '' 's/^stage.after_pipeline.image.0.build_count=.*/stage.after_pipeline.image.0.build_count=1/' \
    "$scratch/multiple-plausible-builds.tsv"
sed -i '' 's/^stage.after_pipeline.image.1.build_count=.*/stage.after_pipeline.image.1.build_count=1/' \
    "$scratch/multiple-plausible-builds.tsv"
printf 'stage.after_pipeline.image.0.build.0=metalfe-11111.1\n' \
    >> "$scratch/multiple-plausible-builds.tsv"
printf 'stage.after_pipeline.image.1.build.0=metalfe-22222.2\n' \
    >> "$scratch/multiple-plausible-builds.tsv"
"$spike_dir/classify.sh" "$scratch/multiple-plausible-builds.tsv" \
    > "$scratch/multiple-plausible-summary.tsv"
if ! grep -q '^conclusion.exact_runtime_compiler_attribution=unavailable$' \
    "$scratch/multiple-plausible-summary.tsv"
then
    echo "multiple plausible builds unexpectedly produced exact attribution" >&2
    exit 1
fi

cp "$result_dir/clean-1.tsv" "$scratch/unavailable-host.tsv"
sed -i '' 's/probe.status=ok/probe.status=unavailable:no-default-device/' \
    "$scratch/unavailable-host.tsv"
if "$spike_dir/validate.sh" "$scratch/probe" "$scratch/unavailable-host.tsv" \
    >/dev/null 2>&1
then
    echo "unavailable-host fixture unexpectedly passed validation" >&2
    exit 1
fi

if "$spike_dir/validate.sh" "$scratch/source-jit-sentinel" "$result_dir/clean-1.tsv" \
    >/dev/null 2>&1
then
    echo "source-JIT sentinel mutation unexpectedly passed validation" >&2
    exit 1
fi

{
    printf 'validator.result_mutation=rejected\n'
    printf 'validator.source_jit_sentinel_mutation=rejected\n'
    printf 'validator.byte_stability_perturbation=rejected\n'
    printf 'validator.observation_drift_perturbation=rejected\n'
    printf 'validator.absent_metadata_fixture=rejected\n'
    printf 'classifier.absent_metadata_fixture=rejected\n'
    printf 'validator.absent_scan_status_fixture=rejected\n'
    printf 'classifier.multiple_plausible_builds=unavailable\n'
    printf 'validator.unavailable_host_fixture=rejected\n'
    printf 'probe.status=validated\n'
} > "$result_dir/validation.tsv"

echo "retained AOT observer results in $result_dir"
