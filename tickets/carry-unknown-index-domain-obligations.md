---
id: carry-unknown-index-domain-obligations
title: Carry Unknown index-domain obligations through verified regions
status: todo
priority: p1
dependencies: [retain-durable-index-domain-proof-evidence]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof]
---

## User-visible outcome

A structurally valid index region no longer becomes a rejection merely because a sound proof lane lacked facts, support, or budget; the verified region carries the exact residual predicate and structured `Unknown` reason without inserting a physical guard.

## Implementation keys

- Stop flattening the existing interval-overlap, unbounded-symbol, and exhaustive-proof budget outcomes into rejection when every non-proof structural obligation succeeded.
- Retain each unresolved `IndexDomainPredicate` with `UnknownReason::{InsufficientFacts, UnsupportedFragment, ResourceLimit}` and preserve the exact exhausted resource where applicable.
- A region with any genuine disproval or structural diagnostic still rejects. Mixed rejection and resource-limit results must not be upgraded.
- Pin the current budget case: a region that exceeds `MAX_EXHAUSTIVE_PROOF_CELLS` or `MAX_EXHAUSTIVE_PROOF_BYTES` verifies with an explicit obligation and no attributable physical bounds check.
- Prove the no-guard check can fail by temporarily inserting or reporting a guard and observing the test fail.
- Rebaseline canonical identities only on the final merged tree if obligation carriage changes them.

## Closes when

The budget and insufficient-facts fixtures verify with typed residual obligations; disproved and malformed regions still fail closed; no physical guard is emitted; retained reasons are inspectable; targeted `tiler-ir` and affected compiler tests pass; every new check has demonstrated its failure path; and `make full` passes.

## Graph maintenance

- On completion, record which former diagnostics became `Unknown` and which remain rejections.
- Release `discharge-residual-index-domain-obligations`.
- Link any unsupported semi-affine construction case to `represent-semi-affine-index-expressions-in-the-ir` rather than approximating it here.
