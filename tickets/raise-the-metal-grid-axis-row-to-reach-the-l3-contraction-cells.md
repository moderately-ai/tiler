---
id: raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells
title: Raise the Metal grid-axis row so the L3 contraction cells are reachable
status: done
priority: p2
dependencies: [establish-an-upper-bound-authority-for-the-metal-grid-axis-row]
related: [integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/build, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, contraction, target-profiles]
---
## User-visible outcome

The L3 profile's own contraction cells become reachable through the accepted AOT and runtime route, so the spike's retained `result_sha256` values at those cells can be taken as the independent cross-check they were retained to be.

## Why they were not reachable when this ticket was filed

Superseded by the outcome below: the row is now a measured 268,435,456 and every refusal recorded here has been observed gone. Kept because it is the derivation that fixed the bound at exactly four, and a reader repairing this area needs it.

**Fact, measured 2026-08-02.** [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) ran one contraction of the profile's index structure `td,od->to` end to end through the accepted route and bit-compared it against the reference at `2x2x3`. It could not run the profile's own cells: the `direct` realization launches one invocation per output element, every correctness cell publishes at least 1,024 of them (`w_decode_kv` is `M=1, N=1024`), and the declared profile's `grid_axis_threads` row is `4`.

Compiling `w_decode_kv` against `BoundMetalCompileDeclaration::first_macos_apple9` resolves rule `target.grid-axis`, predicate `grid-axis`, `Rejected("target-infeasible")`, `required: Threads(1024), available: Threads(4)`, and the target outcome is `NoFeasiblePlan` before any plan composes. `2x3x3` refuses identically at `required: Threads(6)`, which is what fixes the bound at exactly four rather than at "large".

**Fact — the row is a compile guarantee, not a measured maximum.** `crates/tiler-build/src/metal_declaration.rs`'s `FIRST_MACOS_APPLE9` records it as "the macOS 26.5 SDK's `dispatchThreads:` contract proves extent 4 is representable and establishes no upper bound at all, so 4 is a deliberately conservative compile guarantee rather than a maximum." The spike's own `environment.tsv` reports `device_max_threads_per_threadgroup 1024` on the same Apple M4 Max, and the spike dispatched all six cells there — so the hardware is not the constraint and the ledger row is.

## What this ticket owes

- A named authority for a grid-axis extent above four, in the compile-profile authority ledger's own terms: an SDK sentence, a feature-table row, or a retained measurement with its exact procedure. A number chosen because it makes a cell compile is exactly what the ledger exists to refuse.
- The declaration row moved with that authority, and the descriptor identity recomputed — the profile key and canonical descriptor fold this row, so moving it moves every pinned identity derived from it. That is an identity-domain step: executed completely or not at all.
- Re-run of the contraction vertical at one L3 cell, with the retained `result_sha256` compared on a matching host row. This host already matches the spike's correctness row on every recorded field.

## Non-goals

The tiled realization, which [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) owns. Raising the row is about reaching the cells at all; which realization is fastest there is a separate question with its own measurements.

## Closes when

One L3 correctness cell runs through the accepted AOT and runtime route and its result matches the spike's retained `result_sha256` for that cell, or the ledger records why no authority admits an extent that reaches it.

## Outcome — 2026-08-05: the cells are reachable and proved so by compiling; the device comparison is split out

### This branch executes no identity-domain step, and that is the finding to read first

**Fact.** The row and its identity move landed with [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md) and are already in this ticket's base commit `561dfe0b`: `crates/tiler-build/src/metal_declaration.rs:225` reads `grid_axis_threads: 268_435_456`, declared through the profile's own `TargetCompileProfileMeasurementSource`. **Zero pinned identities move on this branch** — the descriptor stays 1,999 bytes, the standard Metal artifact identity stays `3f98afa59d9ef46999acc211f2153a7d194444f5be3d0dd946f4128b57674a69`, and its cache subject stays `8bca5e7825cdd1dc37da5135b0ea7d6dbd3e9ce1557097f2ee9e60e79fe23d07`. Re-stepping the row here would have been the same step twice from two bases, which the dependency edge exists to prevent. The first two bullets of "What this ticket owes" were therefore discharged before this branch started, exactly as the 2026-08-04 worker comment recorded.

### The reachability claim was an inference, and is now checked

