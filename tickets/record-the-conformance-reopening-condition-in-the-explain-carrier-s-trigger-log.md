---
id: record-the-conformance-reopening-condition-in-the-explain-carrier-s-trigger-log
title: Record the conformance reopening condition in the explain carrier's trigger log
status: in-progress
priority: p2
dependencies: []
related: [make-explain-dispositions-assertable-by-a-conformance-suite, decide-the-backend-provider-conformance-harness-public-surface]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, backend-providers, deferred-triggers]
claimed_from: todo
assignee: worker-explaintrigger
lease_expires_at: 1787433470
---
## User-visible outcome

`make-explain-dispositions-assertable-by-a-conformance-suite` carries the reopening condition the acceptance record says it was deferred under, so a trigger check on it can actually fire — and its priority agrees with the queue entry that presented it.

## Why this exists

Found 2026-08-22 by `worker-trigger` while reconciling a sibling decision's trigger, and reported rather than folded in because it is a different defect with a different owner.

**Fact (reported, unverified by the coordinator) — the carrier's trigger log never recorded the condition it was deferred under.** The acceptance record names this ticket as the other carrier moving to `deferred` "with that trigger". Its `## Trigger check log` carries three entries dated 2026-08-05, 2026-08-09, and 2026-08-17; the last is marked **fired** against a *different* condition and carries no reproducing command; nothing appears after the 2026-08-18 acceptance. So **nothing in that log re-evaluates now**, and a coordinator sweeping deferred triggers would read three stale entries and move on.

**Fact (reported, unverified by the coordinator) — its priority disagrees with the artifact that presented it.** Frontmatter says `p2`; `.ticketsplease/decision-queue.md` item 14 calls it `p1`.

**Why this is worth a ticket rather than an inline fix.** AGENTS.md requires every deferred ticket to end with a `## Trigger check log` whose dated entries record `fired`, `not fired`, or `unevaluable` **plus a reproducing command**. An entry marked `fired` with no command, against a condition the accepting record did not name, is the shape that makes a deferred pool look inert while a live trigger sits unchecked underneath it. The sibling reconciliation this came from showed how expensive that is: a trigger there had in fact fired and was logged `not fired` on a misreading, holding a p1 carrier for four days.

## Required work

- Re-audit both Facts at your base and report a per-Fact verdict — the coordinator has verified neither, and both come from one worker's read.
- Read the 2026-08-18 acceptance record in full and determine **which** condition this carrier was actually deferred under. If the acceptance does not name one for this ticket specifically, say so and stop rather than inventing one; an invented trigger is worse than an absent one, because a sweep will then act on it.
- Record a dated entry against the real condition, with a reproducing command — and **run that command before writing it down**. A supplied command that has never been executed is a claim, not a check.
- Re-examine the 2026-08-17 `fired` entry: say what condition it was against, whether that firing still stands, and why it produced no dispatch.
- Settle the priority disagreement by reading which artifact has authority, and move whichever is wrong.

## Non-goals

Implementing explain-disposition assertability; reopening the sibling decision, which is already done; and any edit outside `tickets/`.

## Closes when

The carrier's trigger log names the condition the acceptance record actually deferred it under, every entry carries a command that has been run, the stale `fired` entry is explained, and the priority disagreement is resolved in favour of the artifact with authority.
