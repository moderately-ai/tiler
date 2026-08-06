---
id: accept-the-composed-realization-evaluation-surface
title: Accept the composed realization evaluation surface
status: todo
priority: p2
dependencies: []
related: [compose-a-declared-reduction-topology-into-a-semantic-program-evaluation, accept-the-realization-witness-surface, decide-how-a-pinned-pointwise-grouping-becomes-evaluable, implement-the-realization-witness-vocabulary]
scopes: []
shared_scopes: []
paths: []
tags: [tiler-research, numerics, reference, conformance]
---
## User-visible outcome

Tom decides, or declines, the public surface a composed realization evaluation needs: one driver that answers for a program spending reassociation at both the semantic rewrite and a physical reduction split, and one `ValueId`-keyed reference primitive it is built on. Nothing is released on it until he does.

## Why this exists

**Fact.** [The composed-realization-evaluation derivation](../docs/research/reference/composed-realization-evaluation.md) Part 7 drafts the surface with its evidence and eliminates the four alternatives to it. Under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) a new reference entry point and a new driver type are each a public boundary, so the record drafts and parks rather than adopting.

**Fact — the constraint is already decided and this surface is drafted inside it.** Tom decided 2026-08-06 on [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md) that the reference keeps taking plain scalars and `tiler-reference` never names a plan structure. Item B below is `ValueId`-keyed — `tiler_ir::semantic::ValueId` is already imported at `crates/tiler-reference/src/evaluate.rs:29` — so it stays inside that decision rather than reopening it.

**Fact — the acceptance is two separable questions.**

1. **Item A — the driver.** One entry point taking the retained selected semantic candidate, the plan's witness sequence, and the declared input bindings, returning the expected output tensors or the first refusal. It performs its own pinning.
2. **Item B — the reference primitive.** An evaluation accepting `(ValueId, &Tensor)` pins and observed `ValueId`s, returning declared outputs and observed tensors. The pins generalize `InputBinding` (`crates/tiler-reference/src/tensor.rs:283-286`) from a declared input to any value; the observations expose values `ReferenceEvaluator::evaluate` already computes and drops at `crates/tiler-reference/src/evaluate.rs:280-283`.

**Inference — the ordering matters and is the one thing the record asks Tom to weigh.** The record's stated counterpoint to its own survivor is that item B is a hole where an oracle should be: a caller that pinned a tensor taken from the artifact under test would make the comparison vacuous, and the reference's types cannot tell the two provenances apart. Accepting B without A, or exposing B publicly beside A, keeps that hole open; accepting A as the only public entry closes it structurally rather than by convention.

## What this ticket must produce

Tom's decision, recorded with who accepted, the date, and the venue. The shapes discipline admits: acceptance of A with B kept crate-internal or `#[doc(hidden)]`; acceptance of both as public; acceptance of A with B excluded behind its own ticket; or a redirection.

## Explicit non-goals

Implementing either item; re-deciding the pointwise evaluability fork, which is settled; accepting the realization witness surface, which is decided.

## Closes when

Tom has accepted, excluded, or redirected each item by name, and the record's Part 7 states which.

## Graph maintenance

Filed by [`compose-a-declared-reduction-topology-into-a-semantic-program-evaluation`](compose-a-declared-reduction-topology-into-a-semantic-program-evaluation.md) as the public boundary it was forbidden to self-accept. It depends on nothing, but the surface is only useful once the settled pointwise design's retention lands: the record's refusal 1 (`CandidateProgramNotRetained`) is the interim answer until it does.
