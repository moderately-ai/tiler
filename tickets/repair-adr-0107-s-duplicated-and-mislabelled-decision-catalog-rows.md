---
id: repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows
title: Repair ADR 0107's duplicated and mislabelled decision catalog rows
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## ADR 0107 appears twice, wrong once, and is missing from the chronology

Four defects in `docs/decisions/README.md`, all about one record, from the 2026-08-07 read-only audit:

1. **A row under "Foundation, semantics, and extensions" carries ADR 0077's metadata** — `contracts: [System architecture]; evidence: [Prototype crate layout and Rust MSRV]` — copied from the adjacent row. ADR 0107 declares `applies_to: ["tiler.contract.ir"]` and `catalog_group: "physical-planning-lowering"`, so it is in the wrong group as well as carrying the wrong facts.
2. **A second row for 0107 exists** further down with correct metadata. Two contradictory rows for one record.
3. **Both read `— proposed`.** ADR 0107's frontmatter is `decision_status: "accepted"` — Tom accepted it on 2026-08-07 and the record carries the acceptance provenance.
4. **The chronology omits it entirely**: 107 ADR files on disk, 106 rows, the block ending at `0106`.

## Why this was not caught, and what it means for the fix

The block was emitted by `scripts/docs.py`, deleted at `e197176f`. Its `BEGIN GENERATED` markers were removed on 2026-08-07 and the catalogs restated as **hand-maintained**, which is now the true and recorded obligation — so these rows must be repaired by hand and will stay correct only by reading.

**Verify each of the four at your base before editing**, including the file counts, and report per-defect. The audit that found them is a claim; two of its sibling findings needed correcting on nuances the auditor missed.

## Scope note

Adding the missing chronology entry means the block's population changes. **Report the count before and after** so a reader can tell a repair from a drift — this repository has repeatedly found counts that were correct when written and went stale, and the distinction between "was wrong" and "went stale" is worth stating.

## Closes when

ADR 0107 has exactly one row, in its declared `catalog_group`, carrying its own `applies_to` and evidence, reading `accepted`; the chronology's population equals the ADR file count; and the counts are stated before and after.

## Outcome

All four defects verified at base `209013bdecda631f37d1aff7f2c575f8c19cca3e` before editing, by resolving every catalog row against the frontmatter of the ADR file behind it. Three verified as stated; defect 1 is true in substance with one imprecision.

**Populations.** `ls docs/decisions/[0-9]*.md | wc -l` = **107** files. Topic rows **108 before → 107 after** (the duplicate removed). Chronology rows **106 before → 107 after** (the missing entry added). Both blocks now equal the file count. The chronology's 106 was a **stale** count — correct until 0107 landed uncatalogued — while the topics' 108 was **wrong when written**, because the duplicate row was never a correct state.

**Defect 1 — verified, one imprecision.** The row at `docs/decisions/README.md:24` carried `contracts: [System architecture]; evidence: [Prototype crate layout and Rust MSRV]` under "Foundation, semantics, and extensions", against ADR 0107's `applies_to: ["tiler.contract.ir"]` and `catalog_group: "physical-planning-lowering"` — wrong group and wrong facts, as claimed. The imprecision: it is a **truncated** copy of the adjacent 0077 row, not a whole one. Contracts match 0077 exactly; 0077's evidence carries two records (`prototype-crate-layout-and-msrv`, `artifact-compatibility`) and the bogus row carried only the first.

**Defect 2 — verified.** A second 0107 row at `:113`, with `contracts: [IR stack and invariants]; evidence: [Transformer operation and shape surface derivation]`, matching the frontmatter's `applies_to` and `evidence` exactly. It is the row worth keeping.

**Defect 3 — verified.** Both rows read `— proposed`; frontmatter is `decision_status: "accepted"`, and the record's body carries "accepted by Tom on 2026-08-07, in the interactive orchestration session … Not relayed."

**Defect 4 — verified.** The chronology ran 0001–0106 with no 0107 row.

