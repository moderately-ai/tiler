---
id: decide-whether-a-derived-budget-belongs-in-the-request-subject
title: Decide whether a derived budget belongs in the request subject
status: done
priority: p2
dependencies: []
related: [derive-the-region-shape-budgets-from-the-declaration, state-the-rule-that-a-deterministic-budget-is-a-derivation]
scopes: [contracts/optimizer, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, identity, decision, needs-tom, public-boundary]
---
## Decision accepted — 2026-08-11

Tom accepted retaining all fourteen effective budget values in `VerifiedRequestSubject`, including `region_members` and `region_live_values`.

**Fact — the two equal numbers are not redundant meanings.** Region formation counts realization-stage atoms rather than only semantic occurrences, and its live-value population includes handed intermediates that are absent from the semantic value count. Current registered sequence laws exercise that distinction. The governed equalities `region_members == semantic_operations` and `region_live_values == semantic_values` are numeric coincidences for the bounded profile, not derivations that erase the two policy axes.

**Fact — the original artifact/cache consequence was false.** These bytes bind the compiler-internal request, evidence, receipts, and explain subject. They do not directly enter artifact or cache identity. A budget change moves packaged identity only if it changes the selected packaged content; the literal pin that directly moves is the explain qualifier, currently `request=4f6429492ac63d04` before this prose-only decision record.

**Measurement — removal has no material compiler-host performance case.** The current measured canonical request subject is 244,024 bytes. The disputed two `u32` fields are 8 bytes, about 0.0033%; across the measured ceiling of twelve subject reconstructions, removal would save at most 96 bytes written against about 2.9 MiB. The complete fourteen-value vector is 64 bytes.

**Inference — explicit values are the strict carrier today.** Omitting independently retained policy fields would let different internal compilation questions share canonical subject bytes. A field may leave only after its value is mechanically re-derived from complete already-encoded inputs under versioned derivation authority and that invariant is enforced at construction. These two staged-region fields do not meet that condition.

If a second budget policy becomes real, its public form must be one explicit, complete, typed policy with no `Default`, ambient choice, or per-field fallback. A future opaque policy identity must be content-complete rather than a manually bumped name. Until that trigger fires, adding such an abstraction is premature under ADR 0069.

No value, field, byte order, domain tag, public API, or runtime behaviour changes under this decision.

## The question

`VerifiedRequestSubject::canonical_explain_subject_bytes` writes **all fourteen** deterministic budgets into the canonical request subject. It binds compiler-internal request/evidence identity and the explain qualifier, not artifact or cache identity directly. That is why every budget widening moves every governed compilation's qualifier — "for programs nowhere near any of these bounds as much as for ones at them, because a budget is a property of the *request* rather than of the plan chosen for it", as the budget comment puts it.

**But if a budget is a pure function of the declaration, and the declaration is already in the subject, then encoding the budget too is redundant** — it cannot distinguish two requests, because any two requests with equal declarations have equal budgets by construction. If that holds, the budget bytes can leave the subject, and **every future envelope widening becomes identity-neutral.**

That is the structural fix underneath [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) and [`state-the-rule-that-a-deterministic-budget-is-a-derivation`](state-the-rule-that-a-deterministic-budget-is-a-derivation.md): those two stop the ceiling being raised by hand; this one would stop the raising from costing anything.

## What must be established before any of that is true

**It is stated as a hypothesis and must not be assumed.** The coordinator did not verify what else the subject carries when filing this, and the whole argument turns on that. Establish, by reading rather than inference:

- **What the subject actually encodes.** Read `canonical_explain_subject_bytes` in full. Does it carry the declaration — input count, output count, occurrence count, the program's own identity — in a form from which every derived budget is recoverable? If the derivation's inputs are *not* in the subject, the budgets are not redundant and this ticket ends there, which is a legitimate outcome.
- **Whether every budget is derivable.** The rule ticket's audit is the input here: a budget that stays a genuine constant is not a function of the declaration, so it cannot leave the subject on this argument. The answer may well be "the derived ones may leave, the constants may not", which is a partial and still-valuable result.
- **Whether the subject has non-identity readers.** The subject is also rendered in explain output. A budget removed from the encoding may still need to be *reported*, and those are different requirements — encoding is about distinguishing requests, reporting is about telling a caller what bounded their compilation. Do not conflate them.

## The counter-argument, which is strong and must be answered rather than noted

