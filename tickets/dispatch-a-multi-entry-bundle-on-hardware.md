---
id: dispatch-a-multi-entry-bundle-on-hardware
title: Dispatch a multi-entry bundle on hardware from one expansion
status: review
priority: p2
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, calibrate-and-activate-parallel-reduction-selection, correct-the-declined-strategy-record-for-an-unsplittable-reduction]
scopes: [research/runtime, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifacts, spike]
claimed_from: todo
assignee: agent-multi-entry
lease_expires_at: 1785878830
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

## Outcome

**Delivered, 2026-08-04.** `spikes/runtime/inline-dispatch` gained a second binary, `src/multi_entry.rs`, built by `cargo run --release --bin multi-entry-dispatch-spike`. It is beside the one-entry consumer rather than in place of it and shares `src/adapter.rs` and `src/buffer.rs`, because the adapter's generality over entry count is exactly what a two-entry route tests and a copy would have tested the copy. `default-run = "inline-dispatch-spike"` keeps the cited `cargo run --release` and `cargo run --release -- --halt-after-commit` invocations unambiguous across two `[[bin]]` targets.

**Measurement — Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`), Xcode 27.0 build 27A5228h, Apple metal 32023.921 (`metalfe-32023.921`), `metal` crate 0.33.0, base `2c4d05c`, 2026-08-04.** This is **not** the 2026-08-02 environment the *Why this exists* measurement above was taken in: the Metal toolchain moved from 32023.883 to 32023.921 with the Xcode 27.0 beta installed the same day, and the two are kept as separate rows rather than merged. The ticket's trigger region still selects the split on this tree. The one invocation runs the route twice on the same device with the same operands and exits `0`:

- **Reordered first.** `reordering: WRONG ANSWER, not a refusal — the route completed and the kernel wrote [0.0] where this consumer's own f32 gives [10.0]`, with `2/2 entry(ies) encoded`. Nothing refused it: both payloads validated, both pipelines built, both entries reached terminal success.
- **Then ordered.** `oracle: the dispatched bytes equal this consumer's own f32 arithmetic bit for bit: [10.0]`, `committed route completed: 2/2 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, `result: f32[1], 4 byte(s)`.
- **The entry count is counted, from three populations:** `entries: 2 payload(s) validated, 2 declared by the committed route, 2 encoded; 1 shared allocation(s)`. The two entries are two kernels of one payload — both report `7174 object byte(s)` and the symbols differ, `tiler_kernel_393f5de6952fd574` launching `4×1` and `tiler_kernel_f635c9c18ef7eb80` launching `1×1`.

**The split was not faked.** The region is the ticket's trigger verbatim and no plan portfolio is consulted; the compiler's selection policy answers `flush_and_reassociate_f32` with two entries. The watched perturbation is the *other* admissible contract: restating `flush_subnormals_to_zero_f32` selects the fused one-kernel plan, which dispatches, completes, and **agrees with the oracle** at `1/1 entry(ies) encoded` — so the run exits `1` on the census alone. That is the whole reason the count is asserted, and it is the same pair `a_split_selection_packages_every_entry_in_the_one_embedded_artifact` uses on the artifact.

**The oracle's exact representability is derived, not asserted.** `X = [0.5, 1.25, -2.0, 3.25]`; `x * 2.0` is exact on a power of two and `+ 1.0` leaves the mapped contributors `[2.0, 3.5, -3.0, 7.5]`, all integer multiples of `0.5`. Every subset sum is an integer multiple of `0.5` with magnitude at most `13.0`, needing five significand bits against `f32`'s twenty-four, so no partial sum in any association rounds and every regrouping gives `10.0`.

**Four checks watched failing**, each applied to the working tree, run, and reverted; every one exited `1`. The oracle (`+ 1.0` → `- 1.0`); the readback (`buffer::read_into` removed, giving `[0.0]` against `[10.0]`, which is what places the sound run's value in the readback and distinguishes it from the reordered run's zero); the fused-contract perturbation, which fires the entry census, the shared-pairing check, and the reordering-observability check together — they are reported without short-circuiting precisely because it is the only perturbation that reaches the second and third; and disabling the reversal in `adapter::reverses_encode_order`, which produces `THE REORDERING WAS NOT OBSERVABLE`.

**Measurement — drift found by re-running the cited one-entry transcript.** `cargo run --release` and `cargo run --release -- --halt-after-commit` both still exit `0` at this base. The governed profile key moved from `tiler.metal.macos-apple9.msl4-0.f32.v1` to `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` across four rendered lines, from `5f1b7b1c` "Declare measured BF16 facts on the Metal profile"; the entry symbol is `tiler_kernel_a0f16709d95528ca`, which the README's 2026-08-02 note already tracked. Object length `3859`, four bindings, launch `4×1`, every stage and handover, the oracle's `[6.5, -5.0, -2.0, 1.0]`, and the quoted `1/1 entry(ies) encoded` reproduce byte for byte. The README records this as a dated line rather than rewriting the block, which is its established convention for a measurement.

**`docs/integration/frontends.md` swept.** "A multi-entry bundle produced by an expansion" moved from **Still outstanding** to **Landed** with both halves cited — the producer's `a_split_selection_packages_every_entry_in_the_one_embedded_artifact` and this spike — carrying the environment above and the one-shape boundary. The status paragraph gained the same clause, and the sweep's own preamble now states that items move in the change that discharges them and carry the date they moved.

## Boundaries this run does not cross

- The reordered run's value is `[0.0]` on this host and is **not** asserted: the reducing entry reads a `StorageModePrivate` allocation the mapping entry has not written, and Metal does not specify fresh private storage. Only the disagreement is checked.
- A partial-execution failure across two entries — one entry terminal-successful and the next not — is unwatched on hardware; provoking it means provoking a GPU fault. `crates/tiler-runtime/tests/adapter_route::a_halt_in_the_second_entry_is_a_post_commit_failure_naming_that_entry` remains where it is watched.
- One shape, one contract, one host, no timing. Nothing here is evidence about a three-entry bundle, more than one shared allocation, a wider reduction, or a plan carrying guarded schedule variants; and ADR 0086 still refuses this host, so the route was settled on producer-declared equality.

## Follow-up observed but not owned here

`docs/research/scheduling/two-dimensional-cooperative-staging-relation.md:202` cites `spikes/runtime/inline-dispatch/README.md:51, 87, 90, 91, 117, 125` by line number, and this change inserted content above every one of them. Those line numbers are part of a dated audit of that commit's tree, so rewriting them would falsify the audit rather than maintain it; the file is `research/scheduling` and outside this ticket's scopes in any case. Reported rather than edited.
