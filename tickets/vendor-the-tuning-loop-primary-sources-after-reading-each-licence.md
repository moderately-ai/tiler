---
id: vendor-the-tuning-loop-primary-sources-after-reading-each-licence
title: Vendor the tuning-loop primary sources after reading each licence
status: todo
priority: p3
dependencies: []
related: [design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: [research, cost-model, documentation]
---
## User-visible outcome

`docs/research/cost-model/sources/` classifies each of its eleven reachable records licence-aware rather than uniformly metadata-only, and vendors the bytes of every document whose own terms permit redistribution, so a claim in [the measured-feedback tuning loop](../docs/research/cost-model/measured-feedback-tuning-loop.md) can be checked against a preserved file rather than against a re-retrieval that may serve different bytes.

## Why this exists

**Fact, 2026-08-07.** The source record was written with every row in one class — metadata-only with a retrieval fingerprint — and it says so explicitly: no document had its licence terms read, so no redistribution verdict was reached for any of them. That is the honest classification for a record whose author did not do the reading. It is *not* the classification the [rewrite-search source record](../docs/research/region-search/sources/README.md) and the [numerics source record](../docs/research/numerics/sources/README.md) established, which is licence-aware and per-document: ten of thirty region-search records are vendored because each one's own text or arXiv submission metadata carries an explicit dissemination grant.

**Inference.** Three of the eleven rows are arXiv submissions (AutoTVM `1805.08166v2`, Ansor `2006.06762v5`, TVM `1802.04799v3`) whose submission licence is plausibly permissive, and plausibility is not evidence — the region-search record's own `egg` row makes the point that an arXiv grant is *submission metadata rather than text inside the PDF*, so a reader verifying the verdict checks the abstract page and not the preserved bytes. Each of the eleven needs that reading done and recorded.

## Scope

For each of the eleven reachable rows: read the licence where the document or its submission metadata states one, record what was read and where it was read, reach a vendored / metadata-only verdict, and vendor the bytes where the verdict permits. Where a licence could not be read, say so and keep the row metadata-only — that is a verdict too.

The digests already recorded are the identity a re-acquisition is checked against; a re-retrieval that produces different bytes is a finding and not a refresh. Add an `expected-sources.tsv` and a `verify-sources.sh` matching the region-search convention only if bytes are actually vendored, since a manifest over an empty vendored set checks nothing.

## Non-goals

Re-reading the papers or revising any conclusion in the design record. This ticket changes the preservation classification and nothing else. The three unreachable rows stay unreachable; [`acquire-the-three-unreachable-adaptive-execution-sources`](acquire-the-three-unreachable-adaptive-execution-sources.md) owns those.
