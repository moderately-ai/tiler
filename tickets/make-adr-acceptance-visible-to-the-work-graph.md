---
id: make-adr-acceptance-visible-to-the-work-graph
title: The ticket graph cannot distinguish an ADR written from an ADR accepted
status: review
priority: p1
dependencies: []
related: [accept-adr-0077-metal-aot-crate-admission, accept-adr-0078-public-extension-seams]
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [planning, process, decisions]
claimed_from: todo
assignee: agent-tickets
lease_expires_at: 1784998335
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

## Outcome

**Decision: options 1 and 3 together, not one of them.** They were presented as alternatives and they are not — they close different failures, and neither closes the other's. Option 1 makes the *scheduling* failure structural: a ticket conditional on an unaccepted ADR is held out of the ready frontier by an edge. Option 3 makes the *silent contract corruption* failure detectable: a change that writes a proposal into a normative contract fails the gate. Choosing only option 1 leaves a worker free to write a proposal into a contract — the ticket's own tie-break names that as the failure to prevent first, and option 1 does not prevent it, because a worker doing adjacent work under a different ticket never touches the edge. Choosing only option 3 leaves the graph still dispatching tickets that cannot be done.

Option 1 landed here. Option 3 is scoped to `contracts/navigation`, which this ticket does not hold, and is split out as [`gate-proposed-decision-assertions`](gate-proposed-decision-assertions.md) with its full design, both predicates, both measurement boundaries, and the regression pair already measured. **This ticket is therefore `review`, not `done`:** the tie-break's preferred half is designed and measured but not implemented, and calling that done would overstate it.

**Why option 2 was not taken: because it is already true, and it is the wrong instrument.** Verified independently rather than inherited — `tkt states` lists `awaiting-decision` in the `parked` category, and `ticketsplease.toml` declares only `done` with `satisfies_dependents = true`, so a parked dependency never satisfies a dependent. The agent's use of it was correct. But it acts on the *dependent's own status*, so it requires a worker to already have been dispatched, to have read the ticket, and to have correctly diagnosed the block. That is a control that fires after the cost it is meant to avoid, and one that a less careful worker skips entirely. Option 1 uses the same state on a different node — the acceptance node — where it gates by dependency instead of by diagnosis.

**Measurement — the defect reproduces and the fix removes it.** At `b70da90`, setting `propagate-extension-seam-classification-into-governed-contracts` to `status: todo` while it depended on the drafting ticket put it in `tkt ready --format json` (1 match). After repointing it at `accept-adr-0078-public-extension-seams`, the same ticket at the same `todo` status is absent from `ready` (0 matches) and `tkt rollup` lists it under `blocked` as "waiting on: accept-adr-0078-public-extension-seams". The status field is doing none of that work; the edge is.

**That result is strictly better than parking the dependent.** A parked ticket leaves the board silently; a `todo` ticket behind a parked prerequisite appears in the blocked frontier with its cause named. The block became more visible, not less, while becoming structural.

### What landed

- [`accept-adr-0078-public-extension-seams`](accept-adr-0078-public-extension-seams.md) and [`accept-adr-0077-metal-aot-crate-admission`](accept-adr-0077-metal-aot-crate-admission.md), one per proposed ADR, permanently `awaiting-decision`, each stating that only Tom closes it and what each of accept / amend-then-accept / reject does to the graph.
- `propagate-extension-seam-classification-into-governed-contracts` repointed from the drafting ticket to the acceptance ticket and returned to `todo`, with its diagnostic section corrected — it previously said its status *was* the mechanism, which is no longer true.
- The convention recorded in `ticketsplease.toml` beside `[workflow.states.awaiting-decision]`, where a reader of the state definition meets it.
- [`gate-proposed-decision-assertions`](gate-proposed-decision-assertions.md) carrying option 3.

### Blast radius

**Measured, and it is smaller than the ticket's inference above predicted.** Two ADRs are `decision_status: "proposed"` (0077, 0078). Every ticket naming either was read in full, not grepped:

- **One true instance, the known one.** `propagate-extension-seam-classification-into-governed-contracts` — fixed here.
- **No second graph instance.** `grep -n 'dependencies:.*record-an-adr-for-the-metal-aot-crate-admission' tickets/*.md` returns nothing, so ADR 0077 has no dependent to mis-schedule. Its acceptance ticket was still created, because the record's own status line says the crate is already implemented and pinned while ADR 0056's retained AOT-invocation clause still stands — an undischarged supersession is a standing authority conflict whether or not a ticket depends on it, and it now has a node on the board.
- **Four near-misses, all correctly structured.** `test-two-revisions-of-one-provider-as-a-capability-ambiguity` is cited *by* ADR 0078 rather than conditional on it, and its deliverable — one regression test — survives the record being rejected. `correct-adr-0074-driver-vocabulary-consumers` and `correct-artifact-crate-lockstep-ir-permission` correct *accepted* ADRs against measured source and merely mention a proposed record as provenance. `record-distributivity-dimension-adr` names 0077 only as the highest ADR number. None needed changing.

**Retracted: the ticket's framing that "the blast radius is every proposed ADR".** As a count of latent instances that is right, and it is why the per-ADR acceptance node generalizes. As a description of the present board it overstates: only one of the two proposed ADRs has any dependent at all, and only one ticket was mis-scheduled. The corpus-wide claim in the third bullet was also checked rather than assumed — a probe over every `kind: contract` record found exactly one citation of a proposed decision, `docs/architecture.md:350` citing ADR 0077, and it discloses the status correctly. No contract currently asserts a proposal as fact, so option 3 lands green with no remediation pass ahead of it.

**Correction to a scope claim in the dispatch brief, recorded so it is not repeated.** The gate check was briefed as needing `implementation/workspace` because that scope "covers `scripts/`". It does not: `ticketsplease.toml` maps `scripts/docs.py` and `scripts/tests/**` to `contracts/navigation`, and `implementation/workspace` names only `check_workspace.py`, `check_rust.py`, `check_repository.py`, and `check_ci.py`. Since `scripts/check_repository.py:321` already invokes `scripts/docs.py validate`, the whole of option 3 is landable under `contracts/navigation` with no `implementation/workspace` edit at all. The follow-up ticket declares accordingly.
