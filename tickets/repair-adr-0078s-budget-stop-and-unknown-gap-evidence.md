---
id: repair-adr-0078s-budget-stop-and-unknown-gap-evidence
title: Repair ADR 0078's budget-stop and Unknown-gap evidence
status: todo
priority: p3
dependencies: []
related: [record-the-landed-physical-provider-seam-in-adrs-0078-and-0090]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, documentation, citations]
---
## User-visible outcome

[ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) item 3's exhausted-budget paragraph names symbols that exist, so a reader checking whether the tree still realizes the rule finds the code instead of three absent names.

## Why this exists

`record-the-landed-physical-provider-seam-in-adrs-0078-and-0090` censused ADR 0078's tree-claims on 2026-08-08 and found this one false. It left a dated **Correction pending** note in the record rather than restating the rule in new words, because writing a true replacement requires deriving the current index-domain discharge model and a guess would replace a false Fact with a different false Fact.

**Fact — verified at `750b29e0` by reading, not only by grep.** The paragraph reads "When the exhaustive access proof cannot afford a region, `lowering::refine` records a typed `RefinementBudgetStop` naming the resource, its limit, and the required amount, and the plan stands … `proof_budget_stop` returns the stop only when it is the *sole* diagnostic".

- `grep -rn "RefinementBudgetStop\|proof_budget_stop" crates/` returns nothing.
- `lowering::refine` exists (`grep -n "^fn refine(" crates/tiler-compiler/src/lowering.rs`), but `LoweringError`'s four variants are `Occurrence`, `Resolve`, `Refine`, and `SemanticDischarge` — read the enum, none is a budget stop.
- `legality::IndexRefinementOutcome` is `Refined` or `Pending`, with no budget arm.
- A budget-stop vocabulary survives elsewhere: `RegionBudgetStop` and `ExplainEvent::BudgetStop` in `crates/tiler-compiler/src/region.rs`, and `IndexDomainDischargeRefusalKind::Unknown` reached through `LoweringError::SemanticDischarge`.

## What this must establish before editing

1. Which authority now carries the obligation the paragraph states: an exhausted analysis budget is an `Unknown` gap, neither a rejection nor an admission.
2. Whether the sole-diagnostic guarantee still holds — that a budget stop can never hide a real refusal behind an exhausted analysis — and which code enforces it. If nothing does, that is a defect and belongs in its own ticket rather than in a prose repair.
3. Whether `IndexDomainDischargeRefusalKind::Unknown` and a region-formation budget stop are the same finding or two, since item 3's neighbouring paragraphs turn on keeping distinct findings distinct.

## Closes when

The paragraph cites symbols that resolve, by searchable anchor and not by line number; the dated **Correction pending** note it replaces is itself retired with a dated note rather than deleted; `make citations` passes; and the rule item 3 decides is unchanged.

## Graph maintenance

- This repairs evidence, not a decision. If the derivation shows the tree no longer realizes the rule, stop and file that as a defect — a record must not be edited to match a tree that drifted away from an accepted decision.
- The record's other 2026-08-08 corrections are independent of this one and are already landed.
