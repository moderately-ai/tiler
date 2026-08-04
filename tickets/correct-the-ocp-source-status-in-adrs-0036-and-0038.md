---
id: correct-the-ocp-source-status-in-adrs-0036-and-0038
title: Correct the OCP source status in ADRs 0036 and 0038
status: todo
priority: p3
dependencies: []
related: [acquire-and-classify-the-two-ocp-dtype-specifications, derive-dtype-family-research-tracks-from-the-mature-taxonomy]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, dtype, provenance, adr]
---
## User-visible outcome

A reader of [ADR 0036](../docs/decisions/0036-recognize-standard-binary-and-microscaling-formats.md) or [ADR 0038](../docs/decisions/0038-recognize-ocp-mx-schemes.md) learns that both OCP specifications carry a reviewed identity and an exact digest, instead of learning that they were never retrieved.

## The exact drift

**Fact.** [`acquire-and-classify-the-two-ocp-dtype-specifications`](acquire-and-classify-the-two-ocp-dtype-specifications.md) closed on 2026-07-31: both documents were acquired by hand, their own licence sections were reviewed document by document — OWFa 1.0.2 for OFP8 and OWFa 1.0 for MX — digests were recorded over the exact retrieved bytes, and both rows moved to `metadata-only` because neither carries a self-contained redistribution grant. `docs/research/numerics/sources/verify-sources.sh` now reports `46 records verified (40 vendored, 6 metadata-only, 0 pending-acquisition)`.

**Fact.** Two ADRs still say otherwise. `docs/decisions/0036-recognize-standard-binary-and-microscaling-formats.md:26` reads "both OCP specifications are pending-acquisition, so those pins resolve to"; `docs/decisions/0038-recognize-ocp-mx-schemes.md:26` reads "pending-acquisition after a failed retrieval, with no local copy and no". Reproduce with `grep -rn "pending-acquisition" docs/decisions`.

**Inference.** The claim's practical consequence changed. "Retrieval failed" means the pinned value sets cannot be re-derived at all; `metadata-only` with a digest means they can, through the recorded route, with the result checkable against recorded bytes. A reader planning OCP work acts differently on each.

## Implementation keys

- Correct both spans to state acquisition date, the per-document licence verdict, the digest's existence, and the re-acquisition-plus-digest-check route. Do not restate the digests; the [preservation record](../docs/research/numerics/sources/README.md) owns them.
- Do not change either ADR's `decision_status` or `implementation_status`. This is a source-provenance correction, not a decision movement.
- The three citing records under `docs/research/numerics/` were corrected on 2026-08-04 by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md); check for a fourth before closing, with `grep -rn "pending-acquisition" docs | grep -v sources/`.

## Closes when

Both ADR spans state the current classification, the grep above returns only the preservation record's own machinery, `tkt lint` and `git diff --check` pass.
