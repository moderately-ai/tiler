---
id: catalog-the-kani-verification-research-and-spike
title: Catalog the Kani verification research record and spike
status: todo
priority: p3
dependencies: []
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, catalog, verification]
---
## User-visible outcome

The Kani verification research record and its spike are reachable from the catalogs, so the corpus rule in `docs/document-metadata.md` — "edit the affected catalog entry in the same change that edits the metadata behind it" — is satisfied for them.

## Why this is a separate ticket

`spike-kani-bounded-verification-on-one-inexhaustible-encoder` holds `scopes: [research/verification]`, which maps to `docs/research/verification/**` and `spikes/verification/**`. Both catalogs live in the `contracts/navigation` scope — grep `ticketsplease.toml` for `"contracts/navigation"`, whose glob list carries both `docs/research/README.md` and `spikes/README.md`. The spike worker could not edit them without under-declaring scope, so the rows are preserved verbatim here for whoever holds that scope, per the `AGENTS.md` carrier-ticket practice.

## The two rows to land, verbatim

Into the generated research catalog in `docs/research/README.md`, under **Artifacts, build, and toolchains** (match the surrounding row format and alphabetical position):

```
- [Kani bounded verification of inexhaustible identity encoders](verification/kani-bounded-encoder-verification.md) — pending; executable-model, bounded-measurement, primary-source-synthesis; informs: [Correctness and testing](../correctness-and-testing.md); experiments: [Kani bounded verification of inexhaustible identity encoders](../../spikes/verification/kani-encoder-injectivity/README.md)
```

Into the spike catalog in `spikes/README.md`, matching that file's row format:

```
- [Kani bounded verification of inexhaustible identity encoders](verification/kani-encoder-injectivity/README.md) — reproducible; executable-model, bounded-measurement; supports: [Kani bounded verification of inexhaustible identity encoders](../docs/research/verification/kani-bounded-encoder-verification.md)
```

## Verify before landing

Both rows are written against the frontmatter as it stands at the spike's commit. Read both records' frontmatter and confirm `disposition`, `evidence_classes`, `informs`, and `supports` still say what the rows claim — the rows restate frontmatter and are wrong the moment it moves. `verification/` is a new catalog section subject in both files; check whether the surrounding grouping needs a heading rather than only a row.

## Closes when

Both catalogs carry the rows, the links resolve, and the frontmatter each row restates has been re-read rather than trusted.
