---
id: correct-the-online-single-pass-softmax-fold-legality-fact
title: Correct the online single-pass softmax fold-legality fact
status: todo
priority: p2
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity, correctness]
---
## User-visible outcome

`tiler::softmax-f32@1`'s registered facts stop asserting that the online single-pass form is a reassociation, so a scheduler reading them cannot consume the reassociation permission and believe the rewrite legal.

## Why this exists

**Fact, read in full at `crates/tiler-ir/src/semantic/softmax.rs`.** The module header states: "The online single-pass form is a reassociation, which is a legality question and not a cost one. Rescaling a running sum whenever the maximum changes regroups the contributor sequence of the *sum*, so it is legal exactly where reassociation is granted." `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM`'s doc comment repeats it, and the registered fact value is the string `a-reassociation-of-the-sum-and-not-a-free-implementation-choice`.

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md), which landed after that text was written, derives the opposite in its Part 2: the online fold is a Horner nesting rather than a re-parenthesized sum. Unrolling it gives contributors `exp(x_j - m_j) * prod_{k>j} exp(m_{k-1} - m_k)`, which share no floating-point value with the two-pass fold's `exp(x_j - m_V)`, so no reassociation permission and no permutation permission reaches the rewrite. It consumes distributivity — for which [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) declines a permission — and the elementary-function identity [the elementary-identity dimension record](../docs/research/numerics/elementary-identity-rewrite-dimension.md) names.

**Inference — the error runs in the dangerous direction, which is why this is p2 rather than tidying.** The doc comment's own stated purpose is that the fact exists "so that a scheduler reaching for it has to consume the permission". A scheduler that reads it consumes *reassociation* and believes itself legal under a registered contract that permits reassociation — which is exactly the false inference [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) item 5 exists to prevent, present in the tree, in identity-carrying data. Today nothing reads the fact, so nothing is currently wrong on any execution path; what is wrong is a claim the next reader will act on.

## This is an identity-domain step and is executed completely or not at all

**Fact.** `encode_operation_definition` (`crates/tiler-ir/src/semantic/registry.rs:2811`) writes `definition.canonical_facts().value()` into the definition projection, currently `tiler.semantic-definition-projection.v5`. That projection feeds the registry snapshot identity, which the compiler's explain request qualifier pins — [Numerical semantics](../docs/numerical-semantics.md) records a prior fact-record change advancing "the definition projection to v5, the registry snapshot to v7, and the standard semantic provider to revision 7".

So the change moves a pinned identity. Per AGENTS.md the whole step lands in one commit: the fact string, the version at its owning layer if the encoding changes, the ledger documents, and every pinned identity recomputed on the tree the step lands into with each moved pin enumerated in the report. **A changed fact value with unmoved pins is worse than no change**, because it is a stepped meaning under an unstepped identity.

Whether a *value* change inside an existing record shape advances the projection version at all, or only the resulting digests, is the first thing to establish by reading rather than to assume: a version counts rendering revisions, and this is not one.

## What the corrected claim says

The fact should state what the fold actually consumes rather than deleting the warning, because the warning's purpose — that a scheduler reaching for the form must not treat it as free — is correct and only its *reason* is wrong. The replacement names the distributivity dimension and the elementary-function identity, states that no reassociation or permutation permission reaches the rewrite, and stays a fact string rather than acquiring a structured vocabulary the tree does not have. The module header's paragraph and the constant's doc comment move with it, and the wording follows the refusal discipline the dimension record's Part 7 specifies: a rewrite consuming more than one missing dimension names all of them.

## Non-goals

Admitting any permission; adding an elementary-identity dimension to any type; implementing the online fold; changing `docs/numerical-semantics.md`, which is `contracts/numerics`.

## Closes when

The registered fact, its doc comment, and the module header agree with the certified-bounds derivation; every pinned identity the change moves is recomputed in the same commit and enumerated; and the retained softmax tests still assert the wall they assert today.
