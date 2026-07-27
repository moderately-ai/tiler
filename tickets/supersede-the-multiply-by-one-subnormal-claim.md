---
id: supersede-the-multiply-by-one-subnormal-claim
title: Supersede the multiply-by-one subnormal claim in the Metal realization record
status: done
priority: p3
dependencies: []
related: [check-in-apple-numerical-behaviour-probe]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [docs, numerics, correction]
---
The outcome of `prototype-metal-numerical-realization` states that "an emitted `x * 1.0f` returns `0x00000000` for the operand `0x00000001`", as one of three rows supporting "Apple GPU `f32` arithmetic flushes subnormals in every mode".

That row does not reproduce. `spikes/apple-targets/numerical_probe.py` measures `x * 1.0f` returning `00000001` unchanged under `safe`, `relaxed`, and `fast`, at both `-O0` and `-O2`, on the same host and toolchain build (`case.multiply_one.*` in `spikes/apple-targets/results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`). ADR 0076's re-verification already reached the opposite conclusion for that kernel — "`x * 1.0` is folded to a copy under every mode" — so the ADR and the ticket it cites disagree, and the measurement agrees with the ADR.

The claim itself survives: the other two rows (`x * 2.0f` on a subnormal operand, `x * 0.5f` on a normal operand) reproduce with execution witnesses. Only the `x * 1.0f` row proves nothing, and it is the exact shape that ADR 0076 later identifies as the trap.

Annotate the `prototype-metal-numerical-realization` outcome so a reader does not inherit the wrong row: state that the `x * 1.0f` observation is superseded by `docs/research/apple-targets/numerical-behaviour.md`, that the surrounding claim is unaffected, and why the kernel cannot support it. Do not rewrite the historical outcome; append a correction, since the ticket is `done` and its record is evidence of what was believed at the time.

## Outcome — annotated, with both halves re-measured (2026-07-27)

**The superseded row was reproduced, not taken on the ticket's word.** `case.multiply_one.*.results` returns `00000001 00400000 007fffff 00800000 80400000 80000000 3eb97ef9 3f800000` — every operand unchanged — under `safe`, `relaxed`, and `fast` at both `-O0` and `-O2`. Nothing flushes.

**The surviving claim was re-measured too, because "the other two rows reproduce" is an assertion like any other.** Both hold, and both carry the non-subnormal execution witness that makes them evidence rather than a bit pattern: `multiply_half` sends `00800000` to `00000000` while sending `3eb97ef9` to `3e397ef9`; `multiply_two` sends `00000001`, `00400000`, and `007fffff` to `00000000` while sending `00800000` to `01000000` and `3eb97ef9` to `3f397ef9`. A non-subnormal operand changes in each, so the arithmetic ran.

**One detail sharpens the ticket's framing.** The ticket says the row does not reproduce; the more useful statement is that this kernel *cannot* support the claim regardless of what it returns. `x * 1.0f` is an identity on every operand, so it admits no witness, and the record shows both readings are live — zero floating-point operations survive at `-O2`, while at `-O0` an `fmul` survives and the operand is *still* returned unchanged. A folded multiply and a hardware special case for `1.0` are indistinguishable here. The appended note says that, so a reader learns the shape of the trap and not only that one row was wrong.

**The historical outcome is unrewritten.** The correction is appended to `prototype-metal-numerical-realization` as a dated section, per this ticket's instruction that a `done` ticket's record is evidence of what was believed at the time.
