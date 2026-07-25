---
id: disambiguate-operation-names-shared-across-expression-layers
title: Disambiguate the operation names shared across expression layers
status: in-progress
priority: p2
dependencies: []
related: [disambiguate-select-across-ir-layers, disambiguate-contraction-in-the-glossary, own-operation-family-support-matrix]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, glossary, coherence]
claimed_from: todo
assignee: agent-foundation2
lease_expires_at: 1784997854
---
`Select` was not the only name denoting several unrelated constructs. It was the worst instance, and closing it in `disambiguate-select-across-ir-layers` required enumerating the operation vocabulary of every expression layer, which surfaced the rest of the class. This ticket carries that enumeration so the sweep is not repeated from nothing.

**Fact — the four vocabularies, read at `43f685f`.**

- *Tensor / semantic.* The adopted [operation conformance matrix](../docs/research/numerics/operation-conformance-matrix.md) rows, plus the four keys the standard registry actually registers (`crates/tiler-ir/src/semantic/registry.rs`).
- *Shape expression.* The accepted decisions in [shape environment contract](../docs/research/shapes/shape-environment-contract.md): `Not`, `Equal`, `Select`, `Add`, `All`, `Any`, `Min`, `Max`, `ExactDiv`, `FloorDiv`, `CeilDiv`, `Compare`.
- *ABI expression.* `crates/tiler-ir/src/program/abi.rs`: `ExprNode::{Root, Unary, Binary, Select}`; `AbiUnaryOp::{Not, NarrowU16, NarrowU32}`; `AbiBinaryOp::{CheckedAdd, CheckedSubtract, CheckedMultiply, Minimum, Maximum, FloorDivide, CeilingDivide, ExactDivide, IsMultipleOf, Equal, LessOrEqual, And, Or}`.
- *Structured kernel IR.* The proposed bounded set in [structured kernel IR and verifier boundary](../docs/research/kernel-ir/structured-kernel-ir-verifier.md) — `Constant Builtin Unary Binary Convert Bitcast CheckedNarrow Select For If Yield Load Store AtomicUpdate Barrier Collective` — against the implemented vocabulary in `crates/tiler-ir/src/kernel/model.rs`, whose `BinaryOp` is only `IndexAdd`, `IndexMultiply`, `IndexDivide`, `IndexModulo`, `F32Add`, `F32Multiply`.

**Fact — the exact-spelling collisions that remain after `Select` is closed.**

| Name | Sense 1 | Sense 2 | Hazard |
|---|---|---|---|
| `Minimum` / `Maximum` | Tensor family: NaN-propagating, deterministic `-0.0 < +0.0` | `AbiBinaryOp::Minimum` / `::Maximum`, documented "Unsigned minimum"/"Unsigned maximum" — a domain with no NaN and no signed zero | Highest. Identical to the `Select` failure: `grep -rnw Minimum crates/` finds an implemented `Minimum` and reads as evidence the tensor family is supported. |
| `Equal` | Shape predicate over `ShapeExpr` | `AbiBinaryOp::Equal`, "Unsigned equality" | Low semantically; still two newtyped domain IRs by the accepted decision on domain-specific expression IRs. |
| `Not` | Shape predicate, stated unary in the solver-exchange decision | `AbiUnaryOp::Not`, "Predicate negation" | Low. |
| `Constant` | Registered `tiler::constant-f32@1` | Proposed structured-kernel operation | Moderate: one is a governed key, the other is unimplemented. |
| `Unary` / `Binary` | `ExprNode::Unary` / `::Binary` | Proposed structured-kernel operations | Moderate. |
| `Add` | Tensor `tiler::add-f32@1`, IEEE rounding contract | Shape `Add`, mathematical-integer semantics that explicitly do not wrap, saturate, or overflow | Moderate. |
| `F32Add` / `F32Multiply` | The public semantic authoring facades over `tiler::add-f32@1` and `tiler::multiply-f32@1` (`crates/tiler-ir/src/semantic/standard_operations.rs:133` and `:51`, re-exported from `crates/tiler-ir/src/semantic.rs:53`) | `BinaryOp::F32Add` / `::F32Multiply`, structured-kernel binary operations (`crates/tiler-ir/src/kernel/model.rs:198` and `:200`), consumed by the Metal emitter | High, and unlike the rest of this table both senses are implemented, public, and live in the same crate. One is how a caller authors a semantic graph; the other is an operation inside a lowered kernel body. Found while closing `reconcile-illustrative-operation-names-with-governed-keys`. |

**Inference — a spelling convention already exists implicitly, and is the reason this list is not longer.** The ABI language spells its addition `CheckedAdd` where the tensor family is `Add`; the shape language spells its extrema `Min`/`Max` where the tensor family is `Minimum`/`Maximum`; the structured-kernel research spells its narrowing `CheckedNarrow`. Each avoids a collision that the obvious name would have produced. Nothing records that this is deliberate, so it is applied unevenly — `AbiBinaryOp::Minimum` took the tensor spelling while `AbiBinaryOp::CheckedAdd` did not, and `Select` took the same spelling in all four layers.

**What closes this.** Decide and record one convention, then apply it. The two coherent options are (a) qualify in the glossary, as `disambiguate-select-across-ir-layers` and `disambiguate-contraction-in-the-glossary` both did, which costs nothing at a call site but relies on a reader looking a term up; or (b) make the spelling itself carry the layer, which is what the implicit convention above already half-does, and which would make `AbiBinaryOp::Minimum` an unsigned-specific spelling. Option (b) touches `implementation/ir`, `implementation/artifact`, and `implementation/compiler`, none of which `contracts/foundation` holds, so choosing it means splitting the rename per crate scope rather than doing it here. Do not adopt both for the same name.

**Do not treat this as a rename ticket by default.** Every one of these names is the correct name for its construct within its own layer, and two of them are fixed by accepted decisions. The defect is that the layer is unrecoverable from the name and undocumented anywhere, not that any individual name is wrong.
