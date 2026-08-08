---
id: step-the-coverage-identity-comment-s-stale-semantic-graph-domain
title: Step the coverage identity comment s stale semantic graph domain
status: in-progress
priority: p2
dependencies: []
related: [repair-the-records-the-sourced-semantic-shape-falsifies, pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786182058
---

A doc comment names a semantic-graph domain that has since stepped. **The interesting part is where the repair must land**: an accepted ADR quotes this comment verbatim, so the ADR is faithful and the source is what drifted.

## Facts, coordinator-verified at the merge that found it

**Fact.** `crates/tiler-ir/src/index/refinement.rs`, on `IndexRefinementExecutableCoverageIdentity`, names `tiler.semantic-graph.v2`. The live constant is `tiler.semantic-graph.v3`, declared in `crates/tiler-ir/src/semantic/identity.rs`.

**Fact — and this is why the repair is here and not in the ADR.** `docs/decisions/0104-…md` contains exactly **one** occurrence of `tiler.semantic-graph.v2`, and it sits inside a verbatim quotation of this comment. Editing the ADR would make it misquote its own source. Repair the comment; the ADR's quotation then becomes accurate again by construction. **Do not touch `docs/decisions/**`** — `contracts/decisions`, not this scope.

**Fact — a prior ticket claimed two occurrences at two locations. There is one, and there was one at the ADR's landing commit too.** The second cited location carries `request-subject.v5`, an unrelated domain. Do not go looking for a second site.

## Why p2 rather than p3

The comment describes what a coverage identity folds. A reader who takes `v2` at face value concludes the coverage identity is pinned to a superseded graph domain, which is exactly the kind of wrong premise that produces a wrong identity argument downstream. It is one line and no behaviour, but the claim is load-bearing where it is read.

## What closes this

The comment naming the live domain. **Check whether the sentence is still true once the name is corrected** — a domain step is not always a pure rename, and a comment that was right about `v2` is not automatically right about `v3`. Read `semantic/identity.rs` and confirm what the coverage identity actually folds before changing the digit.

**Then re-read the ADR's quotation** and confirm it now matches the source byte for byte. If it does not, the quotation was already inexact in some other way and that is a separate finding — report it rather than editing the ADR.

**Cite by searchable anchor, not line number.** Doc comments here wrap at 80 columns, so an anchor spanning a line break greps as **absent** — the failure mode `AGENTS.md` records and which has bitten three tickets this week. Run your anchor's grep before committing to it.

**Check this file for other domain names while you are in it, and name the count.** A sibling audit found no test asserts any identity domain string anywhere in the tree, so a stale domain name in prose has nothing catching it — see `pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate`, which is live in this same scope and may land first. Coordinate rather than duplicating: if that work lands a census, reuse it.
