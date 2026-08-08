---
id: correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical
title: Correct the reassociation-Unknown claim a repair block introduced in the BF16 vertical
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## A correction introduced a new false claim, which is the pattern this repository keeps hitting

`crates/tiler-conformance/src/bf16_vertical.rs`'s module header, in the bullet **"Reassociation is withheld rather than proved"**, states that `BF16_FACT_REASSOCIATION_PERMITTED` is `false` and "the question stays open at the operation vocabulary, so a contract that *permits* regrouping leaves the obligation `Unknown`".

**False for the region it is written about.** Coordinator-verified: `push_reduction_obligations` in `crates/tiler-compiler/src/fusion_legality.rs` discharges `ReductionReassociation` as **`SoundProof`** when `!has_reduction || reassociation == Forbidden`. The BF16 vertical is `(x * 1.5) + 0.0` — **pointwise, no reduction** — so `!has_reduction` short-circuits to `SoundProof` *regardless of what the contract permits*. The `Unknown { "unproven-reassociation" }` branch requires a reduction **and** a permitting contract, which is the surviving contraction wall and a different region.

**How it got here, and why that matters more than the sentence.** This text landed on 2026-08-07 inside a *repair block* correcting the coordinator's own earlier over-general claim that reassociation is "withheld as `Unknown`". The repair fixed the framing and then restated a narrower version of the same error. That is the third time this session a correction has introduced a fresh false claim, and it is exactly the failure `AGENTS.md` now warns about: **never restate a false Fact in new words.**

Found by the worker on `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents`, which held `contracts/*` and could not reach `crates/`.

## What is true, stated so the repair does not overshoot a third time

For this vertical's pointwise region the obligation is discharged **`SoundProof`, vacuously** — nothing in the region raises a reduction order to preserve. Say *that*, and keep the honest residue separate: a vacuous discharge is not evidence the reductions are correct, only that none were present. `BF16_FACT_REASSOCIATION_PERMITTED` being `false` is a true and separate fact about the operation vocabulary; it is not what decides this obligation.

**Do not write "records `Unknown`"** in any form for a reduction-free region.

## Closes when

The bullet states the vacuous `SoundProof` discharge with its correct ground; no reduction-free region is described as leaving reassociation `Unknown`; the surviving contraction wall is named as the place the `Unknown` branch is actually reached; and the change is verified by reading `push_reduction_obligations` rather than by citing this ticket.
