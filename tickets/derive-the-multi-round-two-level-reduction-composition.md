---
id: derive-the-multi-round-two-level-reduction-composition
title: Derive the multi-round two-level reduction composition
status: done
priority: p2
dependencies: []
related: [admit-loop-carried-cooperative-staging, admit-a-round-dependent-cooperative-staging-span, compose-the-two-level-subgroup-and-workgroup-reduction, accept-adr-0100-multi-round-reduction-composition, catalogue-adr-0100-and-the-multi-round-composition-record, correct-the-extrema-familys-identity-ground-and-name-its-padding-identity]
scopes: [research/scheduling, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, scheduling, reductions, numerics, graph-repair]
---
## User-visible outcome

ADR 0096's two-level subgroup-then-workgroup reduction either gains a proved multi-round form or retains a precise refusal whose missing leaf-order or identity obligation is named, rather than treating admitted round vocabulary as proof that the composition is numerically legal.

## Historical premise (carrier-time)

**Fact — at filing.** ADR 0096 recorded that its original multi-round trigger had fired: [`admit-loop-carried-cooperative-staging`](admit-loop-carried-cooperative-staging.md) was `done`, `CooperativeTile::rounds` admitted staging rewritten across rounds, and the accepted one-round derivation no longer rested on a vocabulary limitation.

**Inference — at filing.** Admitting storage lifetime across rounds does not derive the contributor leaf order across those rounds and does not supply the identity obligation for a ragged final round. Those are the two facts a numerically legal reduction composition still owed.

## Work

- Re-run ADR 0096's leaf-order derivation over at least one full second round and one ragged final round, keeping reassociation separate from contributor permutation.
- State whether each round has its own identity contributor and how padding or inactive lanes are excluded or identified.
- Compare the result against the round-dependent staged-span work without conflating storage coverage with numerical grouping.
- If more than one public schedule spelling survives, enumerate the exact boundary for Tom under ADR 0075; do not self-accept it or edit the accepted decision to imply acceptance.

## Closes when

The derivation either proves a multi-round composition with its complete leaf order and identity obligations, or records the first counterexample and the typed refusal the schedule needs; ADR 0096's open question is updated to the durable result, and any consequential public boundary remains explicitly Tom's.

## Outcome

**Derivation complete.** [The multi-round two-level reduction composition](../docs/research/scheduling/multi-round-two-level-reduction-composition.md) carries `research_status: "complete"` and states the leaf order, identity obligations, synchronization points, and padding identity against the tree at derivation base `1d918b67` / work commit `d9bd49ef`. The decision-shaped result landed as [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) (drafted proposed on this ticket's scopes; accepted later by Tom).

**Composition proved legal, not refused.** The composed leaf order is lexicographic `(round, subgroup, lane)` / `index = r·T + g·W + l`, consuming reassociation alone; a participant-major nesting is refused as a strided permutation. One lane identity constant seeds each round; the cross-round accumulator escapes because `emit_loop_carried_cooperative` peels round zero. Two synchronization points are required; a single-point form is `UndischargedAntiDependency`. Extrema padding identity is `0xff80_0000`, separate from `EmptyDomainContract::NoIdentity`.

**ADR 0096 open question updated; decision 8 superseded by acceptance.** ADR 0096's open-questions block records the durable result and, after acceptance, that ADR 0100 supersedes decision 8 only. Acceptance and both catalog rows executed under [`accept-adr-0100-multi-round-reduction-composition`](accept-adr-0100-multi-round-reduction-composition.md) at `e10adb74` (2026-08-05); navigation cataloguing also closed [`catalogue-adr-0100-and-the-multi-round-composition-record`](catalogue-adr-0100-and-the-multi-round-composition-record.md). The extrema doc contradiction named in the derivation inventory was repaired under [`correct-the-extrema-familys-identity-ground-and-name-its-padding-identity`](correct-the-extrema-familys-identity-ground-and-name-its-padding-identity.md).

**Five public-boundary items left for Tom under ADR 0075** (research record § "Public-boundary items"; none self-accepted): (1) the composition's round-carrying fields, (2) the block-index statement's shape for the triple, (3) a round-canonicality rule name, (4) a padded-coverage rule name, (5) a stated padding identity for the identity-less family. Implementation remains `not-started` on ADR 0100 — no two-level topology or subgroup coordinate spelling in crates.

**Deferrals stay on the research/ADR records, not reopened here:** cost measurement vs larger single-round `k`; fused two-allocation composition; peel test at two-level emission; round-dependent staged span (explicitly unfired by this derivation; still owned by [`admit-a-round-dependent-cooperative-staging-span`](admit-a-round-dependent-cooperative-staging-span.md)).

**Fact audit — 2026-08-10.** Terminal record repaired: this ticket closed at `7d49e639` without an Outcome and still used live-tense framing under `status: done`. Substance was already delivered; this section records it. Residual product debt outside this ticket: ADR 0096's durable-result paragraph still asserts the multi-round record is `proposed` / "nothing here is decided"; the research record status prose and `disposition: "pending"` plus the research README "pending" row still lag ADR 0100's acceptance (siblings use `adopted` after acceptance).
