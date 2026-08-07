---
id: declare-evaluation-order-preservation-in-the-target-profile
title: Declare evaluation-order preservation in the target profile
status: in-progress
priority: p2
dependencies: []
related: [measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order, admit-a-refutation-only-derived-bound-conformance-oracle]
scopes: [implementation/metal, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, target-profiles, numerics, public-boundary]
claimed_from: todo
assignee: agent-eval-order
lease_expires_at: 1786066716
---
## User-visible outcome

A target profile declares, per math mode, whether the backend compiler preserves an emitted floating-point evaluation order, so the permitted-divergence oracle's pinned-order premise is consulted from a declared fact instead of being asserted by the flags Tiler happens to pass. Today `MetalTargetFacts` (five fields) and `CapabilityAxis` (seven) declare nothing about it, which [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md)'s item 5 records as the gap.

## The measured basis

[Finding 34](../docs/research/apple-targets/numerical-behaviour.md): on the named macOS row, an emitted two-by-two split is re-serialized under `relaxed` and `fast` on both compilation paths and preserved under `safe` in every measured cell. The declaration this ticket adds is honest exactly per row and mode: `Preserved` only where measured, `Unknown` elsewhere (including the qualified numerical row, which finding 34 was not taken on — re-measuring there is in scope or explicitly deferred with the toolchain-authorization constraint named).

## Boundary

A new target-profile field is a public surface under ADR 0075: implement as a labelled draft with an acceptance node parked for Tom, stepping the complete-declaration domain version only if previously-encodable bytes move (an appended row family under per-tag framing does not). Silence must stay fail-closed: a profile that declares nothing about the property answers `Unknown`, which never reaches an executable frontier.

## Closes when

The field exists as a draft with its node, the macOS row declares the measured values with finding 34 as provenance, every other profile answers `Unknown`, and the oracle derivation's item 5 cites the declaration rather than the absence.
