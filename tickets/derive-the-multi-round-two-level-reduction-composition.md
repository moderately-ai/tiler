---
id: derive-the-multi-round-two-level-reduction-composition
title: Derive the multi-round two-level reduction composition
status: todo
priority: p2
dependencies: []
related: [admit-loop-carried-cooperative-staging, admit-a-round-dependent-cooperative-staging-span, compose-the-two-level-subgroup-and-workgroup-reduction]
scopes: [research/scheduling, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, scheduling, reductions, numerics, graph-repair]
---
## User-visible outcome

ADR 0096's two-level subgroup-then-workgroup reduction either gains a proved multi-round form or retains a precise refusal whose missing leaf-order or identity obligation is named, rather than treating admitted round vocabulary as proof that the composition is numerically legal.

## Why this is live

**Fact.** ADR 0096 records that its original trigger has fired: [`admit-loop-carried-cooperative-staging`](admit-loop-carried-cooperative-staging.md) is `done`, `CooperativeTile::rounds` admits staging rewritten across rounds, and the accepted one-round derivation no longer rests on a vocabulary limitation.

**Inference.** Admitting storage lifetime across rounds does not derive the contributor leaf order across those rounds and does not supply the identity obligation for a ragged final round. Those are the two facts a numerically legal reduction composition still owes.

## Work

- Re-run ADR 0096's leaf-order derivation over at least one full second round and one ragged final round, keeping reassociation separate from contributor permutation.
- State whether each round has its own identity contributor and how padding or inactive lanes are excluded or identified.
- Compare the result against the round-dependent staged-span work without conflating storage coverage with numerical grouping.
- If more than one public schedule spelling survives, enumerate the exact boundary for Tom under ADR 0075; do not self-accept it or edit the accepted decision to imply acceptance.

## Closes when

The derivation either proves a multi-round composition with its complete leaf order and identity obligations, or records the first counterexample and the typed refusal the schedule needs; ADR 0096's open question is updated to the durable result, and any consequential public boundary remains explicitly Tom's.
