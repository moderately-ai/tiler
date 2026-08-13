---
id: retain-one-derived-proof-summary-per-shape-environment
title: Retain one derived proof summary per shape environment
status: in-progress
priority: p1
dependencies: [resolve-semantic-shape-inference-over-symbolic-extents]
related: [seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, narrow-symbolic-inference-and-restore-host-owned-refusals, replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, constraints, performance, correctness, identity]
claimed_from: todo
assignee: worker-retain-proof-summary
lease_expires_at: 1786585709
---
## User-visible outcome

One verified symbolic environment solves its semantic closure once, then every semantic and index consumer asks the same immutable proof summary whether extents are equal, determined, positive, or bounded. A later symbolic broadcast uses that same path rather than introducing another solver authority.

## Source-first Fact audit — 2026-08-12, exact base `612468048d541a1017640fc5dcbe5ff9160716cf`

Re-read at this worker's base before any implementation edit. Earlier audits at `e2db0a66` and `f4d1f884` were treated as stale until each cited site was opened here.

**Verified — environment construction still discards the successful semantic solution.** `ShapeEnvBuilder::build`, anchor `constraint::decide(&bound, &relations)?`, decides the canonical semantic relations and then constructs `ShapeEnv` with only `entries`, `constraints`, `guards`, and `identity`. `rg ExtentProofSummary crates/` is empty.

**Verified — every public semantic proof query still resolves the same constraints again.** `ShapeEnv::extent_interval`, `ShapeEnv::proves_positive`, and `ShapeEnv::proves_equal` each rebuild the relation vector and call `constraint::solve`. Those three `constraint::solve` sites are the only ones under `crates/`. `ExtentSources::{determined,proves_positive,proves_equal,interval}` still delegate to those methods; `proves_equal` can still ask for two determined values after the equality-class query.

**Verified — `elementwise_binary_shape` is still the live per-axis semantic consumer.** Anchor `sources.proves_equal(&left, &right)` in `crates/tiler-ir/src/semantic/registry.rs` still walks paired extents and asks once for every symbol-involving axis. Broadcast remains the literal-extent v1 family in `crates/tiler-ir/src/semantic/broadcast.rs`; sourced broadcast v2 is not a live consumer.

**Verified — `IndexRegionBuilder` is the second live consumer.** Anchors `fn extent_interval`, `fn determined`, `fn extents_proved_equal`, and `fn admit_divisor` still ask the environment for intervals, determined values, equality, and positivity. `determined` currently reads a one-point interval rather than a distinct summary query.

**Verified — the solver result is still the wrong retained shape.** `constraint::Solution` still holds a mutable `Classes` disjoint-set forest (`fn find` path-compresses) and per-class `Domains`. Retaining that object would put interior mutation on read-only proof queries.

**Verified — identity exclusion is still written as if it required storing nothing derived.** `ShapeEnv::identity`, anchors `Nothing derived from the constraints is stored` and `storing nothing derived is how this module holds that`, still equate identity exclusion with discarding the successful solve. `docs/ir.md`, fragment `provenance, and semantic constraints but excludes derived solver caches`, still excludes derived caches from identity and does not itself forbid retaining a private summary.

**Still false — one summary plus one guard is not sufficient to decide every guard.** Unchanged logical counterexample: a semantic `a >= b` with a candidate guard `b >= a` is an equality cycle only when considered together. Guard satisfiability stays a separate hypothetical solve over authored semantic relations plus exactly one guard.

**Still an inference — a retained summary is not a second semantic authority.** Authored entries and semantic constraints remain the sole encoded subject; the summary is a projection of the successful solve.

## Source-first Fact audit — 2026-08-12, exact base `e2db0a66812604897cfb7f8a6c6b7a55f231cc41`

**Verified — environment construction discards the proof state it just established.** `ShapeEnvBuilder::build`, anchor `constraint::decide(&bound, &relations)?`, decides the canonical semantic constraints and then retains only authored entries, constraints, guards, and identity. The current `ShapeEnv` documentation says storing nothing derived is how caches stay out of identity; exclusion from identity does not require discarding a deterministic immutable summary.

