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

`awaiting-decision` means research is complete but Tom must choose among genuine
product alternatives. `deferred` means the work is intentionally parked until
its stated trigger. Neither belongs in `tkt ready`.

Before work: read [AGENTS.md](../AGENTS.md), inspect `git status`, create the
dedicated worktree/branch from current `origin/main`, then claim the ticket.
Before integration: run the ticket's tests, `tkt lint`, `git diff --check`, and
`tkt guard` against the true base. A completed ticket must point to its durable
outputs and remaining gates.
