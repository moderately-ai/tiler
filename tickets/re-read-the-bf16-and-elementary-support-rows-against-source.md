---
id: re-read-the-bf16-and-elementary-support-rows-against-source
title: Re-read the BF16 and elementary support rows against source
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-navigation-3
lease_expires_at: 1786031910
---
## The work (maturity audit 2026-08-06, key claims coordinator-verified: KernelConstant::Bf16Bits exists; the four named tickets are done)

BF16 row (`roadmap.md:477`): correct the three vocabulary facts the source refutes (KernelConstant/BinaryOp carry BF16; a BF16 VerifiedKernel exists and bit-agrees; a producer-built artifact round-trips), recount the live tickets from six to two (establish-bf16-optimizer-legality, conform-the-bf16-vertical-end-to-end), keep R4 while stating the non-monotone evidence above it. SiLU row (`:469`): the named wall (admit-the-registered-unary-families...) is done and ADR 0099 is accepted uncited — decide and state the evidence bar (R6 with unit-level emission evidence, or R5 naming the missing compiled golden) rather than leaving a closed ticket as the blocker. RMS-norm/softmax rows (`:470/:471`): replace the closed two-region ticket with the true blocker — no IndexRealizationLaw registered for either family (registry.rs:2391-2434) — and note the sequence surface acceptance. `roadmap.md:555`'s wall list names three done tickets; annotate or replace each.

## Closes when

Each row's claims reproduce against source, the evidence-bar decision is stated not silent, and no closed ticket is named as a live wall.
