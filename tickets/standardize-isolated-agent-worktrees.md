---
id: standardize-isolated-agent-worktrees
title: Standardize isolated agent worktrees
status: done
priority: p0
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: [AGENTS.md]
tags: [tooling, agents, developer-experience]
---
Establish the repository-wide convention for coordinator-created isolated Git worktrees. Create the external sibling root at `/Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler`; document one writable `edit` worktree per ticket, detached exact-commit reviewer worktrees, integration-worktree ownership, naming, claim/base requirements, validation, and safe cleanup in `AGENTS.md`; and add a defensive root `/.worktrees/` ignore for accidental in-repository worktrees. Do not relocate or remove existing worktrees in this ticket. Acceptance requires exact example paths, no raw filesystem deletion guidance, `tkt lint`, documentation validation, `git diff --check`, and guard against the true base.

## Outcome

Established the external per-ticket `edit` and detached role/SHA review
layouts, claim/base/task handoff requirements, exclusive integration ownership,
worktree-local Cargo outputs, exact-commit review checks, and Git-managed
cleanup without raw deletion. Added the defensive root `/.worktrees/` ignore.

As incidental repository-gate hygiene, corrected the already-completed
`correct-reference-value-and-authority-contracts` ticket's outcome heading to
the exact schema spelling. The complete repository gate passed before this
ticket was finalized.
