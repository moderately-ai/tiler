---
id: decide-which-sentence-governs-the-informs-requirement-for-unadopted-research
title: Decide which sentence governs the informs requirement for unadopted research
status: done
priority: p3
dependencies: []
related: [repair-the-four-mistyped-typed-frontmatter-edges]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, metadata, schema]
---
## User-visible outcome

`docs/document-metadata.md` states one rule for whether a research record must carry `informs`, instead of a required-field table and a prose sentence that disagree about unadopted records.

## The tension, surfaced by the edge repair

**Fact** (from [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md)'s outcome). The metadata contract's required-field table types `informs` as required on *every* research record, while the sentence below it binds only *adopted or partially adopted* research. Dropping the open-ticket audit's inadmissible portal edge left that record with no `informs` at all — admissible under the prose sentence, a violation under the table. The corpus now sits in the gap twice: `enforcer-input-property-exclusion` and `open-ticket-audit-2026-07-27`, both `pending`-disposition research with no admissible contract to inform.

**Why it is a decision rather than a fix.** Requiring `informs` on pending research forces either a premature contract edge (the drift the typed-edge check exists to catch) or a dummy target; binding only adopted research means a pending record's connection to the corpus rests on catalogs and body links alone. Both are coherent; the contract currently states both.

## What this must produce

One governing rule, stated once in the contract with the other site rewritten to agree, and the two in-gap records either left conforming (if the adopted-only reading wins) or given admissible edges (if the always-required reading wins). The typed-edge reproducing script in the repair ticket is the validator; re-run it and state the counts. A schema change to a contracts document is a contract edit — if the resolution is consequential beyond these two records, draft and park for Tom rather than self-deciding.

## Closes when

The contract states one rule, the two in-gap records conform to it, and the typed-edge check reports zero mistyped edges at the stated population.

## Outcome — 2026-08-06

**The prose sentence governs: `informs` is required of adopted and partially adopted research and optional on every other disposition.** Delivered on `tkt/decide-which-sentence-governs-the-informs-requirement-for-unadopted-research` as the single commit carrying this text, base `453aef62`. The two in-gap records were not edited; they conform as written, and the contract now says so. Only [`docs/document-metadata.md`](../docs/document-metadata.md) changed, plus this ticket.

**One correction to the filing above.** The two in-gap records do not share a disposition: `open-ticket-audit-2026-07-27` is `pending`, `enforcer-input-property-exclusion` is `informational`. The tension is the same for both, because the winning sentence binds `adopted` and `partially-adopted` alone, but the gap is not a property of `pending`.

### Why the universal requirement loses, and it is not a close call

**Fact — the always-required reading is unsatisfiable for a record that exists, by construction rather than by author laziness.** `informs` admits only a `contract` or `decision` target. What [`open-ticket-audit-2026-07-27`](../docs/research/documentation/open-ticket-audit-2026-07-27.md) informs is the ticket board, `ticketsplease.toml`, and the work-tracking process — and `docs/work-tracking.md` is `kind: "portal"` (`tiler.portal.work-tracking`), while `tickets/**`, `ticketsplease.toml`, and `AGENTS.md` carry no `tiler-doc/v1` frontmatter at all. There is no admissible target in any spelling, and [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md) had already rejected the two nearest governed candidates on the contract's own text: `tiler.contract.document-metadata` on its *Ownership* section, which disclaims ticketsplease's ticket schema, and ADR 0075 because the audit applies its categories rather than supplying evidence for them. So the always-required reading cannot be executed to conformance — its only completions are a false edge, re-kinding a record that genuinely is research, or leaving a permanent known violation, which is the status quo this ticket exists to end. A rule with a member that cannot satisfy it admissibly is not a candidate.

**Fact — the drift the universal reading causes is measured, not hypothesised, and it is the reason this ticket exists.** The mistyped edge the repair found was `informs: ["tiler.portal.status"]` on that same audit: an author obeying the required-field table with no admissible target to name, inventing one. The contract's own paragraph records that no reading caught it and the typed-edge check did. The universal requirement did not produce an edge there; it produced *that* edge.

**Inference — what each rule makes the typed-edge check mean.** The check resolves stored edges against target kinds and never checks presence, so neither rule changes what it reports for these two records — both contribute zero `informs` edges either way. The difference is upstream of the check: the always-required reading converts "this record has no normative target" into pressure to store an edge, and the check is then the only thing standing between that pressure and a false graph. The adopted-only reading removes the pressure, so the check's remaining population is edges an author chose to assert.

**Measurement — the corpus at `453aef62`, which is the strongest argument for the loser and is stated here rather than buried.** Of 101 research records, 99 carry `informs`, including 33 of the 34 `pending` ones and all 63 that are `adopted` or `partially-adopted`; `docs/research/README.md` renders 99 matching `informs:` clauses. So the always-required table describes near-universal practice, and the counterpoint to the survivor is exactly that: making the field optional for the 38 unadopted records stops obliging a future unadopted record to name a target it actually has, and reading is the only thing that will notice one that does not. Three things bound that cost. Naming a target stays the norm, and the precedent is 33 of 34. The contract now states the omission as a *claim* — that the author looked and found none — rather than as a blank, so a missing edge is answerable at review. And corpus membership never rested on `informs` anyway: all 101 records are reached by a research-catalog row, and the reconciliation check in [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) counts rows against records.

**Inference — the "premature contract edge" worry named in the filing is the weakest argument on the winning side, and it is not what decided this.** The contract states that `evidence`, `informs`, and `adopted_by` are independent predicates, so `informs` on a `pending` record asserts only that the research bears on that contract, not that the contract adopted it. That is not premature, and 33 pending records demonstrate it is routinely available. What decides the ticket is the record with no reachable target, not adoption timing.

### What changed in the contract

The winning sentence — "Adopted or partially adopted research has an `informs` or `adopted_by` destination" — is byte-identical and remains the single normative statement; the losing site moved to agree with it. In the kind-specific required-field table, research's `informs` moved from *Required beyond common* to *Optional typed fields*, which also keeps the field licensed under the sentence declaring that column "its exhaustive licence". Two paragraphs follow the invariant list: one stating that a disposition-conditional requirement cannot live in a table of unconditionally required fields, that an omitted `informs` is a claim rather than a blank, and the portal-edge derivation above; one recording the measurement, the licence's narrowness, and the cost. No sentence was added restating the rule.

**Fact — no third site and no ADR amendment.** `informs` appears nowhere else as a presence requirement: `docs/README.md`, `docs/status.md`, `docs/design-map.md`, `docs/work-tracking.md`, `docs/decisions/README.md`, the root `README.md`, and `AGENTS.md` do not mention the field, and no ADR does. [ADR 0054](../docs/decisions/0054-use-typed-documentation-metadata.md), the accepted decision applying to this contract, decides that relationships are typed and singly stored and delegates per-kind field detail to the contract; it names no field, so it needs no amendment. The contract's *Typed relationships* section types `informs` as research to contract and is untouched — this ticket changed when the field is required, never what it means or what it may point at.

### Self-decided rather than parked, against this ticket's escalation clause

The rule picks which of two already-written sentences governs. Every one of the 101 research records that conformed before conforms after: the 99 with `informs` keep an edge that is still licensed, still typed the same way, and still rendered identically in the catalog, and the 2 without one stop being in violation of one of two contradictory sites. Nothing becomes invalid, no record is edited, no catalog row moves, and `informs` means on every record exactly what it meant. The reader who notices is the reader of a record with no `informs` — a population of two, both of which the contract now names. That is the delegated case rather than the escalation case. The judgement that the sentence is worth stating with its cost attached, rather than as a bare rule, is recorded above so a reviewer can attack the derivation instead of the verdict.

### Checks

- Typed-edge check from [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md), run from the repository root at `453aef62` and again after the edit: `population: 279 governed documents with an id and a kind`, `evaluated: 479 typed edges`, `MISTYPED: 0`, identical before and after — this change stores and removes no edge.
- The check was watched failing before being trusted: temporarily repointing this contract's own first `evidence` entry at `tiler.portal.work-tracking` reported `EVIDENCE docs/document-metadata.md: -> tiler.portal.work-tracking is portal, not research` and `MISTYPED: 1`. Reverted; `git diff --stat` afterwards showed only the intended contract edit.
- `tkt lint` clean, `git diff --check` clean, `tkt guard` reports only the declared scopes. `docs/document-metadata.md` maps to `contracts/navigation`, this ticket's exclusive scope, verified in `ticketsplease.toml`; no scope was added. The diff is `docs/` and `tickets/` only, so no Cargo gate applies.
