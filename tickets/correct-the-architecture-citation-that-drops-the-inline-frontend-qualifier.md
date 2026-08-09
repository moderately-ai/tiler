---
id: correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier
title: Correct the architecture citation that drops the inline frontend qualifier
status: done
priority: p1
dependencies: []
related: [repair-the-eight-dangling-links-in-the-runtime-route-answer-record, accept-the-public-route-requirement-answer-boundary]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [citations, contracts, documentation]
---

`docs/research/runtime/backend-scoped-route-requirement-answers.md` cites `docs/architecture.md` for a sentence that document does not contain, dropping the qualifier the sentence turns on. **This is the failure class the citation checker cannot catch**: the pin resolves, so nothing goes red, and the claim is wrong anyway.

*Corrected 2026-08-08 by the worker at `0f319ec8` — the framing above implies an authorial error, and it was not one.* Both quotations were **accurate and correctly pinned when written**. `git show 6f7caf3:docs/architecture.md | sed -n '389p'` — `6f7caf3` being the base the record declares in its own opening line — returns the frontend-pair paragraph carrying "`tiler` is the one crate a consumer names" verbatim, flat, with no carve-out clause and with the pair called the *consumer* frontend pair. `52e088a2` ("State the consumer-neutral compiler mission explicitly", 2026-08-04) added the qualifier and the carve-out and pushed the paragraph to `:435`. So this is **citation rot, not misquotation**: the source moved and the line-number form cannot say so. That does not reduce the severity — a reader today still reaches a false claim — but it changes the remedy from "check the author's care" to "stop pinning contract sentences by line", which is what closing this ticket does. The one genuine misquotation at the same site is the `crates/tiler/src/lib.rs` quotation beside it, which drops the crate's emphasis on *that* contract and ends at a full stop the source spells as a colon.

## Facts, verified 2026-08-08 by the coordinator at `bdbeb2b5`

**Fact — verified at `0f319ec8`.** The record asserts **twice** that `docs/architecture.md` states *"`tiler` is the one crate a consumer names."* `docs/architecture.md` says *"the one crate an inline-frontend consumer names."*

```sh
tr '\n' ' ' < docs/research/runtime/backend-scoped-route-requirement-answers.md | tr -s ' ' | grep -o "the one crate a[^\"]\{0,40\}consumer names" | sort | uniq -c
# 2 the one crate a consumer names
tr '\n' ' ' < docs/architecture.md | tr -s ' ' | grep -o "the one crate a[^\"]\{0,40\}consumer names" | sort | uniq -c
# 1 the one crate an inline-frontend consumer names
```

*Corrected 2026-08-08 by the worker at `0f319ec8` — two claims attached to that reproduction were false, and neither changes the Fact.* **First, the qualifier carries no emphasis in the source.** This ticket and [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md):25 both spell it `**inline-frontend**`; `grep -c -- "\*\*inline-frontend\*\*" docs/architecture.md` returns `0`, so the bold is the citing documents' own and the verbatim sentence has none. **Second, neither file hard-wraps the sentence, and a line-oriented grep finds both strings** — `grep -n "one crate an inline-frontend consumer names" docs/architecture.md` returns line 435, and `grep -n "one crate a consumer names" docs/research/runtime/backend-scoped-route-requirement-answers.md` returns lines 257 and 317. Longest lines are 1,920 and 2,687 characters respectively. The collapsed-whitespace form above is still the better reproduction because it *counts* occurrences across both files in one shape, but the stated reason for needing it was wrong, and a reader who believed it would skip the cheapest check available.

**Fact.** The qualifier is load-bearing, not decorative. The same paragraph in `docs/architecture.md` explicitly carves out consumers constructing arbitrary semantic programs, denying that `tiler` is the facade for them. The unqualified sentence states a flat monopoly the architecture document specifically refuses.

**Fact — verified at `0f319ec8`.** The citation is pinned to `docs/architecture.md:389`, and `sed -n '389p'` returns an **empty line**. The sentence is at `:435`. So the pin is stale by number as well as wrong by content, and the checker passes it anyway.

