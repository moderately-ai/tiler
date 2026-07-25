---
id: widen-the-f16-operation-vocabulary-to-contraction-and-reassociation
title: Widen the f16 operation vocabulary to contraction and reassociation
status: todo
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype, broaden-the-apple-numerical-probe-matrix]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement, dtypes]
---
The `f16` kernels added by `widen-the-apple-numerical-probe-to-a-second-dtype` cover exactly what its question needed: materialization, the multiply in both flush directions, a bare add taking a subnormal straight from the buffer, a surviving `fdiv` in both directions, the identity multiply, and the trap. Contraction, a source-level `fma`, and reassociation have no `f16` counterpart, so findings 6, 16, and 17 of [the record](../docs/research/apple-targets/numerical-behaviour.md) are `f32`-only.

That is a deliberate boundary and not an oversight, but the reason it was cheap to leave has weakened. Finding 21 established that the two dtypes' *arithmetic* differs while their emitted modules do not, which removes the assumption that an `f32` measurement of what a licence does carries to `f16`. Three specific claims now rest on `f32` alone and are cited as if they were about the compiler rather than about a dtype:

- `-ffp-contract=off` is the measured defence against contraction (finding 6), and it is not a defence against a source-level `fma` (finding 16). Whether an `f16` multiply-add contracts under the same settings is unmeasured.
- `relaxed` and `fast` reassociate a two-add chain (finding 17), which is the measurement behind "a target profile that admits `relaxed` or `fast` cannot promise a reduction order". `f16`'s ulp is 2**-10 at 1.0, so the same shape is expressible with `1.0h`, `2**-11`, and `2**-11` and needs no new machinery.

**Trigger for doing it now rather than later.** Any of: a numerical contract that states a contraction or reduction-order obligation per dtype; an emitter that lowers a `MultiplyThenAdd` at `f16`; or a second dtype landing from `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes`, at which point the `f32`-only shape of these three findings becomes the odd one out rather than the default.

**Cost.** One kernel each for the contraction pair, the fused pair, and the reassociation chain, plus a contraction axis for the first two. Keep the covering set bounded: the contraction settings belong behind `TILER_APPLE_NUMERICS_EXHAUSTIVE` unless a finding cites them.

**What closes this.** Each of findings 6, 16, and 17 either reproduces at `f16` and says so, or does not and is restated as a per-dtype measurement; and the record's "the `f16` vocabulary is narrower" boundary is removed rather than reworded.
