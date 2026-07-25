---
id: accept-the-delivered-realization-artifact-surface
title: Accept the delivered-realization artifact surface
status: todo
priority: p1
dependencies: []
related: [record-delivered-numerical-realization, accept-adr-0076-numerical-realizations]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [api, decision, numerics, artifact, needs-tom]
---
ADR 0076 item 4 requires a produced artifact to carry a readable record of the numerical realization it delivered. `record-delivered-numerical-realization` built that record as a tested concrete draft in `crates/tiler-artifact/src/program/realization.rs` and staged it **crate-private** under ADR 0074 convention 7, because reaching it needs a constructor on the artifact builder and a reader on the verified and decoded artifact, and ADR 0075 reserves that surface to Tom. `AGENTS.md` states the same rule: "A tested implementation may serve as a concrete draft, but it is not implicit approval of its public interface."

Nothing is public today. `mod realization;` is private and not re-exported, every item is `pub(crate)`, and the module carries the convention-7 `#![allow(dead_code, reason = …)]`. `wire-the-delivered-realization-record-into-the-artifact` depends on this ticket because nothing outside the crate can construct or read a record until the facade is accepted.

## What would become public

- **`DeliveredNumericalRealization`** — the record. Private fields, read through `profile()` and `honoured(dimension)`. One record per artifact, because `ArtifactProgramBuilder` already enforces one numerical contract and one target profile across every variant (`NumericalContractMismatch`, `TargetProfileMismatch`).
- **`NumericalDimension`** — the four behaviour dimensions of `tiler_ir::schedule::NumericalRealization`, deliberately **not** `#[non_exhaustive]` under ADR 0074's amended convention 5b: this crate's encoder maps it totally and a wildcard there would have to invent an identity byte. A consumer rendering one line per dimension is the same case.
- **`HonouringMeansKey`** — the opaque means key, `from_bytes` and `as_bytes()`, with no `label()`. The elimination that produced this shape is recorded on `record-delivered-numerical-realization`.
- **`HonouredDimensionFact`** — the per-dimension target fact: `means()` and `available_at()`.
- **`DeliveredRealizationBuilder`** — `new(profile)`, `declare(dimension, means, available_at)`, consuming `build()`.
- **`DeliveredRealizationError`** with `rule()`, **`HonouringMeansKeyError`**, and **`UnrecordedRealization`** with its `RULE` constant.
- **`MAX_HONOURING_MEANS_KEY_BYTES`**.
- On the artifact itself, whatever `wire-the-delivered-realization-record-into-the-artifact` adds: a builder entry point, `VerifiedArtifactProgram`'s reader, and `DecodedArtifact`'s reader.

## Two shape questions worth confirming rather than inheriting

**The builder's failure does not return the builder.** ADR 0074 convention 4 requires a consuming terminal; ADR 0058's rationale for *recoverable* ownership is that a large arena-backed draft must be correctable rather than discarded. This draft is four slots and a profile reference, so the draft returns nothing and a caller re-declares. `ArtifactProgramBuilder::build` does return its builder, so this is a deliberate asymmetry inside one crate.

**`UnrecordedRealization` carries its rule as an associated constant, not a `rule()` method.** The rejection has no data to vary over, so a method would take a `self` it cannot read — Clippy's `unused_self` says so directly. Every sibling rejection in this crate spells it `rule()`, so the boundary now has both spellings for one role.

## The question this record does *not* reopen, and its trigger

Whether `tiler-artifact` should also expose a **typed view** of the means — a recognizer over a key it still does not mint — rather than opaque bytes alone. It is not asked now because no consumer branches on the means: comparison, identity, and rendering the key as text are all the opaque form supports and all anything needs today. It becomes forced the moment a consumer must *reason over* the means rather than compare it, and ADR 0076 item 4 names the likely one — the means "changes what a reference comparison should expect from a dimension honoured by emulation rather than natively." A comparator that can only compare bytes would hard-code `b"supported-with-exact-emulation"`, which is the second authority ADR 0076 line 58 forbids, arriving by copy instead of by declaration. **Trigger:** the first consumer that must branch on the means. The right response then is a recognizer whose unknown case is a real `None`, not a relocation of the vocabulary.
