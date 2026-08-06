---
id: correct-the-l6-budget-refusal-item
title: Correct the L6 refusal list's budget item against check_program_budgets
status: todo
priority: p2
dependencies: []
related: [widen-the-deterministic-budgets-to-the-decoder-layer-program, design-model-ingestion-and-complete-execution, assemble-the-decoder-layer-program]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, correction, language-model, budgets]
---
## User-visible outcome

Item 1 of [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md)'s "what refuses today, with exact numbers" list quotes the code that exists, so a reader deriving the decoder layer's remaining blockers from it counts the resources that actually refused and reads the derivations the compiler actually applies.

## Why this exists

**Fact — observed while landing [`widen-the-deterministic-budgets-to-the-decoder-layer-program`](widen-the-deterministic-budgets-to-the-decoder-layer-program.md).** Item 1 states that `verify_program` checks `buffers` against `4.max(program.input_count() + 1)`, lists `host_expression_nodes` `32` among the bounds P2 clears, and concludes that P2 "exceeds three of the four". None of the three survives a read of `check_program_budgets`:

- `buffers` is checked against `program.input_count().saturating_add(3)`, not `4.max(input_count + 1)`. For P2's eighteen inputs that is 21, not 19.
- `host_expression_nodes` is checked against `program.input_count().saturating_mul(2).saturating_add(7)` — 43 for eighteen inputs, against a bound the record lists as 32. So it refused P2 too.
- P2 therefore exceeded **four** resources, not three, and the fourth is one the record's own numbers present as passing.

The check is `sed -n '/fn check_program_budgets/,/^}/p' crates/tiler-compiler/src/request.rs`, read in full rather than grepped.

**Fact.** The listed values are stale independently of the derivations. `regions` is `3`, and the four program-scoped bounds are now `semantic_values` 80, `semantic_operations` 62, `host_expression_nodes` 43, and `buffers` 21 — widened once, sized to this record's own P2, under Tom's D-18 answer.

**Inference.** This is a stale-citation defect rather than a wrong finding: the refusal item 1 reports was real, its owner was correctly named, and only its arithmetic and its count are wrong. Correcting it in place preserves the derivation the record was making.

## Required content

- Item 1 restated against the current `check_program_budgets`: both derived actuals with their exact expressions, the four resources P2 exceeded, and the governed values as they now stand.
- Refusal 1 recorded as **discharged**, naming the ticket and commit that discharged it, so the "honest ordering" paragraph below the list — which reads "refusals 1 and 3 are widenings with owners" — no longer counts it among the open ones.
- The statement that a budget widening does not admit a transformer is kept as-is and is now load-bearing: item 2, the recognizer refusal, is untouched and is what still refuses P2.

## Do not

Do not restate the measured counts from a second source. [`assemble-the-decoder-layer-program`](assemble-the-decoder-layer-program.md) measured them (18 inputs; 58 occurrences over 76 values at C1 prefill, 62 over 80 at C1 decode) and this record's derived floors were already superseded by that outcome; cite it rather than re-deriving.

## Closes when

Item 1 quotes the code that exists, the four-versus-three count is corrected, refusal 1 is marked discharged with its commit, and the ordering paragraph beneath the list agrees with the corrected item.
