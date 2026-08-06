---
id: admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands
title: Admit an excluding-property parameter when a goal-directed input search lands
status: deferred
priority: p3
dependencies: [implement-boundary-property-enforcers]
related: [close-the-enforcer-input-property-exclusion-gap]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, enforcers, physical-planning]
---
## User-visible outcome

When Tiler's physical planning gains a goal-directed input search *and* speculative enforcer insertion, an enforcer's input search excludes the property the enforcer is about to supply, so the search does not build a plan that pays for a property its own producer already delivered.

## Why this is deferred rather than todo

**Fact.** [The enforcer input-property exclusion record](../docs/research/region-search/enforcer-input-property-exclusion.md) derives, against the Volcano paper's own passage and against the planner read at `d7b8604d`, that the redundancy is unreachable today at four independent levels. The two that matter for starting this ticket are that no enforcer exists in code at all, and — the deeper one — that Tiler's producer enumeration takes no property goal, so there is no "enforcer's input search" for an exclusion to be a parameter of.

**Inference — the deepest reason, restated so it is not lost.** Volcano's excluding vector is the *second* physical-property parameter of `FindBestPlan`; Tiler does not construct the *first*. `enumerate_frontier` (`crates/tiler-compiler/src/frontier.rs:2082`) has no required-property argument, `ImplementationContext` (`:1079`) is `{request, subject}`, and `boundary::derive_child_requirements` (`crates/tiler-compiler/src/boundary.rs:1905`) — the one function that would introduce a goal — has no non-test caller. Producers are enumerated bottom-up and independently of any consumer requirement; `selection::satisfy_edge` (`crates/tiler-compiler/src/selection.rs:1461`) then joins them and *rejects* a mismatch with `BoundaryDisagreement::UndischargedHandoff` rather than searching again.

## What the work is, when it starts

Add the excluding property vector as a **parameter of the goal-directed enumeration call**, in the same change that introduces the goal. The record derives that it must not become a third boundary-property dimension: it has no guarantee space and no satisfaction relation, so it fails [the contract's admission rule](../docs/compiler/optimizer.md#boundary-requirements-and-guarantees), and — the consequence that would be expensive to reverse — `BoundaryContract::encode` (`frontier.rs:564`) and `boundary::encode_property_identity` (`boundary.rs:1970`) fold the property sets into plan identity, so a third vector would make two identical plans reached under different exclusions carry different identities. **Plan identity must not encode search state.** Dominance is also not the home: `PlanStructuralCost::dominates` (`selection.rs:175`) already stops a redundant plan being *selected*, but `SelectedPortfolio::non_dominated` (`:563`) is a pure view, so it cannot stop one being *built*, which is the only half exclusion addresses.

## Non-goals

Adding a boundary-property dimension. Making dominance run before plan construction. Any work before both triggers below have fired.

## Trigger

Conjunctive; both are necessary and neither is sufficient.

- **T1 — the compile path constructs a boundary-property goal and drives producer enumeration from it.** That is `boundary::derive_child_requirements` gaining a non-test caller, or `enumerate_frontier`/`ImplementationContext` gaining a required-property parameter. This is the discriminating half: no live ticket is driving toward it, and the optimizer contract holds `memo` in the goal-directed-property-search sense as a reservation rather than a commitment.
- **T2 — enforcer insertion exists and is *speculative* rather than reactive**, generating an enforcer from a required property rather than only from an already-detected `UndischargedHandoff`. [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md) leaving `deferred` is necessary but not sufficient: its own restart condition describes a *reactive* enforcer, and a reactive enforcer cannot produce the redundancy, because a non-empty `unsatisfied_properties` result is the proof the producer did not already guarantee the property.

T2 without T1 does not fire this: a repair at a detected mismatch re-derives nothing. T1 without T2 does not fire it either: a goal-directed search with no enforcer move has no enforcer input to exclude from.

## Trigger check log

- 2026-08-06 — **not fired.** T1 is unmet: every mention of `derive_child_requirements` is still inside its declaring file, whose only call sites are in the `#[cfg(test)] mod tests` opening at `crates/tiler-compiler/src/boundary.rs:1994`. T2 is unmet on both halves: `implement-boundary-property-enforcers` is `deferred`, and no enforcer entity exists in code (`rg -n -i "enforcer" crates/ -g '*.rs'` returns 21 lines, all prose). The T1 check was proved able to say yes as well as no — the identical command shape over the sibling relation `unsatisfied_properties`, which does have a compile-path caller, returns eleven lines. Recheck: `rg -n "derive_child_requirements" crates/ | grep -v "^crates/tiler-compiler/src/boundary.rs"` — empty means not fired.
