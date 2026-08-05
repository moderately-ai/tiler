---
schema: "tiler-doc/v1"
id: "ADR-0099"
kind: "decision"
title: "Project an elementary family's per-point body from one shared statement"
topics: ["optimizer", "numerics", "operation-extensions"]
catalog_group: "physical-planning-lowering"
decision_status: "proposed"
implementation_status: "partial"
applies_to: ["tiler.contract.optimizer", "tiler.contract.ir"]
evidence: ["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]
ticket: "admit-the-registered-unary-families-at-the-compiler-request-boundary"
---

# 0099: Project an elementary family's per-point body from one shared statement

**Status:** proposed

## Context

A registered semantic family whose normative definition pins a *composition* —
`tiler::silu-f32@1`'s `x / (1 + Exp(-x))`, with the negation exact and the
addition and the division rounding once each — has to be realized twice inside
the compiler. The governed index-access lowering emits it as `tiler_ir::index`
scalar applications, which occurrence refinement then proves realizes the
occurrence. The request boundary projects it into the physical
`PointwiseF32Expression` the scheduled region carries to a backend.

Two routes were proposed for making such a family reachable at all:

1. the region vocabulary grows a node per admitted elementary family; or
2. a region gains a way to name an occurrence whose per-point body *is* the
   resolved capability's emitted index region.

## Decision

**The region vocabulary spells an elementary family's per-point body in its
existing primitive nodes, and the compiler states that body exactly once.**

A family is admissible at the request boundary when its per-point body is
expressible in `PointwiseF32Node`. The body is written in one place, against an
abstract per-point sink whose vocabulary is deliberately smaller than the node
enum, and every realization — the index-access lowering's and the request
boundary's projection — is driven from that one statement.

## Consequences

A family whose body is expressible needs no `tiler-ir` change to become
reachable, and gets none: `Exp`, `Divide`, and `Rsqrt` were already nodes, and
already emitted by the Metal backend, before any of them was reachable through
the request boundary.

The boundary's projection is not an independent claim. Because the index-access
lowering emits the same statement, occurrence refinement's proof that the
emitted region realizes the semantic occurrence is also evidence about the
projection: a change to the composition that made it stop realizing the
occurrence fails at refinement, before any region is scheduled.

A family whose *access relation* has no spelling — a reindex, a non-scalar
broadcast — is not made reachable by this decision and continues to refuse by
name. The missing vocabulary there is `LogicalAccess`, and no projection
substitutes for it.

## Alternatives considered

**One node per elementary family.** A `PointwiseF32Node::Silu` would preserve the
semantic family down to the backend. It was rejected because the node vocabulary
is deliberately rounding-explicit — it carries `Divide` rather than a reciprocal
node precisely so that a two-rounding substitution is unstatable — and a single
node standing for a four-rounding composition hides what the vocabulary exists
to expose. It would also relocate rather than remove the second authority, since
each backend would then re-derive the composition.

**Embedding the resolved capability's emitted index region as the region body.**
This was the more interesting route: it would make an out-of-crate provider's
family reachable without a `tiler-ir` change per family, and refinement's proof
would carry over by construction. It was rejected because the emitted region's
scalar vocabulary is an open, registry-driven `ScalarOpKey` space, while
`PointwiseF32Node` is closed — and that closedness is what makes a new physical
meaning a build error at every schedule-identity, KIR-lowering, and backend
emission site rather than a silently unlowerable body. Restricting the embedded
body to a closed subset restores the property and reduces the route to this
decision plus a projection.

**Decomposing the family at the boundary without a shared statement.** Rejected
because it creates two independent claims about one meaning, only one of which
any authority checks.