**A budget is a property of the request, and the current design says so deliberately.** Two callers who ask for the same program under different budgets are asking different questions, and the subject exists to distinguish requests. The redundancy argument only defeats that if the budgets are *not* independently settable — that is, if the governed profile's budgets are the only ones reachable. **If a caller can state its own budgets, they are an input and must be encoded**, and the derivation changes nothing.

So the first thing to settle is whether budgets are ever caller-supplied, or always a property of the profile. Read `CompileRequest`'s construction and every `DeterministicBudgets` construction site.

## If the answer is yes

It is an encoding step on `tiler.compiler.request-subject.v6` and therefore an identity-domain migration with its own evidence and its own acceptance — **do not land it under this node.** File it, with the enumerated pinned population, and state plainly that the migration's cost is paid once against a permanent reduction in future identity movement. That trade is the decision, and it is Tom's.

## Explicit non-goals

No value changes, no encoding change under this ticket, and no pre-empting the two tickets above. This is research that ends in a decision, not an implementation.

## Closes when

The subject's contents are established by reading; whether budgets are ever caller-supplied is answered; the redundancy claim is proved or refuted for each budget class; and either the encoding step is filed with its population and trade stated, or the ticket records why the budgets must stay and what that costs.

## Re-scoped to a decision brief, 2026-08-07 — this is no longer research

Verified by the coordinator reading `crates/tiler-compiler/src/request.rs` and `src/session.rs` in full. **The decision is genuinely still open and is genuinely Tom's** — three landed records reserve it explicitly: `request.rs`, anchor `profile can no longer fire deserves to keep its slot`; `docs/compiler/optimizer.md`, anchor `whether a field in that position keeps its slot`; and [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md), anchor `keep their slots in the canonical request subject`. What changed is that nothing remains to *research*: the tree already answers both questions this ticket sent a worker to establish.

### Struck: the central premise. Budgets are not runtime functions of the declaration.

The ticket argues from "a budget is a pure function of the declaration". **`DeterministicBudgets::governed` is a nullary `const fn` returning fourteen integer literals** (`request.rs`, source anchor `pub(crate) const fn governed() -> Self`). `region_members` is the literal `62`, not `semantic_operations`; `region_boundary_outputs` is the literal `3`, not the declared output count of the program being compiled. The derivation is **authoring-side** — performed once against the C1 decode row of the decoder layer and recorded in prose. Nothing is computed from the request's declaration at runtime.

**The redundancy conclusion survives on a stronger and simpler ground.** Because `governed()` is nullary and no second budget set is reachable, the fourteen budget words are *constant across the entire encodable request space* — trivially non-distinguishing, with no appeal to derivation needed. Restate the argument on **constancy**, not derivation. The ticket's route is neither necessary nor, as stated, correct.

### Struck: the stop rule, which drives to the wrong answer

"If the derivation's inputs are *not* in the subject, the budgets are not redundant and this ticket ends there." `VerifiedRequestSubject::canonical_explain_subject_bytes` carries the semantic graph identity digest, the declared output count, per-output input-key runs, and member ordinal runs — but **no program-wide value or occurrence count**. Applied literally, the stop rule therefore *always* fires "ends there", and on a false ground twice over: injectivity needs functional determination (equal graph identity ⇒ equal declaration ⇒ equal budgets), not byte-level recoverability; and the budgets are constant across the reachable request space regardless of what the subject carries.

### Struck: the closing condition's third conjunct

"the redundancy claim is proved or refuted **for each budget class**" is unevaluable. "Budget class" is defined nowhere here; the `{derived, constant}` partition it implies is precisely the deliverable of `state-the-rule-that-a-deterministic-budget-is-a-derivation`, which is `todo` and is **not a declared dependency** — this ticket declares `dependencies: []` while naming that ticket's unwritten output as its input. It is also moot: post-landing all fourteen are literals, so the partition does not exist in code.

### The question narrows, and the narrow one is what the landed records actually hand over

This ticket asks the broad question — do *any* budgets belong in the subject, all fourteen. All three records that hand it over ask something much narrower: whether the **two numerically coincident fields**, `region_members` and `region_live_values`, keep their slots. They are shadowed by the program bounds for a single-stage occurrence population, but can still bind when realization sequences contribute multiple stage atoms or handed values. Decide that.

**And note the collapse is not universal**, which is the strongest argument for keeping them: `request.rs`, anchor `both bounds still bind on a program whose`, records that region formation's attribution atom is a realization *stage* rather than an occurrence, and its live values include intermediates a staged law hands between stages. The collapse holds only where each occurrence is realized by one region. `region_boundary_outputs` does not collapse at all.

