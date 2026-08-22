---
id: admit-the-selected-data-dependent-index-representation
title: Admit the selected data-dependent index representation
status: in-progress
priority: p1
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, decide-the-data-dependent-index-representation-public-surface]
related: [revise-adr-0108-with-a-complete-data-dependent-index-vertical, admit-an-invocation-scoped-gather-index-validation-receipt, emit-the-indirect-gather-on-metal]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, verification, identity, decision, needs-tom, public-boundary]
claimed_from: todo
assignee: worker-gather-index
lease_expires_at: 1787415365
---
## Status repair — 2026-08-19, this ticket was `blocked` with no surviving ground

Found by the ticket-population sweep and confirmed by the coordinator at `1c56a977`. Both declared dependencies are `done`: `accept-adr-0108-data-dependent-index-coordinate-siting` (ADR 0108 carries `decision_status: "accepted"`, accepted 2026-08-12) and `decide-the-data-dependent-index-representation-public-surface` (which records Tom's acceptance under its own `## Accepted decision — 2026-08-18` heading). The body below stated no blocking reason at all, and `git log -S "status: blocked"` attributes the status to `f01c1c92 tickets: gate data-dependent index public surface` — it was gating on exactly the decision that has since been accepted.

`.ticketsplease/decision-queue.md` item 12 already recorded the consequence on 2026-08-18 — "the p1 carrier `admit-the-selected-data-dependent-index-representation` is unblocked and joins the solo identity-migration queue" — but **the ticket's status was never flipped**, so a p1 sat parked for a day and held `emit-the-indirect-gather-on-metal` and the receipt tickets behind it. Status moved to `todo`; no other field changed.

**What is accepted, and what is still not.** Tom accepted option B — literal-only with the source-side `index_access` field — as the exact reviewed packet at `a25f4268b768f1b0391db34798676f910d4f1660`. That acceptance covers the public surface this ticket implements. It does **not** cover sourced boundary/domain gather support, which stays a separate future decision, and it authorizes no kernel, artifact, Metal, cache, or dispatch route past the KIR `body-refinement` wall. The `needs-tom` and `public-boundary` tags stay on the frontmatter because the *surface* is Tom's and must not be re-spelled by a worker; they are not a second unresolved gate.

**Scheduling.** This is a solo identity migration — it moves the `LogicalAccess` grammar and carries the named narrow ADR 0108 schedule-clause amendment — so it takes the solo migration slot and must not run beside other identity-moving work.

## User-visible outcome

The representation ADR 0108 ultimately accepts is admitted as a complete verified logical index form, while every existing direct-access byte and verifier guarantee remains unchanged.

## Required boundary

- Implement only the accepted nested-read or tagged-access form; do not blend the candidates.
- Carry the outer coordinate, nested source tensor, complete source coordinates, U32 value semantics, rank and reachability checks, exact bounds obligation, compaction/remapping, alpha-equivalence, canonical ordering, encoding, views, errors, reference evaluation, compiler recognition, and explanation as one coherent population.
- Preserve all old canonical bytes and pin every identity-domain step the accepted ADR requires.
- Retain the gather bound as either a static proof or one exact mandatory invocation-validation obligation. This ticket does not mint a runtime receipt and cannot treat an obligation as discharged.
- Keep direct access verification and ADR 0046 unchanged; scatter and data-dependent output shapes remain absent.

## Closes when

The selected form is constructed and inspected through the reviewed surface, all exhaustive consumers are updated, static proof reaches executable coverage, the dynamic form remains pending on the named receipt, subject perturbations independently fail, and targeted plus full gates pass.
