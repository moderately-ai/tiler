---
id: repair-the-two-dangling-adr-0075-links-in-adr-0107
title: Repair the two dangling ADR 0075 links in ADR 0107
status: todo
priority: p1
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## What is broken

`docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md` links to ADR 0075 twice, and both links name a file that does not exist:

- line 22 — `[ADR 0075](0075-approve-public-boundaries-by-change-category.md)`
- line 118 — `[ADR 0075](0075-treat-a-tested-public-boundary-as-a-labelled-draft.md)`

The real file is `docs/decisions/0075-scope-public-boundary-approval-by-change-category.md`, verified present 2026-08-08. Both link texts read "ADR 0075" and both targets are descriptions of what 0075 says rather than its filename, which is how this survived a read.

## Why it matters

Both sites are load-bearing prose: they are the sentences stating that the gather public surface stays a **labelled draft** until separately accepted. A reader checking that claim against the governing ADR is sent nowhere.

`repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows` (done) repaired the **catalog rows** in `docs/decisions/README.md`. These two are in the ADR **body** and were never in that ticket's scope; the one-off link check that ticket's worker ran is what first reported them.

## Closes when

`make citations` reports no link failure in this file, with both links pointing at the real 0075 filename.
