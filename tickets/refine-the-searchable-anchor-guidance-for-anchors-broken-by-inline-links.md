---
id: refine-the-searchable-anchor-guidance-for-anchors-broken-by-inline-links
title: Refine the searchable anchor guidance for anchors broken by inline links
status: in-progress
priority: p2
dependencies: []
related: [correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling, correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [process, documentation, citations]
claimed_from: todo
assignee: coord
lease_expires_at: 1786170921
---

`AGENTS.md` tells every worker to **"Cite by searchable anchor, not by line number,"** on the reasoning that "a line number rots silently and sends a reader into unrelated code; a quoted distinctive phrase or a symbol name fails loudly and can be re-located." That reasoning is sound and the rule should stay. But the rule as written has a failure mode it does not warn about, and the coordinator hit it twice on the day the rule was added.

## Facts, verified 2026-08-08 at `1fab9547`

**Fact — an anchor quoted from rendered prose can be unsearchable in the source.** The coordinator gave a worker the anchor *"The drafted body in the source record deliberately still carries the pre-rename spelling."* `grep -c` for that string in `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md` returns **0**. The source spells it `The drafted body in [the source record](../research/…) deliberately still carries…` — an inline markdown link splits the sentence. The substring `deliberately still carries the pre-rename spelling` returns **1**.

**Fact — the same class defeats a grep through hard-wrapping and through emphasis markers.** `docs/architecture.md` and its citing record both hard-wrap, so neither `the one crate a consumer names` nor its qualified original is findable line-wise; both need `tr '\n' ' ' | tr -s ' '` first. Separately, `crates/tiler-ir/src/semantic/gather.rs` writes `*labelled draft*`, so a grep for `labelled draft public boundary under ADR 0075` finds nothing while the line plainly exists.

**Inference — this is the failure the rule was written to prevent, arriving by another route.** A line number "fails loudly"; an unsearchable anchor **fails silently as absence**, and absence is the more dangerous reading, because a worker concludes the text was removed and may then "restore" a claim that was there all along. `AGENTS.md` already warns "A failed search does not prove absence" and "When grep returns nothing unexpected, read the file" — but that warning sits in a different section from the anchor rule, and nothing connects them.

## What closes this

The anchor guidance amended so it names its own failure mode and says how to pick an anchor that survives: prefer a **distinctive fragment with no inline link, emphasis marker, or line break inside it** — usually the shortest unique clause — over a full sentence copied from the rendered view. Say that a full-sentence anchor should be verified by actually running the grep before it is handed to anyone, which is the same obligation the coordinator section already imposes on supplied commands.

Connect it explicitly to the existing "a failed search does not prove absence" sentence, so the two rules are read together rather than a section apart.

**Do not add a mechanical checker for this.** An anchor's searchability is only meaningful against the file the citation names, and `AGENTS.md` already records that a mechanical check does not discharge a reading obligation. The fix here is the wording of an instruction, not a new gate — and a gate that resolved anchors would itself be one more thing that can quietly stop working.

Note the rule is being applied correctly in the meantime: every worker handed a broken anchor this week located the real text anyway and reported the anchor as imprecise, which is exactly the loud failure the rule intends. This ticket lowers the cost of that, it does not repair a breakage.
