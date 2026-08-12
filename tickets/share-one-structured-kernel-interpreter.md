---
id: share-one-structured-kernel-interpreter
title: Share one structured-kernel interpreter between the IR and compiler tests
status: closed
priority: p3
dependencies: []
related: [lower-a-loop-carried-cooperative-body, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir, implementation/compiler, implementation/workspace, implementation/cargo-lock, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, decision, needs-tom, public-boundary]
closed_reason: superseded
closed_note: "Tom rejected both sharing options on 2026-08-12. The real destination is the production scalar CPU backend; authoritative-evidence migration precedes physical deletion of both test simulators."
---
## Terminal outcome — superseded 2026-08-12

Tom rejected both Option A and Option B. Neither test `KirMachine` becomes shared, public, doc-hidden, or a workspace support crate. Both are physically deleted after their legitimate assertions move to independent reference semantics, structural IR/compiler verification, or execution through a real CPU or Metal backend.

The replacement graph is:

- [`accept-the-production-boundary-for-the-bounded-scalar-cpu-backend`](accept-the-production-boundary-for-the-bounded-scalar-cpu-backend.md) — accept the exact production ownership/API boundary;
- [`promote-the-bounded-scalar-cpu-vertical-into-a-production-backend`](promote-the-bounded-scalar-cpu-vertical-into-a-production-backend.md) — land the real serialized-image CPU path;
- [`execute-the-loop-carried-cooperative-kernel-on-a-real-backend`](execute-the-loop-carried-cooperative-kernel-on-a-real-backend.md) — replace the one multi-round execution claim that currently exists only in the IR simulator;
- [`replace-host-kir-simulator-claims-with-authoritative-evidence`](replace-host-kir-simulator-claims-with-authoritative-evidence.md) — migrate the complete consumer census; and
- [`delete-the-two-host-kir-simulators`](delete-the-two-host-kir-simulators.md) — remove every simulator and compatibility surface.

### Fact repair at `449d54b864b849993692e8bf12117f9064f76b4d`

- **Verified:** two private `KirMachine` implementations exist and differ in barrier nesting, multi-round handling, buffers, BF16, and operation vocabulary.
- **False:** one interpreter can answer “what does this verified kernel compute.” `VerifiedKernel` does not carry the complete launch geometry; both machines infer participants from a staging allocation, and neither covers the live KIR vocabulary.
- **False:** the copies merely need identical semantics. They are non-authoritative test simulators, while `tiler-reference` owns semantic meaning and a backend owns execution.
- **False:** a dev-only crate avoids the architectural defect. It hides the API but institutionalizes a second execution implementation with no product consumer.
- **Imprecise:** the named population was two conformance tests. Compiler pipeline and conformance modules have many additional consumers spanning access maps, bindings, staged values, BF16, nonlinear expressions, reductions, and split plans.
- **Verified replacement:** `spikes/target-profiles/scalar-cpu-vertical`, anchor `A second backend, materially different from Metal`, already proves the correct shape: translate verified KIR to versioned payload bytes, decode without compiler objects, qualify the actual host, route, execute, and compare independently with `tiler-reference`.

The historical packet below is retained to explain what was rejected; it is not live guidance.

## Historical user-visible outcome

One interpreter answers "what does this verified kernel compute", and both crates' conformance tests read it. Until then there are two, they disagree about what a body may contain, and the weaker one on barrier-nesting / multi-round executability is the one the pipeline tests use — not a weaker overall feature surface (the pipeline machine is richer on multi-buffer binding, bf16, and op vocabulary).

## The two, and how they differ

**Fact — `KirMachine` in `crates/tiler-compiler/src/pipeline/tests.rs`** splits the kernel's *top-level* operation list at each barrier and advances every lane through each segment. `barrier_segments` reads only the top level, so a barrier anywhere else is unreachable to it — which was exactly right when the verifier refused every barrier below block depth zero.

**Fact — `KirMachine` in `crates/tiler-ir/src/kernel/tests.rs`** flattens the body into a step stream first, unrolling any loop that contains a barrier into its iterations with explicit accumulator-binding steps, and then splits *that* at each barrier. It has to, because a loop-carried cooperative body's barriers sit inside the round loop.

**Inference — the compiler's copy cannot execute a multi-round cooperative kernel**, and will fail on the first pipeline test that produces one. The second copy exists because `crates/tiler-compiler` was an occupied lane when the loop-carried body landed, not because the two need different semantics.

## What this owns

Deciding where the shared machine lives and moving both call sites onto it. The plausible home is a test-support module in `tiler-ir` — the IR is what it interprets, and `tiler-compiler` already depends on `tiler-ir` — but a `#[cfg(test)]` module is not reachable across crates, so this needs an explicit decision about whether the interpreter becomes a non-test item (a public or `doc(hidden)` API, which is a public boundary and therefore Tom's) or a small dev-only crate. Do not resolve that by making a test helper public without asking.

## Decision packet — 2026-08-09

- **Option A — a non-test, doc-hidden `tiler-ir` support item.** This adds no crate, but makes a test interpreter reachable through a production crate and therefore creates a public Rust surface whose stability and consumer exclusions must be stated.
- **Option B — a dev-only workspace crate, `tiler-kernel-test-support` (recommended).** Both test suites depend on one implementation without turning test machinery into `tiler-ir`'s production API. It costs one small workspace member and lockfile/dependency-graph maintenance, which is narrower than a permanently reachable public helper.

Tom needs to choose the ownership boundary. The implementation may then build one shared authority that preserves the IR machine's barrier-containing-loop flatten (Seed/Iterate/Yield/Exit steps and Rendezvous split) and the compiler machine's multi-buffer, bf16, and op vocabulary surfaces that existing pipeline tests already exercise, then delete both private copies; it must not create a third temporary copy.

## Closes when

One implementation is reachable from both crates' conformance tests, the loop-carried body's flattening survives the move, and each crate's existing conformance assertions still hold on it.
