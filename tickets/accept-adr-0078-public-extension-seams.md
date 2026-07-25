---
id: accept-adr-0078-public-extension-seams
title: Accept or reject ADR 0078, the public extension seam classification
status: awaiting-decision
priority: p1
dependencies: [draft-public-extension-seam-ownership-adr]
related: []
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: []
paths: []
tags: [decisions, governance, extensions]
---
**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. It is the graph node standing for a decision that has not been made, and it exists so that every ticket conditional on that decision is held out of the ready frontier by a dependency edge rather than by a worker noticing the problem after being dispatched. Its permanent status is `awaiting-decision` — a `parked` category state that `tkt ready` excludes and that never satisfies a dependent — until the decision is taken.

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
