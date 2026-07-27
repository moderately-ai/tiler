---
id: pin-the-admitted-unsafe-sites-in-the-workspace-gate
title: Decide whether the workspace gate pins admitted unsafe sites
status: awaiting-decision
priority: p2
dependencies: []
related: [record-the-case-by-case-unsafe-boundary, prototype-metal-runtime-execution]
scopes: [implementation/workspace, contracts/navigation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, workspace, gate, rust-api, needs-tom]
---
ADR 0079 permits unsafe code only at individually admitted sites. The current
compiler lint enforces that an unsafe block needs a local allow, but
`AGENTS.md` correctly states that no check inventories those allows. Review is
the only control over a newly added, moved, removed, or re-justified site.

A former Python workspace gate pinned each admitted
`(package-relative path, item signature, reason)` and had negative mutation
tests. That gate and its tests were deleted when repository verification moved
to the root `Makefile`; no implementation is currently in review.

## Decision

Decide whether to keep review-only enforcement or restore a mechanical
inventory in the current `make full` gate.

- Review-only enforcement is permitted by ADR 0079 and keeps the gate simple,
  but a new allow relies entirely on diff review.
- Mechanical inventory makes the admitted population explicit and can prove
  its own failure path, but adds a source-scanning authority whose parsing
  limits and maintenance cost must be accepted.

## Recommendation

Restore the exact path, item signature, and reason inventory. The permission is
case-by-case, so a count alone is insufficient: moving one site while adding
another must not pass. A negative mutation test must prove that additions,
moves, removals, and reason changes fail.

## Closes when

Tom selects review-only or mechanical enforcement. If mechanical enforcement
is selected, the current gate names every admitted site, documents its parsing
boundary, and includes a negative test that demonstrates the check can fail.
The ADR and `AGENTS.md` must describe the resulting enforcement truthfully.
