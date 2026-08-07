---
id: catalog-the-measured-feedback-tuning-loop-research-record
title: Catalog the measured-feedback tuning-loop research record
status: todo
priority: p2
dependencies: []
related: [design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, cost-model]
---
## User-visible outcome

[The measured-feedback tuning loop](../docs/research/cost-model/measured-feedback-tuning-loop.md) is reachable from the [research catalog](../docs/research/README.md) instead of only by knowing its path.

## Why this is a separate ticket

**Fact.** `docs/research/README.md` is owned by the `contracts/navigation` scope in `ticketsplease.toml`; the ticket that authored the record declared `research/cost-model` and `project/tickets`. The record therefore landed uncatalogued, and this is a scope boundary rather than an oversight — the authoring worker could not have added the row without taking a scope it was not dispatched with, and `contracts/navigation` covers eleven other documents that other lanes edit.

**Fact.** The [documentation metadata contract](../docs/document-metadata.md) states the consequence plainly: the catalogs "were generated views over frontmatter, so they no longer update themselves when a record is added, retitled, or superseded. Edit the affected catalog entry in the same change that edits the metadata behind it; nothing will tell you later that you did not." There is no validator and no CI, so nothing will surface this.

## Scope

Add one row to the **Physical planning and lowering** group of the generated-catalog block in `docs/research/README.md`, matching the record's own frontmatter and the surrounding rows' format. The record carries `catalog_group: "physical-planning-lowering"`, `disposition: pending`, `evidence_classes: ["primary-source-synthesis"]`, and `informs: ["tiler.contract.cost-model", "tiler.contract.optimizer"]`, so the row's tail reads `— pending; primary-source-synthesis; informs: <Cost model>, <Optimizer model>`, with each of those two names a link and the row led by a link to the record itself. **Those link targets are written relative to `docs/research/`, not to this ticket**, which is why they are not spelled as live links here — copy the exact form the neighbouring rows in that group already use rather than transcribing a path out of this file. Place the row in the group's existing alphabetical-by-title position.

**Read the record's frontmatter before writing the row** rather than trusting this ticket: it is a claim about a file this ticket does not own, and the record may have been revised.

## Non-goals

The sources record under `docs/research/cost-model/sources/` gets no catalog row of its own. Neither does the region-search or numerics sources record, and adding one here would make this the only source record in the catalog for no reason a reader could infer.
