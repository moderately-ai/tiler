---
id: refine-the-searchable-anchor-guidance-for-anchors-broken-by-inline-links
title: Refine the searchable anchor guidance for anchors broken by inline links
status: done
priority: p2
dependencies: []
related: [correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling, correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [process, documentation, citations]
---

`AGENTS.md` tells every worker to **"Cite by searchable anchor, not by line number,"** on the reasoning that "a line number rots silently and sends a reader into unrelated code; a quoted distinctive phrase or a symbol name fails loudly and can be re-located." That reasoning is sound and the rule should stay. But the rule as written has a failure mode it does not warn about, and the coordinator hit it twice on the day the rule was added.

## Facts, verified 2026-08-08 at `1fab9547`

*Re-verified 2026-08-08 by the worker at base `6eabf97e`. Two of the three original Facts were wrong; the retired wording is corrected in place below rather than deleted, and the underlying defect survives the repair.*

**Fact — an anchor quoted from rendered prose can be unsearchable in the source.** Verified, but not with the sentence originally cited. The ticket read: "`grep -c` for [*The drafted body in the source record deliberately still carries the pre-rename spelling*] in `docs/decisions/0092-…md` returns **0**." It returns **1**, and already did at `1fab9547`. Commit `accaed84` withdrew that sentence and, per the repository's convention, quoted the retired wording verbatim and unlinked inside a dated correction note — so the anchor the ticket predicted would be missing is present as retired text. The live sentence carries the same defect: the source spells it `The drafted body in [the source record](../research/…) carries the corrected spelling too`, so the rendered flat form returns **0** while the link-free fragment `carries the corrected spelling too` returns **1**.

**False — hard-wrapping is not one of the causes, in this repository.** The ticket claimed `docs/architecture.md` and its citing record "both hard-wrap", so `the one crate a consumer names` needs `tr '\n' ' ' | tr -s ' '` first. Neither file wraps: `architecture.md` is 623 lines with a longest line of 1920 characters, the citing record's longest is 2687, and unwrapping does **not** rescue the anchor (`tr '\n' ' ' < docs/architecture.md | tr -s ' ' | grep -c "the one crate a consumer names"` returns 0). That anchor fails for a different reason — it drops a qualifier, since the source reads `is the one crate an inline-frontend consumer names` — which is the subject of the related citation ticket, not of this one.

**Fact — a line break does defeat the grep, but in wrapped code comments, not prose.** Prose here is unwrapped by convention, so the cause reaches `//!` doc comments. In `crates/tiler-ir/src/semantic/gather.rs`, `public boundary under ADR 0075 until Tom accepts its exact included` returns **0** because the comment wraps mid-sentence, while the single-line `until Tom accepts its` returns **1**; unwrapping the file rescues the spanning phrase, confirming the line break is the cause.

**Fact — emphasis markers defeat the grep.** `crates/tiler-ir/src/semantic/gather.rs` writes `*labelled draft*`, so `labelled draft public boundary under ADR 0075` returns **0** while the line plainly exists. Unwrapping does not rescue it, isolating emphasis from the line-break cause above.

**Inference — this is the failure the rule was written to prevent, arriving by another route.** A line number "fails loudly"; an unsearchable anchor **fails silently as absence**, and absence is the more dangerous reading, because a worker concludes the text was removed and may then "restore" a claim that was there all along. `AGENTS.md` already warns "A failed search does not prove absence" — but nothing connected it to the anchor rule. *Imprecise as originally written, corrected by the worker:* the two are not "in a different section". Both sit under `## Read critical context first`; the warning is in that section's preamble and the anchor rule four paragraphs below it in the `### A ticket's stated Facts are stale until re-read at your own base` subsection. The second quoted phrase, "When grep returns nothing unexpected, read the file", is not in `AGENTS.md` at all — it is from the user's global `CLAUDE.md`, and citing it as `AGENTS.md` text was itself an unsearchable-anchor failure of exactly the kind this ticket describes.

## What closes this

The anchor guidance amended so it names its own failure mode and says how to pick an anchor that survives: prefer a **distinctive fragment with no inline link, emphasis marker, or line break inside it** — usually the shortest unique clause — over a full sentence copied from the rendered view. Say that a full-sentence anchor should be verified by actually running the grep before it is handed to anyone, which is the same obligation the coordinator section already imposes on supplied commands.

Connect it explicitly to the existing "a failed search does not prove absence" sentence, so the two rules are read together rather than a section apart.

**Do not add a mechanical checker for this.** An anchor's searchability is only meaningful against the file the citation names, and `AGENTS.md` already records that a mechanical check does not discharge a reading obligation. The fix here is the wording of an instruction, not a new gate — and a gate that resolved anchors would itself be one more thing that can quietly stop working.

Note the rule is being applied correctly in the meantime: every worker handed a broken anchor this week located the real text anyway and reported the anchor as imprecise, which is exactly the loud failure the rule intends. This ticket lowers the cost of that, it does not repair a breakage.

## Outcome audit — 2026-08-09

Delivered by `713582475b842b20da0200b1b1125a8d7824fa85`. `AGENTS.md` now names rendered inline links, emphasis markers, and line breaks as ways an apparently literal anchor can fail as false absence; directs readers to prefer the shortest distinctive source fragment without those constructs; requires a full-sentence anchor to be tested against its named file before handoff; and explicitly connects the rule to `A failed search does not prove absence`. The repair remained guidance rather than adding a mechanical checker, exactly as scoped.
