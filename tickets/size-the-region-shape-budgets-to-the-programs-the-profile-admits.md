---
id: size-the-region-shape-budgets-to-the-programs-the-profile-admits
title: Decide whether the three region-shape budgets move with semantic_operations
status: done
priority: p2
dependencies: []
related: [region-expansion-exhaustion-loses-the-only-feasible-plan, carry-the-thirty-two-operation-ladder-into-the-five-records, assemble-the-decoder-layer-program, derive-the-region-shape-budgets-from-the-declaration, state-the-rule-that-a-deterministic-budget-is-a-derivation, decide-whether-a-derived-budget-belongs-in-the-request-subject]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, identity, decision]
---
## The decision

**Historical open-decision framing (pre-derive).** At filing, `DeterministicBudgets::governed`'s `semantic_operations` was **62**, sized on 2026-08-05 "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". `region_members` was **32**, and it had not moved with it. Should it and its two siblings — `region_boundary_outputs` (8) and `region_live_values` (64) — move too, and to what?

Only Tom decides this. Every budget is written into the canonical request subject (`VerifiedRequestSubject::canonical_explain_subject_bytes` writes all fourteen), so a widening moves every governed compilation's qualifier and the one pinned identity that encodes it — `explain`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier. No encoding version moves with a *budget value* change: the field set, widths, and order are untouched, so the subject stays injective inside the request-subject domain. **Correction — 2026-08-10.** At decision time that domain was `tiler.compiler.request-subject.v5`; the live encoder now tags `tiler.compiler.request-subject.v6` because `SemanticIdentity` gained a shape-environment subject — an unrelated domain step, not a consequence of this budget derivation. Budget value changes still do not step the encoding.

## Why it is open, and why it was not open before

The 2026-08-05 widening's own comment recorded the ground for leaving every `region_*` bound alone: "none of them admits or refuses a program: each bounds a *search*, and exhausting one costs an alternative while the verified input and complete coverage survive."

[`region-expansion-exhaustion-loses-the-only-feasible-plan`](region-expansion-exhaustion-loses-the-only-feasible-plan.md) established that this is true of two of the five and false of three. `region_members`, `region_boundary_outputs`, and `region_live_values` bound one region's admissible *shape* rather than a search: they declare the largest region the profile forms, and a program whose only implementable cover needs a bigger one has no plan under them however long the search runs. So the ground the earlier decision rested on does not cover these three, and whether they should have moved with `semantic_operations` was never actually decided.