*Corrected 2026-08-08 by the worker at `0f319ec8` — the stated mechanism was wrong.* Partial-path resolution is not what passes this citation. `check-citations.sh` reaches its suffix index only when `exists(path)` is false (`check-citations.sh "if (suffix_count[path] == 1)"`); `docs/architecture.md` is an exact tracked path, so that branch never runs. The citation passes because the **only** test applied to a bare line pin is that it is not past end of file — `check-citations.sh "line %d is past end of file"` — and `389 ≤ 623`. That is the sharper statement of the same point: a line pin is checked for being *in range*, never for landing on content, so a pin onto a blank line is indistinguishable from a correct one. An anchor is the form that can say no.

**Inference — why this is p1 and not a typo.** ADR 0092 decision item 6 rests on this sentence, and `accept-the-public-route-requirement-answer-boundary` holds a reserved item that turns on whether a **dispatching** consumer may name `tiler-metal` — a distinction the unqualified quote erases entirely. A reader reaching the misquote concludes the boundary question is already answered. It is not.

**Fact — the census, run at `0f319ec8`.** Two quotations in this corpus attribute text to `docs/architecture.md` and get it wrong, and both are the sites above. Enumerated from `grep -rn 'architecture\.md' docs/ tickets/` filtered to lines carrying a quotation, then each quoted string checked against source with whitespace collapsed:

- **In the record:** five `architecture.md` mentions. `:257` and `:317` are the two defects. `:342` is the record's own prose *about* the fenced span's `../architecture.md` link and quotes nothing from it. `:379` and `:411` are inside the fenced byte-identical ADR 0092 span — `:379` is decision item 6, which names the sentence by the flat shorthand and directs a restatement without saying which qualifier is being added; it is `contracts/decisions` authority and is reported rather than edited.
- **Siblings under `docs/research/runtime/`:** one mention, `autoregressive-state-and-kv-cache.md:49`, a bare link to `../../architecture.md#initial-placement-execution-and-buffer-model`. No quotation. Heading exists at `:286`. Clean.
- **Wider `docs/research/`:** two more quoting sites, both verified accurate. `program-planning/minimum-correct-physical-realization-profile.md:47` quotes "If no variant's preflight guards hold, the Tensor-level integration invokes a defined fallback rather than entering an unsafe custom operation." — present verbatim. `extensions/backend-provider-composition.md:433` states in the **past tense** that architecture.md "had read" the phrase "remains the live profile until that promotion lands"; the phrase is absent from the file today, which is what that sentence claims, and the `#accepted-prototype-packaging-profile` heading it links exists at `:388`.
- **No other `docs/` or `tickets/` file quotes `docs/architecture.md` for the "one crate" sentence** except `accept-the-public-route-requirement-answer-boundary.md`, which already carries the correction at `:25` and is outside this ticket's scopes.

## What closes this

Both occurrences quoting `docs/architecture.md` accurately, with the qualifier, cited by searchable anchor rather than by line number — the line pin is exactly what rotted here. Where the record's argument depended on the flat reading, say what the qualified sentence does and does not support; do not quietly re-attach the qualifier to a sentence whose conclusion needed the flat version.

**Do not treat this as the only instance.** It survived because it reads plausibly and resolves cleanly. Grep the record — and its siblings — for other quotations attributed to `docs/architecture.md` and verify each against the source with whitespace collapsed. Report the census either way, so "no others" is distinguishable from "did not look".

## Outcome audit — 2026-08-09

Delivered by `2a7a5de1306629aa5daf649b51e8beb41923dae8`. The two live quotations in the backend-scoped route-answer record now carry the source's `inline-frontend` qualifier and cite the architecture sentence by the searchable anchor `is the one crate an inline-frontend consumer names` instead of a line number. Their surrounding reasoning now distinguishes the frontend axis from the dispatch axis and preserves the separate facade question. The genuine neighbouring `crates/tiler` quotation defect was repaired with the same source read; ADR 0092's transferred item-6 shorthand was reported rather than edited out of this research scope and was subsequently repaired by its own completed tickets.
