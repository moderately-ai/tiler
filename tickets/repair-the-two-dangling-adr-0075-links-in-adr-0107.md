---
id: repair-the-two-dangling-adr-0075-links-in-adr-0107
title: Repair the two dangling ADR 0075 links in ADR 0107
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786167946
---
## What is broken

`docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md` links to ADR 0075 twice, and both links name a file that does not exist:

- line 22 — `[ADR 0075](0075-approve-public-boundaries-by-change-category.md)`
- line 118 — `[ADR 0075](0075-treat-a-tested-public-boundary-as-a-labelled-draft.md)`

The real file is `docs/decisions/0075-scope-public-boundary-approval-by-change-category.md`, verified present 2026-08-08. Both link texts read "ADR 0075" and both targets are descriptions of what 0075 says rather than its filename, which is how this survived a read.

## Why it matters

Both sites are load-bearing prose: they are the sentences stating that the gather public surface stays a **labelled draft** until separately accepted. A reader checking that claim against the governing ADR is sent nowhere.

`repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows` (done) repaired the **catalog rows** in `docs/decisions/README.md`. These two are in the ADR **body** and were never in that ticket's scope; the one-off link check that ticket's worker ran is what first reported them.

## Fact audit at base `db3f4d07`

Every Fact above re-read at this base and verified; nothing false or imprecise was found. Both link sites are at the stated lines (22 and 118), both link texts read `ADR 0075`, and `find . -name '0075*'` returns exactly one file — the real `docs/decisions/0075-scope-public-boundary-approval-by-change-category.md`.

**Fact — these were never valid targets, so this is not a stale path.** `git log --follow --name-status -- docs/decisions/0075-scope-public-boundary-approval-by-change-category.md` shows `A` at `9251db4` under the current slug and no `R` since, so ADR 0075 has never been named either linked slug. `git log -S` places `0075-approve-public-boundaries-by-change-category` at `260cee8` (the acceptance commit) and `0075-treat-a-tested-public-boundary-as-a-labelled-draft` at `cf9578e` (the original body). The second is the sentence at `AGENTS.md`'s "A tested public boundary remains a labelled draft until Tom accepts its exact included and excluded surface", spelled as if it were a filename.

**Fact — ADR 0075 is the truthful target for both, not merely the nearest file.** `crates/tiler-ir/src/semantic/gather.rs` states in its module documentation "This is a *labelled draft* public boundary under ADR 0075 until Tom accepts its exact included and excluded surface. Included: the key, the gathered-axis attribute, [`GatherAxis`], [`GatherError`], and the shape rule [`gather_result_shape`]" — the same claim over the same item list that line 22 makes, already attributed to ADR 0075 in the governed source. `docs/roadmap.md`'s gather row carries line 118's sentence almost verbatim for this same family and links the real 0075 filename, as do `docs/ir.md`'s symbolic-coefficient paragraph, ADR 0102, and `docs/research/apple-targets/numerical-behaviour.md`.

**Fact — 0075 keeps the obligation the two sentences invoke.** Its "What this record does not change" states that `AGENTS.md` "remains the operative working contract and still states the unbounded ... obligation", and its Decision routes "Promoting a module or type from `pub(crate)` to `pub`" to Tom. So citing 0075 for a labelled-draft surface names the governing accepted decision, which itself points at the operative sentence. Citing `AGENTS.md` instead would diverge from every sibling site and from the source module, and `AGENTS.md` is not a governed record.

## Closes when

`make citations` reports no link failure in this file, with both links pointing at the real 0075 filename.
