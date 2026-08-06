---
id: decide-how-a-pinned-pointwise-grouping-becomes-evaluable
title: Decide how a pinned pointwise grouping becomes evaluable
status: in-progress
priority: p2
dependencies: []
related: [enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, derive-the-oracle-for-a-permitted-divergence-candidate]
scopes: []
shared_scopes: []
paths: []
tags: [tiler-research, numerics, reference, conformance]
claimed_from: todo
assignee: agent-grouping-fork
lease_expires_at: 1786031369
---
## User-visible outcome

A decision, with the elimination stated so a reader can refute it, on which of two live designs makes a reassociated pointwise chain checkable — so the largest non-reduction freedom site stops being unevaluable.

## Why this exists

**Fact — the grouping is already pinned, and the gap is on the other side.** [The freedom-sites enumeration](../docs/research/reference/plan-freedom-sites.md) Part 3.3 and Part 4 establish that `ScalarProgram::PointwiseF32(expression)` pins the emitted grouping exactly: `PointwiseF32Node::Add { lhs, rhs }` (`crates/tiler-ir/src/schedule/pointwise.rs:91`) is a binary node over dense topological ordinals, and `mint_elementwise` (`crates/tiler-compiler/src/request.rs:4697`) mints it as a faithful image of the possibly-reassociated semantic DAG. So the site is a witness. It is unevaluable.

**Fact — two designs are live and neither is correctness-dominant, which is why this is a decision and not research.**

1. **Retain the selected semantic candidate's program.** The reference already evaluates a `SemanticProgram` exactly (`ReferenceEvaluator::evaluate`, `crates/tiler-reference/src/evaluate.rs:178`). The rewritten program is discarded: it lives only in `SemanticCandidate.proposal` (`crates/tiler-compiler/src/pipeline.rs:328-333`), a private pipeline struct, and the retained `ProgramAlternative` (`pipeline.rs:247-271`) has no semantic-program field. Exact check: `grep -rn "pub fn .*SemanticProgram" crates/tiler-compiler/src/` returns nothing. Retaining it costs memory and a public accessor and needs no new evaluator.
2. **Write an exact evaluator for `PointwiseF32Expression`.** Costs a second evaluator and a new dependency edge — `grep -rn "ScalarProgram\|ReductionTopology\|VerifiedScheduledRegion\|PointwiseF32" crates/tiler-reference/` returns one line today, a doc comment in a test — but answers for the *physical* projection rather than for a semantic form the projection is trusted to mirror, which is the stronger claim.

**Inference — the difference is which artifact the oracle is evidence about**, and that is a genuine priority split rather than a cost trade: design 1 checks that the device matches the semantic program the compiler chose, design 2 checks that it matches the expression the compiler emitted. A projection defect is invisible to the first and caught by the second.

## What this ticket must produce

The elimination run explicitly against correctness, performance, and long-term maintainability, with the surviving design named; or, if both survive, one atomic question for Tom with a small worked tensor program, point, counterpoint, and a recommendation.

## Explicit non-goals

Implementing either design; accepting the witness surface, which is `accept-the-realization-witness-surface`.

## Closes when

One design survives with the derivation stated, or the fork reaches Tom as one atomic question.

## Graph maintenance

Filed by the freedom-sites enumeration, whose Part 7.3 drafts both and declines to pick.
