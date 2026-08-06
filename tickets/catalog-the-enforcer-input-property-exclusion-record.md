---
id: catalog-the-enforcer-input-property-exclusion-record
title: Catalog the enforcer input-property exclusion record
status: todo
priority: p3
dependencies: []
related: [close-the-enforcer-input-property-exclusion-gap]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, navigation]
---
## User-visible outcome

[The enforcer input-property exclusion record](../docs/research/region-search/enforcer-input-property-exclusion.md) appears in the research catalog, so a reader browsing `docs/research/README.md` finds it rather than only reaching it through the formalism record's two inbound links.

## Why this exists

**Fact — a scope boundary, not an oversight.** The record landed under [`close-the-enforcer-input-property-exclusion-gap`](close-the-enforcer-input-property-exclusion-gap.md), whose `research/region-search` scope maps to `docs/research/region-search/**` and `spikes/region-search/**` only. `docs/research/README.md` is `contracts/navigation`, which that ticket did not hold, so its worker could not add the catalog row and filed this instead of leaving the corpus silently inconsistent.

## What to add

One row in the research catalog's `physical-planning-lowering` group, beside the two existing `region-search` rows at `docs/research/README.md:84` and `:99`, rendered in the same shape those rows use. From the record's frontmatter: disposition `informational`, evidence classes `primary-source-synthesis`, `depends_on` [The rewrite-search formalism](../docs/research/region-search/rewrite-search-formalism.md). It has no `informs` edge and no experiment, so the row carries neither clause.

Read the record's frontmatter at the time of the edit rather than trusting the summary above — the catalog's value is that it agrees with the metadata behind it.

## Non-goals

Editing the record. Adding an `informs` edge the record does not declare.
