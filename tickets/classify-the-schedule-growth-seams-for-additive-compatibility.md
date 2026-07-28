---
id: classify-the-schedule-growth-seams-for-additive-compatibility
title: Classify the schedule growth seams for additive compatibility
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
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

## Outcome — nine types classified from call sites, seven marked (2026-07-27)

The seam list was never enumerated anywhere, so rather than guess which six the parent's search seed meant, **every unmarked public enum in `tiler_ir::schedule::model` was classified** — nine types, a superset that cannot miss them.

**The verdict came from the compiler, not from reading.** All nine were marked `#[non_exhaustive]` at once and the workspace compiled; the failures name exactly the types with an out-of-crate exhaustive match. That is a measurement of current call sites, which is what the parent ticket's method asks for.

| verdict | types | evidence |
| --- | --- | --- |
| **stays exhaustive** | `TensorRole`, `ScalarProgram` | `TensorRole` breaks 2 out-of-crate matches (`tiler-compiler`'s `frontier.rs`, `physical.rs`); `ScalarProgram` breaks 1 (`physical.rs`) |
| **`#[non_exhaustive]`** | `ContributorOrder`, `LogicalAccess`, `BoundsProofKind`, `OwnershipProofKind`, `ExecutionBinding`, `TailPolicy`, `ReductionTopology` | every out-of-crate consumer constructs a variant or reads a field; none broke |

**"It compiled" was not accepted as sufficient.** `#[non_exhaustive]` does not constrain an `as` cast, and ADR 0074's amendment counts a discriminant cast as a total map when choosing a clause — so a type mapped by cast would compile cleanly and still be a 5b type. All seven were checked for an out-of-crate cast or `*_tag` function and none has one. The check was run against `AccessMode` first as a control, where it correctly found `access_mode_tag`.

**Both directions have compiling coverage**, per the ticket's method: a `compile_fail,E0004` doctest proving an out-of-crate exhaustive match on `TailPolicy` is an error, a positive doctest proving construction and a wildcard match still compile, and a positive doctest on `ScalarProgram` proving the out-of-crate exhaustive match it must keep supporting still does.

**Two process notes, because both were caught the hard way.**

- The negative doctest was verified to bite: removing `#[non_exhaustive]` from `TailPolicy` makes it **fail**, because the match then compiles. Without that check it would have been indistinguishable from a doctest that passes for the wrong reason.
- Naming `E0004` earned its keep immediately. A first attempt used a `TailPolicy::Masked` variant that does not exist; it failed with `E0599`, and a bare `compile_fail` would have recorded that as coverage. That is the exact failure the ticket warned about, reproduced.

**One search nearly produced a false result.** The first consumer sweep used `grep -r --include=*.rs`, which zsh silently matched to nothing, and reported that *none* of the nine had any out-of-crate consumer. A control against a type known to have them exposed it. Every count above comes from the re-run.
