---
id: correct-the-softmax-divergence-attribution-in-code
title: Correct the softmax divergence attribution in code and the matrix row
status: done
priority: p1
dependencies: []
related: [admit-the-softmax-family, correct-the-softmax-worked-example-and-its-recorded-divergence, scope-transformer-nonlinear-normalization-and-reductions]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, softmax, correction, measurement, transformer]
claimed_from: todo
assignee: agent-softmax-attr
lease_expires_at: 1785893513
---
## User-visible outcome

A reader of `tiler::softmax-f32@1`'s module documentation or its support-matrix row is told what the reference model actually does at the widths where it differs — a different contributor order, not an approximate reciprocal — so the family's own order contract reads as the thing the measurement supports rather than as unrelated to it.

## Why this is filed

Filed 2026-08-01 by [`correct-the-softmax-worked-example-and-its-recorded-divergence`](correct-the-softmax-worked-example-and-its-recorded-divergence.md), whose scopes are `research/numerics` and `project/tickets` and which therefore could not touch the crates or the roadmap. No test fails: every asserted *bit* in the softmax corpus is correct and stays correct. What is wrong is prose, in four places, and a doc comment is a claim the next worker reads as fact.

**Measurement — the attribution is wrong, and the check is one line.** The reference's implied normalization constant at the L3′ worked example is `0x3f2a4d3a`, one ULP below the correctly rounded `1.0 / 0x3fc06957 = 0x3f2a4d3b`. It is **exactly** the correctly rounded reciprocal of `0x3fc06958`, which the row's own four exponentials reach under the contributor order `(e₀, e₂, e₁, e₃)`: `((e₀ + e₂) + e₁) + e₃` is `0x3fc06958` where the strict left fold is `0x3fc06957`. So no approximate reciprocal is required to explain the row. The retained probe carries both denominators and both reciprocals (`softmax_worked_example_denominator_under_order_0_2_1_3`, `softmax_worked_example_reciprocal_of_that_denominator`) in `spikes/numerics/transformer_reference_semantics/results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`.

**Measurement — and at scale, with its boundary.** Over 20,000 rows per width the reference's output row is exactly one scalar multiple of these exponentials at every element of every row, at all five widths. At width four, where summation orders are enumerable, 19,895 of 20,000 implied constants are the correctly rounded reciprocal of a denominator those exponentials reach under some strict left fold or the balanced tree. The enumeration is not every legal grouping, so that count is a lower bound on reachability: it eliminates the approximate-reciprocal hypothesis where it is high and does not establish it where it falls short. Widths eight and eighteen are not enumerable and stay open.

**Measurement — a second, independent error in the same landing's evidence.** `torch.max` over `[+0.0, -0.0]` is **`-0.0`** (`0x80000000`), not `+0.0`. It is an order *dependence*, not an ordering rule: `torch.max` returns the second operand and `torch.amax` the first, each reversing when the operands do, so neither implements the `-0.0 < +0.0` total ordering ADR 0023's Tiler families share. Recorded in the same probe run as `torch_max_of_signed_zeros_*` and `torch_amax_of_signed_zeros_*`. **Nothing in decision D-2 rests on it** — the three stated grounds are about NaN, and the signed-zero ordering is Tiler's own choice — so this is an evidence correction, not a decision reopening.

## Required delivery

Four prose corrections, no behaviour change, no bit changes:

- **`crates/tiler-reference/src/softmax.rs`** — the module section headed "Measurement — the reference model performs an approximation this form excludes". The *form* half is right and is now supported at every width rather than only the narrow ones; the *approximation* half is the defect. Restate it as a contributor-order difference and say why that is the stronger reading: it is the family's own `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` observed rather than a numerical defect, and it means matching the reference's bits would be reproducing an unpermitted reassociation rather than passing a check.
- **`crates/tiler-reference/src/softmax/tests.rs`** — the doc comment on `the_retained_worked_example_reproduces_the_pinned_formula`, which says the reference "applies an approximate reciprocal on rows of four or more contributors". Every assertion in the test body stays; only the attribution moves. Consider asserting the reordered denominator and its reciprocal directly, which would make the comment's claim executable rather than narrated.
- **`crates/tiler-ir/src/semantic/softmax.rs`** — two defects in the D-2 section. The signed-zero sentence is false as measured above. And the gap statement "the retained `record.tsv` contains no softmax row with a NaN score, so this was measured rather than read", with its `grep -i nan …/results/*/record.tsv` check, is now stale: the 2026-08-01 record carries `softmax_row_with_a_nan_score` and `torch_max_of_row_with_a_nan_score`, so the grep now hits them and the measurement is readable from the retained record. Point at the record instead of at a re-run.
- **`docs/roadmap.md`** — the `tiler::softmax-f32@1` row repeats "the divergence is in the *reciprocal*, not the exponential or the sum". The exponential and sum halves are right; the reciprocal attribution is not. The sum is precisely where it is, in the *order* of the sum rather than in its arithmetic.

