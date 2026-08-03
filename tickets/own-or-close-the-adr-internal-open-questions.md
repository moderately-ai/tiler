---
id: own-or-close-the-adr-internal-open-questions
title: Own or close the ADR-internal open questions that assign nobody
status: in-progress
priority: p3
dependencies: []
related: [re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal, resolve-or-retire-the-scalar-lowering-provider-seam, accept-the-public-backend-provider-composition-boundary]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, graph-repair]
claimed_from: todo
assignee: agent-adr-questions
lease_expires_at: 1785786696
---
## User-visible outcome

An open question living inside an accepted ADR has an owner, a stated trigger, or a durable answer — the same standard `docs/open-questions.md` is held to — so "No owner is assigned" stops being a durable state.

## Why this exists

**Fact — three questions state it verbatim.**

- [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md):143 — whether the mature per-operation fusion numerical capability is a registered third-party capability or a projection of the semantic definition. It records the real tension (an extension author knows whether their operation is an ordered reduction; a provider *asserting* a numerical permission is a claim the host cannot verify, the class ADR 0021 requires proof or runtime validation for) and offers a recommendation "offered rather than adopted". Ends: "No owner is assigned."
- ADR 0078:144 — whether `ScalarLoweringProvider` should reach the compile path at all. Ends: "No owner is assigned."
- [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):150 — whether the closed quantitative capability-axis set should become extensible. Records that item 1's elimination of pure data "rests partly on the axis set being closed", and that the CPU vertical's missing vector width, mask and tail support, scalable-vector length, cache levels, and thread count are "not merely undeclared but *inexpressible*". Ends: "No owner is assigned", and notes it depends on item 1's answer.

**Inference — the sweep that would catch these is scoped elsewhere.** [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) audits `docs/open-questions.md` under `contracts/navigation` and cannot reach `docs/decisions/`. An ADR-internal question is subject to the same rule AGENTS.md states — remove an open question only when its answer lives in a durable contract or an accepted ADR — and nothing applies it there.

## What each needs, stated so the ticket is writable

- **ADR 0078:144 is discharged by a live ticket.** [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md) runs exactly that elimination and must state which candidate survived. Point the question at it; do not answer it here.
- **ADR 0078:143 needs an owner or a trigger.** The recommendation is written but unadopted, so the honest outcomes are a ticket that adopts or refutes it, or a stated trigger — plausibly the first registered third-party capability claiming a fusion role. Do not adopt a recommendation by moving it out of the open-questions section; that is acceptance by relocation.
- **ADR 0090:150 is Tom's and gated.** It "depends on item 1's answer", and it subsumes the fourth seam ADR 0090:125 names under item 14 — whether a target triple, ABI, and data layout become `CapabilityAxis` values, which today "survive only inside a profile key string and payload provenance". Surface it as a decision with its dependency stated; **do not answer it**, and do not file implementation work behind it. It is a Tom decision, not fileable implementation.

## Boundaries

- Scope is `contracts/decisions`, plus `contracts/navigation` shared if a question is promoted into `docs/open-questions.md`. No code.
- Sweep the accepted ADRs for the same pattern rather than fixing only these three — find one bug, check all siblings. Reproduce with `grep -rn "No owner is assigned" docs/decisions/` and read each hit in full, since a question can be unowned without using that phrase.
- Closing a question requires its answer in a durable contract or an accepted ADR. A recommendation inside the question is not an answer.

## Closes when

`grep -rn "No owner is assigned" docs/decisions/` returns nothing, or returns only questions carrying a stated trigger a reader can evaluate; each of the three above has an owner, a trigger, or a durable answer; and ADR 0090:150 is recorded as Tom's with its gating dependency named rather than resolved.
