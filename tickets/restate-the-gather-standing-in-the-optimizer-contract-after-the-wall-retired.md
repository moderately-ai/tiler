---
id: restate-the-gather-standing-in-the-optimizer-contract-after-the-wall-retired
title: Restate the gather standing in the optimizer contract after the wall retired
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift]
claimed_from: todo
assignee: worker-optrestate
lease_expires_at: 1787474269
---
## User-visible outcome

The optimizer contract states the gather's current standing, so a reader is not told a wall declines every gather member set when that wall has retired.

## Why this exists

Filed 2026-08-23 by the coordinator as the **known expiry** of `correct-the-optimizer-contract-capability-count-and-gather-standing`, which landed as `27fa3043`. That lane repaired `docs/compiler/optimizer.md` to the standing that was true at the time and **flagged the expiry itself** rather than writing a sentence with a hidden shelf life. This is that flag being actioned, not a defect it left behind.

**Fact — the sentence it wrote is now false, and the change that falsified it has landed.** `thread-resolved-lowering-into-the-governed-spelling-path` landed as `0326745a`. `RegionVocabularyWall::GatherProofUnavailable` **retired**: the variant is gone from the enum in `crates/tiler-compiler/src/physical.rs`, and the two surviving mentions of the name are doc prose recording the retirement, one of them reading `It replaces` alongside the retired name. Verified by the coordinator at `41c0d55f`.

**Fact — what replaced it is narrower, which is the point.** `RegionVocabularyWall::GatherIndexBoundsUnproved` (reason `gather-index-bounds-unproved`) refuses **only** the undischarged population. A gather whose bounds obligation is statically proved is now **spelled** — `physical::gather_region` landed and `govern_spelling` gained a `Gather` arm. So the document's "declines every gather member set" is false in both halves: it no longer declines every set, and the wall that did is gone.

**Fact — a gather is still not compilable end to end, for a new and different reason.** `crates/tiler-ir/src/kernel/lower.rs` answers `LogicalAccess::GatherSource { .. } => Err(KernelDiagnostic::BodyRefinement)`, and `pipeline::planning::kernel_lowering_failure` now classifies that as `("kernel-lowering", "gather-kernel-body")`. **Repair to this, not to "a gather is supported"** — the standing moved one wall down, it did not disappear. [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](lower-the-indirect-gather-read-through-the-structured-kernel-body.md) owns that wall.

## Required work

- Re-audit all three Facts at your base with a verdict, running each command yourself.
- Repair the paragraph to the current standing, following the convention it already carries: a dated correction quoting the retired wording so its grep count **cannot shrink**, then the replacement.
- **Update the pin the old sentence cited.** `a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary` was named as the evidence; establish what that test asserts now and cite what is actually true, rather than carrying the old name forward unread.
- **Cite by searchable anchor, not line number**, and run each anchor's grep against the file it names **before** committing to it. A full sentence lifted from a rendered view fails as *absence* because of a line break, an inline link, an emphasis marker, or a sentence-initial capital.
- Sweep the document for any other claim about gather standing or the retired wall, and report clean results as well as findings.

## Non-goals

`crates/`. Re-deciding any optimizer conclusion. The kernel body itself. The capability count, which `27fa3043` already corrected to twenty-two and fifteen fixed-signature families — verify it still holds but do not re-derive it.

## Closes when

No live claim in `docs/compiler/optimizer.md` names the retired wall as current, the gather's standing is stated at the wall that actually holds it today, the cited pin matches what that test now asserts, retired wording is preserved, and the document sweep is reported with its clean results.
