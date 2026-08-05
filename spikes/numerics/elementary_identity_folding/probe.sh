#!/bin/sh
# Emit AIR for probe.metal under each governed and non-governed math-mode flag
# set, and report the arithmetic each kernel's spelling actually compiled to.
#
# Compile-only. No device is opened, nothing is installed, and no value is
# computed: a row says which operations were emitted, never what they return.
#
# Run from this directory:
#   ./probe.sh                 # print the record to stdout
#   ./probe.sh > record.tsv    # capture it
set -eu

STD=metal4.0
TARGET=air64-apple-macos26.0

# Counted independently of probe.metal on purpose. A population check whose two
# sides both come from the source under test cannot say no: deleting a kernel
# would leave the check agreeing with itself. This literal is the second source.
DECLARED_KERNELS=16

# Column 3 is a canonical signature: every emitted AIR callee and every
# floating-point opcode in the kernel body, sorted, with counts. A fold turns
# one signature into another, so the record is read by comparing a rewritten
# spelling's row against the row of the spelling it would be rewritten into.
emit() {
  label=$1
  shift
  xcrun metal -x metal -std="$STD" -target "$TARGET" "$@" -S -emit-llvm probe.metal -o - |
    awk -v label="$label" '
      /^define / {
        if (name != "") { print_row() }
        name = $0
        sub(/^.*@/, "", name)
        sub(/\(.*$/, "", name)
        delete seen
        next
      }
      /^  %/ {
        line = $0
        if (match(line, /@(air|llvm)\.[a-z0-9_.]+/)) {
          seen[substr(line, RSTART + 1, RLENGTH - 1)]++
        } else if (match(line, / (fmul|fadd|fsub|fdiv|fneg|fpext|fptrunc) /)) {
          seen[substr(line, RSTART + 1, RLENGTH - 2)]++
        }
        next
      }
      END { if (name != "") { print_row() } }
      function print_row(  keys, n, i, j, tmp, sig) {
        n = 0
        for (k in seen) { keys[++n] = k }
        for (i = 1; i < n; i++) {
          for (j = i + 1; j <= n; j++) {
            if (keys[j] < keys[i]) { tmp = keys[i]; keys[i] = keys[j]; keys[j] = tmp }
          }
        }
        sig = ""
        for (i = 1; i <= n; i++) {
          sig = sig (i == 1 ? "" : ";") keys[i] "=" seen[keys[i]]
        }
        printf "%s\t%s\t%s\n", label, name, (sig == "" ? "-" : sig)
      }
    '
}

record=$(
  emit governed             -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off
  emit governed-contracting -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise
  emit compiler-default
  emit fp32-functions-fast  -fmetal-math-fp32-functions=fast
  emit math-mode-fast       -fmetal-math-mode=fast
  emit math-mode-relaxed    -fmetal-math-mode=relaxed
)

# One row per (flag set, kernel), six flag sets. A flag set the compiler
# rejected emits no rows at all, so a wrong count is a missing flag set rather
# than a silently short record.
expected=$((DECLARED_KERNELS * 6))
observed=$(printf '%s\n' "$record" | grep -c .)
if [ "$observed" -ne "$expected" ]; then
  printf 'population mismatch: emitted %s rows, expected %s (%s kernels x 6 flag sets)\n' \
    "$observed" "$expected" "$DECLARED_KERNELS" >&2
  exit 1
fi

printf 'flag_set\tkernel\topcode_signature\n'
printf '%s\n' "$record"
