---
id: derive-the-region-shape-budgets-from-the-declaration
title: Derive the region shape budgets from the declaration
status: in-progress
priority: p2
dependencies: []
related: [size-the-region-shape-budgets-to-the-programs-the-profile-admits, rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets, correct-the-records-the-derived-region-shape-budgets-falsify]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, identity]
claimed_from: todo
assignee: agent-budgets
lease_expires_at: 1786125055
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

## Outcome — 2026-08-07

### The three derivations, stated beside the five

`DeterministicBudgets::governed` now carries eight derived bounds in one list rather than five formulas beside three constants. The three new rules, in the form the five are stated in:

- **`region_members` is `62`: `semantic_operations`**, because a region's members are a subset of the program's own occurrences and a program admitted at all holds no more than that many.
- **`region_boundary_outputs` is `3`: the declared output count**, which is the same count `regions` multiplies by four. The whole-program region exports one value per *named* result and nothing else, because no occurrence outside it consumes anything — so the largest region this profile forms exports exactly the declaration's ordered named outputs.
- **`region_live_values` is `80`: `semantic_values`**, because a region's live values are its boundary inputs and its members' results, which are disjoint subsets of the program's own values. It is tight at the whole-program region, whose boundary inputs are the eighteen declared inputs and whose member results are one per occurrence — the same `18 + 62` `semantic_values` is.

### Two of the three collapse onto the program-scoped bound, and it is said out loud

`region_members` **is** `semantic_operations` and `region_live_values` **is** `semantic_values`, so for a program whose occurrences are each realized by one region neither can fire: `check_program_budgets` has already refused anything larger. That is the derivation's answer rather than a defect in it — the envelope disagreement this work was filed about is dissolved by the two envelopes becoming one formula — but it means two fields that the governed profile can no longer fire keep their slots in the canonical request subject.

Both are still encoded, for two reasons stated in the code. Region formation's attribution atom is a realization *stage* rather than an occurrence, and its live values include the intermediates a staged law hands between stages; neither is a value the program's own occurrence and value counts hold, so both bounds still bind on a program whose families realize region sequences. And a budget set is a *request* field, so these bound one region's shape for any budgets a caller states; the governed profile's coincidence is a property of its declaration rather than of the fields.

**Whether a field the governed profile can no longer fire deserves its slot in the request subject is a public-boundary question and was not taken here.** `region_boundary_outputs` does not collapse and needs no such question: it is the declared output count rather than any program-scoped bound, and it still refuses grown candidates under the governed profile.

`region_boundary_outputs` narrowed, 8 → 3. Every workspace test passes unchanged, and the argument is that a candidate exporting four or more values needs four or more owning writes and is unspellable: a scheduled region writes one owning tensor, and the admitted published-and-consumed shape exports one *value* carrying two roles rather than two values.

### The previously-refused population, and that the change is admission

**Reproduced first.** With `region_members` restored to 32 on this tree, `chain_program(33)` refuses `TargetCompileFailure { class: BudgetExhausted, explain: "259 records" }` through `compile_governed` — the measurement the deciding ticket recorded, re-observed.

**Now.** `crates/tiler-compiler/tests/region_search_budget_coverage.rs`'s `the_population_the_member_bound_refused_compiles_as_one_whole_program_region` compiles every point of `33..=62` and asserts the selected plan's stage coverage equals `vec![operations]` — **one dispatch covering the whole program**. That is admission and not a longer search: the plan is built from the whole-program candidate, which the constant rejected at formation, so no amount of searching could have produced it. Measured end to end through `compile_governed`: 20 ms at n=33, 73 ms at n=62; `n=63` refuses `BudgetExhausted` on `semantic_operations` before any target is consulted.

### A program one step past each derived bound, still refused

`crates/tiler-compiler/src/region.rs`'s `each_derived_region_shape_bound_admits_its_own_size_and_refuses_one_more` drives all three as pairs, at region formation — deliberately, because two of the three derivations are the program-scoped bound itself, so a *program* large enough to reach them is refused for its size first and the region bound would never be observed.

| Bound | Admitted at the bound | Refused one past | Typed stop |
| --- | --- | --- | --- |
| `region_members` = 62 | 62-operation multiply chain forms its whole program | 63-operation chain does not | `region-members` limit 62 actual 63 |
| `region_boundary_outputs` = 3 | three-member region exporting three values emitted | four-member region exporting four values not emitted | `region-boundary-outputs` limit 3 actual 4 |
| `region_live_values` = 80 | 40 declared inputs + 1 self-add = exactly 80 live | 40 inputs + 2 self-adds = 81 live | `region-live-values` limit 80 actual 81 |

Each was watched failing before being trusted, by perturbing the profile one field at a time and restoring it:

- `region_members: 63` → `assertion failed: refused.whole_program_candidate().is_none()` at `region.rs:4554`.
- `region_boundary_outputs: 4` → `a region exporting one value more than the program declares is refused` at `region.rs:4574`.
- `region_live_values: 81` → `assertion failed: refused.whole_program_candidate().is_none()` at `region.rs:4599`.
- `region_members: 32` → `33 operations plan under the derived member bound: … class: BudgetExhausted` at `region_search_budget_coverage.rs:139`.

