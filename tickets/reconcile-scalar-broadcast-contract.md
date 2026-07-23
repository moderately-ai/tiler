---
id: reconcile-scalar-broadcast-contract
title: Reconcile scalar broadcast with the semantic-graph contract
status: todo
priority: p1
dependencies: []
related: []
scopes: [contracts/foundation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, numerics]
---
Direction decided by Tom on 2026-07-23: amend the contract rather than the implementation. The built-in add-f32/multiply-f32 semantic signatures admit a rank-0 scalar operand that adopts the other operand's shape (BinaryF32 inference in crates/tiler-ir/src/semantic/registry.rs), while docs/ir.md's semantic-verifier invariant states "Binary operations use explicit broadcasting" with no carve-out; ADR 0061 describes the scalar facades without reconciling the contract, and no other ADR authorizes the implicit rule.

Amend docs/ir.md to admit rank-0 scalar operands as explicit, documented operation semantics of these binary signatures: a narrow carve-out recorded at the verifier invariant and in the Reindex/Broadcast discussion, keeping the explicit `Broadcast` requirement for all non-scalar broadcasting. Reconcile the adjacent "homogeneous by default" wording in docs/numerical-semantics.md if the amendment touches it, and cross-reference ADR 0061 as the accepted basis for the scalar facades. The cross-reference is written **into ir.md**; this ticket does not hold `contracts/decisions` and must not edit any ADR — if reconciliation appears to require an ADR edit, hand that part to `sweep-adr-contract-coherence-residuals`, which does hold that scope. No code changes; canonical program identities are unaffected. The outcome is contract text under which the implemented inference is authorized rather than contradicted.

## Outcome

`docs/ir.md` now authorizes the implemented inference. The Reindex/Broadcast discussion gains two paragraphs stating that a binary elementwise signature may declare that it accepts a rank-zero operand, that the operand contributes its single value at every output coordinate, that the result takes the other operand's shape, that `tiler::add-f32@1` and `tiler::multiply-f32@1` declare exactly that admission, and that no `Broadcast` node is synthesized so canonical identity is unchanged. The semantic-verifier invariant "Binary operations use explicit broadcasting" carries the same declared exception. Both sites keep the explicit `Broadcast` requirement for operands of nonzero rank, rank padding, extent-one stretching, and every other many-to-one mapping, and both state that a signature which does not declare the admission rejects a rank-zero operand as an ordinary shape disagreement. ADR 0061 is cross-referenced from ir.md as the accepted basis for the `F32Add`/`F32Multiply` facades, with ir.md retaining ownership of the admission itself.

`docs/numerical-semantics.md` was inspected and left unchanged. Its "Ordinary elementwise operations are homogeneous by default" wording is a dtype rule, while the carve-out is a rank/shape rule; the implemented `BinaryF32` inferencer still rejects any operand whose resolved value type is not `tiler::f32@1`, so no weak-scalar or promotion permission is created. The ir.md text states that boundary explicitly rather than duplicating a shape rule into the numerics contract. No ADR was edited and no code changed.
