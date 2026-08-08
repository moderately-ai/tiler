---
id: complete-the-runtime-product-bullet-s-device-execution-retention-claim
title: Complete the runtime product bullet s device execution retention claim
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786188969
---

`docs/status.md`'s runtime-product bullet says device-execution code "is retained in `prototypes/serial-sum-run`". That was complete when written and is not now.

## Facts

**Reported by the worker that refreshed the execution-row census, not coordinator-verified — check before editing.** `crates/tiler-conformance` now holds device-dispatching entry points across three verticals, so the prototype is no longer the only place device execution lives. The bullet is **incomplete rather than false**: the prototype does still retain such code.

**Coordinator-verified:** that worker deliberately left this alone as outside its census, and named it for its own ticket rather than folding it in.

## What closes this

The bullet stating where device execution lives now, without implying the prototype was replaced — it was **retained alongside**, and a sibling established that one vertical *re-homes* the prototype's corpus (three reduction classes × two plan roles × five operand cases reproduces its thirty) while two others are independent. Getting that relation wrong in either direction is the failure.

**Prefer naming the construction over counting.** A sibling replaced a seven-row ledger restatement in this same file with a reference to its owner, on measured evidence: the owner was current on all three rows where the restatement was stale, same tree, same day, and a hand patch two days earlier had **held two days**. If an enumeration in `tiler-conformance` owns this, name it.

**Treatment:** true when written → dated beside. Decide with `git show <commit>:<file>`. Repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note.

**Preserve `git log -S` anchors.** A sibling achieved **14 insertions, 0 deletions** across three documents so every pre-existing byte was unchanged, then ran a ten-word overlap scan of its own inserted lines against the pre-edit file and found **eight** accidental near-quotations that would have created new collisions — including one reproducing its own ticket's anchor. Meet that standard and disclose any occurrence count that moves.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously. This file spells one crossing "between 50 and 51 operations", so an anchor written `50/51` returns 0.

**Do not edit `crates/**`** — read it to describe it correctly. Check the neighbouring claims and **name the count**; three sweeps of this file this week each found more than they were sent for, including a manifest schema version stale by a full step that was labelled coordinator-verified.
