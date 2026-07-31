#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: validate.sh <probe-binary> <result.tsv>" >&2
    exit 2
fi

probe_binary=$1
result=$2

if strings -a "$probe_binary" | grep -Fq 'newLibraryWithSource:options:error:'; then
    echo "runtime source-compilation selector is present in probe binary" >&2
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

for stage in process_start after_device after_library after_function after_pipeline
do
    image_count=$(sed -n "s/^stage.$stage.compiler_image_count=//p" "$result")
    scan_count=$(grep -c "^stage.$stage.image.[0-9][0-9]*.scan_status=" "$result" || true)
    if [ "$scan_count" -ne "$image_count" ]; then
        echo "scan-status population mismatch at $stage" >&2
        exit 1
    fi
done

echo "validated $result"
