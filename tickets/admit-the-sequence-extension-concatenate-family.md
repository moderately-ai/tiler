---
id: admit-the-sequence-extension-concatenate-family
title: Admit the sequence-extension concatenate family
status: in-progress
priority: p1
dependencies: [scope-the-sequence-extending-tensor-family]
related: [design-autoregressive-state-and-kv-cache, admit-an-additive-extent-relation, bind-the-kv-cache-through-the-artifact-and-runtime-interface, admit-the-reindex-and-broadcast-operation-families]
scopes: [implementation/ir, implementation/reference, contracts/foundation]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, semantics, operation-families, kv-cache, language-model]
claimed_from: todo
assignee: agent-concat
lease_expires_at: 1785697181
---
## User-visible outcome

Extending a tensor along one axis has a governed meaning: a program can state a concatenation, it verifies, and the reference evaluates it — so the KV append is a value the compiler can see rather than a host convention nothing checks.

Register one general `Concatenate` key in `tiler-ir`'s standard semantic registry with a canonical axis attribute, a bounded variadic operand arity, and one result, plus a reference evaluator for the exact signature. The disposition this implements — *one general family rather than a narrow sequence-extend key* — is [the sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md)'s, and it is a research disposition rather than an accepted interface: **the exact public boundary is Tom's.**

## Required behaviour

- Operands must agree on rank, on resolved value type, and on every axis except the concatenated one; each disagreement refuses at construction naming the axis and both observed extents, through the accepted three-outcome shape path rather than a shape comparison invented here.
- The family grants no dtype promotion, no weak-scalar rule, and no numerical permission. It is bit-preserving: the evaluator must not apply arithmetic NaN canonicalization, exactly as the structural families' evaluators do not.
- **A zero-extent operand behaves as the normative definition states, explicitly.** [Rung L5's state contract](../docs/research/runtime/autoregressive-state-and-kv-cache.md) makes prefill an occurrence with `C = 0`, so this case is reached by the pinned workload rather than being hypothetical, and inheriting whatever the empty case happens to do is the defect.
- The result extent is the sum of the operands' extents on the concatenated axis. Until [`admit-an-additive-extent-relation`](admit-an-additive-extent-relation.md) lands, an occurrence whose result extent cannot be related to its operands' refuses rather than binding a fresh unconstrained symbol; that refusal is the whole of the family's extent handling in the interim and it must be written, not assumed.
- The axis attribute is validated at construction. `StrictSerialSumF32::infer` is the precedent.

## Non-goals

No lowering, no fusion role, no structured-kernel construct, no `Slice`. A concatenation along an inner axis losing the contiguous-window realization is an applicability predicate over a physical candidate and belongs nowhere near this key.

## Closes when

The key is registered with a schema, an inference, and a normative reference; the reference provider evaluates it; every refusal above has a test that fails without it; and the support matrix's sequence-extension row is updated from evidence.