### What is already settled, so no worker needs to establish it

- **Budgets are never caller-supplied.** The public `CompileRequest`, source anchor `pub struct CompileRequest`, has no budgets field; `compile` always builds `CompilationRequest::governed_preferring`, overriding only `target_profiles` and `capabilities`. Both `DeterministicBudgets` and `CompilationRequest` are `pub(crate)`, and `request` is not a `pub mod`. Every non-`governed()` value in the workspace is inside a `mod tests`. `session.rs`, anchor `Budgets and the shape environment stay internal`, states it outright.
- **The literal pin population is one and it is measured.** The explain qualifier is the one checked-in literal derived from these bytes; the sibling's `0aa252e0bfa16451` → `e59cb8aa9b38ef70` pair is historical, and the live value before this prose-only decision is `4f6429492ac63d04`. `tiler-build`'s artifact goldens did not move because the request subject is not their identity.

**Correction — 2026-08-10.** The `request.rs` comment pair anchors `Exactly one pinned identity encodes these bytes` (budget-byte pin claim on the governed profile) and `tiler-build's standard Metal goldens` (staged-family encoder arm: no pin encodes a *staged* subject) address different subjects. They are not two contradictory descriptions of the budget-pin population. Sibling measurement already separates the domains: budgets are request-subject properties; tiler-build artifact goldens did not move with the budget value change. Residual comment hygiene is optional if a landing opens those sites; it is not a load-bearing board defect for this decision.

### Scopes corrected

`research/verification` **removed** — this ticket does not edit `docs/research/verification/**` or `spikes/verification/**`, so that exclusive scope matched nothing this node should touch. `contracts/optimizer` **added**, because the "budgets stay" branch obliges correcting `docs/compiler/optimizer.md`, anchor `whether a field in that position keeps its slot`, which currently asserts the decision is open; without it that branch could not be completed inside the declaration.

**Correction — 2026-08-10.** The earlier scopes-correction sentence claimed those two verification trees **neither exist**. That existence claim is false at the current tree: both `docs/research/verification/` and `spikes/verification/` are present (e.g. kani-bounded-encoder records). Scope removal remains correct because this ticket still does not edit those paths; only the existence justification was wrong.

## Fact audit — 2026-08-10

- **Live request-subject domain is `tiler.compiler.request-subject.v6`**, not `v5`. Encoder prefix is `b"tiler.compiler.request-subject.v6\0"` (`request.rs`, anchor `tiler.compiler.request-subject.v6`). The domain stepped when the fifth semantic-identity subject (`shape_environment`) was folded; budget field set, widths, and order did not force that step. Stale `v5` still appears in *comments* inside `request.rs` only. Present-tense domain names in this ticket (Tom question; remove-branch encoding-step sentence) were corrected to `v6` with this note.
- Scopes and pin-comment-pair corrections are above in their sections.

## The question for Tom — answered 2026-08-11

**Do `region_members` and `region_live_values` — the two budget fields the governed profile can no longer fire — keep their slots in `tiler.compiler.request-subject.v6`?**

- **Keep them.** They still bind for a program whose families realize region sequences (`request.rs`, anchor `both bounds still bind on a program whose`), and a budget is a *request* field rather than a plan property, so the governed profile's coincidence is a property of its declaration and not of the fields. Removing a field that a future non-governed budget set would need means re-adding it later at an identity-domain step.
- **Remove them.** All fourteen budget words are constant across the entire encodable request space, so they distinguish no two requests and every future envelope widening becomes identity-neutral. This costs one identity-domain step now to avoid paying one on every subsequent widening.

**Accepted: keep them, and record why.** The current single public budget set does not make independently meaningful policy axes a derivation. `request.rs`, anchor `both bounds still bind on a program whose`, names the live staged shape where the two bounds differ from semantic counts. Keeping them preserves exact request/evidence binding at negligible host cost and avoids a `v6` → `v7` removal followed by a later reintroduction. The strongest counterpoint — that one reachable budget set makes the bytes constant over current public calls — is true but insufficient: current reachability is not a constructor-enforced derivation and cannot justify canonicalizing distinct compiler policies together.

**Future trigger:** a second real budget policy or a public budget-policy request. That work must present one required complete typed policy, no silent default, and rederive its canonical carrier before implementation.