**Measurement, 2026-08-07, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`.** For a shared-constant `f32` multiply chain the recognized partition is the whole program and no smaller region is implementable, so 2..=32 operations compile and **33..=62 refuse `BudgetExhausted` on `region_members`**. The governed `semantic_operations` maximum of 62 is unreachable for this family, and the bound that makes it unreachable is 32.

## What each answer enables and prevents

- **Leave 32.** The profile plans no pointwise program above 32 operations, and the `semantic_operations` maximum stays unreachable for any family whose only implementable cover is its whole program. Nothing moves; no identity moves. The strongest counterpoint: the budget that was deliberately sized to the decoder layer is bounded below by one that was not sized to anything in particular, so the profile's stated admission envelope and its actual planning envelope disagree, and a caller learns that only by compiling.
- **Move `region_members` to 62.** The pointwise ladder reaches the `semantic_operations` maximum and the two envelopes agree. Every governed compilation's request subject moves, and the one pinned identity is recomputed with it. The strongest counterpoint: 62 is derived from the decoder layer's *occurrence count*, and a region is not a program — a decoder layer's regions are its recognized partitions, several per output, and none of them is 62 occurrences wide. Sizing a region bound from a program count is the same category error the `regions` bound was corrected for on 2026-08-05.
- **Derive all three from the decoder layer's widest recognized region**, as the five program-scoped bounds were derived from its measured counts. This is the answer that matches the rule `check_program_budgets` states and that every previous widening followed. It needs a number that does not exist yet: the layer's widest region is a property of a plan, and [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md) has the program but the profile refuses to plan it.

**Recommendation: the third, deferred until the decoder layer is plannable, and 32 left standing until then.** A widening now would be sized from a program count rather than a region count, would move every artifact identity in the workspace, and would have to move again when the real number arrives — which is the "second identity move this one cannot honestly absorb" the 2026-08-05 comment already anticipated. What this ticket asks Tom to confirm is that the envelope disagreement is acceptable in the meantime, now that it is measured rather than unknown.

## Decided — derive, do not pick, 2026-08-07

**Tom decided on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator: **none of the three options as framed.** All three were "choose a constant", and choosing a constant sized to today's largest known program is the self-constraint the decision rejects. The three region-shape bounds become **derivations over the declaration**, as the five program-scoped bounds already are.

### Why this node's own recommendation was withdrawn

This ticket recommended the third option deferred until the decoder layer is plannable, with 32 left standing. **That rested on a dependency that does not govern**, and the coordinator checked it at source before recommending against it: `DeterministicBudgets::governed`'s own comment states that "clearing the budget gate [does not] compile a decoder layer — the recognizer's refusal is untouched, and what this widening removes is only the refusal that was about *size*." The layer is blocked by `select_supported_strategy`, a separate refusal with a separate remedy. So the deferral was waiting on something that was never in the way.

### The defect, stated correctly

The five program-scoped bounds are each a **formula over the declaration** — `semantic_values` is eighteen inputs plus one result per occurrence, `regions` is the four-stage widest chain times three declared outputs, `host_expression_nodes` is two per input plus four per output plus three, `buffers` is inputs plus four per output. The three region-shape bounds are **bare constants**: 32, 8, 64, with no derivation and no stated rule. That asymmetry is why one set moved coherently on 2026-08-05 and the other could not.

**The repository has already made this exact correction once.** `regions` was `4` and "checked against a constant rather than derived", on the ground that a region count is a property of a *plan* and this profile plans no decoder layer. The comment records the outcome in terms: *"That ground survives and its conclusion did not."* A plan covers every declared output, so a plan-scoped quantity is still a function of the **declaration**. That precedent applies here unchanged.

### The derivation, which is structural rather than a heuristic

**A region is a subset of the program's occurrences, so it cannot hold more than the program contains.** `semantic_operations` therefore already bounds `region_members`; a separate, tighter constant is a second and undermotivated ceiling, and it is the entire source of the envelope disagreement this ticket was filed about. `region_live_values` bounds against `semantic_values` and `region_boundary_outputs` against the declared outputs on the same reasoning.

**This is not the rejected option two.** Reusing the literal 62 because it is the decoder layer's occurrence count is the category error this ticket named — sizing a region bound from a program count. Deriving `region_members` *from* `semantic_operations` because a region cannot exceed the program is a different claim, and it is true of every program rather than of one.

### What this buys, against the stated criteria

The stated admission envelope and the actual planning envelope agree, because they are the same derivation — the disagreement is dissolved rather than recorded. It is **one identity move and the last of this kind for these three**, because a derivation tracks the declaration instead of being a ceiling raised per program. And it is available now rather than after the layer becomes plannable.

### The risk to measure rather than assume

A wider admissible region may raise per-candidate cost. The shape bounds are not what bound search work — `region_candidates_per_seed` and `region_expansions` do that — but the per-candidate cost of checking a wider region is real and should be measured before landing, not argued.

## Released work

- [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) — the change itself.
- [`state-the-rule-that-a-deterministic-budget-is-a-derivation`](state-the-rule-that-a-deterministic-budget-is-a-derivation.md) — the general rule, so the next constant does not drift into a ceiling.
- [`decide-whether-a-derived-budget-belongs-in-the-request-subject`](decide-whether-a-derived-budget-belongs-in-the-request-subject.md) — the structural question underneath all of it.

**Correction — 2026-08-10.** Implementation of this decision landed under [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md). Live `DeterministicBudgets::governed` region-shape budgets are derivations, not the filing-time bare constants 32 / 8 / 64: `region_members` is **62** (`semantic_operations`), `region_boundary_outputs` is **3** (declared output count), `region_live_values` is **80** (`semantic_values`). Reproduce: `rg -n 'region_members: 62|region_boundary_outputs: 3|region_live_values: 80' crates/tiler-compiler/src/request.rs`. The request-subject domain tag is `tiler.compiler.request-subject.v6` for the unrelated shape-environment subject step; budget value changes still do not step encoding. The Decided section above records the decision-time defect and derivation argument and is left standing for that history.
