---
id: admit-the-sequence-extension-concatenate-family
title: Admit the sequence-extension concatenate family
status: done
priority: p1
dependencies: [scope-the-sequence-extending-tensor-family]
related: [design-autoregressive-state-and-kv-cache, admit-an-additive-extent-relation, admit-the-reindex-and-broadcast-operation-families, bind-repeated-invocations-over-caller-retained-tensors]
scopes: [implementation/ir, implementation/reference, contracts/foundation, implementation/compiler]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, semantics, operation-families, kv-cache, language-model]
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

## Outcome — 2026-08-02, completed in one branch

All of the above landed, plus the compiler seating the worker correctly stopped short of. The worker's own comment on this ticket is the delivery record; this section covers only what the coordinator did after it.

**Why the seating was finished here rather than dispatched.** Registering any key in `StandardSemantics` breaks two `crates/tiler-compiler` tests that no edit inside the original scopes could satisfy: `policy.rs`'s set equality between the numerical capability table and the registry's keys, and `explain.rs`'s request digest folding the registry snapshot. The worker filed `seat-the-concatenate-family-in-the-compiler-capability-table` and recommended instead adding `implementation/compiler` here and finishing in one branch, on the ground that the follow-up would deadlock — it depended on this ticket, which could not reach `done` while the gate was red. That was right, and it is what happened: the scope was added, the two edits made, and **`seat-the-concatenate-family-in-the-compiler-capability-table` was deleted rather than dispatched.** One gate, nothing red on `main`.

**`tiler::concatenate-f32@1` is listed in `UNPLANNED_OPERATIONS`, and for a stronger reason than the BF16 rows beside it.** BF16 is unplanned because no arithmetic in this build realizes it. Concatenate performs **no arithmetic at all**, so there is no numerical dimension a capability row could list — a row would be a claim about a target that concatenating elements never asks of one. The list's own invariant test was checked, not assumed: registered, rowless, resolving to no capability.

**The request digest was recomputed on the merged tree, not pasted.** The worker observed `b81673209f732002` → `a7e2965962778aef` on its branch and explicitly said the value must be recomputed rather than copied; the merged tree produced the same value from an observed run, and the site carries a note saying why it moved.

**Correction to the dispatch brief, which over-attributed one refusal to this ticket.** The brief framed the L5 stale-state case — an allocation valid over `[0, 13)` bound with `C = 14` — as refused by this family's interim behaviour. It is not, and the worker established why by reading: the semantic layer carries **only static extents** (`ShapeEnv`/`ShapeSymbol` appear zero times under `crates/tiler-ir/src/semantic/` and `crates/tiler-ir/src/program/`, while `Shape` appears in 20+ files there), so that case is unreachable from this family and belongs entirely to [`admit-an-additive-extent-relation`](admit-an-additive-extent-relation.md). What landed here is the static-extent half: derive the sum exactly, refuse when it leaves the domain rather than binding a plausible extent the operands do not determine. The graph edge was **not** inverted — only the brief's framing was loose.
