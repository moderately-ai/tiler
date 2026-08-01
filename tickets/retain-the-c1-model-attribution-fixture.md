---
id: retain-the-c1-model-attribution-fixture
title: Retain the C1 model attribution fixture
status: in-progress
priority: p1
dependencies: [retain-the-qwen-conformance-reference-logit-fixture]
related: [design-model-ingestion-and-complete-execution, define-first-metal-lm-workload, prove-the-c1-complete-model-execution, design-model-level-qualification-and-optimization]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, fixture, conformance, attribution, language-model]
claimed_from: todo
assignee: worker-c1-attribution
lease_expires_at: 1785573030
---
## User-visible outcome

A model-level disagreement can be attributed to one of thirty executions instead of to "the model", because the reference's own intermediate values are retained beside its logits.

## Why this exists

**Inference, from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).** [L1's oracle](../docs/research/program-planning/first-metal-lm-workload.md) compares five observables at the model boundary, and that is the pass or fail. It cannot say *where*: a forward pass is 30 executions over ten operation families and four host computations, and the model boundary has no execution ordinal. Conformance evidence and attribution answer different questions and the existing fixture carries only the first.

## Required content

Extending the retained C1 fixture, under the same retention policy [L1](../docs/research/program-planning/first-metal-lm-workload.md) already fixes — full bytes regenerable, digests and comparison values checked in.

- **Per-layer hidden states.** The reference emits them directly. Across the 18 C1 positions that is `28 × 18 × 1024 × 4 = 2,064,384` bytes.
- **Per-layer post-RoPE `K` and `V`.** `28 × 2 × 8 × 18 × 128 × 4 = 4,128,768` bytes — the same figure as L1's 18-position cache budget, because they are the same tensors.
- **The four host computations that joined the comparison surface.** The rotary `cos` and `sin` rows, whose construction L2 moved out of the executed program; the additive causal mask, whose two values are `0xff7fffff` and `0x80000000`; the token IDs, already retained; and one digest over the widened F32 weights.
- Per-tensor digests plus the full-precision values a bounded comparison needs, on the same footing L1 chose for the logits: a digest proves the reference regenerates exactly and cannot support a bounded-error comparison, so both are retained.

## Do not

Do not derive or state any tolerance. [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) owns the bound, and L1 already fixes that composing one from per-operation tolerances is the defect rather than the method. This ticket produces the surface, not the budget.

## Closes when

The fixture regenerates from the pinned checkpoint and the pinned reference sources, the retained record carries every item above with its digest, and the producer stops rather than warns on a manifest mismatch exactly as the existing one does.
