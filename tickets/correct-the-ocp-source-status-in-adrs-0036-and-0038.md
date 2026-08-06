---
id: correct-the-ocp-source-status-in-adrs-0036-and-0038
title: Correct the OCP source status in ADRs 0036 and 0038
status: review
priority: p3
dependencies: []
related: [acquire-and-classify-the-two-ocp-dtype-specifications, derive-dtype-family-research-tracks-from-the-mature-taxonomy]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, dtype, provenance, adr]
claimed_from: todo
assignee: agent-adr-fixes
lease_expires_at: 1786050483
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

## Outcome — 2026-08-06

Both spans corrected at base `76fe3a8e` and delivered as `13c46d52`, the first of two commits on `tkt/correct-the-ocp-source-status-in-adrs-0036-and-0038`. Two ADR files changed, both under `contracts/decisions`.

**Fact — the classification was read from the preservation record before either span was written.** `docs/research/numerics/sources/README.md` was read in full at the `## Hand-acquired records` section. Both OCP rows carry **Acquired 2026-07-31** by an interactive browser session, a SHA-256 digest over the exact retrieved PDF, a licence reviewed in the document's own Section 1 — OWFa 1.0.2 incorporated by reference for `ocp-ofp8-v1.0`, OWFa 1.0 for `ocp-mx-v1.0`, the different revision being why the two were reviewed separately — and a `metadata-only` verdict on the ground that neither document carries a self-contained redistribution grant. No digest is restated in either ADR; the preservation record keeps sole ownership of both.

**Fact — the IEEE row moved too, and the corrected 0036 span had to account for it.** The span bundled IEEE 754-2019 with the two OCP rows under one "resolve to an acquisition route rather than a local copy" clause. That row is now `metadata-only` **with a digest** (purchased and relayed 2026-08-06), so the whole clause is now the re-acquire-and-check route rather than a bare URL, and the corrected sentence says so for all three rather than leaving IEEE described by a clause that no longer fits it.

**The corrected spans.**

- `docs/decisions/0036-recognize-standard-binary-and-microscaling-formats.md:25` now reads: "The RISC-V BF16 contract is vendored there; IEEE 754-2019 and both OCP specifications are metadata-only. The OFP8 and MX documents were acquired on 2026-07-31 and each licence was reviewed in its own document — OWFa 1.0.2 by reference for OFP8, OWFa 1.0 by reference for MX — and neither carries a self-contained redistribution grant, so neither is vendored. Every metadata-only row there records a digest over the exact reviewed bytes, so these pins resolve by re-acquiring the document through the recorded official route and checking it against that digest rather than by reading a local copy."
- `docs/decisions/0038-recognize-ocp-mx-schemes.md:25` now reads: "The OCP MX version 1.0 specification this decision pins is recorded there as metadata-only: it was acquired on 2026-07-31, its licence was reviewed in the document itself — OWFa 1.0 incorporated by reference, with no self-contained redistribution grant — so no copy is vendored, and the row records a digest over the exact reviewed bytes. Re-deriving this decision's pinned scheme definitions means re-acquiring the document through the official route the record names and checking it against that digest."

Neither `decision_status` nor `implementation_status` moved in either file; the diff is the traceability bullet and nothing else.

**Fact — the sweep found a fourth citing record, and it is outside this ticket's scopes.** `grep -rn "pending-acquisition" docs | grep -v sources/` returns exactly one line after the correction: `docs/research/numerics/dtype-family-research-tracks.md:277`, whose finding 5 reads "[ADR 0036] and [ADR 0038] still carry the stale claim and are corrected by a filed ticket, because they are outside this ticket's scopes". That sentence is the *predecessor's* accurate note about the gap this ticket closes, and this commit falsifies its present tense. The file is owned by `research/numerics`, which this ticket does not hold, so it was left alone and is flagged for the coordinator rather than edited. `grep -rn "pending-acquisition" docs/decisions` now exits 1 with no output, against two lines at base.

**Checks, docs-and-tickets only.** No `crates/`, `prototypes/`, or build-configuration path is touched, so no Cargo gate applies. `tkt lint` → `ok: no problems found`. `git diff --check` → no output.
