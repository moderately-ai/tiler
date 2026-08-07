---
id: pin-ticket-source-citations-against-the-tree-they-name
title: Pin ticket source citations against the tree they name
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-pin-tick
lease_expires_at: 1786142810
---
## Why this exists

Every ticket audited on 2026-08-07 carried at least one false Fact. The most damaging class was **line citations that had drifted** — `admit-a-fusion-role-for-the-sub-tensor-selection-slice` had *every* citation stale by 200–400 lines, so a worker following them would have landed in unrelated code and edited the wrong thing. Others named a test that no longer exists, or a count that had since changed.

`AGENTS.md` now carries the reading obligation that actually controls this: **a ticket's stated Facts are stale until re-read at your own base**, and a worker's first deliverable is a per-Fact verdict. This ticket adds the cheap mechanical layer *underneath* that — not in place of it.

## The boundary, stated first because it is the point

**This check cannot verify a claim, and must not be presented as doing so.** A citation can resolve perfectly and still support a statement the code no longer makes — which is exactly what happened to the reassociation-obligation claim, where the file and symbol were right and the described behaviour was wrong. What a checker can catch is the cheapest subset: a path that does not exist, a line past end-of-file, a quoted anchor that appears nowhere.

So the deliverable is a **loud floor**, and its documentation must say plainly that a green result means "the citations point somewhere", never "the ticket's Facts are true".

## What to build

A check over `tickets/**` that, for each source citation:

- resolves the path against the working tree, failing when it does not exist;
- where a line number is given, fails if the file has fewer lines;
- where a **quoted anchor** is given, fails if the quoted text appears nowhere in the named file — this is the half with real signal, and it is why `AGENTS.md` now asks for anchors rather than bare line numbers.

Requirements that are not optional, drawn from checks in this repository that could not fail:

- **Name and count the population.** Report how many tickets and how many citations were examined. A run that parses zero citations and reports no problems must **fail**, not pass.
- **Be multi-line aware.** Citations wrap across lines in ticket prose, and a line-oriented matcher will silently miss them — the same defect that made a `grep` for `allow(unsafe_code` return one doc comment and none of four real attributes.
- **Watch it fail, per failure mode.** Plant a bad path, a past-EOF line, and a missing anchor, separately, and quote each failure. Then plant a citation that resolves but is *semantically* wrong and show the check **passes** — documenting the boundary above by demonstration rather than assertion.
- Exclude closed and superseded tickets, or dated-correction blocks that deliberately quote retired citations; decide which and say why. A condition that demands the repository forget what it corrected is unsatisfiable — that mistake has already been made here once.

## Where it runs

Decide and justify: `tkt lint`-adjacent, a `make` target, or a test. Note that `tickets/**` is not in the delta rule's gated set, so a ticket-only change currently carries the previous green gate — a check that only runs under `make full` would not see most ticket edits. That is an argument for the lighter gate, not an afterthought.

## Non-goals

Verifying that a citation supports its claim. Rewriting existing citations in bulk — repair them as tickets are dispatched, under the reading obligation. Editing `AGENTS.md`, which already carries the rule.
