#!/bin/sh
# Emit AIR for probe.metal under each governed and non-governed math-mode flag
# set, and report which AIR intrinsic each MSL spelling selects.
#
# Compile-only. No device is opened, nothing is installed, and no value is
# computed: a row says which intrinsic was emitted, never what it returns.
#
# Run from this directory:
#   ./probe.sh                 # print the record to stdout
#   ./probe.sh > record.tsv    # capture it
set -eu

STD=metal4.0
TARGET=air64-apple-macos26.0

# Column 1 is the flag-set label used in the retained record and in the
# research record that cites it. "governed" is the flag set the workload
# profile records as the qualified Apple9/F32 baseline.
emit() {
  label=$1
  shift
  xcrun metal -x metal -std="$STD" -target "$TARGET" "$@" -S -emit-llvm probe.metal -o - |
    awk -v label="$label" '
      /^define / {
        name = $0
        sub(/^.*@/, "", name)
        sub(/\(.*$/, "", name)
        next
      }
      /tail call .*@air\./ {
        callee = $0
        sub(/^.*@/, "", callee)
        sub(/\(.*$/, "", callee)
        fmf = ($0 ~ /tail call fast /) ? "fast" : "none"
        printf "%s\t%s\t%s\t%s\n", label, name, callee, fmf
      }
      /fdiv / {
        fmf = ($0 ~ /fdiv fast /) ? "fast" : "none"
        printf "%s\t%s\tllvm.fdiv\t%s\n", label, name, fmf
      }
    '
}

printf 'flag_set\tkernel\tair_callee\tcall_fast_math_flags\n'
emit governed          -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off
emit compiler-default
emit fp32-functions-fast -fmetal-math-fp32-functions=fast
emit math-mode-fast      -fmetal-math-mode=fast
emit math-mode-relaxed   -fmetal-math-mode=relaxed
