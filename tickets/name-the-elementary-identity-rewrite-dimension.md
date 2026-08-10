---
id: name-the-elementary-identity-rewrite-dimension
title: Name the elementary-identity rewrite dimension
status: done
priority: p2
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, carry-the-elementary-identity-dimension-adr, decide-whether-to-admit-an-elementary-identity-permission, correct-the-online-single-pass-softmax-fold-legality-fact, measure-whether-the-metal-runtime-compiler-folds-an-elementary-identity]
scopes: [research/numerics, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, optimizer]
---
## User-visible outcome

A rewrite that uses a transcendental's functional equation — `exp(a) * exp(b) = exp(a + b)`, `log(a*b) = log(a) + log(b)`, `sqrt(a)*sqrt(b) = sqrt(a*b)` — is refused with a reason naming the freedom it consumed, rather than being refused for the nearest dimension that happens to be defined or, worse, admitted because no dimension noticed.

## Why this exists

**Fact.** [Numerical semantics](../docs/numerical-semantics.md) governs the order contract — reassociation, operand permutation, contraction — and [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) adds distributivity. Every one of those dimensions is about **how contributors are combined**.

**Inference, from [the certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 2.** The online-softmax rescaling rewrite is equal to the two-pass fold only by telescoping `exp(x_j - m_j) * exp(m_j - m_V) = exp(x_j - m_V)`. That step is a functional equation of the elementary function rather than an algebraic identity of the ring, and **in floating point it is false**: `fl(exp(a)) * fl(exp(b))` and `fl(exp(a+b))` differ, by an amount governed by the target's `eps_exp` and by the argument magnitudes. No order-contract dimension is about this, because it is not about combining contributors at all — it rewrites *through* the function.

**This is a gap in the dimension set rather than a missing permission inside one**, which is why it is filed separately from [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md). Both gates must open before the softmax rewrite is legal, and they are independent.

## What the research must decide or defer

- **Whether this is one dimension or a family.** The exponential's, the logarithm's, and the square root's identities differ in their error behaviour: `exp`'s telescoping is exact over the reals and its floating-point error is governed by argument magnitudes, while `log`'s product rule carries a cancellation hazard the exponential's does not. ADR 0014's standard applies — a split needs evidence of a capability asymmetry, not an intuition of one.
- **Whether the dimension is per-operation or global**, since accuracy contracts are already per-operation and per-target under [ADR 0042](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md).
- **How the dimension composes with the accuracy obligation the target already answers.** `assess_program_elementary_accuracy` asks whether a target realizes an operation's accuracy contract; rewriting through the identity changes *which* evaluations occur, so the obligation set changes with the rewrite. Whether that is already handled by `readmit_candidate` asking again per candidate, or needs more, is the concrete question.
- **What a refusal says.** ADR 0095 established that a rewrite consuming distributivity must reject naming the missing dimension rather than reporting a forbidden reassociation. The same discipline applies here and is the minimum outcome even if no permission is admitted.

## Non-goals

Admitting a permission (Tom's, under the same boundary as ADR 0095); implementing anything in `crates/`; enumerating every elementary function's identities exhaustively before the first one has a caller.

## Closes when

The dimension is defined in a research record with its worked counterexample, its relationship to the order contract and to the accuracy contract is stated, and either a permission is proposed for Tom or the dimension is named-and-unpermissioned in the shape ADR 0080 used for distributivity. Either outcome must leave the refusal wording specified.

## Scope added by the worker, and why

**`contracts/navigation`, added 2026-08-05.** The deliverable is a research record plus a preserved experiment, and `ticketsplease.toml` routes both catalogs that must move with them — `docs/research/README.md` and `spikes/README.md` — to `contracts/navigation` rather than to `research/numerics`, which covers `docs/research/numerics/**` and `spikes/numerics/**` only. AGENTS.md requires a catalog to be edited in the same change as the metadata behind it, and no gate checks the corpus, so leaving the two rows for a later ticket would leave a record and a spike unreachable from the indexes that are the only way a reader finds them. The mapping was read from `ticketsplease.toml` rather than asserted, and the two edits are one line each. This is declaration and scheduling metadata rather than product-scope expansion; nothing decision- or contract-shaped is applied, and `contracts/decisions` and `contracts/numerics` were deliberately **not** added — the vocabulary proposal is drafted inside the record for [`carry-the-elementary-identity-dimension-adr`](carry-the-elementary-identity-dimension-adr.md) to transfer.

## Outcome

**The dimension is one, it is a fourth, and the correct disposition today is named-and-unpermissioned.** [The elementary-identity rewrite dimension](../docs/research/numerics/elementary-identity-rewrite-dimension.md) carries the derivation: the elimination over all twelve dimensions, the grain argument on ADR 0014's capability-asymmetry standard, the three-layer answer that puts per-function content in the operation capability and per-identity quantitative content in the rule's bound, the accuracy-composition analysis, the vocabulary cost, and the drafted verbatim-landable ADR body.

**Two measurements were taken rather than assumed**, both preserved at [`spikes/numerics/elementary_identity_folding/`](../spikes/numerics/elementary_identity_folding/README.md). The rewrite is observable in binary32 under a *correctly rounded* exponential — 502 of 1681 non-positive integer argument pairs disagree, the smallest at `a = b = -1.0` where the product rounds to `0x3e0a9556` and `exp(-2.0)` is `0x3e0a9555` — which is what proves the accuracy machinery structurally unable to bound it. And the offline Metal compiler folds no elementary identity under any of six math-mode flag sets, which is the evidence a target-profile declaration would rest on if the dimension were admitted; the runtime half is `Unknown` and filed.

**Four tickets file what the derivation makes actionable**, and one finding is a defect rather than a proposal: at the time this research closed, `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` registered the string `a-reassociation-of-the-sum-and-not-a-free-implementation-choice`, which the certified-bounds derivation refutes and which points a scheduler at exactly the wrong permission. Correcting it is an identity-domain step, so it is [`correct-the-online-single-pass-softmax-fold-legality-fact`](correct-the-online-single-pass-softmax-fold-legality-fact.md) rather than a change here.

**Correction — 2026-08-10.** That correcting ticket is `status: done`. The registered value is now `not-a-reassociation-of-the-sum-but-a-horner-nesting-consuming-distributivity-which-no-permission-grants-and-the-subordinate-exponentials-elementary-function-identity-which-no-declared-dimension-names-so-no-reassociation-or-permutation-permission-reaches-it` (string concatenation at the `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` registration in `crates/tiler-ir/src/semantic/softmax.rs`). The old string above is historical finding evidence only — the state this ticket was made against — not a live claim. Reproduce: `rg -n 'SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM' -A6 crates/tiler-ir/src/semantic/softmax.rs`.
