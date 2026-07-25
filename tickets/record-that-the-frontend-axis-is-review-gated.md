---
id: record-that-the-frontend-axis-is-review-gated
title: The frontend axis is gated on a human review, not on engineering
status: todo
priority: p1
dependencies: []
related: []
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [planning, frontend, process]
---
A finding about the parallel-widening plan itself, recorded because the plan's method assumes the axes are independently workable and one of them is not.

**Fact — the frontend axis's dependency structure, read from the tickets.**

- `prototype-inline-proc-macro-frontend` (todo) depends on `prototype-public-compiler-api` and `prototype-neutral-artifact-codec`.
- `prototype-candle-metal-adapter` (todo) depends on `prototype-inline-aot-integration-proof`.
- `prototype-macro-embedding-and-cargo-behavior` (todo) depends on `prototype-inline-proc-macro-frontend`, `prototype-expansion-content-cache`, `prototype-artifact-family-delivery`, and `prototype-metal-aot-slice`.
- `macro-build-environment`, `proc-macro-extension-visibility`, `resolve-macro-environment-alarm-path-dependence`, and `repair-macro-and-embedding-harness-integrity` are already done or closed.

**Fact — `prototype-public-compiler-api`'s closing condition is a review.** That ticket states that "any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit", and its own recorded status names "Tom has not reviewed it" as the first reason it stays open. `pub mod session` exists and works; what is missing is acceptance, not code.

**Inference — exactly one frontend ticket is dependency-satisfied**, `prototype-expansion-content-cache`, and it is a cache rather than a frontend. So the axis cannot be widened past that point by engineering alone.

**Why this matters to the plan rather than only to the schedule.** The parallel-widening method holds that each axis stresses a different seam and that sequencing them would let each be designed around the previous one's answers. The frontend axis is the one that "exercises artifact identity, caching and the public boundary under real reuse" — it is the only axis that tests the public boundary at all, and it is gated on that boundary being accepted. That is not a scheduling accident: an axis whose purpose is to stress a boundary cannot run before the boundary is agreed, so this axis is genuinely ordered after a decision rather than after other engineering.

**The other four axes are unaffected.** Shapes, numerics, operations, and targets each have dependency-satisfied work that touches no public boundary, which is evidence that the axes are otherwise independent as the method assumes.

## Closes when

Either `prototype-public-compiler-api` is reviewed and accepted, unblocking the axis, or the plan records that the frontend axis is deliberately sequenced after that acceptance and does not count toward parallel widening until then. Do not close this by starting frontend work that routes around the unreviewed boundary — that would answer the review question by omission, which is what `prototype-public-compiler-api` exists to prevent.
