---
id: package-a-multi-entry-bundle-from-one-expansion
title: Package and dispatch a multi-entry bundle from one expansion
status: todo
priority: p2
dependencies: []
related: [prototype-inline-aot-integration-proof, dispatch-a-tiler-region-on-metal-hardware, admit-multi-input-elementwise-programs-at-the-compiler-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, artifacts]
---
## User-visible outcome

One `tiler::tensor!` invocation whose selected plan needs more than one executable entry packages all of them into its one embedded artifact, and a consumer dispatches them in the order the artifact declares.

## Why this exists

**Fact.** `docs/integration/frontends.md` states that "one invocation may contain one fused kernel, multiple guarded schedule variants, or a multi-step plan such as a two-pass reduction" and that "macro-local bundle does not mean one GPU kernel". Nothing an expansion produces has ever exercised that.

**Fact.** Today's grammar admits one `out` expression over `*` and `+`, and the region it compiles packages one entry: `spikes/runtime/inline-dispatch`'s transcript reports `commit: committed route completed: 1/1 entry(ies) encoded`.

**Fact.** The multi-entry route *is* exercised, but never against an artifact a macro produced. `crates/tiler-runtime/tests/adapter_route` drives two entries over one shared allocation, including the ordering perturbation at `main.rs:1102` where dispatching back to front returns a wrong answer rather than a refusal — from a hand-built fixture (`fixture.rs`'s materialized member), not from an expansion.

**Inference — the gap is upstream of the loader.** The runtime and the artifact model already carry entry ordering obligations (`tiler_artifact`'s `program::model` orders two entries of a variant). What is missing is a region whose selected plan the compiler splits, and the frontend path that hands `accept_or_publish_metal_plan` a plan with more than one entry.

## Implementation keys

- The trigger must be a real planning outcome, not a grammar knob: a region the optimizer splits (a two-pass reduction is the contract's own example) rather than a `deliver`-style statement asking for two kernels.
- `tiler_macros::aot::deliver` reads `artifact.payloads()` expecting exactly one payload and refuses otherwise as `MalformedArtifact`. Entries and payloads are different axes — one payload per *delivery position*, several entries per payload — so check that the refusal is about the axis it names before widening anything.
- Ordering is a correctness contract, not a convenience. The consumer must not be able to observe a completed route whose entries ran out of order, and the perturbation that proves it is the one `adapter_route` already runs.

## Evidence

- An out-of-tree consumer crate whose one region compiles to more than one entry, dispatched on hardware, checked against the consumer's own arithmetic — the shape `spikes/runtime/inline-dispatch` already establishes for one entry.
- The entry count asserted, so the test cannot pass on a single-entry plan that happened to be selected.
- A deliberate reordering watched failing first.

## Closes when

An expansion produces a multi-entry bundle, a consumer dispatches it, ordering is asserted and its violation observed failing, and `docs/integration/frontends.md`'s remaining-checks list moves the item from outstanding to landed with the citation.
