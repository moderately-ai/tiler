---
id: retain-one-derived-proof-summary-per-shape-environment
title: Retain one derived proof summary per shape environment
status: todo
priority: p1
dependencies: [resolve-semantic-shape-inference-over-symbolic-extents]
related: [seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, narrow-symbolic-inference-and-restore-host-owned-refusals, replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, constraints, performance, correctness, identity]
---
## User-visible outcome

One verified symbolic environment is solved once, then every operation and broadcast axis asks the same immutable proof summary whether extents are equal, determined, positive, or bounded.

## Source-first Fact audit — 2026-08-12, exact base `e2db0a66812604897cfb7f8a6c6b7a55f231cc41`

**Verified — environment construction discards the proof state it just established.** `ShapeEnvBuilder::build`, anchor `constraint::decide(&bound, &relations)?`, decides the canonical semantic constraints and then retains only authored entries, constraints, guards, and identity. The current `ShapeEnv` documentation says storing nothing derived is how caches stay out of identity; exclusion from identity does not require discarding a deterministic immutable summary.

**Verified — every public semantic proof query resolves the same constraints again.** `ShapeEnv::extent_interval`, `ShapeEnv::proves_positive`, and `ShapeEnv::proves_equal` each rebuild the relation vector and call `constraint::solve`. `ExtentSources::{determined,proves_positive,proves_equal,interval}` delegate to those methods; `proves_equal` can additionally ask for two determined values after its equality-class query.

**Verified — semantic inference calls that work per symbolic axis.** `elementwise_binary_shape`, anchor `sources.proves_equal(&left, &right)`, walks paired extents and asks once for every symbol-involving axis. The accepted sourced broadcast v2 relation is a second real consumer of the same equality and determined-value questions. Re-solving per axis is therefore no longer a single-family local cost.

**Inference — a retained summary improves host bounds without becoming semantic authority.** The environment's entries and semantic constraints are already canonical and immutable. A summary derived solely from that population can retain canonical symbol slots, equality-class membership, and closed intervals. It answers the same one-sided questions while the authored environment remains the only encoded subject.

## Work

- Refactor the successful semantic-constraint solve used by `ShapeEnvBuilder::build` into an immutable `ExtentProofSummary` retained by the verified environment. It contains only facts derivable from canonical entries and semantic constraints; variant guards never enter it.
- Make equality, interval, determined-value, and positivity queries use the summary. A literal still answers directly. An undeclared symbol still returns `false` or `None`; no query invents disequality, a value, or a bound.
- Keep the summary out of `ShapeEnvIdentity`, canonical bytes, graph identity, serialization, and public construction. Update prose that currently equates identity exclusion with retaining no derived state.
- Preserve exact behavior for a failed internal lookup: typed public builders already reject undeclared symbols, while read-only proof queries remain fail-closed. Do not add a panic, lazy fallback solve, or second constraint interpretation.
- Keep guard satisfiability separate. Guards are planning predicates and may be decided against the semantic summary plus the one candidate guard; they never widen the semantic proof summary.

## Complexity contract

Environment construction performs one constraint solve and retains O(symbols) derived state. Symbol lookup is O(log symbols) or better against canonical slots; equality, interval, positivity, and determined-value answers are O(1) after lookup. An operation or broadcast remains O(total rank). No query may fall back to resolving the full semantic constraint set.

## Required evidence

- Instrument the semantic solver in a test-only build: one environment construction increments the solve census once, while a population of repeated equality, interval, positivity, and determined-value queries does not increment it again.
- Re-run the complete existing sourced-extent proof matrix and perturb equality class, determined literal, interval, undeclared symbol, and positivity subjects independently.
- Exercise a maximum-rank symbolic elementwise application and the first symbolic broadcast-v2 fixture with a nonzero proof-query census and exactly one semantic solve.
- Prove environment canonical bytes and identities are unchanged when the summary representation is perturbed without changing authored entries or constraints.
- Make removal of summary use fail by perturbing the query implementation while leaving assertions unchanged; quote the repeated-solve failure text.

## Public and identity boundary

No new public type, constructor, serialized field, or identity component is admitted. `ExtentProofSummary` is private derived state. `ShapeEnv` equality may include it only if equality remains provably equivalent to equality of the canonical authored population; otherwise implement equality over authored state explicitly. The summary never becomes a second source of semantic facts.

## Non-goals

Changing the constraint language, admitting a new symbolic operation family, caching variant-guard verdicts, incremental environment mutation, or exposing a general solver/proof API.

## Closes when

Every semantic extent proof query reuses one immutable summary, repeated per-axis solving is made impossible by a load-bearing census, all one-sided proof outcomes remain unchanged, and no canonical identity or public surface moves.
