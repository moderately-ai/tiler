#!/bin/sh
# Rebuild the hot-path fixture at one commit and print its fixed content.
#
# The working tree is never checked out to a historical commit: the commit's
# whole tree is extracted with `git archive` into a scratch directory, which is
# what lets this run beside other agents' branches without touching any of them.
#
#   ./probe.sh <commit> [scratch-root]
#
# Prints one tab-separated row on stdout and nothing else on success. Exits
# non-zero, with the build or run log on stderr, when a commit does not build:
# a silent skip would leave a gap in a sweep that reads as "no change".
set -eu

if [ $# -lt 1 ]; then
  echo "usage: probe.sh <commit> [scratch-root]" >&2
  exit 2
fi

commit=$1
scratch=${2:-${TMPDIR:-/tmp}/tiler-manifest-growth}
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)

# One shared target directory across the sweep. Path dependencies rebuild at
# every commit because their source path moves, so what this reuses is the
# registry dependencies; it is not a correctness shortcut, and each build is
# still a full build of the four Tiler crates at that commit.
target=$scratch/target
tree=$scratch/tree-$commit

# Resolved before anything is extracted. `git archive` sits in the left half of
# a pipeline, where `set -e` does not see its status, so an unresolvable commit
# would otherwise be reported four steps later as a missing Cargo manifest.
(cd "$repo" && git rev-parse --verify --quiet "$commit^{commit}" >/dev/null) || {
  echo "probe.sh: $commit does not resolve to a commit in $repo" >&2
  exit 1
}

rm -rf "$tree"
mkdir -p "$tree" "$target"
(cd "$repo" && git archive "$commit") | tar -x -C "$tree"

spike=$tree/spikes/cache/hot-path-efficiency
if [ ! -f "$spike/Cargo.toml" ]; then
  # The fixture merged into `main` part-way through the interval this sweep
  # covers. Commits before that merge get it from `194744e6`, the commit the
  # retained 2026-08-04 hot-path results were taken at, so the fixture is the
  # same source at every probed commit rather than absent at some of them.
  mkdir -p "$spike"
  (cd "$repo" && git archive 194744e6 spikes/cache/hot-path-efficiency) | tar -x -C "$tree"
fi

mkdir -p "$spike/harness/src/bin"
cp "$here/probe.rs" "$spike/harness/src/bin/probe.rs"

cd "$spike"
CARGO_TARGET_DIR=$target cargo build --quiet --bin probe >&2
row=$("$target/debug/probe")
printf '%s\t%s\n' "$commit" "$row"
rm -rf "$tree"
