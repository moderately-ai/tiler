---
id: gate-proposed-decision-assertions
title: Refuse a contract that asserts a proposed decision as fact
status: in-progress
priority: p1
dependencies: [make-adr-acceptance-visible-to-the-work-graph]
related: []
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [documentation, validation, decisions, governance]
claimed_from: todo
assignee: agent-nav3
lease_expires_at: 1785000986
---
Split out of [`make-adr-acceptance-visible-to-the-work-graph`](make-adr-acceptance-visible-to-the-work-graph.md), which could not hold this scope. That ticket made the *scheduling* failure structural — a ticket conditional on an unaccepted ADR now depends on an `accept-adr-NNNN-*` node that only Tom closes, so it cannot reach the ready frontier. This ticket closes the other, more dangerous half: nothing mechanically prevents a worker from writing a proposed decision into a normative contract as though it were settled, and such a change passes the repository gate today.

`AGENTS.md` states the rule twice — "proposed ADRs and proposed design documents are coherent hypotheses, not commitments", and "do not silently convert a proposal into fact" — and the documentation contract requires the corpus to keep "accepted decisions, proposals, measurements, and future work visibly distinct". None of that is enforced.

**Fact — the scope is `contracts/navigation` alone, and this is a correction to the dispatch brief that split this ticket.** `ticketsplease.toml` maps `scripts/docs.py` and `scripts/tests/**` to `contracts/navigation`; `implementation/workspace` maps `scripts/check_workspace.py`, `check_rust.py`, `check_repository.py`, and `check_ci.py`, and does not map `docs.py`. `scripts/check_repository.py:321` already runs `scripts/docs.py validate` as a gate phase, so a check added inside `docs.py` is gated with no edit to any `implementation/workspace` file. Reproduce: `grep -n 'docs.py' scripts/check_repository.py` and `grep -n 'scripts/docs.py' ticketsplease.toml`.

## Check A — a contract citing a proposed decision must disclose that it is proposed

For every record with `kind: contract`, resolve every Markdown link target. If a target resolves to a `docs/decisions/NNNN-*.md` whose `decision_status` is `"proposed"`, the containing block must also say `proposed`. A contract may legitimately cite an undecided record; it may not cite one silently.

**The corpus already contains the model citation.** `docs/architecture.md:350` reads "No accepted ADR yet records that admission, and [ADR 0077](...) is the proposed record of it … That supersession takes effect when Tom accepts it; until then ADR 0056's retained packaging text still places AOT invocation inside `tiler-metal`, and this section and that proposal are together the record." That paragraph names the status, names what still stands until acceptance, and asserts nothing the record has not earned. Use it as the fixture for the passing case.

**Measurement — the check lands green.** A probe implementing the predicate over the corpus at `b70da90` (paragraph = maximal run of non-blank lines; link targets resolved relative to the citing document; `records` loaded through `docs.load`) reports **2 proposed decisions, 1 contract citation of one, 0 violations**. So this can be added without a corpus-wide remediation pass first. Re-run the probe before implementing, since the count moves when either ADR is accepted or `propagate-extension-seam-classification-into-governed-contracts` lands.

Implementation notes: `validate_links` already parses each record with `MarkdownIt("commonmark")` and already resolves link targets against `root / record.path.parent`, so both halves exist — reuse them rather than adding a second link parser. Prefer the markdown-it token `map` line range over the blank-line heuristic the probe used; the heuristic is strictly coarser, so it under-reports and a token-accurate version may surface violations the probe did not. Handle a `#fragment` suffix on the link target.

**Measurement boundary — state this in the outcome rather than overclaiming.** Check A catches a contract that *cites* a proposed record without disclosing its status. It does not catch a contract that asserts the proposal's content while citing nothing, because no predicate over link structure can. That residual is bounded in practice by the propagation instruction pattern already used in this repository — `propagate-extension-seam-classification-into-governed-contracts` says "Do not restate ADR 0078's reasoning or its open questions in either contract; cite the record" — but it is a convention, not a proof, and the check must not be described as making the silent failure impossible in general. It makes the disclosed-citation path total and leaves the uncited-assertion path to review.

## Check B — no dispatchable ticket may depend on a still-proposed ADR's drafting ticket

Mechanizes the convention recorded in `ticketsplease.toml` beside `[workflow.states.awaiting-decision]`. Decision frontmatter carries `ticket: "<id>"` naming the ticket that drafted it, so the join is exact: for every decision with `decision_status: "proposed"` and a `ticket` field, no ticket in a dispatchable-or-open status (`todo`, `ready`, `in-progress`, `review`) may name that drafting ticket in its `dependencies:` list. The dependent must name the ADR's `accept-adr-NNNN-*` ticket instead. There is no legitimate exception to buy back: an acceptance node itself depends on its drafting ticket, so naming the acceptance node preserves the ordering transitively and additionally waits for the decision. Depending on the drafting ticket directly is therefore always either the same constraint stated weakly or the bug this check exists to catch.

**Measurement — this is a real regression pair, not a hypothetical.** The same probe over `tickets/` reports **1 violation before** the acceptance node was introduced (`propagate-extension-seam-classification-into-governed-contracts`, `status: todo`, depending on `draft-public-extension-seam-ownership-adr`, which drafted proposed ADR 0078) and **0 after**. Reconstruct the failing case by pointing that ticket's `dependencies` back at the drafting ticket; that is the regression test.

`validate_tickets` in `scripts/docs.py` already reads ticket frontmatter with a `^status: ([a-z-]+)$` regex, so extend it there. Read `dependencies:` from the same header slice; do not add a YAML dependency for one list field.

**Measurement boundary.** Check B only sees a declared edge. A ticket conditional on a proposed ADR that declares *no* dependency at all is invisible to it — that is Check A's territory once the work reaches a contract, and neither check sees it while it is only a ticket body. Both proposed ADRs currently carry a `ticket` field; a proposed ADR without one is silently exempt, so fail closed on that case rather than skipping it: a proposed decision whose `ticket` is absent should be an error in its own right.

## Also in this scope — `docs/work-tracking.md` now under-describes `awaiting-decision`

**Fact.** [`docs/work-tracking.md`](../docs/work-tracking.md) is the navigation portal that defines the parked states, and it says: "`awaiting-decision` means research is complete but Tom must choose among genuine product alternatives." That describes a *worker's own* ticket after its research finished. It does not cover the acceptance node that `make-adr-acceptance-visible-to-the-work-graph` introduced, whose whole function is to gate *other* tickets by dependency, whose research was finished by a different ticket, and which only Tom may close.

The convention itself is recorded in `ticketsplease.toml` beside `[workflow.states.awaiting-decision]`, which is `project/tickets`. This portal is `contracts/navigation` and is the document a contributor reads first, so it must state the same rule: a ticket conditional on an ADR being accepted depends on that ADR's `accept-adr-NNNN-*` node rather than on the ticket that drafted the record, and an acceptance node is closed only by Tom. Cite the convention; do not create a second authority over it.

## Closes when

Both checks are implemented in `scripts/docs.py`, covered by tests in `scripts/tests/`, reported through the existing `docs.py validate` phase so `scripts/check_repository.py` needs no edit, each check's failing case is reconstructed by a test rather than only described, the two measurement boundaries above are recorded in the outcome, `docs/work-tracking.md` states the acceptance-node rule, and `uv run --locked python scripts/check_repository.py` passes.

Do not weaken either check to a warning. `AGENTS.md` requires failing closed with an explainable error, and a warning in a gate that already emits none would be read as noise.
