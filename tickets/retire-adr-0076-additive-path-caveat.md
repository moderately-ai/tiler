---
id: retire-adr-0076-additive-path-caveat
title: Retire ADR 0076's additive-path re-establishment caveat
status: todo
priority: p3
dependencies: []
related: [extend-the-numerical-probe-to-an-additive-path-kernel, broaden-the-apple-numerical-probe-matrix, repoint-adr-0076-evidence-at-the-numerical-record]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, metal]
---
`docs/decisions/0076-declare-target-honourable-numerical-realizations.md` states, in two places, that one re-verified observation is not reproduced by the checked-in harness.

Its `Measured evidence` bullet: "One re-verified observation is outside the harness's kernel set and is therefore *not* re-established: the additive-path input flush, since every probe kernel that adds does so after a multiply. `extend-the-numerical-probe-to-an-additive-path-kernel` owns closing that gap."

Its subnormal-flush Measurement: "An emitted `x + 0x00800000` returns `0x00800000` for the operand `0x80400000` … confirming input flushing on the additive path — this last observation alone is not reproduced by the harness, whose every adding kernel adds after a multiply, and `extend-the-numerical-probe-to-an-additive-path-kernel` owns closing it."

Both are now stale in the direction that matters: the observation reproduced. `extend-the-numerical-probe-to-an-additive-path-kernel` added the `add_smallest_normal` kernel — a single `x + 2**-126` with no multiply before it — and `docs/research/apple-targets/numerical-behaviour.md` finding 20 records it returning `00800000` for `80400000` at `-O0` and `-O2`, under `safe`, `relaxed`, and `fast`, on both compilation paths, for both dispatchable families, with an execution witness (`00800000 → 01000000`) reporting `executed` in every configuration. The kernel is admissible under the relaxed modes where the `scale 1.0, bias +0.0` kernel is not, because adding a nonzero constant is an identity on no operand.

Do not delete the caveat's history. It recorded a real gap and the reason it existed; record that it closed, where, and on which environment row, in the way the ADR already handles an evidence refresh elsewhere.

Two further consequences of `broaden-the-apple-numerical-probe-matrix` may belong in the same pass, and both are conclusions the ADR supports rather than contradicts. Finding 16 measures a source-level `fma` fusing at every `-ffp-contract` setting including `off`, so contraction control is a constraint on what the emitter may write and not something the flag enforces on its behalf. Finding 17 measures a two-add chain reassociated under `relaxed` and `fast`, so a target profile admitting those modes cannot promise a reduction order on this row; `qualify-contraction-association-reassociation-permission` may be the better home for that one.

Closes when the ADR no longer claims the additive-path observation is unreproduced and `uv run --locked python scripts/check_repository.py` passes.
