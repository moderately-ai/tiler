#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 /path/to/daisy-checkout" >&2
  exit 2
fi

daisy_root=$1
spike_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
expected_commit=38a0f33915dde03eeadd34786a920e834c1d9110

actual_commit=$(git -C "$daisy_root" rev-parse HEAD)
if [ "$actual_commit" != "$expected_commit" ]; then
  echo "unsupported Daisy revision: $actual_commit" >&2
  echo "expected: $expected_commit" >&2
  exit 3
fi
if [ ! -x "$daisy_root/daisy" ]; then
  echo "missing generated Daisy executable: $daisy_root/daisy" >&2
  exit 4
fi

(
  cd "$daisy_root"
  ./daisy \
    --precision=Float32 \
    --analysis=dataflow \
    --rangeMethod=interval \
    --errorMethod=affine \
    "$spike_dir/scalar_regions.scala"

  ./daisy \
    --functions=materialized_f16 \
    --precision=Float32 \
    --mixed-precision="$spike_dir/mixed-precision.txt" \
    --analysis=dataflow \
    --rangeMethod=interval \
    --errorMethod=affine \
    "$spike_dir/scalar_regions.scala"
)
