---
id: lower-the-concatenate-occurrence-through-partitioned-writes
title: Lower the concatenate occurrence through partitioned writes
status: todo
priority: p1
dependencies: [admit-a-partitioned-write-ownership-contract, admit-sub-range-write-domains-for-unequal-partitions]
related: [scope-the-concatenate-fusion-role-and-lowering, lower-a-two-region-occurrence-through-one-index-access-capability, admit-the-structural-families-into-the-scheduled-region-vocabulary, reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, lowering, indexing, concatenate]
---
## User-visible outcome

A program containing `tiler::concatenate-f32@1` resolves an index-access lowering capability and emits a verified region, so the family stops being a registered identity no plan can consume.

## Why this exists

**Fact — the family has no lowering and no realization law.** `governed_index_access_capabilities` (`crates/tiler-compiler/src/governed.rs:222-334`) registers nine capabilities and none covers a concatenation, so `resolve_index_access` (`crates/tiler-compiler/src/capability.rs:1115-1144`) fails with `MissingCapability`. The semantic registry's index-realization law table (`crates/tiler-ir/src/semantic/registry.rs:2387-2437`) registers twelve laws and none is a concatenation, so refinement fails with `MissingRealizationLaw`. (Nine and `2386-2420` until `admit-a-bf16-index-realization-law-and-refinement-contract` added the three `bf16` rows; what this fact claims — that no registered row is a concatenation — is unchanged by that step.)

**Fact — the fork is decided.** [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) eliminated the piecewise read and selected the partitioned write. The piecewise read is insufficient rather than merely expensive: the case selects a different operand *tensor* per coordinate, which `AccessData`'s single `tensor` field does not express and ADR 0046's map-level piecewise reservation does not reserve, and the read-both-and-select spelling is refused by the bounds proof and needs a predicate dtype `RQ-OP-03` owns. Q-SHAPE-006 therefore does not fire on this family.

**Fact — the coordinate arithmetic already exists.** Operand *k*'s write coordinate on the concatenated axis is `t + offset_k` for a literal `offset_k`, and `IndexNode::LinearCombination` (`crates/tiler-ir/src/index/model.rs:97-100`) carries a literal exact-integer constant. The expression stays `Affine`; nothing widens the coordinate-expression language.

**Fact — the arity forces seven registrations.** `resolve_index_access` keys on the exact `(family, operation, signature)` triple and `LoweringSignature` carries the exact operand and result type lists, while the family admits two through eight operands (`crates/tiler-ir/src/semantic/concatenate.rs:67`, `:79`). Each admitted arity needs its own registered capability, exactly as `MAX_CONCATENATE_OPERANDS`'s own doc comment explains for the reference provider.

## What the work is

Register the index-access capabilities and the matching `IndexRealizationLaw` variant, so the compiler-side emission and the semantic-side law produce the identical region and the refinement comparison is meaningful rather than one-sided.

Emit one write root per operand over the single output, each total over its own contiguous partition of the concatenated axis, with the read being the identity over that operand. The `emitted` scalar-operation list is deliberately empty for the same reason the reindex row's is (`governed.rs:293-298`): a concatenation applies no scalar operation, so declaring one would make refinement's containment check pass over an operation the region never emits.

Decide, and record, whether the region carries one iteration domain partitioned by coordinate or several — the dependency's contract fixes which of these is admitted, and this ticket must not invent a second answer.

Cover the zero-extent operand explicitly. `concatenate_result_shape` admits an operand empty on the concatenated axis and it contributes no coordinate, which is the pinned prefill occurrence (`[8, 0, 128]` joined with `[8, T, 128]`), so the partition set must handle an empty partition without that being a coverage hole.

Confirm whether the pinned explain digest moves, and if it does, execute the identity step completely.

## Explicit non-goals

- The fusion role, which is [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) and is independent of this chain.
- The request-boundary spelling that would let a program containing a concatenate be recognized at all. `request.rs`'s recognizer admits three elementwise keys, a reduction, and a contraction, and the structural families are refused under `operation-set` — that wall is [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md)'s and this ticket does not move it.
- The copy-free windowed realization. That is an M6/M7 physical candidate composing with this lowering, not part of it.
- Any second semantic family for an inner-axis concatenate. The record checked and confirmed the region is axis-uniform; the contiguous-window difference lives in the storage half.

## Closes when

Seven capabilities and one realization law are registered, a concatenate occurrence at each admitted arity emits a region that verifies and refines against the law, the zero-extent operand case is exercised, and a deliberate perturbation of one partition's offset is shown to fail the ownership proof.

## Graph maintenance

- `implementation/ir` is declared alongside `implementation/compiler` because the `IndexRealizationLaw` variant and its registration in the semantic registry live in `crates/tiler-ir/`, and a compiler-side emission with no matching law would make refinement fail closed on every occurrence it lowered.
- Depends on [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) because there is no proof form for the region this ticket emits until that contract exists.
