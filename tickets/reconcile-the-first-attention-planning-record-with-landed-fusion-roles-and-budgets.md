---
id: reconcile-the-first-attention-planning-record-with-landed-fusion-roles-and-budgets
title: Reconcile the first attention planning record with landed fusion roles and budgets
status: todo
priority: p2
dependencies: []
related: [plan-the-materialized-attention-decomposition, implement-general-dag-partitioning, plan-the-recomputing-attention-decomposition, realize-the-attention-contractions-on-metal]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [docs, attention, doc-drift, planning]
---
## User-visible outcome

The durable L4 attention design tells a future planner what the repository can express now, rather than using its 2026-07-31 role population and search budgets as current authority.

## Fact audit at `7a156e6530b2f9a90501dc0230a9a3bf53bbfffb`

- **False live premise — no fusion roles.** `docs/research/program-planning/first-attention-program-vertical.md`, anchors `The plan that exists today — no fusion role` and `No fusion role exists for any family the block needs`, still derives planning and maturity from a role-less block. `crates/tiler-compiler/src/fusion_legality.rs`, anchor `The table below is the complete set of families the governed provider declares a role for`, now registers RMS normalization, softmax, reindex, broadcast, concatenate, slice, and strict tensor contraction as well as the earlier families. The record must preserve what was true when designed and add a dated current correction; it must not erase why those admissions were filed.
- **Imprecise live refusal.** The record still says a contraction, softmax, or normalization occurrence with no registered role withholds the fused alternative. That fail-closed rule remains true in general, but those named governed families no longer exercise it. Recast it as the generic missing-capability refusal and separately state the block's current registered-role population.
- **False current maturity conclusion.** The final `The honest maturity claim` paragraph still assigns proposal-era rungs and concludes that every fused candidate refuses before cost. Re-derive every named family from the current support matrix and source; do not replace the old census with another remembered count.
- **Stale budget account in the owning implementation ticket, sourced from the same era.** `crates/tiler-compiler/src/request.rs`, anchors `region_members: 62`, `region_boundary_outputs: 3`, and `region_live_values: 80`, is the current shape-bound authority. `crates/tiler-compiler/src/region.rs`, anchors `RegionBudgetStop`, `CandidatesPerSeed`, and `Expansions`, proves that passing shape bounds does not guarantee the bounded search reaches one particular region. The record must distinguish shape admission, enumeration, reached rejection, and bounded search loss.
- **Verified surviving evidence.** The exact operation sequence, C1/B1 shapes, `n · 16 · T · S · 4` transient requirement, retained measurement table, and D-11's absence of a declared transient-memory budget remain evidence-bearing. Preserve them unless a full source/record reread finds a separate contradiction.

## Required work

Read the entire research record, current fusion-legality construction and consumers, current region budgets/search stops, current support matrix, and every ticket status the record presents as current. Add dated corrections following the record's convention. Keep historical statements explicitly historical; remove no measurement or elimination rationale merely because its implementation prerequisite later landed. Update the planning ticket only if this full reread discovers another false live premise not already repaired there.

## Non-goals

No compiler, IR, schedule, target-profile, identity, or support-matrix change. Do not claim that registered roles alone make a complete attention plan executable, and do not decide D-11 or select between the materialized and recomputing decompositions.

## Closes when

Every current-looking role, budget, maturity, and reachability statement in the complete record is classified and corrected; the historical design remains readable; all source anchors are source-safe; `tkt lint`, `make citations`, `git diff --check`, and the exact-base guard pass.
