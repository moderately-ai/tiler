---
id: reconcile-the-three-adr-implementation-statuses-outside-the-metadata-vocabulary
title: Reconcile the three ADR implementation statuses outside the metadata vocabulary
status: done
priority: p3
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, metadata, maturity]
---
## User-visible outcome

Every accepted ADR states an `implementation_status` the metadata contract defines, so a reader comparing maturity across records is comparing values from one vocabulary rather than three.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and reproduced by the coordinator at `5f8eccf6`.

**Fact — the contract defines exactly four values.** `docs/document-metadata.md`, anchor ``` `not-started`, `spike-only`, `partial`, or `implemented` ```.

**Fact — three ADRs are outside it.** `grep -rn '^implementation_status:' docs/ | grep -vE '"(not-started|spike-only|partial|implemented)"'` returns exactly three lines at this base:

| record | value |
| --- | --- |
| `docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md` | `"complete"` |
| `docs/decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md` | `"complete"` |
| `docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md` | `"none"` |

**Fact — nothing validates this, so it is silent indefinitely.** AGENTS.md records that of the documentation properties only `make citations` is checked, and states plainly: "Nothing else is validated — not frontmatter". So these will not surface on any gate.

**Why `"complete"` is the wrong direction rather than merely off-vocabulary.** It reads as *stronger* than the contract's top value `implemented`, on records whose actual maturity has to be derived by reading. A maturity claim that overstates is the failure mode AGENTS.md singles out when it asks that maturity and evidence claims stay distinct — reserved type, architectural seam, implemented support, tested guarantee. `"none"` is the mirror case: it is not `not-started`, and a reader cannot tell whether it means the same thing or something weaker.

**This is pre-existing, not introduced by the repair chain — and that is the interesting part.** `0105`'s value arrived at `5bf4f826`, well before. But `0105` **was edited** by the decision-record repair lane at `e163a0ee` and this was not caught, because the lane was auditing citations and counts rather than frontmatter. Worth stating in the repair so a later sweep knows frontmatter is a separate population from prose.

## Required work

- Re-audit the three Facts at your actual base and report a per-Fact verdict; re-run the grep rather than trusting the table.
- For each record, **derive the correct value by reading what the record decided and what the tree implements**, not by mapping `"complete"` → `implemented` and `"none"` → `not-started` mechanically. The contract defines the field as the highest maturity the record's own decided behaviour has reached, and that is a reading, not a translation. State the evidence for each choice.
- Where a record's true maturity is genuinely `partial`, say which decided behaviour is realized and which is not, following the pattern ADR 0094 received on 2026-08-19 — its correction names decision 7's two realized stages against decision 1's unrealized topology.
- Sweep the whole `docs/decisions/` population for any other frontmatter field carrying an off-vocabulary value, not only this one. Report what you found **and what you found clean**; the negative result is the evidence that this ticket's population was three and not more.
- If you conclude the contract's vocabulary should itself gain a value, **stop and report** rather than adding one — the metadata contract is `contracts/navigation` and is not in this ticket's scopes, and widening a shared vocabulary to fit three records is a decision, not a repair.

## Non-goals

Editing `docs/document-metadata.md` (different scope, and widening the vocabulary is a decision). Changing any ADR's `decision_status`, its decisions, or its prose beyond what a corrected maturity claim requires. Adding a frontmatter validator — that is a separate ticket with its own cost, and AGENTS.md's position that frontmatter is unvalidated is deliberate.

## Findings — worker audit at `04823326`

**Per-Fact verdict: all three verified, none repaired.** The contract anchor resolves in `docs/document-metadata.md` and the four values are as stated. The enumeration grep returns exactly the three tabled records with the tabled values, and the census behind it is `partial` 99, `not-started` 82, `implemented` 29, `spike-only` 26, `complete` 2, `none` 1 over the 239 documents carrying the field. The AGENTS.md anchor `Nothing else is validated` resolves.

**Derived values.** `0105` → `implemented`: every item its decision 3 enumerates is absent from `crates/` and `prototypes/`, the porting requirement holds with `register_index_access` declared once above the file's sole `#[cfg(test)]` boundary and called from exactly ten sites below it — the ten ported registry-mechanics tests decision 3 enumerates, `two_revisions_of_one_provider_resolve_to_an_ambiguity` among them by name — `LoweringFamily::IndexAccess` is still tag `1` spelling `"index-access"`, and decision 4's two reservations are honoured by both types still standing as single-variant. `0109` → `implemented`: decision 4 claims items 1–3 are already implemented and each was re-read rather than taken from that claim — `retain_complete_assessments` for the Disproved-before-Unknown precedence and full retention, `semantic_discharge_is_invalid` for the fail-closed classification, and lowering discharge above the single live `enumerate_covers` call for the before-cover boundary — with two landed tests covering them. `0108` → `not-started`: `AccessMode` still holds `Read` and `Write` alone, `IndexNode` still carries five forms, `InvocationValidationRequired`/`StaticallyProved`/`GatherIndexBounds` do not exist, and the accepted 2026-08-18 surface packet's `0x0C` tag is `reserved-and-unwritten at this base`, which is a reserved type rather than implemented support.

**Wider frontmatter sweep — clean except this field.** Across all 113 records in `docs/decisions/[0-9]*.md`: `schema` 113/113 `tiler-doc/v1`; `kind` 113/113 `decision`; `decision_status` 113/113 in vocabulary (111 accepted, 2 superseded); `catalog_group` 113/113 inside the seven controlled values; `id` 113/113 matching `ADR-NNNN`. The key census shows no unknown field and no `related`, which decisions are not licensed to carry. `implementation_status` was the only off-vocabulary field, and its population was three.

**Two defects found on ADR 0109 outside this ticket's population, reported and deliberately not repaired.** Its `evidence: []` is an empty array, which the contract forbids twice over — arrays present are nonempty with no empty placeholder, and an accepted decision has at least one `evidence` research record. Its `applies_to` names `tiler.research.runtime.execution-contract`, whose target is `kind: "research"`, where `applies_to` admits only a normative contract. The two are almost certainly one defect: ADR 0003, ADR 0051, and ADR 0079 all place that exact id in `evidence`, and 0109's own Traceability names the record as an owner alongside its two contracts. Neither is repaired here because filling `evidence` is a content judgement about which record is this decision's evidence, and mistyped typed edges have their own established lane; `applies_to` also sits outside the typed-edge script's `evidence`/`supports`/`informs` population, so nothing would have caught it.

**One stale-prose residual noted, not repaired.** `docs/research/extensions/backend-provider-composition.md` still carries a seam-inventory row classifying `ScalarLoweringProvider` as "implemented support only" and "yes (as a seam)"; that file is ADR 0105's own `evidence` target and is outside `contracts/decisions`. Three present-tense sentences in ADR 0078 also still describe the scalar family as in the tree, its installation as reachable, and its removal as scheduled.

## Closes when

Every `implementation_status` in `docs/decisions/` is one of the contract's four values, each changed value's evidence is stated, the wider frontmatter sweep is reported with both its findings and its clean result, and `make citations` is green.
