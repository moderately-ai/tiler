---
id: extend-the-selected-quantized-profile-to-the-tied-embedding-matrix
title: Extend the selected quantized profile to the tied embedding matrix
status: deferred
priority: p3
dependencies: [implement-first-quantized-backend-profile, admit-an-indirect-gather-family-for-tied-embedding-lookup, reclassify-language-model-work-as-a-conformance-track]
related: [scope-first-quantized-lm-profile, design-autoregressive-state-and-kv-cache]
scopes: [implementation/compiler, implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, language-model, deferred]
---
## Activation boundary

Do not start this before both dependencies deliver. It needs the selected quantized profile working over the weighted projections, and it needs a gather family that can consume a compound value — neither exists.

## User-visible outcome

The workload's single largest weight tensor, the `[151936, 1024]` matrix that `tie_word_embeddings: true` makes serve both the input lookup and the vocabulary projection, joins the selected quantized profile. It is the difference between quantizing 74% of the model's parameters and quantizing all of them.

## Why it is separate, and why the case for it is already measured

**Measurement — the memory case, from [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md).** Extending the selected per-output-channel strict-affine U8 profile from the 196 weighted projections to the tied embedding as well moves the model's weight bytes from 1,064,714,240 (0.447 of the F32 baseline) to 598,726,528 (0.251), with identical measured behaviour on the C1 conformance row — 17 of 18 greedy agreement and the exact 18-token sequence in both variants. There is no accuracy argument against it on the measured row.

**Fact — why it is nonetheless not in the first vertical.** The matrix has two uses with different access relations. As the vocabulary projection's operand it is contraction index structure 1, which the selected profile covers. As the input embedding it is a gather, which the profile does not cover and which Tiler cannot express: [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) owns that family. A per-row scale — one per vocabulary token — is the natural granularity for both uses, which is a convenience of this profile and not a general fact.

**Inference — one plan must not allocate two copies.** The record's workload authority already states that one matrix serves both uses and that "a plan that allocates two copies doubles the largest single allocation in the model for no semantic reason". Quantizing it does not change that, and a quantized copy plus an `f32` copy would be worse than either alone.

## Implementation keys

- One logical compound value, shared by the gather and the contraction, with each use deriving its own access from the same parameter map. Two independently quantized copies is a defect, not a simplification.
- The gather returns a decoded row. Where that decode happens — inside the gather's access, or as a separate stage — is a physical choice that must be stated and costed, not assumed.
- Re-measure the C1 observable rather than inheriting the record's reading: the record measured a *simulated* profile in a reference implementation, not Tiler's own execution.

## Closes when

The tied matrix is one quantized logical value consumed by both its gather and its contraction, no plan allocates two copies, the measured C1 observable is reproduced through Tiler's own execution, and one `make full` passes.

## Graph maintenance

Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md). Advance no ledger cell that this ticket does not itself test.
