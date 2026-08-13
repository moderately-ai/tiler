---
id: enforce-proof-sidecar-byte-budgets-before-producer-allocation
title: Enforce proof-sidecar byte budgets before producer allocation
status: in-progress
priority: p1
dependencies: [decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights]
related: [retire-the-independent-proof-payload-limit-and-route-the-vocabulary-cell]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, proof, correctness, resource-bounds]
claimed_from: todo
assignee: worker-proof-sidecar-budgets
lease_expires_at: 1786585710
---
## User-visible outcome

Proof-sidecar construction satisfies the same fail-closed resource promise as decoding: manifest, identity, payload framing, and complete-container byte limits are checked with exact arithmetic before proportional producer allocation, and rejection leaves no partially admitted case.

## Verified defect

At exact base `62df964ef529aadee4649d4eb9c155152b8c92be`, `ProofSidecarBuilder::push_case` passes borrowed payload rows to `place`, which clones every payload before any cumulative container check. `derive_identity` and `encode_manifest` grow vectors before checking their byte limits, and `encode` appends every framed payload before checking `MAX_PROOF_SIDECAR_BYTES`. This contradicts the `proof` module and `docs/artifact-abi.md` claim that every bound is checked before proportional allocation in both directions.

## Required delivery

- Derive exact encoded manifest, identity, framing, and total sidecar sizes with checked arithmetic before cloning, hashing, reserving, or appending proportional data.
- Preserve transactional `push_case`: overflow or unrepresentable arithmetic leaves the builder unchanged and returns the exact typed limit/overflow reason.
- Consume or move the `Vec<u8>` payloads already owned by `ProofCaseSpec` rather than cloning them where the interface permits.
- Use exact or fallible reservation after validation; do not translate allocation failure into a smaller proof, omitted payload, external reference, or fallback case.
- Keep producer and decoder acceptance coherent and pin the full population of governed byte resources from the owning type rather than a hand count.
- Correct the contract only if a claimed pre-allocation check cannot honestly be provided; do not leave the present false universal in place.

## Required negative controls

Independently perturb manifest size, identity size, cumulative payload framing, sidecar total, arithmetic overflow, and the move-vs-clone path. Assertions remain unchanged and each production subject must fail at its own named boundary.

## Closes when

No producer path performs proportional allocation before its governing size is established, exact package tests and doctests pass, and independent review reconciles every `MAX_PROOF_*` byte bound with both producer and decoder consumption.
