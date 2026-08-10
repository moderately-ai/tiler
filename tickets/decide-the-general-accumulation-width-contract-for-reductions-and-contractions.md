---
id: decide-the-general-accumulation-width-contract-for-reductions-and-contractions
title: Decide the general accumulation-width contract for reductions and contractions
status: todo
priority: p2
dependencies: []
related: [implement-parallel-reduction-strategies, admit-the-rms-normalization-family, admit-the-softmax-family, scope-transformer-nonlinear-normalization-and-reductions, design-the-bf16-computation-and-accumulator-contract]
scopes: [research/numerics, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, reduction, contraction, accumulator, widening]
---
## User-visible outcome

A decided general policy for when a reduction or contraction may accumulate at a width other than the region's element arithmetic type — so L3′ decision D-5's *general* contract and L3/L4 decision D-6's longest-accumulation width question have a live owner after the parallel-reduction rollup closed on equality-only verification. This is a research and design ticket ending in an accepted decision, recorded deferral with trigger, or bounded experiment; it is not implementation of a widening strategy.

## Why this exists

[`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md) is `done`. It made accumulation dtype an explicit topology field and rejects a mismatch with typed `ScheduledRegionDiagnostic::AccumulationWidth` / rule `accumulation-width`. The check is equality against the region's arithmetic type: an accumulator *wider* than the element width is refused by the same rule as a narrower one, so **no widening strategy is admitted**.

The rollup's Outcome still named the residual: *the general accumulation contract is still owned here and still undecided*. Workload research keeps the same open policy under two names:

- **L3′ D-5** — general accumulation-dtype contract for the two sum reductions after both registered keys declare F32 explicitly ([transformer non-linear, normalization, and reductions](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md)).
- **L3/L4 D-6** — accumulation dtype for the contraction's embedded sum / longest workload accumulations ([first Metal contraction realizations](../docs/research/scheduling/first-metal-contraction-realizations.md); L4 attention vertical corrects which extent is longest).

A `done` ticket cannot own an open general contract. This ticket takes that remainder without reopening the rollup's seven close criteria.

## What is already decided (do not re-litigate)

- Accumulation dtype is **explicit** on parallel topologies, not inherited silently from the element dtype.
- Declared width that disagrees with the region's arithmetic type is a typed schedule-verification failure, not a costed feasibility decline.
- Workload **family keys** already declare F32 accumulators for the named sums (`tiler::rms-norm-f32@1`, `tiler::softmax-f32@1`); D-5 is *consumed* for those declarations. What remains is policy for the general case, including whether a *wider* accumulator is ever a legal different computation rather than a malformed region.
- BF16/F32 mixed-precision and conversion boundaries are owned under [design-the-bf16-computation-and-accumulator-contract](design-the-bf16-computation-and-accumulator-contract.md) / ADR 0091 landings — do not fork a second mixed-dtype contract here; relate conclusions, do not restate them.

## Questions this must decide

1. **Is a wider accumulator ever admissible as a schedule alternative for an already-declared element type**, or is equality the permanent contract (wider always a different computation that needs a different registered operation / key)?
2. **What evidence closes D-5's general half and D-6** — reference reproduction alone, measured sensitivity, symbolic longest extent (attention `S`), or an explicit product decision that F32-equals-element is the only supported profile for the selected workload?
3. **Where does the answer live** — operation registered facts, numerical realization, topology field semantics, or a profile honourability row — so identity, validation, and explain stay coherent?
4. **What typed outcome does a refused wider (or narrower) plan return** after any policy change, and does it stay `accumulation-width` or need a distinct rule for "widening not admitted"?

## Non-goals

- Reopening or re-proving the seven close criteria of `implement-parallel-reduction-strategies`.
- Implementing Metal or compiler selection for a widening strategy before the policy is accepted.
- Filing a generic "support every dtype accumulator" implementation ticket.
- Editing research docs in this ticket's close without an accepted decision body to point them at.

## Closes when

Each question above is answered with its elimination, or explicitly deferred with closing evidence and a reconsideration trigger; research D-5/D-6 ownership sentences are retargeted off the done rollup to this ticket or to the accepted decision; and any public-boundary or identity consequence is escalated rather than self-accepted.

## Graph maintenance

- Related to the closed rollup; does **not** depend on reopening it.
- When closed, retarget open D-5/D-6 prose in the L3′ and L3/L4 research records so they no longer assign residual ownership solely to `implement-parallel-reduction-strategies`.
- Enforcers restart restatement remains owned by [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md); not this ticket.
