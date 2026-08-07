---
id: extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation
title: Extend the citation check to docs and repair ADR 0079's drifted test citation
status: todo
priority: p2
dependencies: []
related: [pin-ticket-source-citations-against-the-tree-they-name, keep-the-citation-checker-s-anchor-path-exercised-and-its-boundary-fixture-live]
scopes: [implementation/workspace, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The gap, found by the drift it failed to catch

`check-citations.sh` reads `tickets/**` and nothing else. **`docs/**` carries citations too — ADRs, contracts, and research records all pin `path:line` into the tree — and none of them is checked.**

The instance that exposed it: `docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md` states that `crates/tiler-conformance/src/bf16_vertical/tests.rs` "**lines 497–548**" hold `the_unsafe_site_population_is_the_two_named_ones`. **It is at 696** — coordinator-verified — a **+199** drift. The same paragraph says the walk is "rooted at `CARGO_MANIFEST_DIR` on line 500", which has drifted with it. Reported by the worker on `pin-lint-inheritance-across-the-workspace-member-set`, which did not hold `contracts/decisions`.

This matters more in `docs/` than in `tickets/`. A ticket is consumed once by one worker and then closed; an **accepted ADR is a standing authority** that readers are directed to for years, and `AGENTS.md` ranks accepted ADRs as the *highest* evidence tier. A drifted citation in one sends a reader to unrelated code while carrying that authority.

## Two pieces of work

**1. Repair the ADR.** Re-locate by symbol, not by counting. Per `AGENTS.md`'s "cite by searchable anchor, not by line number", prefer the test's name as the anchor over any line number — the extent has already drifted twice and will again. Nothing about the ADR's *substance* changes: the test still does what the paragraph says, and the paragraph's careful boundary — that it enforces **none** of item 3's four conditions — stays exactly as written. This is a citation repair, not a re-decision.

**2. Extend the checker.** Give `check-citations.sh` a `docs/**` population beside its `tickets/**` one, and **report the two populations separately** so neither can silently collapse into the other. Consider what "terminal" means there: a ticket is skipped when `done` or `closed`, and the equivalent for a document is a **superseded ADR** or a dated correction quoting retired text — which the repository's own convention deliberately preserves. Getting that wrong produces the unsatisfiable condition this repository has already hit once, where a closing condition demanded a grep be empty while the convention required the text to stay.

Expect the first run to fail. **Report the failures rather than weakening the check to make them pass** — the drift above is one, and a corpus this size will hold others.

## Requirements carried from the existing checker

Read `check-citations.sh` in full first; do not reimplement what it already does. It resolves partial paths by unique suffix, skips ambiguous ones, is multi-line aware, and refuses a zero-citation run. All of that must apply to the new population.

**Name and count both populations**, and fail an empty one. **Perturb the subject, never the assertion**, and quote each failure. Note that a sibling p1 ticket, `keep-the-citation-checker-s-anchor-path-exercised-and-its-boundary-fixture-live`, is repairing the anchor-form coverage — coordinate rather than duplicating, and do not remove its fixture.

## Closes when

ADR 0079's citation resolves and is anchored on the symbol; `make citations` covers `docs/**` and `tickets/**` with separately reported counts; the run fails when either population is empty; and every failure the extension surfaces is either repaired or filed.