**Measurement, 2026-08-05, Apple M4 Max / macOS 27.0 `26A5388g`.** `crates/tiler-build`'s new `metal_plan::tests::the_measured_grid_axis_admits_every_l3_contraction_cell` compiles all six L3 correctness cells — `w_decode_kv` 1x1024x1024, `w_vocab_slice` 1x8192x1024, `w_prefill_q` 10x2048x1024, `w_prefill_mlp_in` 128x3072x1024, `w_prefill_mlp_out` 128x1024x3072, `w_prefill_o` 128x1024x2048 — through the ordinary compiler entry point against `BoundMetalCompileDeclaration::first_macos_apple9`, and **every one reaches a selected physical plan.** So does `2x3x3`, the six-output shape this ticket recorded refusing at `required: Threads(6)`.

That is worth a test rather than an arithmetic remark: a bound admitting an extent and a plan *composing* at that extent are different claims, and only the second is what a cell needs. `cargo nextest run -p tiler-build -E 'test(the_measured_grid_axis_admits_every_l3_contraction_cell)'` reproduces it.

**Both halves proved able to fail.** Setting `FIRST_MACOS_APPLE9.grid_axis_threads` back to `4` makes the new test fail with `NoFeasiblePlan` and `the_standard_metal_path_publishes_its_recorded_identities` fail on the moved artifact identity; the row was restored from the index afterwards. The descriptor-length pin correctly does **not** move under that perturbation, because the bound is a fixed-width `u64` whose value moves no bytes — which is what the ledger already says and is a consistency check on that sentence rather than a gap. The test also carries its own boundary in both directions: `16,384 x 16,384` is exactly 268,435,456 output elements and composes, `16,384 x 16,385` refuses on `grid-axis` by name.

### The accepted route still runs green under the moved row

**Measurement, read-only, no files edited.** `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer cargo run -p tiler-prototype-compile -- --out <path>` published seven members against `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` (1,999 descriptor bytes), and `cargo run -p tiler-prototype-run -- --artifact <path>` dispatched them on the Apple M4 Max and exited 0: 30 cases across six serial-sum members, plus the `2x2x3` contraction's five operand cases agreeing bit for bit with the published reference. The identity move did not break the vertical.

Its closing line is now false, and it is not this ticket's to fix: `prototypes/serial-sum-run/src/proof.rs` prints "the L3 profile's own cells are refused by this profile's four-thread grid-axis row and are not published here". That site and its siblings belong to [`correct-the-four-thread-grid-rationales-the-measured-row-falsified`](correct-the-four-thread-grid-rationales-the-measured-row-falsified.md), which holds both prototype scopes.

### Why the third bullet is split out rather than delivered

**Fact — a scope boundary, not a feasibility one.** Dispatching a cell and digesting its executed bytes needs `prototypes/serial-sum-compile` (`implementation/metal-aot`) and `prototypes/serial-sum-run` (`implementation/runtime`). `implementation/runtime` is held exclusively by the live `bind-stage-coverage-to-index-refinement-identity`; file-level disjointness could not be verified against its actual branch diff, because `git diff --name-only $(git merge-base tkt/bind-stage-coverage-to-index-refinement-identity-r2 HEAD) tkt/bind-stage-coverage-to-index-refinement-identity-r2` is empty — that branch carries no commits, so it evidences nothing about which files it will touch. `crates/tiler-build` cannot substitute: it declares no `tiler-runtime` edge and creates no device.

**The cheaper route was eliminated on correctness, not on effort.** Repointing the existing `contraction` member from `2x2x3` at a cell would have stayed entirely inside the unheld `implementation/metal-aot`. It is refused because the published shape is load-bearing: `2x2` is what makes the two operand access relations separately observable, and the producer's own comment records that a `1 x N` result "would let a kernel that confused the two still agree". Every L3 cell is exactly that shape. Repointing would also drop the five adversarial numerical cases — including the `negative-zero-fold` counterexample — from the route. That trades discriminating coverage for a digest, which is a defect rather than an alternative.

The remainder is [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md), holding both prototype scopes and depending on this ticket.

### Boundary

One profile, one host row: macOS 27.0 `26A5388g` / Apple M4 Max / Xcode 26.6 `17F113` / SDK 26.5 `25F70`. Everything above is a **compile-phase** claim plus one green run of the existing route; **no L3 cell was dispatched and no retained `result_sha256` was compared**, so this ticket's own Closes-when is not met by this branch and is carried by the ticket named above. Nothing was timed.

### Scopes

No scope was added. The branch touches `crates/tiler-build` (`implementation/build`), `docs/research/target-profiles` (`research/target-profiles`), and `tickets/` (`project/tickets`, shared) and nothing else.
