---
id: propagate-extension-seam-classification-into-governed-contracts
title: Propagate the extension-seam classification into governed contracts
status: awaiting-decision
priority: p2
dependencies: [draft-public-extension-seam-ownership-adr]
related: []
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

## Not started — awaiting Tom's decision on ADR 0078

**Fact.** `docs/decisions/0078-name-the-intended-public-extension-seams.md` is `decision_status: "proposed"`, and its own status line reads "proposed. Tom accepts; nothing here is operative until he does." Reproduce: `grep -n decision_status docs/decisions/0078-name-the-intended-public-extension-seams.md`.

**Why no part of this ticket landed.** Every deliverable it names is conditional — "On acceptance, represent the classification in the contracts that own the affected areas". Writing the classification into `docs/operation-extensions.md` and `docs/architecture.md` while the record is proposed would convert a proposal into fact inside two governed contracts, which `AGENTS.md` forbids directly ("do not silently convert a proposal into fact") and which is the specific failure the record itself guards against. There is no unconditional remainder to split out: the ticket has no deliverable that survives the ADR being rejected or amended.

**The precedent is explicit.** `propagate-accepted-api-conventions-into-governed-contracts` is the same shape one ADR earlier, and it propagated only after ADR 0074 reached `decision_status: "accepted"` — its title names the accepted state. ADRs 0072, 0074, and 0075, which 0078 depends on, are all `accepted`; 0078 is the only one that is not.

**This ticket is scheduled as ready, and that is wrong.** Its one dependency, `draft-public-extension-seam-ownership-adr`, is `done` — correctly, since drafting a *proposed* ADR was its whole outcome. The dependency graph has no node representing Tom's acceptance, so nothing separates "the ADR has been written" from "the ADR has been decided", and the ticket surfaces in `tkt ready`. It reached a worker's queue that way. Status is therefore `awaiting-decision` rather than `todo`, so the parked category keeps it out of the ready frontier until the gate below clears.

**Trigger for reconsideration.** ADR 0078 reaching `decision_status: "accepted"`. On acceptance, this ticket is unblocked exactly as written; nothing in its body needs revision. If Tom amends the classification before accepting it, re-read items 4 and 5 first — the ticket forbids propagating the physical-implementation provider and the mature fusion numerical capability, which the record leaves as open questions, and an amendment is the most likely way that instruction would go stale.
