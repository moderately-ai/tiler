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
