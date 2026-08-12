---
id: admit-fixed-vector-contributor-partitions
title: Admit fixed-vector contributor partitions
status: todo
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary, admit-shared-contributor-coverage-and-reduction-padding-identity, admit-fixed-vector-ssa-and-unmasked-memory-into-kernel-ir]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/cpu, contracts/decisions, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, scheduling, reductions, public-boundary]
---
## User-visible outcome

Fixed vector lanes may partition one reduction's contributor sequence only through an explicit topology whose coverage, layout, numerical permissions, and padding identity are all independently verified.

## Required delivery

- Add a reduction topology dedicated to fixed-vector contributor partitions; do not overload `ExecutionBinding::FixedVectorMap`.
- Carry an exact fixed lane width, shared `ContributorCoverage`, lane layout, axes/order, accumulation type, and the complete numerical permissions the layout consumes.
- Require lane width and partition population to agree. Contiguous contributor blocks consume reassociation; strided assignment additionally consumes permutation. Neither permission implies the other.
- Preserve `ContributorPartition::covers` for existing topologies. Exact and identity-padded coverage remain separate, and padding identity is verifier-proved rather than copied from the empty-domain result.
- Keep iteration-domain tail policy exact unless another accepted map-tail policy independently applies. Contributor padding never becomes a launch tail.
- Implement through real lane-shaped KIR and a native CPU approach; no generic horizontal-reduce intrinsic, backend inference, simulator, or mock is admissible.

## Closes when

Exact and identity-padded neighbours execute through the real CPU path, reassociation/permutation/coverage/identity failures are independently observed, and existing reduction topologies remain unchanged.
