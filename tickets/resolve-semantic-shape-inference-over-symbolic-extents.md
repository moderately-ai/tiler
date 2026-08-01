---
id: resolve-semantic-shape-inference-over-symbolic-extents
title: Resolve semantic shape inference over symbolic extents
status: todo
priority: p1
dependencies: [carry-a-sourced-shape-on-semantic-values]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, semantic-graph]
---
## User-visible outcome

The registry decides whether two symbolic operands have one shape by asking the environment, so `f32[n] * f32[n]` is admitted for the right reason and `f32[n] * f32[m]` is refused with a typed reason naming both extents.

## Why this exists

**Fact.** The governed profile's elementwise rule is "operand shapes must match or one operand must be scalar" — the exact sentence the frontend quotes back at `crates/tiler-macros/src/region.rs:249`. With fixed extents, "match" is `==` on `u64`. With symbols it cannot be, because two occurrences of one symbol are equal and two different symbols are not provably anything.

**Fact.** `ExtentSources::proves_equal` is the accepted answer and it is deliberately one-sided: "`true` is a proof of equality; `false` means *not proved*, never *proved different*." It reaches `true` by an equality class or by a common determined value, and `ShapeEnv::proves_equal` is reflexive because `same_class` compares union-find roots.

## Implementation keys

- Route symbolic operand comparison through `ExtentSources::proves_equal` and nothing else. Do not add a syntactic symbol-identity shortcut beside it: the environment is the authority, and a second one would disagree the first time a constraint forces two differently spelled symbols together.
- A not-proved pair is a refusal, never a deferral and never a widening. Add a typed `BuildError` variant distinct from the existing shape mismatch, naming both extents and the environment, because a caller acts differently on "these are different sizes" and "this environment does not prove they are the same".
- A rank mismatch stays a rank mismatch. Do not let the symbolic path report it under the new variant.
- Scalar broadcast is decided on rank, not on extents, so it is unchanged; state that explicitly rather than leaving a reader to infer that the rule was reviewed.
- Result shape derivation must produce a `SourcedShape` that names the operand's symbol rather than a fresh one, so the result and its operands share an equality class by construction.

## Evidence

- `f32[n] * f32[n]` admitted, with the proof route asserted rather than only the outcome.
- `f32[n] * f32[m]` refused under an environment with no relation; the same pair admitted once the environment states `m == n`, so the acceptance is evidence about the environment rather than about the spelling.
- `f32[n] * f32[4]` refused when `n` is merely bounded and admitted when the environment determines `n == 4`.
- Each new check perturbed once and observed failing.

## Public boundary

The new `BuildError` variant and its rendered text.
