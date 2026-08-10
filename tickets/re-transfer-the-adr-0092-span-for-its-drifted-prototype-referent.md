---
id: re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent
title: Re-transfer the ADR 0092 span for its drifted prototype referent
status: done
priority: p2
dependencies: []
related: [rename-the-route-resource-floor-vocabulary-for-its-corrected-relation, close-the-serial-sum-run-gpu-family-probe-table, correct-adr-0092-alternatives-considered-prototype-citation]
scopes: [contracts/decisions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, runtime]
---
## Why this exists

`docs/research/runtime/backend-scoped-route-requirement-answers.md` flagged a drifted sentence inside its retained ADR-0092 span **for the ADR 0092 acceptance sweep** on 2026-08-01. The acceptance sweep did not reach it, and no node on the board carried it, so the flag has been sitting in a research record's prose since. Found while executing `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`, which hit the same span-versus-authority condition and followed the same rule.

**Fact — the drifted sentence.** [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md)'s *Alternatives considered* entry **Publish the family vocabulary and let each consumer observe the device itself** reads "written as a table rather than a match — which is what the existing prototype does". At drafting, "the existing prototype" was `prototypes/candle-metal-adapter`; `662d9be` removed its table that evening. The sentence is not false — `prototypes/serial-sum-run/src/proof.rs` still carries the identical table, open under [`close-the-serial-sum-run-gpu-family-probe-table`](close-the-serial-sum-run-gpu-family-probe-table.md) — but its singular referent names a different prototype than its author had in view, and a reader who resolves it to the candle adapter finds nothing there.

**Fact — the record already prescribes the repair and its order.** The paragraph beside the span states it: the sentence "should become 'which is what a prototype still does' in the ADR, and this span should be re-transferred from the ADR at that point rather than corrected here first." The ADR is the authority; the span follows it. Editing inside the span first would fork the byte-identical transfer that makes the span quotable at all.

**Fact — a second correction is now queued behind the same re-transfer.** `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation` corrected decision item 8's `ResourceFloor` spelling in ADR 0092 and deliberately left the span at the pre-rename spelling, recording it beside the span under the same rule. So the re-transfer now carries **two** corrections, not one, and doing them as one act is cheaper and leaves no intermediate state in which the span matches the ADR on one sentence and not the other.

## What closes this

Apply the alternatives-entry correction in ADR 0092 (`the existing prototype` → `a prototype still does`, exact wording the record prescribes), then re-transfer the span in the research record from the ADR so the two are byte-identical again across both the alternatives entry and decision item 8. Verify with `cmp` on the corresponding line pairs rather than by eye, and fold the two "recorded beside the span" notes into one statement of what the re-transfer settled — leaving the note that explains *why* the span is provenance rather than a second authority, which is still true and is cited by AGENTS.md as the standing convention.

**Correction — 2026-08-10.** AGENTS.md does not cite that link-condition note as a standing convention and never did. The convention is the research record's own. AGENTS.md's related rule is only the verbatim-landable transfer sentence (`preserve a verbatim-landable ADR body and file a carrier ticket; editing during transfer creates a fork`). The research record already corrected the same AGENTS-as-convention claim on 2026-08-08.

Do not change `decision_status`, do not reword any decision item beyond the already-applied type spelling, and confirm the prototype claim by reading `prototypes/serial-sum-run/src/proof.rs` at the tip rather than trusting this ticket — if `close-the-serial-sum-run-gpu-family-probe-table` has landed by then, "a prototype still does" is itself false and the sentence needs a different repair, which is the one thing here that must not be applied mechanically.

## Outcome — 2026-08-06, `67abe1da`

Delivered with [`correct-adr-0092-alternatives-considered-prototype-citation`](correct-adr-0092-alternatives-considered-prototype-citation.md) as one act, base `01ad1c99`. Docs-only: `docs/decisions/0092-…md` and `docs/research/runtime/backend-scoped-route-requirement-answers.md`.

