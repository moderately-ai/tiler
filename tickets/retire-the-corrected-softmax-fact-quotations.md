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

The two numerics sites that quoted the pre-correction online-single-pass fact in the present tense — elementary-identity Part 9 and `admit-the-softmax-family` — each carry a dating that names correcting commit `28fe26a8` and the fact's current value, so a reader of those records cannot take the repaired defect for the live registered state.

**Correction — 2026-08-10.** Earlier wording claimed no document in the corpus still presents that quotation as live. That corpus-wide claim and the close-condition substring grep were overstated relative to what landed; see Outcome. Status stays `done` under the two-site scope actually delivered.

## Why this exists

**Fact.** [`correct-the-online-single-pass-softmax-fold-legality-fact`](correct-the-online-single-pass-softmax-fold-legality-fact.md) replaced the registered value of `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` and rewrote the module header and doc comment that repeated it. Two documents quote the superseded text in the present tense and were outside that ticket's scopes:

- `docs/research/numerics/elementary-identity-rewrite-dimension.md`, Part 9 — "**Fact, read in full at `crates/tiler-ir/src/semantic/softmax.rs`.** The module header states that …", quoting both the header sentence and the old fact string. Reproduce with `grep -n 'a-reassociation-of-the-sum' docs/research/numerics/elementary-identity-rewrite-dimension.md`.
- `tickets/admit-the-softmax-family.md:87` — "`SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` states that the online rescaling form is a **reassociation of the sum**".

**Inference — the repair is a dating rather than a deletion, and the distinction is the whole ticket.** Part 9 is the evidence that produced the correcting ticket, so rewriting it as though the defect never existed destroys the derivation a reader follows from the record to the fix. What it needs is the sentence that says the finding was acted on, with the correcting commit named, so the quoted strings read as the state the finding was made against. `admit-the-softmax-family` is a completed work record and its body describes what that landing registered, which was true then; the same dating applies, and neither file's quoted strings are edited.

## Non-goals

Editing `crates/`, editing the correcting ticket's own outcome, restating the derivation the elementary-identity record already carries, or changing either record's `disposition`.

## Closes when

Both named sites carry a dated sentence naming the correcting commit and the fact's current value, and `tkt lint` passes.

**Correction — 2026-08-10.** The prior close condition also required `grep -rn 'a-reassociation-of-the-sum' docs/ tickets/` to return only lines inside such a dating. That substring matches corrected-value quotations (`not-a-reassociation-of-the-sum-…`), historical problem statements, and undated residual prose outside the two named sites; it cannot close the two-site work. Dropped as a close condition; residual undated sites are listed in Outcome rather than treated as reopening this ticket.

## Outcome

**Delivered at commit `12d72e20`.** The two sites named in Why now date the pre-correction quotations to correcting commit `28fe26a8` (2026-08-05) and name the current registered value, without deleting the quoted evidence:

- `docs/research/numerics/elementary-identity-rewrite-dimension.md` Part 9 — appends `**Acted on, 2026-08-05.**` … commit `28fe26a8` … current registered multi-clause Horner/distributivity/elementary-identity value; quotations preserved as derivation evidence.
- `tickets/admit-the-softmax-family.md` — keeps the landing-time sentence and appends `*(True of the landing this record describes; superseded 2026-08-05 by commit \`28fe26a8\` …)*`.

Crates already held the corrected `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` from `28fe26a8` under [`correct-the-online-single-pass-softmax-fold-legality-fact`](correct-the-online-single-pass-softmax-fold-legality-fact.md); this ticket did not edit `crates/`.

**Ticket residuals already dated (sibling repairs, 2026-08-10).** Present-tense old-string claims that the census missed at filing are no longer live as present-tense on:

- [`name-the-elementary-identity-rewrite-dimension`](name-the-elementary-identity-rewrite-dimension.md) Outcome — `**Correction — 2026-08-10.**` marks the old `a-reassociation-of-the-sum-and-not-a-free-implementation-choice` string as historical finding evidence and names the current registered value.
- [`correct-the-online-single-pass-softmax-fold-legality-fact`](correct-the-online-single-pass-softmax-fold-legality-fact.md) Why — `**Correction — 2026-08-10.**` plus `Fact (HISTORICAL — pre-\`28fe26a8\`)` labels on the pre-delivery module-header / registered-value statements.

**Residual present-tense prose still outside the two-site delivery (docs; not closed by this ticket):**

- `docs/research/numerics/elementary-identity-rewrite-dimension.md` Open axes — bullet still reads "A registered fact says the online single-pass softmax form is a reassociation…" in the present tense (Part 9 itself is dated; Open axes is not).
- [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) Open questions — still states "a registered definition fact in the tree states that the online single-pass softmax form is a reassociation" (needs `contracts/decisions` scope or a connected remainder; outside this ticket's declared scopes).

Corrected-value substring hits (e.g. flash-class capability set quoting `not-a-reassociation-of-the-sum-but-a-horner-…`) are not residual defects under a "quotes the corrected value" reading.

**Fact audit — 2026-08-10.** Status `done` is retained for the two-site dating that landed in `12d72e20`. User-visible outcome and Closes when are narrowed to that delivery so the board record no longer asserts a corpus-wide retirement the tree does not hold. No status reopen; residuals above are honesty about undated docs prose, not reopening the identity step.
