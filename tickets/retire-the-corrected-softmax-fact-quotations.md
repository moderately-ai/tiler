---
id: retire-the-corrected-softmax-fact-quotations
title: Retire the corrected softmax fact quotations in the numerics records
status: done
priority: p3
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, correct-the-online-single-pass-softmax-fold-legality-fact]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, documentation]
---
## User-visible outcome

No document in the corpus quotes `tiler::softmax-f32@1`'s online-single-pass fact as saying the rescaling fold is a reassociation, so a reader arriving at the numerics records cannot take a repaired defect for a live one.

## Why this exists

**Fact.** [`correct-the-online-single-pass-softmax-fold-legality-fact`](correct-the-online-single-pass-softmax-fold-legality-fact.md) replaced the registered value of `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` and rewrote the module header and doc comment that repeated it. Two documents quote the superseded text in the present tense and were outside that ticket's scopes:

- `docs/research/numerics/elementary-identity-rewrite-dimension.md`, Part 9 — "**Fact, read in full at `crates/tiler-ir/src/semantic/softmax.rs`.** The module header states that …", quoting both the header sentence and the old fact string. Reproduce with `grep -n 'a-reassociation-of-the-sum' docs/research/numerics/elementary-identity-rewrite-dimension.md`.
- `tickets/admit-the-softmax-family.md:87` — "`SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` states that the online rescaling form is a **reassociation of the sum**".

**Inference — the repair is a dating rather than a deletion, and the distinction is the whole ticket.** Part 9 is the evidence that produced the correcting ticket, so rewriting it as though the defect never existed destroys the derivation a reader follows from the record to the fix. What it needs is the sentence that says the finding was acted on, with the correcting commit named, so the quoted strings read as the state the finding was made against. `admit-the-softmax-family` is a completed work record and its body describes what that landing registered, which was true then; the same dating applies, and neither file's quoted strings are edited.

## Non-goals

Editing `crates/`, editing the correcting ticket's own outcome, restating the derivation the elementary-identity record already carries, or changing either record's `disposition`.

## Closes when

Both sites carry a dated sentence naming the correcting commit and the fact's current value, `grep -rn 'a-reassociation-of-the-sum' docs/ tickets/` returns only lines inside such a dating, and `tkt lint` passes.
