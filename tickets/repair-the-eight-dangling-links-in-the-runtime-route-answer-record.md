---
id: repair-the-eight-dangling-links-in-the-runtime-route-answer-record
title: Repair the eight dangling links in the runtime route answer record
status: done
priority: p1
dependencies: []
related: [correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling, correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The premise below is false: these are not wrong relative paths

*Audited and rewritten 2026-08-08 at `db3f4d07`. The original text is retained below because five tickets were filed on it and because the per-Fact verdicts are only readable against it.*

**All eight links sit inside a retained, byte-identical drafted ADR span, and their paths are wrong for this file on purpose.** They are the whole of the span's `### Traceability` paragraph, byte-identical to `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md`'s own Traceability paragraph — verified with `cmp` on the two extracted lines, not by eye. The span is text drafted for `docs/decisions/`, transferred there byte-identically, and kept here as provenance for that transfer. No link outside the span fails: the file's only failures are those eight.

**Repointing them is out of scope, not merely inadvisable.** Two accepted ADRs assert the surviving byte-identity in the present tense: `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md "editing inside forks the byte-identical transfer"` and `docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md "the byte-identity the landing established still holds after acceptance"`. Both live in `contracts/decisions`, which this ticket does not hold, so repointing from here would leave an accepted ADR asserting something the tree no longer satisfies — in a scope this branch cannot correct.

**The repair is a fence.** `check-citations.sh "fenced block is content proposed for somewhere else"` is the corpus's declared spelling for content whose links belong to another file, and [`repair-the-self-referential-link-in-the-concatenate-fusion-record`](repair-the-self-referential-link-in-the-concatenate-fusion-record.md) reached the same answer independently for the same construct. The cross-record choice is owned by `decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span`; this ticket applies it to the runtime record only.

**Measured cost of the fence, counted rather than estimated.** The fenced region holds **0** links that resolved and **0** pinned citations, so the fence removes nothing from the matcher except the eight failures. `docs` citations 702 → 706 (the four added are this correction's own anchors; none were lost), `docs` links 5,211 → 5,204 (eight out, one added back), tree-wide unresolved links 14 → 6. The cost that is real is rendering: the span's headings, bold, and numbered items now read as literal text.

### Per-Fact verdicts on the original text

| Original Fact | Verdict | Evidence |
|---|---|---|
| Eight markdown links resolve to nothing | **Verified** | 8 `FAIL` lines for this file at base; 14 tree-wide |
| The five named ADR filenames and the three named `..` links | **Verified** | each appears verbatim in the checker output; all eight targets exist and are tracked |
| "Two distinct mistakes, both mechanical" | **False** | one deliberate construct, not two mistakes — all eight are a single transferred paragraph whose paths are correct for `docs/decisions/` |
| "each exists at `docs/decisions/<same-name>`, verified 2026-08-08" | **Verified but not decisive** | target existence is true, and does not establish that the link should point there *from here* |
| "Repair the link, not the check … the fix is the relative path" | **False** | the fix is a fence; the path is wrong for this file by design and correcting it needs `contracts/decisions` |
| "Every one of these is a reader sent nowhere from a live research record" | **Imprecise** | true of the rendered page, but the prose beside the span already told the reader the span's paths resolve from the ADR; the fence makes that visible rather than implied |

### Defects found while auditing, each needing its own ticket

1. **ADR 0092 carries a false sentence.** Its item-8 correction note says that "the drafted body in the source record deliberately still carries the pre-rename spelling" — the quotation's link markup around "the source record" is dropped here so the citation checker reads this as prose about a link rather than as one. It does not: the span's item 8 reads `RouteResourceRequirement`, the post-rename spelling, because the 2026-08-06 re-transfer superseded that sentence. Needs `contracts/decisions`.
2. **A citation that resolves while supporting a claim the file does not make.** This record asserts twice, outside the span, that `docs/architecture.md:389` states "`tiler` is the one crate a consumer names." The real sentence is `docs/architecture.md "is the one crate an inline-frontend consumer names"`, and the dropped qualifier is load-bearing — the same paragraph denies that `tiler` is the accepted facade for consumers compiling arbitrary semantic programs, and ADR 0092 decision item 6 rests on that sentence. The line pin resolves, so no checker can catch it. Needs `research/runtime`, but is beyond what a link repair forces.

## Closes when

**Not closable by repointing.** Closes when the span is fenced, `make citations` reports no link failure in this file, and the false `AGENTS.md` attribution inside the record is repaired to name this record as the convention's author. *Superseded original:* "`make citations` reports no link failure in this file. Repair the link, not the check: each intended target already exists, so the fix is the relative path."

## Original text, retained

### What is broken

`docs/research/runtime/backend-scoped-route-requirement-answers.md` carries **eight** markdown links that resolve to nothing. Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`; this file is the single largest concentration of the fourteen defects that run found.

Two distinct mistakes, both mechanical:

**Five ADR links written as bare filenames**, as if the record lived in `docs/decisions/`. Each names a real accepted ADR, so the target exists and only the prefix is missing:

- `](0090-compose-backends-per-responsibility-rather-than-per-backend.md)`
- `](0086-require-attributable-or-attested-native-translation.md)`
- `](0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md)`
- `](0074-use-explicit-public-api-conventions.md)`
- `](0075-scope-public-boundary-approval-by-change-category.md)`

Each resolves against `docs/research/runtime/` and finds nothing; each exists at `docs/decisions/<same-name>`, verified 2026-08-08.

**Three links with the wrong number of `..` segments:**

- `](../research/runtime/backend-scoped-route-requirement-answers.md)` — a self-link written from `docs/`, resolving to `docs/research/research/runtime/...`
- `](../architecture.md)` — resolves to `docs/research/architecture.md`; the file is `docs/architecture.md`
- `](../artifact-abi.md)` — resolves to `docs/research/artifact-abi.md`; the file is `docs/artifact-abi.md`

### Why it matters

Every one of these is a reader sent nowhere from a live research record, and five of them are the ADRs that record's conclusions rest on. The record is not superseded, so it is exactly the population `AGENTS.md` says a reader follows into the tree.

### Closes when (superseded — see above)

`make citations` reports no link failure in this file. Repair the link, not the check: each intended target already exists, so the fix is the relative path.

## Verify

```sh
./check-citations.sh 2>&1 | grep -A2 backend-scoped-route-requirement-answers
```

## Outcome and completed findings — 2026-08-09

Commit `91f67cc5` fenced the byte-identical drafted ADR span, preserved its
transferred bytes, repaired the record's false AGENTS.md attribution, and made
the link check green without repointing content authored for another directory.
Commit `c8c3da05` closed this ticket.

Both defects found during the full-file audit were separately owned and have
landed:
[`correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling`](correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling.md)
and
[`correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier`](correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier.md).
No dangling-link or neighbouring-record remainder reported here is still
unfiled.
