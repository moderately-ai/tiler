---
id: accept-adr-0078-public-extension-seams
title: Accept or reject ADR 0078, the public extension seam classification
status: done
priority: p1
dependencies: [draft-public-extension-seam-ownership-adr]
related: [propagate-extension-seam-classification-into-governed-contracts]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, governance, extensions]
claimed_from: todo
assignee: agent-dec3
lease_expires_at: 1785005390
---
**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. It is the graph node standing for a decision that has not been made, and it exists so that every ticket conditional on that decision is held out of the ready frontier by a dependency edge rather than by a worker noticing the problem after being dispatched. Its permanent status is `awaiting-decision` — a `parked` category state that `tkt ready` excludes and that never satisfies a dependent — until the decision is taken.

**The Facts and the "Closes when" section below record the board as it stood before Tom decided, and the parts of them that assert a `proposed` status were true then and are false now.** They are kept because they are what the decision was taken against; the Outcome at the foot of this ticket records what the corpus says instead. Each reproduction command still resolves, and now returns the accepted state.

**Fact.** [`docs/decisions/0078-name-the-intended-public-extension-seams.md`](../docs/decisions/0078-name-the-intended-public-extension-seams.md) carries `decision_status: "proposed"`, and its status line reads "proposed. Tom accepts; nothing here is operative until he does." Reproduce: `grep -n decision_status docs/decisions/0078-name-the-intended-public-extension-seams.md`.

**Fact — the drafting ticket is correctly `done` and is not this node.** [`draft-public-extension-seam-ownership-adr`](draft-public-extension-seam-ownership-adr.md) delivered exactly what it promised: a written, evidenced, proposed record. Writing a proposal is a completed outcome. Nothing about that ticket's completion is evidence that the proposal was adopted, and this ticket is the node that carries the difference.

## What Tom is deciding

The record classifies which compiler surfaces Tiler intends as public extension seams, which are permanently internal, the maturity rung each has reached, and what a seam is *not*. Its Decision has six numbered items; item 5 defers the physical-implementation provider on a sharpened trigger, and item 6 names the permanently internal authorities.

Two of the record's own open questions are marked **"This is Tom's"** and one of them gates the rest of it:

- Whether the sharpened trigger in item 5 restates Tom's intent or narrows it. The record says one sentence settles it, and that the answer decides whether the physical-provider classification is ripe now or still waiting.
- Whether target-specific scheduling knowledge is typed target-profile data or code in backend crates. This is prior to, and decides, `frontier::PhysicalImplementationProvider`'s visibility.

The remaining three open questions carry no owner and are recorded unresolved on purpose; accepting the record does not settle them and must not be read as settling them.

## Closes when

`decision_status` in `docs/decisions/0078-name-the-intended-public-extension-seams.md` moves off `proposed`, the record's status line is rewritten to match, `uv run --locked python scripts/docs.py render` regenerates `docs/decisions/README.md`, and `uv run --locked python scripts/check_repository.py` passes.

Three outcomes are available and each has a different consequence for the graph:

- **Accepted.** Set `decision_status: "accepted"`. `docs/decisions/README.md` regenerates, and [`propagate-extension-seam-classification-into-governed-contracts`](propagate-extension-seam-classification-into-governed-contracts.md) becomes ready exactly as written — its body needs no revision. Note that `scripts/docs.py`'s graph validation requires an accepted decision to carry both `applies_to` and `evidence`; ADR 0078 already has both.
- **Amended, then accepted.** Re-read items 4 and 5 of the propagation ticket before it is dispatched. It forbids propagating the physical-implementation provider and the mature fusion numerical capability, which the record leaves as open questions, and an amendment is the most likely way that instruction goes stale.
- **Rejected.** Close this ticket with `tkt close` rather than `done`, so it does not satisfy its dependents, and close the propagation ticket too — that ticket has no deliverable that survives the record being rejected.

`docs/architecture.md` and `docs/operation-extensions.md` currently state nothing that ADR 0078 decides, which is the correct state while it is proposed and is what its `implementation_status: "partial"` reports. Do not treat this ticket as blocked on anything but the decision itself.

## Decision — Tom, 2026-07-25

**Accepted.** `decision_status` moves `proposed` → `accepted`. This releases `propagate-extension-seam-classification-into-governed-contracts`, which was correctly parked and refusing to dispatch: writing a proposed ADR into a normative contract is precisely what the new Check A now refuses.

Repoint that ticket off the acceptance node and return it to the ready pool.

## Outcome

**Accepted by Tom, 2026-07-25.** `decision_status` moved `proposed` → `accepted` on `docs/decisions/0078-name-the-intended-public-extension-seams.md`.

