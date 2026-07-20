---
id: prototype-shape-evidence-spike
title: Measure stable-Rust shape-evidence feasibility
status: done
priority: p0
dependencies: [prototype-typed-value-handles]
related: [prototype-shaped-value-api, prototype-semantic-reference-slice]
scopes: [research/shapes]
shared_scopes: [project/tickets, contracts/foundation, contracts/decisions, contracts/navigation]
paths: [Cargo.lock]
tags: [tiler-research, spike, prototype, shapes, rust-api, measurement]
---
Build a dependency-minimal retained experiment against Rust 1.89 that measures
the non-authoritative shape-evidence layer accepted conceptually by ADR 0061.

Compare fixed-rank and exact-static evidence, checked refinement, explicit
weakening, pointwise propagation, typed reduction axes, invalid or duplicate
axes, forgery resistance, foreign witnesses, and compiler diagnostics. Include
compile-pass/fail cases and generated workloads with 1, 10, 100, and 1,000
distinct static shapes; record clean and incremental check time, optimized build
time, and binary-size growth with complete toolchain provenance.

The spike must demonstrate that all successful refinement is rechecked against
authoritative graph metadata and that solver-only symbolic relationships remain
graph proofs rather than Rust trait claims. Stop and report failure if the
usable design requires nightly `generic_const_exprs`, recursive typenum-style
algebra, overlapping specialization, or materially inferior diagnostics. The
result recommends exact public spelling and the bounded initial evidence
vocabulary; it does not stabilize or implement that API in `tiler-ir`.

## Outcome

The retained Rust 1.89 spike validates privately constructed
`ShapedValue<T, E>`, checked `Rank<R>` and `Exact<S>` refinement, explicit
weakening, one canonical pointwise admission path, compile-time axis bounds and
uniqueness, and graph/subject-bound witnesses. Compile-fail cases preserve
diagnostics for rank and exact-shape mismatch, invalid axes, attempted evidence
implementation, and handle forgery.

Generated 1/10/100/1,000-shape workloads show bounded check and optimized-build
growth on the measured M4 Max host with no optimized binary-size growth. The
recommended initial spelling and measured limits are recorded in
`docs/research/shapes/stable-rust-shape-evidence.md`; general reduction result
rank arithmetic remains deliberately weaker because stable Rust cannot express
`Rank<{R - 1}>` without unstable generic const expressions.
