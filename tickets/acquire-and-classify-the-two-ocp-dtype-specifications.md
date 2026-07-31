---
id: acquire-and-classify-the-two-ocp-dtype-specifications
title: Acquire and classify the two OCP dtype specifications
status: todo
priority: p3
dependencies: []
related: [preserve-primary-dtype-standards-evidence]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtype, provenance]
---
## User-visible outcome

The two pending-acquisition rows in `docs/research/numerics/sources/expected-sources.tsv` — `ocp-ofp8-v1.0` and `ocp-mx-v1.0` — become either vendored or metadata-only records, so ADR 0036's three external pins all have a preserved identity and the manifest carries no unreviewed source.

## Why this is split

**Fact:** `preserve-primary-dtype-standards-evidence` attempted both on 2026-07-31 and every request — plain and browser-UA — returned HTTP 403 with a Cloudflare interstitial (`<title>Just a moment...</title>`, 5,979 and 5,858 bytes respectively). No bytes were retrieved, no digest exists, and the licence terms are unreviewed. The manifest records both with the exact error and the official acquisition route, and `verify-sources.sh` enumerates them in its pending-acquisition class, so the gap is checked rather than assumed closed.

## Implementation keys

- Acquire both documents through an interactive browser session or whatever route the OCP site permits; do not scrape around the interstitial with automation that violates the site's terms.
- Review each document's own licence terms document by document. OCP publications vary; do not assume one verdict covers both.
- If redistribution is permitted, vendor the exact retrieved bytes with the required licence material and flip the manifest row to `vendored` with the computed SHA-256. If not, record digest + bibliographic identity + acquisition route and flip to `metadata-only`, discarding the bytes.
- Update `expected-sources.tsv`, the sources README, and the class counts declared at the top of `verify-sources.sh` in the same change — the script fails if the counts and the manifest disagree, which is the intended failure.
- Re-run `verify-sources.sh` and demonstrate one deliberate class-count mismatch failing before restoring it.

## Closes when

Both rows carry a reviewed classification with a computed digest, the verification script passes with its updated population, and ADR 0036's traceability names no pending source.
