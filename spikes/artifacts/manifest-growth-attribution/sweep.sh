#!/bin/sh
# Probe every gated landing in an interval and print the fixed-content ladder.
#
#   ./sweep.sh <base> <head> [scratch-root] > results/<name>.tsv
#
# The population is the **first-parent** commits of `<base>..<head>` that touch
# `crates/`. First-parent because every one of those was gated green before it
# was published, while a commit inside a merged branch need not build at all;
# `crates/` because a landing that touches no crate cannot move an encoding.
# The count is printed as a comment so a run that reached nothing cannot read
# as a run that found nothing.
set -eu

if [ $# -lt 2 ]; then
  echo "usage: sweep.sh <base> <tip> [scratch-root]" >&2
  exit 2
fi

base=$1
tip=$2
scratch=${3:-${TMPDIR:-/tmp}/tiler-manifest-growth}
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)

commits=$(cd "$repo" && git log --first-parent --format=%h --reverse "$base..$tip" -- crates/)
count=$(printf '%s\n' "$commits" | grep -c .)

printf '# manifest-growth-attribution %s..%s\n' "$base" "$tip"
printf '# population=%s first-parent landings touching crates/\n' "$count"
printf '# probed=%s (the population plus both interval endpoints)\n' "$((count + 2))"

for commit in $base $commits $tip; do
  "$here/probe.sh" "$commit" "$scratch"
done
