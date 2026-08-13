---
id: prove-one-live-extent-artifact-payload-and-pipeline-at-two-n
title: Prove one live-extent artifact payload and pipeline at two N
status: todo
priority: p1
dependencies: [carry-live-extent-operands-through-the-artifact-envelope]
related: [admit-live-extent-operands-to-payload-indexing, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/artifact, implementation/build, implementation/metal, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity, runtime, metal]
---
## User-visible outcome

One compiled artifact, payload subject, and pipeline indexes dense F32 `[2,N]` from the bound input extent, so changing `N` changes the addressed byte without compiling another payload or pipeline.

## Exact gap

This is the parent ticket's first required evidence, which `9a8f53c9` did not produce because the artifact envelope row does not exist yet.

## Required work

- One artifact, payload subject, and pipeline handles dense F32 `[2,N]` at `N = 14` and `N = 15`.
- Semantic `(row = 1, column = 0)` addresses bytes 56 and 60 respectively from the bound input extent.
- Baking either value changes identity and fails the no-specialization assertion. The live value is excluded from artifact, payload, library, and pipeline identity.
- Existing range and launch expressions resolve from the same bound fact. A deliberate disagreement refuses before program work.

## Required evidence

- Both extents execute through one payload and one pipeline, with the two address oracles observed.
- Identity of the artifact, payload, library, and pipeline is equal across the two bindings and unequal to a baked neighbour.
- Targeted build, Metal, runtime, and artifact tests, exact identity blast radius, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Envelope construction is the dependency. `LiveContraction` contributor-loop evidence is [`prove-a-schedule-verified-live-contraction-consumes-s`](prove-a-schedule-verified-live-contraction-consumes-s.md). Inline AOT `deliver` lifting is [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Closes when

Both `N` values run from one identity, the two byte addresses are observed, and specialization is a failing identity check rather than a description.
