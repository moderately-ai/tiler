---
id: make-adr-acceptance-visible-to-the-work-graph
title: The ticket graph cannot distinguish an ADR written from an ADR accepted
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [planning, process, decisions]
claimed_from: todo
assignee: agent-graph
lease_expires_at: 1784998055
---
Found by an agent that reached a ticket as `ready` and could not do it.

**Fact.** `propagate-extension-seam-classification-into-governed-contracts` declares exactly one dependency: the ticket that *drafts* ADR 0078. That drafting ticket is correctly `done` — the ADR was written. But ADR 0078 is `decision_status: "proposed"` and states that nothing in it is operative until the owner accepts it.

**Inference.** Every deliverable of the propagate ticket is conditional on an acceptance that has not happened, so doing the work would convert a proposal into fact — precisely what `AGENTS.md` forbids: "Proposed ADRs and proposed design documents are coherent hypotheses, not commitments and not evidence that Tom personally approved every detail." The agent correctly parked it rather than proceeding.

**The defect is structural, not local to that ticket.** `tkt ready` computes readiness from dependency status alone, and a drafting ticket being `done` is indistinguishable from the decision being made. So any ticket that propagates a proposed ADR into governed contracts will surface as ready and waste a worker, and the failure mode is silent: an agent less careful than this one would have written the proposal into a normative contract and passed the gate, because no check compares a contract's claims against the `decision_status` of the ADR it cites.

**Inference — the blast radius is every proposed ADR.** `grep -c 'decision_status: "proposed"' docs/decisions/*.md` gives the current count; each is a latent instance the moment anything depends on it.

## Scope

Make acceptance expressible. Options, none yet chosen:

- A propagation ticket depends on an explicit *acceptance* ticket rather than on the drafting ticket, so the graph has a node that only the owner can close.
- ~~`tkt` gains a status meaning parked-pending-external-decision~~ — **checked, and it already exists.** `tkt states` lists `awaiting-decision` in the `parked` category, which `ready` excludes, so the agent's use of it was correct rather than ad hoc and this option needs no work. It stops the wasted dispatch *after* a worker has read the ticket and diagnosed the block, which is the cheap half of the problem and not the dangerous half.
- A repository-gate check that a normative contract does not assert something whose only source is an ADR with `decision_status: "proposed"`. This is the only option that catches the silent failure rather than the scheduling waste, and it is also the most work.

Decide which, and say why the others were not taken. Prefer the option that makes the *silent* failure impossible over the one that merely stops the wasted dispatch, unless the cost is disproportionate — and say which you judged.

## Closes when

A ticket gated on an unaccepted ADR cannot surface as `ready`, or a gate check refuses a contract that asserts a proposed decision as fact; the choice among the options is stated with its reason; and `uv run --locked python scripts/check_repository.py` passes.
