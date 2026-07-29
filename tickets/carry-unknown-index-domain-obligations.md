---
id: carry-unknown-index-domain-obligations
title: Carry Unknown index-domain obligations through verified regions
status: done
priority: p1
dependencies: [retain-durable-index-domain-proof-evidence]
related: []
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer]
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

## Outcome

- Former read-side `BoundsNotProven` and `ExtentBoundNotStated` outcomes now produce exact residual predicates with `InsufficientFacts`; proof-cell and proof-integer-storage exhaustion produce `ResourceLimit` with the exact resource, required amount, and limit. Both obsolete diagnostic variants were removed.
- `UnsupportedFragment` is retained as the typed reason for an admitted expression fragment the current prover cannot decide, but this ticket creates no synthetic producer. Semi-affine construction remains with `represent-semi-affine-index-expressions-in-the-ir`.
- Proven out-of-bounds coordinates, malformed structure, boundary-role violations, and unresolved ordinary-write ownership remain hard rejections. A proof-resource stop beside any hard diagnostic remains secondary evidence and cannot upgrade the rejection.
- Discharged and residual atoms share the v8 canonical index-region assessment sequence. Exact subject, predicate, outcome, proof basis or unknown reason, resource, required amount, and limit all participate in identity.
- Logical obligations are independent of value-type family; nominal `bool`, `i4`, and `u4`, parameterized complex, and encoded block-quantized fixtures retain the same coordinate-obligation topology while preserving distinct region identities. Physical packing, component buffers, masks, byte ranges, and ABI checks remain downstream.
- Refinement now distinguishes invalid provider output, completed proof, and valid checked state awaiting discharge. `IndexRefinementOutcome::Pending` owns the exact verified region, semantic occurrence, frozen scalar/capability authorities, and checked operand/result bindings, so region-local predicate handles never outlive their authority and a later discharge stage need not re-run the provider. Pending state mints no refinement identity and the current compiler still fails closed before cover enumeration or executable frontier construction.
- Only sound proof and exact exhaustive finite evidence can mint a discharged predicate. The reserved `Empirical` evidence class is explicitly non-discharging and grants no execution permission.
- The compiler's failure trace retains one provider-attributed `Unknown` record per canonical region-local obligation key, including exact proof-resource quantities, rather than misclassifying the residual as disproved. No unattributed physical guard or execution permission is created.
- Fault injection proved the exact resource fields, unknown reason, checked subject/predicate associations, physical-guard absence, value-type identity, every proof-basis tag, every unknown-reason/resource field, canonical local key, provider attribution, trace quantities, compiler refusal, diagnostic-precedence, pending-state custody, and empirical non-discharge checks can fail.
