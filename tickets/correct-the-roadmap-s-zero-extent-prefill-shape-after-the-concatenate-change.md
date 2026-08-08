---
id: correct-the-roadmap-s-zero-extent-prefill-shape-after-the-concatenate-change
title: Correct the roadmap s zero-extent prefill shape after the concatenate change
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786191117
---

`docs/roadmap.md` says the concatenate family "states the zero-extent rule at L5" and names a `[8, 0, 128]` prefill shape. **That became false at merge `ab64f334`**, which removed the concrete shapes from the normative definition.

## Facts

**Coordinator-verified.** The concatenate normative definition no longer names concrete shapes; the rule is now stated over the operands and the illustration moved to a doc comment, which is **not encoded**. The change moved exactly one pin — the explain request qualifier, `940c09e0821665a6` → `4e10437fec85d7b1` — and no identity domain stepped.

**Reported by the worker that landed it, not coordinator-verified:** this roadmap site is the one document left naming the retired shape.

**Fact — why the shape was removed at all.** It was pinned-workload text (KV heads by head dimension) reaching canonical identity through the registered definition's bytes. Concatenate was reportedly the only family whose normative definition named concrete shapes; the new guard `no_registered_normative_definition_names_a_concrete_shape` walks **every** registered operation and value-type definition, so a future family inherits it.

## What closes this

The row stating the zero-extent rule without the retired shape. **Do not simply delete the shape** — check what the sentence was using it for. If it illustrates the rule, the illustration now lives in the doc comment and can be referenced; if it was standing in for the rule itself, the row needs the rule stated.

**Treatment:** true when written → dated beside. Verify with `git show <commit>:<file>`. Repository **practice**, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note. Note the fifth variant of that hazard, found this session: a repair quoting retired wording makes the retired anchor resolve to **the repair**, so a later reader searching for the origin lands on the correction — disclose any occurrence count that moves.

**Preserve `git log -S` anchors.** This file is heavily anchored and a sibling's rung-cell anchors already occur twice by construction. Two workers this session achieved append-only edits — one at **14 insertions, 0 deletions**, another at 2 — and both ran a **ten-word overlap scan** of their inserted lines against the pre-edit file; one found **eleven** accidental near-quotations and rewrote until zero at ten, eight, and seven words. Meet that standard.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously.

**Check the neighbouring rows and name the count.** Sweeps of this file this session found a rung cell undercounting execution rows and a reduction row whose figures the tree-width rule change did not in fact move — so both directions of error are live here.
