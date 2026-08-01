---
id: project-only-the-final-position-logits
title: Project only the final position's logits
status: todo
priority: p2
dependencies: [assemble-the-embedding-and-vocabulary-projection-programs, admit-a-position-selecting-slice-for-the-rotary-table]
related: [design-model-ingestion-and-complete-execution, own-operation-family-support-matrix, design-model-level-qualification-and-optimization]
scopes: [implementation/ir, implementation/reference, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, slice, logits, residency, language-model]
---
## User-visible outcome

A prefill pass that only needs its last token's logits produces `[1, 151936]` instead of `[T, 151936]`, which at the benchmark row's long end removes more peak residency than the model's entire weight set occupies.

## Why this exists

**Fact.** [L1](../docs/research/program-planning/first-metal-lm-workload.md) records that the pinned reference's `logits_to_keep=0` becomes `slice(0, None)` — the value that reads like "keep none" is the special case that keeps all — which is why the conformance row can retain every prefill logit.

**Inference, from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).** The logits contract therefore has two modes and they are not interchangeable. The conformance row needs every position; a benchmark row needs only the last. At B1-d prefill the difference is **4,978,634,752 bytes against 607,744** — a saving of **4,978,027,008 bytes (4.6361 GiB)**, larger than the 2,384,199,680-byte F32 weight budget. Under the D-B decomposition it moves the model-level prefill peak from 10,895,486,984 B to 5,917,459,976 B.

**This fires the sub-tensor-selection row's *first* trigger**, the one [the support matrix](../docs/roadmap.md#operation-family-support-matrix) already states as "a prefill pass that needs only the final position's logits", and it fires it on residency rather than on convenience. The row's third trigger, the position-identity one, is [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md)'s and is the family this depends on.

## Required content

- Selecting row `T − 1` of a `[T, 1024]` residual stream is injective and not surjective, so it is outside `tiler::reindex-f32@1`'s admitted forms and outside `Broadcast`; the family that admits it is the dependency, and this ticket does not invent a second one.
- The two modes are two program shapes, not a runtime choice: a program that could return either would be two programs presented as one.
- The conformance mode stays, unchanged and default, because the oracle needs it.

## Closes when

A prefill program of the final-position shape verifies, reference-evaluates against the pinned reference's own last-position logits, and the two modes are distinguishable by their declared output shape rather than by a flag.
