---
id: decide-the-index-region-oracle-route-past-its-step-budget
title: Decide the index-region oracle's route past its evaluation-step budget
status: todo
priority: p2
dependencies: []
related: [bound-the-reference-contraction-comparison-for-the-profile-cells, route-the-contraction-conformance-through-the-staged-oracle]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, contraction, language-model]
---
## User-visible outcome

The emitted index region can be executed against a profile cell larger than `w_decode_kv`, or the reach of the index-region oracle is a stated design limit with its cost — so "the emitted region reproduces a measured device result" stops being a claim about one cell by accident.

## Why the contraction staging did not settle this

**Fact — `bound-the-reference-contraction-comparison-for-the-profile-cells` (2026-08-01).** That ticket staged the *contraction evaluator*'s fold and reached all six L3 cells. It did **not** touch the index-region oracle, and the two bounds are different things:

- `MAX_REFERENCE_TENSOR_ELEMENTS` bounds `output_count * contracted_count`, a product computable from the shapes before a single step. Staging it is arithmetic: partition the output, and each slab's product is known in advance.
- `MAX_EVALUATION_STEPS` (`crates/tiler-reference/src/oracle.rs:49`) is a *running counter*, incremented by `step()` on every scalar evaluation, reducer-body evaluation, and index-expression evaluation in one region evaluation. It is not a product of extents and cannot be predicted from the region's shape without modelling the region's own body.

So `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` in `crates/tiler-compiler/src/governed/contraction_conformance.rs` still refuses `w_vocab_slice`'s 8,388,608 contracted points, and every larger cell with it.

## What this must decide, with the elimination stated

Three candidates, and the cheap one is not obviously wrong here:

1. **Partition the region evaluation over its output domain**, the way the contraction staging partitions the fold — but a `VerifiedIndexRegion`'s output domain is a property of the region rather than of a declared index structure, so this needs a public notion of "evaluate this region for this sub-domain" and an argument that the sub-domain does not change any value. That argument is *not* the contraction's: a region may carry reductions whose contributor sequences are region-declared, so the independence has to be re-derived from what the region verifier guarantees.
2. **Carry the counter across calls** so a caller pays in bounded instalments without the oracle needing a domain partition — cheaper, but it turns a per-evaluation budget into caller-held state, and a budget a caller can reset is not a budget.
3. **State the reach as a limit.** The index-region oracle exists to be a second, independent implementation; if the cost of making it reach large cells is a weaker bound or a second independence argument, keeping its reach small and saying so may be the correct answer. If chosen, the refusal's meaning must be documented where the oracle lives rather than only in a consumer's test.

Whichever survives, the measured cost at `w_vocab_slice` and at the largest prefill cell is part of the answer — the contraction evaluator's own six-cell profile costs 10.8 s in the dev profile, and the region oracle is the slower of the two implementations.

## Closes when

The index-region oracle either reaches a stated set of profile cells with its independence argument written where the procedure lives, or its reach is a documented design limit with the cost that decided it — and the consumer test that asserts the refusal cites the decision rather than an open question.
