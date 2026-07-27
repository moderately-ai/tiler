---
id: classify-the-schedule-growth-seams-for-additive-compatibility
title: Classify the schedule growth seams for additive compatibility
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [api]
---

Split from `harden-public-enums-non-exhaustive`, which classified the four Metal AOT boundary types and `IndexExprClass` from their call sites. Read that ticket's Outcome section first — it establishes the method and the two doctest shapes that prove a verdict bites.

The six schedule growth seams were in that ticket's search seed and were deliberately not swept in on the strength of the four types beside them: they are a distinct vocabulary with distinct consumers, and the seed says in terms that it is not a closed authority.

## Method, which is the part worth reusing

For each type, the verdict is decided by **current call sites**, not by declaration:

- `#[non_exhaustive]` only when every out-of-crate consumer is partial or forwarding — it names a variant to construct, or reads a field — and none classifies by exhaustive match.
- Total identity maps, support recognizers, and closed vocabularies stay exhaustive, so a new variant is a compile error at every authority that must classify it. Note the asymmetry does the work: `#[non_exhaustive]` has no effect inside the defining crate, so internal total maps keep breaking exactly as they should.
- A type with **no** out-of-crate consumer gets left alone. Marking it is a type-system reservation protecting nobody, and `AGENTS.md` separates that from an implemented seam.

## Negative coverage

`compile_fail` doctests, not unit tests: a doctest compiles as a separate crate, so `#[non_exhaustive]` applies to it, where a unit test in the same crate would see nothing. Name the expected error code — `E0004` for an exhaustive match on a non-exhaustive enum, `E0639` for a non-exhaustive struct literal. In the parent ticket the first attempt failed with `E0432` instead, an unresolved import, and only the named code revealed it; a bare `compile_fail` would have passed while testing nothing.

Pair every negative case with a positive one proving construction and field reads still compile, so the attribute cannot be over-applied without the gate noticing.

## Closes when

Each of the six seams is classified from its current call sites with the consumer named, compatible seams are `#[non_exhaustive]`, total maps and recognizers remain exhaustive, negative and positive compile coverage protects both directions, and `make full` passes.
