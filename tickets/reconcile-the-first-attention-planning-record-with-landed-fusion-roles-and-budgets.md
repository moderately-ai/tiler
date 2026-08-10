---
id: reconcile-the-first-attention-planning-record-with-landed-fusion-roles-and-budgets
title: Reconcile the first attention planning record with landed fusion roles and budgets
status: in-progress
priority: p2
dependencies: []
related: [plan-the-materialized-attention-decomposition, implement-general-dag-partitioning, plan-the-recomputing-attention-decomposition, realize-the-attention-contractions-on-metal, correct-the-records-the-derived-region-shape-budgets-falsify]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [docs, attention, doc-drift, planning]
claimed_from: todo
assignee: sol-attention-record
lease_expires_at: 1786388862
---
## User-visible outcome

The durable L4 attention design tells a future planner what the repository can express now, rather than using its 2026-07-31 role population and search budgets as current authority.

## Fact audit at `7a156e6530b2f9a90501dc0230a9a3bf53bbfffb`

- **False live premise — no fusion roles.** `docs/research/program-planning/first-attention-program-vertical.md`, anchors `The plan that exists today — no fusion role for any family the block needs` and `No fusion role exists for any family the block needs`, still derives planning and maturity from a role-less block. `crates/tiler-compiler/src/fusion_legality.rs`, anchor `complete set of families the governed provider`, now registers RMS normalization, softmax, reindex, broadcast, concatenate, slice, and strict tensor contraction as well as the earlier families. The record must preserve what was true when designed and add a dated current correction; it must not erase why those admissions were filed.
- **Imprecise live refusal.** The record still says a contraction, softmax, or normalization occurrence with no registered role withholds the fused alternative. That fail-closed rule remains true in general, but those named governed families no longer exercise it. Recast it as the generic missing-capability refusal and separately state the block's current registered-role population.
- **False current maturity conclusion.** The final `The honest maturity claim` paragraph still assigns proposal-era rungs and concludes that every fused candidate refuses before cost. Re-derive every named family from the current support matrix and source; do not replace the old census with another remembered count.
- **Residual budget and search distinction on the L4 record (shape budgets already corrected).** **Correction — 2026-08-10.** The prior packaging called this a "stale budget account in the owning implementation ticket." That overstated residual and mislabeled the locus: `plan-the-materialized-attention-decomposition` already embeds the 62/3/80 fact correction, and the L4 record already carries the 2026-08-07 dated correction from `correct-the-records-the-derived-region-shape-budgets-falsify` (status `done`) that retires the live 32-member stop story, states `region_members` `62` / `region_boundary_outputs` `3` / `region_live_values` `80`, and marks whole-block enumerator reachability unmeasured. `crates/tiler-compiler/src/request.rs` anchors `region_members: 62`, `region_boundary_outputs: 3`, and `region_live_values: 80` remain the current shape-bound authority. Residual work on the L4 record is narrower: (i) ensure no current-looking claim reasserts a 32-member stop; (ii) name search stops `CandidatesPerSeed` and `Expansions` (`crates/tiler-compiler/src/region.rs`, with `RegionBudgetStop`) as distinct from shape admission; (iii) keep the reached-rejection versus bounded-search-loss distinction. Passing shape bounds does not guarantee the bounded search reaches one particular region.
- **False live premise — zero-synchronization schedule profile.** The record still presents barrier-free zero-sync as live Fact: Softmax table row (c), elimination "Threadgroup-cooperative softmax", typed refusal naming the `zero-synchronization schedule profile`, and "synchronization in this program is dispatch ordering and nothing else". Schedule IR now admits `ReductionTopology::CooperativeWorkgroup`, `cooperative_synchronization_requirement`, and `SynchronizationPoint` (`crates/tiler-ir/src/schedule/model.rs`). Preserve the historical design constraint; add a dated correction that recasts zero-sync as proposal-era and names the landed cooperative vocabulary. `plan-the-materialized-attention-decomposition` already fact-corrects this in its own prose and treats residual L4 zero-sync elimination rows as this ticket's problem.
- **Incomplete D-C multi-dimension refusal ground.** Elimination row D-C still says the online single pass "Consumes distributivity … and additionally reassociation." ADR 0101 / `docs/numerical-semantics.md` require that the online-softmax rescaling fold names **both** distributivity and elementary-function identity; multi-dimension refusals must name every missing dimension. The candidate still correctly dies (ADR 0095's decline of distributivity alone suffices); the live ground text is incomplete and must be corrected without reopening D-C.
- **False live reachability — only D-A plan reachable today.** Ladder and elimination language that treats the fully unfused four-tensor plan as `the only D-A plan reachable today` is fusion-role reachability drift: once contraction and elementwise roles exist, missing roles no longer force every fused D-A candidate to `Unknown`. Preserve that a complete attention plan remains undelivered and that StorageHandoff n=1 is unimplemented; do not equate registered roles with an executable plan (Non-goals).
- **Verified surviving evidence.** The exact operation sequence, C1/B1 shapes, `n · 16 · T · S · 4` transient requirement, retained measurement table (including B1-d unfused `18,329,108,488`), and D-11's absence of a declared transient-memory budget remain evidence-bearing. Preserve them unless a full source/record reread finds a separate contradiction.

## Required work

Read the entire research record, current fusion-legality construction and consumers, current region budgets/search stops, current schedule synchronization vocabulary, current numerical-semantics multi-dimension refusal text, current support matrix, and every ticket status the record presents as current. Add dated corrections following the record's convention covering fusion roles, maturity, typed refusals, residual budget/search distinction, zero-synchronization vs cooperative schedule vocabulary, D-C multi-dimension refusal ground, and "only D-A plan reachable today" reachability language. Keep historical statements explicitly historical; remove no measurement or elimination rationale merely because its implementation prerequisite later landed. Update the planning ticket only if this full reread discovers another false live premise not already repaired there (roles, shape budgets, and zero-sync are already repaired on `plan-the-materialized-attention-decomposition`; the L4 record is the remaining locus).

## Non-goals

No compiler, IR, schedule, target-profile, identity, or support-matrix change. Do not claim that registered roles alone make a complete attention plan executable, and do not decide D-11 or select between the materialized and recomputing decompositions.

## Closes when

Every current-looking role, budget, maturity, reachability, schedule-synchronization, and multi-dimension refusal statement in the complete record is classified and corrected (including zero-synchronization live premises versus landed CooperativeWorkgroup/SynchronizationPoint vocabulary, incomplete D-C online-softmax ground, and "only D-A plan reachable today" fusion-role reachability drift); the historical design remains readable; all source anchors are source-safe; `tkt lint`, `make citations`, `git diff --check`, and the exact-base guard pass.
