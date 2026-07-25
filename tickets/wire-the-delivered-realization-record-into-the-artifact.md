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

- **Require the record on the builder.** The record is not optional. Every artifact that exists rests on declared honouring means — a dimension the profile does not speak to contributes ADR 0043 `Unknown` and never reaches an executable frontier, so a candidate with an undeclared dimension does not become an artifact. Making the field required puts that in the type instead of in a check, and leaves only one absent case: an envelope decoded from bytes.
- **Verify the record's profile against the artifact's.** `check_subject` already pins one `TargetProfileRef` across every variant. The record names the profile that declared its means, and the two must be the same profile; a mismatch is a typed rejection, not a tolerated duplication. This is the check that turns the record's copy of the profile from a second statement into evidence.
- **Fold `canonical_bytes` into `encode_identity`.** Two artifacts delivering one contract by different means are not the same artifact. This changes every artifact identity in the workspace, so the pinned and golden values that view it must be recomputed on the merged tree rather than taken from either side.
- **Carry it across the codec.** An envelope section, its encode, its decode, its budget, and its validation. A decoded envelope with no realization section must reject with `UnrecordedRealization` — `require_recorded` is the reader that already does this and is the only one. Decide whether the absence is refused through the existing feature-requirement mechanism (the shape `tiler.artifact.feature.multi-stage-program` uses) or at the section level, and record which.
- **Expose the readers.** `VerifiedArtifactProgram` returns the record directly, because construction made it total. `DecodedArtifact` returns a `Result`, because its bytes are untrusted.
- **Update the fixtures and the module doctest.** Every fixture in `crates/tiler-artifact` gains a record. The `program` module doctest must not teach a caller to mint a means key by hand — that is precisely the reconstruction ADR 0076 item 4 forbids — so it should say in the surrounding prose that `tiler-compiler`'s `HonouringMeans::key` is what mints one and that the literal stands in for a value the producer received.

## What this does not close

No producer supplies a means key. `tiler-compiler` composes honourability today (`crates/tiler-compiler/src/honourability.rs`, `feasibility.rs`) and mints the key with `HonouringMeans::key`, but nothing calls the artifact builder. Wiring the compiler to it needs `implementation/compiler` and is a separate slice; until it lands, the record is supplied only by this crate's own fixtures.

The per-fact provenance stays incomplete — see `carry-the-honourability-fact-provenance-into-the-artifact-record`.