**Fact — this ticket's own caveat fired, and the prescribed wording was not applied.** `close-the-serial-sum-run-gpu-family-probe-table` is `done` at `8a5e20c5`, so "which is what a prototype still does" would have replaced one false sentence with another. Read from source at this commit rather than from the ticket: `prototypes/serial-sum-run/src/proof.rs:1161-1319` drives the walk from `observe_highest_gpu_family` and joins the two vocabularies by the `MTLDevice.h` number both sides transcribe (`binding_apple_enumerator` compares `enumerator as isize` against `AppleGpuFamilyConstant::value()`), refusing an enumerator the binding cannot name as `ProbedGpuFamily::Unnameable`; `prototypes/candle-metal-adapter/src/adapter.rs:658-661` passes the raw value straight to `supportsFamily`. Neither writes a pair table. The wording chosen instead — the table in the past tense as measured evidence, plus decision item 3's construction as the positive case — is reasoned in the sibling ticket's Outcome.

**Correction — 2026-08-10.** The line windows above are not live pins. Locate by symbol: serial-sum-run joins at `const fn binding_apple_enumerator` and walks at `fn probe_apple_families` (both still drive `observe_highest_gpu_family`); candle-metal-adapter's raw pass-through is `pub fn observed_apple_family`, whose body is `observe_highest_gpu_family(|family| raw.supportsFamily(MTLGPUFamily(family.value())))`. Neither prototype writes a pair table. Line numbers are base-bound and are not re-asserted here.

**The re-transfer, verified rather than eyeballed.** Both drifted passages were corrected in the ADR first and the span was then re-transferred from it: decision item 8 (`ADR:46` → span `:378`) and the alternatives entry (`ADR:64` → span `:394`). The span lines were replaced by copying the ADR's exact bytes rather than retyping them.

```sh
ADR=docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md
REC=docs/research/runtime/backend-scoped-route-requirement-answers.md
cmp <(sed -n '46p' "$ADR") <(sed -n '378p' "$REC")   # item 8:            no output, exit 0
cmp <(sed -n '64p' "$ADR") <(sed -n '394p' "$REC")   # alternatives entry: no output, exit 0
```

Both silent, both exit 0. The check was widened past the two known lines rather than stopping at them: a script walked every non-empty line of the span between its opening and closing rules, excluding the `**Title:**` and `**Frontmatter:**` drafting lines, and asserted each appears verbatim in the ADR — with `###` headings matched against their `##` demotions. **28 span content lines checked, 0 divergences**, which is the byte-identity claim discharged across the whole span rather than only where it was known to have drifted.

**Correction — 2026-08-10.** The `ADR:46`/`span:378` and `ADR:64`/`span:394` pins above were correct at delivery (`67abe1da`) but have rotted at later bases (later ADR notes and the item-6 restatement shifted lines). Identify the pairs by subject anchors, not those line numbers: decision item 8 begins `The pattern is available to every backend`; the alternatives entry begins `Publish the family vocabulary and let each consumer observe the device itself`. At the 2026-08-10 audit base those were `ADR:48`↔`REC:389` and `ADR:72`↔`REC:405` (`cmp` silent exit 0). Line numbers are base-bound; re-locate by the anchors. The whole-span content identity claim (28 non-empty span content lines, Title/Frontmatter excluded, `###` demoted to `##`, 0 divergences) still holds when re-walked.

**What the fold settled.** The two "recorded beside the span" notes — the 2026-08-01 prototype-referent note and the 2026-08-05 `ResourceFloor` note — become one paragraph stating both drifts, that both were corrected in the ADR first and re-transferred, that the span is byte-identical again, and that neither correction changed what the record decides. It also records why the prescribed repair changed, so a reader is not left holding a wording this record once promised. The paragraphs explaining *why* the span is provenance rather than a second authority, and the link-condition note AGENTS.md cites as the standing convention, are untouched.

**Correction — 2026-08-10.** "link-condition note AGENTS.md cites as the standing convention" is false for the same reason as under What closes this: AGENTS.md carries no span/repoint/standing-convention sentence about drafted ADR bodies; only the verbatim-landable transfer rule. The research record owns that convention prose.

**Beyond the two prescribed sites, in the same file and the same direction.** The `:74` flag and three siblings carrying the same stale claim were rewritten to current truth: the section heading, the b2 **Inference**, the question-1 elimination bullet, and the deferral bullet. Rewritten in place, not appended to — provenance is in the commit message.

**Checks.** Docs-only diff (`docs/` and `tickets/` only, no crate code), so no cargo gate is owed; `git diff --check` clean; `tkt lint` and `tkt guard` recorded in the dispatch report.
