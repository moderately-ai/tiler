---
id: prototype-shape-evidence-spike
title: Measure stable-Rust shape-evidence feasibility
status: todo
priority: p0
dependencies: [prototype-typed-value-handles]
related: [prototype-shaped-value-api, prototype-semantic-reference-slice]
scopes: [research/shapes]
shared_scopes: [project/tickets, contracts/foundation, contracts/decisions]
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
