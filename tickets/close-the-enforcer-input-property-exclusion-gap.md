---
id: close-the-enforcer-input-property-exclusion-gap
title: Close the enforcer input-property exclusion gap
status: review
priority: p3
dependencies: []
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature, admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands, catalog-the-enforcer-input-property-exclusion-record]
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [research, optimizer, enforcers, boundary-properties]
claimed_from: todo
assignee: agent-enforcer-gap
lease_expires_at: 1786031121
---
## User-visible outcome

A derivation of whether Tiler's enforcer insertion needs Volcano's *excluding physical property vector*, and if so where it belongs — recorded so the answer is not re-derived the first time an enforcer's input search re-derives the property the enforcer was about to supply.

## Why this exists

**Fact.** Volcano (`volcano-icde-1993`, preserved under [the formalism record's sources](../docs/research/region-search/sources/README.md)) carries a parameter the optimizer contract does not: when optimizing an enforcer's input, the required-property vector is relaxed *and* an excluding vector forbids algorithms that already deliver the property being enforced. Its worked case is that under a sort, hybrid hash join must apply and merge-join must not.

**Fact.** [The optimizer contract](../docs/compiler/optimizer.md#boundary-requirements-and-guarantees) says only that "enforcer insertion is cycle-checked". A cycle check catches an enforcer feeding itself; it does not stop the input search from choosing a producer that already guarantees the property, which is a redundant plan rather than a cyclic one.

**Inference.** Tiler has one enforcer family in flight (materialization, layout conversion, encoding repacking) and no evidence this has bitten. The gap is real and its cost today is unmeasured — which is why this is a derivation ticket rather than an implementation one.

## What the record owes

- Whether the redundancy is reachable in the current planner at all, checked by reading the frontier and selection paths rather than assumed.
- If reachable: whether exclusion belongs in the boundary-property system (as a third vector beside requirement and guarantee) or in dominance, and what its identity encoding is — the contract's own admission rule for a new property dimension applies.
- If unreachable today: the exact condition that would make it reachable, so this becomes a deferral with a trigger rather than an open note.

## Non-goals

Implementing exclusion; adding a property dimension without meeting the contract's admission rule.

## Outcome — the derivation is recorded; the redundancy is unreachable (2026-08-06)

**The record.** [Enforcer input-property exclusion](../docs/research/region-search/enforcer-input-property-exclusion.md), linked from [the rewrite-search formalism record](../docs/research/region-search/rewrite-search-formalism.md)'s Part 1 (which called this a real gap) and Part 8 (which listed it as a deferred outcome). Every claim about Tiler's code in it was read at `d7b8604d` and states its reproducing command.

**Answer to the first thing this ticket owed — is the redundancy reachable? No, at four independent levels, and the ticket's own framing understated why.**

- **Fact — no enforcer exists in code.** `rg -n -i "enforcer" crates/ -g '*.rs'` returns 21 lines, every one a doc comment, an `#[allow(reason = …)]` string, or an assertion message; `rg -n -i "fn [a-z_]*enforc|struct [A-Za-z]*Enforc|enum [A-Za-z]*Enforc|::Enforc" crates/` returns four unrelated test-function names containing the verb *enforces*. [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md) is `deferred`. **This is a contract gap and an implementation gap at once**: the optimizer contract names the enforcer family and says insertion "is cycle-checked", describing a mechanism the compiler does not have.
- **Fact — and this is the deeper level — the enumeration takes no property goal.** `enumerate_frontier` (`crates/tiler-compiler/src/frontier.rs:2082`) has no required-property argument; `FrontierRegionSubject` (`:1282`) carries role, members, intermediate element counts, and write target; `ImplementationContext` (`:1079`) is exactly `{request, subject}`. A provider is never told what a consumer requires. `boundary::derive_child_requirements` (`crates/tiler-compiler/src/boundary.rs:1905`), the one function that would introduce a goal, has **no non-test caller** — `rg -n "derive_child_requirements" crates/` returns five lines, all in `boundary.rs`, with the three call sites inside the `#[cfg(test)] mod tests` opening at `:1994`. The crate says so about itself at `boundary.rs:3`.
- **Fact — composition is a bottom-up join with a typed rejection, not a search.** `selection.rs:4-13` states the cover and frontier authorities "neither depends on the other" and that selection is "the first authority allowed to *join* them". `satisfy_edge` (`selection.rs:1461`) calls `unsatisfied_properties` at `:1500` — the sole compile-path call site — and returns `BoundaryDisagreement::UndischargedHandoff` at `:1504`. Nothing is re-enumerated and nothing is inserted.
- **Inference — the fourth level survives the first three being fixed.** The enforcer shape this repository is heading for is *reactive*: `implement-boundary-property-enforcers`'s restart condition is a provider producing an already-refused handoff. An enforcer inserted only where `unsatisfied_properties` returned non-empty **cannot** sit ahead of a producer that already guarantees the property, because that non-empty result is the proof it does not. Volcano's insertion is speculative; the redundancy is a hazard of speculative insertion only.

**Answer to the second thing — third vector, or dominance? Neither, and the ticket's framing rests on a mapping error the record corrects.** Tiler's requirement/guarantee pair is not Volcano's `PhysProp`: `BoundaryContract`'s two sides are "*derived* from the verified region — never taken from the provider" (`frontier.rs:502-505`), so they are Volcano's *derived input* vector, a fact computed upward, while `PhysProp` is a goal handed downward. Volcano's excluding vector is a parameter of `FindBestPlan` alongside the branch-and-bound `Limit` — search state, not plan state. A third boundary-property vector fails [the admission rule](../docs/compiler/optimizer.md#boundary-requirements-and-guarantees) on its face (no guarantee space, no satisfaction relation — its test is that a producer *does* satisfy, the inverse — and no enforcer that discharges it), and its **identity-encoding consequence is disqualifying**: `BoundaryContract::encode` (`frontier.rs:564`) and `boundary::encode_property_identity` (`boundary.rs:1970`) fold the property sets into plan identity, so two identical plans reached under different exclusions would carry different identities, and `verify_selected_plan` — which re-derives the plan and must reproduce the identity — has no access to a search history to reproduce it from. Dominance is not the home either: `PlanStructuralCost::dominates` (`selection.rs:175`) already stops a redundant plan being *selected*, but `SelectedPortfolio::non_dominated` (`:563`) is documented as "a pure *view*" (`selection.rs:36-38`), so the plan is still enumerated, verified, and identity-minted. Dominance covers the correctness half completely and cannot cover the search-cost half, which is the only half exclusion addresses — Volcano's own justification is that algorithms "must not be explored again", the register it also uses for the cost limit ("for optimization speed, not for correctness").

**Source discipline.** `volcano-icde-1993` is `metadata-only`, so the paper was re-acquired from the URL [the source record](../docs/research/region-search/sources/README.md#volcano-icde-1993) names, into a directory outside this repository, and its SHA-256 matched `expected-sources.tsv`'s recorded digest (`77a4930474ee3caf2e774c72d1b842190e299fd4f492ea3577f8307972cc3f5f`, 1 257 723 bytes) exactly before anything was quoted. Bytes read and discarded; nothing vendored; licence verdict unchanged. This is the first exercise of a retrieval fingerprint in that directory and it held.

## Graph maintenance

**Recommended terminal state: `done`, not `deferred` — and this deviates from the dispatch brief, deliberately.** The brief anticipated that an "unreachable" verdict would convert *this* ticket into the deferral. It should not, because this ticket's stated outcome is "a derivation … recorded", which is delivered, and its **non-goals explicitly exclude implementing exclusion** — the only work a fired trigger would authorize. A deferral whose own non-goals forbid the work it waits for is not a coherent board object, and holding a satisfied outcome in a parked state is the deadlock `AGENTS.md` warns about. The remainder is split instead, which is the prescribed shape.

- **Filed — [`admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands`](admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands.md)**, `deferred`, carrying the conjunctive trigger and its `## Trigger check log`. It `depends_on` [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md) (T2 — it cannot start without one), and states T1 in its body because **no ticket owns the goal-directed search**: the optimizer contract holds `memo` in that sense as a reservation, and Q-PLAN-002 is shared-work duplication, not this. If a goal-directed-search ticket is ever filed, that edge should be added.
- **Filed — [`catalog-the-enforcer-input-property-exclusion-record`](catalog-the-enforcer-input-property-exclusion-record.md)**, `todo`. The record needs a row in `docs/research/README.md`, which is `contracts/navigation` and outside this ticket's `research/region-search` scope. Filed rather than skipped, so the catalog gap is a work item instead of silent drift.
- **Corrected in place.** The formalism record's Part 1 bullet and Part 8 deferral entry now point at the derivation, and Part 1 carries the `PhysProp` mapping correction so the two records cannot drift on it.
- **Checked and found clean.** `implementation_status: "not-implemented"` appeared to be corpus drift against `docs/document-metadata.md`'s controlled set (`not-started`, `spike-only`, `partial`, `implemented`); reading it showed the only occurrence was this ticket's own new record before correction. `rg -l 'implementation_status: "not-implemented"' docs/` now returns nothing. No drift ticket is owed.
