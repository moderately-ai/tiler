---
id: admit-a-storage-carrier-for-integer-program-inputs
title: Admit a storage carrier for integer program inputs
status: todo
priority: p1
dependencies: [admit-an-indirect-gather-family-for-tied-embedding-lookup, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, enumerate-the-mature-tensor-dtype-taxonomy, route-an-embedded-artifact-through-a-consumer-storage-seam]
scopes: [implementation/ir, implementation/artifact, implementation/frontend, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, abi, frontend, gather, language-model]
---
## User-visible outcome

A `[T]` token-ID operand reaches a program as an integer, so the one operation between a model's inputs and its logits is fed by a value whose type says what it is.

## Why this exists

**Fact.** The semantic layer already registers integer identities: `crates/tiler-ir/src/semantic/catalog.rs` registers `tiler::u8@1`, `tiler::u16@1`, `tiler::u32@1`, `tiler::i32@1`, and `tiler::i64@1` under ADR 0028, so a `[T]` index operand at `tiler::u32@1` has an admitted identity today.

**Fact.** The runtime-value boundary has no carrier for one. `StorageScalar` at `crates/tiler-ir/src/program/model.rs:264` has exactly two variants, `U8` and `F32`. Reproduce with `grep -n "pub enum StorageScalar" -A 6 crates/tiler-ir/src/program/model.rs`.

**Inference.** The pinned workload's vocabulary is 151,936, which needs eighteen bits, so `U8` cannot carry a token ID. `F32` represents every integer below 2^24 exactly and would therefore work, which is why this is a decision rather than a gap.

## The decision, stated for Tom

Two readings, and [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) recommends the first.

- **Widen the carrier.** A third `StorageScalar` variant at 32 unsigned bits. It costs an artifact-ABI change: `StorageScalar::tag` participates in canonical encoding, and `natural_access_type` maps into the structured-kernel vocabulary, so both move. It buys an index operand whose type refuses a non-integer at the bind boundary.
- **Carry token IDs as `F32`.** No ABI change. It buys a float-to-integer conversion contract under [ADR 0041](../docs/decisions/0041-separate-float-to-integer-conversion-families.md) at the one operation in the program whose out-of-range behaviour reads out of bounds, and it makes `BindError::StorageScalarMismatch` unable to distinguish an index tensor from an activation.

This is a public boundary either way, so it is Tom's.

## Closes when

The carrier question is answered with its consequence stated, the answer is implemented, an index operand's stored type is checked at the bind boundary, and a value of the wrong stored type refuses by name rather than being reinterpreted.
