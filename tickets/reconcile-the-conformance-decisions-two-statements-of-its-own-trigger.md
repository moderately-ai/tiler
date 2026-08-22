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

## Source-first Fact audit — 2026-08-22 at base `77cd01049a614ba95e03bae4018a18dcdb156cf7`

Every anchor below was run with `grep -c` against the file the citation names before being relied on. All three quotations are verbatim; what the audit repairs is not the quoted bytes but the word **requires**, which both Facts apply to sentences the record states as **sufficient** conditions.

| Fact as filed | Verdict | Evidence |
| --- | --- | --- |
| The numbered trigger **requires** an extraction and two fixtures | **Quotation verified, characterization false** | Anchor `A bounded extraction demonstrates two independently authored` returns 1. But its own lead is `sufficient on its own` (returns 1), the recommendation closes `is sufficient to reopen this decision` (returns 1), and the record says its `triggers are independent rather than an all-or-nothing conjunction` (returns 1). Nothing states a necessary condition. Trigger 1 is one sufficient route, not a precondition. |
| The reversal-evidence paragraph **requires** one fixture and no extraction | **Quotation verified, characterization imprecise** | Anchors `Evidence reversing the recommendation is one second independently authored fixture` and `That evidence alone reopens D1` each return 1. It states sufficiency, not a requirement, so it does not compete with Trigger 1 — a weaker sufficient condition subsumes a stronger one rather than contradicting it. |
| The dependent ticket **copied the weaker one** | **False as characterized** | The 2026-08-18 log line is verbatim (`The trigger is one second independently authored backend fixture sharing the portfolio` returns 1 in `tickets/publish-the-backend-provider-conformance-suite.md`), but calling it *the weaker one* presupposes the two are rival readings of a single trigger. It is not a weaker paraphrase: it matches the acceptance record's own single statement of the condition. The carrier copied the operative condition. |

**The ticket's framing was the defect, not either sentence.** The question *which of two statements governs* is malformed, because both are sufficient and neither is necessary. Satisfying either reopens the decision; satisfying neither does not. That is why no amount of preferring the stricter or the looser text could have settled it — the disagreement was never between the texts.

## Acceptance provenance, and what it settles — 2026-08-22

`decide-the-backend-provider-conformance-harness-public-surface`'s `## Accepted decision — 2026-08-18` section records the full provenance AGENTS.md requires: **who** Tom, **date** 2026-08-18, **venue** the live coordination session with the orchestrator, **relay** first-hand by the coordinator, exact accepted packet `ed1d557170ff8a2afb0fac11a39765dfc5b83a00`, and Tom's words `agreed, next decision` to an accept-or-reopen question in explain-then-recommend form.

That section states the condition **once**: *"The recorded reopening trigger is one second independently authored backend fixture sharing the same neutral, non-self-certifying structural and execution subjects as the portfolio; that evidence alone reopens the partial-facade question, and the two held carriers then expand only their named rows."* Its trailing clause about the two held carriers is drawn from numbered triggers 2 and 3, so it was composed with the numbered list in view — it records a reading rather than an oversight, which is the distinction that makes it usable as provenance instead of as one more restatement.

`.ticketsplease/decision-queue.md` item 14, the artifact actually put in front of Tom, carries the same split intact and with the same modality — Recommendation: *"One bounded two-fixture extraction with typed host unavailability, caller-owned policy"* … *"is sufficient to reopen a partial facade independently"*; Strongest counterpoint: *"A second independently authored fixture using the same non-self-certifying structural and execution subjects reverses the recommendation immediately"*. Neither claims necessity, so Tom's assent to a packet containing both is consistent with both being sufficient and inconsistent with only one governing. The record's own Pareto table settles which one was blocking: D1 was *"eliminated at this base solely by the missing neutral subject/second independent fixture, not by the carriers"*.

**Settled: the operative reopening condition is one second independently authored fixture sharing the portfolio's structural and execution subjects.** No question goes to Tom.

## Coordinator ruling overturned — 2026-08-22

The `not fired` ruling of 2026-08-22 is reversed to **fired**, and the reversal is recorded on the carrier's `## Trigger check log` with a reproducing command and that command's actual output. Two grounds were given for it and neither survives.

The first was that no bounded extraction was performed. True, and it does not gate: the extraction is a sufficient route, and requiring it before reopening is circular, because building the shared expression is the reopened decision's own work.

The second was that `crates/tiler-build/tests/custom_backend` cannot carry the execution subject. The underlying manifest fact is **true and re-verified at this base** — `crates/tiler-build/Cargo.toml` declares six tiler dependencies (artifact, cache, compiler, ir, metal, metal-aot) and neither `tiler-runtime` nor `tiler-reference` — but the inference does not reach this condition. No wording names `custom_backend`, and none requires both fixtures to sit inside `make full`. The comparator every wording names is **the portfolio**, which carries both subjects itself: `spikes/runtime/backend-provider-portfolio` assembles through `assemble_plan_artifact` and routes through `route_with_adapter` against `tiler-reference`. A reading that disqualified the retained spike would leave zero fixtures and make the condition unsatisfiable by construction, while the decision record counts that spike as the first fixture throughout.

**The fail-closed argument does not favour `not fired` here.** Reopening a typed *deferral* re-presents a question; it authorizes no public export, no `pub mod`, and no new crate, and the accepted deferral's stop boundary is untouched. Ruling `fired` routes more to Tom, not less.

## Repair as landed, and one deliberate deviation

- `tickets/decide-the-backend-provider-conformance-harness-public-surface.md` gains `## Correction — 2026-08-22`, which states the operative condition once and quotes both retired wordings verbatim. Trigger 1 and the reversal paragraph each gain an inline pointer to it so a top-down reader cannot reach either and read it as a precondition.
- **Trigger 1 was not deleted, and the ticket's `Closes when` anticipated that it would be.** Deleting it would both violate the preserve-retired-wording convention and destroy a genuinely sufficient reopening route. What is now stated exactly once is the *operative reopening condition*; Trigger 1 survives as a labelled stronger-and-sufficient route. This is a deviation from the ticket's literal wording, recorded rather than made silently.
- The carrier's 2026-08-18 entry is confirmed rather than repaired, with a note that its reproduce command — which roots at `crates/tiler-conformance/` and could previously only ever report absence — now has a non-empty root.
