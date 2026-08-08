---
id: retire-adr-0104-s-note-asserting-the-source-comment-still-says-v2
title: Retire ADR 0104 s note asserting the source comment still says v2
status: in-progress
priority: p2
dependencies: []
related: [step-the-coverage-identity-comment-s-stale-semantic-graph-domain]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, identity, documentation]
claimed_from: todo
assignee: w-sol-adr0104
lease_expires_at: 1786206372
---

ADR 0104 carries a note explaining why its quotation was left alone. **The repair it was waiting for has landed, so the note is now the stale part.**

## Facts, coordinator-verified at the merge that landed the repair

**Fact.** The note is anchored by `The stale text is the source comment, not this record`, and it justifies leaving the quotation untouched on the ground that the doc comment on `IndexRefinementExecutableCoverageIdentity` **still says `v2`**.

**Fact.** It no longer does. `step-the-coverage-identity-comment-s-stale-semantic-graph-domain` stepped it to `v3`, dated beside, with the retired spelling quoted in its own note.

**Fact — the direction matters and a coordinator brief got it backwards.** I told that worker the ADR's quotation would become "accurate again by construction" once the source was repaired. The opposite is true, and this ADR says so itself: the quotation was faithful *because* the source said `v2`. Repairing the source is what makes **this note** stale. The worker caught the inversion and reported it rather than working to the brief.

**Fact.** ADR 0104 contains **two** occurrences of `tiler.semantic-graph.v2`, not one: the quotation, and a `Superseded — 2026-08-08` header reading "stepped `tiler.semantic-graph.v2` to `v3`" — a correct historical statement that is **not** a quotation and must not be touched.

## What closes this

The note restated to record that the source was repaired and when, so a reader can tell the quotation is a **historical** one rather than a current reading. **Do not edit the quotation itself** — it remains a faithful record of what the comment said, and the sibling deliberately preserved that by quoting the retired spelling in its own dated note.

**Establish the treatment from history**: this note was true when written, so it is dated beside rather than substituted. That is repository practice — several ADRs state it while applying it and none decides it; cite the practice, not an authority. Say inline that a grep for the retired spelling now lands inside a note, since three of this ADR's occurrences will be exactly that.

**A caveat on anchors, verified both ways by the sibling.** The ADR's *full* quoted sentence does **not** grep in `refinement.rs` and never did — the doc comment wraps it across three lines at 80 columns, so it returned 0 before and after. Only the short fragment resolves. Choose a short, break-free anchor and **run its grep before committing to it**; note also that unescaped brackets read as a character class, so `grep -F` where a citation contains them.

**Check this ADR's other claims about the tree and name the count.** A prior sweep of a sibling ADR found 9 of 17 tree-claim clusters false, most predating the landing that prompted the ticket — so assume the neighbours here are unexamined rather than clean.
