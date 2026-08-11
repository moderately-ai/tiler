---
id: pin-the-local-memory-refusal-band-the-tree-cap-opened
title: Pin the local-memory refusal band the tree cap opened
status: in-progress
priority: p3
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256, activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, evidence-gap]
claimed_from: todo
assignee: terra-local-memory-band
lease_expires_at: 1786423430
---
## User-visible outcome

A profile whose `local-memory-bytes` row sits between the balanced and capped staging requirements refuses the single-workgroup tree with its own typed feasibility diagnostic, and that refusal is observed rather than argued.

## Why this exists

**Fact — the band is real and untested.** [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md) now gives the tree the admissible participant count **nearest** the measured 256, ties narrower. It differs from the balanced choice on **2,350** contributor counts below 4,096, as pinned by `the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs`; the original 2,561 count and “largest not exceeding” description are stale. The tree stages one `f32` slot per participant, so the wider width costs more workgroup memory: at 8,192 contributors the capped 256 participants need **1,024 bytes** where the balanced 128 needed **512**. A profile declaring a `local-memory-bytes` row inside that interval therefore now refuses a tree it would previously have admitted.

**Fact — that refusal was deliberately left with the feasibility authority.** The cap does not narrow its width to fit a small target: doing so would let a cost preference decide legality, which is exactly the separation `WorkgroupTreeUnavailable` and the target rules exist to keep. The refusal surfaces as `Target { rule: "local-memory-bytes", region: …, required: 1024, available: … }` from `verify_schedule`. That is the intended behaviour, not a defect.

**Fact — no production profile in the repository sits in the band, which is why nothing failed.** The authoritative Apple9 declaration carries 32,768 local-memory bytes, far above the worked requirement; the prototype baseline declares zero and refuses every tree at every width. Cite the constructor/declaration symbols rather than the stale line number. The compiler's test-profile helper can state an intermediate row without changing public target authority, so this is a bounded regression rather than a new profile feature.

## What this ticket owes

- A test profile sized *between* the two requirements at a named contributor count — 8,192 with **512 available** against **1,024 required** is the worked instance — driving `single_workgroup_tree_region` plus `verify_schedule` to the typed `local-memory-bytes` refusal, with `required` and `available` asserted rather than only the variant.
- The admitted neighbour beside it: the same shape at a row of at least 1,024 verifies, so the test separates the band from a blanket refusal.
- The refusal watched failing once before restoration, per the repository's standing discipline.

## Explicit non-goals

Do **not** narrow the participant count to fit a target — that is the separation the cap deliberately preserves, and re-litigating it needs new evidence rather than a convenient test. No change to `MEASURED_TREE_PARTICIPANT_CAP`, to the split's partition, or to selection.

## Closes when

The band's refusal and its admitted neighbour are both pinned at 8,192 contributors, the refusal has been observed failing through a subject perturbation, and `capped_tree_partition`'s doc paragraph cites the synthetic regression rather than describing the band as reading-only evidence.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the tree cap, from a consequence that ticket's worker found and recorded rather than smoothed over. It is a separate ticket because it is a bounded evidence gap in a different authority — target feasibility — from the partition rule that exposed it.
