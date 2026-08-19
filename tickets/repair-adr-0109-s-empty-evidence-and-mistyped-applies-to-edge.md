---
id: repair-adr-0109-s-empty-evidence-and-mistyped-applies-to-edge
title: Repair ADR 0109's empty evidence and mistyped applies-to edge
status: todo
priority: p3
dependencies: []
related: [reconcile-the-three-adr-implementation-statuses-outside-the-metadata-vocabulary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, metadata]
---
## User-visible outcome

ADR 0109 carries its evidence edge where the contract puts it, and its `applies_to` names only normative contracts — so a reader following its typed edges reaches the record that justifies it rather than an empty array.

## Why this exists

Filed 2026-08-19 from the ADR-status reconciliation lane, which found these while sweeping and correctly reported rather than repaired them. Verified first-hand by the coordinator at `8d2619e5`.

**Fact — `evidence` is an empty array.** `docs/decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md` frontmatter reads `evidence: []`. The metadata contract forbids this twice over: an array that is present is nonempty with no empty placeholder, and an accepted decision carries at least one `evidence` research record.

**Fact — `applies_to` names a research record.** The same frontmatter reads `applies_to: ["tiler.contract.operation-extensions", "tiler.contract.optimizer", "tiler.research.runtime.execution-contract"]`. The third is a research record — `docs/research/runtime/runtime-execution-contract.md` — where `applies_to` admits only a normative contract.

**Inference — these are one defect, not two.** ADR 0003, ADR 0051, and ADR 0079 all place that exact id in `evidence`. ADR 0109 alone put it in `applies_to` and left `evidence` empty, which is the shape of a single edge written into the wrong field.

**Fact — no existing check could have caught it, and that is worth recording.** `applies_to` sits **outside** the typed-edge script's `evidence`/`supports`/`informs` population, so a "0 mistyped edges" measurement was never evidence about this field. AGENTS.md separately records that frontmatter is unvalidated. So this stood unnoticed and would have continued to.

## Required work

- Re-audit all four items above at your actual base and report a per-Fact verdict before editing.
- Decide **by reading** which record is this decision's evidence, rather than mechanically moving the research id from one field to the other. Moving it is the obvious repair and is probably right — but `docs/document-metadata.md` warns explicitly that forcing a typed edge produces false edges, so state the reasoning that the record you name actually is the evidence this decision rests on. If more than one record qualifies, name them all; if none does, **stop and report** rather than inventing an edge to satisfy the schema.
- Decide whether the third `applies_to` entry should be removed, or replaced by the normative contract this decision genuinely applies to. Those are different claims; say which you concluded and why.
- Check the sibling records that place the same id in `evidence` (ADR 0003, 0051, 0079) to confirm the id and target are what you think they are before relying on them as the precedent.
- Sweep `docs/decisions/` for any other empty typed-edge array or cross-typed edge, since the lane that found this was sweeping for a different field. Report findings **and** clean results.

## Non-goals

Editing `docs/document-metadata.md`; changing ADR 0109's decisions, `decision_status`, or `implementation_status` (a sibling lane just derived the latter and its reasoning stands); building a frontmatter validator, which is a separate ticket with its own cost.

**Also out of scope, recorded so it is not rediscovered as new:** the same lane observed that `docs/research/extensions/backend-provider-composition.md` still classifies `ScalarLoweringProvider` as "implemented support only / yes (as a seam)" while being ADR 0105's own `evidence` target, and that three present-tense sentences in ADR 0078 still describe the scalar family as in-tree, installable, and pending removal. Those are `research/extensions` and a separate ADR's prose respectively — file or dispatch them separately rather than folding them in here.

## Closes when

`evidence` names at least one record whose status as this decision's evidence is argued rather than assumed, `applies_to` names only normative contracts, the sibling-precedent check is reported, the wider sweep is reported with both findings and clean results, and `make citations` is green.
