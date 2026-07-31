---
id: acquire-and-classify-the-two-ocp-dtype-specifications
title: Acquire and classify the two OCP dtype specifications
status: done
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

## Outcome (2026-07-31)

Both documents were acquired by Tom through interactive browser sessions on the OCP document pages — the route this ticket required — after the recorded automated attempts hit the Cloudflare interstitial. The coordinator performed the review and classification directly.

**Licence review, document by document.** OFP8 Revision 1.0's Section 1 governs usage by the "Open Web Foundation Modified Final Specification Agreement (OWFa 1.0.2)"; the MX v1.0 Specification's Section 1 names OWFa **1.0** — a different revision, which is why the two were reviewed separately rather than one verdict covering both. In both, the executed (modified) agreement text is held by OCP rather than carried in the document, so neither carries a self-contained redistribution grant.

**Verdict: both `metadata-only`, fail-closed.** Digests recorded over the exact retrieved bytes — OFP8 `1e1ebad11388cdc1cdb4afa7e226b78f18d4049c6f39c36ecacd747e9ca3c08b` (564,311 bytes), MX `d195d6a36dd4a0c89064af0c479bcaad5c0fe29d63f628502ea6d7c4b4279421` (812,323 bytes) — bytes not vendored. `expected-sources.tsv`, the sources README (both records, the no-bytes summary, and the closing gap paragraph), and `verify-sources.sh`'s declared counts (metadata-only 4→6, pending 2→0) moved in one change. The class-count check was perturbed to `expect_metadata_only=5`, observed failing (`FAIL: metadata-only records: 6, expected 5`, exit 1), restored, and the clean run passes over all 46 declared records. ADR 0036's three external pins now all have preserved identities and the manifest carries no pending source.
