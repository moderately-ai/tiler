---
id: retain-durable-index-domain-proof-evidence
title: Retain durable index-domain proof evidence
status: todo
priority: p1
dependencies: [implement-index-domain-predicates]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof]
---

## User-visible outcome

Every discharged index-domain predicate exposes durable typed evidence from the verified region, so compiler diagnostics and downstream consumers can distinguish how the predicate was established without consulting a transient proof cache.

## Implementation keys

- Add a closed evidence vocabulary that keeps sound proof, exhaustive finite evidence, empirical evidence, and `Unknown` distinct. Do not add ordering, scores, or conversion to a confidence scalar.
- Bind evidence to the exact `IndexDomainPredicate` and verified-region subject it proves. Construction is checked and foreign handles are refused.
- Include retained evidence and its predicate subject in canonical region identity when it changes what downstream consumers may rely on; exclude solver search state and memoization.
- Keep `UnknownReason::{InsufficientFacts, UnsupportedFragment, ResourceLimit}` separate from evidence. This ticket must not turn an unknown obligation into an admitted executable program.
- Add an exhaustive construction-site test covering every evidence variant and prove the test can fail by perturbing one correspondence once.
- Follow ADR 0084 and `docs/research/shapes/constraint-prover-boundary.md`.

## Closes when

Verified regions retain inspectable typed evidence for discharged predicates; the four evidence classes cannot collapse through the public type; identity covers every relied-on subject; targeted `tiler-ir` nextest, Clippy, and doc-tests pass; the new check has been observed failing under deliberate perturbation; and `make full` passes.

## Graph maintenance

- On completion, record the exact public types and identity consequence here.
- Release `carry-unknown-index-domain-obligations`.
- File any new proof lane separately rather than widening this evidence-custody ticket.