## Non-goals

Changing `tiler::softmax-f32@1`'s pinned formula, its facts, its corpus bits, or decision D-2 — none of which this evidence disturbs. Reproducing the reference model's fold order: the pinned strict left fold stands, and a different order is reachable only under the separately resolved reassociation and permutation permissions. Recomputing the C1 attention-block fixture, which belongs to [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) as the model-level bound's owner.

## Closes when

All four sites describe the contributor-order attribution and its measurement boundary, the stale `grep` gap statement points at the retained record, the signed-zero claim matches what was measured in both operand orders, and `make full` is green on the result.

## Outcome

**Done.** All four sites now carry the contributor-order attribution with its boundary, and the worked example's test *executes* the attribution rather than narrating it. No bit asserted anywhere in the softmax corpus changed, no registered fact moved, and no public surface appeared: the two new constants are `#[cfg(test)]`-module items.

**What was wrong, and what the correct attribution is.** All four sites said the reference model applies an *approximate reciprocal* from width four upward. The reference's implied normalization constant at the L3′ worked example is `0x3f2a4d3a`, one ULP below the correctly rounded `1.0 / 0x3fc06957 = 0x3f2a4d3b` — but it is *exactly* the correctly rounded reciprocal of `0x3fc06958`, which the row's own exponentials `[0x3e0a9555, 0x3ebc5ab2, 0x3f800000, 0x00000000]` reach under the contributor order `(e₀, e₂, e₁, e₃)`. So no approximation is needed to explain the row: the exponentials agree bit for bit, and the reciprocal and the multiplies are each correctly rounded — only the denominator's contributor order differs. That is `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` observed rather than a numerical defect, and it inverts the consequence: matching the reference's bits at these widths would be performing an unpermitted reassociation, not passing a conformance check, so the reference model cannot settle the legality question at all.

**The two `crates/tiler-ir/src/semantic/softmax.rs` defects, corrected against the retained record.** The signed-zero sentence claimed `torch.max` over `[+0.0, -0.0]` is `+0.0` and therefore agrees with the `-0.0 < +0.0` ordering; the record's four rows measure `torch.max` = `0x80000000` and `torch.amax` = `0x00000000` on that pair, each reversing when the operands reverse, so each spelling returns a fixed *position* and neither implements the ordering. D-2's three grounds are about NaN, so nothing in the decision moves. The gap statement was doubly stale: the 2026-08-01 record carries `softmax_row_with_a_nan_score` and `torch_max_of_row_with_a_nan_score`, so the NaN measurement is now readable rather than re-run — and the old sentence's own claim that `grep -i nan …/results/*/record.tsv` hits `silu_inputs` and the SiLU rows is false on the current tree, where those two softmax rows are the *only* hits. The replacement points at the record rows by name rather than at a grep.

**Fault-proof, run rather than asserted.** Perturbing the new fold in `the_retained_worked_example_reproduces_the_pinned_formula` from `(e₀, e₂, e₁, e₃)` to the strict left order made it fail at the named assertion — `crates/tiler-reference/src/softmax/tests.rs:424`, `left: 1069574487` (`0x3fc06957`) against `right: 1069574488` (`0x3fc06958`) — and the file was restored and re-run green. So the executable attribution can say no.

**Measurement boundary, carried at every corrected site.** The single-constant result covers all five widths (20,000 rows each, every element); the *order* attribution is enumerated only at width four, where 19,895 of 20,000 constants are reachable under some strict left fold or the balanced tree. The enumeration is not every legal grouping, so that count is a lower bound; widths eight and eighteen are not enumerable and stay open.

**Commands.** `cargo fmt --check -p tiler-ir -p tiler-reference`; `cargo check --workspace`; `cargo clippy -p tiler-ir --all-targets -- -D warnings` and the same for `tiler-reference`; `cargo nextest run -p tiler-ir -p tiler-reference` (982 passed); `cargo test --workspace --doc`; `cargo nextest run --workspace` (2501 passed, 7 skipped — unchanged from the base gate, since the correction adds assertions to an existing test rather than a test).
