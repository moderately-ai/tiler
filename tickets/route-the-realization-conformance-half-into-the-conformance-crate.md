---
id: route-the-realization-conformance-half-into-the-conformance-crate
title: Route the realization-conformance half into the conformance crate
status: todo
priority: p2
dependencies: [carry-the-device-executed-value-proof-into-the-conformance-crate]
related: [retain-contraction-conformance-evidence, publish-an-l3-contraction-cell-through-the-accepted-route, survey-what-belongs-in-the-conformance-crate]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, contraction, migration]
---
## User-visible outcome

All six L3 correctness cells' retained `result_sha256` values are compared against **executed** device results inside `crates/tiler-conformance`, on a matching host row and declining with a named difference on any other — so the spike's retained record becomes a gate rather than a document.

## Why this is separate from its parent ticket

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md).

[`carry-the-device-executed-value-proof-into-the-conformance-crate`](carry-the-device-executed-value-proof-into-the-conformance-crate.md) *relocates* what already runs, which is one cell. This ticket *widens* it to the profile, and the widening is the part that needs its own cost statement.

**Fact — one of six is compared today.** [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md) closed on `w_decode_kv` alone and states its own boundary: "This is one cell of six and one host row." Its non-goals name "the remaining five cells, which follow the first at no architectural cost."

**Fact — the grid-axis bound no longer blocks the rest.** [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md) moved the row to a measured 268,435,456 and proved by compiling that all six cells reach a selected physical plan (`tiler_build::metal_plan::tests::the_measured_grid_axis_admits_every_l3_contraction_cell`).

## What this owns, and what it leaves where it is

[`retain-contraction-conformance-evidence`](retain-contraction-conformance-evidence.md) proposes two halves and holds four scopes because neither half had a home. This ticket takes **one** of them:

- **Realization conformance — this ticket.** The six cells' `result_sha256` against the *executed* result, valid only where the environment row matches, announcing the difference and declining to compare where it does not. That is verbatim [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) item 1's hard requirement, and `implementation/conformance` is now the scope that can carry it.
- **Reference conformance — stays.** The eight adversarial cases against the reference evaluator are target-independent and already live in `crates/tiler-reference/tests/contraction_conformance.rs`, whose own header states "A pass here is evidence about the semantic contract and the host reference evaluator" and disclaims any schedule, kernel, or device. Moving them would be the layer-local migration the crate's third anti-goal refuses.

**The coordinator should narrow `retain-contraction-conformance-evidence` to its reference half when this lands**, rather than leaving two owners for one deliverable. Do not close it from here.

## Cost, so the choice is stated rather than discovered

The comparison is a device dispatch per cell, not a host fold, so `tiler-reference`'s measured 1.1e9-step host cost does not apply. `w_decode_kv` folds 1,048,576 steps on the GPU; the largest two cells fold 402,653,184 each. Measure the wall clock per cell on the qualified host and state whether the whole profile runs on every gate run or whether the four prefill cells sit behind `#[ignore]` with a recorded invocation — the shape `crates/tiler-reference/tests/contraction_profile_cells.rs` already uses. Either answer is defensible; picking one silently is not.

## Required evidence

- Operands generated from the probe's own `SplitMix64` stream (`WORKLOAD_SEED = 0x5445_524D`, right seed `seed ^ 0xA5A5_A5A5_A5A5_A5A5`, values `m * 2^-24`), so each digest is computed over the bytes the device consumed. Read it from `crates/tiler-compiler/src/governed/contraction_conformance.rs` rather than re-deriving it.
- Every environment field compared against `spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/environment.tsv` **before** any comparison, with a non-matching row producing a named unavailable outcome rather than a skip or a pass.
- Each digest check watched refusing before it is trusted — a comparison against a 64-character constant passes trivially if the bytes never reach it. The `RIGHT_SEED_MASK` bit-flip perturbation in `publish-an-l3-contraction-cell-through-the-accepted-route` is the recorded technique.
- The measurement boundary recorded: host, OS build, Xcode, SDK, offline compiler, GPU, family.

## Closes when

All six cells are compared against executed results inside `crates/tiler-conformance`, the per-cell cost is measured and the run/ignore choice is stated with it, every comparison was watched refusing under a deliberate perturbation, a non-matching host row is observed declining by name, and the reference half is confirmed still resident in `crates/tiler-reference`.