**A fifth defect, same class, found by reading the whole catalog.** Each topic group is ordered alphabetically by title — a rule that holds for all 106 non-0107 rows and is violated only by the two 0107 rows. The surviving row at `:113` sat after "Separate logical tensor access from storage addressing", not between 0097 "Admit a two-dimensional…" and 0100 "Admit the multi-round…". Repaired in the same edit, since the row was being rewritten regardless. Nothing else in the catalog is defective: a full resolution of all 107 rows against frontmatter found no other duplicate, missing, extra, mis-statused, misgrouped, mistitled, or mislinked row in either block.

**Coverage of the marker blocks.** Four `BEGIN`/`END` pairs exist across three files — `docs/decisions/README.md` (`ADR TOPICS`, `ADR CHRONOLOGY`), `docs/research/README.md` (`RESEARCH CATALOG`), `spikes/README.md` (`EXPERIMENT CATALOG`). All four have already had `GENERATED` removed; no `BEGIN GENERATED` remains in any document. Only the two in this file are in scope here.

**`make citations` does not cover these rows — established by deliberate break, not assumed.** Both links in the new row were pointed at non-existent files; `make citations` stayed green with byte-identical output, because catalog rows are plain markdown links and the script counts them among its `3246 bare path mention(s) carrying no line or anchor`. The 582 local links in this file were resolved by a separate one-off check, which reported the two dangling links and exited 1; the break was then reverted. The standing check on these rows is reading, exactly as [`../docs/document-metadata.md`](../docs/document-metadata.md) records.

**Commands.** `tkt lint` → `ok: no problems found`. `make citations` → `956 pinned citation(s) resolved across 493 live … file(s)`. `git diff --check` → clean. `tkt guard --base 209013bd tkt/repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows` → `verdict: ok`.

**Gate carry.** The delta touches `docs/decisions/README.md` and this ticket only — nothing under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` — so it carries the latest green gate under the AGENTS.md delta rule, with `tkt lint` and `make citations` rerun.

## Outcome — done, 2026-08-08

Landed at merge `cb56bf8e` (worker commit `23b09e62`). `docs/` + `tickets/` only, carries the green gate.

**Populations, both now equal to the 107 ADR files:** topic rows **108 → 107**, chronology rows **106 → 107**.

**The two counts failed differently, and that is the distinction worth keeping.** The chronology's 106 was **stale** — correct until 0107 landed uncatalogued. The topics' 108 was **wrong when written**, because a duplicate row was never a correct state. This repository has repeatedly confused the two, and a sibling verification pass refuted an entire audit finding this week on exactly that split.

All four named defects verified, with one imprecision: the bogus row was a **truncated** copy of the adjacent 0077 row rather than a whole one — contracts matched exactly, but 0077's evidence carries two records and the copy carried one. Does not change the fix.

**A fifth defect of the same class, found by reading the whole catalog rather than the four named rows.** Each topic group is ordered **alphabetically by title** — a rule holding for all 106 non-0107 rows and violated only by these. Repaired in the same edit. The worker then resolved **all 107 rows** against their ADRs' frontmatter and found no other duplicate, missing, extra, mis-statused, misgrouped, mistitled or mislinked row.

### It proved the gate is blind here rather than recording a green run as evidence

Asked to break its new anchors and watch the check fail, the worker instead discovered **`make citations` does not validate catalog rows at all** — they are plain markdown links, counted among the script's `bare path mention(s) carrying no line or anchor`. It pointed both links at non-existent files and got **byte-identical output**.

**Coordinator-confirmed independently**: planting `](9999-no-such-adr-…)` leaves the check at exit 0 with an unchanged count. Filed as `resolve-the-markdown-links-the-citation-check-cannot-see` (p1) — the population is **3,255 references**, including the entry points `AGENTS.md` directs every reader to.

The worker resolved all **582 local links** with a one-off check that reported both dangling ones and exited 1, so the property is cheaply checkable; what is missing is that it runs. Reporting a blind gate as blind, rather than banking the green, is what made that finding possible.
