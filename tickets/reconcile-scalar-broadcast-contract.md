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

Amend docs/ir.md to admit rank-0 scalar operands as explicit, documented operation semantics of these binary signatures: a narrow carve-out recorded at the verifier invariant and in the Reindex/Broadcast discussion, keeping the explicit `Broadcast` requirement for all non-scalar broadcasting. Reconcile the adjacent "homogeneous by default" wording in docs/numerical-semantics.md if the amendment touches it, and cross-reference ADR 0061 as the accepted basis for the scalar facades. No code changes; canonical program identities are unaffected. The outcome is contract text under which the implemented inference is authorized rather than contradicted.
