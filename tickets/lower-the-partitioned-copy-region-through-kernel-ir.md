---
id: lower-the-partitioned-copy-region-through-kernel-ir
title: Lower the partitioned-copy region through kernel IR
status: todo
priority: p1
dependencies: [admit-the-partitioned-copy-scheduled-region]
related: [plan-concatenate-through-one-partitioned-copy-entry]
scopes: [implementation/ir, implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, kernel-ir, concatenate, ownership, verification]
---
## Outcome

The canonical partitioned-copy schedule lowers to one verified KIR with one entry and no arithmetic operations. Each output coordinate is supplied by exactly one ordered member, with bounds and ownership linked to the scheduled proof.

## Required delivery

Prefer existing index, comparison, predicate, load, and store operations where their exact semantics suffice. If the canonical body uses one predicated store per member, add a dedicated total verifier arm proving the predicates mutually exclusive and exhaustive and tying every store to the one joint ownership witness. Do not relax the generic `stores == 1` rule or accept an arbitrary multi-store program.

One buffer binding serves each distinct source plus the output; member records reference bindings, so `concat(x, x)` has one source binding and two members. No arithmetic node, unguarded store, extra store, missing member, reordered member, or unstated source-selection fallback is admitted.

## Closes when

Scheduled and KIR identities bind every ordered member and proof; canonical-body equality remains the final check; missing/extra/unguarded-store and wrong-member perturbations fail separately; and one verified KIR covers all admitted arities within structural bounds.
