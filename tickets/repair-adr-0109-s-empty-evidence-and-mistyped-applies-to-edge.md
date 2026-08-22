---
id: repair-adr-0109-s-empty-evidence-and-mistyped-applies-to-edge
title: Repair ADR 0109's empty evidence and mistyped applies-to edge
status: done
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

**Correction — 2026-08-22 by `worker-adr0109`, read at `ba46f2b2`: the Inference above is false, and it is the one a mechanical repair would have followed.** The sibling precedent is real — the three ADRs above do carry that id in `evidence`, at line 11 of each — but it is not what happened here. ADR 0109's `applies_to` transcribes its own *Traceability* triple in order, whose three clauses read "owns the provider-facing Unknown and diagnostic rules", "owns semantic discharge and the before-cover executable boundary", and "owns the absence of a runtime plan or fallback". Its *Consequences* likewise group the third with the second as "contracts". So the third entry was a genuine `applies_to` claim about a document whose title reads as a contract while its `kind` is `research`, not a displaced `evidence` edge; the empty `evidence` is an independent omission. The two defects are therefore two, and moving the id from one field to the other would have recorded a support relation the decision never asserted. The verdicts on the three Facts are unchanged: all three verified.

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

## Outcome

**Frontmatter repaired.** `applies_to` is now the two accepted contracts alone; `evidence` is `["tiler.research.shapes.constraint-prover-boundary", "tiler.research.indexing.index-access-model"]`. The record carries a dated *Typed-edge repair* note stating why each record is this decision's evidence and why the third `applies_to` entry was removed rather than replaced. Chosen by reading, not by moving the id: the prover-boundary record derives the `Proved`/`Disproved`/`Unknown` model and the `UnknownReason` variants that decisions 1 and 3 assign meaning to, realized variant-for-variant at `crates/tiler-ir/src/index/predicate.rs "pub enum IndexDomainUnknownReason {"`; the index-access record carries decision 2's "not a plan fallback" half. ADR 0084 — the sibling decision on index-domain predicates — already carries exactly this pair in `evidence`.

**`applies_to` conclusion: removed, not replaced.** No `kind: "contract"` record in the corpus owns runtime execution; the runtime execution contract exists only as a research record. The two contracts it `informs` are refused as substitutes by ADR 0109's own decision 4, and `tiler.contract.ir` would be a governance claim the record nowhere makes. `applies_to` retains two accepted contracts, so the one-contract floor holds.

**Wider sweep — clean apart from this record.** Over 309 governed documents carrying an `id` and a `kind` (`docs/**/*.md`, `spikes/**/README.md`, `README.md`), 830 typed edges were resolved across `applies_to`, `evidence`, `informs`, `adopted_by`, and `supports` — a wider population than the check pinned in [`docs/document-metadata.md`](../docs/document-metadata.md), which omits `applies_to` and `adopted_by`. Results: 1 empty array, 1 mistyped edge, 0 dangling ids, 0 duplicate array members, 0 occurrences of the `governed_by`/`reproduced_by`/`links`/`deps` keys invalid in stored v1. Both defects were ADR 0109's. Of 113 decision-record files, 111 are `accepted`; ADR 0109 was the only one missing `evidence`, and none was missing `applies_to`. So the population was one record and not many.

**Remainder, outside this ticket's scopes.** The ADR catalog row for 0109 in `docs/decisions/README.md` still renders the removed id under `contracts:` and renders no `evidence:` clause. That file belongs to `contracts/navigation`, so the catalog edit that [`docs/document-metadata.md`](../docs/document-metadata.md) requires in the same change as the frontmatter behind it could not be made here. It needs the `contracts:` list cut to the two contracts and an `evidence:` clause added naming the two research records, in the shape ADR 0084's row already uses. `make citations` is green either way, because the stale row's link resolves; the defect is semantic.
