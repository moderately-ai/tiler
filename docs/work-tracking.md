---
schema: "tiler-doc/v1"
id: "tiler.portal.work-tracking"
kind: "portal"
title: "Work tracking"
topics: ["workflow", "ticketsplease"]
---

# Work tracking

Ticketsplease is the live work graph; Markdown status pages are not a duplicate
board.

```sh
tkt rollup                 # overall status and blocked frontier
tkt ready                  # dependency-satisfied dispatchable work
tkt tracks                 # conflict-free parallel batches
tkt show <id>              # ticket, comments, and outcome
tkt reconcile              # branch/worktree/board consistency
```

`awaiting-decision` covers two shapes. A worker's own ticket reaches it when
research is complete but Tom must choose among genuine product alternatives. An
`accept-adr-NNNN-*` node is the other: its research was finished by a different
ticket, and its only function is to hold dependents out of the ready frontier
until Tom accepts the record. Only he closes one. A ticket conditional on an ADR
being accepted therefore depends on that acceptance node, never on the ticket
that drafted the record — drafting a proposed ADR is a completed outcome, so the
drafting ticket is correctly `done` the moment the file exists, and a dependency
on it cannot tell written from decided. That convention lives in
[`ticketsplease.toml`](../ticketsplease.toml), beside
`[workflow.states.awaiting-decision]`, and that file is its authority. Nothing
enforces it any more, so a dispatchable or open ticket depending on such a
drafting ticket is now caught by reading rather than by a check. `deferred`
means the work is intentionally parked until its stated trigger. Neither belongs
in `tkt ready`.

Before work: read [AGENTS.md](../AGENTS.md), inspect `git status`, atomically
claim the ticket, then immediately create or enter its dedicated branch/worktree
from current `origin/main`. Do not edit scoped content between claim and branch
creation.
Before integration: run the ticket's tests, `tkt lint`, `git diff --check`, and
`tkt guard` against the true base. A completed ticket must point to its durable
outputs and remaining gates.
