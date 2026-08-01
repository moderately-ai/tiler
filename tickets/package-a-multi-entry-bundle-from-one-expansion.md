---
id: package-a-multi-entry-bundle-from-one-expansion
title: Package and dispatch a multi-entry bundle from one expansion
status: blocked
priority: p2
dependencies: [calibrate-and-activate-parallel-reduction-selection, denote-a-reduction-region-in-the-inline-macro-grammar]
related: [prototype-inline-aot-integration-proof, dispatch-a-tiler-region-on-metal-hardware, admit-multi-input-elementwise-programs-at-the-compiler-boundary, reach-a-reassociation-permitting-contract-from-a-bound-metal-declaration]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, artifacts]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785611640
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

## Blocked (2026-08-01, base `2aa0824`)

Dispatched as implementation and stopped at the feasibility step this ticket's own keys require: the trigger must be a real planning outcome, and **no region this frontend can expand has a multi-entry plan to select**. The derivation is below so it can be refuted rather than repeated. Nothing was implemented and no crate changed on this branch.

**Measurement — Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19`, Apple metal 32023.883, 2026-08-01.** Programs compiled against `BoundMetalCompileDeclaration::first_macos_apple9`'s profile, reading `Compilation::alternatives()` and `Compilation::selected()`.

1. **A pointwise region has exactly one plan alternative, not merely a cheaper one.** Every grammar-admissible program — `f32` inputs with literal extents combined only by `*` and `+` — retains a single fused, one-kernel alternative. Swept across input counts 2–8, chain depths 1–32, and extents 4–1048576; the portfolio never held a second entry. The cause is structural rather than economic: the governed physical provider's pointwise branch offers an implementation only when a region's members equal the whole recognized pointwise member set, so every multi-region cover contains a region with no implementation and contributes no plan at all. **No cost model, present or future, makes a pointwise region multi-entry.**
2. **The one admissible contract selects the one-kernel plan.** The compiler's recognized serial-sum program (`strict_serial_sum(input * 2.0 + 1.0, axis 1)`) retains fused/1-kernel *and* materialized/2-kernel under `FlushSubnormalsToZeroF32`, and selects the fused one — it Pareto-dominates on all four structural dimensions (`dispatch_count`, `launched_threads`, `temporary_bytes`, `materialization_count`). Selection contains no `if is_fused`; the fused plan wins on cost.
3. **The contract under which a split *is* selected cannot be stated ahead of time.** `ReassociateF32` — landed by `admit-a-reassociating-contract-without-contraction` precisely to make the split reachable — is refused against the bound declaration with `InputSubnormals { required: Preserve }` / `DeclaredUnhonourable`, because the measured `f32` row flushes input subnormals. `StrictF32` and `RelaxedF32` refuse identically. Only `FlushSubnormalsToZeroF32` compiles, which is what `only_one_numerical_contract_is_admissible_for_the_bound_declaration` already pins.
4. **The grammar cannot denote the shape anyway.** Reaching any program with more than one alternative needs the recognizer's serial-sum window — one input, 4–5 operations, scalar constants and a sum. The region grammar has no reduction spelling and no scalar literal. A multi-input reduction is refused with `UnsupportedCapability { rule: "input-arity" }`; softmax, RMS-norm, and SiLU with `rule: "operation-set"` / `"operation-family"`.

**Fact — the two things this ticket suspected were already correct.** `tiler_macros::aot::deliver`'s `[payload] = artifact.payloads()` refusal is about the axis it names and needs no widening: `crates/tiler-build/src/metal_plan.rs` emits one payload per *declared family* and one entry per *stage*, so a multi-entry plan produces one payload and several entries. And the consumer half is already generic — `spikes/runtime/inline-dispatch`'s adapter iterates `preflight.entries()` in `prepare_entries`, `plan_dispatch`, and `dispatch`, pairs `shared_allocations()`, and encodes one Metal encoder per entry for ordering. What is missing is upstream of both: a region whose selected plan has two entries.

**Not faked, deliberately.** Handing `accept_or_publish_metal_plan` a non-selected alternative via `alternatives().find(|plan| !plan.is_fused())` would produce a two-entry artifact today. That is the fixture path `tiler_build`'s `distinct_owner_linked_plans_produce_distinct_artifacts` already exercises, it contradicts this ticket's first implementation key, and it would make the frontend override the optimizer — so it was rejected rather than deferred.

**Filed rather than absorbed.** `reach-a-reassociation-permitting-contract-from-a-bound-metal-declaration` owns finding 3, which had no owner and also constrains `calibrate-and-activate-parallel-reduction-selection`'s stated measurement. `denote-a-reduction-region-in-the-inline-macro-grammar` owns finding 4 and is independently reachable now. Finding 2 is already owned by `calibrate-and-activate-parallel-reduction-selection`, so no duplicate was filed for it; finding 1's wider recognizer question is `admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary`'s.

**Reachable remainder: none.** Every part of this ticket's outcome depends on a multi-entry selected plan existing. There is no frontend-side slice to split out, which is why this is `blocked` rather than partially delivered.

**Why the edges are these two.** `denote-a-reduction-region-in-the-inline-macro-grammar` is required: without it no region has a second alternative at all. `calibrate-and-activate-parallel-reduction-selection` is required because it owns the general mechanism — analytical cost entering selection — that lets any multi-dispatch plan win against a strictly dominating single-dispatch one. `reach-a-reassociation-permitting-contract-from-a-bound-metal-declaration` is deliberately *related* rather than a dependency: it is one of two routes to a selected split, and the other — the two-kernel materialized plan winning under `FlushSubnormalsToZeroF32` — does not need it. Whichever lands first unblocks this ticket; requiring both would overstate the constraint.