### The per-candidate cost measurement

**Measurement, 2026-08-07, Apple M4 Max, macOS 27.0, toolchain `nightly-2026-07-19`, dev profile, coordination host with other agents' builds running.** `crates/tiler-compiler/src/region.rs`'s `the_derived_shape_bounds_leave_the_previously_admitted_candidates_untouched`, fastest of three repetitions per row. The denominator is a new `crate::workcount::REGION_CANDIDATE_FORMATIONS` counter — the node sets `form_candidate` actually checked — rather than the emitted candidate count, which would price every rejected set into the survivors.

| n | node sets checked | emitted | superseded (32/8/64) | derived (62/3/80) |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 157 | 36 | 2855 ns/check | 2739 ns/check |
| 16 | 2821 | 22 | 1816 ns/check | 1767 ns/check |
| 24 | 3574 | 29 | 1782 ns/check | 1750 ns/check |
| 32 | 4208 | 36 | 1817 ns/check | 1843 ns/check |
| 40 | 4880 | 44 | — | 2043 ns/check |
| 48 | 5041 | 52 | — | 2197 ns/check |
| 62 | 5357 | 66 | — | 2592 ns/check |

**No regression on the previously admitted population, and the deterministic half says so without the clock.** At every n ≤ 32 the emitted candidate *sets* and the checked-set *count* are asserted equal under both budget sets, and `form_candidate` is a pure function of the graph, budgets, and contract — an unchanged population over an unchanged code path is unchanged per-candidate work. The timings agree within ±4%, which is inside this host's noise.

**Per-check cost does rise across the newly admitted range**, 1843 ns at n=32 to 2592 ns at n=62 — 41%, because convexity and connectivity are per-member and the sets now legal are larger. It is not a regression: nothing in the previously admitted domain pays it, and what pays it is a program that previously had no plan at all. Total region formation at n=62 is 5357 × 2592 ns ≈ 13.9 ms inside a 73 ms compilation.

### Identity — one pin moved, and four did not

**MOVED. `explain`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier: `0aa252e0bfa16451` → `e59cb8aa9b38ef70`.** Recomputed on this branch at its own commit with the command its ledger comment states. **The coordinator must recompute it on the merged tree** rather than carrying this value across: two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.

No encoding version moved. The field set, widths, and order are untouched, so the subject stays injective inside `tiler.compiler.request-subject.v5`, and no encoding step was needed.

**Confirmed unmoved, each named and observed running rather than assumed:**

- `tiler-build`'s `metal_plan::tests::the_standard_metal_path_publishes_its_recorded_identities` — 1 test run, passed, asserting standard Metal artifact identity `357f0676…`, cache subject `c626e43b…`, and fixed content 65,242 bytes byte for byte.
- `tiler-build`'s `metal_declaration::tests::a_refused_synchronization_verdict_moves_the_profile_descriptor` — 1 test run, passed, asserting canonical descriptor length 2,099.

A budget is a property of the request rather than of an artifact's own bytes, and this is that claim measured rather than assumed.

### Consequence recorded rather than left to be discovered

`BudgetExhausted`'s second route — the empty portfolio after a region budget truncated a target's analysis — is **no longer reachable from the public surface** under governed budgets, because the two collapsed derivations mean a program large enough to reach them is refused for its size first. `crate::session`'s failure-class reachability inventory now says so, and cites `crate::pipeline::tests::a_region_shape_budget_below_the_only_implementable_cover_reports_the_budget`, which drives the path one layer down under a caller-stated budget set and is what keeps it measured. The refusal's inability to name *which* resource was exhausted is untouched here; [`carry-the-exhausted-resource-through-the-budget-refusal`](carry-the-exhausted-resource-through-the-budget-refusal.md) owns it.

### Scope added

`contracts/optimizer`, for [`docs/compiler/optimizer.md`](../docs/compiler/optimizer.md)'s budget list, which stated the three superseded constants as the profile's current values, and for the derivation paragraph beside it. Adding it is scheduling metadata for authorized work; no other live claim holds it.

### Released work

- [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md) — the spike's `OPERATIONS` ladder and `WALLS` table both misstate this tree now, and widening them is a re-measurement rather than an edit.
- [`correct-the-records-the-derived-region-shape-budgets-falsify`](correct-the-records-the-derived-region-shape-budgets-falsify.md) — seven sites across five scopes this ticket does not hold, each enumerated with the claim that is now false; it depends on the re-run because several of them quote the ladder's extent and fit.

### Checks

All from the worktree, all exit 0: `cargo fmt`, `cargo check --workspace --all-targets --locked`, `cargo nextest run -p tiler-compiler --locked` (756 passed, 1 skipped), `cargo clippy -p tiler-compiler --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler --locked`, `cargo nextest run --workspace --locked` (2979 passed, 7 skipped), `cargo test --workspace --doc --locked`, `git diff --check`, `tkt lint`, `tkt guard`, and `make full`.
