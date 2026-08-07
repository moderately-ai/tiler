---
id: correct-the-records-the-derived-region-shape-budgets-falsify
title: Correct the records the derived region-shape budgets falsify
status: todo
priority: p2
dependencies: [rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets]
related: [derive-the-region-shape-budgets-from-the-declaration]
scopes: [contracts/foundation, contracts/artifacts, contracts/decisions, research/artifacts, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, budgets, identity]
---
## User-visible outcome

No governing contract, accepted decision, or research record still states that the compilation path refuses this family above thirty-two operations, or that `region_members` is 32, `region_boundary_outputs` 8, or `region_live_values` 64.

## Why this exists

[`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) made the three region-shape budgets derivations over the declaration on Tom's 2026-08-07 decision: `region_members` is `semantic_operations` (62), `region_live_values` is `semantic_values` (80), and `region_boundary_outputs` is the declared output count (3). The whole 33..=62 range now compiles as one whole-program region, and the first refusal for the chain family is `semantic_operations` at sixty-three.

That ticket held `implementation/compiler` and `contracts/optimizer`, and it corrected [`docs/compiler/optimizer.md`](../docs/compiler/optimizer.md)'s budget list and derivation in the same commit. Every site below is in a scope it did not hold.

## The drifted sites, each with its scope and the claim that is now false

Located by `grep -rn "region_members\|region_live_values\|region_boundary_outputs" docs/ spikes/` on the branch that moved them; each was read at its line before being listed.

**`contracts/foundation` — [`docs/ir.md`](../docs/ir.md) line 1130.** "The path refuses this family above thirty-two operations with `BudgetExhausted` on `region_members`, a bound on one region's admissible shape rather than on program size." False: the path compiles the family to sixty-two. The same sentence's "measured on the ordinary compilation path over the widened 2..=32 ladder with residual zero at all thirty-one points" is a measurement claim whose domain moves with the re-run.

**`contracts/artifacts` — [`docs/artifact-abi.md`](../docs/artifact-abi.md) line 247.** The same two claims in the same words, plus "At the governed `semantic_operations` budget of 62 that is 219,277 bytes" — which stops being an extrapolation once 62 is inside the ladder's domain, and should be restated as a measurement or corrected.

**`contracts/decisions` — [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) line 20.** "The ladder now runs 2..=32; 33..=62 refuse `BudgetExhausted` on `region_members` — a declared bound on one region's admissible shape, whose sizing against `semantic_operations` is a parked decision — so the governed maximum of 62 remains unreachable for this family". Every clause is now false: the sizing was decided, the range compiles, and 62 is reachable. The record's own header measurement paragraph names the domain and the walls.

**`research/artifacts` — [`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) lines 165, 169, 211, 241.** Four separate restatements: "62 is not compilable at all", "the compilation path refuses everything above thirty-two operations", "The remaining wall is `BudgetExhausted` on `region_members` at thirty-three, a declared region-shape bound whose sizing is a parked decision", and "**No program above 32 operations compiles at all.** The 62-operation figure is arithmetic over a fit, not an observation." The last is the one that inverts most sharply: the 62-operation figure becomes an observation.

**`research/program-planning` — [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) line 154.** "The ladder now runs 2..=32, and 33..=62 refuse `BudgetExhausted` on `region_members` (32) … so this record's decoder-layer program at ≥ 51 occurrences is **still not compilable**, now for a declared-bound reason rather than a truncated search." The size reason is gone; the recognizer's refusal — `select_supported_strategy` — is what still blocks the layer, and that is a different statement with a different remedy.

**`research/program-planning` — [`docs/research/program-planning/first-attention-program-vertical.md`](../docs/research/program-planning/first-attention-program-vertical.md) line 186.** "`region_members` bounds a region at 32 semantic occurrences … gives at least forty-four occurrences, and … `EnumerateRegionCandidates` therefore abandons that growth path with a typed budget stop and never decides the block's legality." The arithmetic that made forty-four exceed the bound no longer holds against 62, so this bullet's *conclusion about which wall stops the attention block* has to be re-derived rather than reworded — the block may now reach a different refusal, and naming the wrong one is worse than naming none.

**`research/program-planning` — the spike itself.** Owned separately by [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md), which this ticket depends on: several corrections above quote its ladder extent and its fit, and stating a new extent before the re-run measures one would replace a stale number with an invented one.

## What this ticket owes

Each site above corrected against source rather than against this ticket's summary, with the measurement-bearing sentences carrying the re-run's actual domain, fit, and walls. Where a conclusion inverts rather than shifts — the attention-block wall, the "not compilable at all" paragraph — the record states what the new wall is or that it is unmeasured, and does not paper over the difference.

## Explicit non-goals

Not moving any budget. Not editing `docs/compiler/**`, corrected with the change. Not re-running the identity-growth ladder, which is its own ticket.
