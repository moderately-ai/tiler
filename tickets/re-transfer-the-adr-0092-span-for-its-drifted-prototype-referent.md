---
id: re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent
title: Re-transfer the ADR 0092 span for its drifted prototype referent
status: in-progress
priority: p2
dependencies: []
related: [rename-the-route-resource-floor-vocabulary-for-its-corrected-relation, close-the-serial-sum-run-gpu-family-probe-table, correct-adr-0092-alternatives-considered-prototype-citation]
scopes: [contracts/decisions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, runtime]
claimed_from: todo
assignee: agent-adr0092
lease_expires_at: 1786049402
---
## Why this exists

`docs/research/runtime/backend-scoped-route-requirement-answers.md` flagged a drifted sentence inside its retained ADR-0092 span **for the ADR 0092 acceptance sweep** on 2026-08-01. The acceptance sweep did not reach it, and no node on the board carried it, so the flag has been sitting in a research record's prose since. Found while executing `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`, which hit the same span-versus-authority condition and followed the same rule.

**Fact — the drifted sentence.** [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md)'s *Alternatives considered* entry **Publish the family vocabulary and let each consumer observe the device itself** reads "written as a table rather than a match — which is what the existing prototype does". At drafting, "the existing prototype" was `prototypes/candle-metal-adapter`; `662d9be` removed its table that evening. The sentence is not false — `prototypes/serial-sum-run/src/proof.rs` still carries the identical table, open under [`close-the-serial-sum-run-gpu-family-probe-table`](close-the-serial-sum-run-gpu-family-probe-table.md) — but its singular referent names a different prototype than its author had in view, and a reader who resolves it to the candle adapter finds nothing there.

**Fact — the record already prescribes the repair and its order.** The paragraph beside the span states it: the sentence "should become 'which is what a prototype still does' in the ADR, and this span should be re-transferred from the ADR at that point rather than corrected here first." The ADR is the authority; the span follows it. Editing inside the span first would fork the byte-identical transfer that makes the span quotable at all.

**Fact — a second correction is now queued behind the same re-transfer.** `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation` corrected decision item 8's `ResourceFloor` spelling in ADR 0092 and deliberately left the span at the pre-rename spelling, recording it beside the span under the same rule. So the re-transfer now carries **two** corrections, not one, and doing them as one act is cheaper and leaves no intermediate state in which the span matches the ADR on one sentence and not the other.

## What closes this

Apply the alternatives-entry correction in ADR 0092 (`the existing prototype` → `a prototype still does`, exact wording the record prescribes), then re-transfer the span in the research record from the ADR so the two are byte-identical again across both the alternatives entry and decision item 8. Verify with `cmp` on the corresponding line pairs rather than by eye, and fold the two "recorded beside the span" notes into one statement of what the re-transfer settled — leaving the note that explains *why* the span is provenance rather than a second authority, which is still true and is cited by AGENTS.md as the standing convention.

Do not change `decision_status`, do not reword any decision item beyond the already-applied type spelling, and confirm the prototype claim by reading `prototypes/serial-sum-run/src/proof.rs` at the tip rather than trusting this ticket — if `close-the-serial-sum-run-gpu-family-probe-table` has landed by then, "a prototype still does" is itself false and the sentence needs a different repair, which is the one thing here that must not be applied mechanically.
