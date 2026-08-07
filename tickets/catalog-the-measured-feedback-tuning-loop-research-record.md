---
id: catalog-the-measured-feedback-tuning-loop-research-record
title: Catalog the measured-feedback tuning-loop research record
status: done
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

**Corrected 2026-08-07 — the prescribed tail above was incomplete, and the reason is that it was derived from frontmatter while the catalog carries a clause frontmatter does not store.** The row as landed reads `— pending; primary-source-synthesis; primary documents: <preserved autotuning and adaptive-execution sources>; informs: <Cost model>, <Optimizer model>`. The `primary documents:` clause sits between the evidence classes and `informs:`, and no frontmatter field produces it: `docs/research/region-search/rewrite-search-formalism.md` carries no such key, yet its catalog row names its dedicated source record that way, as do both rows that share the Apple Metal source record. The convention the surrounding rows establish is therefore that a research record with a source record supporting *it* names that record in its own row, which is also what makes the source record reachable from an entry point at all — the outcome this ticket exists to produce. The instruction to "copy the exact form the neighbouring rows in that group already use" was the correct half of the Scope section and it is what the landed row follows.

## Non-goals

The sources record under `docs/research/cost-model/sources/` gets no catalog row of its own. Neither does the region-search or numerics sources record, and adding one here would make this the only source record in the catalog for no reason a reader could infer.

**Clarified 2026-08-07 — "no row of its own" is not "no link", and the sentence above reads as though it were.** Source records already appear in the catalog three times, as the `primary documents:` clause of the row belonging to the record they support: `region-search/sources/README.md` on the rewrite-search formalism row, and `apple-targets/sources/README.md` on both the compile-profile authority ledger and the Apple Metal artifact compatibility rows. The cost-model source record is named the same way on this record's row, per the correction in Scope. The numerics source record is the one with no link, and its situation is not this one: it supports at least four records across two catalog groups, so no single row owns it. Whether it should be reachable is a separate question this ticket does not answer.

## Outcome — done, 2026-08-07

Landed at merge **`dda57de5`** (worker commit `a42686bc`). One row in `docs/research/README.md`, placed in **Physical planning and lowering** at the group's alphabetical position.

**The worker departed from this ticket's prescribed row, deliberately and correctly.** The ticket specified a tail of `— pending; primary-source-synthesis; informs: …`, omitting the `primary documents:` clause that sits between the evidence classes and `informs:`. No frontmatter field produces that clause, but the neighbouring `rewrite-search-formalism` row carries it for its own dedicated source record — **verified by the coordinator in the merged diff**. Following the ticket literally would have left `docs/research/cost-model/sources/README.md` unreachable from any entry point, which is the very defect this ticket exists to fix.

Every other field is read from the record's own frontmatter, and the row claims only that the record exists and is a pending primary-source synthesis — **not** that Tiler has a measured-feedback loop, which would be false.

All three of this ticket's stated Facts held on inspection — the first ticket assessed today with no false Fact in it.

**One pre-existing defect left alone, deliberately:** `The multi-round two-level reduction composition` sits out of alphabetical order, but immediately before the record it composes, and the same topical clustering appears elsewhere in the catalog. Reordering on an inferred convention, in a file other lanes edit, would be a guess. Worth a narrow ticket only if the ordering rule is meant to be strict.
