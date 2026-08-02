---
id: land-the-model-level-qualification-record
title: Land the model-level qualification record at its destination
status: done
priority: p2
dependencies: [design-model-level-qualification-and-optimization]
related: [define-first-metal-lm-workload, design-model-ingestion-and-complete-execution]
scopes: [research/program-planning, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, carrier, research, language-model, navigation]
---
## User-visible outcome

The L8 qualification design lives where a reader looking for it will go — beside the workload profile and the ingestion record it derives from — rather than inside the ticket that produced it, and the research catalog names it.

## Why this exists

**Fact.** [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) declares `research/cost-model`, `research/apple-targets`, `contracts/navigation`, and the shared `project/tickets`. It does not declare `research/program-planning`, which is the scope covering `docs/research/program-planning/**`. That ticket's own stop-condition section holds the four-candidate elimination that placed the record in that directory and rejected both held research scopes and the split across them; it is not restated here.

**Fact.** Rather than self-granting the scope, that ticket drafted the destination file's complete body inside itself, under the heading *Drafted record body — verbatim-landable at `docs/research/program-planning/model-level-qualification.md`*. This ticket is the carrier the corpus's own discipline calls for, and the transfer is byte-identical.

## Required work

- Create `docs/research/program-planning/model-level-qualification.md` from the drafted span, applying exactly the two mechanical transformations the span names and nothing else: the fenced YAML block becomes the file's delimited frontmatter with the `---` delimiters restored, and every `###` heading in the span is promoted to `##`.
- **Do not edit inside the span.** A transfer that corrects a sentence, repoints a link, or reflows a table is a fork rather than a transfer. If something in the span is wrong, land it as drafted and file the correction against the landed file, so that the record and its source stay comparable.
- Confirm the span's relative links resolve from the destination. They were written to resolve from `docs/research/program-planning/` and deliberately do not resolve from `tickets/`; the destination is where that condition is discharged, so every link is checked there rather than assumed.
- Add the record's row to the generated research catalog block in [`docs/research/README.md`](../docs/research/README.md), under **Physical planning and lowering**, in the block's existing alphabetical-by-title order and matching the shape of its neighbours: title link, disposition, evidence classes, `informs` targets, and — since this record cites a retained fixture — its experiment link. The catalog is hand-maintained prose with no gate, so it is edited in the same change as the file it describes.
- Record in the drafted-body section of the source ticket that the transfer happened, the date, and the destination path, so that the span is thereafter provenance rather than a second authority over the same subject.

## Explicit non-goals

No new analysis, no measurement, no ticket refiling, and no edit to the roadmap ladder row — the source ticket already updated it and named this carrier as where the record's link becomes live.

## Closes when

`docs/research/program-planning/model-level-qualification.md` exists with the drafted body's exact content under the two named transformations, every relative link in it resolves from its own directory, the research catalog carries its row, and the source ticket's span is marked as transferred with its destination.
