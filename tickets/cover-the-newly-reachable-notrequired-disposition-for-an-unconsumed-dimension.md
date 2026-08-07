---
id: cover-the-newly-reachable-notrequired-disposition-for-an-unconsumed-dimension
title: Cover the newly reachable NotRequired disposition for an unconsumed dimension
status: todo
priority: p2
dependencies: []
related: [derive-per-locus-numerical-obligations, wire-the-delivered-realization-record-into-the-artifact]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, fail-closed, test-coverage]
---
## What changed, and why it needs its own node

[`derive-per-locus-numerical-obligations`](derive-per-locus-numerical-obligations.md) narrowed the delivered-realization producer so an obligation row is emitted only for an occurrence whose operation can actually consume the dimension. That was the point of the ticket. **It also made a disposition reachable that previously could not be**, and its worker flagged this rather than landing it silently.

**Before**, the producer emitted one row per honoured dimension at `PolicyLocus::Computation` of *every* covered occurrence. A dimension therefore always had a non-empty row set, so the artifact builder could never derive `NotRequired` for an honoured dimension. The superseded module comment stated this in terms: over-stating "never under-states it, which is the safe direction … a missing one would let a dimension's disposition be derived as `NotRequired` — the one producer assertion the neutral artifact cannot check."

**After**, a dimension that *no* covered occurrence consumes yields an empty row set and the artifact derives `NotRequired`. The producer's new module header argues this is correct — "a dimension no covered occurrence consumes is genuinely not required by any packaged route, which is the claim `NotRequired` makes" — and the coordinator agrees the reasoning is sound and that the change is **forced rather than chosen**: you cannot emit a founded locus for an occurrence that founds none, so the alternative is the unfounded `Computation` row this ticket exists to remove.

**But it is untested, and it is the one assertion the neutral artifact cannot re-check.** The worker reports that no current fixture reaches it — all four honoured dimensions retain rows in every program in the suite. So a semantically load-bearing path is newly reachable and no test exercises it.

## What this owes

- A program in which some honoured dimension is consumed by **no** covered occurrence, carried through to a packaged artifact, with `NotRequired` asserted as the derived disposition. Name the program and why that dimension is unconsumed.
- The safety direction pinned: the correctness of `NotRequired` rests entirely on `operation_capability(..).can_consume` never returning `false` for an operation that *can* consume the dimension. `policy.rs` states that `operation_capabilities` is written conservatively and that `unrepresentable_dimension` independently refuses any consumable dimension the realization cannot carry. **Turn that from a stated intention into a check** — a capability row that under-claims must be caught, because a false `can_consume` now silently produces a `NotRequired` claim rather than a redundant row.
- The perturbation watched failing: narrow a capability row so a genuinely consumed dimension reports unconsumed, and observe the artifact assert `NotRequired` where it must not. Restore.

## Explicit non-goals

Not a revert of the narrowing — it is correct and forced. Not a change to the locus derivation, the strictness rule, or the founded-locus refusal, all of which landed with their own evidence.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the producing ticket, from a consequence its worker named and asked for explicit sign-off on rather than treating as authorized. Kept separate because it is a test and safety-direction obligation on a behaviour change, not part of the locus derivation itself.
