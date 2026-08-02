---
id: seat-the-concatenate-family-in-the-compiler-capability-table
title: Seat the concatenate family in the compiler capability table and rebaseline the explain digest
status: todo
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family]
related: [admit-an-additive-extent-relation, admit-the-reindex-and-broadcast-operation-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, operation-families, identity]
---
## User-visible outcome

The workspace gate is green with `tiler::concatenate-f32@1` registered. Today it is not: registering the key in `tiler-ir`'s standard semantic registry fails two `tiler-compiler` tests that no edit inside `crates/tiler-ir` or `crates/tiler-reference` can satisfy, so [`admit-the-sequence-extension-concatenate-family`](admit-the-sequence-extension-concatenate-family.md)'s branch cannot merge alone.

## Why this is a separate ticket

`crates/tiler-compiler/**` maps to `implementation/compiler`, which [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md) held live while the concatenate family landed. The concatenate worker verified file-level disjointness against that worker's actual branch diff — empty at the time — and still stopped, for two reasons its brief named: the sibling is actively working the compiler crate so the disjointness snapshot is guaranteed to move, and the second edit below is a *pinned identity rebaseline*, which is exactly the case where two branches can each be green and still not compose.

## Observed evidence, at commit `3226d68` on `tkt/admit-the-sequence-extension-concatenate-family`

`make full` reached `nextest` with `cargo fmt --all --check`, `cargo check --workspace --all-targets --locked`, and `cargo clippy --workspace --all-targets --locked -- -D warnings` all green, then reported **2363 tests run: 2361 passed, 2 failed**. Both failures are in `tiler-compiler` and nothing else in the workspace moved. The release-profile run reproduces both: **828 tests run: 826 passed, 2 failed**. `cargo test --workspace --doc`, `make doc`, `tkt lint`, and `shellcheck --severity style deps.sh` are all green on the same commit.

### 1. `policy::tests::the_capability_table_names_exactly_the_admitted_operations`

`crates/tiler-compiler/src/policy.rs:778-792` compares `operation_capabilities()` against `FrozenSemanticRegistry::standard()`'s keys in **both** directions, minus `UNPLANNED_OPERATIONS`. The observed diff is exactly one key: the registry side contains `tiler::concatenate-f32@1` and the table side does not.

The test's own doc states the intent: "a newly registered operation still has to be added to the capability table or listed there deliberately". So the decision is which of the two, and it is not free. A capability row enters each dimension it lists into `is_consumable`'s union, which decides whether a *contract* may permit that dimension at all. The concatenation performs no arithmetic and consumes no numerical freedom, so the row is `OperationNumericalCapability { key: "tiler::concatenate-f32@1", consumes: &[] }` — the same shape `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` carry at `policy.rs:418-425` — and that derivation should be stated in the diff rather than assumed. `UNPLANNED_OPERATIONS` is the wrong home: its stated justification is BF16, whose keys nothing downstream can realize, and it is guarded by `every_unplanned_operation_is_registered_and_consumes_no_dimension`.

### 2. `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`

`crates/tiler-compiler/src/explain.rs:4008` pins `"tiler-explain-v7 request=b81673209f732002"`. The request subject covers the frozen semantic registry snapshot, which encodes every registered operation with its schema, facts, and conformance identity, so admitting one further family must move it — which the ledger comment at `explain.rs:3855-3874` already states in those words for the contraction and for the reindex/broadcast pair.

**Observed on the branch:** `b81673209f732002` becomes `a7e2965962778aef`. The trace's own two record lines are unchanged, so nothing about explain's content moved.

Recompute this on the tree the change actually lands into rather than copying the value above: it is reported here as evidence that exactly one digest moves and that the movement is confined to the request qualifier, not as a value to paste. A second branch that also widens the registry, the target declaration, or the lowering registry moves the same digest, and two such branches can both be green and still not compose.

## Required behaviour

- Add the capability row, with the `consumes: &[]` derivation stated, or justify `UNPLANNED_OPERATIONS` against its own doc.
- Rebaseline the pinned digest **from an observed run on the merged tree**, and add a `// Rebaselined from ...` paragraph to the ledger in `explain.rs` in the same commit, following the convention of the fifteen entries above it.
- Decide, and state, whether `tiler::concatenate-f32@1` gets a `FusionOperationRole` in `crates/tiler-compiler/src/fusion_legality.rs`. Nothing forces one — a family without a role resolves to no fusion legality, which `fusion_legality.rs:300-303` documents — so *no role* is a legitimate outcome, but it is the difference between the row sitting at R4 and at R5 in the operation-family support matrix, and it must be a decision rather than an omission.

## Non-goals

No index-access lowering capability in `governed.rs`, no structured-kernel construct, no backend emission. Those are R6 work and the physical realization of a concatenation is still open — a join along an inner axis has no contiguous byte window, which is an applicability predicate over a physical candidate rather than part of this key's identity.

## Closes when

`make full` is green on a tree carrying `tiler::concatenate-f32@1`; the capability decision and the fusion-role decision are each stated in the diff with their derivation; and the moved digest is enumerated by name with its before and after in the landing report.
