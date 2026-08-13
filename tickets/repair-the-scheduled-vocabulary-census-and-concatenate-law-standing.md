---
id: repair-the-scheduled-vocabulary-census-and-concatenate-law-standing
title: Repair the scheduled-vocabulary census and concatenate-law standing
status: review
priority: p1
dependencies: [admit-the-concatenate-family-into-the-scheduled-region-vocabulary]
related: [admit-the-structural-families-into-the-scheduled-region-vocabulary, accept-the-partitioned-concatenate-realization-law]
scopes: [implementation/ir, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, correction, scheduling, concatenate]
claimed_from: todo
assignee: worker-repair-scheduled-census
lease_expires_at: 1786636350
---
## Outcome

Source and compiler contracts state the current scheduled-vocabulary population truthfully before concatenate implementation relies on them. The accepted `PartitionedConcatenate` law no longer claims to await acceptance, and the optimizer census names the actual currently admitted and refused families with their current reasons.

## Facts to re-read at the claimed base

- Reindex and broadcast now have scheduled logical-access forms and request paths.
- Slice now has an accepted index-realization law and governed lowering but remains unplanned at a later physical boundary.
- Concatenate has an accepted multi-root index law and remains unplanned because no scheduled/kernel partitioned-copy program exists.

These statements are stale until the worker reads the full law, policy table, request recognizers, governed capability population, optimizer contract, acceptance tickets, and their tests, then reports a per-Fact verdict.

## Fact audit — 2026-08-13 at `e2071549`

Re-read at this base: `IndexRealizationLaw` (`crates/tiler-ir/src/index/law.rs`), `LogicalAccess` (`crates/tiler-ir/src/schedule/model.rs`), `fn recognize_structural_read` / `fn elementwise_family` / `fn recognize_staged_family` / `fn materializes_its_result` (`crates/tiler-compiler/src/request.rs`), `fn governed_index_access_capabilities` and `GOVERNED_INDEX_ACCESS_CAPABILITIES` (`crates/tiler-compiler/src/governed.rs`), `const UNPLANNED_OPERATIONS` (`crates/tiler-compiler/src/policy.rs`), `FusionNumericalCapabilities::governed` (`crates/tiler-compiler/src/fusion_legality.rs`), `docs/compiler/optimizer.md`, `accept-the-partitioned-concatenate-realization-law`, `accept-the-literal-offset-slice-realization-law`, `accept-the-softmax-realization-law`, and `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`.

- **Fact 1 — verified.** Reindex and broadcast have scheduled logical-access forms and request paths. `LogicalAccess` carries `ReindexBijection` and `BroadcastReplication`. `recognize_structural_read` admits exactly those two keys as mapped reads. `a_broadcast_widening_a_declared_weight_compiles_as_a_replication_relation` is the positive compile pin.

- **Fact 2 — imprecise.** Slice has the accepted `IndexRealizationLaw::Slice` (`Accepted public surface`, 2026-08-11) and one `GovernedSliceF32` lowering. It remains unplanned, but the wall is not "a later physical boundary": `LogicalAccess` has no selection or window map, `recognize_structural_read` does not name `tiler::slice-f32@1`, and `const UNPLANNED_OPERATIONS` still records the request-boundary `operation-set` refusal because the region vocabulary cannot spell its access relation. That is the same class reindex and broadcast used to occupy.

- **Fact 3 — verified, with stale source standing.** Concatenate has the accepted multi-root `IndexRealizationLaw::PartitionedConcatenate` (Tom accepted 2026-08-07, no exclusion) and seven arity lowerings. It remains unplanned because no scheduled or kernel partitioned-copy program exists; `const UNPLANNED_OPERATIONS` states that reason. At this base the law variant still said it was a labelled draft awaiting that already-completed acceptance.

Current `operation-set` wall population among registered families that reach a recognized-arithmetic walk: `tiler::slice-f32@1` (missing `LogicalAccess` selection map) and `tiler::concatenate-f32@1` (missing partitioned-copy schedule/kernel). Softmax is recognized by the law-derived staged arm and then refuses under `missing-capability`. Gather and the strict-affine dequantize do not reach this rule on the ordinary path (`dtype`). Governed index-access capabilities are twenty-one: fourteen fixed-signature families plus one per concatenate arity `2..=8`.

## Required delivery

Repair the complete census and reasons rather than changing one number. Retire only stale standing labels; do not alter public types, recognition, policy, identity, or behavior. Use searchable source anchors and perturb any census check by changing its subject, not its expected count.

## Closes when

The source standing and optimizer census match one independently reproduced current population, citations/lint are green, and the delta changes no behavior or identity bytes.

## Outcome — 2026-08-13

Source and compiler contracts now state the current scheduled-vocabulary population. `IndexRealizationLaw::PartitionedConcatenate` and the sibling accepted `StagedSoftmaxF32` no longer claim to await acceptance; both use the same `Accepted public surface` form as `Slice`. The optimizer census names the currently admitted structural families (`ReindexBijection` / `BroadcastReplication` via `recognize_structural_read`) and the two families still refused under `operation-set` (`tiler::slice-f32@1` for a missing selection `LogicalAccess`; `tiler::concatenate-f32@1` for a missing partitioned-copy program), with the twenty-one-capability governed lowering count derived from fourteen fixed-signature families plus `MIN_CONCATENATE_OPERANDS..=MAX_CONCATENATE_OPERANDS`. Retired three-family / no-reindex-map / twenty-capability wording is quoted only inside the dated correction.

This advances no support-matrix row; it corrects standing labels. No public type, recognition, policy, identity byte, or behavior moved.
