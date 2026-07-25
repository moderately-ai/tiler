---
id: accept-adr-0077-metal-aot-crate-admission
title: Accept or reject ADR 0077, the tiler-metal-aot crate admission
status: awaiting-decision
priority: p1
dependencies: [record-an-adr-for-the-metal-aot-crate-admission]
related: []
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: []
paths: []
tags: [decisions, governance, workspace]
---
**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. It is the graph node standing for a decision that has not been made, so that anything conditional on that decision is held out of the ready frontier by a dependency edge rather than by a worker noticing after being dispatched. Its permanent status is `awaiting-decision` — a `parked` category state that `tkt ready` excludes and that never satisfies a dependent.

**Fact.** [`docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md`](../docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) carries `decision_status: "proposed"`. Reproduce: `grep -n decision_status docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md`.

**Fact — this record points the opposite way from an ordinary proposal, which raises rather than lowers its urgency.** Its own status line says so: the crate, its empty dependency closure, and the development-only `tiler-metal` → `tiler-metal-aot` edge are already implemented and mechanically pinned in `scripts/check_workspace.py`, and what is missing is the decision. Until it is accepted, ADR 0056's retained clause — "MSL emission and AOT invocation remain modules in `tiler-metal`" — still stands as retained text that the workspace contradicts. `AGENTS.md` requires a durable decision to be superseded explicitly rather than silently departed from, and only acceptance of this record performs that supersession.

**Fact — the contradiction is currently disclosed rather than hidden, and that is the state acceptance ends.** [`docs/architecture.md`](../docs/architecture.md) line 350 names ADR 0077 as the *proposed* record, states that its supersession takes effect when Tom accepts it, and states that ADR 0056's retained packaging text still places AOT invocation inside `tiler-metal` until then. That paragraph is the model for how a governed contract may cite a proposed decision without asserting it; it is not a substitute for the decision.

**Fact — no ticket currently declares a dependency on this acceptance.** Reproduce: `grep -n 'dependencies:.*record-an-adr-for-the-metal-aot-crate-admission' tickets/*.md` returns nothing. Three tickets name `record-an-adr-for-the-metal-aot-crate-admission` under `related` only, and each was read in full: `correct-adr-0074-driver-vocabulary-consumers` corrects two falsified factual claims inside *accepted* ADR 0074 against measured source, `correct-artifact-crate-lockstep-ir-permission` corrects a crate doc comment against *accepted* ADRs 0056, 0070, and 0071, and `record-metal-aot-in-architecture-crate-profile` is `done`. None is conditional on ADR 0077 being accepted. This ticket therefore exists to hold the decision itself, and to be the edge target for any future ticket that would propagate ADR 0077's supersession into a contract.

## What Tom is deciding

Whether to admit `tiler-metal-aot` as a sixth reusable crate whose empty dependency closure and development-only inbound edge are decided properties rather than accidents of ordering, and thereby to supersede ADR 0056's retained AOT-invocation clause.

The record is deliberate about what it does *not* supersede, and accepting it accepts those judgements too:

- ADR 0065 is correct exactly as accepted; its "fifth reusable target-independent crate" is an ordinal about `tiler-reference`, not a cap on the profile, and it gains no superseding note.
- ADR 0070's dependency block is incomplete rather than wrong; ADR 0077 `refines` it by restating the block completely with six libraries and both development edges, instead of superseding correct edges to add missing ones.

## Closes when

`decision_status` in `docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md` moves off `proposed`, the record's status line is rewritten to match, `uv run --locked python scripts/docs.py render` regenerates `docs/decisions/README.md`, and `uv run --locked python scripts/check_repository.py` passes.

- **Accepted.** Set `decision_status: "accepted"`. `scripts/docs.py`'s graph validation then requires the accepted decision to carry `applies_to` and `evidence`, which ADR 0077 already has; ADR 0056 is already `decision_status: "superseded"` and already the target of `supersedes` edges from ADRs 0065, 0070, and 0077, so no further metadata moves. `docs/architecture.md`'s paragraph naming this record as proposed becomes stale in the same moment and must be rewritten to state the accepted packaging profile directly; that edit is `contracts/foundation`, so file it as its own ticket if the accepting change does not hold that scope.
- **Rejected.** Close with `tkt close` rather than `done`, so it does not satisfy dependents. Rejection does not restore the workspace to ADR 0056's retained clause — the crate exists and is pinned — so a rejection must be followed immediately by a ticket that either removes the crate or writes a different superseding record. Do not leave the contradiction undisclosed.

## Decision — Tom, 2026-07-25

**Accepted.** `decision_status` moves `proposed` → `accepted`.

Two consequences to carry out rather than assume: the disclosure at `docs/architecture.md:350` is no longer required by the proposed-decision gate (Check A) and may be reworded to cite an accepted decision; and this ADR's own clause that its admission must not be cited as precedent stays in force — it is the reason `admit-the-device-free-runtime-validation-crate` is a separate question rather than a corollary.
