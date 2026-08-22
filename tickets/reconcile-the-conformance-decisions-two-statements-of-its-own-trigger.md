---
id: reconcile-the-conformance-decisions-two-statements-of-its-own-trigger
title: Reconcile the conformance decision's two statements of its own trigger
status: in-progress
priority: p2
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface, publish-the-backend-provider-conformance-suite, supply-the-second-independently-authored-backend-fixture]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decisions, backend-providers, graph-hygiene]
claimed_from: todo
assignee: worker-trigger
lease_expires_at: 1787431855
---
## User-visible outcome

`decide-the-backend-provider-conformance-harness-public-surface` states its reopening trigger once, so whether a delivered fixture reopens that decision is answerable by reading rather than by choosing which of two sentences governs.

## Why this exists

Filed 2026-08-22 by the coordinator while evaluating whether the second independently authored backend fixture fired the trigger. It could not be answered, because the decision record states the trigger twice and the two statements are not equivalent — one is satisfied by that delivery and one is not.

**Fact — the numbered trigger requires an extraction and two fixtures.** Trigger 1 reads: *"A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject without optional responsibility fields, a whole-backend provider trait, parsing diagnostics, or callbacks that can manufacture success."* Anchor: `A bounded extraction demonstrates two independently authored`.

**Fact — the reversal-evidence paragraph requires one fixture and no extraction.** The same ticket's counterargument section reads: *"Evidence reversing the recommendation is one second independently authored fixture that shares exact structural and execution subjects with the portfolio without those defects. That evidence alone reopens D1."* Anchor: `That evidence alone reopens D1`.

**Fact — the dependent ticket copied the weaker one.** `publish-the-backend-provider-conformance-suite`'s `## Trigger check log` entry for 2026-08-18 paraphrases the trigger as *one* second fixture sharing the portfolio's subjects, matching the reversal-evidence sentence rather than Trigger 1.

All three verified by the coordinator at `b3c07259`, each read in the file it names.

**Coordinator ruling, recorded so it is not silently reversed.** The trigger was logged **not fired** on 2026-08-22 on the numbered trigger, because no bounded extraction was performed — that was the delivering ticket's stated non-goal — and because `crates/tiler-build` declares neither `tiler-runtime` nor `tiler-reference` in its manifest, so `tests/custom_backend` structurally cannot carry the execution subject. The fail-closed reading was chosen deliberately: reopening an accepted decision on an ambiguous trigger is the riskier direction. **That ruling stands only until this ticket resolves the ambiguity.**

## Required work

- Re-audit all three Facts at your own base and report a per-Fact verdict.
- Determine which statement the accepting record intended, by reading its acceptance provenance rather than by preferring the stricter or the looser text. If the provenance does not settle it, say so and stop — this becomes Tom's, not a worker's.
- Repair the decision record so it states its trigger once, preserving the retired wording in a dated correction rather than deleting it.
- Repair the dependent ticket's `## Trigger check log` to quote the single surviving statement, and re-evaluate the 2026-08-22 entry against it.

## Non-goals

Reopening the decision; performing the extraction; changing the delivered fixture; and any edit to `crates/`.

## Closes when

The decision states its reopening trigger exactly once, the retired wording is preserved in a dated correction, the dependent ticket's trigger log quotes the surviving statement, and the 2026-08-22 evaluation is re-recorded against it with its reproducing command.
