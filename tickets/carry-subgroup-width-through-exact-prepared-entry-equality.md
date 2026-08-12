---
id: carry-subgroup-width-through-exact-prepared-entry-equality
title: Carry subgroup width through exact prepared-entry equality
status: todo
priority: p1
dependencies: [admit-an-atomic-subgroup-realization-subject-to-target-profiles, make-prepared-entry-observations-typed-and-key-dispatched, generalize-deferred-target-provenance-beyond-capability-axes]
related: [decide-the-prepared-subgroup-width-equality-gate, declare-metal-subgroup-realization-facts-in-the-target-profile, measure-metal-thread-execution-width-across-prepared-pipelines]
scopes: [implementation/compiler, implementation/artifact, implementation/build, implementation/runtime, implementation/metal, implementation/candle, contracts/optimizer, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [subgroup, metal, preflight, routing, feasibility, identity, implementation]
---
## User-visible outcome

Every subgroup-using entry is executable only when its exact prepared pipeline reports the literal width the compiler verified, before the route commits.

## Accepted boundary — 2026-08-11

Use the existing generic `PreparedEntryTargetRequirement` with a dedicated governed subgroup-width property key and `ObservedEqualsRequired`. Do not reuse Metal's live-device `RouteResourceDimension::SubgroupThreads`, add a duplicate wire row, trust the compile profile alone, or compare after commit.

## Required delivery

- Require exactly one profile-level prepared subgroup-width query whenever a profile declares any subgroup subject `Realized`; reject a missing, duplicate, conflicting, or orphan query as specified by the owning profile contract.
- Emit one requirement for every exact entry that uses subgroup transfer, derived from that entry's verified schedule/atomic target subject. Do not deduplicate across entries.
- Make Metal exact-dispatch the governed key/provider to that same retained pipeline's `threadExecutionWidth`. Unknown and mismatch are distinct precommit refusals.
- Preserve the route order: validate payload, live checks, prepare all exact pipelines, observe/compare, plan, consume commit, allocate, encode. No rebuild, pipeline substitution, automatic backend/width retry, or cached verdict.
- Derive a required artifact feature for subgroup prepared-width observation unless exact-base compatibility analysis proves every legacy reader unable to execute the new artifact. An old adapter that parses the row but answers another quantity must reject before observation.
- Recompute the feasibility rule-set key, profile descriptor, request/explain subjects, artifact/cache identities, and every affected pin. Do not step manifest/schema/domain merely because a new value uses an existing framed grammar.
- Perturb missing query, neighbour subject, entry association, provider, key, phase, width, relation, observation kind, pipeline identity, required feature, and post-observation substitution independently.

## Closes when

The real prepared Metal path demonstrates exact equality and every missing, unknown, mismatched, legacy, or cross-pipeline case fails before routing commit with no silent fallback.
