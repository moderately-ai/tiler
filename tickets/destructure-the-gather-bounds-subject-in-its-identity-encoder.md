---
id: destructure-the-gather-bounds-subject-in-its-identity-encoder
title: Destructure the gather bounds subject in its identity encoder
status: todo
priority: p2
dependencies: []
related: [decide-how-the-oracle-independently-checks-a-gather-proof-identity, check-the-retained-gather-resolution-in-the-reference-evaluator]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, identity]
---
## User-visible outcome

Adding a field to `GatherIndexBoundsSubject` becomes a build error at the encoder that frames it, instead of compiling and silently producing a narrower gather proof identity.

## Why this exists

Filed 2026-08-22 by `worker-oracleid2` out of the readiness gate on [`decide-how-the-oracle-independently-checks-a-gather-proof-identity`](decide-how-the-oracle-independently-checks-a-gather-proof-identity.md). It is the cheap in-crate answer to the one dimension on which that packet's runner-up option beat the recommendation: drift.

**Fact — the encoder reads its subject by field access, so a widened subject compiles.** `encode_gather_bounds_identity` in `crates/tiler-ir/src/index/builder/gather.rs` reaches every component through `subject.region`, `subject.access`, `subject.source`, and so on. Nothing in the function is exhaustive over the struct. A field added to `GatherIndexBoundsSubject` — declared `pub(super) struct GatherIndexBoundsSubject` in `crates/tiler-ir/src/index/model.rs` — therefore enters the proof's retained subject and its public accessors while never entering the identity bytes, and no check anywhere fails.

This is the hazard AGENTS.md names under "Size enumerations from the type, not by hand": a hand-written field list satisfied by an enumeration that has stopped covering its domain. The struct cannot be sized by `variant_count`, but it can be destructured, which gives the same build-time guarantee.

## Required work

- Re-audit the Fact at your own base and report a verdict before editing.
- Destructure the subject at the top of `encode_gather_bounds_identity` — bind every field by name, with no `..` rest pattern — and write the bytes from the bindings. The byte output must be identical; this is a build-time guarantee, not an encoding change.
- Confirm the identity domain does not step. No pinned identity, golden, or ledger row may move. If any does, stop: that means the encoding changed and the change was not the intended one.
- Check whether `GatherIndexValidationRequirement`'s path through the same encoder needs the same treatment — it shares `encode_gather_bounds_identity` — and either cover it or say why it is already covered.
- Look for sibling encoders with the same shape. AGENTS.md's highest-signal rule is that finding one instance of a pattern means checking all of them; state which encoders you read and what you found, whether or not you change them.

## Non-goals

Changing what the identity encodes, its field order, or its domain tag. Any public surface change.

## Closes when

The encoder destructures its subject exhaustively; the emitted bytes are unchanged and that is demonstrated rather than asserted; the sibling-encoder sweep is reported; and a perturbation shows the alarm firing — add a field to `GatherIndexBoundsSubject`, quote the build error, and confirm it names the encoder.

## Coordinator re-audit at `3bf144f0`, 2026-08-22 — Fact verified, and narrowed in two ways that matter for the repair

Read in full at this base rather than relayed.

**Verified.** `encode_gather_bounds_identity` in `crates/tiler-ir/src/index/builder/gather.rs` reaches its subject entirely by field access and has no exhaustive construct over the struct. `GatherIndexBoundsSubject` in `crates/tiler-ir/src/index/model.rs` declares **exactly twelve** fields: `region`, `access`, `source`, `index`, `source_type`, `index_type`, `source_shape`, `index_shape`, `result_shape`, `axis`, `source_extent`, `domain`. A thirteenth would compile and silently stay out of the identity bytes.

**Narrowing 1 — all twelve are encoded today, so the byte-identity requirement is genuinely checkable.** I traced each field to its write: `region` through `push_slice`, the three ids through `bounded_index(..).to_be_bytes()`, the two types through `canonical_encoding()`, the three shapes through `push_shape`, `axis` and `source_extent` through `get().to_be_bytes()`, and `domain` through `push_len` plus one `bounded_index` per dimension. **No field is missing right now**, so this ticket is purely a build-time guarantee against future drift and the emitted bytes must not move. That makes "the bytes are unchanged" a real check rather than a formality — if your destructure changes any byte, you have reordered or dropped something and must stop.

**Narrowing 2 — the enum half is already drift-safe; only the struct half is exposed.** The `match kind` at the top of the same function covers `VacuousEmptyResultDomain => 0x01` and `U32RangeContainedBySourceExtent => 0x02` with **no wildcard arm**, so a third `GatherIndexBoundsProofKind` variant is already a build error at this encoder. Do not "fix" that half, and do not add a wildcard to it. State this in your sibling sweep so a later reader can see the asymmetry was deliberate: the enum is guarded by exhaustive matching, the struct needs destructuring to get the same property.

**On the perturbation the Closes-when requires.** Adding a field to `GatherIndexBoundsSubject` must produce a build error that names this encoder. Note the struct is `pub(super)`, so the perturbation is in-crate and cheap. Confirm the error actually names `encode_gather_bounds_identity` rather than only the construction site — a perturbation that reddens the constructor but not the encoder has not demonstrated the property this ticket exists for.

**Still blocked on scope at the time of this audit.** `implementation/ir` is held by `lower-and-emit-the-batched-cooperative-contraction`. This audit is recorded now so the ticket is dispatch-ready the moment that scope frees; re-audit at your own base regardless, since that lane edits this crate.
