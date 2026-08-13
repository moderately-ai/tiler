---
id: establish-hard-exceptional-value-evidence-for-metal-elementary-realizations
title: Establish hard exceptional-value evidence for Metal elementary realizations
status: in-progress
priority: p1
dependencies: [declare-elementary-realizations-on-a-target-profile]
related: [admit-the-registered-unary-families-at-the-compiler-request-boundary, require-both-elementary-evidence-halves-before-target-admission]
scopes: [research/apple-targets, research/numerics, implementation/compiler, implementation/build, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, metal, evidence]
claimed_from: todo
assignee: worker-exceptional-value
lease_expires_at: 1786664925
---
## User-visible outcome

The Metal target declares SiLU `exp`, RMSNorm `rsqrt`, and softmax `exp` only where each operation's exceptional-value behaviour has evidence strong enough to discharge the hard numerical contract. Unsupported rows remain unavailable with a typed reason instead of inheriting permission from empirical tests.

## Why this is separate

**Fact — the retained exceptional-value corpora are empirical.** They are valuable regression evidence, but [`ADR 0042`](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md) does not allow empirical evidence to discharge hard feasibility.

**Fact — the three elementary occurrences are not one interchangeable claim.** SiLU and softmax both use `exp`, but under different surrounding semantics; RMSNorm uses `rsqrt`. Their dtype, compiler mode, normal and exceptional domains, and required result behaviour must be stated independently.

**Inference — restoring availability requires evidence work, not an admission exception.** [`require-both-elementary-evidence-halves-before-target-admission`](require-both-elementary-evidence-halves-before-target-admission.md) intentionally makes the current rows refuse. This ticket owns the evidence needed to make any one of them admissible again.

## Required research and delivery

- Audit each operation occurrence against the exact accepted semantic contract, compiler flags, dtype, Metal language/toolchain row, and exceptional inputs it owes.
- Seek only admissible hard evidence classes: a sound proof, exhaustive finite evidence for a genuinely finite domain, or an applicable normative guarantee. Record `Unknown` where none is available.
- Keep empirical device observations as bounded qualification evidence and never rename them as normative or exhaustive evidence.
- If a primary source is inaccessible, record the exact citation, identifier, access failure, affected claim, and what disagreement could change; do not bypass the publisher's access control.
- After the public declaration path exists, install only the individually supported rows through that path. Unsupported operations remain absent and fail with `no-installed-realization` or `undischarged-evidence` as appropriate.
- Perturb every admitted exceptional-evidence subject independently and retain the typed refusal proving the check reaches that row.

## Stops

- A proposal to narrow or change the semantic contract is a separate numerical decision, not an evidence repair.
- A new compiler/toolchain/device environment creates a new bounded claim; it does not inherit this evidence.
- If no qualifying evidence exists, close the affected row as unsupported and preserve the reconsideration trigger rather than adding a fallback.

## Closes when

Each of the three Metal elementary occurrences has either a qualifying exceptional-value evidence record admitted through the validated public declaration or an explicit typed unsupported outcome with a reproducible reconsideration trigger.
