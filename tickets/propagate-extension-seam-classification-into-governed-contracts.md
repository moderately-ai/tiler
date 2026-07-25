---
id: propagate-extension-seam-classification-into-governed-contracts
title: Propagate the extension-seam classification into governed contracts
status: todo
priority: p2
dependencies: []
related: [accept-adr-0078-public-extension-seams, draft-public-extension-seam-ownership-adr]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions, public-api]
---
Conditional on Tom accepting ADR 0078. That record classifies which surfaces Tiler intends as public extension seams, which are permanently internal, the maturity rung each has reached, and — most of its content — what a seam is *not*. Until it is propagated, the classification lives only in a decision record and no governed contract states it, which is the state `implementation_status: "partial"` reports.

On acceptance, represent the classification in the contracts that own the affected areas, without creating a second authority over what ADR 0078 already decides:

- `docs/operation-extensions.md` owns the public capability surface and the trust, identity, registration, and diagnostic obligations of a provider. It should gain the seam classification and the negative-space rules that constrain a provider surface — offering nothing is a legitimate local result, a resolved provider's claim is re-derived rather than inherited, an unenumerated capability fails closed as `Unknown`, an absent capability and a contended one are different findings, a reservation is not a capability, and a provider revision is provenance rather than a version negotiation.
- `docs/architecture.md` owns component ownership and the packaging profile. It should record which authorities are permanently internal, and the qualification ADR 0078 makes about explain (internal authority, public obligation) and feasibility (internal procedure, with the target-profile data left explicitly undecided).

Do not restate ADR 0078's reasoning or its open questions in either contract; cite the record. Do not propagate anything ADR 0078 leaves unassigned — the physical-implementation provider and the mature fusion numerical capability are recorded as open questions and must not acquire an intent by propagation.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.

## Released 2026-07-25 — Tom accepted ADR 0078

**Fact.** `docs/decisions/0078-name-the-intended-public-extension-seams.md` is `decision_status: "accepted"`. Reproduce: `grep -n decision_status docs/decisions/0078-name-the-intended-public-extension-seams.md`. It was accepted unamended, so this ticket's body needs no revision — the instruction in the paragraph before this section still holds exactly as written, including its refusal to propagate the two surfaces ADR 0078 leaves as open questions.

**The dependency edge is retired rather than satisfied.** [`accept-adr-0078-public-extension-seams`](accept-adr-0078-public-extension-seams.md) is a node only Tom closes, and Tom's decision on it directs this ticket to be repointed off it rather than to wait for that closure. It moves to `related` so the reason this work was parked stays legible; nothing schedules against it any more.

**Why no part of this ticket landed before now.** Every deliverable it names is conditional — "On acceptance, represent the classification in the contracts that own the affected areas". Writing the classification into `docs/operation-extensions.md` and `docs/architecture.md` while the record was proposed would have converted a proposal into fact inside two governed contracts, which `AGENTS.md` forbids directly ("do not silently convert a proposal into fact") and which is the specific failure the record itself guards against. There was no unconditional remainder to split out: the ticket has no deliverable that would have survived the ADR being rejected or amended.

**The precedent is explicit.** `propagate-accepted-api-conventions-into-governed-contracts` is the same shape one ADR earlier, and it propagated only after ADR 0074 reached `decision_status: "accepted"` — its title names the accepted state. ADRs 0072, 0074, and 0075, which 0078 depends on, are all `accepted`, and 0078 now is too, so this ticket is in the same posture that one was when it propagated.

**This ticket was scheduled as ready, and that was wrong.** Its one dependency was `draft-public-extension-seam-ownership-adr`, which is `done` — correctly, since drafting a *proposed* ADR was its whole outcome. The dependency graph had no node representing Tom's acceptance, so nothing separated "the ADR has been written" from "the ADR has been decided", and the ticket surfaced in `tkt ready`. It reached a worker's queue that way, and the worker parked it by hand.

**How that was prevented while the decision was outstanding.** [`make-adr-acceptance-visible-to-the-work-graph`](make-adr-acceptance-visible-to-the-work-graph.md) replaced the drafting dependency with [`accept-adr-0078-public-extension-seams`](accept-adr-0078-public-extension-seams.md), a node that only Tom closes and that sat in `awaiting-decision`. A parked dependency never satisfies a dependent, so this ticket stayed out of the ready frontier structurally rather than by its own status, and `tkt rollup` named the acceptance node as the reason it was blocked. That mechanism did its job: the ticket was held until the decision existed, and it is released by the decision rather than by a worker's judgement.

**The trigger fired on 2026-07-25.** It was ADR 0078 reaching `decision_status: "accepted"`, and it did, unamended. Items 4 and 5 were re-read against the accepted text before the edge was retired: nothing in them moved, so the instruction above still forbids propagating the physical-implementation provider and the mature per-operation fusion numerical capability, both of which the accepted record still carries as open questions rather than classifications.
