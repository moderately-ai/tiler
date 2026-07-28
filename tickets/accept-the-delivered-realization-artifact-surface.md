---
id: accept-the-delivered-realization-artifact-surface
title: Accept the delivered-realization artifact surface
status: awaiting-decision
priority: p1
dependencies: []
related: [record-delivered-numerical-realization, accept-adr-0076-numerical-realizations]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [api, decision, numerics, artifact, needs-tom]
---
## Decision needed (2026-07-28)

**The question, atomic:** accept the `DeliveredRealizationBuilder` + readers shape as staged — a constructor on `ArtifactProgramBuilder` plus readers on `VerifiedArtifactProgram` and `DecodedArtifact` — or name the items to change before it is made public?

| | Accept as staged | Hold |
| --- | --- | --- |
| **Enables** | `wire-the-delivered-realization-record-into-the-artifact` (currently `todo`, and blocked on nothing else) can start; the file-scope convention-7 allow at `crates/tiler-artifact/src/program/realization.rs:1-4` comes off and its 24 `pub(crate)` items become reachable; ADR 0076 item 4 stops being a requirement the code satisfies only internally. | The shape stays free to revise at no compatibility cost, which is exactly what convention 7 staged it for. |
| **Prevents** | The shape becomes a compatibility commitment; a later change to it is a breaking change rather than an edit. | ADR 0076 item 4 stays unmet in anything a consumer can observe — a produced artifact carries no readable record of the realization it delivered — and every dependent slice stays blocked. Nothing further can be done in the meantime: the draft cannot be reached from outside the crate by design (`mod realization;` is private at `crates/tiler-artifact/src/program/mod.rs:339`). |

Two shapes in the list below look like open sub-questions and are not; both were eliminated by derivation, and the derivations are here so a reader can refute them rather than re-ask them.

**No presentation `label()` on `HonouringMeansKey`, and adding one would be wrong.** ADR 0074 convention 2 offers a label so a wide digest can be read; a means key is already text a reader can render, so a label digesting it would make the record *less* readable. The convention is about the role of the value, and this value has the readable role already. (Reproduced from `record-delivered-numerical-realization`; that ticket's copy is the original.)

**`NumericalDimension` is deliberately not `#[non_exhaustive]`** under ADR 0074's amended convention 5b: this crate's encoder maps it totally and a wildcard there would have to invent an identity byte. A consumer rendering one line per dimension is the same case.

## Items to ratify

- **`DeliveredNumericalRealization`** — the record. Private fields, read through `profile()` and `honoured(dimension)`. One record per artifact, because `ArtifactProgramBuilder` already enforces one numerical contract and one target profile across every variant (`NumericalContractMismatch`, `TargetProfileMismatch`).
- **`NumericalDimension`** — the four behaviour dimensions of `tiler_ir::schedule::NumericalRealization`, not `#[non_exhaustive]` for the reason derived above.
- **`HonouringMeansKey`** — the opaque means key, `from_bytes` and `as_bytes()`, with no `label()`, for the reason derived above.
- **`HonouredDimensionFact`** — the per-dimension target fact: `means()` and `available_at()`.
- **`DeliveredRealizationBuilder`** — `new(profile)`, `declare(dimension, means, available_at)`, consuming `build()`.
- **`DeliveredRealizationError`** with `rule()`, **`HonouringMeansKeyError`**, and **`UnrecordedRealization`** with its `RULE` constant.
- **`MAX_HONOURING_MEANS_KEY_BYTES`**.
- On the artifact itself, whatever `wire-the-delivered-realization-record-into-the-artifact` adds: a builder entry point, `VerifiedArtifactProgram`'s reader, and `DecodedArtifact`'s reader.

Builder-error recovery and rejection-rule spelling follow existing artifact conventions or move to separately scoped reviews; they are not additional owner questions in this ticket.

## Counterpoint to accepting — the question this record does *not* reopen, and its trigger

Whether `tiler-artifact` should also expose a **typed view** of the means — a recognizer over a key it still does not mint — rather than opaque bytes alone. It is not asked now because no consumer branches on the means: comparison, identity, and rendering the key as text are all the opaque form supports and all anything needs today. It becomes forced the moment a consumer must *reason over* the means rather than compare it, and ADR 0076 item 4 names the likely one — the means "changes what a reference comparison should expect from a dimension honoured by emulation rather than natively." A comparator that can only compare bytes would hard-code `b"supported-with-exact-emulation"`, which is the second authority ADR 0076 line 58 forbids, arriving by copy instead of by declaration. **Trigger:** the first consumer that must branch on the means. The right response then is a recognizer whose unknown case is a real `None`, not a relocation of the vocabulary.

Accepting the opaque shape now is therefore not a bet that the typed view is unnecessary; it is the position that the recognizer should arrive with its first consumer, when the unknown case has a caller to be answered to.

## Why this is owner-reserved at all

ADR 0076 item 4 requires a produced artifact to carry a readable record of the numerical realization it delivered. `record-delivered-numerical-realization` built that record as a tested concrete draft in `crates/tiler-artifact/src/program/realization.rs` and staged it **crate-private** under ADR 0074 convention 7, because reaching it needs a constructor on the artifact builder and a reader on the verified and decoded artifact, and ADR 0075 reserves that surface to Tom. `AGENTS.md` states the same rule: "A tested implementation may serve as a concrete draft, but it is not implicit approval of its public interface."

## Parked 2026-07-27 — awaiting Tom

The implementation exists, is tested, and is staged `pub(crate)`. Nothing is public today: `mod realization;` is private and not re-exported, every item is `pub(crate)`, and the module carries the convention-7 file allow. The decision above is the only thing that moves this ticket.
