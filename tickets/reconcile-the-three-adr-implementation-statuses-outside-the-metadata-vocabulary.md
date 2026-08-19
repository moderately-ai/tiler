---
id: reconcile-the-three-adr-implementation-statuses-outside-the-metadata-vocabulary
title: Reconcile the three ADR implementation statuses outside the metadata vocabulary
status: in-progress
priority: p3
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, metadata, maturity]
claimed_from: todo
assignee: worker-adrstatus
lease_expires_at: 1787169555
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

## Closes when

Every `implementation_status` in `docs/decisions/` is one of the contract's four values, each changed value's evidence is stated, the wider frontmatter sweep is reported with both its findings and its clean result, and `make citations` is green.
