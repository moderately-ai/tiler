---
schema: "tiler-doc/v1"
id: "ADR-0012"
kind: "decision"
title: "Keep reduction topology in physical plans"
topics: ["numerics","reductions","scheduling"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "reduction-semantics-contract"
---

# 0012: Keep reduction topology in physical plans

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [reduction-semantics-contract](../../tickets/reduction-semantics-contract.md).


## Context

Floating-point reduction results can change when the same inputs are combined
with a different parenthesization. A semantic reduction must therefore
constrain evaluation order. However, a concrete parallel reduction tree also
contains target-dependent scheduling choices such as SIMD width, threadgroup
partitioning, synchronization, and intermediate passes.

Putting that concrete tree in the semantic graph would make tensor meaning
depend on a GPU schedule and would suppress otherwise legal physical
alternatives.

## Decision

A semantic reduction carries an order contract that defines the allowed
evaluation-order or result class. It does not carry a concrete parallel
reduction topology. The contract must be expressive enough to distinguish an
ordered fold, a deterministically selected legal order, and a relaxed result
set when reassociation is permitted; the final public variants and names remain
to be specified.

Physical planning chooses and records the actual topology, including
partitioning, tree shape, synchronization, and multi-pass structure. The
schedule is legal only when its evaluation is contained by the semantic order
contract. The selected topology participates in physical-plan and artifact
identity.

Deterministic order uses a separately defined, explicit stability scope. This
decision does not use `deterministic` as an unqualified promise.

## Consequences

- Semantic reductions remain backend-neutral while still constraining
  floating-point results.
- The optimizer may cost several legal reduction topologies without changing
  the semantic graph.
- Ordered reductions can reject parallel trees that change their evaluation.
- Relaxed reductions can admit target-specific trees only through explicit
  permissions.
- Explain output can distinguish rejection by semantic order from rejection by
  target resources or cost.

## Implementation boundary

Added 2026-08-01 by [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](../../tickets/re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which moved `implementation_status` from `not-started` to `partial` and replaced the 2026-07 exclusion reason "physical reduction topology exists, but no typed semantic order-contract vocabulary". This section states which clauses that value rests on and which it does not, read at `2aa0824`. It is a status record and adds no decision.

**Realized — a semantic reduction constrains evaluation order and carries no topology.** Each registered reduction family declares its order terms as canonical definition facts that enter durable identity: the contraction declares its contributor sequence as `ascending-lexicographic-over-the-canonically-ordered-contracted-index-space` (`crates/tiler-ir/src/semantic/contraction.rs:952`), its seed as `none-the-accumulator-starts-at-the-first-product` (`:956`), its empty contracted domain as `refused-an-unseeded-fold-has-no-empty-result` (`:960`), reassociation and permutation separately as `false` (`:964`, `:968`), and its determinism as `plan-deterministic` (`:988`); the strict serial sum declares `strict-left-fold` (`crates/tiler-ir/src/semantic/registry.rs:2095`). No semantic definition names a tree shape, a partition, a SIMD width, a synchronization point, or a pass count.

**Realized — physical planning chooses and records the actual topology.** `ReductionTopology` at `crates/tiler-ir/src/schedule/model.rs:536` is the typed physical vocabulary: `None`, `Serial`, `MultiPass`, `Contraction`, and `CooperativeWorkgroup`, each recording the reduced axes and contributor order, and the split strategies additionally recording the pass role, the contributor partition, the accumulation width, and — for the cooperative tile — the cross-invocation dataflow and the arrival order of staged partials. `crates/tiler-compiler/src/physical.rs` constructs them, and `:1596` and `:1612` bind a split's two passes and a workgroup tree to the *same* semantic subject the single-dispatch region binds to, which is the decision's "several legal reduction topologies without changing the semantic graph".

**Realized — the schedule is legal only when its evaluation is contained by the contract.** The schedule verifier requires each topology's recorded permissions to equal the region's declared `NumericalRealization` (`crates/tiler-ir/src/schedule/builder.rs:391`, `:672`, `:703`, `:739`, `:776`), and admits the two reassociating strategies only when reassociation is permitted: `builder.rs:831` for the multi-dispatch split and `:967` for the cooperative tile, each checked on reassociation alone so a permitted permutation cannot stand in for it. A contraction with no contracted points is refused at `:403`, and an extrema fold over an empty domain at `:792`, because neither family has an empty-domain identity to commit.

**Realized — the selected topology participates in identity.** `crates/tiler-ir/src/schedule/model.rs:1373` encodes each resolved permission and `:1716`–`:1773` encodes each topology variant's own fields into canonical schedule identity through exhaustive matches over vocabularies whose growth is a build error, so two schedules differing only in reduction topology are two identities.

**Unrealized — the three named order-contract classes.** The decision requires a contract "expressive enough to distinguish an ordered fold, a deterministically selected legal order, and a relaxed result set". Only the first exists, and it exists as the operation family's own identity rather than as a contract value: every admitted reduction is a strict fold, no operation admits a deterministically-selected or relaxed order class, and the two region-level permissions are the whole of the relaxation vocabulary. A reader should not read `partial` as evidence that the class distinction has been designed.

**Unrealized — the explicit stability scope for deterministic order.** The contraction states `plan-deterministic` as a fact string, and nothing defines which executions, artifacts, targets, or toolchains share that promise. ADR 0013 owns the scope vocabulary and reads `not-started`.

**Unrealized — explain output that separates a semantic-order rejection from a resource or cost rejection.** Every containment failure above returns the single class `ScheduledRegionDiagnostic::NumericalOrAccessRefinement` (`crates/tiler-ir/src/schedule/error.rs:88`), so a refused schedule does not report whether its evaluation escaped the order contract or its accesses were malformed.

## Alternatives considered

Storing the exact tree in semantic IR completely specifies evaluation but
mixes target scheduling into tensor meaning. Leaving reduction order entirely
to physical planning permits numerical changes absent from the semantic
contract. A boolean `deterministic` flag is insufficient because it does not
state which executions, artifacts, targets, or toolchains share the promise.
