---
id: publish-the-partial-backend-conformance-facade
title: Publish the partial backend-conformance facade
status: todo
priority: p1
dependencies: [decide-the-backend-provider-conformance-harness-public-surface]
related: [publish-the-backend-provider-conformance-suite, supply-the-second-independently-authored-backend-fixture, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio]
scopes: [implementation/conformance, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, conformance, public-boundary, needs-tom]
---
## User-visible outcome

A third-party backend author can exercise the accepted production seams through a published, explicitly partial conformance surface, with everything it cannot establish typed as unsupported rather than absent or defaulted.

## Why this exists

Filed 2026-08-22 by the coordinator, executing the instruction the accepted typed deferral gives for exactly this event. Its Graph clause reads, verbatim: *"If Trigger 1 fires first, a separately scoped partial-publication carrier (or a truthfully narrowed implementation ticket) must depend on this public-boundary decision and the neutral-subject evidence, not inherit the complete suite's carrier dependencies."* This ticket is that carrier.

**Fact — the reopening condition fired on 2026-08-22.** `crates/tiler-conformance/tests/independent_backend/` landed at merge `829bd1f0` as the second independently authored backend fixture, sharing the portfolio's neutral, non-self-certifying structural and execution subjects, with `tiler-reference` as sole oracle and four perturbations quoted failing.

**Fact — the coordinator first logged this trigger as `not fired`, and that ruling was overturned on evidence.** I read the numbered Trigger 1's body — *"A bounded extraction demonstrates **two** independently authored backend fixtures"* — and treated it as a necessary condition competing with the record's one-fixture statement. **It is not.** Trigger 1's own lead reads *"Partial-facade reopening trigger, **sufficient on its own**"*, the recommendation closes *"is sufficient to reopen this decision"*, and the record states *"triggers are independent rather than an all-or-nothing conjunction"* — all three verified by the coordinator at `0065013e`, each returning 1. **The record states no necessary condition anywhere.** Both sentences are sufficient conditions, and a weaker sufficient condition subsumes a stronger one rather than competing with it, so the question of "which governs" was malformed by construction. `worker-trigger` established this from the acceptance provenance rather than by preferring either text.

**Note the fail-closed reasoning I used was backwards, which is worth carrying.** I ruled `not fired` because reopening an accepted decision seemed the riskier direction. Reopening a typed *deferral* re-presents a question and authorizes no export — so ruling `fired` routes **more** to Tom, not less. A fail-closed instinct suppressed a decision that should have reached him.

**This carrier does not inherit the complete suite's dependencies.** `publish-the-backend-provider-conformance-suite` is `deferred` behind five prerequisites, three of them unfinished, because its stated outcome claims the complete compilation→artifact→route→execution suite. The Graph clause is explicit that a partial carrier must not inherit those. It depends on the reopened decision and the neutral-subject evidence, and on nothing else.

## Required work

- **Do not start until the reopened decision resolves.** `decide-the-backend-provider-conformance-harness-public-surface` is now `todo` and owes a re-derived packet naming the exact supported subset. Publishing a surface before Tom accepts its included and excluded boundary is precisely what AGENTS.md reserves to him.
- Re-audit every Fact above at your base with a per-Fact verdict.
- Whatever is published must type its unsupported population rather than omitting it. The decision's own list is long — third-party reusable reports, certification, arbitrary mathematical correctness, benchmarks, dynamic plugins, adapter discovery, missing-adapter tests, explain-disposition coverage, generic device buffers, non-Metal availability policy, and any pass synthesized from unavailable hardware. **Rows the subject cannot establish stay typed unsupported output, never absent or defaulted success** — the record says so in the same breath as the trigger.
- Perturb the subject separately for each refusal and quote the failure text, including one control that a caller cannot manufacture a pass.

## Non-goals

The complete suite, which `publish-the-backend-provider-conformance-suite` still owns and which remains correctly deferred. Claiming rows 4, 6, or 11 complete — those wait on their own carriers under triggers 2 and 3. Explain-disposition coverage, which may be typed as excluded today.

## Closes when

Tom has accepted the exact published surface with its typed unsupported population, a third-party author can reach the accepted seams through it, no row the subject cannot establish reads as success, and each refusal has been watched firing.


## Correction — 2026-08-22, by `worker-conformance` at base `b6248f91`: the Non-goals paragraph names two carriers that have already landed

The Non-goals above read, verbatim and preserved here so the repair shrinks no grep count: *"Claiming rows 4, 6, or 11 complete — those wait on their own carriers under triggers 2 and 3."* **Both carriers are `done` at this base**, so triggers 2 and 3 have fired and those rows no longer wait on anything.

Verified in source rather than from ticket status. `package-selected-physical-implementation-provenance-in-artifact-identity` landed the selected-physical run through the artifact encode, decode, budget, and builder paths, and exposed it publicly: `pub fn selected_physical_implementations` exists on both `VariantRef` (`crates/tiler-artifact/src/program/model.rs`) and `DecodedVariant` (`crates/tiler-artifact/src/program/codec/view.rs`), so an author can read selected-physical provenance out of decoded bytes today. `carry-required-compilation-selection-identity-on-compile-profile-contexts` landed on 2026-08-19 with a full identity cascade, including the new owned domain `tiler.metal-aot.compilation-selection.v1` and `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` 3 → 4.

Left standing rather than edited in place, because the paragraph is a true record of what was believed when this ticket was filed. A worker reading the Non-goals as current would truthfully-but-wrongly exclude three rows that are now claimable, which is the failure the exclusion existed to prevent, inverted.

**This ticket's own premise is also now in question**, and that is for the reopened decision to answer rather than for this note. The re-derived packet on `decide-the-backend-provider-conformance-harness-public-surface` recommends publishing no facade at all, on the evidence that the fixture merge `829bd1f0` changed no production crate source and that `grep -rn 'tiler_conformance' crates/tiler-conformance/tests/` returns 0 — an external author already reaches every accepted seam with no facade. If Tom accepts that, this carrier closes rather than building. Do not start it before that decision resolves, which the Required work above already says.


## Note — 2026-08-22, by `worker-packet` at base `3291b105`: the packet the correction above cites was returned, and a second one replaced it

The correction above says *"The re-derived packet on `decide-the-backend-provider-conformance-harness-public-surface` recommends publishing no facade at all"* and gives two evidence lines for it. Read that sentence with this note: **the packet it names was returned by independent review**, at commit `5b08faae`, because its crux — that a published report `can only record what the caller tells it` — is refuted five times in public API. One of its two cited evidence lines is also imprecise: the fixture merge `829bd1f0` did change a production crate source, adding 24 `//!` documentation lines to `crates/tiler-conformance/src/lib.rs`. The accurate claim is that it added **no public item to any production crate**, which is what the argument needs; the other cited line, `grep -rn 'tiler_conformance' crates/tiler-conformance/tests/` returning 0, reproduces exactly at this base.

A second re-derivation now sits on that decision. **It reaches the same recommendation — publish no facade — on a replacement ground**: a caller-supplied evidence class is safe to publish where Tiler itself consumes it at a site that can refuse, re-derive, bound, or separate identity, and a third-party conformance report has no such site. So the standing expectation for this carrier is unchanged: **if Tom accepts, this ticket closes rather than building.**

**It has still not resolved**, and the second re-derivation has not been independently reviewed either. `Do not start it before that decision resolves` continues to govern, and a worker must not treat either packet's recommendation as the decision.
