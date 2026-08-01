---
id: re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal
title: Re-own or close the open questions whose owner tickets are terminal
status: todo
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Every question in `docs/open-questions.md` either has a live owner, a stated demand trigger, or is closed into the durable contract that answers it — so no question sits owned by a terminal ticket, which is ownership in name and orphanhood in fact.

## Why

**Fact — the failure mode has already happened once.** Q-ART-008 was owned by `prototype-artifact-family-delivery`, which closed `done` with the close condition unmet, leaving the question unowned until a later worker noticed. A 2026-07-31 audit (reproduce with the script in this ticket's provenance, or re-derive: extract each `Q-*` section's `tickets/*.md` references and compare against ticket statuses) found eight questions whose every referenced ticket is now `done` or `closed`: Q-SEM-001, Q-SEM-007, Q-PLAN-001, Q-PLAN-007, Q-PLAN-009, Q-ART-002, Q-PKG-002, and Q-PKG-003 — the last owned by `prototype-inline-proc-macro-frontend`, which closed the same day. Thirty-three further questions reference no ticket at all; most are deliberate demand-triggered reservations, but nothing distinguishes a stated trigger from an accidental orphan except reading.

## Work

For each of the eight: read the question against what its terminal owner actually delivered; close it into the contract that now answers it, re-point it at the live successor ticket, or record why it stays open with a stated trigger. For the thirty-three unreferenced: verify each states a closure trigger a reader can evaluate; give any that does not either a trigger or an owner. Do not close a question whose answer does not live in a durable contract or an accepted ADR.

## Closes when

The audit re-run reports zero questions owned solely by terminal tickets, and every unreferenced question carries an explicit trigger.
