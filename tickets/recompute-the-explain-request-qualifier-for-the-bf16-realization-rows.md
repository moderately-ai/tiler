---
id: recompute-the-explain-request-qualifier-for-the-bf16-realization-rows
title: Recompute the explain request qualifier for the bf16 realization rows
status: todo
priority: p1
dependencies: []
related: [admit-a-bf16-index-realization-law-and-refinement-contract]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, bf16, identity]
---
## User-visible outcome

The workspace gate is green again on the tree that made the `bf16` family reachable through the refinement layer. Exactly one pinned identity moved, it is known, its replacement value is computed, and this ticket applies it.

## The moved pin

**Fact, measured on `tkt/admit-a-bf16-index-realization-law-and-refinement-contract`.** `crates/tiler-compiler/src/explain.rs:4090` pins the request qualifier of `deterministic_trace_is_sealed_and_rendered_separately`:

- old: `8e06e11fdc3a2889`
- new: `b2d55d5a36e0159b`

**Measurement.** Two independent full-workspace runs on that branch reported the identical `left` value, and it was the only failure in each: `2707 tests run: 2706 passed, 1 failed, 7 skipped`. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and read the `left` value the assertion reports.

## Why it moved, and which half

**Fact.** `VerifiedRequestSubject.realization_registry` is `FrozenIndexRealizationLawRegistry::identity()` (`crates/tiler-compiler/src/request.rs:2324-2328`), and that identity folds the scalar-registry snapshot and the whole count-prefixed law sidecar (`crates/tiler-ir/src/index/refinement.rs`, `from_semantic`). The branch added three `bf16` scalar definitions to the standard scalar registry and three `bf16` index-realization law rows to the standard provider's sidecar, so both land in that one field.

**Fact.** The *semantic* half did not move. `FrozenSemanticRegistry::snapshot_identity` is computed over definitions and operations only (`crates/tiler-ir/src/semantic/registry.rs`, `compute_identity`); the three `bf16` operation families were already registered, and registering a law adds no operation. This is the compiler half moving alone, the same shape as the eighth lowering-capability row's step.

**Inference.** No encoding version is owed. The three law rows reuse the existing constant and pointwise tags, so the sidecar widened by three self-delimiting rows and a count rather than changing shape, and nothing about explain's rendering moved — the trace's own two record lines are unchanged.

## What to do

Replace the literal at `crates/tiler-compiler/src/explain.rs:4090` with the new value and append a paragraph to that test's rebaseline ledger stating the change, which half of the subject moved, and that no encoding version stepped — the convention every prior entry in that ledger follows.

## Why it is a separate ticket

The producing branch holds `implementation/ir` only. `crates/tiler-compiler/**` is `implementation/compiler`, a distinct exclusive scope, and the worker stopped at that boundary rather than editing outside it.

## Closes when

The literal is updated with its ledger paragraph and a full-workspace run is green.
