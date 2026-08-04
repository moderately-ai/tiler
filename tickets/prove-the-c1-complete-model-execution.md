---
id: prove-the-c1-complete-model-execution
title: Prove the C1 complete-model execution
status: todo
priority: p1
dependencies: [drive-the-complete-forward-pass-over-three-artifacts, retain-the-c1-model-attribution-fixture, prove-the-c1-stateful-attention-vertical]
related: [design-model-ingestion-and-complete-execution, define-first-metal-lm-workload, design-model-level-qualification-and-optimization]
scopes: [implementation/runtime, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, proof, conformance, model, language-model]
---
## User-visible outcome

The complete pinned model runs the C1 conformance row on Metal and its logits are compared with the pinned reference at every one of the eighteen positions. **This is rung L6's user-visible outcome.**

## What must hold

From [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md)'s worked example and [L1's oracle](../docs/research/program-planning/first-metal-lm-workload.md).

- **The row.** The ten-token prompt `[785, 3974, 13876, 38835, 34208, 916, 279, 15678, 5562, 13]`, an eight-step budget, eighteen positions reached.
- **The measured sequence.** The ten prompt tokens followed by `576, 3974, 13876, 38835, 34208, 916, 279, 15678` — the base model restarts the pangram. Termination is the fixed budget; EOS 151643 never appears, so this row exercises the budget arm of the termination rule and leaves the EOS arm untested, and the run must say so rather than claim both.
- **The counts.** Nine forward passes of thirty executions is 270 executions, over **exactly three artifact identities**. A fourth is a failure regardless of the logits.
- **The five observables**, at every position: logit agreement under the level [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) owns, greedy-token equality, the declared tie handling, termination, and plan determinism over N repeated executions of the same artifacts with the same inputs on the same device.
- **The state closes against L1 and L5.** The final model KV's logical F32 tensor payload and bytes reached are `28 × 147,456 = 4,128,768`. The selected runtime pool owns two 147,456-byte buffers per logical K/V member at C1 capacity (112 buffers total); neither arithmetic figure is a resident-process measurement. The retained logits total `6,077,440 + 8 × 607,744 = 10,939,392` bytes.
- **One execution in nine routes differently.** `S = 16` at decode 6 is the only positive multiple of sixteen in `{10, …, 18}`, so 28 of the 270 executions select the tiled realization for the value contraction and the rest select direct, through the same artifacts and the same guards.

## Do not

Do not state a tolerance. The bound is L8's and is derived rather than chosen; this ticket compares under whatever level that rung has fixed by the time it runs, and if none exists it reports the deviation with its measurement boundary rather than inventing a threshold.

## Closes when

The row runs end to end, all five observables are evaluated at all eighteen positions, the artifact-identity and execution counts are asserted by a test that can fail, and the run's numerical claim names the operations it covers.