**Verified — every public semantic proof query resolves the same constraints again.** `ShapeEnv::extent_interval`, `ShapeEnv::proves_positive`, and `ShapeEnv::proves_equal` each rebuild the relation vector and call `constraint::solve`. `ExtentSources::{determined,proves_positive,proves_equal,interval}` delegate to those methods; `proves_equal` can additionally ask for two determined values after its equality-class query.

**Verified — semantic inference calls that work per symbolic axis.** `elementwise_binary_shape`, anchor `sources.proves_equal(&left, &right)`, walks paired extents and asks once for every symbol-involving axis. The accepted sourced broadcast v2 relation is a second real consumer of the same equality and determined-value questions. Re-solving per axis is therefore no longer a single-family local cost.

**Inference — a retained summary improves host bounds without becoming semantic authority.** The environment's entries and semantic constraints are already canonical and immutable. A summary derived solely from that population can retain canonical symbol slots, equality-class membership, and closed intervals. It answers the same one-sided questions while the authored environment remains the only encoded subject.

## Refreshed Fact audit — 2026-08-12, exact base `f4d1f884b25ea9dcf99a35e88dd8eb6e2623d8b6`

**Verified — the first two Facts remain current.** `ShapeEnvBuilder::build`, anchor `constraint::decide(&bound, &relations)?`, still discards the successful semantic solution. `ShapeEnv::{extent_interval, proves_positive, proves_equal}` still rebuild the semantic relation vector and call `constraint::solve` independently.

**Imprecise — sourced broadcast v2 is not a live consumer yet.** `elementwise_binary_shape`, anchor `sources.proves_equal(&left, &right)`, is the current per-axis semantic consumer. The second current consumer is `IndexRegionBuilder`, anchors `fn extent_interval`, `fn determined`, `fn extents_proved_equal`, and `fn admit_divisor`, which repeatedly asks the environment for intervals, determined values, equality, and positivity. `replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics` is a dependent future consumer and must add its own high-rank regression when it lands.

**False — one summary plus one guard is not sufficient to decide every guard.** A semantic relation `a >= b` and a candidate guard `b >= a` form an equality cycle only when considered together. A frozen summary containing current equality classes and intervals no longer carries enough authored relation structure to discover that cycle; additive and factorization guards have analogous interactions. Guard satisfiability must remain a separate hypothetical solve over the authored semantic relations plus exactly one guard. It never widens or mutates the retained semantic summary.

**Verified — the solver result has the right bounded information but the wrong retained shape.** `constraint::Solution` contains mutable union-find classes and per-class domains. The environment needs a frozen projection: one normalized class identifier and one closed interval per canonical symbol slot. Retaining the path-compressing solver object would introduce interior mutation or locking into read-only proof queries without adding proof power.

## Work

- Refactor the successful semantic-constraint solve used by `ShapeEnvBuilder::build` into an immutable `ExtentProofSummary` retained by the verified environment. It contains only facts derivable from canonical entries and semantic constraints; variant guards never enter it.
- Make equality, interval, determined-value, and positivity queries use the summary. A literal still answers directly. An undeclared symbol still returns `false` or `None`; no query invents disequality, a value, or a bound.
- Keep the summary out of `ShapeEnvIdentity`, canonical bytes, graph identity, serialization, and public construction. Update prose that currently equates identity exclusion with retaining no derived state.
- Preserve exact behavior for a failed internal lookup: typed public builders already reject undeclared symbols, while read-only proof queries remain fail-closed. Do not add a panic, lazy fallback solve, or second constraint interpretation.
- Freeze the successful solution into canonical-slot records containing only a normalized equality-class identifier and the implied closed interval. Do not retain mutable/path-compressing solver state.
- Keep guard satisfiability separate. Decide each guard against the authored semantic relations plus exactly that guard. Guard solves are hypothetical planning work, never semantic-closure solves, and never widen or mutate the retained proof summary.

