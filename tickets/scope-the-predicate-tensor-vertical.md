---
id: scope-the-predicate-tensor-vertical
title: Scope the predicate tensor vertical
status: deferred
priority: p2
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, own-the-dtype-support-maturity-matrix, enumerate-the-mature-tensor-operation-and-signature-taxonomy, own-operation-family-support-matrix]
scopes: [research/numerics, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, predicate]
---
## User-visible outcome

A reader asking what a `bool` tensor would cost gets one statement covering the semantic operation, the value type, the physical carrier width, the ABI, runtime validation, and target dispatchability together, instead of six partial answers that each assume the others.

## Why this is deferred rather than open

**Fact.** [The dtype support ledger](../docs/dtype-support.md) records `tiler::bool@1` registered as two-valued with deliberately no logical width, `tiler::i1@1` carrying no authority, and no registered operation admitting a `bool` operand at any arity. `KernelType::Bool` and `AbiType::Boolean` are control predicates and are not tensor values.

**Fact — the trigger has not fired, and the elimination is recorded rather than assumed.** [The first attention program vertical](../docs/research/program-planning/first-attention-program-vertical.md) binds a host-built **additive** causal mask as an `f32` input of extent `[T, S]`. The live workload therefore reaches masking without a predicate tensor, so the ledger's `### Logical bool` trigger — a named `Select`, comparison, logical reduction, or frontend workload — is genuinely unmet rather than merely unclaimed.

**Fact — the operation axis states the same gate and forbids closing either side alone.** [The mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-03` gates F-13 comparison, F-14 logical operations, F-16 classification predicates, F-17 elementwise selection, F-28's logical-reduction case, and F-36's mask case, and states that this question and the ledger's `bool` trigger "must close together or neither has". That record calls the group "the single highest-leverage unblocking decision in the inventory".

## Activation trigger

A named workload requires a `Select`, a comparison, a logical reduction, or a boolean mask **as a tensor value**. An additive or multiplicative float mask does not fire it. A control predicate inside a kernel does not fire it.

## What the work would be, when it starts

State the vertical as one thing, in the order the dtype ledger's thirteen rungs impose: the exact semantic operation and its registered key; the value type and its two-valued cardinality; the physical carrier width, which is the decision the whole family turns on and which `AbiType::Boolean` does not answer; the ABI and what the carrier contributes to program identity; runtime semantic validation; and per-`(target family, dtype)` dispatchability. Close `RQ-OP-03` in the same pass or record why it cannot be.

## Closes when

The trigger has fired and the six obligations above are stated together with `RQ-OP-03`'s answer, **or** the trigger is superseded by a decision that removes predicate tensors from the intended product surface.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-1 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- `research/semantic-graph` is declared because the predicate value-kind half of the statement belongs on the operation axis, where `RQ-OP-03` lives; `research/numerics` alone could not carry it.
- Do not move any [dtype support ledger](../docs/dtype-support.md) cell from this ticket. The ledger owns delivered state and a scoping statement delivers nothing.

## Trigger check log

- 2026-08-04 — **not fired.** Track D-1's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md):133. Verified independently against the tree: the only `Bool` in the semantic layer is `CanonicalValueKind::Bool` / `CanonicalValueData::Bool` (`crates/tiler-ir/src/semantic/operation.rs:369-370`, `crates/tiler-ir/src/semantic/types.rs:717,743`), which is the **attribute** vocabulary, not a tensor element type — so no registered operation admits a `bool` operand at any arity and the additive-mask selection still stands. `RQ-OP-03` must close with it. Recheck: `grep -n 'Bool' crates/tiler-ir/src/semantic/types.rs`.
- 2026-08-09 — **not fired.** `tiler::bool@1` remains a recognized catalog identity with no registered operation accepting a predicate tensor; kernel `Bool` values remain control predicates. The selected attention path still binds an additive F32 mask, and no Select/comparison/logical-reduction/boolean-mask tensor workload has appeared.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `grep -n 'Bool' crates/tiler-ir/src/semantic/types.rs`, and run at this base it returns **10** lines. A result other than the 10 recorded here is the changed answer. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
