---
id: share-one-structured-kernel-interpreter
title: Share one structured-kernel interpreter between the IR and compiler tests
status: awaiting-decision
priority: p3
dependencies: []
related: [lower-a-loop-carried-cooperative-body, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir, implementation/compiler, implementation/workspace, implementation/cargo-lock, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, decision, needs-tom, public-boundary]
---
## User-visible outcome

One interpreter answers "what does this verified kernel compute", and both crates' conformance tests read it. Until then there are two, they disagree about what a body may contain, and the weaker one is the one the pipeline tests use.

## The two, and how they differ

**Fact — `KirMachine` in `crates/tiler-compiler/src/pipeline/tests.rs`** splits the kernel's *top-level* operation list at each barrier and advances every lane through each segment. `barrier_segments` reads only the top level, so a barrier anywhere else is unreachable to it — which was exactly right when the verifier refused every barrier below block depth zero.

**Fact — `KirMachine` in `crates/tiler-ir/src/kernel/tests.rs`** flattens the body into a step stream first, unrolling any loop that contains a barrier into its iterations with explicit accumulator-binding steps, and then splits *that* at each barrier. It has to, because a loop-carried cooperative body's barriers sit inside the round loop.

**Inference — the compiler's copy cannot execute a multi-round cooperative kernel**, and will fail on the first pipeline test that produces one. The second copy exists because `crates/tiler-compiler` was an occupied lane when the loop-carried body landed, not because the two need different semantics.

## What this owns

Deciding where the shared machine lives and moving both call sites onto it. The plausible home is a test-support module in `tiler-ir` — the IR is what it interprets, and `tiler-compiler` already depends on `tiler-ir` — but a `#[cfg(test)]` module is not reachable across crates, so this needs an explicit decision about whether the interpreter becomes a non-test item (a public or `doc(hidden)` API, which is a public boundary and therefore Tom's) or a small dev-only crate. Do not resolve that by making a test helper public without asking.

## Decision packet — 2026-08-09

- **Option A — a non-test, doc-hidden `tiler-ir` support item.** This adds no crate, but makes a test interpreter reachable through a production crate and therefore creates a public Rust surface whose stability and consumer exclusions must be stated.
- **Option B — a dev-only workspace crate, `tiler-kernel-test-support` (recommended).** Both test suites depend on one implementation without turning test machinery into `tiler-ir`'s production API. It costs one small workspace member and lockfile/dependency-graph maintenance, which is narrower than a permanently reachable public helper.

Tom needs to choose the ownership boundary. The implementation may then move the complete IR machine, keep one authority, and delete both copies; it must not create a third temporary copy.

## Closes when

One implementation is reachable from both crates' conformance tests, the loop-carried body's flattening survives the move, and each crate's existing conformance assertions still hold on it.
