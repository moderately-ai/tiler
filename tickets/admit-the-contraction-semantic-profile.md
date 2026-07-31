---
id: admit-the-contraction-semantic-profile
title: Admit the contraction semantic profile for the workload's projection structure
status: todo
priority: p1
dependencies: [spike-first-metal-contraction-vertical]
related: [decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys, scope-einsum-contraction-support, admit-the-reindex-and-broadcast-operation-families, own-operation-family-support-matrix]
scopes: [implementation/ir, implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, contraction, matmul, language-model, identity]
---
## User-visible outcome

A program can state that a `[T, 1024]` activation contracts against a `[3072, 1024]` weight over the last axis of both — 197 of the pinned workload's 253 contraction occurrences — and the compiler either accepts the structure or refuses it under a named rule. Today no contraction can be stated at all.

## What ADR 0087 already settled, and what this ticket owes

**Fact.** [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) accepts one keyed family whose node carries a renaming-invariant canonical index structure, with five structural rules rejecting at construction under their own names and an unsupported structure failing closed at capability resolution. It authorizes the *shape* of that admission and not its start; this ticket is the start, bounded to one index structure.

**Proposal — the profile, from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md).** `td,od->to` over `[M, K] x [N, K] -> [M, N]`, F32 operands, accumulator, and result. The canonical encoding by first appearance is operand 0 = `(0, 1)`, operand 1 = `(2, 1)`, output = `(0, 2)`, contracted = `{1}`. **Note that this is not `[M, K] x [K, N]`**: the checkpoint stores every projection weight `[out_features, in_features]`, so the contracted index is the last axis of both operands, and a key shaped for the ordinary matmul would require a transposing `Reindex` on all 197 occurrences.

## Required delivery

- **The key and its inference routine.** One governed `OpKey` in the standard registry, with an `infer` that rejects each of the five structural rules separately — an output index in no operand; a summed index in one operand; an index repeated within one operand; a duplicated output index or an output order that is not a permutation of the free indices; an index in more than two operands — as typed provider diagnostics naming which rule, at construction, never as a generic invalidity.
- **The canonical encoding, with its mutation proof.** ADR 0087 item 1 requires the encoder to be domain-separated, exhaustively encoded, and *mutation-proved*: a perturbation that makes two distinct structures encode equally, or one structure encode two ways, must be demonstrated failing before the encoder is trusted. `ab,cb->ac` and `td,od->to` must produce identical bytes; `td,do->to` must not. The `tiler::strict-serial-sum-f32@1` reduced-axis attribute in `crates/tiler-ir/src/semantic/registry.rs` is the precedent for where the normalization lives.
- **Extent agreement through the accepted path.** The shared `K` resolves through the [shape environment contract](../docs/research/shapes/shape-environment-contract.md)'s three-outcome path, retaining both bindings so a failure names both observed sources.
- **The access relation.** Operand 0's map is `(t, o, d) -> (t, d)` and operand 1's is `(t, o, d) -> (o, d)`. Both are pure projections needing no index arithmetic, so no new access class — but this is the first family whose two operand maps project away *different* iteration coordinates, and the emission must be exercised rather than assumed.
- **The numerical signature.** Computation/input precision, accumulator dtype, result dtype, conversion behaviour, and the order contract, parameterized by the structure per ADR 0087 item 5. The L3 record's table gives every value for this profile. A contraction admitted with only an operand dtype and a result dtype is underspecified against ADR 0009.
- **The support-matrix row**, moved in the same change, with the honest rung.

## Non-goals

Structures 2 and 3 (the attention score and value contractions), any multi-operand form, any lowering capability, any schedule, and any backend. The multi-operand question stays reserved under Q-SEM-015 and rule five is where its future answer lands.

## Closes when

The structure is statable, all five refusals fire under their own names with a test that was watched failing, the canonical encoder's mutation proof exists and was demonstrated, and a contraction occurrence still fails closed at lowering-capability resolution because no capability covers it yet.
