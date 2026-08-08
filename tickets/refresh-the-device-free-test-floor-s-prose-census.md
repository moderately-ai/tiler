---
id: refresh-the-device-free-test-floor-s-prose-census
title: Refresh the device-free test floor s prose census
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-terra-conformance-census
lease_expires_at: 1786214147
---

`DEVICE_FREE_TEST_FLOOR`'s doc comment states a test census that is stale by one. The floor itself still passes, so nothing is red — only the narrative is wrong.

## Facts, coordinator-verified at `aae3da24`

**Fact.** `crates/tiler-conformance/src/portability.rs` says the crate "declares 76 tests" and that a non-Apple host "runs 73". The tree has **77 and 74**.

**Fact.** `fe282f1e` added a test on 2026-08-08 without moving the prose census. The floor is **72** and still passes, so the guard is intact and only the sentence beside it drifted.

## Why it is worth fixing rather than shrugging at

A floor plus a prose census is a deliberate pairing: the floor stops the population silently shrinking, and the census tells a reader what the floor is protecting. When the census drifts, a later reader reconciling the two concludes one of them is broken and has no way to tell which. This session found several checks whose *description* was the wrong half — a bound attributed to the wrong authority in the paragraph titled for it, a variable named `measured_kernel_identity` holding filler bytes, a `const` assertion that was a tautology.

## What closes this

The census matching the tree — **or**, better, derived rather than restated. A hand-written count beside a floor has the same decay the floor exists to prevent, one level up. If the count can be printed from the population the floor already walks, print it; if it cannot, say why and date the figure to a commit.

**Do not move the floor.** 72 is a deliberate margin, not a tracking value; raising it to match the current count would defeat its purpose.

**Establish the treatment from history** with `git show <commit>:<file>`: true when written → dated beside; never true → substituted with the retired wording quoted. Repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note.

**Check the crate's other prose censuses and name the count.** A sibling found this one while sweeping something else, and noted a neighbouring Measurement bullet whose "every one of its runs landed the same day" is still true of *runs* but not of edits. Every sweep this session found more than it was sent for.

**`crates/**` is gated, so run `make full`** and report its exit — **read the log tail rather than trusting a reported code**; a worker this session had exit 2 reported as 0 because the exit line went through `tee`.

Cite by searchable anchor, run its grep before committing, and use `grep -F`.
