---
id: implement-parallel-reduction-strategies
title: Implement parallel reduction strategies
status: todo
priority: p1
dependencies: [implement-first-profile-numerical-policies, implement-analytical-component-cost-model, calibrate-and-activate-parallel-reduction-selection]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reduction, scheduling, numerics]
---
## User-visible outcome

A reduction can be scheduled as a single-workgroup or multi-pass strategy—not only the serial order—with the numerical legality of each order checked against the declared realization. Larger reductions stop being serialized by default only after measured calibration demonstrates a faster valid strategy on the qualified target.

Add single-workgroup and multi-pass reductions beyond the serial schedule. Define empty identities, accumulation dtype, deterministic/relaxed orders, synchronization, partial storage, feasibility and numerical evidence; selection may deliberately choose multiple kernels.

## Split execution graph (2026-07-30)

The former single ticket was not executable as one unit. `implement-the-target-neutral-multi-pass-reduction-strategy` owns explicit cross-dispatch partials without an intra-workgroup barrier. `represent-cooperative-workgroup-reduction-dataflow` and `admit-the-first-typed-synchronization-point-and-atomic-target-authority` precede `implement-the-single-workgroup-synchronized-reduction-strategy`. `realize-parallel-reduction-strategies-on-metal` owns backend lowering, artifact/runtime obligations, and hardware execution. `calibrate-and-activate-parallel-reduction-selection` owns the measured crossover and selection activation.

This parent is a rollup over those dependency-ordered outcomes. It must not implement an arbitrary preference for parallel plans: current structural dominance prefers fewer dispatches, and analytical costs do not yet decide dominance. A claim that a multi-kernel plan “won” is truthful only after calibration connects measured cost evidence to selection.

## Dependency notes (2026-07-28)

**`implement-boundary-property-enforcers` is `deferred`, and its restart condition is a failing test rather than a person.** `frontier.rs::the_bounded_profile_admits_no_undischarged_boundary` (`crates/tiler-compiler/src/frontier.rs:2107`) asserts that the bounded profile's two constant property sets discharge each other; when it fails, that ticket becomes startable and the mismatch that failed it is the enforcer's first real case. The derivation, the per-dimension table, and the list of changes that would fire it are at `tickets/implement-boundary-property-enforcers.md:23-50`; do not restate them. A multi-pass reduction writes partial results one pass consumes, which is a per-region boundary variation the single-region profile has never produced — so this ticket is a candidate for firing that trigger rather than a consumer of the enforcers.

**`implement-first-profile-numerical-policies` is done.** Derive the reduction order from the current merged numerical policy and `NumericalRealization` authorities, not from historical worktree state.

## Closes when

1. A single-workgroup reduction and a multi-pass reduction both exist as schedule alternatives beside the serial one, and the portfolio can retain all three for one program through the split tickets above.
2. The empty domain has a stated identity element per reducer, and an extent-0 reduction produces it in every strategy — the reference case is emittable today (`emit-an-empty-domain-reduction-to-metal`, `done`), so this is testable rather than blocked.
3. The accumulation dtype is an explicit part of the strategy, not inherited silently from the element dtype, and a strategy that would accumulate at a narrower width than the contract allows is rejected with a typed reason.
4. Deterministic and relaxed orders are distinct alternatives whose legality is checked against the declared realization, and **reassociation and contributor permutation stay independent** — a tree reduction needs reassociation, while an atomic or nondeterministic-arrival combine also needs permutation, and a strategy must not be admitted by checking one and using both. The widened `NumericalRealization` now carries permutation explicitly; consume that authority rather than reintroducing a derived or defaulted flag.
5. Synchronization and partial storage are explicit physical contracts of the multi-pass strategy: which pass writes what, where it lives, and what barrier or dispatch boundary makes it visible — never implied by the pass count.
6. Feasibility is separate from cost: a strategy the target cannot honour (threads per workgroup, local memory, barrier availability, a permission the contract withholds) is rejected with an explainable reason rather than costed to infinity, and numerical evidence for each admitted strategy is recorded at its own evidence class.
7. Selection may deliberately return multiple kernels for one reduction only from calibrated target evidence, the explain output says why the multi-kernel plan won, and `make full` passes.

## Graph maintenance

- **You will probably fire the enforcers trigger, on purpose.** A multi-pass reduction writes partials one pass consumes — a per-region boundary variation the bounded profile has never produced. When `the_bounded_profile_admits_no_undischarged_boundary` fails under your change, that is the designed signal (its message names the mismatch): append the mismatch to `implement-boundary-property-enforcers` as its first real case and tell the coordinator it is startable. Do NOT repair the test by widening the property sets.
- **Permutation and reassociation remain separate checks.** Perturb each permission independently and observe the corresponding strategy reject before restoring it.
- **Accumulation-dtype rejection** (criterion 3) is a new typed refusal: give it an explain record and update the census in the same commit.
