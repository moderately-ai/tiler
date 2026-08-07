---
id: catalog-the-four-cost-model-reading-notes-and-correct-the-stale-token-out-key-count
title: Catalog the four cost-model reading notes and correct the stale token-out key count
status: todo
priority: p2
dependencies: []
related: [close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward, admit-an-indirect-gather-family-for-tied-embedding-lookup]
scopes: [contracts/navigation, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Two unrelated debts, both reported by workers that could not reach them

### 1. Four research documents landed uncatalogued

`close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward` added four `kind: research` notes under `docs/research/cost-model/reading-notes/`. `docs/research/README.md` is `contracts/navigation`, which that ticket did not hold, so **no catalog row exists for any of them** and none is reachable from a documentation entry point.

`AGENTS.md` is explicit that this catalog is manually maintained with **no automated validator**, so an uncatalogued record stays invisible until someone happens to look. Add four rows under the cost-model group.

**Follow the row form the file already uses, not the one a ticket describes.** A previous catalog ticket prescribed a tail that omitted the `primary documents:` clause; the worker noticed the neighbouring `rewrite-search-formalism` row carries it, departed from the prescription, and was right — following the ticket literally would have left a source record unreachable. Read the neighbouring rows.

Each row's fields come from the note's own frontmatter. State what each note **is** — our distillation of a document we may not redistribute — not what the paper says.

### 2. A key count the gather landing falsified

`docs/research/program-planning/model-level-qualification.md`'s **A-token-out** row claims **24 registered keys**. It was 26 before `admit-an-indirect-gather-family-for-tied-embedding-lookup` landed and is **27** now.

The count is the lesser half. The row's load-bearing clause is that **none of the registered keys is a gather** — that was true when written and **this landing makes it false**. `tiler::gather-*` is registered. Correct both, and follow the file's own dated-correction convention rather than importing one.

Note the maturity boundary precisely, because it is easy to overstate: the family is admitted at the **semantic layer only**, deliberately and as a decision rather than a deferral. There is **no fusion role** — `classify` returns `None`, so no region derives legality — no index-layer access class, no `VerifiedKernel`, and nothing device-verified. A row implying the workload's first operation is now compilable would be wrong.

## Verify before writing

Both debts were reported by workers as things they could not reach, so both are **secondhand**. Re-read `docs/research/README.md`'s neighbouring rows, each note's frontmatter, and the A-token-out row at your own base before acting, and report per-Fact verdicts. Line citations in this repository have drifted by **+371 and −171 lines** on a single ticket, so re-locate by anchor rather than trusting any number quoted here.

## Closes when

Four catalog rows resolve from `docs/research/`; the A-token-out row states the true key count and no longer claims no key is a gather; and `make citations` passes.
