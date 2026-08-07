---
id: keep-the-citation-checker-s-anchor-path-exercised-and-its-boundary-fixture-live
title: Keep the citation checker's anchor path exercised and its boundary fixture live
status: todo
priority: p1
dependencies: []
related: [pin-ticket-source-citations-against-the-tree-they-name]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The defect, and how it arrived

`check-citations.sh` landed on 2026-08-07 reporting `273 line-only, 4 anchor-only` over the live tree. It now reports **`307 line-only, 0 anchor-only, 0 line+anchor`**. The anchor-matching code path is **exercised by nothing on the real tree**, and neither is the whitespace-collapsing fallback that a wrapped anchor needs.

**Cause, and it was the coordinator's.** Every anchor-only citation lived in `pin-ticket-source-citations-against-the-tree-they-name`, and closing that ticket as `done` removed it from the checked population — the checker skips terminal tickets by design, reading `category = "terminal"` from `ticketsplease.toml`. Routine outcome-recording silently switched off half the checker's own surface. Nothing failed, because a path that runs zero times reports no failures.

**This is the failure mode the checker exists to embody**, turned on the checker: a verdict is only as good as the check's ability to say no, and an unexercised branch cannot say anything. It is the same shape as `cargo doc` over `#[cfg(test)]` modules, and as the population floors that `portability.rs` and `lints.rs` both carry precisely so a collapsed set fails rather than passing quietly.

**The boundary fixture went with it.** That ticket carries a citation that *resolves and is false* — it claims `make check` runs the citation check last, anchored on the verbatim `` `Makefile "check: citations fmt build lint test"` `` while `citations` is in fact prerequisite #1. It is the only standing demonstration that green means "the citations point somewhere" and never "the tickets are true", and both the script header and the ticket say not to "fix" it. It is now inert.

## What to build

**Per-form population floors, not just a total.** The script already fails when it parses zero citations overall — extend that to the forms it supports, so an anchor count of zero is a failure naming the unexercised path rather than a quiet line in the summary. Do the same for the whitespace-collapsed match, which the earlier run recorded as `1 anchor(s) matched only after collapsing whitespace` and which is now also at zero.

**A fixture that outlives a ticket's lifecycle.** The demonstration must not depend on one ticket staying open — that is exactly what just failed. Prefer a fixture the script owns, or a fixture directory the script points at, over a ticket that happens not to be closed yet. Whatever you choose, state why it cannot be switched off by a status change.

Preserve the existing property, verified by the coordinator on the merged tree: **terminal tickets are still skipped**, which is correct and load-bearing — `scripts/check_workspace.py` was deleted at `e197176f` and is accurately named in ten closed tickets recording that history.

## Required evidence

Show the anchor path failing when no anchor citation exists, and passing when one does. Then, with the fixture in place, show a **semantically false but resolving** anchor citation still passing — that is the boundary being demonstrated, and losing it is what this ticket repairs. Perturb the subject, never the assertion, and quote each message.

## Closes when

`make citations` fails on a tree with no anchor-form citation; the boundary fixture is live and cannot be disabled by a ticket status change; and the summary still names and counts every population it checks.
