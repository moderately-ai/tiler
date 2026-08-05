---
id: bound-the-certified-exponential-s-cost-in-its-admitted-argument-region
title: Bound the certified exponential's cost in its admitted argument region
status: in-progress
priority: p2
dependencies: []
related: [cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock]
scopes: [implementation/reference]
shared_scopes: []
paths: []
tags: [numerics, performance]
claimed_from: todo
assignee: agent-exp-bound
lease_expires_at: 1785960717
---
## The finding

**Measurement — Apple M4 Max, dev profile, one `exp_enclosure(2^b, EnclosurePrecision::binary32_corpus())` per row, 2026-08-05.**

| Argument | Time |
| --- | --- |
| `2^0` | 0.33 ms |
| `2^5` | 0.38 ms |
| `2^10` | 0.71 ms |
| `2^13` | 4.1 ms |
| `2^16` | 157 ms |
| `2^20` | 37.5 s |
| `2^22` | over 9 minutes, not run to completion |

**Fact — the governed bound bounds the wrong quantity.** `MAX_ARGUMENT_HALVINGS` in `crates/tiler-reference/src/accuracy.rs` refuses an argument whose binade exceeds 22, and `EnclosureError::ArgumentTooLarge` is that refusal. It bounds the *halving count*, which is the loop trip count, and says nothing about the magnitude of the result those halvings then square up to: `exp(2^22)` needs a numerator of roughly `2^22 * log2(e)` — about six million bits — and every squaring multiplies and normalizes numbers of that size. So an argument the module admits can cost minutes, and the admitted region has no stated cost bound at all.

**Inference — this is a fail-closed gap, not a performance nit.** Everything else in this crate bounds its work before doing it: `MAX_REFERENCE_TENSOR_ELEMENTS`, `MAX_REFERENCE_ELEMENT_BYTES`, `MAX_SERIES_TERMS`, and the evaluator's own `iteration_step_allowance` all refuse with a typed error naming the limit and the observed value. The enclosure's argument bound is the one that admits a case it cannot cost. A caller that reaches it does not get a refusal it can explain; it gets a process that appears to hang.

**Fact — no caller in the tree reaches it today.** `certified_exp_f32` guards at `+89` and `-104` before calling `exp_enclosure`, so the binary32 SiLU and softmax references never present an argument above binade 7. The exposure is `exp_enclosure`'s own public boundary, which is re-exported from `crates/tiler-reference/src/lib.rs` and takes any `ExactRational`.

**Fact — this is pre-existing and not introduced by the reduction-depth change.** `cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock` restated `MAX_ARGUMENT_HALVINGS` as `23 + REDUCED_ARGUMENT_BITS` specifically so that the admitted binade stayed at 22 either way; the table above is a property of the result's magnitude, which that change does not move.

## What to decide

The question is which bound the module should state, and it is a public-boundary question rather than an implementation detail:

1. **Bound the result's magnitude.** Refuse when `argument * log2(e)` exceeds a governed bit width, with a typed error naming the width and the observed one. This is the shape the rest of the crate uses, and it makes the admitted region cost-bounded rather than merely trip-count-bounded.
2. **Bound the total work.** Carry a step or magnitude allowance the way `ReferenceEvaluator::with_iteration_step_allowance` does, so a caller can authorize a large enclosure deliberately and a caller that did not is refused.
3. **Leave it, and say so.** Record that the admitted region above some binade is unbounded in cost, with the reason and the argument that no caller can reach it. This is only tenable if the public boundary is narrowed so that no caller *can*.

Option 3 conflicts with `exp_enclosure` being public and general; options 1 and 2 change what the function refuses, which is a semantic change to a governed refusal and needs its diagnostic code decided rather than invented.

## Closes when

The admitted region has a stated cost bound or a recorded reason for not having one; any new refusal carries a typed error with a stable diagnostic code and a test that watches it refuse *and* watches the admitted neighbour, on the pattern `an_over_large_argument_is_refused` now uses; and the measurement table above is either reproduced against the change or superseded by it.
