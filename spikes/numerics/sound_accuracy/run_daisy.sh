#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 /path/to/daisy-checkout" >&2
  exit 2
fi

daisy_root=$1
spike_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
expected_commit=38a0f33915dde03eeadd34786a920e834c1d9110

actual_commit=$(git -C "$daisy_root" rev-parse HEAD)
if [ "$actual_commit" != "$expected_commit" ]; then
  echo "unsupported Daisy revision: $actual_commit" >&2
  echo "expected: $expected_commit" >&2
  exit 3
fi
tracked_changes=$(git -C "$daisy_root" status --short)
if [ -n "$tracked_changes" ]; then
  echo "unsupported dirty Daisy checkout at $actual_commit" >&2
  echo "$tracked_changes" >&2
  exit 4
fi
if [ ! -x "$daisy_root/daisy" ]; then
  echo "missing generated Daisy executable: $daisy_root/daisy" >&2
  exit 5
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "missing required python3 executable" >&2
  exit 6
fi

timeout_seconds=${TILER_DAISY_TIMEOUT_SECONDS:-60}
exec python3 "$spike_dir/daisy_runner.py" \
  "$daisy_root" "$spike_dir" "$timeout_seconds" "$actual_commit"
