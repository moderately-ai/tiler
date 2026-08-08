---
id: repair-the-status-record-s-grammar-claim-and-its-failing-reproduction-line
title: Repair the status record's grammar claim and its failing reproduction line
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-repair-th
lease_expires_at: 1786162140
---
## A record whose own reproduction block fails on the line that carries its evidence

`docs/status.md` claims of the inline frontend: "**What they do not carry is a grammar**… region syntax, expansion, symbol binding, runtime value adaptation, and the complete cold/warm inline AOT and embedding workflow **all remain open**."

`crates/tiler-macros/src/` holds `grammar.rs`, `region.rs`, `binding.rs`, `delivery.rs`, `numerics.rs`, `aot.rs`, `retention.rs`, `eviction.rs`, `preflight.rs`, `family_cfg.rs`, and `cache_root.rs`. All nine `crates/tiler/tests/facade/fail/*.stderr` goldens are grammar diagnostics.

**The record ships a five-line reproduction block, and the auditor ran it verbatim: lines 1, 2, 4 and 5 pass; line 3 fails.** It tests for a compile-fail golden that does not exist. And `status.md` itself calls that line "the checked-in compile-fail golden behind '`tensor!` has no grammar': it is the **evidence that rejecting undefined input is a tested behaviour rather than a description of one**."

So the sentence's stated evidence is a file that is absent, and **the conclusion falls with its ground**.

## Read before repairing, because the correction is easy to overshoot

The claim is not simply inverted. Establish what the frontend *does* carry and at what maturity — `AGENTS.md` keeps reserved type, architectural seam, implemented support and tested guarantee distinct, and the recurring defect in this repository is a correction that overshoots in the opposite direction from the text it replaces.

Several of the listed items have moved independently since that sentence was written, so **check each of the five separately** rather than treating "all remain open" as one claim to flip. Note the retention read-back landed and is a **labelled draft awaiting Tom's acceptance** under ADR 0075 — do not describe it as accepted.

**Repair the reproduction block too, not only the prose.** A block whose lines are not all runnable is the same defect one layer down; run every line you leave in it and say so.

## Closes when

Every line of the reproduction block runs and passes on a clean tree; the grammar claim states what is carried and at what maturity, with each of the five items checked separately; and no sentence cites an absent file as its evidence.
