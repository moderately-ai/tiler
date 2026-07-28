---
id: exercise-opaque-admissions-downstream-of-the-frontier
title: Exercise opaque admissions downstream of the frontier
status: todo
priority: p2
dependencies: []
related: [integrate-opaque-calls-into-the-physical-frontier, implement-boundary-property-enforcers]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, testing]
---
The frontier admits a registered, well-bound, feasible opaque call, and everything downstream of that admission is untested with an opaque body: selection composition, plan identity encoding, the component-cost `Unknown` arms (`scheduled()?`), the lowering refusal (`unlowerable-opaque-body`), and `pipeline/verify.rs`'s `None` arm. The last two are additionally unreachable in the compile path because no production provider proposes an opaque call (`pipeline/planning.rs` hardcodes the one governed provider and passes an empty registry).

The audit's framing: untested-but-claimed levels are findings. The integrate ticket's closing section now states what is tested at which level; this ticket closes the gap.

## What to drive, at selection level and below

- A cover whose region has both a scheduled admission and an opaque admission: selection must treat them as alternatives, and the plan identities must differ.
- An opaque producer with `Aliasing::MayAliasInputs` composing against a scheduled consumer requiring `MaterializedBuffer`: the handoff must be refused by `unsatisfied_properties` naming `Materialization` — this is the first *reachable* case of the mismatch the enforcers ticket waits for, so `implement-boundary-property-enforcers`'s startable condition should be re-evaluated when this lands (its trigger note names this).
- A selected plan containing an opaque body: `Indexing`/`RedundantWork`/`MemoryTraffic` report `Unknown` (not zero), lowering refuses with `unlowerable-opaque-body`, and verify's `None` arm fires rather than a filtered region list reaching `build_plan_program`.

## Constraint that must survive

Do not fabricate a production provider to make paths reachable in the compile path — test-level providers are the honest subject until caller-supplied physical providers exist. The compile-path unreachability is a stated boundary, not a bug to hide.

## Closes when

- Each bullet above has a test that can fail (perturb once and watch it fire), and the integrate ticket's tested-at-which-level table is updated to match.
