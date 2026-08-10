---
id: realign-the-l2-derivation-operation-family-standing-to-the-family-state-table
title: Realign the L2 derivation's operation-family standing to the family-state table
status: done
priority: p2
dependencies: []
related: [refresh-the-l2-derivation-operation-family-standing]
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

[The L2 derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md)'s *Rung today* column and its standing prose agree again with the roadmap's [family-state table](../docs/roadmap.md#family-state-and-reconsideration-triggers), with each re-stale cell's bound stated from current registry, fusion, and boundary evidence rather than from the 2026-08-06 close snapshot.

## Why this is a separate ticket

**Fact.** [`refresh-the-l2-derivation-operation-family-standing`](refresh-the-l2-derivation-operation-family-standing.md) closed on 2026-08-06 after aligning L2 to the then-current matrix, and its 2026-08-09 remainder note claimed no further correction was owned by that completed refresh. Reopening the parent would erase a historically earned Outcome; this ticket owns post-close re-drift only.

**Correction — 2026-08-10 source audit at `70fe80509456f6e52a90c35674ca7dcda6439a24`.** The original second sentence said the close condition failed again on Softmax, Slice, and Gather. That population is false at this base: Softmax and Slice were corrected before this branch opened, while Gather remains stale. The purpose narrows without changing — align the one remaining stale family standing and record the other named bounds as rechecked.

## Source audit before edits — 2026-08-10

| Ticket Fact | Verdict | Source-safe evidence |
| --- | --- | --- |
| The parent earned its 2026-08-06 close, then claimed no remainder on 2026-08-09 | **Verified** | Parent anchors `## Outcome — 2026-08-06` and `## Current remainder correction — 2026-08-09`; its later `## Current remainder correction — 2026-08-10` already links this ticket rather than reopening the completed work |
| Softmax is re-stale in L2 | **False** | L2 anchor `The Softmax rung cell formerly said no realization law was registered` records `StagedSoftmaxF32`, occurrence recognition, and the later `missing-capability` wall; the live Softmax row states the same bound |
| Slice is re-stale in L2 | **False** | L2 anchor `the 2026-08-06 correction above placed the literal-offset Slice at R4` corrects the literal-offset family to R5 under `CoordinateRelation`, with strided and symbolic relations at R1 and `operation-set` still the request wall |
| Gather is re-stale in L2 | **Verified** | L2 anchors `Indirect gather (tied embedding lookup)`, `The indirect gather is still R1`, and the `Indirect gather` obligations row still deny the registered key and normative reference. Roadmap anchor `Indirect gather: Gather over a tensor-data-derived index` places the F32/U32 family at R4; source anchors `register_standard_gather`, `gather_f32_op`, `The complete normative definition of tiler::gather-f32@1`, and `GatherF32Reference` discharge semantic and reference admission |
| Gather's compiler wall can be summarized as one observed `operation-set` refusal | **Imprecise** | Accepted ADR 0107 and the standard registry prove no realization law, lowering capability, fusion role, or executable plan. Ordinary compilation currently encounters the U32 dispatch/arithmetic gates before the later recognition wall, and [`pin-the-gather-request-boundary-refusal-with-a-test`](pin-the-gather-request-boundary-refusal-with-a-test.md) remains `todo`; this repair states the source-proven no-plan boundary without inventing an end-to-end observed diagnostic |
| Concatenate and RMS normalization need a rung or bound correction | **Verified unchanged** | L2 and roadmap anchors `Sequence extension: Concatenate` agree on R5 with no backend emission; L2 and roadmap anchors `Normalization: tiler::rms-norm-f32@1` agree on R5, structured-kernel-interpreter bit agreement, and no compiler-derived backend emission |

## The re-stale population (re-verified 2026-08-10)

Read each site against the roadmap family-state cell and the named authority before writing. Follow L2's existing quote-and-correct convention rather than silent rewrite.

| Site | Live L2 claim (stale) | Current authority |
| --- | --- | --- |
| Gather *Rung today*, *The Rung column restated*, *What each family owes*, disposition-vs-matrix | Still R1; not on the matrix as its own row; no gather key among registered keys | Roadmap Indirect gather row is **R4** for F32 source + `tiler::u32@1` index; `tiler::gather-f32@1` via `register_standard_gather` / `gather_f32_op` in `crates/tiler-ir/src/semantic/gather.rs`. Rewrite disposition so it does not deny the registered key while still recording any remaining access-class or R5+ walls the roadmap states (R5 needs fusion/IR admission path) |

**Rechecked and unchanged:** Softmax's R5 registered-law / `missing-capability` bound; Slice's R5 `CoordinateRelation` bound and its `operation-set` wall; Concatenate's R5 no-emission bound; RMS normalization's R5 interpreter / no-backend-emission bound. These are audit evidence, not work population.

**Out of scope.** Roadmap family-state edits (softmax already refreshed by a related `done` ticket); L1 handoff table drift; crate or registry changes; support-matrix row movement.

## Closes when

A full read of L2's family table against the roadmap family-state table shows agreement on rung and bound for Gather, with dated corrections at every re-stale Gather claim; Softmax, Slice, Concatenate, and RMS normalization remain unchanged after explicit recheck. The parent Outcome remainder already links this ticket at anchor `## Current remainder correction — 2026-08-10`.

## Checks

`tkt lint`, `git diff --check`, and `tkt guard` against the true base. Documentation-only under `research/shapes` + shared `project/tickets`; no `crates/` path is in scope, so the workspace gate carries when those paths are untouched.

## Outcome — 2026-08-10

The source audit narrowed the live repair from Softmax, Slice, and Gather to Gather alone. The L2 record now places `tiler::gather-f32@1` at **R4** for an F32 source and `tiler::u32@1` indices, with the semantic/reference evidence and the deliberately unchanged lower boundary stated separately: no realization law, admitted indirect `LogicalAccess`, lowering capability, fusion role, U32 storage carrier, backend realization, or executable plan.

The correction reaches every live standing site found by the full read: the occurrence-level expressibility inference, the family table's disposition/rung/derivation, *The Rung column restated*, *What each family owes*, and the capability-ticket outcome. The retired R1/no-key/no-normative-reference wording remains searchable only as dated quotation or immediately corrected history. The parent links this ticket under its `## Current remainder correction — 2026-08-10` anchor and now carries a follow-up correction narrowing its original three-family dispatch snapshot to Gather.

**Rechecked unchanged:** Softmax is R5 with `StagedSoftmaxF32` recognized and `missing-capability` the measured wall; Slice is R5 for the F32 literal-offset family under `CoordinateRelation`, with strided/symbolic forms at R1 and `operation-set` still the request wall; Concatenate is R5 with no backend emission; RMS normalization is R5 with structured-kernel-interpreter bit agreement and no compiler-derived backend emission.

Only `docs/research/shapes/transformer-operation-and-shape-surface.md`, this ticket, and the completed parent ticket changed. No gate-invalidating path under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh` changed, so the current green workspace `make full` carries from the exact base. This delta reruns `make citations`, `tkt lint --format json`, `git diff --check`, and exact-base `tkt guard`.
