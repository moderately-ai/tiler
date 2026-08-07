---
id: vendor-the-tuning-loop-primary-sources-after-reading-each-licence
title: Vendor the tuning-loop primary sources after reading each licence
status: done
priority: p3
dependencies: []
related: [design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: [research, cost-model, documentation]
---
## User-visible outcome

`docs/research/cost-model/sources/` classifies each of its reachable records licence-aware rather than uniformly metadata-only, and vendors the bytes of every document whose own terms permit redistribution, so a claim in [the measured-feedback tuning loop](../docs/research/cost-model/measured-feedback-tuning-loop.md) can be checked against a preserved file rather than against a re-retrieval that may serve different bytes.

**Population update, 2026-08-07.** This ticket was written when the record held eleven reachable rows. [`acquire-the-three-unreachable-adaptive-execution-sources`](acquire-the-three-unreachable-adaptive-execution-sources.md) has since retrieved and recorded two more — `markl-pop-sigmod-2004` and `cole-graefe-sigmod-1994` — under the same metadata-only classification and with no licence verdict reached, so the population is **thirteen**. Read "eleven" below as "thirteen"; nothing else about this ticket's scope changes.

## Why this exists

**Fact, 2026-08-07.** The source record was written with every row in one class — metadata-only with a retrieval fingerprint — and it says so explicitly: no document had its licence terms read, so no redistribution verdict was reached for any of them. That is the honest classification for a record whose author did not do the reading. It is *not* the classification the [rewrite-search source record](../docs/research/region-search/sources/README.md) and the [numerics source record](../docs/research/numerics/sources/README.md) established, which is licence-aware and per-document: ten of thirty region-search records are vendored because each one's own text or arXiv submission metadata carries an explicit dissemination grant.

**Inference.** Three of the eleven rows are arXiv submissions (AutoTVM `1805.08166v2`, Ansor `2006.06762v5`, TVM `1802.04799v3`) whose submission licence is plausibly permissive, and plausibility is not evidence — the region-search record's own `egg` row makes the point that an arXiv grant is *submission metadata rather than text inside the PDF*, so a reader verifying the verdict checks the abstract page and not the preserved bytes. Each of the eleven needs that reading done and recorded.

## Scope

For each of the eleven reachable rows: read the licence where the document or its submission metadata states one, record what was read and where it was read, reach a vendored / metadata-only verdict, and vendor the bytes where the verdict permits. Where a licence could not be read, say so and keep the row metadata-only — that is a verdict too.

The digests already recorded are the identity a re-acquisition is checked against; a re-retrieval that produces different bytes is a finding and not a refresh. Add an `expected-sources.tsv` and a `verify-sources.sh` matching the region-search convention only if bytes are actually vendored, since a manifest over an empty vendored set checks nothing.

## Outcome, 2026-08-07

**All thirteen licences were read or attempted; zero rows are vendorable; no bytes were checked in.** Ten documents had their terms read and none carries a grant this repository may redistribute under. Three could not be read because the host serving them refused this client, and their verdict is `metadata-only` on unread terms — a complete outcome under this ticket's own rule, not a remainder.

**The ticket's central inference is refuted.** The three arXiv rows — AutoTVM `1805.08166v2`, Ansor `2006.06762v5`, TVM `1802.04799v3` — were expected to be "plausibly permissive". All three carry the default `arXiv.org perpetual, non-exclusive license 1.0`: *"I grant arXiv.org a perpetual, non-exclusive license to distribute this article."* That is a grant to arXiv and nothing to a third party. No Creative Commons grant appears on any of the three abstract pages, and none of the three PDFs carries a rights statement of its own. The plausibility was correctly identified as not-evidence, and reading it turned it into evidence pointing the other way.

The ten read verdicts fail for four distinguishable reasons: five modern ACM notices refuse redistribution outright ("to post on servers … requires prior specific permission and/or a fee"); three arXiv non-exclusive licences grant a third party nothing; the *Foundations and Trends* AQP survey carries no grant at all, only a bare `© 2007 A. Deshpande, Z. Ives and V. Raman` line, so it fails on absence rather than refusal; and Cole and Graefe's 1994 notice does grant copying but conditions it on the copies not being for "direct commercial advantage", which this commercial repository does not clearly satisfy — a judgement, recorded as one, on the same reading the rewrite-search record applies to `spores-pvldb-13-11-2020` and `sparse-extraction-oopsla-2024`.

**Blocked and handed to Tom — three rows, two hosts, one lookup each.** `www.vldb.org` returned HTTP 403 host-wide to a plain `curl` (its own site root too), blocking `pqo-vldb-1992` and `plan-diagrams-vldb-2005`; `escholarship.org` returned HTTP 202 with an empty body for every path tried, blocking `halide-autoscheduler-2019`. Each was retried exactly once, identically, to distinguish a transient failure from a standing refusal, and each refused again. **No evasion was attempted on any of them** — no user-agent change, no headed browser, no alternative host sought for the same document. `halide-autoscheduler-2019` is the lowest-value of the three, because the rewrite-search record read the ACM notice of a *different copy* of the same article on 2026-08-05 and found no grant; `plan-diagrams-vldb-2005` is the highest, because the design record's refusal of a per-shape winner table rests on its measured 68-plans-to-7 geometry.

**No `expected-sources.tsv` and no `verify-sources.sh` were added**, per this ticket's own condition: zero bytes are vendored, so every per-file assertion such a check makes would be vacuous. Recorded as a mild disagreement rather than silently: a manifest would still assert the declared population count and id uniqueness, which are not vacuous, and the record now carries those in prose instead. If any row is ever vendored, both files must land in the same change.

**Cross-record finding the ticket did not anticipate.** Four of these thirteen documents are also preserved under the [rewrite-search record](../docs/research/region-search/sources/README.md) — `ansor`, `tvm`, and both Halide autoschedulers. That record reached its licence verdicts independently on 2026-08-05 and **this reading agrees with all four**. Three are the same bytes as well (identical digests, same URLs); `halide-autoscheduler-2019` is held as two different byte streams under two hosts. The overlap is legitimate — each record closes over its own design record's citations — but it creates a maintenance obligation now named in both this ticket and the cost-model record: a future re-reading that moves one of those four rows must move it in both records, or the repository will hold two answers to one licence question. **The sibling record was not edited**, per this ticket's scope.

## Non-goals

Re-reading the papers or revising any conclusion in the design record. This ticket changes the preservation classification and nothing else. The one remaining awaiting-retrieval row stays out of scope; [`acquire-the-three-unreachable-adaptive-execution-sources`](acquire-the-three-unreachable-adaptive-execution-sources.md) owns it.

## Outcome — done, 2026-08-07

Landed at merge **`81f07934`**. **Thirteen sources read or attempted; zero vendorable; zero bytes checked in.** +27 KB of text, no binary. Delta is `docs/` and `tickets/` only, so it carries the green gate.

The reading was the deliverable and it inverted the record's own expectation. Its three arXiv rows were classified "plausibly permissive" as an **Inference** — and reading refuted it: all three carry the default arXiv non-exclusive licence, which grants arXiv distribution rights and not ours. Five ACM papers carry the explicit "requires prior specific permission and/or a fee" clause. The AQP survey has **no grant at all**, only a copyright line, which the worker correctly recorded as absence rather than refusal. Cole & Graefe grants copying but only where "not made or distributed for direct commercial advantage" — recorded as a judgement call rather than smuggled through as a verdict.

**Bytes were re-verified before terms were read.** The record retains no bytes, so each document was re-retrieved and its digest compared against the recorded SHA-256 *before* any licence reading. Ten reproduced exactly; none changed.

### The access-control discipline held, including where it cost something

`www.vldb.org` returned HTTP 403 host-wide and `escholarship.org` returned HTTP 202 with an empty body. Each was retried **once, identically** — same URL, same client — to separate a transient failure from a standing refusal, then abandoned. No user-agent change, no headed browser, and **no alternative host sought for the same document even though one exists** (the Halide project serves that article, and a sibling record already uses that copy). That last restraint was deliberate and correct: the verdict was unchanged either way, so fetching would have added nothing but a step toward the line the rule draws.

### Two judgement calls recorded rather than acted on

The `expected-sources.tsv` / `verify-sources.sh` pair was not built, with the disagreement recorded in the ticket instead: a manifest would still assert population count and id uniqueness, which are not vacuous. And the three licence-unread rows **stay in the reachable class** rather than moving to "awaiting retrieval" — their bytes were retrieved and the design record's claims checked against them, so only the licence reading is missing; reclassifying them would falsely imply the design record cites unread documents.

The ticket also missed that **four of its thirteen documents were already preserved under the region-search record**, which reached licence verdicts for them on 2026-08-05; the independent reading agreed with all four.
