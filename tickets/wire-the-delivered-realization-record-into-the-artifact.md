---
id: wire-the-delivered-realization-record-into-the-artifact
title: Wire the delivered-realization record into the artifact
status: todo
priority: p1
dependencies: [accept-the-delivered-realization-artifact-surface]
related: [record-delivered-numerical-realization, redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: []
paths: []
tags: [implementation, artifact, numerics]
---
`record-delivered-numerical-realization` built the first delivered-realization record and staged it crate-private in `crates/tiler-artifact/src/program/realization.rs`. That four-dimension, dtype-free draft was later disproved and is historical evidence rather than the shape to wire. `redesign-the-delivered-realization-record-from-typed-evidence` owns its replacement. This ticket makes a produced artifact carry the replacement boundary Tom accepts, which is what ADR 0076 item 4 asks for and what a staged draft alone does not supply.

Blocked on `accept-the-delivered-realization-artifact-surface`: every step below adds a public item, and ADR 0075 reserves that to Tom.

## The work

- **Implement the accepted shared and producer boundaries.** Land the ratified shared scalar-arithmetic dimension/subject vocabulary, compiler evidence view, artifact record, and exhaustive `tiler-build` translation together. Do not leave sibling crates translating through copied tags or strings.
- **Make the record required and versioned.** Every executable artifact rests on
  declared honouring means. The builder and decoder must both reject an
  artifact that does not carry a validated record; decoded bytes are not a
  special optional case.
- **Verify the record's profile against the artifact's.** `check_subject` already pins one `TargetProfileRef` across every variant. The record names the profile that declared its means, and the two must be the same profile; a mismatch is a typed rejection, not a tolerated duplication. This is the check that turns the record's copy of the profile from a second statement into evidence.
- **Cross-check every existing realization statement.** Every packaged entry/variant references an existing policy subject, and each record resolution among the eight dimensions already carried by widened `NumericalFacts` equals every overlapping entry statement. Artifact construction and decode reject a missing subject, profile disagreement, dangling obligation/evidence reference, or overlapping behaviour mismatch. The compiler proves operation/policy-locus meaning; `tiler-build` proves its translation; the neutral artifact validates the encoded associations without pretending to re-run compiler consumption analysis.
- **Fold `canonical_bytes` into `encode_identity`.** Two artifacts delivering one contract by different means are not the same artifact. This changes every artifact identity in the workspace, so the pinned and golden values that view it must be recomputed on the merged tree rather than taken from either side.
- **Carry it across the codec.** Choose the canonical versioned encoding that
  fits the envelope's existing compatibility rules, with explicit budgets and
  validation. This ticket requires the behavior, not a particular section
  layout or feature-flag mechanism.
- **Reject unknown numerical families.** Unknown family, schema, subject, dimension, disposition, locus, means, or provenance tags fail closed. Compatibility never skips an unknown numerical contract and still treats the artifact as executable.
- **Update the durable authorities.** Refine ADR 0076's “each dimension's means” language to the accepted complete assessment-disposition contract, update the numerical and artifact contracts, and advance the exact artifact identity domain and manifest schema from their merged-tree authorities.
- **Expose total readers.** Both `VerifiedArtifactProgram` and
  `DecodedArtifact` return the record directly. Untrusted bytes are rejected by
  decoding; a successfully decoded artifact must not preserve an
  `UnrecordedRealization` state for every caller to rediscover.
- **Update the fixtures and the module doctest.** Every fixture in `crates/tiler-artifact` gains a record. The `program` module doctest must construct it through the accepted typed producer path; it must not teach a caller to invent opaque means bytes or provenance.

## Closes when

1. Every executable artifact carries a validated, versioned record: the builder refuses to produce one without it and the decoder refuses to accept one without it, with decoded bytes given no optional path of their own.
2. The record's profile is checked against the artifact's single `TargetProfileRef` and a mismatch is a typed rejection, so the record's copy of the profile is evidence rather than a second statement.
3. `canonical_bytes` is folded into `encode_identity`, two artifacts delivering one contract by different means have different identities, and every pinned or golden identity is **recomputed on the merged tree** rather than taken from either branch.
4. The record crosses the codec under the envelope's existing compatibility rules, with explicit budgets and validation on decode.
5. `VerifiedArtifactProgram` and `DecodedArtifact` both return the record directly — total readers, with no `UnrecordedRealization` state surviving a successful decode for callers to rediscover.
6. Every `tiler-artifact` fixture carries a record, and the `program` module doctest uses the accepted typed producer path without inventing means or provenance.
7. The convention-7 file allow at `crates/tiler-artifact/src/program/realization.rs:1-4` is removed or narrowed to whatever remains genuinely unreached, and `make full` passes.

## What this does not close

This ticket does not redesign the compiler evidence, provenance vocabulary, or artifact record; its dependencies deliver and ratify those. It wires the accepted shape, advances the exact merged-tree domains and schema, and rebaselines the resulting identities.

## User-visible outcome

An artifact consumer can determine which declared numerical realization the
artifact delivers, and two otherwise identical artifacts with different
honouring means have different canonical identities. Missing, malformed, or
profile-mismatched records fail during construction or decode.
