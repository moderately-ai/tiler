---
id: repair-the-flash-class-records-falsified-supplied-greps
title: Repair the flash-class record's falsified supplied greps
status: todo
priority: p2
dependencies: []
related: [reconcile-the-l4-records-self-contradicting-softmax-elimination-row, re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, falsified-evidence]
---
## User-visible outcome

`docs/research/program-planning/flash-class-capability-set.md` hands a reader commands whose output supports the claim beside them, so re-running one confirms the record rather than appearing to reverse it.

## Why this exists

Found 2026-08-22 by `worker-l4row2` during a sibling scan. **Two of the five are falsified supplied greps, and both fail in the direction that reads as reversal** — the record says a search returns nothing or little, and it now returns a great deal, so a reader re-running it concludes the finding is dead when the conclusion is in fact intact.

Verified by the coordinator at `123f1b02`:

- The record states `grep -rn 'SubgroupWidth\|lane_identity\|SubgroupThenWorkgroup' crates/` **"returns nothing"**. It returns **69 lines**.
- It states `grep -rni 'simdgroup' crates/` **"returns five lines"**. It returns **21**.

**The conclusions survive.** `MetalTargetFacts` still has exactly five fields, none of them a subgroup width — the record is right about what it concludes and wrong about what it offers as proof. So this is a **re-evidencing**, not a withdrawal.

**Three further line pins have drifted** (reported by that lane, unverified by the coordinator): `feasibility.rs:211`→241, and the item is `pub(crate)` rather than the `pub` the record states; `target.rs:755`→871; `component_cost.rs:619`→629.

## Required work

- Re-audit all five at your base with a per-Fact verdict, **running each command yourself** and reporting its actual output.
- **Re-evidence rather than withdraw.** Give each surviving conclusion a reproduction that supports it — prefer a claim about structure that stays true (`MetalTargetFacts` has five fields, none a width) over a bare emptiness assertion, which is the shape that rots into apparent reversal.
- Replace drifted line pins with **anchors**; a line number rots silently while an anchor fails loudly. Correct the `pub`/`pub(crate)` mis-statement.
- **Preserve retired wording in dated corrections**; grep counts cannot shrink.
- Check this record's siblings for the same shape — a supplied command whose stated output no longer matches. Report findings **and** clean results.

## Non-goals

Re-deciding any flash-class conclusion; editing `crates/`; and the wider zero-synchronization retirement, which is its own ticket.

## Closes when

Every supplied command's stated output matches what it produces, each surviving conclusion carries a reproduction that supports it, drifted pins are anchors, and the sibling scan is reported with its clean results.
