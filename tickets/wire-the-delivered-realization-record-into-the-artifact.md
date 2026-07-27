---
id: wire-the-delivered-realization-record-into-the-artifact
title: Wire the delivered-realization record into the artifact
status: todo
priority: p1
dependencies: [accept-the-delivered-realization-artifact-surface]
related: [record-delivered-numerical-realization]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, artifact, numerics]
---
`record-delivered-numerical-realization` built the delivered-realization record and staged it crate-private in `crates/tiler-artifact/src/program/realization.rs`. This ticket makes a produced artifact actually carry it, which is what ADR 0076 item 4 asks for and what the draft alone does not supply.

Blocked on `accept-the-delivered-realization-artifact-surface`: every step below adds a public item, and ADR 0075 reserves that to Tom.

## The work

- **Make the record required and versioned.** Every executable artifact rests on
  declared honouring means. The builder and decoder must both reject an
  artifact that does not carry a validated record; decoded bytes are not a
  special optional case.
- **Verify the record's profile against the artifact's.** `check_subject` already pins one `TargetProfileRef` across every variant. The record names the profile that declared its means, and the two must be the same profile; a mismatch is a typed rejection, not a tolerated duplication. This is the check that turns the record's copy of the profile from a second statement into evidence.
- **Fold `canonical_bytes` into `encode_identity`.** Two artifacts delivering one contract by different means are not the same artifact. This changes every artifact identity in the workspace, so the pinned and golden values that view it must be recomputed on the merged tree rather than taken from either side.
- **Carry it across the codec.** Choose the canonical versioned encoding that
  fits the envelope's existing compatibility rules, with explicit budgets and
  validation. This ticket requires the behavior, not a particular section
  layout or feature-flag mechanism.
- **Expose total readers.** Both `VerifiedArtifactProgram` and
  `DecodedArtifact` return the record directly. Untrusted bytes are rejected by
  decoding; a successfully decoded artifact must not preserve an
  `UnrecordedRealization` state for every caller to rediscover.
- **Update the fixtures and the module doctest.** Every fixture in `crates/tiler-artifact` gains a record. The `program` module doctest must not teach a caller to mint a means key by hand — that is precisely the reconstruction ADR 0076 item 4 forbids — so it should say in the surrounding prose that `tiler-compiler`'s `HonouringMeans::key` is what mints one and that the literal stands in for a value the producer received.

## What this does not close

No producer supplies a means key. `tiler-compiler` composes honourability today (`crates/tiler-compiler/src/honourability.rs`, `feasibility.rs`) and mints the key with `HonouringMeans::key`, but nothing calls the artifact builder. Wiring the compiler to it needs `implementation/compiler` and is a separate slice; until it lands, the record is supplied only by this crate's own fixtures.

The per-fact provenance stays incomplete — see `carry-the-honourability-fact-provenance-into-the-artifact-record`.

## User-visible outcome

An artifact consumer can determine which declared numerical realization the
artifact delivers, and two otherwise identical artifacts with different
honouring means have different canonical identities. Missing, malformed, or
profile-mismatched records fail during construction or decode.
