---
id: derive-the-region-shape-budgets-from-the-declaration
title: Derive the region shape budgets from the declaration
status: todo
priority: p2
dependencies: []
related: [size-the-region-shape-budgets-to-the-programs-the-profile-admits]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, identity]
---
## What to do

`DeterministicBudgets::governed`'s `region_members` (32), `region_boundary_outputs` (8) and `region_live_values` (64) are bare constants where the five program-scoped bounds beside them are formulas over the declaration. **Tom decided on 2026-08-07** that they become derivations. Read [`size-the-region-shape-budgets-to-the-programs-the-profile-admits`](size-the-region-shape-budgets-to-the-programs-the-profile-admits.md)'s Decided section in full first — it records why all three "pick a number" options were rejected and why the precedent that governs is `regions`, which made this exact correction already.

**The derivation is structural.** A region is a subset of the program's occurrences, so it cannot hold more than the program contains: `semantic_operations` bounds `region_members`. `region_live_values` bounds against `semantic_values`, and `region_boundary_outputs` against the declared outputs. Derive each; do not restate a number.

**What this is not.** Setting `region_members` to the literal 62 because that is the decoder layer's occurrence count is the category error the deciding ticket rejected — a region is not a program. Deriving it *from* `semantic_operations` is a different claim and true of every program. If your derivation reads as "the decoder layer needs N", stop: you have written the rejected option.

## The obligations that come with it

**Derive tightly, not loosely.** `check_program_budgets` states the rule the five follow: each is derived from the program's own measured counts rather than from the smallest number that lets something through. A bound derived so loosely it can never bind is not a bound; if the honest derivation for one of the three *is* the program-scoped bound itself, say that explicitly and explain why a separate field is still worth encoding — or propose removing the field, which is a public-boundary question and comes back to Tom rather than being taken.

**Measure the per-candidate cost.** The deciding ticket names this as the one risk to measure rather than assume: a wider admissible region may raise the cost of checking each candidate. The search bounds `region_candidates_per_seed` and `region_expansions` are what bound search *work*, so the shape bounds are not doing that job — but per-candidate cost is real. Report a measurement, not an argument. If it regresses materially, stop and report rather than absorbing it.

**Keep the refusal honest.** Today a program refused on `region_members` reports `BudgetExhausted` naming the bound, which reads like an admission decision when it is a compiler-internal ceiling. Whatever the derivation, a refusal must still name which resource was exhausted — see [`carry-the-exhausted-resource-through-the-budget-refusal`](carry-the-exhausted-resource-through-the-budget-refusal.md), which owns that separately and must not be pre-empted or duplicated here.

## Identity — one move, and it is the point that it is the last

Every budget is written into the canonical request subject (`VerifiedRequestSubject::canonical_explain_subject_bytes` writes all fourteen), so **every** governed compilation's qualifier moves — for programs nowhere near any bound as much as for ones at them. Exactly one pinned identity encodes those bytes: `explain`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier. Recompute it **on the merged tree**, not on your branch, and record before and after in the ledger comment beside it.

No encoding version should move: the field set, widths and order are untouched by a value change, so the subject stays injective inside `tiler.compiler.request-subject.v5`. **If you find yourself needing an encoding step, stop and report** — that would mean the change is wider than this ticket.

Current pinned values elsewhere, to verify against and report any difference rather than rebaseline: standard Metal artifact identity `357f0676…`, cache subject `c626e43b…`, fixed content 65,242 bytes, descriptor length 2,099. Those should not move; a budget is a property of the request, not of the artifact's own bytes — **confirm that rather than assuming it.**

## Required evidence

- Each of the three derivations stated as a rule beside the other five, in the same form, so a reader sees one convention rather than two.
- The measured population that used to refuse: `33..=62` operations on a shared-constant `f32` multiply chain refused `BudgetExhausted` on `region_members` — show what they do now, and that the change is admission and not merely a wider search.
- A program one step past each derived bound still refused, so widening admits shapes without removing the bound.
- The per-candidate cost measurement.
- The moved pin enumerated with before and after.

## Closes when

The three are derivations in the same form as the five, the previously-refused population is admitted, a program past each bound is still refused, the cost is measured rather than argued, and the one moved pin is recomputed on the merged tree with its ledger updated.
