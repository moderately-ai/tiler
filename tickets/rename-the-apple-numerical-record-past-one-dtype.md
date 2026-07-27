---
id: rename-the-apple-numerical-record-past-one-dtype
title: Rename the Apple GPU numerical-behaviour record
status: done
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype]
scopes: [research/apple-targets, contracts/navigation, contracts/decisions, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, apple-targets]
---
`docs/research/apple-targets/numerical-behaviour.md` is titled "Apple GPU f32
numerical behaviour", but its later findings measure `f16` and `bf16` as well.
The title now names the record's origin rather than its extent.

The rename is a separate ticket because the title is copied into manually
maintained catalogs and prose citations across navigation, decisions, backend,
and integration documents. Nothing regenerates or validates those copies, so
the ticket must update every checked-in occurrence in the same change. The
record carries an explicit stale-title note that this ticket removes.

**What closes this.** The frontmatter `title` and the `#` heading name what the record measures without naming one dtype; every catalog block that quotes it has been updated by hand, since nothing regenerates them; the three prose citations read correctly; and the record's "this record's title is stale" note is gone rather than reworded.

## Outcome — renamed across a counted population (2026-07-27)

**New title: "Apple GPU numerical behaviour"**, in both the frontmatter `title` and the `#` heading. It names what the record measures without naming a dtype, so a fourth width does not make it stale again.

**The population was enumerated and counted before editing, then re-counted after.** Nothing regenerates these copies and nothing validates them, so "I think I got them all" is not a check. Searching for the title in both its plain and backticked spellings found **eight** occurrences: the frontmatter title, the heading, three catalog blocks (`docs/research/README.md`, `docs/decisions/README.md`, `spikes/README.md`), and three prose citations (`docs/integration/candle.md`, `docs/backends/metal.md`, `docs/decisions/0076-declare-target-honourable-numerical-realizations.md`). All eight were rewritten and the same search now returns zero.

**The record's own note miscounted its catalogs.** It said the title appears "in four generated catalog blocks under `contracts/navigation`". There are three. The note is removed rather than reworded, as this ticket asked, so the miscount goes with it — but it is recorded here because it is the reason the count was re-derived rather than trusted: a stale note's arithmetic is as stale as its claim.

**A ninth site had to change that no title search would have found.** ADR 0076's measured-evidence bullet asserted "That record's title still says `f32`, which was its whole extent when this bullet was written". Renaming the record makes that sentence false, so the rename would have traded one stale statement for another. It now says the record *was* titled for `f32`, why, and that it was retitled — preserving the qualification the sentence exists to make, which is that the measurements **this ADR** rests on are `f32` measurements regardless of what the record has since grown to cover. Found by searching for prose *about* the title rather than for the title itself.

**Nothing else moved.** ADR 0076 keeps its decision, rationale, and `accepted` status; the research record keeps its frontmatter apart from `title`, including `topics`, which already listed `dtypes`.