**The dependent is released.** `propagate-extension-seam-classification-into-governed-contracts` depends on this node and is now dependency-satisfied, back in the ready pool. It was correctly parked before: writing a proposed ADR into a normative contract is the failure `make-adr-acceptance-visible-to-the-work-graph` was filed to make structural, and the check that catches it silently is `gate-proposed-decision-assertions`.

That parking is the acceptance-node mechanism working as designed — the ticket surfaced as ready from its *drafting* dependency alone, an agent diagnosed it rather than writing the proposal into a contract, and the node now exists so the next instance cannot dispatch at all.
## Outcome — landed 2026-07-25, awaiting Tom's close

**Status is `review`, not `done`, deliberately.** The standing rule at the head of this ticket — only Tom closes it, and no agent sets it `done` — was not lifted by the Decision section he added below it. It is also why the Decision directs the dependent to be *repointed* rather than left to be satisfied by this node closing: an edge into a node an agent may not close cannot be the thing that releases the work.

### What moved

`docs/decisions/0078-name-the-intended-public-extension-seams.md` carries `decision_status: "accepted"`, and its status line records acceptance on 2026-07-25 with the decision unchanged from the proposed text. No item of the six-item Decision was amended, so items 4 and 5 of the propagation ticket are not stale and that ticket needed no revision to its instructions — the check its own body asked for before dispatch.

`implementation_status` stays `"partial"`, which is correct and load-bearing: the governing principle in item 1 is realized by every seam in item 2, while the classification itself is stated in no governed contract until the propagation lands, and items 4 and 5 name two surfaces whose posture is recorded as unresolved rather than implemented. Acceptance did not move any of that.

**The status line now says what acceptance does not settle.** All five open questions remain open, including the two marked as Tom's — whether the sharpened trigger in item 5 restates his intent or narrows it, and whether target-specific scheduling knowledge is typed target-profile data or code in backend crates. `frontier::PhysicalImplementationProvider` and the mature per-operation fusion numerical capability stay deliberately unassigned, and the record's Implementation boundary now states that the propagation is released rather than pending, with the standing prohibition on either surface acquiring an intent by propagation restated at the point a propagator will read it.

### Disclosure sites

None. `validate_proposal_disclosure` fires on an inline link from a `kind: contract` record to a decision with `decision_status: "proposed"`; no contract record links ADR 0078 at all, which is exactly what this ticket's body predicted — `docs/architecture.md` and `docs/operation-extensions.md` state nothing ADR 0078 decides, and that was the correct state while it was proposed. Accepting it therefore created no stale disclosure to correct. The only citations outside the decision corpus were tickets, and `draft-public-extension-seam-ownership-adr`'s Outcome, which reported the record as delivered `proposed`, now names the acceptance node as the separate thing that decided it.

### The dependent, released

[`propagate-extension-seam-classification-into-governed-contracts`](propagate-extension-seam-classification-into-governed-contracts.md) declared exactly one dependency, this node. It now declares none and names this node under `related`, so the reason it was parked stays legible while nothing schedules against it. Its parked-state narrative was rewritten from a standing explanation into a record of a trigger that fired, and its stale claim that ADR 0078 "is the only one that is not" accepted among 0072, 0074, 0075, and 0078 is corrected. Its deliverables are untouched: no part of that ticket's work was done here.

### Landed with ADR 0077's acceptance, because the base was red without both

**Measurement.** `git archive 63b02ec | tar -x -C <dir>` then `uv run --locked python scripts/docs.py validate --root <dir>` reports two errors at the dispatch base: this node and `accept-adr-0077-metal-aot-crate-admission` each depend on the ticket that drafted a still-`proposed` ADR, and commit `63b02ec` had moved both out of `awaiting-decision`, which is the parked category `validate_tickets` exempts. Accepting one record clears one error and leaves the other, so no commit that accepts only one is green. The two acceptances therefore land as one commit; that is forced by the gate, not chosen for convenience, and it holds independently of their sharing one generated catalog.

**Guard.** `tkt guard` is branch-scoped, so guarding this branch as this ticket reports `contracts/foundation` under-declared — that scope is reached only by ADR 0077's `docs/architecture.md` disclosure correction, which is the other half of the same commit. Nothing under `contracts/foundation` was touched on this ticket's behalf and the scope is deliberately not added here. Guard the branch as `accept-adr-0077-metal-aot-crate-admission`, which declares the union and reports `WARN` for declared-area overlap only.

### Gate

`uv run --locked python scripts/docs.py render` and the full `uv run --locked python scripts/check_repository.py` both pass; `git diff --check` is clean and `tkt lint` reports no problems.
