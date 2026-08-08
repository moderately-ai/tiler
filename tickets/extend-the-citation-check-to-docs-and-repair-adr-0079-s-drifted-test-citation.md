---
id: extend-the-citation-check-to-docs-and-repair-adr-0079-s-drifted-test-citation
title: Extend the citation check to docs and repair ADR 0079's drifted test citation
status: done
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

## Outcome — done, 2026-08-07

Landed at merge **`623c84fc`**, published only after its two blockers landed with it. `make full` exit 0 on the merged tree, 1,091 release tests.

**Coverage went from 245 ticket citations to 923 across 494 live files** — 263 ticket, 655 docs, 5 fixture, with the two populations counted and floored separately so neither can collapse into the other.

### `main` was held red rather than pushed and fixed forward

The extension surfaced 3 real failures in `docs/research/**`, outside its own scope. The branch was merged **locally** and `origin/main` deliberately left 9 commits behind while the two repairs were dispatched and landed. `make citations` went 3 → 1 → 0; the gate ran once on the complete set.

### The coordinator's brief was false on the load-bearing point

I wrote that a docs tree "has no status, so it is checked unconditionally, as the fixture is". **Documents carry kind-specific status facets and three ADRs are `superseded`** — checking docs unconditionally would have demanded superseded ADRs match today's tree, which is precisely the unsatisfiable condition this ticket family exists to avoid.

The worker defined terminal as **`superseded` only** — the one value in `docs/document-metadata.md` meaning *replaced*, read as `decision_status` on a decision and `disposition` on a research record. Accepted, complete, rejected and informational stay checked, each still being the standing account of its own conclusion. **`implementation_status` is deliberately never consulted**, because the metadata contract calls it a retained high-water mark rather than a live mirror. Retired extents stay writable through the existing bare-path rule, so nothing new was needed for the dated-correction convention.

### ADR 0074: one caught failure, five real ones

The extension flagged one drifted `freeze` citation. Reading the sentence found **all five had drifted** — and four of them still landed *inside* their files, so they resolved silently against unrelated code (723→877, 1021→1960, 826→1115, 412→581). The fifth named a file the terminal had moved out of and pointed past the end of a 142-line file. All five now pin the terminal's own signature, so they break if a terminal stops consuming `self` — the property the Fact actually asserts, rather than a line number that happens to exist.

### A checker gap fixed on merit

Ten upstream-tree citations were spelled as those projects spell them. The script already skipped that category with a stated rationale; its recognizer was version-pinned-path-only. A path is now external when it has a `/` **and its leading segment is a component of no tracked path** — deliberately over components rather than root entries, because `codec/encode.rs` and `semantic/identity.rs` name inner directories, and a root-entry test would call every unresolvable partial path external and silently stop reporting the drift this check exists for. A bare filename is never external, for the same reason.

### Its own first drafts added four failures

The worker's initial versions of the two repair tickets quoted the broken citations in prose, adding four new failures. Fixed by adopting the bare-path-plus-prose-extent convention, with the reason recorded in each. Both repair workers were briefed on that trap and both avoided it, verifying their counts fell by exactly the expected amount with nothing new.

### Demonstrations

Bad path, past-EOF line, and absent anchor each fail by name in the docs population; both population floors fail on an empty corpus; and a **false-but-resolving** anchor still passes, preserving the boundary. The floor demos ran in an isolated fixture repo against a byte-identical script copy, so the real corpus was never edited to make a floor fire — and the superseded skip is what emptied the docs population, proving both mechanisms at once.
