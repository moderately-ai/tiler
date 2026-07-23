---
id: decouple-status-prose-from-the-live-board
title: Decouple status prose from the live ticket board
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, governance, maintenance]
claimed_from: todo
assignee: agent-decouple-status-prose-from-the-live-board
lease_expires_at: 1784832272
---
`docs/status.md` names specific in-flight tickets as "the immediate compiler frontier" and `docs/roadmap.md` enumerates a dependency chain by ticket id. Both rot on every merge: within one working session the named frontier went stale twice, and today two of the three named authorities completed while a fourth listed as downstream also completed. `docs/status.md` currently carries nine ticket links.

This contradicts the repository's own stated authority split. `docs/work-tracking.md` says "Ticketsplease is the live work graph; Markdown status pages are not a duplicate board", and `tkt rollup` already reports the ready frontier and blocked set on demand. Enumerating dispatchable ticket ids in a governed contract duplicates that authority and guarantees drift.

Restructure both documents so they describe the durable phase, boundaries, and evidence state — which is genuinely theirs to own — and defer the dispatchable frontier to the board. Keep links to tickets that are durable references (accepted scope gates, milestone exits, deferred triggers); remove or generalize links whose only purpose is naming what happens to be dispatchable now. Where a frontier statement is genuinely useful to a reader, phrase it so it stays true across a wave, or point at the `tkt` commands work-tracking.md already documents.

As part of the same pass, correct the current staleness: the typed explain authority, the generic IndexRegion reference oracle, and bounded semantic normalization are complete and merged; generic region formation is in flight. Do not simply re-enumerate the new frontier, or this ticket will need filing again next wave.

Run the full documentation gate before completion.

## Outcome

`docs/status.md` and `docs/roadmap.md` now describe the durable phase, the compiler path's stage architecture, and the evidence boundary, and defer the dispatchable frontier and its live ordering to the board (`tkt ready`, `tkt rollup`) that `docs/work-tracking.md` already documents as authoritative.

**`docs/status.md`.** Replaced the "immediate compiler frontier" sentence — which named `typed explain`, `operation capability registration`, and the `generic index-region reference oracle` as the three parallel authorities, two of which had already merged — and the transient "downstream work is split into …" stage list. The Authorized-prototype section now states as monotonic completed evidence that the target-neutral compiler path has begun landing (typed explain, the generic index-region reference oracle, bounded semantic normalization with CSE, and generic fusion-region formation are merged), describes the remaining stages as durable architecture rather than a frontier, and points the reader at `tkt ready`/`tkt rollup` for which authority is dispatchable.

**`docs/roadmap.md`.** Replaced the six-step, ~15-link Milestone 0B dependency chain (which enumerated every in-flight and blocked prototype ticket by id) with durable prose describing the same compiler path as a set of independent authorities with real dependencies, explicitly stating the ordering shifts as work lands and that `tkt rollup`/`tkt ready` — not a chain enumerated in the doc — report complete/dispatchable/blocked state.

**Ticket links kept (durable):** `prototype-public-compiler-api` (milestone exit consumed by the inline frontend, both files); `prototype-optimizer-conformance-gate` (the pivotal gate the backend/runtime proofs wait on, roadmap); `implement-opaque-physical-call-providers` (explicit deferred trigger, roadmap); the four completed authorized-prototype slice links in status.md (`prototype-semantic-reference-slice`, `prototype-target-neutral-baseline-slice`, `prototype-target-neutral-fusion-slice`, `prototype-shared-compiler-ir-ownership`) as fixed completed-evidence citations for the bounded prototype.

**Ticket links removed (transient frontier enumeration):** from status.md, the `typed explain` / `operation capability registration` / `generic index-region reference oracle` frontier trio and the `verifier subject-binding` done-correction link; from roadmap.md, the `harden-compiler-verifier-subject-binding-and-totality` scaffolding link plus the intermediate-stage chain (`typed explain`, `operation capability registration`, `generic index oracle`, `bounded normalization`, `generic region formation`, `semantic/index refinement`, `fusion legality`, `region covers`, `target feasibility`, `scheduled regions`, `physical implementation frontiers`, `complete physical-plan selection`, `structured KIR`, `kernel program IR`, `artifact-facing program model`).

**Avoiding re-enumeration.** The current dispatchable frontier (operation capability registration, target feasibility) is deliberately not named. Completed work is stated as monotonic evidence (merged stays merged), the remaining path is described as the compiler's durable stage architecture, and every "which is dispatchable/blocked" question is delegated to the board — so the next merge cannot make these documents stale in the way that refiled this ticket.

**Gate.** `uv run --locked python scripts/check_repository.py`, `git diff --check`, and `tkt guard tkt/decouple-status-prose-from-the-live-board` all pass. No frontmatter feeding a generated catalog was touched, so no catalog regeneration was required.
