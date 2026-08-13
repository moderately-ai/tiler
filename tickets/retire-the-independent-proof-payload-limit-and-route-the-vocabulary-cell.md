---
id: retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell
title: Retire the independent proof-payload limit and route the vocabulary cell
status: in-progress
priority: p1
dependencies: [decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights, enforce-proof-sidecar-byte-budgets-before-producer-allocation]
related: [route-the-realization-conformance-half-into-the-conformance-crate]
scopes: [implementation/artifact, contracts/artifacts, implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, proof, conformance, public-boundary]
claimed_from: todo
assignee: worker-retire-payload-limit
lease_expires_at: 1786587491
---
## User-visible outcome

One proof payload is admitted whenever the complete proof sidecar remains within its governed container budget. The 32 MiB vocabulary-projection weights route through the ordinary L3 conformance member without a workload special case, and there is no second arbitrary payload-size authority to drift.

## Required delivery

- Remove public `MAX_PROOF_PAYLOAD_BYTES`, `ProofLimitKind::PayloadBytes`, their producer/decoder checks, rustdoc links, displays, tests, and contract rows. Do not retain a deprecated alias in this pre-production tree.
- Make framed-length decoding rely on checked representability, remaining input, and the already-established complete-sidecar bound, with typed malformed/truncated/sidecar-limit distinctions and no partial decoded value.
- Add `w_vocab_slice` to `CONTRACTION_MEMBERS` through the same constructor as the other retained L3 cells. Update derived publishable populations, publication counts, docs, and negative controls; `cases_for(L3CorrectnessCell)` already synthesizes its operands and must not gain a special row.
- Pin that the routed case's complete payload content is within `MAX_PROOF_SIDECAR_BYTES`; separately perturb one total beyond the container and observe the unchanged atomic refusal.
- Record the public breaking removal and exact included/excluded facade after an independent exact-commit review.

## Non-goals

Changing the 256 MiB container limit, a wire/schema version step, payload chunking, compression, external references, lazy resolution, streaming decode, or changing proof-sidecar content identity.

## Closes when

The independent limit no longer exists, all proof byte admission is container-based, the seventh contraction route publishes and validates, no existing content identity moves unexpectedly, and full artifact/conformance gates plus independent review pass.
