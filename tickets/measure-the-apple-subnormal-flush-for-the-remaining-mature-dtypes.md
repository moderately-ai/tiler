---
id: measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes
title: Measure the Apple subnormal flush for the remaining mature dtypes
status: in-progress
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype, enumerate-the-mature-tensor-dtype-taxonomy, carry-the-dtype-on-the-metal-subnormal-flush-fact]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement, dtypes]
claimed_from: todo
assignee: agent-measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes
lease_expires_at: 1785038575
---
Finding 21 of [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md) measures `f32` flushing subnormals and `f16` preserving them on the same hardware, from modules that declare `air.compile.denorms_disable` identically. That settles that the flush depends on the dtype. It settles nothing about *which* dtypes flush: two disagreeing dtypes are a refutation of dtype-independence and not a rule, and a third could behave like either.

Every other format Metal exposes is unmeasured. `bfloat` is the one that matters most for tensor compute and has the same exponent range as `f32` with `f16`'s width, so it discriminates between the two explanations finding 21 leaves undistinguished better than any other candidate: a preserving `bfloat` would be evidence against "narrow formats are evaluated at a wider internal precision", because `bfloat`'s subnormals are not `f32` normals.

**Harness reuse — this is now a table row, not a reshape.** `widen-the-apple-numerical-probe-to-a-second-dtype` moved every width-dependent decision into `Dtype`: the operand vector, the result rendering, the MSL constant spelling, the NaN-canonicalization helper, the exact evaluation through `struct`, the dispatch host's element width and sentinel. Adding a dtype is a `Dtype` row, its kernels with probes derived by `evaluate`, and a `cases` block. Do not add one without checking what the front end spells its fused intrinsic and its arithmetic opcodes as — the operation-count patterns are pinned by a portable test that must be extended in the same change, because a surviving operation counted as zero is indistinguishable from a deleted one.

**Stop condition.** Keep the covering matrix bounded the way it is now: a dtype earns cases in the gate's covering set only for the kernels a finding cites, and the rest go behind `TILER_APPLE_NUMERICS_EXHAUSTIVE`. Report what the addition costs the gate in wall-clock.

**What closes this.** Each added dtype has a witnessed verdict for input flushing, result flushing, and the sign of a flushed zero, in every math mode, on both compilation paths; the record states which dtypes were measured and keeps every unmeasured one explicitly unmeasured; and `carry-the-dtype-on-the-metal-subnormal-flush-fact` gains the per-dtype rows it can declare.
