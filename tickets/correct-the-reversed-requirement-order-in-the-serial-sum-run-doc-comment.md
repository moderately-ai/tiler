---
id: correct-the-reversed-requirement-order-in-the-serial-sum-run-doc-comment
title: Correct the reversed requirement order in the serial sum run doc comment
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786188969
---

A doc comment states the reverse of what its own function does — and it is the **origin** of a false claim that propagated into two tickets and a coordinator brief before anyone read the code.

## Facts

**Coordinator-verified at `e71b5c0d`.** In `prototypes/serial-sum-run/src/proof.rs`, `fn resolve_prepared_route` calls `qualify_live_device` (`resolve_live_device_requirements`) **first**, then `check_direct_requirements`, then `prepare_pipelines`. Its doc comment says the opposite — "the requirements the verified program itself derived … then the live-device rows the artifact carried".

**Fact — the propagation is the point.** That comment produced a false Fact in `check-synchronization-realization-before-the-routing-commit`, which the coordinator repeated in a brief as "one stage earlier than a live-device resolution", which then entered `discharge-the-derived-requirements-in-the-candle-metal-adapter`. The worker on that last ticket traced it back and repaired the parent with a dated correction. **The true statement is that the discharge stage is one earlier than *pipeline preparation*, not than the live-device rows.**

**Fact — the design is unaffected.** `route_with_adapter` also calls `prepare_entries` after live-device resolution, so the landed order is reproduced exactly. Only what may be *claimed* about it changes.

## What closes this

The comment stating the actual order, cited by **searchable anchor**. Check whether the surrounding sentences depend on the reversed reading — a comment wrong about sequence often has neighbours reasoning from it.

**Treatment:** establish with `git log -S` and `git show <commit>:<file>` whether it was ever true. If the order was reversed at some commit and later changed, date beside; if it was never the actual order, substitute with the retired wording quoted. Repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note.

**`prototypes/` is excluded from the style gate but reached by `build`, `test`, and `doc`.** A worker this week had `make full` exit 2 on a broken intra-doc link in this crate. So a warning here is invisible and a broken doc link is not — say which your change could produce, and **read the log tail rather than trusting a reported exit code**: another worker had exit 2 reported as 0 because the exit line went through `tee`.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously.

**Check this file's other ordering and stage claims and name the count.** This one misled three documents; assume it is not alone.