## Complexity contract

Environment construction performs exactly one semantic-closure solve and retains O(symbols) derived state. Construction and `unsatisfiable_guards` may additionally perform separately counted guard-hypothesis solves, one per guard, because those solves answer a different planning question. Symbol lookup is O(log symbols) or better against canonical slots; equality, interval, positivity, and determined-value answers are O(1) after lookup. An operation, index proof, or future broadcast remains O(total rank). No semantic proof query may fall back to resolving the full semantic constraint set.

## Required evidence

- Instrument test-only semantic-closure and guard-hypothesis censuses separately: an unguarded environment construction increments the semantic census once, while repeated equality, interval, positivity, and determined-value queries increment neither census. A guarded fixture proves its independent guard-hypothesis population without changing the semantic count.
- Re-run the complete existing sourced-extent proof matrix and perturb equality class, determined literal, interval, undeclared symbol, and positivity subjects independently.
- Exercise a maximum-rank symbolic elementwise application and a sourced index-region fixture with nonzero interval, equality, determined-value, and positivity query censuses and exactly one semantic solve. The dependent broadcast-v2 ticket owns the equivalent broadcast regression once that family exists.
- Prove environment canonical bytes and identities are unchanged when the summary representation is perturbed without changing authored entries or constraints.
- Make removal of summary use fail by perturbing the query implementation while leaving assertions unchanged; quote the repeated-solve failure text.

## Public and identity boundary

No new public type, constructor, serialized field, or identity component is admitted. `ExtentProofSummary` is private derived state. Implement `ShapeEnv` equality and debug output over the same authored fields observed today, excluding the summary, so an internal representation change cannot alter equality or diagnostics. The summary never becomes a second source of semantic facts.

## Decision — accepted 2026-08-12

**Provenance.** Tom accepted this repaired boundary directly in the ChatGPT coordination thread after reviewing the current-base Fact audit, guard counterexample, identity consequences, host-work bound, and ranked alternatives. Acceptance authorizes the implementation contract but does not mark the implementation complete; this ticket remains `todo`.

Retain one mandatory private frozen `ExtentProofSummary` directly in every verified `ShapeEnv`. It is derived eagerly from the successful semantic solve and stores normalized class membership and implied intervals by canonical symbol slot. Proof queries use binary search over canonical entries and constant-time summary reads after lookup. There is no lazy solve, fallback solve, global cache, lock, mutable retained union-find, public proof API, or second constraint interpreter.

Authored entries and semantic constraints remain the sole authority and the sole inputs to environment identity. The summary is excluded from canonical bytes, identity, serialization, equality, and debug output. Guards remain separate hypothetical solves over authored semantic relations plus one guard, with a separately named census; they never enter semantic closure.

The accepted implementation order is this ticket, then `narrow-symbolic-inference-and-restore-host-owned-refusals`, then sourced broadcast v2. This decision does not authorize production implementation in the coordination turn that recorded it.

## Implementation notes — 2026-08-12

Subject perturbation of `ShapeEnv::proves_equal`: the summary read was replaced with `constraint::solve(..., SolveKind::SemanticClosure)` while leaving `an_unguarded_environment_solves_semantic_closure_once` unchanged. Failure text:

```
assertion `left == right` failed: repeated proof queries must not increment semantic-closure solve
  left: 8
 right: 0
```

The query implementation was restored to the retained summary. `rg ExtentProofSummary crates/` now matches only the private type and ticket-facing comments inside `tiler-ir`; the type is not re-exported.

## Non-goals

Changing the constraint language, admitting a new symbolic operation family, caching variant-guard verdicts, incremental environment mutation, or exposing a general solver/proof API.

## Closes when

Every semantic extent proof query reuses one immutable summary, repeated per-axis solving is made impossible by a load-bearing census, all one-sided proof outcomes remain unchanged, and no canonical identity or public surface moves.
