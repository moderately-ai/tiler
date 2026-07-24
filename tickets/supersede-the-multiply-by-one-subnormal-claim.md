---
id: supersede-the-multiply-by-one-subnormal-claim
title: Supersede the multiply-by-one subnormal claim in the Metal realization record
status: todo
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
