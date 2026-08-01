---
id: rename-the-route-resource-floor-vocabulary-for-its-corrected-relation
title: Rename the route resource floor vocabulary for its corrected relation
status: todo
priority: p2
dependencies: []
related: [correct-the-subgroup-threads-route-dimension-meaning]
scopes: [implementation/artifact, implementation/runtime, implementation/candle, research/runtime, contracts/decisions, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, naming, public-boundary]
---
## User-visible outcome

The type and variant carrying a live-device route resource requirement are named for what they are — a required quantity compared by the relation its dimension fixes — rather than for the floor relation `correct-the-subgroup-threads-route-dimension-meaning` removed.

## Why this is separate from the correction that created it

**Fact.** `correct-the-subgroup-threads-route-dimension-meaning` changed `RouteResourceDimension::SubgroupThreads` from a floor to an equality and corrected every name it could reach inside `crates/tiler-artifact`: the private field and public accessor `minimum()` became `required()`, and `RouteRequirementError::VacuousFloor` became `ZeroResourceQuantity`. Two names it could not reach remain, and both state the removed relation:

- `RouteResourceFloor` — the struct.
- `RouteRequirement::ResourceFloor` — the enum variant.

**Fact — the exact reason they were not renamed.** Both are named outside the `implementation/artifact` and `contracts/artifacts` scopes that ticket held. Reproduce with `grep -rn "ResourceFloor" --include="*.rs" --include="*.md" . | grep -v "^./crates/tiler-artifact/"`:

| Site | Scope |
| --- | --- |
| `crates/tiler-runtime/src/load/route.rs` (2 match arms) | `implementation/runtime` |
| `crates/tiler-runtime/tests/adapter_route/adapter.rs`, `crates/tiler-runtime/tests/identity_join/adapter.rs` | `implementation/runtime` |
| `prototypes/serial-sum-run/src/proof.rs` (a match arm and a `RouteResourceFloor::new` call) | `implementation/runtime` |
| `prototypes/candle-metal-adapter/src/adapter.rs` | `implementation/candle` |
| `spikes/runtime/inline-dispatch/src/adapter.rs` | `research/runtime` |
| `docs/research/runtime/backend-scoped-route-requirement-answers.md` (3 sentences) | `research/runtime` |
| `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md` (2 sentences) | `contracts/decisions` |

The last is the one that makes this a boundary rather than a sweep: ADR 0092 is **accepted**, and its Decision item 8 and its open-questions paragraph both name `ResourceFloor` as a live type. A rename leaves accepted text naming a type that does not exist.

## What to decide

- The replacement name. `RouteResourceRequirement` matches the neighbouring `BackendFeatureRequirement` and the `RouteRequirement` enum it sits in; the cost is that `RouteRequirement::ResourceRequirement` reads redundantly, so the variant may want a different spelling from the struct.
- Whether accepted ADR 0092's two sentences are corrected in place with a marker (the `correct-adr-0074-driver-vocabulary-consumers` precedent, which corrects falsified factual claims inside an accepted ADR against measured source) or left with a note that the type was renamed after acceptance.

## Non-goals

Changing the relation, the wire tags, or any encoded byte — all of that landed with the correcting ticket, and this is a pure rename. Adding a relation to the wire. Adding a dimension.

## Closes when

No name in the workspace states a floor relation for a row the dimension compares by equality, the accepted ADR's text agrees with the types it names, and no encoded byte or artifact identity moved.
