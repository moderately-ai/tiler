---
id: size-the-region-shape-budgets-to-the-programs-the-profile-admits
title: Decide whether the three region-shape budgets move with semantic_operations
status: awaiting-decision
priority: p2
dependencies: []
related: [region-expansion-exhaustion-loses-the-only-feasible-plan, carry-the-thirty-two-operation-ladder-into-the-five-records, assemble-the-decoder-layer-program]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, identity, decision]
---
## The decision

`DeterministicBudgets::governed`'s `semantic_operations` is **62**, sized on 2026-08-05 "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". `region_members` is **32**, and it was not moved with it. Should it and its two siblings — `region_boundary_outputs` (8) and `region_live_values` (64) — move too, and to what?

Only Tom decides this. Every budget is written into the canonical request subject (`VerifiedRequestSubject::canonical_explain_subject_bytes` writes all fourteen), so a widening moves every governed compilation's qualifier and the one pinned identity that encodes it — `explain`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier. No encoding version moves with a value change: the field set, widths, and order are untouched, so the subject stays injective inside `tiler.compiler.request-subject.v5`.

## Why it is open, and why it was not open before

The 2026-08-05 widening's own comment recorded the ground for leaving every `region_*` bound alone: "none of them admits or refuses a program: each bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive."

[`region-expansion-exhaustion-loses-the-only-feasible-plan`](region-expansion-exhaustion-loses-the-only-feasible-plan.md) established that this is true of two of the five and false of three. `region_members`, `region_boundary_outputs`, and `region_live_values` bound one region's admissible *shape* rather than a search: they declare the largest region the profile forms, and a program whose only implementable cover needs a bigger one has no plan under them however long the search runs. So the ground the earlier decision rested on does not cover these three, and whether they should have moved with `semantic_operations` was never actually decided.

**Measurement, 2026-08-07, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`.** For a shared-constant `f32` multiply chain the recognized partition is the whole program and no smaller region is implementable, so 2..=32 operations compile and **33..=62 refuse `BudgetExhausted` on `region_members`**. The governed `semantic_operations` maximum of 62 is unreachable for this family, and the bound that makes it unreachable is 32.

## What each answer enables and prevents

- **Leave 32.** The profile plans no pointwise program above 32 operations, and the `semantic_operations` maximum stays unreachable for any family whose only implementable cover is its whole program. Nothing moves; no identity moves. The strongest counterpoint: the budget that was deliberately sized to the decoder layer is bounded below by one that was not sized to anything in particular, so the profile's stated admission envelope and its actual planning envelope disagree, and a caller learns that only by compiling.
- **Move `region_members` to 62.** The pointwise ladder reaches the `semantic_operations` maximum and the two envelopes agree. Every governed compilation's request subject moves, and the one pinned identity is recomputed with it. The strongest counterpoint: 62 is derived from the decoder layer's *occurrence count*, and a region is not a program — a decoder layer's regions are its recognized partitions, several per output, and none of them is 62 occurrences wide. Sizing a region bound from a program count is the same category error the `regions` bound was corrected for on 2026-08-05.
- **Derive all three from the decoder layer's widest recognized region**, as the five program-scoped bounds were derived from its measured counts. This is the answer that matches the rule `check_program_budgets` states and that every previous widening followed. It needs a number that does not exist yet: the layer's widest region is a property of a plan, and [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md) has the program but the profile refuses to plan it.

**Recommendation: the third, deferred until the decoder layer is plannable, and 32 left standing until then.** A widening now would be sized from a program count rather than a region count, would move every artifact identity in the workspace, and would have to move again when the real number arrives — which is the "second identity move this one cannot honestly absorb" the 2026-08-05 comment already anticipated. What this ticket asks Tom to confirm is that the envelope disagreement is acceptable in the meantime, now that it is measured rather than unknown.
