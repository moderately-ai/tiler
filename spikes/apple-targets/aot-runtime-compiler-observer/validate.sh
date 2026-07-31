#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: validate.sh <probe-source> <result.tsv>" >&2
    exit 2
fi

probe_source=$1
result=$2

if grep -q 'newLibraryWithSource' "$probe_source"; then
    echo "runtime source-compilation selector is present" >&2
    exit 1
fi

for required in \
    'probe.route=native-metallib-library-and-compute-pipeline' \
    'probe.observation_api=dyld-loaded-image-membership-and-image-byte-scan' \
    'stage.process_start.compiler_image_count=' \
    'stage.after_device.compiler_image_count=' \
    'stage.after_library.compiler_image_count=' \
    'stage.after_function.compiler_image_count=' \
    'stage.after_pipeline.compiler_image_count=' \
    'probe.status=ok'
do
    if ! grep -q "^$required" "$result"; then
        echo "missing required result row: $required" >&2
        exit 1
    fi
done

echo "validated $result"
