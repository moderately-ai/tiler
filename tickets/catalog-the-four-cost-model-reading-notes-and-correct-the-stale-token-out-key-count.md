---
id: catalog-the-four-cost-model-reading-notes-and-correct-the-stale-token-out-key-count
title: Catalog the four cost-model reading notes and correct the stale token-out key count
status: done
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

## Outcome — done, 2026-08-07

Landed at merge **`72001fcc`** (worker commit `8f21d461`). Two files, `docs/` only, so it carries the green gate; `tkt lint` and `make citations` both clean on the merged tree.

Four catalog rows added in *Physical planning and lowering*, each following the file's **actual** row form including the `primary documents:` clause — and each pointing at that document's **own anchor** in the source record rather than at the record as a whole. Link text reads "the non-redistributable *X* source record", deliberately **not** "metadata-only": these four are exactly the `local-only` rows, and the source record warns that the preservation class and the redistribution verdict must not be conflated. Non-redistributable is the verdict and is true of all fourteen.

The A-token-out row now reports **27** keys (coordinator-verified independently), names `tiler::gather-f32@1`, and states the maturity boundary without overstating it: the reference evaluator is the *only* enforcement boundary, and **the row's own outcome remains unreachable** — a qualification run compiles no gather and observes the anterior `StorageScalarMismatch` refusal instead. The verdict moved from **No** to **reference evaluator yes, physical path no**.

### The 24 was never wrong — it went stale, which is the more useful reading

`git grep` at the commit the row cites returns exactly **24**; 26 at the gather's parent; 27 now. So this was not an error being corrected but a dated measurement outliving its date, which is why the repair records *which rows were rechecked at which commit* rather than silently refreshing numbers.

### Scope the worker added, flagged rather than absorbed

Re-reading the whole nine-row table — rather than grepping the two cells named — found the **sibling `StorageScalar` row** also stale: cited as "exactly two variants, `U8` and `F32`" at `model.rs:264`; it is at **342** with **three**, `Bf16` having been appended as a carrier. Coordinator-verified. The verdict (**No**) survives because none of the three is an 18-bit integer carrier, but the stated ground was false. Two further cells had drifted line numbers with verdicts intact; those citations became greps.

### Root cause filed, because repairing four rows leaves it in place

`docs/research/README.md:18` carries `<!-- BEGIN GENERATED RESEARCH CATALOG -->` and **no generator exists anywhere in the tree** — coordinator-verified across `.py`, `.sh`, `.rs` and `Makefile`. A reader who sees `GENERATED` reasonably assumes a tool keeps it in sync, which is why four records could land uncatalogued unnoticed. Filed as [`make-the-research-catalog-generated-or-stop-claiming-it-is`](make-the-research-catalog-generated-or-stop-claiming-it-is.md).
