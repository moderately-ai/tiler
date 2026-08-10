---
id: realign-the-l2-derivation-operation-family-standing-to-the-family-state-table
title: Realign the L2 derivation's operation-family standing to the family-state table
status: in-progress
priority: p2
dependencies: []
related: [refresh-the-l2-derivation-operation-family-standing]
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: terra-l2-family-realign
lease_expires_at: 1786403723
---
## User-visible outcome

[The L2 derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md)'s *Rung today* column and its standing prose agree again with the roadmap's [family-state table](../docs/roadmap.md#family-state-and-reconsideration-triggers), with each re-stale cell's bound stated from current registry, fusion, and boundary evidence rather than from the 2026-08-06 close snapshot.

## Why this is a separate ticket

**Fact.** [`refresh-the-l2-derivation-operation-family-standing`](refresh-the-l2-derivation-operation-family-standing.md) closed on 2026-08-06 after aligning L2 to the then-current matrix, and its 2026-08-09 remainder note claimed no further correction was owned by that completed refresh. At the 2026-08-10 ticket audit base, that close condition fails again on Softmax bound prose, Slice rung, and Gather rung/key — landings that arrived after close (softmax realization law, slice fusion role, gather key + matrix row) without a live owner re-reading L2. Reopening the parent would erase a historically earned Outcome; this ticket owns the post-close re-drift only.

## The re-stale population (minimum; re-verified 2026-08-10)

Read each site against the roadmap family-state cell and the named authority before writing. Follow L2's existing quote-and-correct convention rather than silent rewrite.

| Site | Live L2 claim (stale) | Current authority |
| --- | --- | --- |
| Softmax *Rung today* / *What each family owes* Softmax bound | No `IndexRealizationLaw` registered; thirteen laws and this key not among them; request boundary refuses under `operation-set`; inverted claim that the roadmap still names two remaining prerequisites | `IndexRealizationLaw::staged_softmax_f32()` in the standard-provider law loop (fifteen laws total) at `crates/tiler-ir/src/semantic/registry.rs`; refusal measured as `UnsupportedCapability { rule: "missing-capability" }` in `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`; roadmap softmax cell records the three landings and the measured wall |
| Slice *Rung today* and slice/concatenate bound | `tiler::slice-f32@1` is **R4** for literal-offset; R5 awaits a fusion role; "Nothing lowers, fuses, or emits either" | Roadmap Sub-tensor selection row is **R5 for the F32 literal-offset family** (strided/symbolic stay R1); `FusionNumericalCapabilities::governed` inserts `slice_f32_op()` under `FusionOperationRole::CoordinateRelation` in `crates/tiler-compiler/src/fusion_legality.rs`. Drop the false "fuses" third; neither family claims R6 emission |
| Gather *Rung today*, *The Rung column restated*, *What each family owes*, disposition-vs-matrix | Still R1; not on the matrix as its own row; no gather key among registered keys | Roadmap Indirect gather row is **R4** for F32 source + `tiler::u32@1` index; `tiler::gather-f32@1` via `register_standard_gather` / `gather_f32_op` in `crates/tiler-ir/src/semantic/gather.rs`. Rewrite disposition so it does not deny the registered key while still recording any remaining access-class or R5+ walls the roadmap states (R5 needs fusion/IR admission path) |

**Also re-read (bound wording only; do not move rungs without family-state evidence):** Concatenate "lowers / fuses / emits" wording next to the Slice fix; RMS-norm bound against staged compile evidence (`a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit` and the roadmap normalization ceiling) without changing the R5 rung unless the matrix already has.

**Out of scope.** Roadmap family-state edits (softmax already refreshed by a related `done` ticket); L1 handoff table drift; crate or registry changes; support-matrix row movement.

## Closes when

A full read of L2's family table against the roadmap family-state table shows agreement on rung and bound for Softmax, Slice, and Gather at minimum, with dated corrections at every re-stale cell; Concatenate/RMS bound wording either matches current evidence or carries an explicit rechecked-unchanged note; this ticket is linked from the parent Outcome remainder note.

## Checks

`tkt lint`, `git diff --check`, and `tkt guard` against the true base. Documentation-only under `research/shapes` + shared `project/tickets`; no `crates/` path is in scope, so the workspace gate carries when those paths are untouched.
