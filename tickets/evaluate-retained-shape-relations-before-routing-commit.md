---
id: evaluate-retained-shape-relations-before-routing-commit
title: Evaluate retained shape relations against invocation bindings before routing commit
status: todo
priority: p1
dependencies: [admit-an-additive-extent-relation, reclassify-language-model-work-as-a-conformance-track, bind-repeated-invocations-over-caller-retained-tensors]
related: [design-autoregressive-state-and-kv-cache, execute-the-decode-step-path, test-the-autoregressive-state-failure-cases]
scopes: [implementation/ir, implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, shapes, runtime, consumer-neutral, fail-closed, class-generic-capability]
---
## User-visible outcome

An invocation whose live extent bindings violate a retained semantic shape
relation refuses before routing commit and before any program work, so a caller
rebinding a tensor it retained from an earlier invocation cannot pass a stale
extent merely because its allocation is large enough.

## Correctness boundary

**Fact.** The accepted `ExtentRelation::AdditiveEquality` representation and
`ShapeEnvBuilder` check static/root-bound contradictions. A relation with
runtime-bound terms is retained when the canonical lower-bound model exhibits a
solution, but no current launch-preflight consumer evaluates it against the
invocation's live values.

**Inference.** Representation without consumption does not close L5's stale-binding
case — *and after [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md)
this check is the **only** layer that closes any part of it, because no runtime
state survives an invocation to hold a valid length.* The decoded artifact must retain the governed relation and its term
bindings, the invocation must supply each live value from its authoritative
source, and the check must run before the one-way routing commit. A missing,
wrong-domain, contradictory, or unevaluable binding refuses with a typed cause;
it never becomes a fallback after program work begins.

## Required work

- Trace the accepted ShapeEnv relation and binding identities through artifact
  construction, encoding, decoding, and runtime preflight without duplicating a
  second shape solver or letting runtime depend on frontend-specific types.
- Evaluate the same bounded relation vocabulary the artifact claims to carry;
  unsupported future variants refuse rather than being ignored.
- Bind each relation term from its authoritative invocation source — the bound
  input-axis extent — rather than from a caller-supplied scalar. Do not let a
  caller provide a second uncorrelated spelling of a value the bindings already
  determine.
- Preserve routing discipline: semantic binding validation completes before
  `RoutingCommit`, allocation, encoding, or submission.
- Execute any artifact/schema identity step whole, updating its owning ledger
  and recomputing every pin on the merged tree. Stop for Tom if the exact design
  requires a new consequential public type or call-site boundary.

## Required evidence

- The static neighbour `S = 15, C = 14, T = 1` and its runtime-bound equivalent
  both pass.
- `S = 13, C = 14, T = 1` refuses before routing commit, names all three terms
  and observed sides, and fails if the preflight check is removed.
- A missing binding, a binding from the wrong symbol scope/domain, and an
  unsupported retained relation each refuse under distinct typed causes.
- The C1 decode path consumes the check without changing artifact identity per
  step; one artifact remains valid across the changing invocation bindings.

## Closes when

Retained shape relations are identity-bound into the decoded artifact and
evaluated against authoritative invocation values before routing commit; every
negative path above has been watched failing; the decode-step ticket depends on
this consumer; and targeted IR/artifact/runtime/build checks plus the full gate
pass.
