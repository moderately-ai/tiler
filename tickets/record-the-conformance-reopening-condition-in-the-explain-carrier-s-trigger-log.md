---
id: record-the-conformance-reopening-condition-in-the-explain-carrier-s-trigger-log
title: Record the conformance reopening condition in the explain carrier's trigger log
status: done
priority: p2
dependencies: []
related: [make-explain-dispositions-assertable-by-a-conformance-suite, decide-the-backend-provider-conformance-harness-public-surface]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, backend-providers, deferred-triggers]
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

## Source-first Fact audit — 2026-08-22 at base `518c56c35e61a7bd21def8ab9572b4d2e125f99f`

Every anchor below was run with `grep -c` against the file the citation names before being relied on, and every status was read from the file rather than from the reporting summary.

| Fact as filed | Verdict | Evidence |
| --- | --- | --- |
| The carrier's trigger log never recorded the condition it was deferred under | **Verified** | [The carrier's](make-explain-dispositions-assertable-by-a-conformance-suite.md) `## Trigger check log` carried exactly three entries — 2026-08-05, 2026-08-09, 2026-08-17 — and none quotes the acceptance condition, which post-dates all three. The 2026-08-17 entry is marked `**fired.**` against the portfolio being `done` plus the decision ticket owning the coverage choice, and carries no command; the 2026-08-09 entry carries none either. Nothing was dated after 2026-08-18. |
| Its priority disagrees with the artifact that presented it | **Verified as a disagreement, and the direction of the repair is the opposite of the one this ticket's outcome implies** | The carrier's frontmatter reads `priority: p2` and [`.ticketsplease/decision-queue.md`](../.ticketsplease/decision-queue.md) item 14 calls it `` (`p1`, blocked) ``. See the resolution below: the ticket file is the authority and the queue row is the wrong artifact. |

**The acceptance does name a condition for this carrier specifically, so the stop-and-report branch does not apply.** [`decide-the-backend-provider-conformance-harness-public-surface`](decide-the-backend-provider-conformance-harness-public-surface.md) names this carrier by id in its `## Accepted decision — 2026-08-18` section — both carriers move to `deferred` `with that trigger rather than becoming dispatchable` — and *that trigger* is the sentence immediately before it. No trigger was invented.

**The sufficient-versus-necessary shape the brief asked about is present, and it is the same one.** Two conditions bear on this carrier: the operative reopening condition, and the numbered `Explain coverage-expansion trigger` that is this ticket's own subject matter. Both are stated as sufficient and neither as necessary, so they do not compete; they are logged as two independent dated entries with different consequences rather than one being chosen over the other.

## Priority resolution — 2026-08-22

**The ticket file has authority, so the carrier's `p2` frontmatter stands and the queue row is what is wrong.** [`.ticketsplease/decision-queue.md`](../.ticketsplease/decision-queue.md) says so in its own header: `Ticket files remain the authority; this file records presentation order, holds, exact release triggers, and the current recommendation`. Reproduce with `grep -c 'Ticket files remain the authority' .ticketsplease/decision-queue.md`, which returns `1`.

**The queue row was also inaccurate on the day it was written, not merely stale.** The carrier has been `p2` since its creation on 2026-08-05 and has never been `p1`; the only frontmatter changes it has ever taken over those two lines are `status` moves. Item 14 was written on 2026-08-17 in `c50a34f2`, when the carrier was already `p2`. Reproduce with `git log --format='%h %ad %s' --date=short -L '4,5:tickets/make-explain-dispositions-assertable-by-a-conformance-suite.md'`, whose three hunks — `9417f96b` 2026-08-05 creating it at `p2`, `7f839294` 2026-08-17 moving `status` and dropping the backward dependency edge, and `33c5db60` 2026-08-18 moving `status` alone — show no `priority` line ever changing. The queue calls the same carrier `optional` in that row while labelling it `p1`, which is internally inconsistent and consistent with the label having been carried across from the two rows beside it.

**No frontmatter was moved.** The correct repair is to the queue row, and `.ticketsplease/decision-queue.md` is outside this ticket's edit permission — it is also declared in `paths:` on the decision ticket that owns it. Left for the coordinator, with the evidence above. Raising the carrier to `p1` to match the queue would propagate the error into the authoritative artifact.
