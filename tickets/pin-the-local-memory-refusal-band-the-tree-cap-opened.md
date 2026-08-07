---
id: pin-the-local-memory-refusal-band-the-tree-cap-opened
title: Pin the local-memory refusal band the tree cap opened
status: todo
priority: p3
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256, activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, evidence-gap]
---
## User-visible outcome

A profile whose `local-memory-bytes` row sits between the balanced and capped staging requirements refuses the single-workgroup tree with its own typed feasibility diagnostic, and that refusal is observed rather than argued.

## Why this exists

**Fact — the band is real and untested.** [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md) gave the tree the largest admissible participant count not exceeding a measured 256, which is *wider* than the balanced choice on 2,561 of the contributor counts below 4,096. The tree stages one `f32` slot per participant, so the wider width costs more workgroup memory: at 8,192 contributors the capped 256 participants need **1,024 bytes** where the balanced 128 needed **512**. A profile declaring a `local-memory-bytes` row inside that interval therefore now refuses a tree it would previously have admitted.

**Fact — that refusal was deliberately left with the feasibility authority.** The cap does not narrow its width to fit a small target: doing so would let a cost preference decide legality, which is exactly the separation `WorkgroupTreeUnavailable` and the target rules exist to keep. The refusal surfaces as `Target { rule: "local-memory-bytes", region: …, required: 1024, available: … }` from `verify_schedule`. That is the intended behaviour, not a defect.

**Fact — no profile in the repository sits in the band, which is why nothing failed.** The authoritative Apple9 declaration carries `local_memory_bytes: 32_768` (`crates/tiler-build/src/metal_declaration.rs:255`), 8,192 participants' worth, far above the cap; the prototype baseline declares zero and refuses every tree at every width. So the behaviour in the band is currently established by reading the feasibility path rather than by observing it, and the `9415b450` landing says so in `capped_tree_partition`'s own docs rather than claiming coverage it does not have.

## What this ticket owes

- A test profile sized *between* the two requirements at a named contributor count — 8,192 with a row in `[512, 1024)` is the worked instance — driving `single_workgroup_tree_region` plus `verify_schedule` to the typed `local-memory-bytes` refusal, with `required` and `available` asserted rather than only the variant.
- The admitted neighbour beside it: the same shape at a row of at least 1,024 verifies, so the test separates the band from a blanket refusal.
- The refusal watched failing once before restoration, per the repository's standing discipline.

## Explicit non-goals

Do **not** narrow the participant count to fit a target — that is the separation the cap deliberately preserves, and re-litigating it needs new evidence rather than a convenient test. No change to `MEASURED_TREE_PARTICIPANT_CAP`, to the split's partition, or to selection.

## Closes when

The band's refusal and its admitted neighbour are both pinned at a named shape, the refusal has been observed failing, and `capped_tree_partition`'s doc paragraph stops describing the band as argued-not-observed.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the tree cap, from a consequence that ticket's worker found and recorded rather than smoothed over. It is a separate ticket because it is a bounded evidence gap in a different authority — target feasibility — from the partition rule that exposed it.
