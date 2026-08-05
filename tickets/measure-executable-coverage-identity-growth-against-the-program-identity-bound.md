---
id: measure-executable-coverage-identity-growth-against-the-program-identity-bound
title: Measure executable-coverage identity growth against the program identity bound
status: review
priority: p3
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [performance, measurement]
claimed_from: todo
assignee: agent-identity-growth
lease_expires_at: 1785945150
---
## User-visible outcome

The cost curve of proof-bound stage coverage is measured rather than extrapolated, so the point at which an ordinary compilation would hit `MAX_PROGRAM_IDENTITY_BYTES` is a known graph size with a recorded procedure — or the growth is shown to stay far from the bound across realistic programs.

## Why this exists

**Inference (structural) + Measurement (one point), recorded by the independent review of `bind-stage-coverage-to-index-refinement-identity` at `cd3119f5`.** `SemanticGraphIdentity` is a full canonical `Vec<u8>` encoding, not a digest (`crates/tiler-ir/src/semantic/identity.rs:24`), and `encode_executable_coverage_identity` (`crates/tiler-ir/src/index/refinement.rs:2829`) embeds one whole graph identity per covered occurrence, one record per operation. Program identity is therefore Θ(operations × graph-encoding size) — quadratic in graph size — against a hard `MAX_PROGRAM_IDENTITY_BYTES = 64 MiB` (`crates/tiler-ir/src/program/mod.rs:429`) that fails closed with a typed refusal. The implementing worker measured a five-occurrence stage key at 21,366 bytes (~4 KB of evidence per occurrence) and ran no larger case.

The failure mode is fail-closed rather than silently wrong, which is why this is a measurement ticket rather than a defect: an over-large program refuses with a typed reason. What is unknown is how far from realistic program sizes that refusal sits.

## What this ticket owes

- A bounded experiment under `spikes/program-planning/`: identity byte size as a function of operation count over generated programs of increasing size, with the exact generator, environment, and procedure retained, and the observed growth curve compared against the structural Θ(n²) prediction.
- The graph size at which `MAX_PROGRAM_IDENTITY_BYTES` refuses, measured or safely extrapolated with the extrapolation labelled as such.
- A recorded verdict: either the bound is unreachable for the program sizes the roadmap contemplates (state the margin), or a follow-up decision ticket for a digest form — noting that replacing the embedded graph identity means changing the accepted `tiler.ir.index-refinement-executable-coverage.v1` projection, an identity-domain decision that is explicitly not this ticket's to take. The redundancy is provable at the program layer (the encoding folds its one bound graph identity separately and the builder proves every record names it), which is the fact a digest-form proposal would rest on.

## Closes when

The curve is measured and retained with its procedure, the refusal point is stated with its evidence class, and the verdict names either the margin or the follow-up decision ticket.
