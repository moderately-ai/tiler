#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: classify.sh <result.tsv>" >&2
    exit 2
fi

result=$1
start_count=$(sed -n 's/^stage.process_start.compiler_image_count=//p' "$result")
pipeline_count=$(sed -n 's/^stage.after_pipeline.compiler_image_count=//p' "$result")
embedded_build_count=$(sed -n 's/^stage.after_pipeline.image.[0-9][0-9]*.build_count=//p' \
    "$result" | awk '{ total += $1 } END { print total + 0 }')
explicit_build_count=$(grep -c '^stage.after_pipeline.image.[0-9][0-9]*.build.[0-9][0-9]*=' \
    "$result" || true)
mtlcompiler_loaded=$(grep '^stage.process_start.image.[0-9][0-9]*.path=' "$result" \
    | grep -c 'MTLCompiler' || true)

if [ -z "$start_count" ] || [ -z "$pipeline_count" ]; then
    echo "required image-count observation is absent" >&2
    exit 1
fi

printf 'observation.compiler_images_at_process_start=%s\n' "$start_count"
printf 'observation.compiler_images_after_pipeline=%s\n' "$pipeline_count"
printf 'observation.new_compiler_images_during_route=%s\n' \
    "$((pipeline_count - start_count))"
printf 'observation.declared_embedded_compiler_builds_after_pipeline=%s\n' \
    "$embedded_build_count"
printf 'observation.explicit_embedded_compiler_build_rows_after_pipeline=%s\n' \
    "$explicit_build_count"
printf 'observation.mtlcompiler_images_at_process_start=%s\n' "$mtlcompiler_loaded"
printf 'conclusion.exact_runtime_compiler_attribution=unavailable\n'
printf 'conclusion.evidence_class=loaded-image-membership-and-image-byte-scan\n'
printf 'conclusion.reason=dyld membership, deltas, and image strings do not associate a compiler build with native library or pipeline preparation\n'
