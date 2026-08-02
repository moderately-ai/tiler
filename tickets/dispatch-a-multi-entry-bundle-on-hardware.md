---
id: dispatch-a-multi-entry-bundle-on-hardware
title: Dispatch a multi-entry bundle on hardware from one expansion
status: todo
priority: p2
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, calibrate-and-activate-parallel-reduction-selection, correct-the-declined-strategy-record-for-an-unsplittable-reduction]
scopes: [research/runtime, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifacts, spike]
---
## User-visible outcome

An out-of-tree consumer dispatches, on Metal hardware, a bundle one `tiler::tensor!` invocation packaged with more than one executable entry — the entries run in the order the artifact declares, the result equals the consumer's own arithmetic bit for bit, the entry count is asserted, and a deliberate reordering is watched producing a wrong answer rather than a refusal.

## Why this exists

`package-a-multi-entry-bundle-from-one-expansion` delivered the producer half and stopped at a scope boundary rather than at a technical one. Its worker held `implementation/frontend` alone, and every remaining piece of its own *closes when* condition lives outside it: the hardware consumer is `spikes/runtime/**` (`research/runtime`) and the remaining-checks list is `docs/integration/**` (`contracts/integrations`).

**Measurement — Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19`, base `4d08a3f`, 2026-08-02.** The region `in x: f32[rows: 1, cols: 4]; contract flush_and_reassociate_f32; out strict_serial_sum(x * 2.0 + 1.0, [cols])` compiled against `BoundMetalCompileDeclaration::first_macos_apple9`'s profile retains three alternatives with kernel counts `[2, 2, 3]`, and `Compilation::selected()` returns the two-kernel one — `stable_id = program-alternative:3724e762c78ac7a7`, `kernels().len() = 2`, `is_fused() = false`, two ABI entries. The artifact `accept_or_publish_metal_plan` produces carries **one payload** and **two entries** with one stage dependency running front to back, and `tiler_macros::aot::deliver` embeds exactly that artifact. Under `flush_subnormals_to_zero_f32` the same region selects the fused one-kernel plan.

**Fact — what already landed.** `crates/tiler-macros/src/aot/tests.rs`'s `a_split_selection_packages_every_entry_in_the_one_embedded_artifact` asserts the entry count and the declared order on the artifact the expansion embeds, with the flush-only contract as its watched perturbation. `crates/tiler-macros/src/region/tests.rs`'s `the_reduction_grammar_reaches_a_multi_entry_selected_plan` pins that the region a consumer *writes* is what selects the split. `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` states the region in an out-of-tree consumer crate.

**Fact — why the in-tree consumer cannot finish it.** `crates/tiler/tests/facade/pass/inline_region_dispatches.rs` refuses at `validate_payload`, which `route_with_adapter` calls per entry and stops at the first refusal — so that consumer observes entry 0 and never the count. The refusal is correct rather than incidental: ADR 0090 item 8 places payload validation on the backend, a `trybuild` fixture cannot declare the `metal` crate, and returning `Ok` would be claiming the bytes decode into something executable. Only a consumer that really executes a `metallib` reaches `prepare_entries`, `plan_dispatch`, and `dispatch`.

## Implementation keys

- **Do not fake the split.** The trigger stays a real planning outcome: the region states `flush_and_reassociate_f32` and the compiler's selection policy answers with two entries. Handing `accept_or_publish_metal_plan` a non-selected alternative via `alternatives().find(|plan| !plan.is_fused())` is the path the 2026-08-01 run rejected, and that rejection stands.
- **`[rows: 1, cols: 4]` is the window, not a taste.** Measured on this declaration under `flush_and_reassociate_f32`, `[rows: 1, cols: 8]` and `[rows: 2, cols: 4]` are refused as `NoFeasiblePlan` — a regrouping-permitting contract withholds the whole-program fused plan, so a portfolio with no admissible split has no plan at all — and `[rows: 1, cols: 5]` is refused as `InvalidCompilerOutput`, which `correct-the-declined-strategy-record-for-an-unsplittable-reduction` owns. Widening the window is `calibrate-and-activate-parallel-reduction-selection`'s and the reduction-strategy work's, not this ticket's.
- **The spike's existing one-entry claim is cited evidence and must survive.** `spikes/runtime/inline-dispatch`'s transcript (`committed route completed: 1/1 entry(ies) encoded`) is quoted in several documents, so a two-entry region is added *beside* the existing one rather than replacing it. The adapter is already generic over entry count — it iterates `preflight.entries()` in `prepare_entries`, `plan_dispatch`, and `dispatch`, pairs `shared_allocations()`, and encodes one Metal encoder per entry.
- **The oracle is the consumer's own arithmetic and must stay bit-exact under regrouping.** The stated contract permits reassociation, so a comparison is only a statement about the dispatch when every partial sum is exactly representable; choose operands that make it so and say why, as the spike's `LEFT`/`RIGHT`/`ADDEND` comment already does.

## Evidence

- The entry count asserted from the consumer's side, so the run cannot pass on a single-entry plan that happened to be selected.
- A deliberate reordering watched failing first — the perturbation `crates/tiler-runtime/tests/adapter_route` already runs at `main.rs:1102`, where dispatching back to front returns a wrong answer rather than a refusal. A completed route whose entries ran out of order must not be observable.
- The dispatched bytes compared against the consumer's own `f32`, before any other claim.

## Closes when

The spike runs a two-entry region to a completed dispatch with the oracle agreeing and the reordering watched failing, its README records the invocation and the transcript, and `docs/integration/frontends.md`'s remaining-checks list moves "one invocation may contain … a multi-step plan such as a two-pass reduction" from outstanding to landed with the citation.
