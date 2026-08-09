---
id: determine-whether-compiler-lowering-mints-duplicate-pointwise-constants
title: Determine whether compiler lowering mints duplicate pointwise constants
status: todo
priority: p2
dependencies: []
related: [share-identical-constants-in-the-pointwise-expression-canonical-form]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, identity, research]
---
## Why this evidence is separate

**Fact — current builder spelling.** `PointwiseF32ExpressionBuilder::constant` and `PointwiseBf16ExpressionBuilder::constant` each append a draft constant unconditionally. Their `canonicalize_nodes` walks preserve every reachable draft constant rather than hash-consing equal payloads. The schedule witness test `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` therefore proves that the public builders can spell two distinct canonical expressions for one binary32 function.

**Fact — current compiler minting seam.** Compiler lowering does not call those builders directly from semantic operations. `mint_into` replays one `ElementwisePlan`, keeps a `Vec<(ValueId, S::Value)>`, and calls `sink.constant` once for every `ElementwiseMint::Constant` step. Repeated operands naming one semantic `ValueId` reuse the earlier minted value; two distinct semantic constant occurrences with equal bits would reach two constant steps if the semantic and planning layers preserve both occurrences.

**Unknown.** No current test proves whether two semantic programs computing the same function — one reusing a single constant occurrence and one carrying two equal constant occurrences — can both reach `mint_elementwise` and produce distinct `ScalarProgram::PointwiseF32` or `ScalarProgram::PointwiseBf16` payloads. That is a source-and-experiment question, not an identity-policy decision.

## Work

1. Read the semantic constant construction, elementwise planning, `mint_into`, both pointwise builders, and the schedule-witness test in full.
2. Build two semantic programs computing the same function that differ only in whether one exact constant occurrence is reused or two distinct equal-payload occurrences are used.
3. Drive both through the compiler recognition/minting path and compare the resulting scalar programs. Cover `f32`; cover `bf16` if the compiler currently admits an equivalent reachable pair, otherwise record the exact refusal that makes that width unreachable.
4. Perturb the subject so the check demonstrates that it reaches constant-occurrence identity rather than merely comparing two fixtures. Quote the failure.
5. Record the exact source rule if either spelling is rejected or normalized before minting.

Do not change either builder, schedule identity, an identity domain, request/artifact/cache subjects, or Metal goldens here. Those consequences belong to the dependent decision ticket.

## Closes when

The compiler-reachability question is answered by a source-backed compiled pair or by a proved rejection/normalization rule, with a regression test that reaches the decisive seam and a ticket Outcome that states the exact `f32` and `bf16` boundary.
